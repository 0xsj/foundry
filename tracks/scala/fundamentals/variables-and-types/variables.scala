// ============================================================================
// Variables and Types — Scala 3
// ============================================================================
// Run with: scala-cli run variables.scala
// Covers: val/var, type inference, numeric types, strings, Option, tuples,
//         case classes, enums, union types, pattern matching, type aliases,
//         opaque types.
// ============================================================================

// ----------------------------------------------------------------------------
// Type definitions (must be top-level in Scala 3)
// ----------------------------------------------------------------------------

// --- Case Classes ---
// Case classes are Scala's primary data carrier — like Go structs but with
// built-in equality, toString, copy, and pattern matching support.
// Think of them as "value objects" you'd use for configs, payloads, DTOs.

case class WebhookPayload(
  url:         String,
  method:      String,
  retryCount:  Int,
  contentType: Option[String] // nullable fields use Option, not null
)

case class ServiceConfig(
  host:     String  = "localhost",
  port:     Int     = 8080,
  debug:    Boolean = false,
  maxConns: Int     = 100
)

// --- Enums (Scala 3 syntax) ---
// Scala 3 enums replace the verbose sealed trait + case object pattern.
// Unlike Go's iota or TS's enum, Scala enums are full algebraic types.

enum HttpStatus(val code: Int, val reason: String):
  case Ok                extends HttpStatus(200, "OK")
  case Created           extends HttpStatus(201, "Created")
  case BadRequest        extends HttpStatus(400, "Bad Request")
  case Unauthorized      extends HttpStatus(401, "Unauthorized")
  case NotFound          extends HttpStatus(404, "Not Found")
  case InternalError     extends HttpStatus(500, "Internal Server Error")

  def isSuccess: Boolean = code >= 200 && code < 300
  def isClientError: Boolean = code >= 400 && code < 500

enum LogLevel:
  case Debug, Info, Warn, Error

  def isActionable: Boolean = this match
    case Warn | Error => true
    case _            => false

// --- Opaque Types ---
// Zero-cost wrappers that exist only at compile time. The JVM sees the
// underlying type at runtime. Similar to Haskell's newtype or Rust's
// newtype pattern, but with no boxing overhead.
//
// Outside this object, UserId is a distinct type from Int.
// Inside, it's just an Int — no wrapper allocation.

object Types:
  opaque type UserId = Int
  object UserId:
    def apply(id: Int): UserId = id
    extension (id: UserId)
      def value: Int = id
      def isValid: Boolean = id > 0

  opaque type Email = String
  object Email:
    def apply(raw: String): Either[String, Email] =
      if raw.contains("@") then Right(raw)
      else Left(s"Invalid email: $raw")
    extension (e: Email)
      def value: String = e
      def domain: String = e.split("@").last

// --- Type Alias ---
// Unlike opaque types, aliases are interchangeable with the original type.
// Use them for readability, not safety.

type Headers = Map[String, String]
type StatusCode = Int

// ----------------------------------------------------------------------------
// Entry point
// ----------------------------------------------------------------------------

