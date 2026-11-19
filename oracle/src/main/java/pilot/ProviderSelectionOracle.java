package pilot;

import java.util.*;
import java.util.concurrent.Callable;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.ProviderSelection;

/**
 * Oracle for unit P09, row AC-CFG-008 (pekko.actor.provider and ProviderSelection): which ProviderSelection, class name and
 * cluster flag a real {@code ActorSystem.Settings} of Apache Pekko 2.0.0 derives from the value of {@code pekko.actor.provider}
 * (ActorSystem.scala:102-144 and :411-421, ProviderSelectionSpec.scala:38-67). The Settings constructor is public and needs no
 * running system; every case builds one over the merged reference.conf files (as {@link ConfigSettingsOracle} does) and prints
 * what {@code ProviderSelectionType}, {@code ProviderClass} and {@code HasCluster} hold. The priority of a
 * {@code BootstrapSetup.actorRefProvider} over the configuration is AC-CFG-015.
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <result>} or
 * {@code <CASE-ID> -> ERR <ExceptionSimpleName> [<message>]}. Case-id prefixes: SEL (the selection a value gives), EQ (equality of two
 * selections).
 */
final class ProviderSelectionOracle {
  private ProviderSelectionOracle() {}

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

  private static final String REMOTE = "org.apache.pekko.remote.RemoteActorRefProvider";
  private static final String CLUSTER = "org.apache.pekko.cluster.ClusterActorRefProvider";
  private static final String LOCAL = "org.apache.pekko.actor.LocalActorRefProvider";

  /** Settings whose pekko.actor.provider is the given HOCON value text (a quoted string unless stated otherwise). */
  private static ActorSystem.Settings settingsWith(String hoconValue) {
    return ConfigSettingsOracle.set("pekko.actor.provider = " + hoconValue);
  }

  private static ActorSystem.Settings settingsWithString(String value) {
    return settingsWith("\"" + value + "\"");
  }

  private static String selected(String value) {
    ActorSystem.Settings s = settingsWithString(value);
    ProviderSelection p = s.ProviderSelectionType();
    return "identifier=" + p.identifier() + " fqcn=" + p.fqcn() + " hasCluster=" + p.hasCluster()
        + " ProviderClass=" + s.ProviderClass() + " HasCluster=" + s.HasCluster();
  }

  private static String same(String a, String b) {
    return String.valueOf(settingsWithString(a).ProviderSelectionType().equals(settingsWithString(b).ProviderSelectionType()));
  }

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    ConfigSettingsOracle.ids(); // builds the base configuration
    // ProviderSelectionSpec.scala:38-43, :45-51, :53-59, :61-67.
    run("SEL-01 provider = local", () -> selected("local"));
    run("SEL-02 provider = remote", () -> selected("remote"));
    run("SEL-03 provider = cluster", () -> selected("cluster"));
    run("SEL-04 provider = " + REMOTE, () -> selected(REMOTE));
    run("SEL-05 provider = " + CLUSTER, () -> selected(CLUSTER));
    run("SEL-06 provider = other.ActorRefProvider", () -> selected("other.ActorRefProvider"));
    // ActorSystem.scala:137-144: only these exact strings select Local, Remote or Cluster; any other string is Custom.
    run("SEL-07 provider = " + LOCAL + " (the class name of Local is Custom)", () -> selected(LOCAL));
    run("SEL-08 provider = LOCAL (case differs)", () -> selected("LOCAL"));
    run("SEL-09 provider = Remote (case differs)", () -> selected("Remote"));
    run("SEL-10 provider = \" local\" (leading space)", () -> selected(" local"));
    run("SEL-11 provider = \"cluster \" (trailing space)", () -> selected("cluster "));
    run("SEL-12 provider = empty string", () -> selected(""));
    run("SEL-13 provider = 5 (a number)", () -> {
      ActorSystem.Settings s = settingsWith("5");
      return "identifier=" + s.ProviderSelectionType().identifier() + " ProviderClass=" + s.ProviderClass();
    });
    // ProviderSelectionSpec.scala:48 and :56: the short name and the class name select the same value.
    run("EQ-01 remote equals " + REMOTE, () -> same("remote", REMOTE));
    run("EQ-02 cluster equals " + CLUSTER, () -> same("cluster", CLUSTER));
    run("EQ-03 local equals " + LOCAL, () -> same("local", LOCAL));
    run("EQ-04 two Custom with the same name", () -> same("other.ActorRefProvider", "other.ActorRefProvider"));
    run("EQ-05 two Custom with different names", () -> same("other.A", "other.B"));
    run("EQ-06 remote equals cluster", () -> same("remote", "cluster"));
    // A value that is not text: the scalars are read as their text, a list or an object fails in Settings (getString), and null
    // is a missing value (ConfigException.Null). Settings.getString runs after checkValid, which accepts any value for a STRING.
    for (String[] v : new String[][] {
        {"SEL-14 provider = [local] (a list)", "[local]"},
        {"SEL-15 provider = [] (an empty list)", "[]"},
        {"SEL-16 provider = {a=1} (an object)", "{a=1}"},
        {"SEL-17 provider = null", "null"},
        {"SEL-18 provider = true (a boolean)", "true"},
        {"SEL-19 provider = 1.5 (a decimal number)", "1.5"}}) {
      final String hocon = v[1];
      run(v[0], () -> {
        ActorSystem.Settings s = settingsWith(hocon);
        return "identifier=" + s.ProviderSelectionType().identifier() + " ProviderClass=" + s.ProviderClass();
      });
    }
  }
}
