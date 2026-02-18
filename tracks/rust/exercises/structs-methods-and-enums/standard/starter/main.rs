// Config File Parser — Structs, Methods & Enums Exercise (Rust)
//
// Implement a typed configuration value system with a Config struct
// that supports querying, validating, and merging configurations.
//
// Run tests: rustc --test main.rs && ./main

use std::collections::HashMap;

// ---------- Types ----------

/// A typed configuration value.
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
        todo!()
    }

    /// Inserts or overwrites a key.
    fn set(&mut self, key: &str, value: ConfigValue) {
        todo!()
    }

    /// Returns a reference to the value for the given key, or None.
    fn get(&self, key: &str) -> Option<&ConfigValue> {
        todo!()
    }

    /// Returns the inner &str if the key exists and is ConfigValue::Str.
    fn get_str(&self, key: &str) -> Option<&str> {
        todo!()
    }

    /// Returns the inner i64 if the key exists and is ConfigValue::Int.
    fn get_int(&self, key: &str) -> Option<i64> {
        todo!()
    }

    /// Returns the inner bool if the key exists and is ConfigValue::Bool.
    fn get_bool(&self, key: &str) -> Option<bool> {
        todo!()
    }

    /// Validates that required keys exist and have appropriate values.
    /// Returns Ok(()) if valid, or Err with ALL validation errors found.
    fn validate(&self) -> Result<(), Vec<String>> {
        todo!()
    }
}

/// Merges two configs. Keys in `overrides` take precedence over `base`.
/// Returns a new Config with base's name.
fn merge(base: Config, overrides: Config) -> Config {
    todo!()
}

/// Parses a raw string into the most appropriate ConfigValue:
///   "true"/"false" (case-insensitive) -> Bool
///   parseable as i64 -> Int
///   anything else -> Str
fn parse_value(s: &str) -> ConfigValue {
    todo!()
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
        assert_eq!(cfg.get_str("port"), None);   // wrong type
        assert_eq!(cfg.get_str("missing"), None); // missing
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

        // Access the nested section via get
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

        // All three required keys are missing — all three errors should appear
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
        cfg.set("port", ConfigValue::Str(String::from("8080")));  // wrong type

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

        assert_eq!(merged.name, "base");  // name comes from base
        assert_eq!(merged.get_str("host"), Some("api.prod.internal")); // overridden
        assert_eq!(merged.get_int("port"), Some(8080));                  // from base
        assert_eq!(merged.get_bool("debug"), Some(false));               // from overrides
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
    // Example usage
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

    // parse_value demo
    let raw_values = vec!["true", "42", "localhost", "-1", "FALSE", "on"];
    for raw in raw_values {
        println!("parse_value({:?}) = {:?}", raw, parse_value(raw));
    }
}
