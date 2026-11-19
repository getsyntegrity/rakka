package pilot;

import com.typesafe.config.*;
import java.net.URL;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.concurrent.TimeUnit;

/**
 * Oracle for unit AC-CFG (parser and loading): how Typesafe Config 1.4.9 (the HOCON library of Apache Pekko 2.0.0) behaves for
 * the contracts Pekko relies on, plus one KEY case per leaf key of the reference.conf files of pekko-actor and pekko-actor-typed 2.0.0.
 * Ported from the deprecated jshell script (Q-X-12, Q-CFG-4); registered in order, run by id.
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <fields>} or {@code <CASE-ID> -> ERR <Exception> [<message>]}.
 * Case-id prefixes: FB, SUB, INC, DUR, SIZE, BOOL, TYPE, SPECIAL, LIST, REF (hand-written) and KEY (every leaf key of reference.conf).
 */
final class ConfigHoconOracle {
  private ConfigHoconOracle() {}

  static final Map<String, Callable<String>> CASES = new LinkedHashMap<>();

  static void run(String id, Callable<String> f) {
    if (CASES.put(UtilCoreOracle.esc(id), f) != null) throw new IllegalStateException("duplicate case id " + id);
  }

  static List<String> ids() {
    register();
    return new ArrayList<>(CASES.keySet());
  }

  static String line(String escapedId) {
    register();
    Callable<String> f = CASES.get(escapedId);
    if (f == null) throw new IllegalArgumentException("unknown case id " + escapedId);
    String r;
    try {
      r = "ok " + f.call();
    } catch (Throwable t) {
      r = "ERR " + t.getClass().getSimpleName() + " [" + t.getMessage() + "]";
    }
    return UtilCoreOracle.esc(UtilCoreOracle.unesc(escapedId) + " -> " + r);
  }

  static Config parse(String hocon) { return ConfigFactory.parseString(hocon); }
  static Config resolved(Config c) { return c.resolve(); }
  static String dur(String text, TimeUnit unit) { return parse("x = \"" + text + "\"").getDuration("x", unit) + ""; }
  static String size(String text) { return parse("x = \"" + text + "\"").getBytes("x") + ""; }

  private static boolean registered = false;

  static synchronized void register() {
    if (registered) return;
    registered = true;
    registerHandWritten();
    registerCheckValidAndRender();
    registerReferenceKeys();
    registerNumberRendering();
    registerNumberLexing();
    registerSaturatedDurations();
    registerDecimalSizes();
  }

