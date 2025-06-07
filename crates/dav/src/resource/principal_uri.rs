pub trait PrincipalUri: 'static + Clone + Send + Sync {
    fn principal_uri(&self, principal: &str) -> String;
}
