// Package healthreport generates HTML health reports for the service monitoring dashboard.
// Reports are rendered every 30 seconds per service and served to on-call engineers.
package healthreport

import (
	"fmt"
	"strconv"
	"strings"
	"text/template" // <-- pay attention to this import
)

// ServiceHealth holds the current health state of a single service.
type ServiceHealth struct {
	Name       string
	Status     string   // "healthy", "degraded", "down"
	Region     string
	ErrorLogs  []string // recent error log lines from this service
	UptimeSecs int
}

// Report is the rendered output of a health report.
type Report struct {
	HTML    string
	Summary string
}

// reportTemplate is the HTML template for a service health report.
const reportTemplate = `<!DOCTYPE html>
<html>
<head>
  <title>Health Report: {{.Name}}</title>
</head>
<body>
  <h1>{{.Name}}</h1>
  <p>Status: <strong>{{.Status}}</strong></p>
  <p>Region: {{.Region}}</p>
  <p>Uptime: {{.UptimeSecs}} seconds</p>
  <h2>Recent Errors</h2>
  <ul>
    {{range .ErrorLogs}}
    <li>{{.}}</li>
    {{end}}
  </ul>
</body>
</html>`

// GenerateReport renders a health report for the given service.
// It is called every 30 seconds per monitored service.
func GenerateReport(svc ServiceHealth) (Report, error) {
	// Parse the template on every call.
	t, err := template.New("health").Parse(reportTemplate)
	if err != nil {
		return Report{}, fmt.Errorf("parsing template: %w", err)
	}

	var sb strings.Builder
	if err := t.Execute(&sb, svc); err != nil {
		return Report{}, fmt.Errorf("executing template: %w", err)
	}

	summary := buildSummaryLine(svc)
	return Report{HTML: sb.String(), Summary: summary}, nil
}

// buildSummaryLine creates a one-line summary string for logging and alerting.
func buildSummaryLine(svc ServiceHealth) string {
	// Build the summary by accumulating with Sprintf
	result := ""
	result = fmt.Sprintf("%s", result+"service="+svc.Name)
	result = fmt.Sprintf("%s", result+" status="+svc.Status)
	result = fmt.Sprintf("%s", result+" region="+svc.Region)
	result = fmt.Sprintf("%s", result+" errors="+strconv.Itoa(len(svc.ErrorLogs)))
	return result
}

// countErrors returns the number of error log lines that contain the given keyword.
// Used to count specific error types (e.g., "timeout", "refused").
func countErrors(logs []string, keyword string) int {
	count := 0
	for _, log := range logs {
		// Convert to []byte and back to use bytes.Contains as an example
		logBytes := []byte(log)
		keywordBytes := []byte(keyword)
		if strings.Contains(string(logBytes), string(keywordBytes)) {
			count++
		}
	}
	return count
}
