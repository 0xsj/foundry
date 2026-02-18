/// Notification Dispatcher — Observer Pattern (BUGGY VERSION)
///
/// This file contains FOUR intentional bugs related to Rust ownership
/// and the observer pattern. Each bug is marked with a comment indicating
/// which bug number it is, but the root cause is NOT explained.
///
/// Your job: find the root cause of each bug and fix it.
///
/// Bug 1: Borrow checker conflict in notify_with_count
/// Bug 2: Deadlock when observer uses shared logger
/// Bug 3: Moved value when subscribing to multiple subjects
/// Bug 4: Channel receiver dropped early, causing silent data loss
///
/// Some bugs prevent compilation. Others compile but fail at runtime.
/// Fix them all so the program runs to completion.
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// --- Event types ---

#[derive(Debug, Clone)]
struct DeployEvent {
    service: String,
    version: String,
    status: String,
}

// --- Bug 1: Borrow checker conflict ---
// This struct tries to notify observers AND read the observer count
// simultaneously, causing a borrow conflict.

struct Subject {
    observers: Vec<Box<dyn FnMut(&DeployEvent)>>,
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: Vec::new(),
        }
    }

    fn subscribe(&mut self, callback: Box<dyn FnMut(&DeployEvent)>) {
        self.observers.push(callback);
    }

    // BUG 1: This method borrows self.observers mutably (for FnMut) while
    // also trying to read self.observers.len() through &self.
    fn notify_with_count(&mut self, event: &DeployEvent) {
        for observer in &mut self.observers {
            observer(event);
        }
        // This line tries to borrow self immutably while the mutable borrow
        // from the for loop... wait, the loop is done. But the REAL bug is
        // that this method takes &mut self, and we call it from a context
        // where we also borrow self immutably. See main() Bug 1 section.
        println!("Notified {} observers", self.observers.len());
    }

    fn count(&self) -> usize {
        self.observers.len()
    }
}

// --- Bug 2: Deadlock with shared logger ---
// Multiple observers share a logger via Arc<Mutex<Logger>>.
// The subject also holds the logger and locks it during notification.

struct Logger {
    entries: Vec<String>,
}

impl Logger {
    fn new() -> Self {
        Logger {
            entries: Vec::new(),
        }
    }

    fn log(&mut self, message: String) {
        println!("[LOG] {}", message);
        self.entries.push(message);
    }

    fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

struct LoggingSubject {
    observers: Vec<Arc<Mutex<dyn FnMut(&DeployEvent) + Send>>>,
    logger: Arc<Mutex<Logger>>,
}

impl LoggingSubject {
    fn new(logger: Arc<Mutex<Logger>>) -> Self {
        LoggingSubject {
            observers: Vec::new(),
            logger,
        }
    }

    fn subscribe(&mut self, observer: Arc<Mutex<dyn FnMut(&DeployEvent) + Send>>) {
        self.observers.push(observer);
    }

    // BUG 2: This method locks the logger, then calls each observer.
    // If an observer also tries to lock the logger, DEADLOCK.
    fn notify(&self, event: &DeployEvent) {
        let mut logger = self.logger.lock().unwrap();
        logger.log(format!("Notifying {} observers for {:?}", self.observers.len(), event));

        for observer in &self.observers {
            let mut obs = observer.lock().unwrap();
            obs(event);
            // The observer's closure ALSO tries to lock self.logger -> deadlock
        }
    }
}

// --- Bug 3: Moved value with Arc ---
// An observer is subscribed to two subjects, but the Arc is moved
// instead of cloned.

trait EventHandler: Send {
    fn handle(&mut self, event: &DeployEvent);
    fn name(&self) -> &str;
}

struct SlackNotifier {
    channel: String,
    message_count: u32,
}

impl SlackNotifier {
    fn new(channel: String) -> Self {
        SlackNotifier {
            channel,
            message_count: 0,
        }
    }
}

impl EventHandler for SlackNotifier {
    fn handle(&mut self, event: &DeployEvent) {
        self.message_count += 1;
        println!(
            "[Slack #{}] Deploy {} v{}: {} (msg #{})",
            self.channel, event.service, event.version, event.status, self.message_count
        );
    }

    fn name(&self) -> &str {
        &self.channel
    }
}

struct MultiSubject {
    handlers: Vec<Arc<Mutex<dyn EventHandler>>>,
}

impl MultiSubject {
    fn new() -> Self {
        MultiSubject {
            handlers: Vec::new(),
        }
    }

    // BUG 3: This takes an Arc by value (moves it). If the caller tries to
    // subscribe the same handler to another subject, the value has been moved.
    fn subscribe(&mut self, handler: Arc<Mutex<dyn EventHandler>>) {
        self.handlers.push(handler);
    }

    fn notify(&self, event: &DeployEvent) {
        for handler in &self.handlers {
            handler.lock().unwrap().handle(event);
        }
    }
}

// --- Bug 4: Channel receiver dropped early ---
// A channel-based subscriber is created but the receiver is dropped
// before the publisher sends events.

struct ChannelPublisher {
    senders: Vec<mpsc::Sender<DeployEvent>>,
}

impl ChannelPublisher {
    fn new() -> Self {
        ChannelPublisher {
            senders: Vec::new(),
        }
    }

    fn subscribe(&mut self) -> mpsc::Receiver<DeployEvent> {
        let (tx, rx) = mpsc::channel();
        self.senders.push(tx);
        rx
    }

