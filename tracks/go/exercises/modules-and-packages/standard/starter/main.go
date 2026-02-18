// Package main is the entry point for the healthcheck CLI tool.
// This is the monolithic version — everything lives in one file.
// Your task: split this into the multi-package structure described in README.md.
//
// Run with:
//
//	go run . --urls https://go.dev,https://pkg.go.dev --timeout 5
//
// Or with environment variables:
//
//	HEALTHCHECK_URLS="https://go.dev,https://pkg.go.dev" go run .
package main

import (
	"flag"
	"fmt"
	"net/http"
	"os"
	"strings"
	"time"
)

// ============================================================================
// TYPES — currently all in one file. After refactoring, Result moves to
// internal/checker and Config moves to pkg/config.
// ============================================================================

// Config holds the runtime configuration for the health checker.
type Config struct {
	URLs       []string
	TimeoutSec float64
	Verbose    bool
}

// Result represents the outcome of checking a single URL.
type Result struct {
	URL        string
	StatusCode int
	Latency    time.Duration
	Err        error
}

// IsHealthy returns true if the check succeeded with a 2xx status code.
func (r Result) IsHealthy() bool {
	return r.Err == nil && r.StatusCode >= 200 && r.StatusCode < 300
}

// ============================================================================
// CONFIG LOADING — move to pkg/config/config.go
// ============================================================================

const (
	defaultTimeout = 10.0
)

// loadConfig reads configuration from flag overrides and environment variables.
// Environment variables:
//
//	HEALTHCHECK_URLS    — comma-separated list of URLs to check
//	HEALTHCHECK_TIMEOUT — timeout in seconds (float)
//	HEALTHCHECK_VERBOSE — "true" or "1" for verbose output
func loadConfig() (Config, error) {
	var urlFlag string
	var timeoutFlag float64
	var verboseFlag bool

	flag.StringVar(&urlFlag, "urls", "", "comma-separated list of URLs to check")
	flag.Float64Var(&timeoutFlag, "timeout", 0, "request timeout in seconds")
	flag.BoolVar(&verboseFlag, "verbose", false, "verbose output")
	flag.Parse()

	// Environment variable fallbacks
	if urlFlag == "" {
		urlFlag = os.Getenv("HEALTHCHECK_URLS")
	}
	if timeoutFlag == 0 {
		// In a real version we'd parse this from env; using default for now
		timeoutFlag = defaultTimeout
	}
	if !verboseFlag {
		verboseFlag = os.Getenv("HEALTHCHECK_VERBOSE") == "true" ||
			os.Getenv("HEALTHCHECK_VERBOSE") == "1"
	}

	if urlFlag == "" {
		return Config{}, fmt.Errorf("no URLs provided: use --urls or HEALTHCHECK_URLS env var")
	}

	urls := strings.Split(urlFlag, ",")
	for i, u := range urls {
		urls[i] = strings.TrimSpace(u)
	}

	return Config{
		URLs:       urls,
		TimeoutSec: timeoutFlag,
		Verbose:    verboseFlag,
	}, nil
}

// ============================================================================
// HEALTH CHECK LOGIC — move to internal/checker/checker.go
// ============================================================================

// checkURL performs an HTTP GET to the given URL and returns a Result.
// It uses the provided timeout in seconds.
func checkURL(url string, timeoutSec float64) Result {
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

// checkAll runs checkURL for each URL and returns all results.
func checkAll(urls []string, timeoutSec float64) []Result {
	results := make([]Result, 0, len(urls))
	for _, url := range urls {
		results = append(results, checkURL(url, timeoutSec))
	}
	return results
}

// ============================================================================
// REPORTING — move to internal/reporter/reporter.go
// ============================================================================

// printResults formats and prints a table of check results to stdout.
func printResults(results []Result) {
	// Header
	fmt.Printf("%-45s  %-6s  %-8s  %s\n", "URL", "STATUS", "LATENCY", "HEALTHY")
	fmt.Println(strings.Repeat("-", 75))

	var healthy, unhealthy int

	for _, r := range results {
		status := fmt.Sprintf("%d", r.StatusCode)
		latency := fmt.Sprintf("%.0fms", float64(r.Latency.Milliseconds()))
		isHealthy := "yes"

		if r.Err != nil {
			status = "ERR"
			isHealthy = "no"
			latency = fmt.Sprintf("%.0fms", float64(r.Latency.Milliseconds()))
		} else if !r.IsHealthy() {
			isHealthy = "no"
		}

		fmt.Printf("%-45s  %-6s  %-8s  %s\n", r.URL, status, latency, isHealthy)

		if r.IsHealthy() {
			healthy++
		} else {
			unhealthy++
		}
	}

	// Summary line
	fmt.Println(strings.Repeat("-", 75))
	fmt.Printf("checked %d URLs: %d healthy, %d unhealthy\n",
		len(results), healthy, unhealthy)
}

// ============================================================================
// ENTRY POINT — cmd/healthcheck/main.go after refactoring
// ============================================================================

func main() {
	cfg, err := loadConfig()
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		fmt.Fprintf(os.Stderr, "usage: healthcheck --urls <url1,url2,...> [--timeout <secs>]\n")
		os.Exit(1)
	}

	if cfg.Verbose {
		fmt.Printf("checking %d URLs with %.1fs timeout...\n", len(cfg.URLs), cfg.TimeoutSec)
	}

	results := checkAll(cfg.URLs, cfg.TimeoutSec)
	printResults(results)

	// Exit with non-zero status if any check failed
	for _, r := range results {
		if !r.IsHealthy() {
			os.Exit(1)
		}
	}
}
