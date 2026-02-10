# Example: Code Review Exercise — Authentication PR

This is an **example structure** for code review exercises. Use `vault/templates/code-review-exercise.md` as a template.

## Overview

**Scenario:** A team member has implemented JWT token validation for the authentication service.

**Your Task:** Review the proposed changes for correctness, security, performance, and best practices.

---

## Files in this directory

```
code-review/_example-pr-auth/
├── README.md              # This file — PR description and context
├── codebase/              # Existing code for context
│   ├── auth.go
│   └── middleware.go
├── proposed-changes.diff  # The diff to review
├── my-review.md           # Your review (start from template)
├── review-guide.md        # Progressive hints
└── expert-review.md       # Reference review for comparison
```

---

## Creating a New Code Review Exercise

1. **Choose a realistic PR scenario** — authentication, caching, API endpoint, refactoring
2. **Create the directory structure** above
3. **Write the PR description** in README.md (as if written by the PR author)
4. **Include context files** — enough code to understand the system
5. **Create the diff** — actual changes being proposed
6. **Write review guide** with progressive hints
7. **Write expert review** showing what an experienced developer would catch

---

## Skills Practiced

- [ ] Reading unfamiliar code
- [ ] Spotting bugs without running code
- [ ] Security awareness (authentication, validation, injection)
- [ ] Performance considerations (unnecessary allocations, N+1 queries)
- [ ] Best practices (error handling, naming, structure)
- [ ] Giving constructive feedback

---

## Tips for Creating Exercises

**Good PR topics:**
- Authentication/authorization changes
- Database query optimizations
- Error handling improvements
- API endpoint implementations
- Configuration management
- Concurrency patterns

**Intentional issues to include:**
- Obvious bugs (nil checks, off-by-one errors)
- Subtle issues (race conditions, edge cases)
- Security vulnerabilities (SQL injection, XSS, auth bypasses)
- Performance problems (N+1 queries, unnecessary allocations)
- Design concerns (tight coupling, poor abstractions)
- Style issues (inconsistent naming, unclear logic)

**Progressive hint structure:**
1. **Hint 1:** High-level areas to focus on (e.g., "Check error handling")
2. **Hint 2:** More specific (e.g., "Look at the nil check on line 42")
3. **Hint 3:** Almost reveals the issue (e.g., "What happens if token is nil?")
