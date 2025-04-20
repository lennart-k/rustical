use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder, ResponseError,
    http::StatusCode,
    web::{self, Data, Form, Query, Redirect, ServiceConfig},
};
pub use config::OidcConfig;
use config::UserIdClaim;
use error::OidcError;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse, UserInfoClaims,
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub use user_store::UserStore;

mod config;
mod error;
mod user_store;

pub const ROUTE_NAME_OIDC_LOGIN: &str = "oidc_login";
const ROUTE_NAME_OIDC_CALLBACK: &str = "oidc_callback";
const SESSION_KEY_OIDC_STATE: &str = "oidc_state";

#[derive(Debug, Clone)]
pub struct OidcServiceConfig {
    pub default_redirect_route_name: &'static str,
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
    pub groups: Vec<String>,
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
        .map_err(|_| OidcError::Other("Failed to discover OpenID provider"))?;

    Ok(CoreClient::from_provider_metadata(
        provider_metadata.clone(),
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
    req: HttpRequest,
    Form(GetOidcForm { redirect_uri }): Form<GetOidcForm>,
    oidc_config: Data<OidcConfig>,
    session: Session,
) -> Result<HttpResponse, OidcError> {
    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.as_ref().clone(),
        &http_client,
        RedirectUrl::new(req.url_for_static(ROUTE_NAME_OIDC_CALLBACK)?.to_string())?,
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

    session.insert(
        SESSION_KEY_OIDC_STATE,
        OidcState {
            state: csrf_token,
            nonce,
            pkce_verifier,
            redirect_uri,
        },
    )?;

    Ok(Redirect::to(auth_url.to_string())
        .see_other()
        .respond_to(&req)
        .map_into_boxed_body())
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthCallbackQuery {
    code: AuthorizationCode,
    iss: IssuerUrl,
    state: String,
}

/// Handle callback from IdP page
pub async fn route_get_oidc_callback<US: UserStore>(
    req: HttpRequest,
    oidc_config: Data<OidcConfig>,
    session: Session,
    user_store: Data<US>,
    Query(AuthCallbackQuery { code, iss, state }): Query<AuthCallbackQuery>,
    service_config: Data<OidcServiceConfig>,
) -> Result<HttpResponse, OidcError> {
    assert_eq!(iss, oidc_config.issuer);
    let oidc_state = session
        .remove_as::<OidcState>(SESSION_KEY_OIDC_STATE)
        .ok_or(OidcError::Other("No local OIDC state"))?
        .map_err(|_| OidcError::Other("Error parsing OIDC state"))?;

    assert_eq!(oidc_state.state.secret(), &state);

    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.get_ref().clone(),
        &http_client,
        RedirectUrl::new(req.url_for_static(ROUTE_NAME_OIDC_CALLBACK)?.to_string())?,
    )
    .await?;

    let token_response = oidc_client
        .exchange_code(code)?
        .set_pkce_verifier(oidc_state.pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|_| OidcError::Other("Error requesting token"))?;
    let id_claims = token_response
        .id_token()
        .ok_or(OidcError::Other("OIDC provider did not return an ID token"))?
        .claims(&oidc_client.id_token_verifier(), &oidc_state.nonce)?;

    let user_info_claims: UserInfoClaims<GroupAdditionalClaims, CoreGenderClaim> = oidc_client
        .user_info(
            token_response.access_token().clone(),
            Some(id_claims.subject().clone()),
        )?
        .request_async(&http_client)
        .await
        .map_err(|_| OidcError::Other("Error fetching user info"))?;

    if let Some(require_group) = &oidc_config.require_group {
        if !user_info_claims
            .additional_claims()
            .groups
            .contains(require_group)
        {
            return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED)
                .body("User is not in an authorized group to use RustiCal"));
        }
    }

    let user_id = match oidc_config.claim_userid {
        UserIdClaim::Sub => user_info_claims.subject().to_string(),
        UserIdClaim::PreferredUsername => user_info_claims
            .preferred_username()
            .ok_or(OidcError::Other("Missing preferred_username claim"))?
            .to_string(),
    };

    match user_store.user_exists(&user_id).await {
        Ok(false) => {
            // User does not exist
            if !oidc_config.allow_sign_up {
                return Ok(HttpResponse::Unauthorized().body("User sign up disabled"));
            }
            // Create new user
            if let Err(err) = user_store.insert_user(&user_id).await {
                return Ok(err.error_response());
            }
        }
        Ok(true) => {}
        Err(err) => {
            return Ok(err.error_response());
        }
    }

    let default_redirect = req
        .url_for_static(service_config.default_redirect_route_name)?
        .to_string();
    let redirect_uri = oidc_state.redirect_uri.unwrap_or(default_redirect.clone());
    let redirect_uri = req
        .full_url()
        .join(&redirect_uri)
        .ok()
        .and_then(|uri| req.full_url().make_relative(&uri))
        .unwrap_or(default_redirect);

    // Complete login flow
    session.insert(service_config.session_key_user_id, user_id.clone())?;

    Ok(Redirect::to(redirect_uri)
        .temporary()
        .respond_to(&req)
        .map_into_boxed_body())
}

pub fn configure_oidc<US: UserStore>(
    cfg: &mut ServiceConfig,
    oidc_config: OidcConfig,
    service_config: OidcServiceConfig,
    user_store: Arc<US>,
) {
    cfg.app_data(Data::new(oidc_config))
        .app_data(Data::new(service_config))
        .app_data(Data::from(user_store))
        .service(
            web::resource("")
                .name(ROUTE_NAME_OIDC_LOGIN)
                .post(route_post_oidc),
        )
        .service(
            web::resource("/callback")
                .name(ROUTE_NAME_OIDC_CALLBACK)
                .get(route_get_oidc_callback::<US>),
        );
}
