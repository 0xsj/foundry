package config

import (
	"fmt"
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
func LoadConfig(env map[string]string) (Config, error) {
	// TODO: Implement configuration loading
	//
	// 1. Start with default values
	// 2. Override with values from env map (if present)
	// 3. Parse string values to appropriate types
	// 4. Validate ranges
	// 5. Return Config or error

	return Config{}, fmt.Errorf("not implemented")
}
