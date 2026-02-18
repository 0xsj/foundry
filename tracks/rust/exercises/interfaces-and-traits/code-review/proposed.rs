// Job Queue Storage Abstraction — Proposed Code for Review
//
// A storage abstraction for a distributed job queue.
// Supports in-memory (tests/local) and durable (production) backends.
//
// Run: rustc proposed.rs && ./proposed

use std::collections::VecDeque;
use std::fmt;

// ----- Job types -----

/// A job in the queue.
// ISSUE: Missing several useful derives. What should be added?
#[derive(Debug)]
pub struct Job {
    pub id: u64,
    pub kind: String,
    pub payload: String,
    pub attempts: u32,
}

impl Job {
    pub fn new(id: u64, kind: &str, payload: &str) -> Self {
        Job {
            id,
            kind: kind.to_string(),
            payload: payload.to_string(),
            attempts: 0,
        }
    }
}

/// The outcome of processing a job.
// ISSUE: Missing derives. What should be added to make this useful in tests?
#[derive(Debug)]
pub enum JobResult {
    Success,
    Failure { reason: String },
    Retry { delay_ms: u64 },
}

/// A storage error.
#[derive(Debug)]
pub struct StorageError {
    pub message: String,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "storage error: {}", self.message)
    }
}

// ----- Storage trait -----

// ISSUE: This trait does too much. It mixes concerns:
//   1. Basic queue operations (push, pop, len)
//   2. Dead-letter queue management (move_to_dlq, list_dlq, clear_dlq)
//   3. Health and admin operations (health_check, clear_all)
//
// A caller who only needs to push/pop jobs must implement ALL of these methods,
// including the DLQ and health methods they may not need.
//
// What would a better design look like?
pub trait JobStorage {
    // The item type stored in this storage. Fixed as Job.
    // ISSUE: This associated type unnecessarily limits flexibility.
    // What if a storage implementation wanted to store something that wraps Job?
    // What design gives the caller more control over the item type?
    type Item;

    // Core queue operations
    fn push(&mut self, job: Self::Item) -> Result<(), StorageError>;
    fn pop(&mut self) -> Result<Option<Self::Item>, StorageError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // Dead-letter queue operations (should this be a separate trait?)
    fn move_to_dlq(&mut self, job: Self::Item, reason: &str) -> Result<(), StorageError>;
    fn list_dlq(&self) -> &[Self::Item];
    fn clear_dlq(&mut self) -> Result<(), StorageError>;

    // Health and admin
    fn health_check(&self) -> bool;
    fn clear_all(&mut self) -> Result<(), StorageError>;
}

// ----- In-memory implementation -----

/// An in-memory job queue for testing and local development.
pub struct MemoryStorage {
    queue: VecDeque<Job>,
    dlq: Vec<Job>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            queue: VecDeque::new(),
            dlq: Vec::new(),
        }
    }
}

impl JobStorage for MemoryStorage {
    type Item = Job;

