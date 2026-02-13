// Config Parser — Debugging Exercise (Rust)
//
// This code compiles and runs, but produces incorrect results.
// Find and fix the 3 bugs.
//
// Run tests: rustc --test buggy.rs && ./buggy

#[derive(Debug, Clone, PartialEq)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
    name: String,
}

impl Config {
    fn new(name: &str) -> Config {
        Config {
            host: String::new(),
            port: 0,
            debug: false,
            name: name.to_string(),
        }
    }
}

/// Fills in default values for any fields that are empty/zero.
/// Should return a config with defaults applied.
fn apply_defaults(config: Config) -> Config {
    let mut defaults = config.clone();

    if defaults.port == 0 {
        defaults.port = 8080;
    }
    if defaults.host.is_empty() {
        defaults.host = String::from("localhost");
    }

    config
}

/// Parses a port string into a u16.
/// Should reject values outside the valid port range (0-65535).
fn parse_port(input: &str) -> Result<u16, String> {
    let num: u32 = input.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    Ok(num as u16)
}

/// Parses a string as a boolean value.
/// Should return true for "true"/"1"/"yes", false for "false"/"0"/"no".
fn parse_debug(input: &str) -> bool {
    let debug = input.trim();
    let debug = debug.len();
    debug != 0
}

// ----- Config builder (uses the functions above) -----

fn build_config(name: &str, host: &str, port_str: &str, debug_str: &str) -> Result<Config, String> {
    let port = parse_port(port_str)?;
    let debug = parse_debug(debug_str);

    let mut config = Config::new(name);
    config.host = host.to_string();
    config.port = port;
    config.debug = debug;

    Ok(config)
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    // -- apply_defaults --

    #[test]
    fn test_apply_defaults() {
        let config = Config::new("test-service");
        let with_defaults = apply_defaults(config);

        assert_eq!(with_defaults.host, "localhost",
            "empty host should default to 'localhost'");
        assert_eq!(with_defaults.port, 8080,
            "zero port should default to 8080");
        assert_eq!(with_defaults.name, "test-service",
            "name should be preserved");
    }

    #[test]
    fn test_apply_defaults_preserves_existing() {
        let mut config = Config::new("api");
        config.host = String::from("10.0.0.1");
        config.port = 3000;

        let with_defaults = apply_defaults(config);

        assert_eq!(with_defaults.host, "10.0.0.1",
            "non-empty host should be preserved");
        assert_eq!(with_defaults.port, 3000,
            "non-zero port should be preserved");
    }

    // -- port parsing --

    #[test]
    fn test_port_valid() {
        assert_eq!(parse_port("8080").unwrap(), 8080);
        assert_eq!(parse_port("443").unwrap(), 443);
        assert_eq!(parse_port("0").unwrap(), 0);
        assert_eq!(parse_port("65535").unwrap(), 65535);
    }

    #[test]
    fn test_port_overflow() {
        // 70000 is outside the valid u16 range — should be an error
        assert!(parse_port("70000").is_err(),
            "port 70000 exceeds u16::MAX and should return an error");

        // 65536 is one beyond the max — also an error
        assert!(parse_port("65536").is_err(),
            "port 65536 exceeds u16::MAX and should return an error");
    }

    #[test]
    fn test_port_invalid_string() {
        assert!(parse_port("abc").is_err());
        assert!(parse_port("").is_err());
        assert!(parse_port("-1").is_err());
    }

    // -- debug flag --

    #[test]
    fn test_debug_true() {
        assert!(parse_debug("true"));
        assert!(parse_debug("1"));
        assert!(parse_debug("yes"));
    }

    #[test]
    fn test_debug_false() {
        assert!(!parse_debug("false"),
            "'false' should parse as false, not true");
        assert!(!parse_debug("0"),
            "'0' should parse as false, not true");
        assert!(!parse_debug("no"),
            "'no' should parse as false, not true");
    }

    #[test]
    fn test_debug_whitespace() {
        assert!(parse_debug("  true  "));
        assert!(!parse_debug("  false  "));
    }

    // -- Integration --

    #[test]
    fn test_build_config() {
        let config = build_config("web", "0.0.0.0", "3000", "false").unwrap();

        assert_eq!(config.name, "web");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert!(!config.debug, "debug should be false when input is 'false'");
    }

    #[test]
    fn test_build_config_with_defaults() {
        let config = build_config("svc", "", "8080", "true").unwrap();
        let config = apply_defaults(config);

        // Host was empty string, apply_defaults should set "localhost"
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }
}

fn main() {
    println!("Config Parser — run with: rustc --test buggy.rs && ./buggy");

    // Demo the bugs
    let config = Config::new("demo");
    let with_defaults = apply_defaults(config);
    println!("After apply_defaults: host='{}', port={}", with_defaults.host, with_defaults.port);
    println!("  Expected: host='localhost', port=8080");

    match parse_port("70000") {
        Ok(port) => println!("\nparse_port(\"70000\") = {} (should be an error!)", port),
        Err(e) => println!("\nparse_port(\"70000\") = Err({}) (correct)", e),
    }

    let debug_flag = parse_debug("false");
    println!("\nparse_debug(\"false\") = {} (should be false)", debug_flag);
}
