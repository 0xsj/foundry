// Strategy Pattern: Notification Dispatcher
//
// Demonstrates: Trait objects with Box<dyn NotificationStrategy>
//
// Scenario: A notification service that dispatches alerts through different
// channels (email, SMS, webhook). Channels are registered at runtime and
// looked up by name. This is the classic case for dynamic dispatch — the
// set of channels is open and determined by configuration.
//
// Run: rustc notification.rs && ./notification

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Strategy trait
// ---------------------------------------------------------------------------

/// Each notification channel implements this trait.
/// Object safe: no generics, no Self returns, all methods have &self receiver.
trait NotificationStrategy: fmt::Display {
    /// Send a message to a recipient. Returns Ok(delivery_id) or Err(reason).
    fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<String, NotifyError>;

    /// Channel name used for routing.
    fn channel(&self) -> &str;

    /// Whether this channel supports rich HTML content.
    fn supports_html(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct NotifyError {
    channel: String,
    reason: String,
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] notification failed: {}", self.channel, self.reason)
    }
}

// ---------------------------------------------------------------------------
// Concrete strategies
// ---------------------------------------------------------------------------

struct EmailNotifier {
    smtp_host: String,
    smtp_port: u16,
    from_address: String,
}

impl EmailNotifier {
    fn new(host: &str, port: u16, from: &str) -> Self {
        Self {
            smtp_host: host.to_string(),
            smtp_port: port,
            from_address: from.to_string(),
        }
    }
}

impl NotificationStrategy for EmailNotifier {
    fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<String, NotifyError> {
        // In production: use lettre crate for actual SMTP
        println!(
            "  [EMAIL] {}:{} | From: {} To: {} | Subject: {}",
            self.smtp_host, self.smtp_port, self.from_address, recipient, subject
        );
        println!("  [EMAIL] Body: {}", &body[..body.len().min(80)]);
        Ok(format!("email-{}", recipient.replace('@', "-at-")))
    }

    fn channel(&self) -> &str {
        "email"
    }

    fn supports_html(&self) -> bool {
        true
    }
}

impl fmt::Display for EmailNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmailNotifier({}:{})", self.smtp_host, self.smtp_port)
    }
}

// ---

struct SmsNotifier {
    api_key: String,
    from_number: String,
}

impl SmsNotifier {
    fn new(api_key: &str, from: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            from_number: from.to_string(),
        }
    }
}

impl NotificationStrategy for SmsNotifier {
    fn send(&self, recipient: &str, _subject: &str, body: &str) -> Result<String, NotifyError> {
        if body.len() > 160 {
            return Err(NotifyError {
                channel: "sms".to_string(),
                reason: format!("body too long: {} chars (max 160)", body.len()),
            });
        }
        println!(
            "  [SMS] From: {} To: {} | {}",
            self.from_number, recipient, body
        );
        Ok(format!("sms-{}", recipient))
    }

    fn channel(&self) -> &str {
        "sms"
    }
}

impl fmt::Display for SmsNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SmsNotifier(from={}, key={}...)",
            self.from_number,
            &self.api_key[..self.api_key.len().min(4)]
        )
    }
}

// ---

struct WebhookNotifier {
    endpoint: String,
    secret: String,
    timeout_ms: u64,
}

impl WebhookNotifier {
    fn new(endpoint: &str, secret: &str, timeout_ms: u64) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            secret: secret.to_string(),
            timeout_ms,
        }
    }

    fn compute_signature(&self, payload: &str) -> String {
        // In production: HMAC-SHA256
        format!("sha256={:x}", payload.len() * 31 + self.secret.len())
    }
}

impl NotificationStrategy for WebhookNotifier {
    fn send(&self, _recipient: &str, subject: &str, body: &str) -> Result<String, NotifyError> {
        let payload = format!(r#"{{"subject":"{}","body":"{}"}}"#, subject, body);
        let signature = self.compute_signature(&payload);
        println!(
            "  [WEBHOOK] POST {} (timeout: {}ms)",
            self.endpoint, self.timeout_ms
        );
        println!("  [WEBHOOK] X-Signature: {}", signature);
        println!("  [WEBHOOK] Payload: {}", &payload[..payload.len().min(100)]);
        Ok(format!("webhook-{}", signature))
    }

    fn channel(&self) -> &str {
        "webhook"
    }
}

impl fmt::Display for WebhookNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WebhookNotifier({})", self.endpoint)
    }
}

