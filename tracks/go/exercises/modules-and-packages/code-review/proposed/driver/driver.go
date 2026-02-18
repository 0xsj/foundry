// Package driver provides a registry for notification drivers.
// Drivers register themselves via init() in their respective packages.
package driver

import (
	"fmt"
	"sync"
)

// Sender is the interface all notification drivers must implement.
type Sender interface {
	Send(to, subject, body string) error
	Protocol() string
}

// registry holds the global driver map.
// ISSUE: Global mutable state initialized via init() in driver packages.
// This makes the registry invisible to callers — they must remember to
// blank-import driver packages to trigger registration. A forgotten import
// silently produces a missing driver at runtime.
var (
	mu       sync.Mutex
	registry = make(map[string]Sender)
)

// Register adds a driver to the global registry.
// Called from init() functions in driver packages.
func Register(protocol string, s Sender) {
	mu.Lock()
	defer mu.Unlock()
	if _, exists := registry[protocol]; exists {
		// Panic on double-registration — init() order is not guaranteed across packages.
		// ISSUE: panicking in a registration function called from init() is very hard
		// to debug because the stack trace points to init(), not to the actual
		// conflicting registration site.
		panic(fmt.Sprintf("driver: protocol %q already registered", protocol))
	}
	registry[protocol] = s
}

// Get retrieves a driver by protocol name.
func Get(protocol string) (Sender, error) {
	mu.Lock()
	defer mu.Unlock()
	s, ok := registry[protocol]
	if !ok {
		return nil, fmt.Errorf("driver: no driver registered for protocol %q", protocol)
	}
	return s, nil
}

// List returns all registered protocol names.
func List() []string {
	mu.Lock()
	defer mu.Unlock()
	names := make([]string, 0, len(registry))
	for name := range registry {
		names = append(names, name)
	}
	return names
}
