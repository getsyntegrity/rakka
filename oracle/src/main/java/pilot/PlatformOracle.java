package pilot;

import com.typesafe.config.ConfigFactory;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Cancellable;
import org.apache.pekko.actor.Scheduler;
import org.apache.pekko.util.*;
import scala.concurrent.ExecutionContext;
import scala.concurrent.duration.FiniteDuration;

/**
 * Oracle cases of slice P06 (UTIL-PLATAFORMA): AC-UTIL-020 (Unsafe.fastHash, copyUSAsciiStrToBytes), 023 (makePattern),
 * 024 (identity hash order), 025 (time formatting), 026 (isWindows), 027 (Clock) and 028 (SWARUtil).
 * Registered at the END of the util-core list by {@link UtilCoreOracle}, so earlier case ids do not shift.
 * The JVM time zone is pinned to GMT+05:30 by UtilCoreOracle (Helpers.timestamp captures the default zone on load).
 */
final class PlatformOracle {
  private PlatformOracle() {}

  static final Helpers$ H = Helpers$.MODULE$;
  static final SWARUtil$ SWAR = SWARUtil$.MODULE$;

  static final class Tagged {
    final int tag;
    Tagged(int tag) { this.tag = tag; }
  }

  static String hex16(long v) { return String.format("%016x", v); }
  static String hex8(int v) { return String.format("%08x", v); }
  static String hex4(short v) { return String.format("%04x", v & 0xFFFF); }
  static String hexBytes(byte[] b) {
    StringBuilder s = new StringBuilder();
    for (byte x : b) s.append(String.format("%02x", x & 0xFF));
    return s.toString();
  }

  static FiniteDuration nanosDuration(long n) { return FiniteDuration.apply(n, TimeUnit.NANOSECONDS); }

  /** xorshift64 generator shared (by construction) with the Rust twin. */
  static final class Gen {
    long state;
    Gen(long seed) { state = seed; }
    long next() {
      state ^= state << 13;
      state ^= state >>> 7;
      state ^= state << 17;
      return state;
    }
  }

  /** A Scheduler that records the first task of scheduleWithFixedDelay so that the test drives the ticks. */
  static final class CapturingScheduler implements Scheduler {
    Runnable task;
    int scheduleOnceCalls;
    long lastDelayNanos;

    @Override public Cancellable scheduleOnce(FiniteDuration delay, Runnable runnable, ExecutionContext executor) {
      scheduleOnceCalls++;
      lastDelayNanos = delay.toNanos();
      task = runnable;
      return null;
    }

    @Override public Cancellable schedule(FiniteDuration initialDelay, FiniteDuration interval, Runnable runnable, ExecutionContext executor) {
      throw new UnsupportedOperationException();
    }

    @Override public double maxFrequency() { return 1.0; }
  }

  static ScheduledClock scheduled(long intervalNanos, CapturingScheduler scheduler) {
    return new ScheduledClock(nanosDuration(intervalNanos), scheduler, scala.concurrent.ExecutionContext.global());
  }

  static String tsUtc(long... values) {
    StringBuilder out = new StringBuilder();
    for (long v : values) out.append('[').append(v).append("]=").append(H.currentTimeMillisToUTCString(v)).append(' ');
    return out.toString().trim();
  }

  static String ts(long... values) {
    StringBuilder out = new StringBuilder();
    for (long v : values) out.append('[').append(v).append("]=").append(H.timestamp(v)).append(' ');
    return out.toString().trim();
  }

  static final String QUIET = "pekko.loglevel = OFF\npekko.stdout-loglevel = OFF";

  static String clockKind(String intervalText) {
    ActorSystem system = intervalText == null
        ? ActorSystem.create("clock-kind", ConfigFactory.parseString(QUIET).withFallback(ConfigFactory.load()))
        : ActorSystem.create("clock-kind", ConfigFactory.parseString("pekko.scheduled-clock-interval = " + intervalText + "\n" + QUIET).withFallback(ConfigFactory.load()));
    try {
      return Clock$.MODULE$.get(system).getClass().getSimpleName();
    } finally {
      system.terminate();
      try {
        system.getWhenTerminated().toCompletableFuture().get();
      } catch (Exception e) {
        throw new IllegalStateException(e);
      }
    }
  }

