// Package api — test suite under review.
// This code has several issues. Review it before reading expert-review.md.
package api

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// ============================================================================
// Mock: StrictUserStore
//
// This mock tracks exactly which methods were called and what arguments were used.
// It panics if an unexpected method is called.
// ============================================================================

type StrictUserStore struct {
	t           *testing.T
	getExpected string // only this ID is valid
	getReturn   *User
	getErr      error
	createCalls int
}

func (s *StrictUserStore) GetUser(id string) (*User, error) {
	s.t.Helper()
	if id != s.getExpected {
		// Panic if called with unexpected argument — very strict
		s.t.Fatalf("GetUser called with unexpected id: got %q, want %q", id, s.getExpected)
	}
	return s.getReturn, s.getErr
}

func (s *StrictUserStore) CreateUser(u *User) error {
	s.createCalls++
	// Verify the exact internal ID format — this breaks if generateID changes
	if u.ID != "usr_abc123" {
		s.t.Errorf("CreateUser: unexpected ID format %q", u.ID)
	}
	return nil
}

// ============================================================================
// Tests
// ============================================================================

func TestGetUser_Found(t *testing.T) {
	store := &StrictUserStore{
		t:           t,
		getExpected: "user-1",
		getReturn:   &User{ID: "user-1", Email: "alice@example.com", Name: "Alice"},
	}
	handler := NewUserHandler(store)

	req := httptest.NewRequest("GET", "/users/user-1", nil)
	req.SetPathValue("id", "user-1")
	w := httptest.NewRecorder()

	handler.GetUser(w, req)

	// Only checks the status code — not the response body content
	if w.Code != http.StatusOK {
		t.Errorf("status = %d, want %d", w.Code, http.StatusOK)
	}
}

func TestGetUser_NotFound(t *testing.T) {
	store := &StrictUserStore{
		t:           t,
		getExpected: "user-999",
		getErr:      ErrNotFound,
	}
	handler := NewUserHandler(store)

	req := httptest.NewRequest("GET", "/users/user-999", nil)
	req.SetPathValue("id", "user-999")
	w := httptest.NewRecorder()

	handler.GetUser(w, req)

	if w.Code != http.StatusNotFound {
		t.Errorf("status = %d, want %d", w.Code, http.StatusNotFound)
	}
}

func TestGetUser_StoreError(t *testing.T) {
	store := &StrictUserStore{
		t:           t,
		getExpected: "user-1",
		getErr:      &internalError{"db connection refused"},
	}
	handler := NewUserHandler(store)

	req := httptest.NewRequest("GET", "/users/user-1", nil)
	req.SetPathValue("id", "user-1")
	w := httptest.NewRecorder()

	handler.GetUser(w, req)

	if w.Code != http.StatusInternalServerError {
		t.Errorf("status = %d, want %d", w.Code, http.StatusInternalServerError)
	}
}

type internalError struct{ msg string }

func (e *internalError) Error() string { return e.msg }

func TestCreateUser_Success(t *testing.T) {
	store := &StrictUserStore{t: t}
	handler := NewUserHandler(store)

	body := strings.NewReader(`{"email":"bob@example.com","name":"Bob"}`)
	req := httptest.NewRequest("POST", "/users", body)
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	handler.CreateUser(w, req)

	if w.Code != http.StatusCreated {
		t.Errorf("status = %d, want %d", w.Code, http.StatusCreated)
	}
}

func TestCreateUser_MissingEmail(t *testing.T) {
	store := &StrictUserStore{t: t}
	handler := NewUserHandler(store)

	body := strings.NewReader(`{"name":"Bob"}`)
	req := httptest.NewRequest("POST", "/users", body)
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	handler.CreateUser(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("status = %d, want %d", w.Code, http.StatusBadRequest)
	}
}

func TestCreateUser_MissingName(t *testing.T) {
	store := &StrictUserStore{t: t}
	handler := NewUserHandler(store)

	body := strings.NewReader(`{"email":"bob@example.com"}`)
	req := httptest.NewRequest("POST", "/users", body)
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	handler.CreateUser(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("status = %d, want %d", w.Code, http.StatusBadRequest)
	}
}

func TestCreateUser_InvalidJSON(t *testing.T) {
	store := &StrictUserStore{t: t}
	handler := NewUserHandler(store)

	body := strings.NewReader(`not json`)
	req := httptest.NewRequest("POST", "/users", body)
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	handler.CreateUser(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("status = %d, want %d", w.Code, http.StatusBadRequest)
	}
}
