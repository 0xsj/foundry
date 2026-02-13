package config

import "fmt"

// ============================================================================
// CONSTANTS
// Define default values for the config. Think about which should be typed
// and which can be untyped.
// ============================================================================

// TODO: Define constants for DefaultPort, DefaultTimeoutSec,
//       DefaultEnvironment, DefaultMaxRetries

// ============================================================================
// CONFIG STRUCT
// Define the Config struct. Consider:
//   - Which fields are always present (value types)?
//   - Which fields might not be set (pointer types)?
//   - What are the zero values of each field type?
// ============================================================================

// TODO: Define the Config struct

// ============================================================================
// LOAD CONFIG
// Parse a map of string environment variables into a typed Config.
// Return an error if required fields are missing or values can't be parsed.
// ============================================================================

// LoadConfig reads configuration from a string map (simulating env vars)
// and returns a fully populated Config with defaults applied.
func LoadConfig(env map[string]string) (Config, error) {
	// TODO: Implement
	return Config{}, fmt.Errorf("not implemented")
}

// ============================================================================
// STRING REPRESENTATION
// Implement the fmt.Stringer interface so Config prints nicely.
// ============================================================================

// TODO: func (c Config) String() string
