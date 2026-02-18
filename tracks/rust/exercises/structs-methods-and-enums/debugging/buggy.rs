// Event Processor — Debugging Exercise (Rust)
//
// This code has 4 bugs related to structs, enums, and methods.
// Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

// ---------- Types ----------

// BUG 1 is hidden in this derive list.
#[derive(Debug)]
struct EventRecord {
    id: u64,
    kind: String,
    payload: String,
    retries: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum EventOutcome {
    Accepted { record_id: u64 },
    Rejected { record_id: u64, reason: String },
    Deferred { record_id: u64, delay_ms: u64 },
    // BUG 4: there is a missing variant here that the tests expect.
    // Hint: look at what test_process_poison_pill checks for.
}

// ---------- Implementation ----------

impl EventRecord {
    fn new(id: u64, kind: &str, payload: &str) -> EventRecord {
        EventRecord {
            id,
            kind: kind.to_string(),
            payload: payload.to_string(),
            retries: 0,
        }
    }

    // BUG 2: This method signature is wrong.
    // It consumes self when it should only borrow it.
    // After calling is_retriable(), the caller cannot use the record anymore.
    fn is_retriable(self) -> bool {
        self.retries < 3
    }

    fn increment_retries(&mut self) {
        self.retries += 1;
    }

    // BUG 3 is inside this function body.
    fn classify(&self) -> &str {
        match self.kind.as_str() {
            "user.created" => "identity",
            "user.deleted" => "identity",
            "order.placed" => "commerce",
            "order.cancelled" => "commerce",
            "payment.failed" => "commerce",
            // Something is wrong here — this arm is unreachable and
            // the pattern below it is never evaluated.
            _ => "unknown",
            "health.ping" => "system",
        }
    }
}

/// Processes a batch of records. Returns one outcome per record.
fn process_batch(records: &[EventRecord]) -> Vec<EventOutcome> {
    let mut outcomes = Vec::new();

    for record in records {
        let outcome = process_one(record);
        outcomes.push(outcome);
    }

    outcomes
}

fn process_one(record: &EventRecord) -> EventOutcome {
    // Simulate: poison pills are permanently rejected,
    // unhealthy records are deferred, rest are accepted.
    if record.payload == "POISON" {
        return EventOutcome::Rejected {
            record_id: record.id,
            reason: String::from("poison pill rejected"),
        };
    }

    if record.payload.contains("CORRUPTED") {
        return EventOutcome::Deferred {
            record_id: record.id,
            delay_ms: 5000,
        };
    }

    EventOutcome::Accepted { record_id: record.id }
}

/// Extracts the record_id from any outcome variant.
/// BUG 3 causes incorrect behavior here but won't stop compilation.
fn outcome_record_id(outcome: &EventOutcome) -> u64 {
    match outcome {
        EventOutcome::Accepted { record_id } => *record_id,
        EventOutcome::Rejected { record_id, .. } => *record_id,
        EventOutcome::Deferred { record_id, .. } => *record_id,
        // BUG 4: when the missing variant is added, this match must also handle it.
    }
}

/// Splits outcomes into (accepted, rejected, deferred) groups.
fn partition_outcomes(outcomes: Vec<EventOutcome>) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut deferred = Vec::new();

    for outcome in outcomes {
        match outcome {
            EventOutcome::Accepted { record_id } => accepted.push(record_id),
            EventOutcome::Rejected { record_id, .. } => rejected.push(record_id),
            EventOutcome::Deferred { record_id, .. } => deferred.push(record_id),
            // BUG 4: same — must handle the missing variant once added.
        }
    }

    (accepted, rejected, deferred)
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- is_retriable --

    #[test]
    fn test_is_retriable_fresh_record() {
        let record = EventRecord::new(1, "user.created", "{}");
        // BUG 1: This test requires Clone to work.
        // The test checks is_retriable AND uses record after the call.
        // Without Clone, we can't duplicate the record for the is_retriable call.
        assert!(record.is_retriable());
        // If is_retriable consumes self, this line will not compile:
        assert_eq!(record.retries, 0);
    }

    #[test]
    fn test_is_retriable_exhausted() {
        let mut record = EventRecord::new(2, "order.placed", "{}");
        record.retries = 3;

        // Same ownership issue: both lines need record to still be valid.
        assert!(!record.is_retriable());
        assert_eq!(record.retries, 3);
    }

