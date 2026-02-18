// Trait Basics — Interfaces & Traits (Rust)
//
// Covers: defining traits, implementing them, default methods, trait bounds.
//
// Run: rustc trait_basics.rs && ./trait_basics

use std::fmt;

// -------------------------------------------------------------------------
// 1. Defining and implementing a trait
// -------------------------------------------------------------------------

/// A trait representing something that can deliver a notification.
/// This is the contract: provide a channel name and a send method.
trait Notifier {
    // Required method — every implementor must provide this.
    fn channel_name(&self) -> &str;

    // Required method — the core behavior.
    fn send(&self, message: &str) -> Result<(), String>;

    // Default method — calls another required method.
    // Implementors can override this for channel-specific urgent handling.
    fn send_urgent(&self, message: &str) -> Result<(), String> {
        self.send(&format!("[URGENT] {}", message))
    }

    // Default method providing a human-readable description.
    // Uses channel_name() — guaranteed to exist by the trait contract.
    fn describe(&self) -> String {
        format!("Notifier({})", self.channel_name())
    }
}

// --- Email implementation ---

struct EmailNotifier {
    address: String,
}

impl Notifier for EmailNotifier {
    fn channel_name(&self) -> &str {
        "email"
    }

    fn send(&self, message: &str) -> Result<(), String> {
        println!("[Email -> {}] {}", self.address, message);
        Ok(())
    }

    // Uses the default send_urgent — prefixes [URGENT].
    // Uses the default describe.
}

// --- Slack implementation ---

struct SlackNotifier {
    channel: String,
    workspace: String,
}

impl Notifier for SlackNotifier {
    fn channel_name(&self) -> &str {
        "slack"
    }

    fn send(&self, message: &str) -> Result<(), String> {
        println!("[Slack #{} @ {}] {}", self.channel, self.workspace, message);
        Ok(())
    }

    // Override the default urgent behavior: Slack uses @channel mention.
    fn send_urgent(&self, message: &str) -> Result<(), String> {
        self.send(&format!("<!channel> {}", message))
    }
}

// --- Null implementation (useful in tests / dry-run mode) ---

struct NullNotifier;

impl Notifier for NullNotifier {
    fn channel_name(&self) -> &str {
        "null"
    }

    fn send(&self, message: &str) -> Result<(), String> {
        // Silently drop the message. Useful for testing or disabled channels.
        let _ = message;
        Ok(())
    }
}

// -------------------------------------------------------------------------
// 2. Trait bounds — generic functions that work with any Notifier
// -------------------------------------------------------------------------

/// Send a message to a single notifier.
/// T must implement Notifier. Monomorphized at compile time — zero overhead.
fn alert<T: Notifier>(notifier: &T, msg: &str) {
    match notifier.send(msg) {
        Ok(()) => {}
        Err(e) => eprintln!("delivery failed ({}): {}", notifier.channel_name(), e),
    }
}

/// Send a message to all notifiers in a slice.
/// All must be the same concrete type T (homogeneous slice).
fn broadcast<T: Notifier>(notifiers: &[T], msg: &str) {
    for n in notifiers {
        alert(n, msg);
    }
}

/// Send a message and print debug info about the notifier.
/// Multiple bounds: T must implement both Notifier and Display.
fn alert_with_log<T: Notifier + fmt::Display>(notifier: &T, msg: &str) {
    println!("Sending via: {notifier}");
    alert(notifier, msg);
}

// Display impl for EmailNotifier — needed by alert_with_log
impl fmt::Display for EmailNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmailNotifier({})", self.address)
    }
}

// -------------------------------------------------------------------------
// 3. where clause — cleaner syntax for complex bounds
// -------------------------------------------------------------------------

/// Validate that a notifier is reachable by sending a probe.
/// Uses a where clause for readability.
fn probe<N>(notifier: &N) -> bool
where
    N: Notifier + fmt::Debug,
{
    println!("Probing {:?}...", notifier);
    notifier.send("probe").is_ok()
}

#[derive(Debug)]
struct WebhookNotifier {
    url: String,
}

impl Notifier for WebhookNotifier {
    fn channel_name(&self) -> &str {
        "webhook"
    }

    fn send(&self, message: &str) -> Result<(), String> {
        // In real code: make an HTTP POST request
        println!("[Webhook -> {}] {}", self.url, message);
        Ok(())
    }
}

// -------------------------------------------------------------------------
// 4. Supertraits
// -------------------------------------------------------------------------

/// Any type that can be audited must also implement Display (so we can log it)
/// and Notifier (so we can alert about changes).
trait Auditable: fmt::Display + Notifier {
    fn record_id(&self) -> &str;

    // Default: logs a change using Display (from the Display supertrait)
    fn log_change(&self, field: &str, old_val: &str, new_val: &str) {
        println!(
            "[AUDIT] {} (id: {}) changed {}: '{}' -> '{}'",
            self,            // calls Display::fmt
            self.record_id(),
            field,
            old_val,
            new_val,
        );
    }
}

#[derive(Debug)]
struct AlertChannel {
    id: String,
    name: String,
    email: String,
}

impl fmt::Display for AlertChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AlertChannel({})", self.name)
    }
}

impl Notifier for AlertChannel {
    fn channel_name(&self) -> &str {
        "alert-channel"
    }

    fn send(&self, message: &str) -> Result<(), String> {
        println!("[AlertChannel {}] -> {}: {}", self.name, self.email, message);
        Ok(())
    }
}

impl Auditable for AlertChannel {
    fn record_id(&self) -> &str {
        &self.id
    }
    // Uses the default log_change
}

// -------------------------------------------------------------------------
// main
// -------------------------------------------------------------------------

fn main() {
    println!("=== 1. Basic trait usage ===\n");

    let email = EmailNotifier { address: String::from("ops@example.com") };
    let slack = SlackNotifier {
        channel: String::from("incidents"),
        workspace: String::from("acme"),
    };
    let null = NullNotifier;

    email.send("service degraded").unwrap();
    slack.send("service degraded").unwrap();
    null.send("service degraded").unwrap();

    // Default method:
    println!("\n--- Default describe ---");
    println!("{}", email.describe());
    println!("{}", slack.describe());

    // Default vs overridden send_urgent:
    println!("\n--- Urgent: default (email) vs overridden (slack) ---");
    email.send_urgent("database down").unwrap();
    slack.send_urgent("database down").unwrap();

    println!("\n=== 2. Trait bounds ===\n");

    alert(&email, "bounds: single notifier");
    alert_with_log(&email, "bounds: with Display");

    let emails = vec![
        EmailNotifier { address: String::from("alice@example.com") },
        EmailNotifier { address: String::from("bob@example.com") },
    ];
    broadcast(&emails, "broadcast: all email recipients");

    println!("\n=== 3. where clause ===\n");

    let webhook = WebhookNotifier { url: String::from("https://hooks.example.com/alerts") };
    let ok = probe(&webhook);
    println!("probe ok: {ok}");

    println!("\n=== 4. Supertraits ===\n");

    let channel = AlertChannel {
        id: String::from("ch-001"),
        name: String::from("oncall-primary"),
        email: String::from("oncall@example.com"),
    };

    channel.log_change("email", "old@example.com", "oncall@example.com");
    channel.send("supertrait test: can also send").unwrap();
}
