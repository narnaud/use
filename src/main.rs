use clap::Parser;
use std::str;

mod colorize;
use colorize::Colorize;
mod environment;
use environment::*;
mod settings;
use settings::*;
mod context;
use context::*;
mod init;

static CONFIG_FILE_NAME: &str = ".useconfig.yaml";

#[derive(Parser)]
#[command(
    bin_name = "use",
    version,
    about = "Command-line utility to setup environment"
)]
struct Args {
    /// Name of the environment to use
    name: Option<String>,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Parser)]
enum Command {
    ///  Prints the shell function used for shell integration
    Init {
        shell: Shell,
        #[clap(long)]
        print_full_init: bool,
    },
    /// List all environments
    List,
    /// Adjust use's settings
    Set {
        /// Configuration key to edit
        #[clap(requires = "value")]
        key: Option<SettingsKey>,
        /// Value to place into that key
        value: Option<String>,
    },
}

fn main() {
    let config_file_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(CONFIG_FILE_NAME);
    let config_file = config_file_path
        .to_str()
        .expect("Could not convert path to string");

    let mut args = Args::parse();

    if args.name.is_none() && args.command.is_none() {
        args.command = Some(Command::List);
    }

    match args.command {
        Some(Command::Init {
            shell,
            print_full_init,
        }) => {
            if print_full_init {
                init::init_main(shell).expect("can't init_main");
            } else {
                init::init_stub(shell).expect("can't init_stub");
            }
            return;
        }
        Some(Command::List) => {
            list_environments(&read_config_file(config_file).unwrap_or_else(|e| {
                eprintln!("{} reading {} file: {}", "error:".error(), config_file, e);
                std::process::exit(1);
            }))
            .iter()
            .for_each(|key| println!("{}", key));
            return;
        }
        Some(Command::Set { key, value }) => {
            if let Some(key) = key {
                if let Some(value) = value {
                    Settings::set(key, &value);
                }
            } else {
                Settings::print();
            }
            return;
        }
        None => {}
    }

    if !config_file_path.exists() {
        eprintln!("{} {} does not exist", "error:".error(), config_file);
        std::process::exit(1);
    }

    let environments = read_config_file(config_file).unwrap_or_else(|e| {
        eprintln!("{} reading {} file: {}", "error:".error(), config_file, e);
        std::process::exit(1);
    });

    use_environment(
        args.name.unwrap(),
        &environments,
        Settings::default().update_title,
    );
}
