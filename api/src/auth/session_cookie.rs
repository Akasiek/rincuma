use axum::http::{HeaderMap, header};
use cookie::{Cookie, SameSite, time::Duration};

use crate::auth::{session::SESSION_DURATION_SECONDS, token::SessionToken};

const SECURE_SESSION_COOKIE_NAME: &str = "__Host-rincuma_session";
const DEVELOPMENT_SESSION_COOKIE_NAME: &str = "rincuma_session";
pub(super) fn session_token(headers: &HeaderMap, secure: bool) -> Option<SessionToken> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(Cookie::split_parse)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name() == cookie_name(secure))
        .and_then(|cookie| SessionToken::parse(cookie.value()))
}

pub(super) fn session_cookie(token: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((cookie_name(secure), token.to_owned()))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(Duration::seconds(SESSION_DURATION_SECONDS))
        .build()
}

pub(super) fn removal_cookie(secure: bool) -> Cookie<'static> {
    let mut cookie = session_cookie("", secure);
    cookie.make_removal();
    cookie
}

const fn cookie_name(secure: bool) -> &'static str {
    if secure {
        SECURE_SESSION_COOKIE_NAME
    } else {
        DEVELOPMENT_SESSION_COOKIE_NAME
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::{session_cookie, session_token};
    use crate::auth::token::SessionToken;

    #[test]
    fn reads_only_the_cookie_for_the_current_security_mode() {
        let Ok(token) = SessionToken::generate() else {
            return;
        };
        let mut headers = HeaderMap::new();
        let Ok(header_value) =
            HeaderValue::from_str(&format!("rincuma_session={}", token.expose()))
        else {
            return;
        };
        headers.insert(header::COOKIE, header_value);

        assert!(session_token(&headers, false).is_some());
        assert!(session_token(&headers, true).is_none());
    }

    #[test]
    fn secure_cookie_uses_host_prefix_and_security_attributes() {
        let cookie = session_cookie("token", true).to_string();

        assert!(cookie.starts_with("__Host-rincuma_session=token"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("Path=/"));
    }
}
