// Dependency Injection: Testing with Fakes and Spies
//
// Demonstrates how DI enables isolated unit testing. Uses the same UserService
// from the service example, but wires in fakes and spies instead of real
// implementations. Shows table-driven tests with injected test doubles.
//
// Run: go run ./testing/
//
// NOTE: This file demonstrates testing patterns inline. In a real project,
// these would be in *_test.go files. We use main() here so the example
// is runnable with `go run`.

package main

import (
	"context"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"strings"
	"sync"
	"time"
)

// =============================================================================
// Domain and Interfaces (same as service example)
// =============================================================================

type User struct {
	ID        string
	Email     string
	Name      string
	CreatedAt time.Time
	Active    bool
}

type UserRepository interface {
	FindByID(ctx context.Context, id string) (*User, error)
	FindByEmail(ctx context.Context, email string) (*User, error)
	Save(ctx context.Context, user *User) error
	Delete(ctx context.Context, id string) error
}

type EmailSender interface {
	Send(ctx context.Context, to, subject, body string) error
}

// =============================================================================
// UserService (same as service example)
// =============================================================================

type UserService struct {
	repo   UserRepository
	mailer EmailSender
	logger *slog.Logger
}

func NewUserService(repo UserRepository, mailer EmailSender, logger *slog.Logger) *UserService {
	if logger == nil {
		logger = slog.New(slog.NewTextHandler(io.Discard, nil))
	}
	return &UserService{repo: repo, mailer: mailer, logger: logger}
}

func (s *UserService) CreateUser(ctx context.Context, name, email string) (*User, error) {
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
		return nil, fmt.Errorf("save user: %w", err)
	}

	if err := s.mailer.Send(ctx, email, "Welcome!", "Hello "+name+", welcome aboard!"); err != nil {
		s.logger.Warn("failed to send welcome email", "email", email, "error", err)
	}

	return user, nil
}

func (s *UserService) GetUser(ctx context.Context, id string) (*User, error) {
	return s.repo.FindByID(ctx, id)
}

func (s *UserService) DeactivateUser(ctx context.Context, id string) error {
	user, err := s.repo.FindByID(ctx, id)
	if err != nil {
		return fmt.Errorf("find user %s: %w", id, err)
	}
	user.Active = false
	if err := s.repo.Save(ctx, user); err != nil {
		return fmt.Errorf("save user %s: %w", id, err)
	}
	_ = s.mailer.Send(ctx, user.Email, "Account Deactivated",
		"Your account has been deactivated.")
	return nil
}

// =============================================================================
// Test Doubles: Fake Repository
// =============================================================================

// FakeUserRepo is a working in-memory implementation of UserRepository.
// It behaves like a real database without the database. The Err fields
// allow tests to simulate failures.
type FakeUserRepo struct {
	mu       sync.RWMutex
	users    map[string]*User
	SaveErr  error // Set this to simulate save failures
	FindErr  error // Set this to simulate find failures
	Saved    []*User // Records what was saved (spy behavior)
}

var _ UserRepository = (*FakeUserRepo)(nil)

func NewFakeUserRepo() *FakeUserRepo {
	return &FakeUserRepo{
		users: make(map[string]*User),
		Saved: make([]*User, 0),
	}
}

func (f *FakeUserRepo) FindByID(ctx context.Context, id string) (*User, error) {
	if f.FindErr != nil {
		return nil, f.FindErr
	}
	f.mu.RLock()
	defer f.mu.RUnlock()
	user, ok := f.users[id]
	if !ok {
		return nil, fmt.Errorf("user %s not found", id)
	}
	return user, nil
}

func (f *FakeUserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
	if f.FindErr != nil {
		return nil, f.FindErr
	}
	f.mu.RLock()
	defer f.mu.RUnlock()
	for _, u := range f.users {
		if u.Email == email {
			return u, nil
		}
	}
	return nil, fmt.Errorf("user with email %s not found", email)
}

func (f *FakeUserRepo) Save(ctx context.Context, user *User) error {
	if f.SaveErr != nil {
		return f.SaveErr
	}
	f.mu.Lock()
	defer f.mu.Unlock()
	f.users[user.ID] = user
	f.Saved = append(f.Saved, user)
	return nil
}

func (f *FakeUserRepo) Delete(ctx context.Context, id string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	delete(f.users, id)
	return nil
}

