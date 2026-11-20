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
 * Apache Pekko 2.0.0 do (Deployer.scala:28-205, RouterConfig.scala:94,373). A router is printed by its class only
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
    // Deployer.scala:28-47, 62-70
    run("DP-01 Deploy() has the defaults", () -> show(Deploy.apply("", ConfigFactory.empty(), NO_ROUTER, NO_SCOPE, "", "")));
    run("DP-02 Deploy.local has the local scope", () -> show(Deploy.local()));
    run("DP-03 the fields are kept as given", () ->
        show(deploy("/user/a", "x = 1, y = \"s\"", new RoundRobinPool(1), LOCAL, "my-dispatcher", "my-mailbox")));
    run("DP-04 withTags keeps the other fields", () -> show(plain("/a", "d", "m").withTags(tags("t2", "t1"))));
    // Deployer.scala:134-147 (equality over the seven fields)
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
    // Deployer.scala:104-112
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
    // Deployer.scala:176-196
    run("SC-01 local keeps itself over no scope given", () -> scope(LOCAL.withFallback(NO_SCOPE)));
    run("SC-02 local keeps itself over local", () -> scope(LOCAL.withFallback(LOCAL)));
    run("SC-03 no scope given takes local", () -> scope(NO_SCOPE.withFallback(LOCAL)));
    run("SC-04 no scope given over no scope given", () -> scope(NO_SCOPE.withFallback(NO_SCOPE)));
    // RouterConfig.scala:94,373
    run("RT-01 no router takes a router", () -> router(NO_ROUTER.withFallback(new RoundRobinPool(1))));
    run("RT-02 a router over no router", () -> router(new RoundRobinPool(1).withFallback(NO_ROUTER)));
    run("RT-03 no router over no router", () -> router(NO_ROUTER.withFallback(NO_ROUTER)));
    run("RT-04 a router over another router keeps its class", () -> router(new RoundRobinPool(1).withFallback(new RandomPool(1))));
  }
}
