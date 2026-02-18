// Package reporter formats and prints health check results to stdout.
//
// It is internal because its output format is specific to this tool.
// It imports internal/checker only for the Result type — it has no
// knowledge of how checks are performed.
//
// If a JSON output mode were added later, it would live here as a second
// function (PrintJSON or a configurable Writer), not in a new package.
package reporter

import (
	"fmt"
	"strings"

	"github.com/foundry/healthcheck/internal/checker"
)

const (
	colWidth     = 45
	statusWidth  = 6
	latencyWidth = 8
)

// PrintResults formats check results as a text table and writes to stdout.
// It prints one row per result, followed by a summary line showing total
// counts of healthy and unhealthy endpoints.
func PrintResults(results []checker.Result) {
	// Header row
	fmt.Printf("%-*s  %-*s  %-*s  %s\n",
		colWidth, "URL",
		statusWidth, "STATUS",
		latencyWidth, "LATENCY",
		"HEALTHY",
	)
	fmt.Println(strings.Repeat("-", colWidth+statusWidth+latencyWidth+12))

	var healthy, unhealthy int

	for _, r := range results {
		status, latency, health := formatResult(r)
		fmt.Printf("%-*s  %-*s  %-*s  %s\n",
			colWidth, r.URL,
			statusWidth, status,
			latencyWidth, latency,
			health,
		)

		if r.IsHealthy() {
			healthy++
		} else {
			unhealthy++
		}
	}

	// Summary line
	fmt.Println(strings.Repeat("-", colWidth+statusWidth+latencyWidth+12))
	fmt.Printf("checked %d URLs: %d healthy, %d unhealthy\n",
		len(results), healthy, unhealthy)
}

// formatResult converts a Result into display strings for the table columns.
// Unexported — only used within this package.
func formatResult(r checker.Result) (status, latency, health string) {
	latency = fmt.Sprintf("%dms", r.Latency.Milliseconds())

	if r.Err != nil {
		return "ERR", latency, "no"
	}

	status = fmt.Sprintf("%d", r.StatusCode)
	if r.IsHealthy() {
		health = "yes"
	} else {
		health = "no"
	}
	return status, latency, health
}
