package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import com.typesafe.config.ConfigRenderOptions;
import java.util.*;
import java.util.concurrent.Callable;
import org.apache.pekko.actor.Deploy;
import org.apache.pekko.actor.LocalScope$;
import org.apache.pekko.actor.NoScopeGiven$;
import org.apache.pekko.actor.Scope;
import org.apache.pekko.routing.NoRouter$;
import org.apache.pekko.routing.RandomPool;
import org.apache.pekko.routing.RoundRobinPool;
import org.apache.pekko.routing.RouterConfig;

/**
 * Oracle for unit P10, row AC-CFG-013 (Deploy, Scope, LocalScope, NoScopeGiven and withFallback): what the real classes of
 * Apache Pekko 2.0.0 do (Deployer.scala:28-164 and 173-220, RouterConfig.scala:94,373). A router is printed by its class only
 * (the merge of two real pools belongs to the routing module); {@code toString} of a Deploy is not recorded.
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <result>} or {@code <CASE-ID> -> ERR <Exception> [<message>]}.
 * Case-id prefixes: DP (Deploy values and equality), FB (withFallback), SC (Scope.withFallback), RT (RouterConfig.withFallback).
 */
final class DeployOracle {
  private DeployOracle() {}

  static final Map<String, Callable<String>> CASES = new LinkedHashMap<>();

