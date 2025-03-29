use preferences::{AppInfo, Preferences, PreferencesMap};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use crate::colorize;
use colorize::Colorize;
use semver::Version;

/******************************************************************************
 * Constants
 *****************************************************************************/
const APP_INFO: AppInfo = AppInfo {
    name: "use",
    author: "narnaud",
};
const UPDATE_TITLE_KEY: &str = "update-title";

/******************************************************************************
 * Preferences Management
 *****************************************************************************/
mod prefs {
    use super::*;

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
}

pub use prefs::{get_update_title, set_update_title};

/******************************************************************************
 * Data Structures
 *****************************************************************************/
/// Struct to hold the pattern of the environment
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Pattern {
    path: String,
    regex: String,
}

/// Struct to hold the environment configuration
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Environment {
    display: Option<String>,
    pattern: Option<Pattern>,
    defer: Option<Vec<String>>,
    set: Option<HashMap<String, String>>,
    append: Option<HashMap<String, String>>,
    prepend: Option<HashMap<String, String>>,
    path: Option<Vec<String>>,
    #[serde(rename = "use")]
    reuse: Option<Vec<String>>,
    go: Option<String>,

    #[serde(skip)]
    version: Option<String>,
    original_key: Option<String>,
}

impl Environment {
    fn replace_placeholders(&mut self, value: &str) {
        let replace = |s: &mut Option<String>| {
            *s = s.as_ref().map(|v| v.replacen("{}", value, 1));
        };
        let replace_vec = |v: &mut Option<Vec<String>>| {
            *v = v
                .as_ref()
                .map(|items| items.iter().map(|i| i.replacen("{}", value, 1)).collect());
        };
        let replace_map = |m: &mut Option<HashMap<String, String>>| {
            *m = m.as_ref().map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.replacen("{}", value, 1)))
                    .collect()
            });
        };

        replace(&mut self.display);
        replace(&mut self.go);
        replace_vec(&mut self.defer);
        replace_map(&mut self.set);
        replace_map(&mut self.append);
        replace_map(&mut self.prepend);
        replace_vec(&mut self.path);
    }
}

/******************************************************************************
 * Config File Handling
 *****************************************************************************/
/// Create a config file in the home directory if it does not exist
pub fn create_config_file(path: &str) {
    println!("{} {} file", "    Creating".info(), path);
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    let config_content = include_str!("useconfig.yaml");
    file.write_all(config_content.as_bytes())
        .expect("Failed to write to file");
    println!("{} {} file", "     Created".success().update(), path);
}

/// Read the config file and return a map of environments
pub fn read_config_file(
    file_path: &str,
) -> Result<HashMap<String, Environment>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(fs::File::open(file_path)?);
    let mut config: HashMap<String, Environment> = serde_yaml::from_reader(reader)?;

    // Process pattern-based environments
    let pattern_configs: HashMap<String, Environment> = config
        .iter()
        .filter_map(|(name, env)| {
            env.pattern
                .as_ref()
                .map(|_| create_pattern_config(name, env))
        })
        .flatten()
        .collect();

    config.retain(|_, env| env.pattern.is_none());
    config.extend(pattern_configs);

    Ok(config)
}

/******************************************************************************
 * Environment Management
 *****************************************************************************/
/// List all environments in the config file
pub fn list_environments(envs: &HashMap<String, Environment>) -> Vec<&String> {
    let mut keys: Vec<_> = envs.keys().collect();
    keys.sort_by(|a, b| {
        let env_a = &envs[*a];
        let env_b = &envs[*b];

        // Sort by version if environments have the same original key
        if let (Some(key_a), Some(key_b)) = (&env_a.original_key, &env_b.original_key) {
            if key_a == key_b {
                if let (Some(ver_a), Some(ver_b)) = (&env_a.version, &env_b.version) {
                    if let (Ok(v_a), Ok(v_b)) = (Version::parse(ver_a), Version::parse(ver_b)) {
                        return v_b.cmp(&v_a); // Newer versions first
                    }
                }
            }
        }
        a.cmp(b) // Default lexicographical sort
    });
    keys
}

/// Use the environment by printing the configuration to the console
pub fn use_environment(name: String, envs: &HashMap<String, Environment>) {
    let keys = list_environments(envs);
    let names = resolve_dependencies(name, keys.as_ref(), envs);

    // Print environments in reverse order
    for name in names.iter().rev() {
        print_environment(&envs[name]);
    }

    finalize_setup(names.first().unwrap(), envs);
}

