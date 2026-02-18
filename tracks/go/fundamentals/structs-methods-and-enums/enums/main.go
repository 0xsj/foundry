// Run with: go run ./enums/
package main

import (
	"fmt"
	"strings"
)

// ============================================================================
// BASIC ENUM PATTERN
// Named integer type + iota constants + String() method.
// ============================================================================

// ChannelType represents the delivery mechanism for a notification.
type ChannelType int

const (
	ChannelEmail   ChannelType = iota // 0
	ChannelSMS                        // 1
	ChannelWebhook                    // 2
	ChannelSlack                      // 3
	ChannelPush                       // 4
)

// String implements fmt.Stringer. Called automatically by fmt.Printf/Println.
func (c ChannelType) String() string {
	switch c {
	case ChannelEmail:
		return "email"
	case ChannelSMS:
		return "sms"
	case ChannelWebhook:
		return "webhook"
	case ChannelSlack:
		return "slack"
	case ChannelPush:
		return "push"
	default:
		// Always handle the default — forward compatibility + external values
		return fmt.Sprintf("ChannelType(%d)", int(c))
	}
}

// IsAsync returns true for channels that deliver asynchronously.
// Methods on typed constants give you enum-like behavior.
func (c ChannelType) IsAsync() bool {
	return c == ChannelWebhook || c == ChannelPush
}

// MaxRetries returns the default retry limit per channel type.
func (c ChannelType) MaxRetries() int {
	switch c {
	case ChannelEmail:
		return 5
	case ChannelSMS:
		return 3
	case ChannelWebhook:
		return 10 // webhooks are more reliable infra-to-infra
	case ChannelSlack:
		return 2
	case ChannelPush:
		return 1 // push is fire-and-forget
	default:
		return 3
	}
}

func demoBasicEnum() {
	fmt.Println("=== Basic Enum Pattern ===")

	ch := ChannelWebhook
	fmt.Printf("Channel: %v\n", ch)            // calls String() automatically
	fmt.Printf("Async: %v\n", ch.IsAsync())
	fmt.Printf("MaxRetries: %d\n", ch.MaxRetries())

	// Type safety: ChannelType != int
	var rawInt int = 2
	_ = rawInt                                  // can't accidentally use raw int
	_ = ChannelType(rawInt)                     // explicit conversion required

	// Iterate all known values — common pattern
	channels := []ChannelType{ChannelEmail, ChannelSMS, ChannelWebhook, ChannelSlack, ChannelPush}
	for _, c := range channels {
		fmt.Printf("  %-10s async=%-5v retries=%d\n", c, c.IsAsync(), c.MaxRetries())
	}
	fmt.Println()
}

// ============================================================================
// SKIP ZERO WITH IOTA
// Skipping 0 is useful when you want to distinguish "not set" from a real value.
// ============================================================================

type NotificationStatus int

const (
	// Skip 0: a zero-value Status can mean "unset", not "pending"
	_               NotificationStatus = iota // 0 — explicitly skipped
	StatusPending                             // 1
	StatusSending                             // 2
	StatusDelivered                           // 3
	StatusFailed                              // 4
	StatusRetrying                            // 5
)

func (s NotificationStatus) String() string {
	names := [...]string{"(unset)", "pending", "sending", "delivered", "failed", "retrying"}
	if int(s) < len(names) {
		return names[s]
	}
	return fmt.Sprintf("NotificationStatus(%d)", int(s))
}

func (s NotificationStatus) IsFinal() bool {
	return s == StatusDelivered || s == StatusFailed
}

func demoSkipZero() {
	fmt.Println("=== Skip Zero ===")

	var s NotificationStatus // zero value — "unset", not "pending"
	fmt.Printf("Zero value: %v\n", s)

	s = StatusPending
	fmt.Printf("After assignment: %v (isFinal=%v)\n", s, s.IsFinal())

	s = StatusDelivered
	fmt.Printf("Delivered: %v (isFinal=%v)\n\n", s, s.IsFinal())
}

// ============================================================================
// BIT FLAG ENUMS
// Use 1 << iota for flags that can be combined with bitwise OR.
// ============================================================================

type Permission uint

const (
	PermRead    Permission = 1 << iota // 1   (0b0001)
	PermWrite                          // 2   (0b0010)
	PermDelete                         // 4   (0b0100)
	PermAdmin                          // 8   (0b1000)
)

// Has returns true if the permission set includes the given permission.
func (p Permission) Has(perm Permission) bool {
	return p&perm != 0
}

func (p Permission) String() string {
	if p == 0 {
		return "none"
	}
	parts := []string{}
	if p.Has(PermRead) {
		parts = append(parts, "read")
	}
	if p.Has(PermWrite) {
		parts = append(parts, "write")
	}
	if p.Has(PermDelete) {
		parts = append(parts, "delete")
	}
	if p.Has(PermAdmin) {
		parts = append(parts, "admin")
	}
	return strings.Join(parts, "|")
}

