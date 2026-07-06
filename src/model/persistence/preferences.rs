use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::model::time_utils::TimeFormat;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserPreferences {
    #[serde(default)]
    pub monospace_paths: bool,
    #[serde(default)]
    pub highlight_duplicates: bool,
    #[serde(default)]
    pub time_format: TimeFormat,
    #[serde(default = "default_true")]
    pub deletion_confirmation: bool,
    #[serde(default = "default_true")]
    pub trash_confirmation: bool,
    #[serde(default)]
    pub treemap_borders: bool,
}

const fn default_true() -> bool {
    true
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            monospace_paths: false,
            highlight_duplicates: false,
            time_format: TimeFormat::default(),
            deletion_confirmation: true,
            trash_confirmation: true,
            treemap_borders: false,
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "eDirStat").map(|dirs| dirs.config_dir().join("config.toml"))
}

#[must_use]
pub fn load_preferences() -> UserPreferences {
    if let Some(path) = get_config_path()
        && let Ok(contents) = std::fs::read_to_string(&path)
        && let Ok(prefs) = toml::from_str(&contents)
    {
        return prefs;
    }
    UserPreferences::default() // Continue with default settings safely if unreadable/absent
}

pub fn save_preferences(prefs: &UserPreferences) {
    if let Some(path) = get_config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(contents) = toml::to_string_pretty(prefs) {
            let _ = std::fs::write(path, contents);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(prefs.deletion_confirmation);
        assert!(prefs.trash_confirmation);
        assert!(!prefs.monospace_paths);
        assert!(!prefs.highlight_duplicates);
        assert!(!prefs.treemap_borders);
    }

    #[test]
    fn test_deserialize_legacy_config() -> Result<(), toml::de::Error> {
        // Legacy config missing the confirmation fields should default to true
        // and missing treemap_borders should default to false
        let legacy_toml = r"
            monospace_paths = true
            highlight_duplicates = false
        ";
        let prefs: UserPreferences = toml::from_str(legacy_toml)?;
        assert!(prefs.monospace_paths);
        assert!(!prefs.highlight_duplicates);
        assert!(prefs.deletion_confirmation);
        assert!(prefs.trash_confirmation);
        assert!(!prefs.treemap_borders);

        Ok(())
    }

    #[test]
    fn test_roundtrip_config() -> anyhow::Result<()> {
        let prefs = UserPreferences {
            deletion_confirmation: false,
            trash_confirmation: false,
            monospace_paths: true,
            ..Default::default()
        };

        let serialized = toml::to_string(&prefs)?;
        let deserialized: UserPreferences = toml::from_str(&serialized)?;
        assert_eq!(prefs, deserialized);

        Ok(())
    }
}
