#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
use axum::{
    Extension, Form,
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
pub use config::OidcConfig;
use config::UserIdClaim;
use error::OidcError;
use headers::Host;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse, UserInfoClaims,
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
pub use user_store::UserStore;

mod config;
mod error;
mod user_store;

const SESSION_KEY_OIDC_STATE: &str = "oidc_state";

#[derive(Debug, Clone)]
pub struct OidcServiceConfig {
    pub default_redirect_path: &'static str,
    pub session_key_user_id: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
struct OidcState {
    state: CsrfToken,
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
    redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GroupAdditionalClaims {
    #[serde(default)]
    groups: Option<Vec<String>>,
}

impl openidconnect::AdditionalClaims for GroupAdditionalClaims {}

fn get_http_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Something went wrong :(")
}

async fn get_oidc_client(
    OidcConfig {
        issuer,
        client_id,
        client_secret,
        ..
    }: OidcConfig,
    http_client: &reqwest::Client,
    redirect_uri: RedirectUrl,
) -> Result<
    CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointMaybeSet,
        EndpointMaybeSet,
    >,
    OidcError,
> {
    let provider_metadata = CoreProviderMetadata::discover_async(issuer, http_client)
        .await
        .map_err(|err| {
            tracing::error!("An error occured trying to discover OpenID provider: {err}");
            OidcError::Other("Failed to discover OpenID provider")
        })?;

    Ok(CoreClient::from_provider_metadata(
        provider_metadata,
        client_id.clone(),
        client_secret.clone(),
    )
    .set_redirect_uri(redirect_uri))
}

#[derive(Debug, Deserialize)]
pub struct GetOidcForm {
    redirect_uri: Option<String>,
}

/// Endpoint that redirects to the authorize endpoint of the OIDC service
pub async fn route_post_oidc(
    Extension(oidc_config): Extension<OidcConfig>,
    session: Session,
    TypedHeader(host): TypedHeader<Host>,
    Form(GetOidcForm { redirect_uri }): Form<GetOidcForm>,
) -> Result<Response, OidcError> {
    let callback_uri = format!("https://{host}/frontend/login/oidc/callback");

    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &http_client,
        RedirectUrl::new(callback_uri)?,
    )
    .await?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token, nonce) = oidc_client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scopes(oidc_config.scopes.clone())
        .set_pkce_challenge(pkce_challenge)
        .url();

    session
        .insert(
            SESSION_KEY_OIDC_STATE,
            OidcState {
                state: csrf_token,
                nonce,
                pkce_verifier,
                redirect_uri,
            },
        )
        .await?;

    Ok(Redirect::to(auth_url.as_str()).into_response())
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthCallbackQuery {
    code: AuthorizationCode,
    // RFC 9207
    iss: Option<IssuerUrl>,
    state: String,
}

// Handle callback from IdP page
pub async fn route_get_oidc_callback<US: UserStore + Clone>(
    Extension(oidc_config): Extension<OidcConfig>,
    Extension(user_store): Extension<US>,
    Extension(service_config): Extension<OidcServiceConfig>,
    session: Session,
    Query(AuthCallbackQuery { code, iss, state }): Query<AuthCallbackQuery>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Response, OidcError> {
    let callback_uri = format!("https://{host}/frontend/login/oidc/callback");

    if let Some(iss) = iss {
        assert_eq!(iss, oidc_config.issuer);
    }
    let oidc_state = session
        .remove::<OidcState>(SESSION_KEY_OIDC_STATE)
        .await?
        .ok_or(OidcError::Other("No local OIDC state"))?;

    assert_eq!(oidc_state.state.secret(), &state);

    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &http_client,
        RedirectUrl::new(callback_uri)?,
    )
    .await?;

    let token_response = oidc_client
        .exchange_code(code)?
        .set_pkce_verifier(oidc_state.pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|_| OidcError::Other("Error requesting token"))?;
    let id_token_verifier = &oidc_client
        .id_token_verifier()
        .set_other_audience_verifier_fn(|f| oidc_config.additional_audience.iter().any(|g| f == g));
    let id_claims = token_response
        .id_token()
        .ok_or(OidcError::Other("OIDC provider did not return an ID token"))?
        .claims(id_token_verifier, &oidc_state.nonce)?;

    let user_info_claims: UserInfoClaims<GroupAdditionalClaims, CoreGenderClaim> = oidc_client
        .user_info(
            token_response.access_token().clone(),
            Some(id_claims.subject().clone()),
        )?
        .request_async(&http_client)
        .await
        .map_err(|e| OidcError::UserInfo(e.to_string()))?;

    let groups = user_info_claims
        .additional_claims()
        .groups
        .as_deref()
        .unwrap_or_default();

    if let Some(require_group) = &oidc_config.require_group
        && !groups.contains(require_group)
    {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "User is not in an authorized group to use RustiCal",
        )
            .into_response());
    }

    let user_id = match oidc_config.claim_userid {
        UserIdClaim::Sub => user_info_claims.subject().to_string(),
        UserIdClaim::PreferredUsername => user_info_claims
            .preferred_username()
            .ok_or(OidcError::Other("Missing preferred_username claim"))?
            .to_string(),
        UserIdClaim::Email => user_info_claims
            .email()
            .ok_or(OidcError::Other("Missing email claim"))?
            .to_string(),
    };

    match user_store.user_exists(&user_id).await {
        Ok(false) => {
            // User does not exist
            if !oidc_config.allow_sign_up {
                return Ok((StatusCode::UNAUTHORIZED, "User signup is disabled").into_response());
            }
            // Create new user
            if let Err(err) = user_store.insert_user(&user_id).await {
                return Ok(err.into_response());
            }
        }
        Ok(true) => {}
        Err(err) => {
            return Ok(err.into_response());
        }
    }

    let default_redirect = service_config.default_redirect_path.to_owned();
    let base_url: Url = format!("https://{host}").parse().unwrap();
    let redirect_uri = if let Some(redirect_uri) = oidc_state.redirect_uri {
        if let Ok(redirect_url) = base_url.join(&redirect_uri) {
            if redirect_url.origin() == base_url.origin() {
                redirect_url.path().to_owned()
            } else {
                default_redirect
            }
        } else {
            default_redirect
        }
    } else {
        default_redirect
    };

    // Complete login flow
    session
        .insert(service_config.session_key_user_id, user_id.clone())
        .await?;

    Ok(Redirect::to(&redirect_uri).into_response())
}
