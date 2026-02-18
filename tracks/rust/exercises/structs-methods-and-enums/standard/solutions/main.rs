// Config File Parser — Reference Solution
//
// Run tests:  rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

use std::collections::HashMap;

// ---------- Types ----------

/// A typed configuration value.
///
/// Key design decisions:
/// - Variants carry their data directly (no separate wrapper types needed)
/// - Section(Config) makes the type recursive — Box is not needed because
///   Config contains a HashMap, which already uses heap allocation, so the
///   overall size is bounded.
/// - All derives needed: Debug for printing, Clone for merge, PartialEq for tests
#[derive(Debug, Clone, PartialEq)]
enum ConfigValue {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<ConfigValue>),
    Section(Config),
}

/// A named collection of configuration key-value pairs.
#[derive(Debug, Clone, PartialEq)]
struct Config {
    name: String,
    values: HashMap<String, ConfigValue>,
}

// ---------- Implementation ----------

impl Config {
    /// Creates an empty config with the given name.
    fn new(name: &str) -> Config {
        Config {
            name: name.to_string(),
            values: HashMap::new(),
        }
    }

    /// Inserts or overwrites a key.
    fn set(&mut self, key: &str, value: ConfigValue) {
        self.values.insert(key.to_string(), value);
    }

    /// Returns a reference to the value for the given key, or None.
    ///
    /// Key concept: HashMap::get returns Option<&V>. We re-wrap in Option
    /// but the reference is tied to the lifetime of &self.
    fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    /// Returns the inner &str if the key exists and is ConfigValue::Str.
    ///
    /// Key concept: Matching on a reference to an enum. self.values.get(key)
    /// returns Option<&ConfigValue>. Matching Some(ConfigValue::Str(s)) binds
    /// s as &String. We call .as_str() to get the &str with the same lifetime.
    fn get_str(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(ConfigValue::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the inner i64 if the key exists and is ConfigValue::Int.
    ///
    /// Key concept: i64 is Copy, so we dereference (*n) and return the value
    /// rather than a reference. No lifetime concern here.
    fn get_int(&self, key: &str) -> Option<i64> {
        match self.values.get(key) {
            Some(ConfigValue::Int(n)) => Some(*n),
            _ => None,
        }
    }

    /// Returns the inner bool if the key exists and is ConfigValue::Bool.
    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(ConfigValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Validates the config. Returns Ok(()) or Err with ALL errors.
    ///
    /// Key concept: collect all errors rather than returning early on the first.
    /// This is more useful to callers — they see everything that's wrong at once.
    /// Production config loaders almost always take this approach.
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        // Check required keys are present
        for required in &["host", "port", "service_name"] {
            if self.values.get(*required).is_none() {
                errors.push(format!("missing required key: '{}'", required));
            }
        }

        // Validate port type and range (only if port is present — missing is
        // already caught above)
        match self.values.get("port") {
            Some(ConfigValue::Int(n)) => {
                if *n < 1 || *n > 65535 {
                    errors.push(format!(
                        "invalid port {}: must be in range 1..=65535", n
                    ));
                }
            }
            Some(_) => {
                errors.push(String::from("invalid port: must be an integer"));
            }
            None => {
                // Already caught above — do not double-report
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Merges two configs. Keys in `overrides` take precedence over `base`.
/// Returns a new Config with base's name.
///
/// Key concept: both arguments are taken by value (moved). We mutate base
/// directly by extending its HashMap with overrides. This avoids any cloning.
fn merge(mut base: Config, overrides: Config) -> Config {
    for (key, value) in overrides.values {
        base.values.insert(key, value);
    }
    base
}

/// Parses a raw string into the most appropriate ConfigValue.
///
/// Key concept: try the most specific types first (Bool before Int) to avoid
/// misclassifying "true"/"false" as strings.
fn parse_value(s: &str) -> ConfigValue {
    match s.to_lowercase().as_str() {
        "true" => return ConfigValue::Bool(true),
        "false" => return ConfigValue::Bool(false),
        _ => {}
    }

    if let Ok(n) = s.parse::<i64>() {
        return ConfigValue::Int(n);
    }

    ConfigValue::Str(s.to_string())
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction --

    #[test]
    fn test_new_config_is_empty() {
        let cfg = Config::new("test");
        assert_eq!(cfg.name, "test");
        assert_eq!(cfg.values.len(), 0);
    }

    // -- set and get --

    #[test]
    fn test_set_and_get() {
        let mut cfg = Config::new("app");
        cfg.set("host", ConfigValue::Str(String::from("localhost")));
        cfg.set("port", ConfigValue::Int(8080));
        cfg.set("debug", ConfigValue::Bool(true));

        assert_eq!(cfg.get("host"), Some(&ConfigValue::Str(String::from("localhost"))));
        assert_eq!(cfg.get("port"), Some(&ConfigValue::Int(8080)));
        assert_eq!(cfg.get("debug"), Some(&ConfigValue::Bool(true)));
        assert_eq!(cfg.get("missing"), None);
    }

    #[test]
    fn test_set_overwrites() {
        let mut cfg = Config::new("app");
        cfg.set("timeout", ConfigValue::Int(30));
        cfg.set("timeout", ConfigValue::Int(60));
        assert_eq!(cfg.get("timeout"), Some(&ConfigValue::Int(60)));
    }

    // -- Typed accessors --

    #[test]
    fn test_get_str() {
        let mut cfg = Config::new("app");
        cfg.set("host", ConfigValue::Str(String::from("api.internal")));
        cfg.set("port", ConfigValue::Int(443));

        assert_eq!(cfg.get_str("host"), Some("api.internal"));
        assert_eq!(cfg.get_str("port"), None);
        assert_eq!(cfg.get_str("missing"), None);
    }

    #[test]
    fn test_get_int() {
        let mut cfg = Config::new("app");
        cfg.set("port", ConfigValue::Int(9090));
        cfg.set("name", ConfigValue::Str(String::from("svc")));

        assert_eq!(cfg.get_int("port"), Some(9090));
        assert_eq!(cfg.get_int("name"), None);
        assert_eq!(cfg.get_int("missing"), None);
    }

    #[test]
    fn test_get_bool() {
        let mut cfg = Config::new("app");
        cfg.set("debug", ConfigValue::Bool(false));
        cfg.set("port", ConfigValue::Int(8080));

        assert_eq!(cfg.get_bool("debug"), Some(false));
        assert_eq!(cfg.get_bool("port"), None);
        assert_eq!(cfg.get_bool("missing"), None);
    }

    // -- Nested values --

    #[test]
    fn test_section_value() {
        let mut tls_cfg = Config::new("tls");
        tls_cfg.set("cert_file", ConfigValue::Str(String::from("/etc/ssl/cert.pem")));
        tls_cfg.set("key_file", ConfigValue::Str(String::from("/etc/ssl/key.pem")));

        let mut cfg = Config::new("server");
        cfg.set("host", ConfigValue::Str(String::from("0.0.0.0")));
        cfg.set("tls", ConfigValue::Section(tls_cfg));

        if let Some(ConfigValue::Section(tls)) = cfg.get("tls") {
            assert_eq!(tls.get_str("cert_file"), Some("/etc/ssl/cert.pem"));
        } else {
            panic!("expected tls section");
        }
    }

    #[test]
    fn test_list_value() {
        let mut cfg = Config::new("app");
        cfg.set("allowed_origins", ConfigValue::List(vec![
            ConfigValue::Str(String::from("https://app.example.com")),
            ConfigValue::Str(String::from("https://admin.example.com")),
        ]));

        match cfg.get("allowed_origins") {
            Some(ConfigValue::List(items)) => assert_eq!(items.len(), 2),
            _ => panic!("expected list"),
        }
    }

    // -- validate --

    #[test]
    fn test_validate_valid_config() {
        let mut cfg = Config::new("app");
        cfg.set("service_name", ConfigValue::Str(String::from("api")));
        cfg.set("host", ConfigValue::Str(String::from("0.0.0.0")));
        cfg.set("port", ConfigValue::Int(8080));

        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_required_keys() {
        let cfg = Config::new("empty");
        let result = cfg.validate();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3, "expected 3 errors, got: {:?}", errors);

        let joined = errors.join(" ");
        assert!(joined.contains("host"), "expected 'host' in errors: {}", joined);
        assert!(joined.contains("port"), "expected 'port' in errors: {}", joined);
        assert!(joined.contains("service_name"), "expected 'service_name' in errors: {}", joined);
    }

    #[test]
    fn test_validate_port_out_of_range() {
        let mut cfg = Config::new("app");
        cfg.set("service_name", ConfigValue::Str(String::from("api")));
        cfg.set("host", ConfigValue::Str(String::from("0.0.0.0")));
        cfg.set("port", ConfigValue::Int(99999));

        let result = cfg.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let joined = errors.join(" ");
        assert!(joined.contains("port"), "expected port error: {}", joined);
    }

    #[test]
    fn test_validate_port_wrong_type() {
        let mut cfg = Config::new("app");
        cfg.set("service_name", ConfigValue::Str(String::from("api")));
        cfg.set("host", ConfigValue::Str(String::from("0.0.0.0")));
        cfg.set("port", ConfigValue::Str(String::from("8080")));

        let result = cfg.validate();
        assert!(result.is_err());
    }

    // -- merge --

    #[test]
    fn test_merge_overrides_base() {
        let mut base = Config::new("base");
        base.set("host", ConfigValue::Str(String::from("localhost")));
        base.set("port", ConfigValue::Int(8080));
        base.set("debug", ConfigValue::Bool(false));

        let mut overrides = Config::new("prod");
        overrides.set("host", ConfigValue::Str(String::from("api.prod.internal")));
        overrides.set("debug", ConfigValue::Bool(false));

        let merged = merge(base, overrides);

        assert_eq!(merged.name, "base");
        assert_eq!(merged.get_str("host"), Some("api.prod.internal"));
        assert_eq!(merged.get_int("port"), Some(8080));
        assert_eq!(merged.get_bool("debug"), Some(false));
    }

    #[test]
    fn test_merge_adds_new_keys_from_overrides() {
        let base = Config::new("base");
        let mut overrides = Config::new("extra");
        overrides.set("new_feature", ConfigValue::Bool(true));

        let merged = merge(base, overrides);
        assert_eq!(merged.get_bool("new_feature"), Some(true));
    }

    #[test]
    fn test_merge_empty_override() {
        let mut base = Config::new("base");
        base.set("host", ConfigValue::Str(String::from("localhost")));

        let overrides = Config::new("empty");
        let merged = merge(base, overrides);

        assert_eq!(merged.get_str("host"), Some("localhost"));
    }

    // -- parse_value --

    #[test]
    fn test_parse_value_bool() {
        assert_eq!(parse_value("true"), ConfigValue::Bool(true));
        assert_eq!(parse_value("false"), ConfigValue::Bool(false));
        assert_eq!(parse_value("True"), ConfigValue::Bool(true));
        assert_eq!(parse_value("FALSE"), ConfigValue::Bool(false));
    }

    #[test]
    fn test_parse_value_int() {
        assert_eq!(parse_value("42"), ConfigValue::Int(42));
        assert_eq!(parse_value("-100"), ConfigValue::Int(-100));
        assert_eq!(parse_value("0"), ConfigValue::Int(0));
    }

    #[test]
    fn test_parse_value_str() {
        assert_eq!(parse_value("localhost"), ConfigValue::Str(String::from("localhost")));
        assert_eq!(parse_value(""), ConfigValue::Str(String::from("")));
        assert_eq!(parse_value("8080abc"), ConfigValue::Str(String::from("8080abc")));
    }
}

fn main() {
    let mut defaults = Config::new("defaults");
    defaults.set("host", ConfigValue::Str(String::from("localhost")));
    defaults.set("port", ConfigValue::Int(8080));
    defaults.set("service_name", ConfigValue::Str(String::from("my-service")));
    defaults.set("debug", ConfigValue::Bool(false));
    defaults.set("workers", ConfigValue::Int(4));

    let mut prod = Config::new("production");
    prod.set("host", ConfigValue::Str(String::from("api.internal")));
    prod.set("port", ConfigValue::Int(443));
    prod.set("tls_enabled", ConfigValue::Bool(true));

    let active = merge(defaults, prod);
    println!("Active config: {:#?}", active);

    match active.validate() {
        Ok(()) => println!("\nConfig is valid"),
        Err(errors) => {
            println!("\nValidation errors:");
            for e in &errors {
                println!("  - {}", e);
            }
        }
    }

    let raw_values = vec!["true", "42", "localhost", "-1", "FALSE", "on"];
    println!("\nparse_value demos:");
    for raw in raw_values {
        println!("  {:?} => {:?}", raw, parse_value(raw));
    }
}