  private static void run(String id, Callable<String> f) {
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

  private static final Scope LOCAL = LocalScope$.MODULE$;
  private static final Scope NO_SCOPE = NoScopeGiven$.MODULE$;
  private static final RouterConfig NO_ROUTER = NoRouter$.MODULE$;

  private static Config conf(String text) { return ConfigFactory.parseString(text); }

  private static scala.collection.immutable.Set<String> tags(String... names) {
    return scala.jdk.javaapi.CollectionConverters.asScala(new HashSet<>(Arrays.asList(names))).toSet();
  }

  /** A Scala immutable Set built element by element, as {@code Set(a, b, ...)} is: up to four elements keep their insertion order. */
  private static scala.collection.immutable.Set<String> orderedTags(String... names) {
    scala.collection.immutable.Set<String> set = scala.collection.immutable.Set$.MODULE$.<String>empty();
    for (String name : names) set = set.$plus(name);
    return set;
  }

  private static Deploy deploy(String path, String config, RouterConfig router, Scope scope, String dispatcher, String mailbox) {
    return new Deploy(path, conf(config), router, scope, dispatcher, mailbox, tags());
  }

  private static Deploy plain(String path, String dispatcher, String mailbox) {
    return deploy(path, "", NO_ROUTER, NO_SCOPE, dispatcher, mailbox);
  }

  private static String scope(Scope s) { return s == LOCAL ? "Local" : s == NO_SCOPE ? "NoScopeGiven" : "other"; }

  private static String router(RouterConfig r) { return r == NO_ROUTER ? "NoRouter" : r.getClass().getSimpleName(); }

  private static String show(Deploy d) {
    List<String> names = new ArrayList<>(scala.jdk.javaapi.CollectionConverters.asJava(d.tags()));
    Collections.sort(names);
    return "path=\"" + d.path() + "\" config=" + d.config().root().render(ConfigRenderOptions.concise())
        + " router=" + router(d.routerConfig()) + " scope=" + scope(d.scope())
        + " dispatcher=\"" + d.dispatcher() + "\" mailbox=\"" + d.mailbox() + "\" tags=" + names;
  }

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    // Deployer.scala:28-50, 68-75
    run("DP-01 Deploy() has the defaults", () -> show(Deploy.apply("", ConfigFactory.empty(), NO_ROUTER, NO_SCOPE, "", "")));
    run("DP-02 Deploy.local has the local scope", () -> show(Deploy.local()));
    run("DP-03 the fields are kept as given", () ->
        show(deploy("/user/a", "x = 1, y = \"s\"", new RoundRobinPool(1), LOCAL, "my-dispatcher", "my-mailbox")));
    run("DP-04 withTags keeps the other fields", () -> show(plain("/a", "d", "m").withTags(tags("t2", "t1"))));
    // Deployer.scala:146-156 (equality over the seven fields)
    run("DP-05 equal when the seven fields are equal", () ->
        String.valueOf(deploy("/a", "x = 1", new RandomPool(1), LOCAL, "d", "m").equals(deploy("/a", "x = 1", new RandomPool(1), LOCAL, "d", "m"))));
    run("DP-06 different path", () -> String.valueOf(plain("/a", "", "").equals(plain("/b", "", ""))));
    run("DP-07 different dispatcher", () -> String.valueOf(plain("/a", "d1", "").equals(plain("/a", "d2", ""))));
    run("DP-08 different mailbox", () -> String.valueOf(plain("/a", "", "m1").equals(plain("/a", "", "m2"))));
    run("DP-09 different scope", () ->
        String.valueOf(deploy("/a", "", NO_ROUTER, LOCAL, "", "").equals(deploy("/a", "", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-10 different router", () ->
        String.valueOf(deploy("/a", "", new RoundRobinPool(1), NO_SCOPE, "", "").equals(deploy("/a", "", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-11 different tags", () -> String.valueOf(plain("/a", "", "").withTags(tags("t")).equals(plain("/a", "", ""))));
    run("DP-12 same tags", () -> String.valueOf(plain("/a", "", "").withTags(tags("t", "u")).equals(plain("/a", "", "").withTags(tags("u", "t")))));
    run("DP-13 different configuration", () ->
        String.valueOf(deploy("/a", "x = 1", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = 2", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-14 same configuration from other text", () ->
        String.valueOf(deploy("/a", "x = 1", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "\n\n  x = 1  ", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-15 same configuration written as a path or as an object", () ->
        String.valueOf(deploy("/a", "a.b = 1", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "a { b = 1 }", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-16 the whole number 1 equals 1.0", () ->
        String.valueOf(deploy("/a", "x = 1", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = 1.0", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-17 the number 1.5 does not equal the string 1.5", () ->
        String.valueOf(deploy("/a", "x = 1.5", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = \"1.5\"", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-18 lists of the same elements", () ->
        String.valueOf(deploy("/a", "x = [1, 2]", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = [1,2]", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-19 lists in another order", () ->
        String.valueOf(deploy("/a", "x = [1, 2]", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = [2, 1]", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-20 an empty object is not a missing key", () ->
        String.valueOf(deploy("/a", "x = {}", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "", NO_ROUTER, NO_SCOPE, "", ""))));
    run("DP-21 one more key in the other configuration", () ->
        String.valueOf(deploy("/a", "x = 1", NO_ROUTER, NO_SCOPE, "", "").equals(deploy("/a", "x = 1, y = 2", NO_ROUTER, NO_SCOPE, "", ""))));
    // Deployer.scala:109-117
    run("FB-01 the path of the receiver is kept", () -> show(plain("/a", "", "").withFallback(plain("/b", "", ""))));
    run("FB-02 an empty path of the receiver is kept", () -> show(plain("", "", "").withFallback(plain("/b", "", ""))));
    run("FB-03 the configuration of the receiver wins and the rest is merged", () ->
        show(deploy("/a", "a = 1, n { x = 1 }", NO_ROUTER, NO_SCOPE, "", "")
            .withFallback(deploy("/b", "a = 9, b = 2, n { x = 9, y = 2 }", NO_ROUTER, NO_SCOPE, "", ""))));
    run("FB-04 a given dispatcher wins", () -> show(plain("/a", "mine", "").withFallback(plain("/b", "theirs", ""))));
    run("FB-05 an empty dispatcher takes the other one", () -> show(plain("/a", "", "").withFallback(plain("/b", "theirs", ""))));
    run("FB-06 two empty dispatchers stay empty", () -> show(plain("/a", "", "").withFallback(plain("/b", "", ""))));
    run("FB-07 a given mailbox wins", () -> show(plain("/a", "", "mine").withFallback(plain("/b", "", "theirs"))));
    run("FB-08 an empty mailbox takes the other one", () -> show(plain("/a", "", "").withFallback(plain("/b", "", "theirs"))));
    run("FB-09 dispatcher and mailbox are decided one by one", () -> show(plain("/a", "mine", "").withFallback(plain("/b", "theirs", "their-mailbox"))));
    run("FB-10 the tags are not carried", () -> show(plain("/a", "", "").withTags(tags("t1")).withFallback(plain("/b", "", "").withTags(tags("t2")))));
    run("FB-11 the scope of the receiver wins when it is local", () ->
        show(deploy("/a", "", NO_ROUTER, LOCAL, "", "").withFallback(plain("/b", "", ""))));
    run("FB-12 no scope given takes the scope of the other", () ->
        show(plain("/a", "", "").withFallback(deploy("/b", "", NO_ROUTER, LOCAL, "", ""))));
    run("FB-13 the router of the receiver wins", () ->
        show(deploy("/a", "", new RoundRobinPool(1), NO_SCOPE, "", "").withFallback(plain("/b", "", ""))));
    run("FB-14 no router takes the router of the other", () ->
        show(plain("/a", "", "").withFallback(deploy("/b", "", new RandomPool(1), NO_SCOPE, "", ""))));
    run("FB-15 chained fallbacks", () ->
        show(plain("/a", "", "").withFallback(plain("/b", "d", "")).withFallback(plain("/c", "e", "m"))));
    // Deployer.scala:173-220
    run("SC-01 local keeps itself over no scope given", () -> scope(LOCAL.withFallback(NO_SCOPE)));
    run("SC-02 local keeps itself over local", () -> scope(LOCAL.withFallback(LOCAL)));
    run("SC-03 no scope given takes local", () -> scope(NO_SCOPE.withFallback(LOCAL)));
    run("SC-04 no scope given over no scope given", () -> scope(NO_SCOPE.withFallback(NO_SCOPE)));
    // RouterConfig.scala:94,373
    run("RT-01 no router takes a router", () -> router(NO_ROUTER.withFallback(new RoundRobinPool(1))));
    run("RT-02 a router over no router", () -> router(new RoundRobinPool(1).withFallback(NO_ROUTER)));
    run("RT-03 no router over no router", () -> router(NO_ROUTER.withFallback(NO_ROUTER)));
    run("RT-04 a router over another router keeps its class", () -> router(new RoundRobinPool(1).withFallback(new RandomPool(1))));
    // Deployer.scala:163 (toString): Deploy(path, config, routerConfig, scope, dispatcher, mailbox, tags)
    run("TS-01 toString of Deploy()", () -> Deploy.apply("", ConfigFactory.empty(), NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-02 toString of Deploy.local", () -> Deploy.local().toString());
    run("TS-03 toString with path, dispatcher and mailbox", () -> plain("/user/a", "my-dispatcher", "my-mailbox").toString());
    run("TS-04 toString with a nested configuration", () -> deploy("/a", "x = 1, n { b = [1, 2, {c = true}], a = \"s\" }", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-05 toString with numbers of every shape", () -> deploy("/a", "a = 1, b = 1.0, c = 1e5, d = 1.5e-7, e = 12345678901234567890, f = -0, g = 0.1", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-06 toString with null and booleans", () -> deploy("/a", "a = null, b = true, c = false", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-07 toString with keys that need quotes", () -> deploy("/a", "\"a b\" = 1, \"\" = 2, \"x.y\" = 3, \"1\" = 4", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-08 toString with escapes in strings", () -> deploy("/a", "a = \"q\\\"uote\\n\\t\\\\ \\u00e9 \\u0001 \\u001f\"", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-09 toString with Unicode and astral text", () -> deploy("/\u00e9\uD83D\uDE00", "a = \"\u00e9\uD83D\uDE00\", \"\u00e9\" = 1", NO_ROUTER, NO_SCOPE, "d\u00e9", "m\uD83D\uDE00").toString());
    run("TS-10 toString with a path of the form a.b.c", () -> deploy("/a", "a.b.c = 1, a.b.d = 2", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-11 toString with an unresolved substitution", () -> deploy("/a", "a = ${b}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-12 toString with an unresolved optional substitution and a concatenation", () -> deploy("/a", "a = ${?b}, c = ${d} suffix, e = \"x\" ${f}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-13 toString with one tag", () -> plain("/a", "", "").withTags(orderedTags("t")).toString());
    run("TS-14 toString with four tags in insertion order", () -> plain("/a", "", "").withTags(orderedTags("t3", "t1", "t4", "t2")).toString());
    run("TS-15 toString with four tags in the opposite insertion order", () -> plain("/a", "", "").withTags(orderedTags("t2", "t4", "t1", "t3")).toString());
    run("TS-16 toString with five tags", () -> plain("/a", "", "").withTags(orderedTags("t3", "t1", "t4", "t2", "t5")).toString());
    run("TS-17 toString with five tags in another insertion order", () -> plain("/a", "", "").withTags(orderedTags("t5", "t2", "t4", "t1", "t3")).toString());
    run("TS-18 toString with twelve tags", () -> plain("/a", "", "").withTags(orderedTags("alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa", "lambda", "mu")).toString());
    run("TS-19 toString with repeated tags", () -> plain("/a", "", "").withTags(orderedTags("b", "a", "b", "c", "a")).toString());
    run("TS-20 toString after withFallback", () -> deploy("/a", "a = 1", NO_ROUTER, NO_SCOPE, "", "").withFallback(deploy("/b", "b = 2", NO_ROUTER, LOCAL, "d", "m")).toString());
    run("TS-21 toString with the empty object and the empty list", () -> deploy("/a", "a = {}, b = []", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-22 toString with durations and sizes kept as strings", () -> deploy("/a", "t = 10s, m = 512K", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-23 toString with numeric and mixed keys", () -> deploy("/a", "\"10\" = 1, \"9\" = 2, \"2\" = 3, \"007\" = 4, a = 5, B = 6, _ = 7, \"123456789012345678901234567890\" = 8, \"-1\" = 9", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-24 toString with non-ASCII digit keys", () -> deploy("/a", "\"\\u0661\\u0662\" = 1, \"3\" = 2, \"\\uFF11\" = 3", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-25 toString with keys that differ in UTF-16 order and code point order", () -> deploy("/a", "\"\\uFFFF\" = 1, \"\\uD83D\\uDE00\" = 2, \"\\uE000\" = 3", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-26 toString with concatenations of substitutions", () -> deploy("/a", "a = ${x} ${y}, b = ${x}${y}, c = \"a\" ${x} \"b\", d = foo ${x}bar, e = [${x}, 1], f = {g = ${x}}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-27 toString with a substitution after and before an object", () -> deploy("/a", "a = {x = 1}\na = ${b}\nc = ${d}\nc = {y = 2}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-28 toString with substitution paths that need quotes", () -> deploy("/a", "a = ${b.c.d}, e = ${\"q k\".x}, f = ${?g.h}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-29 toString with a substitution under an object", () -> deploy("/a", "a = ${b}\na = {y = 2}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-30 toString with a substitution over a number", () -> deploy("/a", "a = 1\na = ${b}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-31 toString with a number over a substitution", () -> deploy("/a", "a = ${b}\na = 2", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-32 toString with objects around a substitution", () -> deploy("/a", "a = {x = 1}\na = ${b}\na = {z = 3}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-33 toString with two substitutions", () -> deploy("/a", "a = ${b}\na = ${c}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-34 toString with a substitution and a nested key", () -> deploy("/a", "a.b = ${c}\na.b.d = 1\na.e = 2", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-35 toString with an optional substitution over an object", () -> deploy("/a", "a = {x = 1}\na = ${?b}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-36 toString with a number with a trailing zero before a substitution", () -> deploy("/a", "a = 1.50 ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-37 toString with an exponent number before a substitution", () -> deploy("/a", "a = 1e2 ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-38 toString with a boolean and two spaces before a substitution", () -> deploy("/a", "a = true  ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-39 toString with null before a substitution", () -> deploy("/a", "a = null ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-40 toString with a number after a substitution", () -> deploy("/a", "a = ${x} 1.50", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-41 toString with several numbers and a substitution", () -> deploy("/a", "a = 1   2 ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-42 toString with a number with leading zeros between substitutions", () -> deploy("/a", "a = ${x}007${y}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-43 toString with adjacent lists before a substitution", () -> deploy("/a", "a = [1] [2] ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-44 toString with adjacent objects before a substitution", () -> deploy("/a", "a = {x=1} {y=2} ${z}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-45 toString with lists around a substitution", () -> deploy("/a", "a = [1] ${x} [2]", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-46 toString with adjacent objects after a substitution", () -> deploy("/a", "a = ${x} {b=1} {c=2}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-47 toString with adjacent lists after a substitution", () -> deploy("/a", "a = ${x} [1] [2]", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-48 toString with adjacent objects that repeat a key", () -> deploy("/a", "a = {x=1} {x=2} ${z}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-49 toString with adjacent objects that merge a nested key", () -> deploy("/a", "a = {x={k=1}} {x={j=2}} ${z}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-50 toString with substitution paths of digits", () -> deploy("/a", "a = ${1}, b = ${a.007}, c = ${a.1.2}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-51 toString with substitution paths that start with a hyphen or look like keywords", () -> deploy("/a", "a = ${a.-1} ${-} ${-a} ${1-a} ${a.1e5} ${true} ${null.x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-52 toString with substitution paths that need quotes", () -> deploy("/a", "a = ${\"q k\"} ${\"\"} ${\"\\u00e9.x\"} ${\"1\"} ${\"a-b\"} ${\"_c\"} ${\"a b\".c}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-53 toString with substitution paths of non-ASCII letters", () -> deploy("/a", "a = ${\"\\u00e9\"} ${\"\\u03a91\"}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-54 toString with a substitution path of an astral character", () -> deploy("/a", "a = ${\"\\uD83D\\uDE00\"}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-55 toString with a lone number between substitutions", () -> deploy("/a", "a = ${x}1${y}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-56 toString with a lone boolean between substitutions", () -> deploy("/a", "a = ${x}true${y}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-57 toString with a string, a number and a substitution", () -> deploy("/a", "a = \"q\" 1 ${x}", NO_ROUTER, NO_SCOPE, "", "").toString());
    run("TS-58 toString with a substitution path of an astral letter", () -> deploy("/a", "a = ${\"\\uD835\\uDC00\"} ${\"\\u00e9\\uD835\\uDC00\"}", NO_ROUTER, NO_SCOPE, "", "").toString());
  }
}
