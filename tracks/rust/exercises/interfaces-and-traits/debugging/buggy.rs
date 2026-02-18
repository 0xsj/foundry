// Notification Pipeline — Debugging Exercise (Rust)
//
// This code has 4 bugs related to traits and trait objects.
// Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

use std::fmt;

// ---------- Types ----------

/// Severity level for a notification.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A notification payload.
#[derive(Debug, Clone)]
struct Notification {
    severity: Severity,
    title: String,
    body: String,
}

impl Notification {
    fn new(severity: Severity, title: &str, body: &str) -> Self {
        Notification {
            severity,
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}

// ---------- Formatter trait ----------

// BUG 1: This trait is not object-safe because of the generic method.
// The pipeline stores formatters as Box<dyn AlertFormatter>, which requires object safety.
// Fix: make the trait object-safe without removing the ability to format strings and numbers.
trait AlertFormatter {
    fn format_str(&self, label: &str, value: &str) -> String;
    fn format_num(&self, label: &str, value: i64) -> String;

    // This generic method prevents dyn AlertFormatter from working.
    // There is more than one way to fix this — pick the simplest.
    fn format_any<T: fmt::Display>(&self, label: &str, value: T) -> String {
        format!("{}: {}", label, value)
    }
}

/// A plain-text formatter.
struct PlainFormatter;

impl AlertFormatter for PlainFormatter {
    fn format_str(&self, label: &str, value: &str) -> String {
        format!("{}: {}", label, value)
    }

    fn format_num(&self, label: &str, value: i64) -> String {
        format!("{}: {}", label, value)
    }
}

/// A bracket-notation formatter (used for log pipelines).
struct BracketFormatter;

impl AlertFormatter for BracketFormatter {
    fn format_str(&self, label: &str, value: &str) -> String {
        format!("[{}={}]", label, value)
    }

    fn format_num(&self, label: &str, value: i64) -> String {
        format!("[{}={}]", label, value)
    }
}

// ---------- Sender trait ----------

// BUG 2: PagerDutySender is used in format strings with `{}`, but it doesn't
// implement Display. The compiler's error message points to the wrong location
// (the format! call inside dispatch_notification) rather than this struct.
// Fix: implement Display for PagerDutySender.
struct PagerDutySender {
    service_key: String,
}

impl PagerDutySender {
    fn new(key: &str) -> Self {
        PagerDutySender { service_key: key.to_string() }
    }

    fn send(&self, title: &str, body: &str) -> Result<(), String> {
        // In production: POST to PagerDuty Events API
        println!("[PagerDuty -> {}] {}: {}", self.service_key, title, body);
        Ok(())
    }
}

// ---------- Orphan rule violation ----------

// BUG 3: This impl violates the orphan rule — it implements an external trait
// (std::fmt::Display) for an external type (std::vec::Vec<String>).
// Fix: remove this impl and replace the one place it's used with a different approach
// (hint: use .join(", ") directly where needed, or wrap Vec<String> in a newtype).
impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.join(", "))
    }
}

// ---------- Alert registry ----------

/// A registry that stores formatters and dispatches notifications.
struct AlertRegistry {
    // BUG 4: This field uses &dyn AlertFormatter (a borrowed reference),
    // which means the registry cannot own the formatters.
    // The registry outlives any temporary reference passed in, so the borrow checker rejects it.
    // Fix: use Box<dyn AlertFormatter> to take ownership.
    formatters: Vec<&dyn AlertFormatter>,
    tags: Vec<String>,
}

impl AlertRegistry {
    fn new() -> Self {
        AlertRegistry {
            formatters: Vec::new(),
            tags: Vec::new(),
        }
    }

    // BUG 4 (continued): the parameter type must match the field type.
    fn add_formatter(&mut self, formatter: &dyn AlertFormatter) {
        self.formatters.push(formatter);
    }

    fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    fn formatter_count(&self) -> usize {
        self.formatters.len()
    }

    fn dispatch_notification(&self, notification: &Notification, sender: &PagerDutySender) {
        // Format the notification using each registered formatter
        let mut parts: Vec<String> = Vec::new();
        for formatter in &self.formatters {
            parts.push(formatter.format_str("title", &notification.title));
            parts.push(formatter.format_num("severity", notification.severity as i64));
        }

        // BUG 3 symptom: this line uses Display on Vec<String>
        // After fixing BUG 3, you'll need to change this line too.
        let formatted = format!("tags=[{}] parts={}", self.tags, parts);

        if let Err(e) = sender.send(&notification.title, &formatted) {
            eprintln!("dispatch failed: {}", e);
        }
    }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_formatter() {
        let f = PlainFormatter;
        assert_eq!(f.format_str("host", "api.example.com"), "host: api.example.com");
        assert_eq!(f.format_num("port", 8080), "port: 8080");
    }

    #[test]
    fn test_bracket_formatter() {
        let f = BracketFormatter;
        assert_eq!(f.format_str("host", "api.example.com"), "[host=api.example.com]");
        assert_eq!(f.format_num("port", 8080), "[port=8080]");
    }

    #[test]
    fn test_registry_add_formatters() {
        let mut registry = AlertRegistry::new();
        registry.add_formatter(Box::new(PlainFormatter));
        registry.add_formatter(Box::new(BracketFormatter));
        assert_eq!(registry.formatter_count(), 2);
    }

    #[test]
    fn test_pagerduty_sender_display() {
        let sender = PagerDutySender::new("svc-key-abc123");
        // Display must show something containing the service key
        let display = format!("{}", sender);
        assert!(
            display.contains("svc-key-abc123"),
            "expected service key in Display, got: {}",
            display
        );
    }

    #[test]
    fn test_dispatch_does_not_panic() {
        let mut registry = AlertRegistry::new();
        registry.add_formatter(Box::new(PlainFormatter));
        registry.add_tag("env:prod");
        registry.add_tag("team:platform");

        let notif = Notification::new(Severity::Error, "disk full", "root partition at 99%");
        let sender = PagerDutySender::new("svc-key-abc123");
        registry.dispatch_notification(&notif, &sender);
    }
}

fn main() {
    println!("Notification Pipeline — run with: rustc --test buggy.rs && ./buggy");
}
