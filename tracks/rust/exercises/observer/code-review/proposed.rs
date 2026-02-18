/// EventBus — Synchronous typed event dispatch system
///
/// This is a proposed implementation for PR review.
/// It contains several issues that should be identified during review.
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// --- Event Handler Trait ---

/// Trait for handling typed events.
/// Implementors receive events of type E and can process them.
trait EventHandler<E> {
    fn handle(&self, event: &E);

    /// Return a clone of this handler for re-registration purposes.
    fn clone_handler(&self) -> Self
    where
        Self: Sized;
}

// --- EventBus ---

/// A synchronous event bus that dispatches events to registered handlers.
///
/// Handlers are stored as trait objects grouped by event type.
/// When an event is emitted, all handlers for that event type are invoked
/// synchronously in registration order.
struct EventBus {
    /// Maps event TypeId to a list of handlers for that event type.
    /// Handlers are type-erased as Box<dyn Any> and downcast during dispatch.
    handlers: HashMap<TypeId, Vec<Box<dyn Any>>>,

    /// Shared reference to a handler registry for cross-service handler sharing.
    /// Uses Rc for cheap cloning between bus instances.
    shared_registry: Rc<RefCell<Vec<Box<dyn Any>>>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus {
            handlers: HashMap::new(),
            shared_registry: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Register a handler for events of type E.
    ///
    /// The handler will be called synchronously whenever an event of type E
    /// is emitted through this bus.
    fn subscribe<E: 'static>(&mut self, handler: &dyn EventHandler<E>) {
        let type_id = TypeId::of::<E>();
        // Store a reference to the handler
        let handler_ref: &dyn Fn(&E) = &|_event| {
            // This closure captures nothing useful — placeholder
        };
        // Actually, let's store the handler directly
        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(handler_ref as *const dyn Fn(&E)));
    }

    /// Register a closure-based handler for events of type E.
    fn on<E: 'static, F: Fn(&E) + 'static>(&mut self, callback: F) {
        let type_id = TypeId::of::<E>();
        let boxed: Box<dyn Fn(&E)> = Box::new(callback);
        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(boxed));
    }

    /// Emit an event, notifying all registered handlers for this event type.
    fn emit<E: 'static>(&self, event: &E) {
        let type_id = TypeId::of::<E>();
        if let Some(handlers) = self.handlers.get(&type_id) {
            for handler in handlers {
                // Downcast from Box<dyn Any> to the concrete handler type
                if let Some(callback) = handler.downcast_ref::<Box<dyn Fn(&E)>>() {
                    callback(event);
                }
            }
        }
    }

    /// Get the number of handlers registered for a specific event type.
    fn handler_count<E: 'static>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.handlers.get(&type_id).map_or(0, |h| h.len())
    }

    /// Share this bus's registry with another bus instance.
    fn share_registry(&self) -> Rc<RefCell<Vec<Box<dyn Any>>>> {
        self.shared_registry.clone()
    }
}

// --- Domain Events ---

#[derive(Debug, Clone)]
struct OrderCreated {
    order_id: String,
    customer_id: String,
    total_cents: u64,
}

#[derive(Debug, Clone)]
struct PaymentProcessed {
    order_id: String,
    payment_id: String,
    amount_cents: u64,
    success: bool,
}

#[derive(Debug, Clone)]
struct InventoryReserved {
    order_id: String,
    items: Vec<String>,
}

// --- Concrete Handlers ---

struct AuditLogger;

impl EventHandler<OrderCreated> for AuditLogger {
    fn handle(&self, event: &OrderCreated) {
        println!(
            "[Audit] Order {} created by customer {} — ${}",
            event.order_id,
            event.customer_id,
            event.total_cents as f64 / 100.0
        );
    }

    fn clone_handler(&self) -> Self {
        AuditLogger
    }
}

impl EventHandler<PaymentProcessed> for AuditLogger {
    fn handle(&self, event: &PaymentProcessed) {
        println!(
            "[Audit] Payment {} for order {} — {} (${})",
            event.payment_id,
            event.order_id,
            if event.success { "SUCCESS" } else { "FAILED" },
            event.amount_cents as f64 / 100.0
        );
    }

    fn clone_handler(&self) -> Self {
        AuditLogger
    }
}

struct MetricsCollector {
    /// Track order totals for reporting.
    /// Uses interior mutability since handle() takes &self.
    totals: RefCell<HashMap<String, u64>>,
}

impl MetricsCollector {
    fn new() -> Self {
        MetricsCollector {
            totals: RefCell::new(HashMap::new()),
        }
    }
}

impl EventHandler<OrderCreated> for MetricsCollector {
    fn handle(&self, event: &OrderCreated) {
        let mut totals = self.totals.borrow_mut();
        *totals.entry(event.customer_id.clone()).or_insert(0) += event.total_cents;
        println!(
            "[Metrics] Customer {} lifetime total: ${}",
            event.customer_id,
            *totals.get(&event.customer_id).unwrap() as f64 / 100.0
        );
    }

    fn clone_handler(&self) -> Self
    where
        Self: Sized,
    {
        MetricsCollector {
            totals: RefCell::new(self.totals.borrow().clone()),
        }
    }
}

struct FraudDetector {
    threshold_cents: u64,
}

impl FraudDetector {
    fn new(threshold_cents: u64) -> Self {
        FraudDetector { threshold_cents }
    }
}

impl EventHandler<OrderCreated> for FraudDetector {
    fn handle(&self, event: &OrderCreated) {
        if event.total_cents > self.threshold_cents {
            println!(
                "[FRAUD] ALERT: Order {} exceeds threshold (${} > ${})",
                event.order_id,
                event.total_cents as f64 / 100.0,
                self.threshold_cents as f64 / 100.0
            );
        }
    }

    fn clone_handler(&self) -> Self {
        FraudDetector {
            threshold_cents: self.threshold_cents,
        }
    }
}

// --- Main ---

fn main() {
    let mut bus = EventBus::new();

    // Register closure-based handlers
    bus.on::<OrderCreated, _>(|event| {
        println!("[Handler] New order: {} for ${}", event.order_id, event.total_cents as f64 / 100.0);
    });

    bus.on::<PaymentProcessed, _>(|event| {
        println!(
            "[Handler] Payment {}: {}",
            event.payment_id,
            if event.success { "confirmed" } else { "declined" }
        );
    });

    bus.on::<OrderCreated, _>(|event| {
        if event.total_cents > 10000 {
            println!("[Handler] Large order alert: {}", event.order_id);
        }
    });

    println!("Order handlers: {}", bus.handler_count::<OrderCreated>());
    println!("Payment handlers: {}", bus.handler_count::<PaymentProcessed>());
    println!("Inventory handlers: {}\n", bus.handler_count::<InventoryReserved>());

    // Emit events
    bus.emit(&OrderCreated {
        order_id: "ORD-001".into(),
        customer_id: "CUST-42".into(),
        total_cents: 4999,
    });

    println!();

    bus.emit(&OrderCreated {
        order_id: "ORD-002".into(),
        customer_id: "CUST-42".into(),
        total_cents: 25000,
    });

    println!();

    bus.emit(&PaymentProcessed {
        order_id: "ORD-001".into(),
        payment_id: "PAY-100".into(),
        amount_cents: 4999,
        success: true,
    });

    println!();

    bus.emit(&PaymentProcessed {
        order_id: "ORD-002".into(),
        payment_id: "PAY-101".into(),
        amount_cents: 25000,
        success: false,
    });

    // Demonstrate the shared registry (cross-service)
    let _shared = bus.share_registry();
    println!("\nShared registry cloned (Rc count: 2)");
}
