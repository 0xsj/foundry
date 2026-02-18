// Log Parser — Starter
//
// Implement all TODOs below.
// Run tests: rustc --test main.rs && ./main

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

// ---- LogLevel ---------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: write "TRACE", "DEBUG", "INFO", "WARN", or "ERROR" for each variant
        todo!()
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: match s case-insensitively to the five variants.
        // Return Err(format!("unknown log level: {:?}", s)) for unknown strings.
        todo!()
    }
}

// ---- LogLine ----------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct LogLine {
    timestamp: String,
    level: LogLevel,
    service: String,
    message: String,
}

impl fmt::Display for LogLine {
    // TODO: format as "[{level}] {service} {timestamp}: {message}"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl FromStr for LogLine {
    type Err = String;

    // TODO: parse "{timestamp} [{level}] {service}: {message}"
    // Return Err(String) if the format does not match.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

// ---- LogParser --------------------------------------------------------------

struct LogParser {
    // TODO: add fields
    //   entries: Vec<LogLine>
    //   parse_errors: Vec<String>
}

impl LogParser {
    fn new() -> LogParser {
        // TODO
        todo!()
    }

    fn ingest(&mut self, raw: &str) {
        // TODO: parse raw; push to entries on Ok, push raw to parse_errors on Err
        todo!()
    }

    fn ingest_many(&mut self, lines: &[&str]) {
        // TODO: call ingest for each line
        todo!()
    }

    fn entries(&self) -> &[LogLine] {
        // TODO
        todo!()
    }

    fn errors(&self) -> &[String] {
        // TODO
        todo!()
    }

    fn by_level(&self, level: LogLevel) -> Vec<&LogLine> {
        // TODO: return references to entries with the given level
        todo!()
    }

    fn by_service<'a>(&'a self, service: &str) -> Vec<&'a LogLine> {
        // TODO: return references to entries where service name matches
        todo!()
    }
}

// ---- sanitize_header --------------------------------------------------------

fn sanitize_header<'a>(value: &'a str) -> Cow<'a, str> {
    // TODO: return Cow::Borrowed(value) if clean, Cow::Owned(cleaned) if dirty
    // Dirty = contains '\n' or '\r'
    todo!()
}

// ---- ReportBuilder ----------------------------------------------------------

struct ReportBuilder {
    // TODO: add field: parser: LogParser
}

impl ReportBuilder {
    fn new(parser: LogParser) -> ReportBuilder {
        // TODO
        todo!()
    }

    fn summary(&self) -> String {
        // TODO: produce the multi-line summary described in the README
        // Use write!/writeln! into a String buffer.
        todo!()
    }

    fn format_entries(&self, entries: &[&LogLine]) -> String {
        // TODO: format each entry using its Display impl, join with '\n'
        todo!()
    }
}

// ---- main -------------------------------------------------------------------