  static void registerHandWritten() {

    // --- FB: withFallback -------------------------------------------------------------------
    run("FB-01 receiver scalar wins over fallback scalar", () ->
      parse("a = 1").withFallback(parse("a = 2")).getInt("a") + "");
    run("FB-02 objects merge key by key, receiver wins on conflict", () -> {
      Config c = parse("p { a = 1, b = 2 }").withFallback(parse("p { b = 9, c = 3 }"));
      return c.getInt("p.a") + "," + c.getInt("p.b") + "," + c.getInt("p.c");
    });
    run("FB-03 scalar in receiver replaces object in fallback", () ->
      parse("p = 1").withFallback(parse("p { b = 9 }")).root().get("p").valueType() + "");
    run("FB-04 object in receiver over scalar in fallback keeps the receiver object", () ->
      parse("p { b = 9 }").withFallback(parse("p = 1")).root().get("p").valueType() + "");
    run("FB-05 substitution is late-bound: override of referenced key is seen", () -> {
      Config reference = parse("a.base = 1\na.derived = ${a.base}");
      Config app = parse("a.base = 2");
      return app.withFallback(reference).resolve().getInt("a.derived") + "";
    });
    run("FB-06 resolving the fallback first freezes the substitution", () -> {
      Config reference = parse("a.base = 1\na.derived = ${a.base}").resolve();
      Config app = parse("a.base = 2");
      return app.withFallback(reference).resolve().getInt("a.derived") + "";
    });
    run("FB-07 chain of three: first receiver wins", () ->
      parse("x = 1").withFallback(parse("x = 2")).withFallback(parse("x = 3")).getInt("x") + "");
    run("FB-08 explicit config only (no fallback) does not see reference keys", () ->
      parse("pekko.loglevel = DEBUG").hasPath("pekko.actor.provider") + "");
    run("FB-09 hasPath on null value is false", () ->
      parse("a = null").hasPath("a") + "");
    run("FB-10 hasPathOrNull on null value is true", () ->
      parse("a = null").hasPathOrNull("a") + "");

    // --- SUB: substitutions -----------------------------------------------------------------
    run("SUB-01 optional substitution of a missing path leaves the key absent", () ->
      resolved(parse("x = ${?no.such.path}")).hasPath("x") + "");
    run("SUB-02 optional substitution concatenated with an array (reference.conf:78)", () ->
      resolved(parse("e = ${?pekko.library-extensions} [\"d\"]")).getStringList("e") + "");
    run("SUB-03 same, with the optional path defined by the application", () ->
      resolved(parse("pekko.library-extensions = [\"u\"]\ne = ${?pekko.library-extensions} [\"d\"]"))
        .getStringList("e") + "");
    run("SUB-04 required substitution of a missing path fails", () ->
      resolved(parse("x = ${no.such.path}")).hasPath("x") + "");
    run("SUB-05 self-reference concatenates strings", () ->
      resolved(parse("s = foo\ns = ${s}bar")).getString("s"));
    run("SUB-06 += appends to an existing list", () ->
      resolved(parse("a = [1]\na += 2")).getIntList("a") + "");
    run("SUB-07 += on a missing key creates the list", () ->
      resolved(parse("a += 2")).getIntList("a") + "");
    run("SUB-08 substitution to an unset environment variable, optional", () ->
      resolved(parse("x = ${?RAKKA_ORACLE_UNSET_VARIABLE_42}")).hasPath("x") + "");
    run("SUB-09 substitution to an unset environment variable, required", () ->
      resolved(parse("x = ${RAKKA_ORACLE_UNSET_VARIABLE_42}")).hasPath("x") + "");
    run("SUB-10 cyclic substitution fails", () ->
      resolved(parse("a = ${b}\nb = ${a}")).hasPath("a") + "");
    run("SUB-11 substitution inside a quoted string is not substituted", () ->
      resolved(parse("s = \"${a}\"\na = 1")).getString("s"));
    run("SUB-12 a getter on an unresolved config fails", () ->
      parse("a = 1\nb = ${a}").getInt("b") + "");
    run("SUB-13 substitution of an object merges with a later object", () ->
      resolved(parse("base { x = 1 }\nd = ${base} { y = 2 }")).getConfig("d").root().keySet().stream().sorted().collect(java.util.stream.Collectors.toList()) + "");
    run("SUB-14 reference.conf:878 shape: key set to another key's list", () ->
      resolved(parse("w = [\"a\", \"b\"]\nallowed = ${w}")).getStringList("allowed") + "");

    // --- INC: includes ----------------------------------------------------------------------
    run("INC-01 include of a missing file is ignored", () ->
      parse("include \"no-such-file-oracle\"\nx = 1").getInt("x") + "");
    run("INC-02 required include of a missing file fails", () ->
      parse("include required(\"no-such-file-oracle\")\nx = 1").getInt("x") + "");
    run("INC-03 file() include of a missing file is ignored", () ->
      parse("include file(\"/no/such/file/oracle.conf\")\nx = 1").getInt("x") + "");
    run("INC-04 classpath() include of a missing resource is ignored", () ->
      parse("include classpath(\"no-such-resource-oracle\")\nx = 1").getInt("x") + "");

    // --- DUR: durations ---------------------------------------------------------------------
    for (String t : new String[] {"10 ms", "10ms", "10 milliseconds", "1 us", "1us", "5 ns", "1 d", "1d",
        "2 minutes", "5 minutes", "5s", "5 s", "1 h", "100", "1.5 s", "0.5 ms", "-1s", "0", "1 hour", "1 day"}) {
      final String text = t;
      run("DUR-MS " + text, () -> dur(text, TimeUnit.MILLISECONDS));
      run("DUR-NS " + text, () -> dur(text, TimeUnit.NANOSECONDS));
    }
    for (String t : new String[] {"5 foo", "", "abc", "1 s s", "infinite", "off"}) {
      final String text = t;
      run("DUR-ERR " + text, () -> dur(text, TimeUnit.MILLISECONDS));
    }
    run("DUR-SCI scientific notation is accepted", () -> dur("1e3 ms", TimeUnit.MILLISECONDS));
    run("DUR-UNQUOTED 5 minutes", () -> parse("x = 5 minutes").getDuration("x", TimeUnit.MILLISECONDS) + "");
    run("DUR-NUMBER bare number is milliseconds", () -> parse("x = 100").getDuration("x", TimeUnit.MILLISECONDS) + "");
    run("DUR-ROUND 1 ns in millis truncates toward zero", () -> dur("1 ns", TimeUnit.MILLISECONDS));
    run("DUR-ROUND 999999 ns in millis truncates toward zero", () -> dur("999999 ns", TimeUnit.MILLISECONDS));
    run("DUR-ROUND -1500 us in millis truncates toward zero", () -> dur("-1500 us", TimeUnit.MILLISECONDS));
    run("DUR-LARGE overflow", () -> dur("106752 d", TimeUnit.NANOSECONDS));
    run("DUR-LARGE 292 years in ns still fits", () -> dur("106751 d", TimeUnit.NANOSECONDS));

    // --- SIZE: memory sizes -----------------------------------------------------------------
    for (String t : new String[] {"128 KiB", "128KiB", "128 K", "128 k", "1 MB", "1MB", "1 kB", "1 KB", "512", "512 B",
        "512 bytes", "1.5 KiB", "1 GiB", "1 MiB", "1 g", "0", "-1 KiB", "1 TiB", "256 MiB"}) {
      final String text = t;
      run("SIZE " + text, () -> size(text));
    }
    for (String t : new String[] {"unlimited", "128 XiB", "", "KiB", "1 KiB KiB", "8 EiB", "8388608 TiB"}) {
      final String text = t;
      run("SIZE-ERR " + text, () -> size(text));
    }
    run("SIZE-NUMBER bare number is bytes", () -> parse("x = 4096").getBytes("x") + "");
    run("SIZE-MEMORY getMemorySize", () -> parse("x = 4 MiB").getMemorySize("x").toBytes() + "");

    // --- BOOL: booleans ---------------------------------------------------------------------
    for (String v : new String[] {"on", "off", "yes", "no", "true", "false", "\"on\"", "\"off\"", "\"true\"", "\"yes\"",
        "\"maybe\"", "1", "\"ON\""}) {
      final String text = v;
      run("BOOL " + text, () -> parse("x = " + text).getBoolean("x") + "");
    }

    // --- TYPE: getters ----------------------------------------------------------------------
    run("TYPE-01 missing key", () -> parse("a = 1").getInt("b") + "");
    run("TYPE-02 string where int expected", () -> parse("a = abc").getInt("a") + "");
    run("TYPE-03 quoted number as int", () -> parse("a = \"7\"").getInt("a") + "");
    run("TYPE-04 int where string expected", () -> parse("a = 7").getString("a"));
    run("TYPE-05 object where string expected", () -> parse("a { b = 1 }").getString("a"));
    run("TYPE-06 list where string expected", () -> parse("a = [1]").getString("a"));
    run("TYPE-07 null where string expected", () -> parse("a = null").getString("a"));
    run("TYPE-08 long beyond int", () -> parse("a = 3000000000").getInt("a") + "");
    run("TYPE-09 double as int", () -> parse("a = 1.5").getInt("a") + "");
    run("TYPE-10 whole double as int", () -> parse("a = 2.0").getInt("a") + "");
    run("TYPE-11 getConfig on scalar", () -> parse("a = 1").getConfig("a") + "");
    run("TYPE-12 getString of boolean", () -> parse("a = on").getString("a"));
    run("TYPE-13 getDouble of int", () -> parse("a = 3").getDouble("a") + "");

    // --- SPECIAL: literals Pekko interprets itself -------------------------------------------
    run("SPECIAL-01 log-dead-letters = on is a boolean", () -> parse("x = on").getValue("x").valueType() + "");
    run("SPECIAL-02 log-dead-letters = 10 is a number", () -> parse("x = 10").getValue("x").valueType() + "");
    run("SPECIAL-03 infinite is a string", () -> parse("x = infinite").getValue("x").valueType() + "");
    run("SPECIAL-04 unlimited is a string", () -> parse("x = unlimited").getString("x"));
    run("SPECIAL-05 off is a string by getString", () -> parse("x = off").getString("x"));
    run("SPECIAL-06 on is a string by getString", () -> parse("x = on").getString("x"));
    run("SPECIAL-07 5 minutes unquoted is a string", () -> parse("x = 5 minutes").getString("x"));
    run("SPECIAL-08 duration getter on infinite fails", () -> parse("x = infinite").getDuration("x", TimeUnit.MILLISECONDS) + "");

    // --- LIST: lists ------------------------------------------------------------------------
    run("LIST-01 string list", () -> parse("l = [\"a\", \"b\"]").getStringList("l") + "");
    run("LIST-02 empty list", () -> parse("l = []").getStringList("l") + "");
    run("LIST-03 mixed list as string list coerces", () -> parse("l = [1, true, x]").getStringList("l") + "");
    run("LIST-04 list of objects", () -> parse("l = [{a=1}, {a=2}]").getConfigList("l").size() + "");
    run("LIST-05 numeric-keyed object as list", () -> parse("l { \"0\" = a, \"1\" = b }").getStringList("l") + "");
    run("LIST-06 trailing comma allowed", () -> parse("l = [1, 2,]").getIntList("l") + "");
    run("LIST-07 missing comma between elements on one line fails", () -> parse("l = [1 2]").getIntList("l") + "");

    // --- REF: reference.conf excerpts (upstream/pekko/actor/src/main/resources/reference.conf) -
    run("REF-01 library-extensions default (line 78)", () ->
      resolved(parse("pekko.library-extensions = ${?pekko.library-extensions} [\"org.apache.pekko.serialization.SerializationExtension$\"]"))
        .getStringList("pekko.library-extensions") + "");
    run("REF-02 library-extensions with application value first", () -> {
      Config app = parse("pekko.library-extensions += \"my.Ext\"");
      Config ref = parse("pekko.library-extensions = ${?pekko.library-extensions} [\"org.apache.pekko.serialization.SerializationExtension$\"]");
      return app.withFallback(ref).resolve().getStringList("pekko.library-extensions") + "";
    });
    run("REF-03 max-decompressed-size = unlimited (line 863)", () ->
      parse("pekko.serialization.max-decompressed-size = unlimited").getString("pekko.serialization.max-decompressed-size"));
    run("REF-04 direct-buffer-size = 128 KiB (line 966)", () ->
      parse("pekko.io.tcp.direct-buffer-size = 128 KiB").getBytes("pekko.io.tcp.direct-buffer-size") + "");
    run("REF-05 log-dead-letters-suspend-duration = 5 minutes (line 68)", () ->
      parse("pekko.log-dead-letters-suspend-duration = 5 minutes").getDuration("pekko.log-dead-letters-suspend-duration", TimeUnit.MILLISECONDS) + "");
    run("REF-06 key with dots in quotes is one key", () ->
      parse("a.\"b.c\" = 1").hasPath("a.\"b.c\"") + "");
    run("REF-07 unquoted key with hyphens", () -> parse("fork-join-executor { parallelism-min = 8 }").getInt("fork-join-executor.parallelism-min") + "");
    run("REF-08 comment styles # and //", () -> parse("# c\n// c\na = 1 # trailing\nb = 2 // trailing").getInt("a") + parse("b = 2").getInt("b") + "");
    run("REF-09 triple-quoted string keeps newlines", () -> parse("s = \"\"\"a\nb\"\"\"").getString("s"));
    run("REF-10 duplicate keys: later object merges, later scalar wins", () -> {
      Config c = parse("a { x = 1 }\na { y = 2 }\nb = 1\nb = 2");
      return c.getInt("a.x") + "," + c.getInt("a.y") + "," + c.getInt("b");
    });
  }

