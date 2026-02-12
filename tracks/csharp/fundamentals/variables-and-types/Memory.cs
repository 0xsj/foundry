// Memory.cs — C# Memory Model: Value Types, Reference Types, and the CLR
// Run with: dotnet run Memory.cs
// Requires: .NET 8+ / C# 12 (top-level statements)

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

// =============================================================================
// 1. VALUE TYPES (struct) vs REFERENCE TYPES (class)
// =============================================================================
// Value types live on the stack (or inline in the containing type).
// Reference types live on the heap, with a pointer on the stack.
//
// This is the same distinction as:
//   Go:   structs are values; interfaces/slices/maps are reference-like
//   Rust: stack by default; Box<T> for heap
//   TS:   primitives are values; objects are references
//
// In C#, the split is explicit:
//   struct = value type (stack)
//   class  = reference type (heap)

struct GeoCoordinate   // Value type — 16 bytes on the stack
{
    public double Latitude;
    public double Longitude;
}

class ServiceEndpoint  // Reference type — object on heap, 8-byte pointer on stack
{
    public string Host = "";
    public int Port;
    public bool IsHealthy;
}

Console.WriteLine("=== VALUE TYPES vs REFERENCE TYPES ===\n");

// Value type: assignment copies the entire value
var coordA = new GeoCoordinate { Latitude = 47.6062, Longitude = -122.3321 };
var coordB = coordA;  // Full copy — coordB is independent
coordB.Latitude = 0.0;

Console.WriteLine("--- Value type (struct) copy semantics ---");
Console.WriteLine($"coordA.Latitude: {coordA.Latitude}");  // 47.6062 — unchanged
Console.WriteLine($"coordB.Latitude: {coordB.Latitude}");  // 0.0     — independent copy

// Reference type: assignment copies the pointer, not the object
var endpointA = new ServiceEndpoint { Host = "api.internal", Port = 8080, IsHealthy = true };
var endpointB = endpointA;  // Both variables point to the SAME heap object
endpointB.Port = 9090;

Console.WriteLine("\n--- Reference type (class) reference semantics ---");
Console.WriteLine($"endpointA.Port: {endpointA.Port}");  // 9090 — mutated through endpointB!
Console.WriteLine($"endpointB.Port: {endpointB.Port}");  // 9090 — same object
Console.WriteLine($"Same object? {ReferenceEquals(endpointA, endpointB)}");  // true
Console.WriteLine();

// =============================================================================
// 2. SIZES AND LAYOUT
// =============================================================================
// Value types have a predictable, contiguous memory layout.
// The CLR may add padding for alignment (similar to C struct padding).

struct MetricSample   // Observe the size: not just sum of fields
{
    public long Timestamp;    // 8 bytes
    public double Value;      // 8 bytes
    public byte MetricType;   // 1 byte + 7 bytes padding (alignment to 8)
}

Console.WriteLine("=== SIZES AND LAYOUT ===\n");
Console.WriteLine($"GeoCoordinate size:  {Unsafe.SizeOf<GeoCoordinate>()} bytes (2x double)");
Console.WriteLine($"MetricSample size:   {Unsafe.SizeOf<MetricSample>()} bytes (includes padding)");
Console.WriteLine($"  Expected without padding: {8 + 8 + 1} bytes");
Console.WriteLine($"  Actual with alignment:    {Unsafe.SizeOf<MetricSample>()} bytes");

// Arrays of value types are contiguous in memory (cache-friendly)
// Arrays of reference types are arrays of pointers (each element may be anywhere on heap)
var samples = new MetricSample[1000];  // 1000 structs contiguous in memory
var endpoints = new ServiceEndpoint[1000];  // 1000 pointers; objects scattered on heap
Console.WriteLine($"\nMetricSample[1000]: contiguous {Unsafe.SizeOf<MetricSample>() * 1000} bytes");
Console.WriteLine($"ServiceEndpoint[1000]: 1000 pointers + scattered heap objects");
Console.WriteLine();

// =============================================================================
// 3. BOXING AND UNBOXING
// =============================================================================
// Boxing: wrapping a value type in an object on the heap.
// Unboxing: extracting the value type back from the heap object.
//
// This happens implicitly when you assign a value type to `object` or an
// interface. It's a hidden allocation — a performance trap in hot paths.
//
// Go equivalent: interface{} wrapping. Rust equivalent: dyn Trait (sort of).

