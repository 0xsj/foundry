// Builder Pattern: Data Pipeline Builder
//
// Demonstrates a builder for composable ETL-style data pipelines.
// Pipeline stages are added via fluent chaining, and the final Build()
// validates the pipeline (source required, at least one sink, etc.).
//
// Run: go run ./pipeline/

package main

import (
	"fmt"
	"strings"
)

// --- Stage Types ---

// Record represents a single data record flowing through the pipeline.
type Record map[string]string

// Source reads records from an external data source.
type Source interface {
	Name() string
	Read() ([]Record, error)
}

// Transform modifies, filters, or enriches records.
type Transform interface {
	Name() string
	Process(records []Record) ([]Record, error)
}

// Sink writes records to an external destination.
type Sink interface {
	Name() string
	Write(records []Record) error
}

// --- Concrete Sources ---

// CSVSource simulates reading from a CSV file.
type CSVSource struct {
	path string
	data []Record // Simulated data for demo
}

func NewCSVSource(path string, data []Record) *CSVSource {
	return &CSVSource{path: path, data: data}
}

func (s *CSVSource) Name() string { return fmt.Sprintf("CSV(%s)", s.path) }
func (s *CSVSource) Read() ([]Record, error) {
	fmt.Printf("  [source] Reading %d records from %s\n", len(s.data), s.path)
	return s.data, nil
}

// --- Concrete Transforms ---

// FilterEmpty removes records with empty values in the specified field.
type FilterEmpty struct {
	field string
}

func NewFilterEmpty(field string) *FilterEmpty {
	return &FilterEmpty{field: field}
}

func (t *FilterEmpty) Name() string { return fmt.Sprintf("FilterEmpty(%s)", t.field) }
func (t *FilterEmpty) Process(records []Record) ([]Record, error) {
	var result []Record
	for _, r := range records {
		if val, ok := r[t.field]; ok && val != "" {
			result = append(result, r)
		}
	}
	fmt.Printf("  [transform] %s: %d -> %d records\n", t.Name(), len(records), len(result))
	return result, nil
}

// Normalize lowercases all values in the specified field.
type Normalize struct {
	field string
}

func NewNormalize(field string) *Normalize {
	return &Normalize{field: field}
}

func (t *Normalize) Name() string { return fmt.Sprintf("Normalize(%s)", t.field) }
func (t *Normalize) Process(records []Record) ([]Record, error) {
	for i := range records {
		if val, ok := records[i][t.field]; ok {
			records[i][t.field] = strings.ToLower(val)
		}
	}
	fmt.Printf("  [transform] %s: normalized %d records\n", t.Name(), len(records))
	return records, nil
}

// AddField adds a computed field to each record.
type AddField struct {
	name    string
	compute func(Record) string
}

func NewAddField(name string, compute func(Record) string) *AddField {
	return &AddField{name: name, compute: compute}
}

func (t *AddField) Name() string { return fmt.Sprintf("AddField(%s)", t.name) }
func (t *AddField) Process(records []Record) ([]Record, error) {
	for i := range records {
		records[i][t.name] = t.compute(records[i])
	}
	fmt.Printf("  [transform] %s: enriched %d records\n", t.Name(), len(records))
	return records, nil
}

// --- Concrete Sinks ---

// JSONSink simulates writing records as JSON.
type JSONSink struct {
	path string
}

func NewJSONSink(path string) *JSONSink {
	return &JSONSink{path: path}
}

func (s *JSONSink) Name() string { return fmt.Sprintf("JSON(%s)", s.path) }
func (s *JSONSink) Write(records []Record) error {
	fmt.Printf("  [sink] Writing %d records to %s\n", len(records), s.path)
	for i, r := range records {
		if i >= 3 {
			fmt.Printf("    ... and %d more\n", len(records)-3)
			break
		}
		fmt.Printf("    %v\n", r)
	}
	return nil
}

// ConsoleSink prints records to stdout (for debugging).
type ConsoleSink struct{}

func (s *ConsoleSink) Name() string { return "Console" }
func (s *ConsoleSink) Write(records []Record) error {
	fmt.Printf("  [sink] Console output (%d records):\n", len(records))
	for _, r := range records {
		fmt.Printf("    %v\n", r)
	}
	return nil
}

// --- Pipeline Builder ---

// Pipeline represents a configured data pipeline ready to execute.
type Pipeline struct {
	name       string
	source     Source
	transforms []Transform
	sinks      []Sink
}

// Run executes the pipeline: read -> transform -> write.
func (p *Pipeline) Run() error {
	fmt.Printf("\n--- Running pipeline: %s ---\n", p.name)

	// Read from source
	records, err := p.source.Read()
	if err != nil {
		return fmt.Errorf("source %s failed: %w", p.source.Name(), err)
	}

	// Apply transforms in order
	for _, t := range p.transforms {
		records, err = t.Process(records)
		if err != nil {
			return fmt.Errorf("transform %s failed: %w", t.Name(), err)
		}
	}

	// Write to all sinks
	for _, s := range p.sinks {
		if err := s.Write(records); err != nil {
			return fmt.Errorf("sink %s failed: %w", s.Name(), err)
		}
	}

	fmt.Printf("--- Pipeline %s completed ---\n", p.name)
	return nil
}

// String returns a description of the pipeline stages.
func (p *Pipeline) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Pipeline[%s]\n", p.name)
	fmt.Fprintf(&b, "  Source: %s\n", p.source.Name())
	for i, t := range p.transforms {
		fmt.Fprintf(&b, "  Transform %d: %s\n", i+1, t.Name())
	}
	for i, s := range p.sinks {
		fmt.Fprintf(&b, "  Sink %d: %s\n", i+1, s.Name())
	}
	return b.String()
}