  // ---- CV and RW: checkValid (AC-CFG-019) and renderWithRedactions (AC-UTIL-002.5), added with slice P07 -----------------------
  static String cv(String reference, String config, String... restrict) {
    parse(config).resolve().checkValid(parse(reference).resolve(), restrict);
    return "valid";
  }

  /** Pekko's ConfigOps.renderWithRedactions (Helpers.scala), with the user name that the case itself supplies. */
  static String redacted(Config c) {
    String username = System.getProperty("user.name");
    return c.root().render(ConfigRenderOptions.defaults().setShowEnvVariableValues(false)).replace(username, "<username>");
  }

  static void registerCheckValidAndRender() {
    run("CV-01 a config equal to the reference is valid", () -> cv("pekko { a = 1, b = abc }", "pekko { a = 1, b = abc }", "pekko"));
    run("CV-02 a key missing from the config", () -> cv("pekko { a = 1, b = abc }", "pekko { a = 1 }", "pekko"));
    run("CV-03 string where a number is expected is accepted (a string can be converted)", () -> cv("pekko { a = 1 }", "pekko { a = abc }", "pekko"));
    run("CV-04 object where a number is expected", () -> cv("pekko { a = 1 }", "pekko { a { x = 1 } }", "pekko"));
    run("CV-05 number where an object is expected", () -> cv("pekko { a { x = 1 } }", "pekko { a = 2 }", "pekko"));
    run("CV-06 number where a list is expected", () -> cv("pekko { a = [1] }", "pekko { a = 2 }", "pekko"));
    run("CV-07 numeric-keyed object where a list is expected", () -> cv("pekko { a = [1] }", "pekko { a { \"0\" = 1 } }", "pekko"));
    run("CV-08 number where a string is expected", () -> cv("pekko { a = abc }", "pekko { a = 2 }", "pekko"));
    run("CV-09 boolean where a number is expected", () -> cv("pekko { a = 1 }", "pekko { a = true }", "pekko"));
    run("CV-10 list where a number is expected", () -> cv("pekko { a = 1 }", "pekko { a = [1] }", "pekko"));
    run("CV-11 several problems are reported together", () -> cv("pekko { a = 1, b = 2, c { d = 1 } }", "pekko { a { x = 1 }, c = 3 }", "pekko"));
    run("CV-12 problems outside the restricted path are not checked", () -> cv("pekko { a = 1 }\nother { b = 1 }", "pekko { a = 1 }", "pekko"));
    run("CV-13 no restriction checks the whole reference", () -> cv("pekko { a = 1 }\nother { b = 1 }", "pekko { a = 1 }"));
    run("CV-14 null where a number is expected is accepted", () -> cv("pekko { a = 1 }", "pekko { a = null }", "pekko"));
    run("CV-15 extra keys in the config are accepted", () -> cv("pekko { a = 1 }", "pekko { a = 1, z = 2 }", "pekko"));
    run("CV-16 withoutPath drops a key of the reference", () -> {
      Config reference = parse("pekko { a = 1, internal-dispatcher = abc }").resolve().withoutPath("pekko.internal-dispatcher");
      parse("pekko { a = 1, internal-dispatcher { x = 1 } }").resolve().checkValid(reference, "pekko");
      return "valid";
    });
    run("CV-17 the same without withoutPath fails", () -> {
      Config reference = parse("pekko { a = 1, internal-dispatcher { y = 1 } }").resolve();
      parse("pekko { a = 1, internal-dispatcher = abc }").resolve().checkValid(reference, "pekko");
      return "valid";
    });
    run("CV-18 an unresolved config fails", () -> {
      parse("pekko { a = ${b} }").checkValid(parse("pekko { a = 1 }").resolve(), "pekko");
      return "valid";
    });
    run("CV-19 the real reference.conf of pekko-actor is valid against itself", () -> {
      Config reference = ConfigFactory.parseURL(referenceOf("pekko-actor")).resolve();
      reference.checkValid(reference, "pekko");
      return "valid";
    });
    run("CV-20 the real reference.conf is valid for an application that overrides some keys", () -> {
      Config reference = ConfigFactory.parseURL(referenceOf("pekko-actor")).resolve();
      Config app = parse("pekko.loglevel = DEBUG\npekko.log-dead-letters = off\npekko.actor.provider = local");
      app.withFallback(reference).resolve().checkValid(reference, "pekko");
      return "valid";
    });
    run("CV-21 the real reference.conf fails when a key it defines is an object in the application", () -> {
      Config reference = ConfigFactory.parseURL(referenceOf("pekko-actor")).resolve();
      Config app = parse("pekko.loglevel { x = 1 }");
      app.withFallback(reference).resolve().checkValid(reference, "pekko");
      return "valid";
    });
    run("RW-01 render of scalars, nested objects, lists, null and empty values", () ->
      redacted(parse("a = 1\nb = hello\nc { d = [1, 2], e = \"x y\" }\nl = []\no {}\nn = null\nf = 1.5\nbool = true\nw = 1.0").resolve()));
    run("RW-02 an environment variable value is not shown", () -> redacted(parse("x = ${PATH}").resolve()));
    run("RW-03 the user name is replaced", () ->
      redacted(parse("x = \"home of " + System.getProperty("user.name") + "\"").resolve()));
    run("RW-04 render of an unresolved config", () -> redacted(parse("a = 1\nb = ${a}")));
    run("RW-05 render of a merge of two configs", () -> redacted(parse("a.x = 1").withFallback(parse("a.y = 2\nb = 3")).resolve()));
    run("RW-06 keys that need quotes and escapes in strings", () ->
      redacted(parse("\"a.b\" = 1\n\"c d\" = \"q\\\"uote\\nline\"").resolve()));
  }

