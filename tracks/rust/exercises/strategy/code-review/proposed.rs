// Payment Processing with Pluggable Strategies
//
// PR #247: Replace hardcoded Stripe integration with pluggable payment providers.
//
// Author: junior_dev
// Status: Ready for review

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Money {
    amount_cents: u64,
    currency: String,
}

impl Money {
    fn new(amount_cents: u64, currency: &str) -> Self {
        Self {
            amount_cents,
            currency: currency.to_uppercase(),
        }
    }

    fn display_amount(&self) -> String {
        format!(
            "{}.{:02} {}",
            self.amount_cents / 100,
            self.amount_cents % 100,
            self.currency
        )
    }
}

#[derive(Debug, Clone)]
struct PaymentRequest {
    order_id: String,
    amount: Money,
    customer_email: String,
    card_number: String,
    card_expiry: String,  // MM/YY
    card_cvv: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct PaymentResult {
    transaction_id: String,
    status: PaymentStatus,
    provider: String,
    fee_cents: u64,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
enum PaymentStatus {
    Completed,
    Pending,
    Failed,
    Refunded,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Refunded => write!(f, "refunded"),
        }
    }
}

// ---------------------------------------------------------------------------
// REVIEW TARGET: PaymentProcessor trait
//
// Issue 1: Too many required methods. Not every provider supports refunds
//          or recurring billing. A provider that doesn't support refunds
//          has to implement it anyway (probably as a panic or error).
//
// Issue 2: Several methods take &mut self when they don't need to mutate.
//          process_payment doesn't modify the strategy's internal state —
//          it's a stateless RPC call. Using &mut self prevents concurrent
//          usage and requires exclusive access.
//
// Issue 3: validate_card probably doesn't belong on the payment strategy
//          at all — card validation rules (Luhn check, expiry) are universal,
//          not provider-specific.
// ---------------------------------------------------------------------------

trait PaymentProcessor {
    /// Process a payment through this provider.
    fn process_payment(&mut self, request: &PaymentRequest) -> Result<PaymentResult, String>;

    /// Refund a previous transaction.
    fn refund(&mut self, transaction_id: &str, amount: &Money) -> Result<PaymentResult, String>;

    /// Set up recurring billing for a customer.
    fn setup_recurring(
        &mut self,
        customer_id: &str,
        amount: &Money,
        interval_days: u32,
    ) -> Result<String, String>;

    /// Cancel recurring billing.
    fn cancel_recurring(&mut self, subscription_id: &str) -> Result<(), String>;

    /// Validate a card number (Luhn check + expiry).
    fn validate_card(&self, card_number: &str, expiry: &str) -> bool;

    /// Provider name.
    fn name(&self) -> &str;

    /// Provider-specific fee calculation.
    fn calculate_fee(&self, amount: &Money) -> u64;

    /// Health check — is the provider API reachable?
    fn health_check(&mut self) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Stripe implementation
// ---------------------------------------------------------------------------

struct StripeProcessor {
    api_key: String,
    webhook_secret: String,
    request_count: u64,  // tracks total requests for rate limiting
}

impl StripeProcessor {
    fn new(api_key: &str, webhook_secret: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            webhook_secret: webhook_secret.to_string(),
            request_count: 0,
        }
    }
}

impl PaymentProcessor for StripeProcessor {
    fn process_payment(&mut self, request: &PaymentRequest) -> Result<PaymentResult, String> {
        self.request_count += 1;
        println!(
            "  [Stripe] Processing {} for order {}",
            request.amount.display_amount(),
            request.order_id
        );

        if request.amount.amount_cents == 0 {
            return Err("amount must be greater than zero".to_string());
        }

        let fee = self.calculate_fee(&request.amount);
        Ok(PaymentResult {
            transaction_id: format!("stripe_txn_{}", request.order_id),
            status: PaymentStatus::Completed,
            provider: "stripe".to_string(),
            fee_cents: fee,
            message: format!("Payment processed via Stripe (fee: {} cents)", fee),
        })
    }

    fn refund(&mut self, transaction_id: &str, amount: &Money) -> Result<PaymentResult, String> {
        self.request_count += 1;
        Ok(PaymentResult {
            transaction_id: format!("stripe_ref_{}", transaction_id),
            status: PaymentStatus::Refunded,
            provider: "stripe".to_string(),
            fee_cents: 0,
            message: format!("Refunded {} on {}", amount.display_amount(), transaction_id),
        })
    }

