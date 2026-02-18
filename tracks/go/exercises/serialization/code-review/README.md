# Code Review: Config File Loader

## PR Context

A junior developer on your team has implemented a configuration loader for the webhook relay service. The loader reads a JSON config file that describes webhook endpoints, their retry policies, and routing rules. The config format supports an `extensions` field for future customization.

The PR has been submitted for review before going to staging. The developer's self-tests pass. You've been asked to review it before merge.

## Your Task

1. Read `codebase/loader.go` — the config loader being added
2. Write your review in `my-review.md` using the template
3. Look for issues related to:
   - Missing validation after unmarshal
   - Using `interface{}` where a typed struct would be safer
   - Ignored decoder errors
   - Missing `json.RawMessage` for forward-compatible unknown fields
   - Encoding/json conventions and struct tag correctness
4. After writing your review, compare against `expert-review.md`

## Review Checklist

For each issue you find, categorize it:

- **Critical** — Will cause incorrect behavior, panic, or data loss
- **Major** — Design problem that will hurt as the code grows
- **Minor** — Style, naming, or small improvements

## Hints

<details>
<summary>Hint 1: How many issues?</summary>

There are 5 issues: 1 critical, 2 major, 2 minor.
</details>

<details>
<summary>Hint 2: Focus areas</summary>

Look at: (1) what happens after Decode returns nil — is the data validated? (2) what type is used for the extensions field and what risks does that create? (3) what happens to the error from Decode? (4) how would you add new top-level fields in the future without breaking old readers?
</details>

<details>
<summary>Hint 3: The decoder error</summary>

`dec.Decode(&cfg)` returns an error. What does the code do with it?
</details>

<details>
<summary>Hint 4: Forward compatibility</summary>

What is `json.RawMessage` for? How would you use it to preserve unknown fields for forwarding to a downstream service?
</details>
