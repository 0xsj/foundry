# Code Review Exercise — Payment Processing Strategies

## Overview

**Scenario:** A junior developer has submitted a PR implementing pluggable payment processing strategies for an e-commerce platform. The system needs to support multiple payment providers (Stripe, PayPal, bank transfer) with the ability to add new providers without modifying existing code.

**Your Task:** Review the proposed changes in `proposed.rs` and provide feedback on:
- Correctness and bugs
- Trait design and API quality
- Performance concerns (unnecessary allocations, boxing)
- Idiomatic Rust patterns
- Strategy pattern anti-patterns

## Context

**Codebase:** E-commerce order processing service written in Rust. Handles ~10,000 orders per day. Payment processing is called once per order — not a hot loop.

**Feature:** Replace the hardcoded Stripe integration with a pluggable payment strategy system so the platform can support multiple payment providers.

**Files Changed:** `proposed.rs` (single file for review — in production this would be split across modules)

## The Pull Request

### PR Description
```
feat: pluggable payment processing

Adds a PaymentProcessor trait and implementations for Stripe, PayPal,
and BankTransfer. Uses the strategy pattern to make payment providers
swappable. Added an enum-based approach for internal routing.

Closes #247
```

### Changes

See `proposed.rs` for the full implementation.

## Your Review

Use `my-review.md` to document your findings. Structure your review as:

1. **Critical Issues** — Bugs, correctness problems, things that would break in production
2. **Major Concerns** — Design problems, performance issues, maintainability red flags
3. **Minor Suggestions** — Naming, style, small improvements
4. **Positive Feedback** — What was done well

After completing your review, compare with `expert-review.md`.

## Hints (Progressive)

<details>
<summary>Hint 1: Look at the trait design</summary>

Count the number of required methods on the `PaymentProcessor` trait. Is every method genuinely needed for ALL payment providers? What happens when you add a new provider that doesn't support refunds?

</details>

<details>
<summary>Hint 2: Check the mutability</summary>

Look at which methods take `&mut self` vs `&self`. Does the `process_payment` method actually need to mutate the strategy? What are the implications for concurrent usage?

</details>

<details>
<summary>Hint 3: Examine the dispatch choices</summary>

The code uses `Box<dyn PaymentProcessor>` everywhere. Given that the payment provider is chosen once at startup and doesn't change, is dynamic dispatch necessary? Also look at the `PaymentRouter` — is that enum actually implementing the strategy pattern?

</details>

<details>
<summary>Hint 4: Look for the subtle bug</summary>

Check the `validate_card` method in one of the implementations. There's a logic error that would accept invalid cards in production.

</details>

## Learning Objectives

This exercise practices:
- [ ] Reading unfamiliar Rust code and understanding the design intent
- [ ] Identifying trait design smells (too many required methods, wrong mutability)
- [ ] Spotting unnecessary `Box<dyn>` when generics would work
- [ ] Recognizing strategy pattern anti-patterns (enum that defeats the purpose)
- [ ] Evaluating `&mut self` vs `&self` tradeoffs
- [ ] Giving constructive, actionable feedback

## Notes

**Difficulty:** Medium

**Estimated Time:** 20-30 min

**Related Concepts:** [[strategy]], [[patterns/rust/strategy]], [[rust-object-safety]]
