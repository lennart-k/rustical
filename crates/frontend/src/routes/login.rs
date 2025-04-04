use actix_session::Session;
use actix_web::{
    error::ErrorUnauthorized,
    web::{Data, Form, Html, Redirect},
    HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/login.html")]
struct LoginPage;

pub async fn route_get_login(req: HttpRequest) -> impl Responder {
    Html::new(LoginPage.render().unwrap())
        .respond_to(&req)
        .map_into_boxed_body()
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
        session.insert("user", user).unwrap();
        Redirect::to(format!("/frontend/user/{}", &form.username))
            .see_other()
            .respond_to(&req)
            .map_into_boxed_body()
    } else {
        ErrorUnauthorized("Unauthorized").error_response()
    }
}