  // ---- KEY: every leaf key of the reference.conf files of pekko-actor and pekko-actor-typed 2.0.0 (Q-X-1 prerequisite) ------
  /** Leaves of the resolved value at {@code path}: an object is expanded, anything else (list, scalar, null) is a leaf. */
  private static void leaves(ConfigValue v, List<String> path, TreeMap<String, ConfigValue> out) {
    if (v.valueType() == ConfigValueType.OBJECT) {
      ConfigObject o = (ConfigObject) v;
      for (String k : new TreeSet<>(o.keySet())) {
        List<String> p = new ArrayList<>(path);
        p.add(k);
        leaves(o.get(k), p, out);
      }
    } else {
      out.put(ConfigUtil.joinPath(path), v);
    }
  }

  static URL referenceOf(String artifactPrefix) {
    try {
      for (URL u : Collections.list(ConfigHoconOracle.class.getClassLoader().getResources("reference.conf"))) {
        if (u.toString().contains("/" + artifactPrefix + "_2.13-2.0.0.jar!")) return u;
      }
    } catch (java.io.IOException e) {
      throw new IllegalStateException(e);
    }
    throw new IllegalStateException("no reference.conf of " + artifactPrefix + " on the classpath");
  }

  static void registerReferenceKeys() {
    URL actor = referenceOf("pekko-actor");
    URL typed = referenceOf("pekko-actor-typed");
    // Not ConfigFactory.defaultReference(): it overlays system properties and would make the cases environment dependent.
    Config actorUnresolved = ConfigFactory.parseURL(actor);
    Config typedUnresolved = ConfigFactory.parseURL(typed);
    Config actorOwn = actorUnresolved.resolve();
    // typed may refer to actor keys: resolve it against both files, then keep what the typed file adds or overrides
    // (Config#resolveWith cannot be used: it fails on the self-referencing library-extensions of the typed file).
    Config typedMerged = typedUnresolved.withFallback(actorUnresolved).resolve();
    registerFileKeys("actor", actorOwn, null);
    registerFileKeys("actor-typed", typedMerged, actorOwn);
  }

