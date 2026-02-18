package serializer

import (
	"encoding/json"
	"strings"
	"testing"
	"time"
)

var (
	epoch     = time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	deletedAt = time.Date(2026, 3, 1, 0, 0, 0, 0, time.UTC)
	baseURL   = "https://api.example.com"

	activeProduct = Product{
		ID:         "prod_001",
		Name:       "Widget Pro",
		PriceCents: 4999,
		Stock:      100,
		Tags:       []string{"hardware", "featured"},
		CreatedAt:  epoch,
		DeletedAt:  nil,
	}

	deletedProduct = Product{
		ID:         "prod_002",
		Name:       "Legacy Part",
		PriceCents: 999,
		Stock:      0,
		Tags:       []string{"legacy"},
		CreatedAt:  epoch,
		DeletedAt:  &deletedAt,
	}

	noTagProduct = Product{
		ID:         "prod_003",
		Name:       "Plain Widget",
		PriceCents: 100,
		Stock:      5,
		Tags:       nil,
		CreatedAt:  epoch,
		DeletedAt:  nil,
	}
)

func TestUnixTimeMarshal(t *testing.T) {
	u := UnixTime{epoch}
	data, err := json.Marshal(u)
	if err != nil {
		t.Fatalf("MarshalJSON error: %v", err)
	}
	if string(data)[0] == '"' {
		t.Errorf("UnixTime must marshal as integer, got: %s", data)
	}
	if string(data) != "1771408800" {
		t.Errorf("UnixTime marshal = %s, want 1771408800", data)
	}
}

func TestUnixTimeUnmarshal(t *testing.T) {
	var u UnixTime
	if err := json.Unmarshal([]byte("1771408800"), &u); err != nil {
		t.Fatalf("UnmarshalJSON error: %v", err)
	}
	if !u.Equal(epoch) {
		t.Errorf("UnixTime unmarshal = %v, want %v", u.Time, epoch)
	}
}

func TestUnixTimeRoundTrip(t *testing.T) {
	original := UnixTime{epoch}
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var decoded UnixTime
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if !decoded.Equal(epoch) {
		t.Errorf("round-trip: got %v, want %v", decoded.Time, epoch)
	}
}

func TestV1ResponseShape(t *testing.T) {
	resp := NewV1Response(activeProduct)
	data, err := json.Marshal(resp)
	if err != nil {
		t.Fatalf("marshal V1: %v", err)
	}

	var raw map[string]any
	json.Unmarshal(data, &raw)

	checkStringField(t, raw, "id", "prod_001")
	checkStringField(t, raw, "name", "Widget Pro")
	checkNumberField(t, raw, "price_cents", 4999)
	checkNumberField(t, raw, "stock", 100)

	if val, ok := raw["created_at"]; ok {
		if _, isNum := val.(float64); !isNum {
			t.Errorf("V1 created_at must be a number (Unix), got %T: %v", val, val)
		}
	} else {
		t.Error("V1 response missing created_at field")
	}

	if _, ok := raw["deleted_at"]; ok {
		t.Error("V1 response must not contain deleted_at field")
	}
}

func TestV1ResponseTags(t *testing.T) {
	resp := NewV1Response(activeProduct)
	data, _ := json.Marshal(resp)
	var raw map[string]any
	json.Unmarshal(data, &raw)

	tags, ok := raw["tags"].([]any)
	if !ok {
		t.Fatalf("V1 tags should be a JSON array, got %T", raw["tags"])
	}
	if len(tags) != 2 || tags[0] != "hardware" || tags[1] != "featured" {
		t.Errorf("V1 tags = %v, want [hardware featured]", tags)
	}
}

