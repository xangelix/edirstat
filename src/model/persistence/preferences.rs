use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::model::time_utils::TimeFormat;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserPreferences {
    #[serde(default)]
    pub monospace_paths: bool,
    #[serde(default)]
    pub highlight_duplicates: bool,
    #[serde(default)]
    pub time_format: TimeFormat,
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
