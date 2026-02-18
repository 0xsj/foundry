// Range-Over-Function Iterators (Go 1.23+)
//
// Demonstrates iter.Seq, iter.Seq2, and composable iterator pipelines.
// Practical examples include filtering, mapping, chaining, and a paginated
// API result iterator.
//
// Run: go run ./rangefunc/

package main

import (
	"fmt"
	"iter"
	"slices"
	"strings"
)

// --- Core Iterator Combinators ---

// filter keeps elements where pred returns true.
func filter[T any](pred func(T) bool, seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		for v := range seq {
			if pred(v) {
				if !yield(v) {
					return
				}
			}
		}
	}
}

// mapIter transforms each element using f.
func mapIter[T, U any](f func(T) U, seq iter.Seq[T]) iter.Seq[U] {
	return func(yield func(U) bool) {
		for v := range seq {
			if !yield(f(v)) {
				return
			}
		}
	}
}

// take yields at most n elements from seq.
func take[T any](n int, seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		i := 0
		for v := range seq {
			if i >= n {
				return
			}
			if !yield(v) {
				return
			}
			i++
		}
	}
}

// skip discards the first n elements, then yields the rest.
func skip[T any](n int, seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		i := 0
		for v := range seq {
			if i < n {
				i++
				continue
			}
			if !yield(v) {
				return
			}
		}
	}
}

// enumerate wraps an iter.Seq[T] into iter.Seq2[int, T] with an index.
func enumerate[T any](seq iter.Seq[T]) iter.Seq2[int, T] {
	return func(yield func(int, T) bool) {
		i := 0
		for v := range seq {
			if !yield(i, v) {
				return
			}
			i++
		}
	}
}

// chain concatenates multiple iterators into one.
func chain[T any](seqs ...iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		for _, seq := range seqs {
			for v := range seq {
				if !yield(v) {
					return
				}
			}
		}
	}
}

// reduce folds all elements into a single value.
func reduce[T, U any](seq iter.Seq[T], initial U, f func(U, T) U) U {
	acc := initial
	for v := range seq {
		acc = f(acc, v)
	}
	return acc
}

// --- Practical Iterators ---

// naturals returns an infinite sequence of natural numbers starting at 0.
func naturals() iter.Seq[int] {
	return func(yield func(int) bool) {
		n := 0
		for {
			if !yield(n) {
				return
			}
			n++
		}
	}
}

// fibonacci returns an infinite sequence of Fibonacci numbers.
func fibonacci() iter.Seq[int] {
	return func(yield func(int) bool) {
		a, b := 0, 1
		for {
			if !yield(a) {
				return
			}
			a, b = b, a+b
		}
	}
}

// --- Simulated Paginated API ---

// User represents a user record from an API.
type User struct {
	ID    int
	Name  string
	Email string
	Role  string
}

// FakeAPI simulates a paginated REST API with 47 users.
type FakeAPI struct {
	users []User
}

func NewFakeAPI() *FakeAPI {
	roles := []string{"admin", "editor", "viewer"}
	users := make([]User, 47) // 47 users, awkward page boundary
	for i := range users {
		users[i] = User{
			ID:    i + 1,
			Name:  fmt.Sprintf("user-%d", i+1),
			Email: fmt.Sprintf("user%d@example.com", i+1),
			Role:  roles[i%len(roles)],
		}
	}
	return &FakeAPI{users: users}
}

// ListUsers returns a page of users. cursor is the 0-based offset.
// Returns the page, next cursor ("" if no more), and error.
func (api *FakeAPI) ListUsers(cursor int, pageSize int) ([]User, int, error) {
	if cursor >= len(api.users) {
		return nil, 0, nil
	}
	end := cursor + pageSize
	if end > len(api.users) {
		end = len(api.users)
	}
	page := api.users[cursor:end]
	nextCursor := end
	if nextCursor >= len(api.users) {
		nextCursor = -1 // signals "no more pages"
	}
	return page, nextCursor, nil
}

// allUsers returns a lazy iterator over all users, transparently paginating.
// Pages are fetched only as needed -- if the consumer breaks after 5 users,
// only the first page is fetched.
func allUsers(api *FakeAPI, pageSize int) iter.Seq2[User, error] {
	return func(yield func(User, error) bool) {
		cursor := 0
		for {
			page, nextCursor, err := api.ListUsers(cursor, pageSize)
			if err != nil {
				yield(User{}, err)
				return
			}
			if len(page) == 0 {
				return
			}
			for _, user := range page {
				if !yield(user, nil) {
					return // consumer stopped -- no more API calls
				}
			}
			if nextCursor < 0 {
				return // no more pages
			}
			cursor = nextCursor
		}
	}
}

// --- Pull-based iteration with iter.Pull ---

func demonstratePull() {
	fmt.Println("=== iter.Pull: Converting Push to Pull ===")

	// Create a push-based iterator
	seq := slices.Values([]string{"alpha", "bravo", "charlie", "delta"})

	// Convert to pull-based
	next, stop := iter.Pull(seq)
	defer stop() // ALWAYS call stop

	// Pull values one at a time
	for {
		v, ok := next()
		if !ok {
			break
		}
		fmt.Printf("  pulled: %s\n", v)
	}
	fmt.Println()
}

