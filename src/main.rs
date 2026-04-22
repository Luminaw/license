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
    version,
    author
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
    if let Some(license) = spdx::license_id(&id.to_uppercase()) {
        return Some(license);
    }
    if let Some(license) = spdx::license_id(&id.to_lowercase()) {
        return Some(license);
    }

    // Try title case
    let mut chars = id.chars();
    if let Some(f) = chars.next() {
        let mut s = f.to_uppercase().collect::<String>();
        s.push_str(&chars.as_str().to_lowercase());
        if let Some(license) = spdx::license_id(&s) {
            return Some(license);
        }
    }

    None
}

fn get_templates_dir() -> anyhow::Result<PathBuf> {
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
    let templates_dir = get_templates_dir().ok()?;
    let path = templates_dir.join(name);
    fs::read_to_string(path).ok()
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let mut cfg: Config = confy::load("license-manager", None).context("Failed to load config")?;

    let osi_only = cfg.osi_approved_only && !args.ignore_osi_approved;

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
                if cfg.author_name != "Your Name" {
                    match &cfg.author_email {
                        Some(email) if !email.is_empty() => {
                            format!("{} <{}>", cfg.author_name, email)
                        }
                        _ => cfg.author_name.clone(),
                    }
                } else if let Some(proj_author) = &project_info.author {
                    proj_author.clone()
                } else {
                    cfg.author_name.clone()
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
        Commands::Config { key, value } => match (key.as_deref(), value) {
            (None, _) => {
                println!("{HEADER}Current Configuration:{RESET}");
                println!("{:#?}", cfg);
            }
            (Some("path"), _) => {
                let path = confy::get_configuration_file_path("license-manager", None)
                    .context("Failed to get config path")?;
                println!("{HEADER}Config file location:{RESET}");
                println!("{}", path.display());
            }
            (Some("name"), None) | (Some("author_name"), None) => {
                println!("{}", cfg.author_name);
            }
            (Some("name"), Some(v)) | (Some("author_name"), Some(v)) => {
                cfg.author_name = v;
                confy::store("license-manager", None, &cfg).context("Failed to store config")?;
                println!("{}Updated author_name{}", SUCCESS, RESET);
            }
            (Some("email"), None) | (Some("author_email"), None) => {
                println!("{}", cfg.author_email.as_deref().unwrap_or("Not set"));
            }
            (Some("email"), Some(v)) | (Some("author_email"), Some(v)) => {
                cfg.author_email = Some(v);
                confy::store("license-manager", None, &cfg).context("Failed to store config")?;
                println!("{}Updated author_email{}", SUCCESS, RESET);
            }
            (Some("osi_approved_only"), Some(v)) => {
                cfg.osi_approved_only = v.parse().context("Value must be 'true' or 'false'")?;
                confy::store("license-manager", None, &cfg).context("Failed to store config")?;
                println!("{}Updated osi_approved_only to {}{}", SUCCESS, cfg.osi_approved_only, RESET);
            }
            (Some(k), _) => {
                anyhow::bail!("{}Unknown config key: {}{}", ERROR, k, RESET);
            }
        },
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
