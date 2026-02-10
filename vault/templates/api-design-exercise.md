# API Design Exercise Template

## Overview

**Scenario:** [Realistic business context — e.g., "You're building a notification system for a SaaS platform"]

**Your Task:** Design a clean, ergonomic, and maintainable API/interface for [the problem domain].

**Skills Practiced:**
- Interface design
- Naming conventions
- Error handling design
- Balancing simplicity vs flexibility
- API evolution and versioning

---

## Requirements

### Functional Requirements
- [ ] [Requirement 1: core functionality]
- [ ] [Requirement 2: another feature]
- [ ] [Requirement 3: edge case handling]
- [ ] [Requirement 4: configuration]

### Non-Functional Requirements
- [ ] **Ergonomics:** API should be intuitive and hard to misuse
- [ ] **Performance:** [Any performance constraints]
- [ ] **Extensibility:** Should support [future enhancement]
- [ ] **Error Handling:** Clear error messages and recovery options

### Constraints
- [Any technical constraints — e.g., "Must work with existing auth system"]
- [Any design constraints — e.g., "Cannot use global state"]

---

## Your Design Task

Create the public interface for this system. Focus on:

1. **Naming** — Function/method names, parameter names, types
2. **Structure** — Objects, modules, organization
3. **Error Handling** — How are errors represented and handled?
4. **Configuration** — How is the system configured?
5. **Usage Patterns** — What do common use cases look like?

---

## Starter Template

```[language]
// Define your API here
// Focus on the PUBLIC interface, not implementation

// Example structure (adapt to your language):
// - Main interface/trait/class
// - Configuration types
// - Error types
// - Builder pattern (if needed)
// - Common operations
```

---

## Design Constraints Checklist

As you design, consider:

### Usability
- [ ] Can users accomplish common tasks easily?
- [ ] Are the defaults sensible?
- [ ] Is the API hard to misuse?
- [ ] Do functions/methods do what their names suggest?

### Clarity
- [ ] Are names descriptive?
- [ ] Is the structure intuitive?
- [ ] Are errors informative?

### Flexibility
- [ ] Can the API evolve without breaking changes?
- [ ] Are extension points clear?
- [ ] Can users customize behavior when needed?

### Safety
- [ ] Are error conditions represented in types?
- [ ] Can invalid states be represented?
- [ ] Are resources cleaned up properly?

---

## Example Usage Scenarios

Write example code showing how your API would be used:

```[language]
// Scenario 1: Basic usage
// [Show the simplest possible usage]

// Scenario 2: Advanced configuration
// [Show how to customize behavior]

// Scenario 3: Error handling
// [Show how errors are handled]

// Scenario 4: [Specific requirement]
// [Show how this requirement is addressed]
```

---

## Design Alternatives

Document at least 2 different approaches:

### Approach 1: [Name of approach — e.g., "Fluent Builder API"]
**Pros:**
- [Advantage 1]
- [Advantage 2]

**Cons:**
- [Disadvantage 1]
- [Disadvantage 2]

**Example:**
```[language]
// Code example
```

### Approach 2: [Name of approach — e.g., "Functional Options"]
**Pros:**
- [Advantage 1]

**Cons:**
- [Disadvantage 1]

**Example:**
```[language]
// Code example
```

### Your Choice: [Which approach and why?]

---

## Evolution Planning

How would your API handle:

1. **Adding a new feature** — [e.g., "Support SMS in addition to email"]
   - How would you add this without breaking existing code?

2. **Deprecating functionality** — [e.g., "Old auth method is deprecated"]
   - How would you phase this out?

3. **Performance optimization** — [e.g., "Add batching support"]
   - How would you add this as an opt-in feature?

---

## Hints (Progressive)

<details>
<summary>Hint 1: Common API patterns in [language]</summary>

- [Pattern 1: e.g., Builder pattern]
- [Pattern 2: e.g., Functional options]
- [Pattern 3: e.g., Fluent interfaces]

</details>

<details>
<summary>Hint 2: Error handling strategies</summary>

- [Language-specific error handling patterns]
- [When to use Result vs exceptions vs error codes]

</details>

<details>
<summary>Hint 3: Look at similar libraries</summary>

- [Library 1]: [What they did well]
- [Library 2]: [What they did well]
- [Library 3]: [What to avoid]

</details>

---

## Success Criteria

Your design is successful if:
- [ ] Common use cases are simple (< 5 lines of code)
- [ ] Advanced use cases are possible
- [ ] Errors are clear and actionable
- [ ] The API is difficult to misuse
- [ ] The design can evolve without breaking changes
- [ ] Naming is consistent and intuitive

---

## Notes

**Difficulty:** [Easy/Medium/Hard]

**Estimated Time:** [45-90 min]

**Related Concepts:** [[interface-design]], [[error-handling]], [[api-evolution]]

**Real-World Examples:** [Links to similar APIs in popular libraries]
