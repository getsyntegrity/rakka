# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-local-class-name-is-local/release/examples/classic-parity`

Passed: 2618/2621

Fallback rate: not reported by this build

## Failing cases

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

### ps-0007

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=custom fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
+ SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=local fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
```

### ps-0016

Arguments: `provider-selection-case`

stdout:

```diff
- EQ-03 local equals org.apache.pekko.actor.LocalActorRefProvider -> ok false
+ EQ-03 local equals org.apache.pekko.actor.LocalActorRefProvider -> ok true
```
