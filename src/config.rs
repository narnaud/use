use regex::Regex;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::colorize::Colorize;
use crate::context::Context;
use crate::settings::Settings;
use crate::shell::ShellPrinter;
use crate::Shell;

/// Struct to hold the pattern of the environment
#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Pattern {
    path: String,
    regex: String,
}

/// Common struct for global and shell-specific configuration
#[derive(Debug, Deserialize, Clone, PartialEq)]
struct CommonProperties {
    display: Option<String>,
    script: Option<String>,
    set: Option<HashMap<String, String>>,
    append: Option<HashMap<String, String>>,
    prepend: Option<HashMap<String, String>>,
    path: Option<Vec<String>>,
    #[serde(rename = "use")]
    reuse: Option<Vec<String>>,
    go: Option<String>,
}

/// Struct to hold the environment configuration
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Environment {
    // The name is read from the file at a different level, see method read_config_file
    #[serde(skip)]
    name: String,

    #[serde(rename = "for")]
    context: Option<String>,
    pattern: Option<Pattern>,

    #[serde(flatten)]
    global: CommonProperties,
    for_cmd: Option<CommonProperties>,
    #[serde(alias = "for_pwsh")]
    for_powershell: Option<CommonProperties>,

    // Internal properties
    #[serde(skip)]
    version: Option<String>,
    #[serde(skip)]
    original_name: Option<String>,
}

impl Environment {
    /// Replace ${VAR} placeholders with shell-specific environment variable syntax
    fn substitute_env_vars(value: &str, printer: &dyn ShellPrinter) -> String {
        // Match ${VAR_NAME}
        let re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        re.replace_all(value, |caps: &regex::Captures| {
            let var_name = &caps[1];
            printer.env_variable(var_name)
        })
        .to_string()
    }

    /// Replace placeholders in the environment configuration when using a pattern
    fn replace_placeholders(&mut self, value: &str) {
        let replace = |s: &mut Option<String>| {
            *s = s.as_ref().map(|v| v.replacen("{}", value, 1));
        };
        let replace_vec = |v: &mut Option<Vec<String>>| {
            *v = v
                .as_ref()
                .map(|items| items.iter().map(|i| i.replacen("{}", value, 1)).collect());
        };
        let replace_map = |m: &mut Option<HashMap<String, String>>| {
            *m = m.as_ref().map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.replacen("{}", value, 1)))
                    .collect()
            });
        };

        self.name = self.name.replacen("{}", value, 1);
        replace(&mut self.global.display);
        replace(&mut self.global.go);
        replace(&mut self.global.script);
        replace_map(&mut self.global.set);
        replace_map(&mut self.global.append);
        replace_map(&mut self.global.prepend);
        replace_vec(&mut self.global.path);
    }

    /// Fold the shell-specific properties into the global properties
    fn fold(&mut self, context: &Context) {
        let shell_env = match context.shell {
            Shell::Cmd => self.for_cmd.take(),
            Shell::Powershell => self.for_powershell.take(),
            Shell::Unknown => panic!("Unsupported shell"),
        };

        let Some(shell_env) = shell_env else {
            return;
        };

        self.global.display = shell_env.display.clone().or(self.global.display.take());
        self.global.script = shell_env.script.clone().or(self.global.script.take());
        self.global.go = shell_env.go.clone().or(self.global.go.take());

        for (field, other_field) in [
            (&mut self.global.set, &shell_env.set),
            (&mut self.global.append, &shell_env.append),
            (&mut self.global.prepend, &shell_env.prepend),
        ] {
            if let Some(other_map) = other_field {
                if let Some(map) = field {
                    map.extend(other_map.clone());
                } else {
                    *field = Some(other_map.clone());
                }
            }
        }

        for (field, other_field) in [
            (&mut self.global.path, &shell_env.path),
            (&mut self.global.reuse, &shell_env.reuse),
        ] {
            if let Some(other_vec) = other_field {
                if let Some(vec) = field {
                    vec.extend(other_vec.clone());
                } else {
                    *field = Some(other_vec.clone());
                }
            }
        }
    }

    fn env_name(&self) -> &str {
        if let Some(display) = &self.global.display {
            display
        } else {
            &self.name
        }
    }

    /// Print the environment using the provided ShellPrinter
    pub fn print(&self, printer: &dyn ShellPrinter) {
        let text = format!("{} {}", " Configuring".success(), self.env_name());
        println!("{}", printer.echo(&text));

        if let Some(set) = &self.global.set {
            for (key, value) in set {
                let v = Self::substitute_env_vars(value, printer);
                println!("{}", printer.set(key, &v));
            }
        }

        if let Some(append) = &self.global.append {
            for (key, value) in append {
                let v = Self::substitute_env_vars(value, printer);
                println!("{}", printer.append(key, &v));
            }
        }

        if let Some(prepend) = &self.global.prepend {
            for (key, value) in prepend {
                let v = Self::substitute_env_vars(value, printer);
                println!("{}", printer.prepend(key, &v));
            }
        }

        if let Some(paths) = &self.global.path {
            for path in paths {
                let p = Self::substitute_env_vars(path, printer);
                println!("{}", printer.prepend_path(&p));
            }
        }

        if let Some(script) = &self.global.script {
            let s = Self::substitute_env_vars(script, printer);
            println!("{}", printer.run(&s));
        }

        if let Some(go) = &self.global.go {
            let g = Self::substitute_env_vars(go, printer);
            println!("{}", printer.go(&g));
        }

        // Set the USE_PROMPT environment variable
        println!("{}", printer.set("USE_PROMPT", self.name.as_str()));
    }

    /// Display the environment
    pub fn display(&self) {
        println!(
            "🗲 {}: {}",
            self.name.as_str().success(),
            self.global.display.as_deref().unwrap_or("").info()
        );
        if let Some(set) = &self.global.set {
            for (key, value) in set {
                println!("{} = {}", key, value);
            }
        }
        if let Some(append) = &self.global.append {
            for (key, value) in append {
                println!("{} += {}", key, value);
            }
        }
        if let Some(prepend) = &self.global.prepend {
            for (key, value) in prepend {
                println!("{} += {}", key, value);
            }
        }
        if let Some(paths) = &self.global.path {
            for path in paths {
                println!("PATH += {}", path);
            }
        }
        if let Some(script) = &self.global.script {
            println!("{}\n{}{}", "```".info(), script, "```".info());
        }
        if let Some(go) = &self.global.go {
            println!("-> {}", go.as_str().info());
        }
    }
}

