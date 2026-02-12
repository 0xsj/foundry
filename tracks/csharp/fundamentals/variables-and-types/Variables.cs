// Variables.cs — C# Variables & Types (C# 12 / .NET 8, top-level statements)
// Run with: dotnet run Variables.cs
// Or place in a .csproj and run with: dotnet run

#nullable enable

using System;
using System.Net;

// =============================================================================
// 1. VAR KEYWORD AND TYPE INFERENCE
// =============================================================================
// `var` is compile-time type inference — the compiler resolves the actual type.
// Unlike TypeScript's `let` or Go's `:=`, the inferred type is fixed at compile
// time. It's syntactic sugar, not dynamic typing.

var requestTimeout = 30;                   // int (System.Int32)
var endpoint = "https://api.internal/v2";  // string
var retryEnabled = true;                   // bool
var throughput = 1_500_000L;               // long (System.Int64) — numeric separator for readability

Console.WriteLine("--- Type Inference ---");
Console.WriteLine($"requestTimeout: {requestTimeout} ({requestTimeout.GetType()})");
Console.WriteLine($"endpoint: {endpoint} ({endpoint.GetType()})");
Console.WriteLine($"throughput: {throughput} ({throughput.GetType()})");
Console.WriteLine();

// =============================================================================
// 2. NUMERIC TYPES — all with explicit sizes
// =============================================================================
// C# numeric types have guaranteed sizes (unlike C/C++ where int size varies).
// This is similar to Rust's i32/i64/f64 — every type has a known width.

byte  statusCode       = 200;             //  8-bit unsigned  [0, 255]
short portNumber        = 8080;            // 16-bit signed    [-32768, 32767]
int   maxConnections    = 10_000;          // 32-bit signed    [-2^31, 2^31-1]
long  totalRequests     = 9_876_543_210L;  // 64-bit signed    [-2^63, 2^63-1]
float cpuLoadPercent    = 73.4f;           // 32-bit IEEE 754  ~6-7 sig digits
double p99Latency       = 12.345678901;    // 64-bit IEEE 754  ~15-16 sig digits
decimal invoiceAmount   = 1_249.99m;       // 128-bit decimal  28-29 sig digits — use for money

Console.WriteLine("--- Numeric Types ---");
Console.WriteLine($"byte   statusCode:      {statusCode,-15} size: {sizeof(byte)} byte");
Console.WriteLine($"short  portNumber:       {portNumber,-15} size: {sizeof(short)} bytes");
Console.WriteLine($"int    maxConnections:   {maxConnections,-15} size: {sizeof(int)} bytes");
Console.WriteLine($"long   totalRequests:    {totalRequests,-15} size: {sizeof(long)} bytes");
Console.WriteLine($"float  cpuLoadPercent:   {cpuLoadPercent,-15} size: {sizeof(float)} bytes");
Console.WriteLine($"double p99Latency:       {p99Latency,-15} size: {sizeof(double)} bytes");
Console.WriteLine($"decimal invoiceAmount:   {invoiceAmount,-15} size: {sizeof(decimal)} bytes");

// Decimal vs double: financial precision
double  priceDouble  = 0.1 + 0.2;
decimal priceDecimal = 0.1m + 0.2m;
Console.WriteLine($"\n0.1 + 0.2 as double:  {priceDouble}");    // 0.30000000000000004
Console.WriteLine($"0.1 + 0.2 as decimal: {priceDecimal}");     // 0.3
Console.WriteLine();

// =============================================================================
// 3. STRING TYPES
// =============================================================================
// C# strings are immutable reference types (like Go and Java).
// Three special string literal forms beyond regular "...":

// Verbatim strings: @"..." — no escape processing, useful for paths and regex
var certPath = @"C:\certs\production\server.pfx";
var sqlQuery = @"
    SELECT id, email, created_at
    FROM users
    WHERE status = 'active'
    ORDER BY created_at DESC";

// Interpolated strings: $"..." — embed expressions (like JS template literals)
var region = "us-east-1";
var instanceCount = 42;
var statusLine = $"Region {region}: {instanceCount} instances running";

// Combined: $@"..." or @$"..." — interpolated + verbatim
var logPath = $@"C:\logs\{region}\app.log";

// Raw string literals (C# 11+): """...""" — no escaping needed at all.
// The number of quotes can increase to avoid conflicts.
// Indentation is trimmed based on the closing quotes position.
var jsonPayload = """
    {
        "webhook_url": "https://hooks.example.com/v1",
        "events": ["order.created", "order.updated"],
        "secret": "whsec_abc123"
    }
    """;

