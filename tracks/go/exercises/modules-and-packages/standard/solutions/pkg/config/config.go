// Package config provides configuration loading for the healthcheck tool.
//
// It lives in pkg/ because it has no dependencies on internal packages and
// could reasonably be imported by external modules or other tools in a monorepo.
// The sole entry point is Load, which reads from a string map (simulating env vars).
package config

import (
	"fmt"
	"strconv"
	"strings"
)

// Config holds the runtime configuration for the health checker.
type Config struct {
	// URLs is the list of endpoints to check. Required — at least one entry.
	URLs []string

	// TimeoutSec is the per-request HTTP timeout in seconds. Defaults to DefaultTimeout.
	TimeoutSec float64

	// Verbose enables additional progress output to stdout.
	Verbose bool
}

// DefaultTimeout is the fallback request timeout when not specified.
const DefaultTimeout = 10.0

// Load reads configuration from the provided environment variable map.
// Recognized keys:
//
//	HEALTHCHECK_URLS    — comma-separated URLs to check (required)
//	HEALTHCHECK_TIMEOUT — timeout in seconds, e.g. "5.0" (optional, default 10.0)
//	HEALTHCHECK_VERBOSE — "true" or "1" for verbose output (optional)
//
// Returns an error if HEALTHCHECK_URLS is missing or empty, or if numeric
// fields cannot be parsed.
func Load(env map[string]string) (Config, error) {
	cfg := Config{
		TimeoutSec: DefaultTimeout,
	}

	// --- URLs (required) ---
	rawURLs, exists := env["HEALTHCHECK_URLS"]
	if !exists || strings.TrimSpace(rawURLs) == "" {
		return Config{}, fmt.Errorf("HEALTHCHECK_URLS is required")
	}

	parts := strings.Split(rawURLs, ",")
	for _, u := range parts {
		u = strings.TrimSpace(u)
		if u != "" {
			cfg.URLs = append(cfg.URLs, u)
		}
	}

	if len(cfg.URLs) == 0 {
		return Config{}, fmt.Errorf("HEALTHCHECK_URLS must contain at least one URL")
	}

	// --- Timeout (optional) ---
	if timeoutStr, ok := env["HEALTHCHECK_TIMEOUT"]; ok && timeoutStr != "" {
		timeout, err := strconv.ParseFloat(timeoutStr, 64)
		if err != nil {
			return Config{}, fmt.Errorf("invalid HEALTHCHECK_TIMEOUT %q: must be a number", timeoutStr)
		}
		if timeout <= 0 {
			return Config{}, fmt.Errorf("HEALTHCHECK_TIMEOUT must be positive, got %g", timeout)
		}
		cfg.TimeoutSec = timeout
	}

	// --- Verbose (optional) ---
	if v, ok := env["HEALTHCHECK_VERBOSE"]; ok {
		cfg.Verbose = v == "true" || v == "1"
	}

	return cfg, nil
}
