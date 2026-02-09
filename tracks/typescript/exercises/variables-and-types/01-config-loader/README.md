# Config Loader Exercise

## Scenario

You're building a Node.js microservice that loads configuration from environment variables with type-safe parsing and validation. The config includes server settings, timeouts, and feature flags. Invalid values should fail fast with clear errors.

## Brief

Implement a type-safe configuration loader that:
- Parses environment variables into a strongly-typed `Config` interface
- Provides sensible defaults for missing values
- Validates types and ranges
- Returns helpful errors for invalid values
- Uses TypeScript's type system to prevent misuse

## Acceptance Criteria

- [ ] `Config` interface with fields: `host`, `port`, `timeout`, `maxConnections`, `debug`
- [ ] `loadConfig()` function that reads from `Record<string, string>` (simulating `process.env`)
- [ ] Default values: host="localhost", port=8080, timeout=30000, maxConnections=100, debug=false
- [ ] Port validation: must be 1-65535
- [ ] Timeout validation: must be positive
- [ ] Type-safe: no `any` types, full TypeScript strict mode
- [ ] All tests pass

## Constraints

- Enable TypeScript strict mode (`strict: true`)
- No `any` or `as any` casts
- Handle missing keys gracefully (use defaults)
- Handle invalid values explicitly (throw errors)
- Parse strings to appropriate types (number, boolean)

## Files

- `starter/` — Scaffold with TODOs
- `starter/config.test.ts` — Test suite (Jest/Vitest)

## Getting Started

```bash
cd starter
npm test  # Should fail initially
```

## Hints

<details>
<summary>Hint 1: Parsing numbers</summary>

Use `parseInt()` or `Number()` to parse strings to numbers. Check for `NaN` after parsing:

```typescript
const port = parseInt(env.PORT ?? '', 10);
if (isNaN(port)) {
    throw new Error(`Invalid port: ${env.PORT}`);
}
```

</details>

<details>
<summary>Hint 2: Parsing booleans</summary>

Environment variables are always strings. Parse "true"/"false":

```typescript
const debug = env.DEBUG === 'true';
```

</details>

<details>
<summary>Hint 3: Type-safe defaults</summary>

Start with an object containing all defaults, then override:

```typescript
const config: Config = {
    host: env.HOST ?? 'localhost',
    port: parsePort(env.PORT),
    // ...
};
```

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
