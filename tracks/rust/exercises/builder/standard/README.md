# Standard Exercise: HTTP Request Builder

## Scenario

You are building the core HTTP client library for your team's internal API gateway. Every microservice uses this client to make outbound HTTP requests. The builder must be ergonomic for simple cases (GET with a URL) and powerful enough for complex cases (POST with headers, auth, timeout, and a JSON body). Misuse -- like sending a request without a URL -- should be caught at compile time, not runtime.

## Brief

Implement an HTTP request builder using the typestate pattern to enforce that the URL is always set before `build()` can be called. The builder should support method selection, headers, query parameters, request body, authentication, and timeout configuration.

## Acceptance Criteria

- [ ] `HttpMethod` enum with variants `Get`, `Post`, `Put`, `Patch`, `Delete`, `Head`
- [ ] `AuthScheme` enum with variants `Bearer(String)`, `Basic { username: String, password: String }`, `ApiKey { header: String, value: String }`
- [ ] `HttpRequest` struct with fields: `method`, `url`, `headers` (Vec of key-value pairs), `query_params` (Vec of key-value pairs), `body` (Option<String>), `auth` (Option<AuthScheme>), `timeout_ms` (Option<u64>)
- [ ] `RequestBuilder` with typestate: `build()` only available after `url()` is called
- [ ] `RequestBuilder::new()` creates builder with `Get` method as default
- [ ] `.url(url)` transitions builder state from `NoUrl` to `HasUrl`
- [ ] `.method(method)` sets the HTTP method (available in any state)
- [ ] `.header(key, value)` adds a header (accumulating, not replacing)
- [ ] `.query(key, value)` adds a query parameter (accumulating)
- [ ] `.body(body)` sets the request body
- [ ] `.bearer_token(token)` sets Bearer auth
- [ ] `.basic_auth(username, password)` sets Basic auth
- [ ] `.api_key(header, value)` sets API key auth
- [ ] `.timeout_ms(ms)` sets the request timeout
- [ ] `.build()` returns `Result<HttpRequest, String>` -- validates that body is not set for GET/HEAD, returns error if it is
- [ ] All string parameters accept `impl Into<String>` for ergonomics
- [ ] `HttpRequest` implements `Debug`

## Constraints

- No external crates -- stdlib only
- All tests in `tests.rs` must pass (compile with `rustc --test starter/main.rs`)
- GET and HEAD requests must not have a body (return error from `build()`)
- Multiple calls to `.header()` accumulate (do not replace previous headers)
- Multiple calls to auth methods replace (last one wins)

## Files

- `starter/main.rs` -- Scaffold with type definitions and TODOs
- `starter/tests.rs` -- Full test suite (do not modify)
- `solutions/solution.rs` -- Reference implementation
- `solutions/solution_test.rs` -- Solution tests (same as starter tests)
- `my-solution/` -- Your implementation

## Getting Started

```bash
# Run the tests (they will fail initially)
cd starter
rustc --test main.rs && ./main

# After implementing, all tests should pass
```

## Hints

<details>
<summary>Hint 1: Typestate markers</summary>

Define two zero-sized marker structs (`NoUrl` and `HasUrl`) and use `PhantomData` to carry them in the builder's type parameter:

```rust
struct NoUrl;
struct HasUrl;

struct RequestBuilder<UrlState> {
    // ... fields ...
    _state: PhantomData<UrlState>,
}
```

The `url()` method is only available on `RequestBuilder<NoUrl>` and returns `RequestBuilder<HasUrl>`.

</details>

<details>
<summary>Hint 2: State transition</summary>

When transitioning state, you need to construct a new builder with the new phantom type. You cannot just change a field -- the entire type changes:

```rust
impl RequestBuilder<NoUrl> {
    fn url(self, url: impl Into<String>) -> RequestBuilder<HasUrl> {
        RequestBuilder {
            url: Some(url.into()),
            // ... copy all other fields ...
            _state: PhantomData,
        }
    }
}
```

</details>

<details>
<summary>Hint 3: Validation in build()</summary>

Even with typestate enforcing that URL is set, you still need runtime validation for cross-field constraints like "GET requests must not have a body":

```rust
fn build(self) -> Result<HttpRequest, String> {
    match (&self.method, &self.body) {
        (HttpMethod::Get, Some(_)) | (HttpMethod::Head, Some(_)) => {
            return Err("...".to_string());
        }
        _ => {}
    }
    // ...
}
```

</details>

<details>
<summary>Hint 4: Into&lt;String&gt; for ergonomics</summary>

Using `impl Into<String>` lets callers pass `&str`, `String`, or any other type that converts to String:

```rust
fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.headers.push((key.into(), value.into()));
    self
}
```

</details>

## Solution

After completing your implementation, compare with `solutions/solution.rs`.
