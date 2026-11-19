package pilot;

import com.typesafe.config.*;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.concurrent.TimeUnit;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.ReflectiveDynamicAccess;
import scala.jdk.javaapi.CollectionConverters;

/**
 * Oracle for unit P08 (CFG-AJUSTES): what the real {@code ActorSystem.Settings} of Apache Pekko 2.0.0 reads and rejects, and
 * what {@code Settings.amendSlf4jConfig} does. The Settings constructor is public and needs no running system, so every case builds
 * a {@code Settings} from a configuration (HOCON text over the merged reference.conf files of pekko-actor and pekko-actor-typed)
 * and prints its fields.
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <fields>} or {@code <CASE-ID> -> ERR <Exception> [<message>]}.
 * Case-id prefixes: SET (Settings), AMEND (amendSlf4jConfig).
 *
 * The base configuration is built from the TEXT of the two reference.conf files (not from the jar URLs), so that the origin
 * description of every value is {@code String: N} and no case depends on the path of the jar.
 */
final class ConfigSettingsOracle {
  private ConfigSettingsOracle() {}

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

  private static boolean registered = false;
  private static Config base;
  private static Config actorOnly;

  private static String textOf(URL u) {
    try (java.io.InputStream in = u.openStream()) {
      return new String(in.readAllBytes(), StandardCharsets.UTF_8);
    } catch (java.io.IOException e) {
      throw new IllegalStateException(e);
    }
  }

  static Config parse(String hocon) { return ConfigFactory.parseString(hocon); }

  static ActorSystem.Settings settings(Config c) {
    return new ActorSystem.Settings(ConfigSettingsOracle.class.getClassLoader(), c, "oracle");
  }

  /** Settings over {@code app} merged on the base (typed over actor reference.conf, resolved). */
  static ActorSystem.Settings set(String app) { return settings(parse(app).withFallback(base).resolve()); }

  static String list(scala.collection.Seq<String> s) { return CollectionConverters.asJava(s).toString(); }

  static String duration(scala.concurrent.duration.Duration d) {
    return d.isFinite() ? d.toMillis() + "ms" : "Inf";
  }

  static String show(ActorSystem.Settings s) {
    List<String> prefix = new ArrayList<>(CollectionConverters.asJava(s.NoSerializationVerificationNeededClassPrefix()));
    Collections.sort(prefix);
    return "ConfigVersion=" + s.ConfigVersion()
        + " SupervisorStrategyClass=" + s.SupervisorStrategyClass()
        + " CreationTimeout=" + s.CreationTimeout().duration().toMillis() + "ms"
        + " UnstartedPushTimeout=" + s.UnstartedPushTimeout().duration().toMillis() + "ms"
        + " AllowJavaSerialization=" + s.AllowJavaSerialization()
        + " SerializeAllMessages=" + s.SerializeAllMessages()
        + " SerializeAllCreators=" + s.SerializeAllCreators()
        + " NoSerializationVerificationNeededClassPrefix=" + prefix
        + " LogLevel=" + s.LogLevel()
        + " StdoutLogLevel=" + s.StdoutLogLevel()
        + " Loggers=" + list(s.Loggers())
        + " LoggersDispatcher=" + s.LoggersDispatcher()
        + " LoggingFilter=" + s.LoggingFilter()
        + " LoggerStartTimeout=" + s.LoggerStartTimeout().duration().toMillis() + "ms"
        + " LogConfigOnStart=" + s.LogConfigOnStart()
        + " LogDeadLetters=" + s.LogDeadLetters()
        + " LogDeadLettersDuringShutdown=" + s.LogDeadLettersDuringShutdown()
        + " LogDeadLettersSuspendDuration=" + duration(s.LogDeadLettersSuspendDuration())
        + " AddLoggingReceive=" + s.AddLoggingReceive()
        + " DebugAutoReceive=" + s.DebugAutoReceive()
        + " DebugLifecycle=" + s.DebugLifecycle()
        + " FsmDebugEvent=" + s.FsmDebugEvent()
        + " DebugEventStream=" + s.DebugEventStream()
        + " DebugUnhandledMessage=" + s.DebugUnhandledMessage()
        + " DebugRouterMisconfiguration=" + s.DebugRouterMisconfiguration()
        + " Home=" + s.Home()
        + " SchedulerClass=" + s.SchedulerClass()
        + " Daemonicity=" + s.Daemonicity()
        + " JvmExitOnFatalError=" + s.JvmExitOnFatalError()
        + " JvmShutdownHooks=" + s.JvmShutdownHooks()
        + " FailMixedVersions=" + s.FailMixedVersions()
        + " CoordinatedShutdownTerminateActorSystem=" + s.CoordinatedShutdownTerminateActorSystem()
        + " CoordinatedShutdownRunByActorSystemTerminate=" + s.CoordinatedShutdownRunByActorSystemTerminate()
        + " DefaultVirtualNodesFactor=" + s.DefaultVirtualNodesFactor();
  }

