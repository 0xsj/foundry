// Package notifyconfig provides notification-specific configuration.
// ISSUE: This package is too granular. It has one type and one constructor.
// The type and logic could live in a broader "notify" or "notification" package,
// or folded into the existing config package. Creating a separate package for
// a single domain type with four fields adds package-navigation overhead
// without meaningful encapsulation benefit.
package notifyconfig

// NotifyConfig holds notification delivery settings.
// ISSUE: Double stutter — callers write notifyconfig.NotifyConfig. The prefix
// appears twice. Should be notifyconfig.Config or notification.Config.
type NotifyConfig struct {
	MaxRetries   int
	RetryDelayMs int
	BatchSize    int
	DefaultFrom  string
}

// DefaultNotifyConfig returns a NotifyConfig with production-safe defaults.
// ISSUE: Stutter again — callers write notifyconfig.DefaultNotifyConfig().
// Should be notifyconfig.Default() or notifyconfig.New().
func DefaultNotifyConfig() NotifyConfig {
	return NotifyConfig{
		MaxRetries:   3,
		RetryDelayMs: 500,
		BatchSize:    100,
		DefaultFrom:  "notifications@example.com",
	}
}
