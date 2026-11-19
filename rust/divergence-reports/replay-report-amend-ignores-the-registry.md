# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-amend-ignores-the-registry/release/examples/classic-parity`

Passed: 2211/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0172

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = on, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0176

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = true, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = true, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0178

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = yes, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = yes, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0180

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = quoted on, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = quoted on, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0196

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with the default logger and a custom one, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger, my.Logger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = on with the default logger and a custom one, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger, my.Logger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0198

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with a custom filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=my.Filter
+ AMEND use-slf4j = on with a custom filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=my.Filter
```

### cs-0200

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with the slf4j filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
+ AMEND use-slf4j = on with the slf4j filter, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0202

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with no loggers, pekko-slf4j not on the classpath -> ok loggers=[] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = on with no loggers, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0204

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with only a custom logger, pekko-slf4j not on the classpath -> ok loggers=[my.Logger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = on with only a custom logger, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger, my.Logger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
```

### cs-0206

Arguments: `config-settings-case`

stdout:

```diff
- AMEND use-slf4j = on with the default logger twice, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.Logging$DefaultLogger, org.apache.pekko.event.Logging$DefaultLogger] logging-filter=org.apache.pekko.event.DefaultLoggingFilter
+ AMEND use-slf4j = on with the default logger twice, pekko-slf4j not on the classpath -> ok loggers=[org.apache.pekko.event.slf4j.Slf4jLogger] logging-filter=org.apache.pekko.event.slf4j.Slf4jLoggingFilter
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
