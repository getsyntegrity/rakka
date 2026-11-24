Verdict path: full replay (2974 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-69319-gt6pk4jw/target-mailbox-dispatcher-requirement-checked-only-on-bindings/release/examples/classic-parity`

Passed: 2972/2974

Fallback rate: not reported by this build

## Failing cases

### mb-0029

Arguments: `mailbox-case`

stdout:

```diff
- MBX-CMP-02 the queue of the selected mailbox must fulfill the requirement of the dispatcher and that of the actor, the dispatcher's reported first; an actor binding that does not fit gives way to the dispatcher's binding -> dispatcherUnmetBySelected: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/dispatcherUnmetBySelected] with dispatcher [pekko.actor.default-dispatcher] and mailbox [cap4-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.BoundedNodeMessageQueue] does not fulfill requirement for dispatcher [pekko.actor.default-dispatcher]. Must be a subclass of [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics].; dispatcherMetBySelected: created; boundedBindingFallsBack: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/boundedBindingFallsBack] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; bothMetByAudit: created; actorUnmetOnly: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/actorUnmetOnly] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; bothUnmet: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/bothUnmet] with dispatcher [pekko.actor.default-dispatcher] and mailbox [cap4-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.BoundedNodeMessageQueue] does not fulfill requirement for dispatcher [pekko.actor.default-dispatcher]. Must be a subclass of [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics].; plainOnDefault: created;
+ MBX-CMP-02 the queue of the selected mailbox must fulfill the requirement of the dispatcher and that of the actor, the dispatcher's reported first; an actor binding that does not fit gives way to the dispatcher's binding -> dispatcherUnmetBySelected: created; dispatcherMetBySelected: created; boundedBindingFallsBack: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/boundedBindingFallsBack] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; bothMetByAudit: created; actorUnmetOnly: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/actorUnmetOnly] with dispatcher [pekko.actor.default-dispatcher] and mailbox [unbounded] | cause java.lang.IllegalArgumentException: produced message queue type [class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class pilot.MailboxOracle$BoundedWorker]. Must be a subclass of [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics].; bothUnmet: created; plainOnDefault: created;
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
