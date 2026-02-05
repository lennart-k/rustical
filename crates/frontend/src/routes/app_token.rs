use std::{str::FromStr, sync::Arc};

use askama::Template;
use axum::{
    Extension, Form,
    body::Body,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::{ContentType, HeaderMapExt, Host};
use http::{HeaderValue, StatusCode, header};
use percent_encoding::{CONTROLS, utf8_percent_encode};
use rand::{Rng, distr::Alphanumeric};
use rustical_store::auth::{AuthenticationProvider, Principal};
use serde::Deserialize;

pub fn generate_app_token() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(64)
        .collect()
}

#[derive(Template)]
#[template(path = "apple_configuration/template.xml")]
pub struct AppleConfig {
    token_name: String,
    hostname: String,
    user: String,
    memberships: Vec<String>,
    token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostAppTokenForm {
    name: String,
    #[serde(default)]
    apple: bool,
}

pub async fn route_post_app_token<AP: AuthenticationProvider>(
    user: Principal,
    Extension(auth_provider): Extension<Arc<AP>>,
    Path(user_id): Path<String>,
    TypedHeader(host): TypedHeader<Host>,
    Form(PostAppTokenForm { apple, name }): Form<PostAppTokenForm>,
) -> Result<Response, rustical_store::Error> {
    assert!(!name.is_empty());
    assert_eq!(user_id, user.id);
    let token = generate_app_token();
    let mut token_id = auth_provider
        .add_app_token(&user.id, name.clone(), token.clone())
        .await?;
    // Get first 4 characters of token identifier
    token_id.truncate(4);
    // This will be a hint for the token validator which app token hash to verify against
    let token = format!("{token_id}_{token}");
    if apple {
        let profile = AppleConfig {
            token_name: name,
            hostname: host.to_string(),
            user: user.id.clone(),
            memberships: user
                .memberships_without_self()
                .into_iter()
                .map(String::from)
                .collect(),
            token,
        }
        .render()
        .unwrap();
        let mut res = Response::builder().status(StatusCode::OK);
        let hdrs = res.headers_mut().unwrap();
        hdrs.typed_insert(
            ContentType::from_str("application/x-apple-aspen-config; charset=utf-8").unwrap(),
        );
        let filename = format!("rustical-{user_id}.mobileconfig");
        let filename = utf8_percent_encode(&filename, CONTROLS);
        hdrs.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachement; filename*=UTF-8''{filename} filename={filename}",
            ))
            .unwrap(),
        );
        Ok(res.body(Body::new(profile)).unwrap())
    } else {
        Ok((StatusCode::OK, token).into_response())
    }
}

pub async fn route_delete_app_token<AP: AuthenticationProvider>(
    user: Principal,
    Extension(auth_provider): Extension<Arc<AP>>,
    Path((user_id, token_id)): Path<(String, String)>,
) -> Result<Redirect, rustical_store::Error> {
    assert_eq!(user_id, user.id);
    auth_provider.remove_app_token(&user.id, &token_id).await?;
    Ok(Redirect::to("/frontend/user"))
}