/// Struct to hold the list of environments
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    environments: Vec<Environment>,
}

impl Config {
    /// Create a new Config struct
    pub fn new(context: &Context) -> Result<Self, String> {
        let path = Path::new(&context.config_path);
        if !path.exists() {
            return Err(format!("Config file not found at {}", path.display()));
        }
        let environments = read_config_file(path, context)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        Ok(Self { environments })
    }

    /// Get a list of all environment keys
    pub fn list(&self) -> Vec<String> {
        self.environments
            .iter()
            .map(|env| env.name.clone())
            .collect()
    }

    /// Print the environment variables for the specified environment
    pub fn print_env(
        &self,
        name: &str,
        settings: &Settings,
        shell_printer: &dyn ShellPrinter,
    ) -> Result<(), String> {
        // Find the name of all environments needed to be used
        let envs = resolve_dependencies(name, &self.environments)?;

        for env in envs.iter() {
            env.print(shell_printer);
        }

        if settings.update_title {
            shell_printer.change_title(name);
        }

        // All good, just show a small message
        let env = envs.first().unwrap();
        let text = format!(
            "{} setting up {}",
            "    Finished".success(),
            env.env_name().info()
        );
        println!("{}", shell_printer.echo(&text));

        Ok(())
    }

    /// Display the environment variables for the specified environment
    pub fn display_env(&self, name: &str, with_dependencies: bool) -> Result<(), String> {
        // Find the name of all environments needed to be used
        let envs = if with_dependencies {
            resolve_dependencies(name, &self.environments)?
        } else {
            vec![self
                .environments
                .iter()
                .find(|env| env.name == name || env.name.starts_with(name))
                .ok_or_else(|| format!("Environment {} not found", name))?]
        };

        for env in envs.iter() {
            env.display();
        }

        Ok(())
    }
}

/// Resolve dependencies for the given environment name
/// and return a vector of environment names
fn resolve_dependencies<'a>(
    name: &str,
    envs: &'a [Environment],
) -> Result<Vec<&'a Environment>, String> {
    let env = envs
        .iter()
        .find(|env| env.name == name || env.name.starts_with(name));
    if env.is_none() {
        return Err(format!("Environment {} not found", name));
    }
    let env = env.unwrap();
    // Create a vector with env
    let mut current_envs = vec![];

    if let Some(reuse) = &env.global.reuse {
        for env_name in reuse {
            let deps = resolve_dependencies(env_name, envs)?;
            deps.iter().for_each(|dep| {
                if !current_envs.contains(dep) {
                    current_envs.push(dep);
                }
            });
        }
    }
    current_envs.push(env);
    Ok(current_envs)
}

/// Read the config file and return a vector of environments
pub fn read_config_file(
    file_path: &Path,
    context: &Context,
) -> Result<Vec<Environment>, Box<dyn std::error::Error>> {
    // Deserialize the file content
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let env_hash: HashMap<String, Environment> = serde_yaml::from_reader(reader)?;
    create_env_vector(context, env_hash)
}

