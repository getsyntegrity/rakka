package pilot;

import java.time.Duration;
import org.apache.pekko.util.Timeout;

/**
 * Oracle cases of slice P77 (UTIL-FRONTERAS): AC-UTIL-018.2 (JavaDurationConverters, removed in Pekko 2.0.0 in favour
 * of {@code scala.jdk.DurationConverters}). The only converter whose result a crate user can observe is the one
 * {@code Timeout.create(java.time.Duration)} goes through; these cases fix its boundaries (whole seconds against
 * seconds-and-nanos, the shape of the overflow message, the zero duration). FutureConverters, OptionConverters,
 * FunctionConverters and ccompat (018.1, 018.3) have no runtime behaviour of their own in 2.0.0. Registered at the END
 * of the util-core list by {@link UtilCoreOracle}.
 */
final class FronterasOracle {
  private FronterasOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  private static String created(Duration d) { return UtilCoreOracle.nanos(Timeout.create(d)); }

  static void register() {
    run("FR-DUR-01 create(Duration.ZERO)", () -> created(Duration.ZERO));
    run("FR-DUR-02 create(ofNanos(1))", () -> created(Duration.ofNanos(1)));
    run("FR-DUR-03 create(ofSeconds(0, 999999999))", () -> created(Duration.ofSeconds(0, 999_999_999)));
    run("FR-DUR-04 create(ofSeconds(1))", () -> created(Duration.ofSeconds(1)));
    run("FR-DUR-05 create(ofSeconds(86400, 5))", () -> created(Duration.ofSeconds(86_400, 5)));
    run("FR-DUR-06 create(ofSeconds(9223372036))", () -> created(Duration.ofSeconds(9_223_372_036L)));
    run("FR-DUR-07 create(ofSeconds(9223372037))", () -> created(Duration.ofSeconds(9_223_372_037L)));
    run("FR-DUR-08 create(ofSeconds(9223372035, 999999999))", () -> created(Duration.ofSeconds(9_223_372_035L, 999_999_999)));
    run("FR-DUR-09 create(ofSeconds(9223372037, 1))", () -> created(Duration.ofSeconds(9_223_372_037L, 1)));
    run("FR-DUR-10 create(ofSeconds(3600, 500000000))", () -> created(Duration.ofSeconds(3_600, 500_000_000)));
    run("FR-DUR-11 create(ofSeconds(Long.MAX_VALUE, 999999999))", () -> created(Duration.ofSeconds(Long.MAX_VALUE, 999_999_999)));
    run("FR-DUR-12 create(ofSeconds(61, 1))", () -> created(Duration.ofSeconds(61, 1)));
    run("FR-DUR-13 create(ofSeconds(9223372036, 854775807)) eq Timeout(Long.MaxValue ns)", () -> {
      Timeout a = Timeout.create(Duration.ofSeconds(9_223_372_036L, 854_775_807));
      Timeout b = Timeout.apply(Long.MAX_VALUE, java.util.concurrent.TimeUnit.NANOSECONDS);
      return "eq=" + a.equals(b) + " " + UtilCoreOracle.nanos(a);
    });
  }
}
