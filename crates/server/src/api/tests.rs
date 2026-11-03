//! End-to-end API tests against a real router and an in-memory database.

use std::collections::HashMap;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

use crate::app::build_app;
use crate::test_support;

/// A tiny HTTP client that keeps cookies and the CSRF token between requests.
struct Client {
    app: Router,
    cookies: HashMap<String, String>,
    csrf: String,
}

impl Client {
    fn new(app: Router) -> Self {
        Self {
            app,
            cookies: HashMap::new(),
            csrf: String::new(),
        }
    }

    fn cookie_header(&self) -> String {
        self.cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    async fn send(
        &mut self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        let cookie = self.cookie_header();
        if !cookie.is_empty() {
            builder = builder.header(header::COOKIE, cookie);
        }
        if !self.csrf.is_empty() {
            builder = builder.header("x-csrf-token", &self.csrf);
        }
        let body = match body {
            Some(value) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(&value).expect("serialise"))
            }
            None => Body::empty(),
        };

        let response = self
            .app
            .clone()
            .oneshot(builder.body(body).expect("request"))
            .await
            .expect("response");

        let status = response.status();
        let mut new_cookies = Vec::new();
        for value in response.headers().get_all(header::SET_COOKIE) {
            if let Ok(raw) = value.to_str() {
                new_cookies.push(raw.to_owned());
            }
        }
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };

        for raw in new_cookies {
            let pair = raw.split(';').next().unwrap_or_default();
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            if value.is_empty() {
                self.cookies.remove(name);
            } else {
                self.cookies.insert(name.to_owned(), value.to_owned());
            }
        }
        (status, json)
    }

    async fn get(&mut self, uri: &str) -> (StatusCode, Value) {
        self.send(Method::GET, uri, None).await
    }

    async fn post(&mut self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.send(Method::POST, uri, Some(body)).await
    }

    async fn bootstrap(&mut self) -> Value {
        let (status, body) = self.get("/api/v1/bootstrap").await;
        assert_eq!(status, StatusCode::OK, "bootstrap failed: {body}");
        if let Some(token) = body["csrf_token"].as_str() {
            self.csrf = token.to_owned();
        }
        body
    }
}

fn setup_payload(locale: &str, password: &str) -> Value {
    json!({
        "locale": locale,
        "username": "Admin",
        "display_name": "Ada Admin",
        "password": password,
        "instance_name": "Homelab"
    })
}

#[tokio::test]
async fn bootstrap_before_setup_reports_the_wizard() {
    let state = test_support::state().await;
    let mut client = Client::new(build_app(state));

    let body = client.bootstrap().await;
    assert_eq!(body["setup_required"], true);
    assert_eq!(body["authenticated"], false);
    assert_eq!(body["locale"], "en");
    assert_eq!(body["version"], unipanel_core::version());
    assert!(body["user"].is_null());
    assert!(
        client.cookies.contains_key("unipanel_csrf"),
        "an anonymous CSRF cookie should be set"
    );
    assert!(!client.csrf.is_empty());
}

#[tokio::test]
async fn setup_creates_the_instance_and_signs_in() {
    let state = test_support::state().await;
    let mut client = Client::new(build_app(state));

    let boot = client.bootstrap().await;
    assert_eq!(boot["setup_required"], true);

    let (status, body) = client
        .post(
            "/api/v1/setup",
            setup_payload("de", "correct horse battery staple"),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "setup failed: {body}");
    assert_eq!(body["user"]["username"], "admin");
    assert_eq!(body["user"]["role"], "admin");
    assert!(client.cookies.contains_key("unipanel_session"));
    client.csrf = body["csrf_token"].as_str().expect("csrf").to_owned();

    let boot = client.bootstrap().await;
    assert_eq!(boot["setup_required"], false);
    assert_eq!(boot["authenticated"], true);
    assert_eq!(boot["locale"], "de");
    assert_eq!(boot["instance_name"], "Homelab");

    let (status, info) = client.get("/api/v1/system/info").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(info["locale"], "de");

    // The setup endpoint is closed afterwards.
    let (status, body) = client
        .post(
            "/api/v1/setup",
            setup_payload("en", "another long password"),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["key"], "setup.already_completed");
}

