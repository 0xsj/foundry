# Debugging: I/O & Files

There are **4 bugs** in `buggy.go`. Find them all.

## Scenario

A service writes daily report files and reads them back for analysis. The code below was written by a junior engineer who understood Go syntax but didn't fully internalize Go's I/O contracts. The tests in `buggy_test.go` expose each bug — some obviously, some subtly.

## Symptoms (not causes — find those yourself)

1. **Bug 1 — `writeReport`**: The test `TestWriteReport_DataReachesFile` fails: the file is created but is always empty (0 bytes), even though the function returns no error.

2. **Bug 2 — `processLogLines`**: The test `TestProcessLogLines_ReturnsErrorOnIOFailure` fails: the function returns `nil` when the underlying reader errors mid-stream. The function does report correct line counts for successful reads.

3. **Bug 3 — `buildReportPath`**: The test `TestBuildReportPath_WindowsSafe` fails: the path contains a hardcoded `/` separator that's wrong on Windows. This is a platform-portability bug.

4. **Bug 4 — `processAll`**: The test `TestProcessAll_ClosesFilesPromptly` fails: after processing 5 files, all 5 file descriptors are still open, causing resource exhaustion under load. The function appears to work correctly in terms of output.

## Files

- `buggy.go` — contains 4 bugs, one in each function
- `buggy_test.go` — failing tests that expose each bug
- `solution.md` — explanations, fixes, and prevention advice

## Notes

- Each bug is in a separate function — they don't interact
- The bugs map directly to concepts from the I/O & Files lesson
- Read the symptom, form a hypothesis, find the root cause
