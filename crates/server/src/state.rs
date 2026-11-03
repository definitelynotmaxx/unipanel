//! Shared application state.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use unipanel_auth::LoginThrottle;
use unipanel_core::Locale;
use unipanel_crypto::SecretBox;
use unipanel_storage::Db;

use crate::config::Config;

/// How long the instance locale may be cached before it is re-read.
///
/// The CLI can change the locale while the server runs, so the cache is
/// deliberately short-lived instead of being invalidated across processes.
const LOCALE_TTL: Duration = Duration::from_secs(10);

/// A cached snapshot of the instance locale.
#[derive(Debug)]
struct CachedLocale {
    value: Option<Locale>,
    loaded_at: Option<Instant>,
}

/// Read-mostly cache for the immutable-per-setup instance locale.
#[derive(Debug, Clone)]
pub struct LocaleCache {
    inner: Arc<RwLock<CachedLocale>>,
}

impl LocaleCache {
    fn empty() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CachedLocale {
                value: None,
                loaded_at: None,
            })),
        }
    }

    /// Returns the locale, refreshing from the database when the entry is stale.
    pub async fn get(&self, db: &Db) -> Option<Locale> {
        {
            let cached = self.inner.read().await;
            if cached
                .loaded_at
                .is_some_and(|at| at.elapsed() < LOCALE_TTL)
            {
                return cached.value;
            }
        }

        let value = db.instance_locale().await.ok().flatten();
        let mut cached = self.inner.write().await;
        cached.value = value;
        cached.loaded_at = Some(Instant::now());
        value
    }

    /// Overrides the cached value, used right after setup and by tests.
    pub async fn set(&self, value: Option<Locale>) {
        let mut cached = self.inner.write().await;
        cached.value = value;
        cached.loaded_at = Some(Instant::now());
    }
}

/// Everything handlers need, cloned per request.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Database handle.
    pub db: Db,
    /// Resolved configuration.
    pub config: Arc<Config>,
    /// Secret encryption for connector credentials.
    pub secrets: Arc<SecretBox>,
    /// Failed login throttle.
    pub throttle: Arc<LoginThrottle>,
    /// Cached instance locale.
    pub locale: LocaleCache,
    /// Server start time, for uptime reporting.
    pub started_at: time::OffsetDateTime,
}

impl AppState {
    /// Builds state around an initialised database.
    #[must_use]
    pub fn new(db: Db, config: Config) -> Self {
        let locale = LocaleCache::empty();
        let secrets = Arc::new(SecretBox::new(&config.master_key));
        Self {
            db,
            config: Arc::new(config),
            secrets,
            throttle: Arc::new(LoginThrottle::default()),
            locale,
            started_at: unipanel_storage::time::now(),
        }
    }

    /// Current instance locale, if setup has completed.
    pub async fn instance_locale(&self) -> Option<Locale> {
        self.locale.get(&self.db).await
    }

    /// Whether cookies should carry the `Secure` attribute.
    #[must_use]
    pub fn secure_cookies(&self) -> bool {
        self.config.secure_cookies
    }

    /// Seconds since the server started.
    #[must_use]
    pub fn uptime_seconds(&self) -> i64 {
        (unipanel_storage::time::now() - self.started_at)
            .whole_seconds()
            .max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn locale_cache_serves_and_updates() {
        let db = Db::connect_in_memory().await.expect("db");
        db.migrate().await.expect("migrate");
        let cache = LocaleCache::empty();
        assert_eq!(cache.get(&db).await, None);

        db.set_setting(unipanel_storage::settings::KEY_LOCALE, "de")
            .await
            .expect("set");
        // First read populates the cache.
        assert_eq!(cache.get(&db).await, Some(Locale::De));

        // A direct database change is not visible until the TTL expires.
        db.set_setting(unipanel_storage::settings::KEY_LOCALE, "en")
            .await
            .expect("set");
        assert_eq!(cache.get(&db).await, Some(Locale::De));

        cache.set(Some(Locale::En)).await;
        assert_eq!(cache.get(&db).await, Some(Locale::En));
    }
}
