package webhooks

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestHandleWebhook_UserCreated(t *testing.T) {
	payload := `{
		"type": "user.created",
		"data": {
			"email": "alice@example.com",
			"name": "Alice"
		}
	}`

	req := httptest.NewRequest("POST", "/webhook", strings.NewReader(payload))
	w := httptest.NewRecorder()

	HandleWebhook(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}
}

func TestHandleWebhook_PaymentSucceeded(t *testing.T) {
	payload := `{
		"type": "payment.succeeded",
		"data": {
			"user_id": "user_123",
			"amount": 29.99,
			"currency": "usd"
		}
	}`

	req := httptest.NewRequest("POST", "/webhook", strings.NewReader(payload))
	w := httptest.NewRecorder()

	HandleWebhook(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}
}

func TestHandleWebhook_SubscriptionCancelled(t *testing.T) {
	payload := `{
		"type": "subscription.cancelled",
		"data": {
			"user_id": "user_456",
			"tier": "premium"
		}
	}`

	req := httptest.NewRequest("POST", "/webhook", strings.NewReader(payload))
	w := httptest.NewRecorder()

	HandleWebhook(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}
}

func TestHandleWebhook_UnknownEvent(t *testing.T) {
	payload := `{
		"type": "unknown.event",
		"data": {}
	}`

	req := httptest.NewRequest("POST", "/webhook", strings.NewReader(payload))
	w := httptest.NewRecorder()

	HandleWebhook(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("expected status 400, got %d", w.Code)
	}
}
