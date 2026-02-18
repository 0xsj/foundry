// Additional tests for the middleware pipeline.
//
// These tests are provided separately so the starter main.rs stays readable.
// To run all tests together, copy these into main.rs's #[cfg(test)] block,
// or compile: rustc --test main.rs && ./main
//
// These tests exercise edge cases and the full composed pipeline more thoroughly.

// NOTE: These tests assume you've implemented all types in main.rs.
// They duplicate the test module imports and should be merged into
// the main.rs test module before running.

#[cfg(test)]
mod extended_tests {
    // These would be merged into main.rs's test module.
    // Listed here for reference:

    // 1. test_echo_various_methods -- GET, POST, PUT, DELETE all echo correctly
    // 2. test_auth_multiple_valid_tokens -- accepts any token in the list
    // 3. test_rate_limit_different_methods_same_path -- POST /api and GET /api share limit
    // 4. test_metrics_only_counts_4xx_5xx -- 200, 201, 301 are not errors
    // 5. test_full_pipeline_health_bypass -- health endpoint skips auth and rate limiting

    // See the solutions for full implementations of these tests.
}
