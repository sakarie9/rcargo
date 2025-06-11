use crate::get_target_dir;
use crate::utils::{ProjectIdentifier, calculate_directory_size, format_size};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Prompts the user for confirmation before purging.
fn confirm_purge(message: &str) -> Result<bool, Box<dyn std::error::Error>> {
    print!("{} (y/N): ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

/// Handles the purge command to clean up cached target directories.
pub fn handle_purge_command(all: bool, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = get_target_dir();
    let target_path = PathBuf::from(target_dir);

    if !all {
        // Clean current project
        let project_path = env::current_dir()?;
        let project_identifier = ProjectIdentifier::new(&project_path)?;

        let project_target_dir = target_path.join(project_identifier.identifier());

        if project_target_dir.exists() {
            let size_before = calculate_directory_size(&project_target_dir)?;
            let size_str = format_size(size_before);

            let should_purge = yes
                || confirm_purge(&format!(
                    "Are you sure you want to purge project '{}' cache ({})?",
                    project_identifier.name(),
                    size_str
                ))?;

            if should_purge {
                fs::remove_dir_all(&project_target_dir)?;
                println!(
                    "Purged current project '{}' cache (freed {})",
                    project_identifier.name(),
                    size_str
                );
            } else {
                println!("Purge cancelled.");
            }
        } else {
            println!(
                "Current project '{}' has no cached target directory to purge",
                project_identifier.name()
            );
        }
    } else {
        // Clean all projects
        if target_path.exists() {
            let total_size_before = calculate_directory_size(&target_path)?;
            let size_str = format_size(total_size_before);

            let should_purge = yes
                || confirm_purge(&format!(
                    "Are you sure you want to purge ALL cached target directories ({})?",
                    size_str
                ))?;

            if should_purge {
                fs::remove_dir_all(&target_path)?;
                fs::create_dir_all(&target_path)?;
                println!("Purged all cached target directories (freed {})", size_str);
            } else {
                println!("Purge cancelled.");
            }
        } else {
            println!("No cached target directories found to purge");
        }
    }

    Ok(())
}
