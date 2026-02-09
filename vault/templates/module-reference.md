# {{language}} Reference — {{module_name}}

> Extracted from [{{official_source}}]({{source_url}})
> for the `{{module_slug}}` module. Covers: {{topic_1}}, {{topic_2}}, {{topic_3}}.

---

## {{Section_1}}

Source: [{{section_link}}]({{section_url}})

{{formal_definition}}

{{spec_details}}

```{{lang}}
// Official examples from documentation
{{code_from_spec}}
```

### {{subsection}}

{{detailed_spec_content}}

| {{reference_table}} |
|---|
| ... |

---

## {{Section_2}}

Source: [{{section_link}}]({{section_url}})

{{content}}

---

## Guidelines for Reference Content

**Purpose:**
- Authoritative reference extracted from official language documentation
- Quick lookup for exact behavior, syntax rules, and edge cases
- Complement to lesson.md (lesson teaches, reference confirms)

**Structure:**
- One section per major topic (variables, types, constants, etc.)
- Include source links to official docs at start of each section
- Use tables for type lists, sizes, ranges, and comparisons
- Code examples should be minimal and spec-focused

**Content:**
- Extract directly from official sources (Go spec, TS handbook, Rust reference)
- Preserve formal language and technical precision
- Include spec quirks and edge cases (e.g., "must be representable as int")
- Cover syntax, semantics, and constraints

**Formatting:**
- Use `---` horizontal rules between major sections
- Start each section with "Source:" link
- Use tables extensively for type info, operators, rules
- Keep code examples short (< 20 lines)
- Use blockquotes only for direct spec quotes

**What to extract:**
- Type definitions and categories
- Syntax rules and constraints
- Memory layout and representation (when specified)
- Operators and their behavior
- Edge cases and undefined behavior
- Official examples demonstrating syntax

**What to omit:**
- Tutorials and explanations (those go in lesson.md)
- Third-party blog posts (those go in vault notes)
- Implementation-specific details (unless normative)
- Lengthy prose (extract key rules and tables)

**Cross-references:**
- Link to related spec sections
- Note when behavior is implementation-defined vs specified
- Include version/date of spec if language evolves rapidly
