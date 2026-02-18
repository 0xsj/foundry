// Log Parser — Reference Solution
//
// Run tests: rustc --test main.rs && ./main

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as FmtWrite;
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
        f.write_str(match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
        })
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO"  => Ok(LogLevel::Info),
            "WARN"  => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            other   => Err(format!("unknown log level: {:?}", other)),
        }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} {}: {}", self.level, self.service, self.timestamp, self.message)
    }
}

impl FromStr for LogLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expected: "{timestamp} [{level}] {service}: {message}"
        // timestamp has no spaces, so split on the first space
        let (timestamp, rest) = s.split_once(' ')
            .ok_or_else(|| format!("missing space after timestamp: {:?}", s))?;

        // rest = "[LEVEL] service: message"
        let rest = rest.strip_prefix('[')
            .ok_or_else(|| "expected '[' after timestamp".to_string())?;
        let (level_str, rest) = rest.split_once(']')
            .ok_or_else(|| "expected ']' after level".to_string())?;

        // strip the space after ']', tolerating optional extra spaces
        let rest = rest.trim_start();

        // rest = "service: message"
        // split_once uses ": " so a service named "a:b" with message "c" also works
        let (service, message) = rest.split_once(": ")
            .ok_or_else(|| format!("expected ': ' separator in {:?}", rest))?;

        let level = level_str.trim().parse::<LogLevel>()
            .map_err(|e| format!("invalid level: {}", e))?;

        Ok(LogLine {
            timestamp: timestamp.to_string(),
            level,
            service: service.trim().to_string(),
            message: message.to_string(),
        })
    }
}

// ---- LogParser --------------------------------------------------------------

struct LogParser {
    entries: Vec<LogLine>,
    parse_errors: Vec<String>,
}

impl LogParser {
    fn new() -> LogParser {
        LogParser {
            entries: Vec::new(),
            parse_errors: Vec::new(),
        }
    }

    fn ingest(&mut self, raw: &str) {
        match raw.parse::<LogLine>() {
            Ok(line) => self.entries.push(line),
            Err(_)   => self.parse_errors.push(raw.to_string()),
        }
    }

    fn ingest_many(&mut self, lines: &[&str]) {
        for line in lines {
            self.ingest(line);
        }
    }

    fn entries(&self) -> &[LogLine] {
        &self.entries
    }

    fn errors(&self) -> &[String] {
        &self.parse_errors
    }

    fn by_level(&self, level: LogLevel) -> Vec<&LogLine> {
        self.entries.iter()
            .filter(|e| e.level == level)
            .collect()
    }

    fn by_service<'a>(&'a self, service: &str) -> Vec<&'a LogLine> {
        self.entries.iter()
            .filter(|e| e.service == service)
            .collect()
    }
}

// ---- sanitize_header --------------------------------------------------------

fn sanitize_header<'a>(value: &'a str) -> Cow<'a, str> {
    if value.contains('\n') || value.contains('\r') {
        // Uncommon path: strip header-injection characters
        Cow::Owned(value.replace(['\n', '\r'], " "))
    } else {
        // Common path: clean header — borrow, no allocation
        Cow::Borrowed(value)
    }
}

// ---- ReportBuilder ----------------------------------------------------------

struct ReportBuilder {
    parser: LogParser,
}

impl ReportBuilder {
    fn new(parser: LogParser) -> ReportBuilder {
        ReportBuilder { parser }
    }

    fn summary(&self) -> String {
        let p = &self.parser;
        let mut out = String::with_capacity(512);

        writeln!(out, "=== Log Summary ===").unwrap();
        writeln!(out, "Total entries: {}", p.entries.len()).unwrap();
        writeln!(out, "Parse errors:  {}", p.parse_errors.len()).unwrap();
        writeln!(out).unwrap();

        writeln!(out, "By level:").unwrap();
        for level in [LogLevel::Trace, LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error] {
            let count = p.entries.iter().filter(|e| e.level == level).count();
            writeln!(out, "  {:8} {}", level, count).unwrap();
        }

        writeln!(out).unwrap();

        // BTreeSet: sorted automatically, no duplicates
        let services: BTreeSet<&str> = p.entries.iter()
            .map(|e| e.service.as_str())
            .collect();
        let services_list = services.into_iter().collect::<Vec<_>>().join(", ");
        writeln!(out, "Services seen: {}", services_list).unwrap();

        out
    }

    fn format_entries(&self, entries: &[&LogLine]) -> String {
        entries.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ---- main -------------------------------------------------------------------

fn main() {
    let mut parser = LogParser::new();
    parser.ingest_many(&[
        "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
        "2024-01-15T10:30:01Z [INFO]  api-gateway: request accepted",
        "2024-01-15T10:30:02Z [WARN]  db-pool: pool exhausted, waiting",
        "2024-01-15T10:30:03Z [ERROR] auth-service: token expired",
        "2024-01-15T10:30:04Z [INFO]  auth-service: user login ok",
        "not a valid log line",
    ]);

    let report = ReportBuilder::new(parser);
    println!("{}", report.summary());

    // Demonstrate sanitize_header
    let clean = sanitize_header("application/json");
    let dirty = sanitize_header("text/html\nX-Injected: evil");
    println!("clean (borrowed): {}", &*clean);
    println!("dirty (owned):    {}", &*dirty);
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
        assert_eq!(auth.len(), 3);
    }

    #[test]
    fn test_parser_by_level_returns_references() {
        let p = sample_parser();
        let errors: Vec<&LogLine> = p.by_level(LogLevel::Error);
        let first_ptr = errors[0] as *const LogLine;
        let is_entry_0 = std::ptr::eq(first_ptr, &p.entries()[0] as *const LogLine);
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
        assert!(matches!(dirty, Cow::Owned(_)));
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
        assert!(s.contains("ERROR"), "summary should show ERROR: {:?}", s);
        assert!(s.contains("INFO"),  "summary should show INFO: {:?}", s);
        assert!(s.contains("WARN"),  "summary should show WARN: {:?}", s);
    }

    #[test]
    fn test_report_summary_services_alphabetical() {
        let p = sample_parser();
        let builder = ReportBuilder::new(p);
        let s = builder.summary();
        let api_pos  = s.find("api-gateway").expect("api-gateway in summary");
        let auth_pos = s.find("auth-service").expect("auth-service in summary");
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
        for line in formatted.lines() {
            assert!(line.starts_with("[ERROR]"), "each line starts with [ERROR]: {:?}", line);
        }
        assert_eq!(formatted.lines().count(), errors.len());
    }
}
