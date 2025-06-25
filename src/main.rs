use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::sync::OnceLock;

mod commands;
mod utils;

use commands::{handle_purge_command, handle_size_command};
use utils::{ProjectIdentifier, create_target_symlink, is_required_target_dir};

/// Default target directory location when `RCARGO_TARGET_DIR` is not set.
///
/// This directory is typically located on a RAM disk or fast storage to
/// improve compilation performance. Users can override this location by
/// setting the `RCARGO_TARGET_DIR` environment variable.
const DEFAULT_TARGET_DIR: &str = "/tmp/rcargo_targets";

/// Global target directory that is computed once and cached for performance
static TARGET_DIR: OnceLock<String> = OnceLock::new();

fn main() {
    let cli = Cli::parse();

    // Check if version was requested
    if cli.show_version {
        print_version();
        return;
    }

    if let Err(e) = run_rcargo(cli) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

/// Command line interface definition for rcargo.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_version_flag = true)]
#[command(
    about = "A wrapper for Rust's cargo to use a per-project target directory on a fast storage"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show version information
    #[arg(short = 'V', long = "version")]
    pub show_version: bool,

    /// Cargo arguments (when no subcommand is provided)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<String>,
}

/// Available subcommands for rcargo.
#[derive(Subcommand)]
pub enum Commands {
    /// Show target directory sizes
    Size {
        /// Show all cached project target sizes
        #[arg(short, long)]
        all: bool,
    },
    /// Purge cached target directories
    Purge {
        /// Purge all cached project target directories
        #[arg(short, long)]
        all: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// Gets the target directory, computing it once and caching the result.
pub fn get_target_dir() -> &'static str {
    TARGET_DIR.get_or_init(|| {
        env::var("RCARGO_TARGET_DIR").unwrap_or_else(|_| DEFAULT_TARGET_DIR.to_string())
    })
}

/// Prints version information for both rcargo and the underlying cargo tool.
///
/// This function displays the rcargo version from the package metadata and
/// attempts to get and display the cargo version by executing `cargo --version`.
///
/// # Examples
///
/// ```text
/// rcargo 0.1.0
/// cargo 1.75.0 (1d8b05cdd 2023-11-20)
/// ```
fn print_version() {
    // Print rcargo version
    println!("rcargo {}", env!("CARGO_PKG_VERSION"));

    // Print cargo version
    match Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let cargo_version = String::from_utf8_lossy(&output.stdout);
                print!("{}", cargo_version);
            } else {
                eprintln!("Failed to get cargo version");
            }
        }
        Err(e) => {
            eprintln!("Failed to execute cargo --version: {}", e);
        }
    }
}

// Executes the main rcargo functionality based on parsed command line arguments.
fn run_rcargo(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Handle rcargo-specific subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Size { all } => {
                return handle_size_command(all);
            }
            Commands::Purge { all, yes } => {
                return handle_purge_command(all, yes);
            }
        }
    }

    // If no subcommand, proceed with normal cargo execution
    let args = cli.cargo_args;

    // Check if this command requires target directory
    if !is_required_target_dir(&args) {
        // For commands that don't need target directory, just execute cargo directly
        let mut cmd = Command::new("cargo");
        cmd.args(&args);

        let exit_status = cmd.status()?;
        if !exit_status.success() {
            if let Some(code) = exit_status.code() {
                exit(code);
            } else {
                exit(1);
            }
        }
        return Ok(());
    }

    // Get current project information
    let project_path = env::current_dir()?;
    let project_identifier = ProjectIdentifier::new(&project_path)?;

    // Get target directory from environment variable or use default
    let target_dir = get_target_dir();

    // Directly merge target path
    let cargo_target_dir = PathBuf::from(target_dir).join(project_identifier.identifier());

    // Create directory (if it doesn't exist)
    // fs::create_dir_all(&cargo_target_dir)?;

    // Print information message
    println!(
        "RCargo: Target directory redirected to: {}",
        cargo_target_dir.display()
    );

    // Set environment variable and execute the real cargo command
    let mut cmd = Command::new("cargo");
    cmd.args(&args);
    cmd.env("CARGO_TARGET_DIR", &cargo_target_dir);

    let exit_status = cmd.status()?;

    if !exit_status.success() {
        if let Some(code) = exit_status.code() {
            exit(code);
        } else {
            exit(1);
        }
    }

    // Create target symlink after successful execution
    if let Err(e) = create_target_symlink(&project_path, &cargo_target_dir) {
        eprintln!("Warning: Could not create target symlink: {}", e);
    }

    Ok(())
}
