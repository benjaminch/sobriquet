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
use crate::preview::{CommandAnalysis, find_similar};
use crate::shell::{InitShell, Shell, generate_init_script};
use crate::source::AliasLocation;
use crate::stats::{UsageRecord, UsageStats, display_stats};

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
    display: String,
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
        let display = alias.to_string();
        Self { alias, display, stats, all_names, all_aliases, source_location }
    }
}

#[cfg(feature = "interactive")]
impl SkimItem for AliasItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.display)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let analysis = CommandAnalysis::new(&self.alias.command);

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

    let mut builder = SkimOptionsBuilder::default();
    builder
        .height(Some(&config.ui.height))
        .multi(false)
        .prompt(Some(&config.ui.prompt))
        .preview(preview_opt)
        .query(query)
        .nosort(true); // Preserve our frecency sort order

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
        if print_query && !output.query.is_empty() {
            return Ok((output.query.clone(), output.query));
        }
        return Err(AlxError::UserAborted);
    }

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
    writeln!(io::stderr(), "Please use 'alx --list' to see all aliases")?;

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
.B alx
[\fIOPTIONS\fR]
.br
.B alx
\fBinit\fR \fISHELL\fR
.br
.B alx
\fBgenerate\fR \fIKIND\fR
.br
.B alx
\fBconfig\fR
.br
.B alx
\fBstats\fR [\fBclear\fR]
.br
.B alx
\fBaudit\fR [\fIKIND\fR]
.SH DESCRIPTION
.B alx
reads your shell aliases and presents them in an interactive fuzzy finder.
Select an alias and its expanded command will be output to stdout.
.PP
Usage is tracked automatically. Use \fBalx stats\fR to view statistics.
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
Configuration file: \fI~/.config/alx/config.toml\fR
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
Aliases are cached to \fI~/.cache/alx/aliases.json\fR for fast startup.
The default cache TTL is 300 seconds (5 minutes).
Use \fB\-\-refresh\fR to force a cache update.
Set \fBcache_ttl = 0\fR in the config to disable caching.
.SH FILES
.TP
.I ~/.config/alx/config.toml
Configuration file
.TP
.I ~/.cache/alx/aliases.json
Cached aliases
.TP
.I ~/.local/share/alx/stats.json
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
.B alx
Open interactive fuzzy finder
.TP
.B alx \-\-query git
Open with "git" pre-filled
.TP
.B alx \-\-list
List all aliases
.TP
.B alx stats
Show usage statistics
.TP
.B alx init zsh >> ~/.zshrc
Add shell integration
.TP
.B alx audit
Check for secrets and duplicates
.TP
.B alx audit secrets
Check for embedded secrets only
.SH SEE ALSO
.BR alias (1),
.BR fzf (1)
.SH BUGS
https://github.com/benjaminch/alx/issues
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
        assert_eq!(args.query, Some("test".to_string()));
    }

    #[test]
    fn parse_refresh_flag() {
        let args = Args::try_parse_from(["sobriquet", "--refresh"]).unwrap();
        assert!(args.refresh);
        let args = Args::try_parse_from(["sobriquet", "-r"]).unwrap();
        assert!(args.refresh);
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
    fn output_format_default_is_plain() {
        let args = Args::try_parse_from(["sobriquet"]).unwrap();
        assert_eq!(args.format, OutputFormat::Plain);
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
            Alias { name: "a".to_string(), command: "echo a".to_string() };
        let alias2 =
            Alias { name: "b".to_string(), command: "echo b".to_string() };
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
    fn output_format_json() {
        let args =
            Args::try_parse_from(["sobriquet", "--format", "json"]).unwrap();
        assert_eq!(args.format, OutputFormat::Json);
    }
}