func demoBitFlags() {
	fmt.Println("=== Bit Flag Enums ===")

	// Combine permissions with bitwise OR
	editorPerms := PermRead | PermWrite
	fmt.Printf("Editor: %v (raw: %d)\n", editorPerms, editorPerms)
	fmt.Printf("  canRead: %v\n", editorPerms.Has(PermRead))
	fmt.Printf("  canWrite: %v\n", editorPerms.Has(PermWrite))
	fmt.Printf("  canDelete: %v\n", editorPerms.Has(PermDelete))

	adminPerms := PermRead | PermWrite | PermDelete | PermAdmin
	fmt.Printf("Admin: %v\n\n", adminPerms)
}

// ============================================================================
// STRING → ENUM CONVERSION
// Parsing from external input (config, JSON, HTTP params).
// Return (T, error) — don't use sentinel int values.
// ============================================================================

func ChannelTypeFromString(s string) (ChannelType, error) {
	switch strings.ToLower(s) {
	case "email":
		return ChannelEmail, nil
	case "sms":
		return ChannelSMS, nil
	case "webhook":
		return ChannelWebhook, nil
	case "slack":
		return ChannelSlack, nil
	case "push":
		return ChannelPush, nil
	default:
		return 0, fmt.Errorf("unknown channel type %q: valid values are email, sms, webhook, slack, push", s)
	}
}

func demoStringConversion() {
	fmt.Println("=== String Conversion ===")

	inputs := []string{"email", "WEBHOOK", "telegram", "slack"}
	for _, input := range inputs {
		ch, err := ChannelTypeFromString(input)
		if err != nil {
			fmt.Printf("  %q → error: %v\n", input, err)
		} else {
			fmt.Printf("  %q → %v (async=%v)\n", input, ch, ch.IsAsync())
		}
	}
	fmt.Println()
}

// ============================================================================
// TYPE SWITCH — PATTERN MATCHING FOR INTERFACE TYPES
// Different from a regular switch on enum values.
// Used when you have an interface and need to act on the concrete type.
// ============================================================================

// Channel is the interface all delivery channels implement.
type Channel interface {
	Name() string
}

type EmailChannel struct {
	SMTPHost string
}
func (e *EmailChannel) Name() string { return "email" }

type SMSChannel struct {
	Provider string
}
func (s *SMSChannel) Name() string { return "sms" }

type SlackChannel struct {
	Workspace string
	ChannelID string
}
func (s *SlackChannel) Name() string { return "slack" }

// describe uses a type switch to access type-specific fields.
// This is Go's version of pattern matching on an interface.
func describe(ch Channel) string {
	switch c := ch.(type) {
	case *EmailChannel:
		return fmt.Sprintf("email channel via SMTP host %s", c.SMTPHost)
	case *SMSChannel:
		return fmt.Sprintf("SMS channel via provider %s", c.Provider)
	case *SlackChannel:
		return fmt.Sprintf("Slack channel #%s in workspace %s", c.ChannelID, c.Workspace)
	case nil:
		return "nil channel"
	default:
		// Always add default — handles future types and external implementations
		return fmt.Sprintf("unknown channel type %T: %v", c, c)
	}
}

func demoTypeSwitch() {
	fmt.Println("=== Type Switch ===")

	channels := []Channel{
		&EmailChannel{SMTPHost: "smtp.sendgrid.net"},
		&SMSChannel{Provider: "twilio"},
		&SlackChannel{Workspace: "acme-corp", ChannelID: "alerts"},
	}

	for _, ch := range channels {
		fmt.Println(" ", describe(ch))
	}
	fmt.Println()
}

// ============================================================================
// ENUM AS MAP KEY
// Typed constants are comparable — they can be used as map keys.
// ============================================================================

type ChannelConfig struct {
	Endpoint   string
	MaxRetries int
	Timeout    int // seconds
}

func demoEnumMapKey() {
	fmt.Println("=== Enum as Map Key ===")

	// ChannelType is comparable (it's an int underneath) — valid map key.
	configs := map[ChannelType]ChannelConfig{
		ChannelEmail: {
			Endpoint:   "smtp://mail.example.com:587",
			MaxRetries: 5,
			Timeout:    30,
		},
		ChannelWebhook: {
			Endpoint:   "https://hooks.example.com/notify",
			MaxRetries: 10,
			Timeout:    10,
		},
		ChannelSlack: {
			Endpoint:   "https://slack.com/api/chat.postMessage",
			MaxRetries: 2,
			Timeout:    5,
		},
	}

	for _, ch := range []ChannelType{ChannelEmail, ChannelWebhook, ChannelSMS, ChannelSlack} {
		if cfg, ok := configs[ch]; ok {
			fmt.Printf("  %-10v → endpoint=%s timeout=%ds\n", ch, cfg.Endpoint, cfg.Timeout)
		} else {
			fmt.Printf("  %-10v → no config\n", ch)
		}
	}
	fmt.Println()
}

func main() {
	demoBasicEnum()
	demoSkipZero()
	demoBitFlags()
	demoStringConversion()
	demoTypeSwitch()
	demoEnumMapKey()
}
