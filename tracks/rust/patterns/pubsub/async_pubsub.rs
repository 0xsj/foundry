/// Async Pub/Sub — Fan-out to multiple subscribers using threads and bounded channels.
///
/// Demonstrates backpressure via `sync_channel`, fan-out message delivery,
/// graceful shutdown via Drop, and subscriber health monitoring.
///
/// Run: `rustc async_pubsub.rs && ./async_pubsub`

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// --- Types ---

type Topic = String;
type SubscriptionId = u64;

#[derive(Clone, Debug)]
struct Event {
    topic: Topic,
    payload: String,
    sequence: u64,
}

#[derive(Debug, Clone)]
struct PublishResult {
    delivered: usize,
    dropped_full: usize,
    dropped_disconnected: usize,
}

// --- Subscriber metadata ---

struct SubscriberEntry {
    id: SubscriptionId,
    sender: SyncSender<Arc<Event>>,
    name: String,
    created_at: Instant,
    messages_sent: u64,
    messages_dropped: u64,
}

// --- Broker ---

struct BrokerState {
    next_id: SubscriptionId,
    next_seq: u64,
    subscribers: HashMap<Topic, Vec<SubscriberEntry>>,
}

/// A Pub/Sub broker with bounded channels for backpressure.
///
/// Messages are wrapped in `Arc<Event>` so that fan-out to N subscribers
/// only allocates the message once, regardless of subscriber count.
struct AsyncBroker {
    state: Arc<Mutex<BrokerState>>,
    handles: Arc<Mutex<Vec<(SubscriptionId, Option<thread::JoinHandle<()>>)>>>,
}

