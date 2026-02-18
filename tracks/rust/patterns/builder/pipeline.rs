// Builder Pattern: Type-Safe Data Processing Pipeline
//
// Demonstrates a generic pipeline builder where each stage's output type
// must match the next stage's input type. The type system enforces
// pipeline correctness at compile time -- you cannot chain incompatible stages.
//
// Run: rustc pipeline.rs && ./pipeline

use std::fmt;

// --- Stage trait ---

trait Stage<Input, Output> {
    fn process(&self, input: Input) -> Output;
    fn name(&self) -> &str;
}

// --- Pipeline types ---

// A pipeline that takes `In` and produces `Out`
struct Pipeline<In, Out> {
    stages: Vec<String>,  // stage names for display
    runner: Box<dyn Fn(In) -> Out>,
}

impl<In, Out> Pipeline<In, Out> {
    fn run(&self, input: In) -> Out {
        (self.runner)(input)
    }

    fn stage_names(&self) -> &[String] {
        &self.stages
    }
}

impl<In, Out: fmt::Debug> Pipeline<In, Out> {
    fn run_verbose(&self, input: In) -> Out {
        println!("Pipeline: {} stages", self.stages.len());
        for (i, name) in self.stages.iter().enumerate() {
            println!("  [{}] {}", i + 1, name);
        }
        let result = self.run(input);
        println!("  Result: {:?}", result);
        result
    }
}

// --- Pipeline Builder ---

struct PipelineBuilder<In, Current> {
    stages: Vec<String>,
    runner: Box<dyn Fn(In) -> Current>,
}

impl<In: 'static> PipelineBuilder<In, In> {
    fn new() -> Self {
        Self {
            stages: Vec::new(),
            runner: Box::new(|input| input), // identity function
        }
    }
}

impl<In: 'static, Current: 'static> PipelineBuilder<In, Current> {
    // Add a stage: the stage's input must match the pipeline's current output type.
    // The pipeline's output type changes to the stage's output type.
    fn then<Out: 'static>(
        self,
        stage: impl Stage<Current, Out> + 'static,
    ) -> PipelineBuilder<In, Out> {
        let name = stage.name().to_string();
        let prev_runner = self.runner;
        let mut stages = self.stages;
        stages.push(name);

        PipelineBuilder {
            stages,
            runner: Box::new(move |input: In| {
                let intermediate = prev_runner(input);
                stage.process(intermediate)
            }),
        }
    }

    // Add a stage from a closure
    fn then_fn<Out: 'static>(
        self,
        name: &str,
        f: impl Fn(Current) -> Out + 'static,
    ) -> PipelineBuilder<In, Out> {
        let name = name.to_string();
        let prev_runner = self.runner;
        let mut stages = self.stages;
        stages.push(name);

        PipelineBuilder {
            stages,
            runner: Box::new(move |input: In| {
                let intermediate = prev_runner(input);
                f(intermediate)
            }),
        }
    }

    fn build(self) -> Pipeline<In, Current> {
        Pipeline {
            stages: self.stages,
            runner: self.runner,
        }
    }
}

// --- Concrete stages for a log processing pipeline ---

// Stage 1: Parse raw log line into structured fields
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

struct ParseStage;

impl Stage<String, Result<LogEntry, String>> for ParseStage {
    fn process(&self, input: String) -> Result<LogEntry, String> {
        // Expected format: "2025-01-15T10:30:00 [INFO] service: message text"
        let parts: Vec<&str> = input.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return Err(format!("invalid log format: '{}'", input));
        }

        let level = parts[1]
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();

        let source = parts[2].trim_end_matches(':').to_string();

        Ok(LogEntry {
            timestamp: parts[0].to_string(),
            level,
            source,
            message: parts[3].to_string(),
        })
    }

    fn name(&self) -> &str {
        "parse"
    }
}

// Stage 2: Filter by log level
struct FilterStage {
    min_level: String,
}

impl FilterStage {
    fn new(min_level: &str) -> Self {
        Self {
            min_level: min_level.to_uppercase(),
        }
    }

