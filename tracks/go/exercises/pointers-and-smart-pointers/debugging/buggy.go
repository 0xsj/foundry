package scheduler

import (
	"fmt"
	"time"
)

// ============================================================================
// JobStats tracks execution metrics for the scheduler.
// ============================================================================

// JobStats holds counters for a scheduler's execution history.
type JobStats struct {
	CompletedCount int
	FailureCount   int
	TotalRuntimeMs int64
}

// RecordCompletion records a successful job run.
func (s JobStats) RecordCompletion(runtimeMs int64) { // BUG 1: value receiver
	s.CompletedCount++
	s.TotalRuntimeMs += runtimeMs
}

// RecordFailure records a failed job run.
func (s *JobStats) RecordFailure() {
	s.FailureCount++
}

// ============================================================================
// WorkerPool manages a set of workers and tracks stats.
// ============================================================================

// Worker represents a job-processing worker.
type Worker struct {
	ID       string
	Capacity int
	Active   bool
}

// WorkerPool manages a collection of workers.
type WorkerPool struct {
	*JobStats                // BUG 2: embedded pointer, never initialized in NewWorkerPool
	workers  []*Worker
	maxWorkers int
}

// NewWorkerPool creates a ready-to-use WorkerPool.
func NewWorkerPool(maxWorkers int) *WorkerPool {
	return &WorkerPool{
		// JobStats left as nil — BUG 2 is here
		maxWorkers: maxWorkers,
	}
}

// AddWorker adds a worker to the pool.
func (p *WorkerPool) AddWorker(w *Worker) error {
	if len(p.workers) >= p.maxWorkers {
		return fmt.Errorf("pool is at capacity")
	}
	p.workers = append(p.workers, w)
	return nil
}

// Workers returns the list of registered workers.
func (p *WorkerPool) Workers() []*Worker {
	return p.workers
}

// Stats returns the embedded JobStats.
func (p *WorkerPool) Stats() *JobStats {
	return p.JobStats
}

// ============================================================================
// RegisterWorkers adds workers from a slice using range.
// ============================================================================

// RegisterAll adds each worker in the provided slice to the pool.
// It captures pointers to the workers for later reference in workerRefs.
func RegisterAll(workers []Worker, pool *WorkerPool) []*Worker {
	refs := make([]*Worker, len(workers))
	for i, w := range workers { // BUG 3: &w takes address of the loop variable
		pool.AddWorker(&w)
		refs[i] = &w // all refs end up pointing to the same loop variable
	}
	return refs
}

// ============================================================================
// HealthCheck — interface and implementations
// ============================================================================

// HealthChecker can report whether a service is healthy.
type HealthChecker interface {
	IsHealthy() bool
}

// ServiceHealth wraps a HealthChecker and records health status.
type ServiceHealth struct {
	checker HealthChecker
	name    string
}

// NewServiceHealth creates a ServiceHealth. If checker is nil, it should
// be treated as unhealthy.
func NewServiceHealth(name string, checker HealthChecker) *ServiceHealth {
	return &ServiceHealth{
		checker: checker,
		name:    name,
	}
}

// Healthy reports whether the underlying checker says the service is healthy.
// BUG 4: missing nil check — if checker is a non-nil interface wrapping a nil
// concrete pointer, the nil check passes but calling IsHealthy() panics.
func (s *ServiceHealth) Healthy() bool {
	if s.checker == nil {
		return false
	}
	return s.checker.IsHealthy() // panics if checker holds a nil *ConcreteHandler
}

// Name returns the service name.
func (s *ServiceHealth) Name() string {
	return s.name
}

// ============================================================================
// ConcreteHealthHandler — an implementation of HealthChecker
// ============================================================================

// ConcreteHealthHandler is a configurable health check implementation.
type ConcreteHealthHandler struct {
	healthy  bool
	checkedAt *time.Time
}

// NewConcreteHealthHandler creates an initialized handler.
func NewConcreteHealthHandler(healthy bool) *ConcreteHealthHandler {
	return &ConcreteHealthHandler{healthy: healthy}
}

// IsHealthy returns the configured health status.
func (h *ConcreteHealthHandler) IsHealthy() bool {
	now := time.Now()
	h.checkedAt = &now
	return h.healthy
}
