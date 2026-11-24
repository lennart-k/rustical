use askama::Template;
use askama_web::WebTemplate;
use rustical_store::auth::Principal;
use serde::Serialize;

use crate::pages::DefaultLayoutData;

pub trait AdminSection: Template {
    fn name() -> &'static str;
}

#[derive(Debug, Clone, Serialize)]
pub struct PrincipalView {
    pub id: String,
    pub displayname: Option<String>,
    pub principal_type: String,
    pub memberships: Vec<String>,
    pub has_password: bool,
    pub is_admin: bool,
}

impl PrincipalView {
    #[must_use]
    pub fn from_principal(principal: Principal, admin_group: Option<&str>) -> Self {
        Self {
            is_admin: admin_group.is_some_and(|admin_group| principal.is_member_of(admin_group)),
            id: principal.id,
            displayname: principal.displayname,
            principal_type: principal.principal_type.to_string(),
            memberships: principal.memberships,
            has_password: principal.password.is_some(),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin.html")]
pub struct AdminPage<S: AdminSection> {
    pub user: Principal,
    pub section: S,
    pub section_name: &'static str,
}

impl<S: AdminSection> DefaultLayoutData for AdminPage<S> {
    fn get_user(&self) -> Option<&Principal> {
        Some(&self.user)
    }
}