// Raw interpolated strings: $"""..."""
var webhookId = "wh_9x8k2m";
var configJson = $"""
    {{
        "id": "{webhookId}",
        "retry_policy": {{
            "max_attempts": 3,
            "backoff_ms": [100, 500, 2000]
        }}
    }}
    """;
// Note: {{ and }} are literal braces in interpolated raw strings,
// {expr} is interpolation. The number of $ signs controls how many
// braces trigger interpolation.

Console.WriteLine("--- String Types ---");
Console.WriteLine($"Verbatim path: {certPath}");
Console.WriteLine($"Interpolated:  {statusLine}");
Console.WriteLine($"Raw JSON:\n{jsonPayload}");
Console.WriteLine($"Raw interpolated:\n{configJson}");
Console.WriteLine();

// =============================================================================
// 4. BOOLEAN
// =============================================================================
// C# bools are strict — no truthy/falsy coercion like JS/Python.
// `if (count)` won't compile. You must write `if (count > 0)`.

bool isHealthy = true;
bool maintenanceMode = false;
bool canServeTraffic = isHealthy && !maintenanceMode;

Console.WriteLine("--- Boolean ---");
Console.WriteLine($"isHealthy: {isHealthy}, canServeTraffic: {canServeTraffic}");
Console.WriteLine();

// =============================================================================
// 5. NULLABLE VALUE TYPES
// =============================================================================
// Value types (int, double, bool, structs) cannot be null by default.
// Appending ? wraps them in Nullable<T>, which adds a HasValue flag.
// This is different from Go's pointer-based nil or Rust's Option<T>,
// though conceptually similar to Option/Maybe.

int? rateLimitRemaining = null;   // Nullable<int> — header might be absent
double? cpuThreshold = 85.0;

// Null-coalescing: ?? (like JS ??, Go's or-value pattern)
int effectiveLimit = rateLimitRemaining ?? 1000;

// Null-coalescing assignment: ??= (assign only if currently null)
rateLimitRemaining ??= 500;

Console.WriteLine("--- Nullable Value Types ---");
Console.WriteLine($"effectiveLimit (was null, fell back): {effectiveLimit}");
Console.WriteLine($"rateLimitRemaining (after ??= 500):   {rateLimitRemaining}");
Console.WriteLine($"cpuThreshold.HasValue: {cpuThreshold.HasValue}, Value: {cpuThreshold.Value}");
Console.WriteLine();

// =============================================================================
// 6. NULLABLE REFERENCE TYPES
// =============================================================================
// With #nullable enable (top of file), reference types are non-null by default.
// string  = non-nullable string (compiler warns on null assignment)
// string? = explicitly nullable string
// This is a compile-time annotation system — the CLR itself doesn't enforce it.
// Conceptually similar to TypeScript's strict null checks.

string tenantId = "tenant_abc123";    // Non-nullable — compiler enforces
string? overrideRegion = null;        // Explicitly nullable

// Null-conditional: ?. (like JS optional chaining)
int? regionLength = overrideRegion?.Length;

// Null-forgiving: ! (tells compiler "trust me, not null")
// Use sparingly — it suppresses warnings but doesn't prevent runtime NRE.
// string forcedValue = overrideRegion!;  // Dangerous if actually null

Console.WriteLine("--- Nullable Reference Types ---");
Console.WriteLine($"tenantId: {tenantId}");
Console.WriteLine($"overrideRegion: {overrideRegion ?? "(null)"}");
Console.WriteLine($"overrideRegion?.Length: {regionLength?.ToString() ?? "(null)"}");
Console.WriteLine();

// =============================================================================
// 7. TUPLES
// =============================================================================
// C# tuples are value types (System.ValueTuple) — allocated on the stack.
// Unlike Go's multiple return values, C# tuples are first-class types
// you can store, pass around, and destructure.

// Unnamed tuple (positional access via .Item1, .Item2)
var coordinate = (47.6062, -122.3321);
Console.WriteLine("--- Tuples ---");
Console.WriteLine($"Unnamed: lat={coordinate.Item1}, lng={coordinate.Item2}");

// Named tuple — much more readable (names exist only at compile time)
var healthCheck = (Status: "healthy", Latency: 12.5, CheckedAt: DateTime.UtcNow);
Console.WriteLine($"Named: {healthCheck.Status}, {healthCheck.Latency}ms");

