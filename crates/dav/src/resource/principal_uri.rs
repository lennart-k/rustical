use http::Uri;

pub trait PrincipalUri: 'static + Clone + Send + Sync {
    fn principal_collection(&self) -> Uri;
    fn principal_uri(&self, principal: &str) -> Uri;
}
