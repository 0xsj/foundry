// Parsing — strconv, regexp, template basics.
//
// Run with: go run ./parsing/
package main

import (
	"fmt"
	htmltemplate "html/template"
	"log"
	"os"
	"regexp"
	"strconv"
	"strings"
	"text/template"
)

func main() {
	strconvExamples()
	regexpExamples()
	regexpHotLoopTrap()
	textTemplateExamples()
	htmlTemplateEscaping()
}

// ============================================================================
// strconv — type conversions
// ============================================================================

func strconvExamples() {
	fmt.Println("=== strconv ===")

	// String → int: Atoi is the shorthand, ParseInt gives full control
	n, err := strconv.Atoi("42")
	if err != nil {
		log.Fatalf("Atoi: %v", err)
	}
	fmt.Printf("Atoi(%q) = %d\n", "42", n)

	// Always check the error — invalid input silently returns 0 otherwise
	_, err = strconv.Atoi("not-a-number")
	fmt.Printf("Atoi(\"not-a-number\") error: %v\n", err)

	// ParseInt: specify base and bit size
	// base=16 for hex, base=2 for binary, base=0 for auto-detect from prefix
	hex, _ := strconv.ParseInt("FF", 16, 64)
	fmt.Printf("ParseInt(\"FF\", 16) = %d\n", hex)
	bin, _ := strconv.ParseInt("1010", 2, 64)
	fmt.Printf("ParseInt(\"1010\", 2) = %d\n", bin)
	auto, _ := strconv.ParseInt("0xFF", 0, 64) // auto-detects hex prefix
	fmt.Printf("ParseInt(\"0xFF\", 0) = %d\n", auto)

	// int → string
	fmt.Printf("Itoa(42) = %q\n", strconv.Itoa(42))
	fmt.Printf("FormatInt(255, 16) = %q\n", strconv.FormatInt(255, 16))
	fmt.Printf("FormatInt(10, 2) = %q\n", strconv.FormatInt(10, 2))

	// Float conversions
	f, err := strconv.ParseFloat("3.14159", 64)
	fmt.Printf("\nParseFloat(\"3.14159\") = %f err=%v\n", f, err)
	fmt.Printf("FormatFloat(3.14159, 'f', 2, 64) = %q\n",
		strconv.FormatFloat(3.14159, 'f', 2, 64))
	fmt.Printf("FormatFloat(3.14159, 'e', 3, 64) = %q\n",
		strconv.FormatFloat(3.14159, 'e', 3, 64))

	// Bool conversions
	b, _ := strconv.ParseBool("true")
	fmt.Printf("\nParseBool(\"true\") = %v\n", b)
	b, _ = strconv.ParseBool("1")
	fmt.Printf("ParseBool(\"1\") = %v\n", b)
	fmt.Printf("FormatBool(false) = %q\n", strconv.FormatBool(false))

	// Practical example: parsing a health check response
	fmt.Println("\n-- parsing health check response --")
	response := "uptime_seconds=864000 error_rate=0.0042 healthy=true"
	parseHealthCheck(response)

	fmt.Println()
}

// parseHealthCheck demonstrates real-world strconv usage: parsing
// key=value pairs from a service health endpoint response.
func parseHealthCheck(response string) {
	pairs := strings.Fields(response)
	for _, pair := range pairs {
		key, value, ok := strings.Cut(pair, "=")
		if !ok {
			continue
		}
		switch key {
		case "uptime_seconds":
			n, err := strconv.ParseInt(value, 10, 64)
			if err != nil {
				fmt.Printf("  %s: invalid int %q\n", key, value)
				continue
			}
			fmt.Printf("  uptime: %d seconds (%d hours)\n", n, n/3600)
		case "error_rate":
			f, err := strconv.ParseFloat(value, 64)
			if err != nil {
				fmt.Printf("  %s: invalid float %q\n", key, value)
				continue
			}
			fmt.Printf("  error_rate: %.4f (%.2f%%)\n", f, f*100)
		case "healthy":
			b, err := strconv.ParseBool(value)
			if err != nil {
				fmt.Printf("  %s: invalid bool %q\n", key, value)
				continue
			}
			status := "UP"
			if !b {
				status = "DOWN"
			}
			fmt.Printf("  status: %s\n", status)
		}
	}
}

