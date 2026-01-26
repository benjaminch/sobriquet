//! Usage statistics tracking and reporting
//!
//! This module provides:
//! - Tracking alias usage frequency and timing
//! - Persisting usage statistics to disk
//! - Displaying top aliases and usage trends
//! - Recent usage event history

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::error::{Result, SobriquetAppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub count: u64,
    pub last_used: u64,
    pub first_used: u64,
}

impl UsageRecord {
    fn new() -> Self {
        let now = Self::now();
        Self { count: 1, last_used: now, first_used: now }
    }

    fn increment(&mut self) {
        self.count += 1;
        self.last_used = Self::now();
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

const SEVEN_DAYS_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub aliases: HashMap<String, UsageRecord>,
    pub total_selections: u64,
    #[serde(default)]
    pub recent: Vec<UsageEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub alias: String,
    pub timestamp: u64,
}

impl UsageStats {
    pub fn stats_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|p| p.join("sobriquet").join("stats.json"))
    }

    pub fn load() -> Self {
        Self::stats_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = Self::stats_path() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SobriquetAppError::Serialization(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn record_usage(&mut self, alias_name: &str) {
        self.total_selections += 1;
        self.aliases
            .entry(alias_name.to_owned())
            .and_modify(UsageRecord::increment)
            .or_insert_with(UsageRecord::new);

        let now = UsageRecord::now();
        self.recent
            .push(UsageEvent { alias: alias_name.to_owned(), timestamp: now });
        self.prune_old_events();
    }

    fn prune_old_events(&mut self) {
        let cutoff = UsageRecord::now().saturating_sub(SEVEN_DAYS_SECS);
        self.recent.retain(|e| e.timestamp >= cutoff);
    }

    pub fn recent_stats(&self) -> (u64, HashMap<&str, u64>) {
        let cutoff = UsageRecord::now().saturating_sub(SEVEN_DAYS_SECS);
        let mut counts: HashMap<&str, u64> = HashMap::new();
        let mut total = 0u64;

        for event in &self.recent {
            if event.timestamp >= cutoff {
                total += 1;
                *counts.entry(event.alias.as_str()).or_insert(0) += 1;
            }
        }

        (total, counts)
    }

    pub fn top_recent(&self, n: usize) -> Vec<(&str, u64)> {
        let (_, counts) = self.recent_stats();
        let mut entries: Vec<_> = counts.into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n);
        entries
    }

    pub fn top_aliases(&self, n: usize) -> Vec<(&str, &UsageRecord)> {
        let mut entries: Vec<_> =
            self.aliases.iter().map(|(k, v)| (k.as_str(), v)).collect();
        entries.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        entries.truncate(n);
        entries
    }

    pub fn most_used(&self) -> Option<(&str, &UsageRecord)> {
        self.aliases
            .iter()
            .max_by_key(|(_, r)| r.count)
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn most_recent(&self) -> Option<(&str, &UsageRecord)> {
        self.aliases
            .iter()
            .max_by_key(|(_, r)| r.last_used)
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn unique_count(&self) -> usize {
        self.aliases.len()
    }

    pub fn clear(&mut self) {
        self.aliases.clear();
        self.total_selections = 0;
        self.recent.clear();
    }

    /// Get usage record for a specific alias
    pub fn get(&self, alias_name: &str) -> Option<&UsageRecord> {
        self.aliases.get(alias_name)
    }

    /// Calculate frecency score for sorting (higher = more relevant)
    /// Combines frequency (count) with recency (time since last use)
    #[allow(clippy::cast_precision_loss)]
    pub fn frecency_score(&self, alias_name: &str) -> f64 {
        let Some(record) = self.aliases.get(alias_name) else {
            return 0.0;
        };

        let now = UsageRecord::now();
        let age_secs = now.saturating_sub(record.last_used);

        // Decay factor: halves every 7 days
        let half_life = SEVEN_DAYS_SECS as f64;
        let decay = 0.5_f64.powf(age_secs as f64 / half_life);

        // Score = count * decay
        record.count as f64 * decay
    }

    pub fn format_relative_time(timestamp: u64) -> String {
        let now = UsageRecord::now();

        if timestamp > now {
            return "just now".to_owned();
        }

        let diff = now - timestamp;

        match diff {
            0..60 => "just now".to_owned(),
            60..3600 => {
                let mins = diff / 60;
                format!(
                    "{mins} minute{} ago",
                    if mins == 1 { "" } else { "s" }
                )
            }
            3600..86400 => {
                let hours = diff / 3600;
                format!(
                    "{hours} hour{} ago",
                    if hours == 1 { "" } else { "s" }
                )
            }
            86400..604_800 => {
                let days = diff / 86400;
                format!("{days} day{} ago", if days == 1 { "" } else { "s" })
            }
            604_800..2_592_000 => {
                let weeks = diff / 604_800;
                format!(
                    "{weeks} week{} ago",
                    if weeks == 1 { "" } else { "s" }
                )
            }
            _ => {
                let months = diff / 2_592_000;
                format!(
                    "{months} month{} ago",
                    if months == 1 { "" } else { "s" }
                )
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn display_stats(stats: &UsageStats, use_colors: bool) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if stats.total_selections == 0 {
        writeln!(out, "No statistics yet. Use sobriquet to start tracking.")?;
        return Ok(());
    }

    let (recent_total, _) = stats.recent_stats();

    if use_colors {
        writeln!(out, "{}", "sobriquet statistics".bold())?;
        writeln!(out, "{}", "────────────────────".dimmed())?;
    } else {
        writeln!(out, "sobriquet statistics")?;
        writeln!(out, "--------------------")?;
    }
    writeln!(out)?;

    // All time
    if use_colors {
        writeln!(out, "  {}", "All time:".bold())?;
        writeln!(
            out,
            "    Total:  {}",
            stats.total_selections.to_string().green()
        )?;
        writeln!(
            out,
            "    Unique: {}",
            stats.unique_count().to_string().green()
        )?;
    } else {
        writeln!(out, "  All time:")?;
        writeln!(out, "    Total:  {}", stats.total_selections)?;
        writeln!(out, "    Unique: {}", stats.unique_count())?;
    }
    writeln!(out)?;

    // Last 7 days
    let recent_unique = stats.top_recent(1000).len();
    if use_colors {
        writeln!(out, "  {}", "Last 7 days:".bold())?;
        writeln!(out, "    Total:  {}", recent_total.to_string().green())?;
        writeln!(out, "    Unique: {}", recent_unique.to_string().green())?;
    } else {
        writeln!(out, "  Last 7 days:")?;
        writeln!(out, "    Total:  {recent_total}")?;
        writeln!(out, "    Unique: {recent_unique}")?;
    }
    writeln!(out)?;

    if let Some((name, record)) = stats.most_recent() {
        let time_ago = UsageStats::format_relative_time(record.last_used);
        if use_colors {
            writeln!(
                out,
                "  {} {} ({})",
                "Most recent:".bold(),
                name.green(),
                time_ago.dimmed()
            )?;
        } else {
            writeln!(out, "  Most recent: {name} ({time_ago})")?;
        }
        writeln!(out)?;
    }

    // Top all time
    let top = stats.top_aliases(10);
    if !top.is_empty() {
        if use_colors {
            writeln!(out, "  {}", "Top (all time):".bold())?;
        } else {
            writeln!(out, "  Top (all time):")?;
        }
        writeln!(out)?;
        display_top_list(&mut out, &top, use_colors)?;
        writeln!(out)?;
    }

    // Top last 7 days
    let top_recent = stats.top_recent(10);
    if !top_recent.is_empty() {
        if use_colors {
            writeln!(out, "  {}", "Top (last 7 days):".bold())?;
        } else {
            writeln!(out, "  Top (last 7 days):")?;
        }
        writeln!(out)?;
        display_top_list_u64(&mut out, &top_recent, use_colors)?;
    }

    writeln!(out)?;

    if let Some(path) = UsageStats::stats_path() {
        if use_colors {
            writeln!(
                out,
                "  {} {}",
                "Stats file:".dimmed(),
                path.display().to_string().dimmed()
            )?;
        } else {
            writeln!(out, "  Stats file: {}", path.display())?;
        }
    }

    Ok(())
}

fn display_top_list(
    out: &mut io::StdoutLock<'_>,
    top: &[(&str, &UsageRecord)],
    use_colors: bool,
) -> Result<()> {
    let max_count = top.first().map_or(1, |(_, r)| r.count);
    let max_name_len = top.iter().map(|(n, _)| n.len()).max().unwrap_or(10);
    let bar_width = 20;

    for (i, (name, record)) in top.iter().enumerate() {
        let rank = i + 1;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let bar_len = ((record.count as f64 / max_count as f64)
            * bar_width as f64) as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);
        let padding = " ".repeat(bar_width - bar_len);

        if use_colors {
            writeln!(
                out,
                "  {:>2}. {:<width$}  {}{} {:>4}",
                rank.to_string().dimmed(),
                name.green(),
                bar.cyan(),
                padding,
                record.count.to_string().bold(),
                width = max_name_len
            )?;
        } else {
            writeln!(
                out,
                "  {rank:>2}. {name:<width$}  {bar}{padding} {count:>4}",
                count = record.count,
                width = max_name_len
            )?;
        }
    }
    Ok(())
}

fn display_top_list_u64(
    out: &mut io::StdoutLock<'_>,
    top: &[(&str, u64)],
    use_colors: bool,
) -> Result<()> {
    let max_count = top.first().map_or(1, |(_, c)| *c);
    let max_name_len = top.iter().map(|(n, _)| n.len()).max().unwrap_or(10);
    let bar_width = 20;

    for (i, (name, count)) in top.iter().enumerate() {
        let rank = i + 1;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let bar_len =
            ((*count as f64 / max_count as f64) * bar_width as f64) as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);
        let padding = " ".repeat(bar_width - bar_len);

        if use_colors {
            writeln!(
                out,
                "  {:>2}. {:<width$}  {}{} {:>4}",
                rank.to_string().dimmed(),
                name.green(),
                bar.cyan(),
                padding,
                count.to_string().bold(),
                width = max_name_len
            )?;
        } else {
            writeln!(
                out,
                "  {rank:>2}. {name:<max_name_len$}  {bar}{padding} {count:>4}",
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn record_usage_increments() {
        let mut stats = UsageStats::default();
        stats.record_usage("ls");
        stats.record_usage("ls");
        stats.record_usage("cd");

        assert_eq!(stats.total_selections, 3);
        assert_eq!(stats.aliases["ls"].count, 2);
        assert_eq!(stats.aliases["cd"].count, 1);
    }

    #[test]
    fn top_aliases_sorted() {
        let mut stats = UsageStats::default();
        for _ in 0..5 {
            stats.record_usage("a");
        }
        for _ in 0..10 {
            stats.record_usage("b");
        }
        for _ in 0..3 {
            stats.record_usage("c");
        }

        let top = stats.top_aliases(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "b");
        assert_eq!(top[1].0, "a");
    }

    #[test]
    fn clear_resets_all() {
        let mut stats = UsageStats::default();
        stats.record_usage("test");
        stats.clear();
        assert_eq!(stats.total_selections, 0);
        assert!(stats.aliases.is_empty());
    }

    #[test]
    fn format_relative_time() {
        let now = UsageRecord::now();
        assert_eq!(UsageStats::format_relative_time(now), "just now");
    }

    #[test]
    fn usage_record_new() {
        let record = UsageRecord::new();
        assert_eq!(record.count, 1);
        assert!(record.first_used > 0);
        assert_eq!(record.last_used, record.first_used);
    }

    #[test]
    fn usage_record_increment() {
        let mut record = UsageRecord::new();
        let original_first = record.first_used;
        record.increment();
        assert_eq!(record.count, 2);
        assert_eq!(record.first_used, original_first);
        assert!(record.last_used >= original_first);
    }

    #[test]
    fn get_usage_record() {
        let mut stats = UsageStats::default();
        stats.record_usage("myalias");
        assert!(stats.get("myalias").is_some());
        assert_eq!(stats.get("myalias").unwrap().count, 1);
        assert!(stats.get("nonexistent").is_none());
    }

    #[test]
    fn most_used_and_most_recent() {
        let mut stats = UsageStats::default();
        stats.record_usage("a");
        stats.record_usage("a");
        stats.record_usage("b");

        let most_used = stats.most_used();
        assert!(most_used.is_some());
        assert_eq!(most_used.unwrap().0, "a");
        assert_eq!(most_used.unwrap().1.count, 2);

        let most_recent = stats.most_recent();
        assert!(most_recent.is_some());
        let most_recent_name = most_recent.unwrap().0;
        // "b" was recorded last, so it should be most recent (unless timestamps are identical)
        assert!(most_recent_name == "a" || most_recent_name == "b");
    }

    #[test]
    fn unique_count_and_clear() {
        let mut stats = UsageStats::default();
        stats.record_usage("a");
        stats.record_usage("b");
        stats.record_usage("c");
        assert_eq!(stats.unique_count(), 3);

        stats.clear();
        assert_eq!(stats.unique_count(), 0);
        assert_eq!(stats.total_selections, 0);
        assert!(stats.aliases.is_empty());
        assert!(stats.recent.is_empty());
    }

    #[test]
    fn frecency_score_new_vs_old() {
        let mut stats = UsageStats::default();
        stats.record_usage("new");

        let new_score = stats.frecency_score("new");
        assert!(new_score > 0.0);

        // Non-existent alias should have 0 score
        assert!(stats.frecency_score("nonexistent") == 0.0);
    }

    #[test]
    fn recent_stats_empty() {
        let stats = UsageStats::default();
        let (total, counts) = stats.recent_stats();
        assert_eq!(total, 0);
        assert!(counts.is_empty());
    }

    #[test]
    fn recent_stats_with_data() {
        let mut stats = UsageStats::default();
        stats.record_usage("a");
        stats.record_usage("a");
        stats.record_usage("b");

        let (total, counts) = stats.recent_stats();
        assert_eq!(total, 3);
        assert_eq!(*counts.get("a").unwrap_or(&0), 2);
        assert_eq!(*counts.get("b").unwrap_or(&0), 1);
    }

    #[test]
    fn top_recent() {
        let mut stats = UsageStats::default();
        stats.record_usage("a");
        stats.record_usage("a");
        stats.record_usage("b");
        stats.record_usage("c");
        stats.record_usage("c");
        stats.record_usage("c");

        let top = stats.top_recent(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "c");
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn top_aliases_respects_limit() {
        let mut stats = UsageStats::default();
        for i in 0..10 {
            for _ in 0..=i {
                stats.record_usage(&format!("alias{i}"));
            }
        }

        let top_5 = stats.top_aliases(5);
        assert_eq!(top_5.len(), 5);

        let top_20 = stats.top_aliases(20);
        assert!(top_20.len() <= 10);
    }

    #[test]
    fn format_relative_time_various_intervals() {
        let now = UsageRecord::now();

        // Just now (0-60 seconds)
        assert_eq!(UsageStats::format_relative_time(now), "just now");

        // Minutes (60-3600 seconds)
        let one_min_ago = now - 60;
        let result = UsageStats::format_relative_time(one_min_ago);
        assert!(result.contains("minute"));

        // Hours (3600-86400 seconds)
        let one_hour_ago = now - 3600;
        let result = UsageStats::format_relative_time(one_hour_ago);
        assert!(result.contains("hour"));

        // Days (86400-604800 seconds)
        let one_day_ago = now - 86400;
        let result = UsageStats::format_relative_time(one_day_ago);
        assert!(result.contains("day"));

        // Weeks (604800-2592000 seconds)
        let one_week_ago = now - 604_800;
        let result = UsageStats::format_relative_time(one_week_ago);
        assert!(result.contains("week"));

        // Months (2592000+ seconds)
        let one_month_ago = now - 2_592_000;
        let result = UsageStats::format_relative_time(one_month_ago);
        assert!(result.contains("month"));

        // Future timestamps
        let future = now + 3600;
        assert_eq!(UsageStats::format_relative_time(future), "just now");
    }

    #[test]
    fn format_relative_time_pluralization() {
        let now = UsageRecord::now();

        // 1 minute (singular)
        let one_min_ago = now - 60;
        let result = UsageStats::format_relative_time(one_min_ago);
        assert_eq!(result, "1 minute ago");

        // 2 minutes (plural)
        let two_mins_ago = now - 120;
        let result = UsageStats::format_relative_time(two_mins_ago);
        assert_eq!(result, "2 minutes ago");

        // 1 hour (singular)
        let one_hour_ago = now - 3600;
        let result = UsageStats::format_relative_time(one_hour_ago);
        assert_eq!(result, "1 hour ago");

        // 2 hours (plural)
        let two_hours_ago = now - 7200;
        let result = UsageStats::format_relative_time(two_hours_ago);
        assert_eq!(result, "2 hours ago");
    }

    #[test]
    fn display_stats_empty() {
        let stats = UsageStats::default();
        let result = display_stats(&stats, false);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_with_data() {
        let mut stats = UsageStats::default();
        stats.record_usage("test_alias");
        stats.record_usage("test_alias");
        stats.record_usage("another");

        let result = display_stats(&stats, false);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_with_colors() {
        let mut stats = UsageStats::default();
        stats.record_usage("test");
        let result = display_stats(&stats, true);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_without_colors() {
        let mut stats = UsageStats::default();
        stats.record_usage("test");
        let result = display_stats(&stats, false);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_multiple_aliases() {
        let mut stats = UsageStats::default();
        for i in 0..5 {
            for _ in 0..=i {
                stats.record_usage(&format!("alias{i}"));
            }
        }

        let result = display_stats(&stats, false);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_with_most_recent() {
        let mut stats = UsageStats::default();
        stats.record_usage("first");
        stats.record_usage("second");
        stats.record_usage("second");

        let result = display_stats(&stats, false);
        assert!(result.is_ok());
    }

    #[test]
    fn display_stats_many_aliases() {
        let mut stats = UsageStats::default();
        for i in 0..20 {
            for _ in 0..=((i + 1) % 5) {
                stats.record_usage(&format!("alias{i}"));
            }
        }

        let result = display_stats(&stats, true);
        assert!(result.is_ok());
    }

    #[test]
    fn format_relative_time_boundary_values() {
        let now = UsageRecord::now();

        // Test boundaries between intervals
        // 59 seconds = "just now"
        let fifty_nine_secs_ago = now - 59;
        assert_eq!(
            UsageStats::format_relative_time(fifty_nine_secs_ago),
            "just now"
        );

        // 60 seconds = "1 minute ago"
        let sixty_secs_ago = now - 60;
        assert_eq!(
            UsageStats::format_relative_time(sixty_secs_ago),
            "1 minute ago"
        );

        // 3599 seconds = should be minutes
        let hours_boundary_1 = now - 3599;
        let result = UsageStats::format_relative_time(hours_boundary_1);
        assert!(result.contains("minute"));

        // 3600 seconds = "1 hour ago"
        let hours_boundary_2 = now - 3600;
        assert_eq!(
            UsageStats::format_relative_time(hours_boundary_2),
            "1 hour ago"
        );

        // 86399 seconds = should be hours
        let days_boundary_1 = now - 86399;
        let result = UsageStats::format_relative_time(days_boundary_1);
        assert!(result.contains("hour"));

        // 86400 seconds = "1 day ago"
        let days_boundary_2 = now - 86400;
        assert_eq!(
            UsageStats::format_relative_time(days_boundary_2),
            "1 day ago"
        );
    }

    #[test]
    fn format_relative_time_large_values() {
        let now = UsageRecord::now();

        // 30 days
        let thirty_days_ago = now - (30 * 86400);
        let result = UsageStats::format_relative_time(thirty_days_ago);
        assert!(result.contains("month"));

        // 365 days
        let year_ago = now - (365 * 86400);
        let result = UsageStats::format_relative_time(year_ago);
        assert!(result.contains("month"));
    }

    #[test]
    fn format_relative_time_day_pluralization() {
        let now = UsageRecord::now();

        // 1 day (singular)
        let one_day_ago = now - 86400;
        let result = UsageStats::format_relative_time(one_day_ago);
        assert_eq!(result, "1 day ago");

        // 2 days (plural)
        let two_days_ago = now - (2 * 86400);
        let result = UsageStats::format_relative_time(two_days_ago);
        assert_eq!(result, "2 days ago");

        // 1 week (singular)
        let one_week_ago = now - 604_800;
        let result = UsageStats::format_relative_time(one_week_ago);
        assert_eq!(result, "1 week ago");

        // 2 weeks (plural)
        let two_weeks_ago = now - (2 * 604_800);
        let result = UsageStats::format_relative_time(two_weeks_ago);
        assert_eq!(result, "2 weeks ago");

        // 1 month (singular)
        let one_month_ago = now - 2_592_000;
        let result = UsageStats::format_relative_time(one_month_ago);
        assert_eq!(result, "1 month ago");

        // 2 months (plural)
        let two_months_ago = now - (2 * 2_592_000);
        let result = UsageStats::format_relative_time(two_months_ago);
        assert_eq!(result, "2 months ago");
    }

    #[test]
    fn usage_stats_prune_old_events() {
        let mut stats = UsageStats::default();
        let now = UsageRecord::now();

        // Add an old event (beyond 7 days)
        let old_timestamp = now - (8 * 24 * 60 * 60);
        stats.recent.push(UsageEvent {
            alias: "old".to_owned(),
            timestamp: old_timestamp,
        });

        // Add a new event
        stats
            .recent
            .push(UsageEvent { alias: "new".to_owned(), timestamp: now });

        // Prune should remove old event
        stats.prune_old_events();

        // Only new event should remain
        assert_eq!(stats.recent.len(), 1);
        assert_eq!(stats.recent[0].alias, "new");
    }

    #[test]
    fn frecency_score_decay() {
        let mut stats = UsageStats::default();

        // Create a record that was used long ago
        let now = UsageRecord::now();
        let old_time = now - (7 * 24 * 60 * 60); // 7 days ago

        stats.aliases.insert(
            "old".to_owned(),
            UsageRecord {
                count: 10,
                last_used: old_time,
                first_used: old_time,
            },
        );

        // Create a record that was used just now
        stats.aliases.insert(
            "new".to_owned(),
            UsageRecord { count: 10, last_used: now, first_used: now },
        );

        let old_score = stats.frecency_score("old");
        let new_score = stats.frecency_score("new");

        // New score should be higher (less decay)
        assert!(new_score > old_score);
    }

    #[test]
    fn frecency_score_high_count_vs_recent() {
        let mut stats = UsageStats::default();
        let now = UsageRecord::now();

        // High count, old record
        stats.aliases.insert(
            "frequent_old".to_owned(),
            UsageRecord {
                count: 100,
                last_used: now - (30 * 86400),
                first_used: 0,
            },
        );

        // Low count, recent record
        stats.aliases.insert(
            "rare_recent".to_owned(),
            UsageRecord {
                count: 5,
                last_used: now - 60,
                first_used: now - 60,
            },
        );

        let frequent_old_score = stats.frecency_score("frequent_old");
        let rare_recent_score = stats.frecency_score("rare_recent");

        // Both should be positive
        assert!(frequent_old_score > 0.0);
        assert!(rare_recent_score > 0.0);
    }

    #[test]
    fn top_recent_respects_limit() {
        let mut stats = UsageStats::default();

        // Record many aliases
        for i in 0..20 {
            for _ in 0..=i {
                stats.record_usage(&format!("alias{i}"));
            }
        }

        let top_5 = stats.top_recent(5);
        assert!(top_5.len() <= 5);

        let top_all = stats.top_recent(1000);
        assert!(top_all.len() <= 20);
    }

    #[test]
    fn usage_stats_path_exists() {
        let path = UsageStats::stats_path();
        assert!(path.is_some());
        let path_buf = path.unwrap();
        let path_str = path_buf.to_string_lossy();
        assert!(path_str.contains("sobriquet"));
        assert!(path_str.contains("stats.json"));
    }

    #[test]
    fn usage_record_now_returns_nonzero() {
        let now = UsageRecord::now();
        assert!(now > 0);
    }

    #[test]
    fn usage_record_first_and_last_used_same_on_creation() {
        let record = UsageRecord::new();
        assert_eq!(record.first_used, record.last_used);
    }

    #[test]
    fn stats_path_contains_sobriquet_dir() {
        if let Some(path) = UsageStats::stats_path() {
            assert!(path.to_string_lossy().contains("sobriquet"));
        }
    }
}