func TestV2ResponseShape(t *testing.T) {
	resp := NewV2Response(activeProduct, baseURL)
	data, err := json.Marshal(resp)
	if err != nil {
		t.Fatalf("marshal V2: %v", err)
	}

	var raw map[string]any
	json.Unmarshal(data, &raw)

	checkStringField(t, raw, "id", "prod_001")
	checkStringField(t, raw, "name", "Widget Pro")
	checkNumberField(t, raw, "price_cents", 4999)
	checkNumberField(t, raw, "stock", 100)

	if val, ok := raw["created_at"]; ok {
		if _, isString := val.(string); !isString {
			t.Errorf("V2 created_at must be a string (RFC3339), got %T: %v", val, val)
		}
	} else {
		t.Error("V2 response missing created_at field")
	}

	checkStringField(t, raw, "api_version", "v2")

	if _, ok := raw["links"]; !ok {
		t.Error("V2 response missing links field")
	}

	if _, ok := raw["deleted_at"]; ok {
		t.Error("V2 deleted_at should be omitted when nil")
	}
}

func TestV2ResponseLinks(t *testing.T) {
	resp := NewV2Response(activeProduct, baseURL)
	data, _ := json.Marshal(resp)
	var raw map[string]any
	json.Unmarshal(data, &raw)

	links, ok := raw["links"].(map[string]any)
	if !ok {
		t.Fatalf("links should be a JSON object, got %T", raw["links"])
	}

	wantSelf := baseURL + "/products/prod_001"
	wantCollection := baseURL + "/products"

	if links["self"] != wantSelf {
		t.Errorf("links.self = %q, want %q", links["self"], wantSelf)
	}
	if links["collection"] != wantCollection {
		t.Errorf("links.collection = %q, want %q", links["collection"], wantCollection)
	}
}

func TestV2ResponseDeletedAt(t *testing.T) {
	resp := NewV2Response(deletedProduct, baseURL)
	data, _ := json.Marshal(resp)
	var raw map[string]any
	json.Unmarshal(data, &raw)

	if _, ok := raw["deleted_at"]; !ok {
		t.Error("V2 deleted_at should be present when non-nil")
	}
	if val, ok := raw["deleted_at"].(string); ok {
		if !strings.Contains(val, "2026-03-01") {
			t.Errorf("V2 deleted_at = %q, expected to contain 2026-03-01", val)
		}
	} else {
		t.Errorf("V2 deleted_at should be a string, got %T", raw["deleted_at"])
	}
}

func TestV2MetaIsEmbedded(t *testing.T) {
	resp := NewV2Response(activeProduct, baseURL)
	data, _ := json.Marshal(resp)
	var raw map[string]any
	json.Unmarshal(data, &raw)

	if _, ok := raw["meta"]; ok {
		t.Error("V2 must not have a top-level 'meta' key — Meta must be embedded")
	}
}

func TestSerializeJSONV1(t *testing.T) {
	s := NewSerializer(baseURL)
	data, err := s.SerializeJSON([]Product{activeProduct, deletedProduct}, "v1")
	if err != nil {
		t.Fatalf("SerializeJSON v1 error: %v", err)
	}

	var arr []map[string]any
	if err := json.Unmarshal(data, &arr); err != nil {
		t.Fatalf("result is not valid JSON array: %v\ndata: %s", err, data)
	}
	if len(arr) != 2 {
		t.Fatalf("expected 2 items, got %d", len(arr))
	}
	if !strings.Contains(string(data), "\n") {
		t.Error("SerializeJSON should return pretty-printed JSON")
	}
	for _, item := range arr {
		if _, ok := item["deleted_at"]; ok {
			t.Error("V1 JSON must not contain deleted_at")
		}
	}
}

