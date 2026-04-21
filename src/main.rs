mod config;

use clap::{Parser, Subcommand};
use config::Config;
use std::fs;

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
}

fn get_project_description(project_name: &str) -> String {
    if let Ok(content) = fs::read_to_string("Cargo.toml") {
        if let Ok(value) = content.parse::<toml::Value>() {
            if let Some(desc) = value
                .get("package")
                .and_then(|p| p.get("description"))
                .and_then(|d| d.as_str())
            {
                return desc.to_string();
            }
        }
    }
    format!("{}: A new project.", project_name)
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

fn main() {
    let args = Cli::parse();

    let mut cfg: Config = confy::load("license-manager", None).expect("Failed to load config");

    match args.command {
        Commands::Add {
            license_ids,
            force,
            year,
            name,
        } => {
            let year = year.unwrap_or_else(|| chrono::Local::now().format("%Y").to_string());
            let author = name.unwrap_or_else(|| match &cfg.author_email {
                Some(email) if !email.is_empty() => format!("{} <{}>", cfg.author_name, email),
                _ => cfg.author_name.clone(),
            });

            let current_dir = std::env::current_dir().ok();
            let project_name = current_dir
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("project");

            let description = get_project_description(project_name);

            for id in &license_ids {
                if let Some(license) = find_license(id) {
                    let mut text = license.text().to_string();

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

                    text = text.replace("<program>", project_name)
                        .replace("[program]", project_name)
                        .replace("<one line to give the program's name and a brief idea of what it does.>", &description);

                    let filename = if license_ids.len() == 1 {
                        "LICENSE".to_string()
                    } else {
                        format!("LICENSE-{}", license.name)
                    };

                    if fs::metadata(&filename).is_ok() && !force {
                        println!(
                            "{}File {} already exists. Use --force to overwrite.{}",
                            ERROR, filename, RESET
                        );
                        continue;
                    }

                    fs::write(&filename, text).expect("Failed to write license file");
                    println!("{}Successfully created {}{}", SUCCESS, filename, RESET);
                } else {
                    eprintln!(
                        "{}Error: License identifier '{}' not found in SPDX list.{}",
                        ERROR, id, RESET
                    );
                }
            }
        }
        Commands::Config { key, value } => match (key.as_deref(), value) {
            (None, _) => {
                println!("{HEADER}Current Configuration:{RESET}");
                println!("{:#?}", cfg);
            }
            (Some("path"), _) => {
                let path = confy::get_configuration_file_path("license-manager", None)
                    .expect("Failed to get config path");
                println!("{HEADER}Config file location:{RESET}");
                println!("{}", path.display());
            }
            (Some("name"), None) | (Some("author_name"), None) => {
                println!("{}", cfg.author_name);
            }
            (Some("name"), Some(v)) | (Some("author_name"), Some(v)) => {
                cfg.author_name = v;
                confy::store("license-manager", None, &cfg).expect("Failed to store config");
                println!("{}Updated author_name{}", SUCCESS, RESET);
            }
            (Some("email"), None) | (Some("author_email"), None) => {
                println!("{}", cfg.author_email.as_deref().unwrap_or("Not set"));
            }
            (Some("email"), Some(v)) | (Some("author_email"), Some(v)) => {
                cfg.author_email = Some(v);
                confy::store("license-manager", None, &cfg).expect("Failed to store config");
                println!("{}Updated author_email{}", SUCCESS, RESET);
            }
            (Some(k), _) => {
                eprintln!("{}Unknown config key: {}{}", ERROR, k, RESET);
                std::process::exit(1);
            }
        },
        Commands::List { query } => {
            let query = query.map(|q| q.to_lowercase());

            println!("{}{:<20} | {:<50}{}", HEADER, "ID", "Full Name", RESET);
            println!("{DIM}{:-<20}-|-{:-<50}{}", "", "", RESET);

            let mut licenses: Vec<_> = spdx::identifiers::LICENSES.iter().collect();
            licenses.sort_by_key(|l| l.name);

            for license in licenses {
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
            if let Some(license) = find_license(&id) {
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
    }
}
