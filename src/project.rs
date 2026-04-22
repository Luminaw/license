// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Luminaw

use std::fs;
use std::path::Path;

pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
}

pub fn detect() -> ProjectInfo {
    detect_in(Path::new("."))
}

pub fn detect_in(root: &Path) -> ProjectInfo {
    let mut info = ProjectInfo {
        name: get_folder_name(root),
        description: "A new project.".to_string(),
        author: None,
    };

    // 1. Rust (Cargo.toml)
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml"))
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
    if let Ok(content) = fs::read_to_string(root.join("package.json"))
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
    if let Ok(content) = fs::read_to_string(root.join("composer.json"))
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
    if let Ok(content) = fs::read_to_string(root.join("pyproject.toml"))
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
    if let Ok(content) = fs::read_to_string(root.join("pubspec.yaml"))
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
    if let Ok(entries) = fs::read_dir(root) {
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
    if let Ok(content) = fs::read_to_string(root.join("pom.xml")) {
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
    if let Ok(content) = fs::read_to_string(root.join("README.md")) {
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

fn get_folder_name(root: &Path) -> String {
    if root == Path::new(".") {
        std::env::current_dir()
            .ok()
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "project".to_string())
    } else {
        root.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "project".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rust_detection() {
        let dir = tempdir().unwrap();
        let cargo_toml = r#"
            [package]
            name = "test-rust"
            description = "A rust test project"
            authors = ["Rust Author"]
        "#;
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-rust");
        assert_eq!(info.description, "A rust test project");
        assert_eq!(info.author, Some("Rust Author".to_string()));
    }

    #[test]
    fn test_node_detection() {
        let dir = tempdir().unwrap();
        let package_json = r#"{
            "name": "test-node",
            "description": "A node test project",
            "author": "Node Author"
        }"#;
        fs::write(dir.path().join("package.json"), package_json).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-node");
        assert_eq!(info.description, "A node test project");
        assert_eq!(info.author, Some("Node Author".to_string()));
    }

    #[test]
    fn test_python_poetry_detection() {
        let dir = tempdir().unwrap();
        let pyproject_toml = r#"
            [tool.poetry]
            name = "test-python"
            description = "A python test project"
            authors = ["Python Author"]
        "#;
        fs::write(dir.path().join("pyproject.toml"), pyproject_toml).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-python");
        assert_eq!(info.description, "A python test project");
        assert_eq!(info.author, Some("Python Author".to_string()));
    }

    #[test]
    fn test_php_detection() {
        let dir = tempdir().unwrap();
        let composer_json = r#"{
            "name": "test-php",
            "description": "A php test project",
            "authors": [{"name": "PHP Author"}]
        }"#;
        fs::write(dir.path().join("composer.json"), composer_json).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-php");
        assert_eq!(info.description, "A php test project");
        assert_eq!(info.author, Some("PHP Author".to_string()));
    }

    #[test]
    fn test_dart_detection() {
        let dir = tempdir().unwrap();
        let pubspec_yaml = "name: test-dart\ndescription: A dart test project\nauthor: Dart Author";
        fs::write(dir.path().join("pubspec.yaml"), pubspec_yaml).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-dart");
        assert_eq!(info.description, "A dart test project");
        assert_eq!(info.author, Some("Dart Author".to_string()));
    }

    #[test]
    fn test_csharp_detection() {
        let dir = tempdir().unwrap();
        let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
            <PropertyGroup>
                <AssemblyName>test-csharp</AssemblyName>
                <Description>A csharp test project</Description>
                <Authors>CSharp Author</Authors>
            </PropertyGroup>
        </Project>"#;
        fs::write(dir.path().join("test.csproj"), csproj).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-csharp");
        assert_eq!(info.description, "A csharp test project");
        assert_eq!(info.author, Some("CSharp Author".to_string()));
    }

    #[test]
    fn test_java_detection() {
        let dir = tempdir().unwrap();
        let pom_xml = r#"<project>
            <artifactId>test-java</artifactId>
            <description>A java test project</description>
        </project>"#;
        fs::write(dir.path().join("pom.xml"), pom_xml).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "test-java");
        assert_eq!(info.description, "A java test project");
    }

    #[test]
    fn test_readme_fallback() {
        let dir = tempdir().unwrap();
        let readme = "# README Name\nThis is a description from README.";
        fs::write(dir.path().join("README.md"), readme).unwrap();

        let info = detect_in(dir.path());
        assert_eq!(info.name, "README Name");
        assert_eq!(info.description, "This is a description from README.");
    }
}
