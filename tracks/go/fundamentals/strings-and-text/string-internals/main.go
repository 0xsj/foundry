// String internals — string vs []byte vs []rune, UTF-8 mechanics, iteration.
//
// Run with: go run ./string-internals/
package main

import (
	"fmt"
	"unicode"
	"unicode/utf8"
)

func main() {
	stringVsByteVsRune()
	utf8Mechanics()
	iterationPatterns()
	indexingGotcha()
	conversionCosts()
}

// ============================================================================
// string vs []byte vs []rune
// ============================================================================

func stringVsByteVsRune() {
	fmt.Println("=== string vs []byte vs []rune ===")

	s := "café"

	// string: immutable, len = BYTES
	fmt.Printf("string %q\n", s)
	fmt.Printf("  len(s) = %d  ← bytes, not characters!\n", len(s))
	// s[0] = 'C'  // compile error: cannot assign to s[0] (string is immutable)

	// []byte: mutable, len = BYTES
	b := []byte(s) // alloc: copies all bytes
	fmt.Printf("[]byte %v\n", b)
	fmt.Printf("  len(b) = %d  ← still bytes\n", len(b))
	b[0] = 'C'                   // mutable — modifying 'c' to 'C'
	fmt.Printf("  after b[0]='C': %q\n", string(b))

	// []rune: mutable, len = CHARACTERS (Unicode code points)
	r := []rune(s) // alloc: decodes UTF-8 into int32 values
	fmt.Printf("[]rune %v\n", r)
	fmt.Printf("  len(r) = %d  ← character count\n", len(r))
	r[3] = 'e'                    // change é (U+00E9) to e (U+0065)
	fmt.Printf("  after r[3]='e': %q\n", string(r))

	// Summary: use utf8.RuneCountInString for character count without allocating
	charCount := utf8.RuneCountInString(s)
	fmt.Printf("  utf8.RuneCountInString(%q) = %d\n", s, charCount)

	fmt.Println()
}

// ============================================================================
// UTF-8 Encoding Mechanics
// ============================================================================

func utf8Mechanics() {
	fmt.Println("=== UTF-8 Encoding ===")

	// Examine the bytes of characters with different byte widths
	examples := []struct {
		char string
		desc string
	}{
		{"A", "ASCII (1 byte)"},
		{"é", "Latin extended (2 bytes)"},
		{"€", "Euro sign (3 bytes)"},
		{"🎉", "Emoji (4 bytes)"},
		{"世", "CJK ideograph (3 bytes)"},
	}

	for _, ex := range examples {
		r, size := utf8.DecodeRuneInString(ex.char)
		fmt.Printf("  %q  U+%04X  %d byte(s)  (%s)\n", ex.char, r, size, ex.desc)
	}

	// A realistic string showing mixed byte widths
	s := "A€🎉"
	fmt.Printf("\n%q — total bytes: %d, total runes: %d\n",
		s, len(s), utf8.RuneCountInString(s))

	// Dump each byte as hex to see the encoding
	fmt.Printf("bytes: ")
	for i := 0; i < len(s); i++ {
		fmt.Printf("%02X ", s[i])
	}
	fmt.Println()

	// utf8.ValidString: check if a byte sequence is valid UTF-8
	fmt.Printf("\nutf8.ValidString(%q) = %v\n", s, utf8.ValidString(s))

	// A byte sequence that is NOT valid UTF-8
	invalid := string([]byte{0xFF, 0xFE})
	fmt.Printf("utf8.ValidString(%v) = %v\n", []byte(invalid), utf8.ValidString(invalid))

	fmt.Println()
}

// ============================================================================
// Iteration Patterns
// ============================================================================

