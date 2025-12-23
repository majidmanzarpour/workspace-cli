use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Decode base64url-encoded data (used by Gmail API for email bodies)
/// Handles both padded and unpadded input by stripping any trailing '=' characters
pub fn decode_base64url(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    // Remove any padding if present (Gmail API uses unpadded, but be defensive)
    let cleaned = input.trim_end_matches('=');
    URL_SAFE_NO_PAD.decode(cleaned)
}

/// Decode base64url to UTF-8 string
pub fn decode_base64url_string(input: &str) -> Result<String, Base64DecodeError> {
    let bytes = decode_base64url(input)?;
    String::from_utf8(bytes).map_err(Base64DecodeError::Utf8)
}

/// Encode data as base64url (for sending emails)
pub fn encode_base64url(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Encode string as base64url
pub fn encode_base64url_string(input: &str) -> String {
    encode_base64url(input.as_bytes())
}

#[derive(Debug)]
pub enum Base64DecodeError {
    Base64(base64::DecodeError),
    Utf8(std::string::FromUtf8Error),
}

impl std::fmt::Display for Base64DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64(e) => write!(f, "Base64 decode error: {}", e),
            Self::Utf8(e) => write!(f, "UTF-8 decode error: {}", e),
        }
    }
}

impl std::error::Error for Base64DecodeError {}

impl From<base64::DecodeError> for Base64DecodeError {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}