// PipelineBuilder constructs Pipeline instances with fluent chaining.
type PipelineBuilder struct {
	name       string
	source     Source
	transforms []Transform
	sinks      []Sink
	err        error
}

// NewPipeline creates a new PipelineBuilder.
func NewPipeline(name string) *PipelineBuilder {
	if name == "" {
		return &PipelineBuilder{err: fmt.Errorf("pipeline name is required")}
	}
	return &PipelineBuilder{name: name}
}

// Source sets the data source for the pipeline. Only one source is allowed.
func (pb *PipelineBuilder) Source(s Source) *PipelineBuilder {
	if pb.err != nil {
		return pb
	}
	if s == nil {
		pb.err = fmt.Errorf("source cannot be nil")
		return pb
	}
	if pb.source != nil {
		pb.err = fmt.Errorf("pipeline already has a source (%s); only one source is allowed", pb.source.Name())
		return pb
	}
	pb.source = s
	return pb
}

// Transform appends a transform stage to the pipeline.
// Transforms are applied in the order they are added.
func (pb *PipelineBuilder) Transform(t Transform) *PipelineBuilder {
	if pb.err != nil {
		return pb
	}
	if t == nil {
		pb.err = fmt.Errorf("transform cannot be nil")
		return pb
	}
	pb.transforms = append(pb.transforms, t)
	return pb
}

// Sink appends an output sink. Multiple sinks are supported (fan-out).
func (pb *PipelineBuilder) Sink(s Sink) *PipelineBuilder {
	if pb.err != nil {
		return pb
	}
	if s == nil {
		pb.err = fmt.Errorf("sink cannot be nil")
		return pb
	}
	pb.sinks = append(pb.sinks, s)
	return pb
}

// Build validates the pipeline configuration and returns a runnable Pipeline.
func (pb *PipelineBuilder) Build() (*Pipeline, error) {
	if pb.err != nil {
		return nil, fmt.Errorf("pipeline build error: %w", pb.err)
	}
	if pb.source == nil {
		return nil, fmt.Errorf("pipeline %q requires a source", pb.name)
	}
	if len(pb.sinks) == 0 {
		return nil, fmt.Errorf("pipeline %q requires at least one sink", pb.name)
	}
	return &Pipeline{
		name:       pb.name,
		source:     pb.source,
		transforms: pb.transforms,
		sinks:      pb.sinks,
	}, nil
}

// --- Main ---

func main() {
	fmt.Println("=== Data Pipeline Builder ===")
	fmt.Println(strings.Repeat("-", 55))

	// Sample data (simulating CSV records)
	sampleData := []Record{
		{"name": "Alice Johnson", "email": "ALICE@EXAMPLE.COM", "department": "Engineering"},
		{"name": "Bob Smith", "email": "", "department": "Marketing"},
		{"name": "Charlie Brown", "email": "CHARLIE@EXAMPLE.COM", "department": "Engineering"},
		{"name": "Diana Prince", "email": "diana@example.com", "department": ""},
		{"name": "Eve Adams", "email": "EVE@EXAMPLE.COM", "department": "Engineering"},
	}

	// Example 1: Full ETL pipeline
	fmt.Println("\n1. Full ETL Pipeline:")
	pipeline, err := NewPipeline("user-export").
		Source(NewCSVSource("users.csv", sampleData)).
		Transform(NewFilterEmpty("email")).
		Transform(NewFilterEmpty("department")).
		Transform(NewNormalize("email")).
		Transform(NewAddField("source", func(r Record) string {
			return "csv-import"
		})).
		Sink(NewJSONSink("output/users.json")).
		Sink(&ConsoleSink{}).
		Build()

	if err != nil {
		fmt.Printf("   Build error: %v\n", err)
	} else {
		fmt.Print(pipeline)
		if err := pipeline.Run(); err != nil {
			fmt.Printf("   Run error: %v\n", err)
		}
	}

	// Example 2: Minimal pipeline (source + sink, no transforms)
	fmt.Println("\n2. Pass-through Pipeline (no transforms):")
	passthrough, err := NewPipeline("raw-export").
		Source(NewCSVSource("users.csv", sampleData)).
		Sink(&ConsoleSink{}).
		Build()

	if err != nil {
		fmt.Printf("   Build error: %v\n", err)
	} else {
		fmt.Print(passthrough)
		if err := passthrough.Run(); err != nil {
			fmt.Printf("   Run error: %v\n", err)
		}
	}

	// Example 3: Validation errors
	fmt.Println("\n3. Validation errors:")

	// No source
	_, err = NewPipeline("bad").
		Transform(NewFilterEmpty("email")).
		Sink(&ConsoleSink{}).
		Build()
	fmt.Printf("   No source: %v\n", err)

	// No sink
	_, err = NewPipeline("bad").
		Source(NewCSVSource("users.csv", nil)).
		Build()
	fmt.Printf("   No sink:   %v\n", err)

	// Empty name
	_, err = NewPipeline("").
		Source(NewCSVSource("users.csv", nil)).
		Sink(&ConsoleSink{}).
		Build()
	fmt.Printf("   No name:   %v\n", err)

	// Duplicate source
	_, err = NewPipeline("bad").
		Source(NewCSVSource("a.csv", nil)).
		Source(NewCSVSource("b.csv", nil)).
		Sink(&ConsoleSink{}).
		Build()
	fmt.Printf("   Two sources: %v\n", err)

	fmt.Println(strings.Repeat("-", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Builder enforces pipeline invariants (source required, sink required)")
	fmt.Println("- Transforms are composable and order-dependent")
	fmt.Println("- Multiple sinks enable fan-out (write to file AND console)")
	fmt.Println("- Build() validates before returning a runnable Pipeline")
	fmt.Println("- Interfaces allow custom sources, transforms, and sinks")
}
