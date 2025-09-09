use std::str;

pub trait ShellPrinter {
    fn echo(&self, message: &str) -> String;
    fn run(&self, script: &str) -> String {
        script.to_string()
    }
    fn set(&self, key: &str, value: &str) -> String;
    fn append(&self, key: &str, value: &str) -> String;
    fn prepend(&self, key: &str, value: &str) -> String;
    fn prepend_path(&self, path: &str) -> String;
    fn go(&self, path: &str) -> String;

    fn change_title(&self, title: &str) -> String;

    /// Return the shell-specific syntax for referencing an environment variable name.
    /// Example: "PATH" -> "%PATH%" for cmd, "$env:PATH" for PowerShell
    fn env_variable(&self, env: &str) -> String;
}

mod cmd;
mod powershell;
pub use cmd::CmdPrinter;
pub use powershell::PowershellPrinter;
