// ============================================================================
// Memory and JVM Types — Scala 3
// ============================================================================
// Run with: scala-cli run memory.scala
// Covers: value vs reference types, AnyVal/AnyRef hierarchy, boxing,
//         string interning, case class layout, equality semantics,
//         lazy val, value classes.
// ============================================================================

// --- Value class: zero-cost wrapper that avoids allocation ---
// On the JVM, this compiles down to a plain Long in most cases.
// No object header, no heap allocation — just the raw 8 bytes.
// Similar to opaque types but works in Scala 2 as well.

// In Scala 3, value classes should not be case classes — use a plain class
// with a val parameter. case class + AnyVal is deprecated.

class Millis(val value: Long) extends AnyVal:
  def toSeconds: Double = value / 1000.0
  def +(other: Millis): Millis = Millis(value + other.value)

class RequestId(val value: String) extends AnyVal

// --- Case classes for layout demos ---
case class HttpRequest(
  method:      String,
  path:        String,
  statusCode:  Int,
  contentLen:  Long,
  keepAlive:   Boolean
)

case class Compact(a: Byte, b: Byte)
case class Mixed(flag: Boolean, count: Long, tag: Byte)

// ============================================================================

@main def memoryDemo(): Unit =

  // ========================================================================
  // 1. SCALA TYPE HIERARCHY — AnyVal vs AnyRef
  //
  // Scala's type system unifies primitives and objects:
  //
  //              Any                 (top type — like Go's any/interface{})
  //             /   \
  //        AnyVal   AnyRef           (value types vs reference types)
  //        /          \
  //   Int, Double,   String, List,
  //   Boolean, ...   case classes, ...
  //             \   /
  //            Nothing               (bottom type — subtype of everything)
  //
  // AnyVal subtypes (Int, Double, Boolean, etc.) map to JVM primitives
  // when possible — no object header, no heap allocation. They get boxed
  // (wrapped in java.lang.Integer, etc.) when used as generic types.
  // ========================================================================

  println("=== 1. Type Hierarchy ===")

  val x: Int = 42               // JVM primitive int — 4 bytes on stack
  val y: Double = 3.14          // JVM primitive double — 8 bytes on stack
  val z: Boolean = true         // JVM primitive boolean — typically 1 byte
  val s: String = "hello"       // java.lang.String on heap — reference type
  val list: List[Int] = List(1) // scala.collection.immutable.List on heap

  // getClass shows the runtime (JVM) type
  println(s"Int class:     ${x.getClass}")      // int (primitive)
  println(s"Double class:  ${y.getClass}")       // double (primitive)
  println(s"Boolean class: ${z.getClass}")       // boolean (primitive)
  println(s"String class:  ${s.getClass}")       // java.lang.String
  println(s"List class:    ${list.getClass}")     // scala.collection.immutable.$colon$colon

  // In Scala, you can call methods on primitives — the compiler handles it:
  println(s"42.toHexString: ${42.toHexString}")   // "2a" — no boxing needed
  println(s"3.14.ceil: ${3.14.ceil}")             // 4.0

  // ========================================================================
  // 2. BOXING AND UNBOXING
  //
  // When does an Int become a java.lang.Integer?
  // - Stored in a generic collection: List[Int], Map[String, Int]
  // - Passed as Any or AnyVal
  // - Used as a type parameter
  //
  // Array[Int] is special — it uses JVM's int[] (no boxing).
  // List[Int] uses linked nodes with boxed java.lang.Integer.
  // ========================================================================

  println("\n=== 2. Boxing Behavior ===")

  val arr = Array(1, 2, 3, 4, 5)    // JVM int[] — no boxing, contiguous memory
  val lst = List(1, 2, 3, 4, 5)     // Linked list of boxed java.lang.Integer

  println(s"Array[Int] class:       ${arr.getClass}")     // [I (JVM's int[])
  println(s"List[Int] class:        ${lst.getClass}")      // $colon$colon

  // Proof of boxing: extract element and check its class via Any
  val fromArr: Any = arr(0)
  val fromLst: Any = lst(0)
  println(s"arr(0) as Any class:    ${fromArr.getClass}")  // java.lang.Integer (boxed)
  println(s"lst(0) as Any class:    ${fromLst.getClass}")  // java.lang.Integer (boxed)

  // Even arr(0) boxes when assigned to Any. But inside the array, they're
  // unboxed primitives. The boxing happens at the boundary.

  // Performance implication: for hot numeric loops, use Array not List.
  // Array[Int]  ~  Go's []int (contiguous, cache-friendly, no GC pressure per element)
  // List[Int]   ~  each element is a heap-allocated node pointing to a boxed Integer

  // Vector[Int] also boxes, but has good cache behavior due to 32-way branching.

  println(s"\nArray types on JVM:")
  println(s"  Array[Int]:     ${Array(1).getClass}")          // [I
  println(s"  Array[Double]:  ${Array(1.0).getClass}")        // [D
  println(s"  Array[Boolean]: ${Array(true).getClass}")       // [Z
  println(s"  Array[String]:  ${Array("a").getClass}")        // [Ljava.lang.String
  println(s"  Array[Any]:     ${Array[Any](1).getClass}")     // [Ljava.lang.Object (boxed!)

  // ========================================================================
  // 3. STRING INTERNING
  //
  // String literals are interned by the JVM — identical literals share the
  // same object reference. Dynamically constructed strings are not interned
  // unless you call .intern() explicitly.
  // ========================================================================

  println("\n=== 3. String Interning ===")

  val s1 = "webhook"
  val s2 = "webhook"
  val s3 = "web" + "hook"        // compiler folds this into "webhook" at compile time
  val s4 = s"web${"hook"}"       // runtime concatenation via StringContext

  // eq checks referential equality (same object in memory)
  // == checks structural equality (same characters)
  println(s"s1 == s2: ${s1 == s2}")              // true  (same chars)
  println(s"s1 eq s2: ${s1 eq s2}")              // true  (same interned object)
  println(s"s1 == s3: ${s1 == s3}")              // true  (same chars)
  println(s"s1 eq s3: ${s1 eq s3}")              // true  (compile-time constant folding)
  println(s"s1 == s4: ${s1 == s4}")              // true  (same chars)
  println(s"s1 eq s4: ${s1 eq s4}")              // false (s4 allocated at runtime)

  // Calling intern() explicitly forces interning
  val s5 = s4.intern()
  println(s"s1 eq s4.intern(): ${s1 eq s5}")     // true (now points to interned copy)

  // Lesson: always use == in Scala. Use eq only when you specifically need
  // reference identity (rare — caching, identity maps, debugging).

  // ========================================================================
  // 4. CASE CLASS MEMORY LAYOUT
  //
  // A case class on the JVM is a regular object with:
  //   - 16-byte object header (mark word + class pointer on 64-bit JVM)
  //   - fields laid out by the JVM (primitives unboxed, references as pointers)
  //   - padding for alignment
  //
  // Unlike Go, you cannot inspect layout with unsafe.Sizeof. But you can
  // reason about it from JVM rules.
  // ========================================================================

  println("\n=== 4. Case Class Layout ===")

  val req = HttpRequest("GET", "/api/health", 200, 1024L, true)

  // HttpRequest fields on JVM (approximate layout):
  //   [16 bytes] object header
  //   [8 bytes]  method: reference to String
  //   [8 bytes]  path: reference to String
  //   [4 bytes]  statusCode: primitive int
  //   [8 bytes]  contentLen: primitive long
  //   [1 byte]   keepAlive: primitive boolean
  //   [3 bytes]  padding (alignment to 8-byte boundary)
  //   Total: ~48 bytes (approximate — JVM can reorder fields)

  println(s"request: $req")
  println(s"  class: ${req.getClass.getName}")
  println(s"  fields: ${req.getClass.getDeclaredFields.map(_.getName).mkString(", ")}")

  // Compare with a compact struct
  val compact = Compact(1, 2)
  val mixed = Mixed(true, 42L, 7)
  println(s"\nCompact(Byte, Byte): ${compact.getClass.getDeclaredFields.length} fields")
  println(s"Mixed(Boolean, Long, Byte): ${mixed.getClass.getDeclaredFields.length} fields")

  // On JVM, even Compact(Byte, Byte) has a 16-byte object header.
  // So it's at least 16 + 1 + 1 + padding = ~24 bytes.
  // Compare to Zig/Go where Compact{u8, u8} is literally 2 bytes on the stack.
  // This is the cost of running on a managed runtime — but you get GC and safety.

  // ========================================================================
  // 5. EQUALITY: == (structural) vs eq (referential)
  //
  // Scala's == calls .equals() — structural comparison.
  // eq checks if two references point to the same object (Java's ==).
  // This is the opposite of Java's default, which catches many bugs.
  // ========================================================================

  println("\n=== 5. Equality Semantics ===")

  val cfg1 = ServiceConfig(port = 9090)
  val cfg2 = ServiceConfig(port = 9090)
  val cfg3 = cfg1

  println(s"cfg1 == cfg2: ${cfg1 == cfg2}")      // true  (same field values)
  println(s"cfg1 eq cfg2: ${cfg1 eq cfg2}")      // false (different objects)
  println(s"cfg1 eq cfg3: ${cfg1 eq cfg3}")      // true  (same reference)

  // For case classes, == checks all fields recursively.
  // For regular classes, == uses reference equality by default (unless you override equals).

  // Compare across languages:
  // Go:     == on structs is structural (if all fields are comparable)
  // TS/JS:  === on objects is referential (no structural equality built in)
  // Rust:   == requires #[derive(PartialEq)] — then structural
  // Python: == calls __eq__ — structural for dataclasses
  // Scala:  == is structural for case classes, referential for others

  // Numeric equality works across types (unlike some languages)
  println(s"\n42 == 42L: ${42 == 42L}")            // true (Int == Long)
  println(s"42.0 == 42: ${42.0 == 42}")            // true (Double == Int)

  // ========================================================================
  // 6. LAZY VAL — Deferred Initialization
  //
  // lazy val is computed on first access, then cached. Thread-safe by default.
  // Useful for expensive initialization that might not be needed.
  // Similar to Go's sync.Once or Python's functools.lru_cache for singletons.
  // ========================================================================

  println("\n=== 6. lazy val ===")

  println("before lazy val access...")

  lazy val expensiveConfig: Map[String, String] =
    println("  [computing expensive config...]")  // only runs once
    Map(
      "db_host" -> "prod-db.internal",
      "db_port" -> "5432",
      "cache_ttl" -> "300"
    )

  println("lazy val declared but not yet accessed")

  // First access triggers computation
  println(s"db_host: ${expensiveConfig("db_host")}")

  // Second access uses cached value — no recomputation
  println(s"db_port: ${expensiveConfig("db_port")}")

  // Under the hood on the JVM:
  // 1. First thread to access acquires a lock (or uses volatile + double-checked locking)
  // 2. Computes the value
  // 3. Stores the result
  // 4. All subsequent accesses read the cached value without locking
  //
  // This is safer than manual double-checked locking in Java, and cheaper than
  // synchronized on every access.

  // lazy val vs def:
  // - lazy val: computed once, cached forever
  // - def: computed every time it's called
  // - val: computed immediately at declaration

  var callCount = 0
  lazy val lazyOnce = { callCount += 1; callCount }
  def defEveryTime = { callCount += 1; callCount }

  println(s"\nlazy (1st): ${lazyOnce}")   // 1
  println(s"lazy (2nd): ${lazyOnce}")     // 1 (cached)
  println(s"def (1st):  ${defEveryTime}") // 2
  println(s"def (2nd):  ${defEveryTime}") // 3

  // ========================================================================
  // 7. VALUE CLASSES (extends AnyVal) — Zero-Cost Wrappers
  //
  // A class extending AnyVal is erased at runtime — the JVM sees
  // just the underlying primitive. Gives you type safety with no
  // allocation overhead (in most cases).
  //
  // Restrictions:
  //   - Exactly one val parameter
  //   - Must NOT be a case class in Scala 3 (use plain class)
  //   - Cannot be used in pattern matching (erased at runtime)
  //   - Boxing occurs when: stored in collections, used as Any, or
  //     when the value class type is needed for dispatch
  // ========================================================================

  println("\n=== 7. Value Classes ===")

  val latency = Millis(1500L)
  val timeout2 = Millis(3000L)
  val total = latency + timeout2

  println(s"latency: ${latency.value}ms (${latency.toSeconds}s)")
  println(s"timeout: ${timeout2.value}ms")
  println(s"total:   ${total.value}ms (${total.toSeconds}s)")

  // At runtime, latency is just a Long (8 bytes). No Millis object is allocated.
  // This is verifiable: Millis.getClass will show the companion object, not an instance.

  // RequestId wraps String (a reference type), so it provides type safety
  // but won't avoid allocation the way Millis(Long) does.
  val reqId = RequestId("req-abc-123")
  println(s"request ID: ${reqId.value}")

  // When does boxing happen?
  val asAny: Any = latency       // BOXED — needs to be wrapped in Millis object
  println(s"as Any class: ${asAny.getClass}")  // Millis (boxed)

  val inList = List(latency)     // BOXED — generic type parameter
  println(s"in List class: ${inList.head.getClass}")  // Millis (boxed)

  // Compare value classes to opaque types:
  //   - Value class: works in Scala 2 and 3, has restrictions, can still box
  //   - Opaque type:  Scala 3 only, never boxes (truly erased), more flexible
  //   - Use opaque types in new Scala 3 code. Value classes for Scala 2 compat.

  // ========================================================================
  // 8. SUMMARY — Memory Mental Model for Scala/JVM
  // ========================================================================

  println("\n=== 8. Memory Mental Model ===")
  println("""
  |  Concept               | JVM Reality                          | Go Equivalent
  |  ----------------------|--------------------------------------|-------------------------
  |  val x: Int = 42       | primitive int on stack               | var x int = 42
  |  val s: String = "hi"  | reference to interned String on heap | var s string = "hi"
  |  case class instance   | object on heap, ~16-byte header      | struct on stack (usually)
  |  Array[Int]            | int[] — unboxed primitives           | []int — contiguous
  |  List[Int]             | linked nodes, each boxes an Integer  | (no direct equivalent)
  |  lazy val              | computed once + cached (thread-safe)  | sync.Once
  |  extends AnyVal        | erased to primitive (usually)        | (no equivalent)
  |  opaque type           | fully erased at runtime              | type alias (but safer)
  """.stripMargin)

  println("=== done ===")

// --- Supporting case classes referenced in main ---
case class ServiceConfig(
  host:     String  = "localhost",
  port:     Int     = 8080,
  debug:    Boolean = false,
  maxConns: Int     = 100
)
