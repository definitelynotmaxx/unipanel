//! Process configuration from `UNIPANEL_*` environment variables.
//!
//! Every setting has a safe default so `docker run -p 8080:8080 unipanel` works
//! without a configuration file. Anything sensitive is either read from the
//! environment or from a `0600` file in the data directory, never from the
//! database.

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use tracing::{info, warn};
use unipanel_core::brand;
use unipanel_crypto::MasterKey;

/// Default listen address for the container.
pub const DEFAULT_BIND: &str = "0.0.0.0:8080";
/// Default data directory when nothing is configured.
pub const DEFAULT_DATA_DIR: &str = "data";
/// Name of the master key file inside the data directory.
pub const MASTER_KEY_FILE: &str = "master.key";
/// Name of the SQLite database inside the data directory.
pub const DATABASE_FILE: &str = "unipanel.db";

/// Where the database lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseSource {
    /// A file inside the data directory (default).
    File(PathBuf),
    /// An explicit SQLx URL, e.g. for tests or an external Postgres later.
    Url(String),
}

impl std::fmt::Display for DatabaseSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => write!(f, "sqlite file {}", path.display()),
            Self::Url(url) => write!(f, "{}", redact(url)),
        }
    }
}

fn redact(url: &str) -> String {
    match (url.split_once("://"), url.find('@')) {
        (Some((scheme, _)), Some(at)) => format!("{scheme}://***{}", &url[at..]),
        _ => url.split('?').next().unwrap_or(url).to_owned(),
    }
}

/// Fully resolved runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Address the HTTP server binds to.
    pub bind: SocketAddr,
    /// Directory holding the database, master key and future uploads.
    pub data_dir: PathBuf,
    /// Database location.
    pub database: DatabaseSource,
    /// Key used to encrypt connector secrets at rest.
    pub master_key: MasterKey,
    /// Whether `X-Forwarded-*` headers from a reverse proxy are trusted.
    pub trust_proxy: bool,
    /// Whether cookies carry the `Secure` attribute.
    pub secure_cookies: bool,
    /// Optional externally visible URL, used in logs and future emails.
    pub public_url: Option<String>,
}

/// Configuration problems, reported to the operator with an actionable hint.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A value could not be parsed.
    #[error("{name} is invalid: {reason}")]
    Invalid {
        /// Environment variable name.
        name: String,
        /// Why it is invalid.
        reason: String,
    },
    /// The data directory could not be created.
    #[error("could not create data directory {path}: {source}")]
    DataDir {
        /// Directory that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The master key file could not be read or written.
    #[error("could not {action} master key at {path}: {source}")]
    MasterKey {
        /// What was attempted.
        action: &'static str,
        /// File path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The provided master key is not 32 base64-encoded bytes.
    #[error("{name} must be a base64 encoded 32-byte key")]
    MasterKeyFormat {
        /// Environment variable name.
        name: String,
    },
}

impl Config {
    /// Reads the environment and prepares the data directory.
    ///
    /// # Errors
    /// Returns [`ConfigError`] for unparsable values or filesystem failures.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_env_with(None, None)
    }

    /// Reads the environment, then applies the given CLI overrides.
    ///
    /// Overrides are applied after parsing so the CLI never has to mutate
    /// process-wide environment variables.
    ///
    /// # Errors
    /// Returns [`ConfigError`] for unparsable values or filesystem failures.
    pub fn from_env_with(
        data_dir_override: Option<PathBuf>,
        bind_override: Option<SocketAddr>,
    ) -> Result<Self, ConfigError> {
        let brand = brand();
        let data_dir = data_dir_override.unwrap_or_else(|| {
            env::var_os(brand.var("DATA_DIR"))
                .map_or_else(|| PathBuf::from(DEFAULT_DATA_DIR), PathBuf::from)
        });
        std::fs::create_dir_all(&data_dir).map_err(|source| ConfigError::DataDir {
            path: data_dir.clone(),
            source,
        })?;

        let bind: SocketAddr = match bind_override {
            Some(bind) => bind,
            None => match env::var(brand.var("BIND")) {
                Ok(raw) if !raw.trim().is_empty() => {
                    raw.parse().map_err(|err: std::net::AddrParseError| {
                        ConfigError::Invalid {
                            name: brand.var("BIND"),
                            reason: err.to_string(),
                        }
                    })?
                }
                _ => DEFAULT_BIND.parse().map_err(|_| ConfigError::Invalid {
                    name: brand.var("BIND"),
                    reason: DEFAULT_BIND.to_owned(),
                })?,
            },
        };

        let database = match env::var(brand.var("DATABASE_URL")) {
            Ok(url) if !url.trim().is_empty() => DatabaseSource::Url(url),
            _ => DatabaseSource::File(data_dir.join(DATABASE_FILE)),
        };

        let master_key = load_master_key(&data_dir)?;

        let trust_proxy = parse_bool(&brand.var("TRUST_PROXY"), false)?;

        let public_url = env::var(brand.var("PUBLIC_URL"))
            .ok()
            .map(|raw| raw.trim().trim_end_matches('/').to_owned())
            .filter(|raw| !raw.is_empty());

        let secure_cookies = match env::var(brand.var("COOKIE_SECURE")) {
            Ok(raw) if !raw.trim().is_empty() => parse_bool(&brand.var("COOKIE_SECURE"), true)?,
            _ => public_url
                .as_deref()
                .is_some_and(|url| url.starts_with("https://")),
        };

        Ok(Self {
            bind,
            data_dir,
            database,
            master_key,
            trust_proxy,
            secure_cookies,
            public_url,
        })
    }

    /// Path of the SQLite file, when the default database is used.
    #[must_use]
    pub fn database_path(&self) -> Option<&Path> {
        match &self.database {
            DatabaseSource::File(path) => Some(path),
            DatabaseSource::Url(_) => None,
        }
    }
}

