package webhooks

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
)

// HandleWebhook processes incoming webhook events from the payment provider.
// SMELLS: Long function, nested ifs, type assertions, mixed concerns
func HandleWebhook(w http.ResponseWriter, r *http.Request) {
	// Read body
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// Parse JSON
	var raw map[string]interface{}
	if err := json.Unmarshal(body, &raw); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// Extract event type
	eventType, ok := raw["type"].(string)
	if !ok {
		http.Error(w, "Missing or invalid type field", http.StatusBadRequest)
		return
	}

	// Handle different event types
	// SMELL: Long if-else chain, duplicated structure
	if eventType == "user.created" {
		// SMELL: No type checking on data
		data := raw["data"].(map[string]interface{})
		email := data["email"].(string)
		name := data["name"].(string)

		// SMELL: Business logic mixed with parsing
		if email == "" {
			http.Error(w, "Email is required for user.created", http.StatusBadRequest)
			return
		}

		// Send welcome email
		log.Printf("Sending welcome email to %s (%s)", name, email)
		// ... actual email sending code would go here

		w.WriteHeader(http.StatusOK)
		fmt.Fprintf(w, "User created event processed")

	} else if eventType == "payment.succeeded" {
		data := raw["data"].(map[string]interface{})
		amount := data["amount"].(float64)
		currency := data["currency"].(string)
		userID := data["user_id"].(string)

		if userID == "" {
			http.Error(w, "user_id is required", http.StatusBadRequest)
			return
		}
		if amount <= 0 {
			http.Error(w, "amount must be positive", http.StatusBadRequest)
			return
		}

		// Log transaction
		log.Printf("Payment succeeded: user=%s, amount=%.2f %s", userID, amount, strings.ToUpper(currency))

		w.WriteHeader(http.StatusOK)
		fmt.Fprintf(w, "Payment event processed")

	} else if eventType == "subscription.cancelled" {
		data := raw["data"].(map[string]interface{})
		userID := data["user_id"].(string)
		tier := data["tier"].(string)

		if userID == "" {
			http.Error(w, "user_id is required", http.StatusBadRequest)
			return
		}

		// Send retention offer
		log.Printf("Sending retention offer to user %s (was on %s tier)", userID, tier)
		// ... actual retention email code would go here

		w.WriteHeader(http.StatusOK)
		fmt.Fprintf(w, "Cancellation event processed")

	} else {
		// SMELL: Unknown events fail instead of being logged/ignored
		http.Error(w, fmt.Sprintf("Unknown event type: %s", eventType), http.StatusBadRequest)
		return
	}
}