// ============================================================================
// regexp — regular expressions
// ============================================================================

// Compile patterns once at package level — compiled once at program start,
// reused for every call. This is the correct pattern for static patterns.
var (
	// Matches: 2024-01-15T14:30:00Z  or  2024-01-15T14:30:00+05:30
	reTimestamp = regexp.MustCompile(
		`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})`)

	// Matches: key=value or key="quoted value"
	reKeyValue = regexp.MustCompile(`(\w[\w.-]*)=("(?:[^"\\]|\\.)*"|\S+)`)

	// Named groups: capture year, month, day separately
	reDate = regexp.MustCompile(`(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})`)
)

func regexpExamples() {
	fmt.Println("=== regexp ===")

	// MatchString: boolean check
	logLine := `2024-01-15T14:30:00Z ERROR payment-svc "connection refused"`
	fmt.Printf("HasTimestamp: %v\n", reTimestamp.MatchString(logLine))

	// FindString: extract first match
	ts := reTimestamp.FindString(logLine)
	fmt.Printf("Timestamp: %q\n", ts)

	// FindAllString: extract all matches (n=-1 means all)
	s := "retry_count=3 timeout_ms=5000 region=us-east-1"
	allKV := reKeyValue.FindAllString(s, -1)
	fmt.Printf("All key=value pairs: %v\n", allKV)

	// FindStringSubmatch: full match + capture groups
	// m[0] = full match, m[1] = group 1, m[2] = group 2, ...
	m := reKeyValue.FindStringSubmatch(`service="payment svc"`)
	if m != nil {
		fmt.Printf("FindStringSubmatch: full=%q key=%q value=%q\n", m[0], m[1], m[2])
	}

	// Named groups: more readable than positional indices
	dateStr := "2024-01-15"
	namedMatch := reDate.FindStringSubmatch(dateStr)
	names := reDate.SubexpNames()
	result := make(map[string]string)
	for i, name := range names {
		if i != 0 && name != "" {
			result[name] = namedMatch[i]
		}
	}
	fmt.Printf("Named groups: %v\n", result)

	// ReplaceAllString: redact sensitive values
	raw := `user=alice token=secret123 action=login token=anothersecret`
	re := regexp.MustCompile(`token=\S+`)
	redacted := re.ReplaceAllString(raw, "token=[REDACTED]")
	fmt.Printf("Redacted: %q\n", redacted)

	// ReplaceAllStringFunc: dynamic replacement
	// Double all numeric values in a query string
	queryStr := "timeout=30 retries=3 delay=100"
	doubled := regexp.MustCompile(`\d+`).ReplaceAllStringFunc(queryStr, func(s string) string {
		n, _ := strconv.Atoi(s)
		return strconv.Itoa(n * 2)
	})
	fmt.Printf("Doubled values: %q\n", doubled)

	fmt.Println()
}

// ============================================================================
// The Hot Loop Trap
// ============================================================================

