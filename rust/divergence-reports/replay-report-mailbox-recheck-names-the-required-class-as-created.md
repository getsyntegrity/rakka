Verdict path: full replay (2975 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-94773-6g4t_tz0/target-mailbox-recheck-names-the-required-class-as-created/release/examples/classic-parity`

Passed: 2973/2975

Fallback rate: not reported by this build

## Failing cases

### mb-0030

Arguments: `mailbox-case`

stdout:

```diff
- MBX-CMP-03 a type that promises its queue is checked against the queue it creates when the actor is made: actorOf returns, the actor fails to initialise and stops, and its name is free -> created | failure:org.apache.pekko.actor.ActorInitializationException: pekko://mbx/user/parent/kid: Actor [Actor[pekko://mbx/user/parent/kid]] requires mailbox type [interface pilot.MailboxOracle$AuditTrail] got [pilot.MailboxOracle$LifoQueue] | terminated | again | started=1
+ MBX-CMP-03 a type that promises its queue is checked against the queue it creates when the actor is made: actorOf returns, the actor fails to initialise and stops, and its name is free -> created | failure:org.apache.pekko.actor.ActorInitializationException: pekko://mbx/user/parent/kid: Actor [Actor[pekko://mbx/user/parent/kid]] requires mailbox type [interface pilot.MailboxOracle$AuditTrail] got [pilot.MailboxOracle$AuditTrail] | terminated | again | started=1
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
