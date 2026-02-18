// Factory Pattern: Storage Backend Factory
//
// Demonstrates: Dynamic dispatch factory with Box<dyn Trait>
//
// Scenario: A key-value storage layer that supports multiple backends
// (in-memory, filesystem, S3-compatible). The backend is chosen at runtime
// from configuration. This is the classic case for trait object factories --
// the concrete type depends on external input.
//
// Run: rustc storage.rs && ./storage

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct StorageError {
    backend: String,
    operation: String,
    reason: String,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} failed: {}",
            self.backend, self.operation, self.reason
        )
    }
}

// ---------------------------------------------------------------------------
// Storage trait (object-safe)
// ---------------------------------------------------------------------------

/// A key-value storage backend.
///
/// Object-safe: no generics, no Self returns, all methods have &self/&mut self.
trait Storage: fmt::Display {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), StorageError>;
    fn delete(&mut self, key: &str) -> Result<bool, StorageError>;
    fn exists(&self, key: &str) -> Result<bool, StorageError>;
    fn backend_name(&self) -> &str;

    /// List all keys matching a prefix. Default implementation scans all keys.
    fn list_keys(&self, _prefix: &str) -> Result<Vec<String>, StorageError> {
        Err(StorageError {
            backend: self.backend_name().to_string(),
            operation: "list_keys".to_string(),
            reason: "not implemented for this backend".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// InMemoryStorage
// ---------------------------------------------------------------------------

struct InMemoryStorage {
    data: HashMap<String, Vec<u8>>,
    max_keys: usize,
}

impl InMemoryStorage {
    fn new(max_keys: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_keys,
        }
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.data.get(key).cloned())
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        if self.data.len() >= self.max_keys && !self.data.contains_key(key) {
            return Err(StorageError {
                backend: "in-memory".to_string(),
                operation: "set".to_string(),
                reason: format!("max keys reached ({})", self.max_keys),
            });
        }
        self.data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<bool, StorageError> {
        Ok(self.data.remove(key).is_some())
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        Ok(self.data.contains_key(key))
    }

    fn backend_name(&self) -> &str {
        "in-memory"
    }

    fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        Ok(self
            .data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

impl fmt::Display for InMemoryStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InMemoryStorage(keys={}, max={})",
            self.data.len(),
            self.max_keys
        )
    }
}

// ---------------------------------------------------------------------------
// FileSystemStorage (simulated -- uses HashMap with path-prefixed keys)
// ---------------------------------------------------------------------------

struct FileSystemStorage {
    base_path: String,
    // In a real implementation, this would use std::fs
    // Here we simulate with a HashMap for a self-contained example
    files: HashMap<String, Vec<u8>>,
}

impl FileSystemStorage {
    fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
            files: HashMap::new(),
        }
    }

    fn full_path(&self, key: &str) -> String {
        format!("{}/{}", self.base_path, key.replace('.', "/"))
    }
}

impl Storage for FileSystemStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let path = self.full_path(key);
        Ok(self.files.get(&path).cloned())
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        let path = self.full_path(key);
        println!("    [FS] Writing {} bytes to {}", value.len(), path);
        self.files.insert(path, value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<bool, StorageError> {
        let path = self.full_path(key);
        Ok(self.files.remove(&path).is_some())
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let path = self.full_path(key);
        Ok(self.files.contains_key(&path))
    }

    fn backend_name(&self) -> &str {
        "filesystem"
    }

    fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let path_prefix = self.full_path(prefix);
        Ok(self
            .files
            .keys()
            .filter(|k| k.starts_with(&path_prefix))
            .cloned()
            .collect())
    }
}

impl fmt::Display for FileSystemStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileSystemStorage(base={})", self.base_path)
    }
}

// ---------------------------------------------------------------------------
// S3Storage (simulated)
// ---------------------------------------------------------------------------

struct S3Storage {
    bucket: String,
    region: String,
    // Simulated object store
    objects: HashMap<String, Vec<u8>>,
}

impl S3Storage {
    fn new(bucket: &str, region: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            region: region.to_string(),
            objects: HashMap::new(),
        }
    }

    fn object_key(&self, key: &str) -> String {
        format!("s3://{}/{}", self.bucket, key)
    }
}

impl Storage for S3Storage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let obj_key = self.object_key(key);
        Ok(self.objects.get(&obj_key).cloned())
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        if value.len() > 5 * 1024 * 1024 {
            return Err(StorageError {
                backend: "s3".to_string(),
                operation: "set".to_string(),
                reason: "single PUT limited to 5MB; use multipart upload".to_string(),
            });
        }
        let obj_key = self.object_key(key);
        println!(
            "    [S3] PUT {}/{} ({} bytes, region: {})",
            self.bucket,
            key,
            value.len(),
            self.region
        );
        self.objects.insert(obj_key, value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<bool, StorageError> {
        let obj_key = self.object_key(key);
        Ok(self.objects.remove(&obj_key).is_some())
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let obj_key = self.object_key(key);
        Ok(self.objects.contains_key(&obj_key))
    }

    fn backend_name(&self) -> &str {
        "s3"
    }

    fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let full_prefix = self.object_key(prefix);
        Ok(self
            .objects
            .keys()
            .filter(|k| k.starts_with(&full_prefix))
            .cloned()
            .collect())
    }
}

