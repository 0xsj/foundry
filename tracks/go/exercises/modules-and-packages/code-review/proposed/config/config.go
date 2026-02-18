// Package config provides application configuration loading.
package config

import (
	"fmt"
	"os"
	"strconv"
)

// ConfigLoader loads application configuration.
// ISSUE: The name stutters — callers write config.ConfigLoader, not config.Loader.
type ConfigLoader struct {
	prefix string
}

// ConfigData holds the loaded configuration values.
// ISSUE: Same stutter problem — callers write config.ConfigData.
type ConfigData struct {
	ServiceName string
	Port        int
	LogLevel    string
}

// NewConfigLoader creates a new ConfigLoader with the given env prefix.
func NewConfigLoader(prefix string) *ConfigLoader {
	return &ConfigLoader{prefix: prefix}
}

// Load reads configuration from environment variables.
func (cl *ConfigLoader) Load() (*ConfigData, error) {
	portStr := os.Getenv(cl.prefix + "_PORT")
	port := 8080
	if portStr != "" {
		var err error
		port, err = strconv.Atoi(portStr)
		if err != nil {
			return nil, fmt.Errorf("invalid %s_PORT: %w", cl.prefix, err)
		}
	}

	return &ConfigData{
		ServiceName: getEnvOrDefault(cl.prefix+"_NAME", "notification-service"),
		Port:        port,
		LogLevel:    getEnvOrDefault(cl.prefix+"_LOG_LEVEL", "info"),
	}, nil
}

func getEnvOrDefault(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}
