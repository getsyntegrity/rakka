# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-custom-identifier-changed/release/examples/classic-parity`

Passed: 2612/2621

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

### ps-0006

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-06 provider = other.ActorRefProvider -> ok identifier=custom fqcn=other.ActorRefProvider hasCluster=false ProviderClass=other.ActorRefProvider HasCluster=false
+ SEL-06 provider = other.ActorRefProvider -> ok identifier=other fqcn=other.ActorRefProvider hasCluster=false ProviderClass=other.ActorRefProvider HasCluster=false
```

### ps-0007

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=custom fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
+ SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=other fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
```

### ps-0008

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-08 provider = LOCAL (case differs) -> ok identifier=custom fqcn=LOCAL hasCluster=false ProviderClass=LOCAL HasCluster=false
+ SEL-08 provider = LOCAL (case differs) -> ok identifier=other fqcn=LOCAL hasCluster=false ProviderClass=LOCAL HasCluster=false
```

### ps-0009

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-09 provider = Remote (case differs) -> ok identifier=custom fqcn=Remote hasCluster=false ProviderClass=Remote HasCluster=false
+ SEL-09 provider = Remote (case differs) -> ok identifier=other fqcn=Remote hasCluster=false ProviderClass=Remote HasCluster=false
```

### ps-0010

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-10 provider = " local" (leading space) -> ok identifier=custom fqcn= local hasCluster=false ProviderClass= local HasCluster=false
+ SEL-10 provider = " local" (leading space) -> ok identifier=other fqcn= local hasCluster=false ProviderClass= local HasCluster=false
```

### ps-0011

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-11 provider = "cluster " (trailing space) -> ok identifier=custom fqcn=cluster  hasCluster=false ProviderClass=cluster  HasCluster=false
+ SEL-11 provider = "cluster " (trailing space) -> ok identifier=other fqcn=cluster  hasCluster=false ProviderClass=cluster  HasCluster=false
```

### ps-0012

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-12 provider = empty string -> ok identifier=custom fqcn= hasCluster=false ProviderClass= HasCluster=false
+ SEL-12 provider = empty string -> ok identifier=other fqcn= hasCluster=false ProviderClass= HasCluster=false
```

### ps-0013

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-13 provider = 5 (a number) -> ok identifier=custom ProviderClass=5
+ SEL-13 provider = 5 (a number) -> ok identifier=other ProviderClass=5
```
