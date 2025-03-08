use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use crate::colorize;
use colorize::Colorize;

static CONFIG_FILE_EXAMPLE: &str = r#"
{
    "example": {
        "display": "Name of the configuration",
        "use": [
            "other",
            "configuration",
            "names"
        ],
        "defer": [
            "C:\\example\\path\\to\\script.bat",
            "C:\\example\\other\\path\\to\\script.bat"
        ],
        "set": {
            "EXAMPLE_VAR": "example value"
        },
        "append": {
            "EXAMPLE_VAR_APPEND": "value appended to EXAMPLE_VAR_APPEND"
        },
        "prepend": {
            "EXAMPLE_VAR_PREPEND": "value prepended to EXAMPLE_VAR_PREPEND"
        },
        "path": [
            "C:\\example\\path\\to\\add\\to\\path",
            "C:\\example\\other\\path\\to\\add\\to\\path"
        ],
        "go": "C:\\example\\path\\to\\go\\to"
    },
    "msvc2022": {
        "display": "Microsoft Visual Studio 2022 - x64",
        "defer": [
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Auxiliary\\Build\\vcvars64.bat"
        ]
    },
    "qt{}.{}.{}": {
        "display": "Qt {}.{}.{} - MSVC - x64",
        "pattern": {
            "path": "C:\\Qt",
            "regex": "(\\d+)\\.(\\d+)\\.(\\d+)"
        },
        "use": [
            "msvc2022"
        ],
        "set": {
            "QTDIR": "C:\\Qt\\{}.{}.{}\\msvc2019_64\\"
        },
        "append": {
            "CMAKE_PREFIX_PATH": "C:\\Qt\\{}.{}.{}\\msvc2019_64\\"
        },
        "path": [
            "C:\\Qt\\{}.{}.{}\\msvc2019_64\\bin"
        ]
    },
}
"#;

/// Struct to hold the pattern of the environment
///
/// This is used to create multiple environments based on a pattern
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Pattern {
    path: String,
    regex: String,
}

/// Struct to hold the environment configuration
///
/// This matches the structure of the JSON config file
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
}

/// Create a config file in the home directory if it does not exist
pub fn create_config_file(path: &str) {
    println!("{} {} file", "    Creating".info(), path);
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    file.write_all(CONFIG_FILE_EXAMPLE.as_bytes())
        .expect("Failed to write to file");
    println!("{} {} file", "     Created".success().update(), path);
}

/// Read the config file and return a map of environments
pub fn read_config_file(
    file_path: &str,
) -> Result<HashMap<String, Environment>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut config: HashMap<String, Environment> = serde_json::from_reader(reader)?;

    // Create environments based on patterns
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

/// List all environments in the config file
pub fn list_environments(envs: &HashMap<String, Environment>) {
    // Get keys from configs map, sort then and print them
    let mut keys: Vec<_> = envs.keys().collect();
    keys.sort();
    keys.iter().for_each(|key| println!("{}", key));
}

/// Use the environment by printing the configuration to the console
///
/// This function will recursively call itself to print all environments that should be used
/// based on the environment name
pub fn use_environment(name: String, envs: &HashMap<String, Environment>) {
    let current = name.clone();
    let names = list_all_envs_for(name, envs);
    for name in names.iter().rev() {
        let env = envs.get(name).unwrap();
        print_environment(env);
    }

    finalize_setup(&current, envs);
}

/// List all environment that should be used based on the environment name
fn list_all_envs_for(name: String, envs: &HashMap<String, Environment>) -> Vec<String> {
    if !envs.contains_key(name.as_str()) {
        println!(
            "{} Environment {} not found",
            "warning:".warning(),
            name.info()
        );
        std::process::exit(1);
    }

    let mut names = vec![name.clone()];
    let env = envs.get(name.as_str()).unwrap();

    if let Some(reuse) = env.reuse.as_ref() {
        for env_name in reuse.iter() {
            let reuse_env_names = list_all_envs_for(env_name.clone(), envs)
                .into_iter()
                .filter(|name| !names.contains(name))
                .collect::<Vec<String>>();
            names.extend(reuse_env_names);
        }
    }

    names
}

