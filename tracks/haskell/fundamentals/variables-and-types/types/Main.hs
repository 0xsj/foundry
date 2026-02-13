-- ============================================================================
-- Types Deep Dive — Haskell
-- ============================================================================
-- Exploring tuples, lists, Maybe, newtype, and type aliases.
-- Run with: runhaskell Main.hs
-- ============================================================================

module Main where

import Data.Maybe (fromMaybe, isJust, isNothing)

-- ========================================================================
-- 1. TUPLES
-- Fixed size, heterogeneous. Like a Go struct with no field names.
-- ========================================================================

point :: (Int, Int)
point = (3, 4)

person :: (String, Int, Bool)
person = ("Alice", 30, True)

-- Pattern matching to destructure (no indexing!)
getName :: (String, Int, Bool) -> String
getName (name, _, _) = name

getAge :: (String, Int, Bool) -> Int
getAge (_, age, _) = age

-- ========================================================================
-- 2. LISTS
-- Homogeneous, linked list. The core data structure in Haskell.
-- ========================================================================

nums :: [Int]
nums = [1, 2, 3, 4, 5]

-- List is syntactic sugar for cons chains:
-- [1, 2, 3] is really 1 : 2 : 3 : []

-- Prepend is O(1)
withZero :: [Int]
withZero = 0 : nums  -- [0, 1, 2, 3, 4, 5]

-- Concatenation is O(n) in the left list
combined :: [Int]
combined = [10, 20] ++ nums  -- [10, 20, 1, 2, 3, 4, 5]

-- List comprehension (like Python's, Go doesn't have these)
evens :: [Int]
evens = [x | x <- [1..20], even x]  -- [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

squares :: [Int]
squares = [x * x | x <- [1..10]]  -- [1, 4, 9, 16, 25, ...]

-- Infinite list! (lazy evaluation — only computes what you need)
-- Uncomment to explore in GHCi: take 5 naturals
-- naturals :: [Int]
-- naturals = [1..]

-- ========================================================================
-- 3. TYPE ALIASES
-- `type` creates a synonym. Name and String are the SAME type.
-- ========================================================================

type Name = String
type Age = Int
type Person = (Name, Age)

greet :: Person -> String
greet (name, _) = "Hello, " ++ name ++ "!"

-- ========================================================================
-- 4. NEWTYPE
-- Creates a DISTINCT type. Zero runtime cost.
-- Can't mix Celsius and Fahrenheit by accident.
-- ========================================================================

newtype Celsius = Celsius Double deriving (Show)
newtype Fahrenheit = Fahrenheit Double deriving (Show)

celsiusToFahr :: Celsius -> Fahrenheit
celsiusToFahr (Celsius c) = Fahrenheit (c * 9 / 5 + 32)

-- Try this — it won't compile:
-- badConvert = celsiusToFahr (Fahrenheit 100)
-- ERROR: Expected Celsius, got Fahrenheit

-- ========================================================================
-- 5. MAYBE — No Zero Values
-- Haskell doesn't have nil/null. Use Maybe for optional values.
-- The compiler FORCES you to handle the Nothing case.
-- ========================================================================

-- Simulate a config lookup
type Config = [(String, String)]

lookupConfig :: String -> Config -> Maybe String
lookupConfig _ [] = Nothing
lookupConfig key ((k, v):rest)
  | key == k  = Just v
  | otherwise = lookupConfig key rest

-- You MUST handle both cases
showConfigValue :: String -> Config -> String
showConfigValue key cfg =
  case lookupConfig key cfg of
    Nothing  -> key ++ ": <not set>"
    Just val -> key ++ ": " ++ val

-- Or use fromMaybe for a default
getWithDefault :: String -> String -> Config -> String
getWithDefault key def cfg = fromMaybe def (lookupConfig key cfg)

-- ========================================================================
-- 6. PRINT EVERYTHING
-- ========================================================================

sampleConfig :: Config
sampleConfig = [("host", "localhost"), ("port", "8080")]

main :: IO ()
main = do
  putStrLn "=== Tuples ==="
  putStrLn ("point: " ++ show point)
  putStrLn ("person: " ++ show person)
  putStrLn ("getName: " ++ getName person)
  putStrLn ("getAge: " ++ show (getAge person))

  putStrLn "\n=== Lists ==="
  putStrLn ("nums: " ++ show nums)
  putStrLn ("0 : nums: " ++ show withZero)
  putStrLn ("head nums: " ++ show (head nums))
  putStrLn ("tail nums: " ++ show (tail nums))
  putStrLn ("length nums: " ++ show (length nums))
  putStrLn ("evens up to 20: " ++ show evens)
  putStrLn ("squares 1-10: " ++ show squares)

  putStrLn "\n=== Type Alias ==="
  putStrLn (greet ("Bob", 25))

  putStrLn "\n=== Newtype ==="
  let boiling = Celsius 100
  putStrLn ("100°C in Fahrenheit: " ++ show (celsiusToFahr boiling))

  putStrLn "\n=== Maybe ==="
  putStrLn (showConfigValue "host" sampleConfig)
  putStrLn (showConfigValue "port" sampleConfig)
  putStrLn (showConfigValue "debug" sampleConfig)
  putStrLn ("timeout with default: " ++ getWithDefault "timeout" "30" sampleConfig)

  let val = lookupConfig "host" sampleConfig
  putStrLn ("isJust host: " ++ show (isJust val))
  putStrLn ("isNothing host: " ++ show (isNothing val))
  putStrLn ("isNothing missing: " ++ show (isNothing (lookupConfig "nope" sampleConfig)))

  putStrLn "\ndone"
