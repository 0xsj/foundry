// Pluggable Notification System — Reference Solution
//
// Run tests: rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

use std::fmt;

// ---------- Error type ----------

/// A delivery failure with a human-readable message.
///
/// Key design: implements From<String> so callers can use
/// `some_string.into()` or `.map_err(NotifyError::from)`.
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

/// Routing priority.
///
/// Derived PartialOrd uses declaration order, so Low < Normal < High < Critical.
/// This makes range comparisons (`priority >= Priority::High`) work naturally.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

// ---------- Notifier trait ----------

/// The contract for a notification delivery channel.
///
/// Required methods: channel_name, send.
/// Default methods: send_urgent (calls send with [URGENT] prefix), describe.
///
/// Default methods can call required methods — they're guaranteed to exist.
/// Implementors only override when the channel needs different urgent behavior.
trait Notifier {
    fn channel_name(&self) -> &str;

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError>;

    /// Default urgent: prefix the subject with [URGENT] and delegate to send.
    /// Override for channels that have their own urgent semantics (e.g., SMS paging).
    fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        self.send(&format!("[URGENT] {}", subject), body)
    }

    /// Human-readable channel description. Uses channel_name (a required method).
    fn describe(&self) -> String {
        format!("Notifier({})", self.channel_name())
    }
}

// ---------- EmailNotifier ----------

struct EmailNotifier {
    to: String,
    from: String,
}

impl Notifier for EmailNotifier {
    fn channel_name(&self) -> &str {
        "email"
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        println!("[Email] {} -> {} | {}: {}", self.from, self.to, subject, body);
        Ok(())
    }

    // Uses default send_urgent (adds [URGENT] prefix) and default describe.
}

// ---------- SmsNotifier ----------

struct SmsNotifier {
    number: String,
}

impl Notifier for SmsNotifier {
    fn channel_name(&self) -> &str {
        "sms"
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        println!("[SMS] {} | {}: {}", self.number, subject, body);
        Ok(())
    }

    /// Override: SMS urgent uses "ALERT: " prefix in the subject itself —
    /// SMS messages have no headers, so we encode urgency in the text.
    fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        self.send(&format!("ALERT: {}", subject), body)
    }
}

// ---------- WebhookNotifier ----------

/// A notifier that POSTs to an HTTP endpoint.
///
/// The secret is used for HMAC request signing.
/// It must never appear in logs or debug output.
#[allow(dead_code)] // secret is used in production for HMAC signing; unused in this simulation
struct WebhookNotifier {
    url: String,
    secret: String,
}

/// Manual Debug implementation — redacts the secret.
/// If we derived Debug, the secret would appear in any {:?} format call.
impl fmt::Debug for WebhookNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookNotifier")
            .field("url", &self.url)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl Notifier for WebhookNotifier {
    fn channel_name(&self) -> &str {
        "webhook"
    }

    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        // In production: sign body with self.secret, POST to self.url
        // Note: we deliberately do NOT print self.secret here
        println!("[Webhook] POST {} | {}: {}", self.url, subject, body);
        Ok(())
    }

    // Uses default send_urgent and default describe.
}

// ---------- Static dispatch helper ----------

/// Send to one notifier using static dispatch (impl Trait = monomorphized).
///
/// The compiler generates a separate version for each concrete T.
/// Zero overhead — the method call can be inlined.
///
/// Compare to the router's dispatch(), which uses &dyn Notifier
/// for heterogeneous channels.
fn send_to_one(notifier: &impl Notifier, subject: &str, body: &str) {
    if let Err(e) = notifier.send(subject, body) {
        eprintln!("send_to_one failed on {}: {}", notifier.channel_name(), e);
    }
}

// ---------- NotificationRouter ----------

/// Routes notifications across a heterogeneous set of channels.
///
/// channels: Vec<Box<dyn Notifier>> — the only way to hold mixed types in one Vec.
/// Each Box<dyn Notifier> is a fat pointer (data + vtable). Fixed size regardless
/// of the underlying concrete type, so they fit uniformly in the Vec.
struct NotificationRouter {
    channels: Vec<Box<dyn Notifier>>,
}

impl NotificationRouter {
    fn new() -> Self {
        NotificationRouter { channels: Vec::new() }
    }

    fn add_channel(&mut self, channel: Box<dyn Notifier>) {
        self.channels.push(channel);
    }

    fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Dispatch based on priority.
    ///
    /// Returns one Result per channel that was actually called.
    /// Callers can use summarize_results() to count successes/failures.
    fn dispatch(
        &self,
        priority: Priority,
        subject: &str,
        body: &str,
    ) -> Vec<Result<(), NotifyError>> {
        match priority {
            // Low: suppress entirely — return empty vec
            Priority::Low => vec![],

            // Normal: call only the first channel
            Priority::Normal => {
                if let Some(ch) = self.channels.first() {
                    vec![ch.send(subject, body)]
                } else {
                    vec![]
                }
            }

            // High: call all channels with send
            Priority::High => self.channels
                .iter()
                .map(|ch| ch.send(subject, body))
                .collect(),

            // Critical: call all channels with send_urgent
            // Dynamic dispatch through the vtable handles the different urgent
            // behaviors (email uses [URGENT] prefix, SMS uses ALERT: prefix).
            Priority::Critical => self.channels
                .iter()
                .map(|ch| ch.send_urgent(subject, body))
                .collect(),
        }
    }
}

// ---------- Result summarizer ----------

/// Count successes and failures in a batch of results.
fn summarize_results(results: &[Result<(), NotifyError>]) -> (usize, usize) {
    let ok = results.iter().filter(|r| r.is_ok()).count();
    let err = results.len() - ok;
    (ok, err)
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
        assert!(make_email().send_urgent("disk full", "root partition at 99%").is_ok());
    }

    #[test]
    fn test_send_urgent_sms_overrides() {
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

    // Static dispatch — monomorphized, zero overhead
    println!("--- Static dispatch (send_to_one) ---");
    send_to_one(&email, "deployment started", "v2.4.1 rolling out to prod");

    // Dynamic dispatch router — heterogeneous channels
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

    println!("\n--- Low priority (suppressed) ---");
    let r = router.dispatch(Priority::Low, "health check", "all systems nominal");
    println!("channels called: {}", r.len());

    println!("\n--- Normal priority (first channel only) ---");
    let r = router.dispatch(Priority::Normal, "CPU spike", "p95 latency 800ms");
    let (ok, err) = summarize_results(&r);
    println!("sent: {ok}, failed: {err}");

    println!("\n--- High priority (all channels) ---");
    let r = router.dispatch(Priority::High, "error rate elevated", "5xx rate above 1%");
    let (ok, err) = summarize_results(&r);
    println!("sent: {ok}, failed: {err}");

    println!("\n--- Critical priority (all channels, urgent) ---");
    let r = router.dispatch(Priority::Critical, "Database down", "primary replica unreachable");
    let (ok, err) = summarize_results(&r);
    println!("sent: {ok}, failed: {err}");

    println!("\n--- Default methods ---");
    println!("{}", email.describe());
    println!("{}", sms.describe());
    println!("{}", webhook.describe());

    println!("\n--- WebhookNotifier Debug (secret redacted) ---");
    println!("{:?}", webhook);
}
