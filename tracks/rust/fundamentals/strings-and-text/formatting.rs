// formatting.rs — format!, Display, Debug, write!, FromStr/parse
//
// Run: rustc formatting.rs && ./formatting

use std::fmt;
use std::str::FromStr;

fn main() {
    format_macro();
    println!();
    display_vs_debug();
    println!();
    format_specifiers();
    println!();
    write_into_string();
    println!();
    from_str_and_parse();
    println!();
    implementing_from_str();
}

// ---- format! ----------------------------------------------------------------

fn format_macro() {
    println!("=== format! ===");

    // format! always returns a new String — it never panics (unlike unwrap).
    let name = "Alice";
    let port = 8080;
    let s = format!("{} is listening on port {}", name, port);
    println!("{}", s);

    // Named captures (Rust 1.58+)
    let host = "localhost";
    let s = format!("{host}:{port}");
    println!("{}", s);

    // Positional — reuse arguments
    let s = format!("{0}, {0}! I call you {0}.", "echo");
    println!("{}", s);

    // format! vs string concatenation
    let a = String::from("hello");
    let b = String::from("world");

    // + moves a, borrows b — a is gone after this
    let via_plus = a + " " + &b;
    println!("via +: {}", via_plus);
    // println!("{}", a); // would not compile — a was moved

    // format! borrows everything — a and b are still alive (but we moved a above)
    let c = String::from("hello");
    let d = String::from("world");
    let via_format = format!("{} {}", c, d);
    println!("via format!: {}", via_format);
    println!("c and d are still alive: {} {}", c, d); // both still usable
}

// ---- Display vs Debug -------------------------------------------------------

fn display_vs_debug() {
    println!("=== Display vs Debug ===");

    // Debug is for developers. Derive it automatically.
    #[derive(Debug, Clone)]
    struct HttpRequest {
        method: String,
        path: String,
        status: u16,
    }

    // Display is for users. Implement it manually.
    impl fmt::Display for HttpRequest {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} {} → {}", self.method, self.path, self.status)
        }
    }

    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        status: 200,
    };

    // {} uses Display — human-readable
    println!("Display: {}", req);

    // {:?} uses Debug — developer-readable
    println!("Debug:   {:?}", req);

    // {:#?} uses Debug with pretty-printing
    println!("Pretty:\n{:#?}", req);

    // Display for enums — useful for log levels, status codes, etc.
    #[derive(Debug, PartialEq)]
    enum Severity { Info, Warn, Error, Critical }

    impl fmt::Display for Severity {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // write! in fmt returns fmt::Result — propagate it with ?
            match self {
                Severity::Info     => write!(f, "INFO"),
                Severity::Warn     => write!(f, "WARN"),
                Severity::Error    => write!(f, "ERROR"),
                Severity::Critical => write!(f, "CRITICAL"),
            }
        }
    }

    println!("Severity: {}", Severity::Critical);
    println!("Debug:    {:?}", Severity::Critical);

    // A struct with nested Display — using ? to propagate errors
    struct LogLine<'a> {
        severity: Severity,
        message: &'a str,
    }

    impl<'a> fmt::Display for LogLine<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // write! returns fmt::Result; use ? to propagate on error
            write!(f, "[{}] {}", self.severity, self.message)
        }
    }

    let line = LogLine { severity: Severity::Error, message: "connection refused" };
    println!("LogLine: {}", line);
}

// ---- Format Specifiers -------------------------------------------------------

fn format_specifiers() {
    println!("=== Format Specifiers ===");

    // Padding and alignment
    // <  = left-align
    // >  = right-align
    // ^  = center-align
    println!("{:<10}|", "left");      // "left      |"
    println!("{:>10}|", "right");     // "      right|"
    println!("{:^10}|", "center");    // "  center  |"

    // Fill character (before align specifier)
    println!("{:*^10}|", "center");   // "**center**|"
    println!("{:0>5}",   42);         // "00042"

    // Width from a variable (using $)
    let width = 8;
    println!("{:>width$}", "dynamic", width = width);

    // Floats
    println!("{:.2}",     3.14159);   // "3.14"
    println!("{:8.2}",    3.14159);   // "    3.14"
    println!("{:08.2}",   3.14159);   // "00003.14"
    println!("{:+.2}",    3.14159);   // "+3.14"
    println!("{:e}",      1234567.0); // "1.234567e6"

    // Integer bases
    println!("{:b}",  255);           // "11111111"
    println!("{:o}",  255);           // "377"
    println!("{:x}",  255);           // "ff"
    println!("{:X}",  255);           // "FF"
    println!("{:#x}", 255);           // "0xff" — alternate form adds prefix
    println!("{:#b}", 255);           // "0b11111111"

    // Debug vs display for collections (Display not implemented for Vec)
    let v = vec![1, 2, 3];
    println!("{:?}",  v);  // [1, 2, 3]
    println!("{:#?}", v);  // pretty multi-line

    // Pointer address
    let x = 42;
    println!("address of x: {:p}", &x);
}

