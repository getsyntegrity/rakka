package pilot;

import org.apache.pekko.util.PrettyDuration;
import scala.concurrent.duration.Duration;

/**
 * Oracle cases of slice P68 (UTIL-PRETTYDURATION): AC-UTIL-016.5 (util/PrettyDuration.scala:17-86 of Pekko 2.0.0,
 * reached through its public Java forwarders {@code PrettyDuration.format}). Registered at the END of the util-core
 * list by {@link UtilCoreOracle}.
 */
final class PrettyDurationOracle {
  private PrettyDurationOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  /** {@code format(fromNanos(n), includeNanos, precision)} or {@code <Exception>[<message>]}. */
  static String pretty(Duration d, boolean includeNanos, int precision) {
    try {
      return "{" + PrettyDuration.format(d, includeNanos, precision) + "}";
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  static String nanos(long n, boolean includeNanos, int precision) {
    try {
      return pretty(Duration.fromNanos(n), includeNanos, precision);
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  /** {@code [n]={text}} for each value, default rendering ({@code d.pretty}). */
  static String list(long... ns) {
    StringBuilder b = new StringBuilder();
    for (long n : ns) b.append('[').append(n).append("]=").append(nanos(n, false, 4)).append(' ');
    return b.toString().trim();
  }

  static final long US = 1000L, MS = 1000_000L, S = 1000_000_000L, MIN = 60 * S, H = 60 * MIN, D = 24 * H;

  static void register() {
    run("PD-01 spec table", () -> list(9, 95, 999, 1000, 9500, 9500 * US, 9500 * MS, 95 * S, 95 * MIN, 95 * H));
    run("PD-02 infinities and undefined", () -> {
      StringBuilder b = new StringBuilder();
      for (Duration d : new Duration[] {Duration.Inf(), Duration.MinusInf(), Duration.Undefined()}) {
        b.append(pretty(d, false, 4)).append(' ').append(pretty(d, true, 4)).append(' ').append(pretty(d, true, 1)).append(' ')
            .append(pretty(d, false, 40)).append(' ');
      }
      b.append(PrettyDuration.format(Duration.Inf())).append(' ').append(PrettyDuration.format(Duration.MinusInf())).append(' ')
          .append(PrettyDuration.format(Duration.Undefined()));
      return b.toString();
    });
    run("PD-03 infinities with invalid precision", () -> {
      StringBuilder b = new StringBuilder();
      for (Duration d : new Duration[] {Duration.Inf(), Duration.MinusInf(), Duration.Undefined()})
        for (int p : new int[] {0, -1, Integer.MIN_VALUE}) b.append(pretty(d, false, p)).append(' ');
      return b.toString().trim();
    });
    run("PD-04 unit boundaries", () -> {
      StringBuilder b = new StringBuilder();
      for (long u : new long[] {US, MS, S, MIN, H, D}) b.append(list(u - 1, u, u + 1)).append(' ');
      return b.toString().trim();
    });
    run("PD-05 default format equals pretty with 4 digits", () -> {
      StringBuilder b = new StringBuilder();
      for (long n : new long[] {1, 999, 1500, 4 * S, 3 * D})
        b.append(PrettyDuration.format(Duration.fromNanos(n))).append('|').append(nanos(n, false, 4)).append(' ');
      return b.toString().trim();
    });
    run("PD-06 include nanos", () -> {
      StringBuilder b = new StringBuilder();
      for (long n : new long[] {0, 1, 999, 1500, 95 * S, 3 * D + 17, Long.MAX_VALUE, -5, -Long.MAX_VALUE})
        b.append('[').append(n).append("]=").append(nanos(n, true, 4)).append(' ').append(nanos(n, true, 2)).append(' ');
      return b.toString().trim();
    });
    run("PD-07 precision must be positive", () -> {
      StringBuilder b = new StringBuilder();
      for (int p : new int[] {0, -1, -7, Integer.MIN_VALUE}) b.append(nanos(1500, false, p)).append(' ').append(nanos(1500, true, p)).append(' ');
      return b.toString().trim();
    });
    run("PD-08 scientific notation", () -> list(9999 * D, 10000 * D, 99999 * D, 100000 * D, 12345 * D, 106751 * D, 9999 * D + 5 * H,
        9999 * D + 12 * H, 9999 * D + 11 * H));
    run("PD-09 zero", () -> {
      StringBuilder b = new StringBuilder();
      for (int p : new int[] {1, 2, 4, 7, 12}) b.append(nanos(0, false, p)).append(' ');
      return b.toString().trim();
    });
    run("PD-10 negative durations", () -> list(-1, -999, -1000, -1500, -US, -MS, -95 * S, -95 * MIN, -95 * H, -3 * D, -Long.MAX_VALUE));
    run("PD-11 Long.MaxValue", () -> {
      StringBuilder b = new StringBuilder();
      for (int p : new int[] {1, 3, 4, 6, 10, 17, 18, 20}) b.append(nanos(Long.MAX_VALUE, false, p)).append(' ');
      return b.toString().trim();
    });
    run("PD-12 precision from 1 to 25", () -> {
      StringBuilder b = new StringBuilder();
      for (long n : new long[] {1, 7, 1234, 1_234_567, 98_765_432_101L, 123_456_789_012_345L}) {
        for (int p = 1; p <= 25; p++) b.append(nanos(n, false, p)).append(' ');
        b.append("| ");
      }
      return b.toString().trim();
    });
    run("PD-13 large precision", () -> {
      StringBuilder b = new StringBuilder();
      for (int p : new int[] {50, 100, 400}) b.append(nanos(1500, false, p)).append(' ').append(nanos(3 * D, false, p)).append(' ');
      return b.toString().trim();
    });
    run("PD-14 rounds half up on the decimal digits", () -> {
      StringBuilder b = new StringBuilder();
      // 1.0005 ms, 12.345 us, 0.5-ending values: HALF_UP on the shortest decimal of the double, not on its binary expansion.
      for (long n : new long[] {1_000_500, 12_345, 1_005_000, 2_675_000, 1_000_050, 999_950, 9_999_500, 99_995, 9_995, 2_500, 3_500,
          1_235_000_000L, 1_245_000_000L, 8_650_000_000_000L})
        b.append('[').append(n).append("]=").append(nanos(n, false, 4)).append(' ').append(nanos(n, false, 3)).append(' ')
            .append(nanos(n, false, 2)).append(' ').append(nanos(n, false, 1)).append(' ');
      return b.toString().trim();
    });
    run("PD-15 carry to the next power of ten", () -> {
      StringBuilder b = new StringBuilder();
      for (long n : new long[] {9999, 99_995, 999_950, 9_999_500, 999_999, 59_999_500_000L, 3_599_950_000_000L, 86_399_950_000_000L,
          106_751L * D, 9_999L * D + 22 * H}) {
        b.append('[').append(n).append("]=");
        for (int p = 1; p <= 6; p++) b.append(nanos(n, false, p)).append(' ');
      }
      return b.toString().trim();
    });
    run("PD-16 sweep of 3000 pseudo-random magnitudes", () -> sweep(0x2545F4914F6CDD1DL, 3000, false));
    run("PD-17 sweep of 3000 pseudo-random precisions", () -> sweep(0x9E3779B97F4A7C15L, 3000, true));
    run("PD-18 sweep of round values", () -> {
      StringBuilder b = new StringBuilder();
      long[] units = {1, US, MS, S, MIN, H, D};
      for (long u : units)
        for (long k : new long[] {1, 2, 3, 5, 7, 10, 11, 15, 24, 59, 60, 61, 99, 100, 101, 999, 1000, 1001, 9999, 10000, 12345, 99999})
          b.append(nanos(u * k, false, 4)).append(' ');
      return b.toString().trim();
    });
    // A negative duration is shown in nanoseconds: its value is the long converted to a double, an integer from 2^53 on.
    // Java does not print the shortest digits of such a double but those of the exact integer, minus a number of low digits.
    run("PD-19 integral doubles from 2^53", () -> {
      StringBuilder b = new StringBuilder();
      for (int e = 52; e <= 62; e++) {
        long base = 1L << e;
        for (long m : new long[] {base, base + base / 3, base + base / 2 + 12345, 2 * base - 1}) {
          b.append('[').append(-m).append("]=");
          for (int p : new int[] {16, 17, 18, 19, 20, 22}) b.append(nanos(-m, false, p)).append(' ');
        }
      }
      b.append("[max]=");
      for (int p : new int[] {1, 4, 15, 16, 17, 18, 19, 20, 22}) b.append(nanos(-Long.MAX_VALUE, false, p)).append(' ');
      return b.toString().trim();
    });
  }

  /** The same generator as the Rust twin: xorshift64* on the seed; magnitude from 1 ns to 2^63, sign and precision. */
  static String sweep(long seed, int count, boolean varyPrecision) {
    long x = seed;
    StringBuilder b = new StringBuilder();
    for (int i = 0; i < count; i++) {
      x ^= x >>> 12;
      x ^= x << 25;
      x ^= x >>> 27;
      long r = x * 0x2545F4914F6CDD1DL;
      int bits = (int) ((r >>> 58) & 63);          // 0..63
      long mag = (r << 1 >>> 1) >>> (63 - bits);   // up to `bits` significant bits
      long n = ((r >>> 57) & 1) == 0 ? mag : -mag;
      int p = varyPrecision ? 1 + (int) ((r >>> 52) & 15) : 4;
      b.append(nanos(n, ((r >>> 51) & 1) == 1, p)).append(' ');
    }
    return b.toString().trim();
  }
}
