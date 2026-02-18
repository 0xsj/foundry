// Package httpclient provides a retrying HTTP client for platform services.
// This is the proposed PR for review — find the error handling issues.
package httpclient

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// Config holds client configuration.
type Config struct {
	BaseURL    string
	AuthToken  string
	MaxRetries int
	Timeout    time.Duration
}

// Response is the structured response from Do.
type Response struct {
	StatusCode int
	Body       []byte
	Headers    http.Header
}

// ============================================================================
// ISSUE 1: Error type is unexported but used in public API
//
// httpError is the structured error type for non-2xx responses.
// Problem: it's unexported (lowercase). Callers of this package cannot
// use errors.As(err, &httpError{}) because they can't name the type.
// ============================================================================

type httpError struct { // should be HTTPError or HttpError
	StatusCode int
	Body       string
	URL        string
}

func (e *httpError) Error() string {
	return fmt.Sprintf("HTTP %d from %s: %s", e.StatusCode, e.URL, e.Body)
}

// ============================================================================
// ISSUE 2: Bare string errors — no sentinel, no type
//
// These errors are created ad-hoc with no consistent type or sentinel.
// Callers have no way to programmatically detect "is this a timeout?"
// or "is this a config error?" without parsing the error string.
// ============================================================================

// Do executes an HTTP request with retries.
func Do(cfg *Config, method, path string, body io.Reader) (*Response, error) {
	// ISSUE 3: panic for nil config — should be a returned error in a library
	if cfg == nil {
		panic("config must not be nil")
	}

	if method == "" {
		// Inconsistency: this one returns an error (correct), but nil config panics
		return nil, errors.New("method is required")
	}

	var lastErr error
	for attempt := 0; attempt <= cfg.MaxRetries; attempt++ {
		resp, err := doOnce(cfg, method, path, body)
		if err != nil {
			// ISSUE 4: wraps with %v not %w — severs the chain on retry
			lastErr = fmt.Errorf("attempt %d: %v", attempt+1, err)
			time.Sleep(backoff(attempt))
			continue
		}
		return resp, nil
	}

	// ISSUE 5: error message is capitalized and ends with period — non-idiomatic
	return nil, fmt.Errorf("All %d attempts failed. Last error: %v", cfg.MaxRetries+1, lastErr)
}

// doOnce executes a single HTTP attempt.
func doOnce(cfg *Config, method, path string, body io.Reader) (*Response, error) {
	url := cfg.BaseURL + path

	req, err := http.NewRequest(method, url, body)
	if err != nil {
		// ISSUE 6: %v loses the *url.Error that http.NewRequest can return
		return nil, fmt.Errorf("build request: %v", err)
	}

	req.Header.Set("Authorization", "Bearer "+cfg.AuthToken)

	client := &http.Client{Timeout: cfg.Timeout}
	httpResp, err := client.Do(req)
	if err != nil {
		// ISSUE 7: %v again — callers can't use errors.Is(err, context.DeadlineExceeded)
		// to detect timeouts
		return nil, fmt.Errorf("execute request: %v", err)
	}
	defer httpResp.Body.Close()

	respBody, err := io.ReadAll(httpResp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err) // this one uses %w (correct)
	}

	if httpResp.StatusCode < 200 || httpResp.StatusCode >= 300 {
		return nil, &httpError{
			StatusCode: httpResp.StatusCode,
			Body:       strings.TrimSpace(string(respBody)),
			URL:        url,
		}
	}

	return &Response{
		StatusCode: httpResp.StatusCode,
		Body:       respBody,
		Headers:    httpResp.Header,
	}, nil
}

// backoff returns the delay before the next retry.
func backoff(attempt int) time.Duration {
	if attempt == 0 {
		return 100 * time.Millisecond
	}
	return time.Duration(attempt) * 200 * time.Millisecond
}

// IsNotFound checks if an error is a 404 response.
// ISSUE 8: checks error string instead of using errors.As — fragile
func IsNotFound(err error) bool {
	return err != nil && strings.Contains(err.Error(), "HTTP 404")
}

// IsServerError checks if an error is a 5xx response.
// ISSUE 8 continued: same string-parsing approach
func IsServerError(err error) bool {
	return err != nil && strings.Contains(err.Error(), "HTTP 5")
}
