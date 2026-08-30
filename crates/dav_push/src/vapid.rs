use base64ct::{Base64UrlUnpadded, Encoding};
use rustical_xml::XmlSerialize;
use serde::Deserialize;
use thiserror::Error;
use web_push::VapidKey;

#[derive(Debug, Error)]
pub enum VapidError {
    #[error(transparent)]
    WebPushError(#[from] web_push::WebPushError),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Clone)]
pub struct VapidKeypair(pub VapidKey);

impl std::fmt::Debug for VapidKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VapidPublicKeyB64").finish_non_exhaustive()
    }
}

impl VapidKeypair {
    #[must_use]
    pub fn generate_p256() -> Self {
        Self(VapidKey::generate())
    }
    #[must_use]
    pub fn public(&self) -> VapidPublicKey {
        VapidPublicKey(self.0.public_key())
    }

    pub fn from_pem(pem: &str) -> Result<Self, VapidError> {
        Ok(Self(VapidKey::from_pem(pem)?))
    }

    pub fn to_pem(&self) -> Result<String, VapidError> {
        Ok(self.0.to_pem()?)
    }
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub struct VapidPublicKeyB64(pub String);

impl std::fmt::Debug for VapidPublicKeyB64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VapidPublicKeyB64").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
struct VapidPublicKeyProp<'b> {
    #[xml(ty = "attr", rename = "type")]
    pub ty: &'static str,
    #[xml(ty = "text")]
    pub key: &'b str,
}

impl XmlSerialize for VapidPublicKeyB64 {
    fn serialize(
        &self,
        ns: Option<quick_xml::name::Namespace>,
        tag: Option<&str>,
        namespaces: &std::collections::HashMap<quick_xml::name::Namespace, &str>,
        writer: &mut quick_xml::Writer<&mut Vec<u8>>,
    ) -> std::io::Result<()> {
        VapidPublicKeyProp {
            ty: "p256ecdsa",
            key: &self.0,
        }
        .serialize(ns, tag, namespaces, writer)
    }

    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        VapidPublicKeyProp {
            ty: "p256ecdsa",
            key: &self.0,
        }
        .attributes()
    }
}

pub struct VapidPublicKey(Vec<u8>);

impl std::fmt::Debug for VapidPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VapidPublicKey").finish_non_exhaustive()
    }
}

impl VapidPublicKey {
    #[must_use]
    pub fn encode_b64(&self) -> VapidPublicKeyB64 {
        VapidPublicKeyB64(Base64UrlUnpadded::encode_string(&self.0))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::vapid::VapidKeypair;

    // pkcs8-encoded private key
    pub const PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgTB3vavSTXm+w9v6Q
9eCwVFwRhnLfWuO3y2VwpfKhYg+hRANCAARRJ6EbENwBqqjN6v+2jxsalRvqEKUr
3oBLcSuhKaTh5UrFE9kQUxWnmft0yL35yRmHHSpSyD3A4pqUi0satvIp
-----END PRIVATE KEY-----
";

    pub const PRIVATE_KEY_PEM_SEC1: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIMwug/U2ds75hkEIeou9s0kj1ziCJETswt5S9ztJ2L5SoAoGCCqGSM49
AwEHoUQDQgAEyjUeooXqyQxljKSu17126pjAEPTyYNApO6dGQl0PexMn0T7LI3qw
mU9ZOko2Gn7LYp5LqgA0cX6rfDftsKVvtQ==
-----END EC PRIVATE KEY-----";

    pub const PUBLIC_KEY_B64: &str =
        "BFEnoRsQ3AGqqM3q_7aPGxqVG-oQpSvegEtxK6EppOHlSsUT2RBTFaeZ-3TIvfnJGYcdKlLIPcDimpSLSxq28ik";

    #[test]
    fn test_generate_key() {
        let key = VapidKeypair::generate_p256();
        let pem = key.to_pem().unwrap();
        assert!(pem.starts_with("-----BEGIN PRIVATE KEY-----\n"));
        assert!(pem.ends_with("-----END PRIVATE KEY-----\n"));
    }

    #[test]
    fn test_pem_roundtrip() {
        let key = VapidKeypair::from_pem(PRIVATE_KEY_PEM).unwrap();
        assert_eq!(key.to_pem().unwrap(), PRIVATE_KEY_PEM);
    }

    #[test]
    fn test_public_key() {
        let key = VapidKeypair::from_pem(PRIVATE_KEY_PEM).unwrap();
        assert_eq!(key.public().encode_b64().0, PUBLIC_KEY_B64);
    }

    #[test]
    fn test_parse_pkcs8() {
        VapidKeypair::from_pem(PRIVATE_KEY_PEM).unwrap();
    }

    #[test]
    fn test_parse_sec1() {
        VapidKeypair::from_pem(PRIVATE_KEY_PEM_SEC1).unwrap();
    }
}
