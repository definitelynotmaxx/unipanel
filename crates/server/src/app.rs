//! Router assembly and the server entry point.

use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::IntoResponse as _;
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use unipanel_core::ErrorKey;
use unipanel_storage::StorageError;

use crate::api;
use crate::assets;
use crate::config::Config;
use crate::error::ApiError;
use crate::middleware;
use crate::state::AppState;

/// Maximum accepted JSON request body.
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
/// Per-request time limit; streaming endpoints opt out later.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors that can stop the server.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// The listening socket could not be bound.
    #[error("could not bind {address}: {source}")]
    Bind {
        /// Requested address.
        address: String,
        /// Underlying error.
        source: std::io::Error,
    },
    /// The HTTP server stopped with an error.
    #[error("http server error: {0}")]
    Serve(#[from] std::io::Error),
    /// Storage failed during startup.
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

/// Builds the complete application router.
///
/// Layers are added from the inside out: the request id and security headers are
/// outermost so they apply to timeouts and panics as well.
pub fn build_app(state: AppState) -> Router {
    let api_router = api::routes().fallback(unknown_api_route);

    Router::new()
        .route("/healthz", get(api::system::healthz))
        .nest("/api/v1", api_router)
        .fallback(assets::spa)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(CatchPanicLayer::new())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::security_headers,
        ))
        .layer(axum::middleware::from_fn(middleware::request_id))
        .with_state(state)
}

async fn unknown_api_route() -> axum::response::Response {
    ApiError::new(ErrorKey::RequestNotFound).into_response()
}

/// Opens the database, applies migrations and removes stale sessions.
///
/// # Errors
/// Returns [`ServerError::Storage`] when the database cannot be opened or
/// migrated.
pub async fn open_database(config: &Config) -> Result<unipanel_storage::Db, ServerError> {
    let db = match &config.database {
        crate::config::DatabaseSource::File(path) => unipanel_storage::Db::connect_file(path).await?,
        crate::config::DatabaseSource::Url(url) => unipanel_storage::Db::connect(url).await?,
    };
    db.migrate().await?;
    match db.purge_expired_sessions().await {
        Ok(removed) if removed > 0 => tracing::info!(removed, "purged expired sessions"),
        Ok(_) => {}
        Err(err) => tracing::warn!(error = %err, "could not purge expired sessions"),
    }
    Ok(db)
}

/// Runs the HTTP server until `shutdown` resolves.
///
/// # Errors
/// Returns [`ServerError::Bind`] when the port is taken and
/// [`ServerError::Serve`] on a fatal accept error.
pub async fn serve<F>(state: AppState, shutdown: F) -> Result<(), ServerError>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let address = state.config.bind;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|source| ServerError::Bind {
            address: address.to_string(),
            source,
        })?;
    let local = listener.local_addr().unwrap_or(address);

    tracing::info!(
        %local,
        version = unipanel_core::version(),
        data_dir = %state.config.data_dir.display(),
        "UniPanel listening"
    );

    axum::serve(
        listener,
        build_app(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;

    Ok(())
}

/// Resolves when the process receives Ctrl+C or SIGTERM.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("shutdown requested");
}

/// Convenience entry point used by the CLI: open the database and serve.
///
/// # Errors
/// Propagates startup failures.
pub async fn run(config: Config) -> Result<(), ServerError> {
    let db = open_database(&config).await?;
    let state = AppState::new(db, config);
    serve(state, shutdown_signal()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt as _;
    use tower::ServiceExt as _;

    #[tokio::test]
    async fn unknown_api_routes_return_json_not_the_spa() {
        let state = crate::test_support::state().await;
        let response = build_app(state)
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/does-not-exist")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), 404);
        let content_type = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert!(content_type.starts_with("application/json"));
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["error"]["key"], "request.not_found");
    }

    #[tokio::test]
    async fn healthz_reports_ok() {
        let state = crate::test_support::state().await;
        let response = build_app(state)
            .oneshot(
                axum::http::Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), 200);
        let body = response.into_body().collect().await.expect("body").to_bytes();
        assert_eq!(&body[..], b"ok\n");
    }
}
