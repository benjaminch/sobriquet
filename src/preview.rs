//! Preview panel analysis for alias commands

use std::fmt::Write;

use owo_colors::OwoColorize;

use crate::alias::Alias;
use crate::audit::{get_secret_types, has_secret};
use crate::source::{AliasLocation, format_location};

/// Analyze a command and generate preview content
pub struct CommandAnalysis<'a> {
    command: &'a str,
}

impl<'a> CommandAnalysis<'a> {
    pub fn new(command: &'a str) -> Self {
        Self { command }
    }

    /// Get the base binary/command name
    pub fn binary(&self) -> Option<&str> {
        // Skip env vars at the start (e.g., "KUBECONFIG=... kubectl")
        let cmd = self.skip_env_vars();
        cmd.split_whitespace().next()
    }

    /// Skip leading environment variable assignments
    fn skip_env_vars(&self) -> &str {
        let mut rest = self.command.trim();
        while let Some(first_word) = rest.split_whitespace().next() {
            if first_word.contains('=') && !first_word.starts_with('-') {
                // Skip this env var assignment
                rest = rest[first_word.len()..].trim_start();
            } else {
                break;
            }
        }
        rest
    }

    /// Count pipes in the command
    pub fn pipe_count(&self) -> usize {
        // Simple count, doesn't account for quoted pipes
        self.command.matches(" | ").count()
            + self.command.matches(" |& ").count()
    }

    /// Count redirections
    pub fn redirect_count(&self) -> usize {
        let patterns = [" > ", " >> ", " < ", " 2> ", " 2>> ", " &> ", " >& "];
        patterns.iter().map(|p| self.command.matches(p).count()).sum()
    }

    /// Check for potentially dangerous patterns
    pub fn warnings(&self) -> Vec<&'static str> {
        let mut warnings = Vec::new();
        let cmd_lower = self.command.to_lowercase();

        // Dangerous commands
        if cmd_lower.contains("rm -rf") || cmd_lower.contains("rm -fr") {
            warnings.push("⚠ Recursive force delete");
        }
        if cmd_lower.contains("sudo rm") {
            warnings.push("⚠ Sudo delete");
        }
        if cmd_lower.contains(":(){") || cmd_lower.contains(":(){ ") {
            warnings.push("⚠ Fork bomb pattern");
        }
        if cmd_lower.contains("> /dev/sd") || cmd_lower.contains("dd if=") {
            warnings.push("⚠ Direct disk write");
        }
        if cmd_lower.contains("chmod -r 777")
            || cmd_lower.contains("chmod 777")
        {
            warnings.push("⚠ Overly permissive chmod");
        }
        if cmd_lower.contains("--no-preserve-root") {
            warnings.push("⚠ No preserve root");
        }
        if cmd_lower.contains("mkfs.") {
            warnings.push("⚠ Filesystem format");
        }

