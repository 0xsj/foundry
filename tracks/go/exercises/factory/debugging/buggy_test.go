package main

import (
	"sync"
	"testing"
)

// =============================================================
// Bug 1: All services resolve to the same config
// The loop variable is captured by pointer, so all entries
// point to the last iteration's value.
// =============================================================

func TestResolveReturnsCorrectService(t *testing.T) {
	reg := NewRegistry()

	configs := []ServiceConfig{
		{Name: "auth", Host: "localhost", Port: 8001, Protocol: "http"},
		{Name: "billing", Host: "localhost", Port: 8002, Protocol: "tcp"},
		{Name: "notifications", Host: "localhost", Port: 8003, Protocol: "http"},
	}
	reg.Register(configs)

	tests := []struct {
		name         string
		expectedPort int
	}{
		{"auth", 8001},
		{"billing", 8002},
		{"notifications", 8003},
	}

	for _, tt := range tests {
		cfg, err := reg.Resolve(tt.name)
		if err != nil {
			t.Errorf("Resolve(%q): unexpected error: %v", tt.name, err)
			continue
		}
		if cfg.Port != tt.expectedPort {
			t.Errorf("Resolve(%q): expected port %d, got %d", tt.name, tt.expectedPort, cfg.Port)
		}
		if cfg.Name != tt.name {
			t.Errorf("Resolve(%q): expected name %q, got %q", tt.name, tt.name, cfg.Name)
		}
	}
}

// =============================================================
// Bug 2: Create returns nil for unknown service (causes panic)
// Should return an error, not a nil interface.
// =============================================================

func TestCreateUnknownService(t *testing.T) {
	reg := NewRegistry()

	// Should not panic -- should return nil, error
	svc, err := func() (svc Service, err error) {
		defer func() {
			if r := recover(); r != nil {
				err = r.(error)
			}
		}()
		svc = reg.Create("nonexistent")
		// If Create returned nil without error, calling methods will panic
		if svc != nil {
			_ = svc.Name() // This would panic on nil
		}
		return svc, nil
	}()

	// The fix should make Create return (Service, error) and handle unknown services
	// For now, we just check it doesn't panic
	if err != nil {
		t.Logf("Create panicked (expected before fix): %v", err)
	}
	if svc != nil {
		t.Log("Create returned a non-nil service for unknown name")
	}

	// After fixing, this should work:
	// svc, err := reg.Create("nonexistent")
	// if err == nil {
	//     t.Error("expected error for unknown service")
	// }
}

func TestCreateReturnsErrorForUnknown(t *testing.T) {
	reg := NewRegistry()

	// This test validates the fix: Create should return (Service, error)
	// Before the fix, this won't compile because Create returns only Service.
	// After fixing Create's signature to return (Service, error):
	svc, err := reg.Create("nonexistent")
	if err == nil {
		t.Error("expected error for unknown service")
	}
	if svc != nil {
		t.Error("expected nil service for unknown name")
	}
}

// =============================================================
// Bug 3: Data race on concurrent registration
// Multiple goroutines writing to the map without synchronization.
// =============================================================

func TestConcurrentRegistration(t *testing.T) {
	reg := NewRegistry()

	var wg sync.WaitGroup

	// Register services from multiple goroutines
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			reg.Register([]ServiceConfig{
				{
					Name:     fmt.Sprintf("service-%d", id),
					Host:     "localhost",
					Port:     8000 + id,
					Protocol: "http",
				},
			})
		}(i)
	}

	wg.Wait()

	// All 10 services should be registered
	for i := 0; i < 10; i++ {
		name := fmt.Sprintf("service-%d", i)
		cfg, err := reg.Resolve(name)
		if err != nil {
			t.Errorf("Resolve(%q): %v", name, err)
			continue
		}
		if cfg.Port != 8000+i {
			t.Errorf("Resolve(%q): expected port %d, got %d", name, 8000+i, cfg.Port)
		}
	}
}

func TestConcurrentResolve(t *testing.T) {
	reg := NewRegistry()
	reg.Register([]ServiceConfig{
		{Name: "api", Host: "localhost", Port: 8080, Protocol: "http"},
	})

	var wg sync.WaitGroup

	// Concurrent reads should not race with each other
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			cfg, err := reg.Resolve("api")
			if err != nil {
				t.Errorf("Resolve: %v", err)
				return
			}
			if cfg.Port != 8080 {
				t.Errorf("expected port 8080, got %d", cfg.Port)
			}
		}()
	}

	wg.Wait()
}

// =============================================================
// Bug 4: Create returns wrong concrete type (TCPService for HTTP)
// This causes type assertion panics when GetHTTPService is called.
// =============================================================

func TestGetHTTPService(t *testing.T) {
	reg := NewRegistry()
	reg.Register([]ServiceConfig{
		{Name: "api-gateway", Host: "localhost", Port: 8080, Protocol: "http"},
	})

	httpSvc, err := reg.GetHTTPService("api-gateway")
	if err != nil {
		t.Fatalf("GetHTTPService: %v", err)
	}

	if httpSvc.BaseURL() != "http://localhost:8080/api/v1" {
		t.Errorf("expected base URL http://localhost:8080/api/v1, got %s", httpSvc.BaseURL())
	}
}

func TestCreateReturnsCorrectType(t *testing.T) {
	reg := NewRegistry()
	reg.Register([]ServiceConfig{
		{Name: "web", Host: "localhost", Port: 80, Protocol: "http"},
		{Name: "db", Host: "localhost", Port: 5432, Protocol: "tcp"},
	})

	webSvc, err := reg.Create("web")
	if err != nil {
		t.Fatalf("Create(web): %v", err)
	}
	if _, ok := webSvc.(*HTTPService); !ok {
		t.Errorf("expected *HTTPService for protocol=http, got %T", webSvc)
	}

	dbSvc, err := reg.Create("db")
	if err != nil {
		t.Fatalf("Create(db): %v", err)
	}
	if _, ok := dbSvc.(*TCPService); !ok {
		t.Errorf("expected *TCPService for protocol=tcp, got %T", dbSvc)
	}
}

// =============================================================
// Missing import for fmt (needed by concurrent test)
// =============================================================

import "fmt"