  private static void registerFileKeys(String label, Config own, Config exclude) {
    TreeMap<String, ConfigValue> keys = new TreeMap<>();
    leaves(own.root(), new ArrayList<>(), keys);
    if (exclude != null) {
      TreeMap<String, ConfigValue> base = new TreeMap<>();
      leaves(exclude.root(), new ArrayList<>(), base);
      keys.entrySet().removeIf(e -> base.containsKey(e.getKey())
          && base.get(e.getKey()).render(ConfigRenderOptions.concise()).equals(e.getValue().render(ConfigRenderOptions.concise())));
    }
    for (Map.Entry<String, ConfigValue> e : keys.entrySet()) {
      final ConfigValue v = e.getValue();
      run("KEY " + label + " " + e.getKey(), () -> v.valueType() + " " + v.render(ConfigRenderOptions.concise()));
    }
  }

  // ---- NUM-RENDER: how a parsed number is rendered (D-BUG-01). Registered last so the numbering of the older ids is stable. ----
  static final String[] NUM_RENDER_TEXTS = {"1.50", "1e-7", "12345678.9", "-0", "010", "1e400", "1.5e-3", "5e18", "1e18",
      "1.0", "100.0", "0.1", "1E5", "1e2", "123456789012345678", "-1.50", "0.0", "1e+2", "-0.0", "9223372036854775807", "1e19", "-1e19", "9223372036854775808.0", "1.7976931348623157e308", "4.9e-324", "123456.789", "0.001", "0.0001", "1234567.0", "12345678.5", "-2.5e-5", "0.30000000000000004"};

