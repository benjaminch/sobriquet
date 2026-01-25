//! Audit aliases for security issues and duplicates

use std::collections::HashMap;
use std::io::{self, Write};

use clap::ValueEnum;
use owo_colors::OwoColorize;

use crate::alias::Alias;
use crate::error::Result;
use crate::shell::Shell;
use crate::source::{AliasLocation, format_location, locate_alias};

/// Types of secrets we detect
const SECRET_PATTERNS: &[(&str, &[&str])] = &[
    ("API key", &["API_KEY=", "APIKEY=", "api_key=", "apikey="]),
    (
        "Token",
        &[
            "TOKEN=",
            "ACCESS_TOKEN=",
            "AUTH_TOKEN=",
            "BEARER_TOKEN=",
            "token=",
            "access_token=",
        ],
    ),
    (
        "Secret/Password",
        &["SECRET=", "PASSWORD=", "PASSWD=", "PWD=", "secret=", "password="],
    ),
    (
        "AWS credentials",
        &[
            "AWS_SECRET_ACCESS_KEY=",
            "AWS_ACCESS_KEY_ID=",
            "aws_secret_access_key=",
        ],
    ),
    ("Private key", &["PRIVATE_KEY=", "PRIV_KEY=", "private_key="]),
];

/// Prefixes that indicate well-known API key formats
const SECRET_PREFIXES: &[(&str, &str)] = &[
    ("Anthropic key", "sk-ant-"),
    ("OpenAI key", "sk-proj-"),
    ("GitHub PAT", "ghp_"),
    ("GitHub OAuth", "gho_"),
    ("GitHub App", "ghs_"),
    ("GitHub Refresh", "ghr_"),
    ("Slack bot", "xoxb-"),
    ("Slack user", "xoxp-"),
    ("Slack app", "xoxa-"),
    ("Stripe live", "sk_live_"),
    ("Stripe test", "sk_test_"),
    ("AWS Key ID", "AKIA"),
    ("Sendgrid", "SG."),
    ("Twilio", "SK"),
];

/// A detected secret in an alias
#[derive(Debug, Clone)]
pub struct SecretMatch {
    pub kind: &'static str,
    pub snippet: String,
}

/// A group of aliases with the same command
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub command: String,
    pub aliases: Vec<(String, Option<AliasLocation>)>, // (name, location)
}

/// What to audit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum AuditKind {
    #[default]
    All,
    Secrets,
    Duplicates,
}

/// Check if a command contains any secrets
pub fn detect_secrets(command: &str) -> Vec<SecretMatch> {
    let mut matches = Vec::new();

    // Check for pattern-based secrets (KEY=value)
    for (kind, patterns) in SECRET_PATTERNS {
        for pattern in *patterns {
            if command.contains(pattern) {
                // Extract a snippet around the match
                if let Some(pos) = command.find(pattern) {
                    let end = command[pos..]
                        .find(|c: char| c.is_whitespace())
                        .map_or(command.len(), |i| pos + i);
                    let snippet = &command[pos..end];
                    // Mask the actual value
                    let masked = mask_secret(snippet);
                    matches.push(SecretMatch { kind, snippet: masked });
                }
                break; // One match per pattern type is enough
            }
        }
    }

    // Check for prefix-based secrets (like sk-ant-xxx)
    for (kind, prefix) in SECRET_PREFIXES {
        if command.contains(prefix)
            && let Some(pos) = command.find(prefix)
        {
            let end = command[pos..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .map_or(command.len(), |i| pos + i);
            let snippet = &command[pos..end];
            let masked = mask_secret(snippet);
            matches.push(SecretMatch { kind, snippet: masked });
        }
    }

    matches
}

/// Check if a command contains any secret (simple bool check for preview)
pub fn has_secret(command: &str) -> bool {
    // Quick check for common patterns
    for (_, patterns) in SECRET_PATTERNS {
        for pattern in *patterns {
            if command.contains(pattern) {
                return true;
            }
        }
    }

    for (_, prefix) in SECRET_PREFIXES {
        if command.contains(prefix) {
            return true;
        }
    }

    false
}

/// Get the type of secret found (for preview display)
pub fn get_secret_types(command: &str) -> Vec<&'static str> {
    let matches = detect_secrets(command);
    matches.into_iter().map(|m| m.kind).collect()
}

