use crate::OidcState;
pub use crate::config::OidcConfig;
pub use crate::user_store::UserStore;
use crate::{GroupAdditionalClaims, OidcServiceConfig};
use crate::{SESSION_KEY_OIDC_STATE, error::OidcError};
use axum::{
    Extension, Form,
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Host;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl,
    TokenResponse, UserInfoClaims,
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
};
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashSet;
use tower_sessions::Session;

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
    Extension(service_config): Extension<OidcServiceConfig>,
    session: Session,
    TypedHeader(host): TypedHeader<Host>,
    Form(GetOidcForm { redirect_uri }): Form<GetOidcForm>,
) -> Result<Response, OidcError> {
    let callback_uri = format!("https://{host}{path}", path = service_config.callback_path);

    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &get_http_client(),
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
pub async fn route_get_oidc_callback<US: UserStore>(
    Extension(oidc_config): Extension<OidcConfig>,
    Extension(user_store): Extension<US>,
    Extension(service_config): Extension<OidcServiceConfig>,
    session: Session,
    Query(AuthCallbackQuery { code, iss, state }): Query<AuthCallbackQuery>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Response, OidcError> {
    let callback_uri = format!("https://{host}{path}", path = service_config.callback_path);

    if let Some(iss) = iss {
        assert_eq!(iss, oidc_config.issuer);
    }
    let oidc_state = session
        .remove::<OidcState>(SESSION_KEY_OIDC_STATE)
        .await?
        .ok_or(OidcError::Other("No local OIDC state"))?;

    assert_eq!(oidc_state.state.secret(), &state);

    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &get_http_client(),
        RedirectUrl::new(callback_uri)?,
    )
    .await?;

    let token_response = oidc_client
        .exchange_code(code)?
        .set_pkce_verifier(oidc_state.pkce_verifier)
        .request_async(&get_http_client())
        .await
        .map_err(|_| OidcError::Other("Error requesting token"))?;
    let id_token_verifier = &oidc_client
        .id_token_verifier()
        .set_other_audience_verifier_fn(|aud| oidc_config.additional_audiences.contains(aud));
    let id_claims = token_response
        .id_token()
        .ok_or(OidcError::Other("OIDC provider did not return an ID token"))?
        .claims(id_token_verifier, &oidc_state.nonce)?;

    let user_info_claims: UserInfoClaims<GroupAdditionalClaims, CoreGenderClaim> = oidc_client
        .user_info(
            token_response.access_token().clone(),
            Some(id_claims.subject().clone()),
        )?
        .request_async(&get_http_client())
        .await
        .map_err(|e| OidcError::UserInfo(e.to_string()))?;

    let user_id = oidc_config
        .claim_userid
        .extract_user_id(&user_info_claims)?;

    let groups = user_info_claims
        .additional_claims()
        .groups
        .as_deref()
        .unwrap_or_default();

    if let Some(require_group) = &oidc_config.require_group
        && !groups.contains(require_group)
    {
        return Err(OidcError::NotInAuthorisedGroup);
    }

    // Assign membership based on the OIDC group.
    let assign_memberships: Vec<_> = groups
        .iter()
        .filter_map(|group| oidc_config.assign_memberships.get(group))
        .flatten()
        .map(String::as_str)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let user_exists = match user_store.user_exists(&user_id).await {
        Ok(exists) => exists,
        Err(err) => return Ok(err.into_response()),
    };
    // User does not exist
    if !user_exists && !oidc_config.allow_sign_up {
        return Err(OidcError::SignupDisabled);
    }
    // Create new user. This is also executed when the user already exists
    // since it also ensures the correct group memberships
    if let Err(err) = user_store.ensure_user(&user_id, &assign_memberships).await {
        return Ok(err.into_response());
    }

    let default_redirect = service_config.default_redirect_path.to_owned();
    let base_url: Url = format!("https://{host}").parse().unwrap();
    let redirect_uri = if let Some(redirect_uri) = oidc_state.redirect_uri
        && let Ok(redirect_url) = base_url.join(&redirect_uri)
        && redirect_url.origin() == base_url.origin()
    {
        redirect_url.path().to_owned()
    } else {
        default_redirect
    };

    // Complete login flow
    session
        .insert(service_config.session_key_user_id, user_id.clone())
        .await?;

    Ok(Redirect::to(&redirect_uri).into_response())
}
