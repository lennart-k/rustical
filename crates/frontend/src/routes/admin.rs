use std::sync::Arc;

use crate::{
    FrontendConfig,
    pages::admin::{AdminPage, AdminSection, PrincipalView},
    unauthorized_handler,
};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form, Json, Router,
    extract::{Path, Query, Request},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use http::StatusCode;
use rustical_dav::rfc_3986_percent_encode;
use rustical_oidc::OidcConfig;
use rustical_store::{
    Error,
    auth::{AuthenticationProvider, Principal, PrincipalType, hash_password},
};
use serde::{Deserialize, Serialize};

const PER_PAGE: usize = 50;
const SUGGEST_LIMIT: usize = 10;

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePrincipalForm {
    pub id: String,
    pub displayname: Option<String>,
    #[serde(default)]
    pub principal_type: Option<String>,
    pub password: Option<String>,
    pub oauth_only: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGroupForm {
    pub id: String,
    pub displayname: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePrincipalForm {
    pub displayname: Option<String>,
    #[serde(default)]
    pub principal_type: Option<String>,
    pub password: Option<String>,
    pub remove_password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MembershipForm {
    pub group: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupMemberForm {
    pub principal: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PrincipalSearchQuery {
    q: Option<String>,
    page: Option<usize>,
    kind: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Clone)]
struct Paginated<T> {
    items: Vec<T>,
    total: usize,
    page: usize,
    per_page: usize,
    total_pages: usize,
    previous_page: Option<usize>,
    next_page: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct PrincipalSearchItem {
    id: String,
    display_name: Option<String>,
    principal_type: String,
    memberships: Vec<String>,
    is_admin: bool,
    has_password: bool,
    detail_url: String,
}

#[derive(Debug, Clone, Serialize)]
struct PrincipalSearchResponse {
    items: Vec<PrincipalSearchItem>,
    query: Option<String>,
    kind: String,
    page: usize,
    per_page: usize,
    total: usize,
    total_pages: usize,
    previous_page: Option<usize>,
    next_page: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct PrincipalSuggestResponse {
    items: Vec<PrincipalSearchItem>,
    query: Option<String>,
    kind: String,
    limit: usize,
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/admin_principals_section.html")]
struct SettingsPrincipalsSection {
    admin_group: Option<String>,
    allow_password_login: bool,
    oidc_configured: bool,
}

impl AdminSection for SettingsPrincipalsSection {
    fn name() -> &'static str {
        "principals"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/admin_principal_section.html")]
struct SettingsPrincipalSection {
    principal_id: String,
}

impl AdminSection for SettingsPrincipalSection {
    fn name() -> &'static str {
        "principal"
    }
}

#[derive(Debug, Clone, Serialize)]
struct PrincipalCardData {
    principal: PrincipalView,
    members: Vec<PrincipalView>,
    is_current_user: bool,
    allow_password_login: bool,
    oidc_configured: bool,
    admin_group: Option<String>,
}

pub fn settings_router<AP: AuthenticationProvider>() -> Router {
    Router::new()
        .route("/", get(route_settings))
        .route(
            "/principals/search",
            get(route_settings_principals_search::<AP>),
        )
        .route(
            "/principals/suggest",
            get(route_settings_principals_suggest::<AP>),
        )
        .route("/principals", get(route_settings_principals))
        .route("/principals/{id}", get(route_settings_principal::<AP>))
        .route(
            "/principals/{id}/data",
            get(route_settings_principal_data::<AP>),
        )
        .route("/groups", get(route_settings_groups))
        .route("/groups/{id}", get(route_settings_group::<AP>))
        .route("/users/create", post(route_create_principal::<AP>))
        .route("/groups/create", post(route_create_group::<AP>))
        .route(
            "/principals/{id}/update",
            post(route_update_principal::<AP>),
        )
        .route(
            "/principals/{id}/delete",
            post(route_delete_principal::<AP>),
        )
        .route(
            "/principals/{id}/memberships/add",
            post(route_add_membership::<AP>),
        )
        .route(
            "/principals/{id}/memberships/remove",
            post(route_remove_membership::<AP>),
        )
        .route(
            "/groups/{id}/members/add",
            post(route_add_group_member::<AP>),
        )
        .route(
            "/groups/{id}/members/remove",
            post(route_remove_group_member::<AP>),
        )
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn(unauthorized_handler))
}

async fn require_admin(request: Request, next: Next) -> Response {
    let Some(user) = request.extensions().get::<Principal>().cloned() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(config) = request.extensions().get::<FrontendConfig>().cloned() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if !config.admin.is_admin(&user) {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await
}

async fn route_settings() -> Redirect {
    Redirect::to("/frontend/settings/principals")
}

async fn route_settings_groups() -> Redirect {
    Redirect::to("/frontend/settings/principals")
}

async fn route_settings_principals(
    user: Principal,
    Extension(config): Extension<FrontendConfig>,
    Extension(oidc_config): Extension<Option<OidcConfig>>,
) -> Response {
    let section = SettingsPrincipalsSection {
        admin_group: config.admin.admin_group.clone(),
        allow_password_login: config.allow_password_login,
        oidc_configured: oidc_config.is_some(),
    };
    let section_name = SettingsPrincipalsSection::name();

    AdminPage {
        user,
        section,
        section_name,
    }
    .into_response()
}

async fn route_settings_principals_search<AP: AuthenticationProvider>(
    Query(query): Query<PrincipalSearchQuery>,
    Extension(config): Extension<FrontendConfig>,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Json<PrincipalSearchResponse>, Error> {
    let term = normalize(query.q);
    let pagination = search_principal_views(
        auth_provider.get_principals().await?,
        config.admin.admin_group.as_deref(),
        term.as_deref(),
        query.kind.as_deref().unwrap_or("all"),
        query.page,
    );
    let items = pagination
        .items
        .into_iter()
        .map(PrincipalSearchItem::from_principal_view)
        .collect();

    Ok(Json(PrincipalSearchResponse {
        items,
        query: term,
        kind: normalize(query.kind).unwrap_or_else(|| "all".to_owned()),
        page: pagination.page,
        per_page: pagination.per_page,
        total: pagination.total,
        total_pages: pagination.total_pages,
        previous_page: pagination.previous_page,
        next_page: pagination.next_page,
    }))
}

async fn route_settings_principals_suggest<AP: AuthenticationProvider>(
    Query(query): Query<PrincipalSearchQuery>,
    Extension(config): Extension<FrontendConfig>,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Json<PrincipalSuggestResponse>, Error> {
    let term = normalize(query.q);
    let kind = normalize(query.kind).unwrap_or_else(|| "all".to_owned());
    let limit = query.limit.unwrap_or(SUGGEST_LIMIT).clamp(1, PER_PAGE);
    let principals = auth_provider.get_principals().await?;
    let mut views = principal_views_for_search(
        principals,
        config.admin.admin_group.as_deref(),
        term.as_deref(),
        &kind,
    );
    views.truncate(limit);
    let items = views
        .into_iter()
        .map(PrincipalSearchItem::from_principal_view)
        .collect();

    Ok(Json(PrincipalSuggestResponse {
        items,
        query: term,
        kind,
        limit,
    }))
}

async fn route_settings_principal<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    user: Principal,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Response, Error> {
    auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;

    let section = SettingsPrincipalSection { principal_id: id };
    let section_name = SettingsPrincipalSection::name();

    Ok(AdminPage {
        user,
        section,
        section_name,
    }
    .into_response())
}

async fn route_settings_principal_data<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    user: Principal,
    Extension(config): Extension<FrontendConfig>,
    Extension(oidc_config): Extension<Option<OidcConfig>>,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Json<PrincipalCardData>, Error> {
    let principal = auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;
    let member_ids = auth_provider.list_members(&id).await?;
    let members = list_member_views_for_ids(&*auth_provider, &config, &member_ids).await?;

    Ok(Json(PrincipalCardData {
        principal: PrincipalView::from_principal(principal, config.admin.admin_group.as_deref()),
        members,
        is_current_user: user.id == id,
        allow_password_login: config.allow_password_login,
        oidc_configured: oidc_config.is_some(),
        admin_group: config.admin.admin_group.clone(),
    }))
}

async fn route_settings_group<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Response, Error> {
    let principal = auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(Redirect::to(&principal_path(&principal.id)).into_response())
}

async fn route_create_principal<AP: AuthenticationProvider>(
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<CreatePrincipalForm>,
) -> Result<Response, Error> {
    let id = form.id.trim().to_owned();
    if id.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Principal id is required").into_response());
    }

    let principal_type = parse_principal_type(form.principal_type.as_deref())?;
    let oauth_only = form
        .oauth_only
        .as_deref()
        .is_some_and(|value| !value.is_empty() && value != "off");
    let password = if oauth_only {
        None
    } else {
        normalize(form.password)
            .map(|password| hash_password(&password))
            .transpose()?
    };

    auth_provider
        .insert_principal(
            Principal {
                id: id.clone(),
                displayname: normalize(form.displayname),
                principal_type,
                password,
                memberships: Vec::new(),
            },
            false,
        )
        .await?;

    Ok(Redirect::to(&principal_path(&id)).into_response())
}

async fn route_create_group<AP: AuthenticationProvider>(
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<CreateGroupForm>,
) -> Result<Response, Error> {
    let id = form.id.trim().to_owned();
    if id.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Group id is required").into_response());
    }

    auth_provider
        .insert_principal(
            Principal {
                id: id.clone(),
                displayname: normalize(form.displayname),
                principal_type: PrincipalType::Group,
                password: None,
                memberships: Vec::new(),
            },
            false,
        )
        .await?;

    Ok(Redirect::to(&principal_path(&id)).into_response())
}

async fn route_update_principal<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<UpdatePrincipalForm>,
) -> Result<Response, Error> {
    let mut principal = auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;

    if let Some(displayname) = form.displayname {
        let displayname = displayname.trim();
        principal.displayname = (!displayname.is_empty()).then(|| displayname.to_owned());
    }
    if let Some(principal_type) = form.principal_type.as_deref() {
        principal.principal_type = parse_principal_type(Some(principal_type))?;
    }
    if form
        .remove_password
        .as_deref()
        .is_some_and(|value| !value.is_empty() && value != "off")
    {
        principal.password = None;
    }
    if let Some(password) = normalize(form.password) {
        principal.password = Some(hash_password(&password)?);
    }

    auth_provider.insert_principal(principal, true).await?;
    Ok(Redirect::to(&principal_path(&id)).into_response())
}

async fn route_delete_principal<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    user: Principal,
    Extension(auth_provider): Extension<Arc<AP>>,
) -> Result<Response, Error> {
    if id == user.id {
        return Ok((
            StatusCode::FORBIDDEN,
            "You cannot delete the signed-in principal",
        )
            .into_response());
    }

    auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;

    auth_provider.remove_principal(&id).await?;
    Ok(Redirect::to("/frontend/settings/principals").into_response())
}

async fn route_add_membership<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<MembershipForm>,
) -> Result<Response, Error> {
    let group = form.group.trim();
    if group.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Group is required").into_response());
    }
    if group == id {
        return Ok((
            StatusCode::BAD_REQUEST,
            "A principal cannot be a member of itself",
        )
            .into_response());
    }

    auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;
    auth_provider
        .get_principal(group)
        .await?
        .ok_or(Error::NotFound)?;

    auth_provider.add_membership(&id, group).await?;
    Ok(Redirect::to(&membership_redirect(&id, group, form.source.as_deref())).into_response())
}

async fn route_remove_membership<AP: AuthenticationProvider>(
    Path(id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<MembershipForm>,
) -> Result<Response, Error> {
    let group = form.group.trim();
    if group.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Group is required").into_response());
    }

    let principal = auth_provider
        .get_principal(&id)
        .await?
        .ok_or(Error::NotFound)?;
    if !principal.is_member_of(group) {
        return Ok(
            Redirect::to(&membership_redirect(&id, group, form.source.as_deref())).into_response(),
        );
    }

    auth_provider.remove_membership(&id, group).await?;
    Ok(Redirect::to(&membership_redirect(&id, group, form.source.as_deref())).into_response())
}

async fn route_add_group_member<AP: AuthenticationProvider>(
    Path(group): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<GroupMemberForm>,
) -> Result<Response, Error> {
    let principal_id = form.principal.trim();
    if principal_id.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Principal id is required".to_owned(),
        )
            .into_response());
    }
    if principal_id == group {
        return Ok((
            StatusCode::BAD_REQUEST,
            "A principal cannot be a member of itself".to_owned(),
        )
            .into_response());
    }

    auth_provider
        .get_principal(&group)
        .await?
        .ok_or(Error::NotFound)?;
    auth_provider
        .get_principal(principal_id)
        .await?
        .ok_or(Error::NotFound)?;

    auth_provider.add_membership(principal_id, &group).await?;
    Ok(Redirect::to(&principal_path(&group)).into_response())
}

