# Data Structures & Algorithms

Cross-language DSA problems. Each problem has a brief, implementations in multiple languages, and tests.

## Structure

```
dsa/
├── problems/
│   └── <problem-name>/
│       ├── README.md         # Problem brief, constraints, examples
│       ├── go/
│       │   ├── solution.go
│       │   └── solution_test.go
│       ├── typescript/
│       │   ├── solution.ts
│       │   └── solution.test.ts
│       ├── rust/
│       │   ├── src/lib.rs
│       │   └── Cargo.toml
│       ├── python/
│       │   ├── solution.py
│       │   └── test_solution.py
│       └── java/
│           ├── Solution.java
│           └── SolutionTest.java
└── concepts/
    └── *.md                  # Theory notes (feed into vault)
```

## Problem Brief Format

Each problem README follows this structure:

- **Problem**: Clear statement of what to solve
- **Constraints**: Input bounds, edge cases
- **Examples**: Input/output pairs
- **Hints**: Progressive hints (collapsed by default)
- **Complexity Target**: Expected time/space complexity
- **Follow-Up**: Harder variations

## How To Use

1. Read the problem brief
2. Implement in your current focus language
3. Run tests to verify
4. Analyze complexity
5. Optionally implement in additional languages to compare approaches
