// Repository Pattern: User Repository
//
// Demonstrates: Trait-based abstraction for data access with CRUD operations,
// domain-specific queries (find_by_email, list_active), and an in-memory
// HashMap implementation.
//
// Scenario: A user management service that needs to store, retrieve, and query
// user records. The repository trait decouples the service from storage details,
// enabling easy testing and backend swaps.
//
// Run: rustc user_repo.rs && ./user_repo
// Test: rustc --test user_repo.rs && ./user_repo

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub active: bool,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Member,
    Guest,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Member => write!(f, "member"),
            Role::Guest => write!(f, "guest"),
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RepoError {
    NotFound { entity: String, id: String },
    DuplicateKey { entity: String, id: String },
    InvalidInput { field: String, reason: String },
    Internal { message: String },
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepoError::NotFound { entity, id } => write!(f, "{} not found: {}", entity, id),
            RepoError::DuplicateKey { entity, id } => {
                write!(f, "duplicate {} key: {}", entity, id)
            }
            RepoError::InvalidInput { field, reason } => {
                write!(f, "invalid {}: {}", field, reason)
            }
            RepoError::Internal { message } => write!(f, "internal error: {}", message),
        }
    }
}

impl std::error::Error for RepoError {}

// ---------------------------------------------------------------------------
// Repository trait -- the core abstraction
// ---------------------------------------------------------------------------

/// UserRepository defines all operations for managing User entities.
///
/// Design decisions:
/// - Returns owned `User` values (not references) for backend flexibility
/// - Every method returns `Result` even though in-memory can't fail
/// - Domain-specific queries alongside generic CRUD
/// - `&self` for reads, `&mut self` for writes (matches CQRS thinking)
pub trait UserRepository {
    /// Find a user by their unique ID.
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;

    /// Find a user by email address. Emails are unique across users.
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;

    /// Save a user (insert or update). Uses the user's ID as the key.
    fn save(&mut self, user: &User) -> Result<(), RepoError>;

    /// Delete a user by ID. Returns true if a user was actually deleted.
    fn delete(&mut self, id: &str) -> Result<bool, RepoError>;

    /// List all active users.
    fn list_active(&self) -> Result<Vec<User>, RepoError>;

    /// Count users by role.
    fn count_by_role(&self, role: &Role) -> Result<usize, RepoError>;
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

pub struct InMemoryUserRepo {
    users: HashMap<String, User>,
    /// Secondary index: email -> user_id for O(1) email lookups
    email_index: HashMap<String, String>,
}

impl InMemoryUserRepo {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            email_index: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.users.len()
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }
}

impl UserRepository for InMemoryUserRepo {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        Ok(self.users.get(id).cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        // Use the secondary index for O(1) lookup
        let user = self
            .email_index
            .get(email)
            .and_then(|id| self.users.get(id))
            .cloned();
        Ok(user)
    }