/// Mask a secret value, showing only first few chars
fn mask_secret(s: &str) -> String {
    if let Some((key, value)) = s.split_once('=') {
        let masked_value = if value.len() > 8 {
            format!("{}...", &value[..8.min(value.len())])
        } else {
            "***".to_owned()
        };
        format!("{key}={masked_value}")
    } else if s.len() > 12 {
        // For prefix-based keys, show prefix + few chars
        format!("{}...", &s[..12.min(s.len())])
    } else {
        s.to_owned()
    }
}

/// Find duplicate aliases (same command, different names)
pub fn find_duplicates(
    aliases: &[Alias],
    shell: Shell,
) -> Vec<DuplicateGroup> {
    let mut command_map: HashMap<
        String,
        Vec<(String, Option<AliasLocation>)>,
    > = HashMap::new();

    for alias in aliases {
        // Normalize command for comparison (trim whitespace)
        let normalized = alias.command.trim().to_owned();
        let location = locate_alias(&alias.name, shell);
        command_map
            .entry(normalized)
            .or_default()
            .push((alias.name.clone(), location));
    }

    command_map
        .into_iter()
        .filter(|(_, names)| names.len() > 1)
        .map(|(command, aliases)| DuplicateGroup { command, aliases })
        .collect()
}

/// Get names of other aliases with the same command (for preview panel)
pub fn get_duplicate_names<'a>(
    command: &str,
    current_name: &str,
    aliases: &'a [Alias],
) -> Vec<&'a str> {
    let normalized = command.trim();
    aliases
        .iter()
        .filter(|a| a.command.trim() == normalized && a.name != current_name)
        .map(|a| a.name.as_str())
        .collect()
}

