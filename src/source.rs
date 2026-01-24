//! Locate alias definitions in shell configuration files

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::shell::Shell;

/// Result of locating an alias
#[derive(Debug, Clone)]
pub struct AliasLocation {
    pub file: PathBuf,
    pub line: usize,
}

/// Get list of rc files to scan for a given shell
pub fn get_rc_files(shell: Shell) -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    match shell {
        Shell::Zsh => vec![
            home.join(".zshrc"),
            home.join(".zshenv"),
            home.join(".zprofile"),
            home.join(".config/zsh/.zshrc"),
            home.join(".oh-my-zsh/custom"), // Will scan *.zsh
            home.join(".config/zsh"),
        ],
        Shell::Bash => vec![
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".bash_aliases"),
            home.join(".profile"),
        ],
        Shell::Fish => vec![
            home.join(".config/fish/config.fish"),
            home.join(".config/fish/conf.d"), // Will scan *.fish
        ],
    }
}

/// Scan a file for alias definitions, following `source` includes
fn find_alias_in_file(
    alias_name: &str,
    file: &Path,
    shell: Shell,
    visited: &mut HashSet<PathBuf>,
) -> Option<AliasLocation> {
    // Prevent infinite loops from circular includes
    let canonical = file.canonicalize().ok()?;
    if visited.contains(&canonical) || !file.exists() {
        return None;
    }
    visited.insert(canonical);

    let content = fs::read_to_string(file).ok()?;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Check for alias definition
        if is_alias_definition(trimmed, alias_name, shell) {
            return Some(AliasLocation {
                file: file.to_path_buf(),
                line: line_num + 1,
            });
        }

        // Follow source/. includes
        if let Some(sourced_file) = parse_source_line(trimmed, file)
            && let Some(loc) =
                find_alias_in_file(alias_name, &sourced_file, shell, visited)
        {
            return Some(loc);
        }
    }

    None
}

/// Check if a line defines the given alias
fn is_alias_definition(line: &str, alias_name: &str, shell: Shell) -> bool {
    // Skip comments
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return false;
    }

    let line_lower = line.to_lowercase();

    match shell {
        Shell::Zsh | Shell::Bash => {
            // Patterns:
            // alias gs="..."
            // alias gs='...'
            // alias -g GS="..." (zsh global)
            let patterns = [
                format!("alias {alias_name}="),
                format!("alias '{alias_name}'="),
                format!("alias \"{alias_name}\"="),
                format!("alias -g {alias_name}="),
                // Case insensitive check for global aliases
                format!("alias {}=", alias_name.to_lowercase()),
            ];

            patterns.iter().any(|p| {
                line.starts_with(p)
                    || line.contains(&format!(" {p}"))
                    || line.contains(&format!(";{p}"))
                    || line_lower.starts_with(&p.to_lowercase())
            })
        }
        Shell::Fish => {
            // Patterns:
            // abbr -a gs '...'
            // abbr gs '...'
            // alias gs '...'
            // alias gs='...'
            let patterns = [
                format!("abbr -a {alias_name} "),
                format!("abbr {alias_name} "),
                format!("alias {alias_name} "),
                format!("alias {alias_name}="),
            ];

            patterns.iter().any(|p| {
                line.starts_with(p) || line.contains(&format!(" {p}"))
            })
        }
    }
}

/// Parse source/. lines to get included file path
fn parse_source_line(line: &str, current_file: &Path) -> Option<PathBuf> {
    let trimmed = line.trim();

    // Skip comments
    if trimmed.starts_with('#') {
        return None;
    }

    // Extract the path from various source patterns
    let path_str = if let Some(rest) = trimmed.strip_prefix("source ") {
        Some(rest.trim())
    } else if trimmed.starts_with(". ") && !trimmed.starts_with("..") {
        trimmed.strip_prefix(". ").map(str::trim)
    } else if trimmed.contains("&& source ") {
        trimmed.split("&& source ").nth(1).map(str::trim)
    } else if trimmed.contains("&& . ") {
        trimmed
            .split("&& . ")
            .nth(1)
            .map(str::trim)
            .filter(|s| !s.starts_with('.'))
    } else {
        None
    }?;

    // Clean up the path (remove trailing comments, quotes, etc.)
    let path_str = path_str
        .split('#')
        .next()
        .unwrap_or(path_str)
        .split(';')
        .next()
        .unwrap_or(path_str)
        .trim();

    // Expand ~ and $HOME
    let expanded = expand_path(path_str);

    // Handle relative paths
    if expanded.is_relative() {
        current_file.parent().map(|p| p.join(&expanded))
    } else {
        Some(expanded)
    }
}

