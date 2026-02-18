// Factory Pattern: Connection Pool Factory with Associated Types
//
// Demonstrates: Factory trait with associated types, generic factory consumers,
// and the difference between associated types and generic parameters.
//
// Scenario: A service needs to manage connections to different databases
// (Postgres, Redis, SQLite). Each database has its own connection type,
// config type, and error type. The factory trait uses associated types to
// bind these together per implementation.
//
// Run: rustc connection.rs && ./connection

use std::fmt;

// ---------------------------------------------------------------------------
// The Factory Trait (with associated types)
// ---------------------------------------------------------------------------

/// A factory for creating database connections.
///
/// Associated types bind the Connection, Config, and Error types together.
/// Each implementor specifies exactly ONE set of types -- a PostgresFactory
/// always produces PostgresConnections from PostgresConfig.
///
/// Compare to a generic trait `Factory<C, Cfg, E>` which would allow the
/// same factory to produce different connection types (rarely what you want).
trait ConnectionFactory: fmt::Display {
    type Connection: fmt::Debug;
    type Config: fmt::Debug + Clone;
    type Error: fmt::Display;

    /// Create a new connection from config.
    fn connect(&self, config: &Self::Config) -> Result<Self::Connection, Self::Error>;

    /// Validate configuration without creating a connection.
    fn validate(&self, config: &Self::Config) -> Result<(), Self::Error>;

    /// Return the maximum number of connections this backend supports.
    fn max_connections(&self) -> usize;

    /// Backend identifier for logging.
    fn backend_name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Postgres implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct PgConfig {
    host: String,
    port: u16,
    database: String,
    ssl_mode: SslMode,
    statement_timeout_ms: u32,
}

#[derive(Debug, Clone, Copy)]
enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl fmt::Display for SslMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SslMode::Disable => write!(f, "disable"),
            SslMode::Prefer => write!(f, "prefer"),
            SslMode::Require => write!(f, "require"),
        }
    }
}

impl PgConfig {
    fn new(host: &str, database: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 5432,
            database: database.to_string(),
            ssl_mode: SslMode::Prefer,
            statement_timeout_ms: 30_000,
        }
    }

    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn ssl(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }
}

#[derive(Debug)]
struct PgConnection {
    conn_string: String,
    #[allow(dead_code)]
    connected: bool,
    #[allow(dead_code)]
    pid: u32,
}

impl PgConnection {
    fn execute(&self, query: &str) -> String {
        format!("[PG:{}] Executed: {}", self.conn_string, query)
    }
}

struct PgFactory {
    pool_size: usize,
}

impl PgFactory {
    fn new(pool_size: usize) -> Self {
        Self { pool_size }
    }
}

impl ConnectionFactory for PgFactory {
    type Connection = PgConnection;
    type Config = PgConfig;
    type Error = String;

    fn connect(&self, config: &PgConfig) -> Result<PgConnection, String> {
        self.validate(config)?;
        let conn_string = format!(
            "postgres://{}:{}/{}?sslmode={}",
            config.host, config.port, config.database, config.ssl_mode
        );
        println!(
            "    [PG] Connecting to {} (timeout: {}ms)",
            conn_string, config.statement_timeout_ms
        );
        Ok(PgConnection {
            conn_string,
            connected: true,
            pid: 12345, // simulated
        })
    }

    fn validate(&self, config: &PgConfig) -> Result<(), String> {
        if config.host.is_empty() {
            return Err("host is required".into());
        }
        if config.database.is_empty() {
            return Err("database is required".into());
        }
        if config.port == 0 {
            return Err("port must be non-zero".into());
        }
        Ok(())
    }

    fn max_connections(&self) -> usize {
        self.pool_size
    }

    fn backend_name(&self) -> &str {
        "postgres"
    }
}

impl fmt::Display for PgFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PgFactory(pool_size={})", self.pool_size)
    }
}

// ---------------------------------------------------------------------------
// Redis implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RedisConfig {
    host: String,
    port: u16,
    database: u8,
    password: Option<String>,
}