  static final byte[] SAMPLE = {
    0x01, 0x02, 0x03, (byte) 0x84, (byte) 0xFF, 0x00, 0x7F, (byte) 0x80,
    0x55, (byte) 0xAA, 0x10, 0x20, (byte) 0xFE, 0x01, (byte) 0x81, 0x7E
  };

  static String swarGets(boolean viaMethodHandle) {
    StringBuilder out = new StringBuilder();
    for (int i = 0; i + 8 <= SAMPLE.length; i++) {
      out.append("L").append(i).append(":");
      out.append(hex16(viaMethodHandle ? SWAR.getLong(SAMPLE, i, ByteOrder.BIG_ENDIAN) : SWAR.getLongBEWithoutMethodHandle(SAMPLE, i))).append('/');
      out.append(hex16(viaMethodHandle ? SWAR.getLong(SAMPLE, i, ByteOrder.LITTLE_ENDIAN) : SWAR.getLongLEWithoutMethodHandle(SAMPLE, i))).append(' ');
    }
    for (int i = 0; i + 4 <= SAMPLE.length; i++) {
      out.append("I").append(i).append(":");
      out.append(hex8(viaMethodHandle ? SWAR.getInt(SAMPLE, i, ByteOrder.BIG_ENDIAN) : SWAR.getIntBEWithoutMethodHandle(SAMPLE, i))).append('/');
      out.append(hex8(viaMethodHandle ? SWAR.getInt(SAMPLE, i, ByteOrder.LITTLE_ENDIAN) : SWAR.getIntLEWithoutMethodHandle(SAMPLE, i))).append(' ');
    }
    for (int i = 0; i + 2 <= SAMPLE.length; i++) {
      out.append("S").append(i).append(":");
      out.append(hex4(viaMethodHandle ? SWAR.getShort(SAMPLE, i, ByteOrder.BIG_ENDIAN) : SWAR.getShortBEWithoutMethodHandle(SAMPLE, i))).append('/');
      out.append(hex4(viaMethodHandle ? SWAR.getShort(SAMPLE, i, ByteOrder.LITTLE_ENDIAN) : SWAR.getShortLEWithoutMethodHandle(SAMPLE, i))).append(' ');
    }
    return out.toString().trim();
  }

  static String swarPuts(boolean viaMethodHandle) {
    long[] longs = {0L, 1L, -1L, 0x0123456789ABCDEFL, Long.MIN_VALUE, Long.MAX_VALUE};
    int[] ints = {0, 1, -1, 0x12345678, Integer.MIN_VALUE, Integer.MAX_VALUE};
    StringBuilder out = new StringBuilder();
    for (ByteOrder order : new ByteOrder[] {ByteOrder.BIG_ENDIAN, ByteOrder.LITTLE_ENDIAN}) {
      boolean be = order == ByteOrder.BIG_ENDIAN;
      for (long v : longs) {
        for (int at : new int[] {0, 3}) {
          byte[] a = new byte[11];
          if (viaMethodHandle) SWAR.putLong(a, at, v, order);
          else if (be) SWAR.putLongBEWithoutMethodHandle(a, at, v);
          else SWAR.putLongLEWithoutMethodHandle(a, at, v);
          out.append("PL").append(be ? "B" : "L").append(at).append('=').append(hexBytes(a)).append(' ');
        }
      }
      for (int v : ints) {
        for (int at : new int[] {0, 5}) {
          byte[] a = new byte[9];
          if (viaMethodHandle) SWAR.putInt(a, at, v, order);
          else if (be) SWAR.putIntBEWithoutMethodHandle(a, at, v);
          else SWAR.putIntLEWithoutMethodHandle(a, at, v);
          out.append("PI").append(be ? "B" : "L").append(at).append('=').append(hexBytes(a)).append(' ');
          // putShort writes the low 16 bits of an Int
          byte[] s = new byte[5];
          if (viaMethodHandle) SWAR.putShort(s, at % 3, v, order);
          else if (be) SWAR.putShortBEWithoutMethodHandle(s, at % 3, v);
          else SWAR.putShortLEWithoutMethodHandle(s, at % 3, v);
          out.append("PS").append(be ? "B" : "L").append(at % 3).append('=').append(hexBytes(s)).append(' ');
        }
      }
    }
    return out.toString().trim();
  }

