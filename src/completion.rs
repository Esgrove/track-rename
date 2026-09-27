use std::path::PathBuf;

use clap::Command as ClapCommand;
use clap_complete::Shell;

/// Generate a shell completion script for the given shell.
///
/// # Errors
///
/// Returns an error when installing the script fails to locate or create a
/// completion directory, or when the generated file cannot be written.
pub fn generate_shell_completion(
    shell: Shell,
    mut command: ClapCommand,
    install: bool,
    verbose: bool,
    command_name: &str,
) -> anyhow::Result<()> {
    if install {
        let output_directory = get_shell_completion_directory(shell, command_name)?;
        let path = clap_complete::generate_to(shell, &mut command, command_name, output_directory)?;
        if verbose {
            println!("Completion file generated to: {}", path.display());
        }
    } else {
        clap_complete::generate(shell, &mut command, command_name, &mut std::io::stdout());
    }
    Ok(())
}

/// Determine the appropriate directory for storing shell completions.
///
/// First checks if the user-specific directory exists,
/// then checks for the global directory.
/// If neither exist, creates and uses the user-specific directory.
fn get_shell_completion_directory(shell: Shell, name: &str) -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().expect("Failed to get home directory");

    // Special handling for oh-my-zsh.
    // Create custom "plugin", which will then have to be loaded in .zshrc
    if shell == Shell::Zsh {
        let omz_plugins = home.join(".oh-my-zsh/custom/plugins");
        if omz_plugins.exists() {
            let plugin_directory = omz_plugins.join(name);
            std::fs::create_dir_all(&plugin_directory)?;
            return Ok(plugin_directory);
        }
    }

    let user_directory = match shell {
        Shell::PowerShell => {
            if cfg!(windows) {
                home.join(r"Documents\PowerShell\completions")
            } else {
                home.join(".config/powershell/completions")
            }
        }
        Shell::Bash => home.join(".bash_completion.d"),
        Shell::Elvish => home.join(".elvish"),
        Shell::Fish => home.join(".config/fish/completions"),
        Shell::Zsh => home.join(".zsh/completions"),
        _ => anyhow::bail!("Unsupported shell"),
    };

    if user_directory.exists() {
        return Ok(user_directory);
    }

    let global_directory = match shell {
        Shell::Bash => PathBuf::from("/etc/bash_completion.d"),
        Shell::Fish => PathBuf::from("/usr/share/fish/completions"),
        Shell::Zsh => PathBuf::from("/usr/share/zsh/site-functions"),
        // PowerShell and Elvish don't have standard global directories;
        // fall through to creating the user directory.
        _ => {
            std::fs::create_dir_all(&user_directory)?;
            return Ok(user_directory);
        }
    };

    if global_directory.exists() {
        return Ok(global_directory);
    }

    std::fs::create_dir_all(&user_directory)?;
    Ok(user_directory)
}
