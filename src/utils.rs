use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[cfg(unix)]
use std::os::unix::fs as unix_fs;

#[cfg(windows)]
use std::os::windows::fs as windows_fs;

/// Structure for parsing Cargo.toml files.
#[derive(Deserialize)]
struct CargoToml {
    package: Option<Package>,
}

/// Package section of Cargo.toml.
#[derive(Deserialize)]
struct Package {
    name: Option<String>,
}

/// Structure to hold project identifier information.
#[derive(Debug, Clone)]
pub struct ProjectIdentifier {
    name: String,
    hash: String,
}

impl ProjectIdentifier {
    pub fn new(project_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let name = get_project_name(project_path)?;
        let hash = generate_project_hash(project_path);

        Ok(ProjectIdentifier { name, hash })
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identifier(&self) -> String {
        format!("{}-{}", self.name, self.hash)
    }
}

/// Checks if the given path is a Rust project directory.
pub fn is_rust_project(project_path: &Path) -> bool {
    project_path.join("Cargo.toml").exists()
}

/// Recursively calculates the total size of a directory.
pub fn calculate_directory_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                total_size += calculate_directory_size(&entry_path)?;
            } else {
                total_size += entry.metadata()?.len();
            }
        }
    }

    Ok(total_size)
}

/// Formats a byte size into a human-readable string.
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GiB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MiB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KiB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Generates a short hash for a project based on its path.
fn generate_project_hash(project_path: &Path) -> String {
    let project_path_str = project_path.to_string_lossy();
    let project_hash_full = format!("{:x}", md5::compute(project_path_str.as_bytes()));
    project_hash_full[0..7].to_string()
}

/// Extracts the project name from the given project path.
fn get_project_name(project_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Try to read project name from Cargo.toml
    let cargo_toml_path = project_path.join("Cargo.toml");

    if cargo_toml_path.exists() {
        match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => {
                match toml::from_str::<CargoToml>(&content) {
                    Ok(cargo_toml) => {
                        if let Some(package) = cargo_toml.package {
                            if let Some(name) = package.name {
                                return Ok(name);
                            }
                        }
                    }
                    Err(_) => {
                        // Parsing failed, use directory name as fallback
                    }
                }
            }
            Err(_) => {
                // Failed to read file, use directory name as fallback
            }
        }
    }

    // If we couldn't get project name from Cargo.toml, fall back to directory name
    Ok(project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string())
}

// Helper function to create the target symlink
pub fn create_target_symlink(
    project_path: &PathBuf,
    cargo_target_dir: &PathBuf,
) -> Result<(), std::io::Error> {
    if env::var("RCARGO_NO_TARGET_LINK")
        .map_or(false, |val| val.eq_ignore_ascii_case("true") || val == "1")
    {
        return Ok(()); // Skip creating link if RCARGO_NO_TARGET_LINK is set
    }

    let symlink_name_from_env = env::var("RCARGO_TARGET_LINK_NAME").unwrap_or_default();
    let symlink_name = if symlink_name_from_env.is_empty() {
        "target_rcargo"
    } else {
        &symlink_name_from_env
    };

    let symlink_path = project_path.join(symlink_name);
    let mut create_link = true;

    if symlink_path.exists() {
        match fs::symlink_metadata(&symlink_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    // If same target directory, do nothing
                    if let Ok(existing_target) = fs::read_link(&symlink_path) {
                        if existing_target == cargo_target_dir.as_path() {
                            return Ok(());
                        }
                    }
                    // If symlink exists but points to a different target, remove it
                    if let Err(e) = fs::remove_file(&symlink_path) {
                        eprintln!(
                            "Warning: Failed to remove existing symlink at '{}': {}. Proceeding to create new one.",
                            symlink_path.display(),
                            e
                        );
                    }
                } else if metadata.is_file() {
                    eprintln!(
                        "Warning: '{}' already exists and is a file. Skipping symlink creation.",
                        symlink_path.display()
                    );
                    create_link = false;
                } else if metadata.is_dir() {
                    eprintln!(
                        "Warning: '{}' already exists and is a directory. Skipping symlink creation.",
                        symlink_path.display()
                    );
                    create_link = false;
                } else {
                    eprintln!(
                        "Warning: '{}' exists and is not a file, directory, or symlink. Skipping symlink creation.",
                        symlink_path.display()
                    );
                    create_link = false;
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to get metadata for '{}': {}. Attempting to create symlink anyway.",
                    symlink_path.display(),
                    e
                );
            }
        }
    }

    if create_link {
        #[cfg(unix)]
        {
            match unix_fs::symlink(cargo_target_dir, &symlink_path) {
                Ok(_) => println!(
                    "RCargo: Created symlink '{}' -> '{}'",
                    symlink_path.display(),
                    cargo_target_dir.display()
                ),
                Err(e) => eprintln!(
                    "Warning: Failed to create symlink '{}' -> '{}'. Error: {}",
                    symlink_path.display(),
                    cargo_target_dir.display(),
                    e
                ),
            }
        }
        #[cfg(windows)]
        {
            match windows_fs::symlink_dir(cargo_target_dir, &symlink_path) {
                Ok(_) => {
                    println!(
                        "RCargo: Created directory symlink '{}' -> '{}'",
                        symlink_path.display(),
                        cargo_target_dir.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to create symlink '{}' -> '{}'. Error: {}. On Windows, this might require administrator privileges or Developer Mode to be enabled.",
                        symlink_path.display(),
                        cargo_target_dir.display(),
                        e
                    );
                }
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            eprintln!(
                "Warning: Symlink creation for '{}' is not supported on this platform. Skipping.",
                symlink_path.display()
            );
        }
    }
    Ok(())
}

/// Checks if the given cargo command requires target directory creation.
///
/// Returns false for commands that are purely informational or don't involve
/// compilation/building, such as --help, --version, etc.
pub fn is_required_target_dir(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }

    // Commands that don't need target directory
    let non_build_commands = [
        "--help",
        "-h",
        "--version",
        "-V",
        "--list",
        "help",
        "version",
        "search",
        "login",
        "logout",
        "owner",
        "yank",
        "publish",
        "cache",
    ];

    // Check for global flags that don't require building
    for arg in args {
        if non_build_commands.contains(&arg.as_str()) {
            return false;
        }
    }

    // Check for subcommands that don't require building
    if let Some(first_arg) = args.first() {
        // Skip flags to find actual subcommand
        let subcommand = if first_arg.starts_with('-') {
            args.iter().find(|arg| !arg.starts_with('-'))
        } else {
            Some(first_arg)
        };

        if let Some(cmd) = subcommand {
            match cmd.as_str() {
                "search" | "login" | "logout" | "owner" | "yank" | "publish" => return false,
                _ => {}
            }
        }
    }

    true
}
