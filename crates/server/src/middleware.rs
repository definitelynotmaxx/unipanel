//! Security-relevant response headers and request correlation.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::state::AppState;

/// Header carrying the per-request correlation id.
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Adds a request id, available to handlers as an extension.
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let incoming = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| (8..=64).contains(&value.len()))
        .map(ToOwned::to_owned);

    let id = incoming.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    request
        .extensions_mut()
        .insert(crate::extract::RequestId(id.clone()));

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}

/// Applies the Content-Security-Policy and the other hardening headers.
///
/// The CSP nonce is generated here and exposed to the SPA renderer through a
/// request extension, so the boot payload is the only inline script allowed.
pub async fn security_headers(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let nonce = unipanel_auth::token::random_token();
    request
        .extensions_mut()
        .insert(crate::assets::CspNonce(nonce.clone()));

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    let csp = format!(
        "default-src 'self'; \
         script-src 'self' 'nonce-{nonce}'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; \
         font-src 'self'; \
         connect-src 'self'; \
         frame-ancestors 'none'; \
         base-uri 'none'; \
         form-action 'self'; \
         object-src 'none'"
    );
    let mut set = |name: HeaderName, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    };

    set(header::CONTENT_SECURITY_POLICY, &csp);
    set(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(header::X_FRAME_OPTIONS, "DENY");
    set(header::REFERRER_POLICY, "no-referrer");
    set(header::X_XSS_PROTECTION, "0");
    set(
        HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=(), payment=()",
    );
    set(
        HeaderName::from_static("cross-origin-opener-policy"),
        "same-origin",
    );
    if state.secure_cookies() {
        set(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains",
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::routing::get;
    use http_body_util::BodyExt as _;
    use tower::ServiceExt as _;

    async fn app() -> Router {
        let db = unipanel_storage::Db::connect_in_memory().await.expect("db");
        db.migrate().await.expect("migrate");
        let mut config = crate::test_support::config();
        config.secure_cookies = true;
        let state = AppState::new(db, config);
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                security_headers,
            ))
            .layer(axum::middleware::from_fn(request_id))
            .with_state(state)
    }

    #[tokio::test]
    async fn sets_hardening_headers_and_a_request_id() {
        let response = app()
            .await
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), 200);
        let headers = response.headers();
        assert!(headers.contains_key(&REQUEST_ID_HEADER));
        assert!(headers.get(header::CONTENT_SECURITY_POLICY).is_some());
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).expect("header"),
            "nosniff"
        );
        assert_eq!(
            headers.get(header::STRICT_TRANSPORT_SECURITY).expect("hsts"),
            "max-age=31536000; includeSubDomains"
        );
        let csp = headers
            .get(header::CONTENT_SECURITY_POLICY)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert!(csp.contains("frame-ancestors 'none'"));
        assert!(csp.contains("script-src 'self' 'nonce-"));

        // Body drains cleanly (no hanging streams).
        let _ = response.into_body().collect().await.expect("body");
    }
}
