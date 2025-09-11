use clap::Parser;
use std::str;

mod colorize;
mod config;
mod context;
mod init;
mod settings;
mod shell;
use colorize::Colorize;
use config::*;
use context::*;
use settings::*;
use shell::*;

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
    /// Prints the shell function used for shell integration
    Init {
        shell: Shell,
        #[clap(long)]
        print_full_init: bool,
    },
    /// Handles the configuration file
    Config {
        /// Create a new configuration file if it doesn't exist
        #[clap(long)]
        create: bool,
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
    /// Print the environment variables
    Print {
        /// Name of the environment to print
        name: String,
    },
}

fn main() {
    let context = Context::new();
    if context.os == OperatingSystem::Unknown {
        eprintln!("{}: Unsupported operating system", "error:".error());
        std::process::exit(1);
    }

    let mut args = Args::parse();

    // Default to `list` command if no arguments are provided
    if args.name.is_none() && args.command.is_none() {
        args.command = Some(Command::List);
    }

    if let Some(command) = args.command {
        match command {
            Command::Init {
                shell,
                print_full_init,
            } => handle_init(shell, print_full_init),
            Command::Config { create } => handle_config(&context, create),
            Command::List => handle_list(&context),
            Command::Set { key, value } => handle_set(key, value),
            Command::Print { name } => handle_use(&context, name, true),
        }
    } else if let Some(name) = args.name {
        handle_use(&context, name, false);
    }
}

fn handle_init(shell: Shell, print_full_init: bool) {
    let result = if print_full_init {
        init::init_main(shell)
    } else {
        init::init_stub(shell)
    };
    result.unwrap_or_else(|e| {
        eprintln!("{}: {}", "error:".error(), e);
        std::process::exit(1);
    });
}

fn handle_config(context: &Context, create: bool) {
    let result = if create {
        context.create_config_file().map(|_| {
            println!(
                "{} creating the default configuration file at {}",
                "     Finished".success(),
                context.config_path.display()
            );
        })
    } else {
        context.edit_config_file()
    };
    result.unwrap_or_else(|e| {
        eprintln!("{}: {}", "error:".error(), e);
        std::process::exit(1);
    });
}

fn handle_list(context: &Context) {
    let config = Config::new(context).unwrap_or_else(|e| {
        eprintln!("{}: {}", "error:".error(), e);
        std::process::exit(1);
    });

    config.list().iter().for_each(|env| {
        println!("{}", env);
    });
}

fn handle_set(key: Option<SettingsKey>, value: Option<String>) {
    if let (Some(key), Some(value)) = (key, value) {
        Settings::set(key, &value);
    } else {
        Settings::print();
    }
}

fn handle_use(context: &Context, name: String, printing: bool) {
    if !printing && context.shell == Shell::Unknown {
        eprintln!(
            "{}: Unknown shell, make sure to initialize use first (see documentation)",
            "error:".error()
        );
        std::process::exit(1);
    }

    let shell_printer = if printing {
        Box::new(DebugPrinter {}) as Box<dyn ShellPrinter>
    } else {
        create_shell_printer(context)
    };

    let config = Config::new(context).unwrap_or_else(|e| {
        let error = format!("{}: {}", "error:".error(), e);
        shell_printer.echo(&error);
        std::process::exit(1);
    });

    let settings = Settings::new();

    config
        .print_env(&name, &settings, shell_printer.as_ref())
        .unwrap_or_else(|e| {
            let warning = format!("{}: {}", "warning:".warning(), e);
            shell_printer.echo(&warning);
            std::process::exit(1);
        });
}

fn create_shell_printer(context: &Context) -> Box<dyn ShellPrinter> {
    match context.shell {
        Shell::Powershell => Box::new(PowershellPrinter {}),
        Shell::Cmd => Box::new(CmdPrinter {}),
        Shell::Unknown => panic!("Unsupported shell"),
    }
}
