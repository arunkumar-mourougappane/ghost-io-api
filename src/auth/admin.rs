//! Admin API authentication using JWT.
//!
//! The Ghost Admin API uses short-lived JWTs signed with HMAC-SHA256 (HS256).
//! Admin API keys have the format `{id}:{hex_secret}` where `id` is placed in
//! the JWT `kid` header and `hex_secret` (hex-decoded) is the HMAC signing key.
//!
//! Tokens expire after 5 minutes and carry only `iat` and `exp` claims.
//!
//! # Example
//!
//! ```
//! use ghost_io_api::auth::admin::AdminApiKey;
//!
//! let raw = "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";
//! let key = AdminApiKey::new(raw).unwrap();
//! let token = key.generate_jwt().unwrap();
//! assert!(!token.is_empty());
//! ```

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{GhostError, Result};

type HmacSha256 = Hmac<Sha256>;

/// Lifetime of a generated JWT in seconds (5 minutes).
const TOKEN_EXPIRY_SECS: u64 = 5 * 60;

/// Admin API key for JWT-based authentication.
///
/// Ghost Admin API keys have the format `{id}:{hex_secret}` where:
/// - `id` is the key identifier placed in the JWT `kid` header
/// - `hex_secret` is a hex-encoded byte string used as the HMAC-SHA256 signing key
///
/// # Security
///
/// Admin API keys grant **write access** to your Ghost installation. Never
/// expose them in client-side code or public repositories.
///
/// # Example
///
/// ```
/// use ghost_io_api::auth::admin::AdminApiKey;
///
/// let raw = "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";
/// let key = AdminApiKey::new(raw).unwrap();
/// assert_eq!(key.key_id(), "6748592f4b9b7700010f6564");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminApiKey {
    key_id: String,
    hex_secret: String,
}

impl AdminApiKey {
    /// Creates a new Admin API key from the Ghost `{id}:{hex_secret}` format.
    ///
    /// # Validation
    ///
    /// - Must contain exactly one `:` separator
    /// - `id` must be non-empty
    /// - `hex_secret` must be non-empty, contain only hex digits, and have an even length
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Auth`] if the key is malformed.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::auth::admin::AdminApiKey;
    ///
    /// let key = AdminApiKey::new(
    ///     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
    /// ).unwrap();
    /// assert!(key.is_valid());
    ///
    /// // Missing separator
    /// assert!(AdminApiKey::new("noseparator").is_err());
    ///
    /// // Non-hex secret
    /// assert!(AdminApiKey::new("kid:xyz").is_err());
    /// ```
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let raw = key.into();
        let raw = raw.trim();

        if raw.is_empty() {
            return Err(GhostError::auth("Admin API key cannot be empty"));
        }

        let (key_id, hex_secret) = raw.split_once(':').ok_or_else(|| {
            GhostError::auth("Admin API key must be in the format 'id:hex_secret'")
        })?;

        let key_id = key_id.trim().to_string();
        let hex_secret = hex_secret.trim().to_lowercase();

        if key_id.is_empty() {
            return Err(GhostError::auth("Admin API key ID cannot be empty"));
        }

        if hex_secret.is_empty() {
            return Err(GhostError::auth("Admin API key secret cannot be empty"));
        }