#[tokio::test]
async fn setup_requires_csrf_and_a_strong_password() {
    let state = test_support::state().await;
    let mut client = Client::new(build_app(state));
    client.bootstrap().await;

    client.csrf = "wrong".to_owned();
    let (status, body) = client
        .post(
            "/api/v1/setup",
            setup_payload("en", "correct horse battery staple"),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["key"], "request.csrf");

    client.bootstrap().await;
    let (status, body) = client
        .post("/api/v1/setup", setup_payload("en", "short"))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["key"], "password.too_short");
}

#[tokio::test]
async fn setup_rejects_invalid_input() {
    let state = test_support::state().await;
    let mut client = Client::new(build_app(state));
    client.bootstrap().await;

    let mut payload = setup_payload("fr", "correct horse battery staple");
    let (status, body) = client.post("/api/v1/setup", payload.clone()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["key"], "setup.invalid");
    assert_eq!(body["error"]["params"]["field"], "locale");
    payload["locale"] = json!("en");

    payload["username"] = json!("-nope");
    let (status, _) = client.post("/api/v1/setup", payload.clone()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    payload["username"] = json!("admin");
    payload["password"] = json!("adminadmin");
    let (status, body) = client.post("/api/v1/setup", payload).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["key"], "password.too_common");
}

#[tokio::test]
async fn login_logout_and_session_lifecycle() {
    let state = test_support::configured_state("correct horse battery staple").await;
    let mut client = Client::new(build_app(state));
    client.bootstrap().await;

    let (status, body) = client
        .post(
            "/api/v1/auth/login",
            json!({ "username": "admin", "password": "wrong password" }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["key"], "auth.invalid_credentials");

    let (status, body) = client
        .post(
            "/api/v1/auth/login",
            json!({ "username": "Admin", "password": "correct horse battery staple" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "login failed: {body}");
    assert_eq!(body["user"]["display_name"], "Ada Admin");
    client.csrf = body["csrf_token"].as_str().expect("csrf").to_owned();

    let (status, body) = client.get("/api/v1/auth/session").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["username"], "admin");

    let (status, _) = client.post("/api/v1/auth/logout", json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = client.get("/api/v1/auth/session").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["key"], "auth.required");
}

#[tokio::test]
async fn protected_routes_require_a_session() {
    let state = test_support::configured_state("correct horse battery staple").await;
    let mut client = Client::new(build_app(state));
    client.bootstrap().await;

    let (status, body) = client.get("/api/v1/system/info").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["key"], "auth.required");
    assert!(body["request_id"].as_str().is_some());
}

#[tokio::test]
async fn login_is_rate_limited_after_repeated_failures() {
    let state = test_support::configured_state("correct horse battery staple").await;
    let mut client = Client::new(build_app(state));
    client.bootstrap().await;

    let mut last_status = StatusCode::OK;
    let mut last_body = Value::Null;
    for _ in 0..12 {
        let (status, body) = client
            .post(
                "/api/v1/auth/login",
                json!({ "username": "admin", "password": "nope" }),
            )
            .await;
        last_status = status;
        last_body = body;
    }
    assert_eq!(last_status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(last_body["error"]["key"], "auth.rate_limited");
    assert!(last_body["error"]["params"]["retry_after"].is_number());
}

#[tokio::test]
async fn openapi_document_lists_the_public_endpoints() {
    let state = test_support::state().await;
    let mut client = Client::new(build_app(state));
    let (status, doc) = client.get("/api/v1/openapi.json").await;
    assert_eq!(status, StatusCode::OK);
    for path in [
        "/api/v1/bootstrap",
        "/api/v1/setup",
        "/api/v1/auth/login",
        "/api/v1/auth/logout",
        "/api/v1/auth/session",
        "/api/v1/system/health",
        "/api/v1/system/info",
    ] {
        assert!(doc["paths"].get(path).is_some(), "missing {path}");
    }
    assert!(doc["components"]["schemas"]["BootstrapResponse"].is_object());
}

#[tokio::test]
async fn malformed_json_uses_the_error_envelope() {
    let state = test_support::state().await;
    let app = build_app(state);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{ not json"))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.expect("body").to_bytes();
    let json: Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["error"]["key"], "request.invalid");
}
