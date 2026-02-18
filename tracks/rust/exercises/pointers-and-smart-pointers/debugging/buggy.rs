// Notification Dispatcher — Debugging Exercise (Rust)
//
// This code has 4 bugs related to smart pointers.
// Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};
use std::sync::Arc;

// ---------- Shared Channel Config ----------

#[derive(Debug, Clone)]
struct ChannelConfig {
    name: String,
    enabled: bool,
    rate_limit_per_minute: u32,
}

impl ChannelConfig {
    fn new(name: &str, rate_limit: u32) -> ChannelConfig {
        ChannelConfig {
            name: name.to_string(),
            enabled: true,
            rate_limit_per_minute: rate_limit,
        }
    }
}

// ---------- BUG 4: NotificationChannel with single owner where multiple owners needed ----------
//
// Currently NotificationChannel::new takes Rc<ChannelConfig> — that part is correct.
// But the config field is Rc<ChannelConfig>... wait, no.
// The BUG is: config is stored as a CLONE of the inner ChannelConfig, not as a shared Rc.
// Cloning means two channels hold independent copies. Changes to one don't affect the other.
// Fix: store Rc<ChannelConfig> and pass it through without cloning.

struct NotificationChannel {
    // BUG 4: stores a cloned ChannelConfig — breaks shared ownership.
    // Two channels that took Rc::clone() of the same config each hold their own copy.
    // Should store Rc<ChannelConfig> so they point to the same allocation.
    config: ChannelConfig,
    sent_count: u32,
}

impl NotificationChannel {
    // BUG 4: new() clones out of the Rc instead of storing the Rc.
    fn new(config: Rc<ChannelConfig>) -> NotificationChannel {
        NotificationChannel {
            config: (*config).clone(),  // BUG: clones the data, loses sharing
            sent_count: 0,
        }
    }

    fn send(&mut self, message: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        self.sent_count += 1;
        println!("[{}] sent: {}", self.config.name, message);
        true
    }

    fn config_name(&self) -> &str {
        &self.config.name
    }

    // Returns a pointer to the config allocation so tests can check sharing
    fn config_ptr(&self) -> *const ChannelConfig {
        &self.config as *const ChannelConfig
    }
}

// ---------- BUG 1: Reference cycle between Dispatcher and DispatcherNode ----------
//
// Dispatcher holds Rc<DispatcherNode> (via receipt_subscriber).
// DispatcherNode holds Rc<Dispatcher> (via dispatcher).
// This is a cycle: neither allocation's strong count reaches zero.
// Fix: the node's back-reference to the dispatcher should be Weak<Dispatcher>.

struct Dispatcher {
    name: String,
    // The subscriber that wants delivery receipts from this dispatcher.
    // The dispatcher OWNS the subscriber relationship (it chose to notify this node).
    receipt_subscriber: RefCell<Option<Rc<DispatcherNode>>>,
    sent_count: Cell<u32>,
}

struct DispatcherNode {
    id: u32,
    // BUG 1: Rc<Dispatcher> creates a cycle. Should be Weak<Dispatcher>.
    // The node observes the dispatcher; it doesn't own it.
    dispatcher: RefCell<Option<Rc<Dispatcher>>>,
    receipts_received: Cell<u32>,
}

impl Dispatcher {
    fn new(name: &str) -> Rc<Dispatcher> {
        Rc::new(Dispatcher {
            name: name.to_string(),
            receipt_subscriber: RefCell::new(None),
            sent_count: Cell::new(0),
        })
    }

    fn set_receipt_subscriber(dispatcher: &Rc<Dispatcher>, node: &Rc<DispatcherNode>) {
        *dispatcher.receipt_subscriber.borrow_mut() = Some(Rc::clone(node));
    }

    fn dispatch(&self, message: &str) {
        self.sent_count.set(self.sent_count.get() + 1);
        println!("[{}] dispatch: {}", self.name, message);
        if let Some(sub) = self.receipt_subscriber.borrow().as_ref() {
            sub.record_receipt();
        }
    }
}

impl DispatcherNode {
    fn new(id: u32) -> Rc<DispatcherNode> {
        Rc::new(DispatcherNode {
            id,
            dispatcher: RefCell::new(None),
            receipts_received: Cell::new(0),
        })
    }

    fn attach(node: &Rc<DispatcherNode>, dispatcher: &Rc<Dispatcher>) {
        // BUG 1: stores Rc::clone(&dispatcher) — this is the cycle-creating half.
        // Should be: Some(Rc::downgrade(dispatcher)) with dispatcher field as Weak<Dispatcher>
        *node.dispatcher.borrow_mut() = Some(Rc::clone(dispatcher));
        Dispatcher::set_receipt_subscriber(dispatcher, node);
    }

