use crate::colorize::Colorize;
use crate::shell::ShellPrinter;

pub struct DebugPrinter {}

impl ShellPrinter for DebugPrinter {
    fn start(&self, name: &str, env_name: &str) {
        println!("🗲 {}: {}", name.success(), env_name.info());
    }
    fn finish(&self) {
        println!("---");
    }
    fn finalize(&self, _name: &str, _env_name: &str) {
        // Do nothing
    }

    fn echo(&self, message: &str) {
        println!("{}", message);
    }

    fn run(&self, script: &str) {
        println!("{}\n{}\n{}", "```".info(), script, "```".info());
    }

    fn set(&self, key: &str, value: &str) {
        if key == "USE_PROMPT" {
            // Skip printing USE_PROMPT to avoid clutter
            return;
        }
        println!("- {} = {}", key, value);
    }

    fn append(&self, key: &str, value: &str) {
        println!("- {} += {}", key, value);
    }

    fn prepend(&self, key: &str, value: &str) {
        println!("- {} += {}", key, value);
    }

    fn prepend_path(&self, path: &str) {
        println!("- PATH += {}", path);
    }

    fn go(&self, path: &str) {
        println!("⇒ {}", path.info());
    }

    fn change_title(&self, _title: &str) {
        // Do nothing
    }

    fn env_variable(&self, env: &str) -> String {
        format!("${}", env.warning())
    }
}
