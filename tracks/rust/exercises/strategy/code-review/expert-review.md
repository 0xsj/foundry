# Expert Review — Payment Processing Strategies

**PR:** #247 — Pluggable payment processing
**Reviewer:** Senior Rust Developer
**Verdict:** Request changes

---

## Critical Issues

### 1. Card validation never actually checks expiry date

**File:** `proposed.rs`, `StripeProcessor::validate_card`

The `validate_card` method parses the expiry month and year but never checks if the card is expired. After the Luhn check passes, it always returns `true`:

```rust
let _month: u32 = parts[0].parse().unwrap_or(0);
let _year: u32 = parts[1].parse().unwrap_or(0);
true  // BUG: always returns true for expired cards
```

This means expired cards pass validation. In production, this would allow charges on expired cards, which would be rejected by the payment processor anyway — but we'd incur the API call cost and create a bad user experience (card appears valid, then payment fails).

**Fix:** Actually validate the expiry date:

```rust
use std::time::SystemTime;
// Or pass current date as parameter for testability

let month: u32 = parts[0].parse().unwrap_or(0);
let year: u32 = parts[1].parse().unwrap_or(0);
if month < 1 || month > 12 {
    return false;
}
// Compare with current date
// year is 2-digit — add 2000
let full_year = 2000 + year;
// ... check if full_year/month >= current year/month
```

**Severity:** Critical. Expired cards will be charged and fail at the processor level.

### 2. `unwrap_or(0)` masks parse errors silently

In `validate_card`, month/year parsing uses `unwrap_or(0)`:

```rust
let _month: u32 = parts[0].parse().unwrap_or(0);
```

A month of `0` or year of `0` is never valid. If parsing fails (e.g., expiry is "AB/CD"), the code silently treats it as month 0, year 0. Even with the expiry check fixed, this could lead to unexpected behavior.

**Fix:** Return `false` on parse failure:

```rust
let month: u32 = match parts[0].parse() {
    Ok(m) if (1..=12).contains(&m) => m,
    _ => return false,
};
```

---

## Major Concerns

### 3. Trait has too many required methods — Interface Segregation violation

The `PaymentProcessor` trait has **8 required methods**. This forces every payment provider to implement every capability, even when it doesn't make sense:

- `BankTransferProcessor::validate_card` returns `false` — bank transfers don't use cards
- `PayPalProcessor::validate_card` returns `true` — PayPal redirects to their own flow
- `BankTransferProcessor::setup_recurring` returns `Err` — not supported
- `PayPalProcessor::setup_recurring` returns `Err` — not yet implemented

This is the Interface Segregation Principle (ISP) violation. The trait is a "fat interface" that forces implementors to provide nonsensical stubs.

**Fix:** Split into focused traits:

```rust
trait PaymentProcessor {
    fn process_payment(&self, request: &PaymentRequest) -> Result<PaymentResult, String>;
    fn name(&self) -> &str;
    fn calculate_fee(&self, amount: &Money) -> u64;
    fn health_check(&self) -> Result<(), String>;
}

trait Refundable {
    fn refund(&self, transaction_id: &str, amount: &Money) -> Result<PaymentResult, String>;
}

trait RecurringBilling {
    fn setup_recurring(&self, customer_id: &str, amount: &Money, interval_days: u32) -> Result<String, String>;
    fn cancel_recurring(&self, subscription_id: &str) -> Result<(), String>;
}
```

Then implementations opt in to the capabilities they support. The caller can check `if let Some(refundable) = processor.as_any().downcast_ref::<dyn Refundable>()` or use a capability enum.

### 4. `&mut self` used where `&self` would suffice

`process_payment`, `refund`, `setup_recurring`, `cancel_recurring`, and `health_check` all take `&mut self`. Looking at the implementations:

- `StripeProcessor` uses `&mut self` to increment `request_count` — but a request counter is better handled by middleware/metrics, not inside the strategy itself
- `PayPalProcessor` and `BankTransferProcessor` don't mutate any state in most methods

Using `&mut self` has real consequences in Rust:
- You can't call the processor from multiple threads without a `Mutex`
- You can't have concurrent payments through the same processor
- If the processor is behind `Arc<Mutex<dyn PaymentProcessor>>`, every payment serializes behind the lock

**Fix:** Use `&self` for methods that don't genuinely need mutation. Move the request counter to an `AtomicU64` or external metrics system.

### 5. `PaymentRouter` enum defeats the strategy pattern

The `PaymentRouter` wraps trait objects in an enum:

```rust
enum PaymentProvider {
    Stripe(Box<dyn PaymentProcessor>),
    PayPal(Box<dyn PaymentProcessor>),
    BankTransfer(Box<dyn PaymentProcessor>),
}
```

