use clap::Parser;
use std::str;

mod colorize;
use colorize::Colorize;
mod environment;
use environment::*;

static CONFIG_FILE_NAME: &str = ".useconfig.json";

#[derive(Parser, Debug)]
#[command(bin_name = "use", version, about="Command-line utility to setup environment", long_about = None)]
struct Args {
    /// Name of the environment to use
    name: Option<String>,
    /// List all environments
    #[clap(short, long)]
    list: bool,
    /// Create a new config file
    #[clap(short, long)]
    create: bool,
}

fn main() {
    let mut config_file_path = dirs::home_dir().expect("Could not find home directory");
    config_file_path.push(CONFIG_FILE_NAME);
    let config_file = config_file_path
        .to_str()
        .expect("Could not convert path to string");

    let args = Args::parse();
    if args.create {
        create_config_file(config_file);
        std::process::exit(0);
    }

    if !config_file_path.exists() {
        print!("{} {} does not exist", "error:".error(), config_file);
        std::process::exit(1);
    }

    let environments = read_config_file(config_file).unwrap_or_else(|e| {
        println!("{} reading {} file: {}", "error:".error(), config_file, e);
        std::process::exit(1);
    });

    if args.list || args.name.is_none() {
        list_environments(&environments);
        std::process::exit(0);
    }

    let name = args.name.unwrap();
    use_environment(name, &environments);
}
