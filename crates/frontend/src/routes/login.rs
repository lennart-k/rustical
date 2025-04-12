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
struct LoginPage;

pub async fn route_get_login() -> impl Responder {
    LoginPage
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
