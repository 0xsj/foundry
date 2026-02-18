// Dispatch — Interfaces & Traits (Rust)
//
// Covers: impl Trait vs dyn Trait, static vs dynamic dispatch,
//         trait objects (&dyn, Box<dyn>), object safety.
//
// Run: rustc dispatch.rs && ./dispatch

#![allow(dead_code)]

use std::fmt;

// -------------------------------------------------------------------------
// Shared trait for all examples
// -------------------------------------------------------------------------

trait Formatter {
    fn format(&self, payload: &str) -> String;
    fn media_type(&self) -> &'static str;
}

#[derive(Debug)]
struct JsonFormatter;
#[derive(Debug)]
struct CsvFormatter { delimiter: char }
#[derive(Debug)]
struct PlainFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, payload: &str) -> String {
        format!(r#"{{"data":"{}"}}"#, payload)
    }
    fn media_type(&self) -> &'static str { "application/json" }
}

impl Formatter for CsvFormatter {
    fn format(&self, payload: &str) -> String {
        // Minimal CSV: escape commas by quoting
        let parts: Vec<&str> = payload.split(',').collect();
        parts.join(&self.delimiter.to_string())
    }
    fn media_type(&self) -> &'static str { "text/csv" }
}

impl Formatter for PlainFormatter {
    fn format(&self, payload: &str) -> String {
        payload.to_string()
    }
    fn media_type(&self) -> &'static str { "text/plain" }
}

// -------------------------------------------------------------------------
// 1. impl Trait — static dispatch
// -------------------------------------------------------------------------

// impl Trait in argument position:
// The compiler generates a separate, optimized function for each concrete type T.
// The concrete type is determined at the call site at compile time.
// No runtime overhead — can be inlined.
fn encode_static(formatter: &impl Formatter, payload: &str) -> String {
    let body = formatter.format(payload);
    format!("Content-Type: {}\n\n{}", formatter.media_type(), body)
}

// impl Trait in return position:
// The function returns ONE specific concrete type, opaque to the caller.
// The caller knows "I get back something that is a Formatter", not which type.
//
// Limitation: a function with -> impl Formatter can only ever return ONE concrete type.
// You cannot have `if condition { JsonFormatter } else { CsvFormatter }` here.
fn default_formatter() -> impl Formatter {
    JsonFormatter
}

// -------------------------------------------------------------------------
// 2. dyn Trait — dynamic dispatch
// -------------------------------------------------------------------------

// &dyn Formatter: borrowed trait object.
// At runtime, this is a fat pointer: (pointer to data, pointer to vtable).
// The vtable holds function pointers. Dispatch = one indirection.
fn encode_dynamic(formatter: &dyn Formatter, payload: &str) -> String {
    let body = formatter.format(payload);
    format!("Content-Type: {}\n\n{}", formatter.media_type(), body)
}

// Box<dyn Formatter>: owned trait object.
// Used when storing in a struct, returning from a function,
// or building a heterogeneous collection.
fn pick_formatter(content_type: &str) -> Box<dyn Formatter> {
    match content_type {
        "json" => Box::new(JsonFormatter),
        "csv"  => Box::new(CsvFormatter { delimiter: ',' }),
        _      => Box::new(PlainFormatter),
    }
}

// A struct that stores a heterogeneous list of formatters.
// Impossible with impl Trait (all elements would have to be the same type).
// Vec<Box<dyn Formatter>> works because Box<dyn Formatter> has a fixed size.
struct FormatterPipeline {
    stages: Vec<Box<dyn Formatter>>,
}

impl FormatterPipeline {
    fn new() -> Self {
        FormatterPipeline { stages: Vec::new() }
    }

    fn add(&mut self, f: Box<dyn Formatter>) {
        self.stages.push(f);
    }

    fn describe_all(&self) {
        for (i, f) in self.stages.iter().enumerate() {
            println!("  stage {}: {}", i, f.media_type());
        }
    }
}

// -------------------------------------------------------------------------
// 3. Object safety
// -------------------------------------------------------------------------

// This trait IS object-safe: all methods take &self or &mut self,
// no generic methods, no Self in return position (except behind Box).
trait Encoder {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
    fn encoding_name(&self) -> &'static str;
}

// This trait is NOT object-safe as written because of the generic method.
// Uncomment the commented lines to see the compiler error.
//
// trait NotObjectSafe {
//     fn process<T: fmt::Display>(&self, value: T) -> String;
//     // error[E0038]: the trait `NotObjectSafe` cannot be made into an object
//     // note: method `process` has generic type parameters
// }