Console.WriteLine("=== BOXING AND UNBOXING ===\n");

int requestCount = 42;

// Boxing: int (stack) -> object (heap). Allocates a new heap object.
object boxed = requestCount;

// Unboxing: object (heap) -> int (stack). Copies the value back.
int unboxed = (int)boxed;

Console.WriteLine($"Original (stack):  {requestCount}");
Console.WriteLine($"Boxed (heap):      {boxed}");
Console.WriteLine($"Unboxed (stack):   {unboxed}");

// The boxed value is a separate copy — mutating the original doesn't affect it
requestCount = 100;
Console.WriteLine($"\nAfter mutating original to 100:");
Console.WriteLine($"Original:  {requestCount}");  // 100
Console.WriteLine($"Boxed:     {boxed}");           // still 42

// Common boxing traps:
// 1. Passing value types to methods accepting `object`
// 2. Using value types in non-generic collections (ArrayList)
// 3. Calling interface methods on value types without constrained generics
// 4. String formatting with value types: $"{someInt}" boxes the int
//    (modern .NET optimizes some of these cases)

// Demonstrating that boxing creates a new heap allocation each time
object boxed1 = 42;
object boxed2 = 42;
Console.WriteLine($"\nBoxed 42 == Boxed 42 (Equals):          {boxed1.Equals(boxed2)}");       // true
Console.WriteLine($"Boxed 42 same ref (ReferenceEquals):     {ReferenceEquals(boxed1, boxed2)}"); // false — different heap objects
Console.WriteLine();

// =============================================================================
// 4. REF STRUCT — STACK-ONLY TYPES
// =============================================================================
// A ref struct can NEVER escape to the heap. The compiler enforces:
//   - Cannot be a field of a class (or regular struct)
//   - Cannot be boxed
//   - Cannot be used in async methods
//   - Cannot be captured by lambdas
//   - Cannot implement interfaces (until C# 13)
//
// The primary use case: Span<T>, ReadOnlySpan<T>, and custom stack-only buffers.
// This is similar to Rust's lifetime constraints preventing dangling references.

ref struct RequestParser
{
    private ReadOnlySpan<char> _buffer;
    public int Position;

    public RequestParser(ReadOnlySpan<char> buffer)
    {
        _buffer = buffer;
        Position = 0;
    }

    public ReadOnlySpan<char> ReadUntil(char delimiter)
    {
        int start = Position;
        while (Position < _buffer.Length && _buffer[Position] != delimiter)
            Position++;
        var result = _buffer[start..Position];
        if (Position < _buffer.Length) Position++;  // skip delimiter
        return result;
    }
}

Console.WriteLine("=== REF STRUCT ===\n");

// The parser and its spans never touch the heap
ReadOnlySpan<char> rawHeader = "X-Request-Id:req_abc123:extra".AsSpan();
var parser = new RequestParser(rawHeader);
var headerName = parser.ReadUntil(':');
var headerValue = parser.ReadUntil(':');
Console.WriteLine($"Parsed header: {headerName} = {headerValue}");
Console.WriteLine($"Parser position after: {parser.Position}");

// These would NOT compile (enforced by compiler):
// object boxedParser = parser;                     // Cannot box ref struct
// ServiceEndpoint e = new() { /* parser field */ }; // Cannot store in class
// var captured = () => parser.Position;              // Cannot capture in lambda
Console.WriteLine("(ref struct cannot be boxed, stored in class, or captured by lambda)");
Console.WriteLine();

// =============================================================================
// 5. STRING INTERNING
// =============================================================================
// The CLR maintains an intern pool of string literals. Identical string literals
// share the same object reference. This saves memory but means ReferenceEquals
// on literal strings can return true (which surprises people).
//
// Runtime-created strings are NOT automatically interned.

Console.WriteLine("=== STRING INTERNING ===\n");

string literal1 = "api-gateway";
string literal2 = "api-gateway";

// Both literals point to the same interned object
Console.WriteLine($"literal1 == literal2:            {literal1 == literal2}");                  // true
Console.WriteLine($"ReferenceEquals(lit1, lit2):      {ReferenceEquals(literal1, literal2)}");  // true (same interned object)