fn resolve_dependencies(
    name: String,
    keys: &[&String],
    envs: &HashMap<String, Environment>,
) -> Vec<String> {
    let name = find_environment_key(&name, keys).unwrap_or_else(|| {
        println!(
            "{} Environment {} not found",
            "warning:".warning(),
            name.info()
        );
        std::process::exit(1);
    });

    let mut names = vec![name.clone()];
    if let Some(reuse) = &envs[name].reuse {
        for env_name in reuse {
            let deps = resolve_dependencies(env_name.clone(), keys, envs)
                .into_iter()
                .filter(|n| !names.contains(n))
                .collect::<Vec<String>>();
            names.extend(deps);
        }
    }
    names
}

fn find_environment_key<'a>(name: &str, keys: &[&'a String]) -> Option<&'a String> {
    keys.iter()
        .find(|&&k| k == name || k.starts_with(name))
        .copied()
}

fn print_environment(env: &Environment) {
    if let Some(display) = &env.display {
        println!("{} {}", " Configuring".info(), display);
    }
    print_vector("DEFER", &env.defer);
    print_set(&env.set);
    print_add(&env.append, true);
    print_add(&env.prepend, false);
    print_vector("PATH", &env.path);
    if let Some(go) = &env.go {
        println!("GO: {}", go);
    }
    if let Some(display) = &env.display {
        println!("{} {}", "  Configured".success(), display);
    }
}

fn print_vector(label: &str, vec: &Option<Vec<String>>) {
    if let Some(vec) = vec {
        for item in vec {
            println!("{}: {}", label, item);
        }
    }
}

fn print_set(map: &Option<HashMap<String, String>>) {
    if let Some(map) = map {
        for (key, value) in map {
            println!("SET: {}={}", key, value);
        }
    }
}

fn print_add(map: &Option<HashMap<String, String>>, append: bool) {
    if let Some(map) = map {
        for (key, value) in map {
            print_add_value(key, value, append);
        }
    }
}

/// Append or prepend a value to an environment variable
fn print_add_value(key: &str, value: &str, append: bool) {
    let sep = if cfg!(windows) { ';' } else { ':' };

    match std::env::var(key) {
        Ok(current) => {
            let new_value = if append {
                format!("{current}{sep}{value}")
            } else {
                format!("{value}{sep}{current}")
            };
            println!("SET: {}={}", key, new_value);
        }
        Err(_) => println!("SET: {}={}", key, value),
    }
}

/// Send the final setup information
fn finalize_setup(name: &str, envs: &HashMap<String, Environment>) {
    println!("SET: USE_PROMPT={}", name);
    let title = envs[name].display.as_deref().unwrap_or(name);

    if get_update_title() {
        println!("TITLE: {}", title);
    }

    println!("{} setting up {}", "    Finished".success(), title.info());
}

/******************************************************************************
 * Pattern Matching
 *****************************************************************************/
pub fn create_pattern_config(key: &str, env: &Environment) -> HashMap<String, Environment> {
    let mut pattern_config = HashMap::new();
    let pattern = match &env.pattern {
        Some(p) => p,
        None => return pattern_config,
    };

    let path = PathBuf::from(&pattern.path);
    if !path.is_dir() {
        println!(
            "{}({}): {} is not a valid directory",
            "warning".warning(),
            key,
            pattern.path
        );
        return pattern_config;
    }

    let re = Regex::new(&pattern.regex).unwrap_or_else(|e| {
        println!("{}({}): {}", "error".error(), key, e);
        std::process::exit(1);
    });

    for entry in fs::read_dir(path)
        .expect("Could not read directory")
        .flatten()
    {
        if let Ok(name) = entry.file_name().into_string() {
            if let Some(captures) = re.captures(&name) {
                let mut new_env = env.clone();
                let mut new_key = key.to_string();

                for capture in captures.iter().skip(1).flatten() {
                    let value = capture.as_str();
                    new_env.replace_placeholders(value);
                    new_key = new_key.replacen("{}", value, 1);
                }

                new_env.version = captures.get(1).map(|m| m.as_str().to_string());
                new_env.original_key = Some(key.to_string());
                new_env.pattern = None;

                pattern_config.insert(new_key, new_env);
            }
        }
    }

    pattern_config
}
