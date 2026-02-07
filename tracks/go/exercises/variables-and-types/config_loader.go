package config

import (
	"fmt"
	"strconv"
)

// Config holds typed service configuration.
// Notice: Go gives every field a zero value automatically.
// Your job is to apply *application-level* defaults, which are different
// from zero values (e.g. Port should default to 8080, not 0).
type Config struct {
	Host        string
	Port        int
	MaxRetries  int
	Timeout     float64
	Debug       bool
	ServiceName string
}

// Defaults returns a Config with application-level default values.
func Defaults() Config {
	// TODO: return a Config populated with the specified defaults
	return Config{
		Host:        "localhost",
		Port:        8080,
		MaxRetries:  3,
		Timeout:     15.0,
		Debug:       true,
		ServiceName: "users",
	}
}

// LoadConfig takes raw string key-value pairs and returns a typed Config.
// Missing keys should fall back to defaults.
// Invalid values should return an error.

func LoadConfig(raw map[string]string) (Config, error) {
	cfg := Defaults()

	for key, val := range raw {
		switch key {
		case "host":
			cfg.Host = val
		case "port":
			p, err := strconv.Atoi(val)
			if err != nil {
				return cfg, fmt.Errorf("invalid port %q: %w", val, err)
			}
			cfg.Port = p
		case "max_retries":
			mr, err := strconv.Atoi(val)
			if err != nil {
				return cfg, fmt.Errorf("invalid max_retries %q: %w", val, err)
			}
			cfg.MaxRetries = mr
		case "timeout":
			t, err := strconv.ParseFloat(val, 64)
			if err != nil {
				return cfg, fmt.Errorf("invalid timeout %q: %w", val, err)
			}
			cfg.Timeout = t
		case "debug":
			d, err := strconv.ParseBool(val)
			if err != nil {
				return cfg, fmt.Errorf("invalid debug %q: %w", val, err)
			}
			cfg.Debug = d
		case "service_name":
			cfg.ServiceName = val
		}
	}

	return cfg, nil
}
