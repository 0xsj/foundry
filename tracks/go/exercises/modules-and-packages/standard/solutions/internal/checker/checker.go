// Package checker performs HTTP health checks against a list of URLs.
//
// It is internal because its behavior is specific to the healthcheck tool
// and is not intended for reuse by external modules. The central type is
// Result, which flows outward to the reporter for display.
//
// Design note: checker imports nothing from internal/ or pkg/ — it is a pure
// function of (url, timeout) → Result. This makes it independently testable
// without any other part of the system.
package checker

import (
	"net/http"
	"time"
)

// Result represents the outcome of a single HTTP health check.
type Result struct {
	// URL is the endpoint that was checked.
	URL string

	// StatusCode is the HTTP response status code, or 0 on network error.
	StatusCode int

	// Latency is the round-trip time for the request.
	Latency time.Duration

	// Err holds any network or transport error. nil on success.
	Err error
}

// IsHealthy returns true when the check completed without error and the
// server responded with a 2xx status code.
func (r Result) IsHealthy() bool {
	return r.Err == nil && r.StatusCode >= 200 && r.StatusCode < 300
}

// Check performs an HTTP GET to url with the given timeout and returns a Result.
// It is safe to call concurrently — each call creates its own HTTP client.
func Check(url string, timeoutSec float64) Result {
	client := &http.Client{
		Timeout: time.Duration(timeoutSec * float64(time.Second)),
	}

	start := time.Now()
	resp, err := client.Get(url) //nolint:noctx
	latency := time.Since(start)

	if err != nil {
		return Result{
			URL:     url,
			Latency: latency,
			Err:     err,
		}
	}
	defer resp.Body.Close()

	return Result{
		URL:        url,
		StatusCode: resp.StatusCode,
		Latency:    latency,
	}
}

// CheckAll runs Check for each URL sequentially and returns all results.
// URLs are checked in order; the slice length equals len(urls).
func CheckAll(urls []string, timeoutSec float64) []Result {
	results := make([]Result, 0, len(urls))
	for _, url := range urls {
		results = append(results, Check(url, timeoutSec))
	}
	return results
}
