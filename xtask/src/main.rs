//! `cargo xtask <command>`: everything a plugin needs from a new folder to a release.
//!
//! - `new <id> --name <Name>`: a new plugin from `templates/plugin`.
//! - `build <id>`: the plugin for wasm32-wasip2 (`target/wasm32-wasip2/release`, where its plugin.toml points).
//! - `package <id>`: build and pack `dist/<id>-<version>.rcplugin` (tar + zstd) with its SHA-256 file.
//! - `check`: manifests, versions, changelogs, formatting, clippy (native and wasm), tests.
//! - `release <id> [--publish]`: check, package, and print (or, with `--publish`, run) the tag and the GitHub release.

mod checks;
mod commands;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo xtask", about = "Tools of the rustcut-plugins repository")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Creates plugins/<id> from the template.
    New {
        /// Lower case letters, digits and dashes: auto-name.
        id: String,
        /// The name RustCut shows: "Auto-Name".
        #[arg(long)]
        name: String,
    },
    /// Builds a plugin (or all) for wasm32-wasip2.
    Build { id: Option<String> },
    /// Builds and packs dist/<id>-<version>.rcplugin.
    Package { id: String },
    /// Checks every plugin and runs the lints and tests.
    Check {
        /// Skip clippy and the tests (manifests, versions, changelogs and formatting only).
        #[arg(long)]
        fast: bool,
    },
    /// Prepares a release of one plugin: check, package, tag and GitHub release.
    Release {
        id: String,
        /// Really create and push the tag and the GitHub release (otherwise the commands are only printed).
        #[arg(long)]
        publish: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = root();
    let result = match cli.command {
        Command::New { id, name } => commands::new(&root, &id, &name),
        Command::Build { id } => commands::build(&root, id.as_deref()),
        Command::Package { id } => commands::package(&root, &id).map(|file| println!("{}", file.display())),
        Command::Check { fast } => checks::check(&root, fast),
        Command::Release { id, publish } => commands::release(&root, &id, publish),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// The repository's root (the parent of `xtask/`).
fn root() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.parent().map_or(here.clone(), PathBuf::from)
}
