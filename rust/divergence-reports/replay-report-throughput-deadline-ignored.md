Verdict path: full replay (2961 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-99457-336p785y/target-throughput-deadline-ignored/release/examples/classic-parity`

Passed: 2959/2961

Fallback rate: not reported by this build

## Failing cases

### mb-0016

Arguments: `mailbox-case`

stdout:

```diff
- MBX-SC-02 throughput-deadline-time ends the turn after a slow message -> [r:block, r:sleep-x1, r:sleep-y1, r:sleep-x2, r:sleep-y2, r:sleep-x3, r:sleep-y3]
+ MBX-SC-02 throughput-deadline-time ends the turn after a slow message -> [r:block, r:sleep-x1, r:sleep-x2, r:sleep-x3, r:sleep-y1, r:sleep-y2, r:sleep-y3]
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
