//! VAPID (RFC 8292) for WebDAV-Push.
//!
//! rustical already encrypts push payloads (`aes128gcm` / RFC 8291, via the `ece`
//! crate). This adds the missing application-server identification half: a
//! persistent NIST P-256 keypair whose public key is advertised to clients (so
//! they can restrict their push subscription to this server via
//! `applicationServerKey`) and whose private key signs every push request with a
//! `vapid` `Authorization` JWT (ES256). Implemented with `openssl` (already a
//! dependency) — `EcdsaSig` yields the raw `r‖s` the JWS signature needs.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use chrono::{Duration, Utc};
use openssl::{
    bn::BigNumContext,
    ec::{EcGroup, EcKey, PointConversionForm},
    ecdsa::EcdsaSig,
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    sign::Signer,
};

#[derive(Debug, thiserror::Error)]
pub enum VapidError {
    #[error(transparent)]
    OpenSsl(#[from] openssl::error::ErrorStack),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// The server's VAPID public key (base64url), set once at startup. The key is a
/// single process-wide value, so this avoids threading it through every DAV
/// resource just to advertise it in `propfind` responses.
static VAPID_PUBLIC_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Record the server's VAPID public key for advertisement. First value wins.
pub fn set_vapid_public_key(key: String) {
    let _ = VAPID_PUBLIC_KEY.set(key);
}

/// The server's VAPID public key (base64url), if one has been set.
#[must_use]
pub fn vapid_public_key() -> Option<String> {
    VAPID_PUBLIC_KEY.get().cloned()
}

fn p256_group() -> Result<EcGroup, openssl::error::ErrorStack> {
    EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)
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
    /// Generate a fresh keypair.
    pub fn generate() -> Result<Self, VapidError> {
        let group = p256_group()?;
        Ok(Self {
            key: EcKey::generate(&group)?,
        })
    }

    /// Restore from the PEM produced by [`Self::to_pem`].
    pub fn from_pem(pem: &[u8]) -> Result<Self, VapidError> {
        Ok(Self {
            key: EcKey::private_key_from_pem(pem)?,
        })
    }

    /// Serialize the private key as PEM (for persistence).
    pub fn to_pem(&self) -> Result<Vec<u8>, VapidError> {
        Ok(self.key.private_key_to_pem()?)
    }

    /// Public key as `base64url(raw uncompressed point)` — the value clients use
    /// as `applicationServerKey` and the `k` parameter of the auth header.
    pub fn public_key_b64url(&self) -> Result<String, VapidError> {
        let group = p256_group()?;
        let mut ctx = BigNumContext::new()?;
        let bytes = self.key.public_key().to_bytes(
            &group,
            PointConversionForm::UNCOMPRESSED,
            &mut ctx,
        )?;
        Ok(B64.encode(bytes))
    }

    /// Build the `vapid t=<jwt>, k=<pubkey>` `Authorization` header value for a
    /// push to `aud` (the push endpoint's origin, e.g. `https://ntfy.example`).
    /// `sub` is the VAPID contact (a `mailto:` or `https:` URL); `ttl` bounds the
    /// JWT `exp` (RFC 8292 requires ≤ 24h).
    pub fn auth_header(&self, aud: &str, sub: &str, ttl: Duration) -> Result<String, VapidError> {
        let header = B64.encode(br#"{"typ":"JWT","alg":"ES256"}"#);
        let exp = (Utc::now() + ttl).timestamp();
        let claims = B64.encode(serde_json::to_vec(&serde_json::json!({
            "aud": aud,
            "exp": exp,
            "sub": sub,
        }))?);
        let signing_input = format!("{header}.{claims}");

        let pkey = PKey::from_ec_key(self.key.clone())?;
        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;
        signer.update(signing_input.as_bytes())?;
        let der = signer.sign_to_vec()?;

        // JWS/ES256 wants the fixed-size raw signature `r‖s` (32 bytes each,
        // left-padded), not openssl's variable-length DER encoding.
        let ecdsa = EcdsaSig::from_der(&der)?;
        let r = ecdsa.r().to_vec();
        let s = ecdsa.s().to_vec();
        let mut raw = [0u8; 64];
        raw[32 - r.len()..32].copy_from_slice(&r);
        raw[64 - s.len()..64].copy_from_slice(&s);
        let signature = B64.encode(raw);

        let jwt = format!("{signing_input}.{signature}");
        Ok(format!("vapid t={jwt}, k={}", self.public_key_b64url()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::{bn::BigNum, sign::Verifier};

    #[test]
    fn auth_header_is_a_valid_es256_jwt() {
        let kp = VapidKeypair::generate().unwrap();
        let header = kp
            .auth_header("https://push.example", "mailto:admin@example.com", Duration::hours(12))
            .unwrap();

        // Shape: `vapid t=<h>.<c>.<s>, k=<pub>`.
        let rest = header.strip_prefix("vapid t=").unwrap();
        let (jwt, k) = rest.split_once(", k=").unwrap();
        assert_eq!(k, kp.public_key_b64url().unwrap());

        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Header: ES256.
        let hdr = B64.decode(parts[0]).unwrap();
        assert!(std::str::from_utf8(&hdr).unwrap().contains("ES256"));

        // Claims: aud/sub/exp, with exp in the future and within 24h.
        let claims: serde_json::Value =
            serde_json::from_slice(&B64.decode(parts[1]).unwrap()).unwrap();
        assert_eq!(claims["aud"], "https://push.example");
        assert_eq!(claims["sub"], "mailto:admin@example.com");
        let exp = claims["exp"].as_i64().unwrap();
        let now = Utc::now().timestamp();
        assert!(exp > now && exp <= now + 24 * 3600);

        // Signature: 64-byte raw r‖s that verifies against the public key.
        let sig_raw = B64.decode(parts[2]).unwrap();
        assert_eq!(sig_raw.len(), 64);
        let ecdsa = EcdsaSig::from_private_components(
            BigNum::from_slice(&sig_raw[..32]).unwrap(),
            BigNum::from_slice(&sig_raw[32..]).unwrap(),
        )
        .unwrap();
        let pkey = PKey::from_ec_key(kp.key.clone()).unwrap();
        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
        verifier
            .update(format!("{}.{}", parts[0], parts[1]).as_bytes())
            .unwrap();
        assert!(verifier.verify(&ecdsa.to_der().unwrap()).unwrap());
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
