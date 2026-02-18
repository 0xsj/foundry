package payment

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"
)

// =============================================================================
// Payment Strategy Interface
// =============================================================================

// PaymentProcessor defines all operations a payment provider must support.
type PaymentProcessor interface {
	Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error)
	Refund(ctx context.Context, transactionID string, amount float64) error
	GetBalance(ctx context.Context, customerID string) (float64, error)
	Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error)
	CancelSubscription(ctx context.Context, subscriptionID string) error
	ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error)
	GenerateInvoice(ctx context.Context, transactionID string) ([]byte, error)
	ValidateCard(ctx context.Context, cardToken string) (bool, error)
	SetWebhookURL(ctx context.Context, url string) error
}

// Transaction represents a payment transaction record.
type Transaction struct {
	ID         string
	Amount     float64
	Currency   string
	CustomerID string
	Status     string
	CreatedAt  time.Time
}

// =============================================================================
// Stripe Implementation
// =============================================================================

// StripeProcessor handles payments via Stripe.
type StripeProcessor struct {
	apiKey         string
	mu             sync.Mutex
	transactionLog []string // shared mutable state for logging
}

func NewStripeProcessor(apiKey string) *StripeProcessor {
	return &StripeProcessor{
		apiKey:         apiKey,
		transactionLog: make([]string, 0),
	}
}

func (s *StripeProcessor) Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error) {
	if amount <= 0 {
		return "", fmt.Errorf("invalid amount: %.2f", amount)
	}

	txID := fmt.Sprintf("stripe_ch_%d", time.Now().UnixNano())

	// Log the transaction — but the mutex doesn't protect this!
	entry := fmt.Sprintf("[%s] Charged %.2f %s to %s (tx: %s)",
		time.Now().Format(time.RFC3339), amount, currency, customerID, txID)
	s.transactionLog = append(s.transactionLog, entry)

	log.Printf("Stripe charge: %s", entry)
	return txID, nil
}

func (s *StripeProcessor) Refund(ctx context.Context, transactionID string, amount float64) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	entry := fmt.Sprintf("[%s] Refunded %.2f for tx %s",
		time.Now().Format(time.RFC3339), amount, transactionID)
	s.transactionLog = append(s.transactionLog, entry)
	return nil
}

func (s *StripeProcessor) GetBalance(ctx context.Context, customerID string) (float64, error) {
	return 0, fmt.Errorf("not implemented")
}

func (s *StripeProcessor) Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error) {
	return "", fmt.Errorf("not implemented")
}

func (s *StripeProcessor) CancelSubscription(ctx context.Context, subscriptionID string) error {
	return fmt.Errorf("not implemented")
}

func (s *StripeProcessor) ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error) {
	return nil, fmt.Errorf("not implemented")
}

func (s *StripeProcessor) GenerateInvoice(ctx context.Context, transactionID string) ([]byte, error) {
	return nil, fmt.Errorf("not implemented")
}

func (s *StripeProcessor) ValidateCard(ctx context.Context, cardToken string) (bool, error) {
	return false, fmt.Errorf("not implemented")
}

func (s *StripeProcessor) SetWebhookURL(ctx context.Context, url string) error {
	return fmt.Errorf("not implemented")
}

// =============================================================================
// PayPal Implementation
// =============================================================================

// PayPalProcessor handles payments via PayPal.
type PayPalProcessor struct {
	clientID     string
	clientSecret string
}

func NewPayPalProcessor(clientID, clientSecret string) *PayPalProcessor {
	return &PayPalProcessor{
		clientID:     clientID,
		clientSecret: clientSecret,
	}
}

func (p *PayPalProcessor) Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error) {
	if amount <= 0 {
		return "", fmt.Errorf("invalid amount: %.2f", amount)
	}
	txID := fmt.Sprintf("paypal_pay_%d", time.Now().UnixNano())
	log.Printf("PayPal charge: %.2f %s to %s", amount, currency, customerID)
	return txID, nil
}

func (p *PayPalProcessor) Refund(ctx context.Context, transactionID string, amount float64) error {
	log.Printf("PayPal refund: %.2f for %s", amount, transactionID)
	return nil
}