  static void registerNumberRendering() {
    for (String t : NUM_RENDER_TEXTS) {
      final String text = t;
      run("NUM-RENDER " + text, () -> parse("x = " + text).getValue("x").render(ConfigRenderOptions.concise()));
    }
  }

  // ---- NUMLEX: which literals are numbers (D-BUG-02). Result: value type, concise render, getString; errors only by class (the text is D-BUG-13). ----
  static final String[] NUM_LEX_TEXTS = {"1.", ".5", "-.5", "1.e5", "1e5", "1E5", "1e", "1e+", "1e+5", "-1", "--1", "+1", "00", "007", "1.5.2", "1.2.3", "0x10", "1_000", "99999999999999999999", "-99999999999999999999", "9223372036854775807", "9223372036854775808", "-9223372036854775808", "-9223372036854775809", "1+2", "1 2", "1-2", "1.5abc", "123abc", "abc123", "-", "-a", "1e400", "1e-400", "0.0e0", "1..2", "1.e", "3.14", "1E+5", "-0", "- 1", "1 .5", "-.", "-e5", "1.5e", "-1.5E-3"};

  static void registerNumberLexing() {
    for (String t : NUM_LEX_TEXTS) {
      final String text = t;
      run("NUMLEX " + text, () -> {
        try {
          Config c = parse("x = " + text);
          ConfigValue v = c.getValue("x");
          return v.valueType() + " " + v.render(ConfigRenderOptions.concise()) + " " + c.getString("x");
        } catch (ConfigException e) {
          return "error " + e.getClass().getSimpleName();
        }
      });
    }
  }