  static synchronized void register() {
    if (registered) return;
    registered = true;
    Config actor = parse(textOf(ConfigHoconOracle.referenceOf("pekko-actor")));
    Config typed = parse(textOf(ConfigHoconOracle.referenceOf("pekko-actor-typed")));
    actorOnly = actor.resolve();
    base = typed.withFallback(actor).resolve();
    registerDefaults();
    registerDeadLetters();
    registerDebugAndJvm();
    registerTimeoutsAndLogging();
    registerSerializationAndShutdown();
    registerInvalid();
    registerAmend();
  }

  static void registerDefaults() {
    run("SET-DEFAULT the reference configuration", () -> show(set("")));
    run("SET-A9 dispatcher and mailbox keys of ConfigSpec", () -> {
      Config c = set("").config().getConfig("pekko.actor.default-dispatcher");
      Config pool = c.getConfig("default-executor");
      Config fj = c.getConfig("fork-join-executor");
      Config tp = c.getConfig("thread-pool-executor");
      Config mb = set("").config().getConfig("pekko.actor.default-mailbox");
      return "type=" + c.getString("type") + " executor=" + c.getString("executor")
          + " shutdown-timeout=" + c.getDuration("shutdown-timeout", TimeUnit.MILLISECONDS)
          + " throughput=" + c.getInt("throughput")
          + " throughput-deadline-time=" + c.getDuration("throughput-deadline-time", TimeUnit.MILLISECONDS)
          + " attempt-teamwork=" + c.getBoolean("attempt-teamwork")
          + " fallback=" + pool.getString("fallback")
          + " fj-parallelism-min=" + fj.getInt("parallelism-min")
          + " fj-parallelism-factor=" + fj.getDouble("parallelism-factor")
          + " fj-parallelism-max=" + fj.getInt("parallelism-max")
          + " fj-task-peeking-mode=" + fj.getString("task-peeking-mode")
          + " tp-keep-alive-time=" + tp.getDuration("keep-alive-time", TimeUnit.MILLISECONDS)
          + " tp-core-pool-size-factor=" + tp.getDouble("core-pool-size-factor")
          + " tp-max-pool-size-factor=" + tp.getDouble("max-pool-size-factor")
          + " tp-task-queue-size=" + tp.getInt("task-queue-size")
          + " tp-task-queue-type=" + tp.getString("task-queue-type")
          + " tp-allow-core-timeout=" + tp.getBoolean("allow-core-timeout")
          + " tp-fixed-pool-size=" + tp.getString("fixed-pool-size")
          + " mailbox-capacity=" + mb.getInt("mailbox-capacity")
          + " mailbox-push-timeout-time=" + mb.getDuration("mailbox-push-timeout-time", TimeUnit.MILLISECONDS)
          + " mailbox-type=" + mb.getString("mailbox-type");
    });
    run("SET-EXT default library-extensions and extensions", () -> {
      Config c = set("").config();
      return "library-extensions=" + c.getStringList("pekko.library-extensions") + " extensions=" + c.getStringList("pekko.extensions");
    });
    run("SET-EXT library-extensions += and extensions set by the application", () -> {
      Config c = set("pekko.library-extensions += \"my.Ext\"\npekko.extensions = [\"a.B\", \"c.D$\"]").config();
      return "library-extensions=" + c.getStringList("pekko.library-extensions") + " extensions=" + c.getStringList("pekko.extensions");
    });
    run("SET-EXT library-extensions replaced by the application", () -> {
      Config c = set("pekko.library-extensions = [\"only.One\"]").config();
      return "library-extensions=" + c.getStringList("pekko.library-extensions");
    });
    run("SET-EXT extensions of a wrong type", () -> set("pekko.extensions = 5").config().getStringList("pekko.extensions") + "");
    run("SET-PROVIDER default provider key", () -> set("").config().getString("pekko.actor.provider"));
  }

