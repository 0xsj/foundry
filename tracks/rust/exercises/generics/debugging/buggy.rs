// Generic Data Pipeline — Debugging Exercise (Rust)
//
// This code does not compile. There are 4 bugs — one per section.
// Read each compiler error carefully and fix the root cause.
//
// Run: rustc --test buggy.rs

// ---- Bug 1: Missing trait bound — operator requires a trait the type doesn't have ----
//
// largest_by_score takes a slice of items and a scoring function.
// It needs to compare scores to find the maximum. The score type S is generic,
// but there's no bound saying S can be compared with >.
// Fix: add the bound that makes > work on S.

pub fn largest_by_score<T, S>(items: &[T], score_fn: impl Fn(&T) -> S) -> Option<&T> {
    if items.is_empty() {
        return None;
    }
    let mut best = &items[0];
    let mut best_score = score_fn(best);

    for item in &items[1..] {
        let s = score_fn(item);
        // BUG: `>` is not defined for generic S — a trait bound is missing
        if s > best_score {
            best = item;
            best_score = s;
        }
    }
    Some(best)
}

// ---- Bug 2: Wrong type parameter instantiation — turbofish supplies the wrong T ----
//
// summarize_scores computes statistics for a list of score strings.
// The author intended to parse the strings as f64 to compute the average,
// but accidentally used turbofish with the wrong type: parse::<i32>().
// This makes the computation silently truncate decimals.
// Additionally, the collect() call at the end accumulates results into a Vec
// but the type annotation on `scores` is wrong — it says Vec<i32> but the
// return type requires Vec<f64>. The compiler will report a type mismatch.
// Fix: change parse::<i32>() to parse::<f64>() and fix the Vec annotation.

use std::collections::HashMap;

pub fn parse_as_floats(inputs: &[&str]) -> Vec<f64> {
    // BUG: turbofish specifies i32, but the return type is Vec<f64>.
    // The compiler will report a type mismatch — Vec<i32> is not Vec<f64>.
    // Fix: change parse::<i32>() to parse::<f64>().
    let scores: Vec<i32> = inputs
        .iter()
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect();
    scores  // ERROR: mismatched types — Vec<i32> vs Vec<f64>
}

pub fn parse_config_values<T>(raw: &HashMap<&str, &str>, keys: &[&str]) -> Vec<T>
where
    T: std::str::FromStr + Default,
    T::Err: std::fmt::Debug,
{
    keys.iter()
        .map(|key| match raw.get(key) {
            Some(val) => val.parse().unwrap_or_default(),
            None => T::default(),
        })
        .collect()
}

// ---- Bug 3: Generic method makes trait non-object-safe ----
//
// The Processor trait has a process_batch method that is generic over T.
// This prevents using Box<dyn Processor> because the vtable cannot contain
// a generic method — the vtable entry would need to be different for every T,
// but vtables are fixed at compile time.
// Fix: remove the type parameter from process_batch and change it to accept &[&str].

pub trait Processor {
    fn name(&self) -> &str;

    // BUG: generic method — makes Processor non-object-safe (non-dyn-compatible).
    // A vtable cannot contain entries for every possible T.
    fn process_batch<T: std::fmt::Display>(&self, items: &[T]) -> Vec<String>;
}

pub struct UpperCaseProcessor;
pub struct LowerCaseProcessor;

impl Processor for UpperCaseProcessor {
    fn name(&self) -> &str { "uppercase" }
    fn process_batch<T: std::fmt::Display>(&self, items: &[T]) -> Vec<String> {
        items.iter().map(|x| format!("{}", x).to_uppercase()).collect()
    }
}

impl Processor for LowerCaseProcessor {
    fn name(&self) -> &str { "lowercase" }
    fn process_batch<T: std::fmt::Display>(&self, items: &[T]) -> Vec<String> {
        items.iter().map(|x| format!("{}", x).to_lowercase()).collect()
    }
}

// This function requires Processor to be object-safe (dyn-compatible).
// It stores processors in a Vec<Box<dyn Processor>> to support different implementations.
pub fn run_all_processors(processors: &[Box<dyn Processor>], items: &[&str]) -> Vec<Vec<String>> {
    // BUG: can't store Box<dyn Processor> because Processor is not object-safe.
    // The generic method on the trait is the cause.
    processors
        .iter()
        .map(|p| p.process_batch(items))
        .collect()
}

// ---- Bug 4: Returning a reference to a local variable through a generic ----
//
// make_default is supposed to return a reference to a T::default() value.
// The problem: T::default() creates a value on the stack inside make_default.
// The function tries to return a reference to that local — which is dropped
// when make_default returns.
// Fix: return T (owned value) instead of &T, or use a different approach.

pub fn make_default<T: Default>() -> &'static T {
    // BUG: T::default() creates a local value. Returning &local is always invalid.
    // The lifetime 'static promises the reference lives forever — but the local
    // variable is dropped at the end of this function. The compiler rejects this.
    // Fix: return T, not &T. If you genuinely need a static reference, you'd need
    // a global or Box::leak — but for a default value, just return T.
    let default = T::default();
    &default  // ERROR: cannot return reference to local variable
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_largest_by_score_word_length() {
        let words = vec!["short", "a-much-longer-word", "mid"];
        let result = largest_by_score(&words, |w| w.len());
        assert_eq!(result, Some(&"a-much-longer-word"));
    }

    #[test]
    fn test_largest_by_score_numeric() {
        let scores: Vec<i32> = vec![3, 9, 1, 7, 2];
        let result = largest_by_score(&scores, |&x| x);
        assert_eq!(result, Some(&9i32));
    }

    #[test]
    fn test_largest_by_score_empty() {
        let items: Vec<i32> = vec![];
        assert_eq!(largest_by_score(&items, |&x| x), None);
    }

    #[test]
    fn test_parse_as_floats() {
        let result = parse_as_floats(&["1.5", "2.7", "3.0"]);
        assert_eq!(result, vec![1.5f64, 2.7, 3.0]);
    }

    #[test]
    fn test_run_all_processors() {
        let processors: Vec<Box<dyn Processor>> = vec![
            Box::new(UpperCaseProcessor),
            Box::new(LowerCaseProcessor),
        ];
        let results = run_all_processors(&processors, &["Hello", "World"]);
        assert_eq!(results[0], vec!["HELLO", "WORLD"]);
        assert_eq!(results[1], vec!["hello", "world"]);
    }

    #[test]
    fn test_make_default_integer() {
        let v: i32 = make_default();
        assert_eq!(v, 0i32);
    }

    #[test]
    fn test_make_default_bool() {
        let v: bool = make_default();
        assert_eq!(v, false);
    }
}

fn main() {
    println!("Fix the compile errors, then run: rustc --test buggy.rs && ./buggy");
}
