// Pluggable Notification System — Interfaces & Traits Exercise (Rust)
//
// Implement the types and traits described in standard/README.md.
//
// Run tests: rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

use std::fmt;

// ---------- Error type ----------

#[derive(Debug)]
struct NotifyError {
    message: String,
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for NotifyError {
    fn from(s: String) -> NotifyError {
        NotifyError { message: s }
    }
}

// ---------- Priority ----------

/// Routing priority — determines which channels receive a notification.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

// ---------- Notifier trait ----------

trait Notifier {
    fn channel_name(&self) -> &str;
    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError>;

    fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        todo!()
    }

    fn describe(&self) -> String {
        todo!()
    }
}

// ---------- EmailNotifier ----------

struct EmailNotifier {
    to: String,
    from: String,
}

impl Notifier for EmailNotifier {
    fn channel_name(&self) -> &str {
        todo!()
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        todo!()
    }
}

// ---------- SmsNotifier ----------

struct SmsNotifier {
    number: String,
}

impl Notifier for SmsNotifier {
    fn channel_name(&self) -> &str {
        todo!()
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        todo!()
    }

    // Override: SMS urgent prepends "ALERT: " to subject (no separate subject header)
    fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        todo!()
    }
}

// ---------- WebhookNotifier ----------

// Note: implement Debug manually — redact the secret field.
#[allow(dead_code)] // secret is used for HMAC signing in production; unused in this simulation
struct WebhookNotifier {
    url: String,
    secret: String,
}

impl Notifier for WebhookNotifier {
    fn channel_name(&self) -> &str {
        todo!()
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        todo!()
    }
}

// ---------- Static dispatch helper ----------

/// Send to a single notifier using static dispatch (impl Trait).
/// Logs any error to stderr and returns ().
fn send_to_one(notifier: &impl Notifier, subject: &str, body: &str) {
    todo!()
}

// ---------- NotificationRouter ----------

/// Routes notifications to one or more channels based on priority.
/// Uses dynamic dispatch — channels is Vec<Box<dyn Notifier>>.
struct NotificationRouter {
    channels: Vec<Box<dyn Notifier>>,
}

impl NotificationRouter {
    fn new() -> Self {
        todo!()
    }

    fn add_channel(&mut self, channel: Box<dyn Notifier>) {
        todo!()
    }

    fn channel_count(&self) -> usize {
        todo!()
    }

    /// Dispatch a notification. Returns one Result per channel that was called.
    /// Critical: send_urgent to all
    /// High:     send to all
    /// Normal:   send to first channel only
    /// Low:      no-op, empty vec
    fn dispatch(
        &self,
        priority: Priority,
        subject: &str,
        body: &str,
    ) -> Vec<Result<(), NotifyError>> {
        todo!()
    }
}

// ---------- Result summarizer ----------

