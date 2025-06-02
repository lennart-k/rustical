pub trait PrincipalUri: 'static {
    fn principal_uri(&self, principal: &str) -> String;
}