func (p *PayPalProcessor) GetBalance(ctx context.Context, customerID string) (float64, error) {
	return 0, fmt.Errorf("not supported by PayPal integration")
}

func (p *PayPalProcessor) Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error) {
	return "", fmt.Errorf("not supported by PayPal integration")
}

func (p *PayPalProcessor) CancelSubscription(ctx context.Context, subscriptionID string) error {
	return fmt.Errorf("not supported by PayPal integration")
}

func (p *PayPalProcessor) ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error) {
	return nil, fmt.Errorf("not supported by PayPal integration")
}

func (p *PayPalProcessor) GenerateInvoice(ctx context.Context, transactionID string) ([]byte, error) {
	return nil, fmt.Errorf("not supported by PayPal integration")
}

func (p *PayPalProcessor) ValidateCard(ctx context.Context, cardToken string) (bool, error) {
	return false, fmt.Errorf("PayPal does not use card tokens")
}

func (p *PayPalProcessor) SetWebhookURL(ctx context.Context, url string) error {
	return nil
}

// =============================================================================
// Bank Transfer Implementation
// =============================================================================

// BankTransferProcessor handles payments via direct bank transfer.
type BankTransferProcessor struct {
	bankCode   string
	routingNum string
}

func NewBankTransferProcessor(bankCode, routingNum string) *BankTransferProcessor {
	return &BankTransferProcessor{
		bankCode:   bankCode,
		routingNum: routingNum,
	}
}

func (b *BankTransferProcessor) Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error) {
	if amount <= 0 {
		return "", fmt.Errorf("invalid amount: %.2f", amount)
	}
	txID := fmt.Sprintf("bank_tx_%d", time.Now().UnixNano())
	log.Printf("Bank transfer: %.2f %s from %s via %s", amount, currency, customerID, b.bankCode)
	return txID, nil
}

func (b *BankTransferProcessor) Refund(ctx context.Context, transactionID string, amount float64) error {
	return nil
}

func (b *BankTransferProcessor) GetBalance(ctx context.Context, customerID string) (float64, error) {
	return 0, fmt.Errorf("not supported")
}

func (b *BankTransferProcessor) Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error) {
	return "", fmt.Errorf("bank transfers do not support subscriptions")
}

func (b *BankTransferProcessor) CancelSubscription(ctx context.Context, subscriptionID string) error {
	return fmt.Errorf("bank transfers do not support subscriptions")
}

func (b *BankTransferProcessor) ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error) {
	return nil, fmt.Errorf("not supported")
}

func (b *BankTransferProcessor) GenerateInvoice(ctx context.Context, transactionID string) ([]byte, error) {
	return nil, fmt.Errorf("not supported")
}

func (b *BankTransferProcessor) ValidateCard(ctx context.Context, cardToken string) (bool, error) {
	return false, fmt.Errorf("bank transfers do not use cards")
}

func (b *BankTransferProcessor) SetWebhookURL(ctx context.Context, url string) error {
	return fmt.Errorf("not supported")
}

// =============================================================================
// Payment Service (Strategy Consumer)
// =============================================================================

// PaymentService orchestrates payment processing using a selected provider.
type PaymentService struct {
	processor *StripeProcessor // the active payment processor
}

// NewPaymentService creates a new payment service.
func NewPaymentService(processor *StripeProcessor) *PaymentService {
	return &PaymentService{
		processor: processor,
	}
}

// ProcessPayment charges the customer using the configured processor.
func (ps *PaymentService) ProcessPayment(ctx context.Context, customerID string, amount float64, currency string) (string, error) {
	log.Printf("Processing payment of %.2f %s for customer %s", amount, currency, customerID)

	txID, err := ps.processor.Charge(ctx, amount, currency, customerID)
	if err != nil {
		return "", fmt.Errorf("payment failed: %w", err)
	}

	log.Printf("Payment successful: %s", txID)
	return txID, nil
}

// RefundPayment issues a refund through the configured processor.
func (ps *PaymentService) RefundPayment(ctx context.Context, transactionID string, amount float64) error {
	return ps.processor.Refund(ctx, transactionID, amount)
}

// SwitchProcessor changes the active payment processor at runtime.
func (ps *PaymentService) SwitchProcessor(processor *StripeProcessor) {
	ps.processor = processor
}