// Tuple return from method-like local function
(bool Success, int StatusCode, string Body) SimulateRequest(string url)
{
    if (url.Contains("internal"))
        return (true, 200, """{"status": "ok"}""");
    return (false, 503, """{"error": "service unavailable"}""");
}

var response = SimulateRequest("https://api.internal/health");
Console.WriteLine($"Request: success={response.Success}, code={response.StatusCode}");

// Destructuring (like JS destructuring or Go's multi-return)
var (success, code, body) = SimulateRequest("https://external.api/status");
Console.WriteLine($"Destructured: success={success}, code={code}");
Console.WriteLine();

// =============================================================================
// 8. RECORDS
// =============================================================================
// Records provide value-based equality, immutability by default, and concise syntax.
// record class = reference type (heap) with value equality
// record struct = value type (stack) with value equality
// Similar purpose to Go's structs or Rust's #[derive(PartialEq)] structs,
// but with built-in support for immutability and `with` expressions.

// Record class — immutable by default, reference type
record WebhookEvent(string EventType, string Payload, DateTime Timestamp);

// Record struct — immutable, value type (stack-allocated when possible)
record struct RateLimitState(int Remaining, int Limit, DateTime ResetsAt);

// Records get value-based equality for free
var event1 = new WebhookEvent("order.created", "{}", DateTime.UnixEpoch);
var event2 = new WebhookEvent("order.created", "{}", DateTime.UnixEpoch);

Console.WriteLine("--- Records ---");
Console.WriteLine($"Record class equality: {event1 == event2}");  // true (value equality)
Console.WriteLine($"ReferenceEquals: {ReferenceEquals(event1, event2)}");  // false (different objects)

// `with` expression — non-destructive mutation (creates a copy)
var retryEvent = event1 with { Payload = """{"retry": true}""" };
Console.WriteLine($"Original payload: {event1.Payload}");
Console.WriteLine($"Retry payload:    {retryEvent.Payload}");

var rateLimit = new RateLimitState(998, 1000, DateTime.UtcNow.AddMinutes(1));
Console.WriteLine($"Rate limit: {rateLimit}");  // Records have built-in ToString
Console.WriteLine();

// =============================================================================
// 9. ENUMS AND FLAGS ENUMS
// =============================================================================
// Enums are named constants backed by an integer type (default: int).
// Flags enums use [Flags] attribute for bitwise combinations.

enum HttpMethod
{
    Get = 1,
    Post = 2,
    Put = 3,
    Patch = 4,
    Delete = 5
}

[Flags]
enum Permission
{
    None    = 0,
    Read    = 1 << 0,  // 1
    Write   = 1 << 1,  // 2
    Execute = 1 << 2,  // 4
    Admin   = Read | Write | Execute  // 7
}

var method = HttpMethod.Post;
var userPerms = Permission.Read | Permission.Write;

Console.WriteLine("--- Enums ---");
Console.WriteLine($"Method: {method} (value: {(int)method})");
Console.WriteLine($"Permissions: {userPerms} (value: {(int)userPerms})");
Console.WriteLine($"Has Write? {userPerms.HasFlag(Permission.Write)}");
Console.WriteLine($"Has Execute? {userPerms.HasFlag(Permission.Execute)}");
Console.WriteLine();

// =============================================================================
// 10. PATTERN MATCHING
// =============================================================================
// C# pattern matching is extensive — closer to Rust's match than Go's switch.
// Switch expressions return values (expression-based, not statement-based).

// Type patterns + switch expression
object ParseConfigValue(string raw) => raw switch
{
    _ when int.TryParse(raw, out var i) => i,
    _ when double.TryParse(raw, out var d) => d,
    "true" or "false" => bool.Parse(raw),
    _ => raw  // default: keep as string
};

Console.WriteLine("--- Pattern Matching ---");
var configValues = new[] { "8080", "3.14", "true", "us-east-1" };
foreach (var raw in configValues)
{
    var parsed = ParseConfigValue(raw);
    Console.WriteLine($"  \"{raw}\" => {parsed} ({parsed.GetType().Name})");
}

// Property patterns — match on properties of objects
record ServiceHealth(string Name, int ResponseMs, bool IsUp);

string ClassifyService(ServiceHealth svc) => svc switch
{
    { IsUp: false }                       => "DOWN",
    { IsUp: true, ResponseMs: < 50 }     => "HEALTHY",
    { IsUp: true, ResponseMs: < 200 }    => "DEGRADED",
    { IsUp: true, ResponseMs: >= 200 }   => "CRITICAL",
    _                                     => "UNKNOWN"
};

