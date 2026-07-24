use ct_codecs::Encoder;
use openssl::{
    bn::BigNumContext,
    ec::{EcKey, PointConversionForm},
    error::ErrorStack,
    pkey::Private,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VapidError {
    #[error(transparent)]
    OpenSslError(#[from] ErrorStack),
    #[error(transparent)]
    EncodingError(#[from] ct_codecs::Error),
}

#[derive(Debug, Clone)]
pub struct VapidKeypair(pub openssl::ec::EcKey<Private>);
pub struct VapidPublicKey(pub openssl::ec::EcPoint);
#[derive(Debug, Clone, Deserialize)]
pub struct VapidPublicKeyB64(pub String);

impl VapidKeypair {
    pub fn generate_p256() -> Result<Self, VapidError> {
        let group = openssl::ec::EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1)?;
        Ok(Self(EcKey::generate(&group)?))
    }
    pub fn public(&self) -> Result<VapidPublicKey, VapidError> {
        let group = openssl::ec::EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1)?;
        Ok(VapidPublicKey(self.0.public_key().to_owned(&group)?))
    }
}

impl VapidPublicKey {
    pub fn encode(&self) -> Result<VapidPublicKeyB64, VapidError> {
        let group =
            openssl::ec::EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1).unwrap();
        let mut ctx = BigNumContext::new().unwrap();
        let bytes = self
            .0
            .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)
            .unwrap();
        Ok(VapidPublicKeyB64(
            ct_codecs::Base64UrlSafeNoPadding::encode_to_string(bytes)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::vapid::VapidKeypair;

    #[test]
    fn test_generate_key() {
        let key = VapidKeypair::generate_p256().unwrap();
        dbg!(key.public().unwrap().encode().unwrap());
        panic!()
    }
}
