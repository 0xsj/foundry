# Solution: Generic Bounded Channel

## Approach

The design centers on two key choices: using `[Option<T>; N]` for stack-allocated const-generic storage, and a conditional `impl` block to restrict `drain_as_json` to types that implement `Serialize`.

The ring buffer is the core of `Channel<T, N>`. Two indices (`head` and `tail`) track the read and write positions. Both wrap around with `% N` when they reach the capacity, forming the "ring." The invariant: `len` never exceeds `N`.

`SerializableChannel` is a thin newtype wrapper. The wrapper's primary purpose is to gate `drain_as_json` behind a trait bound via a separate `impl` block — the cleanest way in Rust to add methods that only exist for certain instantiations.

## Key Decisions

**Why `T: Copy` and not `T: Clone`?**

Array initialization with the repeat expression `[None; N]` requires `T: Copy`. When `T` is `Option<SomeType>`, `SomeType` must be `Copy` for `None::<Option<SomeType>>` to be copied `N` times. This rules out `String`, `Vec<T>`, and other heap-allocated types.

The trade-off is real: a ring buffer for telemetry readings (sensor values, flags, counters) fits naturally with `Copy` types. If you need a channel for `String` or other non-Copy types, the alternative is a `Vec`-backed channel — which loses the stack-allocated, const-generic size guarantee but gains flexibility.

**Why no bounds on the struct definition?**

Placing bounds on the struct (`pub struct Channel<T: Copy, const N: usize>`) forces every `impl` and every type that mentions `Channel` to repeat the bound. Placing bounds only on `impl` blocks allows code that doesn't use T's capabilities (e.g., `len()`, `is_empty()`) to work without the bound. This is idiomatic Rust.

**Why a separate `impl` block for `drain_as_json`?**

```rust
// This impl block only exists when T: Copy + Serialize
impl<T: Copy + Serialize, const N: usize> SerializableChannel<T, N> {
    pub fn drain_as_json(&mut self) -> Vec<String> { ... }
}
```

This is the correct pattern for conditional methods in Rust. The method literally does not exist in the type if `T` doesn't implement `Serialize`. The compiler enforces this at the call site — no runtime checks, no panic, just a compile error that says "method not found."

**Why an associated function for `capacity()`?**

`N` is part of the type. You don't need an instance to know the capacity. An associated function `fn capacity() -> usize` is callable without a `Channel` value: `Channel::<i32, 16>::capacity()`. This is the pattern standard library types use for size-related constants.

## Variant Approaches

| Approach | Trade-off |
|---|---|
| `[Option<T>; N]` with `T: Copy` (reference solution) | Stack allocated, zero-cost; restricted to Copy types |
| `Vec<Option<T>>` with capacity N | Works with any T; heap allocated; size not in type |
| `arrayvec::ArrayVec` (external crate) | Handles non-Copy; well-tested; adds a dependency |
| `MaybeUninit<T>` array | Works with non-Copy, avoids Option overhead; requires unsafe |

## Connection to Bigger Picture

The conditional `impl` block pattern (`impl<T: Bound> Struct<T>`) is the foundation of Rust's approach to optional capabilities. You'll see it throughout the standard library: `Vec<T>` only implements `Display` when `T: Display`, `Arc<T>` only implements `Send` when `T: Send`. The pattern scales: a struct can have many `impl` blocks, each guarded by different trait bounds, adding different capabilities depending on what T provides.

The const generic design (`Channel<T, const N: usize>`) previews how the standard library types like `[T; N]` work. When you write `[u8; 1024]`, you're using const generics. This module's `Stack<T, CAP>` and `RingBuffer<T, CAP>` in `const_generics.rs` show the same pattern from the implementation side.
