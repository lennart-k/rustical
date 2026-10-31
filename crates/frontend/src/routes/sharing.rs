//! Sharing in the frontend: the shares of a collection (for its owner), the collections shared
//! with the user, and how private and confidential objects appear in the user's shares.
//!
//! Plain HTML forms. Every POST must come from this host (Origin, else Referer): the session
//! cookie alone would let another site of the same site post on the user's behalf.
use crate::pages::DefaultLayoutData;
use crate::pages::user::{Section, UserPage};
use askama::Template;
use askama_web::WebTemplate;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use headers::{HeaderMapExt, Host};
use http::{HeaderMap, StatusCode};
use rustical_store::access;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use rustical_store::shared::Policy;
use rustical_store::{
    Access, AddressbookReadStore, AddressbookStore, CalendarReadStore, CalendarStore, Error,
    ShareRejection, ShareStore, ShareViewUpdate,
};
use std::sync::Arc;

/// The collection kinds that can be shared
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Calendar,
    Addressbook,
}

impl Kind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "calendar" => Some(Self::Calendar),
            "addressbook" => Some(Self::Addressbook),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Calendar => "calendar",
            Self::Addressbook => "addressbook",
        }
    }
}

/// 403 unless the request comes from a page of this host
fn check_same_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    let host = headers.typed_get::<Host>().map(|host| host.to_string());
    let source = headers
        .get(http::header::ORIGIN)
        .or_else(|| headers.get(http::header::REFERER))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| url::Url::parse(value).ok())
        .and_then(|url| {
            let host = url.host_str()?.to_owned();
            Some(
                url.port()
                    .map_or_else(|| host.clone(), |port| format!("{host}:{port}")),
            )
        });
    match (host, source) {
        (Some(host), Some(source)) if host == source => Ok(()),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

fn form(body: &str) -> Vec<(String, String)> {
    url::form_urlencoded::parse(body.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn field<'a>(form: &'a [(String, String)], name: &str) -> Option<&'a str> {
    form.iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn access_of(form: &[(String, String)]) -> Access {
    if field(form, "access") == Some("write") {
        Access::Write
    } else {
        Access::Read
    }
}

fn message(error: &Error) -> String {
    match error {
        Error::ShareRejected(rejection) => match rejection {
            ShareRejection::Owner => "You cannot share a collection with its owner.".to_owned(),
            ShareRejection::OwnerMember => {
                "Members of the owning group already have full access.".to_owned()
            }
            ShareRejection::UnknownSharee => {
                "There is no user or group with this name or e-mail address.".to_owned()
            }
            ShareRejection::AmbiguousSharee => {
                "This name and this e-mail address belong to different users.".to_owned()
            }
            ShareRejection::NotShareable => {
                "Birthday calendars and subscriptions cannot be shared.".to_owned()
            }
        },
        Error::NotFound => "Not found.".to_owned(),
        Error::InvalidVisibility => {
            "Confidential objects may not show more than private ones.".to_owned()
        }
        other => other.to_string(),
    }
}

/// A sharee as the pages show it
pub struct ShareeInfo {
    pub id: String,
    pub displayname: String,
    pub is_group: bool,
    /// The current members, only for a viewer who is a member of the group
    pub members: Option<Vec<String>>,
}

async fn sharee_info<AP: AuthenticationProvider>(
    auth_provider: &AP,
    viewer: &Principal,
    id: &str,
) -> Result<ShareeInfo, Error> {
    let principal = auth_provider
        .get_principal(id)
        .await?
        .ok_or(Error::NotFound)?;
    let is_group = matches!(principal.principal_type, PrincipalType::Group);
    // Member lists are only for members (anyone could otherwise read them by typing a name)
    let members = if is_group && viewer.is_principal(id) {
        Some(auth_provider.list_members(id).await?)
    } else {
        None
    };
    Ok(ShareeInfo {
        displayname: principal
            .displayname
            .unwrap_or_else(|| principal.id.clone()),
        id: principal.id,
        is_group,
        members,
    })
}

/// Resolves what a user typed (principal id or e-mail address), as the store will
async fn resolve_input<AP: AuthenticationProvider>(
    auth_provider: &AP,
    share_store: &dyn ShareStore,
    input: &str,
) -> Result<String, Error> {
    let input = input.trim();
    let by_id = auth_provider.get_principal(input).await?.map(|p| p.id);
    let by_email = share_store.resolve_email(input).await?;
    match (by_id, by_email) {
        (Some(id), Some(other)) if id != other => Err(ShareRejection::AmbiguousSharee.into()),
        (Some(id), _) | (None, Some(id)) => Ok(id),
        (None, None) => Err(ShareRejection::UnknownSharee.into()),
    }
}

/// One share of the collection on its page
pub struct ShareRow {
    pub sharee: ShareeInfo,
    pub access: Access,
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/shares.html")]
pub struct SharesPage {
    pub user: Principal,
    pub kind: Kind,
    pub owner: String,
    pub id: String,
    pub displayname: String,
    pub shares: Vec<ShareRow>,
    /// A calendar's policy: how its private and confidential objects appear to its sharees
    pub visibility: Option<Policy>,
    pub error: Option<String>,
}

impl SharesPage {
    /// The names of the visibility levels, by level
    pub const LEVELS: [&str; 5] = LEVELS;
}

impl DefaultLayoutData for SharesPage {
    fn get_user(&self) -> Option<&Principal> {
        Some(&self.user)
    }
}

/// Step 1 of adding a share. It shows only what the user typed: whom a name or an address
/// belongs to is not for every user to learn.
#[derive(Template, WebTemplate)]
#[template(path = "pages/share_confirm.html")]
pub struct ShareConfirmPage {
    pub user: Principal,
    pub kind: Kind,
    pub owner: String,
    pub id: String,
    pub displayname: String,
    /// What the user typed, shared with whom it belongs to when the share is added
    pub input: String,
    pub access: Access,
    /// The current members, only if the input is the id of a group the user is a member of
    pub members: Option<Vec<String>>,
}

impl DefaultLayoutData for ShareConfirmPage {
    fn get_user(&self) -> Option<&Principal> {
        Some(&self.user)
    }
}

/// The display name of the user's own collection, if the user may manage it
async fn own_collection<CS: CalendarStore, AS: AddressbookStore>(
    cal_store: &CS,
    addr_store: &AS,
    user: &Principal,
    kind: Kind,
    owner: &str,
    id: &str,
) -> Result<String, Box<Response>> {
    if !access::may_use_home(user, owner) {
        return Err(Box::new(StatusCode::FORBIDDEN.into_response()));
    }
    let displayname = match kind {
        Kind::Calendar => CalendarReadStore::get_calendar(cal_store, owner, id, false)
            .await
            .map(|calendar| calendar.meta.displayname),
        Kind::Addressbook => AddressbookReadStore::get_addressbook(addr_store, owner, id, false)
            .await
            .map(|addressbook| addressbook.displayname),
    };
    match displayname {
        Ok(displayname) => Ok(displayname.unwrap_or_else(|| id.to_owned())),
        Err(Error::NotFound) => Err(Box::new(StatusCode::NOT_FOUND.into_response())),
        Err(err) => Err(Box::new(err.into_response())),
    }
}

#[allow(clippy::too_many_arguments)]
async fn shares_page<AP: AuthenticationProvider>(
    auth_provider: &AP,
    share_store: &dyn ShareStore,
    user: Principal,
    kind: Kind,
    owner: String,
    id: String,
    displayname: String,
    error: Option<String>,
) -> Result<Response, Error> {
    let list: Vec<(String, Access)> = match kind {
        Kind::Calendar => share_store
            .list_calendar_shares(&owner, &id)
            .await?
            .into_iter()
            .map(|share| (share.sharee, share.access))
            .collect(),
        Kind::Addressbook => share_store
            .list_addressbook_shares(&owner, &id)
            .await?
            .into_iter()
            .map(|share| (share.sharee, share.access))
            .collect(),
    };
    let mut rows = vec![];
    for (sharee, access) in list {
        rows.push(ShareRow {
            sharee: sharee_info(auth_provider, &user, &sharee).await?,
            access,
        });
    }
    let visibility = match kind {
        Kind::Calendar => Some(share_store.get_visibility(&owner, &id).await?),
        Kind::Addressbook => None,
    };
    Ok(SharesPage {
        user,
        kind,
        owner,
        id,
        displayname,
        shares: rows,
        visibility,
        error,
    }
    .into_response())
}

pub async fn route_shares<AP: AuthenticationProvider, CS: CalendarStore, AS: AddressbookStore>(
    Path((owner, kind, id)): Path<(String, String, String)>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Extension(cal_store): Extension<Arc<CS>>,
    Extension(addr_store): Extension<Arc<AS>>,
    Extension(share_store): Extension<Arc<dyn ShareStore>>,
    user: Principal,
) -> Result<Response, Error> {
    let Some(kind) = Kind::parse(&kind) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let displayname = match own_collection(
        cal_store.as_ref(),
        addr_store.as_ref(),
        &user,
        kind,
        &owner,
        &id,
    )
    .await
    {
        Ok(displayname) => displayname,
        Err(response) => return Ok(*response),
    };
    shares_page(
        auth_provider.as_ref(),
        share_store.as_ref(),
        user,
        kind,
        owner,
        id,
        displayname,
        None,
    )
    .await
}

/// The forms of the shares page: `preview` (step 1 of adding: who is it?), `add` (step 2),
/// `access` (change one share), `remove` (end one or several shares) and, for a calendar,
/// `visibility` (its policy)
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub async fn route_post_shares<
    AP: AuthenticationProvider,
    CS: CalendarStore,
    AS: AddressbookStore,
>(
    Path((owner, kind, id, action)): Path<(String, String, String, String)>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Extension(cal_store): Extension<Arc<CS>>,
    Extension(addr_store): Extension<Arc<AS>>,
    Extension(share_store): Extension<Arc<dyn ShareStore>>,
    user: Principal,
    headers: HeaderMap,
    body: String,
) -> Result<Response, Error> {
    if let Err(status) = check_same_origin(&headers) {
        return Ok((status, "Cross-site request refused").into_response());
    }
    let Some(kind) = Kind::parse(&kind) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let displayname = match own_collection(
        cal_store.as_ref(),
        addr_store.as_ref(),
        &user,
        kind,
        &owner,
        &id,
    )
    .await
    {
        Ok(displayname) => displayname,
        Err(response) => return Ok(*response),
    };
    let form = form(&body);
    let back = format!("/frontend/user/{owner}/{}/{id}/shares", kind.as_str());

    let result: Result<(), Error> = match action.as_str() {
        "preview" => {
            let input = field(&form, "sharee").unwrap_or_default().trim();
            match resolve_input(auth_provider.as_ref(), share_store.as_ref(), input).await {
                Ok(sharee) => {
                    // Member lists only for members, and only for the group's own name
                    let members = if sharee == input && user.is_principal(&sharee) {
                        sharee_info(auth_provider.as_ref(), &user, &sharee)
                            .await?
                            .members
                    } else {
                        None
                    };
                    return Ok(ShareConfirmPage {
                        input: input.to_owned(),
                        user,
                        kind,
                        owner,
                        id,
                        displayname,
                        access: access_of(&form),
                        members,
                    }
                    .into_response());
                }
                Err(err) => Err(err),
            }
        }
        "add" => {
            let sharee = field(&form, "sharee").unwrap_or_default();
            let access = access_of(&form);
            match kind {
                Kind::Calendar => share_store
                    .share_calendar(&owner, &id, sharee, access, Some(&user.id))
                    .await
                    .map(drop),
                Kind::Addressbook => share_store
                    .share_addressbook(&owner, &id, sharee, access, Some(&user.id))
                    .await
                    .map(drop),
            }
        }
        "access" => {
            // Only an existing share: one the sharee ended in the meantime stays ended
            let sharee = field(&form, "sharee").unwrap_or_default();
            let access = access_of(&form);
            match kind {
                Kind::Calendar => {
                    share_store
                        .set_calendar_share_access(&owner, &id, sharee, access)
                        .await
                }
                Kind::Addressbook => {
                    share_store
                        .set_addressbook_share_access(&owner, &id, sharee, access)
                        .await
                }
            }
        }
        "remove" => {
            let mut result = Ok(());
            for (key, sharee) in &form {
                if key != "sharee" {
                    continue;
                }
                let removed = match kind {
                    Kind::Calendar => share_store.unshare_calendar(&owner, &id, sharee).await,
                    Kind::Addressbook => share_store.unshare_addressbook(&owner, &id, sharee).await,
                };
                // Already gone (e.g. the sharee removed it) is fine
                if let Err(err) = removed
                    && !err.is_not_found()
                {
                    result = Err(err);
                }
            }
            result
        }
        "visibility" if kind == Kind::Calendar => {
            let level = |name: &str| field(&form, name).and_then(|value| value.parse::<u8>().ok());
            match (level("private"), level("confidential")) {
                (Some(private), Some(confidential)) => match Policy::new(private, confidential) {
                    Some(policy) => share_store.set_visibility(&owner, &id, policy).await,
                    None => Err(Error::InvalidVisibility),
                },
                _ => Err(Error::InvalidVisibility),
            }
        }
        _ => return Ok(StatusCode::NOT_FOUND.into_response()),
    };
    match result {
        Ok(()) => Ok(Redirect::to(&back).into_response()),
        Err(err) => {
            let message = if action == "access" && err.is_not_found() {
                "This share does not exist any more.".to_owned()
            } else {
                message(&err)
            };
            shares_page(
                auth_provider.as_ref(),
                share_store.as_ref(),
                user,
                kind,
                owner,
                id,
                displayname,
                Some(message),
            )
            .await
        }
    }
}

/// A collection shared with one of the user's homes
pub struct SharedRow {
    pub kind: Kind,
    pub home: String,
    pub name: String,
    pub owner: String,
    pub displayname: String,
    pub own_displayname: Option<String>,
    pub color: Option<String>,
    pub access: Access,
}

impl SharedRow {
    /// The color for a style attribute: the owner sets it unchecked (PROPPATCH), so only a CSS
    /// hex color is used
    #[must_use]
    pub fn css_color(&self) -> &str {
        self.color
            .as_deref()
            .filter(|color| is_hex_color(color))
            .unwrap_or("transparent")
    }

    /// The color for a color input (`#rrggbb` only)
    #[must_use]
    pub fn picker_color(&self) -> &str {
        match self.color.as_deref() {
            Some(color) if is_hex_color(color) && matches!(color.len(), 7 | 9) => &color[..7],
            _ => "#888888",
        }
    }
}

/// `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`
#[must_use]
pub fn is_hex_color(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|digits| {
        matches!(digits.len(), 3 | 4 | 6 | 8) && digits.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

impl Section for SharingSection {
    fn name() -> &'static str {
        "sharing"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/sharing_section.html")]
pub struct SharingSection {
    pub user: Principal,
    pub shared: Vec<SharedRow>,
    pub error: Option<String>,
}

/// The names of the visibility levels, by level
pub const LEVELS: [&str; 5] = [
    "Hidden",
    "Busy (time only)",
    "Title",
    "Title and location",
    "Everything",
];

async fn sharing_section<CS: CalendarStore, AS: AddressbookStore>(
    cal_store: &CS,
    addr_store: &AS,
    share_store: &dyn ShareStore,
    user: Principal,
    error: Option<String>,
) -> Result<Response, Error> {
    let mut shared = vec![];
    for home in user.memberships() {
        for share in share_store.calendars_shared_with(home).await? {
            let Ok(calendar) =
                CalendarReadStore::get_calendar(cal_store, &share.owner, &share.cal_id, false)
                    .await
            else {
                continue;
            };
            shared.push(SharedRow {
                kind: Kind::Calendar,
                home: home.to_owned(),
                displayname: share
                    .displayname
                    .clone()
                    .or(calendar.meta.displayname)
                    .unwrap_or_else(|| share.cal_id.clone()),
                own_displayname: share.displayname,
                color: share.color.or(calendar.meta.color),
                name: share.name,
                owner: share.owner,
                access: share.access,
            });
        }
        for share in share_store.addressbooks_shared_with(home).await? {
            let Ok(addressbook) = AddressbookReadStore::get_addressbook(
                addr_store,
                &share.owner,
                &share.addressbook_id,
                false,
            )
            .await
            else {
                continue;
            };
            shared.push(SharedRow {
                kind: Kind::Addressbook,
                home: home.to_owned(),
                displayname: share
                    .displayname
                    .clone()
                    .or(addressbook.displayname)
                    .unwrap_or_else(|| share.addressbook_id.clone()),
                own_displayname: share.displayname,
                color: share.color,
                name: share.name,
                owner: share.owner,
                access: share.access,
            });
        }
    }
    Ok(UserPage {
        section: SharingSection {
            user: user.clone(),
            shared,
            error,
        },
        user,
    }
    .into_response())
}

pub async fn route_sharing<CS: CalendarStore, AS: AddressbookStore>(
    Path(user_id): Path<String>,
    Extension(cal_store): Extension<Arc<CS>>,
    Extension(addr_store): Extension<Arc<AS>>,
    Extension(share_store): Extension<Arc<dyn ShareStore>>,
    user: Principal,
) -> Result<Response, Error> {
    if user_id != user.id {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    sharing_section(
        cal_store.as_ref(),
        addr_store.as_ref(),
        share_store.as_ref(),
        user,
        None,
    )
    .await
}

/// The forms of the sharing page: `view` (own name and color of a share) and `remove` (end a
/// share in one of the user's homes)
pub async fn route_post_sharing<CS: CalendarStore, AS: AddressbookStore>(
    Path((user_id, action)): Path<(String, String)>,
    Extension(cal_store): Extension<Arc<CS>>,
    Extension(addr_store): Extension<Arc<AS>>,
    Extension(share_store): Extension<Arc<dyn ShareStore>>,
    user: Principal,
    headers: HeaderMap,
    body: String,
) -> Result<Response, Error> {
    if let Err(status) = check_same_origin(&headers) {
        return Ok((status, "Cross-site request refused").into_response());
    }
    if user_id != user.id {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    let form = form(&body);
    let home = field(&form, "home").unwrap_or_default();
    // Every form acts on one of the user's homes
    if !access::may_use_home(&user, home) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    let name = field(&form, "name").unwrap_or_default();
    let kind = field(&form, "kind").and_then(Kind::parse);

    let result = match (action.as_str(), kind) {
        ("view", Some(kind)) => {
            let nonempty = |value: Option<&str>| {
                value
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
            };
            let update = ShareViewUpdate {
                displayname: Some(nonempty(field(&form, "displayname"))),
                color: Some(
                    nonempty(field(&form, "color")).filter(|_| field(&form, "own_color").is_some()),
                ),
                order: None,
            };
            match kind {
                Kind::Calendar => {
                    share_store
                        .update_calendar_share_view(home, name, update)
                        .await
                }
                Kind::Addressbook => {
                    share_store
                        .update_addressbook_share_view(home, name, update)
                        .await
                }
            }
        }
        ("remove", Some(kind)) => match kind {
            Kind::Calendar => match share_store.get_calendar_share(home, name).await? {
                Some(share) => {
                    share_store
                        .unshare_calendar(&share.owner, &share.cal_id, home)
                        .await
                }
                None => Err(Error::NotFound),
            },
            Kind::Addressbook => match share_store.get_addressbook_share(home, name).await? {
                Some(share) => {
                    share_store
                        .unshare_addressbook(&share.owner, &share.addressbook_id, home)
                        .await
                }
                None => Err(Error::NotFound),
            },
        },
        _ => return Ok(StatusCode::NOT_FOUND.into_response()),
    };
    match result {
        Ok(()) => Ok(Redirect::to(&format!("/frontend/user/{}/sharing", user.id)).into_response()),
        Err(err) => {
            sharing_section(
                cal_store.as_ref(),
                addr_store.as_ref(),
                share_store.as_ref(),
                user,
                Some(message(&err)),
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_hex_color;

    #[test]
    fn test_hex_colors_only() {
        for color in ["#abc", "#ABCD", "#123456", "#12345678"] {
            assert!(is_hex_color(color), "{color}");
        }
        for color in [
            "",
            "#",
            "red",
            "#12345",
            "#1234567",
            "#ggg",
            "#123456;background:url(https://evil/x)",
            "red;background:url(https://evil/x)",
            "#１２３",
        ] {
            assert!(!is_hex_color(color), "{color}");
        }
    }
}
