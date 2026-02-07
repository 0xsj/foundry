// ============================================================================
// Variables and Types — Java
// ============================================================================
// Fill in each section. Compile and run with:
//   javac Variables.java && java Variables
// Or just save and ask for a review.
// ============================================================================

public class Variables {
    public static void main(String[] args) {

        // ====================================================================
        // 1. PRIMITIVE TYPES
        // Java has 8 primitives. Declare one of each that you'll commonly use:
        // ====================================================================
        // a) int

        // b) double

        // c) boolean

        // d) char (single character — uses single quotes)

        // e) long (for large numbers — suffix with L)

        // ====================================================================
        // 2. REFERENCE TYPES (WRAPPER CLASSES)
        // Every primitive has a wrapper: int -> Integer, double -> Double, etc.
        // These are objects, not primitives. They can be null.
        // ====================================================================
        // a) Integer (can be null, unlike int)

        // b) String (always a reference type — no primitive for strings)

        // c) Boolean

        // ====================================================================
        // 3. VAR (TYPE INFERENCE)
        // Since Java 10, `var` lets the compiler infer the type.
        // Only works for local variables.
        // ====================================================================
        // a) var name = "redis";

        // b) var port = 6379; — what type does Java infer?

        // c) can you use var without an initializer? (try it, answer in comment)

        // ====================================================================
        // 4. FINAL (CONSTANTS)
        // `final` prevents reassignment — similar to const in other languages.
        // ====================================================================
        // a) final int maxRetries = 3;

        // b) try reassigning it — what error do you get?

        // c) final String name = "redis"; then try name = "kafka";

        // ====================================================================
        // 5. DEFAULT VALUES vs COMPILE ERRORS
        // Class-level fields get defaults (0, false, null).
        // Local variables do NOT — you must initialize before use.
        // ====================================================================
        // a) declare a local int without initializing, then try to print it.
        //    what happens?

        // b) how does this compare to Go's zero values? (answer in a comment)

        // ====================================================================
        // 6. PRINT EVERYTHING
        // Use System.out.println or System.out.printf.
        // To check types: use .getClass().getSimpleName() on wrapper types.
        // For primitives, you can cast to Object first or just state the type.
        // Example: System.out.printf("port = %d (int)%n", port);
        // ====================================================================

        System.out.println("done");
    }
}
