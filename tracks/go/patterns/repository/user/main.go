// Repository Pattern: User Repository
//
// Demonstrates the core repository pattern. A UserService depends on a
// UserStore interface with CRUD + FindByEmail + ListActive methods. An
// in-memory implementation shows how business logic uses the interface
// without knowing storage details.
//
// Key ideas:
//   - Interface defined in the consumer package (UserStore)
//   - In-memory implementation for testing and demonstration
//   - Sentinel errors (ErrNotFound, ErrConflict) instead of database errors
//   - Data is copied in and out to prevent caller mutation of internal state
//   - context.Context on every method
//
// Run: go run ./user/

package main

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"
)

// =============================================================================
// Domain Types
// =============================================================================

// User represents a user in the system.
type User struct {
	ID        string
	Email     string
	Name      string
	Role      string
	Active    bool
	CreatedAt time.Time
	UpdatedAt time.Time
}

// Sentinel errors -- domain-level, not database-level.
// Repository implementations translate database-specific errors into these.
var (
	ErrNotFound = errors.New("not found")
	ErrConflict = errors.New("conflict: entity already exists")
)

// =============================================================================
// Repository Interface (defined by the consumer)
// =============================================================================

// UserStore defines the data operations the user service needs.
// This interface is deliberately small -- it only has the methods this
// service actually calls. Other services can define their own interfaces
// with different method subsets, and the same concrete type can satisfy all of them.
type UserStore interface {
	FindByID(ctx context.Context, id string) (*User, error)
	FindByEmail(ctx context.Context, email string) (*User, error)
	Save(ctx context.Context, user *User) error
	Delete(ctx context.Context, id string) error
	ListActive(ctx context.Context) ([]*User, error)
}

// =============================================================================
// In-Memory Implementation
// =============================================================================

// MemoryUserStore stores users in a map. This is the implementation you use
// in tests and demos. It behaves like a real database:
//   - Generates IDs on insert
//   - Returns copies (not pointers to internal data)
//   - Stores copies (caller can't mutate after saving)
//   - Enforces unique email constraint
type MemoryUserStore struct {
	mu     sync.RWMutex
	users  map[string]*User // indexed by ID
	nextID int
}

// Compile-time interface guard
var _ UserStore = (*MemoryUserStore)(nil)

// NewMemoryUserStore creates an empty in-memory user store.
func NewMemoryUserStore() *MemoryUserStore {
	return &MemoryUserStore{
		users: make(map[string]*User),
	}
}

func (m *MemoryUserStore) FindByID(_ context.Context, id string) (*User, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	u, ok := m.users[id]
	if !ok {
		return nil, ErrNotFound
	}
	// Return a copy -- caller must not mutate our internal state
	copy := *u
	return &copy, nil
}

func (m *MemoryUserStore) FindByEmail(_ context.Context, email string) (*User, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, u := range m.users {
		if strings.EqualFold(u.Email, email) {
			copy := *u
			return &copy, nil
		}
	}
	return nil, ErrNotFound
}

func (m *MemoryUserStore) Save(_ context.Context, user *User) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Check email uniqueness (like a UNIQUE constraint in a real database)
	for _, existing := range m.users {
		if strings.EqualFold(existing.Email, user.Email) && existing.ID != user.ID {
			return ErrConflict
		}
	}

	now := time.Now()

	if user.ID == "" {
		// Insert: generate an ID
		m.nextID++
		user.ID = fmt.Sprintf("user-%d", m.nextID)
		user.CreatedAt = now
	}
	user.UpdatedAt = now

	// Store a copy -- caller's later mutations must not affect our state
	stored := *user
	m.users[stored.ID] = &stored
	return nil
}

func (m *MemoryUserStore) Delete(_ context.Context, id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if _, ok := m.users[id]; !ok {
		return ErrNotFound
	}
	delete(m.users, id)
	return nil
}

func (m *MemoryUserStore) ListActive(_ context.Context) ([]*User, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var active []*User
	for _, u := range m.users {
		if u.Active {
			copy := *u
			active = append(active, &copy)
		}
	}
	return active, nil
}

// =============================================================================
// Business Logic (UserService)
// =============================================================================

// UserService contains business logic for user management.
// It depends on the UserStore interface -- it never imports a database package.
type UserService struct {
	store UserStore
}

// NewUserService creates a user service. The store is injected -- the service
// doesn't know (or care) whether it's talking to Postgres, Redis, or a map.
func NewUserService(store UserStore) *UserService {
	return &UserService{store: store}
}

