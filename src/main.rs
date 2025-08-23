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
    let context = Context::new();
    if context.os == OperatingSystem::Unknown {
        eprintln!("{}: Unsupported operating system", "error:".error());
        std::process::exit(1);
    }

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
            let config = Config::new(&context).unwrap_or_else(|e| {
                eprintln!("{}: {}", "error:".error(), e);
                std::process::exit(1);
            });
            config.list().iter().for_each(|env| {
                println!("{}", env);
            });
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

    let context = Context::new();
    if context.os == OperatingSystem::Unknown {
        eprintln!("{}: Unsupported operating system", "error:".error());
        std::process::exit(1);
    }
    if context.shell == Shell::Unknown {
        eprintln!(
            "{}: Unknown shell, make sure to initialize use first (see documentation)",
            "error:".error()
        );
        std::process::exit(1);
    }

    let settings = Settings::new();
    let shell_printer = create_shell_printer(&context);
    let config = Config::new(&context).unwrap_or_else(|e| {
        show_error(&e, shell_printer.as_ref());
        std::process::exit(1);
    });

    let env_name = args.name.expect("can't unwrap environment name");
    config
        .use_env(&env_name, &settings, shell_printer.as_ref())
        .unwrap_or_else(|e| {
            show_error(&e, shell_printer.as_ref());
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

fn show_error(message: &str, shell_printer: &dyn ShellPrinter) {
    let error = format!("{}: {}", "error:".error(), message);
    println!("{}", shell_printer.echo(&error));
}
