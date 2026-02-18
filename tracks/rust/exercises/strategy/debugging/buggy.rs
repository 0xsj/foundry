// Task Executor with Pluggable Strategies — BUGGY VERSION
//
// This file contains FOUR bugs related to Rust's trait system and
// strategy pattern. Fix them all so the code compiles and runs.
//
// DO NOT rewrite the architecture. Fix the bugs in place.
// The main() function should produce output demonstrating all three strategies.

use std::fmt;

// ---------------------------------------------------------------------------
// Task definition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Task {
    id: String,
    priority: u32,   // higher = more important
    payload: String,
}

impl Task {
    fn new(id: &str, priority: u32, payload: &str) -> Self {
        Self {
            id: id.to_string(),
            priority,
            payload: payload.to_string(),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task(id={}, pri={}, payload={})", self.id, self.priority, self.payload)
    }
}

// ---------------------------------------------------------------------------
// BUG 1: Object safety violation
//
// This trait has a generic method `execute_typed<T>`. Generic methods make a
// trait NOT object safe, meaning you cannot use `dyn ExecutionStrategy`.
// The developer added it thinking "it might be useful for typed tasks later"
// but it prevents the entire trait from being used as a trait object.
// ---------------------------------------------------------------------------

trait ExecutionStrategy {
    /// Execute a batch of tasks according to this strategy.
    fn execute(&mut self, tasks: &[Task]) -> Vec<String>;

    /// Strategy name for logging.
    fn name(&self) -> &str;

    /// Execute a single typed task — BUG: makes trait not object safe
    fn execute_typed<T: fmt::Debug>(&self, task: T) -> String {
        format!("executed: {:?}", task)
    }
}

// ---------------------------------------------------------------------------
// Concrete strategies
// ---------------------------------------------------------------------------

struct SequentialExecutor;

impl ExecutionStrategy for SequentialExecutor {
    fn execute(&mut self, tasks: &[Task]) -> Vec<String> {
        let mut results = Vec::new();
        for task in tasks {
            println!("  [sequential] executing {}", task);
            results.push(format!("done:{}", task.id));
        }
        results
    }

    fn name(&self) -> &str {
        "sequential"
    }
}

struct PriorityExecutor;

impl ExecutionStrategy for PriorityExecutor {
    fn execute(&mut self, tasks: &[Task]) -> Vec<String> {
        let mut sorted: Vec<&Task> = tasks.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority)); // highest first

        let mut results = Vec::new();
        for task in sorted {
            println!("  [priority] executing {} (pri={})", task.id, task.priority);
            results.push(format!("done:{}", task.id));
        }
        results
    }

    fn name(&self) -> &str {
        "priority"
    }
}

struct BatchExecutor {
    batch_size: usize,
}

impl BatchExecutor {
    fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
}

impl ExecutionStrategy for BatchExecutor {
    fn execute(&mut self, tasks: &[Task]) -> Vec<String> {
        let mut results = Vec::new();
        for (i, chunk) in tasks.chunks(self.batch_size).enumerate() {
            println!("  [batch] processing batch {} ({} tasks)", i + 1, chunk.len());
            for task in chunk {
                println!("    executing {}", task);
                results.push(format!("done:{}", task.id));
            }
        }
        results
    }

    fn name(&self) -> &str {
        "batch"
    }
}

// ---------------------------------------------------------------------------
// BUG 2: Lifetime issue
//
// The TaskRunner stores a BORROWED reference to a strategy. But the way it's
// constructed in main(), the strategy is created as a temporary that doesn't
// live long enough. The reference becomes dangling.
// ---------------------------------------------------------------------------

struct TaskRunner<'a> {
    strategy: &'a mut dyn ExecutionStrategy,
    history: Vec<String>,
}

impl<'a> TaskRunner<'a> {
    fn new(strategy: &'a mut dyn ExecutionStrategy) -> Self {
        Self {
            strategy,
            history: Vec::new(),
        }
    }

