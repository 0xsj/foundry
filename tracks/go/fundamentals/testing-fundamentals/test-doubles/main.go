// Package testdoubles demonstrates dependency injection for testability.
//
// The pattern: define interfaces for dependencies that are hard to control
// in tests (time, randomness, external services). Inject them. In tests,
// substitute lightweight fakes or stubs.
//
// Run the tests: go test ./test-doubles/
package testdoubles

import (
	"fmt"
	"sync"
	"time"
)

// ============================================================================
// Clock abstraction — the canonical example of a testable time dependency
// ============================================================================

// Clock abstracts time.Now() so tests can control "now" without sleeping.
type Clock interface {
	Now() time.Time
}

// RealClock delegates to the standard library. Used in production.
type RealClock struct{}

func (RealClock) Now() time.Time { return time.Now() }

// ============================================================================
// Notifier abstraction — a dependency on an external service
// ============================================================================

// Notifier sends a notification to a user.
// In production: email, SMS, push notification.
// In tests: record calls and return a controlled response.
type Notifier interface {
	Notify(userID string, message string) error
}

// ============================================================================
// AlertManager — the type under test
//
// It fires alerts when a metric exceeds a threshold within a time window.
// Depends on Clock and Notifier — both injected, both replaceable in tests.
// ============================================================================

// Alert represents a triggered alert event.
type Alert struct {
	Metric    string
	Value     float64
	Threshold float64
	FiredAt   time.Time
	UserID    string
}

// AlertManager watches metrics and notifies users when thresholds are exceeded.
type AlertManager struct {
	clock     Clock
	notifier  Notifier
	threshold float64
	cooldown  time.Duration // minimum time between alerts for the same metric

	mu         sync.Mutex
	lastFired  map[string]time.Time // metric -> last alert time
	alertsSent []Alert
}

// NewAlertManager creates an AlertManager with the given dependencies.
func NewAlertManager(clock Clock, notifier Notifier, threshold float64, cooldown time.Duration) *AlertManager {
	return &AlertManager{
		clock:     clock,
		notifier:  notifier,
		threshold: threshold,
		cooldown:  cooldown,
		lastFired: make(map[string]time.Time),
	}
}

// Record observes a metric value. If value exceeds threshold and the cooldown
// period has elapsed since the last alert for this metric, it fires an alert
// by notifying the user.
func (am *AlertManager) Record(metric string, value float64, userID string) error {
	am.mu.Lock()
	defer am.mu.Unlock()

	if value <= am.threshold {
		return nil // below threshold — nothing to do
	}

	now := am.clock.Now()
	if last, ok := am.lastFired[metric]; ok {
		if now.Sub(last) < am.cooldown {
			return nil // still in cooldown — suppress alert
		}
	}

	alert := Alert{
		Metric:    metric,
		Value:     value,
		Threshold: am.threshold,
		FiredAt:   now,
		UserID:    userID,
	}

	if err := am.notifier.Notify(userID, am.formatMessage(alert)); err != nil {
		return fmt.Errorf("alertManager: notify %q: %w", userID, err)
	}

	am.lastFired[metric] = now
	am.alertsSent = append(am.alertsSent, alert)
	return nil
}

// AlertsSent returns all alerts fired since creation.
func (am *AlertManager) AlertsSent() []Alert {
	am.mu.Lock()
	defer am.mu.Unlock()
	result := make([]Alert, len(am.alertsSent))
	copy(result, am.alertsSent)
	return result
}

func (am *AlertManager) formatMessage(a Alert) string {
	return fmt.Sprintf("ALERT: %s = %.2f (threshold: %.2f)", a.Metric, a.Value, a.Threshold)
}
