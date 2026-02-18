// Generic Bounded Channel — Reference Solution
//
// Run tests: rustc --test main.rs && ./main

// ---------- Serialize trait ----------

pub trait Serialize {
    fn serialize(&self) -> String;
}

impl Serialize for i32    { fn serialize(&self) -> String { self.to_string() } }
impl Serialize for f64    { fn serialize(&self) -> String { format!("{:.4}", self) } }
impl Serialize for bool   { fn serialize(&self) -> String { self.to_string() } }
impl Serialize for String { fn serialize(&self) -> String { format!("\"{}\"", self) } }

// ---------- Channel ----------

// No bounds on the struct definition.
// Bounds appear only where they are needed: on impl blocks that use T's capabilities.
pub struct Channel<T, const N: usize> {
    data: [Option<T>; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Copy, const N: usize> Channel<T, N> {
    pub fn new() -> Self {
        // [None; N] requires T: Copy for the array repeat expression.
        Channel {
            data: [None; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn send(&mut self, value: T) -> bool {
        if self.len == N {
            return false;  // full — no panic, no block
        }
        self.data[self.tail] = Some(value);
        self.tail = (self.tail + 1) % N;  // wrap around
        self.len += 1;
        true
    }

    pub fn recv(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let item = self.data[self.head].take();
        self.head = (self.head + 1) % N;  // wrap around
        self.len -= 1;
        item
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    // Associated function (no &self): N is a compile-time constant on the type.
    // Called as Channel::<MyType, 16>::capacity() or with a concrete type via turbofish.
    pub fn capacity() -> usize {
        N
    }
}

// ---------- SerializableChannel ----------

pub struct SerializableChannel<T: Copy, const N: usize>(Channel<T, N>);

impl<T: Copy, const N: usize> SerializableChannel<T, N> {
    pub fn new() -> Self {
        SerializableChannel(Channel::new())
    }

    // Delegate to inner channel
    pub fn send(&mut self, value: T) -> bool { self.0.send(value) }
    pub fn recv(&mut self) -> Option<T>      { self.0.recv() }
    pub fn len(&self) -> usize               { self.0.len() }
    pub fn is_empty(&self) -> bool           { self.0.is_empty() }
    pub fn is_full(&self) -> bool            { self.0.is_full() }
    pub fn capacity() -> usize               { Channel::<T, N>::capacity() }
}

// Separate impl block — this method only exists when T: Serialize.
// Callers with a non-Serialize T simply won't see drain_as_json.
// This is the idiomatic way to add "conditional" methods in Rust:
// you don't use if-constraints in the method body; you put the constraint on the impl.
impl<T: Copy + Serialize, const N: usize> SerializableChannel<T, N> {
    pub fn drain_as_json(&mut self) -> Vec<String> {
        let mut result = Vec::new();
        while let Some(item) = self.0.recv() {
            result.push(item.serialize());
        }
        result
    }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ch.recv(), Some(10));
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
        assert!(!ch.send(4));
        assert_eq!(ch.len(), 3);
    }

    #[test]
    fn test_recv_returns_none_when_empty() {
        let mut ch: Channel<i32, 4> = Channel::new();
        assert_eq!(ch.recv(), None);
    }

    #[test]
    fn test_wrap_around() {
        let mut ch: Channel<u32, 4> = Channel::new();
        ch.send(1);
        ch.send(2);
        ch.send(3);

        assert_eq!(ch.recv(), Some(1));
        assert_eq!(ch.recv(), Some(2));

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
        assert!(ch.is_empty());
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
        // Note: String does not implement Copy, so SerializableChannel<String, N> is not
        // constructible with our Copy-bound design. This is an intentional trade-off:
        // the Copy bound keeps initialization simple (array repeat expressions require Copy).
        // For non-Copy types, you'd use a Vec-backed channel instead.
        let mut ch: SerializableChannel<bool, 4> = SerializableChannel::new();
        ch.send(true);
        ch.send(false);
        ch.send(true);

        let json = ch.drain_as_json();
        assert_eq!(json, vec!["true", "false", "true"]);
    }
}

fn main() {
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
    let jsons = sch.drain_as_json();
    println!("drained as json: {:?}", jsons);
    println!("channel empty after drain: {}", sch.is_empty());

    // Note: String is not Copy, so Channel<String, N> won't compile.
    // The Copy bound is the trade-off for stack-allocated const-generic arrays.
    // If you need non-Copy types, use a Vec-backed channel without the Copy constraint.
}
