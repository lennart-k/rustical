use std::str::FromStr;

use http::{
    Uri,
    uri::{InvalidUri, PathAndQuery},
};
use rustical_xml::{ParseValueError, ValueDeserialize, ValueSerialize, XmlError};

use crate::rfc_3986_percent_encode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavPath(PathAndQuery);

#[derive(Debug, thiserror::Error)]
pub enum DavPathError {
    #[error("invalid filename")]
    InvalidFilename,
}

impl DavPath {
    pub fn path(&self) -> &str {
        self.0.path()
    }

    #[must_use]
    pub fn with_trailing_slash(self) -> Self {
        let path = format!("{path}/", path = self.path().trim_end_matches('/'));
        Self(PathAndQuery::from_str(&path).unwrap_or(self.0))
    }

    pub fn subpath(&self, filename: &str) -> Result<Self, DavPathError> {
        let path = format!(
            "{path}/{filename}",
            path = self.path().trim_end_matches('/'),
            filename = rfc_3986_percent_encode(filename)
        );
        Ok(Self(
            PathAndQuery::from_str(&path).map_err(|_| DavPathError::InvalidFilename)?,
        ))
    }
}

impl From<PathAndQuery> for DavPath {
    fn from(value: PathAndQuery) -> Self {
        Self(value)
    }
}

impl TryFrom<Uri> for DavPath {
    type Error = DavPathError;

    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        let parts = value.into_parts();
        if parts.scheme.is_none()
            && parts.authority.is_none()
            && let Some(path_and_query) = parts.path_and_query
        {
            return Ok(Self(path_and_query));
        }

        Err(DavPathError::InvalidFilename)
    }
}

impl std::fmt::Display for DavPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for DavPath {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(PathAndQuery::from_str(s)?))
    }
}

impl From<DavPath> for Uri {
    fn from(value: DavPath) -> Self {
        value.0.into()
    }
}

impl ValueDeserialize for DavPath {
    fn deserialize(val: &str) -> Result<Self, XmlError> {
        Self::from_str(val).map_err(|err| XmlError::InvalidValue(ParseValueError::InvalidUri(err)))
    }
}

impl ValueSerialize for DavPath {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use crate::resource::DavPath;

    #[rstest]
    #[case("ähm.ics")]
    #[case("hallo - test.ics")]
    fn test_subpath(#[case] filename: &str) {
        assert_eq!(
            format!("/{filename}"),
            DavPath::from_str("/")
                .unwrap()
                .subpath(filename)
                .unwrap()
                .path()
        );
    }
}
