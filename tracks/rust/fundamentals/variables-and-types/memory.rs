use std::mem;

fn main() {
    // ========================================================================
    // 1. PRINT SIZES
    // Rust gives you std::mem::size_of and size_of_val.
    // ========================================================================

    let a: i32 = 42;
    let b: f64 = 3.14;
    let c: bool = true;
    let d: char = 'A';
    let e: &str = "hello";
    let f: String = String::from("hello");

    println!("=== Sizes ===");
    println!("i32    — size: {} bytes", mem::size_of_val(&a));
    println!("f64    — size: {} bytes", mem::size_of_val(&b));
    println!("bool   — size: {} bytes", mem::size_of_val(&c));
    println!("char   — size: {} bytes", mem::size_of_val(&d));
    println!("&str   — size: {} bytes", mem::size_of_val(&e));
    println!("String — size: {} bytes", mem::size_of_val(&f));
    // Q: Why is &str 16 bytes and String 24 bytes?
    //    What are the extra 8 bytes in String?
    //    (answer here)

    // ========================================================================
    // 2. ADDRESSES
    // Use {:p} format to print pointer addresses.
    // ========================================================================

    let x: i32 = 42;
    let y: i32 = 43;

    println!("\n=== Addresses ===");
    println!("x addr: {:p}", &x);
    println!("y addr: {:p}", &y);
    // Q: How far apart are x and y in memory? Does that match their sizes?
    //    (answer here)

    // ========================================================================
    // 3. MOVE SEMANTICS
    // When you assign a heap type, ownership MOVES.
    // ========================================================================

    let s1 = String::from("hello");
    println!("\n=== Move Semantics ===");
    println!("s1 addr: {:p}, data addr: {:p}", &s1, s1.as_ptr());

    let s2 = s1; // s1 is MOVED to s2
    // println!("s1: {}", s1); // uncomment to see the compile error
    println!("s2 addr: {:p}, data addr: {:p}", &s2, s2.as_ptr());
    // Q: s2 is at a different stack address than s1 was, but the data pointer
    //    is the same. What does that tell you about what "move" actually does?
    //    (answer here)

    // ========================================================================
    // 4. COPY SEMANTICS
    // Stack types that implement Copy are bitwise copied.
    // ========================================================================

    let a: i32 = 42;
    let b = a; // copy, not move — both are still valid

    println!("\n=== Copy Semantics ===");
    println!("a: {}, addr: {:p}", a, &a);
    println!("b: {}, addr: {:p}", b, &b);
    // Q: a and b have different addresses. The value was copied.
    //    Why doesn't String implement Copy?
    //    (answer here)

    // ========================================================================
    // 5. REFERENCES (BORROWING)
    // A reference is a pointer that the borrow checker tracks.
    // ========================================================================

    let s = String::from("hello");
    let r: &String = &s;

    println!("\n=== References ===");
    println!("s addr:  {:p}", &s);
    println!("r value: {:p} (points to s)", r);
    println!("r addr:  {:p} (where r itself lives)", &r);
    println!("r size:  {} bytes (it's a pointer)", mem::size_of_val(&r));
    // Q: r is 8 bytes (a pointer). It points to s on the stack.
    //    s is 24 bytes on the stack, pointing to heap data.
    //    Draw the full picture: stack → stack → heap.
    //    (answer here)

    // ========================================================================
    // 6. SHADOWING vs MUTATION IN MEMORY
    // ========================================================================

    let x = 5;
    println!("\n=== Shadowing ===");
    println!("x = {}, addr: {:p}", x, &x);

    let x = x + 1; // shadows — new variable
    println!("x = {}, addr: {:p}", x, &x);
    // Q: Did the address change? What does that tell you about shadowing?
    //    (answer here)

    let mut y = 5;
    println!("\ny = {}, addr: {:p}", y, &y);

    y = y + 1; // mutation — same variable
    println!("y = {}, addr: {:p}", y, &y);
    // Q: Did the address change this time? Why or why not?
    //    (answer here)
}
