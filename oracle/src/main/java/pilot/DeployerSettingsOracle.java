package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import com.typesafe.config.ConfigRenderOptions;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import java.util.concurrent.Callable;
import org.apache.pekko.actor.ActorPath;
import org.apache.pekko.actor.ActorPaths;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Deploy;
import org.apache.pekko.actor.Deployer;
import org.apache.pekko.actor.ReflectiveDynamicAccess;
import org.apache.pekko.routing.NoRouter$;

/**
 * Oracle for unit P10, row AC-CFG-012.1 (building the Deployer from {@code pekko.actor.deployment}): what the real
 * {@code Deployer} constructor of Apache Pekko 2.0.0 deploys and throws (Deployer.scala:225-250 and parseConfig at 270-276).
 * Each case is a line of {@code crates/rakka-actor/examples/classic-parity/deployer_settings.tsv}: the body of the deployment
 * section over the reference configuration, and lookups on the Deployer built from it. A router is printed by the name
 * configured in the entry, not by its class: building real routers is AC-CFG-012.3 (P37). The router of an entry that is not
 * {@code from-code} is built for real by Pekko, so a case whose router Pekko cannot build is outside the cases of this family.
 *
 * <p>Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <result>} or {@code <CASE-ID> -> ERR ...} for an
 * error of the whole case. Case-id prefixes: DS (parity), CO (several keys that give the same path: Rust follows a temporary
 * alphabetical policy, Q-CFG-36, so some of them are pending in the Rust twin).
 */
final class DeployerSettingsOracle {
  private DeployerSettingsOracle() {}

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

  private static String text(String w) { return UtilCoreOracle.unesc(w.replace("\\n", "\n")); }

  private static String word(String w) { return w.equals("\"\"") ? "" : UtilCoreOracle.unesc(w); }

  private static List<String> elements(String w) {
    if (w.equals("-")) return Collections.emptyList();
    return Arrays.asList(word(w).split("/", -1));
  }

  private static String show(scala.Option<Deploy> found) {
    if (found.isEmpty()) return "none";
    Deploy d = found.get();
    String router = d.routerConfig() == NoRouter$.MODULE$ ? "NoRouter" : d.config().getString("router");
    String scope = d.scope() == org.apache.pekko.actor.NoScopeGiven$.MODULE$ ? "NoScopeGiven" : "other";
    return "{path=\"" + d.path() + "\" dispatcher=\"" + d.dispatcher() + "\" mailbox=\"" + d.mailbox() + "\" router=" + router
        + " scope=" + scope + " config=" + d.config().root().render(ConfigRenderOptions.concise()) + "}";
  }

  private static String script(String section, String operations) {
    Config user = ConfigFactory.parseString("pekko.actor.deployment {\n" + text(section) + "\n}");
    Config config = user.withFallback(ConfigFactory.defaultReference()).resolve();
    ClassLoader loader = DeployerSettingsOracle.class.getClassLoader();
    Deployer deployer;
    try {
      deployer = new Deployer(new ActorSystem.Settings(loader, config, "oracle"), new ReflectiveDynamicAccess(loader));
    } catch (RuntimeException e) {
      return "ERR " + e.getClass().getSimpleName() + " [" + e.getMessage() + "]";
    }
    List<String> tokens = new ArrayList<>();
    for (String operation : operations.split("; ")) {
      if (operation.isEmpty()) continue;
      String[] words = operation.split(" ", -1);
      switch (words[0]) {
        case "find" -> tokens.add(show(deployer.lookup(scala.jdk.javaapi.CollectionConverters.asScala((Iterable<String>) elements(words[1])))));
        case "lookup" -> {
          ActorPath path = ActorPaths.fromString(word(words[1]));
          tokens.add(show(deployer.lookup(path)));
        }
        default -> throw new IllegalArgumentException("unknown operation " + words[0]);
      }
    }
    return tokens.isEmpty() ? "built" : String.join(" | ", tokens);
  }

  private static List<String[]> scripts() {
    try {
      Path classes = Paths.get(DeployerSettingsOracle.class.getProtectionDomain().getCodeSource().getLocation().toURI());
      Path file = classes.resolve("../../../crates/rakka-actor/examples/classic-parity/deployer_settings.tsv").normalize();
      List<String[]> out = new ArrayList<>();
      for (String line : Files.readAllLines(file)) {
        if (line.isBlank() || line.startsWith("#")) continue;
        String[] f = line.split("\t", 3);
        if (f.length != 3) throw new IllegalStateException("bad line in deployer_settings.tsv: " + line);
        out.add(f);
      }
      return out;
    } catch (java.io.IOException | java.net.URISyntaxException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    for (String[] entry : scripts()) run(entry[0], () -> script(entry[1], entry[2]));
  }
}
