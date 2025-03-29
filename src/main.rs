use clap::Parser;
use std::str;

mod colorize;
use colorize::Colorize;
mod environment;
use environment::*;

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
    /// List all environments
    #[clap(short, long)]
    list: bool,
    /// Create a new config file
    #[clap(short, long)]
    create: bool,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Parser)]
enum Command {
    /// Adjust use's settings
    Set {
        /// Change the terminal title based on the environment chosen
        #[clap(long = "update-title")]
        update_title: Option<bool>,
    },
}

fn main() {
    let config_file_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(CONFIG_FILE_NAME);
    let config_file = config_file_path
        .to_str()
        .expect("Could not convert path to string");

    let args = Args::parse();

    if args.create {
        create_config_file(config_file);
        return;
    }

    if let Some(Command::Set {
        update_title: Some(update),
    }) = args.command
    {
        set_update_title(update);
        return;
    }

    if !config_file_path.exists() {
        eprintln!("{} {} does not exist", "error:".error(), config_file);
        std::process::exit(1);
    }

    let environments = read_config_file(config_file).unwrap_or_else(|e| {
        eprintln!("{} reading {} file: {}", "error:".error(), config_file, e);
        std::process::exit(1);
    });

    if args.list || args.name.is_none() {
        list_environments(&environments)
            .iter()
            .for_each(|key| println!("{}", key));
    } else {
        use_environment(args.name.unwrap(), &environments);
    }
}
