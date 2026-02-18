// HTTP Request Builder -- Reference Solution
//
// Typestate builder pattern enforcing URL is set before build().
// Consuming builder (takes self) for clean chaining.
//
// Run tests: rustc --test solution.rs && ./solution

use std::marker::PhantomData;
use std::fmt;

// --- HTTP Method ---

#[derive(Debug, Clone, PartialEq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
        }
    }
}

// --- Auth Scheme ---

#[derive(Debug, Clone, PartialEq)]
enum AuthScheme {
    Bearer(String),
    Basic { username: String, password: String },
    ApiKey { header: String, value: String },
}

// --- Typestate Markers ---

struct NoUrl;
struct HasUrl;

// --- Target Struct ---

#[derive(Debug)]
struct HttpRequest {
    method: HttpMethod,
    url: String,
    headers: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    body: Option<String>,
    auth: Option<AuthScheme>,
    timeout_ms: Option<u64>,
}

// --- Builder ---

struct RequestBuilder<UrlState> {
    method: HttpMethod,
    url: Option<String>,
    headers: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    body: Option<String>,
    auth: Option<AuthScheme>,
    timeout_ms: Option<u64>,
    _state: PhantomData<UrlState>,
}

// Constructor: starts without a URL
impl RequestBuilder<NoUrl> {
    fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            url: None,
            headers: Vec::new(),
            query_params: Vec::new(),
            body: None,
            auth: None,
            timeout_ms: None,
            _state: PhantomData,
        }
    }

    // State transition: NoUrl -> HasUrl
    fn url(self, url: impl Into<String>) -> RequestBuilder<HasUrl> {
        RequestBuilder {
            method: self.method,
            url: Some(url.into()),
            headers: self.headers,
            query_params: self.query_params,
            body: self.body,
            auth: self.auth,
            timeout_ms: self.timeout_ms,
            _state: PhantomData,
        }
    }
}

// Optional setters: available in any state (generic over UrlState)
impl<S> RequestBuilder<S> {
    fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth = Some(AuthScheme::Bearer(token.into()));
        self
    }

    fn basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth = Some(AuthScheme::Basic {
            username: username.into(),
            password: password.into(),
        });
        self
    }

    fn api_key(
        mut self,
        header: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.auth = Some(AuthScheme::ApiKey {
            header: header.into(),
            value: value.into(),
        });
        self
    }

    fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

// build() only available when URL has been set
impl RequestBuilder<HasUrl> {
    fn build(self) -> Result<HttpRequest, String> {
        // Validate: GET and HEAD must not have a body
        match (&self.method, &self.body) {
            (HttpMethod::Get, Some(_)) => {
                return Err("GET requests must not have a body".to_string());
            }
            (HttpMethod::Head, Some(_)) => {
                return Err("HEAD requests must not have a body".to_string());
            }
            _ => {}
        }

        Ok(HttpRequest {
            method: self.method,
            url: self.url.unwrap(), // safe: typestate guarantees URL is set
            headers: self.headers,
            query_params: self.query_params,
            body: self.body,
            auth: self.auth,
            timeout_ms: self.timeout_ms,
        })
    }
}

// --- Demo ---