// Register creates a new user, enforcing business rules.
func (s *UserService) Register(ctx context.Context, email, name, role string) (*User, error) {
	// Business rule: check for duplicate email
	_, err := s.store.FindByEmail(ctx, email)
	if err == nil {
		return nil, fmt.Errorf("email %s is already registered", email)
	}
	if !errors.Is(err, ErrNotFound) {
		return nil, fmt.Errorf("checking existing user: %w", err)
	}

	// Business rule: validate role
	validRoles := map[string]bool{"admin": true, "editor": true, "viewer": true}
	if !validRoles[role] {
		return nil, fmt.Errorf("invalid role: %s (must be admin, editor, or viewer)", role)
	}

	user := &User{
		Email:  email,
		Name:   name,
		Role:   role,
		Active: true,
	}

	if err := s.store.Save(ctx, user); err != nil {
		return nil, fmt.Errorf("saving user: %w", err)
	}

	return user, nil
}

// Deactivate marks a user as inactive (soft delete).
func (s *UserService) Deactivate(ctx context.Context, id string) error {
	user, err := s.store.FindByID(ctx, id)
	if err != nil {
		return fmt.Errorf("finding user: %w", err)
	}

	if !user.Active {
		return fmt.Errorf("user %s is already inactive", id)
	}

	user.Active = false
	return s.store.Save(ctx, user)
}

// GetActiveUsers returns all active users.
func (s *UserService) GetActiveUsers(ctx context.Context) ([]*User, error) {
	return s.store.ListActive(ctx)
}

// =============================================================================
// Main: Demonstrate the pattern
// =============================================================================

func main() {
	ctx := context.Background()

	// Create an in-memory store -- in production, this would be a Postgres/MySQL repo
	store := NewMemoryUserStore()
	svc := NewUserService(store)

	fmt.Println("=== Repository Pattern: User Service ===")
	fmt.Println(strings.Repeat("-", 50))

	// Register users
	fmt.Println("\n1. Registering users:")
	alice, err := svc.Register(ctx, "alice@example.com", "Alice Chen", "admin")
	if err != nil {
		fmt.Printf("   ERROR: %v\n", err)
	} else {
		fmt.Printf("   Registered: %s (%s) as %s [ID: %s]\n", alice.Name, alice.Email, alice.Role, alice.ID)
	}

	bob, err := svc.Register(ctx, "bob@example.com", "Bob Smith", "editor")
	if err != nil {
		fmt.Printf("   ERROR: %v\n", err)
	} else {
		fmt.Printf("   Registered: %s (%s) as %s [ID: %s]\n", bob.Name, bob.Email, bob.Role, bob.ID)
	}

	carol, err := svc.Register(ctx, "carol@example.com", "Carol Davis", "viewer")
	if err != nil {
		fmt.Printf("   ERROR: %v\n", err)
	} else {
		fmt.Printf("   Registered: %s (%s) as %s [ID: %s]\n", carol.Name, carol.Email, carol.Role, carol.ID)
	}

	// Try to register a duplicate email
	fmt.Println("\n2. Duplicate email detection:")
	_, err = svc.Register(ctx, "alice@example.com", "Alice Two", "viewer")
	fmt.Printf("   Duplicate registration result: %v\n", err)

	// Try to register with invalid role
	fmt.Println("\n3. Role validation:")
	_, err = svc.Register(ctx, "dave@example.com", "Dave Wilson", "superadmin")
	fmt.Printf("   Invalid role result: %v\n", err)

	// List active users
	fmt.Println("\n4. Active users:")
	active, _ := svc.GetActiveUsers(ctx)
	for _, u := range active {
		fmt.Printf("   - %s (%s) [%s]\n", u.Name, u.Email, u.Role)
	}

	// Deactivate a user
	fmt.Println("\n5. Deactivating Bob:")
	if err := svc.Deactivate(ctx, bob.ID); err != nil {
		fmt.Printf("   ERROR: %v\n", err)
	} else {
		fmt.Println("   Bob deactivated successfully")
	}

	// List active users again
	fmt.Println("\n6. Active users after deactivation:")
	active, _ = svc.GetActiveUsers(ctx)
	for _, u := range active {
		fmt.Printf("   - %s (%s) [%s]\n", u.Name, u.Email, u.Role)
	}

	// Demonstrate that FindByEmail still works (deactivated users are findable)
	fmt.Println("\n7. Finding deactivated user by email:")
	found, err := store.FindByEmail(ctx, "bob@example.com")
	if err != nil {
		fmt.Printf("   ERROR: %v\n", err)
	} else {
		fmt.Printf("   Found: %s (active: %v)\n", found.Name, found.Active)
	}

	fmt.Println(strings.Repeat("-", 50))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- UserService depends on UserStore interface, not a concrete type")
	fmt.Println("- MemoryUserStore satisfies UserStore without importing the service package")
	fmt.Println("- Business rules (validation, deduplication) live in the service layer")
	fmt.Println("- Storage details (map operations, locking) live in the repository layer")
	fmt.Println("- Swapping to Postgres means writing a new UserStore -- zero changes to UserService")
}
