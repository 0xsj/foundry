// Typed Event Bus — Proposed Code for Review
//
// A plugin-system event bus where plugins publish and subscribe to typed events.
// Subscribers register a handler closure and receive events matching their filter.

// ----- Types -----

/// An event that can be published to the bus.
#[derive(Debug, Clone)]
pub enum BusEvent {
    PluginLoaded { plugin_id: String, version: String },
    PluginUnloaded { plugin_id: String },
    ConfigChanged { key: String, old_value: String, new_value: String },
    HealthCheck { plugin_id: String, healthy: bool },
    Shutdown,
}

/// The category of an event, used for subscription filtering.
#[derive(Debug, Clone, PartialEq)]
pub enum EventCategory {
    Lifecycle,
    Config,
    Health,
    System,
}

/// A subscriber registration.
#[derive(Debug)]
pub struct Subscriber {
    // ISSUE: These fields are all pub. External code can freely modify
    // subscriber_id, category, and received_count without going through methods.
    pub subscriber_id: u64,
    pub name: String,
    pub category: EventCategory,
    pub received_count: u64,
}

/// Statistics for the bus.
// ISSUE: This struct has no Default impl, even though all fields start at 0.
// Callers must construct it manually with BusStats { published: 0, delivered: 0, dropped: 0 }.
#[derive(Debug, Clone)]
pub struct BusStats {
    pub published: u64,
    pub delivered: u64,
    pub dropped: u64,
}

/// The event bus itself.
#[derive(Debug)]
pub struct EventBus {
    // ISSUE: all pub — callers can directly push to subscribers or mess with stats
    pub subscribers: Vec<Subscriber>,
    pub stats: BusStats,
    pub next_id: u64,
}

// ----- Implementations -----

impl BusEvent {
    /// Returns the category of this event.
    pub fn category(&self) -> EventCategory {
        match self {
            BusEvent::PluginLoaded { .. } => EventCategory::Lifecycle,
            BusEvent::PluginUnloaded { .. } => EventCategory::Lifecycle,
            BusEvent::ConfigChanged { .. } => EventCategory::Config,
            BusEvent::HealthCheck { .. } => EventCategory::Health,
            BusEvent::Shutdown => EventCategory::System,
        }
    }

    /// Returns a short human-readable name for the event.
    // ISSUE: This method clones plugin_id unnecessarily.
    // It only needs to return a &str describing the event type — no data from
    // the fields is needed except as a label. A static string would suffice.
    pub fn name(&self) -> String {
        match self {
            BusEvent::PluginLoaded { plugin_id, .. } => {
                format!("plugin_loaded({})", plugin_id.clone())
            }
            BusEvent::PluginUnloaded { plugin_id } => {
                format!("plugin_unloaded({})", plugin_id.clone())
            }
            BusEvent::ConfigChanged { key, .. } => {
                format!("config_changed({})", key.clone())
            }
            BusEvent::HealthCheck { plugin_id, .. } => {
                format!("health_check({})", plugin_id.clone())
            }
            BusEvent::Shutdown => String::from("shutdown"),
        }
    }
}

impl Subscriber {
    pub fn new(subscriber_id: u64, name: &str, category: EventCategory) -> Subscriber {
        Subscriber {
            subscriber_id,
            name: name.to_string(),
            category,
            received_count: 0,
        }
    }

    pub fn matches(&self, event: &BusEvent) -> bool {
        self.category == event.category()
    }

    pub fn record_received(&mut self) {
        self.received_count += 1;
    }
}

impl BusStats {
    pub fn new() -> BusStats {
        BusStats {
            published: 0,
            delivered: 0,
            dropped: 0,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "published={} delivered={} dropped={}",
            self.published, self.delivered, self.dropped
        )
    }
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus {
            subscribers: Vec::new(),
            stats: BusStats::new(),
            next_id: 1,
        }
    }

    pub fn subscribe(&mut self, name: &str, category: EventCategory) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers.push(Subscriber::new(id, name, category));
        id
    }

    pub fn unsubscribe(&mut self, subscriber_id: u64) -> bool {
        let before = self.subscribers.len();
        self.subscribers.retain(|s| s.subscriber_id != subscriber_id);
        self.subscribers.len() < before
    }

    /// Publishes an event to all matching subscribers.
    pub fn publish(&mut self, event: &BusEvent) {
        self.stats.published += 1;
        let mut delivered = 0u64;

        for subscriber in self.subscribers.iter_mut() {
            if subscriber.matches(event) {
                subscriber.record_received();
                delivered += 1;
            }
        }

        self.stats.delivered += delivered;
        if delivered == 0 {
            self.stats.dropped += 1;
        }
    }

    /// Returns the subscriber with the given ID, if present.
    // ISSUE: This method uses a match with an empty None arm.
    // `if let` would be more idiomatic.
    pub fn find_subscriber(&self, id: u64) -> Option<&Subscriber> {
        let result = self.subscribers.iter().find(|s| s.subscriber_id == id);
        match result {
            Some(s) => Some(s),
            None => None,
        }
    }

    // ISSUE: subscriber_count, delivered_count, and dropped_count should not
    // need to exist as separate methods if callers could just access bus.stats.
    // But since stats is pub, callers bypass these anyway — making them redundant.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