  static void registerDeadLetters() {
    for (String v : new String[] {"on", "off", "true", "false", "ON", "Off", "TRUE", "False", "\"ON\"", "\"off\"", "10", "0", "5", "-1",
        "2147483647", "\"5\"", "yes", "no", "1.5", "abc", "3000000000", "\"\"", "[1]"}) {
      final String text = v;
      run("SET-LDL log-dead-letters = " + text, () -> {
        ActorSystem.Settings s = set("pekko.log-dead-letters = " + text);
        return "LogDeadLetters=" + s.LogDeadLetters();
      });
    }
    for (String v : new String[] {"infinite", "Infinite", "INFINITE", "inf", "5s", "1 minute", "0", "100", "-1s", "off", "unlimited", "1.5s",
        "5 foo", "106752 d", "106751 d", "9223372036854775807 d", "9223372036854775808 d", "-9223372036854775808 d", "1ns", "999999ns", "\"infinite\"", "\"\"", "[1]", "on", "2562047 h"}) {
      final String text = v;
      run("SET-LDSD log-dead-letters-suspend-duration = " + text, () -> {
        ActorSystem.Settings s = set("pekko.log-dead-letters-suspend-duration = " + text);
        return "LogDeadLettersSuspendDuration=" + duration(s.LogDeadLettersSuspendDuration());
      });
    }
    for (String v : new String[] {"on", "off", "true", "false", "yes", "abc", "1", "\"on\""}) {
      final String text = v;
      run("SET-LDDS log-dead-letters-during-shutdown = " + text, () -> {
        ActorSystem.Settings s = set("pekko.log-dead-letters-during-shutdown = " + text);
        return "LogDeadLettersDuringShutdown=" + s.LogDeadLettersDuringShutdown();
      });
    }
  }

  static void registerDebugAndJvm() {
    String[] debug = {"receive", "autoreceive", "lifecycle", "fsm", "event-stream", "unhandled", "router-misconfiguration"};
    for (String k : debug) {
      final String key = k;
      run("SET-DEBUG " + key + " = on", () -> show(set("pekko.actor.debug." + key + " = on")));
    }
    run("SET-DEBUG all seven on", () -> {
      StringBuilder b = new StringBuilder();
      for (String k : debug) b.append("pekko.actor.debug.").append(k).append(" = true\n");
      return show(set(b.toString()));
    });
    run("SET-DEBUG receive of a wrong type", () -> show(set("pekko.actor.debug.receive = abc")));
    run("SET-DEBUG lifecycle quoted", () -> show(set("pekko.actor.debug.lifecycle = \"yes\"")));
    for (String k : new String[] {"daemonic = on", "jvm-exit-on-fatal-error = off", "jvm-shutdown-hooks = off", "fail-mixed-versions = off",
        "daemonic = abc", "jvm-exit-on-fatal-error = 3"}) {
      final String text = k;
      run("SET-JVM " + text, () -> show(set("pekko." + text)));
    }
    run("SET-JVM all flipped", () -> show(set("pekko.daemonic = on\npekko.jvm-exit-on-fatal-error = off\npekko.jvm-shutdown-hooks = off\npekko.fail-mixed-versions = off")));
    for (String h : new String[] {"\"/opt/pekko\"", "\"\"", "\" \"", "x", "[1]", "5"}) {
      final String text = h;
      run("SET-HOME home = " + text, () -> "Home=" + set("pekko.home = " + text).Home());
    }
    run("SET-HOME home = null", () -> "Home=" + set("pekko.home = null").Home());
    run("SET-SCHEDULER implementation = my.Sched", () -> "SchedulerClass=" + set("pekko.scheduler.implementation = my.Sched").SchedulerClass());
    run("SET-SCHEDULER implementation = [1]", () -> "SchedulerClass=" + set("pekko.scheduler.implementation = [1]").SchedulerClass());
  }