    #[test]
    fn test_increment_retries() {
        let mut record = EventRecord::new(3, "payment.failed", "{}");
        record.increment_retries();
        record.increment_retries();
        assert_eq!(record.retries, 2);
        assert!(record.is_retriable());
    }

    // -- classify --

    #[test]
    fn test_classify_identity_events() {
        let r = EventRecord::new(1, "user.created", "{}");
        assert_eq!(r.classify(), "identity");
        let r2 = EventRecord::new(2, "user.deleted", "{}");
        assert_eq!(r2.classify(), "identity");
    }

    #[test]
    fn test_classify_commerce_events() {
        let r = EventRecord::new(1, "order.placed", "{}");
        assert_eq!(r.classify(), "commerce");
        let r2 = EventRecord::new(2, "payment.failed", "{}");
        assert_eq!(r2.classify(), "commerce");
    }

    #[test]
    fn test_classify_system_events() {
        // BUG 3: This test currently returns "unknown" instead of "system"
        // because the wildcard arm shadows the "health.ping" arm.
        let r = EventRecord::new(1, "health.ping", "{}");
        assert_eq!(r.classify(), "system");
    }

    #[test]
    fn test_classify_unknown() {
        let r = EventRecord::new(1, "something.else", "{}");
        assert_eq!(r.classify(), "unknown");
    }

    // -- process_batch --

    #[test]
    fn test_process_batch_mixed() {
        let records = vec![
            EventRecord::new(10, "user.created", "{}"),
            EventRecord::new(11, "order.placed", "CORRUPTED_DATA"),
            EventRecord::new(12, "payment.failed", "{}"),
        ];

        let outcomes = process_batch(&records);
        assert_eq!(outcomes.len(), 3);

        assert_eq!(outcomes[0], EventOutcome::Accepted { record_id: 10 });
        assert!(matches!(outcomes[1], EventOutcome::Deferred { record_id: 11, .. }));
        assert_eq!(outcomes[2], EventOutcome::Accepted { record_id: 12 });
    }

    // -- outcome_record_id --

    #[test]
    fn test_outcome_record_id() {
        assert_eq!(outcome_record_id(&EventOutcome::Accepted { record_id: 42 }), 42);
        assert_eq!(outcome_record_id(&EventOutcome::Rejected {
            record_id: 7,
            reason: String::from("bad"),
        }), 7);
        assert_eq!(outcome_record_id(&EventOutcome::Deferred {
            record_id: 99,
            delay_ms: 1000,
        }), 99);
    }

    // -- poison pill (BUG 4: requires the missing variant) --

    #[test]
    fn test_process_poison_pill() {
        let record = EventRecord::new(99, "internal.control", "POISON");
        let outcome = process_one(&record);

        // This test expects a Rejected outcome — but poison pills should
        // eventually be tracked separately from normal rejections.
        // Add a PoisonPill variant to EventOutcome and update process_one
        // to return it. Then update outcome_record_id and partition_outcomes.
        //
        // The variant should carry: record_id: u64
        assert!(matches!(outcome, EventOutcome::PoisonPill { record_id: 99 }));
    }

    // -- partition --

    #[test]
    fn test_partition_outcomes() {
        let outcomes = vec![
            EventOutcome::Accepted { record_id: 1 },
            EventOutcome::Accepted { record_id: 2 },
            EventOutcome::Rejected { record_id: 3, reason: String::from("bad data") },
            EventOutcome::Deferred { record_id: 4, delay_ms: 2000 },
            EventOutcome::Accepted { record_id: 5 },
        ];

        let (accepted, rejected, deferred) = partition_outcomes(outcomes);

        assert_eq!(accepted.len(), 3);
        assert_eq!(rejected.len(), 1);
        assert_eq!(deferred.len(), 1);
        assert!(accepted.contains(&1));
        assert!(accepted.contains(&2));
        assert!(accepted.contains(&5));
        assert!(rejected.contains(&3));
        assert!(deferred.contains(&4));
    }
}

fn main() {
    println!("Event Processor — run with: rustc --test buggy.rs && ./buggy");
}
