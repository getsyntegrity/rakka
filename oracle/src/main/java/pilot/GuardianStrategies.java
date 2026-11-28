package pilot;

import org.apache.pekko.actor.AllForOneStrategy;
import org.apache.pekko.actor.OneForOneStrategy;
import org.apache.pekko.actor.SupervisorStrategy;
import org.apache.pekko.actor.SupervisorStrategyConfigurator;
import org.apache.pekko.japi.pf.DeciderBuilder;

/**
 * Configurators a scenario can name in {@code "guardian-strategy"} ({@code pekko.actor.guardian-supervisor-strategy}). The Rust harness
 * registers equivalents under the same names.
 */
public final class GuardianStrategies {
  private GuardianStrategies() {}

  /** The exception message names the directive; any other message gets the default decider. */
  private static scala.PartialFunction<Throwable, SupervisorStrategy.Directive> byMessage() {
    return DeciderBuilder.match(Throwable.class, e -> {
      String m = String.valueOf(e.getMessage());
      if (m.startsWith("resume")) return (SupervisorStrategy.Directive) SupervisorStrategy.resume();
      if (m.startsWith("restart")) return (SupervisorStrategy.Directive) SupervisorStrategy.restart();
      if (m.startsWith("stop")) return (SupervisorStrategy.Directive) SupervisorStrategy.stop();
      if (m.startsWith("escalate")) return (SupervisorStrategy.Directive) SupervisorStrategy.escalate();
      return SupervisorStrategy.defaultDecider().isDefinedAt(e)
          ? SupervisorStrategy.defaultDecider().apply(e) : (SupervisorStrategy.Directive) SupervisorStrategy.escalate();
    }).build();
  }

  /** {@code ActorSystemSpec$Strategy}: everything escalates. */
  public static final class Escalate implements SupervisorStrategyConfigurator {
    @Override public SupervisorStrategy create() {
      return new OneForOneStrategy(-1, scala.concurrent.duration.Duration.Inf(),
          DeciderBuilder.match(Throwable.class, e -> (SupervisorStrategy.Directive) SupervisorStrategy.escalate()).build());
    }
  }

  public static final class ByMessage implements SupervisorStrategyConfigurator {
    @Override public SupervisorStrategy create() {
      return new OneForOneStrategy(-1, scala.concurrent.duration.Duration.Inf(), byMessage());
    }
  }

  /** {@link ByMessage} with one restart per child. */
  public static final class OneRetry implements SupervisorStrategyConfigurator {
    @Override public SupervisorStrategy create() {
      return new OneForOneStrategy(1, scala.concurrent.duration.Duration.Inf(), byMessage());
    }
  }

  public static final class AllForOne implements SupervisorStrategyConfigurator {
    @Override public SupervisorStrategy create() {
      return new AllForOneStrategy(-1, scala.concurrent.duration.Duration.Inf(), byMessage());
    }
  }

  /** {@link ByMessage} with {@code loggingEnabled = false}. */
  public static final class Quiet implements SupervisorStrategyConfigurator {
    @Override public SupervisorStrategy create() {
      return new OneForOneStrategy(-1, scala.concurrent.duration.Duration.Inf(), false, byMessage());
    }
  }
}