  // ---- DUR-SAT: durations saturate at Long.MAX_VALUE / MIN_VALUE nanoseconds before the target unit is applied (D-BUG-09) ----
  static final String[] DUR_SAT_TEXTS = {"106752 d", "106751 d", "-106752 d", "-106753 d", "9223372036854775807 ms", "9223372036854775807 s", "1e30 s", "-1e30 s", "1e18 ns", "9.3e18 ns", "9.3e18 us", ".5 s", "+5 s", "+5.5 s", "-.5 s", "5. s", "NaN ms", "Infinity ms", "-Infinity ms", "+Infinity ms", "0x10 ms", "1E3 ms", "5e-1 s", "0.0000005 s", "1.5 ns", "-1.5 ns", "2.9 ns", "106751.99 d", "92233720368 s", "92233720369 s", "9223372036854775807", "-9223372036854775808 ns", "-9223372036854775809 ns", "3e-10 d", "+ ms", "- s", "1.2.3 s", "1e s", "5 +s"};

  static void registerSaturatedDurations() {
    for (String t : DUR_SAT_TEXTS) {
      final String text = t;
      run("DUR-SAT NS " + text, () -> dur(text, TimeUnit.NANOSECONDS));
      run("DUR-SAT MS " + text, () -> dur(text, TimeUnit.MILLISECONDS));
      run("DUR-SAT SEC " + text, () -> dur(text, TimeUnit.SECONDS));
      run("DUR-SAT DAY " + text, () -> dur(text, TimeUnit.DAYS));
    }
  }

  // ---- SIZE-DEC: sizes are exact decimal arithmetic (BigDecimal), not doubles, and out-of-range messages show the exact product (D-BUG-10) ----
  static final String[] SIZE_DEC_TEXTS = {"8.2 MB", "8.2 MiB", "0.1 GB", "1.5 KiB", "-1.5 KiB", "-0.5 B", "0.9 B", "1.1 kB", "8388607.9 TiB", "8388608 TiB", "8 EiB", "7.99 EiB", "9223372036854775807 YiB", "9223372036854775807 B", "9223372036854775808 B", "-9223372036854775808 B", "-9223372036854775809 B", "+5 B", "+5.5 kB", ".5 kB", "5. kB", "1e3 B", "1E3 kB", "1e-3 kB", "2.5e2 B", "1e30 B", "1e-30 GB", "NaN B", "Infinity B", "0x10 B", "1.2.3 B", "1e B", "00012 B", "0.000000000000000000000000001 YiB", "123456789012345678901234567890123456789012345678901e-30 B", "1 YiB", "0 YiB", "8.0 EiB", "9.2233720368547758 EB", "9.223372036854775807 EB", "9.223372036854775808 EB", "-9.223372036854775808 EB", "-9.223372036854775809 EB", "0.5 kB", "2.5 MB", "1023.9 B", "12345678.9 kB"};

  static void registerDecimalSizes() {
    for (String t : SIZE_DEC_TEXTS) {
      final String text = t;
      run("SIZE-DEC " + text, () -> size(text));
    }
  }
}
