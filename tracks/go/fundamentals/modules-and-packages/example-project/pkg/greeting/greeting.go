// Package greeting provides greeting generation utilities.
// It lives in pkg/ to signal that it could be imported by external modules.
// Unlike internal/config, this package has no access restrictions.
package greeting

import "fmt"

// Style controls how a greeting is formatted.
type Style int

const (
	Formal Style = iota // "Good day, Alice."
	Casual              // "Hey, Alice!"
	Terse               // "Hi Alice"
)

// Greeting holds the components of a greeting message.
type Greeting struct {
	Name  string
	Style Style
}

// New creates a Greeting for the given name in the given style.
func New(name string, style Style) Greeting {
	return Greeting{Name: name, Style: style}
}

// String formats the greeting as a string.
// Implements fmt.Stringer so Greeting prints nicely with %v or %s.
func (g Greeting) String() string {
	switch g.Style {
	case Formal:
		return fmt.Sprintf("Good day, %s.", g.Name)
	case Casual:
		return fmt.Sprintf("Hey, %s!", g.Name)
	case Terse:
		return fmt.Sprintf("Hi %s", g.Name)
	default:
		return fmt.Sprintf("Hello, %s.", g.Name)
	}
}

// ForService returns a service-startup greeting combining a service name
// and a style. Useful for printing a startup banner.
func ForService(serviceName, user string, style Style) string {
	return fmt.Sprintf("[%s] %s", serviceName, New(user, style))
}
