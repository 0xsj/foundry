// Testing with Dependency Injection
//
// Demonstrates how DI enables comprehensive unit testing in Rust.
// Shows manual mocks, spy patterns, configurable mocks with closures,
// and interior mutability for call tracking.
//
// Run tests: rustc --test testing.rs && ./testing
// Run demo:  rustc testing.rs && ./testing

use std::cell::RefCell;
use std::collections::HashMap;

// ---- Dependency Traits ----

trait HealthChecker {
    fn check(&self, service_name: &str) -> Result<HealthStatus, String>;
}

#[derive(Debug, Clone, PartialEq)]
enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

trait AlertSender {
    fn send_alert(&self, severity: Severity, message: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq)]
enum Severity {
    Warning,
    Critical,
}

trait MetricsRecorder {
    fn record(&self, metric_name: &str, value: f64);
}

// ---- The Service Under Test ----

struct HealthMonitor<H: HealthChecker, A: AlertSender, M: MetricsRecorder> {
    checker: H,
    alerter: A,
    metrics: M,
    alert_threshold: u32,
}

impl<H: HealthChecker, A: AlertSender, M: MetricsRecorder> HealthMonitor<H, A, M> {
    fn new(checker: H, alerter: A, metrics: M, alert_threshold: u32) -> Self {
        HealthMonitor {
            checker,
            alerter,
            metrics,
            alert_threshold,
        }
    }

    /// Checks health of a service and sends alerts if unhealthy.
    /// Returns the number of consecutive failures.
    fn check_service(&self, name: &str, previous_failures: u32) -> (HealthStatus, u32) {
        let status = match self.checker.check(name) {
            Ok(s) => s,
            Err(e) => HealthStatus::Unhealthy(format!("check failed: {}", e)),
        };

        match &status {
            HealthStatus::Healthy => {
                self.metrics.record(&format!("{}.healthy", name), 1.0);
                if previous_failures > 0 {
                    self.metrics.record(&format!("{}.recovery", name), 1.0);
                }
                (status, 0)
            }
            HealthStatus::Degraded(reason) => {
                self.metrics.record(&format!("{}.degraded", name), 1.0);
                let failures = previous_failures + 1;
                if failures >= self.alert_threshold {
                    let _ = self.alerter.send_alert(
                        Severity::Warning,
                        &format!("{} degraded: {} (failures: {})", name, reason, failures),
                    );
                }
                (status, failures)
            }
            HealthStatus::Unhealthy(reason) => {
                self.metrics.record(&format!("{}.unhealthy", name), 1.0);
                let failures = previous_failures + 1;
                let _ = self.alerter.send_alert(
                    Severity::Critical,
                    &format!("{} unhealthy: {} (failures: {})", name, reason, failures),
                );
                (status, failures)
            }
        }
    }

    /// Checks multiple services, returns a summary.
    fn check_all(&self, services: &[&str]) -> HealthSummary {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;

        for &name in services {
            let (status, _) = self.check_service(name, 0);
            match status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded(_) => degraded += 1,
                HealthStatus::Unhealthy(_) => unhealthy += 1,
            }
        }

        HealthSummary {
            total: services.len(),
            healthy,
            degraded,
            unhealthy,
        }
    }
}

#[derive(Debug, PartialEq)]
struct HealthSummary {
    total: usize,
    healthy: usize,
    degraded: usize,
    unhealthy: usize,
}

// ---- Mock Implementations ----

/// A mock health checker that returns pre-configured results per service name.
struct MockHealthChecker {
    responses: HashMap<String, Result<HealthStatus, String>>,
}

impl MockHealthChecker {
    fn new() -> Self {
        MockHealthChecker {
            responses: HashMap::new(),
        }
    }

    fn with_response(mut self, name: &str, status: Result<HealthStatus, String>) -> Self {
        self.responses.insert(name.to_string(), status);
        self
    }
}

impl HealthChecker for MockHealthChecker {
    fn check(&self, service_name: &str) -> Result<HealthStatus, String> {
        self.responses
            .get(service_name)
            .cloned()
            .unwrap_or(Ok(HealthStatus::Healthy))
    }
}

