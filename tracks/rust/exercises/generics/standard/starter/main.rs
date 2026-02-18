// Generic Bounded Channel — Starter
//
// Run tests: rustc --test main.rs && ./main

// ---------- Serialize trait ----------

/// A simplified serialization trait.
/// In real code you'd use serde, but we define our own here to practice
/// implementing trait bounds on generic types.
pub trait Serialize {
    fn serialize(&self) -> String;
}

impl Serialize for i32    { fn serialize(&self) -> String { self.to_string() } }
impl Serialize for f64    { fn serialize(&self) -> String { format!("{:.4}", self) } }
impl Serialize for bool   { fn serialize(&self) -> String { self.to_string() } }
impl Serialize for String { fn serialize(&self) -> String { format!("\"{}\"", self) } }

// ---------- Channel ----------

/// A bounded, typed channel backed by a const-generic ring buffer.
/// N is the capacity — baked into the type at compile time.
///
/// Struct definition has no bounds (bounds go on impl blocks).
pub struct Channel<T, const N: usize> {
    // TODO: add fields
    // Hint: you need data: [Option<T>; N], plus head, tail, and len
    todo: std::marker::PhantomData<T>,  // remove this when you add real fields
}

impl<T: Copy, const N: usize> Channel<T, N> {
    /// Creates an empty channel.
    pub fn new() -> Self {
        todo!()
    }

    /// Sends a value. Returns false if the buffer is full.
    pub fn send(&mut self, value: T) -> bool {
        todo!()
    }

    /// Receives a value. Returns None if the buffer is empty.
    pub fn recv(&mut self) -> Option<T> {
        todo!()
    }

    /// Number of messages currently buffered.
    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn is_full(&self) -> bool {
        todo!()
    }

    /// Returns the capacity (N) as a runtime value.
    /// This is an associated function — no &self needed.
    pub fn capacity() -> usize {
        todo!()
    }
}

// ---------- SerializableChannel ----------

/// A wrapper around Channel that adds drain_as_json when T: Serialize.
pub struct SerializableChannel<T: Copy, const N: usize>(Channel<T, N>);

impl<T: Copy, const N: usize> SerializableChannel<T, N> {
    pub fn new() -> Self {
        todo!()
    }

    // Delegate these to the inner Channel:
    pub fn send(&mut self, value: T) -> bool { todo!() }
    pub fn recv(&mut self) -> Option<T>      { todo!() }
    pub fn len(&self) -> usize               { todo!() }
    pub fn is_empty(&self) -> bool           { todo!() }
    pub fn is_full(&self) -> bool            { todo!() }
    pub fn capacity() -> usize              { todo!() }
}

