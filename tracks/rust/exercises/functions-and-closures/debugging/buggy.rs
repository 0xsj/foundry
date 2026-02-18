// Event Processor — Debugging Exercise (Rust)
//
// This code does not compile. There are 4 bugs — one per function.
// Read each compiler error carefully and fix the root cause.
//
// Run: rustc --test buggy.rs

// ---- Bug 1: FnMut passed where Fn is expected ----
//
// apply_to_all requires Fn (immutable closure, callable concurrently).
// The closure passed to it mutates captured state — it's FnMut, not Fn.
// Fix: either change what the closure captures, or change the bound on apply_to_all.

fn apply_to_all<F: Fn(&str) -> String>(items: &[&str], f: F) -> Vec<String> {
    items.iter().map(|&s| f(s)).collect()
}

pub fn deduplicate(events: &[&str]) -> Vec<String> {
    let mut last_seen = String::new();

    // BUG: this closure mutates last_seen — it's FnMut, but apply_to_all requires Fn
    let dedup = |event: &str| -> String {
        if event == last_seen {
            String::new()  // duplicate — return empty string as sentinel
        } else {
            last_seen = event.to_string();  // mutation! makes this FnMut
            event.to_string()
        }
    };

    apply_to_all(events, dedup)
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

// ---- Bug 2: move closure borrows after move ----
//
// The closure uses `move` to capture `prefix` (a &str, a reference).
// After the move, the closure is returned — but the reference `prefix` points to
// data that may not live long enough. The borrow checker catches this.
// Fix: capture an owned String instead of borrowing a &str.

pub fn make_prefixer(prefix: &str) -> impl Fn(&str) -> String {
    // BUG: `prefix` is a &str (a borrow). `move` copies the reference into the closure,
    // but the closure outlives this function — so the reference would dangle.
    move |event| format!("{}.{}", prefix, event)
}

// ---- Bug 3: returning a closure without proper return type ----
//
// This function tries to return a closure, but the return type is incorrect.
// The author wrote `fn() -> u32` (a function pointer type), but the closure
// captures `count` — it cannot be a function pointer (those carry no state).
// Fix: use the correct return type for a closure.

pub fn make_counter() -> fn() -> u32 {
    let mut count = 0u32;
    // BUG: the return type says `fn() -> u32` (a function pointer), but this
    // closure captures `count` — capturing closures are NOT function pointers.
    move || {
        count += 1;
        count
    }
}

// ---- Bug 4: closure captures reference to local variable ----
//
// `format_events` builds a formatted string from `events` and a `separator`,
// then tries to return a closure that produces parts of that string.
// The closure captures a reference to `formatted` which is a local variable —
// it will be dropped when the function returns.
// Fix: make the closure own the data it needs (move, own the formatted string).

pub fn format_events(events: &[&str], separator: &str) -> impl Fn(usize) -> &str {
    let formatted: Vec<String> = events
        .iter()
        .map(|&e| e.to_uppercase())
        .collect();

    // BUG: the closure captures a reference to `formatted` (a local Vec).
    // `formatted` is dropped at the end of this function, so the reference dangles.
    // Also: returning &str from a returned closure requires lifetime annotations
    // that can't be satisfied here without owning the data.
    move |index| formatted[index].as_str()
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_removes_consecutive() {
        let events = ["login", "login", "purchase", "purchase", "purchase", "logout"];
        let result = deduplicate(&events);
        assert_eq!(result, vec!["login", "purchase", "logout"]);
    }

    #[test]
    fn test_deduplicate_non_consecutive_kept() {
        let events = ["a", "b", "a"];  // "a" appears twice but not consecutively
        let result = deduplicate(&events);
        assert_eq!(result, vec!["a", "b", "a"]);
    }

    #[test]
    fn test_make_prefixer() {
        let prefixer = make_prefixer("svc");
        assert_eq!(prefixer("login"), "svc.login");
        assert_eq!(prefixer("logout"), "svc.logout");
    }

    #[test]
    fn test_make_counter_increments() {
        let mut counter = make_counter();
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
        assert_eq!(counter(), 3);
    }

    #[test]
    fn test_format_events_by_index() {
        let events = ["login", "purchase", "logout"];
        let get_event = format_events(&events, ", ");
        assert_eq!(get_event(0), "LOGIN");
        assert_eq!(get_event(2), "LOGOUT");
    }
}

fn main() {
    println!("Fix the compile errors, then run: rustc --test buggy.rs && ./buggy");
}
