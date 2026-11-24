Verdict path: full replay (2971 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-2860-76ij6mxc/target-mailbox-requirement-error-names-the-class-without-its-kind/release/examples/classic-parity`

Passed: 2969/2971

Fallback rate: not reported by this build

## Failing cases

### mb-0025

Arguments: `mailbox-case`

stdout:

```diff
- MBX-REQ-02 an actor that requires a queue semantics with no mailbox bound to it fails actorOf at the call and its name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause org.apache.pekko.ConfigurationException: Mailbox Mapping for [interface pilot.MailboxOracle$Unmapped] not configured | instancesStartedAfterRetryWithSameName=1
+ MBX-REQ-02 an actor that requires a queue semantics with no mailbox bound to it fails actorOf at the call and its name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause org.apache.pekko.ConfigurationException: Mailbox Mapping for [pilot.MailboxOracle$Unmapped] not configured | instancesStartedAfterRetryWithSameName=1
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
