use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{AlxError, Result};
use crate::shell::Shell;

#[derive(Debug, Serialize, Deserialize)]
struct AliasCache {
    aliases: Vec<Alias>,
    timestamp: u64,
    shell: String,
}

impl AliasCache {
    fn cache_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|p| p.join("alx").join("aliases.json"))
    }

    fn load() -> Option<Self> {
        let path = Self::cache_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save(&self) -> Option<()> {
        let path = Self::cache_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok()?;
        }
        let json = serde_json::to_string(self).ok()?;
        fs::write(path, json).ok()
    }

    fn is_valid(&self, ttl: u64, shell: &str) -> bool {
        if self.shell != shell {
            return false;
        }
        let now = Self::now();
        now.saturating_sub(self.timestamp) < ttl
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    pub fn clear() {
        if let Some(path) = Self::cache_path() {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn clear_cache() {
    AliasCache::clear();
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    pub name: String,
    pub command: String,
}

impl Alias {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self { name: name.into(), command: command.into() }
    }

    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let line = line.strip_prefix("alias ").unwrap_or(line);
        let eq_pos = line.find('=')?;
        let name = &line[..eq_pos];
        let command = &line[eq_pos + 1..];

        if !Self::is_valid_name(name) {
            return None;
        }

        let command = Self::strip_quotes(command);
        Some(Self::new(name, command))
    }

    pub fn is_valid_name(name: &str) -> bool {
        name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
    }

    pub fn strip_quotes(s: &str) -> &str {
        let bytes = s.as_bytes();
        if bytes.len() >= 2 {
            let first = bytes[0];
            let last = bytes[bytes.len() - 1];
            if (first == b'\'' && last == b'\'')
                || (first == b'"' && last == b'"')
            {
                return &s[1..s.len() - 1];
            }
        }
        s
    }

    pub fn format_colored(&self, use_colors: bool) -> String {
        if use_colors {
            format!(
                "{} {} {}",
                self.name.green().bold(),
                "->".dimmed(),
                self.command.cyan()
            )
        } else {
            format!("{} -> {}", self.name, self.command)
        }
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.name, self.command)
    }
}

pub fn collect_aliases(
    shell: Option<Shell>,
    config: &Config,
) -> Result<Vec<Alias>> {
    let shell = shell.or_else(|| {
        config.shell.prefer.as_ref().and_then(|s| Shell::parse_shell(s))
    });

    let shells =
        shell.map_or_else(|| Shell::detection_order().to_vec(), |s| vec![s]);

    for shell in shells {
        let shell_name = shell.as_str();

        // Try cache first
        if config.shell.cache_ttl > 0
            && let Some(cache) = AliasCache::load()
            && cache.is_valid(config.shell.cache_ttl, shell_name)
        {
            return Ok(cache.aliases);
        }

        // Fetch from shell
        if let Ok(aliases) = try_collect_from_shell(shell)
            && !aliases.is_empty()
        {
            // Save to cache
            if config.shell.cache_ttl > 0 {
                let cache = AliasCache {
                    aliases: aliases.clone(),
                    timestamp: AliasCache::now(),
                    shell: shell_name.to_owned(),
                };
                cache.save();
            }
            return Ok(aliases);
        }
    }

    Err(AlxError::NoAliasesFound)
}

fn try_collect_from_shell(shell: Shell) -> Result<Vec<Alias>> {
    let output = Command::new(shell.as_str())
        .args(shell.alias_args())
        .output()
        .map_err(|e| AlxError::ShellExecution {
            shell: shell.as_str().to_owned(),
            source: e,
        })?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    Ok(parse_alias_output(&content))
}

pub fn parse_alias_output(content: &str) -> Vec<Alias> {
    content.lines().filter_map(Alias::parse).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_zsh_format() {
        let alias = Alias::parse("ls=eza").unwrap();
        assert_eq!(alias.name, "ls");
        assert_eq!(alias.command, "eza");
    }

    #[test]
    fn parse_single_quoted() {
        let alias = Alias::parse("ll='eza -la'").unwrap();
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.command, "eza -la");
    }

    #[test]
    fn parse_double_quoted() {
        let alias = Alias::parse("ll=\"eza -la\"").unwrap();
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.command, "eza -la");
    }

    #[test]
    fn parse_bash_format() {
        let alias = Alias::parse("alias ls='eza'").unwrap();
        assert_eq!(alias.name, "ls");
        assert_eq!(alias.command, "eza");
    }

    #[test]
    fn parse_command_with_equals() {
        let alias = Alias::parse("myvar='FOO=bar command'").unwrap();
        assert_eq!(alias.name, "myvar");
        assert_eq!(alias.command, "FOO=bar command");
    }

    #[test]
    fn parse_complex_command() {
        let alias = Alias::parse("k8s='kubectl --context=prod'").unwrap();
        assert_eq!(alias.command, "kubectl --context=prod");
    }

    #[test]
    fn parse_empty_returns_none() {
        assert!(Alias::parse("").is_none());
        assert!(Alias::parse("   ").is_none());
    }

    #[test]
    fn parse_rejects_invalid_names() {
        assert!(Alias::parse("1alias=cmd").is_none());
        assert!(Alias::parse("-alias=cmd").is_none());
        assert!(Alias::parse(".alias=cmd").is_none());
    }

    #[test]
    fn parse_accepts_valid_names() {
        assert!(Alias::parse("_valid=cmd").is_some());
        assert!(Alias::parse("my-alias=cmd").is_some());
        assert!(Alias::parse("my_alias=cmd").is_some());
    }

    #[test]
    fn display_format() {
        let alias = Alias::new("ls", "eza");
        assert_eq!(alias.to_string(), "ls -> eza");
        assert_eq!(alias.format_colored(false), "ls -> eza");
    }

    #[test]
    fn alias_equality_and_clone() {
        let a = Alias::new("ls", "eza");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn strip_quotes_works() {
        assert_eq!(Alias::strip_quotes("'hello'"), "hello");
        assert_eq!(Alias::strip_quotes("\"hello\""), "hello");
        assert_eq!(Alias::strip_quotes("hello"), "hello");
        assert_eq!(Alias::strip_quotes("'hello\""), "'hello\"");
        assert_eq!(Alias::strip_quotes(""), "");
    }

    #[test]
    fn is_valid_name_works() {
        assert!(Alias::is_valid_name("abc"));
        assert!(Alias::is_valid_name("_abc"));
        assert!(!Alias::is_valid_name("1abc"));
        assert!(!Alias::is_valid_name("-abc"));
        assert!(!Alias::is_valid_name(""));
    }

    #[test]
    fn parse_output_multiple() {
        let output = "ls=eza\nll='eza -la'\ncat=bat";
        let aliases = parse_alias_output(output);
        assert_eq!(aliases.len(), 3);
    }

    #[test]
    fn parse_output_filters_invalid() {
        let output = "1invalid=cmd\nvalid=cmd\n-bad=cmd";
        let aliases = parse_alias_output(output);
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "valid");
    }

    #[test]
    fn serialization_roundtrip() {
        let alias = Alias::new("ls", "eza");
        let json = serde_json::to_string(&alias).unwrap();
        let parsed: Alias = serde_json::from_str(&json).unwrap();
        assert_eq!(alias, parsed);
    }
}
