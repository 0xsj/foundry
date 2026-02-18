// Document Parser Factory
//
// Demonstrates: Factory Method pattern
//
// A configuration loader that parses different file formats (JSON, YAML, TOML).
// Each parser has its own factory that knows how to create and configure the
// right parser for its format. The factory method pattern is used because
// each parser has different initialization requirements.
//
// Run: go run .
package main

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"
)

// ---------------------------------------------------------------------
// Product interfaces -- what parsers produce and consume
// ---------------------------------------------------------------------

// Document represents a parsed configuration document.
type Document struct {
	Format   string
	Data     map[string]any
	Metadata DocumentMetadata
}

// DocumentMetadata holds information about the parsing process.
type DocumentMetadata struct {
	Parser    string
	Strict    bool
	Warnings  []string
}

// Parser reads raw bytes and produces a Document.
type Parser interface {
	Parse(data []byte) (*Document, error)
	Format() string
}

// ---------------------------------------------------------------------
// Factory Method interface -- how parsers are created
// ---------------------------------------------------------------------

// ParserFactory defines the factory method interface.
// Each format implements this to create its own parser.
type ParserFactory interface {
	// Create returns a parser configured for this factory's format.
	Create(opts ParserOptions) Parser

	// Extensions returns file extensions this factory handles (e.g., [".json"]).
	Extensions() []string

	// Name returns the format name.
	Name() string
}

// ParserOptions controls parser behavior.
type ParserOptions struct {
	Strict          bool // fail on unknown fields
	AllowComments   bool // allow comments in formats that support it
	MaxDepth        int  // maximum nesting depth (0 = unlimited)
}

// DefaultParserOptions returns sensible defaults.
func DefaultParserOptions() ParserOptions {
	return ParserOptions{
		Strict:   false,
		MaxDepth: 64,
	}
}

// ---------------------------------------------------------------------
// JSON Parser + Factory
// ---------------------------------------------------------------------

type jsonParser struct {
	strict   bool
	maxDepth int
}

func (p *jsonParser) Parse(data []byte) (*Document, error) {
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		return nil, fmt.Errorf("json parse error: %w", err)
	}

	var warnings []string

	// Check nesting depth if configured
	if p.maxDepth > 0 {
		depth := measureDepth(raw)
		if depth > p.maxDepth {
			return nil, fmt.Errorf("json: nesting depth %d exceeds maximum %d", depth, p.maxDepth)
		}
		if depth > p.maxDepth/2 {
			warnings = append(warnings, fmt.Sprintf("deep nesting detected: %d levels", depth))
		}
	}

	return &Document{
		Format: "json",
		Data:   raw,
		Metadata: DocumentMetadata{
			Parser:   "encoding/json",
			Strict:   p.strict,
			Warnings: warnings,
		},
	}, nil
}

func (p *jsonParser) Format() string { return "json" }

// jsonFactory creates JSON parsers.
type jsonFactory struct{}

func (f *jsonFactory) Create(opts ParserOptions) Parser {
	return &jsonParser{
		strict:   opts.Strict,
		maxDepth: opts.MaxDepth,
	}
}

func (f *jsonFactory) Extensions() []string { return []string{".json"} }
func (f *jsonFactory) Name() string         { return "JSON" }

// ---------------------------------------------------------------------
// YAML Parser + Factory (simulated -- real impl would use gopkg.in/yaml.v3)
// ---------------------------------------------------------------------

type yamlParser struct {
	strict        bool
	allowComments bool
}

func (p *yamlParser) Parse(data []byte) (*Document, error) {
	// Simulated YAML parsing -- in production, use gopkg.in/yaml.v3
	// For this example, we treat simple "key: value" lines as YAML
	result := make(map[string]any)
	var warnings []string

	lines := strings.Split(string(data), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "---") {
			continue
		}
		if strings.HasPrefix(line, "#") {
			if !p.allowComments {
				warnings = append(warnings, fmt.Sprintf("line %d: comment ignored", i+1))
			}
			continue
		}
		parts := strings.SplitN(line, ":", 2)
		if len(parts) != 2 {
			if p.strict {
				return nil, fmt.Errorf("yaml: invalid line %d: %q", i+1, line)
			}
			warnings = append(warnings, fmt.Sprintf("line %d: skipped invalid line", i+1))
			continue
		}
		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])
		result[key] = value
	}

	return &Document{
		Format: "yaml",
		Data:   result,
		Metadata: DocumentMetadata{
			Parser:   "simulated-yaml",
			Strict:   p.strict,
			Warnings: warnings,
		},
	}, nil
}

func (p *yamlParser) Format() string { return "yaml" }

// yamlFactory creates YAML parsers.
type yamlFactory struct{}

func (f *yamlFactory) Create(opts ParserOptions) Parser {
	return &yamlParser{
		strict:        opts.Strict,
		allowComments: opts.AllowComments,
	}
}

func (f *yamlFactory) Extensions() []string { return []string{".yaml", ".yml"} }
func (f *yamlFactory) Name() string         { return "YAML" }

// ---------------------------------------------------------------------
// TOML Parser + Factory (simulated)
// ---------------------------------------------------------------------

type tomlParser struct {
	strict bool
}

