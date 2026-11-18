package pilot;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import org.apache.pekko.util.AlwaysIncreasingClock;
import org.apache.pekko.util.WallClock;
import org.apache.pekko.util.WallClock$;

/**
 * Oracle cases of slice P70 (UTIL-RELOJ): AC-UTIL-016.16 (util/WallClock.scala:27-57 of Pekko 2.0.0). The clock reads
 * {@code System.currentTimeMillis()}, so no case prints a wall-clock value: the cases seed the clock's state
 * ({@code AlwaysIncreasingClock} is an {@code AtomicLong}) far in the future, where every result is exact, or compare
 * invariants (strictly increasing, within the bounds of the surrounding wall-clock readings). No sleeps. Registered at
 * the END of the util-core list by {@link UtilCoreOracle}.
 */
final class WallClockOracle {
  private WallClockOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  static final long FUTURE = 1_000_000_000_000_000_000L;

  static String strictlyIncreasing(long[] xs) {
    for (int i = 1; i < xs.length; i++) if (xs[i] <= xs[i - 1]) return "false";
    return "true";
  }

  static long[] calls(AlwaysIncreasingClock c, int n) {
    long[] r = new long[n];
    for (int i = 0; i < n; i++) r[i] = c.currentTimeMillis();
    return r;
  }

  static String offsets(long[] xs, long base) {
    StringBuilder b = new StringBuilder();
    for (long x : xs) b.append('+').append(x - base).append(' ');
    return b.toString().trim();
  }

  /** Runs {@code threads} threads of {@code each} calls on {@code c}; every result, sorted, plus per-thread order. */
  static long[][] concurrent(AlwaysIncreasingClock c, int threads, int each) throws Exception {
    long[][] out = new long[threads][each];
    Thread[] ts = new Thread[threads];
    java.util.concurrent.CountDownLatch go = new java.util.concurrent.CountDownLatch(1);
    for (int t = 0; t < threads; t++) {
      final int k = t;
      ts[t] = new Thread(() -> {
        try { go.await(); } catch (InterruptedException e) { throw new RuntimeException(e); }
        for (int i = 0; i < each; i++) out[k][i] = c.currentTimeMillis();
      });
      ts[t].start();
    }
    go.countDown();
    for (Thread t : ts) t.join();
    return out;
  }

  static String concurrentReport(long[][] out, long base) {
    boolean eachIncreasing = true;
    List<Long> all = new ArrayList<>();
    for (long[] row : out) {
      if (!strictlyIncreasing(row).equals("true")) eachIncreasing = false;
      for (long x : row) all.add(x);
    }
    Collections.sort(all);
    boolean distinct = true;
    for (int i = 1; i < all.size(); i++) if (all.get(i).equals(all.get(i - 1))) distinct = false;
    String shape = all.get(all.size() - 1) - all.get(0) == all.size() - 1 ? "contiguous" : "gaps";
    return "count=" + all.size() + " distinct=" + distinct + " eachThreadIncreasing=" + eachIncreasing + " " + shape
        + " min=+" + (all.get(0) - base) + " max=+" + (all.get(all.size() - 1) - base);
  }