    fn save(&mut self, user: &User) -> Result<(), RepoError> {
        // Validate email uniqueness: another user (different ID) can't have this email
        if let Some(existing_id) = self.email_index.get(&user.email) {
            if existing_id != &user.id {
                return Err(RepoError::DuplicateKey {
                    entity: "User".to_string(),
                    id: format!("email:{}", user.email),
                });
            }
        }

        // If updating, remove old email index entry
        if let Some(existing) = self.users.get(&user.id) {
            if existing.email != user.email {
                self.email_index.remove(&existing.email);
            }
        }

        // Insert/update
        self.email_index
            .insert(user.email.clone(), user.id.clone());
        self.users.insert(user.id.clone(), user.clone());
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        if let Some(user) = self.users.remove(id) {
            self.email_index.remove(&user.email);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn list_active(&self) -> Result<Vec<User>, RepoError> {
        let active: Vec<User> = self
            .users
            .values()
            .filter(|u| u.active)
            .cloned()
            .collect();
        Ok(active)
    }

    fn count_by_role(&self, role: &Role) -> Result<usize, RepoError> {
        let count = self.users.values().filter(|u| &u.role == role).count();
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Service layer -- depends on the trait, not the implementation
// ---------------------------------------------------------------------------

struct UserService<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> UserService<R> {
    fn new(repo: R) -> Self {
        Self { repo }
    }

    fn register_user(
        &mut self,
        id: &str,
        email: &str,
        name: &str,
    ) -> Result<User, RepoError> {
        // Check for duplicate email
        if let Some(_existing) = self.repo.find_by_email(email)? {
            return Err(RepoError::DuplicateKey {
                entity: "User".to_string(),
                id: format!("email:{}", email),
            });
        }

        let user = User {
            id: id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            active: true,
            role: Role::Member,
        };

        self.repo.save(&user)?;
        Ok(user)
    }

    fn deactivate_user(&mut self, id: &str) -> Result<User, RepoError> {
        let user = self.repo.find_by_id(id)?.ok_or(RepoError::NotFound {
            entity: "User".to_string(),
            id: id.to_string(),
        })?;

        let deactivated = User {
            active: false,
            ..user
        };

        self.repo.save(&deactivated)?;
        Ok(deactivated)
    }

    fn promote_to_admin(&mut self, id: &str) -> Result<User, RepoError> {
        let user = self.repo.find_by_id(id)?.ok_or(RepoError::NotFound {
            entity: "User".to_string(),
            id: id.to_string(),
        })?;

        let promoted = User {
            role: Role::Admin,
            ..user
        };

        self.repo.save(&promoted)?;
        Ok(promoted)
    }

    fn get_active_member_count(&self) -> Result<usize, RepoError> {
        let active = self.repo.list_active()?;
        Ok(active.iter().filter(|u| u.role == Role::Member).count())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(id: &str, email: &str, name: &str) -> User {
        User {
            id: id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            active: true,
            role: Role::Member,
        }
    }

    // --- Repository tests ---

    #[test]
    fn test_save_and_find() {
        let mut repo = InMemoryUserRepo::new();
        let user = make_user("u1", "alice@example.com", "Alice");

        repo.save(&user).unwrap();
        let found = repo.find_by_id("u1").unwrap();

        assert_eq!(found, Some(user));
    }

    #[test]
    fn test_find_nonexistent_returns_none() {
        let repo = InMemoryUserRepo::new();
        assert_eq!(repo.find_by_id("u999").unwrap(), None);
    }

    #[test]
    fn test_find_by_email() {
        let mut repo = InMemoryUserRepo::new();
        let user = make_user("u1", "alice@example.com", "Alice");
        repo.save(&user).unwrap();

        let found = repo.find_by_email("alice@example.com").unwrap();
        assert_eq!(found, Some(user));

        let not_found = repo.find_by_email("nobody@example.com").unwrap();
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_duplicate_email_rejected() {
        let mut repo = InMemoryUserRepo::new();
        repo.save(&make_user("u1", "alice@example.com", "Alice"))
            .unwrap();

        let result = repo.save(&make_user("u2", "alice@example.com", "Bob"));
        assert!(matches!(result, Err(RepoError::DuplicateKey { .. })));
    }

    #[test]
    fn test_update_email() {
        let mut repo = InMemoryUserRepo::new();
        repo.save(&make_user("u1", "old@example.com", "Alice"))
            .unwrap();

        // Update email for same user
        repo.save(&make_user("u1", "new@example.com", "Alice"))
            .unwrap();

        assert!(repo.find_by_email("old@example.com").unwrap().is_none());
        assert!(repo.find_by_email("new@example.com").unwrap().is_some());
    }

    #[test]
    fn test_delete() {
        let mut repo = InMemoryUserRepo::new();
        repo.save(&make_user("u1", "alice@example.com", "Alice"))
            .unwrap();

        assert!(repo.delete("u1").unwrap());
        assert!(!repo.delete("u1").unwrap()); // already deleted
        assert!(repo.find_by_id("u1").unwrap().is_none());
        assert!(repo.find_by_email("alice@example.com").unwrap().is_none());
    }

    #[test]
    fn test_list_active() {
        let mut repo = InMemoryUserRepo::new();
        repo.save(&make_user("u1", "a@x.com", "Alice")).unwrap();
        repo.save(&User {
            active: false,
            ..make_user("u2", "b@x.com", "Bob")
        })
        .unwrap();
        repo.save(&make_user("u3", "c@x.com", "Charlie")).unwrap();

        let active = repo.list_active().unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|u| u.active));
    }

    #[test]
    fn test_count_by_role() {
        let mut repo = InMemoryUserRepo::new();
        repo.save(&make_user("u1", "a@x.com", "Alice")).unwrap();
        repo.save(&make_user("u2", "b@x.com", "Bob")).unwrap();
        repo.save(&User {
            role: Role::Admin,
            ..make_user("u3", "c@x.com", "Charlie")
        })
        .unwrap();

        assert_eq!(repo.count_by_role(&Role::Member).unwrap(), 2);
        assert_eq!(repo.count_by_role(&Role::Admin).unwrap(), 1);
        assert_eq!(repo.count_by_role(&Role::Guest).unwrap(), 0);
    }

    // --- Service tests (using InMemoryUserRepo as the mock) ---

    #[test]
    fn test_service_register_user() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        let user = service
            .register_user("u1", "alice@example.com", "Alice")
            .unwrap();

        assert_eq!(user.email, "alice@example.com");
        assert!(user.active);
        assert_eq!(user.role, Role::Member);
    }

    #[test]
    fn test_service_duplicate_registration() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        service
            .register_user("u1", "alice@example.com", "Alice")
            .unwrap();
        let result = service.register_user("u2", "alice@example.com", "Bob");
        assert!(matches!(result, Err(RepoError::DuplicateKey { .. })));
    }

    #[test]
    fn test_service_deactivate_user() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        service
            .register_user("u1", "alice@example.com", "Alice")
            .unwrap();
        let deactivated = service.deactivate_user("u1").unwrap();

        assert!(!deactivated.active);
    }

    #[test]
    fn test_service_deactivate_nonexistent() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        let result = service.deactivate_user("u999");
        assert!(matches!(result, Err(RepoError::NotFound { .. })));
    }

