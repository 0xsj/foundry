package main

import (
	"fmt"
	"time"
)

// Task represents a background job
type Task struct {
	ID       string
	Type     string
	Payload  map[string]interface{}
	Status   string
	Retries  int
	MaxRetries int
}

// ProcessTask handles task execution with retries
// ANTI-PATTERN: Deeply nested, hard to follow
func ProcessTask(task *Task) error {
	// BUG: No nil check - will panic if task is nil
	if task.Status == "completed" {
		fmt.Println("Task already completed")
		return nil
	} else {
		if task.Status == "failed" {
			fmt.Println("Task permanently failed")
			return fmt.Errorf("task %s failed", task.ID)
		} else {
			if task.Status == "pending" {
				// ANTI-PATTERN: Deep nesting continues
				if task.Type != "" {
					if task.Type == "email" {
						if task.Payload != nil {
							// Actually process the task
							fmt.Printf("Processing email task %s\n", task.ID)
							err := sendEmail(task.Payload)
							if err != nil {
								// ANTI-PATTERN: Retry logic is nested inside success path checking
								if task.Retries < task.MaxRetries {
									task.Retries = task.Retries + 1
									fmt.Printf("Retry %d/%d\n", task.Retries, task.MaxRetries)
									time.Sleep(time.Second * 2)
									return ProcessTask(task) // Recursive retry
								} else {
									task.Status = "failed"
									return fmt.Errorf("max retries exceeded: %w", err)
								}
							} else {
								task.Status = "completed"
								return nil
							}
						} else {
							return fmt.Errorf("payload is nil")
						}
					} else if task.Type == "sms" {
						if task.Payload != nil {
							fmt.Printf("Processing SMS task %s\n", task.ID)
							err := sendSMS(task.Payload)
							if err != nil {
								if task.Retries < task.MaxRetries {
									task.Retries = task.Retries + 1
									fmt.Printf("Retry %d/%d\n", task.Retries, task.MaxRetries)
									time.Sleep(time.Second * 2)
									return ProcessTask(task)
								} else {
									task.Status = "failed"
									return fmt.Errorf("max retries exceeded: %w", err)
								}
							} else {
								task.Status = "completed"
								return nil
							}
						} else {
							return fmt.Errorf("payload is nil")
						}
					} else {
						return fmt.Errorf("unknown task type: %s", task.Type)
					}
				} else {
					return fmt.Errorf("task type is empty")
				}
			} else {
				return fmt.Errorf("invalid status: %s", task.Status)
			}
		}
	}
}

// ANTI-PATTERN: Loop counter bug
func RetryOperation(op func() error, maxAttempts int) error {
	attempt := 1
	// BUG: Loop condition - should use < not <=
	// This causes one extra iteration beyond maxAttempts
	for attempt <= maxAttempts {
		err := op()
		if err == nil {
			return nil
		}
		// BUG: Increment happens after the operation
		// So on the last allowed attempt, we increment past maxAttempts
		fmt.Printf("Attempt %d failed, retrying...\n", attempt)
		attempt = attempt + 1
		time.Sleep(time.Second)
	}
	return fmt.Errorf("failed after %d attempts", attempt)
}

// ANTI-PATTERN: Could use switch instead of if-else chain
func GetTaskPriority(taskType string) int {
	if taskType == "critical" {
		return 1
	} else if taskType == "high" {
		return 2
	} else if taskType == "medium" {
		return 3
	} else if taskType == "low" {
		return 4
	} else {
		return 5 // default
	}
}

// Stub functions
func sendEmail(payload map[string]interface{}) error {
	// Simulated email sending
	return nil
}

func sendSMS(payload map[string]interface{}) error {
	// Simulated SMS sending
	return nil
}

func main() {
	task := &Task{
		ID:         "task-123",
		Type:       "email",
		Status:     "pending",
		Payload:    map[string]interface{}{"to": "user@example.com"},
		MaxRetries: 3,
	}

	err := ProcessTask(task)
	if err != nil {
		fmt.Println("Error:", err)
	}
}
