package pilot;

import org.apache.pekko.util.HashCode$;

/**
 * Oracle cases of slice P65 (UTIL-HASHCODE): AC-UTIL-016.15 (HashCode, HashCode.scala:33-126).
 * Registered at the END of the util-core list by {@link UtilCoreOracle}, so earlier case ids do not shift.
 */
final class HashCodeOracle {
  private HashCodeOracle() {}

  static final HashCode$ HC = HashCode$.MODULE$;

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  static String esc(String s) { return UtilCoreOracle.esc(s); }

  static String boxed(int seed, Object o) { return Integer.toString(HC.hash(seed, o)); }

  static void register() {
    final int[] seeds = {23, 0, 1, -1, Integer.MAX_VALUE, Integer.MIN_VALUE, 123456789};
    run("HC-01 boolean", () -> {
      StringBuilder b = new StringBuilder();
      for (int s : seeds) b.append(s).append(':').append(HC.hash(s, true)).append('/').append(HC.hash(s, false)).append(' ');
      return b.toString().trim();
    });
    run("HC-02 int", () -> {
      StringBuilder b = new StringBuilder();
      for (int s : seeds) for (int v : new int[] {0, 5, -5, Integer.MAX_VALUE, Integer.MIN_VALUE}) b.append(HC.hash(s, v)).append(' ');
      return b.toString().trim();
    });
    run("HC-03 char short byte", () -> {
      StringBuilder b = new StringBuilder();
      for (char c : new char[] {0, 'a', 0x7fff, 0x8000, 0xffff}) b.append(HC.hash(23, c)).append(' ');
      for (short v : new short[] {0, 1, -1, Short.MAX_VALUE, Short.MIN_VALUE}) b.append(boxed(23, v)).append(' ');
      for (byte v : new byte[] {0, 1, -1, Byte.MAX_VALUE, Byte.MIN_VALUE}) b.append(boxed(23, v)).append(' ');
      return b.toString().trim();
    });
    run("HC-04 long", () -> {
      StringBuilder b = new StringBuilder();
      for (long v : new long[] {0L, 1L, -1L, 5000000000L, Long.MAX_VALUE, Long.MIN_VALUE, 0x100000000L, 0xFFFFFFFFL})
        for (int s : new int[] {23, Integer.MIN_VALUE}) b.append(HC.hash(s, v)).append(' ');
      return b.toString().trim();
    });
    run("HC-05 float", () -> {
      StringBuilder b = new StringBuilder();
      for (float v : new float[] {0f, -0f, 1.5f, -1.5f, Float.NaN, Float.intBitsToFloat(0x7fc00001), Float.intBitsToFloat(0xffc00000), Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.MIN_VALUE, Float.MAX_VALUE, Float.MIN_NORMAL})
        b.append(HC.hash(23, v)).append(' ');
      return b.toString().trim();
    });
    run("HC-06 double", () -> {
      StringBuilder b = new StringBuilder();
      for (double v : new double[] {0d, -0d, 1.5d, -1.5d, Double.NaN, Double.longBitsToDouble(0x7ff8000000000001L), Double.longBitsToDouble(0xfff8000000000000L), Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.MIN_VALUE, Double.MAX_VALUE, Double.MIN_NORMAL})
        b.append(HC.hash(23, v)).append(' ');
      return b.toString().trim();
    });
    run("HC-07 string", () -> {
      String astral = new String(Character.toChars(0x1F600));
      StringBuilder b = new StringBuilder();
      for (String s : new String[] {"", "a", "abc", "Hello, World!", "é", "€", astral, "a" + astral + "b", "x".repeat(1000)})
        b.append('[').append(s.length() > 40 ? s.length() + "x" : esc(s)).append("]=").append(HC.hash(23, (Object) s)).append(' ');
      return b.toString().trim();
    });
    run("HC-08 arrays of primitives", () -> {
      StringBuilder b = new StringBuilder();
      b.append(boxed(23, new boolean[] {})).append(' ').append(boxed(23, new boolean[] {true, false, true})).append(' ');
      b.append(boxed(23, new char[] {'a', 0xffff})).append(' ').append(boxed(23, new short[] {1, -1, Short.MIN_VALUE})).append(' ');
      b.append(boxed(23, new int[] {})).append(' ').append(boxed(23, new int[] {1, 2})).append(' ').append(boxed(23, new int[] {Integer.MAX_VALUE, Integer.MIN_VALUE, -1})).append(' ');
      b.append(boxed(23, new long[] {5000000000L, -1L, Long.MIN_VALUE})).append(' ').append(boxed(23, new float[] {1.5f, Float.NaN, -0f})).append(' ');
      b.append(boxed(23, new double[] {1.5, Double.NaN, -0d})).append(' ').append(boxed(23, new byte[] {1, -1, Byte.MIN_VALUE})).append(' ');
      b.append(boxed(Integer.MIN_VALUE, new int[] {7, 8, 9})).append(' ').append(boxed(0, new byte[] {}));
      return b.toString();
    });
    run("HC-09 arrays of objects", () -> {
      StringBuilder b = new StringBuilder();
      b.append(boxed(23, new Object[] {})).append(' ');
      b.append(boxed(23, new Object[] {"a", 1, 2L, 1.5, true, 'c', 1.5f, (short) 3, (byte) -4})).append(' ');
      b.append(boxed(23, new Object[] {new int[] {1, 2}, new Object[] {"x", new long[] {3L}}, new Object[] {}})).append(' ');
      b.append(boxed(23, new String[] {"abc", "", "é"})).append(' ');
      b.append(boxed(23, new Integer[] {1, 2, 3})).append(' ').append(boxed(23, new int[][] {{1, 2}, {3}, {}}));
      return b.toString();
    });
    run("HC-10 boxed values", () -> boxed(23, 5) + " " + boxed(23, true) + " " + boxed(23, 5000000000L) + " " + boxed(23, "abc") + " " + boxed(23, 1.5) + " " + boxed(23, 'a'));
    run("HC-11 chaining example", () -> {
      int result = HashCode$.MODULE$.SEED();
      result = HC.hash(result, 42);
      result = HC.hash(result, (Object) "name");
      result = HC.hash(result, new int[] {1, 2, 3});
      result = HC.hash(result, 3.25);
      result = HC.hash(result, true);
      return HashCode$.MODULE$.SEED() + " " + result;
    });
    run("HC-12 null", () -> boxed(23, null));
    run("HC-13 null element in array", () -> boxed(23, new Object[] {"a", null}));
    run("HC-14 SEED constant", () -> Integer.toString(HC.SEED()));
  }
}
