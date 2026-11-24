Verdict path: fast-kill: case mb-0020 confirmed by a single-case replay (`rust-anything app replay --case`); the unmutated build matches Pekko on it; no full replay ran

# App replay

Build: `$TMPDIR/pekko-mutants-58352-2v0v0ota/target-mailbox-id-error-message-names-the-default-id/release/examples/classic-parity`

Passed: 0/1

Fallback rate: not reported by this build

## Failing cases

### mb-0020

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [typo] | cause org.apache.pekko.ConfigurationException: Mailbox Type [typo] not configured | instancesStartedAfterRetryWithSameName=1
+ MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause org.apache.pekko.ConfigurationException: Mailbox Type [typo] not configured | instancesStartedAfterRetryWithSameName=1
```
