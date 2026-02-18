/// Health Check Monitor — Observer Pattern Exercise (Starter)
///
/// Implement the observer pattern for a health check monitoring system.
/// Follow the TODOs to complete the implementation.
///
/// Run: `rustc main.rs && ./main`
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- Event Types ---

#[derive(Debug, Clone, PartialEq)]
enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone)]
struct HealthEvent {
    service_name: String,
    status: HealthStatus,
    latency_ms: u64,
    timestamp: u64,
    message: Option<String>,
}

// --- Observer Trait ---

trait Observer: Send {
    /// Handle a health event. Receives &mut self for state updates.
    fn on_health_event(&mut self, event: &HealthEvent);

    /// Return a unique name for this observer (used for unsubscribe).
    fn name(&self) -> &str;
}

// --- Health Monitor (Subject) ---

struct HealthMonitor {
    observers: Vec<Arc<Mutex<dyn Observer>>>,
}

impl HealthMonitor {
    fn new() -> Self {
        // TODO: Initialize the monitor
        todo!("Create HealthMonitor")
    }

    /// Add an observer. The caller retains an Arc clone.
    fn subscribe(&mut self, observer: Arc<Mutex<dyn Observer>>) {
        // TODO: Add the observer to the list
        todo!("Implement subscribe")
    }

    /// Remove an observer by its name.
    /// Returns true if an observer was removed, false if not found.
    fn unsubscribe(&mut self, name: &str) -> bool {
        // TODO: Find and remove the observer with the matching name.
        // Hint: Use retain() and lock each observer to check name().
        // Be careful with lock scope.
        todo!("Implement unsubscribe")
    }

    /// Notify all observers of a health event.
    fn notify_all(&self, event: &HealthEvent) {
        // TODO: Iterate through all observers and call on_health_event.
        // Lock each observer's mutex to get &mut access.
        todo!("Implement notify_all")
    }

    fn observer_count(&self) -> usize {
        self.observers.len()
    }
}

// --- Alert Observer ---

/// Tracks consecutive unhealthy checks per service.
/// Fires an alert when the consecutive count reaches the threshold.
struct AlertObserver {
    threshold: u32,
    consecutive_failures: HashMap<String, u32>,
    alerts_fired: u32,
}

impl AlertObserver {
    fn new(threshold: u32) -> Self {
        // TODO: Initialize AlertObserver
        todo!("Create AlertObserver")
    }

    fn alerts_fired(&self) -> u32 {
        self.alerts_fired
    }
}

impl Observer for AlertObserver {
    fn on_health_event(&mut self, event: &HealthEvent) {
        // TODO: Implement alert logic
        // - If status is Healthy, reset the consecutive failure count for this service
        // - If status is Unhealthy, increment the consecutive failure count
        //   - If count reaches threshold, print alert and increment alerts_fired
        // - Degraded can be treated as a warning (optional: print but don't count)
        todo!("Implement AlertObserver on_health_event")
    }

    fn name(&self) -> &str {
        "AlertObserver"
    }
}

// --- Dashboard Observer ---

/// Maintains the latest health status for each service.
struct DashboardObserver {
    service_status: HashMap<String, HealthStatus>,
}

impl DashboardObserver {
    fn new() -> Self {
        // TODO: Initialize DashboardObserver
        todo!("Create DashboardObserver")
    }

    fn get_status(&self, service: &str) -> Option<&HealthStatus> {
        self.service_status.get(service)
    }

    fn all_statuses(&self) -> &HashMap<String, HealthStatus> {
        &self.service_status
    }
}

impl Observer for DashboardObserver {
    fn on_health_event(&mut self, event: &HealthEvent) {
        // TODO: Update the latest status for this service
        todo!("Implement DashboardObserver on_health_event")
    }

    fn name(&self) -> &str {
        "DashboardObserver"
    }
}

// --- Metrics Observer ---

/// Counts events by status and tracks average latency per service.
struct MetricsObserver {
    status_counts: HashMap<String, u64>, // "healthy" -> count, "degraded" -> count, etc.
    service_latencies: HashMap<String, Vec<u64>>, // service -> list of latency samples
    total_events: u64,
}

impl MetricsObserver {
    fn new() -> Self {
        // TODO: Initialize MetricsObserver
        todo!("Create MetricsObserver")
    }

    fn total_events(&self) -> u64 {
        self.total_events
    }

    fn average_latency(&self, service: &str) -> Option<f64> {
        self.service_latencies.get(service).map(|latencies| {
            if latencies.is_empty() {
                0.0
            } else {
                latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
            }
        })
    }

