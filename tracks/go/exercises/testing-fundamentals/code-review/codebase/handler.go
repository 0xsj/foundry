// Package api handles HTTP requests for the user service.
package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
)

// UserStore retrieves and persists user records.
type UserStore interface {
	GetUser(id string) (*User, error)
	CreateUser(u *User) error
}

// User is the domain object.
type User struct {
	ID    string `json:"id"`
	Email string `json:"email"`
	Name  string `json:"name"`
}

// ErrNotFound is returned by UserStore when a user does not exist.
var ErrNotFound = errors.New("user not found")

// UserHandler handles user-related HTTP requests.
type UserHandler struct {
	store UserStore
}

// NewUserHandler creates a handler backed by the given store.
func NewUserHandler(store UserStore) *UserHandler {
	return &UserHandler{store: store}
}

// GetUser handles GET /users/{id}.
// Returns 200 with the user JSON on success.
// Returns 404 if the user does not exist.
// Returns 500 on any other store error.
func (h *UserHandler) GetUser(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id") // Go 1.22 routing
	if id == "" {
		http.Error(w, "missing user id", http.StatusBadRequest)
		return
	}

	user, err := h.store.GetUser(id)
	if err != nil {
		if errors.Is(err, ErrNotFound) {
			http.Error(w, "user not found", http.StatusNotFound)
			return
		}
		http.Error(w, "internal server error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(user); err != nil {
		// headers already sent — best effort
		http.Error(w, "encoding error", http.StatusInternalServerError)
	}
}

// CreateUser handles POST /users.
// Expects a JSON body with "email" and "name" fields.
// Returns 201 with the created user on success.
// Returns 400 for invalid JSON.
// Returns 500 on store error.
func (h *UserHandler) CreateUser(w http.ResponseWriter, r *http.Request) {
	var input struct {
		Email string `json:"email"`
		Name  string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}
	if input.Email == "" || input.Name == "" {
		http.Error(w, "email and name are required", http.StatusBadRequest)
		return
	}

	user := &User{
		ID:    fmt.Sprintf("usr_%s", generateID()),
		Email: input.Email,
		Name:  input.Name,
	}

	if err := h.store.CreateUser(user); err != nil {
		http.Error(w, "internal server error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(user) //nolint:errcheck
}

// generateID produces a unique identifier. In production this uses UUID.
// Simplified here for clarity.
func generateID() string {
	return "abc123"
}
