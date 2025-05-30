use quick_xml::name::Namespace;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NamespaceOwned(pub Vec<u8>);

impl<'a> From<Namespace<'a>> for NamespaceOwned {
    fn from(value: Namespace<'a>) -> Self {
        Self(value.0.to_vec())
    }
}

impl From<String> for NamespaceOwned {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl From<&str> for NamespaceOwned {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl<'a> From<&'a Namespace<'a>> for NamespaceOwned {
    fn from(value: &'a Namespace<'a>) -> Self {
        Self(value.0.to_vec())
    }
}

impl NamespaceOwned {
    pub fn as_ref(&self) -> Namespace {
        Namespace(&self.0)
    }
}
