use once_cell::sync::Lazy;
use regex::Regex;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::context::Context;
use crate::settings::Settings;
use crate::shell::ShellPrinter;
use crate::Shell;

static ENV_VAR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap());

/// Struct to hold the pattern of the environment
#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Pattern {
    path: String,
    regex: String,
}

/// Common struct for global and shell-specific configuration
#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
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

impl CommonProperties {
    /// Merges another CommonProperties into self.
    fn merge(&mut self, other: Self) {
        self.display = other.display.or(self.display.take());
        self.script = other.script.or(self.script.take());
        self.go = other.go.or(self.go.take());

        let merge_map = |target: &mut Option<HashMap<String, String>>,
                         source: Option<HashMap<String, String>>| {
            if let Some(source_map) = source {
                target
                    .get_or_insert_with(Default::default)
                    .extend(source_map);
            }
        };

        merge_map(&mut self.set, other.set);
        merge_map(&mut self.append, other.append);
        merge_map(&mut self.prepend, other.prepend);

        let merge_vec = |target: &mut Option<Vec<String>>, source: Option<Vec<String>>| {
            if let Some(source_vec) = source {
                target
                    .get_or_insert_with(Default::default)
                    .extend(source_vec);
            }
        };

        merge_vec(&mut self.path, other.path);
        merge_vec(&mut self.reuse, other.reuse);
    }
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
        ENV_VAR_REGEX
            .replace_all(value, |caps: &regex::Captures| {
                printer.env_variable(&caps[1])
            })
            .into_owned()
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

