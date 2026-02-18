// Command healthcheck pings a list of URLs and reports their HTTP status.
//
// Usage:
//
//	HEALTHCHECK_URLS="https://go.dev,https://pkg.go.dev" healthcheck
//	HEALTHCHECK_URLS="https://go.dev" HEALTHCHECK_TIMEOUT=5 healthcheck
//	HEALTHCHECK_VERBOSE=true HEALTHCHECK_URLS="https://go.dev" healthcheck
//
// Exit codes:
//
//	0 — all URLs are healthy
//	1 — one or more URLs are unhealthy or an error occurred
package main

import (
	"fmt"
	"os"

	"github.com/foundry/healthcheck/internal/checker"
	"github.com/foundry/healthcheck/internal/reporter"
	"github.com/foundry/healthcheck/pkg/config"
)

func main() {
	// Load configuration from environment variables.
	// envFromOS converts os.Environ() into the map[string]string config.Load expects.
	cfg, err := config.Load(envFromOS())
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		fmt.Fprintf(os.Stderr, "usage: HEALTHCHECK_URLS=\"url1,url2\" healthcheck\n")
		os.Exit(1)
	}

	if cfg.Verbose {
		fmt.Printf("checking %d URLs with %.1fs timeout...\n",
			len(cfg.URLs), cfg.TimeoutSec)
	}

	// Run checks and format output. cmd/healthcheck knows nothing about
	// HTTP mechanics (checker's job) or table formatting (reporter's job).
	results := checker.CheckAll(cfg.URLs, cfg.TimeoutSec)
	reporter.PrintResults(results)

	// Exit non-zero if any check failed — enables use in CI pipelines and
	// shell scripts: healthcheck && deploy.sh
	for _, r := range results {
		if !r.IsHealthy() {
			os.Exit(1)
		}
	}
}

// envFromOS converts the process environment into a map for config.Load.
// Using a map (rather than os.Getenv directly in config) makes config
// independently testable without touching the real environment.
func envFromOS() map[string]string {
	env := make(map[string]string)
	for _, e := range os.Environ() {
		// os.Environ() returns "KEY=VALUE" strings
		for i, c := range e {
			if c == '=' {
				env[e[:i]] = e[i+1:]
				break
			}
		}
	}
	return env
}
