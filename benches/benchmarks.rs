#![allow(clippy::unwrap_used)]

use criterion::{
    BenchmarkId, Criterion, black_box, criterion_group, criterion_main,
};
use std::fmt::Write;
use std::process::Command;
use std::time::Duration;

/// Generate sample alias output similar to real zsh output
fn generate_sample_aliases(count: usize) -> String {
    let mut output = String::new();

    // Add some realistic aliases
    let realistic = [
        "ls=eza",
        "cat=bat",
        "grep=rg",
        "cd=z",
        "ll='eza -la'",
        "la='eza -a'",
        "tree=broot",
        "top=htop",
        "du=dust",
        "vim=nvim",
    ];

    for alias in &realistic {
        output.push_str(alias);
        output.push('\n');
    }

    // Add generated aliases to reach the count
    for i in realistic.len()..count {
        writeln!(output, "alias_{i}='command_{i} --flag --option=value'")
            .unwrap();
    }

    output
}

/// Parse alias output (duplicated from main.rs for benchmarking)
fn parse_alias_output(content: &str) -> Vec<(String, String)> {
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let line = line.strip_prefix("alias ").unwrap_or(line);

        if let Some(eq_pos) = line.find('=') {
            let name = &line[..eq_pos];
            let mut command = &line[eq_pos + 1..];

            if (command.starts_with('\'') && command.ends_with('\''))
                || (command.starts_with('"') && command.ends_with('"'))
            {
                command = &command[1..command.len() - 1];
            }

            if name.is_empty()
                || (!name.starts_with(|c: char| c.is_alphabetic() || c == '_'))
            {
                continue;
            }

            aliases.push((name.to_owned(), command.to_owned()));
        }
    }

    aliases
}

/// Benchmark parsing alias output with different sizes
fn bench_parse_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_aliases");

    for size in &[10, 50, 100, 250, 500, 1000] {
        let input = generate_sample_aliases(*size);

        group.bench_with_input(
            BenchmarkId::new("aliases", size),
            &input,
            |b, input| {
                b.iter(|| parse_alias_output(black_box(input)));
            },
        );
    }

    group.finish();
}

/// Benchmark the full CLI invocation time (cold start)
fn bench_cli_invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_invocation");
    group.sample_size(20); // Fewer samples since this is slower
    group.measurement_time(Duration::from_secs(10));

    // Benchmark just getting help (minimal work)
    group.bench_function("help_flag", |b| {
        b.iter(|| Command::new("./target/release/alx").arg("--help").output());
    });

    group.finish();
}

/// Benchmark collecting aliases from shell
fn bench_collect_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("collect_aliases");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("zsh_alias_command", |b| {
        b.iter(|| Command::new("zsh").args(["-ic", "alias"]).output());
    });

    group.finish();
}

/// Benchmark display formatting
fn bench_display_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("display_formatting");

    let aliases: Vec<(String, String)> = (0..500)
        .map(|i| (format!("alias_{i}"), format!("command_{i} --flag")))
        .collect();

    group.bench_function("format_500_aliases", |b| {
        b.iter(|| {
            let _: Vec<String> = aliases
                .iter()
                .map(|(name, cmd)| format!("{name} -> {cmd}"))
                .collect();
        });
    });

    group.bench_function("join_500_aliases", |b| {
        let formatted: Vec<String> = aliases
            .iter()
            .map(|(name, cmd)| format!("{name} -> {cmd}"))
            .collect();

        b.iter(|| black_box(formatted.join("\n")));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_aliases,
    bench_display_formatting,
    bench_collect_aliases,
    bench_cli_invocation,
);
criterion_main!(benches);