  static void registerTimeoutsAndLogging() {
    for (String t : new String[] {"1s", "2500ms", "0", "-1s", "abc", "1.5 minutes", "off", "1 d", "100"}) {
      final String text = t;
      run("SET-TO creation-timeout = " + text, () -> "CreationTimeout=" + set("pekko.actor.creation-timeout = " + text).CreationTimeout().duration().toMillis() + "ms");
    }
    for (String t : new String[] {"500ms", "1 minute", "-5ms", "x"}) {
      final String text = t;
      run("SET-TO unstarted-push-timeout = " + text, () -> "UnstartedPushTimeout=" + set("pekko.actor.unstarted-push-timeout = " + text).UnstartedPushTimeout().duration().toMillis() + "ms");
    }
    for (String t : new String[] {"7s", "1ms", "0", "never"}) {
      final String text = t;
      run("SET-TO logger-startup-timeout = " + text, () -> "LoggerStartTimeout=" + set("pekko.logger-startup-timeout = " + text).LoggerStartTimeout().duration().toMillis() + "ms");
    }
    run("SET-LOG loggers = []", () -> "Loggers=" + list(set("pekko.loggers = []").Loggers()));
    run("SET-LOG loggers = two names", () -> "Loggers=" + list(set("pekko.loggers = [\"a.B\", \"c.D$E\"]").Loggers()));
    run("SET-LOG loggers with a repeated name", () -> "Loggers=" + list(set("pekko.loggers = [\"a.B\", \"a.B\"]").Loggers()));
    run("SET-LOG loggers = a string", () -> "Loggers=" + list(set("pekko.loggers = \"a.B\"").Loggers()));
    run("SET-LOG loggers = numbers", () -> "Loggers=" + list(set("pekko.loggers = [1, 2]").Loggers()));
    run("SET-LOG loggers = list of objects", () -> "Loggers=" + list(set("pekko.loggers = [{a = 1}]").Loggers()));
    run("SET-LOG logging-filter = x.Filter", () -> "LoggingFilter=" + set("pekko.logging-filter = x.Filter").LoggingFilter());
    run("SET-LOG logging-filter = a number", () -> "LoggingFilter=" + set("pekko.logging-filter = 5").LoggingFilter());
    run("SET-LOG loggers-dispatcher = my-dispatcher", () -> "LoggersDispatcher=" + set("pekko.loggers-dispatcher = my-dispatcher").LoggersDispatcher());
    for (String l : new String[] {"DEBUG", "OFF", "debug", "\"WARNING\"", "5", "off"}) {
      final String text = l;
      run("SET-LOG loglevel = " + text, () -> "LogLevel=" + set("pekko.loglevel = " + text).LogLevel());
      run("SET-LOG stdout-loglevel = " + text, () -> "StdoutLogLevel=" + set("pekko.stdout-loglevel = " + text).StdoutLogLevel());
    }
    for (String l : new String[] {"on", "off", "true", "abc", "1"}) {
      final String text = l;
      run("SET-LOG log-config-on-start = " + text, () -> "LogConfigOnStart=" + set("pekko.log-config-on-start = " + text).LogConfigOnStart());
    }
    run("SET-GUARDIAN guardian-supervisor-strategy = StoppingSupervisorStrategy", () ->
      "SupervisorStrategyClass=" + set("pekko.actor.guardian-supervisor-strategy = \"org.apache.pekko.actor.StoppingSupervisorStrategy\"").SupervisorStrategyClass());
    run("SET-GUARDIAN guardian-supervisor-strategy = a list", () -> "SupervisorStrategyClass=" + set("pekko.actor.guardian-supervisor-strategy = [1]").SupervisorStrategyClass());
  }

