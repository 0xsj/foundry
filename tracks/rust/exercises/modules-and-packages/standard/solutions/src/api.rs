// url-shortener/src/api.rs
//
// The api module owns CLI argument parsing.
//
// Key decision: `parse_args` takes `&[String]` rather than calling
// `std::env::args()` internally. This makes it fully testable — callers
// decide what the "args" are. The binary's main() passes the real args;
// tests pass whatever they want.

/// A parsed CLI command.
///
/// Each variant captures exactly the arguments needed for that operation.
/// Using an enum here means the rest of the codebase handles each case
/// exhaustively — the compiler will warn if a new variant is added but
/// not handled.
#[derive(Debug)]
pub enum Command {
    Shorten { url: String },
    Resolve { key: String },
    Remove { key: String },
    Stats,
    Help,
}

/// Parse a slice of command-line arguments into a `Command`.
///
/// `args` should not include the program name (i.e., pass `args[1..]` from
/// `std::env::args()`). Returns `Err` with a human-readable message on failure.
///
/// # Examples
///
/// ```
/// use url_shortener::parse_args;
///
/// let args = vec!["shorten".to_string(), "https://example.com".to_string()];
/// let cmd = parse_args(&args).unwrap();
/// ```
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    match args.first().map(|s| s.as_str()) {
        Some("shorten") => {
            let url = args
                .get(1)
                .ok_or_else(|| "shorten requires a URL argument".to_string())?;
            Ok(Command::Shorten { url: url.clone() })
        }
        Some("resolve") => {
            let key = args
                .get(1)
                .ok_or_else(|| "resolve requires a key argument".to_string())?;
            Ok(Command::Resolve { key: key.clone() })
        }
        Some("remove") => {
            let key = args
                .get(1)
                .ok_or_else(|| "remove requires a key argument".to_string())?;
            Ok(Command::Remove { key: key.clone() })
        }
        Some("stats") => Ok(Command::Stats),
        Some("help") | None => Ok(Command::Help),
        Some(unknown) => Err(format!("unknown command: '{}'", unknown)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorten() {
        let args = vec!["shorten".to_string(), "https://example.com".to_string()];
        let cmd = parse_args(&args).unwrap();
        assert!(matches!(cmd, Command::Shorten { url } if url == "https://example.com"));
    }

    #[test]
    fn test_parse_resolve() {
        let args = vec!["resolve".to_string(), "abc123".to_string()];
        let cmd = parse_args(&args).unwrap();
        assert!(matches!(cmd, Command::Resolve { key } if key == "abc123"));
    }

    #[test]
    fn test_parse_remove() {
        let args = vec!["remove".to_string(), "abc123".to_string()];
        let cmd = parse_args(&args).unwrap();
        assert!(matches!(cmd, Command::Remove { key } if key == "abc123"));
    }

    #[test]
    fn test_parse_stats() {
        let args = vec!["stats".to_string()];
        assert!(matches!(parse_args(&args).unwrap(), Command::Stats));
    }

    #[test]
    fn test_parse_help_explicit() {
        let args = vec!["help".to_string()];
        assert!(matches!(parse_args(&args).unwrap(), Command::Help));
    }

    #[test]
    fn test_parse_empty_args_returns_help() {
        let args: Vec<String> = vec![];
        assert!(matches!(parse_args(&args).unwrap(), Command::Help));
    }

    #[test]
    fn test_parse_shorten_missing_url() {
        let args = vec!["shorten".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("URL"));
    }

    #[test]
    fn test_parse_unknown_command() {
        let args = vec!["upload".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("upload"));
    }
}
