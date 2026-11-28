package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigException;
import com.typesafe.config.ConfigFactory;
import java.util.concurrent.TimeUnit;
import org.apache.pekko.actor.ActorSystem;

/**
 * Cases {@code SYS} of the config-settings family (AC-CFG-003, AC-CFG-020): a real {@code ActorSystem(name, config)} of
 * Apache Pekko 2.0.0 is created and terminated, or rejects its configuration.
 *
 * pekko-slf4j is hidden from the class loader of the system, because rakka has no Slf4jLogger; a configuration that names
 * that logger itself gets the real class loader. Errors of the configuration library print only their class, because their
 * message describes the origin of the value (Q-CFG-15).
 */
final class SystemFromConfigOracle {
  private SystemFromConfigOracle() {}

  /** Keeps the INFO line of CoordinatedShutdown, which has a timestamp, out of the output. */
  private static final String QUIET = "pekko.loglevel = WARNING\n";

  private static final String CS ="pekko.coordinated-shutdown.";
  private static final String CS_OFF = CS + "terminate-actor-system = off\n";
  private static final String OLD = "pekko.version = \"1.0.0\"\n";
  private static final String BAD_VNF = "pekko.actor.deployment.default.virtual-nodes-factor = abc\n";
  private static final String BAD_PATH = "pekko.actor.deployment.\"/a b\".dispatcher = x\n";
  private static final String SERIALIZE = "pekko.actor.serialize-messages = on\n";

  static void register() {
    add("the empty configuration", "");
    add("pekko.version set to the version", "pekko.version = \"2.0.0\"");
    for (String v : new String[] {"\"0.0.1\"", "\"1.7.1\"", "\"\"", "\"2.0.0-M1\"", "\"2.0\"", "2", "\"2.0.1\"", "\" 2.0.0\"", "\"2.0.0 \""}) {
      add("pekko.version = " + v, "pekko.version = " + v);
    }
    add("another version and serialize-messages on", OLD + SERIALIZE);
    add("another version and a bad virtual-nodes-factor", OLD + BAD_VNF);
    add("another version and an object where a string is expected", OLD + "pekko.loglevel { x = 1 }");
    add("another version and a bad daemonic", OLD + "pekko.daemonic = abc");
    add("another version and use-slf4j", OLD + "pekko.use-slf4j = on");
    add("another version and a rejected deployment path", OLD + BAD_PATH);
    add("an invalid name and another version", "-sys", OLD);
    add("terminate-actor-system off", CS_OFF);
    add("terminate-actor-system off and run-by-terminate on", CS_OFF + CS + "run-by-actor-system-terminate = on");
    add("terminate-actor-system false and run-by-terminate true",
        CS + "terminate-actor-system = false\n" + CS + "run-by-actor-system-terminate = true");
    add("terminate-actor-system no and run-by-terminate yes",
        CS + "terminate-actor-system = no\n" + CS + "run-by-actor-system-terminate = yes");
    add("terminate-actor-system quoted off", CS + "terminate-actor-system = \"off\"");
    add("terminate-actor-system off as a nested object", "pekko.coordinated-shutdown { terminate-actor-system = off }");
    add("both shutdown flags off", CS_OFF + CS + "run-by-actor-system-terminate = off");
    add("both shutdown flags on", CS + "terminate-actor-system = on\n" + CS + "run-by-actor-system-terminate = on");
    add("run-by-terminate off alone", CS + "run-by-actor-system-terminate = off");
    add("terminate-actor-system of a wrong type", CS + "terminate-actor-system = abc");
    add("the shutdown combination and another version", CS_OFF + OLD);
    add("the shutdown combination and a bad virtual-nodes-factor", CS_OFF + BAD_VNF);
    add("the shutdown combination and serialize-messages on", CS_OFF + SERIALIZE);
    add("the shutdown combination and a rejected deployment path", CS_OFF + BAD_PATH);
    add("the shutdown combination and an invalid name", "a b", CS_OFF);
    add("a rejected deployment path", BAD_PATH);
    add("a creation timeout of the application", "pekko.actor.creation-timeout = 7s");
    add("an unstarted push timeout of the application", "pekko.actor.unstarted-push-timeout = 3s");
    add("log-dead-letters off", "pekko.log-dead-letters = off");
    add("use-slf4j on without pekko-slf4j", "pekko.use-slf4j = on");
    add("another system name", "My-Sys_2", "");
    ConfigSettingsOracle.run("SYS the slf4j logger listed by the application", () ->
        create("oracle", "pekko.loggers = [\"" + ConfigSettingsOracle.SLF4J_LOGGER + "\"]",
            SystemFromConfigOracle.class.getClassLoader()));
  }

  private static void add(String label, String app) { add(label, "oracle", app); }

  private static void add(String label, String name, String app) {
    ConfigSettingsOracle.run("SYS " + label, () -> create(name, app,
        new ConfigSettingsOracle.HidingClassLoader(SystemFromConfigOracle.class.getClassLoader())));
  }

  private static String create(String name, String app, ClassLoader loader) throws Exception {
    Config config = ConfigFactory.parseString(QUIET + app);
    ActorSystem system;
    try {
      system = ActorSystem.create(name, config, loader);
    } catch (ConfigException e) {
      return "rejected=" + e.getClass().getSimpleName();
    }
    try {
      ActorSystem.Settings s = system.settings();
      return "name=" + system.name()
          + " settings.name=" + s.name()
          + " Version=" + ActorSystem.Version()
          + " pekko.version=" + s.config().getString("pekko.version")
          + " ConfigVersion=" + s.ConfigVersion()
          + " CreationTimeout=" + s.CreationTimeout().duration().toMillis() + "ms"
          + " UnstartedPushTimeout=" + s.UnstartedPushTimeout().duration().toMillis() + "ms"
          + " LogDeadLetters=" + s.LogDeadLetters()
          + " Loggers=" + ConfigSettingsOracle.list(s.Loggers())
          + " LoggingFilter=" + s.LoggingFilter()
          + " TerminateActorSystem=" + s.CoordinatedShutdownTerminateActorSystem()
          + " RunByTerminate=" + s.CoordinatedShutdownRunByActorSystemTerminate();
    } finally {
      system.terminate();
      system.getWhenTerminated().toCompletableFuture().get(20, TimeUnit.SECONDS);
    }
  }
}