/// Run the full audit
#[allow(clippy::too_many_lines)]
pub fn run_audit(
    aliases: &[Alias],
    shell: Shell,
    kind: AuditKind,
    use_colors: bool,
) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let total = aliases.len();

    // Collect findings
    let secrets: Vec<_> = if kind == AuditKind::Duplicates {
        Vec::new()
    } else {
        aliases
            .iter()
            .filter_map(|a| {
                let matches = detect_secrets(&a.command);
                if matches.is_empty() {
                    None
                } else {
                    let location = locate_alias(&a.name, shell);
                    Some((a.name.clone(), matches, location))
                }
            })
            .collect()
    };

    let duplicates = if kind == AuditKind::Secrets {
        Vec::new()
    } else {
        find_duplicates(aliases, shell)
    };

    // Header
    if use_colors {
        writeln!(out, "{}", "alx audit".bold())?;
        writeln!(out, "{}", "─────────".dimmed())?;
    } else {
        writeln!(out, "alx audit")?;
        writeln!(out, "---------")?;
    }
    writeln!(out)?;
    writeln!(out, "Scanned {total} aliases")?;
    writeln!(out)?;

    let mut issues_found = false;

    // Report secrets
    if !secrets.is_empty() {
        issues_found = true;
        if use_colors {
            writeln!(
                out,
                "{} {} {}",
                "⚠".yellow(),
                secrets.len().to_string().yellow().bold(),
                "alias(es) contain secrets:".yellow()
            )?;
        } else {
            writeln!(out, "⚠ {} alias(es) contain secrets:", secrets.len())?;
        }

        for (name, matches, location) in &secrets {
            let kinds: Vec<_> = matches.iter().map(|m| m.kind).collect();
            let snippets: Vec<_> =
                matches.iter().map(|m| m.snippet.as_str()).collect();
            let loc_str = location
                .as_ref()
                .map_or_else(|| "unknown".to_owned(), format_location);

            if use_colors {
                writeln!(
                    out,
                    "  {} {:<14} {:<16} {}",
                    "•".dimmed(),
                    name.red(),
                    kinds.join(", ").yellow(),
                    loc_str.cyan()
                )?;
                writeln!(out, "    {}", snippets.join(", ").dimmed())?;
            } else {
                writeln!(
                    out,
                    "  • {:<14} {:<16} {}",
                    name,
                    kinds.join(", "),
                    loc_str
                )?;
                writeln!(out, "    {}", snippets.join(", "))?;
            }
        }
        writeln!(out)?;
    }

    // Report duplicates
    if !duplicates.is_empty() {
        issues_found = true;
        if use_colors {
            writeln!(
                out,
                "{} {} {}",
                "⚠".yellow(),
                duplicates.len().to_string().yellow().bold(),
                "duplicate command(s) found:".yellow()
            )?;
        } else {
            writeln!(
                out,
                "⚠ {} duplicate command(s) found:",
                duplicates.len()
            )?;
        }

        for group in &duplicates {
            let cmd_display = truncate_command(&group.command, 30);

            // Format each alias with its location
            let aliases_with_locs: Vec<String> = group
                .aliases
                .iter()
                .map(|(name, loc)| {
                    if let Some(l) = loc {
                        format!("{} ({})", name, format_location(l))
                    } else {
                        name.clone()
                    }
                })
                .collect();

            if use_colors {
                writeln!(out, "  {} {}", "•".dimmed(), cmd_display.dimmed())?;
                for alias_loc in &aliases_with_locs {
                    writeln!(out, "    → {}", alias_loc.cyan())?;
                }
            } else {
                writeln!(out, "  • {cmd_display}")?;
                for alias_loc in &aliases_with_locs {
                    writeln!(out, "    → {alias_loc}")?;
                }
            }
        }
        writeln!(out)?;
    }

    // Summary
    let secret_count = secrets.len();
    let clean_count = total - secret_count;

    if issues_found {
        if use_colors {
            writeln!(
                out,
                "{} {} aliases passed all checks",
                "✓".green(),
                clean_count.to_string().green()
            )?;
        } else {
            writeln!(out, "✓ {clean_count} aliases passed all checks")?;
        }
    } else if use_colors {
        writeln!(out, "{} {}", "✓".green(), "No issues found".green())?;
    } else {
        writeln!(out, "✓ No issues found")?;
    }
    writeln!(out)?;

    Ok(())
}

