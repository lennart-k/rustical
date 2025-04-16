use crate::{
    FrontendConfig,
    config::{OidcConfig, UserIdClaim},
};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::StatusCode,
    web::{Data, Form, Query, Redirect},
};
use error::OidcError;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, EmptyAdditionalClaims, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, TokenResponse, UserInfoClaims,
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
};
use rustical_store::auth::{AuthenticationProvider, User, user::PrincipalType::Individual};
use serde::{Deserialize, Serialize};

mod error;

pub(crate) struct OidcProviderData<'a> {
    pub name: &'a str,
    pub redirect_url: String,
}

const SESSION_KEY_OIDC_STATE: &str = "oidc_state";

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
    config: Data<FrontendConfig>,
    session: Session,
) -> Result<impl Responder, OidcError> {
    let oidc_config = config
        .oidc
        .clone()
        .ok_or(OidcError::IncorrectConfiguration)?;

    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &http_client,
        RedirectUrl::new(req.url_for_static("frontend_oidc_callback")?.to_string())?,
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

    Ok(Redirect::to(auth_url.to_string()).see_other())
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthCallbackQuery {
    code: AuthorizationCode,
    iss: IssuerUrl,
    // scope: String,
    // state: String,
}

pub async fn route_get_oidc_callback<AP: AuthenticationProvider>(
    req: HttpRequest,
    config: Data<FrontendConfig>,
    session: Session,
    auth_provider: Data<AP>,
    Query(AuthCallbackQuery { code, iss }): Query<AuthCallbackQuery>,
) -> Result<impl Responder, OidcError> {
    let oidc_config = config
        .oidc
        .clone()
        .ok_or(OidcError::IncorrectConfiguration)?;
    assert_eq!(iss, oidc_config.issuer);
    let oidc_state = session
        .remove_as::<OidcState>(SESSION_KEY_OIDC_STATE)
        .ok_or(OidcError::Other("No local OIDC state"))?
        .map_err(|_| OidcError::Other("Error parsing OIDC state"))?;

    let http_client = get_http_client();
    let oidc_client = get_oidc_client(
        oidc_config.clone(),
        &http_client,
        RedirectUrl::new(req.url_for_static("frontend_oidc_callback")?.to_string())?,
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

    if let Some(require_group) = oidc_config.require_group {
        if !user_info_claims
            .additional_claims()
            .groups
            .contains(&require_group)
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

    let mut user = auth_provider.get_principal(&user_id).await?;
    if user.is_none() {
        let new_user = User {
            id: user_id,
            displayname: None,
            app_tokens: vec![],
            password: None,
            principal_type: Individual,
            memberships: vec![],
        };

        auth_provider.insert_principal(new_user.clone()).await?;
        user = Some(new_user);
    }

    let default_redirect = req.url_for_static("frontend_user")?.to_string();
    let redirect_uri = oidc_state.redirect_uri.unwrap_or(default_redirect.clone());
    let redirect_uri = req
        .full_url()
        .join(&redirect_uri)
        .ok()
        .and_then(|uri| req.full_url().make_relative(&uri))
        .unwrap_or(default_redirect);

    // Complete login flow
    if let Some(user) = user {
        session.insert("user", user.id.clone())?;

        Ok(Redirect::to(redirect_uri)
            .temporary()
            .respond_to(&req)
            .map_into_boxed_body())
    } else {
        // Add user provisioning
        Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).body("User does not exist"))
    }
}
