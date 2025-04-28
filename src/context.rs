#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum Shell {
    #[clap(name = "cmd", alias = "clink")]
    Cmd,
    #[clap(name = "powershell", alias = "pwsh")]
    PowerShell,
    #[clap(skip)]
    Unknown,
}

/// Context struct to hold the current operating system and shell
#[derive(Debug, Clone)]
pub struct Context {
    pub os: OperatingSystem,
    pub shell: Shell,
}

impl Context {
    pub fn new() -> Self {
        Self {
            os: detect_os(),
            shell: detect_shell(),
        }
    }
}

fn detect_os() -> OperatingSystem {
    #[cfg(target_os = "windows")]
    return OperatingSystem::Windows;

    #[cfg(target_os = "macos")]
    return OperatingSystem::MacOS;

    #[cfg(target_os = "linux")]
    return OperatingSystem::Linux;

    OperatingSystem::Unknown
}

fn detect_shell() -> Shell {
    if let Ok(shell) = std::env::var("USE_SHELL") {
        if shell.contains("cmd") {
            return Shell::Cmd;
        } else if shell.contains("powershell") || shell.contains("pwsh") {
            return Shell::PowerShell;
        }
    }
    Shell::Unknown
}
