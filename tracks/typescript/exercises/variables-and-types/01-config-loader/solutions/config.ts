export interface Config {
  host: string;
  port: number;
  timeout: number;
  maxConnections: number;
  debug: boolean;
}

/**
 * Loads configuration from environment variables.
 * Returns Config with defaults for missing values, or throws for invalid values.
 *
 * This is the reference solution using explicit parsing with helper functions.
 */
export function loadConfig(env: Record<string, string | undefined>): Config {
  return {
    host: env.HOST ?? 'localhost',
    port: parsePort(env.PORT),
    timeout: parseTimeout(env.TIMEOUT),
    maxConnections: parseMaxConnections(env.MAX_CONNECTIONS),
    debug: parseBoolean(env.DEBUG),
  };
}

function parsePort(value: string | undefined): number {
  if (value === undefined) {
    return 8080; // default
  }

  const port = parseInt(value, 10);
  if (isNaN(port)) {
    throw new Error(`Invalid port: "${value}" is not a number`);
  }
  if (port < 1 || port > 65535) {
    throw new Error(`Port ${port} out of valid range (1-65535)`);
  }

  return port;
}

function parseTimeout(value: string | undefined): number {
  if (value === undefined) {
    return 30000; // default (30 seconds)
  }

  const timeout = parseInt(value, 10);
  if (isNaN(timeout)) {
    throw new Error(`Invalid timeout: "${value}" is not a number`);
  }
  if (timeout <= 0) {
    throw new Error(`Timeout must be positive, got ${timeout}`);
  }

  return timeout;
}

function parseMaxConnections(value: string | undefined): number {
  if (value === undefined) {
    return 100; // default
  }

  const maxConn = parseInt(value, 10);
  if (isNaN(maxConn)) {
    throw new Error(`Invalid max_connections: "${value}" is not a number`);
  }
  if (maxConn < 1) {
    throw new Error(`max_connections must be positive, got ${maxConn}`);
  }

  return maxConn;
}

function parseBoolean(value: string | undefined): boolean {
  if (value === undefined) {
    return false; // default
  }

  // Accept common boolean representations
  const normalized = value.toLowerCase();
  if (normalized === 'true' || normalized === '1' || normalized === 'yes') {
    return true;
  }
  if (normalized === 'false' || normalized === '0' || normalized === 'no') {
    return false;
  }

  throw new Error(`Invalid boolean value: "${value}"`);
}