  static String fastHashes(String... texts) {
    StringBuilder out = new StringBuilder();
    for (String t : texts) out.append('[').append(t.length() > 40 ? t.length() + "x" : UtilCoreOracle.esc(t)).append("]=").append(Unsafe.fastHash(t)).append(' ');
    return out.toString().trim();
  }

  static String allAscii() {
    StringBuilder s = new StringBuilder();
    for (char c = 0; c < 128; c++) s.append(c);
    return s.toString();
  }

  static String makePatternBits(int maxLength) {
    StringBuilder out = new StringBuilder();
    for (int len = 0; len <= maxLength; len++) {
      for (String pat : UtilCoreOracle.allStrings("ab*?", len)) {
        java.util.regex.Pattern p = H.makePattern(pat);
        out.append('[').append(pat).append("]=");
        for (int k = 0; k <= 4; k++) for (String in : UtilCoreOracle.allStrings("ab", k)) out.append(p.matcher(in).matches() ? 'T' : 'F');
        out.append(' ');
      }
    }
    return out.toString().trim();
  }

  static String makePatternPairs(String... pairs) {
    StringBuilder out = new StringBuilder();
    for (String pair : pairs) {
      int bar = pair.indexOf('|');
      String pat = pair.substring(0, bar), in = pair.substring(bar + 1);
      out.append('[').append(UtilCoreOracle.esc(pat)).append("]/[").append(UtilCoreOracle.esc(in)).append("]=")
          .append(H.makePattern(pat).matcher(in).matches()).append(" glob=").append(Glob$.MODULE$.matches(pat, in)).append(' ');
    }
    return out.toString().trim();
  }