  static void registerSerializationAndShutdown() {
    run("SET-SER allow-java-serialization = on", () -> "AllowJavaSerialization=" + set("pekko.actor.allow-java-serialization = on").AllowJavaSerialization());
    run("SET-SER serialize-messages = on", () -> show(set("pekko.actor.serialize-messages = on")));
    run("SET-SER serialize-creators = on", () -> show(set("pekko.actor.serialize-creators = on")));
    run("SET-SER serialize-messages and serialize-creators = on", () -> show(set("pekko.actor.serialize-messages = on\npekko.actor.serialize-creators = on")));
    run("SET-SER serialize-messages = off with prefix []", () -> {
      ActorSystem.Settings s = set("pekko.actor.serialize-messages = off\npekko.actor.no-serialization-verification-needed-class-prefix = []");
      return "SerializeAllMessages=" + s.SerializeAllMessages() + " prefix=" + CollectionConverters.asJava(s.NoSerializationVerificationNeededClassPrefix());
    });
    run("SET-SER prefix with a repeated entry", () -> {
      ActorSystem.Settings s = set("pekko.actor.no-serialization-verification-needed-class-prefix = [\"b.\", \"a.\", \"b.\"]");
      List<String> p = new ArrayList<>(CollectionConverters.asJava(s.NoSerializationVerificationNeededClassPrefix()));
      Collections.sort(p);
      return "prefix=" + p;
    });
    run("SET-SER prefix = a string", () -> "prefix=" + CollectionConverters.asJava(set("pekko.actor.no-serialization-verification-needed-class-prefix = \"x.\"").NoSerializationVerificationNeededClassPrefix()));
    run("SET-SER serialize-messages = abc", () -> show(set("pekko.actor.serialize-messages = abc")));
    for (String a : new String[] {"on", "off"}) {
      for (String b : new String[] {"on", "off"}) {
        final String ta = a, tb = b;
        run("SET-CS terminate-actor-system = " + ta + " run-by-actor-system-terminate = " + tb, () -> {
          ActorSystem.Settings s = set("pekko.coordinated-shutdown.terminate-actor-system = " + ta
              + "\npekko.coordinated-shutdown.run-by-actor-system-terminate = " + tb);
          return "CoordinatedShutdownTerminateActorSystem=" + s.CoordinatedShutdownTerminateActorSystem()
              + " CoordinatedShutdownRunByActorSystemTerminate=" + s.CoordinatedShutdownRunByActorSystemTerminate();
        });
      }
    }
    run("SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate", () ->
      show(set("pekko.coordinated-shutdown.terminate-actor-system = off")));
    run("SET-CS run-by-actor-system-terminate = off alone", () -> show(set("pekko.coordinated-shutdown.run-by-actor-system-terminate = off")));
    run("SET-CS terminate-actor-system = abc", () -> show(set("pekko.coordinated-shutdown.terminate-actor-system = abc")));
    run("SET-CS the unsupported combination is reported before the version check", () ->
      show(set("pekko.coordinated-shutdown.terminate-actor-system = off\npekko.version = \"1.0.0\"")));
    run("SET-CS the unsupported combination is reported before a bad virtual-nodes-factor", () ->
      show(set("pekko.coordinated-shutdown.terminate-actor-system = off\npekko.actor.deployment.default.virtual-nodes-factor = abc")));
    for (String v : new String[] {"3", "0", "-2", "abc", "1.5", "\"7\""}) {
      final String text = v;
      run("SET-VNF virtual-nodes-factor = " + text, () ->
        "DefaultVirtualNodesFactor=" + set("pekko.actor.deployment.default.virtual-nodes-factor = " + text).DefaultVirtualNodesFactor());
    }
    for (String v : new String[] {"\"2.0.0\"", "\"1.7.1\"", "\"\"", "\"2.0.0-M1\"", "2", "[1]"}) {
      final String text = v;
      run("SET-VERSION pekko.version = " + text, () -> "ConfigVersion=" + set("pekko.version = " + text).ConfigVersion());
    }
  }

  static void registerInvalid() {
    // A configuration that is not merged with the reference: keys that the reference defines are missing.
    // Not recorded (Q-CFG-15, Q-CFG-17): a missing key is described with the origin of its parent object (a line or a merge of
    // line ranges), and "expecting: object with keys [...]" lists the keys in the iteration order of a JVM HashMap.
    run("SET-INVALID a key of the reference is an object in the application", () -> show(set("pekko.loglevel { x = 1 }")));
    run("SET-INVALID creation-timeout is a list", () -> show(set("pekko.actor.creation-timeout = [1]")));
    run("SET-INVALID creation-timeout is an object", () -> show(set("pekko.actor.creation-timeout { x = 1 }")));
    run("SET-INVALID several problems are reported together", () -> show(set("pekko.loglevel { x = 1 }\npekko.loggers = 5\npekko.daemonic = [1]")));
    run("SET-INVALID internal-dispatcher as a dispatcher name is allowed", () -> show(set("pekko.actor.internal-dispatcher = \"pekko.actor.default-dispatcher\"")));
    run("SET-INVALID internal-dispatcher removed from the application", () -> show(settings(base.withoutPath("pekko.actor.internal-dispatcher"))));
    run("SET-INVALID an unresolved configuration", () -> show(settings(parse("pekko.loglevel = ${x}").withFallback(base))));
    run("SET-INVALID the application adds unknown keys", () -> show(set("pekko.unknown-key = 1\nmy-app.x = 2")));
  }

  static final String SLF4J_LOGGER = "org.apache.pekko.event.slf4j.Slf4jLogger";
  static final String SLF4J_FILTER = "org.apache.pekko.event.slf4j.Slf4jLoggingFilter";
  static final String DEFAULT_LOGGER = "org.apache.pekko.event.Logging$DefaultLogger";
  static final String DEFAULT_FILTER = "org.apache.pekko.event.DefaultLoggingFilter";

