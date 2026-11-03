//! Implementations of the CLI subcommands.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use unipanel_core::brand;

use crate::load_config;

/// Runs the server until shutdown.
pub async fn serve(data_dir: Option<PathBuf>, bind: Option<SocketAddr>) -> anyhow::Result<()> {
    let config = load_config(data_dir, bind)?;
    let name = brand();
    eprintln!(
        "{} {} starting on http://{}",
        name.display_name,
        unipanel_core::version(),
        config.bind
    );
    unipanel_server::run(config)
        .await
        .context("server stopped with an error")
}

/// Applies migrations and exits.
pub async fn migrate(data_dir: Option<PathBuf>) -> anyhow::Result<()> {
    let config = load_config(data_dir, None)?;
    let db = unipanel_server::open_database(&config)
        .await
        .context("could not prepare the database")?;
    db.close().await;
    println!(
        "database ready (schema {}): {}",
        unipanel_core::SCHEMA_VERSION,
        config.database
    );
    Ok(())
}

/// Performs installation checks and prints a report.
pub async fn doctor(data_dir: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{} doctor", brand().display_name);
    println!("  version      {}", unipanel_core::version());
    println!(
        "  platform     {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let config = match load_config(data_dir, None) {
        Ok(config) => config,
        Err(err) => {
            println!("  config       FAIL ({err})");
            bail!("configuration is invalid");
        }
    };
    println!("  data dir     {}", config.data_dir.display());
    println!("  bind         {}", config.bind);
    println!("  database     {}", config.database);
    println!(
        "  cookies      {}",
        if config.secure_cookies {
            "Secure"
        } else {
            "not Secure (plain HTTP)"
        }
    );

    let writable = check_writable(&config.data_dir);
    println!(
        "  data dir     {}",
        if writable { "writable" } else { "FAIL (not writable)" }
    );

    println!("  master key   present");
    if config
        .database_path()
        .is_some_and(|path| !path.exists())
    {
        println!("  database     will be created on first start");
    }

    match unipanel_server::open_database(&config).await {
        Ok(db) => {
            println!("  database     ok (schema {})", unipanel_core::SCHEMA_VERSION);
            if let Ok(setup) = db.is_setup_complete().await {
                println!(
                    "  setup        {}",
                    if setup {
                        "completed"
                    } else {
                        "pending (open the web UI to finish)"
                    }
                );
            }
            db.close().await;
        }
        Err(err) => {
            println!("  database     FAIL ({err})");
            bail!("database check failed");
        }
    }

    match std::net::TcpListener::bind(config.bind) {
        Ok(listener) => {
            drop(listener);
            println!("  port         {} is free", config.bind.port());
        }
        Err(_) => println!(
            "  port         {} is in use (a running instance?)",
            config.bind.port()
        ),
    }

    println!("all checks passed");
    Ok(())
}

fn check_writable(dir: &Path) -> bool {
    let probe = dir.join(".unipanel-write-test");
    let result = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(probe);
    result
}

/// Prints version information.
pub fn version() {
    println!("{} {}", brand().display_name, unipanel_core::version());
    println!("slug: {}", brand().slug);
    println!("schema: {}", unipanel_core::SCHEMA_VERSION);
    println!("platform: {} {}", std::env::consts::OS, std::env::consts::ARCH);
}

/// Writes the OpenAPI document to a file or stdout.
pub fn openapi(output: Option<PathBuf>) -> anyhow::Result<()> {
    let document = unipanel_server::api::openapi_json()?;
    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&path, &document)
                .with_context(|| format!("could not write {}", path.display()))?;
            println!("wrote {}", path.display());
        }
        None => println!("{document}"),
    }
    Ok(())
}

/// Changes the instance locale. Requires an explicit acknowledgement because the
/// language is fixed for everyone.
pub async fn admin_set_locale(
    data_dir: Option<PathBuf>,
    locale: &str,
    acknowledged: bool,
) -> anyhow::Result<()> {
    let target: unipanel_core::Locale = locale
        .parse()
        .map_err(|_| anyhow::anyhow!("unsupported locale {locale:?}"))?;

    if !acknowledged {
        eprintln!(
            "warning: the instance locale applies to every user and cannot be changed in the web UI."
        );
        bail!("re-run with --yes to change the locale to {locale}");
    }

    let config = load_config(data_dir, None)?;
    let db = unipanel_server::open_database(&config)
        .await
        .context("could not open the database")?;
    if db.instance_locale().await?.is_none() {
        db.close().await;
        bail!("the instance is not set up yet; finish the setup wizard first");
    }
    db.set_instance_locale(target).await?;
    db.close().await;

    println!("instance locale is now {locale}");
    println!("restart the server (or wait up to 10 seconds) for the UI to pick it up");
    Ok(())
}