fn main() {
    let mut parser = LogParser::new();
    parser.ingest_many(&[
        "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
        "2024-01-15T10:30:01Z [INFO] api-gateway: request accepted",
        "2024-01-15T10:30:02Z [WARN] db-pool: pool exhausted, waiting",
        "not a valid log line",
    ]);

    let report = ReportBuilder::new(parser);
    println!("{}", report.summary());
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LogLevel --

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(),  "INFO");
        assert_eq!(LogLevel::Warn.to_string(),  "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("trace".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("Info".parse::<LogLevel>(),  Ok(LogLevel::Info));
        assert_eq!("WARN".parse::<LogLevel>(),  Ok(LogLevel::Warn));
        assert_eq!("error".parse::<LogLevel>(), Ok(LogLevel::Error));
        assert!("UNKNOWN".parse::<LogLevel>().is_err());
        assert!("".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_ord() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info  < LogLevel::Warn);
        assert!(LogLevel::Warn  < LogLevel::Error);
    }

    // -- LogLine --

    #[test]
    fn test_logline_from_str_valid() {
        let raw = "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused";
        let line: LogLine = raw.parse().expect("should parse");
        assert_eq!(line.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(line.level,     LogLevel::Error);
        assert_eq!(line.service,   "auth-service");
        assert_eq!(line.message,   "connection refused");
    }

    #[test]
    fn test_logline_from_str_message_with_colon() {
        // Messages may contain ": " — only the first colon is the separator
        let raw = "2024-01-15T10:30:00Z [INFO] proxy: upstream: response 200 OK";
        let line: LogLine = raw.parse().expect("should parse");
        assert_eq!(line.service, "proxy");
        assert_eq!(line.message, "upstream: response 200 OK");
    }

    #[test]
    fn test_logline_from_str_invalid() {
        assert!("not a log line".parse::<LogLine>().is_err());
        assert!("".parse::<LogLine>().is_err());
    }

    #[test]
    fn test_logline_display() {
        let line = LogLine {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level:     LogLevel::Error,
            service:   "auth-service".to_string(),
            message:   "connection refused".to_string(),
        };
        assert_eq!(
            line.to_string(),
            "[ERROR] auth-service 2024-01-15T10:30:00Z: connection refused"
        );
    }

    // -- LogParser --

    fn sample_parser() -> LogParser {
        let mut p = LogParser::new();
        p.ingest_many(&[
            "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
            "2024-01-15T10:30:01Z [INFO]  api-gateway: request accepted",
            "2024-01-15T10:30:02Z [WARN]  db-pool: pool exhausted, waiting",
            "2024-01-15T10:30:03Z [ERROR] auth-service: token expired",
            "2024-01-15T10:30:04Z [INFO]  auth-service: user login ok",
            "not a valid log line at all",
        ]);
        p
    }

    #[test]
    fn test_parser_counts() {
        let p = sample_parser();
        assert_eq!(p.entries().len(),  5, "5 valid lines");
        assert_eq!(p.errors().len(),   1, "1 parse error");
    }

    #[test]
    fn test_parser_by_level() {
        let p = sample_parser();
        let errors = p.by_level(LogLevel::Error);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "connection refused");
        assert_eq!(errors[1].message, "token expired");
    }

    #[test]
    fn test_parser_by_service() {
        let p = sample_parser();
        let auth = p.by_service("auth-service");
        assert_eq!(auth.len(), 3); // 2 errors + 1 info
    }

    #[test]
    fn test_parser_by_level_returns_references() {
        let p = sample_parser();
        let errors: Vec<&LogLine> = p.by_level(LogLevel::Error);
        // The returned references should point into p.entries
        let first_ptr = errors[0] as *const LogLine;
        let entries_ptr = &p.entries()[0] as *const LogLine;
        // errors[0] should be either entry[0] or entry[3] — both are errors
        let is_entry_0 = std::ptr::eq(first_ptr, entries_ptr);
        let is_entry_3 = std::ptr::eq(first_ptr, &p.entries()[3] as *const LogLine);
        assert!(is_entry_0 || is_entry_3, "should be a reference into entries, not a clone");
    }

    // -- sanitize_header --

    #[test]
    fn test_sanitize_header_clean_is_borrowed() {
        let clean = sanitize_header("application/json");
        assert!(
            matches!(clean, Cow::Borrowed(_)),
            "clean header should be Cow::Borrowed — no allocation"
        );
        assert_eq!(&*clean, "application/json");
    }

    #[test]
    fn test_sanitize_header_dirty_is_owned() {
        let dirty = sanitize_header("text/html\nX-Injected: evil");
        assert!(
            matches!(dirty, Cow::Owned(_)),
            "dirty header should be Cow::Owned"
        );
        // Newline replaced with space
        assert!(!dirty.contains('\n'));
        assert!(!dirty.contains('\r'));
    }

    #[test]
    fn test_sanitize_header_carriage_return() {
        let dirty = sanitize_header("value\r\ninjected");
        assert!(matches!(dirty, Cow::Owned(_)));
        assert!(!dirty.contains('\r'));
        assert!(!dirty.contains('\n'));
    }

    // -- ReportBuilder --

    #[test]
    fn test_report_summary_contains_totals() {
        let p = sample_parser();
        let builder = ReportBuilder::new(p);
        let s = builder.summary();
        assert!(s.contains("Total entries: 5"), "summary: {:?}", s);
        assert!(s.contains("Parse errors:  1"), "summary: {:?}", s);
    }

    #[test]
    fn test_report_summary_contains_levels() {
        let p = sample_parser();
        let builder = ReportBuilder::new(p);
        let s = builder.summary();
        assert!(s.contains("ERROR"), "summary should show ERROR count: {:?}", s);
        assert!(s.contains("INFO"),  "summary should show INFO count: {:?}", s);
        assert!(s.contains("WARN"),  "summary should show WARN count: {:?}", s);
    }

    #[test]
    fn test_report_summary_services_alphabetical() {
        let p = sample_parser();
        let builder = ReportBuilder::new(p);
        let s = builder.summary();
        // api-gateway should appear before auth-service alphabetically
        let api_pos  = s.find("api-gateway").expect("api-gateway in summary");
        let auth_pos = s.find("auth-service").expect("auth-service in summary");
        // db-pool comes after both
        let db_pos   = s.find("db-pool").expect("db-pool in summary");
        assert!(api_pos < auth_pos, "api-gateway before auth-service");
        assert!(auth_pos < db_pos,  "auth-service before db-pool");
    }

    #[test]
    fn test_format_entries() {
        let p = sample_parser();
        let errors: Vec<&LogLine> = p.by_level(LogLevel::Error);
        let builder = ReportBuilder::new(sample_parser());
        let formatted = builder.format_entries(&errors);
        // Each line should use Display format
        for line in formatted.lines() {
            assert!(
                line.starts_with("[ERROR]"),
                "each line should start with [ERROR]: {:?}", line
            );
        }
        assert_eq!(formatted.lines().count(), errors.len());
    }
}