// Fix: use trait objects or concrete types instead of generics
trait ObjectSafe {
    fn process_str(&self, value: &str) -> String;    // takes &str — object-safe
    fn process_num(&self, value: i64) -> String;     // takes i64 — object-safe
}

// A method can be excluded from the vtable with `where Self: Sized`.
// This lets non-object-safe methods coexist with object-safe ones.
trait MixedSafety {
    fn safe_method(&self) -> &str;

    // This generic method cannot go in a vtable.
    // Excluding it with `where Self: Sized` makes the trait object-safe,
    // but dyn MixedSafety cannot call this method.
    fn generic_method<T: fmt::Display>(&self, value: T) -> String
    where
        Self: Sized,
    {
        format!("{}", value)
    }
}

struct SafeImpl;

impl MixedSafety for SafeImpl {
    fn safe_method(&self) -> &str {
        "safe"
    }
}

// -------------------------------------------------------------------------
// 4. impl Trait vs dyn Trait: when to use which
// -------------------------------------------------------------------------

// Scenario: a router that selects a formatter at compile time (impl Trait)
// vs at runtime (dyn Trait).

// compile-time selection: known at call site, zero overhead
fn route_static<F: Formatter>(formatter: &F, routes: &[&str]) {
    for route in routes {
        let encoded = formatter.format(route);
        println!("  [static] {} -> {}", route, encoded);
    }
}

// runtime selection: formatter can change based on request headers
fn route_dynamic(formatter: &dyn Formatter, routes: &[&str]) {
    for route in routes {
        let encoded = formatter.format(route);
        println!("  [dynamic] {} -> {}", route, encoded);
    }
}

// -------------------------------------------------------------------------
// main
// -------------------------------------------------------------------------

fn main() {
    println!("=== 1. impl Trait (static dispatch) ===\n");

    let json = JsonFormatter;
    let csv = CsvFormatter { delimiter: '|' };

    println!("{}", encode_static(&json, "hello world"));
    println!("{}", encode_static(&csv, "hello,world"));

    let f = default_formatter();
    println!("{}", f.format("default formatter output"));

    println!("\n=== 2. dyn Trait (dynamic dispatch) ===\n");

    let json_dyn: &dyn Formatter = &json;
    println!("{}", encode_dynamic(json_dyn, "dynamic dispatch"));

    let f_boxed = pick_formatter("json");
    println!("{}", encode_dynamic(f_boxed.as_ref(), "from Box<dyn>"));

    let f_csv = pick_formatter("csv");
    println!("media type chosen at runtime: {}", f_csv.media_type());

    // Heterogeneous collection — only possible with dyn Trait
    let mut pipeline = FormatterPipeline::new();
    pipeline.add(Box::new(JsonFormatter));
    pipeline.add(Box::new(CsvFormatter { delimiter: ',' }));
    pipeline.add(Box::new(PlainFormatter));
    println!("\nPipeline stages:");
    pipeline.describe_all();

    println!("\n=== 3. Object safety ===\n");

    // MixedSafety used as trait object — safe_method works, generic_method excluded
    let obj: &dyn MixedSafety = &SafeImpl;
    println!("safe_method: {}", obj.safe_method());
    // obj.generic_method(42)  — would not compile: method not available on dyn

    // The underlying SafeImpl can still call it:
    let concrete = SafeImpl;
    println!("generic_method (on concrete): {}", concrete.generic_method(42));

    println!("\n=== 4. Static vs dynamic dispatch comparison ===\n");

    let routes = &["/api/users", "/api/orders"];

    println!("Static (compile-time formatter selection):");
    route_static(&json, routes);

    let dynamic_formatter = pick_formatter("csv");
    println!("\nDynamic (runtime formatter selection):");
    route_dynamic(dynamic_formatter.as_ref(), routes);

    println!("\n--- Memory layout ---");
    println!("sizeof JsonFormatter:       {} bytes", std::mem::size_of::<JsonFormatter>());
    println!("sizeof &JsonFormatter:      {} bytes", std::mem::size_of::<&JsonFormatter>());
    println!("sizeof &dyn Formatter:      {} bytes (fat pointer: data ptr + vtable ptr)",
        std::mem::size_of::<&dyn Formatter>());
    println!("sizeof Box<dyn Formatter>:  {} bytes (same fat pointer on heap)",
        std::mem::size_of::<Box<dyn Formatter>>());
}
