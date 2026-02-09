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

// LoadConfig parses configuration using a functional approach with helper closures.
// Trades some clarity for reduced repetition.
func LoadConfig(env map[string]string) (Config, error) {
	config := Config{
		Host:           "localhost",
		Port:           8080,
		Timeout:        30 * time.Second,
		MaxConnections: 100,
	}

	// Helper: get string value or default
	getStr := func(key, defaultVal string) string {
		if val, ok := env[key]; ok {
			return val
		}
		return defaultVal
	}

	// Helper: parse int with validation
	parseInt := func(key string, defaultVal int, validate func(int) error) (int, error) {
		val, ok := env[key]
		if !ok {
			return defaultVal, nil
		}
		i, err := strconv.Atoi(val)
		if err != nil {
			return 0, fmt.Errorf("invalid %s %q: %w", key, val, err)
		}
		if validate != nil {
			if err := validate(i); err != nil {
				return 0, err
			}
		}
		return i, nil
	}

	// Helper: parse duration with validation
	parseDuration := func(key string, defaultVal time.Duration, validate func(time.Duration) error) (time.Duration, error) {
		val, ok := env[key]
		if !ok {
			return defaultVal, nil
		}
		d, err := time.ParseDuration(val)
		if err != nil {
			return 0, fmt.Errorf("invalid %s %q: %w", key, val, err)
		}
		if validate != nil {
			if err := validate(d); err != nil {
				return 0, err
			}
		}
		return d, nil
	}

	config.Host = getStr("HOST", config.Host)

	var err error
	config.Port, err = parseInt("PORT", config.Port, func(p int) error {
		if p < 1 || p > 65535 {
			return fmt.Errorf("port %d out of range (1-65535)", p)
		}
		return nil
	})
	if err != nil {
		return Config{}, err
	}

	config.Timeout, err = parseDuration("TIMEOUT", config.Timeout, func(t time.Duration) error {
		if t <= 0 {
			return fmt.Errorf("timeout must be positive, got %v", t)
		}
		return nil
	})
	if err != nil {
		return Config{}, err
	}

	config.MaxConnections, err = parseInt("MAX_CONNECTIONS", config.MaxConnections, func(m int) error {
		if m < 1 {
			return fmt.Errorf("max_connections must be positive, got %d", m)
		}
		return nil
	})
	if err != nil {
		return Config{}, err
	}

	return config, nil
}