// ---------------------------------------------------------------------------
// Context: the notification service
// ---------------------------------------------------------------------------

struct NotificationService {
    /// Strategies stored as trait objects — the service doesn't know concrete types.
    strategies: HashMap<String, Box<dyn NotificationStrategy>>,
    /// Fallback channel if the requested one isn't registered.
    fallback: Option<String>,
}

impl NotificationService {
    fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            fallback: None,
        }
    }

    fn register(&mut self, strategy: Box<dyn NotificationStrategy>) {
        let channel = strategy.channel().to_string();
        println!("Registered: {} ({})", channel, strategy);
        self.strategies.insert(channel, strategy);
    }

    fn set_fallback(&mut self, channel: &str) {
        self.fallback = Some(channel.to_string());
    }

    fn notify(
        &self,
        channel: &str,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, NotifyError> {
        let strategy = self
            .strategies
            .get(channel)
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .and_then(|fb| self.strategies.get(fb))
            })
            .ok_or_else(|| NotifyError {
                channel: channel.to_string(),
                reason: "no strategy registered and no fallback configured".to_string(),
            })?;

        strategy.send(recipient, subject, body)
    }

    /// Send to ALL registered channels (fan-out).
    fn broadcast(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Vec<Result<String, NotifyError>> {
        self.strategies
            .values()
            .map(|s| s.send(recipient, subject, body))
            .collect()
    }

    fn list_channels(&self) -> Vec<&str> {
        self.strategies.keys().map(|s| s.as_str()).collect()
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Notification Service (Trait Object Strategy) ===\n");

    // Build the service with runtime-registered strategies
    let mut service = NotificationService::new();

    service.register(Box::new(EmailNotifier::new(
        "smtp.example.com",
        587,
        "alerts@example.com",
    )));
    service.register(Box::new(SmsNotifier::new("sk_live_abc123xyz", "+15551234567")));
    service.register(Box::new(WebhookNotifier::new(
        "https://hooks.slack.com/services/T00/B00/xxx",
        "whsec_supersecret",
        5000,
    )));
    service.set_fallback("email");

    println!("\nRegistered channels: {:?}\n", service.list_channels());

    // --- Single channel dispatch ---
    println!("--- Single channel: email ---");
    match service.notify("email", "ops@company.com", "CPU Alert", "Server cpu-4 at 95%") {
        Ok(id) => println!("  Delivered: {}\n", id),
        Err(e) => println!("  Failed: {}\n", e),
    }

    println!("--- Single channel: sms ---");
    match service.notify("sms", "+15559876543", "CPU Alert", "Server cpu-4 at 95%") {
        Ok(id) => println!("  Delivered: {}\n", id),
        Err(e) => println!("  Failed: {}\n", e),
    }

    println!("--- Single channel: sms (too long) ---");
    let long_body = "A".repeat(200);
    match service.notify("sms", "+15559876543", "Alert", &long_body) {
        Ok(id) => println!("  Delivered: {}\n", id),
        Err(e) => println!("  Failed: {}\n", e),
    }

    println!("--- Single channel: webhook ---");
    match service.notify(
        "webhook",
        "n/a",
        "Deployment",
        "v2.3.1 deployed to production",
    ) {
        Ok(id) => println!("  Delivered: {}\n", id),
        Err(e) => println!("  Failed: {}\n", e),
    }

    // --- Fallback ---
    println!("--- Unknown channel (falls back to email) ---");
    match service.notify("slack", "ops@company.com", "Fallback Test", "This goes to email") {
        Ok(id) => println!("  Delivered via fallback: {}\n", id),
        Err(e) => println!("  Failed: {}\n", e),
    }

    // --- Broadcast ---
    println!("--- Broadcast to all channels ---");
    let results = service.broadcast("ops@company.com", "Critical", "Database primary is down");
    for result in results {
        match result {
            Ok(id) => println!("  Delivered: {}", id),
            Err(e) => println!("  Failed: {}", e),
        }
    }
}