  static void register() {
    run("WC-01 a new clock starts at zero", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      return "state=" + c.get();
    });
    run("WC-02 first reading of a new clock is the wall time", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      long before = System.currentTimeMillis();
      long r = c.currentTimeMillis();
      long after = System.currentTimeMillis();
      return "inRange=" + (before <= r && r <= after) + " stateIsResult=" + (c.get() == r);
    });
    run("WC-03 1000 readings strictly increase", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      long before = System.currentTimeMillis();
      long[] r = calls(c, 1000);
      return "strict=" + strictlyIncreasing(r) + " aboveBefore=" + (r[0] >= before) + " stateIsLast=" + (c.get() == r[999]);
    });
    run("WC-04 a clock ahead of the wall time adds one", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(FUTURE);
      return offsets(calls(c, 6), FUTURE) + " state=+" + (c.get() - FUTURE);
    });
    run("WC-05 a clock behind the wall time jumps to it", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(1L);
      long before = System.currentTimeMillis();
      long r = c.currentTimeMillis();
      long after = System.currentTimeMillis();
      return "inRange=" + (before <= r && r <= after) + " stateIsResult=" + (c.get() == r);
    });
    run("WC-06 negative and minimum states are behind the wall time", () -> {
      StringBuilder b = new StringBuilder();
      for (long s : new long[] {-5L, -1L, Long.MIN_VALUE, Long.MIN_VALUE + 1}) {
        AlwaysIncreasingClock c = new AlwaysIncreasingClock();
        c.set(s);
        long before = System.currentTimeMillis();
        long r = c.currentTimeMillis();
        long after = System.currentTimeMillis();
        b.append("[").append(s).append("]=").append(before <= r && r <= after).append(' ');
      }
      return b.toString().trim();
    });
    run("WC-07 the increment wraps around at the top of the Long range", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(Long.MAX_VALUE - 2);
      long a = c.currentTimeMillis(), b = c.currentTimeMillis(), d = c.currentTimeMillis();
      long before = System.currentTimeMillis();
      long e = c.currentTimeMillis();
      long after = System.currentTimeMillis();
      return "a=" + a + " b=" + b + " c=" + d + " wrapped=" + (d == Long.MIN_VALUE) + " nextIsWall=" + (before <= e && e <= after);
    });
    run("WC-08 two clocks are independent", () -> {
      AlwaysIncreasingClock a = new AlwaysIncreasingClock(), b = new AlwaysIncreasingClock();
      a.set(FUTURE);
      long ra = a.currentTimeMillis();
      long before = System.currentTimeMillis();
      long rb = b.currentTimeMillis();
      long after = System.currentTimeMillis();
      return "a=+" + (ra - FUTURE) + " bIsWall=" + (before <= rb && rb <= after) + " aState=+" + (a.get() - FUTURE);
    });
    run("WC-09 interleaved readings of two clocks", () -> {
      AlwaysIncreasingClock a = new AlwaysIncreasingClock(), b = new AlwaysIncreasingClock();
      a.set(FUTURE);
      b.set(FUTURE + 100);
      StringBuilder s = new StringBuilder();
      for (int i = 0; i < 4; i++) {
        s.append("a=+").append(a.currentTimeMillis() - FUTURE).append(" b=+").append(b.currentTimeMillis() - FUTURE).append(' ');
      }
      return s.toString().trim();
    });
    run("WC-10 set between readings", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(FUTURE);
      long r1 = c.currentTimeMillis();
      c.set(FUTURE + 50);
      long r2 = c.currentTimeMillis();
      c.set(FUTURE - 50);
      long r3 = c.currentTimeMillis();
      return "r1=+" + (r1 - FUTURE) + " r2=+" + (r2 - FUTURE) + " r3=+" + (r3 - FUTURE);
    });
    run("WC-11 the shared instance", () -> {
      WallClock w1 = WallClock$.MODULE$.AlwaysIncreasingClock();
      WallClock w2 = WallClock$.MODULE$.AlwaysIncreasingClock();
      long[] r = new long[200];
      for (int i = 0; i < r.length; i++) r[i] = (i % 2 == 0 ? w1 : w2).currentTimeMillis();
      return "same=" + (w1 == w2) + " strict=" + strictlyIncreasing(r);
    });
    run("WC-12 8 threads of 500 readings on a clock ahead of the wall time", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(FUTURE);
      String r = concurrentReport(concurrent(c, 8, 500), FUTURE);
      return r + " state=+" + (c.get() - FUTURE);
    });
    run("WC-13 8 threads of 500 readings on a new clock", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      long before = System.currentTimeMillis();
      long[][] out = concurrent(c, 8, 500);
      String r = concurrentReport(out, before);
      boolean all = true;
      for (long[] row : out) for (long x : row) all &= x >= before;
      return r.replaceAll(" (contiguous|gaps) min=.*", "") + " allAtLeastBefore=" + all;
    });
    run("WC-14 4 threads of 2000 readings on a clock ahead of the wall time", () -> {
      AlwaysIncreasingClock c = new AlwaysIncreasingClock();
      c.set(FUTURE);
      String r = concurrentReport(concurrent(c, 4, 2000), FUTURE);
      return r + " state=+" + (c.get() - FUTURE);
    });
  }
}
