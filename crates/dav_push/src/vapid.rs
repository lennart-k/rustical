use ct_codecs::Encoder;
use openssl::ec::EcGroup;
use openssl::nid::Nid;
use openssl::{
    bn::BigNumContext,
    ec::{EcKey, PointConversionForm},
    error::ErrorStack,
    pkey::Private,
};
use rustical_xml::XmlSerialize;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VapidError {
    #[error(transparent)]
    OpenSslError(#[from] ErrorStack),
    #[error(transparent)]
    EncodingError(#[from] ct_codecs::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Clone)]
pub struct VapidKeypair(pub openssl::ec::EcKey<Private>);

impl std::fmt::Debug for VapidKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VapidPublicKeyB64").finish_non_exhaustive()
    }
}

impl VapidKeypair {
    pub fn generate_p256() -> Result<Self, VapidError> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        Ok(Self(EcKey::generate(&group)?))
    }
    pub fn public(&self) -> Result<VapidPublicKey, VapidError> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        Ok(VapidPublicKey(self.0.public_key().to_owned(&group)?))
    }

    pub fn from_pem(pem: &str) -> Result<Self, VapidError> {
        Ok(Self(EcKey::private_key_from_pem(pem.as_bytes())?))
    }

    pub fn to_pem(&self) -> Result<String, VapidError> {
        Ok(String::from_utf8(self.0.private_key_to_pem()?)?)
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

pub struct VapidPublicKey(pub openssl::ec::EcPoint);

impl std::fmt::Debug for VapidPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VapidPublicKey").finish_non_exhaustive()
    }
}

impl VapidPublicKey {
    pub fn encode_b64(&self) -> Result<VapidPublicKeyB64, VapidError> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let mut ctx = BigNumContext::new()?;
        let bytes = self
            .0
            .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)?;
        Ok(VapidPublicKeyB64(
            ct_codecs::Base64UrlSafeNoPadding::encode_to_string(bytes)?,
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::vapid::VapidKeypair;

    pub const PRIVATE_KEY_PEM: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIEwd72r0k15vsPb+kPXgsFRcEYZy31rjt8tlcKXyoWIPoAoGCCqGSM49
AwEHoUQDQgAEUSehGxDcAaqozer/to8bGpUb6hClK96AS3EroSmk4eVKxRPZEFMV
p5n7dMi9+ckZhx0qUsg9wOKalItLGrbyKQ==
-----END EC PRIVATE KEY-----
";

    pub const PUBLIC_KEY_B64: &str =
        "BFEnoRsQ3AGqqM3q_7aPGxqVG-oQpSvegEtxK6EppOHlSsUT2RBTFaeZ-3TIvfnJGYcdKlLIPcDimpSLSxq28ik";

    #[test]
    fn test_generate_key() {
        let key = VapidKeypair::generate_p256().unwrap();
        let pem = key.to_pem().unwrap();
        assert!(pem.starts_with("-----BEGIN EC PRIVATE KEY-----\n"));
        assert!(pem.ends_with("-----END EC PRIVATE KEY-----\n"));
    }

    #[test]
    fn test_pem_roundtrip() {
        let key = VapidKeypair::from_pem(PRIVATE_KEY_PEM).unwrap();
        assert_eq!(key.to_pem().unwrap(), PRIVATE_KEY_PEM);
    }

    #[test]
    fn test_public_key() {
        let key = VapidKeypair::from_pem(PRIVATE_KEY_PEM).unwrap();
        assert_eq!(
            key.public().unwrap().encode_b64().unwrap().0,
            PUBLIC_KEY_B64
        );
    }
}