  /** A class loader that does not see the slf4j module of Pekko, to model "pekko-slf4j is not on the classpath". */
  static final class HidingClassLoader extends ClassLoader {
    HidingClassLoader(ClassLoader parent) { super(parent); }
    @Override
    protected Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
      if (name.startsWith("org.apache.pekko.event.slf4j.")) throw new ClassNotFoundException(name);
      return super.loadClass(name, resolve);
    }
  }

  static String amend(String app, boolean slf4jAvailable) {
    ClassLoader loader = slf4jAvailable ? ConfigSettingsOracle.class.getClassLoader() : new HidingClassLoader(ConfigSettingsOracle.class.getClassLoader());
    Config in = parse(app).withFallback(actorOnly);
    Config out = ActorSystem.Settings$.MODULE$.amendSlf4jConfig(in, new ReflectiveDynamicAccess(loader));
    return "loggers=" + out.getStringList("pekko.loggers") + " logging-filter=" + out.getString("pekko.logging-filter");
  }

  static void registerAmend() {
    String[][] configs = {
        {"the reference configuration", ""},
        {"use-slf4j = on", "pekko.use-slf4j = on"},
        {"use-slf4j = off", "pekko.use-slf4j = off"},
        {"use-slf4j = true", "pekko.use-slf4j = true"},
        {"use-slf4j = yes", "pekko.use-slf4j = yes"},
        {"use-slf4j = quoted on", "pekko.use-slf4j = \"on\""},
        {"use-slf4j = maybe", "pekko.use-slf4j = maybe"},
        {"use-slf4j = a list", "pekko.use-slf4j = [1]"},
        {"use-slf4j = null", "pekko.use-slf4j = null"},
        {"slf4j logger already configured with the default filter", "pekko.loggers = [\"" + SLF4J_LOGGER + "\"]"},
        {"slf4j logger already configured with the slf4j filter", "pekko.loggers = [\"" + SLF4J_LOGGER + "\"]\npekko.logging-filter = \"" + SLF4J_FILTER + "\""},
        {"slf4j logger already configured with a custom filter", "pekko.loggers = [\"" + SLF4J_LOGGER + "\"]\npekko.logging-filter = my.Filter"},
        {"slf4j logger and the default logger", "pekko.loggers = [\"" + SLF4J_LOGGER + "\", \"" + DEFAULT_LOGGER + "\"]"},
        {"use-slf4j = on with the default logger and a custom one", "pekko.use-slf4j = on\npekko.loggers = [\"" + DEFAULT_LOGGER + "\", \"my.Logger\"]"},
        {"use-slf4j = on with a custom filter", "pekko.use-slf4j = on\npekko.logging-filter = my.Filter"},
        {"use-slf4j = on with the slf4j filter", "pekko.use-slf4j = on\npekko.logging-filter = \"" + SLF4J_FILTER + "\""},
        {"use-slf4j = on with no loggers", "pekko.use-slf4j = on\npekko.loggers = []"},
        {"use-slf4j = on with only a custom logger", "pekko.use-slf4j = on\npekko.loggers = [\"my.Logger\"]"},
        {"use-slf4j = on with the default logger twice", "pekko.use-slf4j = on\npekko.loggers = [\"" + DEFAULT_LOGGER + "\", \"" + DEFAULT_LOGGER + "\"]"},
        {"use-slf4j = off with the slf4j logger", "pekko.use-slf4j = off\npekko.loggers = [\"" + SLF4J_LOGGER + "\"]"},
        {"loggers is a string", "pekko.use-slf4j = on\npekko.loggers = \"x\""},
        {"logging-filter is a list", "pekko.use-slf4j = on\npekko.logging-filter = [1]"},
    };
    for (String[] c : configs) {
      for (boolean available : new boolean[] {true, false}) {
        final String app = c[1];
        final boolean av = available;
        run("AMEND " + c[0] + (available ? ", pekko-slf4j on the classpath" : ", pekko-slf4j not on the classpath"), () -> amend(app, av));
      }
    }
    run("AMEND the amended configuration is amended again without change", () -> {
      Config in = parse("pekko.use-slf4j = on").withFallback(actorOnly);
      ReflectiveDynamicAccess da = new ReflectiveDynamicAccess(ConfigSettingsOracle.class.getClassLoader());
      Config once = ActorSystem.Settings$.MODULE$.amendSlf4jConfig(in, da);
      Config twice = ActorSystem.Settings$.MODULE$.amendSlf4jConfig(once, da);
      return "loggers=" + twice.getStringList("pekko.loggers") + " logging-filter=" + twice.getString("pekko.logging-filter");
    });
  }
}