@main def variablesDemo(): Unit =

  // ========================================================================
  // 1. VAL vs VAR
  // val = immutable binding (like Go's effectively-final or TS's const)
  // var = mutable binding (like Go's var or TS's let)
  // Scala strongly prefers val — idiomatic code is overwhelmingly immutable.
  // ========================================================================

  println("=== 1. val vs var ===")

  val serviceName: String = "webhook-relay"  // immutable — cannot reassign
  var requestCount: Int = 0                  // mutable — can reassign

  requestCount += 1
  requestCount += 1

  println(s"service: $serviceName")
  println(s"requests: $requestCount")

  // val serviceName = "other"   // ERROR: reassignment to val
  // This is different from JS const: in JS, const objects are still mutable
  // internally. In Scala, val just means the binding can't be reassigned —
  // the underlying object could still be mutable (e.g., ArrayBuffer).

  // ========================================================================
  // 2. TYPE INFERENCE
  // Scala infers types from the right-hand side, similar to Go's := but
  // more powerful — Scala can infer generics, function return types, etc.
  // ========================================================================

  println("\n=== 2. Type Inference ===")

  val port = 8080              // inferred as Int
  val timeout = 30.0           // inferred as Double
  val enabled = true           // inferred as Boolean
  val endpoint = "/api/hooks"  // inferred as String
  val retries = 3L             // inferred as Long (L suffix)
  val ratio = 0.95f            // inferred as Float (f suffix)

  // Unlike Go, you can always omit the type when inference is unambiguous.
  // Unlike TS, inference works across complex expressions and generics.

  println(s"port: $port (${port.getClass.getSimpleName})")
  println(s"timeout: $timeout (${timeout.getClass.getSimpleName})")
  println(s"enabled: $enabled (${enabled.getClass.getSimpleName})")
  println(s"endpoint: $endpoint (${endpoint.getClass.getSimpleName})")
  println(s"retries: $retries (${retries.getClass.getSimpleName})")
  println(s"ratio: $ratio (${ratio.getClass.getSimpleName})")

  // ========================================================================
  // 3. NUMERIC TYPES — Sizes and Ranges
  // On the JVM, these map directly to Java primitives when unboxed.
  // ========================================================================

  println("\n=== 3. Numeric Types ===")

  val byteVal: Byte   = 127          // 8-bit signed  [-128, 127]
  val shortVal: Short  = 32767       // 16-bit signed [-32768, 32767]
  val intVal: Int      = 2_147_483_647  // 32-bit signed (underscores for readability)
  val longVal: Long    = 9_223_372_036_854_775_807L  // 64-bit signed
  val floatVal: Float  = 3.14f       // 32-bit IEEE 754
  val doubleVal: Double = 3.141592653589793  // 64-bit IEEE 754
  val charVal: Char    = 'A'         // 16-bit Unicode (UTF-16 code unit)

  println(s"Byte:   $byteVal   range [${Byte.MinValue}, ${Byte.MaxValue}]")
  println(s"Short:  $shortVal  range [${Short.MinValue}, ${Short.MaxValue}]")
  println(s"Int:    $intVal    range [${Int.MinValue}, ${Int.MaxValue}]")
  println(s"Long:   $longVal   range [${Long.MinValue}, ${Long.MaxValue}]")
  println(s"Float:  $floatVal")
  println(s"Double: $doubleVal")
  println(s"Char:   $charVal   (code: ${charVal.toInt})")

  // BigInt and BigDecimal for arbitrary precision (like Python's int or Haskell's Integer)
  val bigNum: BigInt = BigInt("99999999999999999999999999999")
  val precise: BigDecimal = BigDecimal("0.1") + BigDecimal("0.2")
  println(s"BigInt: $bigNum")
  println(s"BigDecimal 0.1 + 0.2: $precise")  // 0.3, not 0.30000000000000004

  // ========================================================================
  // 4. STRINGS AND INTERPOLATION
  // Scala has three string interpolation modes: s"", f"", raw""
  // ========================================================================

  println("\n=== 4. Strings and Interpolation ===")

  val host = "api.example.com"
  val statusCode = 200
  val latencyMs = 3.14159

  // s-interpolation: basic expression embedding (like TS template literals)
  val logLine = s"[$statusCode] $host responded in ${latencyMs}ms"
  println(s"s-string: $logLine")

  // f-interpolation: printf-style formatting
  val formatted = f"latency: $latencyMs%.2f ms, status: $statusCode%04d"
  println(s"f-string: $formatted")

  // raw-interpolation: no escape processing
  val rawPath = raw"C:\Users\config\new_line\test"
  println(s"raw-string: $rawPath")

  // Multi-line strings with stripMargin (like Go's backtick strings, but better)
  val jsonTemplate =
    s"""|{
        |  "host": "$host",
        |  "port": $port,
        |  "status": $statusCode
        |}""".stripMargin
  println(s"multi-line:\n$jsonTemplate")

  // String operations
  val path = "/api/v2/webhooks/retry"
  println(s"starts with /api: ${path.startsWith("/api")}")
  println(s"split on /: ${path.split("/").filter(_.nonEmpty).mkString(", ")}")

  // ========================================================================
  // 5. BOOLEAN AND UNIT
  // Boolean is standard. Unit is Scala's "void" — it has exactly one value: ()
  // ========================================================================

  println("\n=== 5. Boolean and Unit ===")

  val isHealthy: Boolean = true
  val isStale: Boolean = false
  println(s"healthy AND not stale: ${isHealthy && !isStale}")

  // Unit is the type of expressions that return nothing useful.
  // Every println returns Unit. Unlike Go's implicit void, Unit is explicit.
  val result: Unit = println("this println returns Unit")
  println(s"Unit value: $result")  // prints: ()

  // ========================================================================
  // 6. OPTION[A] — Some and None
  // Scala's replacement for null. Like Rust's Option<T> or Haskell's Maybe a.
  // Forces you to handle the absent case at compile time.
  // ========================================================================

  println("\n=== 6. Option[A] ===")

  val configuredTimeout: Option[Int] = Some(30)
  val fallbackTimeout: Option[Int] = None

  // getOrElse — provide a default (like ?? in TS or .unwrap_or() in Rust)
  println(s"timeout: ${configuredTimeout.getOrElse(60)}s")
  println(s"fallback: ${fallbackTimeout.getOrElse(60)}s")

  // map — transform the inner value if present (functor)
  val doubledTimeout = configuredTimeout.map(_ * 2)
  println(s"doubled: $doubledTimeout")  // Some(60)

  // flatMap — chain operations that also return Option (monad)
  def lookupUser(id: Int): Option[String] =
    if id == 1 then Some("alice") else None

  def lookupRole(name: String): Option[String] =
    if name == "alice" then Some("admin") else None

  val role = lookupUser(1).flatMap(lookupRole)
  val noRole = lookupUser(2).flatMap(lookupRole)
  println(s"alice's role: $role")   // Some(admin)
  println(s"unknown role: $noRole") // None

  // for-comprehension — syntactic sugar for flatMap chains
  // (like Haskell's do-notation or Rust's ? operator chaining)
  val roleViaFor = for
    user <- lookupUser(1)
    r    <- lookupRole(user)
  yield r.toUpperCase

  println(s"role via for: $roleViaFor")  // Some(ADMIN)

  // Pattern matching on Option
  configuredTimeout match
    case Some(t) => println(s"configured: ${t}s")
    case None    => println("using default timeout")

  // ========================================================================
  // 7. TUPLES
  // Fixed-size heterogeneous collections. Accessed by position (_1, _2, etc.)
  // or by destructuring. Scala 3 tuples are much improved over Scala 2.
  // ========================================================================

  println("\n=== 7. Tuples ===")

  val endpoint_info: (String, Int, Boolean) = ("/health", 200, true)
  println(s"path: ${endpoint_info._1}, status: ${endpoint_info._2}, cached: ${endpoint_info._3}")

  // Destructuring (like JS destructuring or Go's multi-return)
  val (path2, status, cached) = endpoint_info
  println(s"destructured: $path2 -> $status (cached=$cached)")

  // Named tuples (Scala 3.5+) — uncomment if your Scala version supports them
  // val named = (path = "/api", method = "GET")
  // println(s"named: ${named.path} ${named.method}")

  // Tuples are useful for quick grouping without defining a case class.
  // Use case classes when the structure is reused or needs methods.

  // ========================================================================
  // 8. CASE CLASSES
  // Scala's primary data structure — immutable by default, with built-in
  // equality, toString, copy, and pattern matching.
  // ========================================================================

  println("\n=== 8. Case Classes ===")

  val payload = WebhookPayload(
    url = "https://api.example.com/hooks",
    method = "POST",
    retryCount = 3,
    contentType = Some("application/json")
  )
  println(s"payload: $payload")

  // copy — create a modified clone (like JS spread: { ...obj, field: newVal })
  val retried = payload.copy(retryCount = payload.retryCount + 1)
  println(s"retried: $retried")

  // Default parameters in case class constructors
  val defaultConfig = ServiceConfig()
  val prodConfig = ServiceConfig(host = "prod.example.com", port = 443, maxConns = 1000)
  println(s"default: $defaultConfig")
  println(s"prod:    $prodConfig")

  // Structural equality — == compares values, not references (unlike Java/JS ===)
  val cfg1 = ServiceConfig(port = 9090)
  val cfg2 = ServiceConfig(port = 9090)
  println(s"cfg1 == cfg2: ${cfg1 == cfg2}")  // true (structural equality)

  // ========================================================================
  // 9. ENUMS
  // Scala 3 enums are algebraic data types. They can carry data, have
  // methods, and implement traits. Much more powerful than Java/TS enums.
  // ========================================================================

  println("\n=== 9. Enums ===")

  val status1 = HttpStatus.Ok
  val status2 = HttpStatus.NotFound

  println(s"${status1.code} ${status1.reason} — success? ${status1.isSuccess}")
  println(s"${status2.code} ${status2.reason} — client error? ${status2.isClientError}")

  // Simple enum without parameters
  val level = LogLevel.Warn
  println(s"log level: $level, actionable? ${level.isActionable}")

  // Enum values have ordinal and fromOrdinal
  println(s"Debug ordinal: ${LogLevel.Debug.ordinal}")
  println(s"From ordinal 2: ${LogLevel.fromOrdinal(2)}")  // Warn

  // All values
  println(s"All log levels: ${LogLevel.values.mkString(", ")}")

  // ========================================================================
  // 10. UNION TYPES (Scala 3)
  // A | B means "either A or B" — like TypeScript's union types.
  // No wrapper needed. This is a compile-time construct.
  // ========================================================================

  println("\n=== 10. Union Types ===")

  // A config value can be a String, Int, or Boolean — like a YAML/TOML value
  type ConfigValue = String | Int | Boolean

  def formatConfigValue(v: ConfigValue): String = v match
    case s: String  => s""""$s""""
    case i: Int     => i.toString
    case b: Boolean => if b then "true" else "false"

  val configEntries: List[(String, ConfigValue)] = List(
    "host"     -> "localhost",
    "port"     -> 8080,
    "debug"    -> true,
    "maxConns" -> 256
  )

  for (key, value) <- configEntries do
    println(s"  $key = ${formatConfigValue(value)}")

  // Union types vs Option: unions are for "one of several types",
  // Option is for "present or absent". Don't use String | Null — use Option[String].

  // ========================================================================
  // 11. PATTERN MATCHING
  // Scala's pattern matching is exhaustive (the compiler warns if you miss
  // a case) and works on types, values, structure, and guards.
  // ========================================================================

  println("\n=== 11. Pattern Matching ===")

  // Matching on enum values
  def describeStatus(s: HttpStatus): String = s match
    case HttpStatus.Ok | HttpStatus.Created => "success"
    case HttpStatus.BadRequest              => "bad request — check your payload"
    case HttpStatus.Unauthorized            => "unauthorized — check your API key"
    case HttpStatus.NotFound                => "not found — check the endpoint URL"
    case HttpStatus.InternalError           => "server error — retry with backoff"

  println(describeStatus(HttpStatus.NotFound))
  println(describeStatus(HttpStatus.Ok))

  // Matching on case class structure (destructuring)
  def describePayload(p: WebhookPayload): String = p match
    case WebhookPayload(_, _, retries, _) if retries > 5 =>
      "too many retries — moving to dead letter queue"
    case WebhookPayload(url, "POST", _, Some(ct)) =>
      s"POST to $url with content-type $ct"
    case WebhookPayload(url, method, _, None) =>
      s"$method to $url (no content-type)"
    case other =>
      s"webhook: $other"

  println(describePayload(payload))
  println(describePayload(payload.copy(retryCount = 10)))
  println(describePayload(WebhookPayload("https://x.com", "DELETE", 0, None)))

  // Matching on type (like a type switch in Go or instanceof chain in TS)
  def inspectAny(x: Any): String = x match
    case s: String          => s"string of length ${s.length}"
    case i: Int if i > 0    => s"positive int: $i"
    case i: Int             => s"non-positive int: $i"
    case list: List[?]      => s"list with ${list.size} elements"
    case _                  => s"unknown: $x"

  println(inspectAny("hello"))
  println(inspectAny(42))
  println(inspectAny(-1))
  println(inspectAny(List(1, 2, 3)))

  // ========================================================================
  // 12. TYPE ALIASES AND OPAQUE TYPES
  // Type aliases are just names. Opaque types are compile-time distinct
  // types with zero runtime cost — the JVM sees the underlying type.
  // ========================================================================

  println("\n=== 12. Type Aliases and Opaque Types ===")

  // Type alias — interchangeable with the original type (no safety)
  val responseHeaders: Headers = Map(
    "Content-Type" -> "application/json",
    "X-Request-Id" -> "abc-123"
  )
  val plainMap: Map[String, String] = responseHeaders  // compiles fine — same type
  println(s"headers: $responseHeaders")

  // Opaque type — distinct type at compile time, erased at runtime
  import Types.*

  val userId = UserId(42)
  println(s"userId: ${userId.value}, valid? ${userId.isValid}")

  // This would NOT compile — UserId is not interchangeable with Int:
  // val wrong: Int = userId     // ERROR: type mismatch
  // val wrong2: UserId = 42     // ERROR: outside the companion object

  // Opaque type with validation
  val validEmail = Email("admin@example.com")
  val invalidEmail = Email("not-an-email")
  println(s"valid email: $validEmail")
  println(s"invalid email: $invalidEmail")

  validEmail match
    case Right(e) => println(s"  domain: ${e.domain}")
    case Left(err) => println(s"  error: $err")

  println("\n=== done ===")
