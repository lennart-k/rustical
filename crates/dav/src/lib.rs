#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
pub mod error;
pub mod extensions;
pub mod header;
pub mod namespace;
pub mod privileges;
pub mod resource;
pub mod resources;
pub mod xml;
use std::borrow::Cow;

pub use error::Error;
use http::Uri;
use itertools::Itertools;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str};

/// Minimal Principal trait for a WebDAV service.
/// For the purpose of WebDAV we only need to identify a principal id
/// to correctly return current-user-principal.
pub trait Principal: std::fmt::Debug + Clone + Send + Sync + 'static {
    fn get_id(&self) -> &str;
}

/// Characters that need to be percent-encoded according to WebDAV spec
///   ```txt
///   reserved    = gen-delims / sub-delims
///
///   gen-delims  = ":" / "/" / "?" / "#" / "[" / "]" / "@"
///
///   sub-delims  = "!" / "$" / "&" / "'" / "(" / ")"
///               / "*" / "+" / "," / ";" / "="
///   ```
const RFC_3986: &AsciiSet = &CONTROLS
    .add(b':')
    .add(b'/')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\\')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'=')
    // Not in RFC 3986 but also necessary
    .add(b' ');

#[inline]
#[must_use]
pub fn rfc_3986_percent_encode(input: &'_ str) -> percent_encoding::PercentEncode<'_> {
    percent_encoding::percent_encode(input.as_bytes(), RFC_3986)
}

#[cfg(test)]
mod tests {
    use crate::rfc_3986_percent_encode;

    /// We need to make sure that URI-reserved characters are encoded but not "."
    #[rstest::rstest]
    #[case("hello.ics", "hello.ics")]
    #[case("hello@example.com", "hello%40example.com")]
    #[case("slash/slash", "slash%2Fslash")]
    fn test_percent_encoding(#[case] input: &str, #[case] expected_result: &str) {
        assert_eq!(&rfc_3986_percent_encode(input).to_string(), expected_result);
    }
}

pub fn resolve_child_uri<'a>(
    collection_uri: &'_ Uri,
    child_uri: &'a Uri,
) -> Option<Vec<Cow<'a, str>>> {
    // If explicit scheme given for child it MUST match
    if let Some(child_scheme) = child_uri.scheme() {
        let collection_scheme = collection_uri.scheme()?;
        if child_scheme != collection_scheme {
            return None;
        }
    }

    // If explicit authority given for child it MUST match
    if let Some(child_authority) = child_uri.authority() {
        let collection_authority = collection_uri.authority()?;
        if child_authority != collection_authority {
            return None;
        }
    }

    // To properly handle urldecoding we must handle individual path segments
    let collection_path_segments = collection_uri
        .path()
        .split('/')
        .filter(|seg| !seg.is_empty());
    let mut child_path_segments = child_uri.path().split('/').filter(|seg| !seg.is_empty());

    for collection_segment in collection_path_segments {
        dbg!(collection_segment);
        let Some(child_segment) = child_path_segments.next() else {
            // child_uri is not child of collection_uri
            return None;
        };
        // Make sure that paths on same level match
        if percent_decode_str(collection_segment).collect_vec()
            != percent_decode_str(child_segment).collect_vec()
        {
            return None;
        }
    }

    child_path_segments
        .map(|segment| percent_decode_str(segment).decode_utf8().ok())
        .collect()
}

#[cfg(test)]
mod uri_tests {
    use crate::resolve_child_uri;
    use http::Uri;
    use std::borrow::Cow;

    #[rstest::rstest]
    #[case("https://rustical.example.com", "/hello", Some(vec!["hello".into()]))]
    #[case("https://rustical.example.com/", "/hello", Some(vec!["hello".into()]))]
    // Not absolute
    #[case("https://rustical.example.com/", "hello", None)]
    // Different origins
    #[case(
        "https://rustical.example.com/caldav/principal/user%40example%2Ecom/cal/",
        "https://cal.hello.dev/caldav/principal/user%40example%2Ecom/cal/hello.ics",
        None
    )]
    // Trivial, both equally escaped
    #[case(
        "/caldav/principal/user%40example%2Ecom/cal/",
        "/caldav/principal/user%40example%2Ecom/cal/hello.ics",
        Some(vec!["hello.ics".into()])
    )]
    // Both escaped differently
    #[case(
        "/caldav/principal/user%40example%2Ecom/cal/",
        "/caldav/principal/user@example.com/cal/unescaped.ics",
        Some(vec!["unescaped.ics".into()])
    )]
    // Both escaped differently
    #[case(
        "/caldav/principal/user%40example%2Ecom/cal/",
        "/caldav/principal/user%40example.com/cal/shouldwork.ics",
        Some(vec!["shouldwork.ics".into()])
    )]
    // Both escaped, different paths
    #[case(
        "/caldav/principal/user%40example%2Ecom/cal/",
        "/caldav/principal/user%40example%2Ecom/nocal/hello.ics",
        None
    )]
    // Both escaped differently
    #[case(
        "/caldav/principal/user%40example.com/cal/",
        "/caldav/principal/user%40example%2Ecom/cal/hello.ics",
        Some(vec!["hello.ics".into()])
    )]
    // Both escaped differently
    #[case(
        "/caldav/principal/user%40example.com/cal/",
        "/caldav/principal/user@example.com/cal/unescaped.ics",
        Some(vec!["unescaped.ics".into()])
    )]
    // Both escaped differently
    #[case(
        "/caldav/principal/user%40example.com/cal/",
        "/caldav/principal/user%40example.com/cal/shouldwork.ics",
        Some(vec!["shouldwork.ics".into()])
    )]
    // Different paths
    #[case(
        "/caldav/principal/user%40example.com/cal/",
        "/caldav/principal/user%40example%2Ecom/nocal/hello.ics",
        None
    )]
    // Empty root path
    #[case("https://example.com", "/hello.ics", Some(vec!["hello.ics".into()]))]
    // Escaped child path
    #[case("https://example.com", "/hello%2Eics", Some(vec!["hello.ics".into()]))]
    fn test_resolve_child_uri(
        #[case] collection: &'static str,
        #[case] child: &'static str,
        #[case] expected_out: Option<Vec<Cow<'static, str>>>,
    ) {
        let collection_uri = Uri::from_static(collection);
        let child_uri = Uri::from_static(child);

        assert_eq!(resolve_child_uri(&collection_uri, &child_uri), expected_out);
    }
}
