// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Luminaw

mod config;
mod project;

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use config::Config;
use std::fs;
use std::path::PathBuf;

use anstyle::{AnsiColor, Color, Style};

const HEADER: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightBlue)))
    .bold();
const SUCCESS: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)));
const ERROR: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed)));
const DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
const RESET: Style = Style::new();

#[derive(Parser)]
#[command(
    name = "license",
    about = "A fast and simple license manager for your projects.",
    author,
    version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\nCopyright (C) 2026 Luminaw\n",
        "This program comes with ABSOLUTELY NO WARRANTY.\n",
        "This is free software, and you are welcome to redistribute it\n",
        "under certain conditions."
    )
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Show all licenses, even if not OSI approved.
    #[arg(long, global = true)]
    ignore_osi_approved: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Add one or more licenses to your project.
    ///
    /// Example: license add mit apache-2.0
    Add {
        /// The SPDX identifier(s) of the license(s) to add (e.g., mit, apache-2.0).
        #[arg(required = true)]
        license_ids: Vec<String>,

        /// Overwrite existing LICENSE files without asking.
        #[arg(short, long)]
        force: bool,

        /// Override the copyright year (defaults to current year).
        #[arg(short, long)]
        year: Option<String>,

        /// Override the author name (defaults to config value).
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Get or set configuration values.
    ///
    /// If no arguments are provided, it prints the full configuration.
    /// If only a key is provided, it prints the value for that key.
    /// If both a key and a value are provided, it sets the key to that value.
    ///
    /// Special key: "path" - shows the location of the config file.
    /// Supported keys: name, author_name, email, author_email.
    Config {
        /// The config key to get or set.
        key: Option<String>,
        /// The value to assign to the key.
        value: Option<String>,
    },
    /// List all available licenses from the SPDX list.
    List {
        /// Optional query to filter licenses by name or ID.
        query: Option<String>,
    },
    /// View detailed information and text for a specific license.
    Info {
        /// The SPDX identifier of the license (e.g., MIT).
        id: String,
    },
    /// Generate shell completion scripts.
    Completions {
        /// The shell to generate completions for (bash, zsh, fish, powershell, elvish).
        shell: clap_complete::Shell,
    },
    /// Register a custom license template from a file.
    Create {
        /// The name to register the template as (e.g., "my-license").
        name: String,
        /// The path to the file containing the license text.
        file: PathBuf,
    },
}

fn find_license(id: &str) -> Option<spdx::LicenseId> {
    // Try exact match first
    if let Some(license) = spdx::license_id(id) {
        return Some(license);
    }

    // Try common variants
    let lower_id = id.to_lowercase();
    spdx::identifiers::LICENSES
        .iter()
        .find(|l| l.name.to_lowercase() == lower_id)
        .and_then(|l| spdx::license_id(l.name))
}

fn get_templates_dir() -> anyhow::Result<PathBuf> {
    get_templates_dir_internal(None)
}

fn get_templates_dir_internal(override_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(dir) = override_dir {
        if !dir.exists() {
            fs::create_dir_all(&dir).context("Failed to create override templates directory")?;
        }
        return Ok(dir);
    }

    let config_file = confy::get_configuration_file_path("license-manager", None)
        .context("Failed to get config path")?;
    let config_dir = config_file
        .parent()
        .context("Failed to get config directory")?;
    let templates_dir = config_dir.join("templates");
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir).context("Failed to create templates directory")?;
    }
    Ok(templates_dir)
}

fn get_custom_template(name: &str) -> Option<String> {
    get_custom_template_internal(name, None)
}

