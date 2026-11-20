package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.atomic.AtomicInteger;
import org.apache.pekko.actor.ActorPath;
import org.apache.pekko.actor.ActorPaths;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Deploy;
import org.apache.pekko.actor.Deployer;
import org.apache.pekko.actor.ReflectiveDynamicAccess;

/**
 * Oracle for unit P10, row AC-CFG-012.2 (Deployer.deploy and Deployer.lookup): what the real {@code Deployer} of Apache Pekko
 * 2.0.0 does when deployments with wildcards are added and looked up (Deployer.scala:252-268, WildcardIndex.scala). The
 * Deployer is built over the real settings with a deployment section that holds only {@code default}, so that it starts
 * empty (reading that section is AC-CFG-012.1, a later slice).
 *
 * <p>The scripted cases are the lines of {@code crates/rakka-actor/examples/classic-parity/deployer.tsv}, which the Rust twin
 * runs with the same interpreter; its header explains the operations. The concurrency cases print invariants only (no sleeps,
 * no wall clock): threads released together by a barrier. Output contract (one deterministic line per case):
 * {@code <CASE-ID> -> ok <result>} or {@code <CASE-ID> -> ERR <Exception> [<message>]}. Case-id prefixes: DL (scripts), DC (concurrency).
 */
final class DeployerOracle {
  private DeployerOracle() {}

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

  /**
   * The real Deployer over the real settings, with its index emptied: the constructor already deploys the entries of
   * {@code pekko.actor.deployment} (the {@code /IO-DNS/*} ones of reference.conf, which Settings insists on), and reading that
   * section is AC-CFG-012.1. The index is the private {@code deployments} reference; {@code deploy} and {@code lookup} stay the real ones.
   */
  @SuppressWarnings("unchecked")
  private static Deployer newDeployer() {
    try {
      ClassLoader loader = DeployerOracle.class.getClassLoader();
      Config reference = ConfigFactory.defaultReference().resolve();
      Deployer deployer = new Deployer(new ActorSystem.Settings(loader, reference, "oracle"), new ReflectiveDynamicAccess(loader));
      java.lang.reflect.Field index = Deployer.class.getDeclaredField("deployments");
      index.setAccessible(true);
      ((java.util.concurrent.atomic.AtomicReference<Object>) index.get(deployer))
          .set(org.apache.pekko.util.WildcardIndex$.MODULE$.apply(
              org.apache.pekko.util.WildcardTree$.MODULE$.apply(), org.apache.pekko.util.WildcardTree$.MODULE$.apply()));
      return deployer;
    } catch (ReflectiveOperationException e) {
      throw new IllegalStateException(e);
    }
  }

  private static Deploy deployment(String path, String dispatcher) {
    return new Deploy(path, ConfigFactory.empty(), org.apache.pekko.routing.NoRouter$.MODULE$,
        org.apache.pekko.actor.NoScopeGiven$.MODULE$, dispatcher, "", scala.jdk.javaapi.CollectionConverters.asScala(new HashSet<String>()).toSet());
  }

  private static String found(scala.Option<Deploy> result) {
    return result.isDefined() ? "some(" + result.get().path() + "@" + result.get().dispatcher() + ")" : "none";
  }

  /** The word with its {@code \\uXXXX} escapes decoded; {@code ""} is the empty string. */
  private static String word(String w) { return w.equals("\"\"") ? "" : UtilCoreOracle.unesc(w); }

  private static List<String> elements(String w) {
    if (w.equals("-")) return Collections.emptyList();
    return Arrays.asList(word(w).split("/", -1));
  }

