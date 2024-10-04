use actix_session::Session;
use actix_web::{
    error::ErrorUnauthorized,
    web::{Data, Form, Redirect},
    HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;

use crate::DefaultTemplate;

#[derive(Template)]
#[template(path = "components/login.html")]
struct LoginForm;

pub async fn route_get_login() -> impl Responder {
    DefaultTemplate { body: LoginForm }
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
    // TODO: implement auth check
    dbg!(&form.username, &form.password);
    if let Ok(Some(user)) = auth_provider
        .validate_user_token(&form.username, &form.password)
        .await
    {
        session.insert("user", user).unwrap();
        Redirect::to(format!("/frontend/user/{}", &form.username))
            .see_other()
            .respond_to(&req)
            .map_into_boxed_body()
    } else {
        ErrorUnauthorized("Unauthorized").error_response()
    }
}
