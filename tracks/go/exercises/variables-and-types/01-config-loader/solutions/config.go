package config

import (
	"fmt"
	"strconv"
	"time"
)

// Config holds server configuration.
type Config struct {
	Host           string
	Port           int
	Timeout        time.Duration
	MaxConnections int
}

// LoadConfig parses configuration from a map of environment variables.
// Returns Config with defaults for missing values, or an error for invalid values.
//
// This is the reference solution using explicit parsing with early returns.
func LoadConfig(env map[string]string) (Config, error) {
	// Start with default values
	config := Config{
		Host:           "localhost",
		Port:           8080,
		Timeout:        30 * time.Second,
		MaxConnections: 100,
	}

	// Override host if provided
	if host, ok := env["HOST"]; ok {
		config.Host = host
	}

	// Parse and validate port
	if portStr, ok := env["PORT"]; ok {
		port, err := strconv.Atoi(portStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid port %q: %w", portStr, err)
		}
		if port < 1 || port > 65535 {
			return Config{}, fmt.Errorf("port %d out of valid range (1-65535)", port)
		}
		config.Port = port
	}

	// Parse and validate timeout
	if timeoutStr, ok := env["TIMEOUT"]; ok {
		timeout, err := time.ParseDuration(timeoutStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid timeout %q: %w", timeoutStr, err)
		}
		if timeout <= 0 {
			return Config{}, fmt.Errorf("timeout must be positive, got %v", timeout)
		}
		config.Timeout = timeout
	}

	// Parse and validate max connections
	if maxConnStr, ok := env["MAX_CONNECTIONS"]; ok {
		maxConn, err := strconv.Atoi(maxConnStr)
		if err != nil {
			return Config{}, fmt.Errorf("invalid max_connections %q: %w", maxConnStr, err)
		}
		if maxConn < 1 {
			return Config{}, fmt.Errorf("max_connections must be positive, got %d", maxConn)
		}
		config.MaxConnections = maxConn
	}

	return config, nil
}
