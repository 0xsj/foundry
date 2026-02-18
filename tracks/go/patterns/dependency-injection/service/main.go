// Dependency Injection: API Service with Constructor Injection
//
// A UserService that depends on UserRepository, EmailSender, and Logger
// interfaces. All dependencies are injected through the constructor.
// The composition root in main() wires real implementations together.
//
// Run: go run ./service/

package main

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strings"
	"sync"
	"time"
)

// =============================================================================
// Domain types
// =============================================================================

// User represents a user in the system.
type User struct {
	ID        string
	Email     string
	Name      string
	CreatedAt time.Time
	Active    bool
}

// =============================================================================
// Interfaces -- defined by the consumer (UserService), not the implementor
// =============================================================================

// UserRepository abstracts user persistence. The UserService doesn't know
// whether this is Postgres, MongoDB, or an in-memory map.
type UserRepository interface {
	FindByID(ctx context.Context, id string) (*User, error)
	FindByEmail(ctx context.Context, email string) (*User, error)
	Save(ctx context.Context, user *User) error
	Delete(ctx context.Context, id string) error
}

// EmailSender abstracts email delivery. Could be SMTP, SendGrid, or a mock.
type EmailSender interface {
	Send(ctx context.Context, to, subject, body string) error
}

// =============================================================================
// UserService -- the component that receives injected dependencies
// =============================================================================

// UserService handles user-related business logic. It depends on abstractions
// (interfaces), not concrete implementations. Every dependency is explicit.
type UserService struct {
	repo   UserRepository
	mailer EmailSender
	logger *slog.Logger
}

// NewUserService constructs a UserService with all dependencies injected.
// Required dependencies cause an error if nil. Logger has a sensible default.
func NewUserService(repo UserRepository, mailer EmailSender, logger *slog.Logger) (*UserService, error) {
	if repo == nil {
		return nil, errors.New("user service: repository is required")
	}
	if mailer == nil {
		return nil, errors.New("user service: email sender is required")
	}
	if logger == nil {
		logger = slog.Default()
	}
	return &UserService{
		repo:   repo,
		mailer: mailer,
		logger: logger,
	}, nil
}

// CreateUser registers a new user and sends a welcome email.
func (s *UserService) CreateUser(ctx context.Context, name, email string) (*User, error) {
	// Check for existing user
	existing, err := s.repo.FindByEmail(ctx, email)
	if err == nil && existing != nil {
		return nil, fmt.Errorf("user with email %s already exists", email)
	}

	user := &User{
		ID:        fmt.Sprintf("usr_%d", time.Now().UnixNano()),
		Email:     email,
		Name:      name,
		CreatedAt: time.Now(),
		Active:    true,
	}

	if err := s.repo.Save(ctx, user); err != nil {
		s.logger.Error("failed to save user", "email", email, "error", err)
		return nil, fmt.Errorf("save user: %w", err)
	}

	s.logger.Info("user created", "id", user.ID, "email", email)

	// Send welcome email (non-critical -- log errors but don't fail)
	if err := s.mailer.Send(ctx, email, "Welcome!", "Hello "+name+", welcome aboard!"); err != nil {
		s.logger.Warn("failed to send welcome email", "email", email, "error", err)
	}

	return user, nil
}

// GetUser retrieves a user by ID.
func (s *UserService) GetUser(ctx context.Context, id string) (*User, error) {
	user, err := s.repo.FindByID(ctx, id)
	if err != nil {
		s.logger.Error("failed to find user", "id", id, "error", err)
		return nil, fmt.Errorf("find user %s: %w", id, err)
	}
	return user, nil
}

// DeactivateUser marks a user as inactive and notifies them.
func (s *UserService) DeactivateUser(ctx context.Context, id string) error {
	user, err := s.repo.FindByID(ctx, id)
	if err != nil {
		return fmt.Errorf("find user %s: %w", id, err)
	}

	user.Active = false
	if err := s.repo.Save(ctx, user); err != nil {
		return fmt.Errorf("save user %s: %w", id, err)
	}

	s.logger.Info("user deactivated", "id", id, "email", user.Email)

	_ = s.mailer.Send(ctx, user.Email, "Account Deactivated",
		"Your account has been deactivated. Contact support to reactivate.")

	return nil
}

// =============================================================================
// Concrete implementations -- these satisfy the interfaces above
// but don't import or know about UserService or its interfaces.
// =============================================================================

// InMemoryUserRepo is a simple in-memory repository. In production, this
// would be PostgresUserRepo, MongoUserRepo, etc.
type InMemoryUserRepo struct {
	mu    sync.RWMutex
	users map[string]*User
}

