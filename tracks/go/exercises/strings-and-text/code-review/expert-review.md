# Expert Review: PR #847 — Health Report Generator

## Critical Issues

### 1. Using `text/template` for HTML output — XSS vulnerability

**File:** `proposed.go`, import line

The package imports `"text/template"` but renders HTML that includes user-controlled data (`ServiceHealth.Name`, `ServiceHealth.Status`, `ServiceHealth.Region`, `ServiceHealth.ErrorLogs`). `text/template` does **no escaping** — it renders values verbatim.

If a service name is set to `<script>alert(document.cookie)</script>` (which an attacker could do by compromising a service registry or metric label), the HTML report served to on-call engineers would execute that script. This is a textbook XSS vulnerability.

**Fix:** Replace `"text/template"` with `"html/template"`:

```go
import "html/template"  // NOT text/template
```

The API is identical. `html/template` performs contextual escaping: values in HTML element context are HTML-escaped, values in URL attributes are URL-escaped, values in `<script>` context are JS-escaped. The template does not need to change — only the import.

```go
// With html/template, if Name = `<script>alert(1)</script>`:
// Renders as: <h1>&lt;script&gt;alert(1)&lt;/script&gt;</h1>
// Not as:     <h1><script>alert(1)</script></h1>
```

**Note:** `text/template` and `html/template` have identical function signatures. The only difference is the import path. This is exactly why the bug is so easy to introduce — autocomplete or a copy-paste can silently pick the wrong package.

**Severity: Critical.** This renders a security monitoring tool susceptible to being turned against the engineers using it.

---

## Major Concerns

### 2. Template parsed on every `GenerateReport` call

**File:** `GenerateReport` function, inside the function body

```go
func GenerateReport(svc ServiceHealth) (Report, error) {
    t, err := template.New("health").Parse(reportTemplate)  // ← here
    ...
}
```

`template.Parse` is not cheap — it lexes and parses the template text, validates the syntax, and compiles it into an internal representation. Calling this every 30 seconds for 100 services means 200 unnecessary template compilations per minute.

**Fix:** Parse the template once at package level:

```go
var tmpl = template.Must(
    template.New("health").Parse(reportTemplate),
)

func GenerateReport(svc ServiceHealth) (Report, error) {
    var sb strings.Builder
    if err := tmpl.Execute(&sb, svc); err != nil {
        return Report{}, fmt.Errorf("executing template: %w", err)
    }
    summary := buildSummaryLine(svc)
    return Report{HTML: sb.String(), Summary: summary}, nil
}
```

`template.Must` panics if parsing fails — appropriate here because the template is a static string literal. If it fails, it's a programmer error (caught immediately at startup), not a runtime condition.

The parsed `*template.Template` is safe for concurrent use without synchronization.

**Severity: Major.** Functionally correct but wasteful. For a system that runs continuously, this is an ongoing CPU and GC tax.

---

### 3. `buildSummaryLine` builds a string with serial `fmt.Sprintf` calls

**File:** `buildSummaryLine` function

```go
result := ""
result = fmt.Sprintf("%s", result+"service="+svc.Name)
result = fmt.Sprintf("%s", result+" status="+svc.Status)
result = fmt.Sprintf("%s", result+" region="+svc.Region)
result = fmt.Sprintf("%s", result+" errors="+strconv.Itoa(len(svc.ErrorLogs)))
```

This has multiple problems:
1. `fmt.Sprintf("%s", x)` where `x` is already a string is an anti-pattern — it allocates a string to copy a string. `fmt.Sprintf` is for formatting with mixed types and verbs; it's overkill here.
2. Each `result + "..."` concatenation inside `Sprintf` allocates a new intermediate string.
3. The `strconv.Itoa` is fine, but `fmt.Sprintf` accepts `%d` directly.

This is calling `buildSummaryLine` once per report — it's not as performance-critical as the template parsing, but it's still unnecessarily complex. The simplest fix is `fmt.Sprintf` used correctly:

```go
func buildSummaryLine(svc ServiceHealth) string {
    return fmt.Sprintf("service=%s status=%s region=%s errors=%d",
        svc.Name, svc.Status, svc.Region, len(svc.ErrorLogs))
}
```

Or with `strings.Builder` if you prefer to avoid `fmt`:
```go
func buildSummaryLine(svc ServiceHealth) string {
    var b strings.Builder
    b.Grow(64)
    fmt.Fprintf(&b, "service=%s status=%s region=%s errors=%d",
        svc.Name, svc.Status, svc.Region, len(svc.ErrorLogs))
    return b.String()
}
```

**Severity: Major.** The current code is confusing (why is `Sprintf("%s", ...)` being used?), harder to read than a single `Sprintf` call, and allocates more than necessary.

---

## Minor Suggestions

### 4. `countErrors` converts `[]byte` ↔ `string` unnecessarily

**File:** `countErrors` function

```go
logBytes := []byte(log)
keywordBytes := []byte(keyword)
if strings.Contains(string(logBytes), string(keywordBytes)) {
```

`log` and `keyword` are already `string`. Converting them to `[]byte` and immediately back to `string` is pure noise — it allocates two `[]byte` slices and two `string` copies for no reason. Just call `strings.Contains` directly:

```go
func countErrors(logs []string, keyword string) int {
    count := 0
    for _, log := range logs {
        if strings.Contains(log, keyword) {
            count++
        }
    }
    return count
}
```

The function is currently unexported (lowercase), so this is an internal cleanup. Still worth fixing to avoid confusing future readers.

**Severity: Minor.** No user-visible impact; just allocation noise and confusing code.

---

### 5. `countErrors` is defined but never called

The function exists in the PR but is not used anywhere in the package. Go will reject this with a compiler error if the function is in its own file (`declared and not used`), but since it's in the same file as used code, it compiles.

If this is dead code, remove it. If it's intended for future use, add a TODO comment. Dead code in security-adjacent packages is particularly undesirable — it's harder to audit.

---

## Positive Feedback

- Using `strings.Builder` for template output is correct and efficient. `strings.Builder` is the right tool when writing to an `io.Writer` that will be converted to `string`.
- Error wrapping with `fmt.Errorf("...: %w", err)` is idiomatic — callers can use `errors.Is`/`errors.As`.
- The `ServiceHealth` struct is clean and well-named. Separating `HTML` and `Summary` in `Report` is good — callers may need only one.
- Exporting `ServiceHealth` and `Report` but not internal helpers is appropriate package design.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `text/template` for HTML — XSS vulnerability | `html/template` auto-escaping |
| 2 | Major | Template parsed every call — parse at package level | `template.Must` at package level |
| 3 | Major | `buildSummaryLine` — serial Sprintf anti-pattern | `fmt.Sprintf` used correctly |
| 4 | Minor | `countErrors` converts string → []byte → string unnecessarily | Unnecessary conversions |
| 5 | Minor | `countErrors` is dead code | Code hygiene |

The critical issue (wrong template package) is a common Go footgun precisely because the two packages have identical APIs. The fix is a one-line import change. This should block merge until addressed.
