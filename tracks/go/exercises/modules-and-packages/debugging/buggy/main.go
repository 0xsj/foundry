// Package main wires together the notification routing system.
// This file has compile errors — see README.md for the symptoms.
package main

import (
	"fmt"

	"github.com/foundry/notify/pkg/a"
	"github.com/foundry/notify/pkg/b"
)

func main() {
	// Get routing configuration for a payment failure event
	routes := a.RouteEvent("payment.failed")
	fmt.Printf("routes for payment.failed: %+v\n", routes)

	// BUG 2: cfg is a ChannelConfig value, but retryCount is defined as
	// an unexported field. Wait — ChannelConfig has Priority (exported) and
	// Channel (exported). There's no retryCount on ChannelConfig at all.
	// Actually the field the developer wanted is on the sender struct inside
	// package b — but that struct is unexported, so they can't access it here.
	// This line attempts to access a field that doesn't exist on ChannelConfig.
	if len(routes) > 0 {
		cfg := routes[0]
		fmt.Printf("retry count: %d\n", cfg.retryCount) // BUG 2: no such field
	}

	// Create an email sender and use it as a Sender interface
	// BUG 3: NewSender returns *b.sender (unexported type), but we're trying
	// to use it as b.Sender (the exported interface). The compiler rejects this
	// because external packages cannot name or use unexported types.
	var s b.Sender = b.NewSender("smtp") // BUG 3: type mismatch
	if s != nil {
		fmt.Printf("sender protocol: %s\n", s.Protocol())
	}
}
