// Metrics API Response Types — Proposed Code for Review
//
// This module defines the typed response structs for the metrics API.
// Previously all responses used serde_json::Value — this PR replaces
// them with typed structs.
//
// The API contract (expected JSON):
//   GET /v1/metrics/services/{service_id}
//   → { "serviceId": "auth", "displayName": "Auth Service",
//       "createdAt": 1708300000, "updatedAt": 1708300001,
//       "owner": { "userId": "u1", "teamName": "Platform" },
//       "stats": { "request_count": 1000, "error_rate": 0.02, "p99_ms": 450 },
//       "tags": ["production", "critical"],
//       "alert_threshold": null,
//       "environment": "PRODUCTION" }

use serde::{Deserialize, Serialize};

// ISSUE: The entire struct uses snake_case field names, but the API uses camelCase.
// There's no #[serde(rename_all = "camelCase")] on the struct, and only some
// fields have manual renames. This means fields without renames will serialize
// with wrong names (e.g., "service_name" instead of "serviceName").
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMetrics {
    // ISSUE: field named "service_name" but JSON key is "serviceId"
    // Rename is applied correctly here, but the underlying field name is misleading:
    // it's called "service_name" in Rust but it holds an ID, not a name.
    #[serde(rename = "serviceId")]
    pub service_name: String,

    // ISSUE: No rename — this will serialize as "display_name" but API expects "displayName"
    pub display_name: String,

    // These timestamps belong together — they're begging for a Timestamps struct with flatten.
    // ISSUE: No rename — "created_at" instead of "createdAt"
    pub created_at: u64,
    // ISSUE: No rename — "updated_at" instead of "updatedAt"
    pub updated_at: u64,

    // ISSUE: These are all pub — owner, stats, and tags can be mutated directly.
    // This breaks any invariants the struct tries to maintain.
    pub owner: OwnerInfo,
    pub stats: ServiceStats,
    pub tags: Vec<String>,

    // This is correct — None serializes as null (or is skipped with skip_serializing_if)
    // But there's no skip_serializing_if here, so it will produce "alert_threshold": null
    // instead of omitting the field when None. Depends on whether the frontend expects null
    // or absence — document this decision.
    pub alert_threshold: Option<f64>,

    // ISSUE: This is a stringly-typed field. "PRODUCTION", "STAGING", "DEVELOPMENT"
    // are a fixed set of values — they should be an enum, not a String.
    // A typo like "PRODUCTON" would only be caught at runtime (or not at all).
    pub environment: String,
}

// ISSUE: OwnerInfo has no rename_all — "user_id" serializes as "user_id" but API expects "userId"
// and "team_name" should be "teamName"
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnerInfo {
    pub user_id: String,
    pub team_name: String,
}

// ServiceStats has correct field names (they match snake_case in both JSON and Rust)
// because the API happens to use snake_case for this nested object. No issue here.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStats {
    pub request_count: u64,
    pub error_rate: f64,
    pub p99_ms: u64,
}

// ISSUE: This struct has the timestamps as separate fields.
// They appear in many response types. With #[serde(flatten)], a Timestamps struct
// could be reused across ServiceMetrics, UserMetrics, etc.
// As-is, the created_at/updated_at fields are duplicated everywhere they're needed.
#[derive(Debug, Serialize, Deserialize)]
pub struct Timestamps {
    pub created_at: u64,
    pub updated_at: u64,
}

// Correct approach (not used in ServiceMetrics):
// #[serde(flatten)]
// timestamps: Timestamps,

// The environment field should be this enum instead of String.
// It's defined here but NOT used in ServiceMetrics — the PR forgot to update the field type.
#[derive(Debug, Serialize, Deserialize)]
pub enum Environment {
    Production,
    Staging,
    Development,
}

// A response envelope used for all list endpoints
// ISSUE: data field type is too permissive — Vec<serde_json::Value> loses all type information.
// Should be generic: Vec<T> where T: Serialize + Deserialize.
// But the PR author hardcoded Value to avoid dealing with generics.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub data: Vec<serde_json::Value>, // ISSUE: loses type information
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_metrics_round_trip() {
        let json = r#"{
            "serviceId": "auth-service",
            "display_name": "Auth Service",
            "created_at": 1708300000,
            "updated_at": 1708300001,
            "owner": {"user_id": "u1", "team_name": "Platform"},
            "stats": {"request_count": 1000, "error_rate": 0.02, "p99_ms": 450},
            "tags": ["production", "critical"],
            "alert_threshold": null,
            "environment": "PRODUCTION"
        }"#;

        // This test uses snake_case keys for the wrong fields — it only works because
        // the struct fields lack rename attributes.
        // After fixing rename_all, this test would need to be updated to use camelCase.
        let metrics: ServiceMetrics = serde_json::from_str(json).unwrap();
        assert_eq!(metrics.service_name, "auth-service");
        assert_eq!(metrics.display_name, "Auth Service");
    }

    #[test]
    fn test_environment_is_string() {
        // This test confirms the current (wrong) behavior:
        // environment is a String, so any value is accepted.
        let json = r#"{
            "serviceId": "x",
            "display_name": "X",
            "created_at": 0,
            "updated_at": 0,
            "owner": {"user_id": "u", "team_name": "t"},
            "stats": {"request_count": 0, "error_rate": 0.0, "p99_ms": 0},
            "tags": [],
            "alert_threshold": null,
            "environment": "PRODUCTON"
        }"#;
        // Typo "PRODUCTON" is silently accepted because the field is String
        let metrics: ServiceMetrics = serde_json::from_str(json).unwrap();
        assert_eq!(metrics.environment, "PRODUCTON"); // silently wrong
    }

    #[test]
    fn test_list_response() {
        let json = r#"{"total": 100, "page": 1, "per_page": 20, "data": []}"#;
        let resp: ListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total, 100);
        // data is Vec<Value> — no type information about what's inside
    }
}

fn main() {
    println!("Metrics API — proposed types for review");
}