    fn setup_recurring(
        &mut self,
        customer_id: &str,
        amount: &Money,
        interval_days: u32,
    ) -> Result<String, String> {
        self.request_count += 1;
        Ok(format!(
            "stripe_sub_{}_{}_{}d",
            customer_id,
            amount.amount_cents,
            interval_days
        ))
    }

    fn cancel_recurring(&mut self, subscription_id: &str) -> Result<(), String> {
        self.request_count += 1;
        println!("  [Stripe] Cancelled subscription {}", subscription_id);
        Ok(())
    }

    fn validate_card(&self, card_number: &str, expiry: &str) -> bool {
        // Basic Luhn check
        let digits: Vec<u32> = card_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let checksum: u32 = digits
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &d)| {
                if i % 2 == 1 {
                    let doubled = d * 2;
                    if doubled > 9 { doubled - 9 } else { doubled }
                } else {
                    d
                }
            })
            .sum();

        if checksum % 10 != 0 {
            return false;
        }

        // Check expiry
        let parts: Vec<&str> = expiry.split('/').collect();
        if parts.len() != 2 {
            return false;
        }
        // BUG: This always returns true after the Luhn check passes,
        // because it doesn't actually validate the expiry date.
        // It parses the parts but never checks if the date is in the future.
        let _month: u32 = parts[0].parse().unwrap_or(0);
        let _year: u32 = parts[1].parse().unwrap_or(0);
        true
    }

    fn name(&self) -> &str {
        "stripe"
    }

    fn calculate_fee(&self, amount: &Money) -> u64 {
        // Stripe: 2.9% + 30 cents
        (amount.amount_cents as f64 * 0.029) as u64 + 30
    }

    fn health_check(&mut self) -> Result<(), String> {
        self.request_count += 1;
        // In production: actually ping Stripe API
        println!("  [Stripe] Health check OK (total requests: {})", self.request_count);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PayPal implementation
// ---------------------------------------------------------------------------

struct PayPalProcessor {
    client_id: String,
    client_secret: String,
    sandbox: bool,
}

impl PayPalProcessor {
    fn new(client_id: &str, client_secret: &str, sandbox: bool) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            sandbox,
        }
    }
}

impl PaymentProcessor for PayPalProcessor {
    fn process_payment(&mut self, request: &PaymentRequest) -> Result<PaymentResult, String> {
        let env = if self.sandbox { "sandbox" } else { "live" };
        println!(
            "  [PayPal/{}] Processing {} for order {}",
            env,
            request.amount.display_amount(),
            request.order_id
        );

        let fee = self.calculate_fee(&request.amount);
        Ok(PaymentResult {
            transaction_id: format!("pp_txn_{}", request.order_id),
            status: PaymentStatus::Completed,
            provider: "paypal".to_string(),
            fee_cents: fee,
            message: format!("Payment processed via PayPal {} (fee: {} cents)", env, fee),
        })
    }

    fn refund(&mut self, transaction_id: &str, amount: &Money) -> Result<PaymentResult, String> {
        Ok(PaymentResult {
            transaction_id: format!("pp_ref_{}", transaction_id),
            status: PaymentStatus::Refunded,
            provider: "paypal".to_string(),
            fee_cents: 0,
            message: format!("Refunded {} on {}", amount.display_amount(), transaction_id),
        })
    }

    fn setup_recurring(
        &mut self,
        _customer_id: &str,
        _amount: &Money,
        _interval_days: u32,
    ) -> Result<String, String> {
        // PayPal has its own recurring billing, but this implementation
        // just stubs it out because we haven't integrated it yet
        Err("PayPal recurring billing not yet implemented".to_string())
    }

    fn cancel_recurring(&mut self, _subscription_id: &str) -> Result<(), String> {
        Err("PayPal recurring billing not yet implemented".to_string())
    }

    fn validate_card(&self, _card_number: &str, _expiry: &str) -> bool {
        // PayPal doesn't use card numbers directly — redirect to PayPal flow
        // This method doesn't make sense for PayPal but we have to implement it
        true
    }

    fn name(&self) -> &str {
        "paypal"
    }

    fn calculate_fee(&self, amount: &Money) -> u64 {
        // PayPal: 3.49% + 49 cents for standard
        (amount.amount_cents as f64 * 0.0349) as u64 + 49
    }