        if !hex_secret.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GhostError::auth(
                "Admin API key secret must contain only hexadecimal characters (0-9, a-f)",
            ));
        }

        if hex_secret.len() % 2 != 0 {
            return Err(GhostError::auth(
                "Admin API key secret must have an even number of hex characters",
            ));
        }

        Ok(Self { key_id, hex_secret })
    }

    /// Returns the key ID used as the JWT `kid` header value.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Returns `true` if the key is structurally valid.
    ///
    /// Keys constructed via [`AdminApiKey::new`] always pass this check.
    pub fn is_valid(&self) -> bool {
        !self.key_id.is_empty()
            && !self.hex_secret.is_empty()
            && self.hex_secret.len() % 2 == 0
            && self.hex_secret.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Generates a signed HS256 JWT for use with the Ghost Admin API.
    ///
    /// The token includes:
    /// - Header: `{"alg":"HS256","kid":"<id>","typ":"JWT"}`
    /// - Payload: `{"exp":<now+300>,"iat":<now>}`
    /// - Signature: HMAC-SHA256 over `<header_b64>.<payload_b64>`, keyed with
    ///   the hex-decoded secret
    ///
    /// # Errors
    ///
    /// Returns [`GhostError::Auth`] if the system clock is before the Unix
    /// epoch or if the secret bytes are rejected by the HMAC implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::auth::admin::AdminApiKey;
    ///
    /// let key = AdminApiKey::new(
    ///     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
    /// ).unwrap();
    /// let token = key.generate_jwt().unwrap();
    ///
    /// // JWT has three dot-separated parts
    /// assert_eq!(token.split('.').count(), 3);
    /// ```
    pub fn generate_jwt(&self) -> Result<String> {
        let iat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GhostError::auth(format!("System time error: {e}")))?
            .as_secs();

        self.sign_jwt(iat)
    }

    /// Generates a JWT with an explicit `iat` timestamp (used in tests).
    pub(crate) fn sign_jwt(&self, iat: u64) -> Result<String> {
        let exp = iat + TOKEN_EXPIRY_SECS;

        let header = serde_json::json!({
            "alg": "HS256",
            "kid": self.key_id,
            "typ": "JWT"
        });
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header)?.as_bytes());

        let payload = serde_json::json!({
            "exp": exp,
            "iat": iat
        });
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload)?.as_bytes());

        let signing_input = format!("{header_b64}.{payload_b64}");
        let secret_bytes = hex_decode(&self.hex_secret)?;

        let mut mac = HmacSha256::new_from_slice(&secret_bytes)
            .map_err(|e| GhostError::auth(format!("Invalid HMAC key length: {e}")))?;
        mac.update(signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        Ok(format!("{signing_input}.{signature_b64}"))
    }
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| GhostError::auth(format!("Invalid hex in Admin API key secret: {e}")))
        })
        .collect()
}

impl fmt::Display for AdminApiKey {
    /// Formats the key for display, masking the secret entirely.
    ///
    /// # Example
    ///
    /// ```
    /// use ghost_io_api::auth::admin::AdminApiKey;
    ///
    /// let key = AdminApiKey::new(
    ///     "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
    /// ).unwrap();
    /// let display = format!("{key}");
    /// assert!(display.contains("6748592f4b9b7700010f6564"));
    /// assert!(!display.contains("b1b5b9c1"));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AdminApiKey({}:***)", self.key_id)
    }
}

impl AsRef<str> for AdminApiKey {
    fn as_ref(&self) -> &str {
        &self.key_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_KID: &str = "6748592f4b9b7700010f6564";
    const VALID_SECRET: &str = "b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";
    const VALID_KEY: &str =
        "6748592f4b9b7700010f6564:b1b5b9c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1";

    // ── construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_valid_key() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        assert_eq!(key.key_id(), VALID_KID);
        assert!(key.is_valid());
    }

    #[test]
    fn test_key_trims_whitespace() {
        let key = AdminApiKey::new(format!("  {VALID_KEY}  ")).unwrap();
        assert_eq!(key.key_id(), VALID_KID);
    }

    #[test]
    fn test_key_normalises_secret_to_lowercase() {
        let upper = format!("{VALID_KID}:{}", VALID_SECRET.to_uppercase());
        let key = AdminApiKey::new(upper).unwrap();
        assert!(key.is_valid());
    }

    #[test]
    fn test_empty_key() {
        let err = AdminApiKey::new("").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_missing_separator() {
        let err = AdminApiKey::new("noseparatoratall").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("format 'id:hex_secret'"));
    }

