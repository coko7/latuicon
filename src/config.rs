use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Deserializer};

use crate::icon_picker::IconPickerTab;
use crate::theme::Theme;

/// Persistent user configuration, loaded from TOML file.
/// Falls back to CLI/env defaults for `None` values.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, deserialize_with = "deserialize_theme")]
    pub theme: Option<Theme>,

    #[serde(default, deserialize_with = "deserialize_tab")]
    pub default_tab: Option<IconPickerTab>,
}

impl Config {
    /// Loads config from `explicit_path`, or from the default config location
    /// if `explicit_path` is `None`.
    pub fn load(explicit_path: Option<&Path>) -> Result<Config, String> {
        match explicit_path {
            Some(path) => Self::load_from(path, true),
            None => Self::load_from(&default_path(), false),
        }
    }

    fn load_from(path: &Path, required: bool) -> Result<Config, String> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound && !required => {
                return Ok(Config::default());
            }
            Err(err) => {
                return Err(format!(
                    "could not read config file {}: {err}",
                    path.display()
                ));
            }
        };

        toml::from_str(&content)
            .map_err(|err| format!("could not parse config file {}: {err}", path.display()))
    }
}

/// Per-OS config directory:
/// * Linux: `$XDG_CONFIG_HOME/latuicon`
/// * macOS: `~/Library/Application Support/latuicon`
/// * Windows: `%APPDATA%\latuicon`
fn default_path() -> PathBuf {
    match dirs::config_dir() {
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
        Some(s) => Theme::from_str(&s, true)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

fn deserialize_tab<'de, D>(deserializer: D) -> Result<Option<IconPickerTab>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(s) => IconPickerTab::from_str(&s, true)
            .map(Some)
            .map_err(serde::de::Error::custom),
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
        let config = parse(
            "theme = \"dracula\"\ntab_order = [\"all\", \"emoji\"]\n[database]\nenabled = true\n",
        );
        assert_eq!(config.theme, Some(Theme::Dracula));
    }

    #[test]
    fn invalid_theme_value_fails_to_parse() {
        let result: Result<Config, _> = toml::from_str("theme = \"not-a-theme\"\n");
        assert!(result.is_err());
    }

    #[test]
    fn load_returns_defaults_when_default_location_is_missing() {
        let config = Config::load_from(
            Path::new("/nonexistent/path/latuicon-test-config.toml"),
            false,
        )
        .unwrap();
        assert_eq!(config.theme, None);
        assert_eq!(config.default_tab, None);
    }

    #[test]
    fn load_errs_when_explicit_config_path_is_missing() {
        let result = Config::load(Some(Path::new(
            "/nonexistent/path/latuicon-test-config.toml",
        )));
        assert!(result.is_err());
    }

    #[test]
    fn load_errs_on_invalid_toml_instead_of_falling_back() {
        let path = std::env::temp_dir().join("latuicon-test-invalid-config.toml");
        fs::write(&path, "theme = \"not-a-theme\"\n").unwrap();

        let result = Config::load(Some(&path));
        fs::remove_file(&path).ok();

        assert!(result.is_err());
    }
}
