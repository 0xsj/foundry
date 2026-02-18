# Standard Exercise: Generic Bounded Channel

## Scenario

You're building a lightweight inter-thread message bus for an embedded telemetry service. The service
collects readings from multiple sensors and routes them to processing workers. You need a typed,
bounded channel abstraction — like Go's buffered channels, but with Rust's type system enforcing both
the message type and the buffer capacity at compile time.

The channel must also support a "serializable" mode: when the message type implements `Serialize`,
the channel exposes an additional method to drain all messages as JSON strings. This is used for
debugging and audit logging.

## Brief

Implement `Channel<T, const N: usize>` — a bounded, typed, single-producer/single-consumer channel
backed by a fixed-capacity ring buffer. The capacity `N` is a const generic so it's part of the
type: `Channel<Reading, 16>` and `Channel<Reading, 256>` are distinct types with distinct buffer
sizes, with no heap allocation for the buffer.

Additionally, implement `SerializableChannel<T, const N: usize>` as a thin wrapper around
`Channel<T, N>` that adds a `drain_as_json` method — available only when `T: Serialize`.

## Acceptance Criteria

1. **`Channel<T, const N: usize>`** struct
   - Backed by a const-generic ring buffer (no `Vec` for the buffer — use `[Option<T>; N]`)
   - `T: Copy` is required (simplifies array initialization with const generics)

2. **`Channel::new() -> Channel<T, N>`**
   - Creates an empty channel

3. **`Channel::send(&mut self, value: T) -> bool`**
   - Returns `true` if the value was added, `false` if the buffer is full
   - Does not block or panic on full

4. **`Channel::recv(&mut self) -> Option<T>`**
   - Returns `Some(T)` if a message is available, `None` if empty
   - FIFO ordering — first sent, first received

5. **`Channel::len(&self) -> usize`**
   - Number of messages currently buffered

6. **`Channel::is_empty(&self) -> bool`** and **`Channel::is_full(&self) -> bool`**

7. **`Channel::capacity() -> usize`**
   - Returns `N` (the const generic capacity)
   - Associated function (no `&self`), since N is known at compile time

8. **`Serialize` trait** (simplified version — do not import serde)
   - Define your own `Serialize` trait:
     ```rust
     pub trait Serialize {
         fn serialize(&self) -> String;
     }
     ```
   - Implement it for `i32`, `f64`, `bool`, and `String`

9. **`SerializableChannel<T, const N: usize>`**
   - A newtype wrapper: `pub struct SerializableChannel<T: Copy, const N: usize>(Channel<T, N>);`
   - Delegates `send`, `recv`, `len`, `is_empty`, `is_full`, `capacity` to the inner channel

10. **`SerializableChannel::drain_as_json(&mut self) -> Vec<String>`**
    - Only available when `T: Serialize` (use a separate `impl` block with the bound)
    - Drains all messages from the channel and returns them as serialized strings
    - The channel is empty after this call

## Constraints

- No external crates — stdlib only
- No `Vec` for the buffer storage — use `[Option<T>; N]`
- `Channel::capacity()` must be an associated function (not a method), since `N` is a compile-time constant
- All tests must pass with `rustc --test main.rs`

## Hints

<details>
<summary>Hint 1: Initializing [Option<T>; N] with const generics</summary>

This is one of the trickier parts. You can't write `[None::<T>; N]` directly unless `T: Copy`.
The simplest approach:

```rust
impl<T: Copy, const N: usize> Channel<T, N> {
    pub fn new() -> Self {
        Channel {
            data: [None; N],  // requires T: Copy
            head: 0,
            tail: 0,
            len: 0,
        }
    }
}
```

`T: Copy` is required on the struct or the impl block for this to work. Place it on the impl block,
not the struct definition, so the struct definition itself has no bounds.

</details>

<details>
<summary>Hint 2: Conditional impl block for Serialize</summary>

```rust
impl<T: Copy + Serialize, const N: usize> SerializableChannel<T, N> {
    pub fn drain_as_json(&mut self) -> Vec<String> {
        // ...
    }
}
```

This method only exists in the type system when T implements both Copy and Serialize.
Callers with `SerializableChannel<TypeThatDoesntSerialize, 8>` simply won't see this method.

</details>

<details>
<summary>Hint 3: Associated function for capacity</summary>

```rust
impl<T: Copy, const N: usize> Channel<T, N> {
    pub fn capacity() -> usize {
        N  // no &self needed — N is known from the type
    }
}
```

Call it as `Channel::<MyType, 16>::capacity()` or with turbofish from a known type.

</details>

<details>
<summary>Hint 4: Ring buffer wrap-around</summary>

```rust
fn send(&mut self, value: T) -> bool {
    if self.len == N { return false; }
    self.data[self.tail] = Some(value);
    self.tail = (self.tail + 1) % N;
    self.len += 1;
    true
}

fn recv(&mut self) -> Option<T> {
    if self.len == 0 { return None; }
    let item = self.data[self.head].take();
    self.head = (self.head + 1) % N;
    self.len -= 1;
    item
}
```

`% N` wraps the head and tail indices around when they reach the capacity.

</details>