func iterationPatterns() {
	fmt.Println("=== Iteration Patterns ===")

	s := "café"

	// for range: decodes runes — correct for character processing
	fmt.Println("for range (rune iteration):")
	for i, r := range s {
		fmt.Printf("  byte offset %d: %q (U+%04X, %d bytes)\n",
			i, r, r, utf8.RuneLen(r))
	}
	// NOTE: i is byte offset, not character index.
	// 'é' starts at byte 3 and is 2 bytes, so the next char would be at byte 5.

	// for i loop: raw byte iteration
	fmt.Println("\nfor i (byte iteration):")
	for i := 0; i < len(s); i++ {
		fmt.Printf("  s[%d] = 0x%02X\n", i, s[i])
	}

	// Converting to []rune for character-indexed access
	fmt.Println("\n[]rune (character indexing):")
	runes := []rune(s)
	for i, r := range runes {
		fmt.Printf("  rune[%d] = %q (U+%04X)\n", i, r, r)
	}

	// Practical example: get the 3rd character (index 2)
	char := []rune(s)[2] // 'f' — correct
	fmt.Printf("\ncharacter at index 2: %q\n", char)

	// Wrong approach: byte index (would give garbage for multibyte chars)
	byteAtIdx2 := s[2] // 'f' — works here only because f is ASCII
	fmt.Printf("byte at index 2:      0x%02X (%q)\n", byteAtIdx2, byteAtIdx2)

	fmt.Println()
}

// ============================================================================
// The Indexing Gotcha
// ============================================================================

func indexingGotcha() {
	fmt.Println("=== Indexing Gotcha ===")

	s := "naïve" // 'ï' is U+00EF, 2 bytes

	fmt.Printf("string: %q, len=%d bytes, %d chars\n",
		s, len(s), utf8.RuneCountInString(s))

	// Demonstrate the bug: naive byte-based "uppercase first char"
	// This works for ASCII but corrupts multibyte strings
	fmt.Println("\nBUGGY approach (byte indexing):")
	if s[0] >= 'a' && s[0] <= 'z' {
		buggy := string(s[0]-32) + s[1:]
		fmt.Printf("  buggy result: %q — OK here because n is ASCII\n", buggy)
	}

	// Demonstrate with a non-ASCII first character
	emoji := "🎉party"
	fmt.Printf("\n%q — first byte: 0x%02X (not a character!)\n", emoji, emoji[0])

	// CORRECT approach: use []rune
	fmt.Println("\nCORRECT approach (rune-aware):")
	correct := capitalizeFirst(s)
	fmt.Printf("  capitalizeFirst(%q) = %q\n", s, correct)
	correct = capitalizeFirst("élan")
	fmt.Printf("  capitalizeFirst(%q) = %q\n", "élan", correct)

	fmt.Println()
}

// capitalizeFirst correctly uppercases the first character of any UTF-8 string.
func capitalizeFirst(s string) string {
	if s == "" {
		return s
	}
	r := []rune(s)
	r[0] = unicode.ToUpper(r[0])
	return string(r)
}

// ============================================================================
// Conversion Costs
// ============================================================================

func conversionCosts() {
	fmt.Println("=== Conversion Costs ===")

	s := "hello, world"

	// Every conversion allocates — avoid in hot paths
	b1 := []byte(s)  // alloc: copies all bytes
	b2 := []byte(s)  // alloc: separate copy
	_ = b1
	_ = b2

	// Converting []byte back to string also allocates
	// (unless the compiler can prove the []byte won't be modified)
	result := string(b1) // alloc in general case
	fmt.Printf("round-trip: %q\n", result)

	// Exception: the compiler eliminates the allocation for string(b) used
	// directly in a comparison or map lookup
	key := []byte("user:42")
	m := map[string]int{"user:42": 100}
	// This does NOT allocate — compiler optimization
	if val, ok := m[string(key)]; ok {
		fmt.Printf("map lookup with string([]byte): %d (no alloc)\n", val)
	}

	// Best practice: work in []byte when doing I/O, convert once at the end
	fmt.Println("\nBest practice: convert once at the end")
	data := processBytes([]byte("input data"))
	fmt.Printf("processed: %q\n", string(data)) // single conversion

	fmt.Println()
}

// processBytes simulates a function that works with raw bytes throughout,
// avoiding intermediate string allocations.
func processBytes(input []byte) []byte {
	// Work entirely in []byte — no string conversions needed
	result := make([]byte, len(input))
	for i, b := range input {
		if b >= 'a' && b <= 'z' {
			result[i] = b - 32 // uppercase (ASCII only — for demo)
		} else {
			result[i] = b
		}
	}
	return result
}
