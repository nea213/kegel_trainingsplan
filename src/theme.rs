use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemeMode {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }

    pub const fn document_theme(self) -> Option<&'static str> {
        match self {
            Self::Light => Some("light"),
            Self::Dark => Some("dark"),
            Self::System => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Light => "Hell",
            Self::Dark => "Dunkel",
            Self::System => "System",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Light => "Helle Oberflaeche fuer klare Tagesplanung",
            Self::Dark => "Dunkles Layout mit ruhigem Fokus auf Trainingsdaten",
            Self::System => "Folgt automatisch deinem Geraete-Theme",
        }
    }

    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub fn from_storage(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub mode: Signal<ThemeMode>,
    pub synced_user_id: Signal<Option<i32>>,
}

impl ThemeContext {
    pub fn current(self) -> ThemeMode {
        (self.mode)()
    }

    pub fn needs_sync(self, user_id: i32) -> bool {
        (self.synced_user_id)() != Some(user_id)
    }

    pub fn sync_authenticated_user(mut self, user_id: i32, theme_mode: ThemeMode) {
        self.synced_user_id.set(Some(user_id));
        self.mode.set(theme_mode);
    }

    pub fn set_authenticated_user_theme(mut self, user_id: i32, theme_mode: ThemeMode) {
        self.synced_user_id.set(Some(user_id));
        self.mode.set(theme_mode);
    }

    pub fn reset_to_system(mut self) {
        self.synced_user_id.set(None);
        self.mode.set(ThemeMode::System);
    }
}
