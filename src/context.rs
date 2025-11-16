use std::ffi::OsString;

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
    Powershell,
    #[clap(skip)]
    Unknown,
}

/// Context struct to hold the current operating system and shell
pub struct Context {
    pub os: OperatingSystem,
    pub shell: Shell,
    pub config_path: OsString,
}

impl Context {
    pub fn new() -> Self {
        Self {
            os: detect_os(),
            shell: detect_shell(),
            config_path: get_config_path(),
        }
    }

    pub fn check_old_config(&self) -> bool {
        let old_config_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".config")
            .join("use")
            .join("useconfig.yaml");
        let config_path = std::path::Path::new(&self.config_path);
        !config_path.exists() && old_config_path.exists()
    }

    pub fn check(&self, context: &str) -> bool {
        // split context string per ','
        let contexts: Vec<&str> = context.split(',').collect();
        for context in contexts {
            if (context == "windows" && self.os != OperatingSystem::Windows)
                || (context == "macos" && self.os != OperatingSystem::MacOS)
                || (context == "linux" && self.os != OperatingSystem::Linux)
            {
                return false;
            }

            if (context == "cmd" && self.shell != Shell::Cmd)
                || (context == "powershell" && self.shell != Shell::Powershell)
                || (context == "pwsh" && self.shell != Shell::Powershell)
            {
                return false;
            }
        }
        true
    }

    pub fn create_config_file(&self) -> Result<(), std::io::Error> {
        let config_path = std::path::Path::new(&self.config_path);
        if config_path.exists() {
            return Err(std::io::Error::other("Config file already exists"));
        }
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(config_path, CONFIG_YAML)?;
        Ok(())
    }

    pub fn edit_config_file(&self) -> Result<(), std::io::Error> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| default_editor());
        let status = std::process::Command::new(editor)
            .arg(&self.config_path)
            .status()?;
        if !status.success() {
            return Err(std::io::Error::other("Failed to open editor"));
        }
        Ok(())
    }
}

fn get_config_path() -> OsString {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config")
        .join("use.yaml")
        .into()
}

fn detect_os() -> OperatingSystem {
    if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOS
    } else {
        OperatingSystem::Unknown
    }
}

fn detect_shell() -> Shell {
    if let Ok(shell) = std::env::var("USE_SHELL") {
        if shell.contains("cmd") {
            return Shell::Cmd;
        } else if shell.contains("powershell") || shell.contains("pwsh") {
            return Shell::Powershell;
        }
    }
    Shell::Unknown
}

fn default_editor() -> String {
    if cfg!(target_os = "windows") {
        "notepad".to_string()
    } else {
        "vim".to_string()
    }
}

const CONFIG_YAML: &str = include_str!("config.yaml");