// Compile-time check: does InMemoryUserRepo satisfy UserRepository?
var _ UserRepository = (*InMemoryUserRepo)(nil)

func NewInMemoryUserRepo() *InMemoryUserRepo {
	return &InMemoryUserRepo{
		users: make(map[string]*User),
	}
}

func (r *InMemoryUserRepo) FindByID(ctx context.Context, id string) (*User, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	user, ok := r.users[id]
	if !ok {
		return nil, fmt.Errorf("user %s not found", id)
	}
	return user, nil
}

func (r *InMemoryUserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	for _, u := range r.users {
		if u.Email == email {
			return u, nil
		}
	}
	return nil, fmt.Errorf("user with email %s not found", email)
}

func (r *InMemoryUserRepo) Save(ctx context.Context, user *User) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.users[user.ID] = user
	return nil
}

func (r *InMemoryUserRepo) Delete(ctx context.Context, id string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if _, ok := r.users[id]; !ok {
		return fmt.Errorf("user %s not found", id)
	}
	delete(r.users, id)
	return nil
}

// ConsoleEmailSender prints emails to stdout. In production, this would
// be an SMTP client, SendGrid adapter, etc.
type ConsoleEmailSender struct{}

var _ EmailSender = (*ConsoleEmailSender)(nil)

func (c *ConsoleEmailSender) Send(ctx context.Context, to, subject, body string) error {
	fmt.Printf("  [EMAIL] To: %s | Subject: %s | Body: %s\n", to, subject, body)
	return nil
}

// =============================================================================
// Composition Root -- wire everything together in main()
// =============================================================================

func main() {
	fmt.Println("=== Dependency Injection: Service Example ===")
	fmt.Println(strings.Repeat("=", 50))

	// --- Composition root: create and wire all dependencies ---

	// 1. Infrastructure
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))

	// 2. Repositories (depend on infrastructure)
	userRepo := NewInMemoryUserRepo()

	// 3. External services (depend on infrastructure/config)
	mailer := &ConsoleEmailSender{}

	// 4. Business services (depend on repositories and external services)
	userService, err := NewUserService(userRepo, mailer, logger)
	if err != nil {
		logger.Error("failed to create user service", "error", err)
		os.Exit(1)
	}

	// --- Use the wired service ---
	ctx := context.Background()

	fmt.Println("\n--- Creating users ---")
	alice, err := userService.CreateUser(ctx, "Alice Chen", "alice@example.com")
	if err != nil {
		logger.Error("failed to create user", "error", err)
	} else {
		fmt.Printf("  Created: %s (%s)\n", alice.Name, alice.ID)
	}

	bob, err := userService.CreateUser(ctx, "Bob Smith", "bob@example.com")
	if err != nil {
		logger.Error("failed to create user", "error", err)
	} else {
		fmt.Printf("  Created: %s (%s)\n", bob.Name, bob.ID)
	}

	// Try duplicate
	_, err = userService.CreateUser(ctx, "Alice Clone", "alice@example.com")
	if err != nil {
		fmt.Printf("  Expected error: %v\n", err)
	}

	fmt.Println("\n--- Looking up user ---")
	found, err := userService.GetUser(ctx, alice.ID)
	if err != nil {
		logger.Error("lookup failed", "error", err)
	} else {
		fmt.Printf("  Found: %s (active: %v)\n", found.Name, found.Active)
	}

	fmt.Println("\n--- Deactivating user ---")
	if err := userService.DeactivateUser(ctx, bob.ID); err != nil {
		logger.Error("deactivation failed", "error", err)
	}

	found, _ = userService.GetUser(ctx, bob.ID)
	fmt.Printf("  Bob is now active: %v\n", found.Active)

	// --- Demonstrate nil dependency validation ---
	fmt.Println("\n--- Nil dependency validation ---")
	_, err = NewUserService(nil, mailer, logger)
	fmt.Printf("  nil repo:   %v\n", err)

	_, err = NewUserService(userRepo, nil, logger)
	fmt.Printf("  nil mailer: %v\n", err)

	fmt.Println(strings.Repeat("=", 50))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- UserService never creates its own dependencies")
	fmt.Println("- All wiring happens in main() (the composition root)")
	fmt.Println("- Swapping InMemoryUserRepo for PostgresRepo: change one line in main()")
	fmt.Println("- Swapping ConsoleEmailSender for SendGrid: change one line in main()")
	fmt.Println("- UserService code doesn't change at all")
}
