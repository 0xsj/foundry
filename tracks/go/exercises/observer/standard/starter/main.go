package healthmonitor

import (
	"fmt"
	"sync"
	"time"
)

// Ensure imports are used (remove these when implementing).
var (
	_ = sync.RWMutex{}
	_ = time.Now
)

// ServiceStatus represents the health state of a service.
type ServiceStatus int

const (
	StatusUnknown ServiceStatus = iota
	StatusHealthy
	StatusDegraded
	StatusUnhealthy
)

func (s ServiceStatus) String() string {
	switch s {
	case StatusUnknown:
		return "Unknown"
	case StatusHealthy:
		return "Healthy"
	case StatusDegraded:
		return "Degraded"
	case StatusUnhealthy:
		return "Unhealthy"
	default:
		return fmt.Sprintf("ServiceStatus(%d)", int(s))
	}
}

// StatusChange is the event payload delivered to observers when a service
// transitions between health states.
type StatusChange struct {
	Service    string
	Previous   ServiceStatus
	Current    ServiceStatus
	Timestamp  time.Time
	Metadata   map[string]string // optional: error message, response time, etc.
}

// HealthObserver is the observer interface. Any type that wants to react
// to service health changes implements this.
type HealthObserver interface {
	OnStatusChange(change StatusChange) error
}

// serviceState tracks the last known status of a service.
type serviceState struct {
	Status   ServiceStatus
	LastSeen time.Time
}

// HealthMonitor is the subject that tracks service health and notifies observers.
type HealthMonitor struct {
	mu        sync.RWMutex
	observers []HealthObserver
	services  map[string]*serviceState
}

// NewHealthMonitor creates a new monitor with no observers and no known services.
func NewHealthMonitor() *HealthMonitor {
	// TODO: Initialize and return a HealthMonitor
	return nil
}

// Register adds an observer that will be notified of status changes.
func (m *HealthMonitor) Register(observer HealthObserver) {
	// TODO: Add the observer to the list.
	// Must be safe for concurrent use.
}

// Unregister removes an observer. If the observer is not registered, this is a no-op.
func (m *HealthMonitor) Unregister(observer HealthObserver) {
	// TODO: Remove the observer from the list.
	// Must be safe for concurrent use.
}

// ReportStatus records the current health status of a service.
// If the status has changed from the previous report, all observers are notified.
// Returns any errors from observer notifications.
func (m *HealthMonitor) ReportStatus(service string, status ServiceStatus, metadata map[string]string) []error {
	// TODO: Implement this method.
	//
	// Steps:
	// 1. Check if the status changed from the last known status for this service.
	//    (If the service hasn't been seen before, its previous status is StatusUnknown.)
	// 2. Update the stored status.
	// 3. If the status changed, notify all observers.
	//    - Copy the observer list under a read lock.
	//    - Release the lock BEFORE calling observers.
	//    - Handle observer panics gracefully (recover and continue).
	//    - Collect errors from observers that return errors.
	// 4. Return collected errors (nil if no errors).
	return nil
}

// CurrentStatus returns the last known status for a service.
// Returns StatusUnknown if the service has never reported.
func (m *HealthMonitor) CurrentStatus(service string) ServiceStatus {
	// TODO: Return the current status for the named service.
	return StatusUnknown
}

// AllStatuses returns a snapshot of all known service statuses.
func (m *HealthMonitor) AllStatuses() map[string]ServiceStatus {
	// TODO: Return a copy of the current status map.
	// Do not return the internal map directly -- that would be a data race.
	return nil
}
