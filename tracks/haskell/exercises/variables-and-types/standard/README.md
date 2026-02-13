# Exercise: Environment Config Parser

## Scenario

Your team is building a Haskell service that reads configuration from environment variables (represented as key-value string pairs). Unlike Go where you'd use struct fields with zero values, Haskell has no zero values — you need to use `Maybe` for optional fields and explicit parsing for type conversions.

## Brief

Implement a `Config` type and a `parseConfig` function that:

1. Takes a list of key-value pairs `[(String, String)]` (simulating env vars)
2. Parses string values into typed Haskell values
3. Uses `Maybe` for optional fields (not sentinel values)
4. Uses `newtype` for type-safe port and timeout values
5. Returns `Either String Config` — `Left` with error message or `Right` with parsed config

## Acceptance Criteria

- [ ] `newtype Port = Port Int` and `newtype TimeoutSec = TimeoutSec Double` defined for type safety
- [ ] `Config` record with: `cfgHost :: String`, `cfgPort :: Port`, `cfgDebug :: Bool`, `cfgMaxRetries :: Maybe Int`, `cfgTimeout :: TimeoutSec`, `cfgEnv :: String`
- [ ] `parseConfig :: [(String, String)] -> Either String Config` parses the env list
- [ ] `HOST` is required — return `Left` if missing or empty
- [ ] `PORT` defaults to `8080` if not present. If present but not a valid integer, return `Left`
- [ ] `MAX_RETRIES` is `Nothing` when not set, `Just n` when set (even if `n` is 0)
- [ ] `DEBUG` parses `"true"/"false"/"1"/"0"` — defaults to `False`
- [ ] `TIMEOUT_SEC` defaults to `30.0` if not set
- [ ] `ENVIRONMENT` defaults to `"development"` if not set
- [ ] `showConfig :: Config -> String` produces a human-readable summary

## Constraints

- Use only base library (`Data.Maybe`, `Data.Char`, `Text.Read`)
- No third-party packages
- Use `newtype` for Port and TimeoutSec (not bare Int/Double)
- Use `Either String` for error handling (not exceptions)

## Concepts Exercised

- Immutable bindings and `where`/`let` clauses
- `Maybe` for optional values (vs Go's nil pointers)
- `Either` for error handling (vs Go's `(value, error)` returns)
- `newtype` for type safety (vs Go's `type X int`)
- Pattern matching for destructuring
- Explicit type conversions (`read`, `reads`)
- List operations for lookup

## Hints

<details>
<summary>Hint 1: Safe string-to-number parsing</summary>

Don't use `read` directly — it crashes on bad input. Use `reads` or `readMaybe`:

```haskell
import Text.Read (readMaybe)

parsePort :: String -> Maybe Int
parsePort s = readMaybe s
```
</details>

<details>
<summary>Hint 2: Looking up keys</summary>

```haskell
-- lookup is built-in: lookup :: Eq a => a -> [(a, b)] -> Maybe b
lookup "HOST" env  -- returns Maybe String
```
</details>

<details>
<summary>Hint 3: Either for error chaining</summary>

```haskell
-- Pattern match to chain Either results:
case parsePort portStr of
  Nothing -> Left ("invalid PORT: " ++ portStr)
  Just p  -> Right (Port p)
```
</details>
