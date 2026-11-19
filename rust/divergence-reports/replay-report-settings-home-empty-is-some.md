# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-zlxslczp/target-settings-home-empty-is-some/release/examples/classic-parity`

Passed: 2201/2222

Fallback rate: not reported by this build

## Failing cases

### cs-0001

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEFAULT the reference configuration -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEFAULT the reference configuration -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0064

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG receive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG receive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0065

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG autoreceive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=true DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG autoreceive = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=true DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0066

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0067

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG fsm = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=true DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG fsm = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=true DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0068

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG event-stream = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=true DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG event-stream = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=true DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0069

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG unhandled = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=true DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG unhandled = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=true DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0070

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG router-misconfiguration = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=true Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG router-misconfiguration = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=true Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0071

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG all seven on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=true DebugLifecycle=true FsmDebugEvent=true DebugEventStream=true DebugUnhandledMessage=true DebugRouterMisconfiguration=true Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG all seven on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=true DebugAutoReceive=true DebugLifecycle=true FsmDebugEvent=true DebugEventStream=true DebugUnhandledMessage=true DebugRouterMisconfiguration=true Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0073

Arguments: `config-settings-case`

stdout:

```diff
- SET-DEBUG lifecycle quoted -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-DEBUG lifecycle quoted -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=true FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0074

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM daemonic = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM daemonic = on -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0075

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM jvm-exit-on-fatal-error = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=false JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM jvm-exit-on-fatal-error = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=false JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0076

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM jvm-shutdown-hooks = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=false FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM jvm-shutdown-hooks = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=false FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0077

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM fail-mixed-versions = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM fail-mixed-versions = off -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0080

Arguments: `config-settings-case`

stdout:

```diff
- SET-JVM all flipped -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=false JvmShutdownHooks=false FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-JVM all flipped -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=true JvmExitOnFatalError=false JvmShutdownHooks=false FailMixedVersions=false CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0082

Arguments: `config-settings-case`

stdout:

```diff
- SET-HOME home = "" -> ok Home=None
+ SET-HOME home = "" -> ok Home=Some()
```

### cs-0145

Arguments: `config-settings-case`

stdout:

```diff
- SET-CS run-by-actor-system-terminate = off alone -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=false DefaultVirtualNodesFactor=10
+ SET-CS run-by-actor-system-terminate = off alone -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=false DefaultVirtualNodesFactor=10
```

### cs-0165

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID internal-dispatcher as a dispatcher name is allowed -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID internal-dispatcher as a dispatcher name is allowed -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0166

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID internal-dispatcher removed from the application -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID internal-dispatcher removed from the application -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
```

### cs-0168

Arguments: `config-settings-case`

stdout:

```diff
- SET-INVALID the application adds unknown keys -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=None SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
+ SET-INVALID the application adds unknown keys -> ok ConfigVersion=2.0.0 SupervisorStrategyClass=org.apache.pekko.actor.DefaultSupervisorStrategy CreationTimeout=20000ms UnstartedPushTimeout=10000ms AllowJavaSerialization=false SerializeAllMessages=false SerializeAllCreators=false NoSerializationVerificationNeededClassPrefix=[org.apache.pekko.] LogLevel=INFO StdoutLogLevel=WARNING Loggers=[org.apache.pekko.event.Logging$DefaultLogger] LoggersDispatcher=pekko.actor.default-dispatcher LoggingFilter=org.apache.pekko.event.DefaultLoggingFilter LoggerStartTimeout=5000ms LogConfigOnStart=false LogDeadLetters=10 LogDeadLettersDuringShutdown=false LogDeadLettersSuspendDuration=300000ms AddLoggingReceive=false DebugAutoReceive=false DebugLifecycle=false FsmDebugEvent=false DebugEventStream=false DebugUnhandledMessage=false DebugRouterMisconfiguration=false Home=Some() SchedulerClass=org.apache.pekko.actor.LightArrayRevolverScheduler Daemonicity=false JvmExitOnFatalError=true JvmShutdownHooks=true FailMixedVersions=true CoordinatedShutdownTerminateActorSystem=true CoordinatedShutdownRunByActorSystemTerminate=true DefaultVirtualNodesFactor=10
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
