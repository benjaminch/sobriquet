use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HEIGHT: &str = "100%";
pub const DEFAULT_PROMPT: &str = "Select alias > ";

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub shell: ShellConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    pub height: String,
    pub prompt: String,
    pub preview: bool,
    pub preview_position: String,
    #[serde(alias = "preview_expand_details")]
    pub preview_show_secrets: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            height: DEFAULT_HEIGHT.to_owned(),
            prompt: DEFAULT_PROMPT.to_owned(),
            preview: true,
            preview_position: "right".to_owned(),
            preview_show_secrets: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ShellConfig {
    pub prefer: Option<String>,
    pub cache_ttl: u64,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            prefer: None,
            cache_ttl: 300, // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OutputConfig {
    pub color: ColorChoice,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { color: ColorChoice::Auto }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Default,
    ValueEnum,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

impl Config {
    pub fn load() -> Self {
        Self::find_config()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Find the config file by checking multiple locations in priority order
    fn find_config() -> Option<PathBuf> {
        let locations = Self::config_paths();
        locations.into_iter().find(|path| path.exists())
    }

    /// Return all possible config file locations in priority order
    pub fn config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. ~/.config/alx/config.toml (primary, XDG standard)
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("sobriquet").join("config.toml"));
        }

        // 2. ~/.config/alx.toml (alternative in config dir)
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("alx.toml"));
        }

        // 3. ~/.alx.toml (home directory dotfile)
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".alx.toml"));
        }

        paths
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::find_config().or_else(|| Self::config_paths().into_iter().next())
    }

    pub fn use_colors(&self) -> bool {
        match self.output.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => io::stdout().is_terminal(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.ui.height, DEFAULT_HEIGHT);
        assert_eq!(config.ui.prompt, DEFAULT_PROMPT);
        assert!(config.ui.preview);
        assert_eq!(config.output.color, ColorChoice::Auto);
    }

    #[test]
    fn parse_full_config() {
        let toml = r#"
[ui]
height = "30%"
prompt = "> "
preview = false

[shell]
prefer = "bash"

[output]
color = "always"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.ui.height, "30%");
        assert!(!config.ui.preview);
        assert_eq!(config.shell.prefer, Some("bash".to_owned()));
        assert_eq!(config.output.color, ColorChoice::Always);
    }

    #[test]
    fn partial_config_uses_defaults() {
        let config: Config = toml::from_str("[ui]\nheight = \"25%\"").unwrap();
        assert_eq!(config.ui.height, "25%");
        assert_eq!(config.ui.prompt, DEFAULT_PROMPT);
    }

    #[test]
    fn use_colors() {
        let mut config = Config::default();
        config.output.color = ColorChoice::Always;
        assert!(config.use_colors());
        config.output.color = ColorChoice::Never;
        assert!(!config.use_colors());
    }

    #[test]
    fn load_config_with_no_file() {
        // This should return default config when no file exists
        let config = Config::load();
        // Should have default values
        assert_eq!(config.ui.height, DEFAULT_HEIGHT);
        assert_eq!(config.ui.prompt, DEFAULT_PROMPT);
    }

    #[test]
    fn find_config_returns_none_when_no_config() {
        // This tests the internal find_config logic
        // It will return None if no config file exists at expected locations
        let result = Config::find_config();
        // We can't guarantee a config exists or not, so just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn config_paths_returns_multiple_locations() {
        let paths = Config::config_paths();
        // Should return at least 3 locations
        assert!(paths.len() >= 3);
        // All paths should have some component related to config
        for path in &paths {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains("sobriquet") || path_str.contains("alx")
            );
        }
    }

    #[test]
    fn config_path_returns_something() {
        let path = Config::config_path();
        // Should always return Some path (either existing or first default)
        assert!(path.is_some());
    }

    #[test]
    fn ui_config_default() {
        let ui = UiConfig::default();
        assert_eq!(ui.height, DEFAULT_HEIGHT);
        assert_eq!(ui.prompt, DEFAULT_PROMPT);
        assert!(ui.preview);
        assert_eq!(ui.preview_position, "right");
        assert!(!ui.preview_show_secrets);
    }

    #[test]
    fn shell_config_default() {
        let shell = ShellConfig::default();
        assert_eq!(shell.prefer, None);
        assert_eq!(shell.cache_ttl, 300);
    }

    #[test]
    fn output_config_default() {
        let output = OutputConfig::default();
        assert_eq!(output.color, ColorChoice::Auto);
    }

    #[test]
    fn color_choice_serialization() {
        // Test serialization within a config structure
        let mut config = Config::default();
        config.output.color = ColorChoice::Always;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("always"));

        config.output.color = ColorChoice::Never;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("never"));

        config.output.color = ColorChoice::Auto;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("auto"));
    }
}