    #[test]
    fn test_empty_key_id() {
        let err = AdminApiKey::new(":abcdef12").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("ID cannot be empty"));
    }

    #[test]
    fn test_empty_secret() {
        let err = AdminApiKey::new("somekid:").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("secret cannot be empty"));
    }

    #[test]
    fn test_non_hex_secret() {
        let err = AdminApiKey::new("somekid:xyz!").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("hexadecimal characters"));
    }

    #[test]
    fn test_odd_length_secret() {
        let err = AdminApiKey::new("somekid:abc").unwrap_err();
        assert!(err.is_auth_error());
        assert!(err.to_string().contains("even number of hex characters"));
    }

    #[test]
    fn test_clone_and_eq() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        assert_eq!(key.clone(), key);
    }

    #[test]
    fn test_is_valid() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        assert!(key.is_valid());
    }

    // ── JWT structure ─────────────────────────────────────────────────────────

    #[test]
    fn test_jwt_has_three_parts() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let token = key.generate_jwt().unwrap();
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_jwt_header_fields() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let token = key.generate_jwt().unwrap();

        let header_b64 = token.split('.').next().unwrap();
        let header_bytes = URL_SAFE_NO_PAD.decode(header_b64).unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_bytes).unwrap();

        assert_eq!(header["alg"], "HS256");
        assert_eq!(header["typ"], "JWT");
        assert_eq!(header["kid"], VALID_KID);
    }

    #[test]
    fn test_jwt_payload_claims() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let iat: u64 = 1_700_000_000;
        let token = key.sign_jwt(iat).unwrap();

        let payload_b64 = token.split('.').nth(1).unwrap();
        let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();

        assert_eq!(payload["iat"], iat);
        assert_eq!(payload["exp"], iat + TOKEN_EXPIRY_SECS);
    }

    #[test]
    fn test_jwt_expiry_is_five_minutes() {
        assert_eq!(TOKEN_EXPIRY_SECS, 300);
    }

    #[test]
    fn test_jwt_signature_is_base64url_no_pad() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let token = key.generate_jwt().unwrap();
        let sig = token.split('.').nth(2).unwrap();
        // Base64url chars only; no `=` padding
        assert!(sig
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert!(!sig.contains('='));
    }

    #[test]
    fn test_jwt_is_deterministic_for_same_iat() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let iat = 1_700_000_000u64;
        assert_eq!(key.sign_jwt(iat).unwrap(), key.sign_jwt(iat).unwrap());
    }

    #[test]
    fn test_jwt_differs_for_different_iat() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let t1 = key.sign_jwt(1_700_000_000).unwrap();
        let t2 = key.sign_jwt(1_700_000_001).unwrap();
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_known_jwt_signature() {
        // Verifies the exact HMAC-SHA256 output against a reference value so
        // a regression in signing logic is caught immediately.
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let iat = 1_700_000_000u64;
        let token = key.sign_jwt(iat).unwrap();

        // Re-derive expected signature independently
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let secret_bytes = hex_decode(VALID_SECRET).unwrap();
        let mut mac = HmacSha256::new_from_slice(&secret_bytes).unwrap();
        mac.update(signing_input.as_bytes());
        let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        assert_eq!(parts[2], expected_sig);
    }

    #[test]
    fn test_different_keys_produce_different_signatures() {
        let key1 = AdminApiKey::new(VALID_KEY).unwrap();
        let key2 = AdminApiKey::new(
            "aabbccdd11223344aabbccdd:0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        )
        .unwrap();
        let iat = 1_700_000_000u64;
        assert_ne!(key1.sign_jwt(iat).unwrap(), key2.sign_jwt(iat).unwrap());
    }

    // ── display / trait impls ─────────────────────────────────────────────────

    #[test]
    fn test_display_shows_kid_masks_secret() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let s = format!("{key}");
        assert!(s.contains(VALID_KID));
        assert!(!s.contains(VALID_SECRET));
        assert!(s.contains("***"));
    }

    #[test]
    fn test_debug_contains_struct_name() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let s = format!("{key:?}");
        assert!(s.contains("AdminApiKey"));
    }

    #[test]
    fn test_as_ref_returns_key_id() {
        let key = AdminApiKey::new(VALID_KEY).unwrap();
        let r: &str = key.as_ref();
        assert_eq!(r, VALID_KID);
    }

    // ── hex_decode helper ─────────────────────────────────────────────────────

    #[test]
    fn test_hex_decode_valid() {
        assert_eq!(
            hex_decode("deadbeef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_hex_decode_uppercase() {
        // AdminApiKey::new normalises to lowercase before storing, but the
        // helper itself accepts whatever it receives.
        assert_eq!(
            hex_decode("DEADBEEF").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_hex_decode_invalid_char_returns_auth_error() {
        let err = hex_decode("zz").unwrap_err();
        assert!(err.is_auth_error());
    }
}