/// Formats an event for logging purposes.
// ISSUE: Takes EventCategory by value (moves it), but only reads it.
// Should take &EventCategory instead.
pub fn format_for_log(event: &BusEvent, category: EventCategory) -> String {
    format!("[{:?}] {}", category, event.name())
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_and_publish() {
        let mut bus = EventBus::new();
        let id = bus.subscribe("plugin-manager", EventCategory::Lifecycle);
        assert_eq!(id, 1);

        let event = BusEvent::PluginLoaded {
            plugin_id: String::from("auth-plugin"),
            version: String::from("1.0.0"),
        };

        bus.publish(&event);

        assert_eq!(bus.stats.published, 1);
        assert_eq!(bus.stats.delivered, 1);
        assert_eq!(bus.stats.dropped, 0);
    }

    #[test]
    fn test_unmatched_event_is_dropped() {
        let mut bus = EventBus::new();
        bus.subscribe("config-watcher", EventCategory::Config);

        // Publish a lifecycle event — config watcher should not receive it
        bus.publish(&BusEvent::PluginLoaded {
            plugin_id: String::from("my-plugin"),
            version: String::from("2.0"),
        });

        assert_eq!(bus.stats.dropped, 1);
        assert_eq!(bus.stats.delivered, 0);
    }

    #[test]
    fn test_unsubscribe() {
        let mut bus = EventBus::new();
        let id = bus.subscribe("temp", EventCategory::System);
        assert_eq!(bus.subscriber_count(), 1);

        let removed = bus.unsubscribe(id);
        assert!(removed);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_find_subscriber() {
        let mut bus = EventBus::new();
        let id = bus.subscribe("health-monitor", EventCategory::Health);

        let found = bus.find_subscriber(id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "health-monitor");

        let missing = bus.find_subscriber(999);
        assert!(missing.is_none());
    }

    #[test]
    fn test_multiple_subscribers_same_category() {
        let mut bus = EventBus::new();
        bus.subscribe("listener-a", EventCategory::Health);
        bus.subscribe("listener-b", EventCategory::Health);
        bus.subscribe("listener-c", EventCategory::Config);

        let event = BusEvent::HealthCheck {
            plugin_id: String::from("db-plugin"),
            healthy: true,
        };

        bus.publish(&event);

        assert_eq!(bus.stats.delivered, 2);
        assert_eq!(bus.stats.dropped, 0);
    }

    #[test]
    fn test_received_count_increments() {
        let mut bus = EventBus::new();
        let id = bus.subscribe("counter", EventCategory::System);

        bus.publish(&BusEvent::Shutdown);
        bus.publish(&BusEvent::Shutdown);

        let sub = bus.find_subscriber(id).unwrap();
        assert_eq!(sub.received_count, 2);
    }

    #[test]
    fn test_stats_summary() {
        let stats = BusStats {
            published: 10,
            delivered: 8,
            dropped: 2,
        };
        assert_eq!(stats.summary(), "published=10 delivered=8 dropped=2");
    }

    #[test]
    fn test_format_for_log() {
        let event = BusEvent::PluginLoaded {
            plugin_id: String::from("auth"),
            version: String::from("1.0"),
        };
        let log = format_for_log(&event, EventCategory::Lifecycle);
        assert!(log.contains("Lifecycle"));
    }
}

fn main() {
    let mut bus = EventBus::new();

    let _mgr = bus.subscribe("plugin-manager", EventCategory::Lifecycle);
    let _cfg = bus.subscribe("config-watcher", EventCategory::Config);
    let _hlt = bus.subscribe("health-monitor", EventCategory::Health);

    let events = vec![
        BusEvent::PluginLoaded {
            plugin_id: String::from("auth-plugin"),
            version: String::from("1.2.0"),
        },
        BusEvent::ConfigChanged {
            key: String::from("log_level"),
            old_value: String::from("info"),
            new_value: String::from("debug"),
        },
        BusEvent::HealthCheck {
            plugin_id: String::from("auth-plugin"),
            healthy: true,
        },
        BusEvent::Shutdown,
    ];

    for event in &events {
        println!("publishing: {}", event.name());
        bus.publish(event);
    }

    println!("\n{}", bus.stats.summary());
}
