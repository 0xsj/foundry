/// Type-Safe Event Bus — Pub/Sub using enums for compile-time topic verification.
///
/// Each event variant carries its own data. Subscribers use pattern matching
/// to process only the events they care about. No string-based routing —
/// typos are caught at compile time.
///
/// Run: `rustc typed_events.rs && ./typed_events`

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

// --- Typed Topics and Events ---

/// Topics are an enum — adding or removing a topic is a compile-time change.
/// Any match statement over topics that isn't exhaustive produces a warning.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MetricTopic {
    Cpu,
    Memory,
    Disk,
    Network,
}

/// Events carry their own data via enum variants.
/// This replaces stringly-typed payloads with structured, type-safe data.
#[derive(Clone, Debug)]
enum MetricEvent {
    Counter {
        name: String,
        value: u64,
        labels: Vec<(String, String)>,
    },
    Gauge {
        name: String,
        value: f64,
    },
    Histogram {
        name: String,
        value: f64,
        bucket_bounds: Vec<f64>,
    },
}

impl MetricEvent {
    fn name(&self) -> &str {
        match self {
            MetricEvent::Counter { name, .. } => name,
            MetricEvent::Gauge { name, .. } => name,
            MetricEvent::Histogram { name, .. } => name,
        }
    }
}

// --- Typed Broker ---

type SubscriptionId = u64;

struct TypedBrokerInner {
    next_id: SubscriptionId,
    subscribers: HashMap<MetricTopic, Vec<(SubscriptionId, Sender<MetricEvent>)>>,
}

#[derive(Clone)]
struct TypedBroker {
    inner: Arc<Mutex<TypedBrokerInner>>,
}

impl TypedBroker {
    fn new() -> Self {
        TypedBroker {
            inner: Arc::new(Mutex::new(TypedBrokerInner {
                next_id: 0,
                subscribers: HashMap::new(),
            })),
        }
    }

    /// Subscribe to a specific topic. Returns an ID and receiver.
    fn subscribe(&self, topic: MetricTopic) -> (SubscriptionId, Receiver<MetricEvent>) {
        let (tx, rx) = mpsc::channel();
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        inner.subscribers.entry(topic).or_default().push((id, tx));
        (id, rx)
    }

    /// Subscribe to multiple topics at once. Returns one receiver that merges all topics.
    fn subscribe_many(&self, topics: &[MetricTopic]) -> (Vec<SubscriptionId>, Receiver<MetricEvent>) {
        let (tx, rx) = mpsc::channel();
        let mut inner = self.inner.lock().unwrap();
        let mut ids = Vec::with_capacity(topics.len());

        for topic in topics {
            let id = inner.next_id;
            inner.next_id += 1;
            inner
                .subscribers
                .entry(topic.clone())
                .or_default()
                .push((id, tx.clone()));
            ids.push(id);
        }

        (ids, rx)
    }

    /// Publish an event to a topic. Prunes disconnected subscribers.
    fn publish(&self, topic: MetricTopic, event: MetricEvent) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(subs) = inner.subscribers.get_mut(&topic) {
            subs.retain(|(_, tx)| tx.send(event.clone()).is_ok());
        }
    }

    /// Unsubscribe by ID from a specific topic.
    fn unsubscribe(&self, topic: &MetricTopic, id: SubscriptionId) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(subs) = inner.subscribers.get_mut(topic) {
            subs.retain(|(sub_id, _)| *sub_id != id);
        }
    }

    fn topic_stats(&self) -> Vec<(MetricTopic, usize)> {
        let inner = self.inner.lock().unwrap();
        let mut stats: Vec<_> = inner
            .subscribers
            .iter()
            .map(|(topic, subs)| (topic.clone(), subs.len()))
            .collect();
        stats.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        stats
    }
}

// --- Subscriber Processors ---

/// Aggregates gauge values, computing running average.
struct GaugeAggregator {
    name: String,
    values: Vec<f64>,
}

impl GaugeAggregator {
    fn new(name: &str) -> Self {
        GaugeAggregator {
            name: name.to_string(),
            values: Vec::new(),
        }
    }

    fn process(&mut self, event: &MetricEvent) {
        if let MetricEvent::Gauge { name, value } = event {
            self.values.push(*value);
            let avg: f64 = self.values.iter().sum::<f64>() / self.values.len() as f64;
            println!(
                "  [{}] Gauge '{}' = {:.1} (avg: {:.1}, samples: {})",
                self.name,
                name,
                value,
                avg,
                self.values.len()
            );
        }
    }
}

/// Watches counter values and triggers alerts above threshold.
struct ThresholdAlerter {
    name: String,
    threshold: u64,
    alerts_fired: u32,
}

impl ThresholdAlerter {
    fn new(name: &str, threshold: u64) -> Self {
        ThresholdAlerter {
            name: name.to_string(),
            threshold,
            alerts_fired: 0,
        }
    }

