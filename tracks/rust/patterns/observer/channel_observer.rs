/// Channel-Based Observer — Using `mpsc::Sender` clones for fan-out.
///
/// Each subscriber gets its own channel receiver. The publisher sends
/// cloned events to all subscribers. Dead subscribers (dropped receivers)
/// are automatically pruned.
///
/// This is the most Go-like approach to Observer in Rust.
///
/// Run: `rustc channel_observer.rs && ./channel_observer`
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

// --- Event types ---

#[derive(Clone, Debug)]
struct DeployEvent {
    service: String,
    version: String,
    environment: String,
    status: DeployStatus,
}

#[derive(Clone, Debug, PartialEq)]
enum DeployStatus {
    Started,
    RollingOut,
    Completed,
    Failed(String),
}

// --- Publisher (Subject) ---

struct DeployPublisher {
    subscribers: Vec<Sender<DeployEvent>>,
}

impl DeployPublisher {
    fn new() -> Self {
        DeployPublisher {
            subscribers: Vec::new(),
        }
    }

    /// Create a new subscription. Returns a Receiver the subscriber owns.
    fn subscribe(&mut self) -> Receiver<DeployEvent> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    /// Create a bounded subscription with back-pressure.
    fn subscribe_bounded(&mut self, capacity: usize) -> Receiver<DeployEvent> {
        let (tx, rx) = mpsc::sync_channel(capacity);
        self.subscribers.push(tx);
        rx
    }

    /// Publish an event to all subscribers.
    /// Automatically removes subscribers whose receivers have been dropped.
    fn publish(&mut self, event: DeployEvent) {
        let before = self.subscribers.len();
        self.subscribers
            .retain(|tx| tx.send(event.clone()).is_ok());
        let pruned = before - self.subscribers.len();
        if pruned > 0 {
            println!(
                "  [Publisher] Pruned {} dead subscriber(s), {} remaining",
                pruned,
                self.subscribers.len()
            );
        }
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

// --- Subscriber implementations ---

/// Audit logger: records every deploy event to a log
fn spawn_audit_logger(rx: Receiver<DeployEvent>) -> thread::JoinHandle<Vec<String>> {
    thread::spawn(move || {
        let mut log_entries = Vec::new();
        while let Ok(event) = rx.recv() {
            let entry = format!(
                "[AUDIT] {} v{} -> {} : {:?}",
                event.service, event.version, event.environment, event.status
            );
            println!("{}", entry);
            log_entries.push(entry);
        }
        println!("[AUDIT] Channel closed, logger shutting down");
        log_entries
    })
}

/// Slack alerter: only notifies on failures
fn spawn_slack_alerter(rx: Receiver<DeployEvent>) -> thread::JoinHandle<u32> {
    thread::spawn(move || {
        let mut alert_count = 0u32;
        while let Ok(event) = rx.recv() {
            if let DeployStatus::Failed(reason) = &event.status {
                alert_count += 1;
                println!(
                    "[SLACK] :rotating_light: Deploy FAILED — {} v{} in {}: {}",
                    event.service, event.version, event.environment, reason
                );
            }
        }
        println!("[SLACK] Channel closed, alerter shutting down");
        alert_count
    })
}

/// Metrics collector: counts events by status
fn spawn_metrics_collector(rx: Receiver<DeployEvent>) -> thread::JoinHandle<(u32, u32, u32)> {
    thread::spawn(move || {
        let mut started = 0u32;
        let mut completed = 0u32;
        let mut failed = 0u32;

        while let Ok(event) = rx.recv() {
            match event.status {
                DeployStatus::Started => started += 1,
                DeployStatus::Completed => completed += 1,
                DeployStatus::Failed(_) => failed += 1,
                _ => {} // Ignore intermediate states
            }
            println!(
                "[METRICS] Deploys — started: {}, completed: {}, failed: {}",
                started, completed, failed
            );
        }
        println!("[METRICS] Channel closed, collector shutting down");
        (started, completed, failed)
    })
}

// --- Main ---

fn main() {
    let mut publisher = DeployPublisher::new();

    // Subscribe three observers — each gets its own channel
    let audit_rx = publisher.subscribe();
    let slack_rx = publisher.subscribe();
    let metrics_rx = publisher.subscribe();

    println!(
        "=== {} subscribers registered ===\n",
        publisher.subscriber_count()
    );

    // Spawn observer threads
    let audit_handle = spawn_audit_logger(audit_rx);
    let slack_handle = spawn_slack_alerter(slack_rx);
    let metrics_handle = spawn_metrics_collector(metrics_rx);

    // Simulate deployment events
    let events = vec![
        DeployEvent {
            service: "api-gateway".into(),
            version: "2.4.0".into(),
            environment: "staging".into(),
            status: DeployStatus::Started,
        },
        DeployEvent {
            service: "api-gateway".into(),
            version: "2.4.0".into(),
            environment: "staging".into(),
            status: DeployStatus::RollingOut,
        },
        DeployEvent {
            service: "api-gateway".into(),
            version: "2.4.0".into(),
            environment: "staging".into(),
            status: DeployStatus::Completed,
        },
        DeployEvent {
            service: "payment-service".into(),
            version: "1.8.3".into(),
            environment: "production".into(),
            status: DeployStatus::Started,
        },
        DeployEvent {
            service: "payment-service".into(),
            version: "1.8.3".into(),
            environment: "production".into(),
            status: DeployStatus::Failed("health check timeout after 60s".into()),
        },
    ];

    for event in events {
        publisher.publish(event);
        // Small delay to keep output readable
        thread::sleep(Duration::from_millis(50));
    }

    // Drop the publisher — this closes all sender channels,
    // causing each subscriber's recv() loop to exit
    println!("\n=== Dropping publisher (closing channels) ===\n");
    drop(publisher);

    // Wait for all subscribers to finish and collect results
    let log_entries = audit_handle.join().unwrap();
    let alert_count = slack_handle.join().unwrap();
    let (started, completed, failed) = metrics_handle.join().unwrap();

    // Summary
    println!("\n=== Final Summary ===");
    println!("Audit log entries: {}", log_entries.len());
    println!("Slack alerts sent: {}", alert_count);
    println!(
        "Metrics: {} started, {} completed, {} failed",
        started, completed, failed
    );

    // Demonstrate dead subscriber pruning
    println!("\n=== Dead Subscriber Pruning Demo ===\n");

    let mut publisher2 = DeployPublisher::new();
    let _rx_a = publisher2.subscribe();
    let rx_b = publisher2.subscribe();
    let _rx_c = publisher2.subscribe();

    println!("Before drop: {} subscribers", publisher2.subscriber_count());

    // Drop subscriber B's receiver
    drop(rx_b);

    // Next publish will detect the dead subscriber and prune it
    publisher2.publish(DeployEvent {
        service: "test".into(),
        version: "0.0.1".into(),
        environment: "dev".into(),
        status: DeployStatus::Started,
    });

    println!("After pruning: {} subscribers", publisher2.subscriber_count());
}
