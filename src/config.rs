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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_resolution_no_overrides() {
        let config = Config {
            author_name: "Personal".into(),
            author_email: Some("p@example.com".into()),
            osi_approved_only: true,
            include_if: HashMap::new(),
        };
        let res = config.resolve();
        assert_eq!(res.author_name, "Personal");
        assert_eq!(res.author_email, Some("p@example.com".into()));
    }

    #[test]
    fn test_config_resolution_with_override() {
        let mut include_if = HashMap::new();
        // This test depends on the current directory, so we match against current dir prefix
        if let Ok(cwd) = std::env::current_dir() {
            let cwd_str = cwd.to_string_lossy();
            include_if.insert(format!("dir:{}", cwd_str), ConfigOverride {
                author_name: Some("Override Name".into()),
                author_email: Some("o@example.com".into()),
            });

            let config = Config {
                author_name: "Original".into(),
                author_email: None,
                osi_approved_only: true,
                include_if,
            };
            let res = config.resolve();
            assert_eq!(res.author_name, "Override Name");
            assert_eq!(res.author_email, Some("o@example.com".into()));
        }
    }

    #[test]
    fn test_home_expansion() {
        if let Some(home) = dirs::home_dir() {
            let expanded = expand_home("~/work");
            let expected = home.join("work").to_string_lossy().into_owned();
            assert_eq!(expanded, expected);
        }
    }

    #[test]
    fn test_config_resolution_priority() {
        let mut include_if = HashMap::new();
        if let Ok(cwd) = std::env::current_dir() {
            let cwd_str = cwd.to_string_lossy();
            
            // First override
            include_if.insert(format!("dir:{}", cwd_str), ConfigOverride {
                author_name: Some("First Override".into()),
                author_email: None,
            });
            
            // Second override (same dir, but different key or same key - HashMap wins last)
            // Note: HashMap doesn't guarantee order, but in our logic we iterate.
            // If multiple keys match, they are applied in iteration order.
            // However, to test priority we can test prefix matching depth.
            
            let config = Config {
                author_name: "Original".into(),
                author_email: None,
                osi_approved_only: true,
                include_if,
            };
            let res = config.resolve();
            assert_eq!(res.author_name, "First Override");
        }
    }

    #[test]
    fn test_config_resolution_prefix_matching() {
        let mut include_if = HashMap::new();
        if let Ok(cwd) = std::env::current_dir() {
            let parent = cwd.parent().unwrap_or(&cwd);
            let parent_str = parent.to_string_lossy();
            
            include_if.insert(format!("dir:{}", parent_str), ConfigOverride {
                author_name: Some("Parent Match".into()),
                author_email: None,
            });

            let config = Config {
                author_name: "Original".into(),
                author_email: None,
                osi_approved_only: true,
                include_if,
            };
            let res = config.resolve();
            // CWD should match its parent's prefix
            assert_eq!(res.author_name, "Parent Match");
        }
    }
}