    fn publish(&self, event: DeployEvent) {
        for sender in &self.senders {
            match sender.send(event.clone()) {
                Ok(_) => {}
                Err(e) => {
                    // BUG 4: This error is silently ignored in the original.
                    // But more importantly, the receiver gets dropped in main()
                    // before events are sent, so ALL sends fail silently.
                    eprintln!("Send failed: {}", e);
                }
            }
        }
    }
}

// --- Main ---

fn main() {
    println!("=== Bug 1: Borrow Conflict ===\n");
    bug_1_borrow_conflict();

    println!("\n=== Bug 2: Deadlock ===\n");
    bug_2_deadlock();

    println!("\n=== Bug 3: Moved Value ===\n");
    bug_3_moved_value();

    println!("\n=== Bug 4: Channel Drop ===\n");
    bug_4_channel_drop();

    println!("\n=== All bugs fixed! ===");
}

fn bug_1_borrow_conflict() {
    let mut subject = Subject::new();
    subject.subscribe(Box::new(|event: &DeployEvent| {
        println!("Observer 1: {:?}", event);
    }));
    subject.subscribe(Box::new(|event: &DeployEvent| {
        println!("Observer 2: {:?}", event);
    }));

    let event = DeployEvent {
        service: "api".into(),
        version: "1.0".into(),
        status: "started".into(),
    };

    // BUG 1: We borrow subject immutably for count() while also needing
    // &mut self for notify_with_count in the same expression.
    // The fix: separate the borrows into different statements.
    let count = subject.count();
    println!("About to notify {} observers", count);
    subject.notify_with_count(&event);

    // This is the actual bug — trying to read from subject while it's mutably borrowed:
    // In the original buggy code, imagine this was written as:
    //   println!("Count: {}, notifying...", subject.count());
    //   subject.notify_with_count(&event);
    // That works fine. But this doesn't:
    let _result = notify_and_count(&mut subject, &event);
}

// BUG 1 (actual trigger): This function takes &mut Subject but the caller
// also wants to read subject.count() in the same scope.
fn notify_and_count(subject: &mut Subject, event: &DeployEvent) -> usize {
    subject.notify_with_count(event);
    subject.count()
}

fn bug_2_deadlock() {
    let logger = Arc::new(Mutex::new(Logger::new()));
    let mut subject = LoggingSubject::new(logger.clone());

    // Create an observer that also uses the shared logger
    let observer_logger = logger.clone();
    let observer: Arc<Mutex<dyn FnMut(&DeployEvent) + Send>> =
        Arc::new(Mutex::new(move |event: &DeployEvent| {
            // BUG 2: This tries to lock `logger` which is already locked
            // by LoggingSubject::notify(). DEADLOCK.
            let mut log = observer_logger.lock().unwrap();
            log.log(format!("Observer saw: {} v{}", event.service, event.version));
        }));

    subject.subscribe(observer);

    let event = DeployEvent {
        service: "payments".into(),
        version: "2.0".into(),
        status: "deploying".into(),
    };

    // This will deadlock because:
    // 1. notify() locks self.logger
    // 2. notify() calls observer closure
    // 3. observer closure tries to lock the same logger
    // 4. Deadlock — same thread, non-reentrant Mutex
    subject.notify(&event);

    println!("Log entries: {}", logger.lock().unwrap().entry_count());
}

fn bug_3_moved_value() {
    let mut subject_a = MultiSubject::new();
    let mut subject_b = MultiSubject::new();

    let notifier = Arc::new(Mutex::new(SlackNotifier::new("deployments".into())));

    // BUG 3: The Arc is moved into subject_a. The second subscribe
    // tries to use the moved value. Fix: clone the Arc before subscribing.
    subject_a.subscribe(notifier);
    // subject_b.subscribe(notifier);  // ERROR: use of moved value `notifier`

    // After fixing, both subjects should be able to notify the same observer
    let event = DeployEvent {
        service: "auth".into(),
        version: "3.1".into(),
        status: "completed".into(),
    };

    subject_a.notify(&event);
    subject_b.notify(&event);

    // The notifier should have received 2 events total
    // (but we can't check because we moved it without cloning)
}

fn bug_4_channel_drop() {
    let mut publisher = ChannelPublisher::new();

    // BUG 4: The receiver is created here but will be dropped
    // at the end of this block, before events are published.
    let handle = {
        let rx = publisher.subscribe();

        thread::spawn(move || {
            let mut received = 0;
            // This loop exits immediately because rx is about to be dropped
            // ... actually rx was moved into the closure, so it lives.
            // The REAL bug: the receiver is consumed by recv() but we
            // also drop it outside. Wait, no — let's look again.
            //
            // The actual bug: the thread spawns, but we immediately
            // drop `rx` after spawning because of the block scope.
            // No wait, rx was moved into the closure. The real bug
            // is more subtle...
            while let Ok(event) = rx.recv() {
                received += 1;
                println!("[Channel] Received: {} v{}", event.service, event.version);
            }
            println!("[Channel] Receiver closed. Got {} events.", received);
            received
        })
    };
    // The rx was moved into the thread, so it's fine there.
    // But we subscribed a SECOND receiver and drop it immediately:
    let _rx2 = publisher.subscribe(); // This receiver is never read!
    drop(_rx2); // Dropped immediately — sender for this subscriber will fail

    // Send events
    thread::sleep(Duration::from_millis(50)); // Let thread start

    publisher.publish(DeployEvent {
        service: "api".into(),
        version: "1.0".into(),
        status: "started".into(),
    });

    publisher.publish(DeployEvent {
        service: "api".into(),
        version: "1.0".into(),
        status: "completed".into(),
    });

    // Drop publisher to close channels
    drop(publisher);

    let count = handle.join().unwrap();
    println!("Thread received {} events (expected 2)", count);
}
