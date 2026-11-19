# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-amend-always-replaces-the-filter/release/examples/classic-parity`

Passed: 2218/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0191

Arguments: `config-settings-case`

stdout:

```diff
- AMEND slf4j logger already configured with a custom filter, pekko-slf4j on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=my.Filter
+ AMEND slf4j logger already configured with a custom filter, pekko-slf4j on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0192

Arguments: `config-settings-case`

stdout:

```diff
- AMEND slf4j logger already configured with a custom filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=my.Filter
+ AMEND slf4j logger already configured with a custom filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0197

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with a custom filter, pekko-slf4j on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=my.Filter
+ AMEND use-slf4j = on with a custom filter, pekko-slf4j on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
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
