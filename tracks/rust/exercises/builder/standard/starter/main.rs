// HTTP Request Builder -- Starter Code
//
// Implement the builder pattern with typestate enforcement.
// The URL must be set before build() can be called.
//
// Run tests: rustc --test main.rs && ./main

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

// TODO: Implement RequestBuilder::new() -> RequestBuilder<NoUrl>
//       Default method should be Get.

// TODO: Implement url() on RequestBuilder<NoUrl>
//       Should transition state to HasUrl.
//       Accept impl Into<String>.

// TODO: Implement optional setter methods on RequestBuilder<UrlState>
//       (available in any state):
//       - method(HttpMethod) -> Self
//       - header(key, value) -> Self         (accumulating)
//       - query(key, value) -> Self          (accumulating)
//       - body(impl Into<String>) -> Self
//       - bearer_token(impl Into<String>) -> Self
//       - basic_auth(username, password) -> Self
//       - api_key(header, value) -> Self
//       - timeout_ms(u64) -> Self
//
//       All string params should accept impl Into<String>.

// TODO: Implement build() on RequestBuilder<HasUrl>
//       Returns Result<HttpRequest, String>.
//       Validate: GET and HEAD must not have a body.

fn main() {
    // Example usage (will work once implemented):
    //
    // let request = RequestBuilder::new()
    //     .url("https://api.example.com/users")
    //     .header("Content-Type", "application/json")
    //     .bearer_token("abc123")
    //     .build()
    //     .unwrap();
    //
    // println!("{:?}", request);

    println!("Implement the builder and run tests with: rustc --test main.rs && ./main");
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

        // basic_auth was called last, so it should win
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
        // Method can be set before url -- optional setters work in any state
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
            .url(url)                    // String
            .header("Key", "Value")      // &str, &str
            .bearer_token(token)         // String
            .build()
            .unwrap();

        assert_eq!(req.url, "https://api.example.com/test");
    }

    #[test]
    fn test_post_without_body_succeeds() {
        // POST without body is valid (e.g., triggering an action)
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
        // DELETE with body is valid per HTTP spec
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
