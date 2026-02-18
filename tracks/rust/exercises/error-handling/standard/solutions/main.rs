// Config File Parser — Reference Solution
//
// This is a Cargo project. Setup:
//   cargo new config-parser --name config_parser
//   # copy to src/main.rs
//   # Cargo.toml:
//   #   [dependencies]
//   #   thiserror = "2"
//   #   tempfile = "3"   # dev-dependency for tests
//   cargo test

use std::collections::HashMap;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// All errors that can occur when loading a configuration file.
///
/// Using thiserror derive macros to eliminate boilerplate.
/// Each variant carries enough context to pinpoint the problem without
/// needing to look at source code.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error reading the file — includes the path for context.
    /// Note: #[source] not #[from] because we add the `path` context field.
    /// #[from] would generate From<io::Error> but can't inject `path`.
    #[error("I/O error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// A line in the file doesn't match any valid syntax.
    #[error("parse error in '{path}' on line {line}: {message}")]
    ParseError {
        path: String,
        line: usize,
        message: String,
    },

    /// A required key is absent from its section.
    #[error("missing required field '{key}' in section [{section}]")]
    MissingField { section: String, key: String },

    /// A field is present but its value is the wrong type or out of range.
    #[error("invalid value '{value}' for '{key}' in [{section}]: expected {expected}")]
    InvalidValue {
        section: String,
        key: String,
        value: String,
        expected: String,
    },

    /// Multiple validation failures. Collected so the caller sees all problems
    /// at once rather than fix-one-at-a-time.
    #[error("config validation failed ({} error(s)):\n  {}", .0.len(), .0.join("\n  "))]
    ValidationErrors(Vec<String>),
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Raw parsed config — section name → key → value (all strings).
pub type RawConfig = HashMap<String, HashMap<String, String>>;

