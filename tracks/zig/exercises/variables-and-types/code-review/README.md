# Code Review: Feature Flag System

## PR Context

A teammate has submitted a pull request for a feature flag system that the team will use to control rollouts. The system supports three flag types:

- **Boolean flags**: simple on/off toggles (e.g., "dark_mode" = enabled)
- **Percentage flags**: gradual rollouts (e.g., "new_checkout" = 30% of users)
- **Variant flags**: A/B testing with named variants (e.g., "button_color" = "blue" or "red")

The code compiles and basic tests pass. Your team lead has asked you to review before merge. The PR author is relatively new to Zig and coming from a Python background.

## Your Task

1. Read `proposed.zig` carefully
2. Fill in `my-review.md` with issues you find
3. Categorize each issue as Critical, Major, or Minor
4. Note what was done well
5. Compare your review with `expert-review.md`

## What to Look For

- Type safety and correct use of Zig's type system
- Proper optional handling (no unsafe unwraps)
- Tagged union and switch exhaustiveness
- Redundant state and "make invalid states unrepresentable"
- String/slice comparison semantics
- Numeric type safety in casts
- Idiomatic Zig patterns

## Concepts Tested

- Tagged unions and exhaustive switches
- Optional types and safe unwrapping
- Explicit casting with `@intCast` and bounds
- Redundant enum that duplicates tagged union information
- Slice equality (`==` vs `std.mem.eql`)
- Named constants vs magic numbers
