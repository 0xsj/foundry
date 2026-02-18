// advanced_strings.rs — Cow<str>, OsString, Path/PathBuf, regex basics
//
// Run: rustc advanced_strings.rs && ./advanced_strings
//
// Note: the regex section uses only the standard library (no external crate).
// It demonstrates manual pattern matching as a baseline. The `regex` crate
// is covered in the lesson — add it to a Cargo project with:
//   cargo add regex

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main() {
    cow_str();
    println!();
    os_strings();
    println!();
    path_and_pathbuf();
    println!();
    manual_parsing();
    println!();
    string_building_patterns();
}

// ---- Cow<str> ---------------------------------------------------------------

fn cow_str() {
    println!("=== Cow<str> ===");

    // Cow<str> is either Borrowed(&str) or Owned(String).
    // Use it when the common case is "no modification needed" (borrow),
    // but some inputs require a modification (must own).

    // Example: sanitize an HTTP header value.
    // Most headers are valid — no allocation needed.
    // A small fraction contain newlines (header injection) — must be cleaned.
    fn sanitize_header(value: &str) -> Cow<str> {
        if value.contains('\n') || value.contains('\r') {
            // Rare path: allocate a new String with bad chars stripped
            Cow::Owned(value.replace(['\n', '\r'], ""))
        } else {
            // Common path: borrow the original — no allocation
            Cow::Borrowed(value)
        }
    }

    let clean   = sanitize_header("application/json");
    let dirty   = sanitize_header("text/html\nX-Injected: evil");

    // Cow<str> derefs to str, so you use it the same way regardless of variant
    println!("clean:  {:?}", &*clean);
    println!("dirty:  {:?}", &*dirty);

    // Demonstrate that clean is Borrowed (no heap activity)
    match &clean {
        Cow::Borrowed(_) => println!("clean is Borrowed — zero allocation"),
        Cow::Owned(_)    => println!("clean is Owned — allocated"),
    }
    match &dirty {
        Cow::Borrowed(_) => println!("dirty is Borrowed"),
        Cow::Owned(_)    => println!("dirty is Owned — sanitized copy allocated"),
    }

    // Another pattern: Cow in a struct field
    // When most instances use a static string but some need dynamic strings.
    struct EventTag<'a> {
        name: Cow<'a, str>,
    }

    impl<'a> EventTag<'a> {
        // Accept &'a str — zero allocation for the common case
        fn new(name: &'a str) -> Self {
            EventTag { name: Cow::Borrowed(name) }
        }

        // Accept String when the caller has constructed something dynamically
        fn new_owned(name: String) -> Self {
            EventTag { name: Cow::Owned(name) }
        }
    }

    let static_tag  = EventTag::new("user.login");
    let dynamic_tag = EventTag::new_owned(format!("user.{}", "signup"));
    println!("static tag:  {}", static_tag.name);
    println!("dynamic tag: {}", dynamic_tag.name);

    // Cow::into_owned: force an owned String, cloning if Borrowed
    let s: Cow<str> = Cow::Borrowed("hello");
    let owned: String = s.into_owned();  // clones — we needed ownership
    println!("into_owned: {:?}", owned);

    let s: Cow<str> = Cow::Owned(String::from("hello"));
    let owned: String = s.into_owned();  // unwraps — already owned, no clone
    println!("into_owned (was Owned): {:?}", owned);
}

// ---- OsString / OsStr -------------------------------------------------------

fn os_strings() {
    println!("=== OsString / OsStr ===");

    // OsStr / OsString exist for OS-native strings — not guaranteed UTF-8.
    // On Unix: arbitrary bytes. On Windows: WTF-8 (superset of UTF-8).
    // You encounter these when working with environment variables, command-line
    // arguments, and file system entries.

    // Creating OsStr from a &str — always succeeds (UTF-8 ⊆ OS strings)
    let flag: &OsStr = OsStr::new("--verbose");
    println!("OsStr:   {:?}", flag);

    // OsString (owned) from a &str
    let config_name: OsString = OsString::from("config.yaml");
    println!("OsString: {:?}", config_name);

    // Comparing OsStr values
    let arg = OsStr::new("--dry-run");
    if arg == OsStr::new("--dry-run") {
        println!("dry-run flag detected");
    }

    // The conversion that may fail: OsStr → &str
    // Fails if the bytes are not valid UTF-8 (rare on modern systems, but possible)
    let name = OsStr::new("report.csv");
    match name.to_str() {
        Some(s) => println!("valid UTF-8: {}", s),
        None    => println!("not valid UTF-8 — cannot convert to &str"),
    }

    // OsString → String (same potential failure)
    let os = OsString::from("metrics_2024.csv");
    if let Some(s) = os.to_str() {
        println!("as &str: {}", s);
        let owned: String = s.to_string(); // now you have a String
        println!("as String: {}", owned);
    }

    // Reading environment variables gives you OsString
    // (demonstrated without std::env to keep this file standalone)
    let path_val = OsString::from("/usr/local/bin:/usr/bin:/bin");
    let path_str = path_val.to_str().unwrap_or("<non-UTF-8 PATH>");
    println!("PATH: {}", path_str);

    // Comparing OsStr with &str directly does NOT work — different types
    // This is a common gotcha:
    let arg = OsStr::new("output.csv");
    // if arg == "output.csv" { }  // compile error: can't compare &OsStr with &str directly
    if arg == OsStr::new("output.csv") {   // must wrap in OsStr::new
        println!("filename matches");
    }
    // Alternative: convert OsStr to &str and then compare
    if arg.to_str() == Some("output.csv") {
        println!("filename matches (via to_str)");
    }
}

// ---- Path / PathBuf ---------------------------------------------------------

