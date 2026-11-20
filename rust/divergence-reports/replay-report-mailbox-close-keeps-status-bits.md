# App replay

Build: `$TMPDIR/pekko-mutants-qfq24hxe/target-mailbox-close-keeps-status-bits/release/examples/classic-parity`

Passed: 2817/2819

Fallback rate: not reported by this build

## Failing cases

### mb-0005

Arguments: `mailbox-case`

stdout:

```diff
- MBX-ST-05 open to closed -> before open scheduled=true suspendCount=2 raw=10 | close1=true closed scheduled=false suspendCount=0 raw=1 close2=false closed scheduled=false suspendCount=0 raw=1
+ MBX-ST-05 open to closed -> before open scheduled=true suspendCount=2 raw=10 | close1=true open scheduled=true suspendCount=2 raw=11 close2=true open scheduled=true suspendCount=2 raw=11
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
