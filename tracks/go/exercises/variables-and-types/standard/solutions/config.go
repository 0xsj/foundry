package config

import (
	"fmt"
	"strconv"
)

// ============================================================================
// CONSTANTS
// Untyped constants adapt to context. Typed constants are locked to their type.
// ============================================================================

const (
	DefaultPort        = 8080
	DefaultTimeoutSec  = 30.0
	DefaultEnvironment = "development"
	DefaultMaxRetries  = 3
)

// ============================================================================
// CONFIG STRUCT
// Mix of value types (always present) and pointer types (optional).
// A nil *int means "not set" — distinct from *int pointing to 0.
// ============================================================================

type Config struct {
	Host        string
	Port        int
	Debug       bool
	MaxRetries  *int    // nil = not set, &0 = explicitly zero
	TimeoutSec  float64
	Environment string
}

// ============================================================================
// LOAD CONFIG
// Parse string env vars into typed fields. Apply defaults. Validate required.
// ============================================================================

func LoadConfig(env map[string]string) (Config, error) {
	cfg := Config{
		Port:        DefaultPort,
		TimeoutSec:  DefaultTimeoutSec,
		Environment: DefaultEnvironment,
	}

	// --- Host (required) ---
	host, exists := env["HOST"]
	if !exists || host == "" {
		return Config{}, fmt.Errorf("HOST is required")
	}
	cfg.Host = host

	// --- Port (optional, must be valid int if present) ---
	if portStr, exists := env["PORT"]; exists {
		port, err := strconv.Atoi(portStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid PORT %q: %w", portStr, err)
		}
		cfg.Port = port
	}

	// --- Debug (optional, parses true/false/1/0) ---
	if debugStr, exists := env["DEBUG"]; exists {
		debug, err := strconv.ParseBool(debugStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid DEBUG %q: %w", debugStr, err)
		}
		cfg.Debug = debug
	}

	// --- MaxRetries (optional, pointer field) ---
	// Key distinction: if the key isn't in env, MaxRetries stays nil.
	// If the key IS present (even as "0"), we parse and point to it.
	if retriesStr, exists := env["MAX_RETRIES"]; exists {
		retries, err := strconv.Atoi(retriesStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid MAX_RETRIES %q: %w", retriesStr, err)
		}
		cfg.MaxRetries = &retries // retries is a new local var — safe to take address
	}

	// --- TimeoutSec (optional, must be valid float if present) ---
	if timeoutStr, exists := env["TIMEOUT_SEC"]; exists {
		timeout, err := strconv.ParseFloat(timeoutStr, 64)
		if err != nil {
			return Config{}, fmt.Errorf("invalid TIMEOUT_SEC %q: %w", timeoutStr, err)
		}
		cfg.TimeoutSec = timeout
	}

	// --- Environment (optional) ---
	if envStr, exists := env["ENVIRONMENT"]; exists && envStr != "" {
		cfg.Environment = envStr
	}

	return cfg, nil
}

// ============================================================================
// STRING REPRESENTATION
// Implements fmt.Stringer. Note: Config has a value receiver — it doesn't
// need to modify the struct, and Config is small enough to copy cheaply.
// ============================================================================

func (c Config) String() string {
	retries := "<not set>"
	if c.MaxRetries != nil {
		retries = strconv.Itoa(*c.MaxRetries)
	}

	return fmt.Sprintf(
		"Config{host=%s, port=%d, debug=%v, maxRetries=%s, timeout=%.1fs, env=%s}",
		c.Host, c.Port, c.Debug, retries, c.TimeoutSec, c.Environment,
	)
}
