// string_types.rs — String vs &str, conversions, slicing, chars/bytes
//
// Run: rustc string_types.rs && ./string_types

fn main() {
    owned_vs_borrowed();
    println!();
    memory_layout();
    println!();
    why_no_indexing();
    println!();
    chars_vs_bytes();
    println!();
    slicing();
    println!();
    conversions();
    println!();
    function_signatures();
}

// ---- Owned vs Borrowed -------------------------------------------------------

fn owned_vs_borrowed() {
    println!("=== Owned vs Borrowed ===");

    // String: heap-allocated, owns its bytes, has a length and capacity.
    let owned: String = String::from("hello");
    println!("owned:    {:?}  len={} cap={}", owned, owned.len(), owned.capacity());

    // &str: a fat pointer (ptr + len) into some existing UTF-8 memory.
    // Here the data lives in the program binary (static storage).
    let literal: &str = "hello";
    println!("literal:  {:?}  len={}", literal, literal.len());

    // Borrowing from a String produces a &str.
    // The &str points into the String's heap buffer.
    let slice: &str = &owned;
    println!("slice:    {:?}  points into owned's heap memory", slice);

    // Partial slice — still a &str, still just a view.
    let partial: &str = &owned[1..4]; // bytes 1..4 = "ell"
    println!("partial:  {:?}", partial);

    // Key rule: you can always get a &str from a String (zero cost).
    // Going the other way (&str → String) always allocates.
}

// ---- Memory Layout -----------------------------------------------------------

fn memory_layout() {
    println!("=== Memory Layout ===");

    let s = String::from("rust");

    // std::mem::size_of tells us the stack size of the type.
    // A String is 3 usize fields: ptr + len + cap = 24 bytes on 64-bit.
    println!("size of String: {} bytes", std::mem::size_of::<String>());

    // A &str is 2 usize fields: ptr + len = 16 bytes on 64-bit.
    println!("size of &str:   {} bytes", std::mem::size_of::<&str>());

    // The actual string bytes live on the heap.
    let ptr = s.as_ptr();
    println!("stack lives at:  {:p}", &s);
    println!("heap lives at:   {:p}", ptr);
    println!("they are different addresses — stack ptr → heap bytes");
}

// ---- Why You Can't Index a String --------------------------------------------

fn why_no_indexing() {
    println!("=== Why No Indexing ===");

    let s = "café";

    // s[0] would be a compile error. But here's why it matters:
    // Let's look at the actual bytes:
    let bytes: &[u8] = s.as_bytes();
    print!("bytes of \"café\": ");
    for b in bytes {
        print!("{} ", b);
    }
    println!();
    // Output: 99 97 102 195 169
    // 'c'=99, 'a'=97, 'f'=102, 'é'=195+169 (two bytes!)

    println!("byte length: {}", s.len());         // 5
    println!("char length: {}", s.chars().count()); // 4

    // s[3] as a byte gives you 195 — one half of 'é'.
    // s[3] as a char makes no sense.
    // s[3..4] would panic at runtime (not on a char boundary).
    // Rust just disallows integer indexing entirely to avoid this class of bug.
    println!("Rust does not allow s[i] — use .chars() or explicit byte slices");
}

// ---- chars() vs bytes() ------------------------------------------------------

fn chars_vs_bytes() {
    println!("=== chars() vs bytes() ===");

    let s = "café 🦀";

    // chars(): iterate over Unicode scalar values (what you mean by "characters")
    print!("chars: ");
    for ch in s.chars() {
        print!("'{}' ", ch);
    }
    println!();
    println!("char count: {}", s.chars().count()); // 7: c a f é (space) 🦀

    // bytes(): iterate over raw UTF-8 bytes
    print!("bytes: ");
    for b in s.bytes() {
        print!("{} ", b);
    }
    println!();
    println!("byte count (len): {}", s.len()); // 11: 5 for café, 1 space, 4 for 🦀

    // The crab emoji is 4 bytes in UTF-8.
    // In JavaScript: "🦀".length === 2 (UTF-16 surrogate pairs).
    // In Rust:       "🦀".len() === 4  (UTF-8 bytes).
    println!();

    // Getting the nth char (O(n) — must scan from start)
    let third: Option<char> = s.chars().nth(2);
    println!("3rd char: {:?}", third); // Some('f')

    // char_indices gives you (byte_offset, char) pairs
    println!("char_indices:");
    for (i, ch) in s.char_indices() {
        println!("  byte {}: '{}'", i, ch);
    }
}