var services = new ServiceHealth[]
{
    new("auth-service", 12, true),
    new("payment-api", 150, true),
    new("search-index", 450, true),
    new("notification-svc", 0, false),
};

Console.WriteLine("\nService health dashboard:");
foreach (var svc in services)
    Console.WriteLine($"  {svc.Name,-20} {svc.ResponseMs,4}ms  [{ClassifyService(svc)}]");

// Relational + logical patterns
string DescribeLatency(double ms) => ms switch
{
    <= 10                => "excellent",
    > 10 and <= 50       => "good",
    > 50 and <= 200      => "acceptable",
    > 200 and <= 1000    => "slow",
    > 1000               => "critical",
    _                    => "unknown"   // handles NaN
};

Console.WriteLine($"\n45ms latency is: {DescribeLatency(45)}");
Console.WriteLine($"500ms latency is: {DescribeLatency(500)}");
Console.WriteLine();

// =============================================================================
// 11. CONST VS READONLY
// =============================================================================
// const: compile-time constant, inlined at call sites. Must be a literal or
//        computed from other constants. Similar to Go's const.
// readonly: runtime constant, set once (in declaration or constructor).
//           No top-level readonly in C# — it's a field modifier on classes/structs.
// For top-level code, const is the option. readonly applies to struct/class fields.

const int MaxRetries = 3;
const string DefaultRegion = "us-east-1";
const double TimeoutSeconds = 30.0;

// readonly is demonstrated here via a struct
readonly struct ConnectionConfig
{
    // readonly fields — set in constructor, immutable after
    public readonly string Host;
    public readonly int Port;
    public readonly int MaxPoolSize;

    public ConnectionConfig(string host, int port, int maxPoolSize)
    {
        Host = host;
        Port = port;
        MaxPoolSize = maxPoolSize;
    }

    // readonly struct means the entire struct is immutable.
    // The compiler enforces that no method mutates fields.
    // This also enables performance optimizations: the compiler can pass
    // readonly structs by reference without defensive copies.
}

var dbConfig = new ConnectionConfig("db.internal", 5432, 20);
Console.WriteLine("--- Const vs Readonly ---");
Console.WriteLine($"const MaxRetries: {MaxRetries}");
Console.WriteLine($"const DefaultRegion: {DefaultRegion}");
Console.WriteLine($"readonly struct: {dbConfig.Host}:{dbConfig.Port} (pool: {dbConfig.MaxPoolSize})");
Console.WriteLine();

// =============================================================================
// 12. SPAN<T> — STACK-ALLOCATED SLICING
// =============================================================================
// Span<T> is a ref struct that provides a view into contiguous memory
// (arrays, stack-allocated buffers, native memory) without heap allocation.
// Similar in concept to Go slices or Rust slices (&[T]), but with the
// constraint that it can never escape to the heap (can't be a field in a class,
// can't be captured by a lambda, can't cross async boundaries).

Console.WriteLine("--- Span<T> ---");

int[] requestLatencies = { 12, 45, 3, 89, 23, 67, 8, 156, 34, 5 };

// Span provides zero-copy slicing — no new array allocated
Span<int> recentLatencies = requestLatencies.AsSpan(5, 5);  // last 5 elements
Console.WriteLine($"Full array length: {requestLatencies.Length}");
Console.WriteLine($"Span slice length: {recentLatencies.Length}");

// Mutations through the span affect the original array
recentLatencies[0] = 999;
Console.WriteLine($"After span mutation, original[5] = {requestLatencies[5]}");  // 999

// Stack-allocated buffer with Span (no heap allocation at all)
Span<byte> headerBuffer = stackalloc byte[64];
headerBuffer[0] = 0x48;  // 'H'
headerBuffer[1] = 0x54;  // 'T'
headerBuffer[2] = 0x54;  // 'T'
headerBuffer[3] = 0x50;  // 'P'
Console.WriteLine($"Stack buffer first 4 bytes: {(char)headerBuffer[0]}{(char)headerBuffer[1]}{(char)headerBuffer[2]}{(char)headerBuffer[3]}");

// ReadOnlySpan for immutable views — common for string processing
ReadOnlySpan<char> logLine = "2026-02-12T10:30:00Z INFO request processed in 45ms".AsSpan();
ReadOnlySpan<char> timestamp = logLine[..20];
ReadOnlySpan<char> level = logLine[21..25];
Console.WriteLine($"Parsed timestamp: {timestamp}");
Console.WriteLine($"Parsed level: {level}");
Console.WriteLine();

Console.WriteLine("=== All variable and type demos complete ===");