fn parse_bool(name: &str, default: bool) -> Result<bool, ConfigError> {
    match env::var(name) {
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "" => Ok(default),
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            other => Err(ConfigError::Invalid {
                name: name.to_owned(),
                reason: format!("expected a boolean, got {other:?}"),
            }),
        },
        Err(_) => Ok(default),
    }
}

fn load_master_key(data_dir: &Path) -> Result<MasterKey, ConfigError> {
    let brand = brand();
    let env_key = brand.var("MASTER_KEY");
    if let Ok(raw) = env::var(&env_key) {
        if !raw.trim().is_empty() {
            info!("using master key from {env_key}");
            return MasterKey::from_base64(&raw)
                .map_err(|_| ConfigError::MasterKeyFormat { name: env_key });
        }
    }

    let file_var = brand.var("MASTER_KEY_FILE");
    if let Ok(raw) = env::var(&file_var) {
        if !raw.trim().is_empty() {
            let path = PathBuf::from(raw);
            let contents = std::fs::read_to_string(&path).map_err(|source| {
                ConfigError::MasterKey {
                    action: "read",
                    path: path.clone(),
                    source,
                }
            })?;
            return MasterKey::from_base64(&contents)
                .map_err(|_| ConfigError::MasterKeyFormat { name: file_var });
        }
    }

    let path = data_dir.join(MASTER_KEY_FILE);
    if path.exists() {
        let contents =
            std::fs::read_to_string(&path).map_err(|source| ConfigError::MasterKey {
                action: "read",
                path: path.clone(),
                source,
            })?;
        return MasterKey::from_base64(&contents).map_err(|_| {
            ConfigError::MasterKeyFormat {
                name: path.display().to_string(),
            }
        });
    }

    // First run without any key: generate one and persist it with owner-only
    // permissions. Losing this file means losing access to stored secrets.
    let key = MasterKey::generate();
    write_private_file(&path, &key.to_base64()).map_err(|source| ConfigError::MasterKey {
        action: "write",
        path: path.clone(),
        source,
    })?;
    warn!(
        path = %path.display(),
        "generated a new master key; back this file up or secrets become unrecoverable"
    );
    Ok(key)
}

fn write_private_file(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write as _;

    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt as _;
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?
    };
    #[cfg(not(unix))]
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_credentials() {
        assert_eq!(
            redact("postgres://user:pass@db/unipanel?sslmode=require"),
            "postgres://***@db/unipanel"
        );
        assert_eq!(redact("sqlite:///data/db"), "sqlite:///data/db");
    }

    #[test]
    fn master_key_round_trip_through_a_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("master.key");
        let key = MasterKey::generate();
        write_private_file(&path, &key.to_base64()).expect("write");
        let read = std::fs::read_to_string(&path).expect("read");
        let parsed = MasterKey::from_base64(&read).expect("parse");
        assert_eq!(parsed.as_bytes(), key.as_bytes());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }
}