func main() {
	fmt.Println("=== Basic Range-Over-Function ===")
	fmt.Print("  naturals (first 8): ")
	for v := range take(8, naturals()) {
		fmt.Printf("%d ", v)
	}
	fmt.Println()

	fmt.Print("  fibonacci (first 10): ")
	for v := range take(10, fibonacci()) {
		fmt.Printf("%d ", v)
	}
	fmt.Println()
	fmt.Println()

	// --- Composition: filter, map, take ---
	fmt.Println("=== Composing Iterators ===")

	// First 5 even Fibonacci numbers
	evenFibs := take(5, filter(func(n int) bool { return n%2 == 0 }, fibonacci()))
	fmt.Print("  first 5 even Fibonacci: ")
	for v := range evenFibs {
		fmt.Printf("%d ", v)
	}
	fmt.Println()

	// Squares of numbers 10-19
	squares := mapIter(func(n int) int { return n * n }, take(10, skip(10, naturals())))
	fmt.Print("  squares of 10..19: ")
	for v := range squares {
		fmt.Printf("%d ", v)
	}
	fmt.Println()
	fmt.Println()

	// --- Enumerate ---
	fmt.Println("=== Enumerate ===")
	words := slices.Values([]string{"hello", "world", "from", "iterators"})
	for i, w := range enumerate(words) {
		fmt.Printf("  [%d] %s\n", i, w)
	}
	fmt.Println()

	// --- Chain ---
	fmt.Println("=== Chain Multiple Iterators ===")
	seq1 := slices.Values([]string{"a", "b", "c"})
	seq2 := slices.Values([]string{"x", "y", "z"})
	seq3 := slices.Values([]string{"1", "2", "3"})
	combined := chain(seq1, seq2, seq3)
	fmt.Print("  chained: ")
	for v := range combined {
		fmt.Printf("%s ", v)
	}
	fmt.Println()
	fmt.Println()

	// --- Reduce ---
	fmt.Println("=== Reduce ===")
	sum := reduce(take(100, naturals()), 0, func(acc, v int) int { return acc + v })
	fmt.Printf("  sum of 0..99: %d\n", sum)

	// Concatenate with reduce
	words2 := slices.Values([]string{"Go", "iterators", "are", "powerful"})
	sentence := reduce(words2, "", func(acc, v string) string {
		if acc == "" {
			return v
		}
		return acc + " " + v
	})
	fmt.Printf("  sentence: %q\n", sentence)
	fmt.Println()

	// --- Collecting into a slice ---
	fmt.Println("=== Collect Into Slice ===")
	first20Fibs := slices.Collect(take(20, fibonacci()))
	fmt.Printf("  first 20 Fibonacci: %v\n", first20Fibs)
	fmt.Println()

	// --- Paginated API ---
	fmt.Println("=== Paginated API Iterator ===")
	api := NewFakeAPI()

	// Get all admin users -- pagination is transparent
	adminCount := 0
	for user, err := range allUsers(api, 10) {
		if err != nil {
			fmt.Printf("  error: %v\n", err)
			break
		}
		if user.Role == "admin" {
			adminCount++
			if adminCount <= 5 {
				fmt.Printf("  admin: %s (%s)\n", user.Name, user.Email)
			}
		}
	}
	fmt.Printf("  total admins found: %d\n", adminCount)
	fmt.Println()

	// Get first 3 editors -- stops after first page is partially consumed
	fmt.Println("  first 3 editors:")
	editorCount := 0
	for user, err := range allUsers(api, 10) {
		if err != nil {
			break
		}
		if user.Role == "editor" {
			fmt.Printf("    %s (%s)\n", user.Name, user.Email)
			editorCount++
			if editorCount >= 3 {
				break // stops iteration -- no more API pages fetched
			}
		}
	}
	fmt.Println()

	// --- iter.Pull ---
	demonstratePull()

	// --- Practical: processing log-like data ---
	fmt.Println("=== Log Processing Pipeline ===")
	logLines := slices.Values([]string{
		"2024-01-15T10:00:00Z INFO  server started on :8080",
		"2024-01-15T10:00:01Z DEBUG connection pool initialized",
		"2024-01-15T10:00:05Z ERROR failed to connect to redis: dial timeout",
		"2024-01-15T10:00:06Z WARN  retry attempt 1 for redis connection",
		"2024-01-15T10:00:07Z ERROR redis connection failed after 3 retries",
		"2024-01-15T10:00:10Z INFO  fallback cache activated",
		"2024-01-15T10:01:00Z DEBUG health check passed",
		"2024-01-15T10:02:00Z ERROR disk usage above 90%: /var/log at 94%",
	})

	// Pipeline: keep only ERROR lines, extract the message part
	errors := mapIter(
		func(line string) string {
			// Extract message after level
			parts := strings.SplitN(line, " ", 4)
			if len(parts) >= 4 {
				return parts[3]
			}
			return line
		},
		filter(
			func(line string) bool {
				return strings.Contains(line, "ERROR")
			},
			logLines,
		),
	)

	fmt.Println("  error messages:")
	for msg := range errors {
		fmt.Printf("    - %s\n", msg)
	}

	fmt.Println()
	fmt.Println("Key takeaways:")
	fmt.Println("- iter.Seq[T] and iter.Seq2[K,V] are the standard iterator types")
	fmt.Println("- Combinators (filter, map, take, skip) compose without allocations")
	fmt.Println("- Infinite sequences are fine -- the consumer decides when to stop")
	fmt.Println("- Paginated APIs become flat, lazy sequences")
	fmt.Println("- iter.Pull converts push iterators to pull-based when needed")
	fmt.Println("- slices.Collect, slices.Values bridge between slices and iterators")
}