  private static String script(String text) {
    Deployer deployer = newDeployer();
    int deploys = 0;
    List<String> tokens = new ArrayList<>();
    for (String operation : text.split("; ")) {
      String[] words = operation.split(" ", -1);
      try {
        switch (words[0]) {
          case "deploy" -> {
            deploys++;
            deployer.deploy(deployment(word(words[1]), "d" + deploys));
            tokens.add("ok");
          }
          case "find" -> tokens.add(found(deployer.lookup(scala.jdk.javaapi.CollectionConverters.asScala((Iterable<String>) elements(words[1])))));
          case "lookup" -> {
            ActorPath path = ActorPaths.fromString(word(words[1]));
            tokens.add(found(deployer.lookup(path)));
          }
          default -> throw new IllegalArgumentException("unknown operation " + words[0]);
        }
      } catch (IllegalArgumentException e) {
        if (e.getMessage() != null && e.getMessage().startsWith("unknown operation")) throw e;
        tokens.add("ERR " + e.getClass().getSimpleName() + " [" + e.getMessage() + "]");
      } catch (RuntimeException e) {
        tokens.add("ERR " + e.getClass().getSimpleName() + " [" + e.getMessage() + "]");
      }
    }
    return String.join(" | ", tokens);
  }

  private static List<String[]> scripts() {
    try {
      Path classes = Paths.get(DeployerOracle.class.getProtectionDomain().getCodeSource().getLocation().toURI());
      Path file = classes.resolve("../../../crates/rakka-actor/examples/classic-parity/deployer.tsv").normalize();
      List<String[]> out = new ArrayList<>();
      for (String line : Files.readAllLines(file)) {
        if (line.isBlank() || line.startsWith("#")) continue;
        String[] f = line.split("\t", 2);
        if (f.length != 2) throw new IllegalStateException("bad line in deployer.tsv: " + line);
        out.add(f);
      }
      return out;
    } catch (java.io.IOException | java.net.URISyntaxException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  /** {@code threads} threads released together by a barrier, each adding {@code perThread} deployments of its own. */
  private static String concurrentDeploys(int threads, int perThread) throws Exception {
    Deployer deployer = newDeployer();
    CyclicBarrier start = new CyclicBarrier(threads);
    List<Thread> workers = new ArrayList<>();
    AtomicInteger failures = new AtomicInteger();
    for (int t = 0; t < threads; t++) {
      final int thread = t;
      Thread worker = new Thread(() -> {
        try {
          start.await();
          for (int i = 0; i < perThread; i++) deployer.deploy(deployment("/t" + thread + "/n" + i, "d"));
        } catch (Exception e) {
          failures.incrementAndGet();
        }
      });
      workers.add(worker);
      worker.start();
    }
    for (Thread worker : workers) worker.join();
    int present = 0;
    for (int t = 0; t < threads; t++)
      for (int i = 0; i < perThread; i++) {
        List<String> path = Arrays.asList("t" + t, "n" + i);
        if (deployer.lookup(scala.jdk.javaapi.CollectionConverters.asScala((Iterable<String>) path)).isDefined()) present++;
      }
    return "failures=" + failures.get() + " present=" + present + " of " + (threads * perThread);
  }

  private static String concurrentSamePath(int threads) throws Exception {
    Deployer deployer = newDeployer();
    CyclicBarrier start = new CyclicBarrier(threads);
    List<Thread> workers = new ArrayList<>();
    for (int t = 0; t < threads; t++) {
      final int thread = t;
      Thread worker = new Thread(() -> {
        try {
          start.await();
          deployer.deploy(deployment("/same/path", "d" + thread));
        } catch (Exception e) {
          throw new IllegalStateException(e);
        }
      });
      workers.add(worker);
      worker.start();
    }
    for (Thread worker : workers) worker.join();
    scala.Option<Deploy> result = deployer.lookup(scala.jdk.javaapi.CollectionConverters.asScala((Iterable<String>) Arrays.asList("same", "path")));
    boolean oneOfThem = result.isDefined() && result.get().dispatcher().matches("d[0-" + (threads - 1) + "]");
    return "found one of the deployments=" + oneOfThem;
  }

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    for (String[] entry : scripts()) run(entry[0], () -> script(entry[1]));
    run("DC-01 concurrent deploys of different paths are all kept", () -> concurrentDeploys(8, 25));
    run("DC-02 concurrent deploys of one path keep one of them", () -> concurrentSamePath(8));
  }
}
