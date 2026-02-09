# Interview Questions by Concept

Map each curriculum module to relevant interview questions, coding problems, and key talking points.

---

## Variables and Types

### Common Interview Questions

**Conceptual:**
- "Explain the difference between stack and heap allocation."
- "What is the difference between a value type and a reference type?"
- "How does Go determine whether a variable escapes to the heap?"
- "What are the advantages and disadvantages of static vs dynamic typing?"
- "Explain how string interning works."

**Language-Specific:**
- **Go**:
  - "Why can you read from a nil map but not write to it?"
  - "Explain escape analysis. When does a variable escape to the heap?"
  - "What are zero values in Go and why are they useful?"
  - "What's the difference between `var x int` and `x := 0`?"
  - "Why doesn't Go have implicit type conversions?"
- **TypeScript**: "What's the difference between `interface` and `type`?"
- **Rust**: "Explain ownership and borrowing rules."
- **Python**: "What's the difference between `is` and `==`?"
- **Java**: "Explain autoboxing and unboxing. What are the performance implications?"
- **C#**: "What's the difference between `struct` and `class`?"

**Debugging/Code Review:**
- "Why might this code panic/throw at runtime?" (nil pointer, type assertion, overflow)
- "How would you refactor this to be type-safe?"
- "What's the memory footprint of this struct?"

### Related LeetCode Problems

- N/A (too fundamental for LeetCode)

### Key Talking Points

When discussing variables and types in interviews:

1. **Memory Management**: Show understanding of stack vs heap, when allocations happen, GC implications
   - **Go**: Mention escape analysis — compiler decides stack vs heap allocation automatically
2. **Type Safety**: Discuss tradeoffs of static typing (safety, tooling) vs dynamic (flexibility)
   - **Go**: Strong typing with no implicit conversions (even `int` vs `int64`)
3. **Zero Values**: Language-specific defaults and their implications
   - **Go**: All types have usable zero values (no uninitialized variables). Zero slice is safe, zero map is read-only.
4. **Performance**: How type choice affects memory layout and performance
   - **Go**: Struct layout in memory is predictable. String is a 16-byte header (pointer + length).
5. **Null Safety**: Different approaches (Rust's Option, TypeScript strict mode, Go's explicit nil)
   - **Go**: Explicit `nil` for pointers/slices/maps/channels, but no built-in `Option` type

### Related Modules

- [[fundamentals/variables-and-types]]
- [[fundamentals/memory-and-ownership]]

---

## Hash Maps / Dictionaries

### Common Interview Questions

**Conceptual:**
- "How does a hash map work internally?"
- "What is a hash collision and how do hash maps handle it?"
- "What makes a good hash function?"
- "What's the time complexity of hash map operations?"
- "When would you use a hash map vs an array?"

**Language-Specific:**
- **Go**: "Why do you need to use `make()` for maps?"
- **TypeScript**: "What's the difference between `Map` and `{}`?"
- **Rust**: "What makes a type `Hash`-able?"
- **Python**: "What types can be dictionary keys?"
- **Java**: "What's the difference between `HashMap` and `Hashtable`?"
- **C#**: "What's the difference between `Dictionary` and `Hashtable`?"

**Design:**
- "Design a cache with O(1) get and put operations" → LRU Cache
- "Design a data structure that supports insert, delete, and getRandom in O(1)" → LeetCode 380

### LeetCode Problems

