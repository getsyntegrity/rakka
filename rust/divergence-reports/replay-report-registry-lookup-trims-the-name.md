# App replay

Build: `$TMPDIR/pekko-mutants-0nhdpmjn/target-registry-lookup-trims-the-name/release/examples/classic-parity`

Passed: 2594/2596

Fallback rate: not reported by this build

## Failing cases

### da-0018

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-07 createInstanceFor name with surrounding spaces -> ERR ClassNotFoundException [ org.apache.pekko.dispatch.UnboundedMailbox ]
+ CNF-07 createInstanceFor name with surrounding spaces -> ok created org.apache.pekko.dispatch.UnboundedMailbox
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
