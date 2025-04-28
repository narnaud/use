use std::path::PathBuf;
use std::{env, io};
use which::which;

use crate::context::Shell;
use crate::colorize::Colorize;

// The workflow and part of the code is copied from starship:
// https://starship.rs
// We use a two-phase init:
// - first phase is a simple command to the shell
// - second phase is a script that is executed by the shell

struct UsePath {
    native_path: PathBuf,
}
impl UsePath {
    fn init() -> io::Result<Self> {
        let exe_name = option_env!("CARGO_PKG_NAME").unwrap_or("use");
        let native_path = which(exe_name).or_else(|_| env::current_exe())?;
        Ok(Self { native_path })
    }
    fn str_path(&self) -> io::Result<&str> {
        let current_exe = self
            .native_path
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "can't convert to str"))?;
        Ok(current_exe)
    }
    /// `PowerShell` specific path escaping
    fn sprint_pwsh(&self) -> io::Result<String> {
        self.str_path()
            .map(|s| s.replace('\'', "''"))
            .map(|s| format!("'{s}'"))
    }
    /// Command Shell specific path escaping
    fn sprint_cmdexe(&self) -> io::Result<String> {
        self.str_path().map(|s| format!("\"{s}\""))
    }
}

/// Print the init stub for the given shell
pub fn init_stub(shell: Shell) -> io::Result<()> {
    let use_path = UsePath::init()?;

    match shell {
        Shell::PowerShell => print!(
                r#"Invoke-Expression (& {} init powershell --print-full-init | Out-String)"#,
                use_path.sprint_pwsh()?
            ),
        Shell::Cmd =>
            print_script(CLINK_INIT, &use_path.sprint_cmdexe()?),
        _ => {
            eprintln!("{} Unsupported shell: {shell:?}", "error:".error());
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unsupported shell",
            ));
        }
    }
    Ok(())
}

/// Print the main script for the given shell
/// This function is used when passing `--print-full-init` to the init command
pub fn init_main(shell: Shell) -> io::Result<()> {
    let use_path = UsePath::init()?;

    match shell {
        Shell::PowerShell =>
            print_script(POWERSHELL_INIT, &use_path.sprint_pwsh()?),
        Shell::Cmd =>
            print_script(CLINK_INIT, &use_path.sprint_cmdexe()?),
        _ => {
            eprintln!("{} Unsupported shell: {shell:?}", "error:".error());
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unsupported shell",
            ));
        }
    }
    Ok(())
}

/// Print the given script after replacing the placeholder with the given path
fn print_script(script: &str, path: &str) {
    let script = script.replace("::USE::", path);
    print!("{script}");
}

const POWERSHELL_INIT: &str = include_str!("use.ps1");
const CLINK_INIT: &str = include_str!("use.lua");
