use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HEIGHT: &str = "50%";
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
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            height: DEFAULT_HEIGHT.to_owned(),
            prompt: DEFAULT_PROMPT.to_owned(),
            preview: true,
            preview_position: "right".to_owned(),
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
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            })
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("alx").join("config.toml"))
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
}
