use crate::colorize::Colorize;
use std::str;

pub trait ShellPrinter {
    fn start(&self, _name: &str, env_name: &str) {
        let text = format!("{} {}", " Configuring".success(), env_name);
        self.echo(&text);
    }
    fn finish(&self) {
        // Do nothing by default
    }

    fn finalize(&self, _name: &str, env_name: &str) {
        let text = format!(
            "{} setting up {}",
            "    Finished".success(),
            env_name.info()
        );
        self.echo(&text);
    }

    fn run(&self, script: &str) {
        println!("{}", script);
    }

    fn echo(&self, message: &str);
    fn set(&self, key: &str, value: &str);
    fn append(&self, key: &str, value: &str);
    fn prepend(&self, key: &str, value: &str);
    fn prepend_path(&self, path: &str);
    fn go(&self, path: &str);

    fn change_title(&self, title: &str);

    /// Return the shell-specific syntax for referencing an environment variable name.
    /// Example: "PATH" -> "%PATH%" for cmd, "$env:PATH" for PowerShell
    fn env_variable(&self, env: &str) -> String;
}

mod cmd;
mod debug;
mod powershell;
pub use cmd::CmdPrinter;
pub use debug::DebugPrinter;
pub use powershell::PowershellPrinter;
