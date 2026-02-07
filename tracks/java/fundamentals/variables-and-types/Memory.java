// ============================================================================
// Memory in Java
// ============================================================================
// Compile and run: javac Memory.java && java Memory
// ============================================================================

public class Memory {

    // Class-level field — gets a default value (zero value)
    static int classField;
    static String classRef;

    public static void main(String[] args) {

        // ====================================================================
        // 1. PRIMITIVE SIZES
        // Java doesn't expose addresses directly, but you know the sizes.
        // ====================================================================

        System.out.println("=== Primitive Sizes ===");
        System.out.println("byte:    " + Byte.BYTES + " byte");
        System.out.println("short:   " + Short.BYTES + " bytes");
        System.out.println("int:     " + Integer.BYTES + " bytes");
        System.out.println("long:    " + Long.BYTES + " bytes");
        System.out.println("float:   " + Float.BYTES + " bytes");
        System.out.println("double:  " + Double.BYTES + " bytes");
        System.out.println("char:    " + Character.BYTES + " bytes");
        // Q: boolean size is JVM-dependent (usually 1 byte, sometimes 4).
        //    Why isn't it standardized?
        //    (answer here)

        // ====================================================================
        // 2. DEFAULT VALUES (CLASS FIELDS vs LOCAL)
        // ====================================================================

        System.out.println("\n=== Default Values ===");
        System.out.println("classField (int): " + classField);    // 0
        System.out.println("classRef (String): " + classRef);      // null

        // int localVar;
        // System.out.println(localVar); // uncomment — compile error
        // Q: Why does Java treat class fields and local variables differently?
        //    How does this compare to Go's approach?
        //    (answer here)

        // ====================================================================
        // 3. REFERENCE EQUALITY vs VALUE EQUALITY
        // ====================================================================

        String a = "hello";
        String b = "hello";
        String c = new String("hello");

        System.out.println("\n=== String Identity ===");
        System.out.println("a == b (interned):     " + (a == b));
        System.out.println("a == c (new):          " + (a == c));
        System.out.println("a.equals(c) (content): " + a.equals(c));
        // Q: a and b are the same object (string pool). c is a different object.
        //    == compares references, .equals() compares content.
        //    How does this compare to Python's `is` vs `==`?
        //    (answer here)

        // ====================================================================
        // 4. AUTOBOXING AND INTEGER CACHE
        // ====================================================================

        Integer x = 127;
        Integer y = 127;
        Integer p = 128;
        Integer q = 128;

        System.out.println("\n=== Integer Cache ===");
        System.out.println("127 == 127 (Integer): " + (x == y));
        System.out.println("128 == 128 (Integer): " + (p == q));
        System.out.println("128.equals(128):      " + p.equals(q));
        // Q: Java caches Integer objects for -128 to 127.
        //    This is almost identical to Python's small int cache.
        //    Why do both languages do this?
        //    (answer here)

        // ====================================================================
        // 5. PASS BY VALUE (of references)
        // Java is ALWAYS pass by value. But for objects, the "value" is the reference.
        // ====================================================================

        int[] arr = {1, 2, 3};
        modifyArray(arr);

        System.out.println("\n=== Pass by Value of Reference ===");
        System.out.print("arr after modifyArray: ");
        for (int val : arr) System.out.print(val + " ");
        System.out.println();
        // Q: The array was modified inside the method. But Java is "pass by value".
        //    What exactly was copied — the array, or the reference to it?
        //    (answer here)

        int num = 42;
        modifyPrimitive(num);
        System.out.println("num after modifyPrimitive: " + num);
        // Q: num is unchanged. Why? How does this compare to Go's value semantics?
        //    (answer here)

        // ====================================================================
        // 6. FINAL DOESN'T MEAN IMMUTABLE
        // ====================================================================

        final int[] finalArr = {1, 2, 3};
        finalArr[0] = 999;          // legal — contents can change
        // finalArr = new int[]{4, 5}; // illegal — can't reassign reference

        System.out.println("\n=== final ===");
        System.out.print("finalArr: ");
        for (int val : finalArr) System.out.print(val + " ");
        System.out.println();
        // Q: final prevents reassignment, not mutation.
        //    This is exactly like const in TypeScript and let in Rust.
        //    What would you need for true immutability?
        //    (answer here)

        // ====================================================================
        // 7. NULL — THE BILLION DOLLAR MISTAKE
        // ====================================================================

        String name = null;
        // System.out.println(name.length()); // NullPointerException at runtime

        System.out.println("\n=== Null Safety ===");
        System.out.println("name is null: " + (name == null));
        // Q: Java has no compile-time null safety (without annotations).
        //    How does each other language handle "no value"?
        //    Go: nil (zero value for pointers/slices/maps)
        //    Rust: Option<T> (must handle explicitly)
        //    TypeScript: strictNullChecks
        //    Python: None (duck typed, no enforcement)
        //    Which approach do you think is best? Why?
        //    (answer here)
    }

    static void modifyArray(int[] arr) {
        arr[0] = 999;
    }

    static void modifyPrimitive(int x) {
        x = 999; // modifies the local copy only
    }
}
