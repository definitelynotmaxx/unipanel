//! Embedded single-page application.
//!
//! The Vite build in `web/dist` is compiled into the binary with `rust-embed`.
//! `index.html` is never served verbatim: the server injects the instance locale
//! and a small boot payload (with a CSP nonce), which is how the frontend knows
//! which language bundle to load before any JavaScript runs.

use axum::body::Body;
use axum::extract::{Extension, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse as _, Response};
use rust_embed::RustEmbed;
use serde::Serialize;

use crate::state::AppState;

/// Assets produced by `npm run build` in `web/`.
#[derive(RustEmbed)]
#[folder = "../../web/dist/"]
struct Assets;

/// Per-request CSP nonce, inserted by [`crate::middleware::security_headers`].
#[derive(Debug, Clone)]
pub struct CspNonce(pub String);

/// Payload exposed to the SPA as `window.__UNIPANEL__`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootPayload {
    locale: String,
    setup_required: bool,
    instance_name: String,
    version: String,
}

/// Serves an asset or the SPA shell.
pub async fn spa(
    State(state): State<AppState>,
    uri: Uri,
    nonce: Option<Extension<CspNonce>>,
) -> Response {
    let path = uri.path().trim_start_matches('/');
    let is_index_request = path.is_empty() || !looks_like_asset(path);

    if is_index_request {
        return serve_index(&state, nonce.as_ref().map(|Extension(nonce)| nonce.0.as_str())).await;
    }

    if path.contains("..") || path.contains('\\') {
        return (StatusCode::BAD_REQUEST, "bad path\n").into_response();
    }

    match Assets::get(path) {
        Some(asset) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache = if path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "public, max-age=3600"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, cache)
                .body(Body::from(asset.data.into_owned()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => (StatusCode::NOT_FOUND, "not found\n").into_response(),
    }
}

/// A path without a file extension is treated as a client-side route.
fn looks_like_asset(path: &str) -> bool {
    path.starts_with("assets/") || path.rsplit('/').next().is_some_and(|last| last.contains('.'))
}

async fn serve_index(state: &AppState, nonce: Option<&str>) -> Response {
    let Some(index) = Assets::get("index.html") else {
        return missing_frontend();
    };
    let Ok(template) = std::str::from_utf8(index.data.as_ref()) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "invalid index.html\n").into_response();
    };

    let setup_required = state.db.is_setup_complete().await.map(|done| !done).unwrap_or(false);
    let locale = state
        .instance_locale()
        .await
        .unwrap_or_default();
    let instance_name = state
        .db
        .instance_name()
        .await
        .unwrap_or_else(|_| unipanel_core::brand().display_name.to_owned());

    let payload = BootPayload {
        locale: locale.as_str().to_owned(),
        setup_required,
        instance_name,
        version: unipanel_core::version().to_owned(),
    };
    // `</script>` inside a JSON string would end the bootstrap script early.
    let serialised = serde_json::to_string(&payload)
        .unwrap_or_else(|_| "{}".to_owned())
        .replace('<', "\\u003c");

    let html = render_index(template, locale.as_str(), &serialised, nonce);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(html))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Injects locale and boot payload into the Vite HTML shell.
#[must_use]
pub fn render_index(template: &str, locale: &str, boot_json: &str, nonce: Option<&str>) -> String {
    let html = template
        .replacen("<html lang=\"en\"", &format!("<html lang=\"{locale}\""), 1)
        .replacen("<html lang='en'", &format!("<html lang='{locale}'"), 1);

    let nonce_attribute = nonce
        .map(|nonce| format!(" nonce=\"{nonce}\""))
        .unwrap_or_default();
    let script = format!("<script{nonce_attribute}>window.__UNIPANEL__={boot_json};</script>");

    match html.find("</head>") {
        Some(index) => {
            let mut out = String::with_capacity(html.len() + script.len());
            out.push_str(&html[..index]);
            out.push_str(&script);
            out.push_str(&html[index..]);
            out
        }
        None => format!("{html}{script}"),
    }
}

fn missing_frontend() -> Response {
    let body = if cfg!(debug_assertions) {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>UniPanel</title></head>
         <body style=\"font-family:ui-monospace,monospace;background:#0b0c0b;color:#e8e4da;padding:3rem\">
         <h1>Frontend not built</h1>
         <p>Run <code>npm ci &amp;&amp; npm run build</code> inside <code>web/</code>, then reload.</p>
         <p>For live development use the Vite dev server on port 5173; it proxies <code>/api</code> to this server.</p>
         </body></html>"
    } else {
        "frontend assets missing from this build\n"
    };
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>x</title></head><body><div id=\"root\"></div></body></html>";

    #[test]
    fn injects_locale_and_boot_payload_before_head_end() {
        let html = render_index(TEMPLATE, "de", r#"{"locale":"de"}"#, Some("abc123"));
        assert!(html.contains("<html lang=\"de\">"));
        assert!(html.contains("nonce=\"abc123\""));
        assert!(html.contains("window.__UNIPANEL__={\"locale\":\"de\"};"));
        let head_end = html.find("</head>").expect("head");
        let script = html.find("window.__UNIPANEL__").expect("script");
        assert!(script < head_end);
    }

    #[test]
    fn works_without_a_nonce() {
        let html = render_index(TEMPLATE, "en", "{}", None);
        assert!(html.contains("<script>window.__UNIPANEL__={};</script>"));
    }

    #[test]
    fn still_renders_when_the_shell_has_no_head() {
        let html = render_index("<div></div>", "en", "{}", None);
        assert!(html.ends_with("</script>"));
    }

    #[test]
    fn distinguishes_routes_from_assets() {
        assert!(!looks_like_asset("setup"));
        assert!(!looks_like_asset("resources/abc"));
        assert!(looks_like_asset("assets/index-abc.js"));
        assert!(looks_like_asset("favicon.svg"));
        assert!(looks_like_asset("robots.txt"));
    }
}
