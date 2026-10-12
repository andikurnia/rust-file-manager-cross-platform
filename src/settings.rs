use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub window_width: i32,
    pub window_height: i32,
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    List,
    Column,
    Icon,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_width: 900,
            window_height: 650,
            view_mode: ViewMode::List,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, content);
        }
    }

    fn settings_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::home_dir().unwrap_or_else(|| "/".into());
                home.join(".config")
            });
        config_dir.join("finder").join("settings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.window_width, 900);
        assert_eq!(settings.window_height, 650);
        assert_eq!(settings.view_mode, ViewMode::List);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings {
            window_width: 1024,
            window_height: 768,
            view_mode: ViewMode::Icon,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.window_width, 1024);
        assert_eq!(deserialized.window_height, 768);
        assert_eq!(deserialized.view_mode, ViewMode::Icon);
    }

    #[test]
    fn test_view_mode_equality() {
        assert_eq!(ViewMode::List, ViewMode::List);
        assert_eq!(ViewMode::Column, ViewMode::Column);
        assert_eq!(ViewMode::Icon, ViewMode::Icon);
        assert_ne!(ViewMode::List, ViewMode::Icon);
        assert_ne!(ViewMode::List, ViewMode::Column);
    }
}