| Difficulty | Problems | Concepts Tested |
|-----------|----------|-----------------|
| Easy | [LC 1: Two Sum](https://leetcode.com/problems/two-sum/) | Hash map for O(1) lookup |
| Easy | [LC 383: Ransom Note](https://leetcode.com/problems/ransom-note/) | Character frequency counting |
| Medium | [LC 49: Group Anagrams](https://leetcode.com/problems/group-anagrams/) | Hash map with custom keys |
| Medium | [LC 146: LRU Cache](https://leetcode.com/problems/lru-cache/) | Hash map + doubly linked list |
| Medium | [LC 380: Insert Delete GetRandom O(1)](https://leetcode.com/problems/insert-delete-getrandom-o1/) | Hash map + array |

### Key Talking Points

1. **Time Complexity**: Average O(1) insert/lookup, worst case O(n) with collisions
2. **Hash Functions**: Good distribution, deterministic, fast to compute
3. **Collision Resolution**: Chaining (linked lists) vs open addressing (probing)
4. **Load Factor**: When to resize (typically 0.7-0.75)
5. **Key Requirements**: Must be hashable and immutable (in most languages)

### Related Modules

- [[fundamentals/variables-and-types]]
- [[fundamentals/maps-and-sets]]
- [[dsa/arrays-and-hashing]]

---

## Error Handling

### Common Interview Questions

**Conceptual:**
- "What's the difference between exceptions and error values?"
- "When should you use panic/throw vs returning an error?"
- "How do you handle errors in asynchronous code?"
- "What are the tradeoffs of checked vs unchecked exceptions?"

**Language-Specific:**
- **Go**: "Explain Go's error handling philosophy. Why no exceptions?"
- **TypeScript**: "How do you handle errors in async/await?"
- **Rust**: "Explain `Result<T, E>` and when to use `?`"
- **Python**: "What's the difference between `except Exception` and `except BaseException`?"
- **Java**: "What's the difference between checked and unchecked exceptions?"
- **C#**: "When should you catch and re-throw exceptions?"

**Design:**
- "Design an error handling strategy for a microservices architecture"
- "How would you implement retry logic with exponential backoff?"

### Related Modules

- [[fundamentals/error-handling]]
- [[patterns/circuit-breaker]]

---

## Concurrency

### Common Interview Questions

**Conceptual:**
- "What's the difference between concurrency and parallelism?"
- "Explain race conditions and how to prevent them."
- "What is a deadlock? How do you detect and prevent it?"
- "What are the tradeoffs of threads vs async/await?"

**Language-Specific:**
- **Go**: "Explain goroutines and channels. How do they differ from threads?"
- **TypeScript**: "How does the event loop work in Node.js?"
- **Rust**: "Explain Send and Sync traits."
- **Python**: "What is the GIL and how does it affect concurrency?"
- **Java**: "Explain the difference between `synchronized` and `ReentrantLock`."
- **C#**: "What's the difference between `Task` and `Thread`?"

**Design:**
- "Design a rate limiter" (requires synchronization)
- "Design a thread-safe cache"

### LeetCode Problems

| Difficulty | Problems |
|-----------|----------|
| Medium | [LC 1114: Print in Order](https://leetcode.com/problems/print-in-order/) |
| Medium | [LC 1115: Print FooBar Alternately](https://leetcode.com/problems/print-foobar-alternately/) |
| Medium | [LC 1116: Print Zero Even Odd](https://leetcode.com/problems/print-zero-even-odd/) |

### Related Modules

- [[fundamentals/concurrency]]
- [[patterns/pub-sub]]
- [[system-design/rate-limiting]]

---

## To Be Added

This file will grow as modules are completed. Next sections to add:

- Functions and Closures
- Interfaces and Traits
- Generics
- Design Patterns (Strategy, Observer, Factory, etc.)
- Data Structures (Arrays, Linked Lists, Trees, Graphs)
- Algorithms (Sorting, Searching, DP, etc.)
- System Design topics

---

## Interview Prep Strategy

1. **Master fundamentals first**: Variables, types, memory management
2. **Understand language idioms**: Each language has preferred patterns
3. **Practice explaining tradeoffs**: "X is better for Y, but Z is better for A because..."
4. **Connect to real-world usage**: "I've seen this pattern in [project/library]"
5. **Be honest about gaps**: "I haven't used feature X but here's my understanding..."

## Company-Specific Patterns

See [[interview-prep/by-company]] for company-specific interview patterns and expectations.
