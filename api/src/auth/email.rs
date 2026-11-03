const MAX_EMAIL_LENGTH: usize = 254;

/// Returns the canonical form of an email address, or `None` when it is invalid.
pub(crate) fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;

    let invalid = email.len() > MAX_EMAIL_LENGTH
        || local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || email.chars().any(char::is_whitespace);

    (!invalid).then_some(email)
}
