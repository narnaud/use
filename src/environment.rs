use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

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
    "qt6.8": {
        "display": "Qt 6.8.2 - MSVC - x64",
        "use": [
            "msvc2022"
        ],
        "set": {
            "QTDIR": "C:\\Qt\\6.8.2\\msvc2019_64\\"
        },
        "append": {
            "CMAKE_PREFIX_PATH": "C:\\Qt\\6.8.2\\msvc2019_64\\"
        },
        "path": [
            "C:\\Qt\\6.8.2\\msvc2019_64\\bin"
        ]
    }
}
"#;

/// Struct to hold the environment configuration
///
/// This matches the structure of the JSON config file
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Environment {
    display: Option<String>,
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
    let config = serde_json::from_reader(reader)?;
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
    let title = (envs.get(name).unwrap().display)
        .as_deref()
        .unwrap_or(name);
    println!("TITLE: {}", title);
    println!("{} setting up {}", "    Finished".success(), title.info());
}