fn main() {
    // Simple GET
    let req = RequestBuilder::new()
        .url("https://api.example.com/health")
        .build()
        .unwrap();
    println!("Simple GET: {:?}\n", req);

    // Full POST
    let req = RequestBuilder::new()
        .method(HttpMethod::Post)
        .url("https://api.example.com/users")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .query("notify", "true")
        .body(r#"{"name": "alice", "role": "admin"}"#)
        .bearer_token("eyJhbGciOiJIUzI1NiJ9.token")
        .timeout_ms(5000)
        .build()
        .unwrap();
    println!("Full POST: {:?}\n", req);

    // Error case: GET with body
    let result = RequestBuilder::new()
        .url("https://api.example.com/users")
        .body("oops")
        .build();
    println!("GET with body: {:?}\n", result);

    // This would not compile:
    // let req = RequestBuilder::new().build();  // no method `build` on RequestBuilder<NoUrl>
    println!("Compile-time safety: build() is only available after url() is called.");
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_get_request() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/health")
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.url, "https://api.example.com/health");
        assert!(req.headers.is_empty());
        assert!(req.query_params.is_empty());
        assert!(req.body.is_none());
        assert!(req.auth.is_none());
        assert!(req.timeout_ms.is_none());
    }

    #[test]
    fn test_post_with_body() {
        let req = RequestBuilder::new()
            .method(HttpMethod::Post)
            .url("https://api.example.com/users")
            .header("Content-Type", "application/json")
            .body(r#"{"name": "alice"}"#)
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.url, "https://api.example.com/users");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0], ("Content-Type".to_string(), "application/json".to_string()));
        assert_eq!(req.body.as_deref(), Some(r#"{"name": "alice"}"#));
    }

    #[test]
    fn test_get_with_body_fails() {
        let result = RequestBuilder::new()
            .url("https://api.example.com/users")
            .body("should not be here")
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_lowercase().contains("body") || err.to_lowercase().contains("get"),
            "Error should mention body or GET: {}",
            err
        );
    }

    #[test]
    fn test_head_with_body_fails() {
        let result = RequestBuilder::new()
            .method(HttpMethod::Head)
            .url("https://api.example.com/users")
            .body("should not be here")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_headers_accumulate() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/data")
            .header("Accept", "application/json")
            .header("X-Request-Id", "abc-123")
            .header("X-Trace-Id", "trace-456")
            .build()
            .unwrap();

        assert_eq!(req.headers.len(), 3);
        assert_eq!(req.headers[0].0, "Accept");
        assert_eq!(req.headers[1].0, "X-Request-Id");
        assert_eq!(req.headers[2].0, "X-Trace-Id");
    }

    #[test]
    fn test_query_params_accumulate() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/search")
            .query("q", "rust builder")
            .query("page", "1")
            .query("limit", "25")
            .build()
            .unwrap();

        assert_eq!(req.query_params.len(), 3);
        assert_eq!(req.query_params[0], ("q".to_string(), "rust builder".to_string()));
        assert_eq!(req.query_params[1], ("page".to_string(), "1".to_string()));
        assert_eq!(req.query_params[2], ("limit".to_string(), "25".to_string()));
    }

    #[test]
    fn test_bearer_token_auth() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/me")
            .bearer_token("my-secret-token")
            .build()
            .unwrap();

        assert_eq!(
            req.auth,
            Some(AuthScheme::Bearer("my-secret-token".to_string()))
        );
    }

    #[test]
    fn test_basic_auth() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/me")
            .basic_auth("admin", "password123")
            .build()
            .unwrap();

        assert_eq!(
            req.auth,
            Some(AuthScheme::Basic {
                username: "admin".to_string(),
                password: "password123".to_string(),
            })
        );
    }

    #[test]
    fn test_api_key_auth() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/data")
            .api_key("X-Api-Key", "key-abc-123")
            .build()
            .unwrap();

        assert_eq!(
            req.auth,
            Some(AuthScheme::ApiKey {
                header: "X-Api-Key".to_string(),
                value: "key-abc-123".to_string(),
            })
        );
    }

    #[test]
    fn test_auth_last_one_wins() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/me")
            .bearer_token("first-token")
            .basic_auth("admin", "pass")
            .build()
            .unwrap();

        assert_eq!(
            req.auth,
            Some(AuthScheme::Basic {
                username: "admin".to_string(),
                password: "pass".to_string(),
            })
        );
    }

    #[test]
    fn test_timeout() {
        let req = RequestBuilder::new()
            .url("https://api.example.com/slow")
            .timeout_ms(5000)
            .build()
            .unwrap();

        assert_eq!(req.timeout_ms, Some(5000));
    }

    #[test]
    fn test_full_request() {
        let req = RequestBuilder::new()
            .method(HttpMethod::Put)
            .url("https://api.example.com/users/42")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .query("version", "2")
            .body(r#"{"name": "updated"}"#)
            .bearer_token("token-xyz")
            .timeout_ms(10_000)
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Put);
        assert_eq!(req.url, "https://api.example.com/users/42");
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.query_params.len(), 1);
        assert!(req.body.is_some());
        assert!(req.auth.is_some());
        assert_eq!(req.timeout_ms, Some(10_000));
    }

    #[test]
    fn test_method_set_before_url() {
        let req = RequestBuilder::new()
            .method(HttpMethod::Delete)
            .timeout_ms(3000)
            .url("https://api.example.com/users/42")
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Delete);
        assert_eq!(req.timeout_ms, Some(3000));
    }

    #[test]
    fn test_accepts_string_and_str() {
        let url = String::from("https://api.example.com/test");
        let token = String::from("my-token");

        let req = RequestBuilder::new()
            .url(url)
            .header("Key", "Value")
            .bearer_token(token)
            .build()
            .unwrap();

        assert_eq!(req.url, "https://api.example.com/test");
    }

    #[test]
    fn test_post_without_body_succeeds() {
        let req = RequestBuilder::new()
            .method(HttpMethod::Post)
            .url("https://api.example.com/trigger")
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.body.is_none());
    }

    #[test]
    fn test_delete_with_body_succeeds() {
        let req = RequestBuilder::new()
            .method(HttpMethod::Delete)
            .url("https://api.example.com/batch")
            .body(r#"{"ids": [1, 2, 3]}"#)
            .build()
            .unwrap();

        assert_eq!(req.method, HttpMethod::Delete);
        assert!(req.body.is_some());
    }
}
