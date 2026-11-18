package pilot;

import org.apache.pekko.PekkoVersion$;
import org.apache.pekko.UnsupportedPekkoVersion;

/**
 * Oracle cases of slice P66 (UTIL-PEKKOVERSION): AC-UTIL-005.1 (PekkoVersion.scala:19-83, Version.current from
 * project/VersionGenerator.scala:20-40). Registered at the END of the util-core list by {@link UtilCoreOracle}.
 */
final class PekkoVersionOracle {
  private PekkoVersionOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  /** Outcome of the three-argument require: {@code ok}, or {@code <Exception>[<message>]}. */
  static String outcome(String library, String required, String current) {
    try {
      PekkoVersion$.MODULE$.require(library, required, current);
      return "ok";
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  /** Several {@code required|current} pairs, each reported as {@code [required]/[current]=outcome}. */
  static String pairs(String... pairs) {
    StringBuilder b = new StringBuilder();
    for (String pair : pairs) {
      int bar = pair.indexOf('|');
      String required = pair.substring(0, bar), current = pair.substring(bar + 1);
      b.append('[').append(UtilCoreOracle.esc(required)).append("]/[").append(UtilCoreOracle.esc(current)).append("]=")
          .append(outcome("lib", required, current)).append(' ');
    }
    return b.toString().trim();
  }

  static void register() {
    run("PV-01 current version", () -> {
      String conf;
      try (java.io.InputStream in = PekkoVersion$.class.getClassLoader().getResourceAsStream("version.conf")) {
        conf = new String(in.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
      }
      return "current=[" + org.apache.pekko.Version$.MODULE$.current() + "] conf=[" + UtilCoreOracle.esc(conf) + "]";
    });
    run("PV-02 spec: satisfied", () -> pairs(
        "2.5.6|2.5.6", "2.5.6|2.5.7", "2.5.6|2.6.0", "2.5.6|2.5.7-RC10", "2.6.0-RC1|2.6.0-RC1", "2.6.0-RC1|2.6.0-RC2",
        "2.6.0-RC1|2.7.0", "2.5.6|2.5.7-M10", "2.5.7-M9|2.5.7-M9", "2.5.7-M9|2.5.7-M10", "2.5.7-M9|2.5.8",
        "2.5.7-M9|2.5.7-RC1"));
    run("PV-03 spec: unsatisfied", () -> pairs(
        "2.5.6|2.5.6-RC1", "2.5.6-RC2|2.5.6-RC1", "2.5.6|2.5.6-M1", "2.5.7-M10|2.5.7-M9", "2.5.7-RC1|2.5.7-M9",
        "2.5.6|3.0.0", "2.5.6|1.0.0", "2.5.6|2.4.19", "2.5.6|2.5.5"));
    run("PV-04 spec: snapshot and nonsense", () -> pairs(
        "2.5.6|2.5-SNAPSHOT", "2.5.6|2.5-20180109-133700", "2.5-SNAPSHOT|2.5-SNAPSHOT",
        "2.5-20180109-133700|2.5-20180109-133700", "2.5.6|nonsense", "nonsense|2.5.6", "nonsense|nonsense",
        "2.5-SNAPSHOT|2.5.6", "2.5.6|2.5.6-SNAPSHOT", "2.5.6-SNAPSHOT|2.5.6"));
    run("PV-05 messages", () -> {
      StringBuilder b = new StringBuilder();
      for (String lib : new String[] {"my-library", "", "café €", "[x]", "a\nb"})
        b.append(outcome(lib, "2.5.6", "2.5.5")).append(' ');
      b.append(outcome("lib", "2.5.6-M1", "2.5.6-M1".replace("M1", "M0"))).append(' ');
      b.append(outcome("lib", "2.5.6-RC3", "2.5.6-M9"));
      return b.toString();
    });
    run("PV-06 pre-release matrix", () -> {
      String[] kinds = {"", "-M1", "-M2", "-RC1", "-RC2"};
      StringBuilder b = new StringBuilder();
      for (String required : kinds)
        for (String current : kinds)
          b.append(outcome("lib", "2.5.6" + required, "2.5.6" + current).startsWith("ok") ? '1' : '0');
      return b.toString();
    });
    run("PV-07 numbers compared as text then as ints", () -> pairs(
        "2.5.6|2.05.6", "2.05.6|2.5.6", "2.5.6|2.5.06", "2.5.06|2.5.6", "02.5.6|2.5.6", "2.5.6|02.5.6", "2.6.0|2.05.9",
        "2.05.9|2.5.8", "2.5.007|2.5.7", "2.5.7|2.5.0007", "2.5.7|2.5.007-M1", "2.5.7-M01|2.5.7-M1", "2.5.7-M2|2.5.7-M02",
        "2.5.7-M10|2.5.7-M9", "2.5.7-M9|2.5.7-M10", "2.5.2147483647|2.5.2147483646", "2.5.2147483647|2.5.2147483647"));
    run("PV-08 numbers out of int range", () -> pairs(
        "2.5.6|2.5.99999999999", "2.5.99999999999|2.5.6", "2.99999999999.0|2.5.0", "2.5.0|2.99999999999.0",
        "99999999999.5.6|99999999999.5.7", "99999999999.5.6|99999999998.5.7", "2.5.99999999999|2.5.99999999999",
        "2.5.99999999999|2.5.99999999998", "2.5.6-M99999999999|2.5.6-M1", "2.5.6-M1|2.5.6-M99999999999",
        "2.5.6-RC99999999999|2.5.6-RC99999999999", "2.5.6-RC99999999999|2.5.6-RC99999999998",
        "2.5.2147483648|2.5.1", "2.5.1|2.5.2147483648", "2.2147483648.1|2.1.1", "2.5.6-M2147483648|2.5.6-M1"));
    run("PV-09 strings that are not a version", () -> pairs(
        "2.5.6| 2.5.6", "2.5.6|2.5.6 ", " 2.5.6|2.5.5", "2.5.6 |2.5.5", "v2.5.6|2.5.5", "2.5.6|v2.5.5", "2.5.6-|2.5.5",
        "2.5.6-M|2.5.5", "2.5.6-m1|2.5.5", "2.5.6-rc1|2.5.5", "2.5.6-RC|2.5.5", "2.5.6.1|2.5.5", "2.5|2.5.5",
        "2.5.|2.5.5", "..|2.5.5", "|2.5.5", "2.5.6|", "|", "2.5.6-M1-M1|2.5.5", "2.5.6-M1x|2.5.5", "2.5.6\n|2.5.5",
        "2.5.6|2.5.5\n", "2.5.6-RC1+3|2.5.5", "2.5.6+3|2.5.5", "-2.5.6|2.5.5", "2.-5.6|2.5.5", "2.5.6-M-1|2.5.5",
        "2.5.6-M+1|2.5.5"));
    run("PV-10 digits outside ASCII", () -> pairs(
        "2.5.6|٢.٥.٥", "٢.٥.٦|2.5.5", "2.5.6|2.5.٥", "2.5.٦|2.5.5",
        "2.5.6|２.５.５", "2.5.6-M١|2.5.6-M1", "2.5.6-M١|2.5.6"));
    run("PV-11 equal strings are never checked", () -> pairs(
        "2.5.6|2.5.6", "2.5.6-M1|2.5.6-M1", "nonsense|nonsense", "|", "99999999999.9.9|99999999999.9.9",
        "2.5.6-M99999999999|2.5.6-M99999999999"));
    run("PV-12 public require against the current version", () -> {
      StringBuilder b = new StringBuilder();
      for (String required : new String[] {"2.0.0", "1.7.1", "2.0.1", "2.1.0", "3.0.0", "1.0.0", "2.0.0-RC1", "2.0.0-M1", "2.0-SNAPSHOT", "nonsense", "1.9.9", "2.0.0-RC99999999999"}) {
        String r;
        try {
          PekkoVersion$.MODULE$.require("my-lib", required);
          r = "ok";
        } catch (Throwable t) {
          r = t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
        }
        b.append('[').append(required).append("]=").append(r).append(' ');
      }
      return b.toString().trim();
    });
    run("PV-13 the exception", () -> {
      try {
        PekkoVersion$.MODULE$.require("lib", "9.0.0", "2.0.0");
        return "no exception";
      } catch (UnsupportedPekkoVersion e) {
        return "class=" + e.getClass().getName() + " super=" + e.getClass().getSuperclass().getName() + " final="
            + java.lang.reflect.Modifier.isFinal(e.getClass().getModifiers()) + " cause=" + e.getCause() + " toString=["
            + e + "] suppressed=" + e.getSuppressed().length;
      }
    });
    run("PV-14 many required versions against 2.0.0", () -> {
      StringBuilder b = new StringBuilder();
      for (int major = 1; major <= 3; major++)
        for (int minor = 0; minor <= 2; minor++)
          for (int patch = 0; patch <= 2; patch++)
            for (String pre : new String[] {"", "-M1", "-RC1"})
              b.append(outcome("l", major + "." + minor + "." + patch + pre, "2.0.0").startsWith("ok") ? '1' : '0');
      return b.toString();
    });
  }
}
