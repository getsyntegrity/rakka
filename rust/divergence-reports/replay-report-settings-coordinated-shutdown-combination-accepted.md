# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-coordinated-shutdown-combination-accepted/release/examples/classic-parity`

Passed: 2217/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0142

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = off run-by-actor-system-terminate = on -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS terminate-actor-system = off run-by-actor-system-terminate = on -> ok CoordinatedShutdownTerminateActorSystem=false CoordinatedShutdownRunByActorSystemTerminate=true
```

### cs-0144

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=false CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0147

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS the unsupported combination is reported before the version check -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS the unsupported combination is reported before the version check -> ERR ConfigurationException [Pekko JAR version [2.0.0] does not match the provided config version [1.0.0]]
```

### cs-0148

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS the unsupported combination is reported before a bad virtual-nodes-factor -> ERR ConfigurationException [pekko.coordinated-shutdown.run-by-actor-system-terminate=on and pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.]
+ SET-CS the unsupported combination is reported before a bad virtual-nodes-factor -> ERR WrongType [String: 2: pekko.actor.deployment.default.virtual-nodes-factor has type STRING rather than NUMBER]
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