// Runtime-created string with same value — different object
string runtime = string.Concat("api-", "gateway");
Console.WriteLine($"\nliteral1 == runtime:             {runtime == literal1}");                   // true (value equality)
Console.WriteLine($"ReferenceEquals(lit1, runtime):   {ReferenceEquals(literal1, runtime)}");     // false (different objects)

// Explicitly intern a runtime string
string interned = string.Intern(runtime);
Console.WriteLine($"\nAfter interning:");
Console.WriteLine($"ReferenceEquals(lit1, interned): {ReferenceEquals(literal1, interned)}");     // true (now same object)
Console.WriteLine();

// =============================================================================
// 6. EQUALITY — THREE WAYS TO COMPARE
// =============================================================================
// == operator:       Can be overloaded. For strings: value equality.
//                    For classes: reference equality by default.
//                    For records: value equality (compiler-generated).
// .Equals():         Virtual method. Default: reference equality for classes,
//                    bitwise equality for structs. Can be overridden.
// ReferenceEquals(): Always checks if two variables point to the same object.
//                    Cannot be overridden. Always reference identity.

Console.WriteLine("=== EQUALITY ===\n");

// Class: == is reference equality by default (unless overloaded)
var ep1 = new ServiceEndpoint { Host = "api.internal", Port = 8080 };
var ep2 = new ServiceEndpoint { Host = "api.internal", Port = 8080 };

Console.WriteLine("--- Class (ServiceEndpoint) ---");
Console.WriteLine($"ep1 == ep2:              {ep1 == ep2}");                  // false (different refs)
Console.WriteLine($"ep1.Equals(ep2):         {ep1.Equals(ep2)}");            // false (default: ref equality)
Console.WriteLine($"ReferenceEquals(ep1,ep2): {ReferenceEquals(ep1, ep2)}");  // false

// Struct: Equals compares fields (value equality), but == must be explicitly defined
var c1 = new GeoCoordinate { Latitude = 47.6, Longitude = -122.3 };
var c2 = new GeoCoordinate { Latitude = 47.6, Longitude = -122.3 };

Console.WriteLine("\n--- Struct (GeoCoordinate) ---");
Console.WriteLine($"c1.Equals(c2):           {c1.Equals(c2)}");              // true (value comparison)
// c1 == c2 won't compile unless operator== is defined on the struct

// Record class: == is value equality (compiler-generated)
record CacheKey(string Namespace, string Key);
var key1 = new CacheKey("sessions", "user_123");
var key2 = new CacheKey("sessions", "user_123");

Console.WriteLine("\n--- Record class (CacheKey) ---");
Console.WriteLine($"key1 == key2:              {key1 == key2}");                  // true (value equality!)
Console.WriteLine($"key1.Equals(key2):         {key1.Equals(key2)}");            // true
Console.WriteLine($"ReferenceEquals(key1,key2): {ReferenceEquals(key1, key2)}");  // false (still different objects)
Console.WriteLine($"key1.GetHashCode():        {key1.GetHashCode()}");
Console.WriteLine($"key2.GetHashCode():        {key2.GetHashCode()}");            // same hash!
Console.WriteLine();

// =============================================================================
// 7. RECORD STRUCT vs RECORD CLASS — MEMORY DIFFERENCES
// =============================================================================
// record class: heap-allocated, reference semantics, value equality
// record struct: stack-allocated (usually), value semantics, value equality
//
// Both get: Equals, GetHashCode, ToString, `with` expressions, deconstruct.
// The difference is WHERE they live in memory and COPY behavior.

record class AuditEntry(string UserId, string Action, DateTime Timestamp);
record struct MetricPoint(string Name, double Value, long EpochMs);

Console.WriteLine("=== RECORD CLASS vs RECORD STRUCT ===\n");

// Record class: assignment copies the reference
var audit1 = new AuditEntry("user_42", "login", DateTime.UtcNow);
var audit2 = audit1;  // Same heap object

Console.WriteLine("--- record class (AuditEntry) ---");
Console.WriteLine($"Same object after assignment? {ReferenceEquals(audit1, audit2)}");  // true

