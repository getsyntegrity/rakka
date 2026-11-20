package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.Callable;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.BootstrapSetup;
import org.apache.pekko.actor.ProviderSelection;
import org.apache.pekko.actor.setup.ActorSystemSetup;
import org.apache.pekko.actor.setup.Setup;

/**
 * Oracle for unit P09, row AC-CFG-015 (Setup, ActorSystemSetup and BootstrapSetup): what the real classes of Apache Pekko 2.0.0
 * do (ActorSystemSetup.scala:25-97, ActorSystem.scala:48-179, ActorSystemSetupSpec.scala:31-78, ActorSystemSetupTest.java:23-34).
 * The setups of the spec are Java classes here ({@link DummySetup}, {@link DummySetup2}, {@link DummySetup3}, {@link JavaSetup}).
 * The priority of {@code BootstrapSetup.actorRefProvider} over {@code pekko.actor.provider} is read from the public
 * {@code ActorSystem.Settings(classLoader, config, name, setup)} constructor (ActorSystem.scala:410-421).
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <result>} or {@code <CASE-ID> -> ERR <Exception> [<message>]}.
 * Case-id prefixes: AS (ActorSystemSetup), TS (its toString), BS (BootstrapSetup), PR (provider priority in Settings).
 */
final class ActorSystemSetupOracle {
  private ActorSystemSetupOracle() {}

  public static class DummySetup extends Setup {
    public final String name;
    public DummySetup(String name) { this.name = name; }
  }

  public static class DummySetup2 extends Setup {
    public final String name;
    public DummySetup2(String name) { this.name = name; }
  }

  public static class DummySetup3 extends Setup {
    public final String name;
    public DummySetup3(String name) { this.name = name; }
  }

  public static class JavaSetup extends Setup {
    public final String name;
    public JavaSetup(String name) { this.name = name; }
  }

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

  private static <T extends Setup> String got(ActorSystemSetup setups, Class<T> type, java.util.function.Function<T, String> show) {
    Optional<T> found = setups.get(type);
    return found.isPresent() ? show.apply(found.get()) : "absent";
  }

  private static String names(ActorSystemSetup s) {
    return "DummySetup=" + got(s, DummySetup.class, d -> d.name) + " DummySetup2=" + got(s, DummySetup2.class, d -> d.name);
  }

  private static String bootstrap(BootstrapSetup b) {
    String config = b.config().isDefined() ? "x=" + b.config().get().getInt("x") : "absent";
    String provider = b.actorRefProvider().isDefined() ? b.actorRefProvider().get().identifier() : "absent";
    return "config=" + config + " provider=" + provider;
  }

  private static Config configWithX(int x) { return ConfigFactory.parseString("x = " + x); }