    fn run(&mut self, tasks: &[Task]) -> Vec<String> {
        println!("Running with strategy: {}", self.strategy.name());
        let results = self.strategy.execute(tasks);
        self.history.extend(results.clone());
        results
    }

    fn history(&self) -> &[String] {
        &self.history
    }
}

// ---------------------------------------------------------------------------
// BUG 3: Missing Send bound
//
// The AsyncTaskQueue tries to send a trait object across threads, but
// `dyn ExecutionStrategy` doesn't have a `Send` bound. Rust requires
// explicit opt-in for thread safety on trait objects.
// ---------------------------------------------------------------------------

struct AsyncTaskQueue {
    strategy: Box<dyn ExecutionStrategy>,
    pending: Vec<Task>,
}

impl AsyncTaskQueue {
    fn new(strategy: Box<dyn ExecutionStrategy>) -> Self {
        Self {
            strategy,
            pending: Vec::new(),
        }
    }

    fn enqueue(&mut self, task: Task) {
        self.pending.push(task);
    }

    fn process_in_thread(&mut self) -> std::thread::JoinHandle<Vec<String>> {
        let mut strategy = std::mem::replace(
            &mut self.strategy,
            Box::new(SequentialExecutor) as Box<dyn ExecutionStrategy>,
        );
        let tasks: Vec<Task> = self.pending.drain(..).collect();

        // This requires the strategy to be Send
        std::thread::spawn(move || {
            strategy.execute(&tasks)
        })
    }
}

// ---------------------------------------------------------------------------
// BUG 4: Wrong dispatch — static when dynamic is needed
//
// The function `run_with_strategy` uses a generic parameter, which means
// the strategy type is fixed at compile time. But in main(), we want to
// select from a Vec of different strategies at runtime. A generic function
// can't accept heterogeneous strategy types from a collection.
// ---------------------------------------------------------------------------

fn run_with_strategy<S: ExecutionStrategy>(strategy: &mut S, tasks: &[Task]) -> Vec<String> {
    println!("\n--- Running with {} ---", strategy.name());
    strategy.execute(tasks)
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let tasks = vec![
        Task::new("deploy-api", 3, "deploy v2.1.0"),
        Task::new("run-migrations", 5, "migrate users table"),
        Task::new("clear-cache", 1, "invalidate CDN"),
        Task::new("notify-team", 2, "send Slack notification"),
        Task::new("run-tests", 4, "integration test suite"),
    ];

    // --- Demo 1: TaskRunner with borrowed strategy (BUG 2) ---
    println!("=== Demo 1: TaskRunner ===");
    // BUG 2: SequentialExecutor is created as a temporary — the reference
    // in TaskRunner outlives it.
    let mut runner = TaskRunner::new(&mut SequentialExecutor);
    runner.run(&tasks);
    println!("History: {:?}\n", runner.history());

    // --- Demo 2: AsyncTaskQueue (BUG 3) ---
    println!("=== Demo 2: Async Queue ===");
    let mut queue = AsyncTaskQueue::new(Box::new(PriorityExecutor));
    for task in &tasks {
        queue.enqueue(task.clone());
    }
    let handle = queue.process_in_thread();
    let results = handle.join().expect("thread panicked");
    println!("Async results: {:?}\n", results);

    // --- Demo 3: Runtime strategy selection (BUG 4) ---
    println!("=== Demo 3: Runtime Strategy Selection ===");
    let mut strategies: Vec<Box<dyn ExecutionStrategy>> = vec![
        Box::new(SequentialExecutor),
        Box::new(PriorityExecutor),
        Box::new(BatchExecutor::new(2)),
    ];

    for strategy in &mut strategies {
        // BUG 4: run_with_strategy is generic — it can't accept &mut Box<dyn ExecutionStrategy>
        // because Box<dyn ExecutionStrategy> doesn't implement ExecutionStrategy directly.
        // We need dynamic dispatch here, not static.
        let results = run_with_strategy(strategy.as_mut(), &tasks);
        println!("Results: {:?}\n", results);
    }
}
