// Data Transformation Pipeline — Reference Solution
//
// Run tests:  rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

// ---------- Types ----------

/// A transformer takes ownership of a record and either returns it (possibly
/// modified) wrapped in Some, or discards it by returning None.
///
/// Key concept: Box<dyn FnMut> allows:
///   1. Storing closures of different concrete types in the same Vec
///   2. Closures that mutate captured state (FnMut, not just Fn)
///
/// The 'static bound means the closure cannot capture borrowed references
/// with non-static lifetimes — it must own all its captured data.
pub type Transformer<T> = Box<dyn FnMut(T) -> Option<T>>;

/// A configurable pipeline of transformation steps.
pub struct Pipeline<T> {
    transformers: Vec<Transformer<T>>,
}

// ---------- Implementation ----------

impl<T> Pipeline<T> {
    /// Creates a new empty pipeline.
    pub fn new() -> Pipeline<T> {
        Pipeline {
            transformers: Vec::new(),
        }
    }

    /// Adds a transformation step. Builder pattern: consumes self, returns new Pipeline.
    ///
    /// Key concept: `impl FnMut(T) -> Option<T> + 'static` is a generic bound.
    /// The compiler generates a monomorphized version for each concrete closure type.
    /// `Box::new(step)` performs type erasure: different concrete closure types
    /// all become `Box<dyn FnMut(T) -> Option<T>>` in the Vec.
    ///
    /// The `mut self` parameter means we can push to self.transformers in place.
    pub fn add_step(mut self, step: impl FnMut(T) -> Option<T> + 'static) -> Pipeline<T> {
        self.transformers.push(Box::new(step));
        self
    }

    /// Runs all transformation steps against the given records.
    ///
    /// Key concept: We use a fold-like pattern. For each record, we thread it
    /// through all transformers. If any returns None, we short-circuit with None.
    ///
    /// `mut self` lets us call `&mut self.transformers[i]` because FnMut requires
    /// exclusive access.
    pub fn run(mut self, records: Vec<T>) -> Vec<T> {
        records
            .into_iter()
            .filter_map(|record| {
                // Thread the record through each transformer in sequence.
                // fold starts with Some(record) and threads it through each step.
                // If any step returns None, fold keeps None (short-circuits by design:
                // once acc is None, transformer is never called on None because we
                // use and_then).
                self.transformers.iter_mut().fold(
                    Some(record),
                    |acc, transformer| acc.and_then(|r| transformer(r)),
                )
            })
            .collect()
    }
}

// ---------- Transformer factory functions ----------

/// Returns a Transformer that keeps records where predicate returns true.
///
/// Key concept: The predicate borrows &T (read-only), but the transformer
/// takes ownership of T. We check the predicate on a borrow, then either
/// return the owned value or discard it.
pub fn keep_if<T: 'static>(predicate: impl Fn(&T) -> bool + 'static) -> Transformer<T> {
    Box::new(move |record| {
        if predicate(&record) {
            Some(record)
        } else {
            None
        }
    })
}

/// Returns a Transformer that applies transform to every record (always keeps it).
///
/// Key concept: Always returns Some — this step only transforms, never filters.
pub fn map_field<T: 'static>(transform: impl Fn(T) -> T + 'static) -> Transformer<T> {
    Box::new(move |record| Some(transform(record)))
}

/// Returns a Transformer that keeps only the first n records, discarding the rest.
///
/// Key concept: `seen` is mutable state captured by the closure. This makes
/// the closure FnMut (it mutates `seen` on every call). That's why the type
/// alias uses `FnMut` rather than `Fn`.
///
/// The `move` keyword transfers ownership of `seen` (a usize, which is Copy,
/// so it's actually copied — but `move` is still required to force the capture).
pub fn take_first<T: 'static>(n: usize) -> Transformer<T> {
    let mut seen = 0usize;
    Box::new(move |record| {
        if seen < n {
            seen += 1;
            Some(record)
        } else {
            None
        }
    })
}

// ---------- Standalone function ----------

