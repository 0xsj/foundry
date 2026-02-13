module Main where

-- Run with: runhaskell -i. BuggyTest.hs

import Buggy

assert :: String -> Bool -> IO ()
assert label True  = putStrLn ("  PASS: " ++ label)
assert label False = putStrLn ("  FAIL: " ++ label)

main :: IO ()
main = do
  putStrLn "=== Grade Tracker Tests ===\n"

  -- BUG 1: average of empty grades should be Nothing, not crash
  putStrLn "Test: Average of empty grades"
  let emptyStudent = ("Alice", []) :: GradeRecord
  case average emptyStudent of
    Nothing -> putStrLn "  PASS: returns Nothing for no grades"
    Just _  -> putStrLn "  FAIL: should return Nothing for empty grades"

  putStrLn "\nTest: Average of normal grades"
  let bob = ("Bob", [80, 90, 70]) :: GradeRecord
  case average bob of
    Nothing -> putStrLn "  FAIL: should return Just for non-empty grades"
    Just avg -> assert ("average is 80.0, got " ++ show avg) (abs (avg - 80.0) < 0.01)

  -- BUG 2: Letter grades should map correctly
  putStrLn "\nTest: Letter grades"
  assert ("95 -> A, got " ++ letterGrade 95) (letterGrade 95 == "A")
  assert ("90 -> A, got " ++ letterGrade 90) (letterGrade 90 == "A")
  assert ("85 -> B, got " ++ letterGrade 85) (letterGrade 85 == "B")
  assert ("80 -> B, got " ++ letterGrade 80) (letterGrade 80 == "B")
  assert ("75 -> C, got " ++ letterGrade 75) (letterGrade 75 == "C")
  assert ("70 -> C, got " ++ letterGrade 70) (letterGrade 70 == "C")
  assert ("65 -> D, got " ++ letterGrade 65) (letterGrade 65 == "D")
  assert ("55 -> F, got " ++ letterGrade 55) (letterGrade 55 == "F")

  -- BUG 3: GPA should be a proper floating-point average
  putStrLn "\nTest: GPA calculation"
  -- [90, 80, 70, 60] -> points [4, 3, 2, 1] -> avg 2.5
  let gpaResult = gpa [90, 80, 70, 60]
  assert ("GPA of [90,80,70,60] is 2.5, got " ++ show gpaResult)
    (abs (gpaResult - 2.5) < 0.01)

  -- [95, 95, 95] -> points [4, 4, 4] -> avg 4.0
  let perfectGpa = gpa [95, 95, 95]
  assert ("GPA of [95,95,95] is 4.0, got " ++ show perfectGpa)
    (abs (perfectGpa - 4.0) < 0.01)

  -- Empty should be 0.0
  assert ("GPA of [] is 0.0") (gpa [] == 0.0)

  -- Top score
  putStrLn "\nTest: Top score"
  assert "top of [80, 95, 70] is 95" (topScore [80, 95, 70] == 95)
  assert "top of [100] is 100" (topScore [100] == 100)

  putStrLn "\n=== Done ==="
