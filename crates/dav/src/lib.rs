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
pub use error::Error;
use percent_encoding::{AsciiSet, CONTROLS};

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
