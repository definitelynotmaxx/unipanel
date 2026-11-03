//! Test utilities: a fully wired application backed by an in-memory database.

use std::path::PathBuf;

use unipanel_crypto::MasterKey;
use unipanel_storage::Db;

use crate::config::{Config, DatabaseSource};
use crate::state::AppState;

/// A configuration that never touches the network and uses an in-memory DB.
#[must_use]
pub fn config() -> Config {
    Config {
        bind: "127.0.0.1:0".parse().expect("valid address"),
        data_dir: std::env::temp_dir().join(format!("unipanel-test-{}", uuid::Uuid::new_v4())),
        database: DatabaseSource::File(PathBuf::from(":memory:")),
        master_key: MasterKey::from_slice(&[9_u8; 32]).expect("key"),
        trust_proxy: false,
        secure_cookies: false,
        public_url: None,
    }
}

/// Builds a fresh state with migrations applied.
pub async fn state() -> AppState {
    let db = Db::connect_in_memory().await.expect("in-memory database");
    db.migrate().await.expect("migrations");
    AppState::new(db, config())
}

/// Builds a test state that already completed setup with the given password.
pub async fn configured_state(password: &str) -> AppState {
    let state = state().await;
    let hash = unipanel_auth::hash_password(password).expect("hash");
    state
        .db
        .setup_instance(&unipanel_storage::settings::InstanceSetup {
            locale: unipanel_core::Locale::En,
            username: "admin",
            display_name: "Ada Admin",
            password_hash: &hash,
            instance_name: "Test Panel",
        })
        .await
        .expect("setup");
    state.locale.set(Some(unipanel_core::Locale::En)).await;
    state
}