/// Truncate a command for display
fn truncate_command(cmd: &str, max_len: usize) -> String {
    let trimmed = cmd.trim();
    if trimmed.len() <= max_len {
        format!("\"{trimmed}\"")
    } else {
        format!("\"{}...\"", &trimmed[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_secrets_api_key() {
        let matches = detect_secrets("API_KEY=sk-12345 curl api.example.com");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].kind, "API key");
    }

    #[test]
    fn test_detect_secrets_anthropic() {
        let matches =
            detect_secrets("ANTHROPIC_API_KEY=sk-ant-api03-abc123 claude");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_detect_secrets_github_pat() {
        let matches = detect_secrets(
            "gh auth login --with-token ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].kind, "GitHub PAT");
    }

    #[test]
    fn test_detect_secrets_none() {
        let matches = detect_secrets("git status");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_has_secret() {
        assert!(has_secret("API_KEY=secret123 curl"));
        assert!(has_secret("some command with sk-ant-xxx"));
        assert!(!has_secret("git push origin main"));
    }

    #[test]
    fn test_mask_secret() {
        assert_eq!(
            mask_secret("API_KEY=verylongsecretkey"),
            "API_KEY=verylong..."
        );
        assert_eq!(
            mask_secret("sk-ant-api03-abcdefghijklmnop"),
            "sk-ant-api03..."
        );
    }

    #[test]
    fn test_get_duplicate_names() {
        let aliases = vec![
            Alias { name: "gs".to_owned(), command: "git status".to_owned() },
            Alias { name: "gst".to_owned(), command: "git status".to_owned() },
            Alias { name: "gss".to_owned(), command: "git status".to_owned() },
        ];

        let dups = get_duplicate_names("git status", "gs", &aliases);
        assert_eq!(dups.len(), 2);
        assert!(dups.contains(&"gst"));
        assert!(dups.contains(&"gss"));
    }

    #[test]
    fn test_get_secret_types() {
        let types = get_secret_types("API_KEY=xxx TOKEN=yyy");
        assert!(types.contains(&"API key"));
        assert!(types.contains(&"Token"));
    }

    #[test]
    fn test_detect_secrets_aws_key() {
        let secrets = detect_secrets("AKIA2EXAMPLEKEY123");
        assert!(!secrets.is_empty());
    }

    #[test]
    fn test_detect_secrets_password() {
        let secrets = detect_secrets("password=supersecret123");
        assert!(!secrets.is_empty());
    }

    #[test]
    fn test_has_secret_returns_true() {
        assert!(has_secret("export GITHUB_TOKEN=ghp_abc123"));
    }

    #[test]
    fn test_has_secret_returns_false() {
        assert!(!has_secret("ls -la /tmp"));
    }

    #[test]
    fn test_find_duplicates_single_command() {
        let aliases = vec![
            Alias { name: "a".to_owned(), command: "git status".to_owned() },
            Alias { name: "b".to_owned(), command: "git status".to_owned() },
            Alias { name: "c".to_owned(), command: "git log".to_owned() },
        ];
        let dups = find_duplicates(&aliases, Shell::Zsh);
        assert_eq!(dups.len(), 1);
    }

    #[test]
    fn test_find_duplicates_multiple_commands() {
        let aliases = vec![
            Alias { name: "a".to_owned(), command: "echo 1".to_owned() },
            Alias { name: "b".to_owned(), command: "echo 1".to_owned() },
            Alias { name: "c".to_owned(), command: "echo 2".to_owned() },
            Alias { name: "d".to_owned(), command: "echo 2".to_owned() },
        ];
        let dups = find_duplicates(&aliases, Shell::Bash);
        assert_eq!(dups.len(), 2);
    }

    #[test]
    fn test_find_duplicates_no_duplicates() {
        let aliases = vec![
            Alias { name: "a".to_owned(), command: "unique1".to_owned() },
            Alias { name: "b".to_owned(), command: "unique2".to_owned() },
        ];
        let dups = find_duplicates(&aliases, Shell::Fish);
        assert!(dups.is_empty());
    }

    #[test]
    fn test_get_duplicate_names_with_multiple() {
        let aliases = vec![
            Alias { name: "g1".to_owned(), command: "git status".to_owned() },
            Alias { name: "g2".to_owned(), command: "git status".to_owned() },
            Alias { name: "g3".to_owned(), command: "git status".to_owned() },
        ];
        let dups = get_duplicate_names("git status", "g1", &aliases);
        assert_eq!(dups.len(), 2);
    }

    #[test]
    fn test_get_secret_types_multiple() {
        let types =
            get_secret_types("GITHUB_TOKEN=abc AWS_KEY=xyz PASSWORD=test");
        assert!(types.len() >= 2);
    }

    #[test]
    fn test_get_secret_types_empty() {
        let types = get_secret_types("no secrets here");
        assert!(types.is_empty());
    }

    #[test]
    fn test_detect_secrets_anthropic_key() {
        let secrets = detect_secrets("sk-ant-v1-example123");
        assert!(!secrets.is_empty());
    }

    #[test]
    fn test_truncate_command_long_string() {
        let long_cmd = "a".repeat(100);
        let truncated = truncate_command(&long_cmd, 20);
        assert!(truncated.len() <= 30);
    }

    #[test]
    fn test_truncate_command_short_string() {
        let short_cmd = "test";
        let truncated = truncate_command(short_cmd, 20);
        assert_eq!(truncated.len(), 6);
    }

    #[test]
    fn test_find_duplicates_empty_list() {
        let aliases: Vec<Alias> = vec![];
        let dups = find_duplicates(&aliases, Shell::Zsh);
        assert!(dups.is_empty());
    }

    #[test]
    fn test_find_duplicates_single_alias() {
        let aliases =
            vec![Alias { name: "a".to_owned(), command: "cmd".to_owned() }];
        let dups = find_duplicates(&aliases, Shell::Zsh);
        assert!(dups.is_empty());
    }
}