impl RedisConfig {
    fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 6379,
            database: 0,
            password: None,
        }
    }

    fn database(mut self, db: u8) -> Self {
        self.database = db;
        self
    }

    fn password(mut self, pwd: &str) -> Self {
        self.password = Some(pwd.to_string());
        self
    }
}

#[derive(Debug)]
struct RedisConnection {
    addr: String,
    #[allow(dead_code)]
    db: u8,
}

impl RedisConnection {
    fn get(&self, key: &str) -> String {
        format!("[REDIS:{}] GET {}", self.addr, key)
    }

    fn set(&self, key: &str, value: &str) -> String {
        format!("[REDIS:{}] SET {} = {}", self.addr, key, value)
    }
}

struct RedisFactory {
    max_conn: usize,
}

impl RedisFactory {
    fn new(max_conn: usize) -> Self {
        Self { max_conn }
    }
}

impl ConnectionFactory for RedisFactory {
    type Connection = RedisConnection;
    type Config = RedisConfig;
    type Error = String;

    fn connect(&self, config: &RedisConfig) -> Result<RedisConnection, String> {
        self.validate(config)?;
        let addr = format!("{}:{}", config.host, config.port);
        let auth_status = if config.password.is_some() {
            "authenticated"
        } else {
            "no-auth"
        };
        println!(
            "    [REDIS] Connecting to {} db={} ({})",
            addr, config.database, auth_status
        );
        Ok(RedisConnection {
            addr,
            db: config.database,
        })
    }

    fn validate(&self, config: &RedisConfig) -> Result<(), String> {
        if config.host.is_empty() {
            return Err("host is required".into());
        }
        if config.database > 15 {
            return Err(format!(
                "Redis database must be 0-15, got {}",
                config.database
            ));
        }
        Ok(())
    }

    fn max_connections(&self) -> usize {
        self.max_conn
    }

    fn backend_name(&self) -> &str {
        "redis"
    }
}

impl fmt::Display for RedisFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RedisFactory(max_conn={})", self.max_conn)
    }
}

// ---------------------------------------------------------------------------
// SQLite implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SqliteConfig {
    path: String,
    read_only: bool,
    journal_mode: String,
}

impl SqliteConfig {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            read_only: false,
            journal_mode: "wal".to_string(),
        }
    }

    fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    fn in_memory() -> Self {
        Self {
            path: ":memory:".to_string(),
            read_only: false,
            journal_mode: "memory".to_string(),
        }
    }
}

#[derive(Debug)]
struct SqliteConnection {
    path: String,
    #[allow(dead_code)]
    read_only: bool,
}

impl SqliteConnection {
    fn execute(&self, sql: &str) -> String {
        format!("[SQLITE:{}] {}", self.path, sql)
    }
}

struct SqliteFactory;

impl ConnectionFactory for SqliteFactory {
    type Connection = SqliteConnection;
    type Config = SqliteConfig;
    type Error = String;

    fn connect(&self, config: &SqliteConfig) -> Result<SqliteConnection, String> {
        self.validate(config)?;
        let mode = if config.read_only { "ro" } else { "rw" };
        println!(
            "    [SQLITE] Opening {} (mode={}, journal={})",
            config.path, mode, config.journal_mode
        );
        Ok(SqliteConnection {
            path: config.path.clone(),
            read_only: config.read_only,
        })
    }

    fn validate(&self, config: &SqliteConfig) -> Result<(), String> {
        if config.path.is_empty() {
            return Err("path is required".into());
        }
        Ok(())
    }

    fn max_connections(&self) -> usize {
        1 // SQLite is single-writer
    }

    fn backend_name(&self) -> &str {
        "sqlite"
    }
}

impl fmt::Display for SqliteFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SqliteFactory")
    }
}

// ---------------------------------------------------------------------------
// Generic code that works with ANY factory (static dispatch)
// ---------------------------------------------------------------------------