impl fmt::Display for S3Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S3Storage(bucket={}, region={})", self.bucket, self.region)
    }
}

// ---------------------------------------------------------------------------
// Storage configuration
// ---------------------------------------------------------------------------

struct StorageConfig {
    backend: String,
    params: HashMap<String, String>,
}

impl StorageConfig {
    fn new(backend: &str) -> Self {
        Self {
            backend: backend.to_string(),
            params: HashMap::new(),
        }
    }

    fn param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    fn get_param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    fn get_param_or(&self, key: &str, default: &str) -> String {
        self.params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

// ---------------------------------------------------------------------------
// THE FACTORY FUNCTION
// ---------------------------------------------------------------------------

/// Creates a storage backend based on configuration.
///
/// This is the core factory -- it takes a config, inspects the backend field,
/// and returns the appropriate implementation as a trait object.
///
/// The caller receives `Box<dyn Storage>` and interacts purely through the
/// trait interface. It never knows (or needs to know) the concrete type.
fn create_storage(config: &StorageConfig) -> Result<Box<dyn Storage>, String> {
    match config.backend.as_str() {
        "memory" | "in-memory" => {
            let max_keys: usize = config
                .get_param("max_keys")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000);
            Ok(Box::new(InMemoryStorage::new(max_keys)))
        }
        "file" | "filesystem" | "fs" => {
            let base_path = config.get_param_or("base_path", "/tmp/storage");
            Ok(Box::new(FileSystemStorage::new(&base_path)))
        }
        "s3" => {
            let bucket = config
                .get_param("bucket")
                .ok_or("s3 backend requires 'bucket' parameter")?;
            let region = config.get_param_or("region", "us-east-1");
            Ok(Box::new(S3Storage::new(bucket, &region)))
        }
        other => Err(format!(
            "unknown storage backend: '{}'. Available: memory, file, s3",
            other
        )),
    }
}

// ---------------------------------------------------------------------------
// Demo: using the factory
// ---------------------------------------------------------------------------

fn run_storage_demo(mut store: Box<dyn Storage>) {
    println!("  Backend: {}", store);
    println!("  Type: {}", store.backend_name());

    // Write
    store
        .set("config.app.name", b"my-service")
        .expect("set failed");
    store
        .set("config.app.version", b"2.3.1")
        .expect("set failed");
    store
        .set("config.db.host", b"db.internal")
        .expect("set failed");

    // Read
    if let Ok(Some(value)) = store.get("config.app.name") {
        println!(
            "  Read config.app.name = {}",
            String::from_utf8_lossy(&value)
        );
    }

    // Exists
    println!(
        "  config.app.name exists: {:?}",
        store.exists("config.app.name")
    );
    println!(
        "  config.missing exists: {:?}",
        store.exists("config.missing")
    );

    // List
    match store.list_keys("config") {
        Ok(keys) => println!("  Keys with 'config' prefix: {:?}", keys),
        Err(e) => println!("  List not supported: {}", e),
    }

    // Delete
    let deleted = store.delete("config.app.version").expect("delete failed");
    println!("  Deleted config.app.version: {}", deleted);

    println!();
}

fn main() {
    println!("=== Storage Backend Factory (Dynamic Dispatch) ===\n");

    // --- In-Memory backend ---
    println!("--- Creating in-memory storage ---");
    let config = StorageConfig::new("memory").param("max_keys", "100");
    match create_storage(&config) {
        Ok(store) => run_storage_demo(store),
        Err(e) => println!("  Factory error: {}\n", e),
    }

    // --- Filesystem backend ---
    println!("--- Creating filesystem storage ---");
    let config = StorageConfig::new("fs").param("base_path", "/var/data/kv");
    match create_storage(&config) {
        Ok(store) => run_storage_demo(store),
        Err(e) => println!("  Factory error: {}\n", e),
    }

    // --- S3 backend ---
    println!("--- Creating S3 storage ---");
    let config = StorageConfig::new("s3")
        .param("bucket", "my-app-config")
        .param("region", "eu-west-1");
    match create_storage(&config) {
        Ok(store) => run_storage_demo(store),
        Err(e) => println!("  Factory error: {}\n", e),
    }

    // --- Error case: missing required param ---
    println!("--- S3 without bucket (should fail) ---");
    let config = StorageConfig::new("s3");
    match create_storage(&config) {
        Ok(store) => run_storage_demo(store),
        Err(e) => println!("  Factory error: {}\n", e),
    }

    // --- Error case: unknown backend ---
    println!("--- Unknown backend ---");
    let config = StorageConfig::new("redis");
    match create_storage(&config) {
        Ok(store) => run_storage_demo(store),
        Err(e) => println!("  Factory error: {}\n", e),
    }

    // --- Demonstrating the trait object's opacity ---
    println!("--- Multiple backends in a Vec<Box<dyn Storage>> ---");
    let configs = vec![
        StorageConfig::new("memory"),
        StorageConfig::new("fs").param("base_path", "/tmp/multi"),
        StorageConfig::new("s3")
            .param("bucket", "multi-test")
            .param("region", "ap-southeast-1"),
    ];

    let stores: Vec<Box<dyn Storage>> = configs
        .iter()
        .filter_map(|c| create_storage(c).ok())
        .collect();

    println!("  Created {} storage backends:", stores.len());
    for store in &stores {
        println!("    - {} ({})", store.backend_name(), store);
    }
}
