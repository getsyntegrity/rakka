# App replay

Build: `$TMPDIR/pekko-mutants-famb5jti/target-system-queue-clean-up-drops-pending/release/examples/classic-parity`

Passed: 2822/2824

Fallback rate: not reported by this build

## Failing cases

### mb-0013

Arguments: `mailbox-case`

stdout:

```diff
- MBX-SQ-02 sentinel NoMessage closes the system queue -> drain=[Suspend] | afterSentinel has=false drain=[] (actor==null: dropped, not forwarded)
+ MBX-SQ-02 sentinel NoMessage closes the system queue -> drain=[] | afterSentinel has=false drain=[] (actor==null: dropped, not forwarded)
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
