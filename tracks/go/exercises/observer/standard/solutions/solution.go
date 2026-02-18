package healthmonitor

import (
	"fmt"
	"sync"
	"time"
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
	Service   string
	Previous  ServiceStatus
	Current   ServiceStatus
	Timestamp time.Time
	Metadata  map[string]string
}

// HealthObserver is the observer interface.
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
	// mu protects both observers and services.
	// We use RWMutex because reads (notification snapshots, status queries)
	// are far more frequent than writes (register, unregister, status updates).
	mu        sync.RWMutex
	observers []HealthObserver
	services  map[string]*serviceState
}

// NewHealthMonitor creates a new monitor with no observers and no known services.
func NewHealthMonitor() *HealthMonitor {
	return &HealthMonitor{
		services: make(map[string]*serviceState),
	}
}

// Register adds an observer that will be notified of status changes.
// Safe for concurrent use.
func (m *HealthMonitor) Register(observer HealthObserver) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.observers = append(m.observers, observer)
}

// Unregister removes an observer. If the observer is not registered, this is a no-op.
// Safe for concurrent use.
func (m *HealthMonitor) Unregister(observer HealthObserver) {
	m.mu.Lock()
	defer m.mu.Unlock()
	for i, obs := range m.observers {
		if obs == observer {
			// Remove by shifting -- preserves order, which matters for
			// predictable notification ordering in tests.
			m.observers = append(m.observers[:i], m.observers[i+1:]...)
			return
		}
	}
}

// ReportStatus records the current health status of a service.
// If the status changed from the previous report, all observers are notified.
// Returns any errors from observer notifications (including recovered panics).
func (m *HealthMonitor) ReportStatus(service string, status ServiceStatus, metadata map[string]string) []error {
	now := time.Now()

	// Step 1: Check and update status under write lock.
	m.mu.Lock()
	state, exists := m.services[service]
	var previous ServiceStatus
	if exists {
		previous = state.Status
	} else {
		previous = StatusUnknown
		state = &serviceState{}
		m.services[service] = state
	}

	changed := previous != status
	state.Status = status
	state.LastSeen = now

	// Step 2: If no change, release lock and return early.
	if !changed {
		m.mu.Unlock()
		return nil
	}

	// Step 3: Snapshot the observer list while we still hold the lock.
	// We need a copy because we'll call observers after releasing the lock.
	snapshot := make([]HealthObserver, len(m.observers))
	copy(snapshot, m.observers)
	m.mu.Unlock()

	// Step 4: Build the change event.
	change := StatusChange{
		Service:   service,
		Previous:  previous,
		Current:   status,
		Timestamp: now,
		Metadata:  metadata,
	}

	// Step 5: Notify all observers. Call each one safely -- recover from panics
	// and collect errors. Continue notifying even if one observer fails.
	var errs []error
	for _, obs := range snapshot {
		if err := safeNotify(obs, change); err != nil {
			errs = append(errs, err)
		}
	}

	if len(errs) == 0 {
		return nil
	}
	return errs
}

// safeNotify calls an observer's OnStatusChange method, recovering from panics.
// If the observer panics, the panic is converted to an error.
func safeNotify(obs HealthObserver, change StatusChange) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("observer panicked: %v", r)
		}
	}()
	return obs.OnStatusChange(change)
}

// CurrentStatus returns the last known status for a service.
// Returns StatusUnknown if the service has never reported.
func (m *HealthMonitor) CurrentStatus(service string) ServiceStatus {
	m.mu.RLock()
	defer m.mu.RUnlock()
	state, exists := m.services[service]
	if !exists {
		return StatusUnknown
	}
	return state.Status
}

// AllStatuses returns a snapshot of all known service statuses.
// The returned map is a copy -- modifying it does not affect the monitor.
func (m *HealthMonitor) AllStatuses() map[string]ServiceStatus {
	m.mu.RLock()
	defer m.mu.RUnlock()
	result := make(map[string]ServiceStatus, len(m.services))
	for name, state := range m.services {
		result[name] = state.Status
	}
	return result
}
