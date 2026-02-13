module Buggy where

-- | A student's grade record.
type StudentName = String
type Score = Int  -- 0-100
type GradeRecord = (StudentName, [Score])

-- | Calculate the average score for a student.
-- Should return Nothing if the student has no grades.
average :: GradeRecord -> Maybe Double
average (_, grades) =
  let total = sum grades
      count = length grades
  in Just (fromIntegral total / fromIntegral count)

-- | Convert a numeric score (0-100) to a letter grade.
letterGrade :: Score -> String
letterGrade score
  | score >= 60 = "D"
  | score >= 70 = "C"
  | score >= 80 = "B"
  | score >= 90 = "A"
  | score < 60  = "F"
  | otherwise   = "?"

-- | Calculate GPA on a 4.0 scale from a list of scores.
-- A=4, B=3, C=2, D=1, F=0
gpa :: [Score] -> Double
gpa [] = 0.0
gpa scores =
  let points = map scoreToPoints scores
      total = sum points
      count = length scores
  in fromIntegral total `div` fromIntegral count

-- | Convert a score to GPA points.
scoreToPoints :: Score -> Int
scoreToPoints s
  | s >= 90   = 4
  | s >= 80   = 3
  | s >= 70   = 2
  | s >= 60   = 1
  | otherwise = 0

-- | Get the highest score from a list.
-- Uses head after sorting — seems reasonable?
topScore :: [Score] -> Score
topScore scores = head (reverse (quickSort scores))

-- Simple quicksort for demonstration
quickSort :: Ord a => [a] -> [a]
quickSort [] = []
quickSort (x:xs) = quickSort smaller ++ [x] ++ quickSort bigger
  where
    smaller = filter (<= x) xs
    bigger  = filter (> x) xs

-- | Summary for a student.
summary :: GradeRecord -> String
summary record@(name, scores) =
  name ++ ": avg=" ++ showAvg (average record)
       ++ ", top=" ++ show (topScore scores)
       ++ ", gpa=" ++ show (gpa scores)
  where
    showAvg Nothing  = "N/A"
    showAvg (Just a) = show (round a :: Int)
