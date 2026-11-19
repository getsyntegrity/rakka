# App replay

Build: `$TMPDIR/pekko-mutants-0nhdpmjn/target-class-not-found-drops-the-name/release/examples/classic-parity`

Passed: 2586/2596

Fallback rate: not reported by this build

## Failing cases

### da-0001

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-01 createInstanceFor foo.NonExistingClass -> ERR ClassNotFoundException [foo.NonExistingClass]
+ CNF-01 createInstanceFor foo.NonExistingClass -> ERR ClassNotFoundException []
```

### da-0002

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-02 createInstanceFor org.apache.pekko.actor.foo.NonExistingClass -> ERR ClassNotFoundException [org.apache.pekko.actor.foo.NonExistingClass]
+ CNF-02 createInstanceFor org.apache.pekko.actor.foo.NonExistingClass -> ERR ClassNotFoundException []
```

### da-0004

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-04 createInstanceFor name differing only in case -> ERR ClassNotFoundException [org.apache.pekko.dispatch.unboundedmailbox]
+ CNF-04 createInstanceFor name differing only in case -> ERR ClassNotFoundException []
```

### da-0005

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-05 createInstanceFor the provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
+ CNF-05 createInstanceFor the provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException []
```

### da-0006

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-06 createInstanceFor the provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]
+ CNF-06 createInstanceFor the provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException []
```

### da-0014

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-01 provider foo.NonExisting -> ERR ClassNotFoundException [foo.NonExisting]
+ PRV-01 provider foo.NonExisting -> ERR ClassNotFoundException []
```

### da-0015

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-02 provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.remote.RemoteActorRefProvider]
+ PRV-02 provider org.apache.pekko.remote.RemoteActorRefProvider -> ERR ClassNotFoundException []
```

### da-0016

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-03 provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException [org.apache.pekko.cluster.ClusterActorRefProvider]
+ PRV-03 provider org.apache.pekko.cluster.ClusterActorRefProvider -> ERR ClassNotFoundException []
```

### da-0018

Arguments: `dynamic-access-case`

stdout:

```diff
- CNF-07 createInstanceFor name with surrounding spaces -> ERR ClassNotFoundException [ org.apache.pekko.dispatch.UnboundedMailbox ]
+ CNF-07 createInstanceFor name with surrounding spaces -> ERR ClassNotFoundException []
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