// ---- String Slicing ----------------------------------------------------------

fn slicing() {
    println!("=== Slicing ===");

    let s = "hello world";

    // Safe slices — splitting on known boundaries
    let hello = &s[0..5];
    let world = &s[6..11];
    println!("hello={:?}  world={:?}", hello, world);

    // split() and friends always return slices at char boundaries — safe
    let parts: Vec<&str> = s.split_whitespace().collect();
    println!("split_whitespace: {:?}", parts);

    // find() returns the byte index of a pattern — safe to slice on
    if let Some(pos) = s.find("world") {
        let rest = &s[pos..];
        println!("from 'world': {:?}", rest);
    }

    // Slice the last n chars safely
    let s = "hello café";
    let last_five_chars: String = s.chars().rev().take(4).collect::<String>()
                                   .chars().rev().collect();
    println!("last 4 chars: {:?}", last_five_chars);

    // Demonstrate the panic boundary (commented out — would panic at runtime)
    // let bad = &"café"[3..4]; // PANIC: not a char boundary ('é' starts at 3, is 2 bytes)
    println!("slicing on non-char boundaries panics at runtime — use split/find");
}

// ---- Conversions -------------------------------------------------------------

fn conversions() {
    println!("=== Conversions ===");

    // &str → String (multiple equivalent ways)
    let a: String = "hello".to_string();    // via ToString trait
    let b: String = String::from("hello");  // via From trait
    let c: String = "hello".to_owned();     // via ToOwned trait — most explicit intent
    assert_eq!(a, b);
    assert_eq!(b, c);
    println!("&str → String: all three methods produce identical Strings");

    // String → &str (zero cost — just borrows)
    let owned = String::from("hello");
    let borrowed: &str = &owned;            // explicit borrow
    let explicit: &str = owned.as_str();    // explicit method
    assert_eq!(borrowed, explicit);
    println!("String → &str: {:?}", borrowed);

    // Deref coercion: &String → &str automatically
    // This is why functions that accept &str work with &String too.
    fn print_str(s: &str) { println!("got: {}", s); }
    print_str(&owned);   // &String coerces to &str without any explicit conversion

    // String → Vec<u8> (consuming conversion)
    let owned = String::from("hello");
    let bytes: Vec<u8> = owned.into_bytes();  // owned is moved
    println!("bytes: {:?}", bytes);

    // Vec<u8> → String (validates UTF-8)
    match String::from_utf8(bytes) {
        Ok(s) => println!("back to string: {:?}", s),
        Err(e) => println!("invalid UTF-8: {}", e),
    }
}

// ---- Function Signatures: &str vs &String ------------------------------------

fn function_signatures() {
    println!("=== Function Signatures ===");

    // Prefer &str in parameters — works with both String and &str via Deref coercion
    fn count_vowels(s: &str) -> usize {
        s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
    }

    let owned = String::from("Hello, Rust!");
    let literal = "Hello, Rust!";

    // Both work — no conversion needed
    println!("vowels in owned:   {}", count_vowels(&owned));
    println!("vowels in literal: {}", count_vowels(literal));

    // Prefer String in return types when building new data
    fn normalize(s: &str) -> String {
        s.trim().to_lowercase()
    }
    println!("normalized: {:?}", normalize("  HELLO  "));

    // When to return &str: only when you're slicing the input
    fn first_word(s: &str) -> &str {
        match s.find(' ') {
            Some(pos) => &s[..pos],
            None => s,
        }
    }
    println!("first word: {:?}", first_word("hello world"));

    // The returned &str borrows from the parameter — the lifetimes are tied.
    // This is fine; Rust's borrow checker enforces it automatically.
}