func regexpHotLoopTrap() {
	fmt.Println("=== regexp Hot Loop Trap ===")

	lines := []string{
		"2024-01-15 INFO server started",
		"2024-01-15 ERROR connection refused",
		"2024-01-15 WARN rate limit exceeded",
	}

	// BAD: compiles the regexp on every iteration.
	// regexp.MustCompile builds a finite automaton — expensive.
	// In a loop over 10,000 lines, this is 10,000 compilations.
	fmt.Println("BAD: compile in loop (don't do this):")
	errorCount := 0
	for _, line := range lines {
		re := regexp.MustCompile(`\bERROR\b`) // compiled fresh every iteration!
		if re.MatchString(line) {
			errorCount++
		}
	}
	fmt.Printf("  error lines: %d\n", errorCount)

	// GOOD: compile once before the loop.
	// The compiled *Regexp is safe for concurrent use.
	fmt.Println("GOOD: compile once before loop:")
	reError := regexp.MustCompile(`\bERROR\b`) // compiled once
	errorCount = 0
	for _, line := range lines {
		if reError.MatchString(line) { // reuse compiled regexp
			errorCount++
		}
	}
	fmt.Printf("  error lines: %d\n", errorCount)

	// BEST for static patterns: package-level variable (see reTimestamp above).
	// Compiled at program start, zero overhead at call time.
	fmt.Println("BEST: package-level variable (see var block above)")

	fmt.Println()
}

// ============================================================================
// text/template — data-driven text generation
// ============================================================================

const alertTmpl = `
=== Service Alert ===
Service:    {{.ServiceName}}
Severity:   {{.Severity}}
Region:     {{.Region}}
Message:    {{.Message}}
Runbook:    https://runbook.internal/{{.ServiceName}}/{{lower .Severity}}
`

type Alert struct {
	ServiceName string
	Severity    string
	Region      string
	Message     string
}

func textTemplateExamples() {
	fmt.Println("=== text/template ===")

	// Add a custom function: lower-cases a string
	// Custom functions let you keep templates readable without embedding logic
	funcMap := template.FuncMap{
		"lower": strings.ToLower,
	}

	t := template.Must(
		template.New("alert").Funcs(funcMap).Parse(alertTmpl),
	)

	alert := Alert{
		ServiceName: "payment-svc",
		Severity:    "CRITICAL",
		Region:      "us-east-1",
		Message:     "Database connection pool exhausted",
	}

	// Execute writes to any io.Writer — here, stdout
	if err := t.Execute(os.Stdout, alert); err != nil {
		log.Fatalf("template: %v", err)
	}

	// Execute to a strings.Builder — get the result as a string
	var sb strings.Builder
	if err := t.Execute(&sb, Alert{
		ServiceName: "auth-svc",
		Severity:    "WARNING",
		Region:      "eu-west-1",
		Message:     "Elevated error rate",
	}); err != nil {
		log.Fatalf("template: %v", err)
	}
	// sb.String() now has the rendered template — ready to send via email, Slack, etc.
	fmt.Printf("Rendered to string (length %d bytes)\n", len(sb.String()))
}

// ============================================================================
// html/template — auto-escaping prevents XSS
// ============================================================================

const profileTmpl = `<!DOCTYPE html>
<html>
<head><title>Profile</title></head>
<body>
  <h1>Welcome, {{.Name}}!</h1>
  <p>Role: {{.Role}}</p>
</body>
</html>`

type UserProfile struct {
	Name string
	Role string
}

func htmlTemplateEscaping() {
	fmt.Println("\n=== html/template — auto-escaping ===")

	// html/template has the identical API to text/template.
	// The critical difference: it auto-escapes values based on context.
	t := htmltemplate.Must(htmltemplate.New("profile").Parse(profileTmpl))

	// Benign user data — renders normally
	fmt.Println("Normal user:")
	t.Execute(os.Stdout, UserProfile{Name: "Alice", Role: "admin"}) //nolint

	// Malicious user data — the XSS payload is automatically escaped
	// html/template renders < as &lt;, > as &gt;, " as &#34;, etc.
	fmt.Println("\nMalicious user (attempt XSS):")
	t.Execute(os.Stdout, UserProfile{ //nolint
		Name: `<script>alert("xss")</script>`,
		Role: `admin" onclick="steal()`,
	})
	// Output: &lt;script&gt;alert(&#34;xss&#34;)&lt;/script&gt;
	// The browser receives escaped text, not executable script.

	fmt.Println()
	fmt.Println("Lesson: always use html/template for HTML output.")
	fmt.Println("text/template and html/template have the same API — it's an easy mistake.")
}
