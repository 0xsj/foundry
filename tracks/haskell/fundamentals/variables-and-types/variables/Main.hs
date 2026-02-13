-- ============================================================================
-- Variables and Types — Haskell
-- ============================================================================
-- Run with: runhaskell Main.hs
-- Or load in GHCi: ghci Main.hs, then type main
-- ============================================================================

module Main where

-- ========================================================================
-- 1. BINDINGS (not variables!)
-- Haskell has no mutable variables. Every binding is permanent.
-- These are like `const` in JS — there's no `let` or `var` option.
-- ========================================================================

myName :: String
myName = "foundry"

myAge :: Int
myAge = 42

pi' :: Double
pi' = 3.14159

isActive :: Bool
isActive = True

-- Try uncommenting this — it will fail:
-- myAge = 100  -- ERROR: Multiple declarations of 'myAge'

-- ========================================================================
-- 2. TYPE INFERENCE
-- Haskell can figure out types without annotations.
-- Use :t in GHCi to see what the compiler infers.
-- ========================================================================

inferredInt = 42          -- Integer (default for integer literals)
inferredDouble = 3.14     -- Double (default for fractional literals)
inferredString = "hello"  -- String (which is [Char])
inferredBool = False      -- Bool

-- ========================================================================
-- 3. TYPE CONVERSIONS
-- No implicit conversions. You must be explicit.
-- Compare with Go: int64(x) — same philosophy, different syntax.
-- ========================================================================

anInt :: Int
anInt = 42

-- Convert Int to Double
asDouble :: Double
asDouble = fromIntegral anInt  -- 42.0

-- Convert Double to Int (multiple strategies)
rounded :: Int
rounded = round 3.7       -- 4

truncated :: Int
truncated = truncate 3.7  -- 3

floored :: Int
floored = floor 3.7       -- 3

ceiled :: Int
ceiled = ceiling 3.7      -- 4

-- ========================================================================
-- 4. STRINGS ARE LISTS
-- String = [Char]. This means all list operations work on strings.
-- ========================================================================

greeting :: String
greeting = "hello, world"

firstChar :: Char
firstChar = head greeting     -- 'h'

restOfString :: String
restOfString = tail greeting  -- "ello, world"

stringLength :: Int
stringLength = length greeting  -- 12 (O(n) — walks the linked list!)

combined :: String
combined = "hello" ++ " " ++ "world"  -- "hello world"

-- ========================================================================
-- 5. LET AND WHERE
-- Local bindings. Two styles, same purpose.
-- ========================================================================

-- Using 'where' (top-down style)
circleArea :: Double -> Double
circleArea radius = myPi * radius * radius
  where
    myPi = 3.14159

-- Using 'let ... in' (bottom-up style)
cylinderVolume :: Double -> Double -> Double
cylinderVolume radius height =
  let base = 3.14159 * radius * radius
  in base * height

-- ========================================================================
-- 6. PRINT EVERYTHING
-- 'show' converts any Show-able type to String (like fmt.Sprintf in Go)
-- 'putStrLn' prints a string with newline (like fmt.Println in Go)
-- ========================================================================

main :: IO ()
main = do
  putStrLn "=== Bindings ==="
  putStrLn ("myName: " ++ myName)
  putStrLn ("myAge: " ++ show myAge)
  putStrLn ("pi': " ++ show pi')
  putStrLn ("isActive: " ++ show isActive)

  putStrLn "\n=== Type Inference ==="
  putStrLn ("inferredInt: " ++ show inferredInt)
  putStrLn ("inferredDouble: " ++ show inferredDouble)
  putStrLn ("inferredString: " ++ show inferredString)

  putStrLn "\n=== Conversions ==="
  putStrLn ("Int 42 as Double: " ++ show asDouble)
  putStrLn ("round 3.7: " ++ show rounded)
  putStrLn ("truncate 3.7: " ++ show truncated)
  putStrLn ("floor 3.7: " ++ show floored)
  putStrLn ("ceiling 3.7: " ++ show ceiled)

  putStrLn "\n=== Strings ==="
  putStrLn ("greeting: " ++ greeting)
  putStrLn ("first char: " ++ show firstChar)
  putStrLn ("length: " ++ show stringLength)
  putStrLn ("combined: " ++ combined)

  putStrLn "\n=== Functions ==="
  putStrLn ("circleArea 5: " ++ show (circleArea 5))
  putStrLn ("cylinderVolume 5 10: " ++ show (cylinderVolume 5 10))

  putStrLn "\ndone"
