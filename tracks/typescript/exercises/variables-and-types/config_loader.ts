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
  return {
    host: "localhost",
    port: 3000,
    maxRetries: 3,
    timeout: 10,
    debug: true,
    serviceName: 'users'
  }
}

// Parse raw string key-value pairs into a typed Config.
// Missing keys should fall back to defaults.
// Invalid values should throw an Error.
function loadConfig(raw: Record<string, string>): Config {
  // TODO: implement
  // Hint: parseInt, parseFloat, and what counts as a valid bool?
  const config = defaults()

  if (raw.host !== undefined) {
    config.host = raw.host
  }

  if (raw.port !== undefined) {
    const port = parseInt(raw.port, 10)
    if (Number.isNaN(port)) {
      throw new Error(`Invalid port: ${raw.port}`)
    }
    config.port = port;
  }

  if (raw.maxRetries !== undefined) {
    const maxRetries = parseInt(raw.maxRetries, 10);
    if (Number.isNaN(maxRetries)) {
      throw new Error(`Invalid maxRetries: ${raw.maxRetries}`);
    }
    config.maxRetries = maxRetries;
  }

  if (raw.timeout !== undefined) {
    const timeout = parseFloat(raw.timeout);
    if (Number.isNaN(timeout)) {
      throw new Error(`Invalid timeout: ${raw.timeout}`);
    }
    config.timeout = timeout;
  }

  if (raw.debug !== undefined) {
    if (raw.debug === "true") {
      config.debug = true;
    } else if (raw.debug === "false") {
      config.debug = false;
    } else {
      throw new Error(`Invalid debug value: ${raw.debug}`);
    }
  }

  if (raw.serviceName !== undefined) {
    config.serviceName = raw.serviceName;
  }

  return config
}

export { Config, defaults, loadConfig };
