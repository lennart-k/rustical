//! VAPID (RFC 8292) for WebDAV-Push.
//!
//! rustical already encrypts push payloads (`aes128gcm` / RFC 8291, via the `ece`
//! crate). This adds the missing application-server identification half: a
//! persistent NIST P-256 keypair whose public key is advertised to clients (so
//! they can restrict their push subscription to this server via
//! `applicationServerKey`) and whose private key signs every push request with a
//! `vapid` `Authorization` JWT (ES256). The JWT is produced with `jsonwebtoken`;
//! `openssl` (already a dependency) handles keypair generation, PEM persistence,
//! and exporting the raw public-key point clients use as `applicationServerKey`.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use openssl::{
    bn::BigNumContext,
    ec::{EcGroup, EcKey, PointConversionForm},
    nid::Nid,
    pkey::{PKey, Private},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum VapidError {
    #[error(transparent)]
    OpenSsl(#[from] openssl::error::ErrorStack),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

/// VAPID JWT claims (RFC 8292 §2): audience, expiry, and an optional contact.
#[derive(Serialize)]
struct VapidClaims<'a> {
    aud: &'a str,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<&'a str>,
}

/// A persistent VAPID (RFC 8292) application-server keypair (NIST P-256 / ES256).
#[derive(Clone)]
pub struct VapidKeypair {
    key: EcKey<Private>,
}

impl std::fmt::Debug for VapidKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose private key material in logs.
        f.debug_struct("VapidKeypair").finish_non_exhaustive()
    }
}

impl VapidKeypair {
    pub fn generate() -> Result<Self, VapidError> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        Ok(Self {
            key: EcKey::generate(&group)?,
        })
    }

    pub fn from_pem(pem: &[u8]) -> Result<Self, VapidError> {
        Ok(Self {
            key: EcKey::private_key_from_pem(pem)?,
        })
    }

    pub fn to_pem(&self) -> Result<Vec<u8>, VapidError> {
        Ok(self.key.private_key_to_pem()?)
    }

    /// Public key as `base64url(raw uncompressed point)` — the value clients use
    /// as `applicationServerKey` and the `k` parameter of the auth header.
    pub fn public_key_b64url(&self) -> Result<String, VapidError> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let mut ctx = BigNumContext::new()?;
        let bytes =
            self.key
                .public_key()
                .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)?;
        Ok(B64.encode(bytes))
    }

    /// The private key as PKCS#8 PEM, the form `jsonwebtoken`'s `EncodingKey`
    /// expects (persistence uses SEC1 via [`Self::to_pem`]).
    fn signing_key_pem(&self) -> Result<Vec<u8>, VapidError> {
        Ok(PKey::from_ec_key(self.key.clone())?.private_key_to_pem_pkcs8()?)
    }

    /// Build the `vapid t=<jwt>, k=<pubkey>` `Authorization` header value for a
    /// push to `aud` (the push endpoint's origin, e.g. `https://ntfy.example`).
    /// `sub` is the optional VAPID contact; `ttl` bounds the `exp`
    /// (RFC 8292 requires <= 24h).
    pub fn auth_header(
        &self,
        aud: &str,
        sub: Option<&str>,
        ttl: Duration,
    ) -> Result<String, VapidError> {
        let claims = VapidClaims {
            aud,
            exp: (Utc::now() + ttl).timestamp(),
            sub,
        };
        let key = EncodingKey::from_ec_pem(&self.signing_key_pem()?)?;
        let jwt = encode(&Header::new(Algorithm::ES256), &claims, &key)?;
        Ok(format!("vapid t={jwt}, k={}", self.public_key_b64url()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, Validation, decode};

    #[test]
    fn auth_header_is_a_valid_es256_jwt() {
        let kp = VapidKeypair::generate().unwrap();
        let header = kp
            .auth_header(
                "https://push.example",
                Some("mailto:admin@example.com"),
                Duration::hours(12),
            )
            .unwrap();

        // Shape: `vapid t=<jwt>, k=<pub>`.
        let rest = header.strip_prefix("vapid t=").unwrap();
        let (jwt, k) = rest.split_once(", k=").unwrap();
        assert_eq!(k, kp.public_key_b64url().unwrap());

        // The JWT verifies against the keypair's public key (ES256), and the
        // standard claims are present and within RFC 8292's 24h bound.
        let pub_pem = PKey::from_ec_key(kp.key.clone())
            .unwrap()
            .public_key_to_pem()
            .unwrap();
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&["https://push.example"]);
        validation.set_required_spec_claims(&["exp", "aud"]);
        let data = decode::<serde_json::Value>(
            jwt,
            &DecodingKey::from_ec_pem(&pub_pem).unwrap(),
            &validation,
        )
        .unwrap();
        assert_eq!(data.header.alg, Algorithm::ES256);
        assert_eq!(data.claims["sub"], "mailto:admin@example.com");
        let exp = data.claims["exp"].as_i64().unwrap();
        let now = Utc::now().timestamp();
        assert!(exp > now && exp <= now + 24 * 3600);
    }

    #[test]
    fn omits_sub_claim_when_none() {
        let kp = VapidKeypair::generate().unwrap();
        let header = kp
            .auth_header("https://push.example", None, Duration::hours(12))
            .unwrap();
        let jwt = header
            .strip_prefix("vapid t=")
            .unwrap()
            .split_once(", k=")
            .unwrap()
            .0;
        let claims: serde_json::Value =
            serde_json::from_slice(&B64.decode(jwt.split('.').nth(1).unwrap()).unwrap()).unwrap();
        assert_eq!(claims["aud"], "https://push.example");
        assert!(claims.get("sub").is_none());
    }

    #[test]
    fn pem_roundtrip_preserves_public_key() {
        let kp = VapidKeypair::generate().unwrap();
        let pem = kp.to_pem().unwrap();
        let restored = VapidKeypair::from_pem(&pem).unwrap();
        assert_eq!(
            kp.public_key_b64url().unwrap(),
            restored.public_key_b64url().unwrap()
        );
    }
}
