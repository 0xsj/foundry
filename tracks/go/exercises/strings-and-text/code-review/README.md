# Code Review: Template-Based Report Generator

## PR Context

**PR #847 — "Add HTML report generator for service health dashboard"**

A teammate has added a new package that generates HTML health reports for the service monitoring dashboard. The package takes structured health data and renders it into an HTML page that's served to on-call engineers.

The PR touches security-sensitive code (HTML generation), performance-sensitive code (runs every 30 seconds per service), and introduces a new dependency pattern. Give it a thorough review.

## What to Review

Read `proposed.go` and leave your review in `my-review.md`.

Focus on:
1. **Correctness and security** — does it behave as documented? Are there any security vulnerabilities?
2. **Performance** — given this runs every 30 seconds for potentially 100 services, are there efficiency concerns?
3. **Go idioms** — does it follow Go conventions? Is it idiomatic?
4. **Maintainability** — is it clear, testable, and easy to change?

## Review Guide

<details>
<summary>Hint 1: Security</summary>
Look carefully at the import statement. There are two template packages in Go's standard library. Think about what happens when user-controlled data (service names, error messages) is rendered into an HTML page.
</details>

<details>
<summary>Hint 2: Performance — function-level resource creation</summary>
Look at what happens every time `GenerateReport` is called. Are there expensive operations that could be done once at package level or during initialization instead?
</details>

<details>
<summary>Hint 3: String building</summary>
Look at `buildSummaryLine`. Is the approach appropriate for the task? Think about what `fmt.Sprintf` does internally and whether there's a better-suited function here.
</details>

<details>
<summary>Hint 4: Unnecessary conversions</summary>
Look for places where data is converted between types when a simpler path exists. In particular, look at `countErrors`.
</details>
