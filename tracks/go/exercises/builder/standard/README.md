# Standard Exercise: HTTP Request Builder

## Scenario

You are building an internal API client SDK for a microservices platform. Each service needs to make HTTP requests to other services with consistent authentication, retry behavior, timeouts, and tracing headers. Rather than constructing `http.Request` objects manually every time, you are creating a request builder that encapsulates the boilerplate and validates requests before they are sent.

## Brief

Implement two complementary builders:

1. **`Client`** -- configured via functional options. Holds base URL, default headers, authentication, and default timeout. Created once, reused for many requests.

2. **`RequestBuilder`** -- configured via fluent method chaining. Builds individual HTTP requests with method, path, headers, query params, body, and per-request overrides. Created from a `Client` and used to build a single `*http.Request`.

## Acceptance Criteria

### Client (Functional Options)

- [ ] `NewClient(baseURL string, opts ...ClientOption) (*Client, error)` constructor
- [ ] `WithDefaultHeader(key, value string)` -- adds a header sent on every request
- [ ] `WithBearerToken(token string)` -- sets Authorization header on all requests
- [ ] `WithTimeout(d time.Duration)` -- sets default timeout
- [ ] `WithRetries(n int)` -- sets default retry count (0 = no retries)
- [ ] `WithUserAgent(ua string)` -- sets User-Agent header
- [ ] Validation: baseURL must be non-empty, timeout must be positive, retries must be >= 0
- [ ] `Client.Request()` returns a new `*RequestBuilder` pre-configured with client defaults

### RequestBuilder (Fluent Chaining)

- [ ] `Get(path string)`, `Post(path string)`, `Put(path string)`, `Delete(path string)` -- set HTTP method and path
- [ ] `Header(key, value string)` -- add a request header (merges with client defaults)
- [ ] `QueryParam(key, value string)` -- add a URL query parameter
- [ ] `Body(data []byte)` -- set request body (raw bytes)
- [ ] `JSONBody(v any)` -- marshal value as JSON body and set Content-Type header
- [ ] `Timeout(d time.Duration)` -- override client timeout for this request
- [ ] `Build(ctx context.Context)` -- validate and return `*http.Request, error`

### Validation at Build()

- [ ] Method must be set (GET, POST, PUT, DELETE)
- [ ] Path must be non-empty
- [ ] GET and DELETE requests must not have a body
- [ ] POST and PUT requests should warn (not error) if body is empty
- [ ] Constructed URL must be valid (baseURL + path + query params)

### Tests

All tests in `main_test.go` must pass.

## Constraints

- Do not make actual HTTP calls -- the builder produces `*http.Request` objects only
- Use `encoding/json` for JSONBody
- Use `net/url` for URL construction and query param encoding
- The builder must be safe to use sequentially (not concurrently -- that is a different exercise)

## Hints

<details>
<summary>Hint 1: Client structure</summary>

The `Client` should store defaults that get copied into each `RequestBuilder`:

```go
type Client struct {
    baseURL    string
    headers    http.Header
    timeout    time.Duration
    retries    int
}
```

</details>

<details>
<summary>Hint 2: RequestBuilder accumulates state</summary>

```go
type RequestBuilder struct {
    client  *Client
    method  string
    path    string
    headers http.Header
    query   url.Values
    body    []byte
    timeout time.Duration
    err     error
}
```

The `err` field captures the first error so Build() can report it.

</details>

<details>
<summary>Hint 3: Merging headers</summary>

Client defaults are the base. Request-level headers override or add to them:

```go
func (rb *RequestBuilder) Build(ctx context.Context) (*http.Request, error) {
    // Start with client defaults
    merged := rb.client.headers.Clone()
    // Add request-specific headers (overrides client defaults)
    for key, values := range rb.headers {
        merged[key] = values
    }
    // ...
}
```

</details>
