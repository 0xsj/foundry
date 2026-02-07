// Config holds typed service configuration.
// Note: TypeScript types are erased at runtime — the raw input
// is still a Record<string, string> with no compile-time safety.
// Your parsing logic IS the runtime type safety.
interface Config {
  host: string;
  port: number;
  maxRetries: number;
  timeout: number;
  debug: boolean;
  serviceName: string;
}

// Return a Config with application-level defaults.
function defaults(): Config {
  // TODO: implement
  throw new Error("not implemented");
}

// Parse raw string key-value pairs into a typed Config.
// Missing keys should fall back to defaults.
// Invalid values should throw an Error.
function loadConfig(raw: Record<string, string>): Config {
  // TODO: implement
  // Hint: parseInt, parseFloat, and what counts as a valid bool?
  throw new Error("not implemented");
}

export { Config, defaults, loadConfig };
