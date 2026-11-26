Verdict path: full replay (2973 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-48105-744txmb3/target-mailbox-actor-class-is-named-without-its-kind/release/examples/classic-parity`

Passed: 2971/2973

Fallback rate: not reported by this build

## Failing cases

### mb-0028

Arguments: `mailbox-case`

stdout:

```diff
- MBX-CMP-01 an actor whose requirement the queue promised by the selected mailbox does not fulfill fails actorOf at the call; one it fulfills, or an actor that requires nothing, is created -> unboundedOnPlain: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/unboundedOnPlain] with dispatcher [pekko.actor.default-dispatcher] and mailbox [plain-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [interface org.apache.pekko.dispatch.MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$UnboundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics].; auditOnUnbounded: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/auditOnUnbounded] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$AuditTrailWorker]. Must be a subclass of [interface pilot.MailboxOracle$AuditTrail].; boundedOnUnbounded: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/boundedOnUnbounded] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; unboundedOnAudit: created; auditOnAudit: created; anyOnPlain: created; nothingOnPlain: created;
+ MBX-CMP-01 an actor whose requirement the queue promised by the selected mailbox does not fulfill fails actorOf at the call; one it fulfills, or an actor that requires nothing, is created -> unboundedOnPlain: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/unboundedOnPlain] with dispatcher [pekko.actor.default-dispatcher] and mailbox [plain-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [interface org.apache.pekko.dispatch.MessageQueue] does not fulfill requirement for actor class [pilot.MailboxOracle$UnboundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics].; auditOnUnbounded: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/auditOnUnbounded] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [pilot.MailboxOracle$AuditTrailWorker]. Must be a subclass of [interface pilot.MailboxOracle$AuditTrail].; boundedOnUnbounded: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/boundedOnUnbounded] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; unboundedOnAudit: created; auditOnAudit: created; anyOnPlain: created; nothingOnPlain: created;
```

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```
