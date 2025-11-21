use crate::colorize::Colorize;
use crate::shell::ShellPrinter;
use console::style;

pub struct DebugPrinter {}

impl ShellPrinter for DebugPrinter {
    fn start(&self, name: &str, env_name: &str) {
        println!(
            "{} {}",
            style(name).on_green().black(),
            style(env_name).blue()
        );
    }
    fn finish(&self) {
        println!();
    }
    fn finalize(&self, _name: &str, _env_name: &str) {
        // Do nothing
    }

    fn echo(&self, message: &str) {
        println!("{}", message);
    }

    fn run(&self, script: &str) {
        // Split script into lines for better formatting
        // then add | at the beginning of each line
        let formatted_script = script
            .lines()
            .map(|line| format!("{} {} {}", style('│').green(), style('┃').blue(), line))
            .collect::<Vec<_>>()
            .join("\n");
        println!("{}", formatted_script);
    }

    fn set(&self, key: &str, value: &str) {
        if key == "USE_PROMPT" {
            // Skip printing USE_PROMPT to avoid clutter
            return;
        }
        println!("{} {} = {}", style('│').green(), key, value);
    }

    fn append(&self, key: &str, value: &str) {
        println!("{} {} += {}", style('│').green(), key, value);
    }

    fn prepend(&self, key: &str, value: &str) {
        println!("{} {} += {}", style('│').green(), key, value);
    }

    fn prepend_path(&self, path: &str) {
        println!("{} PATH += {}", style('│').green(), path);
    }

    fn go(&self, path: &str) {
        println!("{} {}", style("└→").green(), path);
    }

    fn change_title(&self, _title: &str) {
        // Do nothing
    }

    fn env_variable(&self, env: &str) -> String {
        format!("${}", env.warning())
    }
}
