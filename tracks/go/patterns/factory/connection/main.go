// Database Connection Factory -- Abstract Factory
//
// Demonstrates: Abstract Factory pattern
//
// A database abstraction that creates families of related objects:
// Connection, QueryBuilder, and Migrator. Each database type (postgres, sqlite)
// provides its own family of compatible implementations.
//
// The key constraint: you should never mix a postgres QueryBuilder with
// a sqlite Connection. The abstract factory ensures consistency.
//
// Run: go run .
package main

import (
	"fmt"
	"strings"
)

// ---------------------------------------------------------------------
// Product interfaces -- the family of related objects
// ---------------------------------------------------------------------

// Connection represents a database connection.
type Connection interface {
	Execute(query string, args ...any) (Result, error)
	Close() error
	DSN() string
}

// Result represents a query result.
type Result struct {
	RowsAffected int
	Columns      []string
}

// QueryBuilder constructs SQL queries for a specific dialect.
type QueryBuilder interface {
	Select(table string, columns ...string) QueryBuilder
	Where(condition string, args ...any) QueryBuilder
	Limit(n int) QueryBuilder
	Build() (string, []any)
	Dialect() string
}

// Migrator manages database schema migrations.
type Migrator interface {
	CreateTable(name string, columns map[string]string) string
	AddColumn(table, column, colType string) string
	MigrationTableSQL() string
	Dialect() string
}

// ConnectionConfig holds connection parameters.
type ConnectionConfig struct {
	Host     string
	Port     int
	Database string
	User     string
	Password string
	FilePath string // for SQLite
	Options  map[string]string
}

// ---------------------------------------------------------------------
// Abstract Factory interface
// ---------------------------------------------------------------------

// DatabaseKit creates a family of related database objects.
// All objects created by a single kit are guaranteed to be compatible.
type DatabaseKit interface {
	CreateConnection(cfg ConnectionConfig) (Connection, error)
	CreateQueryBuilder() QueryBuilder
	CreateMigrator() Migrator
	DriverName() string
}

// ---------------------------------------------------------------------
// PostgreSQL family
// ---------------------------------------------------------------------

type postgresKit struct{}

func NewPostgresKit() DatabaseKit { return &postgresKit{} }

func (k *postgresKit) DriverName() string { return "postgres" }

func (k *postgresKit) CreateConnection(cfg ConnectionConfig) (Connection, error) {
	if cfg.Host == "" {
		cfg.Host = "localhost"
	}
	if cfg.Port == 0 {
		cfg.Port = 5432
	}
	dsn := fmt.Sprintf("postgres://%s:%s@%s:%d/%s",
		cfg.User, cfg.Password, cfg.Host, cfg.Port, cfg.Database)

	if sslMode, ok := cfg.Options["sslmode"]; ok {
		dsn += "?sslmode=" + sslMode
	}

	fmt.Printf("  [postgres] Connecting to %s:%d/%s\n", cfg.Host, cfg.Port, cfg.Database)
	return &pgConnection{dsn: dsn, connected: true}, nil
}

func (k *postgresKit) CreateQueryBuilder() QueryBuilder {
	return &pgQueryBuilder{}
}

func (k *postgresKit) CreateMigrator() Migrator {
	return &pgMigrator{}
}

// --- Postgres Connection ---

type pgConnection struct {
	dsn       string
	connected bool
}

func (c *pgConnection) Execute(query string, args ...any) (Result, error) {
	if !c.connected {
		return Result{}, fmt.Errorf("postgres: connection closed")
	}
	fmt.Printf("  [postgres] EXEC: %s  args=%v\n", query, args)
	return Result{RowsAffected: 1, Columns: []string{"id", "name"}}, nil
}

func (c *pgConnection) Close() error {
	c.connected = false
	fmt.Println("  [postgres] Connection closed")
	return nil
}

func (c *pgConnection) DSN() string { return c.dsn }

// --- Postgres QueryBuilder ---

type pgQueryBuilder struct {
	table     string
	columns   []string
	where     []string
	whereArgs []any
	limit     int
}

func (b *pgQueryBuilder) Select(table string, columns ...string) QueryBuilder {
	b.table = table
	b.columns = columns
	return b
}

func (b *pgQueryBuilder) Where(condition string, args ...any) QueryBuilder {
	b.where = append(b.where, condition)
	b.whereArgs = append(b.whereArgs, args...)
	return b
}

func (b *pgQueryBuilder) Limit(n int) QueryBuilder {
	b.limit = n
	return b
}