/// Read the config file from a string
/// This is used for testing purposes
#[cfg(test)]
pub fn read_config_from_string(
    content: &str,
    context: &Context,
) -> Result<Vec<Environment>, Box<dyn std::error::Error>> {
    let env_hash: HashMap<String, Environment> = serde_yaml::from_str(content)?;
    create_env_vector(context, env_hash)
}

/// Create a vector of environments from the given hash map
fn create_env_vector(
    context: &Context,
    env_hash: HashMap<String, Environment>,
) -> Result<Vec<Environment>, Box<dyn std::error::Error>> {
    let mut envs = Vec::new();
    for (name, mut env) in env_hash {
        env.name = name;
        env.fold(context);
        envs.push(env);
    }

    // Filter environments based on the context
    envs.retain(|env| {
        if let Some(env_context) = &env.context {
            context.check(env_context)
        } else {
            true
        }
    });

    // Process pattern-based environments
    let pattern_envs: Vec<Environment> = envs
        .iter()
        .filter_map(|env| env.pattern.as_ref().map(|_| create_pattern_envs(env)))
        .flatten()
        .collect();
    envs.retain(|env| env.pattern.is_none());
    envs.extend(pattern_envs);

    sort_environments(&mut envs);

    Ok(envs)
}

/// Sort the environments, using the original key and version
/// to determine the order. If the original key is the same, sort by version.
fn sort_environments(environments: &mut [Environment]) {
    environments.sort_by(|env_a, env_b| {
        // Sort by version if environments have the same original key
        if let (Some(key_a), Some(key_b)) = (&env_a.original_name, &env_b.original_name) {
            if key_a == key_b {
                if let (Some(ver_a), Some(ver_b)) = (&env_a.version, &env_b.version) {
                    if let (Ok(v_a), Ok(v_b)) = (Version::parse(ver_a), Version::parse(ver_b)) {
                        return v_b.cmp(&v_a); // Newer versions first
                    }
                }
            }
        }
        env_a.name.cmp(&env_b.name) // Default lexicographical sort
    });
}