    fn record_receipt(&self) {
        self.receipts_received.set(self.receipts_received.get() + 1);
    }

    fn get_dispatcher_name(&self) -> Option<String> {
        // With Rc: just clone the name from the Rc
        // After fix to Weak: must call .upgrade() first
        self.dispatcher.borrow().as_ref().map(|d| d.name.clone())
    }
}

// ---------- BUG 2: borrow_mut() used for a read-only operation in get_count ----------
//
// get_count() only reads from the HashMap but calls borrow_mut().
// This acquires an exclusive write lock on the RefCell. If anything else borrows counts
// in the same call chain, it will panic. It also prevents concurrent reads.
// Fix: use borrow() for read-only access.

struct RateLimiter {
    counts: RefCell<std::collections::HashMap<String, u32>>,
    limits: RefCell<std::collections::HashMap<String, u32>>,
}

impl RateLimiter {
    fn new() -> RateLimiter {
        RateLimiter {
            counts: RefCell::new(std::collections::HashMap::new()),
            limits: RefCell::new(std::collections::HashMap::new()),
        }
    }

    fn set_limit(&self, channel: &str, limit: u32) {
        self.limits.borrow_mut().insert(channel.to_string(), limit);
    }

    fn check_and_record(&self, channel: &str) -> bool {
        let limit = *self.limits.borrow().get(channel).unwrap_or(&100);
        let mut counts = self.counts.borrow_mut();
        let count = counts.entry(channel.to_string()).or_insert(0);
        if *count >= limit {
            return false;
        }
        *count += 1;
        true
    }

    // BUG 2: Uses borrow_mut() to read — takes an exclusive lock unnecessarily.
    // If check_and_record is also active (holding borrow_mut on counts), this panics.
    // Fix: use borrow() instead of borrow_mut().
    fn get_count(&self, channel: &str) -> u32 {
        *self.counts.borrow_mut().get(channel).unwrap_or(&0)
    }

    // BUG 2 (demonstrating the panic): this method calls both check_and_record AND
    // get_count in a way that nests the borrows. With borrow_mut() in get_count,
    // the second borrow on counts panics.
    fn check_and_log(&self, channel: &str) -> bool {
        let allowed = self.check_and_record(channel);
        // counts is still mutably borrowed by check_and_record's borrow_mut() call.
        // get_count() also calls borrow_mut() on counts -> PANIC.
        // (Actually: check_and_record releases its borrow at the end of the call,
        //  so this won't panic unless they overlap. The real panic scenario is shown
        //  in the test below where we hold the borrow across calls.)
        let count = self.get_count(channel);
        println!("[{}] allowed={}, total_sent={}", channel, allowed, count);
        allowed
    }
}

// BUG 2 (direct panic demonstration): hold a borrow_mut then call get_count
#[allow(dead_code)]
fn demonstrate_double_borrow_panic(rl: &RateLimiter, channel: &str) {
    let mut counts = rl.counts.borrow_mut(); // mutable borrow: active
    let _ = counts.entry(channel.to_string()).or_insert(0);
    // Now call get_count which also tries to borrow_mut counts:
    // This PANICS: already mutably borrowed: BorrowMutError
    // let _ = rl.get_count(channel);
}

// ---------- BUG 3: Rc cannot be sent across threads ----------
//
// spawn_channel_worker takes Rc<ChannelConfig>.
// Rc is NOT Send: the compiler will reject this with a clear error:
//   "Rc<ChannelConfig> cannot be sent between threads safely"
// Fix: change the parameter type to Arc<ChannelConfig>.

