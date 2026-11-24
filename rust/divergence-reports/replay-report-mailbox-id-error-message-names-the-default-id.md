Verdict path: full replay (2966 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-86196-2hvd7507/target-mailbox-id-error-message-names-the-default-id/release/examples/classic-parity`

Passed: 2964/2966

Fallback rate: not reported by this build

## Failing cases

### mb-0020

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [typo] | cause org.apache.pekko.ConfigurationException: Mailbox Type [typo] not configured | instancesStartedAfterRetryWithSameName=1
+ MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [pekko.actor.default-mailbox] | cause org.apache.pekko.ConfigurationException: Mailbox Type [typo] not configured | instancesStartedAfterRetryWithSameName=1
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
