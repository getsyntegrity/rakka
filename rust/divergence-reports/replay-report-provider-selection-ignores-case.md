# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-selection-ignores-case/release/examples/classic-parity`

Passed: 2610/2621

Fallback rate: not reported by this build

## Failing cases

### da-0024

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-08 provider LOCAL (case differs) -> ERR ClassNotFoundException [LOCAL]
+ PRV-08 provider LOCAL (case differs) -> ok created
```

### da-0025

Arguments: `dynamic-access-case`

stdout:

```diff
- PRV-09 provider other.ActorRefProvider -> ERR ClassNotFoundException [other.ActorRefProvider]
+ PRV-09 provider other.ActorRefProvider -> ERR ClassNotFoundException [other.actorrefprovider]
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

### ps-0004

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-04 provider = org.apache.pekko.remote.RemoteActorRefProvider -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
+ SEL-04 provider = org.apache.pekko.remote.RemoteActorRefProvider -> ok identifier=custom fqcn=org.apache.pekko.remote.remoteactorrefprovider hasCluster=false ProviderClass=org.apache.pekko.remote.remoteactorrefprovider HasCluster=false
```

### ps-0005

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-05 provider = org.apache.pekko.cluster.ClusterActorRefProvider -> ok identifier=cluster fqcn=org.apache.pekko.cluster.ClusterActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.cluster.ClusterActorRefProvider HasCluster=true
+ SEL-05 provider = org.apache.pekko.cluster.ClusterActorRefProvider -> ok identifier=custom fqcn=org.apache.pekko.cluster.clusteractorrefprovider hasCluster=false ProviderClass=org.apache.pekko.cluster.clusteractorrefprovider HasCluster=false
```

### ps-0006

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-06 provider = other.ActorRefProvider -> ok identifier=custom fqcn=other.ActorRefProvider hasCluster=false ProviderClass=other.ActorRefProvider HasCluster=false
+ SEL-06 provider = other.ActorRefProvider -> ok identifier=custom fqcn=other.actorrefprovider hasCluster=false ProviderClass=other.actorrefprovider HasCluster=false
```

### ps-0007

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=custom fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
+ SEL-07 provider = org.apache.pekko.actor.LocalActorRefProvider (the class name of Local is Custom) -> ok identifier=custom fqcn=org.apache.pekko.actor.localactorrefprovider hasCluster=false ProviderClass=org.apache.pekko.actor.localactorrefprovider HasCluster=false
```

### ps-0008

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-08 provider = LOCAL (case differs) -> ok identifier=custom fqcn=LOCAL hasCluster=false ProviderClass=LOCAL HasCluster=false
+ SEL-08 provider = LOCAL (case differs) -> ok identifier=local fqcn=org.apache.pekko.actor.LocalActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.actor.LocalActorRefProvider HasCluster=false
```

### ps-0009

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-09 provider = Remote (case differs) -> ok identifier=custom fqcn=Remote hasCluster=false ProviderClass=Remote HasCluster=false
+ SEL-09 provider = Remote (case differs) -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
```

### ps-0014

Arguments: `provider-selection-case`

stdout:

```diff
- EQ-01 remote equals org.apache.pekko.remote.RemoteActorRefProvider -> ok true
+ EQ-01 remote equals org.apache.pekko.remote.RemoteActorRefProvider -> ok false
```

### ps-0015

Arguments: `provider-selection-case`

stdout:

```diff
- EQ-02 cluster equals org.apache.pekko.cluster.ClusterActorRefProvider -> ok true
+ EQ-02 cluster equals org.apache.pekko.cluster.ClusterActorRefProvider -> ok false
```
