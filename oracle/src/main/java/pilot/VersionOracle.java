package pilot;

import org.apache.pekko.util.Version;
import org.apache.pekko.util.Version$;

/**
 * Oracle cases of slice P67 (UTIL-VERSION): AC-UTIL-005.2 (util/Version.scala:19-202 of Pekko 2.0.0). Measured with the
 * 2.0.0 jar BEFORE any Rust was written (Q-UTIL-6: the Owner asked for the corrected 2.0.0 behaviour, measured).
 * Registered at the END of the util-core list by {@link UtilCoreOracle}.
 */
final class VersionOracle {
  private VersionOracle() {}

  private static void run(String id, java.util.concurrent.Callable<String> f) { UtilCoreOracle.run(id, f); }

  /** {@code Version(v)} outcome: {@code ok} or {@code <Exception>[<message>]}. */
  static String parse(String v) {
    try {
      Version$.MODULE$.apply(v);
      return "ok";
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  /** {@code compareTo} (raw int) or the exception of building either side. */
  static String cmp(String a, String b) {
    try {
      Version x = Version$.MODULE$.apply(a), y = Version$.MODULE$.apply(b);
      return Integer.toString(x.compareTo(y));
    } catch (Throwable t) {
      return t.getClass().getSimpleName() + "[" + t.getMessage() + "]";
    }
  }

  /** {@code [a]/[b]=compareTo eq=equals} for each {@code a|b}. */
  static String rel(String... pairs) {
    StringBuilder b = new StringBuilder();
    for (String pair : pairs) {
      int bar = pair.indexOf('|');
      String x = pair.substring(0, bar), y = pair.substring(bar + 1);
      String eq;
      try {
        eq = Boolean.toString(Version$.MODULE$.apply(x).equals(Version$.MODULE$.apply(y)));
      } catch (Throwable t) {
        eq = "-";
      }
      b.append('[').append(UtilCoreOracle.esc(x)).append("]/[").append(UtilCoreOracle.esc(y)).append("]=").append(cmp(x, y))
          .append(" eq=").append(eq).append(' ');
    }
    return b.toString().trim();
  }

  static final String[] MATRIX = {"1.2", "1.2.0", "1.2-M1", "1.2.0-M1", "1.2-M2", "1.2.0-RC1", "1.2-RC1", "1.2.1", "1.2.0-SNAPSHOT",
      "1.2.0+3-abc", "1.2.0+10-abc", "1.2.0-3-abc", "1.2.0-", "1.2.0-M", "1.2.0-m1", "1", "1-M1", "foo", "1.2.0-M1-x"};

  static void register() {
    run("VR-01 three digits (spec)", () -> rel("1.2.3|1.2.3", "1.2.3|1.2.4", "1.2.4|1.2.3", "3.2.1|1.2.3", "3.2.1|3.3.1",
        "3.2.0|3.2.1", "1.2.3|1.3.3", "1.3.3|1.2.3", "10.0.0|9.9.9", "0.0.10|0.0.9"));
    run("VR-02 more than three digits", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"1.2.3.4", "1.2.3.4.5", "1.2.3.4-M1", "1.2.3.", "1.2.3..", "1.2.3.4.", "1.2.3.x"})
        b.append('[').append(v).append("]=").append(parse(v)).append(' ');
      return b.toString().trim();
    });
    run("VR-03 two digits (spec)", () -> rel("1.2|1.2.0", "1.2.0|1.2", "1.2|1.3", "1.2.1|1.2", "1.2|1.2.1", "2.4|2.3", "3.2|3.2.7",
        "3.2.7|3.2", "1.2|1.2", "1.10|1.9"));
    run("VR-04 one digit (spec)", () -> rel("1|1.0", "1.0|1", "1|1.0.0", "1.0.0|1", "1.0|1.0.0", "3|2", "2b|2a", "2a|2b",
        "2020-09-07|2020-08-30", "2020-08-30|2020-09-07", "2|2", "10|9", "99999999999|99999999998", "99999999999|1"));
    run("VR-05 qualifier (spec)", () -> rel("1.2.3-M1|1.2.3-M1", "1.2.0-M1|1.2-M1", "1.2-M1|1.2.0-M1", "1.2-M1|1.2.0",
        "1.2.0|1.2-M1", "1.2.3-M2|1.2.3-M1", "1.2.3-M1|1.2.3-M2", "1.2.3-M1|1.2.3", "1.2.3|1.2.3-M1"));
    run("VR-06 non digits (spec)", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"1.x.3", "1.2x.3", "1.2.x", "1.2.3x", "x.3", "1.x", "1.2x", "x.y.z", "1.2.3-x", "1.2.3-1x"})
        b.append('[').append(v).append("]=").append(parse(v)).append(' ');
      return b.toString().trim();
    });
    run("VR-07 dynver (spec)", () -> rel("1.0.10+3-1234abcd|1.0.10+10-1234abcd", "1.0.10+10-1234abcd|1.0.10+3-1234abcd",
        "1.0.10+3-1234abcd|1.0.10+3-1234abcd", "1.0.10+3-1234abcd|1.0.10+3-1234abce", "1.0.10-3-1234abcd|1.0.10-10-1234abcd",
        "1.0.10-10-1234abcd|1.0.10+3-1234abcd", "1.0+3-1234abcd|1.0.0+3-1234abcd", "1.0+3-1234abcd|1.0+10-1234abcd",
        "HEAD+20140707-1030|HEAD+20140707-1030", "HEAD+20140707-1030|HEAD+20140707-1031", "HEAD+20140707-1030|1.0.0",
        "1.0.10+3-1234abcd|1.0.10", "1.0.10|1.0.10+3-1234abcd", "1.0.10-SNAPSHOT|1.0.10+3-1234abcd",
        "1.0.10+3-1234abcd|1.0.10-SNAPSHOT", "1.0.10+99999999999-abc|1.0.10+3-abc", "1.0.10+3|1.0.10+3",
        "1.0.10+3|1.0.10+4", "1.0.10+3-|1.0.10+3-a"));
    run("VR-08 no digits (spec)", () -> rel("foo|foo", "foo|bar", "bar|foo", "foo|1.2.3", "1.2.3|foo", "foo|0.0.0", "1-foo|01-foo",
        "1-foo|02-foo", "02-foo|1-foo", "2-foo|1-foo", "foo|Foo", "Foo|foo", "foo|foo1", "foo1|foo"));
    run("VR-09 qualifier matrix (raw compareTo, row vs column)", () -> {
      StringBuilder b = new StringBuilder();
      for (String a : MATRIX) {
        b.append('[').append(a).append("]:");
        for (String c : MATRIX) b.append(cmp(a, c)).append(',');
        b.append(' ');
      }
      return b.toString().trim();
    });
    run("VR-10 antisymmetry of the matrix", () -> {
      StringBuilder bad = new StringBuilder();
      int n = 0, broken = 0;
      for (String a : MATRIX)
        for (String c : MATRIX) {
          n++;
          int ab = Integer.signum(Integer.parseInt(cmp(a, c))), ba = Integer.signum(Integer.parseInt(cmp(c, a)));
          if (ab != -ba) {
            broken++;
            bad.append('[').append(a).append("]/[").append(c).append("] ");
          }
        }
      return "pairs=" + n + " broken=" + broken + " " + bad.toString().trim();
    });
    run("VR-11 qualifier on one side only", () -> rel("1.2.0|1.2.0-M1", "1.2.0-M1|1.2.0", "1.2.0|1.2-M1", "1.2-M1|1.2.0",
        "1.2.0|1.2.0-RC1", "1.2.0-RC1|1.2.0", "1.2.0|1.2.0-SNAPSHOT", "1.2.0-SNAPSHOT|1.2.0", "1.2|1.2-M1", "1.2-M1|1.2", "1|1-M1",
        "1-M1|1", "1.2.3|1.2.3-M1", "1.2.3-M1|1.2.3", "1.2.3|1.2.3-", "1.2.3-|1.2.3", "2.0.0|1.9.9-M1", "1.9.9-M1|2.0.0",
        "1.2.0-M1|1.2.1", "1.2.1|1.2.0-M1"));
    run("VR-12 qualifier text order (raw String.compareTo)", () -> rel("1.2.0-M1|1.2.0-M2", "1.2.0-M2|1.2.0-M1",
        "1.2.0-M1|1.2.0-M10", "1.2.0-M10|1.2.0-M1", "1.2.0-M9|1.2.0-M10", "1.2.0-M1|1.2.0-RC1", "1.2.0-RC1|1.2.0-M1",
        "1.2.0-a|1.2.0-B", "1.2.0-B|1.2.0-a", "1.2.0-abc|1.2.0-abd", "1.2.0-ab|1.2.0-abcd", "1.2.0-abcd|1.2.0-ab",
        "1.2.0-é|1.2.0-e", "1.2.0-😀|1.2.0-￿", "1.2.0-￿|1.2.0-😀", "1.2.0-M1|1.2.0-m1"));
    run("VR-13 equals and hashCode", () -> {
      StringBuilder b = new StringBuilder();
      String[] vs = {"1.2", "1.2.0", "1", "1.0", "1.0.0", "1.2-M1", "1.2.0-M1", "1.2.3", "foo", "bar", "1.0.10+3-abc", "1.0.10+10-abc",
          "0.0.0", "0", "", "x"};
      for (String v : vs) {
        if (v.isEmpty()) continue;
        b.append('[').append(v).append("]=").append(Version$.MODULE$.apply(v).hashCode()).append(' ');
      }
      b.append("zero=").append(Version$.MODULE$.Zero().hashCode()).append(' ').append(Version$.MODULE$.Zero().version());
      return b.toString().trim();
    });
    run("VR-14 hashCode agrees with equals", () -> {
      StringBuilder b = new StringBuilder();
      int pairs = 0, disagree = 0;
      for (String x : MATRIX)
        for (String y : MATRIX) {
          Version vx = Version$.MODULE$.apply(x), vy = Version$.MODULE$.apply(y);
          pairs++;
          if (vx.equals(vy) && vx.hashCode() != vy.hashCode()) {
            disagree++;
            b.append('[').append(x).append("]/[").append(y).append("] ");
          }
        }
      return "pairs=" + pairs + " equal-but-different-hash=" + disagree + " " + b.toString().trim();
    });
    run("VR-15 malformed strings", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"", ".", "..", "...", "1.", "1..", ".1", "..1", "1..2", ".1.2", "1.2.", " 1.2", "1.2 ", "1. 2", "-1", "+1",
          "1.-2", "1.+2", "-1.2", "-1.2.3", "1.-2.3", "1.2.-3", "1.2.+3", "1.2.3-", "1.2.3+", "1.2.3-+", "1.2.3+-", "1.2-", "1.2+", "1-", "1+",
          "1.", "a", "a.b", "a.b.c", "1.2.3-M1+5", "1.2.3+5-M1", "1.2.3+5-", "1.2.3+-5-x", "1.2.3--5-x", "1.2.3-5", "1.2.3-5-", "1.2.3-5x-y",
          "1.2.3-x-5", "v1.2.3", "1.2.3\n", "\n", " ", "1.2.3 "})
        b.append('[').append(UtilCoreOracle.esc(v)).append("]=").append(UtilCoreOracle.esc(parse(v))).append(' ');
      return b.toString().trim();
    });
    run("VR-16 numbers out of int range", () -> rel("2147483647.0.0|1.0.0", "2147483648.0.0|1.0.0", "1.2147483648.0|1.0.0",
        "1.0.2147483648|1.0.0", "1.2.3-2147483648-x|1.2.3", "2147483647|2147483646", "2147483648|2147483647",
        "2147483648|2147483649", "99999999999|99999999999", "99999999999.1.1|1.1.1", "1.2.99999999999-M1|1.2.1", "1.99999999999-M1|1.2",
        "1.2+2147483648-x|1.2"));
    run("VR-17 negative and wrapping numbers", () -> rel("-1.2.3|1.2.3", "1.2.3|-1.2.3", "-1.2.3|-1.2.3", "-1.2.3|-2.2.3",
        "2147483647.0.0|-1.0.0", "-1.0.0|2147483647.0.0", "-2147483648.0.0|2147483647.0.0", "2147483647.0.0|-2147483648.0.0",
        "1.2147483647.0|1.-1.0", "1.-1.0|1.2147483647.0", "1.2.-3|1.2.3", "1.2.3|1.2.-3", "1.2.-2147483648|1.2.2147483647",
        "1.2.2147483647|1.2.-2147483648"));
    run("VR-18 digits outside ASCII", () -> rel("٢.٥.٥|2.5.5", "2.5.5|٢.٥.٥", "2.5.٥|2.5.5",
        "２.５.５|2.5.5", "٢|2", "2|٢", "٢b|2b", "٢b|٣b", "1.2.3-M١|1.2.3-M1", "1.٢|1.2",
        "1.٢-M1|1.2-M1"));
    run("VR-19 equal strings short-circuit", () -> rel("1.2.3|1.2.3", "foo|foo", "1.2.3-M1|1.2.3-M1", "99999999999|99999999999",
        "1.2.0+3-abc|1.2.0+3-abc", "-1.2.3|-1.2.3"));
    run("VR-20 version field and Zero", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"1.2", "1.2.0-M1", "foo", "2020-09-07", "1.0.10+3-abc"})
        b.append('[').append(Version$.MODULE$.apply(v).version()).append("]=[").append(Version$.MODULE$.apply(v)).append("] ");
      b.append("zero=[").append(Version$.MODULE$.Zero()).append("] 1.2>zero=").append(cmp("1.2", "0.0.0")).append(" zero>foo=")
          .append(cmp("0.0.0", "foo")).append(" 0=zero:").append(cmp("0", "0.0.0"));
      return b.toString().trim();
    });
    run("VR-21 public constructor parses lazily", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"1.x", "1.2.3.4", "", "1.2.3"}) {
        Version x = new Version(v);
        String same;
        try { same = Integer.toString(x.compareTo(new Version(v))); } catch (Throwable t) { same = t.getClass().getSimpleName(); }
        String other;
        try { other = Integer.toString(x.compareTo(new Version("1.2.3-zz"))); } catch (Throwable t) { other = t.getClass().getSimpleName() + "[" + t.getMessage() + "]"; }
        String h;
        try { h = Integer.toString(new Version(v).hashCode()); } catch (Throwable t) { h = t.getClass().getSimpleName(); }
        b.append('[').append(v).append("] same=").append(same).append(" other=").append(other).append(" hash=").append(h).append(' ');
      }
      return b.toString().trim();
    });
    run("VR-22 parse exceptions", () -> {
      StringBuilder b = new StringBuilder();
      for (String v : new String[] {"", "1.2.3.4", "1.x.3", "1.2x", "x.3", "1.2.3x", "1.", ".", "1.2.3.4.5", "99999999999.1", "1.99999999999.1"}) {
        try {
          Version$.MODULE$.apply(v);
          b.append('[').append(v).append("]=ok ");
        } catch (Throwable t) {
          b.append('[').append(v).append("]=").append(t.getClass().getName()).append(" cause=").append(t.getCause()).append(" msg=[")
              .append(t.getMessage()).append("] ");
        }
      }
      return b.toString().trim();
    });
    run("VR-23 sorting", () -> {
      java.util.List<Version> l = new java.util.ArrayList<>();
      for (String v : new String[] {"1.2.0", "1.2.0-RC1", "1.2.0-M2", "1.2.0-M1", "1.1.9", "1.2.1", "1.2.0+3-abc", "1.2.0+10-abc", "1.2",
          "1.2-M1", "1.10.0", "1.9.0"})
        l.add(Version$.MODULE$.apply(v));
      java.util.Collections.sort(l);
      StringBuilder b = new StringBuilder();
      for (Version v : l) b.append(v).append(' ');
      Version max = java.util.Collections.max(l), min = java.util.Collections.min(l);
      return b.toString().trim() + " max=" + max + " min=" + min;
    });
  }
}