  static void register() {
    // ---- AC-UTIL-026: Helpers.scala:44 (os.name contains "win"); the value is that of the host running the oracle.
    UtilCoreOracle.run("WIN-01 isWindows", () -> "isWindows=" + H.isWindows());

    // ---- AC-UTIL-025: Helpers.scala:73-115
    UtilCoreOracle.run("TS-UTC-01 currentTimeMillisToUTCString", () -> tsUtc(0L, 1L, 9L, 10L, 99L, 100L, 999L, 1000L, 59999L, 60000L, 3599999L, 3600000L, 36000000L, 86399999L, 86400000L, 86400001L, 90061001L, 1557014400000L, 1557100799999L, Long.MAX_VALUE));
    UtilCoreOracle.run("TS-UTC-02 negative timestamps", () -> tsUtc(-1L, -9L, -10L, -999L, -1000L, -60000L, -3600000L, -36000000L, -86399999L, -86400000L, -86400001L, -1234567L, Long.MIN_VALUE));
    // zone = GMT+05:30 (fixed): the oracle pins user.timezone before Helpers loads.
    UtilCoreOracle.run("TS-LOCAL-01 timestamp small values", () -> ts(0L, 1L, 999L, 1000L, 86399999L, 86400000L, -1L, -1000L, -19800000L, -19800001L));
    UtilCoreOracle.run("TS-LOCAL-02 timestamp calendar edges", () -> ts(951782400000L, 951868800000L, 1582934400000L, 1583020800000L, 4107542400000L, -2203891200000L, -2203977600000L, -62135596800000L, -62167219200000L, -62167239000001L, 253402300799999L, 253402281000000L, Long.MAX_VALUE, Long.MIN_VALUE));

    // ---- AC-UTIL-024: Helpers.scala:49-71. identityHashCode is not reproducible: only invariants are recorded.
    UtilCoreOracle.run("ID-01 compareIdentityHash invariants", () -> {
      List<Object> objects = new ArrayList<>();
      for (int i = 0; i < 60; i++) objects.add(new Object());
      boolean selfZero = true, antisym = true, domain = true, transitive = true;
      for (Object a : objects) {
        selfZero &= H.compareIdentityHash(a, a) == 0;
        for (Object b : objects) {
          int ab = H.compareIdentityHash(a, b);
          domain &= ab == -1 || ab == 0 || ab == 1;
          antisym &= ab == -H.compareIdentityHash(b, a);
          for (Object c : objects.subList(0, 12)) {
            if (ab < 0 && H.compareIdentityHash(b, c) < 0) transitive &= H.compareIdentityHash(a, c) < 0;
          }
        }
      }
      return "self=" + selfZero + " antisym=" + antisym + " domain=" + domain + " transitive=" + transitive;
    });
    UtilCoreOracle.run("ID-02 identityHashComparator with tie-break", () -> {
      int[] tieBreakCalls = {0};
      Comparator<Tagged> byTag = (x, y) -> { tieBreakCalls[0]++; return Integer.compare(x.tag, y.tag); };
      Comparator<Tagged> comparator = H.identityHashComparator(byTag);
      List<Tagged> items = new ArrayList<>();
      for (int i = 0; i < 80; i++) items.add(new Tagged(i));
      boolean selfZero = true, noZeroForDistinct = true, antisym = true;
      for (Tagged a : items) {
        selfZero &= comparator.compare(a, a) == 0;
        for (Tagged b : items) {
          if (a != b) noZeroForDistinct &= comparator.compare(a, b) != 0;
          antisym &= Integer.signum(comparator.compare(a, b)) == -Integer.signum(comparator.compare(b, a));
        }
      }
      List<Tagged> sorted = new ArrayList<>(items);
      sorted.sort(comparator);
      boolean strictlyIncreasing = true;
      for (int i = 1; i < sorted.size(); i++) strictlyIncreasing &= comparator.compare(sorted.get(i - 1), sorted.get(i)) < 0;
      java.util.Collections.reverse(items);
      List<Tagged> sortedFromReverse = new ArrayList<>(items);
      sortedFromReverse.sort(comparator);
      return "self=" + selfZero + " distinct=" + noZeroForDistinct + " antisym=" + antisym + " sorted=" + strictlyIncreasing + " stable=" + sorted.equals(sortedFromReverse);
    });

    // ---- AC-UTIL-027: Clock.scala:36-125. nanoTime has an arbitrary origin: deltas and invariants only.
    UtilCoreOracle.run("CK-01 NanoClock", () -> {
      NanoClock clock = new NanoClock();
      boolean monotonic = true;
      long previous = clock.currentTime();
      for (int i = 0; i < 1000; i++) {
        long now = clock.currentTime();
        monotonic &= now - previous >= 0;
        previous = now;
      }
      StringBuilder out = new StringBuilder("monotonic=" + monotonic);
      long[] durations = {0L, 1L, 1_000_000_000L, -1_000_000_000L, 4_611_686_018_427_387_903L};
      for (long d : durations) {
        long before = clock.currentTime();
        long earlier = clock.earlierTime(nanosDuration(d));
        long after = clock.currentTime();
        long reconstructed = earlier + d;
        out.append(" d=").append(d).append(":inRange=").append(reconstructed - before >= 0 && after - reconstructed >= 0);
      }
      return out.toString();
    });
    UtilCoreOracle.run("CK-02 ScheduledClock increments by one until maxIncrement", () -> {
      CapturingScheduler scheduler = new CapturingScheduler();
      ScheduledClock clock = scheduled(100_003L, scheduler); // maxIncrement = 100003 - 100000 = 3
      long base = clock.earlierTime(nanosDuration(0));
      StringBuilder out = new StringBuilder("scheduled=" + (scheduler.task != null) + " delayNanos=" + scheduler.lastDelayNanos + " reached=" + clock.maxIncrementReached());
      for (int i = 1; i <= 6; i++) {
        long c = clock.currentTime();
        out.append(" c").append(i).append("=+").append(c - base).append(",reached=").append(clock.maxIncrementReached());
      }
      out.append(" earlier(0)=+").append(clock.earlierTime(nanosDuration(0)) - base).append(" earlier(5)=+").append(clock.earlierTime(nanosDuration(5)) - base);
      return out.toString();
    });
    UtilCoreOracle.run("CK-03 ScheduledClock maxIncrement boundaries", () -> {
      StringBuilder out = new StringBuilder();
      for (long interval : new long[] {1L, 100_000L, 100_001L, 100_002L, 100_004L}) {
        ScheduledClock clock = scheduled(interval, new CapturingScheduler());
        long base = clock.earlierTime(nanosDuration(0));
        out.append("interval=").append(interval).append(':');
        for (int i = 0; i < 6; i++) out.append(" +").append(clock.currentTime() - base);
        out.append(" reached=").append(clock.maxIncrementReached()).append("; ");
      }
      ScheduledClock oneMs = scheduled(1_000_000L, new CapturingScheduler()); // maxIncrement = 900000
      long base = oneMs.earlierTime(nanosDuration(0));
      long last = 0;
      for (int i = 1; i <= 900_000; i++) last = oneMs.currentTime() - base;
      out.append("interval=1000000: n=900000 +").append(last).append(" reached=").append(oneMs.maxIncrementReached());
      out.append(" n+1 +").append(oneMs.currentTime() - base).append(" n+2 +").append(oneMs.currentTime() - base);
      return out.toString();
    });
    UtilCoreOracle.run("CK-04 ScheduledClock update resumes the increments", () -> {
      CapturingScheduler scheduler = new CapturingScheduler();
      ScheduledClock clock = scheduled(100_003L, scheduler);
      long base = clock.earlierTime(nanosDuration(0));
      for (int i = 0; i < 5; i++) clock.currentTime();
      boolean reachedBefore = clock.maxIncrementReached();
      long stuck = clock.currentTime() - base;
      scheduler.task.run(); // the scheduler tick: update()
      long afterUpdate = clock.earlierTime(nanosDuration(0));
      boolean reachedAfter = clock.maxIncrementReached();
      long next = clock.currentTime();
      return "reachedBefore=" + reachedBefore + " stuck=+" + stuck + " movedForward=" + (afterUpdate - (base + stuck) >= 0) + " reachedAfter=" + reachedAfter
          + " nextIsPlusOne=" + (next - afterUpdate == 1);
    });
    UtilCoreOracle.run("CK-05 Clock extension selection", () -> "default=" + clockKind(null) + " 0=" + clockKind("0") + " 0s=" + clockKind("0 s") + " 1ms=" + clockKind("1 ms") + " 1s=" + clockKind("1 s"));

    // ---- AC-UTIL-028: SWARUtil.scala:64-457
    UtilCoreOracle.run("SW-01 compilePattern for every byte", () -> {
      StringBuilder out = new StringBuilder();
      for (int b = -128; b <= 127; b++) out.append(hex16(SWAR.compilePattern((byte) b))).append(' ');
      return out.toString().trim();
    });
    UtilCoreOracle.run("SW-02 applyPattern", () -> {
      StringBuilder out = new StringBuilder();
      long[] words = {0L, -1L, 0x0102030405060708L, 0x8080808080808080L, 0x7F7F7F7F7F7F7F7FL, 0x0001000100010001L, 0xFF00FF00FF00FF00L};
      long[] patterns = {SWAR.compilePattern((byte) 0), SWAR.compilePattern((byte) 1), SWAR.compilePattern((byte) 0x7F), SWAR.compilePattern((byte) 0x80), SWAR.compilePattern((byte) 0xFF)};
      for (long w : words) for (long p : patterns) out.append(hex16(SWAR.applyPattern(w, p))).append(' ');
      Gen gen = new Gen(0x9E3779B97F4A7C15L);
      for (int i = 0; i < 200; i++) {
        long w = gen.next();
        byte probe = (byte) gen.next();
        out.append(hex16(SWAR.applyPattern(w, SWAR.compilePattern(probe)))).append(' ');
        long planted = (w & ~(0xFFL << (8 * (i % 8)))) | ((probe & 0xFFL) << (8 * (i % 8)));
        out.append(hex16(SWAR.applyPattern(planted, SWAR.compilePattern(probe)))).append(' ');
      }
      return out.toString().trim();
    });
    UtilCoreOracle.run("SW-03 getIndex and getLastIndex", () -> {
      StringBuilder out = new StringBuilder();
      for (int mask = 0; mask < 256; mask++) {
        long word = 0;
        for (int b = 0; b < 8; b++) if ((mask & (1 << b)) != 0) word |= 0x80L << (8 * b);
        out.append(SWAR.getIndex(word)).append('/').append(SWAR.getLastIndex(word)).append(' ');
      }
      long[] others = {1L, 0xFFL, 0x100L, 0x0000FFFF00000000L, Long.MIN_VALUE, -1L, 0x00FF000000000000L, 0x0000000100000000L};
      for (long w : others) out.append(SWAR.getIndex(w)).append('/').append(SWAR.getLastIndex(w)).append(' ');
      return out.toString().trim();
    });
    UtilCoreOracle.run("SW-04 getLong getInt getShort", () -> swarGets(true));
    UtilCoreOracle.run("SW-05 putLong putInt putShort", () -> swarPuts(true));
    UtilCoreOracle.run("SW-06 variants without MethodHandle", () -> swarGets(false) + " | " + swarPuts(false));
    UtilCoreOracle.run("SW-07 byte-wise getShort getInt getLong", () -> {
      StringBuilder out = new StringBuilder();
      for (int i = 0; i + 8 <= SAMPLE.length; i += 3) {
        byte b0 = SAMPLE[i], b1 = SAMPLE[i + 1], b2 = SAMPLE[i + 2], b3 = SAMPLE[i + 3], b4 = SAMPLE[i + 4], b5 = SAMPLE[i + 5], b6 = SAMPLE[i + 6], b7 = SAMPLE[i + 7];
        out.append(hex4(SWAR.getShortBE(b0, b1))).append('/').append(hex4(SWAR.getShortLE(b0, b1))).append(' ');
        out.append(hex8(SWAR.getIntBE(b0, b1, b2, b3))).append('/').append(hex8(SWAR.getIntLE(b0, b1, b2, b3))).append(' ');
        out.append(hex16(SWAR.getLongBE(b0, b1, b2, b3, b4, b5, b6, b7))).append('/').append(hex16(SWAR.getLongLE(b0, b1, b2, b3, b4, b5, b6, b7))).append(' ');
      }
      return out.toString().trim();
    });

    // ---- AC-UTIL-020: Unsafe.java:100-146 (ASCII text only: the non-ASCII result depends on the JVM, Q-UTIL-18 proposed)
    UtilCoreOracle.run("UN-01 fastHash of ASCII text", () -> fastHashes("", "a", "abc", "abcdefghijklmnopqrstuvxyz", "Hello, World!", allAscii(), "x".repeat(1000), "\u0000", "\u007f"));
    UtilCoreOracle.run("UN-02 copyUSAsciiStrToBytes of ASCII text", () -> {
      StringBuilder out = new StringBuilder();
      for (String text : new String[] {"", "abc", "abcdefghijklmnopqrstuvxyz", allAscii()}) {
        byte[] target = new byte[text.length() + 3];
        java.util.Arrays.fill(target, (byte) 0x5A);
        Unsafe.copyUSAsciiStrToBytes(text, target);
        out.append(hexBytes(target)).append(' ');
      }
      return out.toString().trim();
    });
    UtilCoreOracle.run("UN-03 fastHash of non-ASCII text", () -> {
      StringBuilder out = new StringBuilder();
      for (String text : new String[] {"é", "aéb", "€", "😀"}) {
        String shown = UtilCoreOracle.esc(text);
        String result;
        try {
          result = String.valueOf(Unsafe.fastHash(text));
        } catch (Throwable t) {
          result = t.getClass().getSimpleName();
        }
        out.append('[').append(shown).append("]=").append(result).append(' ');
      }
      return out.toString().trim();
    });

    // ---- AC-UTIL-023: Helpers.scala:46-47. Rust does not reproduce java.util.regex (Q-UTIL-8 (c)); the glob side is covered by GL-*.
    UtilCoreOracle.run("MP-EXH makePattern lengths 0-4", () -> makePatternBits(4));
    UtilCoreOracle.run("MP-NL makePattern vs Glob with line breaks and astral", () -> makePatternPairs("a?b|a\nb", "a*b|a\nb", "a?b|a\u0085b", "a?b|a b", "a?b|a🙂b", "a??b|a🙂b", "?|\r", "*|\r\n"));
  }
}
