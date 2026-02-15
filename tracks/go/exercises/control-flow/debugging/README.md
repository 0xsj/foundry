# Debugging Exercise: Configuration Validator

## Symptoms

A configuration validation system has been deployed but is behaving incorrectly:

1. **Bug Report #1:** Valid configurations are being rejected
2. **Bug Report #2:** Invalid port numbers (negative, too large) are passing validation
3. **Bug Report #3:** The validator reports "0 errors found" even when errors exist
4. **Bug Report #4:** When multiple validation rules fail, only some are reported

Your task: Find and fix the bugs without looking at the solution first.

## Context

The validator checks configuration files for a web service. It should:
- Validate port is in range 1-65535
- Validate timeout is positive
- Validate at least one admin email is provided
- Collect ALL validation errors before returning

## Files

- `buggy.go` - The buggy validator implementation
- `buggy_test.go` - Failing tests that expose the bugs

## Instructions

1. Run the tests: `go test -v`
2. Examine the failing tests to understand what's broken
3. Read the code and identify the bugs
4. Fix the bugs one at a time
5. Re-run tests after each fix
6. Once all tests pass, compare your fixes to the solution

## Debugging Strategy

1. **Start with tests** - understand expected vs actual behavior
2. **Trace control flow** - follow the execution path
3. **Check loop bounds** - off-by-one errors are common
4. **Check conditionals** - are you testing the right thing?
5. **Check early returns** - are you exiting too early?

## Hints

<details>
<summary>Hint 1: Check range conditions</summary>

Look at how port validation checks the range. Is the condition correct?
</details>

<details>
<summary>Hint 2: Check loop behavior</summary>

Look at the admin email validation loop. Does it check all emails?
</details>

<details>
<summary>Hint 3: Check error reporting</summary>

Look at how errors are collected and returned. Is the count correct?
</details>

<details>
<summary>Hint 4: Look for early returns</summary>

Are there any early returns that skip remaining validations?
</details>