// ---- write! Into a String Buffer ---------------------------------------------

fn write_into_string() {
    println!("=== write! into String ===");

    use std::fmt::Write; // brings write! for String into scope

    // build a report line by line — no per-line allocation
    let mut report = String::with_capacity(256);

    let metrics = [
        ("cpu_usage",    78.4_f64),
        ("memory_mb",    1024.0_f64),
        ("disk_read_mb", 32.1_f64),
    ];

    writeln!(report, "=== System Report ===").unwrap();
    for (name, value) in &metrics {
        // write! appends to the String buffer, no extra allocation
        writeln!(report, "  {:15} {:8.2}", name, value).unwrap();
    }
    writeln!(report, "Total metrics: {}", metrics.len()).unwrap();

    print!("{}", report);

    // Why not format! in the loop?
    // format! creates a new String each call. write! appends to an existing buffer.
    // For tight loops building large output, write! is more efficient.

    // write! vs writeln!
    // write!   — no trailing newline
    // writeln! — appends '\n' automatically (like println! vs print!)
}

// ---- parse() and FromStr ----------------------------------------------------

fn from_str_and_parse() {
    println!("=== parse() and FromStr ===");

    // parse() works for any type that implements FromStr
    let n: i32 = "42".parse().unwrap();
    let f: f64 = "3.14".parse().unwrap();
    let b: bool = "true".parse().unwrap();
    println!("parsed i32: {}, f64: {}, bool: {}", n, f, b);

    // When type cannot be inferred, use the turbofish
    let n = "100".parse::<u64>().unwrap();
    println!("turbofish: {}", n);

    // parse() returns Result — handle errors explicitly
    match "not_a_number".parse::<i32>() {
        Ok(n)  => println!("parsed: {}", n),
        Err(e) => println!("parse error: {}", e),
    }

    // Real-world pattern: parsing config values
    let raw_config = vec![
        ("port",    "8080"),
        ("workers", "4"),
        ("debug",   "true"),
        ("timeout", "30"),
    ];

    for (key, value) in &raw_config {
        match *key {
            "port"    => println!("port = {}", value.parse::<u16>().unwrap_or(3000)),
            "workers" => println!("workers = {}", value.parse::<usize>().unwrap_or(1)),
            "debug"   => println!("debug = {}", value.parse::<bool>().unwrap_or(false)),
            "timeout" => println!("timeout = {}s", value.parse::<u32>().unwrap_or(30)),
            other     => println!("unknown key: {}", other),
        }
    }
}

// ---- Implementing FromStr ---------------------------------------------------

fn implementing_from_str() {
    println!("=== Implementing FromStr ===");

    // A log level enum that can be parsed from a string
    #[derive(Debug, PartialEq)]
    enum LogLevel { Trace, Debug, Info, Warn, Error }

    impl fmt::Display for LogLevel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                LogLevel::Trace => "TRACE",
                LogLevel::Debug => "DEBUG",
                LogLevel::Info  => "INFO",
                LogLevel::Warn  => "WARN",
                LogLevel::Error => "ERROR",
            };
            f.write_str(s)
        }
    }

    impl FromStr for LogLevel {
        type Err = String; // or a custom error type in production

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_uppercase().as_str() {
                "TRACE" => Ok(LogLevel::Trace),
                "DEBUG" => Ok(LogLevel::Debug),
                "INFO"  => Ok(LogLevel::Info),
                "WARN"  => Ok(LogLevel::Warn),
                "ERROR" => Ok(LogLevel::Error),
                other   => Err(format!("unknown log level: {:?}", other)),
            }
        }
    }

    // Now .parse() works automatically
    let level: LogLevel = "warn".parse().unwrap();
    println!("parsed level: {:?}", level);
    println!("display:      {}", level);

    let levels = ["trace", "DEBUG", "Info", "WARN", "error", "UNKNOWN"];
    for raw in &levels {
        match raw.parse::<LogLevel>() {
            Ok(l)  => println!("  {:8} → {}", raw, l),
            Err(e) => println!("  {:8} → ERROR: {}", raw, e),
        }
    }

    // Round-trip: Display → parse → Display
    let original = LogLevel::Error;
    let as_string = original.to_string();                  // "ERROR"
    let parsed: LogLevel = as_string.parse().unwrap();     // back to LogLevel::Error
    assert_eq!(original, parsed);
    println!("round-trip works: {} → {:?} → {}", original, parsed, parsed);
}