    fn push(&mut self, job: Job) -> Result<(), StorageError> {
        self.queue.push_back(job);
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<Job>, StorageError> {
        Ok(self.queue.pop_front())
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn move_to_dlq(&mut self, job: Job, _reason: &str) -> Result<(), StorageError> {
        self.dlq.push(job);
        Ok(())
    }

    fn list_dlq(&self) -> &[Job] {
        &self.dlq
    }

    fn clear_dlq(&mut self) -> Result<(), StorageError> {
        self.dlq.clear();
        Ok(())
    }

    fn health_check(&self) -> bool {
        true
    }

    fn clear_all(&mut self) -> Result<(), StorageError> {
        self.queue.clear();
        self.dlq.clear();
        Ok(())
    }
}

// ----- Worker -----

/// A worker that pulls jobs from storage and processes them.
pub struct Worker {
    // ISSUE: This field uses Box<dyn JobStorage<Item = Job>> for dynamic dispatch.
    // But the Worker is always constructed with a concrete type, and the tests
    // never swap the storage at runtime. Is dynamic dispatch the right choice here?
    // When would Box<dyn JobStorage<...>> actually be necessary?
    storage: Box<dyn JobStorage<Item = Job>>,
    processed_count: u64,
}

impl Worker {
    pub fn new(storage: Box<dyn JobStorage<Item = Job>>) -> Self {
        Worker {
            storage,
            processed_count: 0,
        }
    }

    pub fn run_one(&mut self) -> Result<Option<JobResult>, StorageError> {
        match self.storage.pop()? {
            None => Ok(None),
            Some(mut job) => {
                job.attempts += 1;
                self.processed_count += 1;

                // Simulate processing: fail jobs with "fail" in payload
                let result = if job.payload.contains("fail") {
                    JobResult::Failure { reason: String::from("simulated failure") }
                } else if job.payload.contains("retry") {
                    JobResult::Retry { delay_ms: 1000 }
                } else {
                    JobResult::Success
                };

                // Move failures to DLQ after max attempts
                if matches!(&result, JobResult::Failure { .. }) && job.attempts >= 3 {
                    self.storage.move_to_dlq(job, "max attempts reached")?;
                }

                Ok(Some(result))
            }
        }
    }

    pub fn storage_len(&self) -> usize {
        self.storage.len()
    }

    pub fn processed_count(&self) -> u64 {
        self.processed_count
    }
}

// ----- Helper function -----

/// Log each job in the DLQ.
// ISSUE: This function uses &dyn JobStorage<Item = Job> for a single, non-polymorphic
// use case. The function is only ever called with MemoryStorage. Is dyn necessary here?
// What would impl JobStorage<Item = Job> give you instead?
pub fn log_dlq_contents(storage: &dyn JobStorage<Item = Job>) {
    let items = storage.list_dlq();
    if items.is_empty() {
        println!("DLQ is empty");
        return;
    }
    println!("DLQ contains {} jobs:", items.len());
    for job in items {
        println!("  {:?}", job);
    }
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut store = MemoryStorage::new();
        store.push(Job::new(1, "email", "send-welcome")).unwrap();
        store.push(Job::new(2, "email", "send-receipt")).unwrap();

        assert_eq!(store.len(), 2);
        let job = store.pop().unwrap().unwrap();
        assert_eq!(job.id, 1);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut store = MemoryStorage::new();
        assert!(store.is_empty());
        store.push(Job::new(1, "email", "body")).unwrap();
        assert!(!store.is_empty());
    }

    #[test]
    fn test_worker_processes_job() {
        let mut store = MemoryStorage::new();
        store.push(Job::new(1, "report", "generate-monthly")).unwrap();

        let mut worker = Worker::new(Box::new(store));
        let result = worker.run_one().unwrap();
        assert!(matches!(result, Some(JobResult::Success)));
        assert_eq!(worker.processed_count(), 1);
    }

    #[test]
    fn test_worker_empty_queue() {
        let store = MemoryStorage::new();
        let mut worker = Worker::new(Box::new(store));
        let result = worker.run_one().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_dlq_logging_does_not_panic() {
        let mut store = MemoryStorage::new();
        store.move_to_dlq(Job::new(99, "cleanup", "fail"), "test").unwrap();
        log_dlq_contents(&store);
    }
}

fn main() {
    let mut storage = MemoryStorage::new();

    storage.push(Job::new(1, "email", "send-welcome")).unwrap();
    storage.push(Job::new(2, "report", "generate-monthly")).unwrap();
    storage.push(Job::new(3, "cleanup", "fail")).unwrap();

    println!("Queue length: {}", storage.len());
    println!("Health: {}", storage.health_check());

    let mut worker = Worker::new(Box::new(storage));

    for _ in 0..3 {
        match worker.run_one() {
            Ok(Some(result)) => println!("Processed: {:?}", result),
            Ok(None) => println!("Queue empty"),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Total processed: {}", worker.processed_count());
}
