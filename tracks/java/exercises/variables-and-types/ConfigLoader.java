package config;

import java.util.Map;

/**
 * Typed service configuration.
 *
 * Java uses a record here — an immutable data carrier introduced in Java 16.
 * Note: Java primitives (int, double, boolean) have default zero values in
 * class fields, but records require explicit construction.
 */
public record Config(
    String host,
    int port,
    int maxRetries,
    double timeout,
    boolean debug,
    String serviceName
) {

    /** Return a Config with application-level defaults. */
    public static Config defaults() {
        // TODO: implement
        throw new UnsupportedOperationException("not implemented");
    }

    /**
     * Parse raw string key-value pairs into a typed Config.
     * Missing keys should fall back to defaults.
     * Invalid values should throw IllegalArgumentException.
     *
     * Hint: Integer.parseInt, Double.parseDouble, Boolean.parseBoolean
     * Note: Boolean.parseBoolean never throws — it returns false for anything
     * that isn't "true". Is that what you want?
     */
    public static Config load(Map<String, String> raw) {
        // TODO: implement
        throw new UnsupportedOperationException("not implemented");
    }
}
