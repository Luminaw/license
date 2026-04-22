# license

A fast and simple license manager for your projects, written in Rust.

[![CI](https://github.com/Luminaw/license/actions/workflows/ci.yml/badge.svg)](https://github.com/Luminaw/license/actions)
[![Release](https://github.com/Luminaw/license/actions/workflows/release.yml/badge.svg)](https://github.com/Luminaw/license/releases)

## Features

- **Quick Add**: Add licenses (MIT, Apache-2.0, etc.) to your project with a single command.
- **Auto-Fill**: Automatically fills in the current year, author name, and project description.
- **Smart Autodetection**: Automatically detects project metadata from Rust, Node.js, Python, PHP, C#, Java, and Flutter projects.
- **SPDX Integration**: Uses the official SPDX license list for accurate and up-to-date license texts.
- **Configuration**: Save your author name and email to avoid re-typing them.
- **Searchable**: List and filter hundreds of available licenses.
- **Info Preview**: View license details and OSI compliance status before adding.

## Supported Project Types

The tool automatically detects the project name, description, and author from the following files:
- **Rust**: `Cargo.toml`
- **Node.js**: `package.json`
- **PHP**: `composer.json`
- **Python**: `pyproject.toml`
- **Dart / Flutter**: `pubspec.yaml`
- **C# / .NET**: `*.csproj`
- **Java / Maven**: `pom.xml`
- **Fallback**: `README.md` (detects name and first paragraph)

## Installation

### From Source

```bash
cargo install --path .
```

### From Releases

Download the pre-compiled binary for your platform from the [Releases](https://github.com/USER_NAME/license/releases) page.

## Usage

### Configuration

Set your author information:

```bash
license config name "Your Name"
license config email "your.email@example.com"
```

Show current configuration and file path:

```bash
license config
license config path
```

### Adding Licenses

Add a single license:

```bash
license add mit
```

Add multiple licenses:

```bash
license add mit apache-2.0
```

Overwrite existing files:

```bash
license add mit --force
```

Override default values:

```bash
license add mit --year 2023 --name "Company Name"
```

### Finding Licenses

List all available licenses:

```bash
license list
```

Search for a license:

```bash
license list "gnu"
```

View detailed information:

```bash
license info mit
```

### Custom Templates

You can register your own license templates (e.g., for internal company licenses):

```bash
# Register a template
license create my-license ./path/to/my-license.txt

# Use it
license add my-license
```

Custom templates support the same auto-fill placeholders as standard licenses (`<year>`, `<owner>`, etc.).

### Shell Completions

Generate completion scripts for your favorite shell:

```bash
# Bash
license completions bash > /etc/bash_completion.d/license

# Zsh
license completions zsh > /usr/local/share/zsh/site-functions/_license

# Fish
license completions fish > ~/.config/fish/completions/license.fish

# PowerShell
license completions powershell > license.ps1
```

## CI/CD

This project uses GitHub Actions for:
- **CI**: Automated testing and linting on every push to `master`.
- **Release**: Automated binary builds and GitHub Releases on version tags (e.g., `v0.1.0`).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
