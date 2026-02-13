module Main where

-- Run with: runhaskell ConfigTest.hs

import Config

-- Simple test runner (no dependencies needed)
assert :: String -> Bool -> IO ()
assert label True  = putStrLn ("  PASS: " ++ label)
assert label False = putStrLn ("  FAIL: " ++ label)

assertLeft :: String -> Either String a -> IO ()
assertLeft label (Left _)  = putStrLn ("  PASS: " ++ label)
assertLeft label (Right _) = putStrLn ("  FAIL: " ++ label ++ " (expected Left, got Right)")

assertRight :: Show a => String -> (a -> Bool) -> Either String a -> IO ()
assertRight label _ (Left e)    = putStrLn ("  FAIL: " ++ label ++ " (got Left: " ++ e ++ ")")
assertRight label check (Right a)
  | check a   = putStrLn ("  PASS: " ++ label)
  | otherwise  = putStrLn ("  FAIL: " ++ label ++ " (got Right: " ++ show a ++ ")")

main :: IO ()
main = do
  putStrLn "=== Config Parser Tests ===\n"

  -- Test 1: Full environment
  putStrLn "Test: Full environment"
  let fullEnv = [ ("HOST", "api.example.com")
                , ("PORT", "9090")
                , ("DEBUG", "true")
                , ("MAX_RETRIES", "5")
                , ("TIMEOUT_SEC", "30.5")
                , ("ENVIRONMENT", "production")
                ]
  assertRight "parses successfully" (const True) (parseConfig fullEnv)
  assertRight "host is correct" (\c -> cfgHost c == "api.example.com") (parseConfig fullEnv)
  assertRight "port is 9090" (\c -> cfgPort c == Port 9090) (parseConfig fullEnv)
  assertRight "debug is True" (\c -> cfgDebug c == True) (parseConfig fullEnv)
  assertRight "maxRetries is Just 5" (\c -> cfgMaxRetries c == Just 5) (parseConfig fullEnv)
  assertRight "timeout is 30.5" (\c -> cfgTimeout c == TimeoutSec 30.5) (parseConfig fullEnv)
  assertRight "env is production" (\c -> cfgEnv c == "production") (parseConfig fullEnv)

  -- Test 2: Defaults applied
  putStrLn "\nTest: Defaults applied (only HOST)"
  let minEnv = [("HOST", "localhost")]
  assertRight "parses successfully" (const True) (parseConfig minEnv)
  assertRight "host is localhost" (\c -> cfgHost c == "localhost") (parseConfig minEnv)
  assertRight "port defaults to 8080" (\c -> cfgPort c == Port 8080) (parseConfig minEnv)
  assertRight "debug defaults to False" (\c -> cfgDebug c == False) (parseConfig minEnv)
  assertRight "maxRetries is Nothing" (\c -> cfgMaxRetries c == Nothing) (parseConfig minEnv)
  assertRight "timeout defaults to 30.0" (\c -> cfgTimeout c == TimeoutSec 30.0) (parseConfig minEnv)
  assertRight "env defaults to development" (\c -> cfgEnv c == "development") (parseConfig minEnv)

  -- Test 3: Missing HOST
  putStrLn "\nTest: Missing HOST"
  assertLeft "returns Left" (parseConfig [("PORT", "8080")])

  -- Test 4: Empty HOST
  putStrLn "\nTest: Empty HOST"
  assertLeft "returns Left" (parseConfig [("HOST", "")])

  -- Test 5: Invalid PORT
  putStrLn "\nTest: Invalid PORT"
  assertLeft "returns Left" (parseConfig [("HOST", "localhost"), ("PORT", "abc")])

  -- Test 6: MAX_RETRIES = 0 (intentionally zero, not missing)
  putStrLn "\nTest: MAX_RETRIES explicitly set to 0"
  let zeroRetries = [("HOST", "localhost"), ("MAX_RETRIES", "0")]
  assertRight "maxRetries is Just 0, not Nothing"
    (\c -> cfgMaxRetries c == Just 0) (parseConfig zeroRetries)

  -- Test 7: Debug variants
  putStrLn "\nTest: DEBUG variants"
  let mkEnv d = [("HOST", "localhost"), ("DEBUG", d)]
  assertRight "\"true\" -> True" (\c -> cfgDebug c == True) (parseConfig (mkEnv "true"))
  assertRight "\"false\" -> False" (\c -> cfgDebug c == False) (parseConfig (mkEnv "false"))
  assertRight "\"1\" -> True" (\c -> cfgDebug c == True) (parseConfig (mkEnv "1"))
  assertRight "\"0\" -> False" (\c -> cfgDebug c == False) (parseConfig (mkEnv "0"))

  -- Test 8: Invalid TIMEOUT_SEC
  putStrLn "\nTest: Invalid TIMEOUT_SEC"
  assertLeft "returns Left" (parseConfig [("HOST", "localhost"), ("TIMEOUT_SEC", "abc")])

  -- Test 9: showConfig doesn't crash
  putStrLn "\nTest: showConfig"
  case parseConfig minEnv of
    Left e  -> putStrLn ("  FAIL: parse failed: " ++ e)
    Right c -> do
      let s = showConfig c
      assert "returns non-empty string" (not (null s))
      putStrLn ("  Output: " ++ s)

  putStrLn "\n=== Done ==="