    fn health_check(&mut self) -> Result<(), String> {
        let env = if self.sandbox { "sandbox" } else { "live" };
        println!("  [PayPal/{}] Health check OK", env);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bank Transfer implementation
// ---------------------------------------------------------------------------

struct BankTransferProcessor {
    bank_name: String,
    routing_number: String,
}

impl BankTransferProcessor {
    fn new(bank_name: &str, routing_number: &str) -> Self {
        Self {
            bank_name: bank_name.to_string(),
            routing_number: routing_number.to_string(),
        }
    }
}

impl PaymentProcessor for BankTransferProcessor {
    fn process_payment(&mut self, request: &PaymentRequest) -> Result<PaymentResult, String> {
        println!(
            "  [Bank/{}] Processing {} for order {}",
            self.bank_name,
            request.amount.display_amount(),
            request.order_id
        );

        // Bank transfers are always pending (take 1-3 business days)
        Ok(PaymentResult {
            transaction_id: format!("bank_txn_{}", request.order_id),
            status: PaymentStatus::Pending,
            provider: format!("bank_{}", self.bank_name),
            fee_cents: self.calculate_fee(&request.amount),
            message: "Bank transfer initiated. Funds will arrive in 1-3 business days.".to_string(),
        })
    }

    fn refund(&mut self, _transaction_id: &str, _amount: &Money) -> Result<PaymentResult, String> {
        // Bank transfers can't be refunded through our system —
        // requires manual intervention
        Err("Bank transfer refunds require manual processing. Contact support.".to_string())
    }

    fn setup_recurring(
        &mut self,
        _customer_id: &str,
        _amount: &Money,
        _interval_days: u32,
    ) -> Result<String, String> {
        Err("Recurring billing not supported for bank transfers".to_string())
    }

    fn cancel_recurring(&mut self, _subscription_id: &str) -> Result<(), String> {
        Err("Recurring billing not supported for bank transfers".to_string())
    }

    fn validate_card(&self, _card_number: &str, _expiry: &str) -> bool {
        // Bank transfers don't use cards — this method is nonsensical here
        // but we have to implement it
        false
    }

    fn name(&self) -> &str {
        "bank_transfer"
    }

    fn calculate_fee(&self, _amount: &Money) -> u64 {
        // Flat fee for bank transfers
        150 // $1.50
    }

    fn health_check(&mut self) -> Result<(), String> {
        println!("  [Bank/{}] Health check OK", self.bank_name);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// REVIEW TARGET: PaymentRouter — Enum-based "strategy"
//
// This wraps the trait objects in an enum, then matches on the enum to
// delegate. This defeats the entire purpose of the strategy pattern.
// The enum creates a closed set — adding a new provider means modifying
// PaymentRouter. The trait object already provides open extensibility.
//
// The developer said "I added the enum for type safety" but it actually
// removes the benefit of dynamic dispatch.
// ---------------------------------------------------------------------------

enum PaymentProvider {
    Stripe(Box<dyn PaymentProcessor>),
    PayPal(Box<dyn PaymentProcessor>),
    BankTransfer(Box<dyn PaymentProcessor>),
}

struct PaymentRouter {
    providers: Vec<PaymentProvider>,
}

impl PaymentRouter {
    fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    fn register(&mut self, provider: PaymentProvider) {
        self.providers.push(provider);
    }

    fn process(
        &mut self,
        provider_name: &str,
        request: &PaymentRequest,
    ) -> Result<PaymentResult, String> {
        // The enum match here is pointless — we're matching on an enum
        // that wraps trait objects, when we could just use the trait objects directly.
        for provider in &mut self.providers {
            match provider {
                PaymentProvider::Stripe(p) => {
                    if provider_name == "stripe" {
                        return p.process_payment(request);
                    }
                }
                PaymentProvider::PayPal(p) => {
                    if provider_name == "paypal" {
                        return p.process_payment(request);
                    }
                }
                PaymentProvider::BankTransfer(p) => {
                    if provider_name == "bank_transfer" {
                        return p.process_payment(request);
                    }
                }
            }
        }
        Err(format!("no provider registered for '{}'", provider_name))
    }
}

// ---------------------------------------------------------------------------
// REVIEW TARGET: OrderService — Uses Box<dyn> when generics would work
//
// The OrderService is created once at startup with a fixed provider.
// It never changes providers at runtime. Using Box<dyn PaymentProcessor>
// here adds unnecessary heap allocation and dynamic dispatch.
// A generic parameter would give zero-cost abstraction.
//
// However: for this use case (~10K orders/day), the performance difference
// is negligible. The design concern is more about unnecessary complexity
// and signaling — Box<dyn> implies runtime flexibility that isn't needed.
// ---------------------------------------------------------------------------

struct OrderService {
    payment: Box<dyn PaymentProcessor>,
    order_log: Vec<(String, PaymentResult)>,
}

impl OrderService {
    fn new(payment: Box<dyn PaymentProcessor>) -> Self {
        Self {
            payment,
            order_log: Vec::new(),
        }
    }

    fn place_order(
        &mut self,
        order_id: &str,
        amount_cents: u64,
        currency: &str,
        email: &str,
        card: &str,
        expiry: &str,
        cvv: &str,
    ) -> Result<PaymentResult, String> {
        // Validate card using the payment processor
        if !self.payment.validate_card(card, expiry) {
            return Err("invalid card".to_string());
        }

        let request = PaymentRequest {
            order_id: order_id.to_string(),
            amount: Money::new(amount_cents, currency),
            customer_email: email.to_string(),
            card_number: card.to_string(),
            card_expiry: expiry.to_string(),
            card_cvv: cvv.to_string(),
            metadata: HashMap::new(),
        };

        let result = self.payment.process_payment(&request)?;
        self.order_log.push((order_id.to_string(), result.clone()));
        Ok(result)
    }

    fn order_history(&self) -> &[(String, PaymentResult)] {
        &self.order_log
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Payment Processing System ===\n");

    // --- OrderService with Stripe ---
    println!("--- Order Service (Stripe) ---");
    let stripe = StripeProcessor::new("sk_test_abc123", "whsec_xyz789");
    let mut service = OrderService::new(Box::new(stripe));

    match service.place_order(
        "ORD-001",
        4999,
        "USD",
        "customer@example.com",
        "4242424242424242",
        "12/27",
        "123",
    ) {
        Ok(result) => println!("  Order placed: {} — {}\n", result.transaction_id, result.message),
        Err(e) => println!("  Order failed: {}\n", e),
    }

    // --- OrderService with PayPal ---
    println!("--- Order Service (PayPal) ---");
    let paypal = PayPalProcessor::new("client_abc", "secret_xyz", true);
    let mut service = OrderService::new(Box::new(paypal));

    match service.place_order(
        "ORD-002",
        2500,
        "EUR",
        "buyer@example.com",
        "5555555555554444",
        "06/26",
        "456",
    ) {
        Ok(result) => println!("  Order placed: {} — {}\n", result.transaction_id, result.message),
        Err(e) => println!("  Order failed: {}\n", e),
    }

    // --- Payment Router ---
    println!("--- Payment Router ---");
    let mut router = PaymentRouter::new();
    router.register(PaymentProvider::Stripe(Box::new(StripeProcessor::new(
        "sk_test_abc123",
        "whsec_xyz789",
    ))));
    router.register(PaymentProvider::PayPal(Box::new(PayPalProcessor::new(
        "client_abc",
        "secret_xyz",
        true,
    ))));
    router.register(PaymentProvider::BankTransfer(Box::new(
        BankTransferProcessor::new("First National", "021000021"),
    )));

    let test_request = PaymentRequest {
        order_id: "ORD-003".to_string(),
        amount: Money::new(9999, "USD"),
        customer_email: "test@example.com".to_string(),
        card_number: "4242424242424242".to_string(),
        card_expiry: "12/27".to_string(),
        card_cvv: "123".to_string(),
        metadata: HashMap::new(),
    };

    for provider in &["stripe", "paypal", "bank_transfer"] {
        match router.process(provider, &test_request) {
            Ok(result) => println!("  {}: {} ({})", provider, result.status, result.message),
            Err(e) => println!("  {}: error — {}", provider, e),
        }
    }

    // --- Health checks ---
    println!("\n--- Health Checks ---");
    let mut processors: Vec<Box<dyn PaymentProcessor>> = vec![
        Box::new(StripeProcessor::new("sk_test_abc123", "whsec_xyz789")),
        Box::new(PayPalProcessor::new("client_abc", "secret_xyz", true)),
        Box::new(BankTransferProcessor::new("First National", "021000021")),
    ];

    for processor in &mut processors {
        match processor.health_check() {
            Ok(()) => println!("  {} — healthy", processor.name()),
            Err(e) => println!("  {} — unhealthy: {}", processor.name(), e),
        }
    }
}
