# Cross-Language Rosetta Stone

Quick reference for common operations. Click language headers for official docs.

---

## File I/O

### Read Entire File

| [Go](https://pkg.go.dev/os) | [TypeScript](https://nodejs.org/api/fs.html) | [Rust](https://doc.rust-lang.org/std/fs/) | [Python](https://docs.python.org/3/library/functions.html#open) | [Java](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/nio/file/Files.html) |
|---|---|---|---|---|
| `os.ReadFile(path)` | `fs.readFileSync(path, 'utf8')` | `fs::read_to_string(path)?` | `open(path).read()` | `Files.readString(Path.of(path))` |
| Returns `([]byte, error)` | Returns `string \| throws` | Returns `Result<String>` | Returns `str \| raises` | Returns `String \| throws` |

### Write Entire File

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `os.WriteFile(path, data, 0644)` | `fs.writeFileSync(path, data)` | `fs::write(path, data)?` | `Path(path).write_text(data)` | `Files.writeString(path, data)` |

### Append to File

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `f, _ := os.OpenFile(path, os.O_APPEND\|os.O_CREATE, 0644)` | `fs.appendFileSync(path, data)` | `OpenOptions::new().append(true).open(path)?` | `open(path, 'a').write(data)` | `Files.write(path, data, APPEND)` |

### Check if File Exists

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `_, err := os.Stat(path); err == nil` | `fs.existsSync(path)` | `Path::new(path).exists()` | `Path(path).exists()` | `Files.exists(Path.of(path))` |

---

## HTTP Requests

### GET Request

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `http.Get(url)` | `fetch(url)` | `reqwest::get(url).await?` | `requests.get(url)` | `HttpClient.send(request)` |
| Returns `(*Response, error)` | Returns `Promise<Response>` | Returns `Result<Response>` | Returns `Response \| raises` | Returns `HttpResponse<String>` |

### POST Request with JSON

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `http.Post(url, "application/json", body)` | `fetch(url, {method: 'POST', body: JSON.stringify(data)})` | `client.post(url).json(&data).send().await?` | `requests.post(url, json=data)` | `HttpRequest.newBuilder().POST(body).build()` |

---

## JSON

### Parse JSON String

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `json.Unmarshal([]byte(str), &v)` | `JSON.parse(str)` | `serde_json::from_str(str)?` | `json.loads(str)` | `new ObjectMapper().readValue(str, Class)` |

### Serialize to JSON String

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `json.Marshal(v)` | `JSON.stringify(obj)` | `serde_json::to_string(&v)?` | `json.dumps(obj)` | `new ObjectMapper().writeValueAsString(obj)` |

---

## String Manipulation

### Split String

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strings.Split(s, ",")` | `s.split(",")` | `s.split(",").collect()` | `s.split(",")` | `s.split(",")` |

### Join Array of Strings

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strings.Join(arr, ",")` | `arr.join(",")` | `arr.join(",")` | `",".join(arr)` | `String.join(",", arr)` |

### Trim Whitespace

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strings.TrimSpace(s)` | `s.trim()` | `s.trim()` | `s.strip()` | `s.trim()` |

### Replace All Occurrences

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strings.ReplaceAll(s, "old", "new")` | `s.replaceAll("old", "new")` | `s.replace("old", "new")` | `s.replace("old", "new")` | `s.replace("old", "new")` |

---

## Collections

### Map Over Array

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `for _, v := range arr { ... }` (manual) | `arr.map(fn)` | `arr.iter().map(fn).collect()` | `[fn(x) for x in arr]` | `arr.stream().map(fn).toList()` |

### Filter Array

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| Manual loop | `arr.filter(fn)` | `arr.iter().filter(fn).collect()` | `[x for x in arr if fn(x)]` | `arr.stream().filter(fn).toList()` |

### Reduce/Fold Array

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| Manual loop | `arr.reduce(fn, init)` | `arr.iter().fold(init, fn)` | `functools.reduce(fn, arr, init)` | `arr.stream().reduce(init, fn)` |

### Sort Array

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `sort.Ints(arr)` (in-place) | `arr.sort()` (in-place) | `arr.sort()` (in-place) | `sorted(arr)` (new) | `arr.sort(Comparator)` (in-place) |

---

## Error Handling

### Try-Catch Equivalent

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `if err != nil { ... }` | `try { ... } catch (e) { ... }` | `match result { Ok(v) => ..., Err(e) => ... }` | `try: ... except E as e: ...` | `try { ... } catch (E e) { ... }` |

### Throw/Panic

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `panic("message")` | `throw new Error("message")` | `panic!("message")` | `raise Exception("message")` | `throw new Exception("message")` |

---

## Concurrency

### Start Async Task

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `go func() { ... }()` | `async function() { ... }` | `tokio::spawn(async { ... })` | `asyncio.create_task(coro)` | `CompletableFuture.runAsync(() -> ...)` |

### Wait for Multiple Tasks

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `sync.WaitGroup` | `Promise.all([...])` | `tokio::join!(...)` | `await asyncio.gather(...)` | `CompletableFuture.allOf(...)` |

---

## Testing

### Basic Test Function

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `func TestFoo(t *testing.T) { ... }` | `test('foo', () => { ... })` | `#[test] fn foo() { ... }` | `def test_foo(): ...` | `@Test void foo() { ... }` |

### Assertion

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `if got != want { t.Errorf(...) }` | `expect(got).toBe(want)` | `assert_eq!(got, want)` | `assert got == want` | `assertEquals(want, got)` |

---

## Variables and Types

### Type Declarations

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `var x int = 42` or `x := 42` | `let x: number = 42` | `let x: i32 = 42` | `x = 42` (dynamic) | `int x = 42` or `var x = 42` |

### String to Int Conversion

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strconv.Atoi(s)` returns `(int, error)` | `parseInt(s, 10)` | `s.parse::<i32>()?` | `int(s)` raises on error | `Integer.parseInt(s)` |

### Int to String Conversion

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `strconv.Itoa(n)` | `n.toString()` | `n.to_string()` | `str(n)` | `Integer.toString(n)` |

### Map/Dictionary Initialization

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `m := make(map[K]V)` (required) | `const m = new Map<K, V>()` or `{}` | `HashMap::new()` | `d = {}` or `dict()` | `new HashMap<K, V>()` |
| `m := map[string]int{"a": 1}` | `const m = {a: 1}` | `hashmap!{"a": 1}` | `d = {"a": 1}` | `Map.of("a", 1)` |

### Pointer Operations

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `p := &x` (address of) | N/A (all objects are references) | `let p = &x` (borrow) | N/A (all objects are references) | N/A (objects are references) |
| `*p` (dereference) | N/A | `*p` (dereference) | N/A | N/A |

### Type Assertion/Conversion

| Go | TypeScript | Rust | Python | Java |
|---|---|---|---|---|
| `v, ok := i.(Type)` | `v as Type` (compile-time) | `v as TargetType` | `isinstance(v, Type)` (runtime) | `(Type) v` (runtime cast) |
| `int64(x)` (explicit conversion) | `x as number` | `x as i64` | `int(x)` | `(int) x` |

---

## Notes

- **Go**: Explicit error handling, no exceptions. Escape analysis for stack/heap. Zero values always safe.
- **TypeScript**: JavaScript superset, runtime = Node.js or browser. Type system erased at runtime.
- **Rust**: Ownership system, `?` operator for error propagation. Compile-time memory safety.
- **Python**: Dynamic typing, indentation-based syntax. Everything is an object.
- **Java**: Verbose but explicit, strong ecosystem. Primitives vs objects (autoboxing).
