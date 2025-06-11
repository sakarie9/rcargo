use crate::get_target_dir;
use crate::utils::{ProjectIdentifier, calculate_directory_size, format_size, is_rust_project};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Handles the size command to display target directory sizes.
pub fn handle_size_command(all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = get_target_dir();
    let target_path = PathBuf::from(target_dir);

    if all {
        // Show all project sizes
        show_all_project_sizes(&target_path)?;
    } else {
        // Show current project size or all if not in a Rust project
        let project_path = env::current_dir()?;

        // Check if current directory is a Rust project
        if is_rust_project(&project_path) {
            // Show current project size
            let project_identifier = ProjectIdentifier::new(&project_path)?;
            let project_target_dir = target_path.join(project_identifier.identifier());

            if project_target_dir.exists() {
                let size = calculate_directory_size(&project_target_dir)?;
                println!(
                    "Current project '{}' target size: {}",
                    project_identifier.name(),
                    format_size(size)
                );
            } else {
                println!(
                    "Current project '{}' has no cached target directory",
                    project_identifier.name()
                );
            }
        } else {
            // Not in a Rust project, show all cached projects
            show_all_project_sizes(&target_path)?;
        }
    }

    Ok(())
}

/// Shows all cached project target directories and their sizes.
fn show_all_project_sizes(target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if target_path.exists() {
        println!("All cached project target directories:");
        let mut total_size = 0;

        for entry in fs::read_dir(target_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_size = calculate_directory_size(&entry.path())?;
                total_size += dir_size;
                println!(
                    "  {}: {}",
                    entry.file_name().to_string_lossy(),
                    format_size(dir_size)
                );
            }
        }

        println!("Total cache size: {}", format_size(total_size));
    } else {
        println!("No cached target directories found");
    }
    Ok(())
}
