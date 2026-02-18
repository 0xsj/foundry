// Job Scheduler — Debugging Exercise (Rust)
//
// This code compiles and runs, but produces incorrect results.
// Find and fix the 4 bugs.
//
// Run tests: rustc --test buggy.rs && ./buggy

#[derive(Debug, Clone, PartialEq)]
enum JobType {
    DataSync,
    ReportGeneration,
    EmailDispatch,
    Cleanup,
}

#[derive(Debug, Clone)]
struct Job {
    id: u32,
    job_type: JobType,
    priority: u8,   // 1-10
    retry_count: u32,
    payload: String,
}

impl Job {
    fn new(id: u32, job_type: JobType, priority: u8, payload: &str) -> Job {
        Job {
            id,
            job_type,
            priority,
            retry_count: 0,
            payload: payload.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum JobOutcome {
    Processed,
    Skipped,
    Cancelled,
    Failed(String),
}

// ---------- Bug 1: Unreachable match arm (wrong arm order) ----------

/// Routes a job to a queue name based on priority.
/// Priority 8-10: "urgent-queue"
/// Priority 5-7:  "standard-queue"
/// Priority 1-4:  "low-queue"
/// Otherwise:     "invalid"
fn route_job(job: &Job) -> &'static str {
    // The author put the catch-all range before the specific range.
    // The compiler warning was silenced with #[allow(unreachable_patterns)].
    #[allow(unreachable_patterns)]
    match job.priority {
        1..=10 => "standard-queue",
        8..=10 => "urgent-queue",    // BUG: unreachable — 1..=10 matches first
        5..=7 => "standard-queue",
        1..=4 => "low-queue",
        _ => "invalid",
    }
}

// ---------- Bug 2: schedule_retry doesn't mutate the job ----------

/// Schedules a retry for the given job.
/// Increments retry_count and returns the new count.
/// Returns None if the job has already been retried 3 or more times.
fn schedule_retry(job: &mut Job) -> Option<u32> {
    // BUG: clones the job to increment, then checks the CLONE, but never
    // writes back to the original. job.retry_count is never changed.
    let mut job_copy = job.clone();
    job_copy.retry_count += 1;

    if job_copy.retry_count >= 3 {
        None
    } else {
        Some(job_copy.retry_count)
    }
}

// ---------- Bug 3: drain_queue loops forever ----------

/// Drains a job queue, processing each job.
/// Returns (processed_count, skipped_count).
fn drain_queue(mut queue: Vec<Job>) -> (u32, u32) {
    let mut processed = 0u32;
    let mut skipped = 0u32;

    // BUG: the loop condition never becomes false.
    // `queue` is never modified inside the loop — items are read but not removed.
    while !queue.is_empty() {
        let job = queue.last().unwrap(); // peek at last item — does not remove it

        if job.priority == 0 {
            skipped += 1;
        } else {
            processed += 1;
        }
        // Items are never removed — queue.is_empty() is always false
    }

    (processed, skipped)
}

// ---------- Bug 4: Cancelled counted as Processed ----------

/// Counts outcomes from a list of job results.
/// Returns (processed, skipped, cancelled, failed).
fn count_outcomes(outcomes: &[JobOutcome]) -> (u32, u32, u32, u32) {
    let mut processed = 0u32;
    let mut skipped = 0u32;
    let mut cancelled = 0u32;
    let mut failed = 0u32;

    for outcome in outcomes {
        match outcome {
            JobOutcome::Processed => processed += 1,
            JobOutcome::Skipped => skipped += 1,
            // BUG: Cancelled is matched by the _ wildcard BEFORE reaching
            // the explicit Cancelled arm, because the author accidentally
            // put the wildcard arm before the Cancelled arm.
            _ => processed += 1,  // BUG: this matches Cancelled (and Failed)
            JobOutcome::Cancelled => cancelled += 1,  // unreachable
            JobOutcome::Failed(_) => failed += 1,     // unreachable
        }
    }

    (processed, skipped, cancelled, failed)
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Bug 1: route_job --

    #[test]
    fn test_route_urgent_high_priority() {
        let job = Job::new(1, JobType::DataSync, 9, "payload");
        assert_eq!(route_job(&job), "urgent-queue",
            "priority 9 should go to urgent-queue");
    }

    #[test]
    fn test_route_urgent_boundary() {
        let job = Job::new(2, JobType::DataSync, 8, "payload");
        assert_eq!(route_job(&job), "urgent-queue",
            "priority 8 (boundary) should go to urgent-queue");
    }

    #[test]
    fn test_route_standard() {
        let job = Job::new(3, JobType::EmailDispatch, 6, "payload");
        assert_eq!(route_job(&job), "standard-queue");
    }

    #[test]
    fn test_route_low() {
        let job = Job::new(4, JobType::Cleanup, 3, "payload");
        assert_eq!(route_job(&job), "low-queue");
    }

    // -- Bug 2: schedule_retry --

    #[test]
    fn test_schedule_retry_increments() {
        let mut job = Job::new(1, JobType::DataSync, 5, "data");

        // Each call should increment retry_count on the actual job
        schedule_retry(&mut job);
        assert_eq!(job.retry_count, 1, "retry_count should be 1 after first retry");

        schedule_retry(&mut job);
        assert_eq!(job.retry_count, 2, "retry_count should be 2 after second retry");
    }

    #[test]
    fn test_schedule_retry_max() {
        let mut job = Job::new(2, JobType::ReportGeneration, 4, "report");
        job.retry_count = 2;

        let result = schedule_retry(&mut job);
        assert!(result.is_none(), "should return None when retry_count reaches 3");
        assert_eq!(job.retry_count, 3, "retry_count should still be incremented to 3");
    }

    #[test]
    fn test_schedule_retry_returns_new_count() {
        let mut job = Job::new(3, JobType::EmailDispatch, 7, "email");
        let result = schedule_retry(&mut job);
        assert_eq!(result, Some(1), "should return Some(1) on first retry");
    }

    // -- Bug 3: drain_queue --

    #[test]
    fn test_drain_queue() {
        let queue = vec![
            Job::new(1, JobType::DataSync, 5, "a"),
            Job::new(2, JobType::Cleanup, 3, "b"),
            Job::new(3, JobType::EmailDispatch, 7, "c"),
        ];
        let (processed, skipped) = drain_queue(queue);
        assert_eq!(processed, 3, "all 3 jobs should be processed");
        assert_eq!(skipped, 0);
    }

    #[test]
    fn test_drain_queue_empty() {
        let (processed, skipped) = drain_queue(vec![]);
        assert_eq!(processed, 0);
        assert_eq!(skipped, 0);
    }

    // -- Bug 4: count_outcomes --

    #[test]
    fn test_count_outcomes() {
        let outcomes = vec![
            JobOutcome::Processed,
            JobOutcome::Processed,
            JobOutcome::Skipped,
            JobOutcome::Cancelled,
            JobOutcome::Cancelled,
            JobOutcome::Failed("timeout".to_string()),
        ];

        let (processed, skipped, cancelled, failed) = count_outcomes(&outcomes);
        assert_eq!(processed, 2, "processed should be 2");
        assert_eq!(skipped, 1, "skipped should be 1");
        assert_eq!(cancelled, 2, "cancelled should be 2, not counted as processed");
        assert_eq!(failed, 1, "failed should be 1");
    }

    #[test]
    fn test_count_outcomes_all_processed() {
        let outcomes = vec![
            JobOutcome::Processed,
            JobOutcome::Processed,
            JobOutcome::Processed,
        ];
        let (processed, skipped, cancelled, failed) = count_outcomes(&outcomes);
        assert_eq!(processed, 3);
        assert_eq!(skipped, 0);
        assert_eq!(cancelled, 0);
        assert_eq!(failed, 0);
    }
}

fn main() {
    // Demo the bugs
    let job = Job::new(1, JobType::DataSync, 9, "sync all tables");
    println!("route_job(priority=9) = {} (expected: urgent-queue)", route_job(&job));

    let mut retry_job = Job::new(2, JobType::EmailDispatch, 5, "welcome email");
    schedule_retry(&mut retry_job);
    println!("retry_count after 1 retry = {} (expected: 1)", retry_job.retry_count);

    let queue = vec![
        Job::new(3, JobType::Cleanup, 3, "a"),
        Job::new(4, JobType::ReportGeneration, 7, "b"),
    ];
    println!("drain_queue: starting (this will hang if Bug 3 is not fixed)");
    // Uncomment after fixing Bug 3:
    // let (p, s) = drain_queue(queue);
    // println!("drain_queue: processed={}, skipped={}", p, s);
    drop(queue);

    let outcomes = vec![
        JobOutcome::Processed,
        JobOutcome::Cancelled,
        JobOutcome::Failed("err".to_string()),
    ];
    let (p, s, c, f) = count_outcomes(&outcomes);
    println!("count_outcomes: processed={} skipped={} cancelled={} failed={}", p, s, c, f);
    println!("  (expected: processed=1 skipped=0 cancelled=1 failed=1)");
}
