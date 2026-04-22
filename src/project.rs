// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Luminaw

use std::fs;

pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
}

pub fn detect() -> ProjectInfo {
    let mut info = ProjectInfo {
        name: get_folder_name(),
        description: "A new project.".to_string(),
        author: None,
    };

    // 1. Rust (Cargo.toml)
    if let Ok(content) = fs::read_to_string("Cargo.toml")
        && let Ok(value) = content.parse::<toml::Value>()
        && let Some(package) = value.get("package")
    {
        if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
            info.name = name.to_string();
        }
        if let Some(desc) = package.get("description").and_then(|v| v.as_str()) {
            info.description = desc.to_string();
        }
        if let Some(authors) = package.get("authors").and_then(|v| v.as_array())
            && let Some(first_author) = authors.first().and_then(|v| v.as_str())
        {
            info.author = Some(first_author.to_string());
        }
        return info;
    }

    // 2. Node.js (package.json)
    if let Ok(content) = fs::read_to_string("package.json")
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&content)
    {
        if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
            info.name = name.to_string();
        }
        if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
            info.description = desc.to_string();
        }
        if let Some(author) = value.get("author") {
            if let Some(s) = author.as_str() {
                info.author = Some(s.to_string());
            } else if let Some(obj) = author.as_object()
                && let Some(name) = obj.get("name").and_then(|v| v.as_str())
            {
                info.author = Some(name.to_string());
            }
        }
        return info;
    }

    // 3. PHP (composer.json)
    if let Ok(content) = fs::read_to_string("composer.json")
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&content)
    {
        if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
            info.name = name.to_string();
        }
        if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
            info.description = desc.to_string();
        }
        if let Some(authors) = value.get("authors").and_then(|v| v.as_array())
            && let Some(first) = authors.first().and_then(|v| v.as_object())
            && let Some(name) = first.get("name").and_then(|v| v.as_str())
        {
            info.author = Some(name.to_string());
        }
        return info;
    }

    // 4. Python (pyproject.toml)
    if let Ok(content) = fs::read_to_string("pyproject.toml")
        && let Ok(value) = content.parse::<toml::Value>()
    {
        // Try [project] (PEP 621)
        if let Some(project) = value.get("project") {
            if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                info.name = name.to_string();
            }
            if let Some(desc) = project.get("description").and_then(|v| v.as_str()) {
                info.description = desc.to_string();
            }
            if let Some(authors) = project.get("authors").and_then(|v| v.as_array())
                && let Some(first) = authors
                    .first()
                    .and_then(|v| v.get("name"))
                    .and_then(|v| v.as_str())
            {
                info.author = Some(first.to_string());
            }
            return info;
        }
        // Try [tool.poetry]
        if let Some(poetry) = value.get("tool").and_then(|v| v.get("poetry")) {
            if let Some(name) = poetry.get("name").and_then(|v| v.as_str()) {
                info.name = name.to_string();
            }
            if let Some(desc) = poetry.get("description").and_then(|v| v.as_str()) {
                info.description = desc.to_string();
            }
            if let Some(authors) = poetry.get("authors").and_then(|v| v.as_array())
                && let Some(first) = authors.first().and_then(|v| v.as_str())
            {
                info.author = Some(first.to_string());
            }
            return info;
        }
    }

    // 5. Dart / Flutter (pubspec.yaml)
    if let Ok(content) = fs::read_to_string("pubspec.yaml")
        && let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content)
    {
        if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
            info.name = name.to_string();
        }
        if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
            info.description = desc.to_string();
        }
        if let Some(author) = value.get("author").and_then(|v| v.as_str()) {
            info.author = Some(author.to_string());
        }
        return info;
    }

    // 6. C# / .NET (*.csproj)
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension()
                && ext == "csproj"
                && let Ok(content) = fs::read_to_string(entry.path())
            {
                // Name
                if let Some(start) = content.find("<AssemblyName>") {
                    let start_idx = start + 14;
                    if let Some(end) = content[start_idx..].find("</AssemblyName>") {
                        info.name = content[start_idx..start_idx + end].to_string();
                    }
                }
                // Description
                if let Some(start) = content.find("<Description>") {
                    let start_idx = start + 13;
                    if let Some(end) = content[start_idx..].find("</Description>") {
                        info.description = content[start_idx..start_idx + end].to_string();
                    }
                }
                // Author
                if let Some(start) = content.find("<Authors>") {
                    let start_idx = start + 9;
                    if let Some(end) = content[start_idx..].find("</Authors>") {
                        info.author = Some(content[start_idx..start_idx + end].to_string());
                    }
                }
                return info;
            }
        }
    }

    // 7. Java / Maven (pom.xml)
    if let Ok(content) = fs::read_to_string("pom.xml") {
        if let Some(start) = content.find("<artifactId>") {
            let start_idx = start + 12;
            if let Some(end) = content[start_idx..].find("</artifactId>") {
                info.name = content[start_idx..start_idx + end].to_string();
            }
        }
        if let Some(start) = content.find("<description>") {
            let start_idx = start + 13;
            if let Some(end) = content[start_idx..].find("</description>") {
                info.description = content[start_idx..start_idx + end].trim().to_string();
            }
        }
        return info;
    }

    // 8. Fallback: README.md
    if let Ok(content) = fs::read_to_string("README.md") {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                info.name = stripped.to_string();
            } else if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && info.description == "A new project."
            {
                info.description = trimmed.to_string();
            }
        }
    }

    info
}

fn get_folder_name() -> String {
    std::env::current_dir()
        .ok()
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "project".to_string())
}
