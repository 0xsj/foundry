/// Weak Observer — Using `Weak<T>` references for automatic observer cleanup.
///
/// The subject holds `Weak<Mutex<dyn Observer>>` references. When observers
/// are dropped (all strong references gone), the weak reference returns None
/// on upgrade. The subject automatically prunes dead observers during notification.
///
/// This approach is genuinely better than GC languages, where forgetting to
/// unsubscribe causes memory leaks. Here, the type system enforces cleanup.
///
/// Run: `rustc weak_observer.rs && ./weak_observer`
use std::sync::{Arc, Mutex, Weak};

// --- Observer trait ---

trait Observer: Send {
    /// Handle an incoming event. Receives &mut self so observers can update state.
    fn on_event(&mut self, event: &ConfigEvent);

    /// A human-readable name for this observer (used in logging).
    fn name(&self) -> &str;
}

// --- Event types ---

#[derive(Debug, Clone)]
struct ConfigEvent {
    key: String,
    old_value: Option<String>,
    new_value: String,
    source: String,
}

// --- Subject with weak references ---

struct ConfigWatcher {
    observers: Vec<Weak<Mutex<dyn Observer>>>,
    config: std::collections::HashMap<String, String>,
}

impl ConfigWatcher {
    fn new() -> Self {
        ConfigWatcher {
            observers: Vec::new(),
            config: std::collections::HashMap::new(),
        }
    }

    /// Subscribe an observer. Takes a reference to an Arc — the caller retains
    /// ownership. The subject only holds a Weak reference.
    fn subscribe(&mut self, observer: &Arc<Mutex<dyn Observer>>) {
        let name = observer.lock().unwrap().name().to_string();
        self.observers.push(Arc::downgrade(observer));
        println!("  [ConfigWatcher] Subscribed: {}", name);
    }

    /// Update a config value and notify all live observers.
    /// Dead observers are automatically pruned.
    fn set(&mut self, key: String, value: String) {
        let old_value = self.config.insert(key.clone(), value.clone());
        let event = ConfigEvent {
            key,
            old_value,
            new_value: value,
            source: "api".into(),
        };
        self.notify(&event);
    }

    /// Notify all live observers. Prunes dead ones.
    fn notify(&mut self, event: &ConfigEvent) {
        let before = self.observers.len();

        self.observers.retain(|weak| {
            match weak.upgrade() {
                Some(strong) => {
                    // Observer is alive — deliver the event
                    strong.lock().unwrap().on_event(event);
                    true
                }
                None => {
                    // Observer was dropped — remove from list
                    false
                }
            }
        });

        let pruned = before - self.observers.len();
        if pruned > 0 {
            println!(
                "  [ConfigWatcher] Pruned {} dead observer(s), {} remaining",
                pruned,
                self.observers.len()
            );
        }
    }

    fn observer_count(&self) -> usize {
        // Count only live observers
        self.observers
            .iter()
            .filter(|w| w.strong_count() > 0)
            .count()
    }
}

// --- Concrete observers ---

/// Logs all config changes to stdout (simulating a file logger).
struct AuditLogger {
    entries: Vec<String>,
}

impl AuditLogger {
    fn new() -> Self {
        AuditLogger {
            entries: Vec::new(),
        }
    }

    fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Observer for AuditLogger {
    fn on_event(&mut self, event: &ConfigEvent) {
        let entry = format!(
            "[AUDIT] {} changed from {:?} to {:?} (source: {})",
            event.key, event.old_value, event.new_value, event.source
        );
        println!("    {}", entry);
        self.entries.push(entry);
    }

    fn name(&self) -> &str {
        "AuditLogger"
    }
}

/// Watches for specific config keys and triggers reloads.
struct ServiceReloader {
    watched_keys: Vec<String>,
    reload_count: u32,
}

impl ServiceReloader {
    fn new(watched_keys: Vec<String>) -> Self {
        ServiceReloader {
            watched_keys,
            reload_count: 0,
        }
    }
}

impl Observer for ServiceReloader {
    fn on_event(&mut self, event: &ConfigEvent) {
        if self.watched_keys.contains(&event.key) {
            self.reload_count += 1;
            println!(
                "    [RELOADER] Key '{}' changed — triggering reload #{}",
                event.key, self.reload_count
            );
        }
    }