    fn level_priority(level: &str) -> u8 {
        match level {
            "DEBUG" => 0,
            "INFO" => 1,
            "WARN" => 2,
            "ERROR" => 3,
            "FATAL" => 4,
            _ => 0,
        }
    }
}

impl Stage<Result<LogEntry, String>, Option<LogEntry>> for FilterStage {
    fn process(&self, input: Result<LogEntry, String>) -> Option<LogEntry> {
        match input {
            Ok(entry) => {
                let entry_priority = Self::level_priority(&entry.level);
                let min_priority = Self::level_priority(&self.min_level);
                if entry_priority >= min_priority {
                    Some(entry)
                } else {
                    None
                }
            }
            Err(_) => None, // drop unparseable entries
        }
    }

    fn name(&self) -> &str {
        "filter"
    }
}

// Stage 3: Enrich with metadata
#[derive(Debug, Clone)]
struct EnrichedEntry {
    entry: LogEntry,
    hostname: String,
    environment: String,
}

struct EnrichStage {
    hostname: String,
    environment: String,
}

impl Stage<Option<LogEntry>, Option<EnrichedEntry>> for EnrichStage {
    fn process(&self, input: Option<LogEntry>) -> Option<EnrichedEntry> {
        input.map(|entry| EnrichedEntry {
            entry,
            hostname: self.hostname.clone(),
            environment: self.environment.clone(),
        })
    }

    fn name(&self) -> &str {
        "enrich"
    }
}

// Stage 4: Format for output
struct FormatStage {
    format: OutputFormat,
}

enum OutputFormat {
    Json,
    Logfmt,
}

impl Stage<Option<EnrichedEntry>, String> for FormatStage {
    fn process(&self, input: Option<EnrichedEntry>) -> String {
        match input {
            None => String::new(),
            Some(enriched) => match self.format {
                OutputFormat::Json => {
                    format!(
                        r#"{{"timestamp":"{}","level":"{}","source":"{}","message":"{}","host":"{}","env":"{}"}}"#,
                        enriched.entry.timestamp,
                        enriched.entry.level,
                        enriched.entry.source,
                        enriched.entry.message,
                        enriched.hostname,
                        enriched.environment,
                    )
                }
                OutputFormat::Logfmt => {
                    format!(
                        "ts={} level={} source={} msg=\"{}\" host={} env={}",
                        enriched.entry.timestamp,
                        enriched.entry.level,
                        enriched.entry.source,
                        enriched.entry.message,
                        enriched.hostname,
                        enriched.environment,
                    )
                }
            },
        }
    }

    fn name(&self) -> &str {
        "format"
    }
}

// --- Demo ---