    fn process(&mut self, event: &MetricEvent) {
        if let MetricEvent::Counter { name, value, labels } = event {
            if *value > self.threshold {
                self.alerts_fired += 1;
                let label_str: String = labels
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "  [{}] ALERT #{}: counter '{}' = {} > {} [{}]",
                    self.name, self.alerts_fired, name, value, self.threshold, label_str
                );
            }
        }
    }
}

// --- Demo ---

fn main() {
    let broker = TypedBroker::new();

    // --- Subscriber 1: CPU gauge aggregator ---
    let (_id1, rx1) = broker.subscribe(MetricTopic::Cpu);
    let h1 = std::thread::spawn(move || {
        let mut agg = GaugeAggregator::new("CpuAggregator");
        while let Ok(event) = rx1.recv() {
            agg.process(&event);
        }
        println!("  [CpuAggregator] Shut down. {} samples collected.", agg.values.len());
    });

    // --- Subscriber 2: Memory gauge aggregator ---
    let (_id2, rx2) = broker.subscribe(MetricTopic::Memory);
    let h2 = std::thread::spawn(move || {
        let mut agg = GaugeAggregator::new("MemAggregator");
        while let Ok(event) = rx2.recv() {
            agg.process(&event);
        }
        println!("  [MemAggregator] Shut down. {} samples collected.", agg.values.len());
    });

    // --- Subscriber 3: Multi-topic alert system (CPU + Network) ---
    let (_ids3, rx3) = broker.subscribe_many(&[MetricTopic::Cpu, MetricTopic::Network]);
    let h3 = std::thread::spawn(move || {
        let mut alerter = ThresholdAlerter::new("MultiAlerter", 1000);
        while let Ok(event) = rx3.recv() {
            alerter.process(&event);
        }
        println!(
            "  [MultiAlerter] Shut down. {} alerts fired.",
            alerter.alerts_fired
        );
    });

    // --- Subscriber 4: Universal audit logger (all topics) ---
    let all_topics = vec![
        MetricTopic::Cpu,
        MetricTopic::Memory,
        MetricTopic::Disk,
        MetricTopic::Network,
    ];
    let (_ids4, rx4) = broker.subscribe_many(&all_topics);
    let h4 = std::thread::spawn(move || {
        let mut count = 0u64;
        while let Ok(event) = rx4.recv() {
            count += 1;
            println!("  [Audit #{}] {:?}", count, event);
        }
        println!("  [Audit] Shut down. {} events logged.", count);
    });

    // Give threads a moment to start
    std::thread::sleep(std::time::Duration::from_millis(10));

    println!("=== Topic stats: {:?} ===\n", broker.topic_stats());

    // --- Publish typed events ---

    println!("--- Publishing CPU events ---");
    broker.publish(
        MetricTopic::Cpu,
        MetricEvent::Gauge {
            name: "cpu.usage".into(),
            value: 45.2,
        },
    );
    broker.publish(
        MetricTopic::Cpu,
        MetricEvent::Gauge {
            name: "cpu.usage".into(),
            value: 78.9,
        },
    );
    // Counter event on CPU topic — alerter will check threshold
    broker.publish(
        MetricTopic::Cpu,
        MetricEvent::Counter {
            name: "cpu.context_switches".into(),
            value: 1500,
            labels: vec![("host".into(), "web-01".into())],
        },
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    println!("\n--- Publishing Memory events ---");
    broker.publish(
        MetricTopic::Memory,
        MetricEvent::Gauge {
            name: "mem.heap_used".into(),
            value: 2048.0,
        },
    );
    broker.publish(
        MetricTopic::Memory,
        MetricEvent::Gauge {
            name: "mem.heap_used".into(),
            value: 2560.0,
        },
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    println!("\n--- Publishing Network events ---");
    broker.publish(
        MetricTopic::Network,
        MetricEvent::Counter {
            name: "net.packets_dropped".into(),
            value: 2000,
            labels: vec![
                ("interface".into(), "eth0".into()),
                ("direction".into(), "inbound".into()),
            ],
        },
    );
    broker.publish(
        MetricTopic::Network,
        MetricEvent::Histogram {
            name: "net.latency_ms".into(),
            value: 45.0,
            bucket_bounds: vec![10.0, 25.0, 50.0, 100.0, 250.0],
        },
    );

    // Disk events — only the audit logger is subscribed
    println!("\n--- Publishing Disk events ---");
    broker.publish(
        MetricTopic::Disk,
        MetricEvent::Gauge {
            name: "disk.usage_pct".into(),
            value: 87.3,
        },
    );

    // Let all events flow through
    std::thread::sleep(std::time::Duration::from_millis(50));

    // --- Compile-time safety demo ---
    // Uncomment the following line to see the compiler error:
    // broker.publish(MetricTopic::Database, MetricEvent::Gauge { ... });
    // error[E0599]: no variant named `Database` found for enum `MetricTopic`
    //
    // This is the advantage of enum topics over string topics:
    // typos and invalid topics are caught at compile time.

    println!("\n--- Shutting down ---\n");
    drop(broker);

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
    h4.join().unwrap();

    println!("\n=== All subscribers shut down cleanly ===");
}
