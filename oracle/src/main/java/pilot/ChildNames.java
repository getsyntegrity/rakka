package pilot;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Props;
import org.apache.pekko.dispatch.MailboxType;
import org.apache.pekko.dispatch.MessageQueue;
import org.apache.pekko.dispatch.UnboundedMailbox;

/** `spawn-held`, `release-held` and `spawn-concurrently` of {@link Scenario}. One scenario per process, hence the statics. */
final class ChildNames {
  private ChildNames() {}

  static final String HOCON = "held-mailbox.mailbox-type = \"pilot.ChildNames$HeldMailbox\"\n";

  private static final CountDownLatch ENTERED = new CountDownLatch(1), RELEASED = new CountDownLatch(1);
  private static FutureTask<ActorRef> held;

  /** An unbounded mailbox type that `actorOf` makes after reserving the name; making it waits for `release-held`. */
  public static final class HeldMailbox implements MailboxType {
    public HeldMailbox(ActorSystem.Settings settings, com.typesafe.config.Config config) throws InterruptedException {
      ENTERED.countDown();
      RELEASED.await(30, TimeUnit.SECONDS);
    }

    @Override public MessageQueue create(scala.Option<ActorRef> owner, scala.Option<ActorSystem> system) {
      return new UnboundedMailbox().create(owner, system);
    }
  }

  /** `system.actorOf(props, name)` on another thread; returns once it is held, with the name reserved. */
  static boolean spawnHeld(ActorSystem system, String name) throws InterruptedException {
    held = new FutureTask<>(() -> system.actorOf(Props.create(Scenario.Noop.class).withMailbox("held-mailbox"), name));
    new Thread(held).start();
    return ENTERED.await(30, TimeUnit.SECONDS);
  }

  static ActorRef releaseHeld() throws Exception {
    RELEASED.countDown();
    return held.get(30, TimeUnit.SECONDS);
  }

  /** The result fields of `spawn-concurrently`: `threads` simultaneous `system.actorOf(props, name)`, `rounds` times. */
  static String concurrently(ActorSystem system, Map<String, Object> step) throws Exception {
    int threads = ((Long) step.get("threads")).intValue(), rounds = ((Long) step.get("rounds")).intValue();
    int winners = 0;
    boolean oneWinnerPerName = true;
    Map<String, Integer> refused = new TreeMap<>(); // "<class>: <message>", the name written <n>
    AtomicInteger running = new AtomicInteger(), most = new AtomicInteger();
    ExecutorService pool = Executors.newFixedThreadPool(threads);
    try {
      for (int round = 0; round < rounds; round++) {
        String name = step.containsKey("name") ? (String) step.get("name") : step.get("prefix") + "" + round;
        CyclicBarrier together = new CyclicBarrier(threads);
        List<Future<ActorRef>> attempts = new ArrayList<>();
        for (int i = 0; i < threads; i++) {
          attempts.add(pool.submit(() -> {
            together.await();
            most.accumulateAndGet(running.incrementAndGet(), Math::max);
            try {
              return system.actorOf(Props.create(Scenario.Noop.class), name);
            } finally {
              running.decrementAndGet();
            }
          }));
        }
        int won = 0;
        for (Future<ActorRef> attempt : attempts) {
          try {
            attempt.get(30, TimeUnit.SECONDS);
            won++;
          } catch (java.util.concurrent.ExecutionException e) {
            Throwable cause = e.getCause();
            refused.merge(cause.getClass().getName() + ": " + cause.getMessage().replace(name, "<n>"), 1, Integer::sum);
          }
        }
        winners += won;
        oneWinnerPerName &= won == 1;
      }
    } finally {
      pool.shutdown();
    }
    List<String> entries = new ArrayList<>();
    refused.forEach((refusal, count) -> entries.add(Cli.Json.str(refusal) + ":" + count));
    String fields = ",\"ok\":true,\"rounds\":" + rounds + ",\"winners\":" + winners
        + ",\"oneWinnerPerName\":" + oneWinnerPerName + ",\"refused\":{" + String.join(",", entries) + "}";
    // whether the calls overlapped is only certain over many rounds, so only those report it
    return Boolean.TRUE.equals(step.get("overlap")) ? fields + ",\"overlapped\":" + (most.get() > 1) : fields;
  }
}