This is an anti-pattern. The enum creates a closed set — adding a new provider requires modifying the `PaymentProvider` enum AND the match in `PaymentRouter::process`. This is exactly what the strategy pattern is designed to avoid.

The trait object (`Box<dyn PaymentProcessor>`) already provides open extensibility. Wrapping it in an enum adds complexity with no benefit.

**Fix:** Replace with a simple `HashMap`:

```rust
struct PaymentRouter {
    providers: HashMap<String, Box<dyn PaymentProcessor>>,
}

impl PaymentRouter {
    fn register(&mut self, provider: Box<dyn PaymentProcessor>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    fn process(&mut self, provider_name: &str, request: &PaymentRequest) -> Result<PaymentResult, String> {
        let provider = self.providers.get_mut(provider_name)
            .ok_or_else(|| format!("no provider: {}", provider_name))?;
        provider.process_payment(request)
    }
}
```

### 6. `validate_card` doesn't belong on the payment strategy

Card validation (Luhn algorithm, expiry check) is universal — it doesn't vary by payment provider. Putting it on the trait means:
- The same Luhn code is duplicated across providers (or absent — PayPal/Bank just return `true`/`false`)
- Providers that don't use cards have nonsensical implementations
- The validation logic can't be tested independently

**Fix:** Extract card validation into a standalone function or struct:

```rust
fn validate_card(card_number: &str, expiry: &str) -> bool {
    luhn_check(card_number) && expiry_valid(expiry)
}
```

Call it in `OrderService::place_order` before delegating to the payment processor, not through the processor itself.

---

## Minor Suggestions

### 7. `Money::currency` should be an enum, not a String

Using `String` for currency allows invalid values like `"NOPE"` or `""`. An enum (or a validated newtype) prevents this:

```rust
enum Currency {
    USD, EUR, GBP, // ...
}
```

Or at minimum, validate in the constructor.

### 8. `PaymentRequest` contains raw card data

Storing card numbers, CVVs, and expiry dates as plain `String` fields is a PCI compliance concern. In production, this should be tokenized before reaching the application layer. Consider using a `CardToken` newtype that can only be created through a tokenization service.

### 9. `OrderService` takes too many parameters in `place_order`

Seven positional parameters are hard to read and easy to misorder:

```rust
service.place_order("ORD-001", 4999, "USD", "customer@example.com", "4242424242424242", "12/27", "123")
```

Use a builder or accept a `PaymentRequest` directly.

### 10. Missing `Debug` derives on strategy structs

The strategy structs don't derive `Debug`. This makes logging and error messages harder. Add `#[derive(Debug)]` or implement `fmt::Debug` manually (redacting secrets like API keys).

### 11. `Box<dyn PaymentProcessor>` in `OrderService` when generics would work

`OrderService` is created once at startup with a fixed provider. It never swaps providers at runtime. A generic parameter would communicate this and avoid the heap allocation:

```rust
struct OrderService<P: PaymentProcessor> {
    payment: P,
    order_log: Vec<(String, PaymentResult)>,
}
```

However, at ~10K orders/day, the performance difference is negligible. This is a clarity/design signal issue, not a performance one. Either approach is defensible.

---

## Positive Feedback

1. **Money type with cents-based arithmetic** — Using `u64` cents instead of floating point is correct for financial calculations. Good catch.

2. **Clear provider naming** — Each provider has a `name()` method used for routing and logging. This is a good pattern.

3. **Error messages are descriptive** — The error strings include context (provider name, transaction IDs). In production these should be structured errors, but for this stage the strings are informative.

4. **Fee calculation is provider-specific** — Correctly recognizing that fees vary by provider and making it part of the strategy interface.

5. **Sandbox/live mode for PayPal** — Good separation of test vs production environments.

---

## Summary

The biggest issues are:

1. **Fat interface** — Split `PaymentProcessor` into focused traits (ISP)
2. **Enum wrapping trait objects** — Remove the `PaymentProvider` enum, use `HashMap<String, Box<dyn>>` directly
3. **`&mut self` everywhere** — Use `&self` for stateless operations to enable concurrency
4. **Card validation bug** — Actually check the expiry date
5. **Card validation placement** — Extract to standalone function, not on the strategy

The overall architecture is sound — the developer correctly identified the strategy pattern as the right approach and implemented it with trait objects. The issues are in the details of trait design and Rust-specific idioms.

---

## Related Concepts

- [[strategy]] — Strategy pattern fundamentals
- [[interface-segregation]] — ISP principle
- [[rust-object-safety]] — Trait object rules
- [[rust-send-sync]] — Thread safety bounds