// Record struct: assignment copies the entire value
var metric1 = new MetricPoint("cpu_usage", 73.4, DateTimeOffset.UtcNow.ToUnixTimeMilliseconds());
var metric2 = metric1;  // Full copy on stack

Console.WriteLine("\n--- record struct (MetricPoint) ---");
Console.WriteLine($"metric1 == metric2: {metric1 == metric2}");  // true (value equality)
// Can't use ReferenceEquals on value types (would box them into different objects)

// `with` on record class: creates a NEW heap object (shallow copy + mutation)
var audit3 = audit1 with { Action = "logout" };
Console.WriteLine($"\nAudit1 action: {audit1.Action}");   // login
Console.WriteLine($"Audit3 action: {audit3.Action}");     // logout
Console.WriteLine($"Same object?   {ReferenceEquals(audit1, audit3)}");  // false

// `with` on record struct: copies value, modifies the copy (all on stack)
var metric3 = metric1 with { Value = 95.2 };
Console.WriteLine($"\nMetric1 value: {metric1.Value}");   // 73.4
Console.WriteLine($"Metric3 value: {metric3.Value}");     // 95.2
Console.WriteLine($"metric1 == metric3: {metric1 == metric3}");  // false (different values)
Console.WriteLine();

// =============================================================================
// 8. NULLABLE<T> INTERNALS
// =============================================================================
// int? is syntactic sugar for Nullable<int>.
// Nullable<T> is a struct with two fields:
//   - bool HasValue
//   - T Value
//
// When HasValue is false, accessing Value throws InvalidOperationException.
// Nullable<T> is a value type — it lives on the stack, NOT the heap.
// Size: sizeof(T) + 1 byte (HasValue) + padding.
//
// This differs from Rust's Option<T> which uses niche optimization
// (e.g., Option<NonZeroU32> is same size as u32).

Console.WriteLine("=== NULLABLE<T> INTERNALS ===\n");

int? connectionTimeout = 30;
int? readTimeout = null;

Console.WriteLine($"connectionTimeout.HasValue: {connectionTimeout.HasValue}");  // true
Console.WriteLine($"connectionTimeout.Value:    {connectionTimeout.Value}");      // 30
Console.WriteLine($"readTimeout.HasValue:       {readTimeout.HasValue}");          // false

// Accessing Value on null throws — use GetValueOrDefault() or ?? instead
try
{
    _ = readTimeout.Value;
}
catch (InvalidOperationException ex)
{
    Console.WriteLine($"readTimeout.Value threw: {ex.GetType().Name}");
}

Console.WriteLine($"readTimeout.GetValueOrDefault():   {readTimeout.GetValueOrDefault()}");     // 0
Console.WriteLine($"readTimeout.GetValueOrDefault(60): {readTimeout.GetValueOrDefault(60)}");   // 60
Console.WriteLine($"readTimeout ?? 60:                 {readTimeout ?? 60}");                    // 60

// Size comparison: Nullable<T> adds overhead
Console.WriteLine($"\nsizeof(int):         {sizeof(int)} bytes");
Console.WriteLine($"sizeof(Nullable<int>): {Unsafe.SizeOf<int?>()} bytes");  // 8 (4 + 1 + 3 padding)
Console.WriteLine($"sizeof(double):        {sizeof(double)} bytes");
Console.WriteLine($"sizeof(Nullable<double>): {Unsafe.SizeOf<double?>()} bytes");  // 16 (8 + 1 + 7 padding)

// Boxing a nullable: if HasValue is false, boxes to null (not Nullable<T>)
int? nullableVal = 42;
int? nullableNull = null;
object boxedVal = nullableVal;    // Boxes the int, not the Nullable<int>
object boxedNull = nullableNull;  // Results in actual null, not boxed Nullable

Console.WriteLine($"\nBoxed 42: {boxedVal} (type: {boxedVal?.GetType()})");          // System.Int32, not Nullable<Int32>
Console.WriteLine($"Boxed null: {boxedNull?.ToString() ?? "(null)"} (is null: {boxedNull is null})");  // true null
Console.WriteLine();

