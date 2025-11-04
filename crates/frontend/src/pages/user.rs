use askama::Template;
use askama_web::WebTemplate;
use rustical_store::auth::Principal;

use crate::pages::DefaultLayoutData;

pub trait Section: Template {
    fn name() -> &'static str;
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/user.html")]
pub struct UserPage<S: Section> {
    pub user: Principal,
    pub section: S,
}

impl<S: Section> DefaultLayoutData for UserPage<S> {
    fn get_user(&self) -> Option<&Principal> {
        Some(&self.user)
    }
}