/// Validated, typed configuration struct.
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
            other   => Err(format!(
                "'{}' is not a valid log level (expected: debug, info, warn, error)",
                other
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a config file into a map of section → key → value.
///
/// Lines starting with '#' are comments and are skipped.
/// Blank lines are skipped.
/// '[section_name]' sets the current section.
/// 'key = value' adds to the current section.
/// Anything else is a `ParseError`.
pub fn parse_config(path: &str) -> Result<RawConfig, ConfigError> {
    // Cannot use ? here: io::Error -> ConfigError::Io requires a path field,
    // which From<io::Error> cannot provide. We use map_err to inject context.
    let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_string(),
        source: e,
    })?;

    let mut result: RawConfig = HashMap::new();
    let mut current_section = "default".to_string();

    for (idx, raw_line) in contents.lines().enumerate() {
        let line_num = idx + 1;
        let line = raw_line.trim();

        // Skip comments and blank lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header: [section_name]
        if line.starts_with('[') {
            if line.ends_with(']') && line.len() > 2 {
                current_section = line[1..line.len() - 1].trim().to_string();
            } else {
                return Err(ConfigError::ParseError {
                    path: path.to_string(),
                    line: line_num,
                    message: format!("malformed section header: '{}'", line),
                });
            }
            continue;
        }

        // Key-value pair: key = value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if key.is_empty() {
                return Err(ConfigError::ParseError {
                    path: path.to_string(),
                    line: line_num,
                    message: "key cannot be empty".to_string(),
                });
            }
            result
                .entry(current_section.clone())
                .or_default()
                .insert(key, value);
            continue;
        }

        // Unrecognized line
        return Err(ConfigError::ParseError {
            path: path.to_string(),
            line: line_num,
            message: format!(
                "expected '[section]' or 'key = value', got '{}'",
                line
            ),
        });
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/// Validate a RawConfig, coercing string values into typed fields.
///
/// Deliberately collects ALL errors before returning — callers see the full
/// list of problems, not just the first one. This is achieved by pushing to
/// `errors` instead of using `?` for each field check.
pub fn validate(raw: &RawConfig, _path: &str) -> Result<Config, ConfigError> {
    let mut errors: Vec<String> = Vec::new();

    let server = raw.get("server");
    let logging = raw.get("logging");
    let timeouts = raw.get("timeouts");

    // Helper: get a field value or push a missing-field error and return None
    let get_field = |section_map: Option<&HashMap<String, String>>,
                     section: &str,
                     key: &str,
                     errors: &mut Vec<String>|
     -> Option<String> {
        match section_map.and_then(|m| m.get(key)) {
            Some(v) => Some(v.clone()),
            None => {
                errors.push(format!(
                    "missing required field '{}' in section [{}]",
                    key, section
                ));
                None
            }
        }
    };

    // [server] host
    let host = get_field(server, "server", "host", &mut errors)
        .unwrap_or_default();

    // [server] port — must be a valid u16 in range 1..=65535
    let port: Option<u16> = match server.and_then(|m| m.get("port")) {
        None => {
            errors.push("missing required field 'port' in section [server]".to_string());
            None
        }
        Some(v) => match v.parse::<u32>() {
            Err(_) => {
                errors.push(format!(
                    "invalid value '{}' for 'port' in [server]: expected integer 1–65535",
                    v
                ));
                None
            }
            Ok(n) if n == 0 || n > 65535 => {
                errors.push(format!(
                    "invalid value '{}' for 'port' in [server]: must be in range 1–65535",
                    n
                ));
                None
            }
            Ok(n) => Some(n as u16),
        },
    };

    // [server] max_connections
    let max_connections: Option<u32> = match server.and_then(|m| m.get("max_connections")) {
        None => {
            errors.push(
                "missing required field 'max_connections' in section [server]".to_string(),
            );
            None
        }
        Some(v) => match v.parse::<u32>() {
            Err(_) => {
                errors.push(format!(
                    "invalid value '{}' for 'max_connections' in [server]: expected u32",
                    v
                ));
                None
            }
            Ok(n) => Some(n),
        },
    };

    // [logging] log_level
    let log_level: Option<LogLevel> = match logging.and_then(|m| m.get("log_level")) {
        None => {
            errors.push(
                "missing required field 'log_level' in section [logging]".to_string(),
            );
            None
        }
        Some(v) => match v.parse::<LogLevel>() {
            Err(e) => {
                errors.push(format!("invalid value for 'log_level' in [logging]: {}", e));
                None
            }
            Ok(level) => Some(level),
        },
    };

    // [timeouts] timeout_ms
    let timeout_ms: Option<u64> = match timeouts.and_then(|m| m.get("timeout_ms")) {
        None => {
            errors.push(
                "missing required field 'timeout_ms' in section [timeouts]".to_string(),
            );
            None
        }
        Some(v) => match v.parse::<u64>() {
            Err(_) => {
                errors.push(format!(
                    "invalid value '{}' for 'timeout_ms' in [timeouts]: expected u64",
                    v
                ));
                None
            }
            Ok(ms) => Some(ms),
        },
    };

    // If any errors were collected, return them all at once
    if !errors.is_empty() {
        return Err(ConfigError::ValidationErrors(errors));
    }

    // All fields present and valid — safe to unwrap (we checked above)
    Ok(Config {
        host,
        port: port.unwrap(),
        max_connections: max_connections.unwrap(),
        log_level: log_level.unwrap(),
        timeout_ms: timeout_ms.unwrap(),
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load and validate a configuration file.
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let raw = parse_config(path)?;
    validate(&raw, path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(content: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(content.as_bytes()).expect("write");
        file
    }

    const VALID: &str = "
# Deployment config

[server]
host = localhost
port = 8080
max_connections = 100

[logging]
log_level = info

[timeouts]
timeout_ms = 5000
";

    #[test]
    fn test_valid_config() {
        let f = write_temp(VALID);
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
        let f = write_temp(content);
        match load_config(f.path().to_str().unwrap()).unwrap_err() {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")),
                    "expected error mentioning 'port', got: {:?}", msgs);
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_type() {
        let content = "[server]\nhost = localhost\nport = notanumber\nmax_connections = 50\n[logging]\nlog_level = info\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp(content);
        match load_config(f.path().to_str().unwrap()).unwrap_err() {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_range() {
        let content = "[server]\nhost = localhost\nport = 70000\nmax_connections = 50\n[logging]\nlog_level = info\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp(content);
        match load_config(f.path().to_str().unwrap()).unwrap_err() {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_unknown_log_level() {
        let content = "[server]\nhost = localhost\nport = 8080\nmax_connections = 50\n[logging]\nlog_level = verbose\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp(content);
        match load_config(f.path().to_str().unwrap()).unwrap_err() {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("log_level")));
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_errors_collected() {
        // Missing port AND invalid log_level — must both appear
        let content = "[server]\nhost = localhost\nmax_connections = 50\n[logging]\nlog_level = verbose\n[timeouts]\ntimeout_ms = 1000\n";
        let f = write_temp(content);
        match load_config(f.path().to_str().unwrap()).unwrap_err() {
            ConfigError::ValidationErrors(msgs) => {
                assert!(msgs.iter().any(|m| m.contains("port")), "missing port error in {:?}", msgs);
                assert!(msgs.iter().any(|m| m.contains("log_level")), "missing log_level error in {:?}", msgs);
                assert!(msgs.len() >= 2, "expected >= 2 errors, got {}", msgs.len());
            }
            other => panic!("expected ValidationErrors, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_error_bad_line() {
        let content = "[server]\nthis line is not valid\nport = 8080\n";
        let f = write_temp(content);
        assert!(
            matches!(
                load_config(f.path().to_str().unwrap()).unwrap_err(),
                ConfigError::ParseError { .. }
            )
        );
    }

    #[test]
    fn test_file_not_found() {
        assert!(
            matches!(
                load_config("/nonexistent/path/config.toml").unwrap_err(),
                ConfigError::Io { .. }
            )
        );
    }
}

fn main() {
    if let Some(path) = std::env::args().nth(1) {
        match load_config(&path) {
            Ok(config) => println!("{:#?}", config),
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Usage: config_parser <path>");
        println!("Run tests: cargo test");
    }
}