func (b *pgQueryBuilder) Build() (string, []any) {
	cols := "*"
	if len(b.columns) > 0 {
		// PostgreSQL uses double-quoted identifiers
		quoted := make([]string, len(b.columns))
		for i, c := range b.columns {
			quoted[i] = fmt.Sprintf(`"%s"`, c)
		}
		cols = strings.Join(quoted, ", ")
	}

	query := fmt.Sprintf(`SELECT %s FROM "%s"`, cols, b.table)

	if len(b.where) > 0 {
		// PostgreSQL uses $1, $2, ... placeholders
		conditions := make([]string, len(b.where))
		for i, w := range b.where {
			conditions[i] = strings.Replace(w, "?", fmt.Sprintf("$%d", i+1), 1)
		}
		query += " WHERE " + strings.Join(conditions, " AND ")
	}

	if b.limit > 0 {
		query += fmt.Sprintf(" LIMIT %d", b.limit)
	}

	return query, b.whereArgs
}

func (b *pgQueryBuilder) Dialect() string { return "postgres" }

// --- Postgres Migrator ---

type pgMigrator struct{}

func (m *pgMigrator) CreateTable(name string, columns map[string]string) string {
	var cols []string
	for colName, colType := range columns {
		pgType := mapToPostgresType(colType)
		cols = append(cols, fmt.Sprintf(`    "%s" %s`, colName, pgType))
	}
	return fmt.Sprintf("CREATE TABLE IF NOT EXISTS \"%s\" (\n%s\n);",
		name, strings.Join(cols, ",\n"))
}

func (m *pgMigrator) AddColumn(table, column, colType string) string {
	return fmt.Sprintf(`ALTER TABLE "%s" ADD COLUMN IF NOT EXISTS "%s" %s;`,
		table, column, mapToPostgresType(colType))
}

func (m *pgMigrator) MigrationTableSQL() string {
	return `CREATE TABLE IF NOT EXISTS "schema_migrations" (
    "version" BIGINT PRIMARY KEY,
    "applied_at" TIMESTAMPTZ DEFAULT NOW()
);`
}

func (m *pgMigrator) Dialect() string { return "postgres" }

func mapToPostgresType(generic string) string {
	switch strings.ToLower(generic) {
	case "string":
		return "TEXT"
	case "int":
		return "INTEGER"
	case "bool":
		return "BOOLEAN"
	case "timestamp":
		return "TIMESTAMPTZ"
	case "id":
		return "BIGSERIAL PRIMARY KEY"
	default:
		return generic
	}
}

// ---------------------------------------------------------------------
// SQLite family
// ---------------------------------------------------------------------

type sqliteKit struct{}

func NewSQLiteKit() DatabaseKit { return &sqliteKit{} }

func (k *sqliteKit) DriverName() string { return "sqlite" }

func (k *sqliteKit) CreateConnection(cfg ConnectionConfig) (Connection, error) {
	path := cfg.FilePath
	if path == "" {
		path = ":memory:"
	}
	fmt.Printf("  [sqlite] Opening database: %s\n", path)
	return &sqliteConnection{path: path, connected: true}, nil
}

func (k *sqliteKit) CreateQueryBuilder() QueryBuilder {
	return &sqliteQueryBuilder{}
}

func (k *sqliteKit) CreateMigrator() Migrator {
	return &sqliteMigrator{}
}

// --- SQLite Connection ---

type sqliteConnection struct {
	path      string
	connected bool
}

func (c *sqliteConnection) Execute(query string, args ...any) (Result, error) {
	if !c.connected {
		return Result{}, fmt.Errorf("sqlite: database closed")
	}
	fmt.Printf("  [sqlite] EXEC: %s  args=%v\n", query, args)
	return Result{RowsAffected: 1}, nil
}

func (c *sqliteConnection) Close() error {
	c.connected = false
	fmt.Println("  [sqlite] Database closed")
	return nil
}

func (c *sqliteConnection) DSN() string { return c.path }

// --- SQLite QueryBuilder ---

type sqliteQueryBuilder struct {
	table     string
	columns   []string
	where     []string
	whereArgs []any
	limit     int
}

func (b *sqliteQueryBuilder) Select(table string, columns ...string) QueryBuilder {
	b.table = table
	b.columns = columns
	return b
}

func (b *sqliteQueryBuilder) Where(condition string, args ...any) QueryBuilder {
	b.where = append(b.where, condition)
	b.whereArgs = append(b.whereArgs, args...)
	return b
}

func (b *sqliteQueryBuilder) Limit(n int) QueryBuilder {
	b.limit = n
	return b
}

func (b *sqliteQueryBuilder) Build() (string, []any) {
	cols := "*"
	if len(b.columns) > 0 {
		// SQLite uses backtick-quoted identifiers
		quoted := make([]string, len(b.columns))
		for i, c := range b.columns {
			quoted[i] = fmt.Sprintf("`%s`", c)
		}
		cols = strings.Join(quoted, ", ")
	}

	query := fmt.Sprintf("SELECT %s FROM `%s`", cols, b.table)

	if len(b.where) > 0 {
		// SQLite uses ? placeholders (no rewriting needed)
		query += " WHERE " + strings.Join(b.where, " AND ")
	}

	if b.limit > 0 {
		query += fmt.Sprintf(" LIMIT %d", b.limit)
	}

	return query, b.whereArgs
}

func (b *sqliteQueryBuilder) Dialect() string { return "sqlite" }

