use crate::shell::ShellPrinter;

pub struct PowershellPrinter {}

impl ShellPrinter for PowershellPrinter {
    fn echo(&self, message: &str) -> String {
        format!("Write-Host '{}'", message)
    }

    fn set(&self, key: &str, value: &str) -> String {
        format!("$env:{} = '{}'", key, value)
    }

    fn append(&self, key: &str, value: &str) -> String {
        format!("$env:{} += \";{}\"", key, value)
    }

    fn prepend(&self, key: &str, value: &str) -> String {
        format!("$env:{} = \"{};$env:{}\"", key, value, key)
    }

    fn prepend_path(&self, path: &str) -> String {
        format!("$env:PATH = \"{};$env:PATH\"", path)
    }

    fn go(&self, path: &str) -> String {
        format!("Set-Location {}", path)
    }

    fn change_title(&self, title: &str) -> String {
        format!("$host.ui.RawUI.WindowTitle = '{}'", title)
    }

    fn env_variable(&self, env: &str) -> String {
        format!("$env:{}", env)
    }
}
