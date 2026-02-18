// const_generics.rs — Const generics, array-size generics, stack-allocated structures
//
// Run: rustc const_generics.rs && ./const_generics

use std::fmt::Display;

// ---------- 1. Basic const generic: working with arrays of known size ----------

// [T; N] is a distinct type for each N — [i32; 4] and [i32; 8] are unrelated types.
// Without const generics, you'd have to write separate functions for each size,
// or accept &[T] (a runtime-sized slice).
fn sum_array<T, const N: usize>(arr: [T; N]) -> T
where
    T: std::ops::Add<Output = T> + Default + Copy,
{
    arr.iter().fold(T::default(), |acc, &x| acc + x)
}

fn dot_product<T, const N: usize>(a: [T; N], b: [T; N]) -> T
where
    T: std::ops::Add<Output = T> + std::ops::Mul<Output = T> + Default + Copy,
{
    let mut result = T::default();
    for i in 0..N {
        result = result + a[i] * b[i];
    }
    result
}

// ---------- 2. Struct with a const generic capacity ----------

// A fixed-capacity stack — stack-allocated, no heap involved.
// CAP is baked into the type: Stack<i32, 8> and Stack<i32, 16> are different types.
// The caller chooses capacity at compile time; there is no runtime reallocation.
struct Stack<T, const CAP: usize> {
    data: [Option<T>; CAP],
    top: usize,
}

// We need T: Copy to initialize [Option<T>; CAP] with [None; CAP].
// This is a current limitation of const generics + array initialization.
impl<T: Copy, const CAP: usize> Stack<T, CAP> {
    fn new() -> Self {
        Stack {
            data: [None; CAP],
            top: 0,
        }
    }

    fn push(&mut self, item: T) -> bool {
        if self.top == CAP {
            return false;  // full — no panic, just return false
        }
        self.data[self.top] = Some(item);
        self.top += 1;
        true
    }

    fn pop(&mut self) -> Option<T> {
        if self.top == 0 {
            return None;
        }
        self.top -= 1;
        self.data[self.top].take()
    }

    fn peek(&self) -> Option<&T> {
        if self.top == 0 {
            None
        } else {
            self.data[self.top - 1].as_ref()
        }
    }

    fn is_empty(&self) -> bool { self.top == 0 }
    #[allow(dead_code)]
    fn is_full(&self) -> bool { self.top == CAP }
    #[allow(dead_code)]
    fn len(&self) -> usize { self.top }

    // capacity() is a compile-time constant — no runtime cost
    fn capacity() -> usize { CAP }
}

// ---------- 3. Ring buffer with const capacity ----------

// A bounded FIFO queue that wraps around when it reaches the end.
// Used in embedded systems and real-time applications where heap allocation is forbidden.
// CAP must be a power of two for the modulo optimization to work cleanly (not enforced here).
struct RingBuffer<T: Copy, const CAP: usize> {
    data: [Option<T>; CAP],
    head: usize,  // read position
    tail: usize,  // write position
    len: usize,
}

impl<T: Copy, const CAP: usize> RingBuffer<T, CAP> {
    fn new() -> Self {
        RingBuffer {
            data: [None; CAP],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn send(&mut self, item: T) -> bool {
        if self.len == CAP {
            return false;  // full — caller must drain before sending more
        }
        self.data[self.tail] = Some(item);
        self.tail = (self.tail + 1) % CAP;
        self.len += 1;
        true
    }

    fn recv(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let item = self.data[self.head].take();
        self.head = (self.head + 1) % CAP;
        self.len -= 1;
        item
    }

    fn len(&self) -> usize { self.len }
    #[allow(dead_code)]
    fn capacity() -> usize { CAP }
    #[allow(dead_code)]
    fn is_empty(&self) -> bool { self.len == 0 }
    #[allow(dead_code)]
    fn is_full(&self) -> bool { self.len == CAP }
}

// ---------- 4. Const generic function on arrays with multiple sizes ----------

// Concatenate two arrays of potentially different sizes into a Vec.
// The types N and M are distinct const generic parameters.
fn concat_arrays<T: Copy, const N: usize, const M: usize>(
    a: [T; N],
    b: [T; M],
) -> Vec<T> {
    let mut result = Vec::with_capacity(N + M);
    result.extend_from_slice(&a);
    result.extend_from_slice(&b);
    result
}

// ---------- 5. A compile-time matrix type ----------

// A 2D matrix with dimensions baked into the type.
// Matrix<f32, 3, 3> and Matrix<f32, 4, 4> are completely different types.
// The compiler will refuse to multiply incompatible matrices (see transpose).
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: Vec<T>,  // using Vec here to keep initialization simple
}

impl<T: Default + Clone + Copy + Display, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    fn new() -> Self {
        Matrix {
            data: vec![T::default(); ROWS * COLS],
        }
    }

    fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[row * COLS + col] = value;
    }

    fn get(&self, row: usize, col: usize) -> T {
        self.data[row * COLS + col]
    }

    fn print(&self) {
        for row in 0..ROWS {
            for col in 0..COLS {
                print!("{:6}", self.get(row, col));
            }
            println!();
        }
    }
}

// Transpose returns a Matrix with swapped ROWS and COLS dimensions.
// The type signature enforces that you can't accidentally transpose into the wrong shape.
impl<T: Default + Clone + Copy + Display, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    fn transpose(&self) -> Matrix<T, COLS, ROWS> {
        let mut result = Matrix::<T, COLS, ROWS>::new();
        for row in 0..ROWS {
            for col in 0..COLS {
                result.set(col, row, self.get(row, col));
            }
        }
        result
    }
}

// ---------- Main ----------

fn main() {
    // 1. Array functions with const size
    println!("=== Array functions ===");
    let sum_i = sum_array([1u32, 2, 3, 4, 5]);
    let sum_f = sum_array([1.0f64, 2.0, 3.0]);
    println!("sum [1..5]: {}", sum_i);
    println!("sum floats: {}", sum_f);

    let dp = dot_product([1i32, 2, 3], [4, 5, 6]);  // 1*4 + 2*5 + 3*6 = 32
    println!("dot product: {}", dp);

    // 2. Stack with const capacity
    println!("\n=== Fixed-capacity stack ===");
    let mut stack: Stack<u32, 4> = Stack::new();
    println!("capacity: {}", Stack::<u32, 4>::capacity());

    stack.push(10);
    stack.push(20);
    stack.push(30);
    stack.push(40);
    let overflow = stack.push(50);  // false — full
    println!("push to full stack: {}", overflow);
    println!("peek: {:?}", stack.peek());

    while let Some(item) = stack.pop() {
        println!("pop: {}", item);
    }
    println!("empty: {}", stack.is_empty());

    // 3. Ring buffer
    println!("\n=== Ring buffer (capacity 4) ===");
    let mut buf: RingBuffer<u32, 4> = RingBuffer::new();

    buf.send(10);
    buf.send(20);
    buf.send(30);

    println!("recv: {:?}", buf.recv());  // 10 — FIFO
    println!("recv: {:?}", buf.recv());  // 20

    buf.send(40);
    buf.send(50);
    let overflow = buf.send(60);  // false — capacity 4, have 30+40+50 = 3, this is 4th: true
    println!("send to buffer (should succeed): {}", overflow);
    let really_overflow = buf.send(70);
    println!("send to full buffer: {}", really_overflow);

    println!("buffer len: {}", buf.len());
    while let Some(v) = buf.recv() {
        println!("recv: {}", v);
    }

    // 4. Concat arrays of different sizes
    println!("\n=== Concat arrays ===");
    let combined = concat_arrays([1u32, 2, 3], [4u32, 5, 6, 7, 8]);
    println!("combined: {:?}", combined);

    // 5. Matrix with compile-time dimensions
    println!("\n=== Matrix 2x3 ===");
    let mut m: Matrix<i32, 2, 3> = Matrix::new();
    m.set(0, 0, 1); m.set(0, 1, 2); m.set(0, 2, 3);
    m.set(1, 0, 4); m.set(1, 1, 5); m.set(1, 2, 6);
    m.print();

    println!("transposed (3x2):");
    let t = m.transpose();  // returns Matrix<i32, 3, 2> — enforced by the type system
    t.print();

    // These would not compile — type mismatch on matrix dimensions:
    // let wrong: Matrix<i32, 4, 4> = m.transpose();  // ERROR: expected Matrix<i32,3,2>

    // 6. Size demonstration — const generics change the size of the struct
    println!("\n=== Sizes with const generics ===");
    println!("Stack<u32, 4> size:  {} bytes", std::mem::size_of::<Stack<u32, 4>>());
    println!("Stack<u32, 16> size: {} bytes", std::mem::size_of::<Stack<u32, 16>>());
    println!("RingBuffer<u8, 8> size:  {} bytes", std::mem::size_of::<RingBuffer<u8, 8>>());
    println!("RingBuffer<u8, 64> size: {} bytes", std::mem::size_of::<RingBuffer<u8, 64>>());
}
