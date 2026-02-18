// Package config provides internal configuration for the example project.
// It is in internal/ and cannot be imported by external modules.
package config

import (
	"fmt"
	"os"
	"strconv"
)

// AppConfig holds the application's runtime configuration.
// Unexported fields (port) are only accessible within this package.
type AppConfig struct {
	Env  string // "development", "staging", "production"
	Port int
	Name string

	// debug is unexported — callers use IsDebug()
	debug bool
}

// Default values. Unexported because these are implementation details.
const (
	defaultEnv  = "development"
	defaultPort = 8080
	defaultName = "example-service"
)

// Load reads configuration from environment variables and applies defaults.
// This is the only constructor — it's the package's public API surface.
func Load() (*AppConfig, error) {
	cfg := &AppConfig{
		Env:  getEnvOrDefault("APP_ENV", defaultEnv),
		Name: getEnvOrDefault("APP_NAME", defaultName),
	}

	portStr := getEnvOrDefault("PORT", strconv.Itoa(defaultPort))
	port, err := strconv.Atoi(portStr)
	if err != nil {
		return nil, fmt.Errorf("invalid PORT %q: must be an integer", portStr)
	}
	cfg.Port = port

	cfg.debug = os.Getenv("DEBUG") == "true" || os.Getenv("DEBUG") == "1"

	return cfg, nil
}

// IsDebug returns whether debug mode is enabled.
// Provides read-only access to the unexported field.
func (c *AppConfig) IsDebug() bool {
	return c.debug
}

// Addr returns the listen address in host:port form.
func (c *AppConfig) Addr() string {
	return fmt.Sprintf(":%d", c.Port)
}

// getEnvOrDefault returns the environment variable value or a fallback.
// Unexported — only used within this package.
func getEnvOrDefault(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}
