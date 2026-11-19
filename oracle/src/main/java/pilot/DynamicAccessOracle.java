package pilot;

import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import java.util.*;
import java.util.concurrent.Callable;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.ReflectiveDynamicAccess;
import scala.collection.immutable.Nil$;
import scala.reflect.ClassTag$;
import scala.util.Failure;
import scala.util.Try;

/**
 * Oracle for unit P09, row AC-CFG-016 (DynamicAccess, the by-name factory lookup): what Apache Pekko 2.0.0 observably does when
 * a class name is given to {@code ReflectiveDynamicAccess} and when {@code pekko.actor.provider} names a class. Only what a
 * caller can observe is recorded: the exception class and its message for a name that is not on the classpath, whether a
 * name is on the classpath, and whether a default-constructor class is created. Reflection itself (constructor signatures,
 * private constructors, Scala module fields) has no Rust equivalent and is not emulated (Q-X-2).
 *
 * Output contract (one deterministic line per case): {@code <CASE-ID> -> ok <result>} or
 * {@code <CASE-ID> -> ERR <ExceptionSimpleName> [<message>]}. Case-id prefixes: CNF (createInstanceFor of a missing class),
 * ON (classIsOnClasspath), INST (createInstanceFor with a default constructor), PRV (the class named by
 * {@code pekko.actor.provider} while an ActorSystem is created: ActorSystem.scala:996-1011 {@code createInstanceFor(..).get}).
 */
final class DynamicAccessOracle {
  private DynamicAccessOracle() {}

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

  private static final ReflectiveDynamicAccess ACCESS =
      new ReflectiveDynamicAccess(DynamicAccessOracle.class.getClassLoader());

  /** {@code createInstanceFor[Object](fqcn, Nil)}; the failure is thrown, the instance's class name is returned. */
  private static String created(String fqcn) throws Exception {
    @SuppressWarnings("unchecked")
    scala.collection.immutable.Seq<scala.Tuple2<Class<?>, Object>> noArguments =
        (scala.collection.immutable.Seq<scala.Tuple2<Class<?>, Object>>) (Object) Nil$.MODULE$;
    Try<Object> attempt = ACCESS.createInstanceFor(fqcn, noArguments, ClassTag$.MODULE$.Object());
    if (attempt instanceof Failure<Object> failure) {
      if (failure.exception() instanceof Exception e) throw e;
      throw new IllegalStateException(failure.exception());
    }
    return "created " + attempt.get().getClass().getName();
  }

  private static String onClasspath(String fqcn) {
    return String.valueOf(ACCESS.classIsOnClasspath(fqcn));
  }

  /** Creates an ActorSystem whose {@code pekko.actor.provider} is {@code provider}; reports only whether it was created. */
  private static String providerLoads(String provider) throws Exception {
    Config config = ConfigFactory.parseString(
            "pekko.actor.provider = \"" + provider + "\"\npekko.loglevel = OFF\npekko.stdout-loglevel = OFF\n")
        .withFallback(ConfigFactory.load());
    ActorSystem system = ActorSystem.create("provider-probe", config);
    try {
      return "created";
    } finally {
      scala.concurrent.Await.result(system.terminate(), scala.concurrent.duration.Duration.apply(30, "seconds"));
    }
  }

  private static final String REMOTE = "org.apache.pekko.remote.RemoteActorRefProvider";
  private static final String CLUSTER = "org.apache.pekko.cluster.ClusterActorRefProvider";
  private static final String LOCAL = "org.apache.pekko.actor.LocalActorRefProvider";
  private static final String UNBOUNDED_MAILBOX = "org.apache.pekko.dispatch.UnboundedMailbox";

  private static boolean registered = false;

  private static void register() {
    if (registered) return;
    registered = true;
    // DynamicAccessSpec.scala:50-57 and :77-86 (a class that does not exist is ClassNotFoundException carrying its name).
    run("CNF-01 createInstanceFor foo.NonExistingClass", () -> created("foo.NonExistingClass"));
    run("CNF-02 createInstanceFor org.apache.pekko.actor.foo.NonExistingClass", () -> created("org.apache.pekko.actor.foo.NonExistingClass"));
    run("CNF-03 createInstanceFor empty name", () -> created(""));
    run("CNF-04 createInstanceFor name differing only in case", () -> created("org.apache.pekko.dispatch.unboundedmailbox"));
    run("CNF-05 createInstanceFor the provider " + REMOTE, () -> created(REMOTE));
    run("CNF-06 createInstanceFor the provider " + CLUSTER, () -> created(CLUSTER));
    // DynamicAccessSpec.scala:72-75.
    run("ON-01 classIsOnClasspath i.just.made.it.up.to.hurt.Myself", () -> onClasspath("i.just.made.it.up.to.hurt.Myself"));
    run("ON-02 classIsOnClasspath org.apache.pekko.actor.Actor", () -> onClasspath("org.apache.pekko.actor.Actor"));
    run("ON-03 classIsOnClasspath " + REMOTE, () -> onClasspath(REMOTE));
    run("ON-04 classIsOnClasspath " + CLUSTER, () -> onClasspath(CLUSTER));
    run("ON-05 classIsOnClasspath " + LOCAL, () -> onClasspath(LOCAL));
    run("ON-06 classIsOnClasspath empty name", () -> onClasspath(""));
    // DynamicAccessSpec.scala:40-48 (a class with a default constructor is created and is not null).
    run("INST-01 createInstanceFor " + UNBOUNDED_MAILBOX, () -> created(UNBOUNDED_MAILBOX));
    // ActorSystem.scala:996-1011: the provider is created by createInstanceFor(ProviderClass, ..).get, so a missing class is the
    // ClassNotFoundException itself (not a ConfigurationException).
    run("PRV-01 provider foo.NonExisting", () -> providerLoads("foo.NonExisting"));
    run("PRV-02 provider " + REMOTE, () -> providerLoads(REMOTE));
    run("PRV-03 provider " + CLUSTER, () -> providerLoads(CLUSTER));
    run("PRV-04 provider " + LOCAL, () -> providerLoads(LOCAL));
    // Class.forName matches the name exactly: no case folding and no trimming.
    run("CNF-07 createInstanceFor name with surrounding spaces", () -> created(" " + UNBOUNDED_MAILBOX + " "));
    run("ON-07 classIsOnClasspath name differing only in case", () -> onClasspath("org.apache.pekko.actor.actor"));
    run("ON-08 classIsOnClasspath name with a trailing space", () -> onClasspath("org.apache.pekko.actor.Actor "));
  }
}
