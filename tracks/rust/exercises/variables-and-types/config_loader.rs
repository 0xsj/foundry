use std::collections::HashMap;

/// Config holds typed service configuration.
/// Note: Rust has no zero values — every field must be explicitly set.
/// The Default trait is the idiomatic way to handle defaults.
#[derive(Debug, PartialEq)]
pub struct Config {
    pub host: String,
    pub port: i32,
    pub max_retries: i32,
    pub timeout: f64,
    pub debug: bool,
    pub service_name: String,
}

impl Default for Config {
    fn default() -> Self {
        // TODO: return a Config with application-level defaults
        todo!()
    }
}

/// Parse raw string key-value pairs into a typed Config.
/// Missing keys should fall back to defaults.
/// Invalid values should return an Err.
pub fn load_config(raw: &HashMap<String, String>) -> Result<Config, String> {
    // TODO: implement
    // Hint: str::parse::<i32>(), str::parse::<f64>(), str::parse::<bool>()
    Err("not implemented".to_string())
}