/// A spy alert sender that records all alerts sent.
/// Uses RefCell for interior mutability since send_alert takes &self.
struct SpyAlertSender {
    alerts: RefCell<Vec<(Severity, String)>>,
    should_fail: bool,
}

impl SpyAlertSender {
    fn new() -> Self {
        SpyAlertSender {
            alerts: RefCell::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        SpyAlertSender {
            alerts: RefCell::new(Vec::new()),
            should_fail: true,
        }
    }

    fn alert_count(&self) -> usize {
        self.alerts.borrow().len()
    }

    fn alerts(&self) -> Vec<(Severity, String)> {
        self.alerts.borrow().clone()
    }

    fn assert_alert_sent(&self, severity: Severity, message_contains: &str) {
        let alerts = self.alerts.borrow();
        let found = alerts.iter().any(|(s, m)| *s == severity && m.contains(message_contains));
        assert!(
            found,
            "expected alert with severity {:?} containing {:?}, got: {:?}",
            severity, message_contains, *alerts
        );
    }

    fn assert_no_alerts(&self) {
        let alerts = self.alerts.borrow();
        assert!(
            alerts.is_empty(),
            "expected no alerts, got: {:?}",
            *alerts
        );
    }
}

impl AlertSender for SpyAlertSender {
    fn send_alert(&self, severity: Severity, message: &str) -> Result<(), String> {
        self.alerts.borrow_mut().push((severity, message.to_string()));
        if self.should_fail {
            Err("alert delivery failed".to_string())
        } else {
            Ok(())
        }
    }
}

/// A spy metrics recorder that tracks all recorded metrics.
struct SpyMetrics {
    recordings: RefCell<Vec<(String, f64)>>,
}

impl SpyMetrics {
    fn new() -> Self {
        SpyMetrics {
            recordings: RefCell::new(Vec::new()),
        }
    }

    fn assert_recorded(&self, name: &str, value: f64) {
        let recordings = self.recordings.borrow();
        let found = recordings.iter().any(|(n, v)| n == name && (*v - value).abs() < f64::EPSILON);
        assert!(
            found,
            "expected metric '{}' = {}, got: {:?}",
            name, value, *recordings
        );
    }