        warnings
    }

    /// Detect shell-specific syntax
    pub fn shell_hints(&self) -> Vec<&'static str> {
        let mut hints = Vec::new();

        // Zsh specific
        if self.command.contains("${(") || self.command.contains("${=") {
            hints.push("zsh: parameter expansion");
        }
        if self.command.contains("=()") || self.command.contains("=(") {
            hints.push("zsh: process substitution");
        }
        if self.command.contains("print -P")
            || self.command.contains("print -z")
        {
            hints.push("zsh: print builtin");
        }
        if self.command.contains("zparseopts") {
            hints.push("zsh: zparseopts");
        }

        // Bash specific
        if self.command.contains("declare ") || self.command.contains("local ")
        {
            hints.push("bash/zsh: declare/local");
        }
        if self.command.contains("[[") && self.command.contains("]]") {
            hints.push("bash/zsh: extended test");
        }
        if self.command.contains("shopt ") {
            hints.push("bash: shopt");
        }

        // Fish specific
        if self.command.contains("set -gx") || self.command.contains("set -Ux")
        {
            hints.push("fish: set variable");
        }
        if self.command.contains("string ") {
            hints.push("fish: string builtin");
        }

        // POSIX / universal
        if hints.is_empty()
            && !self.command.contains("${")
            && !self.command.contains("$((")
        {
            hints.push("POSIX compatible");
        }

        hints
    }

    /// Parse command into components
    pub fn breakdown(&self) -> CommandBreakdown<'_> {
        let cmd = self.skip_env_vars();
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        let binary = parts.first().copied();
        let mut flags = Vec::new();
        let mut args = Vec::new();
        let mut subcommand = None;

        for (i, part) in parts.iter().enumerate().skip(1) {
            if part.starts_with('-') {
                flags.push(*part);
            } else if i == 1
                && !part.contains('/')
                && !part.contains('.')
                && parts.len() > 2
            {
                // Likely a subcommand (e.g., "git status", "docker run")
                // Only treat as subcommand if there are more parts after it
                subcommand = Some(*part);
            } else {
                args.push(*part);
            }
        }

        // Extract env vars
        let env_vars: Vec<&str> = self
            .command
            .split_whitespace()
            .take_while(|w| w.contains('=') && !w.starts_with('-'))
            .collect();

        CommandBreakdown { binary, subcommand, flags, args, env_vars }
    }

    /// Generate the full preview text with ANSI colors
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub fn generate_preview(
        &self,
        alias_name: &str,
        usage_count: Option<u64>,
        last_used: Option<&str>,
        similar: &[&str],
        _all_aliases: &[Alias],
        source_location: Option<&AliasLocation>,
        duplicates: &[&str],
    ) -> String {
        let mut preview = String::with_capacity(512);

        // Header
        let _ = writeln!(preview, "{}", alias_name.cyan().bold());
        let _ = writeln!(preview);

        // Command section
        let _ = writeln!(preview, "  {}", "Command".bright_black());
        let _ = writeln!(preview, "  {}", self.command);
        let _ = writeln!(preview);

        // Warnings (if any)
        let warnings = self.warnings();
        let has_secret_warning = has_secret(self.command);

        if !warnings.is_empty() || has_secret_warning || !duplicates.is_empty()
        {
            let _ = writeln!(preview, "  {}", "Warnings".red());
            for warning in &warnings {
                let _ = writeln!(
                    preview,
                    "  {} {}",
                    "•".red(),
                    warning.trim_start_matches("⚠ ").red()
                );
            }
            if has_secret_warning {
                let secret_types = get_secret_types(self.command);
                let types_str = if secret_types.is_empty() {
                    "secret/token".to_owned()
                } else {
                    secret_types.join(", ")
                };
                let _ = writeln!(
                    preview,
                    "  {} {}",
                    "•".red(),
                    format!("Contains {types_str}").red()
                );
            }
            if !duplicates.is_empty() {
                let _ = writeln!(
                    preview,
                    "  {} Duplicates: {}",
                    "•".yellow(),
                    duplicates.join(", ").yellow()
                );
            }
            let _ = writeln!(preview);
        }

        // Details section
        let breakdown = self.breakdown();
        let _ = writeln!(preview, "  {}", "Details".bright_black());

        // Binary + subcommand
        if let Some(bin) = breakdown.binary {
            let _ = write!(
                preview,
                "  {} {}",
                "Binary:".bright_black(),
                bin.green()
            );
            if let Some(sub) = breakdown.subcommand {
                let _ = write!(preview, " {}", sub.green());
            }
            let _ = writeln!(preview);
        }

        // Flags
        if !breakdown.flags.is_empty() {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Flags: ".bright_black(),
                breakdown.flags.join(" ").yellow()
            );
        }

        // Args
        if !breakdown.args.is_empty() {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Args:  ".bright_black(),
                breakdown.args.join(" ")
            );
        }

        // Env vars
        if !breakdown.env_vars.is_empty() {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Env:   ".bright_black(),
                breakdown.env_vars.join(" ").blue()
            );
        }

        // Pipes/redirects
        let pipes = self.pipe_count();
        let redirects = self.redirect_count();
        if pipes > 0 || redirects > 0 {
            let mut flow_parts = Vec::new();
            if pipes > 0 {
                flow_parts.push(format!(
                    "{} pipe{}",
                    pipes,
                    if pipes == 1 { "" } else { "s" }
                ));
            }
            if redirects > 0 {
                flow_parts.push(format!(
                    "{} redirect{}",
                    redirects,
                    if redirects == 1 { "" } else { "s" }
                ));
            }
            let _ = writeln!(
                preview,
                "  {} {}",
                "Flow:  ".bright_black(),
                flow_parts.join(", ")
            );
        }

        // Shell compatibility
        let hints = self.shell_hints();
        if !hints.is_empty() {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Shell: ".bright_black(),
                hints.join(", ").bright_black()
            );
        }

        let _ = writeln!(preview);

        // Info section
        let _ = writeln!(preview, "  {}", "Info".bright_black());

        // Source location
        if let Some(loc) = source_location {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Source:".bright_black(),
                format_location(loc).cyan()
            );
        } else {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Source:".bright_black(),
                "dynamic".bright_black()
            );
        }

        // Usage stats
        if let Some(count) = usage_count {
            let _ = writeln!(
                preview,
                "  {} {} time{}",
                "Used:  ".bright_black(),
                count,
                if count == 1 { "" } else { "s" }
            );
        }
        if let Some(last) = last_used {
            let _ =
                writeln!(preview, "  {} {}", "Last:  ".bright_black(), last);
        }

        // Similar aliases
        if !similar.is_empty() {
            let _ = writeln!(
                preview,
                "  {} {}",
                "Similar:".bright_black(),
                similar.join(", ").cyan()
            );
        }

        preview
    }
}

