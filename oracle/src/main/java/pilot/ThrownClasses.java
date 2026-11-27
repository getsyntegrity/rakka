package pilot;

import org.apache.pekko.actor.ActorInitializationException;

/** The throwables of the `throw-class` behaviour: known, unknown and derived classes (Q-SUP-1). */
final class ThrownClasses {
  private ThrownClasses() {}

  public static final class UserFailure extends RuntimeException { UserFailure(String m) { super(m); } }
  public static final class CheckedFailure extends Exception { CheckedFailure(String m) { super(m); } }
  public static class ParentFailure extends IllegalArgumentException { ParentFailure(String m) { super(m); } }
  public static final class ChildFailure extends ParentFailure { ChildFailure(String m) { super(m); } }
  public static final class UserError extends Error { UserError(String m) { super(m); } }
  public static final class BareFailure extends Throwable { BareFailure(String m) { super(m); } }
  public static final class InitFailure extends ActorInitializationException {
    InitFailure(String m, Throwable cause) { super(null, m, cause); }
  }

  /** Throws the throwable of `kind` with `message`, checked or not. */
  static void raise(String kind, String message) {
    ThrownClasses.<RuntimeException>rethrow(of(kind, message));
  }

  @SuppressWarnings("unchecked")
  private static <T extends Throwable> void rethrow(Throwable t) throws T { throw (T) t; }

  private static Throwable of(String kind, String message) {
    switch (kind) {
      case "user": return new UserFailure(message);
      case "checked": return new CheckedFailure(message);
      case "child": return new ChildFailure(message);
      case "number": return new NumberFormatException(message);
      case "init": return new InitFailure(message, null);
      case "init-caused": return new InitFailure(message, new IllegalStateException("cause"));
      case "error": return new Error(message);
      case "assertion": return new AssertionError((Object) message);
      case "user-error": return new UserError(message);
      case "bare": return new BareFailure(message);
      default: throw new IllegalArgumentException("unknown throwable kind " + kind);
    }
  }
}