    fn count_for(&self, name: &str) -> usize {
        self.recordings.borrow().iter().filter(|(n, _)| n == name).count()
    }
}

impl MetricsRecorder for SpyMetrics {
    fn record(&self, metric_name: &str, value: f64) {
        self.recordings.borrow_mut().push((metric_name.to_string(), value));
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a monitor with default mocks
    fn create_monitor(
        checker: MockHealthChecker,
        alerter: SpyAlertSender,
    ) -> HealthMonitor<MockHealthChecker, SpyAlertSender, SpyMetrics> {
        HealthMonitor::new(checker, alerter, SpyMetrics::new(), 2)
    }

    #[test]
    fn healthy_service_records_metric_and_no_alert() {
        let checker = MockHealthChecker::new()
            .with_response("api", Ok(HealthStatus::Healthy));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        let (status, failures) = monitor.check_service("api", 0);

        assert_eq!(status, HealthStatus::Healthy);
        assert_eq!(failures, 0);
        monitor.metrics.assert_recorded("api.healthy", 1.0);
        monitor.alerter.assert_no_alerts();
    }

    #[test]
    fn unhealthy_service_sends_critical_alert_immediately() {
        let checker = MockHealthChecker::new()
            .with_response("db", Ok(HealthStatus::Unhealthy("connection refused".into())));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        let (status, failures) = monitor.check_service("db", 0);

        assert_eq!(status, HealthStatus::Unhealthy("connection refused".into()));
        assert_eq!(failures, 1);
        monitor.metrics.assert_recorded("db.unhealthy", 1.0);
        monitor.alerter.assert_alert_sent(Severity::Critical, "connection refused");
    }

    #[test]
    fn degraded_below_threshold_does_not_alert() {
        let checker = MockHealthChecker::new()
            .with_response("cache", Ok(HealthStatus::Degraded("slow response".into())));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        // First degraded check (previous_failures = 0, so failures becomes 1)
        // Threshold is 2, so no alert
        let (status, failures) = monitor.check_service("cache", 0);

        assert_eq!(status, HealthStatus::Degraded("slow response".into()));
        assert_eq!(failures, 1);
        monitor.metrics.assert_recorded("cache.degraded", 1.0);
        monitor.alerter.assert_no_alerts();
    }

    #[test]
    fn degraded_at_threshold_sends_warning() {
        let checker = MockHealthChecker::new()
            .with_response("cache", Ok(HealthStatus::Degraded("slow response".into())));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        // Simulate previous_failures = 1, so this check makes it 2 (= threshold)
        let (_, failures) = monitor.check_service("cache", 1);

        assert_eq!(failures, 2);
        monitor.alerter.assert_alert_sent(Severity::Warning, "slow response");
    }

    #[test]
    fn recovery_after_failures_records_recovery_metric() {
        let checker = MockHealthChecker::new()
            .with_response("api", Ok(HealthStatus::Healthy));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        // Service was previously failing (3 failures), now healthy
        let (status, failures) = monitor.check_service("api", 3);

        assert_eq!(status, HealthStatus::Healthy);
        assert_eq!(failures, 0);
        monitor.metrics.assert_recorded("api.recovery", 1.0);
    }

    #[test]
    fn checker_error_treated_as_unhealthy() {
        let checker = MockHealthChecker::new()
            .with_response("api", Err("timeout".into()));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        let (status, _) = monitor.check_service("api", 0);

        assert_eq!(status, HealthStatus::Unhealthy("check failed: timeout".into()));
        monitor.alerter.assert_alert_sent(Severity::Critical, "check failed: timeout");
    }

    #[test]
    fn check_all_produces_correct_summary() {
        let checker = MockHealthChecker::new()
            .with_response("api", Ok(HealthStatus::Healthy))
            .with_response("db", Ok(HealthStatus::Unhealthy("down".into())))
            .with_response("cache", Ok(HealthStatus::Degraded("slow".into())));
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        let summary = monitor.check_all(&["api", "db", "cache"]);

        assert_eq!(summary, HealthSummary {
            total: 3,
            healthy: 1,
            degraded: 1,
            unhealthy: 1,
        });
    }

    #[test]
    fn alert_failure_does_not_crash_monitoring() {
        // Alerter is configured to fail -- service should still work
        let checker = MockHealthChecker::new()
            .with_response("api", Ok(HealthStatus::Unhealthy("down".into())));
        let alerter = SpyAlertSender::failing();
        let monitor = create_monitor(checker, alerter);

        let (status, failures) = monitor.check_service("api", 0);

        // Service still returns correct status even though alert failed
        assert_eq!(status, HealthStatus::Unhealthy("down".into()));
        assert_eq!(failures, 1);
        // Alert was attempted (recorded in spy even though it "failed")
        assert_eq!(monitor.alerter.alert_count(), 1);
    }

    #[test]
    fn unknown_service_defaults_to_healthy() {
        let checker = MockHealthChecker::new(); // No responses configured
        let alerter = SpyAlertSender::new();
        let monitor = create_monitor(checker, alerter);

        let (status, failures) = monitor.check_service("unknown-service", 0);

        assert_eq!(status, HealthStatus::Healthy);
        assert_eq!(failures, 0);
    }
}

// ---- Main: Demo ----

fn main() {
    println!("=== Testing with Dependency Injection ===\n");

    println!("Key patterns demonstrated:");
    println!("  1. MockHealthChecker -- pre-configured responses per service");
    println!("  2. SpyAlertSender   -- records calls via RefCell for assertions");
    println!("  3. SpyMetrics       -- tracks metric recordings");
    println!("  4. Builder pattern  -- .with_response() for readable test setup");
    println!();
    println!("Run tests with: rustc --test testing.rs && ./testing");
    println!();
    println!("Why this works:");
    println!("  - HealthMonitor is generic over H, A, M");
    println!("  - In tests: H=MockHealthChecker, A=SpyAlertSender, M=SpyMetrics");
    println!("  - In production: H=HttpChecker, A=PagerDutySender, M=PrometheusRecorder");
    println!("  - Same business logic, different dependencies");
    println!("  - No database, no HTTP, no external services in tests");
}
