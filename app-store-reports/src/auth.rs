use crate::error::Error;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const AUDIENCE: &str = "appstoreconnect-v1";
const MAX_TOKEN_LIFETIME: Duration = Duration::from_secs(20 * 60);

/// App Store Connect API credentials: an Issuer ID, a Key ID, and the EC
/// private key downloaded as a `.p8` file from App Store Connect.
pub struct Credentials {
    issuer_id: String,
    key_id: String,
    encoding_key: EncodingKey,
}

impl Credentials {
    /// Reads and parses the `.p8` file as an EC private key immediately, so a
    /// bad key fails fast with one clear error instead of failing deep inside
    /// a download loop.
    pub fn load(
        issuer_id: impl Into<String>,
        key_id: impl Into<String>,
        private_key_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let path: &Path = private_key_path.as_ref();
        let pem = std::fs::read(path).map_err(|source| Error::ReadPrivateKey {
            path: path.to_path_buf(),
            source,
        })?;
        let encoding_key =
            EncodingKey::from_ec_pem(&pem).map_err(|source| Error::InvalidPrivateKey {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(Self {
            issuer_id: issuer_id.into(),
            key_id: key_id.into(),
            encoding_key,
        })
    }

    pub(crate) fn generate_token(&self) -> Result<String, Error> {
        generate_token(
            &self.issuer_id,
            &self.key_id,
            &self.encoding_key,
            MAX_TOKEN_LIFETIME,
            SystemTime::now(),
        )
    }
}

#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    iat: u64,
    exp: u64,
    aud: &'a str,
}

/// `now` and `lifetime` are explicit parameters (rather than reading the clock
/// internally) so unit tests can pin them instead of depending on wall-clock time.
fn generate_token(
    issuer_id: &str,
    key_id: &str,
    encoding_key: &EncodingKey,
    lifetime: Duration,
    now: SystemTime,
) -> Result<String, Error> {
    let lifetime = lifetime.min(MAX_TOKEN_LIFETIME);
    let now_secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let claims = Claims {
        iss: issuer_id,
        iat: now_secs,
        exp: now_secs + lifetime.as_secs(),
        aud: AUDIENCE,
    };
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id.to_string());
    jsonwebtoken::encode(&header, &claims, encoding_key).map_err(Error::SignToken)
}
