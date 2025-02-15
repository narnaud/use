use std::{io::{self, Write}, str};
use log::debug;

static CONFIG_FILE_NAME: &str = ".useconfig.json";
static CONFIG_FILE_EXAMPLE: &str = r#"
{
    "example": {
        "display": "Name of the configuration",
        "scripts": [
            "C:/example/path/to/script.bat",
            "C:/example/other/path/to/script.bat"
        ],
        "use": [
            "other",
            "configuration",
            "names"
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
            "C:/example/path/to/add/to/path",
            "C:/example/other/path/to/add/to/path"
        ],
        "go": "C:/example/path/to/go/to",
    },
    "msvc2022": {
        "display": "Microsoft Visual Studio 2022 - x64",
        "scripts": [
            "C:/Program Files/Microsoft Visual Studio/2022/Professional/VC/Auxiliary/Build/vcvars64.bat"
        ]
    },
    "qt6.8": {
        "display": "Qt 6.8.2 - MSVC - x64",
        "set": {
            "QTDIR": "C:/Qt/6.8.2/msvc2019_64/"
        },
        "append": {
            "CMAKE_PREFIX_PATH": "C:/Qt/6.8.2/msvc2019_64/"
        },
        "path": [
            "C:/Qt/6.8.2/msvc2019_64/bin"
        ]
    },
}
"#;

/// Function to get the current home directory
fn get_home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

/// Create a config file in the home directory if it does not exist
fn create_config_file_example(path: &str) {
    // Open the file and writhe the CONFIG_FILE_CONTENT to it
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    file.write_all(CONFIG_FILE_EXAMPLE.as_bytes()).expect("Failed to write to file");
}

/// Check the config file, and create one if the user wants to
fn check_config_file(path: std::path::PathBuf) -> bool {
    match path.exists() {
        true => true,
        false => {
            // Ask the user if they want to create the file
            print!("~/{} does not exist, do you want to create it? [y/n] ", CONFIG_FILE_NAME);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read input");

            // Check if the input is 'y'
            if input.trim() == "y" {
                create_config_file_example(path.to_str().unwrap());
                println!("~/{} created with some example content.", CONFIG_FILE_NAME);
            }
            false
        }
    }
}

fn main() {
    env_logger::init();

    let mut config_file = get_home_dir().expect("Could not find home directory");
    config_file.push(CONFIG_FILE_NAME);

    if !check_config_file(config_file.clone()) {
        std::process::exit(0);
    }
    debug!("Config file: {:?}", config_file.to_str().unwrap());
}