// TODO: Add a separate impl block for drain_as_json that only exists when T: Serialize.
// This method drains all buffered messages, serializes each, and returns the strings.

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Channel basics ---

    #[test]
    fn test_new_channel_is_empty() {
        let ch: Channel<i32, 8> = Channel::new();
        assert!(ch.is_empty());
        assert_eq!(ch.len(), 0);
    }

    #[test]
    fn test_capacity_is_const() {
        assert_eq!(Channel::<i32, 8>::capacity(), 8);
        assert_eq!(Channel::<f64, 16>::capacity(), 16);
        assert_eq!(Channel::<bool, 1>::capacity(), 1);
    }

    #[test]
    fn test_send_and_recv_basic() {
        let mut ch: Channel<i32, 4> = Channel::new();
        assert!(ch.send(10));
        assert!(ch.send(20));
        assert_eq!(ch.len(), 2);
        assert_eq!(ch.recv(), Some(10));  // FIFO
        assert_eq!(ch.recv(), Some(20));
        assert_eq!(ch.recv(), None);
    }

    #[test]
    fn test_fifo_ordering() {
        let mut ch: Channel<u32, 8> = Channel::new();
        for i in 0..5u32 {
            ch.send(i);
        }
        let received: Vec<u32> = (0..5).filter_map(|_| ch.recv()).collect();
        assert_eq!(received, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_send_returns_false_when_full() {
        let mut ch: Channel<i32, 3> = Channel::new();
        assert!(ch.send(1));
        assert!(ch.send(2));
        assert!(ch.send(3));
        assert!(ch.is_full());
        assert!(!ch.send(4));  // full — returns false, does not panic
        assert_eq!(ch.len(), 3);
    }

    #[test]
    fn test_recv_returns_none_when_empty() {
        let mut ch: Channel<i32, 4> = Channel::new();
        assert_eq!(ch.recv(), None);
    }

    #[test]
    fn test_wrap_around() {
        // Fill, partially drain, then refill to test ring buffer wrap-around.
        let mut ch: Channel<u32, 4> = Channel::new();
        ch.send(1);
        ch.send(2);
        ch.send(3);

        assert_eq!(ch.recv(), Some(1));
        assert_eq!(ch.recv(), Some(2));

        // head has moved; tail should wrap when we push more
        ch.send(4);
        ch.send(5);

        let received: Vec<u32> = (0..3).filter_map(|_| ch.recv()).collect();
        assert_eq!(received, vec![3, 4, 5]);
    }

    #[test]
    fn test_channel_with_f64() {
        let mut ch: Channel<f64, 4> = Channel::new();
        ch.send(1.5);
        ch.send(2.7);
        assert_eq!(ch.recv(), Some(1.5));
        assert_eq!(ch.recv(), Some(2.7));
    }

    // --- SerializableChannel ---

    #[test]
    fn test_serializable_channel_delegates_correctly() {
        let mut ch: SerializableChannel<i32, 8> = SerializableChannel::new();
        assert!(ch.send(100));
        assert!(ch.send(200));
        assert_eq!(ch.len(), 2);
        assert_eq!(ch.recv(), Some(100));
        assert_eq!(SerializableChannel::<i32, 8>::capacity(), 8);
    }

    #[test]
    fn test_drain_as_json_integers() {
        let mut ch: SerializableChannel<i32, 8> = SerializableChannel::new();
        ch.send(1);
        ch.send(2);
        ch.send(3);

        let json = ch.drain_as_json();
        assert_eq!(json, vec!["1", "2", "3"]);
        assert!(ch.is_empty());  // drained
    }

    #[test]
    fn test_drain_as_json_floats() {
        let mut ch: SerializableChannel<f64, 4> = SerializableChannel::new();
        ch.send(3.14);
        ch.send(2.718);

        let json = ch.drain_as_json();
        assert_eq!(json, vec!["3.1400", "2.7180"]);
        assert!(ch.is_empty());
    }

    #[test]
    fn test_drain_empty_channel() {
        let mut ch: SerializableChannel<i32, 4> = SerializableChannel::new();
        let json = ch.drain_as_json();
        assert!(json.is_empty());
    }

    #[test]
    fn test_drain_as_json_bools() {
        // Note: String does not implement Copy, so Channel<String, N> won't compile.
        // The Copy bound is the cost of using [Option<T>; N] for stack-allocated storage.
        let mut ch: SerializableChannel<bool, 4> = SerializableChannel::new();
        ch.send(true);
        ch.send(false);
        ch.send(true);

        let json = ch.drain_as_json();
        assert_eq!(json, vec!["true", "false", "true"]);
    }
}

fn main() {
    // Demonstration
    println!("=== Channel<i32, 4> ===");
    let mut ch: Channel<i32, 4> = Channel::new();
    ch.send(10);
    ch.send(20);
    ch.send(30);
    println!("capacity: {}", Channel::<i32, 4>::capacity());
    println!("len: {}", ch.len());
    while let Some(v) = ch.recv() {
        println!("recv: {}", v);
    }

    println!("\n=== SerializableChannel<f64, 4> ===");
    let mut sch: SerializableChannel<f64, 4> = SerializableChannel::new();
    sch.send(98.6);
    sch.send(37.2);
    sch.send(42.0);
    // drain_as_json only available after you implement the Serialize impl block
    // let jsons = sch.drain_as_json();
    println!("len: {}", sch.len());
}
