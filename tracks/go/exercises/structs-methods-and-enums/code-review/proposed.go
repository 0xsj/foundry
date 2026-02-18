package notifyqueue

import "fmt"

// Priority represents the urgency level of a notification.
type Priority int

const (
	PriorityCritical Priority = iota // 0 — highest
	PriorityHigh                     // 1
	PriorityNormal                   // 2
	PriorityLow                      // 3
)

// QueueStats tracks usage metrics for the notification queue.
// Embedded in NotifyQueue to expose stats methods directly.
type QueueStats struct {
	EnqueueCount int  // total enqueues since creation
	DequeueCount int  // total dequeues since creation
	PeakSize     int  // maximum observed queue length
}

// RecordEnqueue updates stats after an enqueue operation.
func (s QueueStats) RecordEnqueue(currentSize int) {  // ISSUE: value receiver
	s.EnqueueCount++
	if currentSize > s.PeakSize {
		s.PeakSize = currentSize
	}
}

// RecordDequeue updates stats after a dequeue operation.
func (s *QueueStats) RecordDequeue() {
	s.DequeueCount++
}

// Notification is the item stored in the queue.
type Notification struct {
	ID        string
	Priority  Priority
	Recipient string
	Body      string
}

// NotifyQueue is a priority-aware notification queue.
type NotifyQueue struct {
	*QueueStats                          // ISSUE: embedded pointer, not initialized in New
	Buckets    map[Priority][]Notification  // ISSUE: exported — external code can bypass queue methods
	Paused     map[Priority]bool            // ISSUE: exported — external code can mutate directly
	maxPerBucket int
}

// New creates a ready-to-use NotifyQueue.
func New(maxPerBucket int) *NotifyQueue {
	return &NotifyQueue{
		// QueueStats is omitted — left as nil pointer
		Buckets:      make(map[Priority][]Notification),
		Paused:       make(map[Priority]bool),
		maxPerBucket: maxPerBucket,
	}
}

// Enqueue adds a notification to the appropriate priority bucket.
func (q *NotifyQueue) Enqueue(n Notification) error {
	if q.Paused[n.Priority] {
		return fmt.Errorf("priority %d is paused", n.Priority)
	}

	bucket := q.Buckets[n.Priority]
	if len(bucket) >= q.maxPerBucket {
		return fmt.Errorf("bucket for priority %d is full", n.Priority)
	}

	q.Buckets[n.Priority] = append(bucket, n)
	q.RecordEnqueue(len(q.Buckets[n.Priority]))  // panics: QueueStats is nil
	return nil
}

// Dequeue removes and returns the highest-priority available notification.
// Returns false if all buckets are empty or paused.
func (q *NotifyQueue) Dequeue() (Notification, bool) {
	for _, priority := range []Priority{PriorityCritical, PriorityHigh, PriorityNormal, PriorityLow} {
		if q.Paused[priority] {
			continue
		}
		bucket := q.Buckets[priority]
		if len(bucket) == 0 {
			continue
		}
		n := bucket[0]
		q.Buckets[priority] = bucket[1:]
		q.RecordDequeue()  // panics: QueueStats is nil
		return n, true
	}
	return Notification{}, false
}

// Pause stops processing for the given priority level.
func (q NotifyQueue) Pause(priority Priority) {  // ISSUE: value receiver
	q.Paused[priority] = true
}

// Resume re-enables processing for the given priority level.
func (q *NotifyQueue) Resume(priority Priority) {
	delete(q.Paused, priority)
}

// Size returns the total number of notifications across all buckets.
func (q *NotifyQueue) Size() int {
	total := 0
	for _, bucket := range q.Buckets {
		total += len(bucket)
	}
	return total
}

// Stats returns a summary string.
func (q *NotifyQueue) Stats() string {
	return fmt.Sprintf("enqueued=%d dequeued=%d peak=%d current=%d",
		q.EnqueueCount, q.DequeueCount, q.PeakSize, q.Size())
}

// PriorityFromInt converts an int to a Priority.
// Returns PriorityLow for any unrecognized value.  ISSUE: silent fallback
func PriorityFromInt(n int) Priority {
	switch Priority(n) {
	case PriorityCritical, PriorityHigh, PriorityNormal, PriorityLow:
		return Priority(n)
	}
	return PriorityLow  // silent fallback — caller can't tell if input was valid
}
