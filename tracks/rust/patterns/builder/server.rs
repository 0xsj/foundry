// Builder Pattern: Typestate HTTP Server Config
//
// Demonstrates a typestate builder that enforces at compile time
// that `address` must be set before `build()` can be called.
// PhantomData markers carry zero runtime cost.
//
// Run: rustc server.rs && ./server

use std::marker::PhantomData;

// --- Typestate markers (zero-sized) ---

struct NeedsAddress;
struct HasAddress;

// --- Target struct ---

#[derive(Debug)]
struct ServerConfig {
    address: String,
    port: u16,
    max_connections: usize,
    read_timeout_ms: u64,
    write_timeout_ms: u64,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    cors_origins: Vec<String>,
}

// --- Builder ---

struct ServerBuilder<AddrState> {
    address: Option<String>,
    port: u16,
    max_connections: usize,
    read_timeout_ms: u64,
    write_timeout_ms: u64,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    cors_origins: Vec<String>,
    _state: PhantomData<AddrState>,
}

// Constructor: starts in NeedsAddress state
impl ServerBuilder<NeedsAddress> {
    fn new() -> Self {
        Self {
            address: None,
            port: 8080,
            max_connections: 1024,
            read_timeout_ms: 30_000,
            write_timeout_ms: 30_000,
            tls_cert: None,
            tls_key: None,
            cors_origins: Vec::new(),
            _state: PhantomData,
        }
    }

    // Transitions NeedsAddress -> HasAddress
    fn address(self, addr: impl Into<String>) -> ServerBuilder<HasAddress> {
        ServerBuilder {
            address: Some(addr.into()),
            port: self.port,
            max_connections: self.max_connections,
            read_timeout_ms: self.read_timeout_ms,
            write_timeout_ms: self.write_timeout_ms,
            tls_cert: self.tls_cert,
            tls_key: self.tls_key,
            cors_origins: self.cors_origins,
            _state: PhantomData,
        }
    }
}

// Optional setters: available in any state
impl<S> ServerBuilder<S> {
    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }

    fn read_timeout_ms(mut self, ms: u64) -> Self {
        self.read_timeout_ms = ms;
        self
    }

    fn write_timeout_ms(mut self, ms: u64) -> Self {
        self.write_timeout_ms = ms;
        self
    }

    fn tls(mut self, cert: impl Into<String>, key: impl Into<String>) -> Self {
        self.tls_cert = Some(cert.into());
        self.tls_key = Some(key.into());
        self
    }

    fn cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origins.push(origin.into());
        self
    }
}

// build() is only available when address has been set
impl ServerBuilder<HasAddress> {
    fn build(self) -> ServerConfig {
        ServerConfig {
            address: self.address.unwrap(), // safe: typestate guarantees this is set
            port: self.port,
            max_connections: self.max_connections,
            read_timeout_ms: self.read_timeout_ms,
            write_timeout_ms: self.write_timeout_ms,
            tls_cert: self.tls_cert,
            tls_key: self.tls_key,
            cors_origins: self.cors_origins,
        }
    }
}

// --- Demo ---

fn main() {
    // Minimal: only required field
    let minimal = ServerBuilder::new()
        .address("127.0.0.1")
        .build();

    println!("Minimal config:\n{:#?}\n", minimal);

    // Full configuration
    let production = ServerBuilder::new()
        .port(443)
        .max_connections(10_000)
        .read_timeout_ms(5_000)
        .write_timeout_ms(10_000)
        .address("0.0.0.0")
        .tls("/etc/ssl/cert.pem", "/etc/ssl/key.pem")
        .cors_origin("https://app.example.com")
        .cors_origin("https://admin.example.com")
        .build();

    println!("Production config:\n{:#?}\n", production);

    // This would NOT compile -- build() does not exist on ServerBuilder<NeedsAddress>:
    // let broken = ServerBuilder::new().port(8080).build();
    // Error: no method named `build` found for struct `ServerBuilder<NeedsAddress>`

    println!("Note: optional fields can be set before or after the required address field.");
    println!("The order of method calls does not matter, only that address() is called.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config() {
        let config = ServerBuilder::new()
            .address("localhost")
            .build();

        assert_eq!(config.address, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 1024);
        assert!(config.tls_cert.is_none());
        assert!(config.cors_origins.is_empty());
    }

    #[test]
    fn test_full_config() {
        let config = ServerBuilder::new()
            .address("0.0.0.0")
            .port(443)
            .max_connections(5000)
            .tls("cert.pem", "key.pem")
            .cors_origin("https://example.com")
            .build();

        assert_eq!(config.address, "0.0.0.0");
        assert_eq!(config.port, 443);
        assert_eq!(config.max_connections, 5000);
        assert_eq!(config.tls_cert.as_deref(), Some("cert.pem"));
        assert_eq!(config.tls_key.as_deref(), Some("key.pem"));
        assert_eq!(config.cors_origins, vec!["https://example.com"]);
    }

    #[test]
    fn test_optional_fields_before_required() {
        // Optional fields can be set before the required address
        let config = ServerBuilder::new()
            .port(9090)
            .max_connections(500)
            .address("10.0.0.1")
            .build();

        assert_eq!(config.address, "10.0.0.1");
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn test_multiple_cors_origins() {
        let config = ServerBuilder::new()
            .address("localhost")
            .cors_origin("https://a.com")
            .cors_origin("https://b.com")
            .cors_origin("https://c.com")
            .build();

        assert_eq!(config.cors_origins.len(), 3);
    }

    #[test]
    fn test_accepts_string_and_str() {
        let addr = String::from("192.168.1.1");
        let config = ServerBuilder::new()
            .address(addr)  // String
            .tls("cert".to_string(), "key")  // mixed String and &str
            .build();

        assert_eq!(config.address, "192.168.1.1");
    }
}
