# Expert Review: HTTP Handler Tests PR

## Critical Issues

### 1. `StrictUserStore` tests implementation details, not behavior

**Location:** `StrictUserStore.CreateUser` — checks `u.ID != "usr_abc123"`

```go
func (s *StrictUserStore) CreateUser(u *User) error {
    s.createCalls++
    if u.ID != "usr_abc123" {
        s.t.Errorf("CreateUser: unexpected ID format %q", u.ID)
    }
    return nil
}
```

This mock verifies the exact internal ID value (`"usr_abc123"`) produced by `generateID()`. That's an implementation detail — the test knows the internals of a private helper function that the public API makes no guarantees about.

**Why it's a problem:** If the team changes `generateID()` to use real UUIDs (the right thing to do), every test using `StrictUserStore` breaks — not because `CreateUser` is broken, but because the test was asserting internal behavior. The test should verify the *contract*: that `CreateUser` was called with a user that has a non-empty ID and the correct email/name from the request.

**Better approach:**
```go
func (s *FakeUserStore) CreateUser(u *User) error {
    if u.ID == "" {
        s.t.Errorf("CreateUser: user has empty ID")
    }
    if u.Email == "" || u.Name == "" {
        s.t.Errorf("CreateUser: user missing email or name")
    }
    s.created = append(s.created, u)
    return nil
}
```

This verifies the contract (ID is non-empty, required fields are set) without locking in a specific ID string.

---

### 2. `TestGetUser_Found` doesn't verify the response body

**Location:** `TestGetUser_Found`

```go
if w.Code != http.StatusOK {
    t.Errorf("status = %d, want %d", w.Code, http.StatusOK)
}
// That's it — no body check
```

The test verifies the status code but not the response body. `GetUser` could return `200 OK` with `{"id":"wrong-user","email":""}` and this test would pass. Status codes are necessary but not sufficient. HTTP handler tests should verify the full contract: status code + relevant body content + critical headers.

**Better approach:**
```go
if w.Code != http.StatusOK {
    t.Fatalf("status = %d, want %d", w.Code, http.StatusOK)
}
if ct := w.Header().Get("Content-Type"); !strings.Contains(ct, "application/json") {
    t.Errorf("Content-Type = %q, want application/json", ct)
}
var got User
if err := json.NewDecoder(w.Body).Decode(&got); err != nil {
    t.Fatalf("decode response: %v", err)
}
if got.ID != "user-1" {
    t.Errorf("user.ID = %q, want %q", got.ID, "user-1")
}
if got.Email != "alice@example.com" {
    t.Errorf("user.Email = %q, want %q", got.Email, "alice@example.com")
}
```

---

## Major Concerns

### 3. `StrictUserStore` panics with `t.Fatalf` on unexpected calls — fragile design

**Location:** `StrictUserStore.GetUser`

```go
func (s *StrictUserStore) GetUser(id string) (*User, error) {
    if id != s.getExpected {
        s.t.Fatalf("GetUser called with unexpected id: got %q, want %q", id, s.getExpected)
    }
    // ...
}
```

This mock panics (via `t.Fatalf`) if the handler calls `GetUser` with any ID other than the one pre-configured. The consequence: any refactor that changes how the handler reads the ID (e.g., switching from `PathValue` to a different routing approach) will cause a panic inside the mock instead of a clean test failure.

Mocks that crash on unexpected calls are useful when you're verifying exact call patterns. But for a handler that simply reads an ID from the request and passes it to the store, you want a **fake** that accepts any call and returns configured responses:

```go
type FakeUserStore struct {
    users  map[string]*User
    errors map[string]error
}

func (f *FakeUserStore) GetUser(id string) (*User, error) {
    if err, ok := f.errors[id]; ok {
        return nil, err
    }
    if u, ok := f.users[id]; ok {
        return u, nil
    }
    return nil, ErrNotFound
}
```

This is more flexible and survives refactors. The test verifies *what the handler returns to the client*, not *how it called the store*.

---

### 4. Six individual test functions instead of a table test

**Location:** `TestGetUser_Found`, `TestGetUser_NotFound`, `TestGetUser_StoreError`, `TestCreateUser_Success`, `TestCreateUser_MissingEmail`, `TestCreateUser_MissingName`, `TestCreateUser_InvalidJSON`

Each test function is nearly identical boilerplate: create store, create handler, create request, record, call handler, check status. Adding another test case requires duplicating all of that.

The idiomatic Go approach is a table-driven test:

```go
func TestGetUser(t *testing.T) {
    users := map[string]*User{
        "user-1": {ID: "user-1", Email: "alice@example.com", Name: "Alice"},
    }
    store := newFakeStore(users)
    handler := NewUserHandler(store)

    tests := []struct {
        name       string
        userID     string
        wantStatus int
        wantEmail  string
    }{
        {name: "found", userID: "user-1", wantStatus: 200, wantEmail: "alice@example.com"},
        {name: "not found", userID: "user-999", wantStatus: 404},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            req := httptest.NewRequest("GET", "/users/"+tt.userID, nil)
            req.SetPathValue("id", tt.userID)
            w := httptest.NewRecorder()
            handler.GetUser(w, req)
            // ...assertions...
        })
    }
}
```

---

## Minor Suggestions

### 5. No test for missing `id` path value (the 400 path)

`GetUser` has an explicit 400 case when `id == ""`, but there's no test for it. This is a small gap — easy to add to a table test.

### 6. `t.Helper()` missing on `StrictUserStore` methods

The mock calls `t.Fatalf` and `t.Errorf` but doesn't call `t.Helper()`. Failures report at the wrong line (inside the mock, not at the test that triggered the behavior).

### 7. Response body could use `t.Fatal` instead of `t.Error` when body decoding fails

If you add body assertions, use `t.Fatalf` when the body can't be decoded — checking fields of a zero-value struct gives misleading follow-on errors.

---

## Positive Feedback

- Using `httptest.NewRequest` and `httptest.NewRecorder` is correct — the standard Go approach for testing HTTP handlers without starting a real server
- `req.SetPathValue("id", ...)` is correctly used for Go 1.22's built-in routing
- Covering the error path (`ErrNotFound` → 404, store error → 500) shows good instinct for edge cases
- Separate tests per endpoint (`GetUser` vs `CreateUser`) is reasonable organization

---

## Summary

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | Critical | `StrictUserStore.CreateUser` | Asserts internal implementation detail (exact ID string) |
| 2 | Critical | `TestGetUser_Found` | Status code only — response body not verified |
| 3 | Major | `StrictUserStore.GetUser` | Over-strict mock panics on unexpected args — breaks on refactor |
| 4 | Major | All tests | Six individual functions instead of table-driven tests |
| 5 | Minor | `GetUser` tests | Missing test for empty ID path value (400 case) |
| 6 | Minor | `StrictUserStore` | `t.Helper()` missing — failures point to wrong line |
| 7 | Minor | Future body checks | Use `t.Fatalf` on decode failure to prevent misleading follow-on errors |

## The Core Lesson

**Test behavior, not implementation.** A handler test should verify:
- "Given this request, what status code does the client get?"
- "Given this request, what body does the client get?"
- "Given this request, what headers does the client get?"

It should NOT verify:
- "What exact internal ID format was passed to the store?"
- "In what order were store methods called?"
- "Was this exact private helper function invoked?"

Mocks that verify exact internal call signatures are useful for services with complex interaction protocols (e.g., transaction management, saga coordination). For a simple CRUD handler, a **fake** with real-ish behavior and **behavior assertions** (what did the client receive?) is more robust and less brittle.