/// Applies a Vec of closures (all the same concrete type) to records.
///
/// Key concept: Generic over a single closure type `F`. All steps in `steps`
/// must be the same concrete type. This avoids heap allocation (no Box) but
/// limits flexibility — you can't mix closures from different factory functions.
///
/// Compare to `Pipeline::run` which uses `Vec<Box<dyn FnMut>>` (trait objects)
/// to allow heterogeneous closure types.
pub fn apply_pipeline<T, F: FnMut(T) -> Option<T>>(records: Vec<T>, mut steps: Vec<F>) -> Vec<T> {
    records
        .into_iter()
        .filter_map(|record| {
            steps.iter_mut().fold(
                Some(record),
                |acc, step| acc.and_then(|r| step(r)),
            )
        })
        .collect()
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Pipeline::new and basic run --

    #[test]
    fn test_empty_pipeline_returns_all() {
        let records = vec!["alpha", "beta", "gamma"];
        let result = Pipeline::new().run(records);
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    // -- keep_if --

    #[test]
    fn test_keep_if_filters() {
        let records = vec![
            "user.login",
            "db.query",
            "user.logout",
            "db.error",
            "user.purchase",
        ];

        let result = Pipeline::new()
            .add_step(keep_if(|s: &&str| s.starts_with("user.")))
            .run(records);

        assert_eq!(result, vec!["user.login", "user.logout", "user.purchase"]);
    }

    #[test]
    fn test_keep_if_none_match() {
        let records = vec!["a", "b", "c"];
        let result = Pipeline::new()
            .add_step(keep_if(|s: &&str| s.len() > 10))
            .run(records);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keep_if_all_match() {
        let records = vec!["hello", "world"];
        let result = Pipeline::new()
            .add_step(keep_if(|s: &&str| s.len() > 0))
            .run(records);
        assert_eq!(result, vec!["hello", "world"]);
    }

    // -- map_field --

    #[test]
    fn test_map_field_transforms() {
        let records = vec![
            String::from("  alice  "),
            String::from("  bob"),
            String::from("carol  "),
        ];

        let result = Pipeline::new()
            .add_step(map_field(|s: String| s.trim().to_string()))
            .run(records);

        assert_eq!(result, vec!["alice", "bob", "carol"]);
    }

    #[test]
    fn test_map_field_uppercase() {
        let records = vec![
            String::from("hello"),
            String::from("world"),
        ];
        let result = Pipeline::new()
            .add_step(map_field(|s: String| s.to_uppercase()))
            .run(records);
        assert_eq!(result, vec!["HELLO", "WORLD"]);
    }

    // -- take_first --

    #[test]
    fn test_take_first_limits() {
        let records = vec![1, 2, 3, 4, 5, 6, 7];
        let result = Pipeline::new()
            .add_step(take_first(3))
            .run(records);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_take_first_zero() {
        let records = vec![1, 2, 3];
        let result = Pipeline::new()
            .add_step(take_first(0))
            .run(records);
        assert!(result.is_empty());
    }

    #[test]
    fn test_take_first_more_than_available() {
        let records = vec![1, 2];
        let result = Pipeline::new()
            .add_step(take_first(10))
            .run(records);
        assert_eq!(result, vec![1, 2]);
    }

    // -- Chained steps --

    #[test]
    fn test_chained_steps_order() {
        // Pipeline: filter to user events, then uppercase, then take first 2
        let records = vec![
            String::from("user.login"),
            String::from("db.query"),
            String::from("user.purchase"),
            String::from("user.logout"),
            String::from("db.error"),
        ];

        let result = Pipeline::new()
            .add_step(keep_if(|s: &String| s.starts_with("user.")))
            .add_step(map_field(|s: String| s.to_uppercase()))
            .add_step(take_first(2))
            .run(records);

        assert_eq!(result, vec!["USER.LOGIN", "USER.PURCHASE"]);
    }

    #[test]
    fn test_early_discard_skips_later_steps() {
        // If step 1 discards a record, step 2 should never see it.
        let records = vec![
            String::from("keep"),
            String::from("drop-me"),
            String::from("keep-too"),
        ];

        let result = Pipeline::new()
            .add_step(keep_if(|s: &String| !s.starts_with("drop")))
            .add_step(map_field(|s: String| s.to_uppercase()))
            .run(records);

        assert_eq!(result, vec!["KEEP", "KEEP-TOO"]);
    }

    // -- apply_pipeline --

    #[test]
    fn test_apply_pipeline_basic() {
        let records = vec![10i32, 20, 30, 40, 50];
        let steps: Vec<Box<dyn FnMut(i32) -> Option<i32>>> = vec![
            Box::new(|x| if x > 15 { Some(x) } else { None }),
            Box::new(|x| if x < 45 { Some(x) } else { None }),
        ];
        let result = apply_pipeline(records, steps);
        assert_eq!(result, vec![20, 30, 40]);
    }

    #[test]
    fn test_apply_pipeline_empty_steps() {
        let records = vec![1, 2, 3];
        let steps: Vec<Box<dyn FnMut(i32) -> Option<i32>>> = vec![];
        let result = apply_pipeline(records, steps);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_apply_pipeline_all_dropped() {
        let records = vec![1i32, 2, 3];
        let steps: Vec<Box<dyn FnMut(i32) -> Option<i32>>> = vec![
            Box::new(|_| None),
        ];
        let result = apply_pipeline(records, steps);
        assert!(result.is_empty());
    }
}

fn main() {
    let raw_records = vec![
        String::from("  USER.LOGIN  "),
        String::from("db.query"),
        String::from("  USER.PURCHASE"),
        String::from("cache.miss"),
        String::from("  USER.LOGOUT  "),
        String::from("db.error"),
        String::from("user.view"),
    ];

    let pipeline = Pipeline::new()
        .add_step(map_field(|s: String| s.trim().to_string()))
        .add_step(map_field(|s: String| s.to_lowercase()))
        .add_step(keep_if(|s: &String| s.starts_with("user.")))
        .add_step(take_first(3));

    let result = pipeline.run(raw_records);

    println!("Pipeline output ({} records):", result.len());
    for record in &result {
        println!("  {}", record);
    }
}