func (p *tomlParser) Parse(data []byte) (*Document, error) {
	// Simulated TOML parsing -- in production, use github.com/BurntSushi/toml
	result := make(map[string]any)
	var warnings []string
	currentSection := ""

	lines := strings.Split(string(data), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Section header: [section]
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			currentSection = strings.Trim(line, "[]")
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			if p.strict {
				return nil, fmt.Errorf("toml: invalid line %d: %q", i+1, line)
			}
			continue
		}

		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])
		value = strings.Trim(value, `"`)

		if currentSection != "" {
			key = currentSection + "." + key
		}
		result[key] = value
	}

	if len(warnings) == 0 {
		warnings = nil
	}

	return &Document{
		Format: "toml",
		Data:   result,
		Metadata: DocumentMetadata{
			Parser:   "simulated-toml",
			Strict:   p.strict,
			Warnings: warnings,
		},
	}, nil
}

func (p *tomlParser) Format() string { return "toml" }

// tomlFactory creates TOML parsers.
type tomlFactory struct{}

func (f *tomlFactory) Create(opts ParserOptions) Parser {
	return &tomlParser{strict: opts.Strict}
}

func (f *tomlFactory) Extensions() []string { return []string{".toml"} }
func (f *tomlFactory) Name() string         { return "TOML" }

// ---------------------------------------------------------------------
// Factory registry -- maps extensions to factories
// ---------------------------------------------------------------------

// ParserRegistry holds all registered parser factories.
type ParserRegistry struct {
	factories  map[string]ParserFactory // extension -> factory
	byName     map[string]ParserFactory // format name -> factory
}

func NewParserRegistry() *ParserRegistry {
	return &ParserRegistry{
		factories: make(map[string]ParserFactory),
		byName:    make(map[string]ParserFactory),
	}
}

// RegisterFactory adds a parser factory for all its extensions.
func (r *ParserRegistry) RegisterFactory(factory ParserFactory) {
	r.byName[strings.ToLower(factory.Name())] = factory
	for _, ext := range factory.Extensions() {
		r.factories[ext] = factory
	}
}

// ParserForFile returns a parser appropriate for the given filename.
func (r *ParserRegistry) ParserForFile(filename string, opts ParserOptions) (Parser, error) {
	ext := strings.ToLower(filepath.Ext(filename))
	factory, ok := r.factories[ext]
	if !ok {
		supported := make([]string, 0, len(r.factories))
		for ext := range r.factories {
			supported = append(supported, ext)
		}
		return nil, fmt.Errorf("no parser for extension %q (supported: %s)",
			ext, strings.Join(supported, ", "))
	}
	return factory.Create(opts), nil
}

// ParserByName returns a parser by format name.
func (r *ParserRegistry) ParserByName(name string, opts ParserOptions) (Parser, error) {
	factory, ok := r.byName[strings.ToLower(name)]
	if !ok {
		return nil, fmt.Errorf("no parser named %q", name)
	}
	return factory.Create(opts), nil
}

// ---------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------

func measureDepth(m map[string]any) int {
	maxD := 1
	for _, v := range m {
		if nested, ok := v.(map[string]any); ok {
			d := 1 + measureDepth(nested)
			if d > maxD {
				maxD = d
			}
		}
	}
	return maxD
}

// ---------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------

func main() {
	fmt.Println("=== Document Parser Factory Demo ===")
	fmt.Println()

	// Set up the registry with all available parser factories
	reg := NewParserRegistry()
	reg.RegisterFactory(&jsonFactory{})
	reg.RegisterFactory(&yamlFactory{})
	reg.RegisterFactory(&tomlFactory{})

	// Simulated config files
	files := map[string][]byte{
		"config.json": []byte(`{
			"database": {
				"host": "localhost",
				"port": 5432,
				"name": "myapp"
			},
			"cache_ttl": 300
		}`),

		"config.yaml": []byte(`# Application config
database_host: localhost
database_port: 5432
app_name: myservice
debug: true`),

		"config.toml": []byte(`# Server configuration
[server]
host = "0.0.0.0"
port = "8080"

[database]
driver = "postgres"
dsn = "postgres://localhost/myapp"`),
	}

	opts := DefaultParserOptions()

	for filename, content := range files {
		fmt.Printf("--- Parsing: %s ---\n", filename)

		// Factory creates the right parser based on file extension
		parser, err := reg.ParserForFile(filename, opts)
		if err != nil {
			fmt.Printf("  Error: %v\n\n", err)
			continue
		}

		doc, err := parser.Parse(content)
		if err != nil {
			fmt.Printf("  Parse error: %v\n\n", err)
			continue
		}

		fmt.Printf("  Format: %s (parser: %s)\n", doc.Format, doc.Metadata.Parser)
		fmt.Printf("  Keys: ")
		for k, v := range doc.Data {
			fmt.Printf("%s=%v  ", k, v)
		}
		fmt.Println()

		if len(doc.Metadata.Warnings) > 0 {
			fmt.Printf("  Warnings: %v\n", doc.Metadata.Warnings)
		}
		fmt.Println()
	}

	// Demonstrate error for unsupported format
	fmt.Println("--- Unsupported format ---")
	_, err := reg.ParserForFile("config.xml", opts)
	if err != nil {
		fmt.Printf("  Expected error: %v\n", err)
	}

	// Demonstrate strict mode
	fmt.Println("\n--- Strict mode (YAML with invalid line) ---")
	strictOpts := ParserOptions{Strict: true}
	parser, _ := reg.ParserByName("yaml", strictOpts)
	_, err = parser.Parse([]byte("valid_key: value\nthis is not valid yaml"))
	if err != nil {
		fmt.Printf("  Strict error: %v\n", err)
	}
}
