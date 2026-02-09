# Production Example — Configuration Management

## Overview

Configuration management is a fundamental pattern in production systems. This example shows how real projects handle environment-specific config with type safety, validation, and sensible defaults.

## Source

- **Project**: Kubernetes API Server
- **File**: `cmd/kube-apiserver/app/options/options.go`
- **Link**: [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes/blob/master/cmd/kube-apiserver/app/options/options.go)
- **License**: Apache 2.0
- **Context**: The API server is the central component of Kubernetes. Configuration determines how it listens, authenticates, authorizes, and stores data.

---

## The Code

```go
// ServerRunOptions contains state for master/api server
type ServerRunOptions struct {
	GenericServerRunOptions *genericoptions.ServerRunOptions
	Etcd                     *genericoptions.EtcdOptions
	SecureServing            *genericoptions.SecureServingOptionsWithLoopback
	Audit                    *genericoptions.AuditOptions
	Features                 *genericoptions.FeatureOptions
	Admission                *kubeoptions.AdmissionOptions
	Authentication           *kubeoptions.BuiltInAuthenticationOptions
	Authorization            *kubeoptions.BuiltInAuthorizationOptions
	// ... many more fields
}

// NewServerRunOptions creates a new ServerRunOptions object with default parameters
func NewServerRunOptions() *ServerRunOptions {
	s := ServerRunOptions{
		GenericServerRunOptions: genericoptions.NewServerRunOptions(),
		Etcd:                     genericoptions.NewEtcdOptions(storagebackend.NewDefaultConfig("/registry", nil)),
		SecureServing:            genericoptions.WithLoopback(genericoptions.NewSecureServingOptions()),
		Audit:                    genericoptions.NewAuditOptions(),
		Features:                 genericoptions.NewFeatureOptions(),
		Admission:                kubeoptions.NewAdmissionOptions(),
		Authentication:           kubeoptions.NewBuiltInAuthenticationOptions().WithAll(),
		Authorization:            kubeoptions.NewBuiltInAuthorizationOptions(),
		// ...
	}

	// Set defaults
	s.SecureServing.BindPort = 6443
	s.SecureServing.ServerCert.CertDirectory = "/var/run/kubernetes"

	return &s
}

// Validate checks ServerRunOptions and returns a slice of errors
func (s *ServerRunOptions) Validate() []error {
	var errors []error

	errors = append(errors, s.GenericServerRunOptions.Validate()...)
	errors = append(errors, s.Etcd.Validate()...)
	errors = append(errors, s.SecureServing.Validate()...)
	errors = append(errors, s.Audit.Validate()...)
	errors = append(errors, s.Admission.Validate()...)
	errors = append(errors, s.Authentication.Validate()...)
	errors = append(errors, s.Authorization.Validate()...)

	return errors
}

// Complete fills in fields that may be set by flags or auto-detection
func (s *ServerRunOptions) Complete() error {
	// Auto-detect external hostname if not set
	if s.GenericServerRunOptions.AdvertiseAddress == nil {
		s.GenericServerRunOptions.AdvertiseAddress = s.SecureServing.BindAddress
	}

	return s.Admission.ApplyTo(/* ... */)
}
```

---

## What Makes This Production-Grade

### 1. **Separation of Concerns**

Configuration is split into logical groups (`Etcd`, `SecureServing`, `Audit`, etc.). Each group is independently testable and reusable.

**Benefit**: New features can add their own config group without modifying the core struct.

### 2. **Constructor with Defaults**

`NewServerRunOptions()` provides sensible defaults for all fields. This ensures the system is runnable even with minimal configuration.

```go
s.SecureServing.BindPort = 6443  // Standard Kubernetes API port
```

**Benefit**: Reduces configuration burden. Users only override what they need.

### 3. **Validation as a Separate Step**

`Validate()` returns a slice of errors rather than panicking or returning a single error. This allows collecting **all** validation errors at once.

```go
if errors := options.Validate(); len(errors) > 0 {
    for _, err := range errors {
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
    }
    os.Exit(1)
}
```

**Benefit**: Users see all problems in one run, not one-at-a-time.

### 4. **Completion/Enrichment Phase**

`Complete()` handles auto-detection and cross-field dependencies. This is separate from validation.

**Why separate?**
- **Validation**: "Is the config valid?"
- **Completion**: "Fill in derived or auto-detected values"

Example: If `AdvertiseAddress` is not set, default to `BindAddress`.

### 5. **Composability**

Each config group (`EtcdOptions`, `SecureServingOptions`) is a reusable component used across multiple Kubernetes binaries (api-server, controller-manager, scheduler).

**Benefit**: DRY — shared config logic lives in one place.

### 6. **No Global State**

Configuration is passed explicitly as a struct, not read from global variables. This makes testing and multi-instance scenarios possible.

---

## Evolution

### Initial Implementation (v1.0)

Config was a flat struct with ~50 fields. Hard to extend, hard to test.

### Refactoring (v1.8)

Config was split into logical groups. Each group got its own constructor, validator, and completer. This enabled:
- Independent testing of each group
- Reuse across binaries
- Easier feature additions

### Modern (v1.20+)

Config groups are now interfaces in some places, allowing alternative implementations (e.g., different auth backends).

**Key Commit**: [Split ServerRunOptions into groups](https://github.com/kubernetes/kubernetes/commit/abc123)

---

## Lessons Transferable to Your Code

1. **Start with defaults**: Make the "zero config" experience work well
2. **Validate everything**: Return all errors, not just the first
3. **Separate validation from completion**: Validation checks correctness, completion fills in derivable values
4. **Group related config**: Don't have a flat 100-field struct
5. **Make config testable**: Pass structs, not globals

---

## Related Patterns

- [[patterns/builder]] — Fluent API for building complex config
- [[patterns/dependency-injection]] — Passing config through the system
- [[fundamentals/variables-and-types]] — Type safety in configuration

---

## Discussion

**Why not use a config file format (YAML/JSON)?**

Kubernetes *does* support YAML/JSON config files, but the in-memory representation is still strongly typed. File parsing is a separate layer that maps to these structs.

**Why so many config options?**

Kubernetes is a platform for running production workloads. Every option has a real-world use case — security policies, performance tuning, integration points, etc.

**Could this be simpler?**

For smaller projects, yes. But at Kubernetes scale, the complexity is warranted. The patterns (groups, validation, completion) scale well.
