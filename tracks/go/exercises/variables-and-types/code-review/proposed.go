package ratelimiter

import (
	"fmt"
	"math"
)

// Tier represents the pricing tier for a tenant.
type Tier int

const (
	Free Tier = iota
	Pro
	Enterprise
)

// TenantConfig holds rate limiting settings for a single tenant.
type TenantConfig struct {
	TenantID      string
	Tier          Tier
	RequestsPerMin int
	BurstSize     int
	Enabled       bool
}

// RateLimiter manages per-tenant rate limiting configuration.
type RateLimiter struct {
	tenants  map[string]TenantConfig
	defaults map[Tier]TenantConfig
}

// New creates a rate limiter with default configs per tier.
func New() *RateLimiter {
	rl := &RateLimiter{}

	rl.defaults = map[Tier]TenantConfig{
		Free:       {RequestsPerMin: 60, BurstSize: 10, Enabled: true},
		Pro:        {RequestsPerMin: 600, BurstSize: 100, Enabled: true},
		Enterprise: {RequestsPerMin: 6000, BurstSize: 1000, Enabled: true},
	}

	return rl
}

// Register adds a new tenant with tier defaults.
func (rl *RateLimiter) Register(tenantID string, tier Tier) {
	cfg := rl.defaults[tier]
	cfg.TenantID = tenantID
	cfg.Tier = tier
	rl.tenants[tenantID] = cfg
}

// UpdateLimit changes the requests-per-minute for a tenant.
// The caller passes the desired per-second rate; we convert to per-minute.
func (rl *RateLimiter) UpdateLimit(tenantID string, perSecond int) error {
	cfg, ok := rl.tenants[tenantID]
	if !ok {
		return fmt.Errorf("tenant %s not found", tenantID)
	}

	cfg.RequestsPerMin = perSecond * 60
	rl.tenants[tenantID] = cfg
	return nil
}

// ScaleLimit multiplies a tenant's current limit by a factor.
// Used during traffic spikes to temporarily increase capacity.
func (rl *RateLimiter) ScaleLimit(tenantID string, factor float64) error {
	cfg, ok := rl.tenants[tenantID]
	if !ok {
		return fmt.Errorf("tenant %s not found", tenantID)
	}

	scaled := float64(cfg.RequestsPerMin) * factor
	cfg.RequestsPerMin = int(scaled)
	rl.tenants[tenantID] = cfg
	return nil
}

// GetConfig returns the config for a tenant.
func (rl *RateLimiter) GetConfig(tenantID string) *TenantConfig {
	cfg := rl.tenants[tenantID]
	return &cfg
}

// DisableAll disables rate limiting for all tenants.
func (rl *RateLimiter) DisableAll() {
	for _, cfg := range rl.tenants {
		cfg.Enabled = false
	}
}

// Stats returns a summary string with tenant count and total capacity.
func (rl *RateLimiter) Stats() string {
	var totalCapacity int
	for _, cfg := range rl.tenants {
		totalCapacity += cfg.RequestsPerMin
	}
	return fmt.Sprintf("tenants=%d, totalCapacity=%d req/min", len(rl.tenants), totalCapacity)
}

// ValidateLimit checks if a per-second rate is within platform bounds.
// Max platform-wide is 1 million requests per minute.
func ValidateLimit(perSecond int) bool {
	perMinute := perSecond * 60
	return perMinute > 0 && perMinute <= 1_000_000
}

// MaxBurst calculates burst size as a percentage of the per-minute limit.
// burstPct is 0-100.
func MaxBurst(requestsPerMin int, burstPct int) int {
	return requestsPerMin * burstPct / 100
}

// TierFromString converts a string to a Tier.
func TierFromString(s string) Tier {
	switch s {
	case "free":
		return Free
	case "pro":
		return Pro
	case "enterprise":
		return Enterprise
	}
	return math.MinInt // signal "unknown"
}
