// config-loader/src/config.rs

// BUG 4: RawConfig is used in main.rs (via load_raw's return type and
// load_defaults's parameter), but it's declared without `pub`.
// It needs to be visible outside this module.
struct RawConfig {
    pub host: String,
    pub port: u16,
    pub timeout_str: String,
}

/// Application config after defaults are applied.
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub timeout_str: String,
}

/// Load raw config from a file path. Returns a RawConfig with whatever
/// was in the file (empty strings / zeros if the file doesn't exist).
pub fn load_raw(_path: &str) -> RawConfig {
    // Simplified: just return defaults in this exercise
    RawConfig {
        host: String::new(),
        port: 0,
        timeout_str: String::new(),
    }
}

/// Apply default values for any unset fields.
// BUG 5: This function is private but called from main.rs.
// It needs to be visible to the parent module (main.rs in this case).
fn load_defaults(raw: RawConfig) -> AppConfig {
    AppConfig {
        host: if raw.host.is_empty() {
            "localhost".to_string()
        } else {
            raw.host
        },
        port: if raw.port == 0 { 8080 } else { raw.port },
        timeout_str: if raw.timeout_str.is_empty() {
            "30s".to_string()
        } else {
            raw.timeout_str
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_defaults_fills_host() {
        let raw = RawConfig {
            host: String::new(),
            port: 0,
            timeout_str: String::new(),
        };
        let cfg = load_defaults(raw);
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.timeout_str, "30s");
    }

    #[test]
    fn test_load_defaults_preserves_values() {
        let raw = RawConfig {
            host: "10.0.0.1".to_string(),
            port: 3000,
            timeout_str: "60s".to_string(),
        };
        let cfg = load_defaults(raw);
        assert_eq!(cfg.host, "10.0.0.1");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.timeout_str, "60s");
    }
}
