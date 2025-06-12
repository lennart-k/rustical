pub trait PrincipalUri: 'static + Clone + Send + Sync {
    fn principal_collection(&self) -> String;
    fn principal_uri(&self, principal: &str) -> String;
}
