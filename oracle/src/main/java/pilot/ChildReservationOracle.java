package pilot;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.function.BooleanSupplier;

import com.typesafe.config.ConfigFactory;
import org.apache.pekko.actor.AbstractActor;
import org.apache.pekko.actor.ActorCell;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorRefWithCell;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Props;

/**
 * What `ActorCell.reserveChild` throws while a cell terminates and after (AC-REF-009.2, `ChildrenContainer.reserve`): one line per
 * case, oracle-only. It calls the internal method: `actorOf` checks `isTerminating` before it reserves, so only a race reaches these.
 */
final class ChildReservationOracle {
  private ChildReservationOracle() {}

  /** Blocks in `receive`, so that its stopping parent keeps waiting for it. */
  public static final class Stuck extends AbstractActor {
    private final CountDownLatch blocked, release;
    public Stuck(CountDownLatch blocked, CountDownLatch release) { this.blocked = blocked; this.release = release; }
    @Override public Receive createReceive() {
      return receiveBuilder().matchAny(m -> { blocked.countDown(); release.await(30, TimeUnit.SECONDS); }).build();
    }
  }

  public static final class Parent extends AbstractActor {
    private final CountDownLatch blocked, release;
    public Parent(CountDownLatch blocked, CountDownLatch release) { this.blocked = blocked; this.release = release; }
    @Override public void preStart() {
      getContext().actorOf(Props.create(Stuck.class, () -> new Stuck(blocked, release))).tell("block", getSelf());
    }
    @Override public Receive createReceive() { return receiveBuilder().build(); }
  }

  static void printAll() {
    ActorSystem system = ActorSystem.create("reservation",
        ConfigFactory.parseString("pekko.loglevel = OFF\npekko.stdout-loglevel = OFF\n"));
    CountDownLatch blocked = new CountDownLatch(1), release = new CountDownLatch(1);
    ActorRef parent = system.actorOf(Props.create(Parent.class, () -> new Parent(blocked, release)), "parent");
    await(() -> blocked.getCount() == 0);
    ActorCell cell = (ActorCell) ((ActorRefWithCell) parent).underlying();
    system.stop(parent);
    await(() -> cell.childrenRefs().isTerminating());
    print("RSV-01 reserveChild(kid) while the cell waits for a child to die before it terminates", cell);
    release.countDown();
    await(cell::isTerminated);
    print("RSV-02 reserveChild(kid) once the cell has terminated", cell);
    system.terminate();
  }

  private static void print(String id, ActorCell cell) {
    String observed;
    try {
      cell.reserveChild("kid");
      observed = "reserved";
    } catch (IllegalStateException e) {
      observed = e.getClass().getName() + ": " + e.getMessage();
    }
    System.out.println(id + " -> " + observed);
  }

  private static void await(BooleanSupplier condition) {
    long end = System.nanoTime() + TimeUnit.SECONDS.toNanos(10);
    while (!condition.getAsBoolean()) {
      if (System.nanoTime() > end) throw new IllegalStateException("timed out");
      Thread.onSpinWait();
    }
  }
}