fn path_and_pathbuf() {
    println!("=== Path / PathBuf ===");

    // PathBuf is the owned, mutable path type (built on OsString).
    // Path is the borrowed slice type (built on OsStr).

    // Building paths
    let mut config_dir = PathBuf::from("/etc/myapp");
    config_dir.push("conf.d");         // /etc/myapp/conf.d
    config_dir.push("server.toml");    // /etc/myapp/conf.d/server.toml
    println!("config path: {:?}", config_dir);

    // join() is like push but returns a new PathBuf (non-mutating)
    let base = PathBuf::from("/var/log");
    let app_log = base.join("myapp").join("error.log");
    println!("log path: {:?}", app_log);

    // Path inspection methods
    let p = Path::new("/var/log/myapp/error.log");
    println!("file_name:  {:?}", p.file_name());    // Some("error.log")
    println!("file_stem:  {:?}", p.file_stem());    // Some("error")
    println!("extension:  {:?}", p.extension());    // Some("log")
    println!("parent:     {:?}", p.parent());       // Some("/var/log/myapp")
    println!("is_absolute:{:?}", p.is_absolute());  // true

    // Converting Path to &str (may fail for non-UTF-8 paths)
    match p.to_str() {
        Some(s) => println!("as &str: {}", s),
        None    => println!("non-UTF-8 path"),
    }

    // Always-safe display (even for non-UTF-8 paths)
    println!("display: {}", p.display());

    // Building paths from components
    let mut output = PathBuf::new();
    output.push("output");
    output.push("reports");
    output.push("2024");
    output.set_file_name("summary");
    output.set_extension("csv");
    println!("output path: {:?}", output);  // "output/reports/2024/summary.csv"

    // Changing the extension
    let template = PathBuf::from("/templates/email.html");
    let text_version = template.with_extension("txt");
    println!("text version: {:?}", text_version);  // "/templates/email.txt"

    // Iterating over components
    let p = Path::new("/usr/local/bin/rustc");
    print!("components: ");
    for component in p.components() {
        print!("{:?} ", component);
    }
    println!();

    // Joining is cross-platform — Path uses '/' on Unix, '\' on Windows
    // Always use Path::join instead of string concatenation for paths.
    let joined = Path::new("/var/data").join("uploads").join("avatar.png");
    println!("joined: {:?}", joined);
}

// ---- Manual Parsing (std-only alternative to regex) -------------------------

fn manual_parsing() {
    println!("=== Manual Log Parsing (no regex crate) ===");

    // In real projects you'd use the `regex` crate for this.
    // This shows what manual parsing looks like, which is also good to know.
    //
    // Log format: "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused"

    #[derive(Debug)]
    struct LogEntry<'a> {
        timestamp: &'a str,
        level: &'a str,
        service: &'a str,
        message: &'a str,
    }

    fn parse_log_line(line: &str) -> Option<LogEntry<'_>> {
        // Split on the first space after timestamp
        let (timestamp, rest) = line.split_once(' ')?;

        // rest should start with "[LEVEL]"
        let rest = rest.strip_prefix('[')?;
        let (level, rest) = rest.split_once(']')?;
        let rest = rest.strip_prefix(' ')?;

        // rest should be "service: message"
        let (service, message) = rest.split_once(':')?;
        let message = message.trim_start();

        Some(LogEntry { timestamp, level, service, message })
    }

    let lines = [
        "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
        "2024-01-15T10:30:01Z [INFO] api-gateway: request accepted",
        "2024-01-15T10:30:02Z [WARN] db-pool: pool exhausted, waiting",
    ];

    for line in &lines {
        match parse_log_line(line) {
            Some(entry) => println!(
                "  ts={} level={} svc={} msg={}",
                entry.timestamp, entry.level, entry.service, entry.message
            ),
            None => println!("  PARSE FAIL: {:?}", line),
        }
    }

    // Key str methods used here:
    // split_once(pat) — splits at the first occurrence, returns Option<(&str, &str)>
    // strip_prefix(pat) — removes prefix, returns Option<&str>
    // trim_start() — strips leading whitespace, returns &str
    // All return &str slices into the original — no allocation.
}

// ---- String Building Patterns -----------------------------------------------

fn string_building_patterns() {
    println!("=== String Building Patterns ===");

    // Pattern 1: Joining with separators — use .join()
    let fields = ["user_id", "email", "created_at", "status"];
    let header = fields.join(",");
    println!("CSV header: {}", header);

    // Pattern 2: Building incrementally — use write! into a String
    let rows: Vec<(u64, &str, &str)> = vec![
        (1, "alice@example.com", "active"),
        (2, "bob@example.com",   "inactive"),
        (3, "carol@example.com", "active"),
    ];

    let mut csv = String::with_capacity(256);
    writeln!(csv, "id,email,status").unwrap();
    for (id, email, status) in &rows {
        writeln!(csv, "{},{},{}", id, email, status).unwrap();
    }
    print!("{}", csv);

    // Pattern 3: Repeated string — use .repeat()
    let separator = "-".repeat(40);
    println!("{}", separator);

    // Pattern 4: Building from an iterator — use .collect::<String>()
    let words = vec!["the", "quick", "brown", "fox"];
    let sentence: String = words.join(" ");
    let title_case: String = sentence
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("title case: {}", title_case);

    // Pattern 5: Checking and extracting with split_once
    let query_string = "format=json&pretty=true&limit=100";
    for pair in query_string.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            println!("  {} = {}", key, value);
        }
    }

    // Pattern 6: Replacing characters using a closure
    let filename = "user report (Q4 2024).xlsx";
    let safe_filename: String = filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect();
    println!("safe filename: {}", safe_filename);
}
