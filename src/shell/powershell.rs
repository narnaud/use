use crate::shell::ShellPrinter;

pub struct PowershellPrinter {}

impl ShellPrinter for PowershellPrinter {
    fn echo(&self, message: &str) {
        println!("Write-Host '{}'", message);
    }

    fn set(&self, key: &str, value: &str) {
        println!("$env:{} = '{}'", key, value);
    }

    fn append(&self, key: &str, value: &str) {
        println!("$env:{} += \";{}\"", key, value);
    }

    fn prepend(&self, key: &str, value: &str) {
        println!("$env:{} = \"{};$env:{}\"", key, value, key);
    }

    fn prepend_path(&self, path: &str) {
        println!("$env:PATH = \"{};$env:PATH\"", path);
    }

    fn go(&self, path: &str) {
        println!("Set-Location {}", path);
    }

    fn change_title(&self, title: &str) {
        println!("$host.ui.RawUI.WindowTitle = '{}'", title);
    }

    fn env_variable(&self, env: &str) -> String {
        format!("$env:{}", env)
    }
}
