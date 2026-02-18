// Dependency Injection via Generics (Static Dispatch)
//
// Demonstrates a UserService with generic dependencies.
// The compiler monomorphizes each concrete type combination,
// producing zero-cost abstraction -- no vtable, no heap allocation.
//
// Run: rustc service.rs && ./service

use std::collections::HashMap;

// ---- Dependency Traits ----

trait UserRepository {
    fn find_by_id(&self, id: u64) -> Option<String>;
    fn save(&self, id: u64, name: &str) -> Result<(), String>;
    fn delete(&self, id: u64) -> Result<bool, String>;
}

trait EmailSender {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

trait AuditLogger {
    fn log(&self, actor: &str, action: &str, target: &str);
}

// ---- Production Implementations ----

struct InMemoryRepo {
    users: std::cell::RefCell<HashMap<u64, String>>,
}

impl InMemoryRepo {
    fn new() -> Self {
        InMemoryRepo {
            users: std::cell::RefCell::new(HashMap::new()),
        }
    }

    fn with_seed(data: Vec<(u64, &str)>) -> Self {
        let mut users = HashMap::new();
        for (id, name) in data {
            users.insert(id, name.to_string());
        }
        InMemoryRepo {
            users: std::cell::RefCell::new(users),
        }
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

    fn delete(&self, id: u64) -> Result<bool, String> {
        Ok(self.users.borrow_mut().remove(&id).is_some())
    }
}

struct ConsoleEmailSender;

impl EmailSender for ConsoleEmailSender {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        println!("  [EMAIL] To: {} | Subject: {} | Body: {}", to, subject, body);
        Ok(())
    }
}

struct ConsoleAuditLogger;

impl AuditLogger for ConsoleAuditLogger {
    fn log(&self, actor: &str, action: &str, target: &str) {
        println!("  [AUDIT] {} performed '{}' on '{}'", actor, action, target);
    }
}

// ---- The Service: Generic over all dependencies ----

struct UserService<R: UserRepository, E: EmailSender, A: AuditLogger> {
    repo: R,
    email: E,
    audit: A,
}

impl<R: UserRepository, E: EmailSender, A: AuditLogger> UserService<R, E, A> {
    // Constructor injection -- all dependencies passed in
    fn new(repo: R, email: E, audit: A) -> Self {
        UserService { repo, email, audit }
    }

    fn register(&self, id: u64, name: &str, email_addr: &str) -> Result<(), String> {
        // Check if user already exists
        if self.repo.find_by_id(id).is_some() {
            return Err(format!("user {} already exists", id));
        }

        // Save user
        self.repo.save(id, name)?;

        // Send welcome email
        self.email.send(
            email_addr,
            "Welcome!",
            &format!("Hello {}, your account is ready.", name),
        )?;

        // Audit
        self.audit.log("system", "register", &format!("user:{}", id));

        Ok(())
    }

    fn lookup(&self, id: u64) -> Option<String> {
        let result = self.repo.find_by_id(id);
        if result.is_some() {
            self.audit.log("system", "lookup", &format!("user:{}", id));
        }
        result
    }

    fn deactivate(&self, id: u64) -> Result<bool, String> {
        let deleted = self.repo.delete(id)?;
        if deleted {
            self.audit.log("system", "deactivate", &format!("user:{}", id));
        }
        Ok(deleted)
    }
}

// ---- Composition Root ----

fn main() {
    println!("=== Dependency Injection via Generics ===\n");

    // Wire dependencies in main() -- the composition root
    let repo = InMemoryRepo::with_seed(vec![
        (1, "Alice"),
        (2, "Bob"),
    ]);
    let email = ConsoleEmailSender;
    let audit = ConsoleAuditLogger;

    // UserService<InMemoryRepo, ConsoleEmailSender, ConsoleAuditLogger>
    // The compiler generates a specialized version for this exact type combination.
    // No vtable, no heap allocation, no runtime overhead.
    let service = UserService::new(repo, email, audit);

    // Register a new user
    println!("--- Registering user 3 (Charlie) ---");
    match service.register(3, "Charlie", "charlie@example.com") {
        Ok(()) => println!("  Registration successful\n"),
        Err(e) => println!("  Registration failed: {}\n", e),
    }

    // Try to register a duplicate
    println!("--- Registering user 1 (duplicate) ---");
    match service.register(1, "Alice Again", "alice@example.com") {
        Ok(()) => println!("  Registration successful\n"),
        Err(e) => println!("  Registration failed: {}\n", e),
    }

    // Look up a user
    println!("--- Looking up user 2 ---");
    match service.lookup(2) {
        Some(name) => println!("  Found: {}\n", name),
        None => println!("  Not found\n"),
    }

    // Deactivate a user
    println!("--- Deactivating user 1 ---");
    match service.deactivate(1) {
        Ok(true) => println!("  Deactivated successfully\n"),
        Ok(false) => println!("  User not found\n"),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Verify deactivation
    println!("--- Looking up deactivated user 1 ---");
    match service.lookup(1) {
        Some(name) => println!("  Found: {} (unexpected!)\n", name),
        None => println!("  Not found (correctly deactivated)\n"),
    }

    println!("=== Key Insight ===");
    println!("The type of `service` is:");
    println!("  UserService<InMemoryRepo, ConsoleEmailSender, ConsoleAuditLogger>");
    println!("The compiler knows every concrete type. All calls are direct (no vtable).");
    println!("Swapping InMemoryRepo for PostgresRepo only changes the composition root.");
}