/// Returns (success_count, failure_count).
fn summarize_results(results: &[Result<(), NotifyError>]) -> (usize, usize) {
    todo!()
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_email() -> EmailNotifier {
        EmailNotifier {
            to: String::from("oncall@example.com"),
            from: String::from("alerts@example.com"),
        }
    }

    fn make_sms() -> SmsNotifier {
        SmsNotifier { number: String::from("+15551234567") }
    }

    fn make_webhook() -> WebhookNotifier {
        WebhookNotifier {
            url: String::from("https://hooks.example.com/alerts"),
            secret: String::from("s3cr3t"),
        }
    }

    // -- channel_name --

    #[test]
    fn test_channel_names() {
        assert_eq!(make_email().channel_name(), "email");
        assert_eq!(make_sms().channel_name(), "sms");
        assert_eq!(make_webhook().channel_name(), "webhook");
    }

    // -- describe (default method) --

    #[test]
    fn test_describe_default() {
        assert_eq!(make_email().describe(), "Notifier(email)");
        assert_eq!(make_webhook().describe(), "Notifier(webhook)");
    }

    // -- send --

    #[test]
    fn test_send_returns_ok() {
        assert!(make_email().send("subject", "body").is_ok());
        assert!(make_sms().send("subject", "body").is_ok());
        assert!(make_webhook().send("subject", "body").is_ok());
    }

    // -- send_urgent --

    #[test]
    fn test_send_urgent_email_uses_default() {
        // EmailNotifier uses the default send_urgent (prefixes [URGENT]).
        // We can't easily inspect the printed output, but we can verify it returns Ok.
        assert!(make_email().send_urgent("disk full", "root partition at 99%").is_ok());
    }

    #[test]
    fn test_send_urgent_sms_overrides() {
        // SmsNotifier overrides send_urgent — also returns Ok.
        assert!(make_sms().send_urgent("disk full", "root partition at 99%").is_ok());
    }

    // -- WebhookNotifier: secret must not appear in Debug output --

    #[test]
    fn test_webhook_secret_redacted() {
        let w = make_webhook();
        let debug_str = format!("{:?}", w);
        assert!(
            !debug_str.contains("s3cr3t"),
            "secret should not appear in Debug output, got: {}",
            debug_str
        );
        assert!(
            debug_str.contains("REDACTED"),
            "expected REDACTED in Debug output, got: {}",
            debug_str
        );
    }

    // -- send_to_one (static dispatch) --

    #[test]
    fn test_send_to_one_does_not_panic() {
        // Just verifies it doesn't panic on valid input.
        send_to_one(&make_email(), "test", "body");
        send_to_one(&make_sms(), "test", "body");
    }

    // -- NotificationRouter construction --

    #[test]
    fn test_router_starts_empty() {
        let router = NotificationRouter::new();
        assert_eq!(router.channel_count(), 0);
    }

    #[test]
    fn test_router_add_channels() {
        let mut router = NotificationRouter::new();
        router.add_channel(Box::new(make_email()));
        router.add_channel(Box::new(make_sms()));
        router.add_channel(Box::new(make_webhook()));
        assert_eq!(router.channel_count(), 3);
    }

    // -- dispatch: Low priority --

    #[test]
    fn test_dispatch_low_sends_nothing() {
        let mut router = NotificationRouter::new();
        router.add_channel(Box::new(make_email()));
        let results = router.dispatch(Priority::Low, "test", "body");
        assert_eq!(results.len(), 0);
    }

    // -- dispatch: Normal priority --

    #[test]
    fn test_dispatch_normal_sends_to_first_only() {
        let mut router = NotificationRouter::new();
        router.add_channel(Box::new(make_email()));
        router.add_channel(Box::new(make_sms()));
        router.add_channel(Box::new(make_webhook()));

        let results = router.dispatch(Priority::Normal, "disk usage high", "root at 85%");
        assert_eq!(results.len(), 1, "Normal should call exactly one channel");
        assert!(results[0].is_ok());
    }

    #[test]
    fn test_dispatch_normal_empty_router() {
        let router = NotificationRouter::new();
        let results = router.dispatch(Priority::Normal, "test", "body");
        assert_eq!(results.len(), 0);
    }

    // -- dispatch: High priority --

    #[test]
    fn test_dispatch_high_sends_to_all() {
        let mut router = NotificationRouter::new();
        router.add_channel(Box::new(make_email()));
        router.add_channel(Box::new(make_sms()));
        router.add_channel(Box::new(make_webhook()));

        let results = router.dispatch(Priority::High, "service down", "api.example.com unreachable");
        assert_eq!(results.len(), 3, "High should call all 3 channels");
        for r in &results {
            assert!(r.is_ok());
        }
    }

    // -- dispatch: Critical priority --

    #[test]
    fn test_dispatch_critical_sends_urgent_to_all() {
        let mut router = NotificationRouter::new();
        router.add_channel(Box::new(make_email()));
        router.add_channel(Box::new(make_sms()));

        // Critical calls send_urgent. Both channels return Ok.
        let results = router.dispatch(Priority::Critical, "data loss detected", "replica diverged");
        assert_eq!(results.len(), 2, "Critical should call all channels");
        for r in &results {
            assert!(r.is_ok());
        }
    }

    // -- summarize_results --

    #[test]
    fn test_summarize_all_ok() {
        let results: Vec<Result<(), NotifyError>> = vec![Ok(()), Ok(()), Ok(())];
        let (ok, err) = summarize_results(&results);
        assert_eq!(ok, 3);
        assert_eq!(err, 0);
    }

    #[test]
    fn test_summarize_mixed() {
        let results: Vec<Result<(), NotifyError>> = vec![
            Ok(()),
            Err(NotifyError { message: String::from("timeout") }),
            Ok(()),
            Err(NotifyError { message: String::from("connection refused") }),
        ];
        let (ok, err) = summarize_results(&results);
        assert_eq!(ok, 2);
        assert_eq!(err, 2);
    }

    #[test]
    fn test_summarize_empty() {
        let (ok, err) = summarize_results(&[]);
        assert_eq!(ok, 0);
        assert_eq!(err, 0);
    }
}

fn main() {
    let email = EmailNotifier {
        to: String::from("oncall@example.com"),
        from: String::from("alerts@example.com"),
    };
    let sms = SmsNotifier { number: String::from("+15551234567") };
    let webhook = WebhookNotifier {
        url: String::from("https://hooks.example.com/alerts"),
        secret: String::from("s3cr3t"),
    };

    // Static dispatch
    send_to_one(&email, "static dispatch test", "everything looks fine");

    // Dynamic dispatch router
    let mut router = NotificationRouter::new();
    router.add_channel(Box::new(EmailNotifier {
        to: String::from("oncall@example.com"),
        from: String::from("alerts@example.com"),
    }));
    router.add_channel(Box::new(SmsNotifier { number: String::from("+15551234567") }));
    router.add_channel(Box::new(WebhookNotifier {
        url: String::from("https://hooks.example.com/alerts"),
        secret: String::from("s3cr3t"),
    }));

    println!("\n--- Normal priority (first channel only) ---");
    let r = router.dispatch(Priority::Normal, "CPU spike", "p95 latency 800ms");
    let (ok, err) = summarize_results(&r);
    println!("sent: {ok}, failed: {err}");

    println!("\n--- Critical priority (all channels, urgent) ---");
    let r = router.dispatch(Priority::Critical, "Database down", "primary replica unreachable");
    let (ok, err) = summarize_results(&r);
    println!("sent: {ok}, failed: {err}");

    // describe and Debug
    println!("\n--- Describe ---");
    println!("{}", email.describe());
    println!("{}", sms.describe());
    println!("{}", webhook.describe());

    println!("\n--- WebhookNotifier Debug (secret should be redacted) ---");
    println!("{:?}", webhook);
}
