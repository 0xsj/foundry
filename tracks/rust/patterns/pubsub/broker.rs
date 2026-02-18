/// In-Process Message Broker — Pub/Sub pattern with topic-based routing.
///
/// A thread-safe broker using `Arc<Mutex<>>`. Publishers emit messages to topics.
/// Subscribers receive messages through `mpsc::Receiver`. Dead subscribers
/// are automatically pruned when their channels disconnect.
///
/// Run: `rustc broker.rs && ./broker`

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// --- Types ---

type Topic = String;
type SubscriptionId = u64;

#[derive(Clone, Debug)]
struct Message {
    topic: Topic,
    payload: String,
    timestamp: u64,
}

// --- Broker ---

struct BrokerInner {
    next_id: SubscriptionId,
    subscribers: HashMap<Topic, Vec<(SubscriptionId, Sender<Message>)>>,
}

/// Thread-safe message broker. Clone the Arc to share across threads.
#[derive(Clone)]
struct Broker {
    inner: Arc<Mutex<BrokerInner>>,
}

impl Broker {
    fn new() -> Self {
        Broker {
            inner: Arc::new(Mutex::new(BrokerInner {
                next_id: 0,
                subscribers: HashMap::new(),
            })),
        }
    }

    /// Subscribe to a topic. Returns a subscription ID and a Receiver for messages.
    fn subscribe(&self, topic: &str) -> (SubscriptionId, Receiver<Message>) {
        let (tx, rx) = mpsc::channel();
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        inner
            .subscribers
            .entry(topic.to_string())
            .or_default()
            .push((id, tx));
        (id, rx)
    }

    /// Publish a message to a topic. Automatically prunes dead subscribers.
    fn publish(&self, topic: &str, payload: String) {
        let mut inner = self.inner.lock().unwrap();

        // Simple incrementing timestamp for demo purposes
        static mut COUNTER: u64 = 0;
        let ts = unsafe {
            COUNTER += 1;
            COUNTER
        };

        let msg = Message {
            topic: topic.to_string(),
            payload,
            timestamp: ts,
        };

        if let Some(subs) = inner.subscribers.get_mut(topic) {
            // retain only live subscribers
            subs.retain(|(_, tx)| tx.send(msg.clone()).is_ok());
        }
    }

    /// Unsubscribe by ID. Returns true if the subscription was found and removed.
    fn unsubscribe(&self, topic: &str, id: SubscriptionId) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(subs) = inner.subscribers.get_mut(topic) {
            let before = subs.len();
            subs.retain(|(sub_id, _)| *sub_id != id);
            subs.len() < before
        } else {
            false
        }
    }

    /// Return subscriber counts per topic.
    fn stats(&self) -> HashMap<String, usize> {
        let inner = self.inner.lock().unwrap();
        inner
            .subscribers
            .iter()
            .map(|(topic, subs)| (topic.clone(), subs.len()))
            .collect()
    }
}

// --- Demo ---

fn main() {
    let broker = Broker::new();

    // --- Subscriber 1: Dashboard aggregator for CPU metrics ---
    let (sub1_id, rx1) = broker.subscribe("metrics.cpu");
    let handle1 = thread::spawn(move || {
        let mut readings = Vec::new();
        while let Ok(msg) = rx1.recv() {
            readings.push(msg.payload.clone());
            println!(
                "  [Dashboard] CPU metric #{}: {}",
                readings.len(),
                msg.payload
            );
        }
        println!("  [Dashboard] Channel closed. Total readings: {}", readings.len());
        readings
    });

    // --- Subscriber 2: Alert system for CPU metrics ---
    let (_sub2_id, rx2) = broker.subscribe("metrics.cpu");
    let handle2 = thread::spawn(move || {
        let mut alert_count = 0u32;
        while let Ok(msg) = rx2.recv() {
            // Parse the value and check threshold
            if let Some(val_str) = msg.payload.strip_prefix("usage=") {
                if let Ok(val) = val_str.parse::<f64>() {
                    if val > 80.0 {
                        alert_count += 1;
                        println!(
                            "  [Alerter] HIGH CPU: {}% (alert #{})",
                            val, alert_count
                        );
                    }
                }
            }
        }
        println!("  [Alerter] Channel closed. Total alerts: {}", alert_count);
    });

    // --- Subscriber 3: Audit logger for disk metrics ---
    let (_sub3_id, rx3) = broker.subscribe("metrics.disk");
    let handle3 = thread::spawn(move || {
        while let Ok(msg) = rx3.recv() {
            println!(
                "  [Audit] [ts={}] {}: {}",
                msg.timestamp, msg.topic, msg.payload
            );
        }
        println!("  [Audit] Channel closed.");
    });

    // Give subscribers a moment to start
    thread::sleep(Duration::from_millis(10));

    println!("=== Broker stats: {:?} ===\n", broker.stats());

    // --- Publisher: emit metric events ---
    println!("--- Publishing CPU metrics ---");
    broker.publish("metrics.cpu", "usage=45.2".into());
    broker.publish("metrics.cpu", "usage=67.8".into());
    broker.publish("metrics.cpu", "usage=92.1".into());
    broker.publish("metrics.cpu", "usage=55.3".into());

    println!("--- Publishing disk metrics ---");
    broker.publish("metrics.disk", "iops=1200".into());
    broker.publish("metrics.disk", "iops=3400".into());

    // Publishing to a topic with no subscribers — silently dropped
    broker.publish("metrics.network", "bandwidth=500mbps".into());

    // Let subscribers process
    thread::sleep(Duration::from_millis(50));

    // --- Unsubscribe dashboard from CPU metrics ---
    println!("\n--- Unsubscribing dashboard (id={}) ---\n", sub1_id);
    broker.unsubscribe("metrics.cpu", sub1_id);

    // This message only reaches the alerter, not the dashboard
    broker.publish("metrics.cpu", "usage=88.0".into());

    thread::sleep(Duration::from_millis(50));

    println!("\n--- Shutting down broker (dropping all senders) ---\n");
    // Drop the broker — all Senders are dropped, closing all channels
    drop(broker);

    // Wait for all subscriber threads to finish
    let readings = handle1.join().expect("dashboard thread panicked");
    handle2.join().expect("alerter thread panicked");
    handle3.join().expect("audit thread panicked");

    println!("\n=== Shutdown complete. Dashboard collected {} readings ===", readings.len());
}
