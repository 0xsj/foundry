# Notification Delivery System Factory

## Scenario

Your team operates a notification platform that dispatches messages across multiple channels -- email (SMTP), SMS (Twilio-like API), push notifications (FCM-like), and webhooks (HTTP POST). Each channel has different configuration requirements, initialization sequences, and delivery semantics. The current codebase has a 400-line `SendNotification` function with a massive switch statement that grows every time a new channel is added.

You've been asked to refactor this into a clean factory-based architecture where new channels can be added without modifying existing code.

## Brief

Implement a notification dispatcher factory that:
1. Defines a `Dispatcher` interface for sending notifications
2. Provides channel-specific implementations (email, SMS, push, webhook)
3. Uses a registry pattern so new channels can be registered without modifying factory code
4. Handles configuration validation per channel
5. Supports creating dispatchers from a unified config format

## Acceptance Criteria

- [ ] `Dispatcher` interface with `Send(ctx, recipient, message) error` and `Channel() string` methods
- [ ] `Notification` struct with `Subject`, `Body`, `Priority`, and `Metadata` fields
- [ ] `ChannelConfig` struct with `Type`, `Params` (map), and `Enabled` flag
- [ ] `Register(name, factory)` function to register channel factories
- [ ] `NewDispatcher(config)` function that looks up the registry and returns the right dispatcher
- [ ] Email dispatcher: validates `smtp_host`, `smtp_port`, `from_address` params
- [ ] SMS dispatcher: validates `api_key`, `from_number` params
- [ ] Push dispatcher: validates `server_key`, `project_id` params
- [ ] Webhook dispatcher: validates `url`, `secret` params; signs payloads with HMAC
- [ ] `ListChannels()` returns all registered channel names
- [ ] Proper error handling: unknown channel type, missing required config, nil config
- [ ] All dispatchers are goroutine-safe (use appropriate synchronization)
- [ ] Tests pass: `go test -v`

## Constraints

- Each dispatcher must validate its own configuration at creation time (fail fast, not at send time)
- The webhook dispatcher must compute an HMAC-SHA256 signature of the payload
- Priority must be one of: "low", "normal", "high", "critical"
- Sending to an empty recipient must return an error
- The registry must be safe for concurrent registration and lookup

## Files

- `starter/main.go` -- Scaffold with interfaces, types, and TODOs
- `starter/main_test.go` -- Full test suite (run with `go test -v`)

## Getting Started

```bash
cd starter
go test -v  # Should fail initially -- implement the TODOs
```

## Hints

<details>
<summary>Hint 1: Registry structure</summary>

Use a package-level map guarded by `sync.RWMutex`. The factory function type should be `func(params map[string]string) (Dispatcher, error)` -- this lets each channel validate its own config.

</details>

<details>
<summary>Hint 2: Config validation pattern</summary>

Create a helper function that checks for required params:

```go
func requireParams(params map[string]string, required ...string) error {
    for _, key := range required {
        if params[key] == "" {
            return fmt.Errorf("missing required parameter: %s", key)
        }
    }
    return nil
}
```

</details>

<details>
<summary>Hint 3: HMAC signing for webhooks</summary>

```go
import "crypto/hmac"
import "crypto/sha256"
import "encoding/hex"

mac := hmac.New(sha256.New, []byte(secret))
mac.Write(payload)
signature := hex.EncodeToString(mac.Sum(nil))
```

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
