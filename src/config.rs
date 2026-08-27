use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::ValueEnum;
use serde::{Deserialize, Deserializer};

use crate::icon_picker::IconPickerTab;
use crate::theme::Theme;

/// Persistent user configuration, loaded from a TOML file.
///
/// Every field is optional: an absent file, an absent field, or a value
/// that fails to parse falls back to the CLI/env defaults in `main.rs`.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_theme")]
    pub theme: Option<Theme>,
    #[serde(default, deserialize_with = "deserialize_tab")]
    pub default_tab: Option<IconPickerTab>,
}

impl Config {
    /// Loads config from `path`, or from the default XDG location if `path`
    /// is `None`. Missing file: silent defaults. Unreadable/invalid file: a
    /// warning on stderr, then defaults — never fatal.
    pub fn load(path: Option<&Path>) -> Config {
        let path = path.map(Path::to_path_buf).unwrap_or_else(default_path);

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Config::default(),
            Err(err) => {
                eprintln!(
                    "latuicon: warning: could not read config file {}: {err}",
                    path.display()
                );
                return Config::default();
            }
        };

        match toml::from_str(&content) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "latuicon: warning: could not parse config file {}: {err}",
                    path.display()
                );
                Config::default()
            }
        }
    }
}

/// `$XDG_CONFIG_HOME/latuicon/config.toml`, falling back to
/// `~/.config/latuicon/config.toml` when `XDG_CONFIG_HOME` is unset/empty.
fn default_path() -> PathBuf {
    let config_home = env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var("HOME").ok().map(|home| PathBuf::from(home).join(".config")));

    match config_home {
        Some(dir) => dir.join("latuicon").join("config.toml"),
        None => PathBuf::from("latuicon.toml"),
    }
}

fn deserialize_theme<'de, D>(deserializer: D) -> Result<Option<Theme>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(s) => Theme::from_str(&s, true).map(Some).map_err(serde::de::Error::custom),
    }
}

fn deserialize_tab<'de, D>(deserializer: D) -> Result<Option<IconPickerTab>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(s) => IconPickerTab::from_str(&s, true).map(Some).map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml_str: &str) -> Config {
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn empty_config_has_no_overrides() {
        let config = parse("");
        assert_eq!(config.theme, None);
        assert_eq!(config.default_tab, None);
    }

    #[test]
    fn parses_theme_and_default_tab() {
        let config = parse("theme = \"mocha\"\ndefault_tab = \"nerd-font\"\n");
        assert_eq!(config.theme, Some(Theme::Mocha));
        assert_eq!(config.default_tab, Some(IconPickerTab::NerdFont));
    }

    #[test]
    fn accepts_tab_aliases_and_is_case_insensitive() {
        let config = parse("default_tab = \"NERD\"\n");
        assert_eq!(config.default_tab, Some(IconPickerTab::NerdFont));
    }

    #[test]
    fn unknown_fields_are_ignored_for_forward_compatibility() {
        let config = parse("theme = \"dracula\"\ntab_order = [\"all\", \"emoji\"]\n[database]\nenabled = true\n");
        assert_eq!(config.theme, Some(Theme::Dracula));
    }

    #[test]
    fn invalid_theme_value_fails_to_parse() {
        let result: Result<Config, _> = toml::from_str("theme = \"not-a-theme\"\n");
        assert!(result.is_err());
    }

    #[test]
    fn load_returns_defaults_when_file_missing() {
        let config = Config::load(Some(Path::new("/nonexistent/path/latuicon-test-config.toml")));
        assert_eq!(config.theme, None);
        assert_eq!(config.default_tab, None);
    }
}
