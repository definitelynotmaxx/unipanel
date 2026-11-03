//! UniPanel command line interface.
//!
//! All long-running work happens through this binary so the container image has a
//! single entry point: `unipanel serve` runs the server, `unipanel doctor` checks
//! an installation and `unipanel admin` performs the few operations that are
//! deliberately not exposed in the web UI.

mod commands;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// Self-hosted control panel for game servers, containers and VMs.
#[derive(Debug, Parser)]
#[command(
    name = "unipanel",
    version,
    about = "Self-hosted control panel for Pterodactyl, Docker, Proxmox and custom APIs",
    long_about = None,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the HTTP server (default).
    Serve(ServeArgs),
    /// Apply pending database migrations and exit.
    Migrate(CommonArgs),
    /// Check configuration, database and key material.
    Doctor(CommonArgs),
    /// Print version and build information.
    Version,
    /// Print the OpenAPI document.
    Openapi {
        /// Write to this file instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Instance administration commands. Not available in the web UI on purpose.
    Admin(AdminArgs),
}

#[derive(Debug, clap::Args)]
struct CommonArgs {
    /// Data directory (defaults to UNIPANEL_DATA_DIR or ./data).
    #[arg(long, env = "UNIPANEL_DATA_DIR")]
    data_dir: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
struct ServeArgs {
    /// Data directory (defaults to UNIPANEL_DATA_DIR or ./data).
    #[arg(long, env = "UNIPANEL_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// Listen address, e.g. 0.0.0.0:8080.
    #[arg(long, env = "UNIPANEL_BIND")]
    bind: Option<SocketAddr>,
}

#[derive(Debug, clap::Args)]
struct AdminArgs {
    #[command(subcommand)]
    command: AdminCommand,
}

#[derive(Debug, Subcommand)]
enum AdminCommand {
    /// Change the instance language. The UI never offers this.
    SetLocale {
        /// Target locale: en or de.
        #[arg(long, value_parser = ["en", "de"])]
        locale: String,
        /// Required acknowledgement that the change affects every user.
        #[arg(long)]
        yes: bool,
        #[arg(long, env = "UNIPANEL_DATA_DIR")]
        data_dir: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(run(cli))
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command.unwrap_or(Command::Serve(ServeArgs {
        data_dir: None,
        bind: None,
    })) {
        Command::Serve(args) => commands::serve(args.data_dir, args.bind).await,
        Command::Migrate(args) => commands::migrate(args.data_dir).await,
        Command::Doctor(args) => commands::doctor(args.data_dir).await,
        Command::Version => {
            commands::version();
            Ok(())
        }
        Command::Openapi { output } => commands::openapi(output),
        Command::Admin(args) => match args.command {
            AdminCommand::SetLocale {
                locale,
                yes,
                data_dir,
            } => commands::admin_set_locale(data_dir, &locale, yes).await,
        },
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("UNIPANEL_LOG")
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init();
}

/// Applies CLI overrides on top of the environment configuration.
pub(crate) fn load_config(
    data_dir: Option<PathBuf>,
    bind: Option<SocketAddr>,
) -> anyhow::Result<unipanel_server::Config> {
    if let Some(dir) = &data_dir {
        std::fs::create_dir_all(dir)
            .map_err(|err| anyhow::anyhow!("could not create {}: {err}", dir.display()))?;
    }
    // `Config::from_env_with` is the single reader of the environment; overrides
    // are applied there instead of mutating process-wide variables.
    Ok(unipanel_server::Config::from_env_with(data_dir, bind)?)
}