  private static String providerOf(String configProvider, ActorSystemSetup setup) {
    ActorSystem.Settings s = new ActorSystem.Settings(ActorSystemSetupOracle.class.getClassLoader(),
        ConfigSettingsOracle.parse("pekko.actor.provider = \"" + configProvider + "\"").withFallback(ConfigSettingsOracle.baseConfig()).resolve(),
        "oracle", setup);
    ProviderSelection p = s.ProviderSelectionType();
    return "identifier=" + p.identifier() + " ProviderClass=" + s.ProviderClass() + " HasCluster=" + s.HasCluster();
  }

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    ConfigSettingsOracle.ids(); // builds the base configuration
    // ActorSystemSetupSpec.scala:31-37
    run("AS-01 store and retrieve a setup", () -> names(ActorSystemSetup.empty().withSetup(new DummySetup("Al Dente"))));
    run("AS-02 get on an empty setup", () -> names(ActorSystemSetup.empty()));
    // ActorSystemSetupSpec.scala:39-45
    run("AS-03 replace the setup of the same class", () ->
        names(ActorSystemSetup.empty().withSetup(new DummySetup("Al Dente")).withSetup(new DummySetup("Earl E. Bird"))));
    // ActorSystemSetupSpec.scala:47-55: a and b and c
    run("AS-04 and with a repeated class", () -> {
      Setup a = new DummySetup("Al Dente");
      Setup b = new DummySetup("Earl E. Bird");
      Setup c = new DummySetup2("Amanda Reckonwith");
      return names(a.and(b).and(c));
    });
    run("AS-05 and with two different classes", () -> names(new DummySetup("Al Dente").and(new DummySetup2("Amanda Reckonwith"))));
    // ActorSystemSetupSpec.scala:57-65
    run("AS-06 create with several setups", () -> {
      ActorSystemSetup s = ActorSystemSetup.create(new DummySetup("Manny Kin"), new DummySetup2("Pepe Roni"));
      return "DummySetup=" + (s.get(DummySetup.class).isPresent() ? "present" : "absent")
          + " DummySetup2=" + (s.get(DummySetup2.class).isPresent() ? "present" : "absent")
          + " DummySetup3=" + (s.get(DummySetup3.class).isPresent() ? "present" : "absent");
    });
    run("AS-07 create with two setups of the same class", () ->
        names(ActorSystemSetup.create(new DummySetup("first"), new DummySetup("second"))));
    // ActorSystemSetupTest.java:23-34
    run("AS-08 the Java API", () -> {
      JavaSetup javaSetting = new JavaSetup("Jasmine Rice");
      Optional<JavaSetup> result = ActorSystemSetup.create().withSetup(javaSetting).get(JavaSetup.class);
      return result.isPresent() + " " + (result.get() == javaSetting) + " " + result.get().name;
    });
    run("AS-09 withSetup leaves the original setup unchanged", () -> {
      ActorSystemSetup original = ActorSystemSetup.empty();
      ActorSystemSetup changed = original.withSetup(new DummySetup("Al Dente"));
      return "original " + names(original) + " changed " + names(changed);
    });
    run("AS-10 and on an ActorSystemSetup equals withSetup", () -> {
      ActorSystemSetup base = ActorSystemSetup.empty().withSetup(new DummySetup("Al Dente"));
      return names(base.and(new DummySetup2("Amanda Reckonwith"))) + " | " + names(base.withSetup(new DummySetup2("Amanda Reckonwith")));
    });
    // ActorSystemSetup.scala:94 toString
    run("TS-01 toString of an empty setup", () -> ActorSystemSetup.empty().toString());
    run("TS-02 toString with one setup", () -> ActorSystemSetup.empty().withSetup(new DummySetup("a")).toString());
    run("TS-03 toString with two setups in insertion order", () ->
        ActorSystemSetup.empty().withSetup(new DummySetup2("b")).withSetup(new DummySetup("a")).toString());
    run("TS-04 toString after replacing the first setup keeps its position", () ->
        ActorSystemSetup.empty().withSetup(new DummySetup("a")).withSetup(new DummySetup2("b")).withSetup(new DummySetup("c")).toString());
    run("TS-05 toString with four setups", () ->
        ActorSystemSetup.create(new DummySetup3("c"), new JavaSetup("j"), new DummySetup2("b"), new DummySetup("a")).toString());
    run("TS-06 toString of create with a repeated class", () ->
        ActorSystemSetup.create(new DummySetup("a"), new DummySetup2("b"), new DummySetup("c")).toString());
    run("TS-07 toString with a BootstrapSetup", () -> ActorSystemSetup.create(BootstrapSetup.create()).toString());
    // ActorSystem.scala:48-98 and :160-178
    run("BS-01 BootstrapSetup()", () -> bootstrap(BootstrapSetup.apply()));
    run("BS-02 BootstrapSetup.create()", () -> bootstrap(BootstrapSetup.create()));
    run("BS-03 BootstrapSetup(config)", () -> bootstrap(BootstrapSetup.apply(configWithX(1))));
    run("BS-04 BootstrapSetup.create(config)", () -> bootstrap(BootstrapSetup.create(configWithX(2))));
    run("BS-05 withConfig", () -> bootstrap(BootstrapSetup.apply().withConfig(configWithX(3))));
    run("BS-06 withActorRefProvider(Remote)", () -> bootstrap(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.remote())));
    run("BS-07 withActorRefProvider(Cluster)", () -> bootstrap(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.cluster())));
    run("BS-08 withActorRefProvider(Local)", () -> bootstrap(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.local())));
    run("BS-09 withActorRefProvider(Custom)", () ->
        bootstrap(BootstrapSetup.apply().withActorRefProvider(new ProviderSelection.Custom("other.ActorRefProvider"))));
    run("BS-10 withConfig then withActorRefProvider keeps the config", () ->
        bootstrap(BootstrapSetup.apply().withConfig(configWithX(4)).withActorRefProvider(ProviderSelection.cluster())));
    run("BS-11 withActorRefProvider then withConfig keeps the provider", () ->
        bootstrap(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.remote()).withConfig(configWithX(5))));
    run("BS-12 withConfig twice replaces the config", () ->
        bootstrap(BootstrapSetup.apply().withConfig(configWithX(6)).withConfig(configWithX(7))));
    run("BS-13 withers leave the original unchanged", () -> {
      BootstrapSetup original = BootstrapSetup.apply();
      BootstrapSetup changed = original.withConfig(configWithX(8)).withActorRefProvider(ProviderSelection.remote());
      return "original " + bootstrap(original) + " changed " + bootstrap(changed);
    });
    run("BS-14 a BootstrapSetup stored in an ActorSystemSetup", () -> {
      BootstrapSetup b = BootstrapSetup.apply().withConfig(configWithX(9));
      Optional<BootstrapSetup> found = ActorSystemSetup.create(b).get(BootstrapSetup.class);
      return found.isPresent() + " " + (found.get() == b) + " " + bootstrap(found.get());
    });
    // not produced by Rust (Q-CFG-32): the JVM-only fields
    run("BS-15 withClassloader", () -> {
      ClassLoader loader = new ClassLoader() {};
      BootstrapSetup b = BootstrapSetup.apply().withClassloader(loader);
      return "classLoader=" + (b.classLoader().isDefined() && b.classLoader().get() == loader);
    });
    run("BS-16 withDefaultExecutionContext", () -> {
      scala.concurrent.ExecutionContext ec = scala.concurrent.ExecutionContext.global();
      BootstrapSetup b = BootstrapSetup.apply().withDefaultExecutionContext(ec);
      return "defaultExecutionContext=" + (b.defaultExecutionContext().isDefined() && b.defaultExecutionContext().get() == ec);
    });
    // ActorSystem.scala:410-421: a BootstrapSetup.actorRefProvider has priority over pekko.actor.provider; Settings.setup
    run("PR-01 no setup uses the configuration (local)", () -> providerOf("local", ActorSystemSetup.empty()));
    run("PR-02 an empty BootstrapSetup equals none", () -> providerOf("remote", ActorSystemSetup.create(BootstrapSetup.apply())));
    run("PR-03 the provider of the BootstrapSetup wins over the configuration (Remote over local)", () ->
        providerOf("local", ActorSystemSetup.create(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.remote()))));
    run("PR-04 the provider of the BootstrapSetup wins (Cluster over remote)", () ->
        providerOf("remote", ActorSystemSetup.create(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.cluster()))));
    run("PR-05 the provider of the BootstrapSetup wins (Local over a custom class)", () ->
        providerOf("other.ActorRefProvider", ActorSystemSetup.create(BootstrapSetup.apply().withActorRefProvider(ProviderSelection.local()))));
    run("PR-06 a Custom provider of the BootstrapSetup is selected through its identifier", () ->
        providerOf("local", ActorSystemSetup.create(BootstrapSetup.apply().withActorRefProvider(new ProviderSelection.Custom("other.ActorRefProvider")))));
    run("PR-07 a BootstrapSetup with only a config keeps the configured provider", () ->
        providerOf("cluster", ActorSystemSetup.create(BootstrapSetup.apply().withConfig(configWithX(1)))));
    run("PR-08 other setups do not affect the provider", () ->
        providerOf("remote", ActorSystemSetup.create(new DummySetup("Tad Moore"))));
    // ActorSystemSetupSpec.scala:67-78: the setup is available from settings.setup
    run("PR-09 the setup is available from Settings.setup", () -> {
      ActorSystem.Settings s = new ActorSystem.Settings(ActorSystemSetupOracle.class.getClassLoader(),
          ConfigSettingsOracle.parse("").withFallback(ConfigSettingsOracle.baseConfig()).resolve(), "oracle",
          ActorSystemSetup.create(new DummySetup("Tad Moore")));
      return names(s.setup());
    });
    run("PR-10 Settings without a setup has an empty one", () -> {
      ActorSystem.Settings s = ConfigSettingsOracle.set("");
      return names(s.setup()) + " " + s.setup();
    });
  }
}