/// Health check using a factory. This function is generic over the factory
/// type -- the compiler monomorphizes it for each concrete factory.
///
/// Notice: the function signature uses `F::Config` and `F::Connection` --
/// associated types let us refer to the factory's types without additional
/// generic parameters.
fn health_check<F: ConnectionFactory>(factory: &F, config: &F::Config) -> bool {
    println!("  Health check for {} ({}):", factory.backend_name(), factory);
    match factory.validate(config) {
        Ok(()) => {
            println!("    Config valid");
            match factory.connect(config) {
                Ok(conn) => {
                    println!("    Connected: {:?}", conn);
                    println!("    Max connections: {}", factory.max_connections());
                    true
                }
                Err(e) => {
                    println!("    Connection failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            println!("    Config invalid: {}", e);
            false
        }
    }
}

/// Create N connections from a factory (simple pool simulation).
fn create_pool<F: ConnectionFactory>(
    factory: &F,
    config: &F::Config,
    size: usize,
) -> Vec<F::Connection> {
    let actual_size = size.min(factory.max_connections());
    let mut connections = Vec::with_capacity(actual_size);
    for _ in 0..actual_size {
        match factory.connect(config) {
            Ok(conn) => connections.push(conn),
            Err(e) => {
                println!("    Pool creation error: {}", e);
                break;
            }
        }
    }
    connections
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Connection Factory (Associated Types) ===\n");

    // --- Postgres ---
    println!("--- Postgres ---");
    let pg_factory = PgFactory::new(20);
    let pg_config = PgConfig::new("db.internal", "myapp")
        .port(5432)
        .ssl(SslMode::Require);

    let healthy = health_check(&pg_factory, &pg_config);
    println!("  Healthy: {}\n", healthy);

    if let Ok(conn) = pg_factory.connect(&pg_config) {
        println!("  {}", conn.execute("SELECT count(*) FROM users"));
        println!();
    }

    // --- Redis ---
    println!("--- Redis ---");
    let redis_factory = RedisFactory::new(50);
    let redis_config = RedisConfig::new("cache.internal")
        .database(2)
        .password("s3cret");

    let healthy = health_check(&redis_factory, &redis_config);
    println!("  Healthy: {}\n", healthy);

    if let Ok(conn) = redis_factory.connect(&redis_config) {
        println!("  {}", conn.set("session:abc", "user_id=42"));
        println!("  {}", conn.get("session:abc"));
        println!();
    }

    // --- SQLite ---
    println!("--- SQLite ---");
    let sqlite_factory = SqliteFactory;
    let sqlite_config = SqliteConfig::in_memory();

    let healthy = health_check(&sqlite_factory, &sqlite_config);
    println!("  Healthy: {}\n", healthy);

    if let Ok(conn) = sqlite_factory.connect(&sqlite_config) {
        println!(
            "  {}",
            conn.execute("CREATE TABLE events (id INTEGER PRIMARY KEY, name TEXT)")
        );
        println!();
    }

    // --- Pool creation ---
    println!("--- Pool creation ---");
    println!("  Creating Postgres pool (requested=5, max={}):", pg_factory.max_connections());
    let pg_pool = create_pool(&pg_factory, &pg_config, 5);
    println!("  Pool size: {}\n", pg_pool.len());

    println!("  Creating SQLite pool (requested=5, max={}):", sqlite_factory.max_connections());
    let sqlite_pool = create_pool(&sqlite_factory, &sqlite_config, 5);
    println!("  Pool size: {} (SQLite is single-writer)\n", sqlite_pool.len());

    // --- Validation failure ---
    println!("--- Validation failures ---");
    let bad_pg = PgConfig::new("", "myapp");
    let valid = health_check(&pg_factory, &bad_pg);
    println!("  Healthy: {}\n", valid);

    let bad_redis = RedisConfig::new("cache.internal").database(99);
    let valid = health_check(&redis_factory, &bad_redis);
    println!("  Healthy: {}", valid);
}
