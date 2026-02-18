// Command-Line Task Processor — Reference Solution
//
// Run tests:  rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

// ---------- Types ----------

#[derive(Debug, Clone, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Parses a case-insensitive string into a Priority.
    ///
    /// Key concept: match on a borrowed &str slice from to_lowercase().
    /// `to_lowercase()` returns an owned String, `.as_str()` borrows it
    /// temporarily so we can match against string literals.
    fn from_str(s: &str) -> Option<Priority> {
        match s.to_lowercase().as_str() {
            "low" => Some(Priority::Low),
            "medium" => Some(Priority::Medium),
            "high" => Some(Priority::High),
            "critical" => Some(Priority::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    title: String,
    done: bool,
    priority: Priority,
    tags: Vec<String>,
}

#[derive(Debug)]
enum Command {
    Add { title: String, priority: Priority },
    Done(u32),
    Delete(u32),
    List,
    Priority { id: u32, level: Priority },
    Tag { id: u32, tag: String },
    Filter(Priority),
    Stats,
    Unknown(String),
}

// ---------- Parsing ----------

/// Parses a text line into a Command.
///
/// Key concepts:
/// - match on a string slice (the first word/token)
/// - if let chains to safely extract optional values without panicking
/// - .parse::<u32>().ok() converts parse errors to None (never panics)
/// - Unknown variant captures anything that doesn't parse cleanly
fn parse_command(line: &str) -> Command {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Command::Unknown(line.to_string());
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let verb = parts.next().unwrap_or(""); // splitn always returns at least one item
    let rest = parts.next().unwrap_or("").trim();

    match verb {
        "add" => parse_add(rest, line),
        "done" => {
            // .ok() turns parse errors into None; the outer match becomes Unknown
            if let Some(id) = rest.parse::<u32>().ok() {
                Command::Done(id)
            } else {
                Command::Unknown(line.to_string())
            }
        }
        "delete" => {
            if let Some(id) = rest.parse::<u32>().ok() {
                Command::Delete(id)
            } else {
                Command::Unknown(line.to_string())
            }
        }
        "list" => Command::List,
        "priority" => parse_priority_cmd(rest, line),
        "tag" => parse_tag_cmd(rest, line),
        "filter" => {
            if let Some(p) = Priority::from_str(rest) {
                Command::Filter(p)
            } else {
                Command::Unknown(line.to_string())
            }
        }
        "stats" => Command::Stats,
        _ => Command::Unknown(line.to_string()),
    }
}

fn parse_add(rest: &str, raw_line: &str) -> Command {
    if rest.is_empty() {
        return Command::Unknown(raw_line.to_string());
    }

    let tokens: Vec<&str> = rest.split_whitespace().collect();

    // Check if the last token is a priority specifier "priority:<level>"
    // @ binding: bind the matched token while also checking the condition.
    let (title_tokens, priority) = if let Some(&last) = tokens.last() {
        if let Some(level_str) = last.strip_prefix("priority:") {
            if let Some(p) = Priority::from_str(level_str) {
                // Priority token found and valid — exclude it from title
                (&tokens[..tokens.len() - 1], p)
            } else {
                // "priority:" prefix found but level is invalid — treat whole thing as title
                (tokens.as_slice(), Priority::Medium)
            }
        } else {
            (tokens.as_slice(), Priority::Medium)
        }
    } else {
        return Command::Unknown(raw_line.to_string());
    };

    if title_tokens.is_empty() {
        return Command::Unknown(raw_line.to_string());
    }

    Command::Add {
        title: title_tokens.join(" "),
        priority,
    }
}

fn parse_priority_cmd(rest: &str, raw_line: &str) -> Command {
    // "priority <id> <level>" — two arguments
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id_str = parts.next().unwrap_or("");
    let level_str = parts.next().unwrap_or("").trim();

    // if let chain: both must succeed, otherwise Unknown
    if let (Some(id), Some(level)) = (id_str.parse::<u32>().ok(), Priority::from_str(level_str)) {
        Command::Priority { id, level }
    } else {
        Command::Unknown(raw_line.to_string())
    }
}

fn parse_tag_cmd(rest: &str, raw_line: &str) -> Command {
    // "tag <id> <tag>" — two arguments
    let mut parts = rest.splitn(2, char::is_whitespace);
    let id_str = parts.next().unwrap_or("");
    let tag = parts.next().unwrap_or("").trim().to_string();

    if let Some(id) = id_str.parse::<u32>().ok() {
        if !tag.is_empty() {
            Command::Tag { id, tag }
        } else {
            Command::Unknown(raw_line.to_string())
        }
    } else {
        Command::Unknown(raw_line.to_string())
    }
}

// ---------- Execution ----------

/// Executes a command against the task list.
///
/// Key concepts:
/// - match on Command variants with destructuring
/// - iter().position() to find tasks by id without cloning
/// - Pattern guards inside match for conditional logic
/// - @ binding to name a matched value (tag dedup case)
fn execute(command: Command, tasks: &mut Vec<Task>) -> String {
    match command {
        Command::Add { title, priority } => {
            let id = tasks.len() as u32 + 1;
            tasks.push(Task {
                id,
                title: title.clone(),
                done: false,
                priority,
                tags: Vec::new(),
            });
            format!("added task #{}: {}", id, title)
        }

        Command::Done(id) => {
            match tasks.iter().position(|t| t.id == id) {
                Some(idx) => {
                    tasks[idx].done = true;
                    format!("task #{} marked done", id)
                }
                None => format!("task #{} not found", id),
            }
        }

        Command::Delete(id) => {
            match tasks.iter().position(|t| t.id == id) {
                Some(idx) => {
                    tasks.remove(idx);
                    format!("deleted task #{}", id)
                }
                None => format!("task #{} not found", id),
            }
        }

        Command::List => {
            if tasks.is_empty() {
                return "no tasks".to_string();
            }
            tasks.iter().map(format_task).collect::<Vec<_>>().join("\n")
        }

        Command::Priority { id, level } => {
            match tasks.iter().position(|t| t.id == id) {
                Some(idx) => {
                    tasks[idx].priority = level.clone();
                    format!("task #{} priority set to {:?}", id, level)
                }
                None => format!("task #{} not found", id),
            }
        }

        Command::Tag { id, tag } => {
            match tasks.iter().position(|t| t.id == id) {
                Some(idx) => {
                    // Skip duplicate tags — if let on the contains check
                    if !tasks[idx].tags.contains(&tag) {
                        tasks[idx].tags.push(tag.clone());
                    }
                    format!("tagged task #{} with {}", id, tag)
                }
                None => format!("task #{} not found", id),
            }
        }

        Command::Filter(level) => {
            let matching: Vec<String> = tasks.iter()
                .filter(|t| t.priority == level)
                .map(format_task)
                .collect();

            if matching.is_empty() {
                format!("no tasks with priority {:?}", level)
            } else {
                matching.join("\n")
            }
        }

        Command::Stats => {
            let total = tasks.len() as u32;
            let mut done = 0u32;
            let mut critical = 0u32;

            for task in tasks.iter() {
                // match on multiple fields simultaneously
                match (&task.done, &task.priority) {
                    (true, _) => done += 1,
                    _ => {}
                }
                if task.priority == Priority::Critical {
                    critical += 1;
                }
            }

            format!(
                "total: {}, done: {}, pending: {}, critical: {}",
                total, done, total - done, critical
            )
        }

        Command::Unknown(line) => {
            format!("unknown command: {}", line)
        }
    }
}

fn format_task(task: &Task) -> String {
    let done_marker = if task.done { "x" } else { " " };
    let priority_label = format!("{:?}", task.priority).to_uppercase();
    let tag_str = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" (tags: {})", task.tags.join(", "))
    };
    format!("[{}] #{} [{}] {}{}", done_marker, task.id, priority_label, task.title, tag_str)
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Priority parsing --

    #[test]
    fn test_priority_from_str_valid() {
        assert_eq!(Priority::from_str("low"), Some(Priority::Low));
        assert_eq!(Priority::from_str("MEDIUM"), Some(Priority::Medium));
        assert_eq!(Priority::from_str("High"), Some(Priority::High));
        assert_eq!(Priority::from_str("critical"), Some(Priority::Critical));
    }

    #[test]
    fn test_priority_from_str_invalid() {
        assert_eq!(Priority::from_str("urgent"), None);
        assert_eq!(Priority::from_str(""), None);
        assert_eq!(Priority::from_str("42"), None);
    }

    // -- Command parsing: add --

    #[test]
    fn test_parse_add_basic() {
        match parse_command("add Fix login bug") {
            Command::Add { title, priority } => {
                assert_eq!(title, "Fix login bug");
                assert_eq!(priority, Priority::Medium);
            }
            cmd => panic!("expected Add, got {:?}", cmd),
        }
    }

    #[test]
    fn test_parse_add_with_priority() {
        match parse_command("add Fix login bug priority:high") {
            Command::Add { title, priority } => {
                assert_eq!(title, "Fix login bug");
                assert_eq!(priority, Priority::High);
            }
            cmd => panic!("expected Add, got {:?}", cmd),
        }
    }

    #[test]
    fn test_parse_add_critical() {
        match parse_command("add Patch CVE-2024-1234 priority:critical") {
            Command::Add { title, priority } => {
                assert_eq!(title, "Patch CVE-2024-1234");
                assert_eq!(priority, Priority::Critical);
            }
            cmd => panic!("expected Add, got {:?}", cmd),
        }
    }

    // -- Command parsing: other commands --

    #[test]
    fn test_parse_done() {
        assert!(matches!(parse_command("done 3"), Command::Done(3)));
    }

    #[test]
    fn test_parse_delete() {
        assert!(matches!(parse_command("delete 7"), Command::Delete(7)));
    }

    #[test]
    fn test_parse_list() {
        assert!(matches!(parse_command("list"), Command::List));
    }

    #[test]
    fn test_parse_priority() {
        match parse_command("priority 2 critical") {
            Command::Priority { id: 2, level: Priority::Critical } => {}
            cmd => panic!("expected Priority, got {:?}", cmd),
        }
    }

    #[test]
    fn test_parse_tag() {
        match parse_command("tag 1 security") {
            Command::Tag { id: 1, tag } => assert_eq!(tag, "security"),
            cmd => panic!("expected Tag, got {:?}", cmd),
        }
    }

    #[test]
    fn test_parse_filter() {
        assert!(matches!(parse_command("filter high"), Command::Filter(Priority::High)));
    }

    #[test]
    fn test_parse_stats() {
        assert!(matches!(parse_command("stats"), Command::Stats));
    }

    #[test]
    fn test_parse_unknown() {
        assert!(matches!(parse_command("deploy all the things"), Command::Unknown(_)));
        assert!(matches!(parse_command("done abc"), Command::Unknown(_)));
        assert!(matches!(parse_command(""), Command::Unknown(_)));
    }

    // -- Execute: add --

    #[test]
    fn test_execute_add() {
        let mut tasks = Vec::new();
        let result = execute(Command::Add {
            title: "Write tests".to_string(),
            priority: Priority::High,
        }, &mut tasks);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Write tests");
        assert_eq!(tasks[0].priority, Priority::High);
        assert_eq!(tasks[0].id, 1);
        assert!(!tasks[0].done);
        assert!(result.contains("1"));
        assert!(result.contains("Write tests"));
    }

    #[test]
    fn test_execute_add_sequential_ids() {
        let mut tasks = Vec::new();
        execute(Command::Add { title: "Task A".to_string(), priority: Priority::Low }, &mut tasks);
        execute(Command::Add { title: "Task B".to_string(), priority: Priority::Low }, &mut tasks);
        execute(Command::Add { title: "Task C".to_string(), priority: Priority::Low }, &mut tasks);

        assert_eq!(tasks[0].id, 1);
        assert_eq!(tasks[1].id, 2);
        assert_eq!(tasks[2].id, 3);
    }

    // -- Execute: done --

    #[test]
    fn test_execute_done() {
        let mut tasks = vec![Task {
            id: 1, title: "Deploy".to_string(), done: false,
            priority: Priority::High, tags: vec![],
        }];
        let result = execute(Command::Done(1), &mut tasks);
        assert!(tasks[0].done);
        assert!(result.contains("1"));
    }

    #[test]
    fn test_execute_done_not_found() {
        let mut tasks = Vec::new();
        let result = execute(Command::Done(42), &mut tasks);
        assert!(result.contains("not found") || result.contains("42"));
    }

    // -- Execute: delete --

    #[test]
    fn test_execute_delete() {
        let mut tasks = vec![
            Task { id: 1, title: "A".to_string(), done: false, priority: Priority::Low, tags: vec![] },
            Task { id: 2, title: "B".to_string(), done: false, priority: Priority::Low, tags: vec![] },
        ];
        let result = execute(Command::Delete(1), &mut tasks);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, 2);
        assert!(result.contains("1"));
    }

    // -- Execute: priority --

    #[test]
    fn test_execute_set_priority() {
        let mut tasks = vec![Task {
            id: 1, title: "Refactor".to_string(), done: false,
            priority: Priority::Low, tags: vec![],
        }];
        execute(Command::Priority { id: 1, level: Priority::Critical }, &mut tasks);
        assert_eq!(tasks[0].priority, Priority::Critical);
    }

    // -- Execute: tag --

    #[test]
    fn test_execute_tag() {
        let mut tasks = vec![Task {
            id: 1, title: "Fix auth".to_string(), done: false,
            priority: Priority::High, tags: vec![],
        }];
        execute(Command::Tag { id: 1, tag: "security".to_string() }, &mut tasks);
        assert!(tasks[0].tags.contains(&"security".to_string()));
    }

    #[test]
    fn test_execute_tag_no_duplicates() {
        let mut tasks = vec![Task {
            id: 1, title: "Fix auth".to_string(), done: false,
            priority: Priority::High, tags: vec!["security".to_string()],
        }];
        execute(Command::Tag { id: 1, tag: "security".to_string() }, &mut tasks);
        assert_eq!(tasks[0].tags.len(), 1);
    }

    // -- Execute: list --

    #[test]
    fn test_execute_list() {
        let mut tasks = vec![
            Task { id: 1, title: "Deploy".to_string(), done: true,
                   priority: Priority::High, tags: vec!["prod".to_string()] },
            Task { id: 2, title: "Write docs".to_string(), done: false,
                   priority: Priority::Low, tags: vec![] },
        ];
        let result = execute(Command::List, &mut tasks);
        assert!(result.contains("[x]"), "done task should show [x]");
        assert!(result.contains("[ ]"), "pending task should show [ ]");
        assert!(result.contains("Deploy"));
        assert!(result.contains("Write docs"));
        assert!(result.contains("prod"), "tags should appear");
    }

    // -- Execute: filter --

    #[test]
    fn test_execute_filter() {
        let mut tasks = vec![
            Task { id: 1, title: "A".to_string(), done: false, priority: Priority::High, tags: vec![] },
            Task { id: 2, title: "B".to_string(), done: false, priority: Priority::Low, tags: vec![] },
            Task { id: 3, title: "C".to_string(), done: true, priority: Priority::High, tags: vec![] },
        ];
        let result = execute(Command::Filter(Priority::High), &mut tasks);
        assert!(result.contains("A"));
        assert!(result.contains("C"));
        assert!(!result.contains("B"), "low priority task should be excluded");
    }

    // -- Execute: stats --

    #[test]
    fn test_execute_stats() {
        let mut tasks = vec![
            Task { id: 1, title: "A".to_string(), done: true, priority: Priority::High, tags: vec![] },
            Task { id: 2, title: "B".to_string(), done: false, priority: Priority::Critical, tags: vec![] },
            Task { id: 3, title: "C".to_string(), done: false, priority: Priority::Low, tags: vec![] },
        ];
        let result = execute(Command::Stats, &mut tasks);
        assert!(result.contains("total: 3"), "result was: {}", result);
        assert!(result.contains("done: 1"), "result was: {}", result);
        assert!(result.contains("pending: 2"), "result was: {}", result);
        assert!(result.contains("critical: 1"), "result was: {}", result);
    }

    // -- Execute: unknown --

    #[test]
    fn test_execute_unknown() {
        let mut tasks = Vec::new();
        let result = execute(Command::Unknown("foo bar".to_string()), &mut tasks);
        assert!(result.contains("unknown") || result.contains("foo bar"));
    }

    // -- Round-trip integration --

    #[test]
    fn test_full_workflow() {
        let mut tasks = Vec::new();

        let script = vec![
            "add Deploy to production priority:critical",
            "add Update changelog priority:low",
            "add Run smoke tests priority:high",
            "done 2",
            "tag 1 deploy",
            "tag 1 prod",
            "priority 3 medium",
        ];

        for line in script {
            let cmd = parse_command(line);
            execute(cmd, &mut tasks);
        }

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].priority, Priority::Critical);
        assert!(tasks[1].done);
        assert_eq!(tasks[0].tags, vec!["deploy", "prod"]);
        assert_eq!(tasks[2].priority, Priority::Medium);
    }
}

fn main() {
    let mut tasks = Vec::new();

    let commands = vec![
        "add Deploy API to staging priority:high",
        "add Write release notes",
        "add Fix memory leak priority:critical",
        "add Update dependencies priority:low",
        "tag 3 performance",
        "tag 3 security",
        "done 2",
        "priority 4 medium",
        "list",
        "filter high",
        "stats",
        "done 99",
    ];

    for line in commands {
        println!("> {}", line);
        let cmd = parse_command(line);
        let result = execute(cmd, &mut tasks);
        println!("{}\n", result);
    }
}
