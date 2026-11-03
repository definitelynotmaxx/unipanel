//! Minimal cookie helpers.
//!
//! `axum-extra` ships a full cookie jar, but reading one value and writing two
//! cookies is small enough to keep local, and it avoids pulling the `cookie`
//! crate's parsing rules into the security-critical path.

use axum::http::HeaderMap;

/// One parsed cookie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Decoded-ish value (percent decoding is not needed for our URL-safe tokens).
    pub value: String,
}

/// Parses the `Cookie` header into name/value pairs.
#[must_use]
pub fn parse_cookies(headers: &HeaderMap) -> Vec<Cookie> {
    let mut cookies = Vec::new();
    for header in headers.get_all(axum::http::header::COOKIE) {
        let Ok(raw) = header.to_str() else {
            continue;
        };
        for pair in raw.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            cookies.push(Cookie {
                name: name.trim().to_owned(),
                value: value.trim().to_owned(),
            });
        }
    }
    cookies
}

/// Reads a cookie by name.
#[must_use]
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    parse_cookies(headers)
        .into_iter()
        .find(|cookie| cookie.name == name)
        .map(|cookie| cookie.value)
        .filter(|value| !value.is_empty())
}

/// Options for [`set_cookie`].
#[derive(Debug, Clone, Copy)]
pub struct CookieOptions {
    /// Adds `HttpOnly` (session cookies must set this).
    pub http_only: bool,
    /// Adds `Secure`.
    pub secure: bool,
    /// Lifetime in seconds; `None` makes it a session cookie.
    pub max_age_seconds: Option<i64>,
}

impl CookieOptions {
    /// An `HttpOnly`, `SameSite=Lax` session cookie.
    #[must_use]
    pub const fn session(secure: bool, max_age_seconds: i64) -> Self {
        Self {
            http_only: true,
            secure,
            max_age_seconds: Some(max_age_seconds),
        }
    }

    /// A cookie that is cleared immediately.
    #[must_use]
    pub const fn cleared(secure: bool) -> Self {
        Self {
            http_only: true,
            secure,
            max_age_seconds: Some(0),
        }
    }
}

/// Builds a `Set-Cookie` header value.
#[must_use]
pub fn set_cookie(name: &str, value: &str, options: CookieOptions) -> String {
    let mut cookie = format!("{name}={value}; Path=/; SameSite=Lax");
    if options.http_only {
        cookie.push_str("; HttpOnly");
    }
    if options.secure {
        cookie.push_str("; Secure");
    }
    if let Some(max_age) = options.max_age_seconds {
        cookie.push_str(&format!("; Max-Age={max_age}"));
    }
    cookie
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_str(cookie).expect("header"),
        );
        headers
    }

    #[test]
    fn parses_multiple_cookies_and_tolerates_junk() {
        let cookies = parse_cookies(&headers("a=1; b=2;; c=; =d; e=3=4"));
        assert_eq!(
            cookies
                .iter()
                .find(|c| c.name == "a")
                .map(|c| c.value.as_str()),
            Some("1")
        );
        assert_eq!(cookie_value(&headers("x=1; y=2"), "y").as_deref(), Some("2"));
        assert_eq!(cookie_value(&headers("x=1"), "y"), None);
    }

    #[test]
    fn builds_a_hardened_cookie() {
        let cookie = set_cookie("s", "v", CookieOptions::session(true, 60));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Max-Age=60"));
    }
}
