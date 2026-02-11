-- ============================================================================
-- Types Deep Dive — Haskell
-- ============================================================================
-- Exploring the type system. Run with: runghc types.hs
-- Or better: load in GHCi and use :type to inspect types interactively
--   ghci types.hs
--   :type someFunction
-- ============================================================================

module Main where

main :: IO ()
main = do
    putStrLn "=== 1. Type Inference ==="

    -- ========================================================================
    -- 1. TYPE INFERENCE
    -- Haskell infers the most general type possible.
    -- Load this file in GHCi and use :type to check.
    -- ========================================================================

    -- In GHCi, try:
    --   :type 42
    --   :type 42 :: Int
    --   :type 3.14
    --   :type "hello"
    --   :type 'a'
    --   :type True
    --   :type (+)
    --   :type head

    -- Q: What type does GHCi give for 42? Why isn't it just Int?
    --    (answer here)

    -- Q: What does Num a => mean in a type signature?
    --    (answer here)

    putStrLn "(use GHCi :type for this section)"


    putStrLn "\n=== 2. Int vs Integer ==="

    -- ========================================================================
    -- 2. Int vs Integer
    -- Int is fixed precision. Integer is arbitrary precision.
    -- ========================================================================

    let smallInt = 42 :: Int
    let bigInteger = 2 ^ 100 :: Integer
    let overflow = maxBound :: Int

    putStrLn ("Int:     " ++ show smallInt)
    putStrLn ("Integer: " ++ show bigInteger)
    putStrLn ("maxBound Int: " ++ show overflow)

    -- Q: What is maxBound for Int on your machine?
    --    Is it the same as Go's math.MaxInt64?
    --    (answer here)

    -- Q: What happens if you do (maxBound :: Int) + 1?
    --    How does this compare to Zig's overflow behavior?
    --    (try it in GHCi)


    putStrLn "\n=== 3. Type Aliases vs Newtypes ==="

    -- ========================================================================
    -- 3. TYPE ALIASES vs NEWTYPES
    -- type = just a new name (no safety)
    -- newtype = new distinct type (compile-time safety, zero runtime cost)
    -- ========================================================================

    let userName = UserName "alice"
    let userId = UserId 42

    putStrLn ("user: " ++ getUserName userName)
    putStrLn ("id:   " ++ show (getUserId userId))

    -- Q: Can you pass a plain String where UserName is expected? Try it.
    --    How does this compare to Go's type aliases?
    --    (answer here)

    -- Q: Can you pass a plain Int where UserId is expected?
    --    (answer here)


    putStrLn "\n=== 4. Polymorphism ==="

    -- ========================================================================
    -- 4. POLYMORPHISM
    -- Functions can work on any type that satisfies constraints.
    -- ========================================================================

    -- identity works on anything
    putStrLn ("identity 42: " ++ show (identity (42 :: Int)))
    putStrLn ("identity True: " ++ show (identity True))
    putStrLn ("identity \"hi\": " ++ identity "hi")

    -- double works on any Num
    putStrLn ("double 21: " ++ show (double (21 :: Int)))
    putStrLn ("double 1.5: " ++ show (double (1.5 :: Double)))

    -- Q: In GHCi, check :type identity and :type double
    --    What do the type variables (a, Num a =>) mean?
    --    How does this compare to generics in Go/TS/Zig?
    --    (answer here)


    putStrLn "\n=== 5. Records ==="

    -- ========================================================================
    -- 5. RECORDS
    -- Record syntax gives you accessor functions for free.
    -- ========================================================================

    let config = Config
            { configPort = 8080
            , configHost = "localhost"
            , configDebug = False
            }

    putStrLn ("port: " ++ show (configPort config))
    putStrLn ("host: " ++ configHost config)

    -- Record update syntax
    let devConfig = config { configDebug = True, configPort = 3000 }
    putStrLn ("dev port: " ++ show (configPort devConfig))
    putStrLn ("dev debug: " ++ show (configDebug devConfig))

    -- Q: How does record update compare to spread in JS ({ ...config, debug: true })?
    --    Key difference: the original config is NOT modified.
    --    (answer here)


    putStrLn "\n=== 6. Sum Types and Exhaustiveness ==="

    -- ========================================================================
    -- 6. SUM TYPES
    -- Pattern matching on sum types must be exhaustive.
    -- ========================================================================

    putStrLn (describeResult (Success 42))
    putStrLn (describeResult (Failure "timeout"))
    putStrLn (describeResult Loading)

    -- Q: Try removing one of the cases in describeResult.
    --    What warning/error do you get?
    --    How does this compare to Zig's exhaustive switch?
    --    (answer here)


    putStrLn "\n=== done ==="


-- ============================================================================
-- Type definitions
-- ============================================================================

-- Newtype wrappers
newtype UserName = UserName String
newtype UserId = UserId Int

getUserName :: UserName -> String
getUserName (UserName s) = s

getUserId :: UserId -> Int
getUserId (UserId n) = n

-- Polymorphic functions
identity :: a -> a
identity x = x

double :: Num a => a -> a
double x = x + x

-- Record
data Config = Config
    { configPort  :: Int
    , configHost  :: String
    , configDebug :: Bool
    } deriving (Show)

-- Sum type with payloads
data Result
    = Success Int
    | Failure String
    | Loading
    deriving (Show)

describeResult :: Result -> String
describeResult (Success n) = "success: " ++ show n
describeResult (Failure e) = "failure: " ++ e
describeResult Loading     = "loading..."