async fn route_remove_group_member<AP: AuthenticationProvider>(
    Path(group): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<GroupMemberForm>,
) -> Result<Response, Error> {
    let principal_id = form.principal.trim();
    if principal_id.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Principal id is required".to_owned(),
        )
            .into_response());
    }

    auth_provider
        .get_principal(&group)
        .await?
        .ok_or(Error::NotFound)?;
    let principal = auth_provider
        .get_principal(principal_id)
        .await?
        .ok_or(Error::NotFound)?;

    if !principal.is_member_of(&group) {
        return Ok(Redirect::to(&principal_path(&group)).into_response());
    }

    auth_provider
        .remove_membership(principal_id, &group)
        .await?;
    Ok(Redirect::to(&principal_path(&group)).into_response())
}

async fn list_member_views_for_ids<AP: AuthenticationProvider + ?Sized>(
    auth_provider: &AP,
    config: &FrontendConfig,
    member_ids: &[String],
) -> Result<Vec<PrincipalView>, Error> {
    let mut members = Vec::new();
    for id in member_ids {
        if let Some(principal) = auth_provider.get_principal(id).await? {
            members.push(PrincipalView::from_principal(
                principal,
                config.admin.admin_group.as_deref(),
            ));
        }
    }

    members.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(members)
}

fn search_principal_views(
    principals: Vec<Principal>,
    admin_group: Option<&str>,
    term: Option<&str>,
    kind: &str,
    page: Option<usize>,
) -> Paginated<PrincipalView> {
    let views = principal_views_for_search(principals, admin_group, term, kind);
    paginate(views, page)
}

