# App replay

Build: `$TMPDIR/pekko-mutants-v1vgu0u9/target-mailbox-queue-count-off-by-one/release/examples/classic-parity`

Passed: 2819/2822

Fallback rate: not reported by this build

## Failing cases

### mb-0009

Arguments: `mailbox-case`

stdout:

```diff
- MBX-MQ-01 default queue is FIFO and counts -> new: n=0 has=false | enqueue n=1 has=true n=2 has=true n=3 has=true | dequeue [a, b, c] then null | after n=0 has=false
+ MBX-MQ-01 default queue is FIFO and counts -> new: n=1 has=false | enqueue n=2 has=true n=3 has=true n=4 has=true | dequeue [a, b, c] then null | after n=1 has=false
```

### mb-0011

Arguments: `mailbox-case`

stdout:

```diff
- MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order -> source n=0 has=false | target n=3 has=true order [a, b, c]
+ MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order -> source n=1 has=false | target n=4 has=true order [a, b, c]
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
