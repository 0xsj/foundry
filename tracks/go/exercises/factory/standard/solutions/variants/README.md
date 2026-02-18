# Solution Variants -- Notification Factory

## Approach Overview

The reference solution uses a **registry pattern** with package-level `Register()` and `init()` self-registration. This is the most common factory pattern in production Go code (mirrors `database/sql`).

## Key Decisions

1. **Registry over switch**: The registry pattern allows new channels to be added without modifying the factory function. In a real codebase, each channel would be in its own package with its own `init()`.

2. **`FactoryFunc` type over factory interface**: A function type is simpler than an interface when the factory only does one thing (create a dispatcher). If factories needed lifecycle methods (health checks, shutdown), an interface would be better.

3. **Validation at creation time**: Each dispatcher validates its config in the factory function, not in `Send()`. This follows the "fail fast" principle -- if SMTP host is missing, you learn at startup, not when the first notification fires at 3 AM.

4. **Common `validateSend` helper**: All dispatchers share the same pre-send validation (context, recipient, notification). This eliminates duplication without over-abstracting.

## Variant Approaches

| Approach | Pros | Cons | When to Use |
|----------|------|------|-------------|
| **Registry + init()** (reference) | Extensible, decoupled, idiomatic | Global state, implicit registration | Plugin architecture, multiple teams |
| **Switch factory** | Simple, explicit, easy to trace | Must modify factory for new channels | Small team, stable channel set |
| **Config-driven factory** | Channels defined in YAML/JSON | Less type safety, runtime errors | Ops-managed channel configuration |
| **Functional options** | Fine-grained per-channel config | More ceremony for simple cases | Complex per-channel initialization |

### Switch Factory Variant

```go
func NewDispatcher(cfg ChannelConfig) (Dispatcher, error) {
    switch cfg.Type {
    case "email":
        return newEmailDispatcher(cfg.Params)
    case "sms":
        return newSMSDispatcher(cfg.Params)
    case "push":
        return newPushDispatcher(cfg.Params)
    case "webhook":
        return newWebhookDispatcher(cfg.Params)
    default:
        return nil, fmt.Errorf("unknown channel: %s", cfg.Type)
    }
}
```

This is perfectly fine for 4 channels that won't grow. The registry adds value when:
- Third-party packages need to add channels
- Channels are loaded as plugins
- You want to add channels without recompiling

### Config-Driven Variant

```go
// Channels defined in YAML:
// channels:
//   - type: email
//     enabled: true
//     params:
//       smtp_host: smtp.example.com

func LoadDispatchers(configPath string) (map[string]Dispatcher, error) {
    var configs []ChannelConfig
    // parse YAML into configs...

    dispatchers := make(map[string]Dispatcher)
    for _, cfg := range configs {
        if !cfg.Enabled { continue }
        d, err := NewDispatcher(cfg)
        if err != nil { return nil, err }
        dispatchers[cfg.Type] = d
    }
    return dispatchers, nil
}
```

This is the natural evolution when ops teams need to manage channels without code changes.

## Performance Notes

Factory creation is a startup-time cost. The registry lookup is O(1) map access with a read lock. There is no performance concern here -- factories are called once per channel during initialization, not per notification.

The `Send` path is what matters for performance, and that's independent of the factory pattern.
