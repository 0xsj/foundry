-- ============================================================================
-- Variables and Types — Haskell
-- ============================================================================
-- Fill in each section. Run with: runghc variables.hs
-- Or load in GHCi: ghci variables.hs, then call main
-- ============================================================================

module Main where

main :: IO ()
main = do
    putStrLn "=== 1. Bindings ==="

    -- ========================================================================
    -- 1. BINDINGS
    -- Haskell has no variables — only immutable bindings.
    -- let binds a name to a value inside do-blocks.
    -- ========================================================================

    -- a) bind an Int
    -- let x = TODO

    -- b) bind a String
    -- let name = TODO

    -- c) bind a Double
    -- let pi' = TODO

    -- d) bind a Bool
    -- let active = TODO

    -- Print them:
    -- putStrLn ("x = " ++ show x)
    -- putStrLn ("name = " ++ name)
    -- putStrLn ("pi = " ++ show pi')
    -- putStrLn ("active = " ++ show active)


    putStrLn "\n=== 2. Type Annotations ==="

    -- ========================================================================
    -- 2. TYPE ANNOTATIONS
    -- Use :: to specify the type of a binding.
    -- Try giving the wrong type — what error do you get?
    -- ========================================================================

    -- a) annotate as Int
    -- let (a :: Int) = 42

    -- b) annotate as Double
    -- let (b :: Double) = 42

    -- c) what happens if you write: let (c :: Int) = 3.14
    --    (try it and note the error)

    -- d) force a literal to Integer (arbitrary precision)
    -- let big = 2 ^ 100 :: Integer
    -- putStrLn ("2^100 = " ++ show big)


    putStrLn "\n=== 3. Numeric Conversions ==="

    -- ========================================================================
    -- 3. NUMERIC CONVERSIONS
    -- No implicit conversions. Use fromIntegral, round, etc.
    -- ========================================================================

    let intVal = 42 :: Int

    -- a) convert Int to Double using fromIntegral
    -- let doubleVal = TODO
    -- putStrLn ("as Double: " ++ show doubleVal)

    -- b) convert Double to Int using round
    -- let rounded = round (3.7 :: Double) :: Int
    -- putStrLn ("rounded: " ++ show rounded)

    -- c) try: let wrong = intVal + (3.14 :: Double)
    --    what error do you get? why?

    putStrLn ("intVal = " ++ show intVal)


    putStrLn "\n=== 4. Strings and Characters ==="

    -- ========================================================================
    -- 4. STRINGS AND CHARACTERS
    -- String = [Char]. A string is a list of characters.
    -- ========================================================================

    let greeting = "hello, world"

    -- a) get the first character
    -- let first = TODO
    -- putStrLn ("first: " ++ [first])   -- wrap Char in list to make String

    -- b) get the length
    -- putStrLn ("length: " ++ show (length greeting))

    -- c) concatenate two strings with ++
    -- let full = TODO
    -- putStrLn full

    -- d) reverse the string
    -- putStrLn ("reversed: " ++ reverse greeting)

    -- e) prove String == [Char]:
    -- putStrLn (show ("hi" == ['h', 'i']))   -- should be True

    putStrLn greeting


    putStrLn "\n=== 5. Lists ==="

    -- ========================================================================
    -- 5. LISTS
    -- Homogeneous, singly-linked. Core data structure in Haskell.
    -- ========================================================================

    let nums = [1, 2, 3, 4, 5] :: [Int]

    -- a) prepend 0 using the : (cons) operator
    -- let withZero = TODO
    -- print withZero

    -- b) concatenate two lists with ++
    -- let combined = TODO
    -- print combined

    -- c) use a range to create [1..10]
    -- let range = TODO
    -- print range

    -- d) use a list comprehension: double each number in [1..5]
    -- let doubles = TODO
    -- print doubles

    -- e) filter: keep only even numbers from [1..20]
    -- let evens = TODO
    -- print evens

    -- f) take the first 5 from an infinite list [1..]
    -- let firstFive = TODO
    -- print firstFive

    print nums


    putStrLn "\n=== 6. Tuples ==="

    -- ========================================================================
    -- 6. TUPLES
    -- Fixed-size, heterogeneous.
    -- ========================================================================

    -- a) create a 2-tuple (pair) of (String, Int)
    -- let person = TODO

    -- b) extract elements with fst and snd
    -- putStrLn ("name: " ++ fst person)
    -- putStrLn ("age: " ++ show (snd person))

    -- c) create a 3-tuple and extract the third element with pattern matching
    -- let triple = ("hello", 42, True)
    -- let (_, _, third) = triple
    -- putStrLn ("third: " ++ show third)

    putStrLn "(tuples done)"


    putStrLn "\n=== 7. Algebraic Data Types ==="

    -- ========================================================================
    -- 7. ALGEBRAIC DATA TYPES
    -- Custom types using `data`. See the definitions above main.
    -- ========================================================================

    -- a) create a Circle with radius 5
    -- let c = TODO
    -- putStrLn ("circle area: " ++ show (area c))

    -- b) create a Rectangle 3x4
    -- let r = TODO
    -- putStrLn ("rect area: " ++ show (area r))

    -- c) create a Color and check if it's warm
    -- let myColor = TODO
    -- putStrLn ("is warm? " ++ show (isWarm myColor))

    putStrLn "(ADTs done)"


    putStrLn "\n=== 8. Pattern Matching ==="

    -- ========================================================================
    -- 8. PATTERN MATCHING
    -- Destructure values directly in function arguments or case expressions.
    -- ========================================================================

    -- a) use describeList on different lists
    -- putStrLn (describeList ([] :: [Int]))
    -- putStrLn (describeList [1])
    -- putStrLn (describeList [1, 2])
    -- putStrLn (describeList [1, 2, 3])

    -- b) use a case expression on a Color
    -- let colorName = case Red of
    --       Red   -> "red"
    --       Green -> "green"
    --       Blue  -> "blue"
    -- putStrLn ("color: " ++ colorName)

    putStrLn "(pattern matching done)"

    putStrLn "\n=== done ==="


-- ============================================================================
-- Type definitions (must be at top level)
-- ============================================================================

data Shape = Circle Double | Rectangle Double Double
    deriving (Show)

area :: Shape -> Double
area (Circle r)      = pi * r * r
area (Rectangle w h) = w * h

data Color = Red | Green | Blue
    deriving (Show, Eq)

isWarm :: Color -> Bool
isWarm Red = True
isWarm _   = False

describeList :: [a] -> String
describeList []    = "empty"
describeList [_]   = "one element"
describeList [_,_] = "two elements"
describeList _     = "many elements"
