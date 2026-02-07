// ============================================================================
// Variables and Types — Rust
// ============================================================================
// Fill in each section. Run with: rustc variables.rs && ./variables
// Or just save and ask for a review.
// ============================================================================

fn main() {
    // ========================================================================
    // 1. IMMUTABLE BY DEFAULT
    // Declare variables with `let`. These are immutable — you can't reassign.
    // ========================================================================
    // a) a string slice (&str)

    // b) an i32

    // c) an f64

    // d) a bool

    // ========================================================================
    // 2. MUTABLE VARIABLES
    // Use `let mut` to make a variable reassignable.
    // Declare a mutable variable and then change its value.
    // ========================================================================
    // a) declare a mutable i32 and reassign it

    // b) try reassigning an immutable variable — what error do you get?
    //    (comment it out after you see the error)

    // ========================================================================
    // 3. TYPE ANNOTATIONS vs INFERENCE
    // Rust infers types aggressively. Declare with and without annotations.
    // ========================================================================
    // a) let x: i32 = 42;

    // b) let y = 42; — what type does Rust infer?

    // c) let z: f32 = 3.14; — note: Rust defaults floats to f64, not f32

    // ========================================================================
    // 4. SHADOWING
    // Rust lets you re-declare a variable with the same name using `let`.
    // This is different from mutation — it creates a NEW variable.
    // ========================================================================
    // a) declare `let x = 5;` then `let x = x + 1;` — what is x?

    // b) declare `let s = "42";` then `let s: i32 = s.parse().unwrap();`
    //    shadowing let you CHANGE THE TYPE. Try this.

    // ========================================================================
    // 5. CONSTANTS
    // `const` in Rust requires a type annotation and must be compile-time known.
    // ========================================================================
    // a) declare a const (e.g. const MAX_RETRIES: i32 = 3;)

    // b) how is const different from let? (answer in a comment)

    // ========================================================================
    // 6. String vs &str
    // This is unique to Rust. A &str is a borrowed string slice (stack/static).
    // A String is an owned, heap-allocated string.
    // ========================================================================
    // a) let greeting: &str = "hello";

    // b) let owned: String = String::from("hello");

    // c) let also_owned: String = "hello".to_string();

    // d) when would you use each? (answer in a comment)

    // ========================================================================
    // 7. PRINT EVERYTHING
    // Use println! macro. For types, use {:?} for debug printing.
    // Example: println!("x = {}, type is i32", x);
    // ========================================================================

    println!("done");
}