    #[test]
    fn test_service_promote_to_admin() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        service
            .register_user("u1", "alice@example.com", "Alice")
            .unwrap();
        let promoted = service.promote_to_admin("u1").unwrap();

        assert_eq!(promoted.role, Role::Admin);
    }

    #[test]
    fn test_service_active_member_count() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        service
            .register_user("u1", "a@x.com", "Alice")
            .unwrap();
        service
            .register_user("u2", "b@x.com", "Bob")
            .unwrap();
        service
            .register_user("u3", "c@x.com", "Charlie")
            .unwrap();
        service.promote_to_admin("u3").unwrap();
        service.deactivate_user("u2").unwrap();

        // u1 active member, u2 inactive, u3 active admin
        assert_eq!(service.get_active_member_count().unwrap(), 1);
    }
}

// ---------------------------------------------------------------------------
// Main: demo
// ---------------------------------------------------------------------------

fn main() {
    println!("=== User Repository Pattern Demo ===\n");

    let repo = InMemoryUserRepo::new();
    let mut service = UserService::new(repo);

    // Register users
    println!("--- Registering users ---");
    let alice = service
        .register_user("u1", "alice@company.com", "Alice Chen")
        .unwrap();
    println!("  Registered: {} ({})", alice.name, alice.email);

    let bob = service
        .register_user("u2", "bob@company.com", "Bob Singh")
        .unwrap();
    println!("  Registered: {} ({})", bob.name, bob.email);

    let charlie = service
        .register_user("u3", "charlie@company.com", "Charlie Kim")
        .unwrap();
    println!("  Registered: {} ({})", charlie.name, charlie.email);

    // Try duplicate
    println!("\n--- Duplicate email ---");
    match service.register_user("u4", "alice@company.com", "Eve") {
        Err(e) => println!("  Rejected: {}", e),
        Ok(_) => println!("  ERROR: should have been rejected"),
    }

    // Promote and deactivate
    println!("\n--- Role changes ---");
    let promoted = service.promote_to_admin("u1").unwrap();
    println!("  {} promoted to {}", promoted.name, promoted.role);

    let deactivated = service.deactivate_user("u2").unwrap();
    println!("  {} deactivated (active={})", deactivated.name, deactivated.active);

    // Query
    println!("\n--- Queries ---");
    let active_members = service.get_active_member_count().unwrap();
    println!("  Active members: {}", active_members);

    let active = service.repo.list_active().unwrap();
    println!("  Active users: {:?}", active.iter().map(|u| &u.name).collect::<Vec<_>>());

    // Lookup by email
    println!("\n--- Email lookup ---");
    match service.repo.find_by_email("charlie@company.com").unwrap() {
        Some(user) => println!("  Found: {} (id={})", user.name, user.id),
        None => println!("  Not found"),
    }

    // Delete
    println!("\n--- Delete ---");
    let deleted = service.repo.delete("u2").unwrap();
    println!("  Deleted u2: {}", deleted);
    println!("  Total users: {}", service.repo.len());
}
