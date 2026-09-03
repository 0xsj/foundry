package main

import "fmt"

// Ports are the kind of thing you write as a bare number and never think about.
// Go thinks about it: this constant has no type yet.
const defaultPort = 8080

func main() {
	// Declared and not assigned. Go fills each one with its type's zero value
	// before a single line of your code runs. Not undefined. Not null.
	var host string
	var port int
	var tlsEnabled bool
	var readTimeout float64

	fmt.Printf("zero values: %q %d %t %v\n", host, port, tlsEnabled, readTimeout)

	host = "127.0.0.1"
	port = defaultPort
	tlsEnabled = true
	readTimeout = 2.5

	// Short declaration. The type is inferred from the value, not guessed.
	maxConns := 512

	// Go will not mix numeric types for you, even when the conversion is
	// obviously safe. Drop the float64(...) here and it will not compile.
	perConn := readTimeout / float64(maxConns)
	budgetMs := int(readTimeout * 1000)

	fmt.Printf("%s:%d tls=%t\n", host, port, tlsEnabled)
	fmt.Printf("per-conn %.6fs · budget %dms · conns %d\n", perConn, budgetMs, maxConns)
}