// =============================================================================
// 9. READONLY STRUCT — ENFORCED IMMUTABILITY
// =============================================================================
// A `readonly struct` guarantees no method or property mutates state.
// The compiler enforces this: all fields must be readonly or init-only.
//
// Performance benefit: when a readonly struct is passed by `in` reference,
// the compiler doesn't need to create a defensive copy (it knows the method
// can't mutate the struct). This matters in tight loops with large structs.
//
// Compare to: Rust's default immutability, Go's lack of const structs.

readonly struct HttpResponse
{
    public int StatusCode { get; }
    public string ReasonPhrase { get; }
    public long ContentLength { get; }
    public DateTime ReceivedAt { get; }

    public HttpResponse(int statusCode, string reasonPhrase, long contentLength)
    {
        StatusCode = statusCode;
        ReasonPhrase = reasonPhrase;
        ContentLength = contentLength;
        ReceivedAt = DateTime.UtcNow;
    }

    // This method is fine — it only reads
    public bool IsSuccess => StatusCode >= 200 && StatusCode < 300;

    // This would NOT compile in a readonly struct:
    // public void Reset() { StatusCode = 0; }  // Error: cannot assign to readonly member

    public override string ToString() =>
        $"HTTP {StatusCode} {ReasonPhrase} ({ContentLength} bytes)";
}

Console.WriteLine("=== READONLY STRUCT ===\n");

var response = new HttpResponse(200, "OK", 4096);
Console.WriteLine($"Response: {response}");
Console.WriteLine($"IsSuccess: {response.IsSuccess}");

// Demonstrating the `in` parameter optimization
static void LogResponse(in HttpResponse resp)
{
    // `in` passes by reference (no copy), and because HttpResponse is
    // a readonly struct, the compiler knows this method can't mutate it.
    // Without readonly, the compiler would create a defensive copy here.
    Console.WriteLine($"  Logging: {resp}");
}

LogResponse(in response);  // Zero-copy pass — no defensive copy needed

// Contrast: a regular (non-readonly) struct passed by `in` gets a defensive
// copy every time a method is called on it, because the compiler can't prove
// the method won't mutate the struct. This is a subtle performance trap.

Console.WriteLine("\n--- Summary: when to use readonly struct ---");
Console.WriteLine("  1. Data that should never change after construction");
Console.WriteLine("  2. Types frequently passed by `in` reference");
Console.WriteLine("  3. Types used in hot loops where defensive copies hurt");
Console.WriteLine("  4. Good default for all value types unless mutation is needed");
Console.WriteLine();

// =============================================================================
// 10. PUTTING IT TOGETHER — MEMORY LAYOUT VISUALIZATION
// =============================================================================

Console.WriteLine("=== MEMORY LAYOUT SUMMARY ===\n");
Console.WriteLine("Stack                          Heap");
Console.WriteLine("-----                          ----");
Console.WriteLine("[int requestCount = 42   ]     (nothing — lives entirely on stack)");
Console.WriteLine("[GeoCoordinate coord     ]     (nothing — struct, 16 bytes on stack)");
Console.WriteLine("  .Latitude  = 47.6            ");
Console.WriteLine("  .Longitude = -122.3          ");
Console.WriteLine("[ref endpoint  --------]-----> [ServiceEndpoint object]");
Console.WriteLine("  (8-byte pointer)               .Host = \"api.internal\"");
Console.WriteLine("                                 .Port = 8080");
Console.WriteLine("                                 .IsHealthy = true");
Console.WriteLine("[record struct metric   ]       (nothing — value type on stack)");
Console.WriteLine("  .Name  = (string ref)-------> [\"cpu_usage\" string object]");
Console.WriteLine("  .Value = 73.4                ");
Console.WriteLine("[int? timeout           ]       (nothing — Nullable<int> on stack)");
Console.WriteLine("  .HasValue = true             ");
Console.WriteLine("  .Value    = 30               ");
Console.WriteLine("[object boxed ----------]-----> [boxed int: 42]");
Console.WriteLine("  (8-byte pointer)               (12-byte object header + 4-byte int)");
Console.WriteLine("[Span<int> slice        ]       (nothing — ref struct, stack only)");
Console.WriteLine("  .Reference = (ptr to array)  ");
Console.WriteLine("  .Length    = 5               ");
Console.WriteLine();

Console.WriteLine("=== All memory demos complete ===");
