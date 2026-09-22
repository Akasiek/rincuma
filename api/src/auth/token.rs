use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use thiserror::Error;

const TOKEN_LENGTH: usize = 32;

pub(crate) struct SessionToken(String);

#[derive(Debug, Error)]
#[error("failed to generate a session token")]
pub(crate) struct TokenGenerationError;

impl SessionToken {
    pub(crate) fn generate() -> Result<Self, TokenGenerationError> {
        let mut bytes = [0_u8; TOKEN_LENGTH];
        getrandom::fill(&mut bytes).map_err(|_| TokenGenerationError)?;

        Ok(Self(URL_SAFE_NO_PAD.encode(bytes)))
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        let decoded = URL_SAFE_NO_PAD.decode(value).ok()?;
        (decoded.len() == TOKEN_LENGTH).then(|| Self(value.to_owned()))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }

    pub(crate) fn digest(&self) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(self.0.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::SessionToken;

    #[test]
    fn generates_url_safe_token_and_stable_digest() {
        let token_result = SessionToken::generate();
        assert!(token_result.is_ok());
        let Ok(token) = token_result else {
            return;
        };
        let parsed_digest = SessionToken::parse(token.expose()).map(|token| token.digest());

        assert_eq!(token.expose().len(), 43);
        assert_eq!(Some(token.digest()), parsed_digest);
        assert_ne!(token.expose(), token.digest());
    }

    #[test]
    fn rejects_malformed_token() {
        assert!(SessionToken::parse("not a session token").is_none());
        assert!(SessionToken::parse("c2hvcnQ").is_none());
    }
}