// Seed adds a user directly to the fake (for test setup, bypassing Save).
func (f *FakeUserRepo) Seed(users ...*User) {
	f.mu.Lock()
	defer f.mu.Unlock()
	for _, u := range users {
		f.users[u.ID] = u
	}
}

// =============================================================================
// Test Doubles: Spy Email Sender
// =============================================================================

// SpyEmailSender records all emails sent. Use it to verify that the right
// emails were sent without actually sending anything.
type SpyEmailSender struct {
	Sent    []SentEmail
	SendErr error // Set this to simulate send failures
}

type SentEmail struct {
	To      string
	Subject string
	Body    string
}

var _ EmailSender = (*SpyEmailSender)(nil)

func (s *SpyEmailSender) Send(ctx context.Context, to, subject, body string) error {
	if s.SendErr != nil {
		return s.SendErr
	}
	s.Sent = append(s.Sent, SentEmail{To: to, Subject: subject, Body: body})
	return nil
}

// =============================================================================
// Test Helpers
// =============================================================================

// setupTestService creates a test composition root. This is the test equivalent
// of main() -- it wires up fakes instead of real implementations.
func setupTestService() (*UserService, *FakeUserRepo, *SpyEmailSender) {
	repo := NewFakeUserRepo()
	mailer := &SpyEmailSender{}
	logger := slog.New(slog.NewTextHandler(io.Discard, nil)) // Silence logs in tests
	svc := NewUserService(repo, mailer, logger)
	return svc, repo, mailer
}

// =============================================================================
// Test Runner (simulates go test output)
// =============================================================================

type testResult struct {
	name   string
	passed bool
	err    string
}

var results []testResult

func assert(name string, condition bool, msg string) {
	if condition {
		results = append(results, testResult{name: name, passed: true})
	} else {
		results = append(results, testResult{name: name, passed: false, err: msg})
	}
}

func printResults() {
	passed, failed := 0, 0
	for _, r := range results {
		if r.passed {
			fmt.Printf("  PASS: %s\n", r.name)
			passed++
		} else {
			fmt.Printf("  FAIL: %s -- %s\n", r.name, r.err)
			failed++
		}
	}
	fmt.Printf("\n  Results: %d passed, %d failed, %d total\n", passed, failed, passed+failed)
}

// =============================================================================
// Tests
// =============================================================================

func testCreateUser_Success() {
	svc, repo, mailer := setupTestService()
	ctx := context.Background()

	user, err := svc.CreateUser(ctx, "Alice", "alice@example.com")

	assert("CreateUser returns no error", err == nil,
		fmt.Sprintf("unexpected error: %v", err))
	assert("CreateUser returns user with correct name", user != nil && user.Name == "Alice",
		"user name mismatch")
	assert("CreateUser returns user with correct email", user != nil && user.Email == "alice@example.com",
		"user email mismatch")
	assert("CreateUser returns active user", user != nil && user.Active,
		"user should be active")

	// Verify side effects through the fakes
	assert("User was saved to repo", len(repo.Saved) == 1,
		fmt.Sprintf("expected 1 save, got %d", len(repo.Saved)))
	assert("Welcome email was sent", len(mailer.Sent) == 1,
		fmt.Sprintf("expected 1 email, got %d", len(mailer.Sent)))
	if len(mailer.Sent) > 0 {
		assert("Email sent to correct address", mailer.Sent[0].To == "alice@example.com",
			fmt.Sprintf("email sent to %s", mailer.Sent[0].To))
		assert("Email has welcome subject", mailer.Sent[0].Subject == "Welcome!",
			fmt.Sprintf("subject was %s", mailer.Sent[0].Subject))
	}
}

func testCreateUser_DuplicateEmail() {
	svc, repo, _ := setupTestService()
	ctx := context.Background()

	repo.Seed(&User{ID: "existing", Email: "taken@example.com", Name: "Existing"})

	_, err := svc.CreateUser(ctx, "New User", "taken@example.com")

	assert("Duplicate email returns error", err != nil, "expected error for duplicate email")
	assert("Error mentions existing email",
		err != nil && strings.Contains(err.Error(), "already exists"),
		fmt.Sprintf("error should mention 'already exists': %v", err))
}

func testCreateUser_RepoFailure() {
	svc, repo, mailer := setupTestService()
	ctx := context.Background()

	repo.SaveErr = errors.New("connection refused")

	_, err := svc.CreateUser(ctx, "Alice", "alice@example.com")

	assert("Repo failure returns error", err != nil, "expected error when repo fails")
	assert("Error wraps repo error",
		err != nil && strings.Contains(err.Error(), "connection refused"),
		fmt.Sprintf("should contain repo error: %v", err))
	assert("No email sent on repo failure", len(mailer.Sent) == 0,
		"should not send email if save failed")
}