fn principal_views_for_search(
    principals: Vec<Principal>,
    admin_group: Option<&str>,
    term: Option<&str>,
    kind: &str,
) -> Vec<PrincipalView> {
    let mut views: Vec<_> = principals
        .into_iter()
        .map(|principal| PrincipalView::from_principal(principal, admin_group))
        .filter(|principal| principal_kind_matches(principal, kind))
        .filter(|principal| principal_matches(principal, term))
        .collect();

    sort_principal_views(&mut views);
    views
}

fn paginate<T>(mut items: Vec<T>, requested_page: Option<usize>) -> Paginated<T> {
    let total = items.len();
    let total_pages = total.div_ceil(PER_PAGE).max(1);
    let page = requested_page.unwrap_or(1).clamp(1, total_pages);
    let start = page.saturating_sub(1).saturating_mul(PER_PAGE).min(total);
    let end = start.saturating_add(PER_PAGE).min(total);
    let items = items.drain(start..end).collect();

    Paginated {
        items,
        total,
        page,
        per_page: PER_PAGE,
        total_pages,
        previous_page: (page > 1).then_some(page - 1),
        next_page: (page < total_pages).then_some(page + 1),
    }
}

fn principal_kind_matches(principal: &PrincipalView, kind: &str) -> bool {
    match kind {
        "group" => principal.principal_type == "GROUP",
        "person" => principal.principal_type != "GROUP",
        _ => true,
    }
}