    fn status_count(&self, status: &str) -> u64 {
        *self.status_counts.get(status).unwrap_or(&0)
    }
}

impl Observer for MetricsObserver {
    fn on_health_event(&mut self, event: &HealthEvent) {
        // TODO: Implement metrics collection
        // - Increment total_events
        // - Increment the count for the event's status category
        //   (use lowercase status name as key: "healthy", "degraded", "unhealthy")
        // - Record the latency sample for this service
        todo!("Implement MetricsObserver on_health_event")
    }

    fn name(&self) -> &str {
        "MetricsObserver"
    }
}

// --- Main ---

fn main() {
    let mut monitor = HealthMonitor::new();

    // Create observers — keep Arc clones so we can inspect state afterward
    let alert_obs = Arc::new(Mutex::new(AlertObserver::new(3)));
    let dashboard_obs = Arc::new(Mutex::new(DashboardObserver::new()));
    let metrics_obs = Arc::new(Mutex::new(MetricsObserver::new()));

    // Subscribe
    monitor.subscribe(alert_obs.clone());
    monitor.subscribe(dashboard_obs.clone());
    monitor.subscribe(metrics_obs.clone());

    println!("Observers registered: {}\n", monitor.observer_count());

    // Simulate health check events
    let events = vec![
        HealthEvent {
            service_name: "api-gateway".into(),
            status: HealthStatus::Healthy,
            latency_ms: 45,
            timestamp: 1000,
            message: None,
        },
        HealthEvent {
            service_name: "payment-service".into(),
            status: HealthStatus::Unhealthy,
            latency_ms: 5000,
            timestamp: 1030,
            message: Some("Connection timeout".into()),
        },
        HealthEvent {
            service_name: "payment-service".into(),
            status: HealthStatus::Unhealthy,
            latency_ms: 5000,
            timestamp: 1060,
            message: Some("Connection timeout".into()),
        },
        HealthEvent {
            service_name: "payment-service".into(),
            status: HealthStatus::Unhealthy,
            latency_ms: 5000,
            timestamp: 1090,
            message: Some("Connection timeout".into()),
        },
        HealthEvent {
            service_name: "api-gateway".into(),
            status: HealthStatus::Degraded,
            latency_ms: 800,
            timestamp: 1120,
            message: Some("High latency".into()),
        },
        HealthEvent {
            service_name: "payment-service".into(),
            status: HealthStatus::Healthy,
            latency_ms: 120,
            timestamp: 1150,
            message: None,
        },
    ];

    for event in &events {
        println!("--- Event: {} -> {:?} ---", event.service_name, event.status);
        monitor.notify_all(event);
        println!();
    }

    // Inspect observer state
    println!("=== Final State ===\n");

    let alert = alert_obs.lock().unwrap();
    println!("Alerts fired: {}", alert.alerts_fired());
    drop(alert);

    let dashboard = dashboard_obs.lock().unwrap();
    println!("Dashboard:");
    for (service, status) in dashboard.all_statuses() {
        println!("  {} -> {:?}", service, status);
    }
    drop(dashboard);

    let metrics = metrics_obs.lock().unwrap();
    println!("Total events: {}", metrics.total_events());
    println!("Healthy count: {}", metrics.status_count("healthy"));
    println!("Unhealthy count: {}", metrics.status_count("unhealthy"));
    println!(
        "Avg latency (api-gateway): {:.1}ms",
        metrics.average_latency("api-gateway").unwrap_or(0.0)
    );
    println!(
        "Avg latency (payment-service): {:.1}ms",
        metrics.average_latency("payment-service").unwrap_or(0.0)
    );
    drop(metrics);

    // Test unsubscribe
    println!("\n=== Unsubscribe Test ===\n");
    println!("Before unsubscribe: {} observers", monitor.observer_count());
    let removed = monitor.unsubscribe("AlertObserver");
    println!("Removed AlertObserver: {}", removed);
    println!("After unsubscribe: {} observers", monitor.observer_count());

    // Notify again — AlertObserver should not receive this
    monitor.notify_all(&HealthEvent {
        service_name: "payment-service".into(),
        status: HealthStatus::Unhealthy,
        latency_ms: 3000,
        timestamp: 1180,
        message: Some("Still failing".into()),
    });

    // Unsubscribe non-existent observer
    let removed = monitor.unsubscribe("NonExistent");
    println!("Removed NonExistent: {}", removed);
}
