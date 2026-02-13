# Expert Review: Inventory Management System

## Critical Issues

### 1. `fromJust` crashes on missing product (line 37)

`getPrice` calls `fromJust (findProduct pid inv)`. If the product doesn't exist, `fromJust Nothing` throws a runtime exception. This is Haskell's equivalent of a nil pointer panic in Go.

```haskell
-- Bug: crashes if product doesn't exist
getPrice pid inv = prodPrice (fromJust (findProduct pid inv))

-- Fix: return Maybe, let the caller decide
getPrice :: ProductID -> Inventory -> Maybe Price
getPrice pid inv = prodPrice <$> findProduct pid inv
-- Or with pattern matching:
getPrice pid inv = case findProduct pid inv of
  Nothing -> Nothing
  Just p  -> Just (prodPrice p)
```

**Concept:** `fromJust` is a partial function — it's an escape hatch from `Maybe` that reintroduces crashes. Using it defeats the purpose of `Maybe`. The compiler can't help you if you bypass its safety mechanisms.

### 2. `mostExpensive` and `averagePrice` crash on empty inventory (lines 51, 55)

`foldr1` crashes on an empty list (it's partial, like `head`). `averagePrice` divides by `length inv` which is 0 for an empty inventory → `NaN` or crash. The `inventoryReport` function calls both, so printing a report for an empty inventory crashes.

```haskell
-- Bug: foldr1 on [] throws an exception
mostExpensive inv = foldr1 (\a b -> ...) inv

-- Fix: return Maybe
mostExpensive :: Inventory -> Maybe Product
mostExpensive [] = Nothing
mostExpensive inv = Just (foldr1 (\a b -> if prodPrice a >= prodPrice b then a else b) inv)

-- Bug: division by zero
averagePrice inv = total / fromIntegral count  -- count is 0 for []

-- Fix: return Maybe
averagePrice :: Inventory -> Maybe Double
averagePrice [] = Nothing
averagePrice inv = Just (sum (map prodPrice inv) / fromIntegral (length inv))
```

**Concept:** Partial functions (`head`, `tail`, `foldr1`, `fromJust`) are the #1 source of runtime crashes in Haskell. Always handle the empty case explicitly. Use `Maybe` or pattern match on `[]`.

---

## Major Concerns

### 3. `Double` for money — floating-point precision (lines 7, 47)

Using `Double` for prices means `0.1 + 0.2 /= 0.3`. In an inventory system, this compounds across thousands of products. A total value of `$1000.00` might display as `$999.9999999999998`.

```haskell
-- Problem: Double has floating-point precision issues
type Price = Double

-- Better: use Integer cents (or a Rational/Decimal library)
newtype Cents = Cents Integer deriving (Show, Eq, Ord)
-- $10.50 = Cents 1050
```

**Concept:** This is a cross-language issue (same problem in Go with `float64` for money). Use integer cents or a decimal library for financial calculations. Haskell's `Rational` type is exact but slow; for production, use the `Decimal` package.

### 4. Bare type aliases provide no type safety (lines 6-8)

`ProductID`, `Price`, and `Quantity` are type aliases (`type`), not newtypes. This means `String`, `ProductID`, and `prodName` are all interchangeable — you can pass a product name where a product ID is expected, and the compiler won't catch it.

```haskell
-- Bug: these are all just String, the compiler can't tell them apart
type ProductID = String
findProduct "Widget X" inv  -- passing a name, not an ID — compiles fine!

-- Fix: use newtype for distinct identity
newtype ProductID = ProductID String deriving (Show, Eq)
-- Now findProduct (ProductID "Widget X") is required — intent is explicit
```

**Concept:** `type` creates aliases (same type, zero safety). `newtype` creates distinct types (compiler-enforced safety, zero runtime cost). Use `newtype` when you want the compiler to prevent mixing up values of the same underlying type.

---

## Minor Suggestions

### 5. `addProduct` appends to end — O(n) per insertion (line 27)

```haskell
addProduct p inv = inv ++ [p]  -- O(n) — traverses entire list
```

`++` on `inv ++ [p]` is O(n) in the length of `inv`. Prepending with `p : inv` is O(1). For an inventory that grows over time, this adds up. If order matters, prepend and reverse when needed, or use a `Data.Map` keyed by `ProductID`.

```haskell
-- Better: O(1) prepend
addProduct p inv = p : inv

-- Best for lookup-heavy use: Map
type Inventory = Map ProductID Product
```

### 6. `inventoryReport` traverses the list multiple times (lines 68-74)

`inventoryReport` calls `map showProduct`, `totalValue` (one traversal), `averagePrice` (two traversals: `map` + `length`), `mostExpensive` (one traversal), and `length` (one traversal). That's 6+ traversals of the same list.

For a learning exercise this is fine, but worth noting: in production Haskell, you'd fold once to collect all stats in a single pass, or use a stricter data structure.

---

## Positive Feedback

- Clean record syntax for `Product` — good use of Haskell's built-in accessor functions
- `findProduct` correctly returns `Maybe` — doesn't crash on missing products
- `updateQty` uses record update syntax `p { prodQty = newQty }` — idiomatic
- `lowStock` is clean and composable with `filter`
- `showProduct` formatting is readable
- Good separation of concerns — pure functions, no IO mixed in

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `fromJust` crashes on missing product | Partial function, Maybe |
| 2 | Critical | `mostExpensive`/`averagePrice` crash on `[]` | Empty list, partial functions |
| 3 | Major | `Double` for money | Floating-point precision |
| 4 | Major | Type aliases instead of newtypes | type vs newtype safety |
| 5 | Minor | `++` append is O(n) | List performance |
| 6 | Minor | Multiple list traversals | Efficiency awareness |