pub struct CommandBreakdown<'a> {
    pub binary: Option<&'a str>,
    pub subcommand: Option<&'a str>,
    pub flags: Vec<&'a str>,
    pub args: Vec<&'a str>,
    pub env_vars: Vec<&'a str>,
}

/// Find similar alias names
pub fn find_similar<'a>(
    name: &str,
    all_names: &'a [&'a str],
    max: usize,
) -> Vec<&'a str> {
    let mut similar: Vec<_> = all_names
        .iter()
        .filter(|&&n| n != name)
        .filter(|&&n| {
            // Same prefix (at least 2 chars)
            let prefix_match = name.len() >= 2
                && n.len() >= 2
                && name[..2.min(name.len())] == n[..2.min(n.len())];

            // Contains as substring
            let contains = n.contains(name) || name.contains(n);

            // Edit distance would be nice but keeping it simple
            prefix_match || contains
        })
        .take(max)
        .copied()
        .collect();

    similar.truncate(max);
    similar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_extraction() {
        assert_eq!(CommandAnalysis::new("git status").binary(), Some("git"));
        assert_eq!(
            CommandAnalysis::new("KUBECONFIG=~/.kube/config kubectl get pods")
                .binary(),
            Some("kubectl")
        );
        assert_eq!(CommandAnalysis::new("ls -la").binary(), Some("ls"));
    }

    #[test]
    fn test_pipe_count() {
        assert_eq!(CommandAnalysis::new("ls | grep foo").pipe_count(), 1);
        assert_eq!(
            CommandAnalysis::new("cat file | grep x | sort | uniq")
                .pipe_count(),
            3
        );
        assert_eq!(CommandAnalysis::new("echo hello").pipe_count(), 0);
    }

    #[test]
    fn test_warnings() {
        assert!(
            !CommandAnalysis::new("rm -rf /tmp/test").warnings().is_empty()
        );
        assert!(CommandAnalysis::new("ls -la").warnings().is_empty());
    }

    #[test]
    fn test_breakdown() {
        let analysis = CommandAnalysis::new("git commit -m 'test' --amend");
        let breakdown = analysis.breakdown();
        assert_eq!(breakdown.binary, Some("git"));
        assert_eq!(breakdown.subcommand, Some("commit"));
        assert!(breakdown.flags.contains(&"-m"));
        assert!(breakdown.flags.contains(&"--amend"));
    }

    #[test]
    fn test_similar() {
        let names = vec!["gs", "gst", "gss", "ga", "gc", "gco"];
        let similar = find_similar("gs", &names, 3);
        assert!(similar.contains(&"gst"));
        assert!(similar.contains(&"gss"));
    }

    #[test]
    fn test_redirect_count() {
        assert_eq!(
            CommandAnalysis::new("echo hello > file.txt").redirect_count(),
            1
        );
        assert_eq!(
            CommandAnalysis::new("cmd 2> error.log > output.txt")
                .redirect_count(),
            2
        );
        assert_eq!(
            CommandAnalysis::new("cat < input.txt >> output.txt")
                .redirect_count(),
            2
        );
        assert_eq!(CommandAnalysis::new("ls").redirect_count(), 0);
    }

    #[test]
    fn test_shell_hints_zsh() {
        assert!(
            CommandAnalysis::new("echo ${(L)var}")
                .shell_hints()
                .iter()
                .any(|h| h.contains("zsh"))
        );
        assert!(
            CommandAnalysis::new("cat =(echo test)")
                .shell_hints()
                .iter()
                .any(|h| h.contains("zsh"))
        );
        assert!(
            CommandAnalysis::new("print -P '%1~'")
                .shell_hints()
                .iter()
                .any(|h| h.contains("zsh"))
        );
        assert!(
            CommandAnalysis::new("zparseopts -D a=opt")
                .shell_hints()
                .iter()
                .any(|h| h.contains("zsh"))
        );
    }

    #[test]
    fn test_shell_hints_bash() {
        assert!(
            CommandAnalysis::new("declare -r var=1")
                .shell_hints()
                .iter()
                .any(|h| h.contains("bash/zsh"))
        );
        assert!(
            CommandAnalysis::new("if [[ $x ]]; then echo test; fi")
                .shell_hints()
                .iter()
                .any(|h| h.contains("extended test"))
        );
        assert!(
            CommandAnalysis::new("shopt -s globstar")
                .shell_hints()
                .iter()
                .any(|h| h.contains("bash: shopt"))
        );
        assert!(
            CommandAnalysis::new("local var=test")
                .shell_hints()
                .iter()
                .any(|h| h.contains("bash/zsh"))
        );
    }

    #[test]
    fn test_shell_hints_fish() {
        assert!(
            CommandAnalysis::new("set -gx VAR val")
                .shell_hints()
                .iter()
                .any(|h| h.contains("fish"))
        );
        assert!(
            CommandAnalysis::new("set -Ux VAR val")
                .shell_hints()
                .iter()
                .any(|h| h.contains("fish"))
        );
        assert!(
            CommandAnalysis::new("string match pattern $var")
                .shell_hints()
                .iter()
                .any(|h| h.contains("fish"))
        );
    }

    #[test]
    fn test_shell_hints_posix() {
        assert!(
            CommandAnalysis::new("ls -la")
                .shell_hints()
                .iter()
                .any(|h| h.contains("POSIX"))
        );
        assert!(
            CommandAnalysis::new("echo hello")
                .shell_hints()
                .iter()
                .any(|h| h.contains("POSIX"))
        );
    }

    #[test]
    fn test_warnings_dangerous_patterns() {
        // rm -rf
        assert!(
            CommandAnalysis::new("rm -rf /")
                .warnings()
                .iter()
                .any(|w| w.contains("Recursive"))
        );
        assert!(
            CommandAnalysis::new("rm -fr /")
                .warnings()
                .iter()
                .any(|w| w.contains("Recursive"))
        );

        // sudo rm
        assert!(
            CommandAnalysis::new("sudo rm file")
                .warnings()
                .iter()
                .any(|w| w.contains("Sudo delete"))
        );

        // fork bomb
        assert!(
            CommandAnalysis::new(":(){:|:&};:")
                .warnings()
                .iter()
                .any(|w| w.contains("Fork bomb"))
        );

        // disk write
        assert!(
            CommandAnalysis::new("dd if=/dev/zero of=/dev/sda")
                .warnings()
                .iter()
                .any(|w| w.contains("disk"))
        );

        // chmod
        assert!(
            CommandAnalysis::new("chmod 777 /")
                .warnings()
                .iter()
                .any(|w| w.contains("permissive"))
        );
        assert!(
            CommandAnalysis::new("chmod -r 777 /")
                .warnings()
                .iter()
                .any(|w| w.contains("permissive"))
        );

        // no preserve root
        assert!(
            CommandAnalysis::new("rm --no-preserve-root /")
                .warnings()
                .iter()
                .any(|w| w.contains("preserve"))
        );

        // mkfs
        assert!(
            CommandAnalysis::new("mkfs.ext4 /dev/sda1")
                .warnings()
                .iter()
                .any(|w| w.contains("Filesystem"))
        );
    }

    #[test]
    fn test_warnings_case_insensitive() {
        assert!(!CommandAnalysis::new("RM -RF /").warnings().is_empty());
        assert!(!CommandAnalysis::new("CHMOD 777 /").warnings().is_empty());
    }

    #[test]
    fn test_breakdown_with_env_vars() {
        let analysis = CommandAnalysis::new("VAR=value CMD arg");
        let breakdown = analysis.breakdown();
        assert!(breakdown.env_vars.contains(&"VAR=value"));
        assert_eq!(breakdown.binary, Some("CMD"));
        assert!(breakdown.args.contains(&"arg"));
    }

    #[test]
    fn test_breakdown_multiple_env_vars() {
        let analysis = CommandAnalysis::new("VAR1=v1 VAR2=v2 cmd arg");
        let breakdown = analysis.breakdown();
        assert_eq!(breakdown.env_vars.len(), 2);
    }

    #[test]
    fn test_breakdown_no_subcommand_with_path() {
        let analysis = CommandAnalysis::new("cmd /path/to/file");
        let breakdown = analysis.breakdown();
        assert_eq!(breakdown.subcommand, None);
        assert!(breakdown.args.contains(&"/path/to/file"));
    }

    #[test]
    fn test_breakdown_no_subcommand_with_dot_path() {
        let analysis = CommandAnalysis::new("cmd ./script.sh");
        let breakdown = analysis.breakdown();
        assert_eq!(breakdown.subcommand, None);
        assert!(breakdown.args.contains(&"./script.sh"));
    }

    #[test]
    fn test_pipe_count_ampersand_redirect() {
        assert_eq!(CommandAnalysis::new("cmd1 |& cmd2").pipe_count(), 1);
    }

    #[test]
    fn test_binary_empty_command() {
        assert_eq!(CommandAnalysis::new("").binary(), None);
        assert_eq!(CommandAnalysis::new("   ").binary(), None);
    }

    #[test]
    fn test_redirect_all_types() {
        assert_eq!(CommandAnalysis::new("cmd > out").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd >> out").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd < in").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd 2> err").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd 2>> err").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd &> out").redirect_count(), 1);
        assert_eq!(CommandAnalysis::new("cmd >& out").redirect_count(), 1);
    }
}