/// Expand ~ and $HOME in path
fn expand_path(path: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy();

    let cleaned = path.trim_matches('"').trim_matches('\'');

    let expanded =
        cleaned.replace("$HOME", &home_str).replace("${HOME}", &home_str);

    if let Some(rest) = expanded.strip_prefix("~/") {
        home.join(rest)
    } else if expanded == "~" {
        home
    } else {
        PathBuf::from(expanded)
    }
}

/// Check if file has appropriate extension for shell
fn is_shell_file(path: &Path, shell: Shell) -> bool {
    let ext = path.extension().and_then(|e| e.to_str());
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

    match shell {
        Shell::Zsh => ext == Some("zsh") || name.starts_with('.'),
        Shell::Bash => {
            ext == Some("sh") || ext == Some("bash") || name.starts_with('.')
        }
        Shell::Fish => ext == Some("fish"),
    }
}

/// Main entry point: find where an alias is defined
pub fn locate_alias(alias_name: &str, shell: Shell) -> Option<AliasLocation> {
    let rc_files = get_rc_files(shell);
    let mut visited = HashSet::new();

    for file in rc_files {
        if file.is_dir() {
            // Scan directory for shell files
            if let Ok(entries) = fs::read_dir(&file) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && is_shell_file(&path, shell)
                        && let Some(loc) = find_alias_in_file(
                            alias_name,
                            &path,
                            shell,
                            &mut visited,
                        )
                    {
                        return Some(loc);
                    }
                }
            }
        } else if let Some(loc) =
            find_alias_in_file(alias_name, &file, shell, &mut visited)
        {
            return Some(loc);
        }
    }

    None
}

/// Build a map of all alias locations (for caching)
pub fn locate_all_aliases(
    alias_names: &[&str],
    shell: Shell,
) -> std::collections::HashMap<String, AliasLocation> {
    let mut locations = std::collections::HashMap::new();

    for name in alias_names {
        if let Some(loc) = locate_alias(name, shell) {
            locations.insert((*name).to_owned(), loc);
        }
    }

    locations
}

/// Format location for display (shortens home path)
pub fn format_location(loc: &AliasLocation) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let display_path = loc.file.strip_prefix(&home).map_or_else(
        |_| loc.file.display().to_string(),
        |p| format!("~/{}", p.display()),
    );

    format!("{}:{}", display_path, loc.line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alias_definition_zsh() {
        assert!(is_alias_definition(
            "alias gs=\"git status\"",
            "gs",
            Shell::Zsh
        ));
        assert!(is_alias_definition(
            "alias gs='git status'",
            "gs",
            Shell::Zsh
        ));
        assert!(is_alias_definition(
            "alias -g GS=\"git status\"",
            "GS",
            Shell::Zsh
        ));
        assert!(!is_alias_definition(
            "alias ga=\"git add\"",
            "gs",
            Shell::Zsh
        ));
        assert!(!is_alias_definition(
            "# alias gs=\"git status\"",
            "gs",
            Shell::Zsh
        ));
    }

    #[test]
    fn test_is_alias_definition_fish() {
        assert!(is_alias_definition(
            "abbr -a gs 'git status'",
            "gs",
            Shell::Fish
        ));
        assert!(is_alias_definition(
            "abbr gs 'git status'",
            "gs",
            Shell::Fish
        ));
        assert!(is_alias_definition(
            "alias gs 'git status'",
            "gs",
            Shell::Fish
        ));
    }

    #[test]
    fn test_expand_path() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("~/.zshrc"), home.join(".zshrc"));
        assert_eq!(expand_path("$HOME/.zshrc"), home.join(".zshrc"));
        assert_eq!(expand_path("${HOME}/.zshrc"), home.join(".zshrc"));
        assert_eq!(expand_path("/etc/profile"), PathBuf::from("/etc/profile"));
    }

    #[test]
    fn test_format_location() {
        let home = dirs::home_dir().unwrap_or_default();
        let loc = AliasLocation { file: home.join(".zshrc"), line: 42 };
        assert_eq!(format_location(&loc), "~/.zshrc:42");
    }
}