        if let Some(shell_props) = shell_env {
            self.global.merge(shell_props);
        }
    }

    fn display_name(&self) -> &str {
        self.global.display.as_deref().unwrap_or(&self.name)
    }

    /// Sort the environment hashmap by dependencies and return a vector of (key, value) tuples
    fn sort_env_by_dependencies(env_map: &HashMap<String, String>) -> Vec<(String, String)> {
        use std::collections::HashSet;

        // Build dependency graph
        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
        for (key, value) in env_map.iter() {
            let mut set = HashSet::new();
            for cap in ENV_VAR_REGEX.captures_iter(value) {
                let dep = cap[1].to_string();
                if env_map.contains_key(&dep) {
                    set.insert(dep);
                }
            }
            deps.insert(key.clone(), set);
        }

        // Topological sort
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        fn visit(
            key: &str,
            env_map: &HashMap<String, String>,
            deps: &HashMap<String, HashSet<String>>,
            visited: &mut HashSet<String>,
            result: &mut Vec<(String, String)>,
        ) {
            if !visited.insert(key.to_string()) {
                return;
            }
            if let Some(dep_set) = deps.get(key) {
                for dep in dep_set {
                    visit(dep, env_map, deps, visited, result);
                }
            }
            if let Some(val) = env_map.get(key) {
                result.push((key.to_string(), val.clone()));
            }
        }

        let mut keys: Vec<_> = env_map.keys().collect();
        keys.sort(); // for deterministic order

        for key in keys {
            visit(key, env_map, &deps, &mut visited, &mut result);
        }

        result
    }

    /// Print the environment using the provided ShellPrinter
    pub fn print(&self, printer: &dyn ShellPrinter) {
        printer.start(&self.name, self.display_name());

        let process_map = |map: &Option<HashMap<String, String>>, action: &dyn Fn(&str, &str)| {
            if let Some(map) = map {
                for (key, value) in Self::sort_env_by_dependencies(map) {
                    let v = Self::substitute_env_vars(&value, printer);
                    action(&key, &v);
                }
            }
        };

        process_map(&self.global.set, &|k, v| printer.set(k, v));
        process_map(&self.global.append, &|k, v| printer.append(k, v));
        process_map(&self.global.prepend, &|k, v| printer.prepend(k, v));

        if let Some(paths) = &self.global.path {
            for path in paths {
                let p = Self::substitute_env_vars(path, printer);
                printer.prepend_path(&p);
            }
        }

        if let Some(script) = &self.global.script {
            let s = Self::substitute_env_vars(script.trim(), printer);
            printer.run(&s);
        }

        if let Some(go) = &self.global.go {
            let g = Self::substitute_env_vars(go, printer);
            printer.go(&g);
        }

        // Set the USE_PROMPT environment variable
        printer.set("USE_PROMPT", self.name.as_str());
        printer.finish();
    }

    /// Create pattern-based environments from the given environment
    pub fn create_pattern_envs(&self) -> Vec<Environment> {
        let mut pattern_envs = Vec::new();
        let pattern = match &self.pattern {
            Some(p) => p,
            None => return pattern_envs,
        };

        let path = PathBuf::from(&pattern.path);
        if !path.is_dir() {
            return pattern_envs;
        }

        let re = match Regex::new(&pattern.regex) {
            Ok(r) => r,
            Err(_) => return pattern_envs,
        };

        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return pattern_envs,
        };

        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if let Some(captures) = re.captures(&name) {
                    let mut new_env = self.clone();

                    for capture in captures.iter().skip(1).flatten() {
                        new_env.replace_placeholders(capture.as_str());
                    }

                    new_env.version = captures.get(1).map(|m| m.as_str().to_string());
                    new_env.original_name = Some(self.name.to_string());
                    new_env.pattern = None;

                    pattern_envs.push(new_env);
                }
            }
        }

        pattern_envs
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
        let environments = Self::read_config_file(path, context)
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
        let envs = self.resolve_dependencies(name)?;

        for env in &envs {
            env.print(shell_printer);
        }

        if settings.update_title {
            shell_printer.change_title(name);
        }

        // All good, just show a small message
        if let Some(env) = envs.last() {
            shell_printer.finalize(&env.name, env.display_name());
        }
        Ok(())
    }

    /// Resolve dependencies for the given environment name
    /// and return a vector of environment names
    fn resolve_dependencies<'a>(&'a self, name: &str) -> Result<Vec<&'a Environment>, String> {
        let env = self
            .environments
            .iter()
            .find(|env| env.name == name || env.name.starts_with(name))
            .ok_or_else(|| format!("Environment {} not found", name))?;

        let mut current_envs = Vec::new();

        if let Some(reuse) = &env.global.reuse {
            for env_name in reuse {
                let deps = self.resolve_dependencies(env_name)?;
                for dep in deps {
                    if !current_envs.contains(&dep) {
                        current_envs.push(dep);
                    }
                }
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
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let env_hash: HashMap<String, Environment> = serde_yaml::from_reader(reader)?;
        Self::create_env_vector(context, env_hash)
    }

    /// Read the config file from a string
    /// This is used for testing purposes
    #[cfg(test)]
    fn read_config_from_string(
        content: &str,
        context: &Context,
    ) -> Result<Vec<Environment>, Box<dyn std::error::Error>> {
        let env_hash: HashMap<String, Environment> = serde_yaml::from_str(content)?;
        Self::create_env_vector(context, env_hash)
    }

    /// Create a vector of environments from the given hash map
    fn create_env_vector(
        context: &Context,
        env_hash: HashMap<String, Environment>,
    ) -> Result<Vec<Environment>, Box<dyn std::error::Error>> {
        let mut envs: Vec<Environment> = env_hash
            .into_iter()
            .map(|(name, mut env)| {
                env.name = name;
                env.fold(context);
                env
            })
            .filter(|env| env.context.as_ref().is_none_or(|c| context.check(c)))
            .collect();

        // Process pattern-based environments
        let pattern_envs: Vec<Environment> = envs
            .iter()
            .filter_map(|env| env.pattern.as_ref().map(|_| env.create_pattern_envs()))
            .flatten()
            .collect();
        envs.retain(|env| env.pattern.is_none());
        envs.extend(pattern_envs);

        Self::sort_environments(&mut envs);

        Ok(envs)
    }

    /// Sort the environments, using the original key and version
    /// to determine the order. If the original key is the same, sort by version.
    fn sort_environments(environments: &mut [Environment]) {
        environments.sort_by(|a, b| {
            if let (Some(key_a), Some(key_b)) = (&a.original_name, &b.original_name) {
                if key_a == key_b {
                    if let (Some(ver_a), Some(ver_b)) = (&a.version, &b.version) {
                        if let (Ok(v_a), Ok(v_b)) = (Version::parse(ver_a), Version::parse(ver_b)) {
                            return v_b.cmp(&v_a); // Newer versions first
                        }
                    }
                }
            }
            a.name.cmp(&b.name) // Default lexicographical sort
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Context, OperatingSystem};
    use crate::Shell;
    use std::ffi::OsString;

    #[test]
    fn test_sort_env_by_dependencies() {
        // Multiple dependencies between keys
        let mut env_map = HashMap::new();
        env_map.insert("KEY1".to_string(), "foo/${KEY2}/${KEY4}".to_string());
        env_map.insert("KEY2".to_string(), "foo".to_string());
        env_map.insert("KEY3".to_string(), "foo/${KEY2}".to_string());
        env_map.insert("KEY4".to_string(), "foo/${KEY3}".to_string());

        let ordered = Environment::sort_env_by_dependencies(&env_map);
        let ordered_keys: Vec<_> = ordered.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(ordered_keys, vec!["KEY2", "KEY3", "KEY4", "KEY1"]);
    }

    #[test]
    fn test_sort_env_by_dependencies_with_external() {
        // Dependencies with external
        let mut env_map = HashMap::new();
        env_map.insert(
            "KEY1".to_string(),
            "foo/${KEY2}/${KEY3}/${EXTERNAL}".to_string(),
        );
        env_map.insert("KEY2".to_string(), "foo/${EXTERNAL}".to_string());
        env_map.insert("KEY3".to_string(), "foo/${KEY2}/${EXTERNAL}".to_string());

        let ordered = Environment::sort_env_by_dependencies(&env_map);
        let ordered_keys: Vec<_> = ordered.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(ordered_keys, vec!["KEY2", "KEY3", "KEY1"]);
    }

    #[test]
    fn test_sort_env_by_dependencies_with_circular_dependencies() {
        // Dependencies with circular references
        let mut env_map = HashMap::new();
        env_map.insert("KEY1".to_string(), "foo/${KEY2}/${EXTERNAL}".to_string());
        env_map.insert("KEY2".to_string(), "foo/${KEY3}/${EXTERNAL}".to_string());
        env_map.insert("KEY3".to_string(), "foo/${KEY1}/${EXTERNAL}".to_string());

        let ordered = Environment::sort_env_by_dependencies(&env_map);
        // Take care of circular dependencies by ensuring all keys are present
        assert_eq!(ordered.len(), 3);
        let keys: Vec<String> = ordered.iter().map(|(k, _)| k.clone()).collect();
        assert!(keys.contains(&"KEY1".to_string()));
        assert!(keys.contains(&"KEY2".to_string()));
        assert!(keys.contains(&"KEY3".to_string()));
    }

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
        let envs = Config::read_config_from_string(yaml, &context).unwrap();

        let config = Config { environments: envs };

        // Resolve dependencies for envA
        let resolved = config.resolve_dependencies("envA").unwrap();

        // Should contain envA and envB
        let names: Vec<_> = resolved.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["envB", "envA"]);
    }
}
