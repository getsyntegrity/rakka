import com.typesafe.config.Config;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.dispatch.BoundedMailbox;
import org.apache.pekko.dispatch.MailboxType;
import org.apache.pekko.dispatch.MessageQueue;
import org.apache.pekko.dispatch.ProducesMessageQueue;
import org.apache.pekko.dispatch.UnboundedMailbox;
import scala.Option;
import scala.concurrent.duration.FiniteDuration;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

/** The scenario of the Rust test `a_failed_root_guardian_leaves_when_terminated_pending`: a default mailbox type that PROMISES an unbounded queue (so that the selection-time check
 *  passes) but CREATES a bounded one for the first actor that asks (the root guardian), so that the check of `Dispatch.init` on the queue it really created fails there. */
public class FirstBoundedMailbox implements MailboxType, ProducesMessageQueue<UnboundedMailbox.MessageQueue> {
    public static final AtomicInteger CREATED = new AtomicInteger();
    public static volatile ActorSystem CAPTURED;       // the system the root guardian belongs to: the only handle left when ActorSystem.create throws

    public FirstBoundedMailbox(ActorSystem.Settings settings, Config config) { }

    @Override
    public MessageQueue create(Option<ActorRef> owner, Option<ActorSystem> system) {
        int index = CREATED.getAndIncrement();
        if (index == 0 && system.isDefined()) CAPTURED = system.get();
        System.out.println("  [mailbox type] create() call #" + index + " for " + owner.map(Object::toString).getOrElse(() -> "<no owner>") + (index == 0 ? " -> BOUNDED queue" : " -> unbounded queue"));
        return index == 0 ? new BoundedMailbox(1, FiniteDuration.apply(0, TimeUnit.SECONDS)).create(owner, system) : new UnboundedMailbox().create(owner, system);
    }
}
