use crate::{FrontendConfig, oidc::OidcProviderData};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    error::ErrorUnauthorized,
    web::{Data, Form, Redirect},
};
use askama::Template;
use askama_web::WebTemplate;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;

#[derive(Template, WebTemplate)]
#[template(path = "pages/login.html")]
struct LoginPage<'a> {
    oidc_data: Option<OidcProviderData<'a>>,
}

pub async fn route_get_login(req: HttpRequest, config: Data<FrontendConfig>) -> impl Responder {
    LoginPage {
        oidc_data: config.oidc.as_ref().map(|oidc| OidcProviderData {
            name: &oidc.name,
            redirect_url: req
                .url_for_static("frontend_login_oidc")
                .unwrap()
                .to_string(),
        }),
    }
    .respond_to(&req)
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    username: String,
    password: String,
}

pub async fn route_post_login<AP: AuthenticationProvider>(
    req: HttpRequest,
    form: Form<PostLoginForm>,
    session: Session,
    auth_provider: Data<AP>,
) -> HttpResponse {
    if let Ok(Some(user)) = auth_provider
        .validate_user_token(&form.username, &form.password)
        .await
    {
        session.insert("user", user.id).unwrap();
        Redirect::to(format!("/frontend/user/{}", &form.username))
            .see_other()
            .respond_to(&req)
            .map_into_boxed_body()
    } else {
        ErrorUnauthorized("Unauthorized").error_response()
    }
}