fn get_custom_template_internal(name: &str, override_dir: Option<PathBuf>) -> Option<String> {
    let templates_dir = get_templates_dir_internal(override_dir).ok()?;
    let path = templates_dir.join(name);
    fs::read_to_string(path).ok()
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let cfg: Config = confy::load("license-manager", None).context("Failed to load config")?;
    let resolved_cfg = cfg.resolve();

    let osi_only = resolved_cfg.osi_approved_only && !args.ignore_osi_approved;

    match args.command {
        Commands::Add {
            license_ids,
            force,
            year,
            name,
        } => {
            let year = year.unwrap_or_else(|| chrono::Local::now().format("%Y").to_string());
            let project_info = project::detect();

            let author = name.unwrap_or_else(|| {
                // Priority: 1. Config file (resolved), 2. Project metadata, 3. Default config value
                if resolved_cfg.author_name != "Your Name" {
                    match &resolved_cfg.author_email {
                        Some(email) if !email.is_empty() => {
                            format!("{} <{}>", resolved_cfg.author_name, email)
                        }
                        _ => resolved_cfg.author_name.clone(),
                    }
                } else if let Some(proj_author) = &project_info.author {
                    proj_author.clone()
                } else {
                    resolved_cfg.author_name.clone()
                }
            });

            for id in &license_ids {
                let (text, name_to_use) = if let Some(custom_text) = get_custom_template(id) {
                    if osi_only {
                        eprintln!(
                            "{}Error: License identifier '{}' is a custom template, but osi_approved_only is enabled.{}",
                            ERROR, id, RESET
                        );
                        continue;
                    }
                    (custom_text, id.to_string())
                } else if let Some(license) = find_license(id) {
                    if osi_only && !license.is_osi_approved() {
                        eprintln!(
                            "{}Error: License identifier '{}' is not OSI approved.{}",
                            ERROR, id, RESET
                        );
                        continue;
                    }
                    (license.text().to_string(), license.name.to_string())
                } else {
                    eprintln!(
                        "{}Error: License identifier '{}' not found in SPDX list or custom templates.{}",
                        ERROR, id, RESET
                    );
                    continue;
                };

                let mut text = text;
                text = text
                    .replace("<year>", &year)
                    .replace("[year]", &year)
                    .replace("[yyyy]", &year)
                    .replace("{year}", &year);

                text = text
                    .replace("<copyright holders>", &author)
                    .replace("[name of copyright owner]", &author)
                    .replace("<name of copyright owner>", &author)
                    .replace("<name of author>", &author)
                    .replace("[fullname]", &author)
                    .replace("[owner]", &author)
                    .replace("{owner}", &author);

                text = text
                    .replace("<program>", &project_info.name)
                    .replace("[program]", &project_info.name)
                    .replace(
                        "<one line to give the program's name and a brief idea of what it does.>",
                        &project_info.description,
                    );

                let filename = if license_ids.len() == 1 {
                    "LICENSE".to_string()
                } else {
                    format!("LICENSE-{}", name_to_use)
                };

                if fs::metadata(&filename).is_ok() && !force {
                    println!(
                        "{}File {} already exists. Use --force to overwrite.{}",
                        ERROR, filename, RESET
                    );
                    continue;
                }

                fs::write(&filename, text).context("Failed to write license file")?;
                println!("{}Successfully created {}{}", SUCCESS, filename, RESET);
            }
        }
        Commands::Config { key, value } => {
            let mut cfg: Config =
                confy::load("license-manager", None).context("Failed to load config")?;
            match (key.as_deref(), value) {
                (None, _) => {
                    println!("{HEADER}Base Configuration:{RESET}");
                    println!("{:#?}", cfg);
                    println!("\n{HEADER}Resolved Configuration (current directory):{RESET}");
                    let res = cfg.resolve();
                    println!("author_name:       {}", res.author_name);
                    println!("author_email:      {:?}", res.author_email);
                    println!("osi_approved_only: {}", res.osi_approved_only);
                }
                (Some("path"), _) => {
                    let path = confy::get_configuration_file_path("license-manager", None)
                        .context("Failed to get config path")?;
                    println!("{HEADER}Config file location:{RESET}");
                    println!("{}", path.display());
                }
                (Some("name") | Some("author_name"), Some(v)) => {
                    cfg.author_name = v;
                    confy::store("license-manager", None, &cfg)
                        .context("Failed to store config")?;
                    println!("{}Updated author_name{}", SUCCESS, RESET);
                }
                (Some("email") | Some("author_email"), Some(v)) => {
                    cfg.author_email = Some(v);
                    confy::store("license-manager", None, &cfg)
                        .context("Failed to store config")?;
                    println!("{}Updated author_email{}", SUCCESS, RESET);
                }
                (Some("osi_approved_only"), Some(v)) => {
                    cfg.osi_approved_only = v.parse().context("Value must be 'true' or 'false'")?;
                    confy::store("license-manager", None, &cfg)
                        .context("Failed to store config")?;
                    println!(
                        "{}Updated osi_approved_only to {}{}",
                        SUCCESS, cfg.osi_approved_only, RESET
                    );
                }
                (Some(k), _) => {
                    anyhow::bail!("{}Unknown config key: {}{}", ERROR, k, RESET);
                }
            }
        }
        Commands::List { query } => {
            let query = query.map(|q| q.to_lowercase());

            println!("{}{:<20} | {:<50}{}", HEADER, "ID", "Full Name", RESET);
            println!("{DIM}{:-<20}-|-{:-<50}{}", "", "", RESET);

            let mut licenses: Vec<_> = spdx::identifiers::LICENSES.iter().collect();
            licenses.sort_by_key(|l| l.name);

            // Custom templates
            if !osi_only
                && let Ok(templates_dir) = get_templates_dir()
                && let Ok(entries) = fs::read_dir(templates_dir)
            {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        let matches = match &query {
                            Some(q) => name.to_lowercase().contains(q),
                            None => true,
                        };
                        if matches {
                            println!("{:<20} | {:<50}", name, format!("{} (custom)", name));
                        }
                    }
                }
            }

            for license in licenses {
                if osi_only
                    && !find_license(license.name)
                        .map(|id| id.is_osi_approved())
                        .unwrap_or(false)
                {
                    continue;
                }

                let matches = match &query {
                    Some(q) => {
                        license.name.to_lowercase().contains(q)
                            || license.full_name.to_lowercase().contains(q)
                    }
                    None => true,
                };

                if matches {
                    println!("{:<20} | {:<50}", license.name, license.full_name);
                }
            }
        }
        Commands::Info { id } => {
            if let Some(custom_text) = get_custom_template(&id) {
                if osi_only {
                    eprintln!(
                        "{}Error: License identifier '{}' is a custom template, but osi_approved_only is enabled.{}",
                        ERROR, id, RESET
                    );
                    return Ok(());
                }
                println!("{HEADER}ID:{RESET}          {}", id);
                println!("{HEADER}Type:{RESET}        Custom Template");

                println!("\n{HEADER}License Text Preview:{RESET}");
                println!("{DIM}--------------------------------------------------{RESET}");
                let preview: String = custom_text.lines().take(15).collect::<Vec<_>>().join("\n");
                println!("{}", preview);
                if custom_text.lines().count() > 15 {
                    println!("{DIM}... (use 'add' to generate full file) ...{RESET}");
                }
                println!("{DIM}--------------------------------------------------{RESET}");
            } else if let Some(license) = find_license(&id) {
                if osi_only && !license.is_osi_approved() {
                    eprintln!(
                        "{}Error: License identifier '{}' is not OSI approved.{}",
                        ERROR, id, RESET
                    );
                    return Ok(());
                }
                println!("{HEADER}ID:{RESET}          {}", license.name);
                println!("{HEADER}Full Name:{RESET}   {}", license.full_name);
                println!(
                    "{HEADER}OSI Approved:{RESET} {}",
                    if license.is_osi_approved() {
                        "Yes"
                    } else {
                        "No"
                    }
                );

                println!("\n{HEADER}License Text Preview:{RESET}");
                println!("{DIM}--------------------------------------------------{RESET}");
                let preview: String = license
                    .text()
                    .lines()
                    .take(15)
                    .collect::<Vec<_>>()
                    .join("\n");
                println!("{}", preview);
                if license.text().lines().count() > 15 {
                    println!("{DIM}... (use 'add' to generate full file) ...{RESET}");
                }
                println!("{DIM}--------------------------------------------------{RESET}");
            } else {
                eprintln!(
                    "{}Error: License identifier '{}' not found.{}",
                    ERROR, id, RESET
                );
            }
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Create { name, file } => {
            let templates_dir = get_templates_dir()?;
            let dest = templates_dir.join(&name);
            fs::copy(&file, &dest).context("Failed to copy template file")?;
            println!(
                "{}Successfully registered custom template: {}{}",
                SUCCESS, name, RESET
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_license_casing() {
        assert!(find_license("mit").is_some());
        assert!(find_license("MIT").is_some());
        assert!(find_license("Mit").is_some());
        assert!(find_license("apache-2.0").is_some());
    }

    #[test]
    fn test_custom_templates() {
        let dir = tempdir().unwrap();
        let templates_dir = dir.path().join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        let template_name = "my-license";
        let template_content = "Custom License Text";
        fs::write(templates_dir.join(template_name), template_content).unwrap();

        let found = get_custom_template_internal(template_name, Some(templates_dir));
        assert_eq!(found, Some(template_content.to_string()));
    }

    #[test]
    fn test_osi_filter_logic() {
        // Test that find_license works and we can check is_osi_approved
        let mit = find_license("mit").unwrap();
        assert!(mit.is_osi_approved());

        let gpl = find_license("gpl-3.0-only").unwrap();
        assert!(gpl.is_osi_approved());

        // Note: It's hard to find a non-OSI license in the default SPDX list that we can depend on,
        // but we can at least verify that our logic for filtering works.
    }
}