    fn name(&self) -> &str {
        "ServiceReloader"
    }
}

/// Validates config values and warns on invalid ranges.
struct ConfigValidator {
    warnings: Vec<String>,
}

impl ConfigValidator {
    fn new() -> Self {
        ConfigValidator {
            warnings: Vec::new(),
        }
    }
}

impl Observer for ConfigValidator {
    fn on_event(&mut self, event: &ConfigEvent) {
        // Example validation: warn if timeout values are too low
        if event.key.contains("timeout") {
            if let Ok(ms) = event.new_value.parse::<u64>() {
                if ms < 100 {
                    let warning = format!(
                        "Config '{}' set to {}ms — dangerously low timeout",
                        event.key, ms
                    );
                    println!("    [VALIDATOR] WARNING: {}", warning);
                    self.warnings.push(warning);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "ConfigValidator"
    }
}

// --- Main ---

fn main() {
    let mut watcher = ConfigWatcher::new();

    // Create observers — the caller owns the Arc, watcher gets Weak
    let logger: Arc<Mutex<dyn Observer>> = Arc::new(Mutex::new(AuditLogger::new()));
    let reloader: Arc<Mutex<dyn Observer>> = Arc::new(Mutex::new(ServiceReloader::new(vec![
        "db.connection_string".into(),
        "redis.host".into(),
    ])));
    let validator: Arc<Mutex<dyn Observer>> = Arc::new(Mutex::new(ConfigValidator::new()));

    println!("=== Subscribing observers ===\n");
    watcher.subscribe(&logger);
    watcher.subscribe(&reloader);
    watcher.subscribe(&validator);
    println!("\nLive observers: {}\n", watcher.observer_count());

    // --- Phase 1: All observers alive ---
    println!("=== Phase 1: All observers active ===\n");

    watcher.set("db.connection_string".into(), "postgres://prod:5432/app".into());
    println!();

    watcher.set("api.request_timeout".into(), "5000".into());
    println!();

    watcher.set("redis.host".into(), "redis-cluster.internal:6379".into());
    println!();

    // --- Phase 2: Drop the validator (simulate component shutdown) ---
    println!("=== Phase 2: Dropping ConfigValidator ===\n");

    drop(validator);
    // The Weak reference in watcher now points to nothing.
    // Next notification will detect and prune it.

    println!("Live observers before notify: {}\n", watcher.observer_count());

    watcher.set("api.request_timeout".into(), "50".into());
    println!();

    // Note: the validator would have warned about 50ms timeout,
    // but it has been dropped. No warning printed.

    println!("Live observers after notify: {}\n", watcher.observer_count());

    // --- Phase 3: Drop the reloader too ---
    println!("=== Phase 3: Dropping ServiceReloader ===\n");

    drop(reloader);

    watcher.set("db.connection_string".into(), "postgres://prod:5433/app_v2".into());
    println!();

    println!("Live observers: {}\n", watcher.observer_count());

    // --- Phase 4: Inspect remaining observer state ---
    println!("=== Phase 4: Inspect logger state ===\n");

    // Because we still hold the Arc<Mutex<dyn Observer>>, we can downcast
    // or access the observer. But dyn Observer doesn't let us call
    // AuditLogger-specific methods directly. We need a separate Arc
    // with the concrete type for that.

    // Better approach: keep a concrete-typed Arc alongside the trait object Arc
    let concrete_logger = Arc::new(Mutex::new(AuditLogger::new()));
    let trait_logger: Arc<Mutex<dyn Observer>> = concrete_logger.clone();

    let mut watcher2 = ConfigWatcher::new();
    watcher2.subscribe(&trait_logger);

    watcher2.set("feature.new_ui".into(), "enabled".into());
    println!();

    // Access concrete type through the concrete Arc
    println!(
        "Logger has {} entries",
        concrete_logger.lock().unwrap().entry_count()
    );

    // Drop the concrete reference — trait object Arc still holds it alive
    drop(concrete_logger);
    watcher2.set("feature.new_ui".into(), "disabled".into());
    println!();

    // Drop the last strong reference — now the weak ref in watcher2 dies
    drop(trait_logger);
    watcher2.set("feature.new_ui".into(), "enabled_v2".into());
    println!();

    println!("Final observer count: {}", watcher2.observer_count());
}
