// Config File Parser — Starter
//
// This is a Cargo project exercise. To run:
//   cargo new config-parser --name config_parser
//   # copy this file to src/main.rs
//   # add to Cargo.toml: thiserror = "2"
//   cargo test

use std::collections::HashMap;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// All errors that can occur when loading a configuration file.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    // TODO: Add ParseError variant
    // - Fields: path (String), line (usize), message (String)
    // - Display: "parse error in '{path}' on line {line}: {message}"

    // TODO: Add MissingField variant
    // - Fields: section (String), key (String)
    // - Display: "missing required field '{key}' in section [{section}]"

    // TODO: Add InvalidValue variant
    // - Fields: section (String), key (String), value (String), expected (String)
    // - Display: "invalid value '{value}' for '{key}' in [{section}]: expected {expected}"

    // TODO: Add ValidationErrors variant
    // - Field: Vec<String>
    // - Display: "config validation failed:\n  {errors joined with newline}"
    //   Hint: use a custom Display format with the errors joined

    /// Placeholder to allow compilation — remove once real variants are added.
    #[error("not yet implemented")]
    _Todo,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Raw parsed config — a map of section name to key-value pairs.
pub type RawConfig = HashMap<String, HashMap<String, String>>;

/// Validated, typed configuration.
#[derive(Debug, PartialEq)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub log_level: LogLevel,
    pub timeout_ms: u64,
}

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info"  => Ok(LogLevel::Info),
            "warn"  => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            other   => Err(format!("'{}' is not a valid log level (debug/info/warn/error)", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a config file into a RawConfig map.
///
/// Format:
/// - Lines starting with '#' are comments
/// - Blank lines are ignored
/// - '[section_name]' sets the current section (default: "default")
/// - 'key = value' adds to the current section
/// - Anything else is a ParseError
pub fn parse_config(path: &str) -> Result<RawConfig, ConfigError> {
    // TODO: Read the file using fs::read_to_string.
    // Propagate io::Error as ConfigError::Io using map_err (? alone won't work
    // here because we need to include the path as context).

    // TODO: Iterate over lines, tracking the current section and line number.
    // Parse [section] headers and key=value pairs.
    // Return ConfigError::ParseError for malformed lines.

    todo!("implement parse_config")
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/// Validate a RawConfig, producing a typed Config.
///
/// Collects ALL validation errors before returning. Does NOT short-circuit
/// on the first missing or invalid field.
pub fn validate(raw: &RawConfig, path: &str) -> Result<Config, ConfigError> {
    let _ = path; // used in error messages if you include it — optional
    let mut errors: Vec<String> = Vec::new();

    // TODO: Check for each required field. For each one:
    //   - If missing: push a descriptive error message to `errors`
    //   - If present but invalid type: push a descriptive error message to `errors`
    //   - If valid: store in a local variable (Option<T>)
    //
    // Required fields:
    //   [server]   host (String)
    //   [server]   port (u16, must be 1–65535)
    //   [server]   max_connections (u32)
    //   [logging]  log_level (LogLevel: debug/info/warn/error)
    //   [timeouts] timeout_ms (u64)

    // TODO: After checking all fields, if errors is non-empty, return
    //       Err(ConfigError::ValidationErrors(errors))

    // TODO: Otherwise, unwrap() the Option fields (safe at this point — all
    //       validated) and return Ok(Config { ... })

    todo!("implement validate")
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load and validate a configuration file.
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    // TODO: call parse_config, then validate
    todo!("implement load_config")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_config(content: &str) -> tempfile::NamedTempFile {
        // NOTE: add tempfile = "3" to Cargo.toml for this helper,
        // or write the file to a fixed path and clean up manually.
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(content.as_bytes()).expect("write");
        file
    }

    const VALID_CONFIG: &str = r#"
# Deployment config

[server]
host = localhost
port = 8080
max_connections = 100

[logging]
log_level = info

[timeouts]
timeout_ms = 5000
"#;

    #[test]
    fn test_valid_config() {
        let f = write_temp_config(VALID_CONFIG);
        let config = load_config(f.path().to_str().unwrap()).expect("valid config");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_missing_required_field() {
        let content = "[server]\nhost = localhost\nmax_connections = 50\n[logging]\nlog_level = info\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        match err {
            ConfigError::ValidationErrors(msgs) => {
                assert!(
                    msgs.iter().any(|m| m.contains("port")),
                    "expected error mentioning 'port', got: {:?}", msgs
                );
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_type() {
        let content = "[server]\nhost = localhost\nport = notanumber\nmax_connections = 50\n[logging]\nlog_level = info\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        match err {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_range() {
        let content = "[server]\nhost = localhost\nport = 70000\nmax_connections = 50\n[logging]\nlog_level = info\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        match err {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_unknown_log_level() {
        let content = "[server]\nhost = localhost\nport = 8080\nmax_connections = 50\n[logging]\nlog_level = verbose\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        match err {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("log_level")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_errors_collected() {
        // Missing port AND invalid log_level — both should be in the error list
        let content = "[server]\nhost = localhost\nmax_connections = 50\n[logging]\nlog_level = verbose\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        match err {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")), "missing port error");
                assert!(msgs.iter().any(|m| m.contains("log_level")), "missing log_level error");
                assert!(msgs.len() >= 2, "expected at least 2 errors, got {}", msgs.len());
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_error_bad_line() {
        let content = "[server]\nthis line is not valid\nport = 8080\n";
        let f = write_temp_config(content);
        let err = load_config(f.path().to_str().unwrap()).unwrap_err();
        assert!(
            matches!(err, ConfigError::ParseError { .. }),
            "expected ParseError, got {:?}", err
        );
    }

    #[test]
    fn test_file_not_found() {
        let err = load_config("/nonexistent/path/config.toml").unwrap_err();
        assert!(
            matches!(err, ConfigError::Io { .. }),
            "expected Io error, got {:?}", err
        );
    }
}

fn main() {
    println!("Run with: cargo test");
    println!("Or: cargo run -- path/to/config.toml");

    if let Some(path) = std::env::args().nth(1) {
        match load_config(&path) {
            Ok(config) => println!("{:#?}", config),
            Err(e) => eprintln!("error: {}", e),
        }
    }
}
