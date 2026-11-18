package pilot;

import org.apache.pekko.*;
import org.apache.pekko.util.*;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.concurrent.TimeUnit;

/** Oracle cases ported from the deprecated jshell script (Q-X-12); registered in order, run by id. */
final class UtilCoreOracle {
  private UtilCoreOracle() {}

  // Helpers.timestamp captures the default zone when Helpers loads: pin a fixed zone before anything touches it (slice P06).
  static { System.setProperty("user.timezone", "GMT+05:30"); }

  static final java.util.Map<String, Callable<String>> CASES = new java.util.LinkedHashMap<>();

  static void run(String id, Callable<String> f) {
    if (CASES.put(esc(id), f) != null) throw new IllegalStateException("duplicate case id " + id);
  }

  /** Case ids in registration order (ASCII-escaped). */
  static java.util.List<String> ids() {
    return new java.util.ArrayList<>(CASES.keySet());
  }

  /** The oracle line of one case: {@code <id> -> ok <fields>} or {@code <id> -> ERR <Exception> [<message>]}. */
  static String line(String escapedId) {
    Callable<String> f = CASES.get(escapedId);
    if (f == null) throw new IllegalArgumentException("unknown case id " + escapedId);
    String r;
    try {
      r = "ok " + f.call();
    } catch (Throwable t) {
      r = "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]";
    }
    return esc(unesc(escapedId) + " -> " + r);
  }


  static String esc(String s) {
    StringBuilder b = new StringBuilder();
    for (char c : s.toCharArray()) {
      if (c < 0x20 || c > 0x7e) b.append(String.format("\\u%04X", (int) c));
      else b.append(c);
    }
    return b.toString();
  }

  static String unesc(String s) {
    StringBuilder b = new StringBuilder();
    for (int i = 0; i < s.length(); i++) {
      if (s.startsWith("\\u", i) && i + 6 <= s.length() && s.substring(i + 2, i + 6).matches("[0-9A-F]{4}")) {
        b.append((char) Integer.parseInt(s.substring(i + 2, i + 6), 16));
        i += 5;
      } else {
        b.append(s.charAt(i));
      }
    }
    return b.toString();
  }


  static String globPairs(String... pairs) {
    StringBuilder out = new StringBuilder();
    for (String pair : pairs) {
      int bar = pair.indexOf('|');
      String pat = pair.substring(0, bar), in = pair.substring(bar + 1);
      String shownPat = pat.length() > 40 ? pat.length() + "x" : esc(pat);
      String shownIn = in.length() > 40 ? in.length() + "x" : esc(in);
      out.append('[').append(shownPat).append("]/[").append(shownIn).append("]=").append(Glob$.MODULE$.matches(pat, in)).append(' ');
    }
    return out.toString().trim();
  }

  static java.util.List<String> allStrings(String alphabet, int length) {
    java.util.List<String> all = new java.util.ArrayList<>();
    all.add("");
    for (int i = 0; i < length; i++) {
      java.util.List<String> next = new java.util.ArrayList<>();
      for (String prefix : all) for (char c : alphabet.toCharArray()) next.add(prefix + c);
      all = next;
    }
    return all;
  }

  static final Helpers$ H = Helpers$.MODULE$;

  static String nanos(Timeout t) { return "nanos=" + t.duration().toNanos(); }

  static String b64(long l) { return "[" + H.base64(l, H.base64$default$2()) + "]"; }

  static scala.Function0<Object> f0(Object v) {
    return new scala.runtime.AbstractFunction0<Object>() { public Object apply() { return v; } };
  }

  static scala.Function1<Object, Object> pred(java.util.function.Predicate<Object> p) {
    return new scala.runtime.AbstractFunction1<Object, Object>() { public Object apply(Object x) { return p.test(x); } };
  }

  static String cause(Throwable e) { return e.getCause() == null ? "none" : "[" + e.getCause().getMessage() + "]"; }

  static String exc(Throwable e) {
    return "class=[" + e.getClass().getSimpleName() + "] msg=[" + e.getMessage() + "] cause=" + cause(e);
  }

