use crate::shell::ShellPrinter;

pub struct CmdPrinter {}

impl ShellPrinter for CmdPrinter {
    fn echo(&self, message: &str) -> String {
        format!("echo {}", message)
    }

    fn set(&self, key: &str, value: &str) -> String {
        format!("@set {}={}", key, value)
    }

    fn append(&self, key: &str, value: &str) -> String {
        format!("@set {}=%{}%;{}", key, key, value)
    }

    fn prepend(&self, key: &str, value: &str) -> String {
        format!("@set {}={};%{}%", key, value, key)
    }

    fn prepend_path(&self, path: &str) -> String {
        format!("@set PATH={};%PATH%", path)
    }

    fn go(&self, path: &str) -> String {
        format!("cd {}", path)
    }

    fn change_title(&self, title: &str) -> String {
        format!("TITLE {}", title)
    }

    // fn env_variable(&self, env: &str) -> String {
    //     format!("%{}%", env)
    // }
}
