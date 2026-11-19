# App replay

Build: `$TMPDIR/pekko-mutants-hrmb2f40/target-provider-remote-has-cluster/release/examples/classic-parity`

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

### ps-0002

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-02 provider = remote -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
+ SEL-02 provider = remote -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=true
```

### ps-0004

Arguments: `provider-selection-case`

stdout:

```diff
- SEL-04 provider = org.apache.pekko.remote.RemoteActorRefProvider -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=false ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=false
+ SEL-04 provider = org.apache.pekko.remote.RemoteActorRefProvider -> ok identifier=remote fqcn=org.apache.pekko.remote.RemoteActorRefProvider hasCluster=true ProviderClass=org.apache.pekko.remote.RemoteActorRefProvider HasCluster=true
```
