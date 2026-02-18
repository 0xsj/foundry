// Package main is the entry point for the example project.
// It wires together internal packages (config) and reusable packages (greeting),
// demonstrating the difference between internal/ and pkg/ visibility.
//
// Run with:
//
//	go run .
//
// Or with environment variables:
//
//	APP_ENV=production APP_NAME=myapp PORT=9000 go run .
package main

import (
	"fmt"
	"log"
	"os"

	// Internal package — only this module can import it.
	// The import path is: <module-path>/<relative-path>
	"github.com/foundry/example/internal/config"

	// pkg/ package — could be imported by external modules.
	"github.com/foundry/example/pkg/greeting"
)

func main() {
	// Load configuration from environment.
	// This calls into internal/config — a package only we control.
	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "config error: %v\n", err)
		os.Exit(1)
	}

	// Print startup banner using the greeting package.
	// greeting.Formal, greeting.Casual, greeting.Terse are exported constants.
	banner := greeting.ForService(cfg.Name, "world", greeting.Casual)
	log.Println(banner)

	// Show config details.
	log.Printf("env=%s  addr=%s  debug=%v", cfg.Env, cfg.Addr(), cfg.IsDebug())

	// The config.AppConfig type is exported, but config.defaultPort is not.
	// You can declare a variable of the type, but can't access unexported fields.
	var _ *config.AppConfig = cfg // this compiles fine
	// cfg.debug = true           // this would be a compile error

	log.Println("startup complete — press Ctrl+C to stop")
}
