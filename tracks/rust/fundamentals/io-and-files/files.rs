// File Operations — I/O and Files (Rust)
//
// Demonstrates File::open, File::create, OpenOptions, Path/PathBuf,
// directory operations, metadata, and RAII resource cleanup.
//
// Run: rustc files.rs && ./files
// Note: this creates a temporary directory under /tmp — cleaned up on exit.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

// ---- 1. Path vs PathBuf ----
//
// Path  = borrowed, like &str. Used in function arguments.
// PathBuf = owned, like String. Used when building or storing paths.
//
// The rule: accept &Path (via impl AsRef<Path>), store PathBuf.

fn build_log_path(base_dir: impl AsRef<Path>, service: &str, date: &str) -> PathBuf {
    // join() handles OS-specific separators — never use string concatenation
    PathBuf::from(base_dir.as_ref())
        .join(service)
        .join(format!("{}.log", date))
}

fn demonstrate_paths() {
    let path = build_log_path("/var/log", "auth", "2026-02-18");
    println!("[path]   full path:  {}", path.display());
    println!("[path]   parent:     {}", path.parent().unwrap().display());
    println!("[path]   file_name:  {}", path.file_name().unwrap().to_string_lossy());
    println!("[path]   extension:  {}", path.extension().unwrap().to_string_lossy());
    println!("[path]   stem:       {}", path.file_stem().unwrap().to_string_lossy());
    println!("[path]   is_absolute:{}", path.is_absolute());

    // with_extension replaces the extension
    let gz_path = path.with_extension("log.gz");
    println!("[path]   gz variant: {}", gz_path.display());
}

// ---- 2. File::open / File::create / RAII ----
//
// Files close automatically when dropped — no defer, no try/finally.
// The scope block below is explicit (and optional here) but shows how
// to control exactly when a file closes.

fn write_and_read_file(path: &Path) -> io::Result<String> {
    // Write phase — BufWriter for efficient writes
    {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "service=auth")?;
        writeln!(writer, "latency_p99=42ms")?;
        writeln!(writer, "errors=0")?;
        writer.flush()?; // explicit flush before drop
    } // file and writer dropped here — file.close() called by RAII

    // Read phase — file is closed, safe to re-open
    fs::read_to_string(path)
}

// ---- 3. OpenOptions — fine-grained file control ----
//
// File::open   = OpenOptions::new().read(true)
// File::create = OpenOptions::new().write(true).create(true).truncate(true)
// For anything else, use OpenOptions explicitly.

fn append_audit_log(path: &Path, event: &str, user: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)   // create if missing
        .append(true)   // all writes go to end — position is always EOF
        .open(path)?;

    writeln!(file, "[{}] user={} event={}", "2026-02-18T10:30:00Z", user, event)?;
    Ok(())
}

// ---- 4. Reading files line by line ----
//
// Always use BufReader for line-by-line reading.
// File::read alone would read one byte per syscall — catastrophically slow.

fn count_events_by_type(log_path: &Path) -> io::Result<(usize, usize)> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file); // wrap in BufReader for efficient line iteration

    let mut errors = 0;
    let mut infos = 0;

    for line in reader.lines() {
        let line = line?;
        if line.contains("ERROR") {
            errors += 1;
        } else if line.contains("INFO") {
            infos += 1;
        }
    }

    Ok((errors, infos))
}

// ---- 5. Directory operations ----

fn setup_data_dirs(base: &Path) -> io::Result<()> {
    // create_dir_all is idempotent — does not fail if directories already exist
    fs::create_dir_all(base.join("cache"))?;
    fs::create_dir_all(base.join("logs"))?;
    fs::create_dir_all(base.join("uploads"))?;
    println!("[dirs]   created directory tree under {}", base.display());
    Ok(())
}

fn list_directory(dir: &Path) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = entry.metadata()?;
        let kind = if meta.is_dir() { "dir" } else { "file" };
        entries.push(format!("{} ({})", name, kind));
    }
    entries.sort(); // read_dir order is not guaranteed
    Ok(entries)
}

// ---- 6. File metadata ----

fn print_file_info(path: &Path) -> io::Result<()> {
    let meta = fs::metadata(path)?;
    println!("[meta]   path:     {}", path.display());
    println!("[meta]   size:     {} bytes", meta.len());
    println!("[meta]   is_file:  {}", meta.is_file());
    println!("[meta]   readonly: {}", meta.permissions().readonly());
    Ok(())
}

// ---- 7. Error handling — inspecting ErrorKind ----
//
// io::ErrorKind lets you branch on the cause without parsing error messages.
// This is the equivalent of checking errors.Is(err, fs.ErrNotFound) in Go.

fn load_config_with_fallback(path: &Path, fallback: &str) -> io::Result<String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Missing config is expected — use defaults
            println!("[config] {} not found, using fallback", path.display());
            Ok(fallback.to_string())
        }
        Err(e) => Err(e), // propagate permission errors, I/O failures, etc.
    }
}

fn main() -> io::Result<()> {
    demonstrate_paths();
    println!();

    // Use /tmp for a self-contained demo that doesn't litter the repo
    let base = PathBuf::from("/tmp/foundry-io-demo");
    fs::remove_dir_all(&base).ok(); // clean up any previous run

    setup_data_dirs(&base)?;
    println!();

    // Write and read a config file
    let config_path = base.join("cache").join("config.txt");
    let content = write_and_read_file(&config_path)?;
    println!("[file]   config contents:\n{}", content.trim());
    print_file_info(&config_path)?;
    println!();

    // Append several audit log entries
    let audit_path = base.join("logs").join("audit.log");
    append_audit_log(&audit_path, "login", "alice")?;
    append_audit_log(&audit_path, "config_change", "alice")?;
    append_audit_log(&audit_path, "logout", "alice")?;

    // Create a log file with mixed content for counting
    let app_log = base.join("logs").join("app.log");
    fs::write(&app_log,
        "2026-02-18 INFO  started\n\
         2026-02-18 ERROR disk full\n\
         2026-02-18 INFO  retrying\n\
         2026-02-18 ERROR write failed\n"
    )?;

    let (errors, infos) = count_events_by_type(&app_log)?;
    println!("[log]    {} ERROR lines, {} INFO lines in app.log", errors, infos);
    println!();

    // List the logs directory
    let entries = list_directory(&base.join("logs"))?;
    println!("[dir]    logs/:");
    for entry in &entries {
        println!("           {}", entry);
    }
    println!();

    // Config fallback demo
    let missing_path = base.join("missing.toml");
    let config = load_config_with_fallback(&missing_path, "[defaults]")?;
    println!("[config] loaded: {}", config);

    // Cleanup
    fs::remove_dir_all(&base)?;
    println!();
    println!("[done]   cleaned up {}", base.display());

    Ok(())
}