impl AsyncBroker {
    fn new() -> Self {
        AsyncBroker {
            state: Arc::new(Mutex::new(BrokerState {
                next_id: 0,
                next_seq: 0,
                subscribers: HashMap::new(),
            })),
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Subscribe to a topic with a bounded channel.
    ///
    /// `buffer_size` controls backpressure: when the buffer is full,
    /// messages to this subscriber are dropped (not blocking the publisher).
    fn subscribe<F>(
        &self,
        topic: &str,
        name: &str,
        buffer_size: usize,
        handler: F,
    ) -> SubscriptionId
    where
        F: FnMut(Arc<Event>) + Send + 'static,
    {
        let (tx, rx) = mpsc::sync_channel::<Arc<Event>>(buffer_size);

        let mut state = self.state.lock().unwrap();
        let id = state.next_id;
        state.next_id += 1;

        state
            .subscribers
            .entry(topic.to_string())
            .or_default()
            .push(SubscriberEntry {
                id,
                sender: tx,
                name: name.to_string(),
                created_at: Instant::now(),
                messages_sent: 0,
                messages_dropped: 0,
            });

        // Spawn a worker thread for this subscriber
        let sub_name = name.to_string();
        let handle = thread::spawn(move || {
            Self::subscriber_loop(sub_name, rx, handler);
        });

        self.handles.lock().unwrap().push((id, Some(handle)));

        id
    }

    /// The subscriber's event processing loop.
    fn subscriber_loop<F>(name: String, rx: Receiver<Arc<Event>>, mut handler: F)
    where
        F: FnMut(Arc<Event>),
    {
        let mut processed = 0u64;
        while let Ok(event) = rx.recv() {
            processed += 1;
            handler(event);
        }
        println!(
            "  [{}] Receiver closed after processing {} events.",
            name, processed
        );
    }

    /// Publish an event to a topic. Uses `try_send` — never blocks the publisher.
    ///
    /// Returns statistics about delivery: how many received, how many were dropped.
    fn publish(&self, topic: &str, payload: String) -> PublishResult {
        let mut state = self.state.lock().unwrap();
        state.next_seq += 1;
        let seq = state.next_seq;

        let event = Arc::new(Event {
            topic: topic.to_string(),
            payload,
            sequence: seq,
        });

        let mut delivered = 0usize;
        let mut dropped_full = 0usize;
        let mut dropped_disconnected = 0usize;

        if let Some(subs) = state.subscribers.get_mut(topic) {
            subs.retain_mut(|entry| {
                match entry.sender.try_send(event.clone()) {
                    Ok(()) => {
                        entry.messages_sent += 1;
                        delivered += 1;
                        true // keep subscriber
                    }
                    Err(TrySendError::Full(_)) => {
                        entry.messages_dropped += 1;
                        dropped_full += 1;
                        true // keep subscriber — it's just slow
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        dropped_disconnected += 1;
                        false // remove — subscriber is gone
                    }
                }
            });
        }

        PublishResult {
            delivered,
            dropped_full,
            dropped_disconnected,
        }
    }

    /// Get health stats for all subscribers across all topics.
    fn subscriber_health(&self) -> Vec<(String, String, u64, u64, Duration)> {
        let state = self.state.lock().unwrap();
        let mut stats = Vec::new();

        for (topic, subs) in &state.subscribers {
            for entry in subs {
                stats.push((
                    topic.clone(),
                    entry.name.clone(),
                    entry.messages_sent,
                    entry.messages_dropped,
                    entry.created_at.elapsed(),
                ));
            }
        }

        stats
    }

    /// Graceful shutdown: drop all senders, then join all subscriber threads.
    fn shutdown(self) {
        // Step 1: Clear all senders from broker state.
        // This drops the SyncSender ends, causing Receiver::recv to return Err.
        {
            let mut state = self.state.lock().unwrap();
            state.subscribers.clear();
        }

        // Step 2: Join all subscriber threads.
        let mut handles = self.handles.lock().unwrap();
        for (_id, handle) in handles.iter_mut() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
    }
}

// --- Demo ---

fn main() {
    let broker = AsyncBroker::new();

    // --- Subscriber 1: Fast processor (large buffer) ---
    broker.subscribe("events.orders", "OrderProcessor", 100, |event| {
        println!(
            "  [OrderProcessor] seq={} {}",
            event.sequence, event.payload
        );
    });

    // --- Subscriber 2: Slow processor (small buffer — will experience backpressure) ---
    broker.subscribe("events.orders", "SlowAuditor", 2, |event| {
        // Simulate slow processing
        thread::sleep(Duration::from_millis(50));
        println!(
            "  [SlowAuditor]    seq={} {} (slow)",
            event.sequence, event.payload
        );
    });

    // --- Subscriber 3: Payment processor on a different topic ---
    broker.subscribe("events.payments", "PaymentProcessor", 50, |event| {
        println!(
            "  [PaymentProcessor] seq={} {}",
            event.sequence, event.payload
        );
    });

    // --- Subscriber 4: Analytics on both topics ---
    // Subscribe to orders
    broker.subscribe("events.orders", "Analytics", 20, |event| {
        println!(
            "  [Analytics]      seq={} [{}] {}",
            event.sequence, event.topic, event.payload
        );
    });
    // Subscribe to payments separately (same name, different topic subscription)
    broker.subscribe("events.payments", "Analytics", 20, |event| {
        println!(
            "  [Analytics]      seq={} [{}] {}",
            event.sequence, event.topic, event.payload
        );
    });

    // Give subscribers time to start
    thread::sleep(Duration::from_millis(10));

    // --- Publish a burst of order events ---
    println!("=== Publishing 10 order events rapidly ===\n");
    for i in 1..=10 {
        let result = broker.publish(
            "events.orders",
            format!("order-{:04} placed (item_count={})", i, i * 3),
        );
        println!(
            "  Publish order-{:04}: delivered={}, dropped_full={}, dropped_disconnected={}",
            i, result.delivered, result.dropped_full, result.dropped_disconnected
        );
    }

    // --- Publish some payment events ---
    println!("\n=== Publishing 3 payment events ===\n");
    for i in 1..=3 {
        let result = broker.publish(
            "events.payments",
            format!("payment-{:04} processed (${})", i, i * 99),
        );
        println!(
            "  Publish payment-{:04}: delivered={}, dropped_full={}, dropped_disconnected={}",
            i, result.delivered, result.dropped_full, result.dropped_disconnected
        );
    }

    // Wait for processing
    thread::sleep(Duration::from_millis(200));

    // --- Health check ---
    println!("\n=== Subscriber Health ===\n");
    for (topic, name, sent, dropped, uptime) in broker.subscriber_health() {
        println!(
            "  [{topic}] {name}: sent={sent}, dropped={dropped}, uptime={:.1}s",
            uptime.as_secs_f64()
        );
    }

    // --- Demonstrate that publishing to unknown topic is harmless ---
    println!("\n=== Publishing to non-existent topic ===");
    let result = broker.publish("events.unknown", "this goes nowhere".into());
    println!(
        "  Result: delivered={}, dropped_full={}, dropped_disconnected={}",
        result.delivered, result.dropped_full, result.dropped_disconnected
    );

    // --- Graceful shutdown ---
    println!("\n=== Graceful shutdown ===\n");
    broker.shutdown();

    println!("\n=== All subscriber threads joined. Shutdown complete. ===");
}
