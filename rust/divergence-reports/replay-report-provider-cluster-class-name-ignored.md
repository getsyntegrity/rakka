# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-cluster-class-name-ignored/release/examples/classic-parity`

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

### ps-0005

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-05 provider = org.apache.pekko.cluster.ClusterActorRefProvider -> ok identifier=cluster fqcn=org.apache.pekko.cluster.ClusterActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.cluster.ClusterActorRefProvider HasCluster=true
+ SEL-05 provider = org.apache.pekko.cluster.ClusterActorRefProvider -> ok identifier=custom fqcn=org.apache.pekko.cluster.ClusterActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.cluster.ClusterActorRefProvider HasCluster=false
```

### ps-0015

Arguments: `provider-selection-case`

stdout:

```diff
- EQ-02 cluster equals org.apache.pekko.cluster.ClusterActorRefProvider -> ok true
+ EQ-02 cluster equals org.apache.pekko.cluster.ClusterActorRefProvider -> ok false
```