func TestSerializeJSONV2(t *testing.T) {
	s := NewSerializer(baseURL)
	data, err := s.SerializeJSON([]Product{activeProduct, deletedProduct}, "v2")
	if err != nil {
		t.Fatalf("SerializeJSON v2 error: %v", err)
	}

	var arr []map[string]any
	if err := json.Unmarshal(data, &arr); err != nil {
		t.Fatalf("result is not valid JSON array: %v", err)
	}
	if len(arr) != 2 {
		t.Fatalf("expected 2 items, got %d", len(arr))
	}
	if arr[0]["api_version"] != "v2" {
		t.Errorf("item[0] api_version = %v, want v2", arr[0]["api_version"])
	}
	if _, ok := arr[0]["deleted_at"]; ok {
		t.Error("active product V2 must not have deleted_at")
	}
	if _, ok := arr[1]["deleted_at"]; !ok {
		t.Error("deleted product V2 must have deleted_at")
	}
}

func TestSerializeJSONUnknownVersion(t *testing.T) {
	s := NewSerializer(baseURL)
	_, err := s.SerializeJSON([]Product{activeProduct}, "v3")
	if err == nil {
		t.Error("SerializeJSON with unknown version should return an error")
	}
}

func TestSerializeCSV(t *testing.T) {
	s := NewSerializer(baseURL)
	data, err := s.SerializeCSV([]Product{activeProduct, deletedProduct, noTagProduct})
	if err != nil {
		t.Fatalf("SerializeCSV error: %v", err)
	}

	lines := strings.Split(strings.TrimRight(string(data), "\n"), "\n")
	if len(lines) != 4 {
		t.Fatalf("expected 4 lines (1 header + 3 data), got %d:\n%s", len(lines), string(data))
	}

	expectedHeader := "id,name,price_cents,stock,tags,created_at,deleted_at"
	if lines[0] != expectedHeader {
		t.Errorf("header = %q, want %q", lines[0], expectedHeader)
	}

	active := lines[1]
	if !strings.Contains(active, "prod_001") {
		t.Errorf("row 1 missing prod_001: %q", active)
	}
	if !strings.Contains(active, "hardware;featured") {
		t.Errorf("row 1 tags should be semicolon-joined: %q", active)
	}
	activeCols := strings.Split(active, ",")
	if len(activeCols) < 7 {
		t.Fatalf("row 1 should have 7 columns, got %d", len(activeCols))
	}
	if activeCols[6] != "" {
		t.Errorf("row 1 deleted_at should be empty, got %q", activeCols[6])
	}

	deleted := lines[2]
	if !strings.Contains(deleted, "2026-03-01") {
		t.Errorf("row 2 deleted_at should contain 2026-03-01: %q", deleted)
	}

	noTag := lines[3]
	cols := strings.Split(noTag, ",")
	if len(cols) < 5 {
		t.Fatalf("row 3 should have at least 5 columns, got %d", len(cols))
	}
	if cols[4] != "" {
		t.Errorf("row 3 tags should be empty string, got %q", cols[4])
	}
}

func TestSerializeCSVHeader(t *testing.T) {
	s := NewSerializer(baseURL)
	data, err := s.SerializeCSV([]Product{})
	if err != nil {
		t.Fatalf("SerializeCSV empty error: %v", err)
	}
	lines := strings.Split(strings.TrimRight(string(data), "\n"), "\n")
	if len(lines) < 1 || lines[0] != "id,name,price_cents,stock,tags,created_at,deleted_at" {
		t.Errorf("CSV must always include header row, got: %q", string(data))
	}
}

func checkStringField(t *testing.T, m map[string]any, key, want string) {
	t.Helper()
	val, ok := m[key]
	if !ok {
		t.Errorf("missing field %q", key)
		return
	}
	if s, ok := val.(string); !ok || s != want {
		t.Errorf("field %q = %v (%T), want %q", key, val, val, want)
	}
}

func checkNumberField(t *testing.T, m map[string]any, key string, want float64) {
	t.Helper()
	val, ok := m[key]
	if !ok {
		t.Errorf("missing field %q", key)
		return
	}
	n, ok := val.(float64)
	if !ok {
		t.Errorf("field %q = %v (%T), want float64", key, val, val)
		return
	}
	if n != want {
		t.Errorf("field %q = %v, want %v", key, n, want)
	}
}
