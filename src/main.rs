use clap::Parser;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::str;

static CONFIG_FILE_NAME: &str = ".useconfig.json";
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
        "go": "C:\\example\\path\\to\\go\\to",
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
    },
}
"#;

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Environment {
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the environment to use
    env_name: Option<String>,
    /// List all environments
    #[clap(short, long)]
    list: bool,
    /// Create a new config file
    #[clap(short, long)]
    create: bool,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let mut config_file = dirs::home_dir().expect("Could not find home directory");
    config_file.push(CONFIG_FILE_NAME);

    if !config_file.exists() {
        print!("Error ~/{} does not exist", CONFIG_FILE_NAME);
        std::process::exit(1);
    }
    debug!("Find config file: {:?}", config_file.to_str().unwrap());

    let environments = match read_config_file(config_file.to_str().unwrap()) {
        Ok(envrionments) => envrionments,
        Err(e) => {
            println!("Error reading ~/{} file: {}", CONFIG_FILE_NAME, e);
            std::process::exit(1);
        }
    };
    debug!("Read config file");

    if args.create {
        create_config_file(config_file.to_str().unwrap());
        println!("Created ~/{} file", CONFIG_FILE_NAME);
        std::process::exit(0);
    }
    if args.list {
        list_environments(environments);
        std::process::exit(0);
    }

    let env_name = match args.env_name {
        Some(env_name) => match environments.get(&env_name) {
            Some(_) => env_name,
            None => {
                println!("Error: Environment {} not found", env_name);
                std::process::exit(1);
            }
        },
        None => {
            list_environments(environments);
            std::process::exit(0);
        }
    };
    debug!("Use environment: {}", env_name);

    let use_envs = get_use_environments(env_name.as_str(), &environments);
    let env = merge_environments(use_envs);
    print_environment(&env);
}

/// Create a config file in the home directory if it does not exist
fn create_config_file(path: &str) {
    // Open the file and writhe the CONFIG_FILE_CONTENT to it
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    file.write_all(CONFIG_FILE_EXAMPLE.as_bytes()).expect("Failed to write to file");
}

/// Read the congig file and return a map of environments
fn read_config_file(file_path: &str) -> Result<HashMap<String, Environment>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Function to list all environments in the config file
fn list_environments(configs: HashMap<String, Environment>) {
    // Get keys from configs map, sort then and print them
    let mut keys: Vec<_> = configs.keys().collect();
    keys.sort();
    keys.iter().for_each(|key| println!("{}", key));
}

/// List all environment that should be used based on the environment name
fn get_use_environments(
    env_name: &str,
    envs: &HashMap<String, Environment>,
) -> Vec<Environment> {
    let mut use_envs: Vec<Environment> = Vec::new();
    let env = envs.get(env_name).unwrap();
    use_envs.push(env.clone());

    if let Some(reuse) = env.reuse.as_ref() {
        for env_name in reuse.iter() {
            let reuse_envs = get_use_environments(env_name, envs);
            // Add the environment to the list of environments to use
            // Only if it is not already in the list
            for reuse_env in reuse_envs.iter() {
                if !use_envs.contains(reuse_env) {
                    use_envs.push(reuse_env.clone());
                }
            }
        }
    }

    use_envs.reverse();
    use_envs
}

/// Merge all environments into one environment
fn merge_environments(envs: Vec<Environment>) -> Environment {
    let mut result_env = envs[0].clone();

    // Merge all environments into one
    for env in envs.iter().skip(1) {
        result_env.display = env.display.clone().or(result_env.display);
        result_env.defer.get_or_insert_with(Vec::new).extend(env.defer.clone().unwrap_or_default());
        result_env.set.get_or_insert_with(HashMap::new).extend(env.set.clone().unwrap_or_default());
        result_env.append.get_or_insert_with(HashMap::new).extend(env.append.clone().unwrap_or_default());
        result_env.prepend.get_or_insert_with(HashMap::new).extend(env.prepend.clone().unwrap_or_default());
        result_env.path.get_or_insert_with(Vec::new).extend(env.path.clone().unwrap_or_default());
        result_env.go = env.go.clone().or(result_env.go);
    }

    result_env
}

/// Print the environment to the console
fn print_environment(env: &Environment) {
    let print_map = |label: &str, map: &Option<HashMap<String, String>>| {
        if let Some(map) = map {
            for (key, value) in map {
                println!("{}: {} = {}", label, key, value);
            }
        }
    };

    let print_vec = |label: &str, vec: &Option<Vec<String>>| {
        if let Some(vec) = vec {
            for item in vec {
                println!("{}: {}", label, item);
            }
        }
    };

    if let Some(display) = &env.display {
        println!("DISPLAY: {}", display);
    }
    print_vec("DEFER", &env.defer);
    print_map("SET", &env.set);
    print_map("APPEND", &env.append);
    print_map("PREPEND", &env.prepend);
    print_vec("PATH", &env.path);
    if let Some(go) = &env.go {
        println!("GO: {}", go);
    }
}
