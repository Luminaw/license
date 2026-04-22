// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Luminaw

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub author_name: String,
    pub author_email: Option<String>,
    #[serde(default = "default_osi_approved")]
    pub osi_approved_only: bool,
    #[serde(default)]
    pub include_if: HashMap<String, ConfigOverride>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConfigOverride {
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

fn default_osi_approved() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            author_name: "Your Name".into(),
            author_email: None,
            osi_approved_only: true,
            include_if: HashMap::new(),
        }
    }
}

impl Config {
    pub fn resolve(&self) -> ResolvedConfig {
        let mut name = self.author_name.clone();
        let mut email = self.author_email.clone();

        if let Ok(cwd) = std::env::current_dir() {
            let cwd_str = cwd.to_string_lossy();

            for (condition, overrides) in &self.include_if {
                if let Some(dir_pattern) = condition.strip_prefix("dir:") {
                    let expanded_pattern = expand_home(dir_pattern);
                    let pattern_path = Path::new(&expanded_pattern);

                    // Check if CWD is inside the pattern path
                    if cwd.starts_with(pattern_path) || cwd_str.starts_with(&expanded_pattern) {
                        if let Some(n) = &overrides.author_name {
                            name = n.clone();
                        }
                        if let Some(e) = &overrides.author_email {
                            email = Some(e.clone());
                        }
                    }
                }
            }
        }

        ResolvedConfig {
            author_name: name,
            author_email: email,
            osi_approved_only: self.osi_approved_only,
        }
    }
}

pub struct ResolvedConfig {
    pub author_name: String,
    pub author_email: Option<String>,
    pub osi_approved_only: bool,
}

fn expand_home(path: &str) -> String {
    if let Some(suffix) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(suffix).to_string_lossy().into_owned();
    }
    path.to_string()
}
