package pilot;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import org.apache.pekko.util.UUIDComparator;
import org.apache.pekko.util.UUIDComparator$;

/**
 * Oracle cases of slice P69 (UTIL-UUID-ORDEN): AC-UTIL-016.14 (util/UUIDComparator.scala:24-75 of Pekko 2.0.0). The
 * behaviour of {@code java.util.UUID} ({@code version()}, {@code timestamp()}, the bit layout) is taken from its public
 * Javadoc and from this oracle only. Registered at the END of the util-core list by {@link UtilCoreOracle}.
 */
final class UuidOrderOracle {
  private UuidOrderOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  static final UUIDComparator C = new UUIDComparator();

  static UUID u(long msb, long lsb) { return new UUID(msb, lsb); }

  /** msb with the given version nibble and the rest from {@code rest} (version bits cleared first). */
  static long withVersion(long rest, int version) { return (rest & ~0xF000L) | ((long) version << 12); }

  static String hex(long v) { return String.format("%016x", v); }

  static String show(UUID x) { return hex(x.getMostSignificantBits()) + ":" + hex(x.getLeastSignificantBits()); }

  /** Raw {@code compare} of one pair: {@code [a]/[b]=<int>}. */
  static String pair(UUID a, UUID b) { return "[" + show(a) + "]/[" + show(b) + "]=" + C.compare(a, b); }

