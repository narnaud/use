use preferences::{AppInfo, Preferences, PreferencesMap};

const APP_INFO: AppInfo = AppInfo {
    name: "use",
    author: "narnaud",
};
const UPDATE_TITLE_KEY: &str = "update-title";

pub struct Settings {
    pub update_title: bool,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum SettingsKey {
    /// Change the terminal title based on the environment chosen
    UpdateTitle,
}

impl Settings {
    fn load() -> PreferencesMap<String> {
        PreferencesMap::load(&APP_INFO, env!("CARGO_PKG_NAME")).unwrap_or_default()
    }

    pub fn set(key: SettingsKey, value: &str) {
        let mut settings = Settings::default();
        match key {
            SettingsKey::UpdateTitle => settings.update_title = value.parse().unwrap_or(false),
        }
        settings.save();
    }

    fn save(self) {
        let mut prefs: PreferencesMap<String> = Default::default();
        prefs.insert(UPDATE_TITLE_KEY.into(), self.update_title.to_string());
        prefs
            .save(&APP_INFO, env!("CARGO_PKG_NAME"))
            .expect("Failed to save preferences");
    }

    pub fn print() {
        let settings = Settings::default();
        println!("update-title    {}", settings.update_title);
    }
}

impl Default for Settings {
    fn default() -> Self {
        let prefs = Settings::load();
        Self {
            update_title: prefs.get(UPDATE_TITLE_KEY).is_none_or(|s| s == "true"),
        }
    }
}
