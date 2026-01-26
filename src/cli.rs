//! Command-line interface and interactive alias selection
//!
//! This module provides the main CLI functionality including:
//! - Command-line argument parsing with clap
//! - Interactive fuzzy finder interface (skim)
//! - Alias listing and formatting (plain, JSON)
//! - Shell completion generation
//! - Usage statistics tracking
//! - Security auditing integration

#[cfg(feature = "interactive")]
use std::borrow::Cow;
use std::io::{self, Write};
use std::process::ExitCode;

#[cfg(feature = "interactive")]
use std::sync::Arc;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell as CompletionShell};
use serde_json;

#[cfg(feature = "interactive")]
use skim::prelude::*;

use crate::alias::{Alias, collect_aliases};
use crate::audit::{self, AuditKind};
use crate::config::{ColorChoice, Config};
use crate::error::{AlxError, Result};
#[cfg(feature = "interactive")]
use crate::preview::{CommandAnalysis, find_similar};
use crate::shell::{InitShell, Shell, generate_init_script};
#[cfg(feature = "interactive")]
use crate::source::AliasLocation;
#[cfg(feature = "interactive")]
use crate::stats::{UsageRecord, UsageStats, display_stats};
#[cfg(not(feature = "interactive"))]
use crate::stats::{UsageStats, display_stats};

const APP_NAME: &str = "sobriquet";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = APP_NAME, version, about, long_about = None)]
#[command(
    after_help = "For more information, visit: https://github.com/benjaminch/alx"
)]
pub struct Args {
    #[arg(short, long, help = "List all aliases")]
    list: bool,

    #[arg(short, long, value_enum, help = "Shell to use")]
    shell: Option<Shell>,

    #[arg(short, long, help = "Start with query")]
    query: Option<String>,