/// Create pattern-based environments from the given environment
fn create_pattern_envs(env: &Environment) -> Vec<Environment> {
    let mut pattern_envs = Vec::new();
    let pattern = match &env.pattern {
        Some(p) => p,
        None => return pattern_envs,
    };

    let path = PathBuf::from(&pattern.path);
    if !path.is_dir() || !path.exists() {
        return pattern_envs;
    }

    let re = Regex::new(&pattern.regex);
    if re.is_err() {
        return pattern_envs;
    }
    let re = re.unwrap();

    for entry in fs::read_dir(path)
        .expect("Could not read directory")
        .flatten()
    {
        if let Ok(name) = entry.file_name().into_string() {
            if let Some(captures) = re.captures(&name) {
                let mut new_env = env.clone();

                for capture in captures.iter().skip(1).flatten() {
                    let value = capture.as_str();
                    new_env.replace_placeholders(value);
                }

                new_env.version = captures.get(1).map(|m| m.as_str().to_string());
                new_env.original_name = Some(env.name.to_string());
                new_env.pattern = None;

                pattern_envs.push(new_env);
            }
        }
    }

    pattern_envs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Context, OperatingSystem};
    use crate::Shell;
    use std::ffi::OsString;

    #[test]
    fn test_replace_placeholders() {
        let mut env = Environment {
            name: "test-{}".to_string(),
            context: None,
            pattern: None,
            global: CommonProperties {
                display: Some("Display {}".to_string()),
                script: Some("echo {}".to_string()),
                set: Some(HashMap::from([("KEY".to_string(), "value-{}".to_string())])),
                append: Some(HashMap::from([(
                    "APPEND".to_string(),
                    "append-{}".to_string(),
                )])),
                prepend: Some(HashMap::from([(
                    "PREPEND".to_string(),
                    "prepend-{}".to_string(),
                )])),
                path: Some(vec!["path/to/{}".to_string()]),
                reuse: None,
                go: Some("go-to-{}".to_string()),
            },
            for_cmd: None,
            for_powershell: None,
            version: None,
            original_name: None,
        };

        env.replace_placeholders("123");

        assert_eq!(env.name, "test-123");
        assert_eq!(env.global.display, Some("Display 123".to_string()));
        assert_eq!(env.global.script, Some("echo 123".to_string()));
        assert_eq!(
            env.global.set,
            Some(HashMap::from([(
                "KEY".to_string(),
                "value-123".to_string()
            )]))
        );
        assert_eq!(
            env.global.append,
            Some(HashMap::from([(
                "APPEND".to_string(),
                "append-123".to_string()
            )]))
        );
        assert_eq!(
            env.global.prepend,
            Some(HashMap::from([(
                "PREPEND".to_string(),
                "prepend-123".to_string()
            )]))
        );
        assert_eq!(env.global.path, Some(vec!["path/to/123".to_string()]));
        assert_eq!(env.global.go, Some("go-to-123".to_string()));
    }

    #[test]
    fn test_replace_placeholders_multiple_occurrences() {
        let mut env = Environment {
            name: "test-{}-{}".to_string(),
            context: None,
            pattern: None,
            global: CommonProperties {
                display: Some("Display {} multiple {}".to_string()),
                script: Some("echo {} twice {}".to_string()),
                set: None,
                append: None,
                prepend: None,
                path: None,
                reuse: None,
                go: None,
            },
            for_cmd: None,
            for_powershell: None,
            version: None,
            original_name: None,
        };

        env.replace_placeholders("123");
        env.replace_placeholders("456");

        // Only the first occurrence should be replaced
        assert_eq!(env.name, "test-123-456");
        assert_eq!(
            env.global.display,
            Some("Display 123 multiple 456".to_string())
        );
        assert_eq!(env.global.script, Some("echo 123 twice 456".to_string()));
    }

    #[test]
    fn test_fold_with_cmd_shell() {
        let mut env = Environment {
            name: "test".to_string(),
            context: None,
            pattern: None,
            global: CommonProperties {
                display: Some("Global Display".to_string()),
                script: None,
                set: Some(HashMap::from([(
                    "GLOBAL_KEY".to_string(),
                    "global_value".to_string(),
                )])),
                append: Some(HashMap::from([(
                    "GLOBAL_APPEND".to_string(),
                    "global_append".to_string(),
                )])),
                prepend: None,
                path: Some(vec!["global/path".to_string()]),
                reuse: Some(vec!["global_reuse".to_string()]),
                go: None,
            },
            for_cmd: Some(CommonProperties {
                display: Some("CMD Display".to_string()),
                script: Some("cmd.exe /c echo test".to_string()),
                set: Some(HashMap::from([
                    ("CMD_KEY".to_string(), "cmd_value".to_string()),
                    ("GLOBAL_KEY".to_string(), "cmd_override".to_string()),
                ])),
                append: None,
                prepend: Some(HashMap::from([(
                    "CMD_PREPEND".to_string(),
                    "cmd_prepend".to_string(),
                )])),
                path: Some(vec!["cmd/path".to_string()]),
                reuse: Some(vec!["cmd_reuse".to_string()]),
                go: Some("cmd_go".to_string()),
            }),
            for_powershell: None,
            version: None,
            original_name: None,
        };

        let context = Context {
            os: OperatingSystem::Windows,
            shell: Shell::Cmd,
            config_path: OsString::new(),
        };

        env.fold(&context);

        // CMD specific properties should override global ones
        assert_eq!(env.global.display, Some("CMD Display".to_string()));
        assert_eq!(env.global.script, Some("cmd.exe /c echo test".to_string()));
        assert_eq!(env.global.go, Some("cmd_go".to_string()));

        // Maps should be merged
        assert_eq!(
            env.global.set,
            Some(HashMap::from([
                ("GLOBAL_KEY".to_string(), "cmd_override".to_string()),
                ("CMD_KEY".to_string(), "cmd_value".to_string()),
            ]))
        );
        assert_eq!(
            env.global.append,
            Some(HashMap::from([(
                "GLOBAL_APPEND".to_string(),
                "global_append".to_string()
            )]))
        );
        assert_eq!(
            env.global.prepend,
            Some(HashMap::from([(
                "CMD_PREPEND".to_string(),
                "cmd_prepend".to_string()
            )]))
        );

        // Vectors should be extended
        assert_eq!(
            env.global.path,
            Some(vec!["global/path".to_string(), "cmd/path".to_string()])
        );
        assert_eq!(
            env.global.reuse,
            Some(vec!["global_reuse".to_string(), "cmd_reuse".to_string()])
        );

        // Shell-specific properties should be consumed
        assert!(env.for_cmd.is_none());
    }

    #[test]
    fn test_resolve_dependencies_from_yaml() {
        // Write a sample config.yaml with dependencies
        let yaml = r#"
envA:
  display: "Environment A"
  set:
    VAR_A: "A"
  use:
    - envB
envB:
  display: "Environment B"
  set:
    VAR_B: "B"
envC:
  display: "Environment C"
  set:
    VAR_C: "C"
"#;

        // Create a context
        let context = Context {
            os: OperatingSystem::Windows,
            shell: Shell::Cmd,
            config_path: OsString::new(),
        };

        // Read environments from the config file
        let envs = read_config_from_string(yaml, &context).unwrap();

        // Resolve dependencies for envA
        let resolved = resolve_dependencies("envA", &envs).unwrap();

        // Should contain envA and envB
        let names: Vec<_> = resolved.iter().map(|e| e.name.as_str()).collect();
        assert!(names[0] == "envB");
        assert!(names[1] == "envA");
        assert_eq!(names.len(), 2);
    }
}
