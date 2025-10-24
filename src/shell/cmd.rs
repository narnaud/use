use crate::shell::ShellPrinter;

pub struct CmdPrinter {}

impl ShellPrinter for CmdPrinter {
    fn echo(&self, message: &str) {
        println!("echo {}", message);
    }

    fn set(&self, key: &str, value: &str) {
        println!("@set {}={}", key, value);
    }

    fn append(&self, key: &str, value: &str) {
        println!("@set {}=%{}%;{}", key, key, value);
    }

    fn prepend(&self, key: &str, value: &str) {
        println!("@set {}={};%{}%", key, value, key);
    }

    fn prepend_path(&self, path: &str) {
        println!("@set PATH={};%PATH%", path);
    }

    fn go(&self, path: &str) {
        // Use chdir, as may be aliased to something else
        println!("chdir /D {}", path);
    }

    fn change_title(&self, title: &str) {
        println!("TITLE {}", title);
    }

    fn env_variable(&self, env: &str) -> String {
        format!("%{}%", env)
    }
}
