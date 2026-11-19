# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-debug-lifecycle-reads-fsm/release/examples/classic-parity`

Passed: 2218/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0066

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0067

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG fsm = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=true DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG fsm = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=true DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0073

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle quoted -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle quoted -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
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
