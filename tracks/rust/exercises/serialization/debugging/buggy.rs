// Config Loader — Debugging Exercise
//
// This code has 4 bugs related to serialization concepts.
// Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

// ─────────────────────────────────────────────────────────────────────────────
// JSON helpers
// ─────────────────────────────────────────────────────────────────────────────

fn get_str(json: &str, field: &str) -> Option<String> {
    let key = format!(r#""{field}":""#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn get_u64(json: &str, field: &str) -> Option<u64> {
    let key = format!(r#""{field}":"#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    if rest.starts_with('"') { return None; }
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn get_bool(json: &str, field: &str) -> Option<bool> {
    if json.contains(&format!(r#""{field}":true"#)) { Some(true) }
    else if json.contains(&format!(r#""{field}":false"#)) { Some(false) }
    else { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// Config types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    // BUG 2 is in this function.
    // Hint: look carefully at the variant names being matched.
    fn from_str(s: &str) -> Result<LogLevel, String> {
        match s {
            "debug" => Ok(LogLevel::Debug),
            "info"  => Ok(LogLevel::Info),
            "warn"  => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            // This is what serde's #[serde(rename_all = "UPPERCASE")] would catch at compile time.
            // We're matching on lowercase but the JSON uses uppercase strings.
            // The bug is that the matcher doesn't handle the actual values in the test data.
            other   => Err(format!("unknown log level: '{other}'")),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    pool_size: u32,
}

// A service configuration loaded from JSON
#[derive(Debug)]
struct ServiceConfig {
    service_name: String,
    log_level: LogLevel,

    // BUG 1: This field should use a default when absent, but currently panics.
    // Hint: what does .unwrap() do when called on None?
    retry_count: u32,

    database: DatabaseConfig,

    // BUG 3 is here.
    // This field models "a description that might not be present, and if present,
    // might be explicitly null". The type should be Option<Option<String>>.
    // Currently it's typed wrong, causing incorrect behavior.
    description: Option<String>,

    // BUG 4: This field stores a timestamp from the JSON.
    // The parse function below expects a specific format, but the test data
    // uses a different format. Find the mismatch.
    last_updated: Option<u64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing
// ─────────────────────────────────────────────────────────────────────────────

fn parse_database(json: &str) -> Result<DatabaseConfig, String> {
    let host = get_str(json, "host")
        .ok_or_else(|| String::from("missing database.host"))?;
    let port = get_u64(json, "port")
        .ok_or_else(|| String::from("missing database.port"))? as u16;
    let name = get_str(json, "db_name")
        .ok_or_else(|| String::from("missing database.db_name"))?;
    let pool_size = get_u64(json, "pool_size").unwrap_or(5) as u32;

    Ok(DatabaseConfig { host, port, name, pool_size })
}

fn parse_service_config(json: &str) -> Result<ServiceConfig, String> {
    let service_name = get_str(json, "service_name")
        .ok_or_else(|| String::from("missing required field: service_name"))?;

    // Parse log_level string into the enum
    let log_level = get_str(json, "log_level")
        .map(|s| LogLevel::from_str(&s))
        .transpose()?
        .unwrap_or_default();

    // BUG 1: .unwrap() panics when retry_count is absent.
    // In serde, you'd use #[serde(default)] to get 0 when the field is missing.
    // Fix this so it returns a default value instead of panicking.
    let retry_count = get_u64(json, "retry_count").unwrap() as u32;

    // Parse the nested database section
    // We extract the database object by finding its JSON substring
    let db_json = extract_nested_object(json, "database")
        .ok_or_else(|| String::from("missing required field: database"))?;
    let database = parse_database(&db_json)?;

    // BUG 3: This field is typed as Option<String> but should be Option<Option<String>>.
    // The semantics: outer None = field absent from JSON, inner None = field present but null.
    // With the wrong type, we can't distinguish "absent" from "explicitly null".
    // Fix the type here and in the struct definition above.
    let description: Option<String> = get_str(json, "description");

    // BUG 4: parse_timestamp expects "YYYY-MM-DDTHH:MM:SSZ" format.
    // The test data uses "YYYY-MM-DD HH:MM:SS" (space instead of T, no trailing Z).
    // Fix parse_timestamp to accept the actual format in the test data.
    let last_updated = get_str(json, "last_updated")
        .map(|s| parse_timestamp(&s))
        .transpose()?;

    Ok(ServiceConfig {
        service_name,
        log_level,
        retry_count,
        database,
        description,
        last_updated,
    })
}

// Extracts the value of a nested JSON object key as a substring
fn extract_nested_object(json: &str, field: &str) -> Option<String> {
    let key = format!(r#""{field}""#);
    let start = json.find(&key)? + key.len();
    let rest = json[start..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();

    if !rest.starts_with('{') { return None; }

    let mut depth = 0;
    let mut end = 0;
    for (i, c) in rest.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 { end = i + 1; break; }
            }
            _ => {}
        }
    }
    if end == 0 { return None; }
    Some(rest[..end].to_string())
}

// Parses a timestamp string "YYYY-MM-DDTHH:MM:SSZ" into a Unix-like epoch value.
// BUG 4: The format expected here doesn't match the test data format.
// The test uses "2026-02-18 14:30:00" but this function expects "T" separator and "Z" suffix.
fn parse_timestamp(s: &str) -> Result<u64, String> {
    // Simplified: just extract the year as a stand-in for a real timestamp
    // The bug is in the format checking below.
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return Err(format!("invalid timestamp format, expected YYYY-MM-DDTHH:MM:SSZ, got '{s}'"));
    }
    if !parts[1].ends_with('Z') {
        return Err(format!("timestamp must end with Z, got '{s}'"));
    }
    // Return year as a fake epoch (real code would compute proper Unix time)
    let year: u64 = parts[0][..4].parse()
        .map_err(|_| format!("invalid year in timestamp '{s}'"))?;
    Ok(year * 365 * 24 * 3600) // fake epoch — just for illustration
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_base_config() -> String {
        // Complete valid config that all tests can start from
        r#"{
            "service_name": "auth-service",
            "log_level": "INFO",
            "retry_count": 3,
            "database": {
                "host": "db.internal",
                "port": 5432,
                "db_name": "auth_db",
                "pool_size": 10
            }
        }"#.to_string()
    }

    // BUG 1: retry_count uses .unwrap() — panics when field is absent
    #[test]
    fn test_missing_retry_count_uses_default() {
        // retry_count is intentionally absent — should default to 0, not panic
        let json = r#"{
            "service_name": "payment-service",
            "log_level": "INFO",
            "database": {
                "host": "db.internal",
                "port": 5432,
                "db_name": "pay_db"
            }
        }"#;
        let config = parse_service_config(json).expect("should not return error");
        assert_eq!(config.retry_count, 0); // absent = default 0
    }

    // BUG 2: LogLevel::from_str doesn't handle the actual values in the test data
    #[test]
    fn test_deserialize_log_level() {
        // The JSON uses uppercase "INFO", "WARN", "ERROR", "DEBUG"
        // but from_str matches lowercase only
        let json = r#"{
            "service_name": "s",
            "log_level": "INFO",
            "database": {"host":"h","port":5432,"db_name":"d"}
        }"#;
        let config = parse_service_config(json).expect("should parse");
        assert_eq!(config.log_level, LogLevel::Info);

        let json2 = r#"{
            "service_name": "s",
            "log_level": "WARN",
            "database": {"host":"h","port":5432,"db_name":"d"}
        }"#;
        let config2 = parse_service_config(json2).expect("should parse");
        assert_eq!(config2.log_level, LogLevel::Warn);
    }

    // BUG 3: description typed as Option<String> instead of Option<Option<String>>
    #[test]
    fn test_nested_optional_description() {
        // Case 1: description is absent — outer None
        let json_absent = r#"{
            "service_name": "s",
            "log_level": "INFO",
            "database": {"host":"h","port":5432,"db_name":"d"}
        }"#;

        // Case 2: description is present and has a value — outer Some, inner Some
        let json_present = r#"{
            "service_name": "s",
            "log_level": "INFO",
            "database": {"host":"h","port":5432,"db_name":"d"},
            "description": "primary auth service"
        }"#;

        let absent = parse_service_config(json_absent).expect("should parse");
        let present = parse_service_config(json_present).expect("should parse");

        // With Option<Option<String>>:
        // absent.description == None (outer None: field not in JSON)
        // present.description == Some(Some("primary auth service")) (outer Some: present, inner Some: has value)
        assert!(absent.description.is_none());
        // This assertion fails with Option<String> because Some("primary auth service") != Some(Some(...))
        // With the correct type, this should be Some(Some("primary auth service"))
        // For the test to pass, the type must be Option<Option<String>> and the value
        // must match Some(Some("primary auth service".to_string()))
        //
        // The test as written just checks that present.description is Some:
        assert!(present.description.is_some());
    }

    // BUG 4: timestamp format mismatch
    #[test]
    fn test_timestamp_parsing() {
        // The test data uses "2026-02-18 14:30:00" (space separator, no Z suffix)
        // but parse_timestamp expects "2026-02-18T14:30:00Z" (T separator, Z suffix)
        let json = r#"{
            "service_name": "s",
            "log_level": "INFO",
            "database": {"host":"h","port":5432,"db_name":"d"},
            "last_updated": "2026-02-18 14:30:00"
        }"#;
        let config = parse_service_config(json).expect("should parse timestamp");
        assert!(config.last_updated.is_some());
        // The year-based epoch for 2026:
        let expected = 2026u64 * 365 * 24 * 3600;
        assert_eq!(config.last_updated.unwrap(), expected);
    }

    // Control test — should pass once all 4 bugs are fixed
    #[test]
    fn test_full_config_parses() {
        let json = r#"{
            "service_name": "inventory-service",
            "log_level": "DEBUG",
            "retry_count": 5,
            "database": {
                "host": "inventory-db.internal",
                "port": 5432,
                "db_name": "inventory",
                "pool_size": 20
            },
            "description": "manages product inventory",
            "last_updated": "2026-01-15 09:00:00"
        }"#;

        let config = parse_service_config(json).expect("full config should parse");
        assert_eq!(config.service_name, "inventory-service");
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.retry_count, 5);
        assert_eq!(config.database.host, "inventory-db.internal");
        assert_eq!(config.database.pool_size, 20);
        assert!(config.description.is_some());
        assert!(config.last_updated.is_some());
    }
}

fn main() {
    println!("Config Loader — run with: rustc --test buggy.rs && ./buggy");
}
