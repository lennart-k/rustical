use std::{str::FromStr, sync::Arc};

use askama::Template;
use axum::{
    Extension, Form,
    body::Body,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Host;
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, StatusCode, header};
use rand::{Rng, distributions::Alphanumeric};
use rustical_store::auth::{AuthenticationProvider, User};
use serde::Deserialize;
use uuid::Uuid;

pub fn generate_app_token() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(64)
        .collect()
}

#[derive(Template)]
#[template(path = "apple_configuration/template.xml")]
pub struct AppleConfig {
    token_name: String,
    account_description: String,
    hostname: String,
    caldav_principal_url: String,
    carddav_principal_url: String,
    user: String,
    token: String,
    caldav_profile_uuid: Uuid,
    carddav_profile_uuid: Uuid,
    plist_uuid: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PostAppTokenForm {
    name: String,
    #[serde(default)]
    apple: bool,
}

pub async fn route_post_app_token<AP: AuthenticationProvider>(
    user: User,
    Extension(auth_provider): Extension<Arc<AP>>,
    Path(user_id): Path<String>,
    Host(hostname): Host,
    Form(PostAppTokenForm { apple, name }): Form<PostAppTokenForm>,
) -> Result<Response, rustical_store::Error> {
    assert!(!name.is_empty());
    assert_eq!(user_id, user.id);
    let token = generate_app_token();
    auth_provider
        .add_app_token(&user.id, name.to_owned(), token.clone())
        .await?;
    if apple {
        let profile = AppleConfig {
            token_name: name,
            account_description: format!("{}@{}", &user.id, &hostname),
            hostname: hostname.clone(),
            caldav_principal_url: format!("https://{hostname}/caldav/principal/{user_id}"),
            carddav_principal_url: format!("https://{hostname}/carddav/principal/{user_id}"),
            user: user.id.to_owned(),
            token,
            caldav_profile_uuid: Uuid::new_v4(),
            carddav_profile_uuid: Uuid::new_v4(),
            plist_uuid: Uuid::new_v4(),
        }
        .render()
        .unwrap();
        let mut res = Response::builder().status(StatusCode::OK);
        let hdrs = res.headers_mut().unwrap();
        hdrs.typed_insert(
            ContentType::from_str("application/x-apple-aspen-config; charset=utf-8").unwrap(),
        );
        let filename = format!("rustical-{}.mobileconfig", user_id);
        hdrs.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachement; filename*=UTF-8''{} filename={}",
                filename, filename
            ))
            .unwrap(),
        );
        Ok(res.body(Body::new(profile)).unwrap())
    } else {
        Ok((StatusCode::OK, token).into_response())
    }
}

pub async fn route_delete_app_token<AP: AuthenticationProvider>(
    user: User,
    Extension(auth_provider): Extension<Arc<AP>>,
    Path((user_id, token_id)): Path<(String, String)>,
) -> Result<Redirect, rustical_store::Error> {
    assert_eq!(user_id, user.id);
    auth_provider.remove_app_token(&user.id, &token_id).await?;
    Ok(Redirect::to("/frontend/user"))
}