  static String timestampOf(UUID x) {
    try {
      return Long.toString(x.timestamp());
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  static long state;

  static long next() {
    state ^= state >>> 12;
    state ^= state << 25;
    state ^= state >>> 27;
    return state * 0x2545F4914F6CDD1DL;
  }

  /** Mixes every raw result of a sweep: count of negative, zero, positive and a rolling hash of the raw ints. */
  static String sweep(long seed, int pairs, int[] versions, boolean tieHeavy) {
    state = seed;
    int neg = 0, zero = 0, pos = 0, h = 17, antisym = 0, trans = 0;
    List<UUID> all = new ArrayList<>();
    for (int i = 0; i < pairs; i++) {
      long m1 = withVersion(next(), versions[(int) ((next() >>> 1) % versions.length)]), l1 = next();
      long m2, l2;
      int mode = tieHeavy ? i % 4 : 0;
      if (mode == 0) {
        m2 = withVersion(next(), versions[(int) ((next() >>> 1) % versions.length)]);
        l2 = next();
      } else if (mode == 1) {
        m2 = m1;
        l2 = next();
      } else if (mode == 2) {
        int bit = (int) ((next() >>> 1) % 128);
        m2 = bit < 64 ? m1 ^ (1L << bit) : m1;
        l2 = bit < 64 ? l1 : l1 ^ (1L << (bit - 64));
      } else {
        m2 = m1;
        l2 = l1;
      }
      UUID a = u(m1, l1), b = u(m2, l2);
      int r = C.compare(a, b);
      if (r < 0) neg++; else if (r == 0) zero++; else pos++;
      h = h * 31 + r;
      if (Integer.signum(C.compare(b, a)) != -Integer.signum(r)) antisym++;
      if (all.size() < 60) { all.add(a); all.add(b); }
    }
    for (int i = 0; i + 2 < all.size(); i++) {
      UUID a = all.get(i), b = all.get(i + 1), c = all.get(i + 2);
      if (C.compare(a, b) <= 0 && C.compare(b, c) <= 0 && C.compare(a, c) > 0) trans++;
    }
    return "neg=" + neg + " zero=" + zero + " pos=" + pos + " hash=" + h + " antisymmetry-violations=" + antisym
        + " transitivity-violations=" + trans;
  }

  static void register() {
    run("UU-01 version of the msb", () -> {
      StringBuilder b = new StringBuilder();
      for (long msb : new long[] {0L, 0x1000L, 0x4000L, 0xF000L, 0x0FFFL, 0xFFFFFFFFFFFF0FFFL, 0xFFFFFFFFFFFFFFFFL, 0x8000000000008000L,
          0x123456789ABC5DEFL}) {
        b.append('[').append(hex(msb)).append("]=").append(u(msb, 0).version()).append(' ');
      }
      return b.toString().trim();
    });
    run("UU-02 timestamp of version 1", () -> {
      StringBuilder b = new StringBuilder();
      for (long msb : new long[] {0x0000000000001000L, 0x00000001_0000_1000L, 0x00000000_0001_1000L, 0x00000000_0000_1001L,
          0xFFFFFFFF_FFFF_1FFFL, 0x80000000_8000_1800L, 0x12345678_9ABC_1DEFL, 0xFFFFFFFF_FFFF_1000L, 0x00000000_0000_1FFFL}) {
        b.append('[').append(hex(msb)).append("]=").append(timestampOf(u(msb, 0x8000000000000001L))).append(' ');
      }
      return b.toString().trim();
    });
    run("UU-03 timestamp of other versions", () -> {
      StringBuilder b = new StringBuilder();
      for (int v : new int[] {0, 2, 3, 4, 5, 6, 7, 8, 15}) {
        b.append('v').append(v).append('=').append(timestampOf(u(withVersion(0x123456789ABC0DEFL, v), 5))).append(' ');
      }
      return b.toString().trim();
    });
    run("UU-04 order by version number", () -> {
      StringBuilder b = new StringBuilder();
      for (int v1 = 0; v1 < 16; v1 += 3) {
        for (int v2 = 15; v2 >= 0; v2 -= 4) {
          b.append(pair(u(withVersion(0x0123456789AB0000L, v1), 7), u(withVersion(0x0123456789AB0000L, v2), 7))).append(' ');
        }
      }
      return b.toString().trim();
    });
    run("UU-05 version before every other bit", () -> String.join(" ",
        pair(u(withVersion(0xFFFFFFFFFFFFFFFFL, 3), -1L), u(withVersion(0L, 4), 0L)),
        pair(u(withVersion(0L, 3), 0L), u(withVersion(0xFFFFFFFFFFFFFFFFL, 4), -1L)),
        pair(u(withVersion(0xFFFFFFFFFFFFFFFFL, 1), -1L), u(withVersion(0L, 2), 0L)),
        pair(u(withVersion(0L, 15), 0L), u(withVersion(0L, 0), 0L))));
    run("UU-06 unsigned lsb of version 4", () -> String.join(" ",
        pair(u(0x4000000000000000L, 0x8000000000000001L), u(0x4000000000000000L, 1L)),
        pair(u(0x4000000000000000L, 1L), u(0x4000000000000000L, 0x8000000000000001L)),
        pair(u(0x4000000000000000L, -1L), u(0x4000000000000000L, 0L)),
        pair(u(0x4000000000000000L, 0x7FFFFFFFFFFFFFFFL), u(0x4000000000000000L, 0x8000000000000000L)),
        pair(u(0x4000000000000000L, 1L), u(0x4000000000000000L, 2L)),
        pair(u(0x4000000000000000L, 0L), u(0x4000000000000000L, 0L))));
    run("UU-07 unsigned msb of version 4", () -> String.join(" ",
        pair(u(0xF000000000004000L, 0L), u(0x7000000000004000L, 0L)),
        pair(u(0x7000000000004000L, 0L), u(0xF000000000004000L, 0L)),
        pair(u(0x8000000000004000L, 0L), u(0x0000000000004000L, 0L)),
        pair(u(0xFFFFFFFFFFFF4FFFL, 0L), u(0x0000000000004000L, 0L)),
        pair(u(0x00000001FFFF4000L, 0L), u(0x00000000FFFF4000L, 0L)),
        pair(u(0x80000000FFFF4000L, 0L), u(0x7FFFFFFFFFFF4000L, 0L))));
    run("UU-08 the two 32-bit halves", () -> String.join(" ",
        pair(u(0x8000000040000000L, 0L), u(0x7FFFFFFF40000000L, 0L)),
        pair(u(0x0000000140008000L, 0L), u(0x0000000140007FFFL, 0L)),
        pair(u(0x0000000040000000L, 0x0000000080000000L), u(0x0000000040000000L, 0x000000007FFFFFFFL)),
        pair(u(0x0000000040000000L, 0x8000000000000000L), u(0x0000000040000000L, 0x00000000FFFFFFFFL)),
        pair(u(0x0000000040000000L, 0xFFFFFFFF00000000L), u(0x0000000040000000L, 0x7FFFFFFFFFFFFFFFL)),
        pair(u(0x0000000040000000L, 0xFFFFFFFF00000001L), u(0x0000000040000000L, 0xFFFFFFFF00000002L))));
    run("UU-09 raw magnitudes of the result", () -> String.join(" ",
        pair(u(0x0000000040000000L, 0L), u(0x7FFFFFFF40000000L, 0L)),
        pair(u(0x7FFFFFFF40000000L, 0L), u(0x0000000040000000L, 0L)),
        pair(u(0x8000000040000000L, 0L), u(0xFFFFFFFF40000000L, 0L)),
        pair(u(0xFFFFFFFF40000000L, 0L), u(0x8000000040000000L, 0L)),
        pair(u(0x4000000000000000L, 0x8000000000000000L), u(0x4000000000000000L, 0xFFFFFFFF00000000L)),
        pair(u(0x4000000000000000L, 0x0000000000000000L), u(0x4000000000000000L, 0x7FFFFFFF00000000L)),
        pair(u(0x4000000000000000L, 0x0000000000000000L), u(0x4000000000000000L, 0x000000007FFFFFFFL)),
        pair(u(0x4000000000000000L, 0x00000000FFFFFFFFL), u(0x4000000000000000L, 0x0000000000000000L))));
    run("UU-10 version 1 by timestamp", () -> String.join(" ",
        pair(u(0x00000002_0000_1000L, 0L), u(0x00000001_0000_1000L, 0L)),
        pair(u(0x00000000_0001_1000L, 0L), u(0x00000001_0000_1000L, 0L)),
        pair(u(0x00000000_0000_1001L, 0L), u(0x00000001_0000_1000L, 0L)),
        pair(u(0x00000000_0000_1001L, 0L), u(0x00000000_0000_1000L, 0L)),
        pair(u(0x00000000_0000_1000L, 0L), u(0x00000000_0000_1001L, 0L)),
        pair(u(0x80000000_0000_1000L, 0L), u(0x7FFFFFFF_0000_1000L, 0L))));
    run("UU-11 version 1 ignores the msb order", () -> String.join(" ",
        // a large msb (time_hi in the low 16 bits) but a small timestamp, against the opposite
        pair(u(0xFFFFFFFF_FFFF_1000L, 0L), u(0x00000000_0000_1FFFL, 0L)),
        pair(u(0x00000000_0000_1FFFL, 0L), u(0xFFFFFFFF_FFFF_1000L, 0L)),
        pair(u(0x00000001_0000_1000L, 0L), u(0x00000000_0000_1FFFL, 0L)),
        pair(u(0x0000FFFF_FFFF_1000L, 0L), u(0xFFFF0000_0000_1000L, 0L)),
        pair(u(0xFFFF0000_0000_1000L, 0L), u(0x0000FFFF_FFFF_1000L, 0L))));
    run("UU-12 version 1 timestamp ties by unsigned lsb", () -> String.join(" ",
        pair(u(0x12345678_9ABC_1DEFL, 0x8000000000000001L), u(0x12345678_9ABC_1DEFL, 1L)),
        pair(u(0x12345678_9ABC_1DEFL, 1L), u(0x12345678_9ABC_1DEFL, 0x8000000000000001L)),
        pair(u(0x12345678_9ABC_1DEFL, -1L), u(0x12345678_9ABC_1DEFL, 0L)),
        pair(u(0x12345678_9ABC_1DEFL, 0x00000000FFFFFFFFL), u(0x12345678_9ABC_1DEFL, 0xFFFFFFFF00000000L)),
        pair(u(0x12345678_9ABC_1DEFL, 5L), u(0x12345678_9ABC_1DEFL, 5L)),
        pair(u(0x12345678_9ABC_1DEFL, 5L), u(0x12345678_9ABC_1DEFL, 6L))));
    run("UU-13 version 1 lsb tie-break extremes", () -> {
      // the version nibble is the same (1); the timestamp has 60 bits, so every msb bit but the nibble counts, and the
      // lsb tie-break is the only other input
      return String.join(" ",
          pair(u(0x00000000_0000_1000L, 3L), u(0x00000000_0000_1000L, 3L)),
          pair(u(0x00000000_0000_1000L, 3L), u(0x00000000_0000_1000L, 0xC000000000000003L)),
          pair(u(0x00000000_0000_1000L, 0xC000000000000003L), u(0x00000000_0000_1000L, 3L)),
          pair(u(0xFFFFFFFF_FFFF_1FFFL, 0L), u(0xFFFFFFFF_FFFF_1FFFL, 0L)),
          pair(u(0xFFFFFFFF_FFFF_1FFFL, 1L), u(0xFFFFFFFF_FFFF_1FFEL, 0xFFFFFFFFFFFFFFFFL)));
    });
    run("UU-14 versions 0, 2, 3, 5 to 15 order like version 4", () -> {
      StringBuilder b = new StringBuilder();
      for (int v : new int[] {0, 2, 3, 5, 6, 7, 8, 9, 15}) {
        b.append('v').append(v).append(' ')
            .append(pair(u(withVersion(0xF000000000000000L, v), 1L), u(withVersion(0x7000000000000000L, v), 1L))).append(' ')
            .append(pair(u(withVersion(0x1000000000000000L, v), 0x8000000000000000L), u(withVersion(0x1000000000000000L, v), 1L)))
            .append("; ");
      }
      return b.toString().trim();
    });
    run("UU-15 every version against every version", () -> {
      StringBuilder b = new StringBuilder();
      for (int v1 = 0; v1 < 16; v1++) {
        for (int v2 = 0; v2 < 16; v2++) {
          b.append(C.compare(u(withVersion(0x0123456789AB0DEFL, v1), -2L), u(withVersion(0x0123456789AB0DEFL, v2), 3L))).append(',');
        }
        b.append(' ');
      }
      return b.toString().trim();
    });
    run("UU-16 reflexive and null-like extremes", () -> String.join(" ",
        pair(u(0L, 0L), u(0L, 0L)),
        pair(u(-1L, -1L), u(-1L, -1L)),
        pair(u(0L, 0L), u(-1L, -1L)),
        pair(u(-1L, -1L), u(0L, 0L)),
        pair(u(Long.MIN_VALUE, Long.MIN_VALUE), u(Long.MAX_VALUE, Long.MAX_VALUE)),
        pair(u(0x1000L, 0L), u(0x1000L, 0L))));
    run("UU-17 the shared comparator instance", () -> {
      UUIDComparator shared = UUIDComparator$.MODULE$.comparator();
      return "same=" + (shared == UUIDComparator$.MODULE$.comparator()) + " distinctFromNew=" + (shared != C) + " "
          + "cmp=" + shared.compare(u(0x4000000000000000L, 0x8000000000000001L), u(0x4000000000000000L, 1L));
    });
    run("UU-18 sorting a mixed list", () -> {
      List<UUID> l = new ArrayList<>();
      state = 20260518L;
      for (int i = 0; i < 24; i++) {
        int[] vs = {1, 4, 3, 1, 4};
        l.add(u(withVersion(next() & 0x00000000_00FF_FFFFL | (i % 2 == 0 ? 0L : 0x80000000_0000_0000L), vs[i % vs.length]), next()));
      }
      l.sort(C);
      StringBuilder b = new StringBuilder();
      for (UUID x : l) b.append(show(x)).append(' ');
      return b.toString().trim();
    });
    run("UU-19 sweep of 3000 random pairs", () -> sweep(0x9E3779B97F4A7C15L, 3000, new int[] {1, 2, 3, 4, 5, 8}, false));
    run("UU-20 sweep of 3000 pairs with shared bits", () -> sweep(0xD1B54A32D192ED03L, 3000, new int[] {1, 4}, true));
    run("UU-21 sweep of 3000 version 1 pairs", () -> sweep(0x2545F4914F6CDD1DL, 3000, new int[] {1}, true));
  }
}
