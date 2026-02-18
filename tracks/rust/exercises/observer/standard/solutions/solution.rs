/// Health Check Monitor — Observer Pattern Exercise (Reference Solution)
///
/// This solution uses `Arc<Mutex<dyn Observer>>` to allow the subject (HealthMonitor)
/// and the caller to share access to each observer. Observers implement a trait
/// with `&mut self` receivers for mutable state updates during notification.
///
/// Run: `rustc solution.rs && ./solution`
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- Event Types ---

#[derive(Debug, Clone, PartialEq)]
enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    fn as_str(&self) -> &str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
        }
    }
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
    fn on_health_event(&mut self, event: &HealthEvent);
    fn name(&self) -> &str;
}

// --- Health Monitor (Subject) ---

struct HealthMonitor {
    observers: Vec<Arc<Mutex<dyn Observer>>>,
}

impl HealthMonitor {
    fn new() -> Self {
        HealthMonitor {
            observers: Vec::new(),
        }
    }

    fn subscribe(&mut self, observer: Arc<Mutex<dyn Observer>>) {
        self.observers.push(observer);
    }

    fn unsubscribe(&mut self, name: &str) -> bool {
        let initial_len = self.observers.len();

        // We need to lock each observer to check its name, but we must not
        // hold locks across the retain call. Collect indices to remove first.
        let to_remove: Vec<usize> = self
            .observers
            .iter()
            .enumerate()
            .filter(|(_, obs)| {
                let guard = obs.lock().unwrap();
                guard.name() == name
            })
            .map(|(i, _)| i)
            .collect();

        // Remove in reverse order to maintain valid indices
        for &i in to_remove.iter().rev() {
            self.observers.remove(i);
        }

        self.observers.len() < initial_len
    }

    fn notify_all(&self, event: &HealthEvent) {
        for observer in &self.observers {
            let mut guard = observer.lock().unwrap();
            guard.on_health_event(event);
        }
    }

    fn observer_count(&self) -> usize {
        self.observers.len()
    }
}

// --- Alert Observer ---

struct AlertObserver {
    threshold: u32,
    consecutive_failures: HashMap<String, u32>,
    alerts_fired: u32,
}

impl AlertObserver {
    fn new(threshold: u32) -> Self {
        AlertObserver {
            threshold,
            consecutive_failures: HashMap::new(),
            alerts_fired: 0,
        }
    }

    fn alerts_fired(&self) -> u32 {
        self.alerts_fired
    }
}

impl Observer for AlertObserver {
    fn on_health_event(&mut self, event: &HealthEvent) {
        match event.status {
            HealthStatus::Healthy => {
                // Reset consecutive failures for this service
                self.consecutive_failures.remove(&event.service_name);
            }
            HealthStatus::Degraded => {
                // Degraded is a warning — log but don't count toward failure threshold
                println!(
                    "  [Alert] WARNING: {} is degraded — {}",
                    event.service_name,
                    event.message.as_deref().unwrap_or("no details")
                );
            }
            HealthStatus::Unhealthy => {
                let count = self
                    .consecutive_failures
                    .entry(event.service_name.clone())
                    .or_insert(0);
                *count += 1;

                if *count >= self.threshold {
                    self.alerts_fired += 1;
                    println!(
                        "  [Alert] CRITICAL: {} has been unhealthy for {} consecutive checks! \
                         Alert #{} — {}",
                        event.service_name,
                        count,
                        self.alerts_fired,
                        event.message.as_deref().unwrap_or("no details")
                    );
                } else {
                    println!(
                        "  [Alert] {} unhealthy ({}/{} before alert) — {}",
                        event.service_name,
                        count,
                        self.threshold,
                        event.message.as_deref().unwrap_or("no details")
                    );
                }
            }
        }
    }

    fn name(&self) -> &str {
        "AlertObserver"
    }
}

// --- Dashboard Observer ---

struct DashboardObserver {
    service_status: HashMap<String, HealthStatus>,
}

impl DashboardObserver {
    fn new() -> Self {
        DashboardObserver {
            service_status: HashMap::new(),
        }
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
        let previous = self
            .service_status
            .insert(event.service_name.clone(), event.status.clone());

        match previous {
            Some(ref prev) if *prev != event.status => {
                println!(
                    "  [Dashboard] {} status changed: {:?} -> {:?}",
                    event.service_name, prev, event.status
                );
            }
            None => {
                println!(
                    "  [Dashboard] {} initial status: {:?}",
                    event.service_name, event.status
                );
            }
            _ => {
                // Status unchanged — no output
            }
        }
    }

    fn name(&self) -> &str {
        "DashboardObserver"
    }
}

// --- Metrics Observer ---

struct MetricsObserver {
    status_counts: HashMap<String, u64>,
    service_latencies: HashMap<String, Vec<u64>>,
    total_events: u64,
}

impl MetricsObserver {
    fn new() -> Self {
        MetricsObserver {
            status_counts: HashMap::new(),
            service_latencies: HashMap::new(),
            total_events: 0,
        }
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
        self.total_events += 1;

        // Count by status
        *self
            .status_counts
            .entry(event.status.as_str().to_string())
            .or_insert(0) += 1;

        // Record latency sample
        self.service_latencies
            .entry(event.service_name.clone())
            .or_default()
            .push(event.latency_ms);

        println!(
            "  [Metrics] Recorded event #{} — {} {:?} ({}ms)",
            self.total_events, event.service_name, event.status, event.latency_ms
        );
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
        println!(
            "--- Event: {} -> {:?} ---",
            event.service_name, event.status
        );
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
    println!("Degraded count: {}", metrics.status_count("degraded"));
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
    println!("\n--- Event after unsubscribe ---");
    monitor.notify_all(&HealthEvent {
        service_name: "payment-service".into(),
        status: HealthStatus::Unhealthy,
        latency_ms: 3000,
        timestamp: 1180,
        message: Some("Still failing".into()),
    });

    // AlertObserver should still show 1 alert (not 2)
    let alert = alert_obs.lock().unwrap();
    println!("\nAlerts fired (should still be 1): {}", alert.alerts_fired());
    drop(alert);

    // Unsubscribe non-existent observer
    let removed = monitor.unsubscribe("NonExistent");
    println!("Removed NonExistent: {}", removed);
}
