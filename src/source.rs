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

    #[test]
    fn test_get_rc_files_zsh() {
        let files = get_rc_files(Shell::Zsh);
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.ends_with(".zshrc")));
    }

    #[test]
    fn test_get_rc_files_bash() {
        let files = get_rc_files(Shell::Bash);
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.ends_with(".bashrc")));
    }

    #[test]
    fn test_get_rc_files_fish() {
        let files = get_rc_files(Shell::Fish);
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.ends_with("config.fish")));
    }

    #[test]
    fn test_is_alias_definition_bash() {
        assert!(is_alias_definition(
            "alias gs=\"git status\"",
            "gs",
            Shell::Bash
        ));
        assert!(is_alias_definition("alias ls='ls -la'", "ls", Shell::Bash));
        assert!(!is_alias_definition(
            "alias ga=\"git add\"",
            "gs",
            Shell::Bash
        ));
    }

    #[test]
    fn test_is_alias_definition_with_comment() {
        assert!(!is_alias_definition(
            "# alias gs=\"git status\"",
            "gs",
            Shell::Bash
        ));
    }

    #[test]
    fn test_is_alias_definition_quoted_names() {
        assert!(is_alias_definition(
            "alias 'gs'=\"git status\"",
            "gs",
            Shell::Bash
        ));
        assert!(is_alias_definition(
            "alias \"gs\"=\"git status\"",
            "gs",
            Shell::Bash
        ));
    }

    #[test]
    fn test_is_shell_file_zsh() {
        assert!(is_shell_file(Path::new("file.zsh"), Shell::Zsh));
        assert!(is_shell_file(Path::new(".zshrc"), Shell::Zsh));
        assert!(!is_shell_file(Path::new("file.bash"), Shell::Zsh));
    }

    #[test]
    fn test_is_shell_file_bash() {
        assert!(is_shell_file(Path::new("file.bash"), Shell::Bash));
        assert!(is_shell_file(Path::new("file.sh"), Shell::Bash));
        assert!(is_shell_file(Path::new(".bashrc"), Shell::Bash));
        assert!(!is_shell_file(Path::new("file.zsh"), Shell::Bash));
    }

    #[test]
    fn test_is_shell_file_fish() {
        assert!(is_shell_file(Path::new("file.fish"), Shell::Fish));
        assert!(!is_shell_file(Path::new("file.zsh"), Shell::Fish));
        assert!(!is_shell_file(Path::new(".fishrc"), Shell::Fish));
    }

    #[test]
    fn test_parse_source_line_source_prefix() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("source ~/.zshenv", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_dot_prefix() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line(". ~/.zshenv", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_with_comment() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("source ~/.zshenv # comment", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_with_semicolon() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("source ~/.zshenv; echo done", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_with_and_operator() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line(
            "test -f ~/.zshenv && source ~/.zshenv",
            current,
        );
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_quoted_path() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("source \"~/.zshenv\"", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_single_quoted_path() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("source '~/.zshenv'", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_comment_only() {
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line("# source ~/.zshenv", current);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_source_line_relative_path() {
        let current = Path::new("/home/user/.config/zsh/.zshrc");
        let result = parse_source_line("source aliases.zsh", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_expand_path_tilde_slash() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("~/test"), home.join("test"));
    }

    #[test]
    fn test_expand_path_tilde_only() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("~"), home);
    }

    #[test]
    fn test_expand_path_home_var() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("$HOME/test"), home.join("test"));
    }

    #[test]
    fn test_expand_path_home_braces_var() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("${HOME}/test"), home.join("test"));
    }

    #[test]
    fn test_expand_path_absolute() {
        assert_eq!(expand_path("/etc/profile"), PathBuf::from("/etc/profile"));
    }

    #[test]
    fn test_expand_path_relative() {
        assert_eq!(
            expand_path("relative/path"),
            PathBuf::from("relative/path")
        );
    }

    #[test]
    fn test_expand_path_with_double_quotes() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("\"~/.zshrc\""), home.join(".zshrc"));
    }

    #[test]
    fn test_expand_path_with_single_quotes() {
        let home = dirs::home_dir().unwrap_or_default();
        assert_eq!(expand_path("'~/.zshrc'"), home.join(".zshrc"));
    }

    #[test]
    fn test_is_alias_definition_case_insensitive() {
        assert!(is_alias_definition(
            "ALIAS gs=\"git status\"",
            "gs",
            Shell::Bash
        ));
    }

    #[test]
    fn test_is_alias_definition_with_spaces() {
        assert!(is_alias_definition(
            "  alias gs=\"git status\"",
            "gs",
            Shell::Bash
        ));
    }

    #[test]
    fn test_is_alias_definition_fish_alias_syntax() {
        assert!(is_alias_definition(
            "alias gs=\"git status\"",
            "gs",
            Shell::Fish
        ));
    }

    #[test]
    fn test_is_alias_definition_fish_abbr_with_multiple_args() {
        // The implementation looks for "abbr -a gs " (with space after name)
        // Not "abbr -a -U gs" which has flags between -a and name
        assert!(is_alias_definition(
            "abbr -a gs 'git status'",
            "gs",
            Shell::Fish
        ));
    }

    #[test]
    fn test_alias_location_struct() {
        let loc = AliasLocation {
            file: PathBuf::from("/home/user/.zshrc"),
            line: 10,
        };
        assert_eq!(loc.line, 10);
    }

    #[test]
    fn test_parse_source_line_and_with_dot() {
        let current = Path::new("/home/user/.zshrc");
        let result =
            parse_source_line("test -f ~/.zshenv && . ~/.zshenv", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_source_line_with_parent_dir_path() {
        // The pattern ". .." will match because ". " prefix is found
        // But the filter checks `!s.starts_with('.')` to reject parent dir refs
        let current = Path::new("/home/user/.zshrc");
        let result = parse_source_line(". ./aliases", current);
        assert!(result.is_some());
    }

    #[test]
    fn test_format_location_non_home() {
        let loc =
            AliasLocation { file: PathBuf::from("/etc/profile"), line: 50 };
        assert!(format_location(&loc).contains("/etc/profile"));
    }
}