// --- SQLite Migrator ---

type sqliteMigrator struct{}

func (m *sqliteMigrator) CreateTable(name string, columns map[string]string) string {
	var cols []string
	for colName, colType := range columns {
		sqliteType := mapToSQLiteType(colType)
		cols = append(cols, fmt.Sprintf("    `%s` %s", colName, sqliteType))
	}
	return fmt.Sprintf("CREATE TABLE IF NOT EXISTS `%s` (\n%s\n);",
		name, strings.Join(cols, ",\n"))
}

func (m *sqliteMigrator) AddColumn(table, column, colType string) string {
	return fmt.Sprintf("ALTER TABLE `%s` ADD COLUMN `%s` %s;",
		table, column, mapToSQLiteType(colType))
}

func (m *sqliteMigrator) MigrationTableSQL() string {
	return "CREATE TABLE IF NOT EXISTS `schema_migrations` (\n" +
		"    `version` INTEGER PRIMARY KEY,\n" +
		"    `applied_at` TEXT DEFAULT (datetime('now'))\n);"
}

func (m *sqliteMigrator) Dialect() string { return "sqlite" }

func mapToSQLiteType(generic string) string {
	switch strings.ToLower(generic) {
	case "string":
		return "TEXT"
	case "int":
		return "INTEGER"
	case "bool":
		return "INTEGER" // SQLite has no native BOOLEAN
	case "timestamp":
		return "TEXT" // SQLite stores timestamps as TEXT
	case "id":
		return "INTEGER PRIMARY KEY AUTOINCREMENT"
	default:
		return generic
	}
}

// ---------------------------------------------------------------------
// Top-level factory function -- selects the right kit
// ---------------------------------------------------------------------

// NewDatabaseKit returns the appropriate abstract factory for the given driver.
func NewDatabaseKit(driver string) (DatabaseKit, error) {
	switch driver {
	case "postgres", "pg":
		return NewPostgresKit(), nil
	case "sqlite", "sqlite3":
		return NewSQLiteKit(), nil
	default:
		return nil, fmt.Errorf("unsupported database driver: %q", driver)
	}
}

// ---------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------

func main() {
	fmt.Println("=== Database Connection Factory (Abstract Factory) Demo ===")
	fmt.Println()

	drivers := []struct {
		name   string
		config ConnectionConfig
	}{
		{
			name: "postgres",
			config: ConnectionConfig{
				Host:     "db.prod.internal",
				Port:     5432,
				Database: "myapp",
				User:     "appuser",
				Password: "***",
				Options:  map[string]string{"sslmode": "require"},
			},
		},
		{
			name: "sqlite",
			config: ConnectionConfig{
				FilePath: "/var/lib/myapp/data.db",
			},
		},
	}

	for _, d := range drivers {
		fmt.Printf("=== %s ===\n\n", strings.ToUpper(d.name))

		// Create the abstract factory
		kit, err := NewDatabaseKit(d.name)
		if err != nil {
			fmt.Printf("Error: %v\n\n", err)
			continue
		}

		// Create a connection
		conn, err := kit.CreateConnection(d.config)
		if err != nil {
			fmt.Printf("Connection error: %v\n\n", err)
			continue
		}
		defer conn.Close()

		// Create a query builder -- guaranteed compatible with the connection
		qb := kit.CreateQueryBuilder()
		query, args := qb.
			Select("users", "id", "email", "created_at").
			Where("active = ?", true).
			Where("role = ?", "admin").
			Limit(10).
			Build()
		fmt.Printf("  Query (%s dialect): %s\n", qb.Dialect(), query)
		fmt.Printf("  Args: %v\n\n", args)

		// Execute through the connection
		result, err := conn.Execute(query, args...)
		if err != nil {
			fmt.Printf("  Exec error: %v\n", err)
			continue
		}
		fmt.Printf("  Rows affected: %d\n\n", result.RowsAffected)

		// Create a migrator -- also guaranteed compatible
		migrator := kit.CreateMigrator()
		createSQL := migrator.CreateTable("users", map[string]string{
			"id":         "id",
			"email":      "string",
			"active":     "bool",
			"created_at": "timestamp",
		})
		fmt.Printf("  Migration (%s dialect):\n%s\n\n", migrator.Dialect(), createSQL)

		// Show the migration tracking table
		fmt.Printf("  Migration table:\n%s\n\n", migrator.MigrationTableSQL())

		conn.Close()
		fmt.Println()
	}

	// Demonstrate that all objects from a kit share the same dialect
	fmt.Println("=== Dialect Consistency Check ===")
	for _, driverName := range []string{"postgres", "sqlite"} {
		kit, _ := NewDatabaseKit(driverName)
		qb := kit.CreateQueryBuilder()
		mig := kit.CreateMigrator()
		fmt.Printf("  %s kit -> QueryBuilder=%s, Migrator=%s (consistent: %v)\n",
			driverName, qb.Dialect(), mig.Dialect(), qb.Dialect() == mig.Dialect())
	}
}