func testCreateUser_EmailFailureDoesNotBlockCreation() {
	svc, _, mailer := setupTestService()
	ctx := context.Background()

	mailer.SendErr = errors.New("SMTP timeout")

	user, err := svc.CreateUser(ctx, "Alice", "alice@example.com")

	assert("User created despite email failure", err == nil,
		fmt.Sprintf("user creation should succeed: %v", err))
	assert("User returned despite email failure", user != nil,
		"should return user even if email fails")
}

func testGetUser_Found() {
	svc, repo, _ := setupTestService()
	ctx := context.Background()

	repo.Seed(&User{ID: "u1", Email: "alice@example.com", Name: "Alice", Active: true})

	user, err := svc.GetUser(ctx, "u1")

	assert("GetUser returns no error", err == nil,
		fmt.Sprintf("unexpected error: %v", err))
	assert("GetUser returns correct user", user != nil && user.Name == "Alice",
		"wrong user returned")
}

func testGetUser_NotFound() {
	svc, _, _ := setupTestService()
	ctx := context.Background()

	_, err := svc.GetUser(ctx, "nonexistent")

	assert("GetUser returns error for missing user", err != nil,
		"expected error for nonexistent user")
}

func testDeactivateUser_Success() {
	svc, repo, mailer := setupTestService()
	ctx := context.Background()

	repo.Seed(&User{ID: "u1", Email: "bob@example.com", Name: "Bob", Active: true})

	err := svc.DeactivateUser(ctx, "u1")

	assert("DeactivateUser returns no error", err == nil,
		fmt.Sprintf("unexpected error: %v", err))

	// Verify the user was actually deactivated
	user, _ := repo.FindByID(ctx, "u1")
	assert("User is now inactive", user != nil && !user.Active,
		"user should be inactive after deactivation")

	// Verify notification email was sent
	assert("Deactivation email was sent", len(mailer.Sent) == 1,
		fmt.Sprintf("expected 1 email, got %d", len(mailer.Sent)))
	if len(mailer.Sent) > 0 {
		assert("Deactivation email subject",
			strings.Contains(mailer.Sent[0].Subject, "Deactivated"),
			fmt.Sprintf("subject: %s", mailer.Sent[0].Subject))
	}
}

func testDeactivateUser_NotFound() {
	svc, _, _ := setupTestService()
	ctx := context.Background()

	err := svc.DeactivateUser(ctx, "nonexistent")

	assert("DeactivateUser errors for missing user", err != nil,
		"expected error for nonexistent user")
}

// =============================================================================
// Main -- run all tests
// =============================================================================

func main() {
	fmt.Println("=== Dependency Injection: Testing with Fakes and Spies ===")
	fmt.Println(strings.Repeat("=", 55))

	fmt.Println("\n--- TestCreateUser_Success ---")
	testCreateUser_Success()

	fmt.Println("\n--- TestCreateUser_DuplicateEmail ---")
	testCreateUser_DuplicateEmail()

	fmt.Println("\n--- TestCreateUser_RepoFailure ---")
	testCreateUser_RepoFailure()

	fmt.Println("\n--- TestCreateUser_EmailFailureDoesNotBlockCreation ---")
	testCreateUser_EmailFailureDoesNotBlockCreation()

	fmt.Println("\n--- TestGetUser_Found ---")
	testGetUser_Found()

	fmt.Println("\n--- TestGetUser_NotFound ---")
	testGetUser_NotFound()

	fmt.Println("\n--- TestDeactivateUser_Success ---")
	testDeactivateUser_Success()

	fmt.Println("\n--- TestDeactivateUser_NotFound ---")
	testDeactivateUser_NotFound()

	fmt.Println(strings.Repeat("=", 55))
	printResults()

	fmt.Println(strings.Repeat("=", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- setupTestService() is the test composition root (replaces main())")
	fmt.Println("- FakeUserRepo is a real working implementation, not just stubs")
	fmt.Println("- SpyEmailSender records calls for later assertion")
	fmt.Println("- Error injection (SaveErr, SendErr) tests failure paths")
	fmt.Println("- Seed() pre-populates the fake for specific test scenarios")
	fmt.Println("- The UserService code is identical in tests and production")
	fmt.Println("- No mocking framework needed -- plain Go structs")
}
