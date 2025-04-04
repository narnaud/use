use preferences::{AppInfo, Preferences, PreferencesMap};

const APP_INFO: AppInfo = AppInfo {
    name: "use",
    author: "narnaud",
};
const UPDATE_TITLE_KEY: &str = "update-title";

/// Get the preferences map
fn load() -> PreferencesMap<String> {
    PreferencesMap::load(&APP_INFO, env!("CARGO_PKG_NAME")).unwrap_or_default()
}

/// Save the preferences map
fn save(prefs: &PreferencesMap<String>) {
    prefs
        .save(&APP_INFO, env!("CARGO_PKG_NAME"))
        .expect("Failed to save preferences");
}

/// Store the update title preference.
pub fn set_update_title(enabled: bool) {
    let mut prefs = load();
    prefs.insert(UPDATE_TITLE_KEY.into(), enabled.to_string());
    save(&prefs);
}

/// Get the update title preference.
pub fn get_update_title() -> bool {
    load().get(UPDATE_TITLE_KEY).is_none_or(|s| s == "true")
}
