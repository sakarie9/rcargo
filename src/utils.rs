use serde::Deserialize;
use std::fs;
use std::path::Path;

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