fn spawn_channel_worker(config: Rc<ChannelConfig>) -> std::thread::JoinHandle<String> {
    // BUG 3: Rc<ChannelConfig> is not Send. This line does not compile.
    // Fix: change config: Rc<ChannelConfig> to config: Arc<ChannelConfig>
    std::thread::spawn(move || {
        format!("worker for channel: {}", config.name)
    })
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- BUG 1: Rc cycle (memory leak) --

    #[test]
    fn test_dispatcher_node_attach_and_dispatch() {
        let dispatcher = Dispatcher::new("email");
        let node = DispatcherNode::new(1);

        DispatcherNode::attach(&node, &dispatcher);
        dispatcher.dispatch("hello");
        dispatcher.dispatch("world");

        assert_eq!(node.receipts_received.get(), 2);
        assert_eq!(dispatcher.sent_count.get(), 2);
    }

    #[test]
    fn test_dispatcher_node_get_name() {
        let dispatcher = Dispatcher::new("sms");
        let node = DispatcherNode::new(2);
        DispatcherNode::attach(&node, &dispatcher);
        assert_eq!(node.get_dispatcher_name(), Some(String::from("sms")));
    }

    // This test catches the memory leak from the Rc cycle.
    // With the bug: weak.upgrade() returns Some after drop (dispatcher still alive due to cycle).
    // With the fix (Weak in node): weak.upgrade() returns None (dispatcher freed on drop).
    #[test]
    fn test_no_rc_cycle_dispatcher_freed_on_drop() {
        let dispatcher = Dispatcher::new("push");
        let node = DispatcherNode::new(3);
        DispatcherNode::attach(&node, &dispatcher);

        let weak = Rc::downgrade(&dispatcher);
        drop(dispatcher);  // drop the local strong reference

        // BUG 1: With the cycle, strong_count is still 1 (node holds it).
        // With the fix, strong_count is 0 and upgrade() returns None.
        assert!(weak.upgrade().is_none(),
            "dispatcher should be freed — fix the Rc cycle by using Weak in DispatcherNode");
    }

    // -- BUG 2: borrow_mut for read-only --

    #[test]
    fn test_rate_limiter_basic() {
        let rl = RateLimiter::new();
        rl.set_limit("email", 3);

        assert!(rl.check_and_record("email"));
        assert!(rl.check_and_record("email"));
        assert!(rl.check_and_record("email"));
        assert!(!rl.check_and_record("email"));  // blocked
    }

    #[test]
    fn test_rate_limiter_get_count() {
        let rl = RateLimiter::new();
        rl.set_limit("sms", 5);
        rl.check_and_record("sms");
        rl.check_and_record("sms");
        // BUG 2: This passes when borrows don't overlap. But check_and_log below panics.
        assert_eq!(rl.get_count("sms"), 2);
    }

    // This test catches the runtime panic from the double borrow.
    // check_and_log calls check_and_record (which holds borrow_mut on counts)
    // and then get_count (which also calls borrow_mut on counts).
    // After check_and_record returns, its borrow is released, so in this
    // sequential call the panic doesn't manifest — the bug is in get_count
    // using borrow_mut when borrow() would be correct.
    // The fix (borrow() in get_count) also prevents future panics if the code
    // is ever changed to hold the borrow across the get_count call.
    #[test]
    fn test_check_and_log_does_not_panic() {
        let rl = RateLimiter::new();
        rl.set_limit("push", 2);
        // This should NOT panic — fix get_count to use borrow() not borrow_mut()
        let allowed = rl.check_and_log("push");
        assert!(allowed);
    }

    // -- BUG 3: Rc not Send (compile error) --
    // To fix: change spawn_channel_worker to take Arc<ChannelConfig> and update this test.
    // The test below uses Arc — the function signature needs to match.
    #[test]
    fn test_channel_worker_thread() {
        let config = Arc::new(ChannelConfig::new("push", 100));
        // BUG 3: This will not compile until spawn_channel_worker takes Arc<ChannelConfig>.
        // The function currently takes Rc<ChannelConfig>, which is not Send.
        let handle = spawn_channel_worker(Rc::new(ChannelConfig::new("push", 100)));
        let result = handle.join().unwrap();
        assert!(result.contains("push"));
        drop(config);  // suppress unused variable warning on config
    }

    // -- BUG 4: config cloned instead of shared --

    #[test]
    fn test_two_channels_share_config() {
        let shared = Rc::new(ChannelConfig::new("email", 100));

        let channel_a = NotificationChannel::new(Rc::clone(&shared));
        let channel_b = NotificationChannel::new(Rc::clone(&shared));

        // BUG 4: With the bug, both channels cloned the config — they point to different allocations.
        // With the fix, both channels share the Rc — same allocation.
        assert_eq!(channel_a.config_ptr(), channel_b.config_ptr(),
            "both channels should point to the same config allocation — fix: store Rc<ChannelConfig>");
    }

    #[test]
    fn test_channel_sends() {
        let config = Rc::new(ChannelConfig::new("sms", 10));
        let mut channel = NotificationChannel::new(Rc::clone(&config));
        assert!(channel.send("test message"));
        assert_eq!(channel.sent_count, 1);
    }
}

fn main() {
    println!("Notification Dispatcher — run with: rustc --test buggy.rs && ./buggy");
}
