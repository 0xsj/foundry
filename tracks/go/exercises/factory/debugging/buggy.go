package main

import (
	"fmt"
)

// =============================================================
// Service types
// =============================================================

// ServiceConfig holds configuration for a service.
type ServiceConfig struct {
	Name     string
	Host     string
	Port     int
	Protocol string // "http" or "tcp"
}

// Service is the interface all service clients implement.
type Service interface {
	Name() string
	Address() string
	Ping() error
}

// HTTPService is a service accessible over HTTP.
type HTTPService struct {
	config ServiceConfig
}

func (s *HTTPService) Name() string    { return s.config.Name }
func (s *HTTPService) Address() string { return fmt.Sprintf("http://%s:%d", s.config.Host, s.config.Port) }
func (s *HTTPService) Ping() error     { return nil }
func (s *HTTPService) BaseURL() string { return s.Address() + "/api/v1" }

// TCPService is a service accessible over raw TCP.
type TCPService struct {
	config ServiceConfig
}

func (s *TCPService) Name() string    { return s.config.Name }
func (s *TCPService) Address() string { return fmt.Sprintf("%s:%d", s.config.Host, s.config.Port) }
func (s *TCPService) Ping() error     { return nil }

// =============================================================
// Service Registry
// =============================================================

// Registry stores service configurations and creates service clients.
type Registry struct {
	services map[string]*ServiceConfig
}

// NewRegistry creates a new service registry.
func NewRegistry() *Registry {
	return &Registry{
		services: make(map[string]*ServiceConfig),
	}
}

// Register adds a service configuration to the registry.
//
// BUG 1: Pointer to loop variable
// BUG 3: No mutex protection
func (r *Registry) Register(configs []ServiceConfig) {
	for _, cfg := range configs {
		r.services[cfg.Name] = &cfg
	}
}

// Resolve looks up a service by name and returns its config.
func (r *Registry) Resolve(name string) (*ServiceConfig, error) {
	cfg, ok := r.services[name]
	if !ok {
		return nil, fmt.Errorf("service not found: %s", name)
	}
	return cfg, nil
}

// Create instantiates a Service client from the registry.
//
// BUG 2: Returns nil interface without error for unknown service
// BUG 4: Creates wrong concrete type
func (r *Registry) Create(name string) Service {
	cfg, _ := r.Resolve(name)

	switch cfg.Protocol {
	case "http":
		return &TCPService{config: *cfg}
	case "tcp":
		return &TCPService{config: *cfg}
	default:
		return &TCPService{config: *cfg}
	}
}

// GetHTTPService looks up a service and asserts it's an HTTP service.
func (r *Registry) GetHTTPService(name string) (*HTTPService, error) {
	svc := r.Create(name)
	httpSvc := svc.(*HTTPService) // will panic if not HTTPService
	return httpSvc, nil
}

func main() {
	reg := NewRegistry()

	configs := []ServiceConfig{
		{Name: "auth", Host: "localhost", Port: 8001, Protocol: "http"},
		{Name: "billing", Host: "localhost", Port: 8002, Protocol: "tcp"},
		{Name: "notifications", Host: "localhost", Port: 8003, Protocol: "http"},
	}

	reg.Register(configs)

	// Should print different addresses for each service
	for _, name := range []string{"auth", "billing", "notifications"} {
		cfg, err := reg.Resolve(name)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			continue
		}
		fmt.Printf("%s -> %s:%d\n", name, cfg.Host, cfg.Port)
	}
}
