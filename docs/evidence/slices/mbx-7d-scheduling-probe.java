// Standalone probe (not part of oracle/): does Pekko 2.0.0 start a mailbox turn for a message that NonBlockingBoundedMailbox refuses?
// Build and run (JDK 17, jars of oracle/target/lib):
//   CP="oracle/target/lib/*"; javac -cp "$CP" -d "$OUT" mbx-7d-scheduling-probe.java && java -cp "$OUT:$CP" Mbx7dSchedulingProbe
import com.typesafe.config.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import org.apache.pekko.actor.*;
import org.apache.pekko.dispatch.*;

class Mbx7dSchedulingProbe {
  static final AtomicInteger EXECUTES = new AtomicInteger();
  static final AtomicInteger REJECT_NEXT = new AtomicInteger();
  static final List<String> RECEIVED = new CopyOnWriteArrayList<>();
  static final List<String> DEAD = new CopyOnWriteArrayList<>();

  /** One-thread executor of the dispatcher `counting-disp` that counts (and can reject) every submitted mailbox turn. */
  public static final class CountingConfigurator extends ExecutorServiceConfigurator {
    public CountingConfigurator(Config config, DispatcherPrerequisites prerequisites) { super(config, prerequisites); }
    @Override public ExecutorServiceFactory createExecutorServiceFactory(String id, ThreadFactory threadFactory) {
      return () -> new ThreadPoolExecutor(1, 1, 0L, TimeUnit.SECONDS, new LinkedBlockingQueue<>(), threadFactory) {
        @Override public void execute(Runnable task) {
          EXECUTES.incrementAndGet();
          if (REJECT_NEXT.getAndUpdate(n -> n > 0 ? n - 1 : 0) > 0) throw new RejectedExecutionException("injected");
          super.execute(task);
        }
      };
    }
  }

  public static final class Worker extends AbstractActor {
    @Override public Receive createReceive() { return receiveBuilder().match(String.class, RECEIVED::add).build(); }
  }

  public static final class DeadListener extends AbstractActor {
    @Override public Receive createReceive() { return receiveBuilder().match(DeadLetter.class, d -> DEAD.add(String.valueOf(d.message()))).build(); }
  }

  public static void main(String[] args) throws Exception {
    for (int capacity : new int[] {0, 1}) {
      EXECUTES.set(0); RECEIVED.clear(); DEAD.clear(); REJECT_NEXT.set(0);
      ActorSystem sys = ActorSystem.create("probe", ConfigFactory.parseString(
          "counting-disp { type = Dispatcher\n executor = \"Mbx7dSchedulingProbe$CountingConfigurator\"\n throughput = 5 }\n"
        + "cap-mailbox { mailbox-type = \"org.apache.pekko.dispatch.NonBlockingBoundedMailbox\"\n mailbox-capacity = " + capacity + " }\n")
          .withFallback(ConfigFactory.load()));
      ActorRef listener = sys.actorOf(Props.create(DeadListener.class), "listener");
      sys.eventStream().subscribe(listener, DeadLetter.class);
      ActorRef w = sys.actorOf(Props.create(Worker.class).withDispatcher("counting-disp").withMailbox("cap-mailbox"), "w");
      Thread.sleep(500); // the start-up turn ends
      int before = EXECUTES.get();
      for (String m : new String[] {"a", "b", "c"}) w.tell(m, ActorRef.noSender());
      Thread.sleep(500);
      System.out.println("capacity=" + capacity + " turnsSubmittedForThreeTells=" + (EXECUTES.get() - before)
          + " received=" + RECEIVED + " dead=" + DEAD);
      if (capacity == 0) {
        REJECT_NEXT.set(2);
        String outcome;
        try { w.tell("rejected-executor", ActorRef.noSender()); outcome = "tell returned"; }
        catch (RejectedExecutionException e) { outcome = "tell threw RejectedExecutionException"; }
        Thread.sleep(300);
        System.out.println("capacity=0 executorRejectsTwice -> " + outcome + " rejectionsLeft=" + REJECT_NEXT.get() + " deadLetters=" + DEAD.size());
      }
      sys.terminate();
      sys.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
    }
  }
}
