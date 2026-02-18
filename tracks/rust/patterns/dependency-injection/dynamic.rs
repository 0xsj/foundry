// Dependency Injection via Trait Objects (Dynamic Dispatch)
//
// Same UserService concept as service.rs, but using Box<dyn Trait>
// instead of generics. Dependencies are resolved at runtime via vtable.
//
// Compare: service.rs (generics) vs this file (trait objects)
//
// Run: rustc dynamic.rs && ./dynamic

use std::collections::HashMap;
use std::cell::RefCell;

// ---- Dependency Traits ----
// Same traits as service.rs -- the contract is identical

trait UserRepository {
    fn find_by_id(&self, id: u64) -> Option<String>;
    fn save(&self, id: u64, name: &str) -> Result<(), String>;
}

trait NotificationSender {
    fn send(&self, to: &str, message: &str) -> Result<(), String>;
    fn channel_name(&self) -> &str;
}

// ---- Multiple Implementations ----

struct InMemoryRepo {
    users: RefCell<HashMap<u64, String>>,
}

impl InMemoryRepo {
    fn new(seed: Vec<(u64, &str)>) -> Self {
        let mut users = HashMap::new();
        for (id, name) in seed {
            users.insert(id, name.to_string());
        }
        InMemoryRepo { users: RefCell::new(users) }
    }
}

impl UserRepository for InMemoryRepo {
    fn find_by_id(&self, id: u64) -> Option<String> {
        self.users.borrow().get(&id).cloned()
    }

    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        self.users.borrow_mut().insert(id, name.to_string());
        Ok(())
    }
}

struct FileRepo {
    path: String,
}

impl FileRepo {
    fn new(path: &str) -> Self {
        FileRepo { path: path.to_string() }
    }
}

impl UserRepository for FileRepo {
    fn find_by_id(&self, id: u64) -> Option<String> {
        // Simulated file-based lookup
        println!("    (FileRepo: would read {}/user_{}.json)", self.path, id);
        Some(format!("file_user_{}", id))
    }

    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        println!("    (FileRepo: would write {}/user_{}.json = {})", self.path, id, name);
        Ok(())
    }
}

struct EmailNotifier {
    smtp_host: String,
}

impl EmailNotifier {
    fn new(host: &str) -> Self {
        EmailNotifier { smtp_host: host.to_string() }
    }
}

impl NotificationSender for EmailNotifier {
    fn send(&self, to: &str, message: &str) -> Result<(), String> {
        println!("  [EMAIL via {}] To: {} -- {}", self.smtp_host, to, message);
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "email"
    }
}

struct WebhookNotifier {
    endpoint: String,
}

impl WebhookNotifier {
    fn new(endpoint: &str) -> Self {
        WebhookNotifier { endpoint: endpoint.to_string() }
    }
}

impl NotificationSender for WebhookNotifier {
    fn send(&self, to: &str, message: &str) -> Result<(), String> {
        println!("  [WEBHOOK POST {}] Recipient: {} -- {}", self.endpoint, to, message);
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "webhook"
    }
}

struct SlackNotifier {
    channel: String,
}

impl SlackNotifier {
    fn new(channel: &str) -> Self {
        SlackNotifier { channel: channel.to_string() }
    }
}

impl NotificationSender for SlackNotifier {
    fn send(&self, to: &str, message: &str) -> Result<(), String> {
        println!("  [SLACK #{}] @{}: {}", self.channel, to, message);
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "slack"
    }
}

// ---- The Service: Using Trait Objects ----
// No generic parameters -- concrete struct type.
// Dependencies are trait objects (Box<dyn Trait>).

struct UserService {
    repo: Box<dyn UserRepository>,
    // A Vec of trait objects -- impossible with generics alone
    // because different notifier types have different sizes
    notifiers: Vec<Box<dyn NotificationSender>>,
}

impl UserService {
    fn new(
        repo: Box<dyn UserRepository>,
        notifiers: Vec<Box<dyn NotificationSender>>,
    ) -> Self {
        UserService { repo, notifiers }
    }

    fn register(&self, id: u64, name: &str) -> Result<(), String> {
        if self.repo.find_by_id(id).is_some() {
            return Err(format!("user {} already exists", id));
        }

        self.repo.save(id, name)?;

        // Notify through ALL configured channels
        // This is where trait objects shine -- heterogeneous collection
        for notifier in &self.notifiers {
            notifier.send(
                name,
                &format!("Welcome to the platform, {}!", name),
            )?;
        }

        Ok(())
    }

    fn active_channels(&self) -> Vec<&str> {
        self.notifiers.iter().map(|n| n.channel_name()).collect()
    }
}

// ---- Factory Function: Runtime Selection ----
// This pattern is common when config determines which implementation to use

fn create_repo(backend: &str) -> Box<dyn UserRepository> {
    match backend {
        "memory" => Box::new(InMemoryRepo::new(vec![(1, "Alice"), (2, "Bob")])),
        "file" => Box::new(FileRepo::new("/var/data/users")),
        _ => panic!("unknown backend: {}", backend),
    }
}

fn create_notifiers(channels: &[&str]) -> Vec<Box<dyn NotificationSender>> {
    channels.iter().map(|ch| -> Box<dyn NotificationSender> {
        match *ch {
            "email" => Box::new(EmailNotifier::new("smtp.example.com")),
            "webhook" => Box::new(WebhookNotifier::new("https://hooks.example.com/notify")),
            "slack" => Box::new(SlackNotifier::new("alerts")),
            _ => panic!("unknown notification channel: {}", ch),
        }
    }).collect()
}

// ---- Composition Root ----

fn main() {
    println!("=== Dependency Injection via Trait Objects ===\n");

    // Simulate reading config
    let db_backend = "memory";
    let notification_channels = vec!["email", "slack", "webhook"];

    // Create dependencies based on runtime config
    let repo = create_repo(db_backend);
    let notifiers = create_notifiers(&notification_channels);

    // Wire the service -- no generic parameters
    let service = UserService::new(repo, notifiers);

    println!("Active notification channels: {:?}\n", service.active_channels());

    // Register a user -- notifications sent to all channels
    println!("--- Registering user 3 (Charlie) ---");
    match service.register(3, "Charlie") {
        Ok(()) => println!("  Registration successful\n"),
        Err(e) => println!("  Registration failed: {}\n", e),
    }

    // Try duplicate
    println!("--- Registering user 1 (duplicate) ---");
    match service.register(1, "Alice") {
        Ok(()) => println!("  Registration successful\n"),
        Err(e) => println!("  Registration failed: {}\n", e),
    }

    println!("=== Comparison with Generics ===\n");
    println!("Generics (service.rs):");
    println!("  + Zero-cost abstraction (no vtable)");
    println!("  + Compiler optimizations (inlining)");
    println!("  - Can't store different types in a Vec");
    println!("  - Type signatures grow with each dependency\n");
    println!("Trait objects (this file):");
    println!("  + Clean type signatures (no generics)");
    println!("  + Heterogeneous collections (Vec<Box<dyn Trait>>)");
    println!("  + Runtime-configurable dependencies");
    println!("  - ~1ns overhead per method call (vtable lookup)");
    println!("  - Heap allocation for each Box<dyn Trait>");
    println!("\nRule of thumb: use generics for core hot paths,");
    println!("trait objects for configuration and extensibility.");
}