  static String[] lc = {
    "Hello WORLD 123", "TITLE", "", "ПРИВЕТ", "ＡＢ",
    "𐐀", "ß", "İ", "ΑΣ", "ΑΣΑ", "Σ"
  };

  static final Object RM = module();
  static Object module() {
    try {
      return Class.forName("org.apache.pekko.util.Helpers$Requiring$").getField("MODULE$").get(null);
    } catch (ReflectiveOperationException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  static Object req(Object v, Object cond, scala.Function0<Object> msg) throws Exception {
    Class<?> c = cond instanceof Boolean ? boolean.class : scala.Function1.class;
    try {
      return RM.getClass().getMethod("requiring$extension", Object.class, c, scala.Function0.class).invoke(RM, v, cond, msg);
    } catch (java.lang.reflect.InvocationTargetException e) {
      if (e.getCause() instanceof Exception) throw (Exception) e.getCause();
      throw (Error) e.getCause();
    }
  }

  static {
    // Oracle for unit AC-UTIL (core part): prints how Apache Pekko 2.0.0 behaves for
    // Timeout, Helpers (base64, toRootLowerCase, Requiring), Done/NotUsed and the base
    // exceptions (PekkoException, ConfigurationException).
    // Run: oracle/run.sh util-core-all
    //
    // Output contract (one deterministic line per case):
    //   <CASE-ID> -> ok <fields>
    //   <CASE-ID> -> ERR <ExceptionSimpleName> [<message verbatim>]
    // Every char outside printable ASCII is written as \\uXXXX (UTF-16 code unit), so
    // the file is pure ASCII. No identity hashes and no timestamps.
    // Case-id prefixes: TO (Timeout), B64 (Helpers.base64), LC (toRootLowerCase),
    // RQ (Requiring), DN (Done, NotUsed), EX (PekkoException, ConfigurationException).
    // Cases marked PENDING-Qxx in their id comment describe behavior the Rust port
    // does not reproduce yet (open Owner question); the golden keeps them.
    
    
    // ---- Timeout (AC-UTIL-001): Timeout.scala:23-55
    run("TO-01 Timeout.zero", () -> nanos(Timeout.zero()));
    for (TimeUnit u : TimeUnit.values()) {
      run("TO-02 Timeout(3, " + u + ")", () -> nanos(Timeout.apply(3L, u)));
      long max = u.convert(Long.MAX_VALUE, TimeUnit.NANOSECONDS);
      run("TO-MAX " + u, () -> "length=" + max + " " + nanos(Timeout.apply(max, u)));
      // Rust takes a u64: for NANOSECONDS the length past the bound is 2^63, which is Long.MIN_VALUE here.
      run("TO-OVER " + u, () -> nanos(Timeout.apply(max + 1, u)));
    }
    run("TO-03 Timeout(0, DAYS)", () -> nanos(Timeout.apply(0L, TimeUnit.DAYS)));
    run("TO-04 Timeout(1500, MILLISECONDS)", () -> nanos(Timeout.apply(1500L, TimeUnit.MILLISECONDS)));
    run("TO-05 Timeout(Long.MaxValue, DAYS)", () -> nanos(Timeout.apply(Long.MAX_VALUE, TimeUnit.DAYS)));
    run("TO-06 eq Timeout(1000 ms) Timeout(1 s)", () -> {
      Timeout a = Timeout.apply(1000L, TimeUnit.MILLISECONDS), b = Timeout.apply(1L, TimeUnit.SECONDS);
      return "eq=" + a.equals(b) + " hashEq=" + (a.hashCode() == b.hashCode());
    });
    run("TO-07 ne Timeout(1 s) Timeout(2 s)", () -> "eq=" + Timeout.apply(1L, TimeUnit.SECONDS).equals(Timeout.apply(2L, TimeUnit.SECONDS)));
    run("TO-08 create(ofMillis(250))", () -> nanos(Timeout.create(java.time.Duration.ofMillis(250))));
    run("TO-09 create(ofSeconds(9223372036, 854775807))", () -> nanos(Timeout.create(java.time.Duration.ofSeconds(9223372036L, 854775807))));
    // PENDING-Q-UTIL-1: a negative timeout is valid in Pekko; std Duration has no negatives.
    run("TO-NEG-01 Timeout(-5, MILLISECONDS)", () -> nanos(Timeout.apply(-5L, TimeUnit.MILLISECONDS)));
    run("TO-NEG-02 Timeout(Long.MinValue, NANOSECONDS)", () -> nanos(Timeout.apply(Long.MIN_VALUE, TimeUnit.NANOSECONDS)));
    run("TO-NEG-03 create(ofMillis(-250))", () -> nanos(Timeout.create(java.time.Duration.ofMillis(-250))));
    // PENDING-Q-UTIL-1: Pekko cannot build more than 2^63-1 ns through create; Rust Timeout::new accepts them.
    run("TO-NEG-04 create(ofSeconds(9223372036, 854775808))", () -> nanos(Timeout.create(java.time.Duration.ofSeconds(9223372036L, 854775808))));
    run("TO-NEG-05 create(ofSeconds(Long.MAX_VALUE))", () -> nanos(Timeout.create(java.time.Duration.ofSeconds(Long.MAX_VALUE))));
    // PENDING-Q-UTIL-2: toString keeps the unit the duration was built with.
    run("TO-STR-01 Timeout(100, MILLISECONDS)", () -> "str=[" + Timeout.apply(100L, TimeUnit.MILLISECONDS) + "]");
    run("TO-STR-02 Timeout.zero", () -> "str=[" + Timeout.zero() + "]");
    run("TO-STR-03 Timeout(1000, MILLISECONDS)", () -> "str=[" + Timeout.apply(1000L, TimeUnit.MILLISECONDS) + "]");
    
    // ---- Helpers.base64 (AC-UTIL-002): Helpers.scala:115-123
    for (long l : new long[] { 0, 1, 25, 26, 51, 52, 61, 62, 63, 64, 65, 4095, 4096, 1L << 32, -1, -64, Long.MIN_VALUE, Long.MAX_VALUE })
      run("B64 " + l, () -> b64(l));
    run("B64-SB prefix x, 5", () -> "[" + H.base64(5L, new StringBuilder("x")) + "]");
    run("B64-SB prefix temp/$, 64", () -> "[" + H.base64(64L, new StringBuilder("temp/$")) + "]");
    run("B64-SB empty prefix, 0", () -> "[" + H.base64(0L, new StringBuilder()) + "]");
    
    // ---- Helpers.toRootLowerCase (AC-UTIL-002): Helpers.scala:40. Only chars older than Unicode 10.
    for (String s : lc) run("LC " + esc(s), () -> "[" + esc(H.toRootLowerCase(s)) + "]");
    
    // ---- Helpers.Requiring (AC-UTIL-002): Helpers.scala:143-168
    // Helpers$Requiring$ has no source-level name: reach the module and its methods by reflection and
    // rethrow the real exception, so class and message are Pekko's.
    run("RQ-01 5.requiring(true)", () -> "value=" + req(5, true, f0("unused")));
    run("RQ-02 5.requiring(false, msg)", () -> "value=" + req(5, false, f0("idle-cpu-level must be between 1 and 10")));
    run("RQ-03 1.requiring(false, 42)", () -> "value=" + req(1, false, f0(42)));
    run("RQ-04 1.requiring(false, empty)", () -> "value=" + req(1, false, f0("")));
    run("RQ-05 3.requiring(_ > 0)", () -> "value=" + req(3, pred(x -> ((Integer) x) > 0), f0("must be > 0")));
    run("RQ-06 0.requiring(_ > 0)", () -> "value=" + req(0, pred(x -> ((Integer) x) > 0), f0("nr-of-selectors must be > 0")));
    run("RQ-07 abc.requiring(len == 3)", () -> "value=" + req("abc", pred(x -> ((String) x).length() == 3), f0("len")));
    // PENDING-Q-UTIL-3: a null message prints as "null"; Rust has no null.
    run("RQ-NULL 1.requiring(false, null)", () -> "value=" + req(1, false, f0(null)));
    // PENDING-Q-UTIL-3: msg is by-name, evaluated only when the condition fails.
    run("RQ-LAZY msg not evaluated when cond holds", () -> {
      int[] calls = {0};
      req(1, true, new scala.runtime.AbstractFunction0<Object>() { public Object apply() { calls[0]++; return "m"; } });
      return "calls=" + calls[0];
    });
    
    // ---- Done, NotUsed (AC-UTIL-003): Done.scala:25-40, NotUsed.scala:22-37
    run("DN-01 Done", () -> "str=[" + Done$.MODULE$ + "] eqSelf=" + Done$.MODULE$.equals(Done$.MODULE$.getInstance()));
    run("DN-02 NotUsed", () -> "str=[" + NotUsed$.MODULE$ + "] eqSelf=" + NotUsed$.MODULE$.equals(NotUsed$.MODULE$.getInstance()));
    run("DN-03 Done.done() is Done.getInstance()", () -> "same=" + (Done$.MODULE$.done() == Done$.MODULE$.getInstance()));
    
    // ---- PekkoException, ConfigurationException (AC-UTIL-004): PekkoException.scala:19-43
    run("EX-01 PekkoException(boom)", () -> exc(new PekkoException("boom")));
    run("EX-02 PekkoException(empty)", () -> exc(new PekkoException("")));
    run("EX-03 PekkoException(boom, cause)", () -> exc(new PekkoException("boom", new IllegalStateException("inner"))));
    run("EX-04 ConfigurationException(bad)", () -> exc(new ConfigurationException("bad")));
    run("EX-05 ConfigurationException(bad, cause)", () -> exc(new ConfigurationException("bad", new IllegalStateException("why"))));
    run("EX-06 PekkoException(msg, null)", () -> exc(new PekkoException("m", null)));
    // PENDING-Q-UTIL-4: ConfigurationException extends PekkoException.
    run("EX-SUB ConfigurationException isA PekkoException", () -> "isA=" + (((Object) new ConfigurationException("x")) instanceof PekkoException));
    // PENDING-Q-UTIL-3: a null message.
    run("EX-NULL PekkoException(null)", () -> exc(new PekkoException(null)));
    

    // ---- Slice P02 (2026-11-18): gaps found when comparing the rows with the recorded cases
    run("TO-06N eq Timeout(-1000 ms) Timeout(-1 s)", () -> {
      Timeout a = Timeout.apply(-1000L, TimeUnit.MILLISECONDS), b = Timeout.apply(-1L, TimeUnit.SECONDS);
      return "eq=" + a.equals(b) + " hashEq=" + (a.hashCode() == b.hashCode());
    });
    for (String s2 : new String[] {"\u1C90", "\uA7C0"}) run("LC " + esc(s2), () -> "[" + esc(H.toRootLowerCase(s2)) + "]");
    run("EX-TS-01 PekkoException(boom).toString", () -> "str=[" + new PekkoException("boom") + "]");
    run("EX-TS-02 ConfigurationException(bad, cause).toString", () -> "str=[" + new ConfigurationException("bad", new IllegalStateException("why")) + "]");
    run("EX-TS-03 PekkoException(empty).toString", () -> "str=[" + new PekkoException("") + "]");

    // ---- Slice P05 (2026-11-18): Glob (AC-UTIL-016.6): Glob.scala:38-73, GlobSpec.scala:35-91
    run("GL-01 literals", () -> globPairs("abc|abc", "abc|abd", "|", "|a", "a|"));
    run("GL-02 question mark", () -> globPairs("a?c|abc", "a?c|ac", "a?c|abbc", "???|abc"));
    run("GL-03 star", () -> globPairs("*|", "*|anything", "a*|a", "a*c|abbbc", "a*c|abbbd", "**|ab", "*b*|abc"));
    run("GL-04 star positions", () -> globPairs("*a|ba", "*a|ab", "a*a|aa", "a*a|a", "*ab*|xxabyy", "*ab*|xxaybb", "?*|", "?*|a", "*?|a", "a**b|ab", "*?*?*|ab", "*?*?*|a"));
    // ? and * consume one UTF-16 code unit (Q-UTIL-8): a code point outside the BMP is two units, and line breaks are ordinary characters.
    String astral = new String(Character.toChars(0x1F600));
    run("GL-U16-01 ? vs astral", () -> globPairs("?|" + astral, "??|" + astral, "???|" + astral, "*|" + astral, "a?b|a" + astral + "b", "a??b|a" + astral + "b"));
    run("GL-U16-02 literal astral", () -> globPairs(astral + "|" + astral, "?" + astral + "|x" + astral, "*" + astral + "|ab" + astral, astral + "*|" + astral + "ab", "?|\u00e9", "\u00e9|\u00e9", "\u00e9?|\u00e9a"));
    run("GL-U16-03 line breaks", () -> globPairs("a?b|a\nb", "a*b|a\r\nb", "*|\n", "?|\n", "a?b|a\u2028b", "a?|a\u0085"));
    run("GL-U16-04 case and metachars", () -> globPairs("A|a", "a.c|abc", "a.c|a.c", "[a]|a", "[a]|[a]", "a\\|a\\", "\\Q*\\E|\\Qxx\\E", "(a)|(a)", "a+|aa"));
    // 0-4 character patterns over {a,b,*,?} against every 0-4 character input over {a,b}: GlobSpec.scala:60-73 without the regex side.
    for (int len = 0; len <= 4; len++) {
      final int n = len;
      run("GL-EXH patterns of length " + n, () -> {
        StringBuilder out = new StringBuilder();
        for (String pat : allStrings("ab*?", n)) {
          out.append('[').append(pat).append("]=");
          for (int k = 0; k <= 4; k++) for (String in : allStrings("ab", k)) out.append(Glob$.MODULE$.matches(pat, in) ? 'T' : 'F');
          out.append(' ');
        }
        return out.toString().trim();
      });
    }
    // GlobSpec.scala:75-90: no catastrophic backtracking; the answer is what is recorded, speed is checked by the Rust test.
    run("GL-PERF 12 stars vs 36 a", () -> globPairs(("*a".repeat(12) + "*b") + "|" + "a".repeat(36), ("*a".repeat(12) + "*") + "|" + "a".repeat(36), ("*a".repeat(12) + "*a") + "|" + "a".repeat(12)));
    run("GL-PERF 20 stars vs 2000 a", () -> globPairs(("*a".repeat(20) + "*b") + "|" + "a".repeat(2000), ("*a".repeat(20) + "*a") + "|" + "a".repeat(2000), ("*a".repeat(20) + "*a") + "|" + "a".repeat(19)));
    // A pattern `*` or `?` against the same character in the input: Glob.scala:51 compares the literal first.
    run("GL-05 wildcards in the input", () -> globPairs("*|*", "?|?", "*|?", "a*|a*", "a*b|a*b", "a*b|a*", "*a|**a", "?*|**", "*?|?*", "**|*"));

    // ---- Slice P06 (2026-11-18): UTIL-PLATAFORMA (AC-UTIL-020, 023..028)
    PlatformOracle.register();

    // ---- Slice P65 (2026-11-18): UTIL-HASHCODE (AC-UTIL-016.15)
    HashCodeOracle.register();

    // ---- Slice P66 (2026-11-18): UTIL-PEKKOVERSION (AC-UTIL-005.1)
    PekkoVersionOracle.register();

    // ---- Slice P67 (2026-11-18): UTIL-VERSION (AC-UTIL-005.2)
    VersionOracle.register();

    // ---- Slice P68 (2026-11-18): UTIL-PRETTYDURATION (AC-UTIL-016.5)
    PrettyDurationOracle.register();

    // ---- Slice P69 (2026-11-18): UTIL-UUID-ORDEN (AC-UTIL-016.14)
    UuidOrderOracle.register();

    // ---- Slice P70 (2026-11-18): UTIL-RELOJ (AC-UTIL-016.16)
    WallClockOracle.register();

    // ---- Slice P75 (2026-11-18): UTIL-ORIGEN-CLOSURES (AC-UTIL-016.8)
    LineNumbersOracle.register();

    // ---- Slice P03 (2026-11-18): UTIL-ESTRUCTURAS (AC-UTIL-007..014, 015.1, 016.4, 017.1..017.5)
    StructuresOracle.register();
  }
}