fn main() {
    println!("=== Type-Safe Pipeline Builder ===\n");

    // Build a log processing pipeline:
    // String -> Result<LogEntry> -> Option<LogEntry> -> Option<EnrichedEntry> -> String
    //
    // Each stage's input type must match the previous stage's output type.
    // The compiler enforces this -- you cannot reorder or skip stages
    // without getting a type error.

    let pipeline = PipelineBuilder::new()
        .then(ParseStage)
        .then(FilterStage::new("WARN"))
        .then(EnrichStage {
            hostname: "web-prod-01".to_string(),
            environment: "production".to_string(),
        })
        .then(FormatStage {
            format: OutputFormat::Json,
        })
        .build();

    println!("--- Processing log lines ---\n");

    let log_lines = vec![
        "2025-01-15T10:30:00 [INFO] auth: user login successful",
        "2025-01-15T10:30:01 [WARN] ratelimit: approaching threshold for IP 10.0.0.5",
        "2025-01-15T10:30:02 [ERROR] db: connection pool exhausted",
        "2025-01-15T10:30:03 [DEBUG] cache: key 'session:abc' expired",
        "malformed log line without proper format",
    ];

    for line in &log_lines {
        let result = pipeline.run(line.to_string());
        if !result.is_empty() {
            println!("  {}", result);
        }
    }

    // Build a different pipeline with logfmt output
    println!("\n--- Logfmt pipeline ---\n");

    let logfmt_pipeline = PipelineBuilder::new()
        .then(ParseStage)
        .then(FilterStage::new("ERROR"))
        .then(EnrichStage {
            hostname: "api-prod-03".to_string(),
            environment: "production".to_string(),
        })
        .then(FormatStage {
            format: OutputFormat::Logfmt,
        })
        .build();

    for line in &log_lines {
        let result = logfmt_pipeline.run(line.to_string());
        if !result.is_empty() {
            println!("  {}", result);
        }
    }

    // Pipeline with closure stages
    println!("\n--- Pipeline with closure stages ---\n");

    let counting_pipeline: Pipeline<Vec<i32>, String> = PipelineBuilder::new()
        .then_fn("filter_positive", |nums: Vec<i32>| -> Vec<i32> {
            nums.into_iter().filter(|&n| n > 0).collect()
        })
        .then_fn("sum", |nums: Vec<i32>| -> i64 {
            nums.iter().map(|&n| n as i64).sum()
        })
        .then_fn("format", |total: i64| -> String {
            format!("Total: {}", total)
        })
        .build();

    let data = vec![-3, 5, -1, 8, 2, -7, 10];
    println!("  Input: {:?}", data);
    counting_pipeline.run_verbose(data);

    // Type safety demo: the following would NOT compile because
    // the types don't chain correctly:
    //
    // let broken = PipelineBuilder::<String, String>::new()
    //     .then(FilterStage::new("WARN"))  // ERROR: FilterStage expects Result<LogEntry>
    //     .build();                        // but pipeline currently outputs String
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_processes_valid_log() {
        let pipeline = PipelineBuilder::new()
            .then(ParseStage)
            .then(FilterStage::new("INFO"))
            .then(EnrichStage {
                hostname: "test".to_string(),
                environment: "test".to_string(),
            })
            .then(FormatStage {
                format: OutputFormat::Json,
            })
            .build();

        let result = pipeline.run(
            "2025-01-15T10:30:00 [ERROR] db: connection failed".to_string(),
        );
        assert!(result.contains("\"level\":\"ERROR\""));
        assert!(result.contains("\"host\":\"test\""));
    }

    #[test]
    fn test_pipeline_filters_below_threshold() {
        let pipeline = PipelineBuilder::new()
            .then(ParseStage)
            .then(FilterStage::new("ERROR"))
            .then(EnrichStage {
                hostname: "test".to_string(),
                environment: "test".to_string(),
            })
            .then(FormatStage {
                format: OutputFormat::Json,
            })
            .build();

        let result = pipeline.run(
            "2025-01-15T10:30:00 [INFO] auth: login ok".to_string(),
        );
        assert!(result.is_empty(), "INFO should be filtered when min is ERROR");
    }

    #[test]
    fn test_pipeline_handles_malformed_input() {
        let pipeline = PipelineBuilder::new()
            .then(ParseStage)
            .then(FilterStage::new("DEBUG"))
            .then(EnrichStage {
                hostname: "test".to_string(),
                environment: "test".to_string(),
            })
            .then(FormatStage {
                format: OutputFormat::Json,
            })
            .build();

        let result = pipeline.run("bad input".to_string());
        assert!(result.is_empty());
    }

    #[test]
    fn test_closure_pipeline() {
        let pipeline = PipelineBuilder::new()
            .then_fn("double", |x: i32| x * 2)
            .then_fn("to_string", |x: i32| format!("result: {}", x))
            .build();

        assert_eq!(pipeline.run(21), "result: 42");
    }

    #[test]
    fn test_pipeline_stage_names() {
        let pipeline = PipelineBuilder::new()
            .then(ParseStage)
            .then(FilterStage::new("INFO"))
            .then_fn("custom", |entry: Option<LogEntry>| {
                entry.map(|e| e.message).unwrap_or_default()
            })
            .build();

        let names = pipeline.stage_names();
        assert_eq!(names, &["parse", "filter", "custom"]);
    }

    #[test]
    fn test_identity_pipeline() {
        let pipeline: Pipeline<String, String> = PipelineBuilder::new().build();
        assert_eq!(pipeline.run("hello".to_string()), "hello");
    }
}
