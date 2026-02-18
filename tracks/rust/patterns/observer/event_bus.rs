/// Event Bus — Observer pattern using closures and `Box<dyn Fn>`.
///
/// A typed event bus that supports registration, notification, and
/// unsubscription via subscription IDs. This is the closure-based
/// approach — the simplest idiomatic Rust observer.
///
/// Run: `rustc event_bus.rs && ./event_bus`
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- Event types ---

#[derive(Debug, Clone)]
enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
struct AlertEvent {
    service: String,
    severity: Severity,
    message: String,
}

// --- Event Bus ---

type SubscriptionId = u64;

struct EventBus<E> {
    next_id: SubscriptionId,
    listeners: HashMap<SubscriptionId, Box<dyn Fn(&E)>>,
}

impl<E> EventBus<E> {
    fn new() -> Self {
        EventBus {
            next_id: 0,
            listeners: HashMap::new(),
        }
    }

    /// Register a callback. Returns a subscription ID for later unsubscription.
    fn subscribe(&mut self, callback: impl Fn(&E) + 'static) -> SubscriptionId {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.insert(id, Box::new(callback));
        id
    }

    /// Remove a listener by its subscription ID.
    fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        self.listeners.remove(&id).is_some()
    }

    /// Notify all registered listeners with the given event.
    fn emit(&self, event: &E) {
        for callback in self.listeners.values() {
            callback(event);
        }
    }

    /// Return the number of active subscriptions.
    fn subscriber_count(&self) -> usize {
        self.listeners.len()
    }
}

// --- Demo: Alert Monitoring System ---

fn main() {
    let mut bus: EventBus<AlertEvent> = EventBus::new();

    // Observer 1: Console logger — just prints everything
    let log_id = bus.subscribe(|event: &AlertEvent| {
        println!(
            "[LOG] [{:?}] {}: {}",
            event.severity, event.service, event.message
        );
    });

    // Observer 2: Critical alert counter — uses Arc<Mutex> for shared mutable state
    let critical_count = Arc::new(Mutex::new(0u32));
    let counter_clone = critical_count.clone();

    let _counter_id = bus.subscribe(move |event: &AlertEvent| {
        if matches!(event.severity, Severity::Critical) {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            println!("[COUNTER] Critical alert #{}", count);
        }
    });

    // Observer 3: Slack notifier (only warnings and above)
    let _slack_id = bus.subscribe(|event: &AlertEvent| {
        if matches!(event.severity, Severity::Warning | Severity::Critical) {
            println!(
                "[SLACK] :warning: {} — {} ({})",
                event.service, event.message, event.service
            );
        }
    });

    println!("=== {} subscribers registered ===\n", bus.subscriber_count());

    // Emit some events
    bus.emit(&AlertEvent {
        service: "api-gateway".into(),
        severity: Severity::Info,
        message: "Health check passed".into(),
    });

    println!();

    bus.emit(&AlertEvent {
        service: "payment-service".into(),
        severity: Severity::Warning,
        message: "Response latency above 500ms".into(),
    });

    println!();

    bus.emit(&AlertEvent {
        service: "auth-service".into(),
        severity: Severity::Critical,
        message: "Connection pool exhausted".into(),
    });

    println!();

    // Check the critical count from outside the closure
    println!(
        "Total critical alerts observed: {}",
        critical_count.lock().unwrap()
    );

    // Unsubscribe the logger
    println!("\n=== Unsubscribing logger ===\n");
    bus.unsubscribe(log_id);
    println!("Subscribers remaining: {}\n", bus.subscriber_count());

    // This event will NOT be logged, but will still hit counter and slack
    bus.emit(&AlertEvent {
        service: "db-primary".into(),
        severity: Severity::Critical,
        message: "Replication lag exceeding 30s".into(),
    });

    println!(
        "\nFinal critical count: {}",
        critical_count.lock().unwrap()
    );
}