/// Print the environment to the console
fn print_environment(env: &Environment) {
    let print_vec = |label: &str, vec: &Option<Vec<String>>| {
        if let Some(vec) = vec {
            for item in vec {
                println!("{}: {}", label, item);
            }
        }
    };

    let print_set = |map: &Option<HashMap<String, String>>| {
        if let Some(map) = map {
            for (key, value) in map {
                println!("SET: {}={}", key, value);
            }
        }
    };

    let print_add = |map: &Option<HashMap<String, String>>, append: bool| {
        if let Some(map) = map {
            for (key, value) in map {
                print_add_value(key, value, append);
            }
        }
    };

    if let Some(display) = &env.display {
        println!("{} {}", " Configuring".info(), display);
    }
    print_vec("DEFER", &env.defer);
    print_set(&env.set);
    print_add(&env.append, true);
    print_add(&env.prepend, false);
    print_vec("PATH", &env.path);
    if let Some(go) = &env.go {
        println!("GO: {}", go);
    }
    if let Some(display) = &env.display {
        println!("{} {}", "  Configured".success().update(), display);
    }
}

/// Append or prepend a value to an environment variable
fn print_add_value(key: &String, value: &String, append: bool) {
    match std::env::var(key) {
        // If the variable exists append the value to it using ; on windows, and : on linux
        Ok(current) => {
            let sep = if cfg!(windows) { ';' } else { ':' };
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

/// Send the final information, mostly for updating the terminal title and prompt
fn finalize_setup(name: &str, envs: &HashMap<String, Environment>) {
    println!("SET: USE_PROMPT={}", name);
    let title = (envs.get(name).unwrap().display).as_deref().unwrap_or(name);
    println!("TITLE: {}", title);
    println!("{} setting up {}", "    Finished".success(), title.info());
}

/// If the environment has a pattern, create a new environment for each matches
pub fn create_pattern_config(key: &String, env: &Environment) -> HashMap<String, Environment> {
    let mut pattern_config = HashMap::new();
    let pattern = &env.pattern.as_ref().unwrap();

    let path = PathBuf::from(&pattern.path);
    if !path.exists() || !path.is_dir() {
        println!(
            "{}({}): {} is not a valid directory",
            "warning".warning(),
            key,
            pattern.path
        );
        return pattern_config;
    }

    // get all files or dirs in the pattern path
    let entries = path
        .read_dir()
        .expect("Could not read directory")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .map(|entry| entry.to_string());

    let re = match Regex::new(&pattern.regex) {
        Ok(re) => re,
        Err(e) => {
            println!("{}({}): {}", "error".error(), key, e);
            std::process::exit(1);
        }
    };
    for entry in entries.filter(|e| re.is_match(e)) {
        let captures = re.captures(&entry).unwrap();
        let mut new_env = env.clone();
        let mut new_key = key.clone();
        for capture in captures.iter().skip(1).flatten() {
            let capture = capture.as_str();
            replace_in_env(&mut new_env, capture);
            new_key = new_key.replacen("{}", capture, 1);
        }
        new_env.pattern = None;
        pattern_config.insert(new_key, new_env);
    }

    pattern_config
}

/// Replace a {} in the environment with a value
fn replace_in_env(env: &mut Environment, value: &str) {
    let replace_in_string =
        |string: &Option<String>| string.as_ref().map(|s| s.replacen("{}", value, 1));
    let replace_in_vec = |vec: &Option<Vec<String>>| {
        vec.as_ref()
            .map(|v| v.iter().map(|item| item.replacen("{}", value, 1)).collect())
    };

    let replace_in_map = |map: &Option<HashMap<String, String>>| {
        map.as_ref().map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.replacen("{}", value, 1)))
                .collect()
        })
    };

    env.display = replace_in_string(&env.display);
    env.go = replace_in_string(&env.go);
    env.defer = replace_in_vec(&env.defer);
    env.set = replace_in_map(&env.set);
    env.append = replace_in_map(&env.append);
    env.prepend = replace_in_map(&env.prepend);
    env.path = replace_in_vec(&env.path);
}
