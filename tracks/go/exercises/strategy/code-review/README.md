# Code Review Exercise -- Payment Processing with Strategy Pattern

## Overview

**Scenario:** A teammate has submitted a PR implementing a payment processing system using the Strategy pattern. The system supports multiple payment providers (Stripe, PayPal, bank transfer) and uses the strategy pattern to select the provider at runtime. Your task is to review the code for correctness, design quality, and adherence to Go idioms.

**Your Task:** Review the proposed changes in `proposed.go` and provide feedback on:
- Correctness and bugs
- Interface design (is it well-factored?)
- Thread safety
- Error handling
- Go idioms and best practices
- Strategy pattern usage

## Context

**Codebase:** An e-commerce platform's payment service. Currently supports Stripe only. This PR adds PayPal and bank transfer support using the Strategy pattern.

**Feature:** Multi-provider payment processing with runtime provider selection based on user preference and transaction amount.

**Files Changed:** `proposed.go` (new file -- the payment processing module)

## The Pull Request

### PR Description
```
feat: add multi-provider payment processing

Implements the strategy pattern for payment processing. Users can now
pay via Stripe, PayPal, or bank transfer. The processor selects the
right provider based on user settings.

- Added PaymentProcessor interface with all payment operations
- Added Stripe, PayPal, and BankTransfer implementations
- Added PaymentService that orchestrates payments
- Added transaction logging
```

### Changes

Review the code in `proposed.go`.

## Your Review

Use `my-review.md` to document your findings:

1. **Critical Issues** -- Bugs, security vulnerabilities, breaking changes
2. **Major Concerns** -- Design problems, performance issues, maintainability
3. **Minor Suggestions** -- Style, naming, small refactors
4. **Positive Feedback** -- What was done well

## Hints (Progressive)

<details>
<summary>Hint 1: Look at the interface</summary>

How many methods does `PaymentProcessor` have? Is this a well-designed interface or a "God interface"? Consider the Interface Segregation Principle.

</details>

<details>
<summary>Hint 2: Look at the struct fields</summary>

What type is `PaymentService.processor`? Is it the interface type, or a concrete type? What does this mean for swappability?

</details>

<details>
<summary>Hint 3: Look at shared state</summary>

The `StripeProcessor` has a `transactionLog` field. What happens when multiple goroutines call `Charge` simultaneously? Is the `mu` mutex protecting everything it should?

</details>

<details>
<summary>Hint 4: Look at nil checks</summary>

What happens if `PaymentService` is created without a processor? Trace through `ProcessPayment` with a nil processor.

</details>

## Learning Objectives

This exercise practices:
- [x] Reading unfamiliar code quickly
- [x] Identifying bugs without running code
- [x] Spotting interface design problems
- [x] Thread safety awareness
- [x] Giving constructive feedback
- [x] Understanding Go idioms and Strategy pattern best practices

## Notes

**Difficulty:** Medium

**Estimated Time:** 20-30 min

**Related Concepts:** [[patterns/strategy]], [[fundamentals/interfaces-and-traits]], [[fundamentals/concurrency]]