fn sort_principal_views(views: &mut [PrincipalView]) {
    views.sort_by(|left, right| {
        let left_is_group = left.principal_type == "GROUP";
        let right_is_group = right.principal_type == "GROUP";
        right_is_group
            .cmp(&left_is_group)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn principal_matches(principal: &PrincipalView, term: Option<&str>) -> bool {
    let Some(term) = normalized_term(term) else {
        return true;
    };

    principal.id.to_lowercase().contains(&term)
        || principal
            .displayname
            .as_deref()
            .is_some_and(|displayname| displayname.to_lowercase().contains(&term))
        || principal.principal_type.to_lowercase().contains(&term)
        || principal
            .memberships
            .iter()
            .any(|membership| membership.to_lowercase().contains(&term))
}

impl PrincipalSearchItem {
    fn from_principal_view(principal: PrincipalView) -> Self {
        Self {
            detail_url: principal_path(&principal.id),
            id: principal.id,
            display_name: principal.displayname,
            principal_type: principal.principal_type,
            memberships: principal.memberships,
            is_admin: principal.is_admin,
            has_password: principal.has_password,
        }
    }
}

fn membership_redirect(principal_id: &str, group_id: &str, source: Option<&str>) -> String {
    if source == Some("group") {
        group_path(group_id)
    } else {
        principal_path(principal_id)
    }
}

fn normalize(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn normalized_term(term: Option<&str>) -> Option<String> {
    let term = term?.trim().to_lowercase();
    (!term.is_empty()).then_some(term)
}

fn parse_principal_type(value: Option<&str>) -> Result<PrincipalType, Error> {
    match value {
        None | Some("") => Ok(PrincipalType::Individual),
        Some(value) => Ok(PrincipalType::try_from(
            value.trim().to_ascii_uppercase().as_str(),
        )?),
    }
}

fn principal_path(id: &str) -> String {
    format!(
        "/frontend/settings/principals/{}",
        rfc_3986_percent_encode(id)
    )
}

fn group_path(id: &str) -> String {
    format!("/frontend/settings/groups/{}", rfc_3986_percent_encode(id))
}