    #[arg(
        short,
        long,
        value_enum,
        default_value = "plain",
        help = "Output format"
    )]
    format: OutputFormat,

    #[arg(long, value_enum, help = "Color mode")]
    color: Option<ColorChoice>,

    #[arg(long, help = "Print query if no match")]
    print_query: bool,

    #[arg(short = 'r', long, help = "Refresh alias cache")]
    refresh: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
    JsonPretty,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Generate shell integration script
    Init {
        #[arg(value_enum)]
        shell: InitShell,
    },
    /// Generate completions or man page
    Generate {
        #[arg(value_enum)]
        kind: GenerateKind,
    },
    /// Show config file path
    Config,
    /// Show usage statistics
    Stats {
        #[command(subcommand)]
        action: Option<StatsAction>,
    },
    /// Audit aliases for security issues and duplicates
    Audit {
        /// What to check: all (default), secrets, duplicates
        #[arg(value_enum)]
        kind: Option<AuditKind>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum StatsAction {
    /// Clear all statistics
    Clear,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GenerateKind {
    CompleteZsh,
    CompleteBash,
    CompleteFish,
    Man,
}

#[cfg(feature = "interactive")]
struct AliasItem {
    alias: Alias,
    display_raw: String,
    stats: Option<UsageRecord>,
    all_names: Vec<String>,
    all_aliases: Vec<Alias>,
    source_location: Option<AliasLocation>,
}

#[cfg(feature = "interactive")]
impl AliasItem {
    fn new(
        alias: Alias,
        stats: Option<UsageRecord>,
        all_names: Vec<String>,
        all_aliases: Vec<Alias>,
        source_location: Option<AliasLocation>,
    ) -> Self {
        let display_raw = alias.to_string();

        Self {
            alias,
            display_raw,
            stats,
            all_names,
            all_aliases,
            source_location,
        }
    }
}

#[cfg(feature = "interactive")]
impl SkimItem for AliasItem {
    fn text(&self) -> Cow<'_, str> {
        // Check toggle file to determine if we should show raw (unmasked) version
        let toggle_file =
            std::env::temp_dir().join("sobriquet_preview_expand");
        let show_raw = toggle_file.exists();

        if show_raw {
            // Show raw command with secrets visible
            Cow::Borrowed(&self.display_raw)
        } else {
            // Mask secrets by default for security
            let command_masked =
                audit::mask_secrets_in_command(&self.alias.command);
            let display_masked =
                format!("{} -> {}", self.alias.name, command_masked);
            Cow::Owned(display_masked)
        }
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        // Check toggle file to determine if we should show raw (unmasked) version
        let toggle_file =
            std::env::temp_dir().join("sobriquet_preview_expand");
        let show_raw = toggle_file.exists();

        // Use raw command if toggle is on, otherwise masked command
        let command_to_analyze = if show_raw {
            Cow::Borrowed(&self.alias.command)
        } else {
            // Mask secrets by default for security
            Cow::Owned(audit::mask_secrets_in_command(&self.alias.command))
        };

        let analysis = CommandAnalysis::new(&command_to_analyze);

        let usage_count = self.stats.as_ref().map(|r| r.count);
        let last_used = self
            .stats
            .as_ref()
            .map(|r| UsageStats::format_relative_time(r.last_used));

        let name_refs: Vec<&str> =
            self.all_names.iter().map(String::as_str).collect();
        let similar = find_similar(&self.alias.name, &name_refs, 3);

        // Get duplicate aliases
        let duplicates = audit::get_duplicate_names(
            &self.alias.command,
            &self.alias.name,
            &self.all_aliases,
        );

        let preview = analysis.generate_preview(
            &self.alias.name,
            usage_count,
            last_used.as_deref(),
            &similar,
            &self.all_aliases,
            self.source_location.as_ref(),
            &duplicates,
            show_raw,
        );

        ItemPreview::AnsiText(preview)
    }
}

/// Sort aliases by frecency score (most used/recent first)
fn sort_by_frecency(aliases: &[Alias], stats: &UsageStats) -> Vec<Alias> {
    let mut sorted: Vec<_> = aliases.to_vec();
    sorted.sort_by(|a, b| {
        let score_a = stats.frecency_score(&a.name);
        let score_b = stats.frecency_score(&b.name);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted
}

#[cfg(feature = "interactive")]
fn run_fuzzy_finder(
    aliases: &[Alias],
    config: &Config,
    query: Option<&str>,
    print_query: bool,
    stats: &UsageStats,
    shell: Shell,
) -> Result<(String, String)> {
    if aliases.is_empty() {
        return Err(AlxError::NoAliasesFound);
    }

    let preview_opt = config.ui.preview.then_some("");

    // Create a temporary state file for toggling preview details
    let toggle_file = std::env::temp_dir().join("sobriquet_preview_expand");

    // Initialize with config value
    if config.ui.preview_show_secrets {
        let _ = std::fs::write(&toggle_file, "1");
    } else {
        let _ = std::fs::remove_file(&toggle_file);
    }

    // Create toggle command script for the keybinding
    let toggle_script = format!(
        "test -f {} && rm {} || echo 1 > {}",
        toggle_file.display(),
        toggle_file.display(),
        toggle_file.display()
    );

    let bind_option =
        format!("ctrl-x:execute-silent({toggle_script})+refresh-preview");

    let mut builder = SkimOptionsBuilder::default();
    builder
        .height(Some(&config.ui.height))
        .multi(false)
        .prompt(Some(&config.ui.prompt))
        .preview(preview_opt)
        .query(query)
        .nosort(true) // Preserve our frecency sort order
        .bind(vec![bind_option.as_str()]);

    let preview_window = format!("{}:wrap", config.ui.preview_position);
    if config.ui.preview {
        builder.preview_window(Some(&preview_window));
    }

    let options = builder.build().expect("Failed to build skim options");

    // Collect all alias names for "similar aliases" feature
    let all_names: Vec<String> =
        aliases.iter().map(|a| a.name.clone()).collect();
    let all_aliases: Vec<Alias> = aliases.to_vec();

    let items: Vec<Arc<dyn SkimItem>> = aliases
        .iter()
        .map(|a| {
            let record = stats.get(&a.name).cloned();
            let source_loc = crate::source::locate_alias(&a.name, shell);
            Arc::new(AliasItem::new(
                a.clone(),
                record,
                all_names.clone(),
                all_aliases.clone(),
                source_loc,
            )) as Arc<dyn SkimItem>
        })
        .collect();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in items {
        let _ = tx.send(item);
    }
    drop(tx);

    let Some(output) = Skim::run_with(&options, Some(rx)) else {
        return Err(AlxError::UserAborted);
    };

    if output.is_abort {
        // Clean up toggle file
        let _ = std::fs::remove_file(&toggle_file);
        if print_query && !output.query.is_empty() {
            return Ok((output.query.clone(), output.query));
        }
        return Err(AlxError::UserAborted);
    }

    // Clean up toggle file on success
    let _ = std::fs::remove_file(&toggle_file);

    output
        .selected_items
        .first()
        .map(|item| {
            let text = item.output();
            match text.split_once(" -> ") {
                Some((name, cmd)) => (cmd.to_owned(), name.to_owned()),
                None => (text.to_string(), text.into_owned()),
            }
        })
        .ok_or(AlxError::UserAborted)
}

#[cfg(not(feature = "interactive"))]
fn run_fuzzy_finder(
    aliases: &[Alias],
    _config: &Config,
    _query: Option<&str>,
    _print_query: bool,
    _stats: &UsageStats,
    _shell: Shell,
) -> Result<(String, String)> {
    // Interactive mode is not supported on non-Unix platforms
    // Fallback to listing mode
    if aliases.is_empty() {
        return Err(AlxError::NoAliasesFound);
    }

    // Print available aliases and ask for a selection
    writeln!(
        io::stderr(),
        "Interactive mode not supported on this platform."
    )?;
    writeln!(io::stderr(), "Available aliases:")?;
    for alias in aliases {
        writeln!(io::stderr(), "  {}", alias.name)?;
    }
    writeln!(io::stderr())?;
    writeln!(
        io::stderr(),
        "Please use 'sobriquet --list' to see all aliases"
    )?;

    Err(AlxError::NoAliasesFound)
}

fn output_aliases(
    aliases: &[Alias],
    format: OutputFormat,
    use_colors: bool,
) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match format {
        OutputFormat::Plain => {
            for alias in aliases {
                writeln!(handle, "{}", alias.format_colored(use_colors))?;
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(aliases)
                .map_err(|e| AlxError::Serialization(e.to_string()))?;
            writeln!(handle, "{json}")?;
        }
        OutputFormat::JsonPretty => {
            let json = serde_json::to_string_pretty(aliases)
                .map_err(|e| AlxError::Serialization(e.to_string()))?;
            writeln!(handle, "{json}")?;
        }
    }

    Ok(())
}

fn generate_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    clap_complete::generate(generator, cmd, APP_NAME, &mut io::stdout());
}

#[allow(clippy::too_many_lines)]
pub fn generate_man_page() -> String {
    let date = "2025-01-24";

    format!(
        r#".TH SOBRIQUET 1 "{date}" "{APP_NAME} {APP_VERSION}" "User Commands"
.SH NAME
sobriquet - fuzzy finder for shell aliases
.SH SYNOPSIS
.B sobriquet
[\fIOPTIONS\fR]
.br
.B sobriquet
\fBinit\fR \fISHELL\fR
.br
.B sobriquet
\fBgenerate\fR \fIKIND\fR
.br
.B sobriquet
\fBconfig\fR
.br
.B sobriquet
\fBstats\fR [\fBclear\fR]
.br
.B sobriquet
\fBaudit\fR [\fIKIND\fR]
.SH DESCRIPTION
.B sobriquet
reads your shell aliases and presents them in an interactive fuzzy finder.
Select an alias and its expanded command will be output to stdout.
.PP
Usage is tracked automatically. Use \fBsobriquet stats\fR to view statistics.
.SH OPTIONS
.TP
.BR \-l ", " \-\-list
List all aliases without interactive selection.
.TP
.BR \-s ", " \-\-shell " " \fISHELL\fR
Shell to use for reading aliases: \fBzsh\fR, \fBbash\fR, \fBfish\fR.
.TP
.BR \-q ", " \-\-query " " \fISTRING\fR
Start with the given query pre-filled.
.TP
.BR \-f ", " \-\-format " " \fIFORMAT\fR
Output format for \-\-list: \fBplain\fR (default), \fBjson\fR, \fBjson-pretty\fR.
.TP
.BR \-\-color " " \fIWHEN\fR
Color mode: \fBauto\fR (default), \fBalways\fR, \fBnever\fR.
.TP
.BR \-\-print\-query
Print the query if no match is selected.
.TP
.BR \-r ", " \-\-refresh
Force refresh of the alias cache.
.TP
.BR \-h ", " \-\-help
Print help.
.TP
.BR \-V ", " \-\-version
Print version.
.SH SUBCOMMANDS
.TP
.B init \fISHELL\fR
Generate shell integration script for \fBzsh\fR, \fBbash\fR, or \fBfish\fR.
.TP
.B generate \fIKIND\fR
Generate completions or man page: \fBcomplete-zsh\fR, \fBcomplete-bash\fR, \fBcomplete-fish\fR, \fBman\fR.
.TP
.B config
Show the configuration file path.
.TP
.B stats
Show usage statistics.
.TP
.B stats clear
Clear all usage statistics.
.TP
.B audit [\fIKIND\fR]
Audit aliases for security issues and duplicates.
\fIKIND\fR can be: \fBall\fR (default), \fBsecrets\fR, \fBduplicates\fR.
Detects embedded API keys, tokens, passwords, and finds aliases with identical commands.
Shows file location where each alias is defined.
.SH CONFIGURATION
Configuration file: \fI~/.config/sobriquet/config.toml\fR
.PP
.RS
.nf
[ui]
height = "50%"
prompt = "> "
preview = true

[shell]
prefer = "zsh"
cache_ttl = 300  # seconds, 0 to disable

[output]
color = "auto"
.fi
.RE
.SH CACHING
Aliases are cached to \fI~/.cache/sobriquet/aliases.json\fR for fast startup.
The default cache TTL is 300 seconds (5 minutes).
Use \fB\-\-refresh\fR to force a cache update.
Set \fBcache_ttl = 0\fR in the config to disable caching.
.SH FILES
.TP
.I ~/.config/sobriquet/config.toml
Configuration file
.TP
.I ~/.cache/sobriquet/aliases.json
Cached aliases
.TP
.I ~/.local/share/sobriquet/stats.json
Usage statistics
.SH EXIT STATUS
.TP
.B 0
Success
.TP
.B 1
Cancelled or no aliases
.TP
.B 2
Error
.SH EXAMPLES
.TP
.B sobriquet
Open interactive fuzzy finder
.TP
.B sobriquet \-\-query git
Open with "git" pre-filled
.TP
.B sobriquet \-\-list
List all aliases
.TP
.B sobriquet stats
Show usage statistics
.TP
.B sobriquet init zsh >> ~/.zshrc
Add shell integration
.TP
.B sobriquet audit
Check for secrets and duplicates
.TP
.B sobriquet audit secrets
Check for embedded secrets only
.SH SEE ALSO
.BR alias (1),
.BR fzf (1)
.SH BUGS
https://github.com/benjaminch/sobriquet/issues
.SH AUTHOR
Benjamin Chausse
"#
    )
}

#[allow(clippy::too_many_lines)]
pub fn run() -> Result<ExitCode> {
    let args = Args::parse();
    let mut config = Config::load();

    if let Some(color) = args.color {
        config.output.color = color;
    }

    if let Some(command) = args.command {
        match command {
            Commands::Init { shell } => {
                print!("{}", generate_init_script(shell));
                return Ok(ExitCode::SUCCESS);
            }
            Commands::Generate { kind } => {
                let mut cmd = Args::command();
                match kind {
                    GenerateKind::CompleteZsh => {
                        generate_completions(CompletionShell::Zsh, &mut cmd);
                    }
                    GenerateKind::CompleteBash => {
                        generate_completions(CompletionShell::Bash, &mut cmd);
                    }
                    GenerateKind::CompleteFish => {
                        generate_completions(CompletionShell::Fish, &mut cmd);
                    }
                    GenerateKind::Man => {
                        print!("{}", generate_man_page());
                    }
                }
                return Ok(ExitCode::SUCCESS);
            }
            Commands::Config => {
                match Config::config_path() {
                    Some(path) => println!("{}", path.display()),
                    None => eprintln!("Could not determine config directory"),
                }
                return Ok(ExitCode::SUCCESS);
            }
            Commands::Stats { action } => {
                let mut stats = UsageStats::load();
                match action {
                    Some(StatsAction::Clear) => {
                        stats.clear();
                        stats.save()?;
                        println!("Statistics cleared.");
                    }
                    None => {
                        display_stats(&stats, config.use_colors())?;
                    }
                }
                return Ok(ExitCode::SUCCESS);
            }
            Commands::Audit { kind } => {
                let shell = args.shell.unwrap_or_else(|| {
                    config
                        .shell
                        .prefer
                        .as_ref()
                        .and_then(|s| Shell::parse_shell(s))
                        .unwrap_or(Shell::Zsh)
                });
                // For audit, we allow empty aliases (graceful degradation)
                let aliases =
                    collect_aliases(args.shell, &config).unwrap_or_default();
                audit::run_audit(
                    &aliases,
                    shell,
                    kind.unwrap_or_default(),
                    config.use_colors(),
                )?;
                return Ok(ExitCode::SUCCESS);
            }
        }
    }

    if args.refresh {
        crate::alias::clear_cache();
    }

    let detected_shell = args.shell.unwrap_or_else(|| {
        config
            .shell
            .prefer
            .as_ref()
            .and_then(|s| Shell::parse_shell(s))
            .unwrap_or(Shell::Zsh)
    });
    let aliases = collect_aliases(args.shell, &config)?;
    let mut stats = UsageStats::load();

    // Sort aliases by frecency (most used/recent first)
    let sorted_aliases = sort_by_frecency(&aliases, &stats);

    if args.list {
        output_aliases(&sorted_aliases, args.format, config.use_colors())?;
        Ok(ExitCode::SUCCESS)
    } else {
        let result = run_fuzzy_finder(
            &sorted_aliases,
            &config,
            args.query.as_deref(),
            args.print_query,
            &stats,
            detected_shell,
        );

        match result {
            Ok((command, alias_name)) => {
                stats.record_usage(&alias_name);
                let _ = stats.save();
                print!("{command}");
                Ok(ExitCode::SUCCESS)
            }
            Err(AlxError::UserAborted) => Ok(ExitCode::from(1)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_option() {
        let args =
            Args::try_parse_from(["sobriquet", "--color", "always"]).unwrap();
        assert_eq!(args.color, Some(ColorChoice::Always));
    }

    #[test]
    fn parse_list_flag() {
        assert!(Args::try_parse_from(["sobriquet", "--list"]).unwrap().list);
        assert!(Args::try_parse_from(["sobriquet", "-l"]).unwrap().list);
    }

    #[test]
    fn parse_shell_option() {
        let args =
            Args::try_parse_from(["sobriquet", "--shell", "bash"]).unwrap();
        assert_eq!(args.shell, Some(Shell::Bash));
    }

    #[test]
    fn parse_format_option() {
        let args =
            Args::try_parse_from(["sobriquet", "--format", "json"]).unwrap();
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn man_page_has_required_sections() {
        let man = generate_man_page();
        assert!(man.contains(".SH NAME"));
        assert!(man.contains(".SH SYNOPSIS"));
        assert!(man.contains(".SH OPTIONS"));
        assert!(man.contains("stats"));
    }

    #[test]
    fn parse_query_option() {
        let args =
            Args::try_parse_from(["sobriquet", "--query", "test"]).unwrap();
        assert_eq!(args.query, Some("test".to_owned()));
    }

    #[test]
    fn parse_print_query_flag() {
        let args =
            Args::try_parse_from(["sobriquet", "--print-query"]).unwrap();
        assert!(args.print_query);
    }

    #[test]
    fn parse_color_never() {
        let args =
            Args::try_parse_from(["sobriquet", "--color", "never"]).unwrap();
        assert_eq!(args.color, Some(ColorChoice::Never));
    }

    #[test]
    fn parse_color_auto() {
        let args =
            Args::try_parse_from(["sobriquet", "--color", "auto"]).unwrap();
        assert_eq!(args.color, Some(ColorChoice::Auto));
    }

    #[test]
    fn output_format_json() {
        let args =
            Args::try_parse_from(["sobriquet", "--format", "json"]).unwrap();
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn test_output_aliases_plain() {
        let aliases = [
            Alias { name: "gs".to_owned(), command: "git status".to_owned() },
            Alias { name: "gc".to_owned(), command: "git commit".to_owned() },
        ];
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name, "gs");
    }

    #[test]
    fn test_output_format_variants() {
        assert_eq!(OutputFormat::Plain, OutputFormat::Plain);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_eq!(OutputFormat::JsonPretty, OutputFormat::JsonPretty);
    }

    #[test]
    fn test_sort_by_frecency_respects_order() {
        let alias1 =
            Alias { name: "cmd1".to_owned(), command: "echo 1".to_owned() };
        let alias2 =
            Alias { name: "cmd2".to_owned(), command: "echo 2".to_owned() };
        let alias3 =
            Alias { name: "cmd3".to_owned(), command: "echo 3".to_owned() };
        let aliases = vec![alias1, alias2, alias3];
        let stats = UsageStats::default();
        let sorted = sort_by_frecency(&aliases, &stats);
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    #[cfg(feature = "interactive")]
    fn test_command_analysis_with_complex_command() {
        let cmd =
            "KUBECONFIG=~/.kube/config kubectl get pods -o json | jq '.items'";
        let analysis = CommandAnalysis::new(cmd);
        assert_eq!(analysis.binary(), Some("kubectl"));
        assert!(analysis.pipe_count() > 0);
    }

    #[test]
    fn test_man_page_contains_all_sections() {
        let man = generate_man_page();
        assert!(man.contains(".SH DESCRIPTION"));
        assert!(man.contains(".SH OPTIONS"));
        assert!(man.contains(".SH SUBCOMMANDS"));
        assert!(man.contains(".SH CONFIGURATION"));
        assert!(man.contains(".SH FILES"));
        assert!(man.contains(".SH EXIT STATUS"));
        assert!(man.contains(".SH EXAMPLES"));
    }

    #[test]
    fn test_man_page_examples_section() {
        let man = generate_man_page();
        assert!(man.contains("sobriquet"));
        assert!(man.contains("\\-\\-list"));
        assert!(man.contains("stats"));
        assert!(man.contains("audit"));
        assert!(man.contains("init"));
    }

    #[test]
    fn test_args_defaults() {
        let args = Args::try_parse_from(["sobriquet"]).unwrap();
        assert!(!args.list);
        assert!(!args.refresh);
        assert!(!args.print_query);
        assert_eq!(args.format, OutputFormat::Plain);
        assert!(args.command.is_none());
    }

    #[test]
    fn test_shell_option_all_variants() {
        let zsh =
            Args::try_parse_from(["sobriquet", "--shell", "zsh"]).unwrap();
        assert_eq!(zsh.shell, Some(Shell::Zsh));
        let bash =
            Args::try_parse_from(["sobriquet", "--shell", "bash"]).unwrap();
        assert_eq!(bash.shell, Some(Shell::Bash));
        let fish =
            Args::try_parse_from(["sobriquet", "--shell", "fish"]).unwrap();
        assert_eq!(fish.shell, Some(Shell::Fish));
    }

    #[test]
    fn test_output_format_json_pretty() {
        let args =
            Args::try_parse_from(["sobriquet", "--format", "json-pretty"])
                .unwrap();
        assert_eq!(args.format, OutputFormat::JsonPretty);
    }

    #[test]
    fn test_short_flags() {
        let args = Args::try_parse_from(["sobriquet", "-l"]).unwrap();
        assert!(args.list);
        let args = Args::try_parse_from(["sobriquet", "-r"]).unwrap();
        assert!(args.refresh);
    }

    #[test]
    fn test_args_with_subcommand() {
        let args = Args::try_parse_from(["sobriquet", "stats"]).unwrap();
        assert!(args.command.is_some());
    }

    #[test]
    fn test_frecency_with_large_dataset() {
        let mut aliases = vec![];
        for i in 0..100 {
            aliases.push(Alias {
                name: format!("alias{i}"),
                command: format!("cmd {i}"),
            });
        }
        let mut stats = UsageStats::default();
        for i in 0..100 {
            stats.record_usage(&format!("alias{}", i % 20));
        }
        let sorted = sort_by_frecency(&aliases, &stats);
        assert_eq!(sorted.len(), 100);
    }

    #[test]
    fn test_app_version_constant() {
        assert!(!APP_VERSION.is_empty());
        assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_app_name_constant() {
        assert_eq!(APP_NAME, "sobriquet");
    }

    #[test]
    fn parse_json_pretty_format() {
        let args =
            Args::try_parse_from(["sobriquet", "--format", "json-pretty"])
                .unwrap();
        assert_eq!(args.format, OutputFormat::JsonPretty);
    }

    #[test]
    fn sort_by_frecency_empty() {
        let aliases = vec![];
        let stats = UsageStats::default();
        let sorted = sort_by_frecency(&aliases, &stats);
        assert!(sorted.is_empty());
    }

    #[test]
    fn sort_by_frecency_with_stats() {
        let alias1 =
            Alias { name: "a".to_owned(), command: "echo a".to_owned() };
        let alias2 =
            Alias { name: "b".to_owned(), command: "echo b".to_owned() };
        let aliases = vec![alias1.clone(), alias2.clone()];

        let mut stats = UsageStats::default();
        for _ in 0..5 {
            stats.record_usage("b");
        }
        for _ in 0..10 {
            stats.record_usage("a");
        }

        let sorted = sort_by_frecency(&aliases, &stats);
        assert_eq!(sorted[0].name, "a");
        assert_eq!(sorted[1].name, "b");
    }

    #[test]
    fn man_page_contains_version() {
        let man = generate_man_page();
        assert!(man.contains(APP_VERSION));
    }

    #[test]
    fn man_page_contains_subcommands() {
        let man = generate_man_page();
        assert!(man.contains("init"));
        assert!(man.contains("generate"));
        assert!(man.contains("config"));
        assert!(man.contains("audit"));
    }

    #[test]
    fn parse_multiple_options() {
        let args = Args::try_parse_from([
            "sobriquet",
            "--list",
            "--format",
            "json",
            "--shell",
            "bash",
            "--color",
            "always",
        ])
        .unwrap();

        assert!(args.list);
        assert_eq!(args.format, OutputFormat::Json);
        assert_eq!(args.shell, Some(Shell::Bash));
        assert_eq!(args.color, Some(ColorChoice::Always));
    }

    #[test]
    fn fish_shell_parsing() {
        let args =
            Args::try_parse_from(["sobriquet", "--shell", "fish"]).unwrap();
        assert_eq!(args.shell, Some(Shell::Fish));
    }

    #[test]
    fn test_output_aliases_json() {
        let aliases =
            vec![Alias { name: "ls".to_owned(), command: "eza".to_owned() }];
        let result = output_aliases(&aliases, OutputFormat::Json, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_aliases_json_pretty() {
        let aliases = vec![Alias {
            name: "ll".to_owned(),
            command: "eza -la".to_owned(),
        }];
        let result = output_aliases(&aliases, OutputFormat::JsonPretty, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_aliases_plain_with_colors() {
        let aliases = vec![Alias {
            name: "gs".to_owned(),
            command: "git status".to_owned(),
        }];
        let result = output_aliases(&aliases, OutputFormat::Plain, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_aliases_plain_no_colors() {
        let aliases = vec![Alias {
            name: "gc".to_owned(),
            command: "git commit".to_owned(),
        }];
        let result = output_aliases(&aliases, OutputFormat::Plain, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_aliases_empty() {
        let aliases: Vec<Alias> = vec![];
        let result = output_aliases(&aliases, OutputFormat::Plain, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_completions_zsh() {
        let mut cmd = Args::command();
        generate_completions(CompletionShell::Zsh, &mut cmd);
        // If it doesn't panic, it works
    }

    #[test]
    fn test_generate_completions_bash() {
        let mut cmd = Args::command();
        generate_completions(CompletionShell::Bash, &mut cmd);
        // If it doesn't panic, it works
    }

    #[test]
    fn test_generate_completions_fish() {
        let mut cmd = Args::command();
        generate_completions(CompletionShell::Fish, &mut cmd);
        // If it doesn't panic, it works
    }

    #[test]
    fn test_generate_man_page_content() {
        let man = generate_man_page();
        assert!(man.contains("sobriquet"));
        assert!(man.contains(".SH NAME"));
        assert!(man.contains(".SH DESCRIPTION"));
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Plain);
    }

    #[test]
    fn test_parse_refresh_flag() {
        let args = Args::try_parse_from(["sobriquet", "-r"]).unwrap();
        assert!(args.refresh);
        let args = Args::try_parse_from(["sobriquet", "--refresh"]).unwrap();
        assert!(args.refresh);
    }

    #[test]
    fn test_commands_init_subcommand() {
        let args = Args::try_parse_from(["sobriquet", "init", "zsh"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Init { .. })));
    }

    #[test]
    fn test_commands_config_subcommand() {
        let args = Args::try_parse_from(["sobriquet", "config"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Config)));
    }

    #[test]
    fn test_commands_stats_subcommand() {
        let args = Args::try_parse_from(["sobriquet", "stats"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Stats { .. })));
    }

    #[test]
    fn test_commands_audit_subcommand() {
        let args = Args::try_parse_from(["sobriquet", "audit"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Audit { .. })));
    }

    #[test]
    fn test_commands_generate_subcommand() {
        let args =
            Args::try_parse_from(["sobriquet", "generate", "man"]).unwrap();
        assert!(matches!(args.command, Some(Commands::Generate { .. })));
    }

    #[test]
    fn test_stats_clear_subcommand() {
        let args =
            Args::try_parse_from(["sobriquet", "stats", "clear"]).unwrap();
        if let Some(Commands::Stats { action }) = args.command {
            assert!(matches!(action, Some(StatsAction::Clear)));
        }
    }

    #[test]
    fn test_audit_with_kind() {
        let args =
            Args::try_parse_from(["sobriquet", "audit", "secrets"]).unwrap();
        if let Some(Commands::Audit { kind }) = args.command {
            assert_eq!(kind, Some(AuditKind::Secrets));
        }
    }

    #[test]
    #[cfg(feature = "interactive")]
    fn test_alias_item_new() {
        let alias = Alias::new("test", "echo test");
        let item = AliasItem::new(alias.clone(), None, vec![], vec![], None);
        assert_eq!(item.alias.name, "test");
        assert!(!item.display_raw.is_empty());
    }

    #[test]
    #[cfg(feature = "interactive")]
    fn test_alias_item_text() {
        let alias = Alias::new("test", "echo test");
        let item = AliasItem::new(alias, None, vec![], vec![], None);
        let text = item.text();
        assert!(text.contains("test"));
    }
}
