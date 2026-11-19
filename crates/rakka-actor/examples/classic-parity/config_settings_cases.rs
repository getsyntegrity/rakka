// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Cases of the AC-CFG settings oracle (`ConfigSettingsOracle.java`, unit P08): each rebuilds one oracle case with the Rust
//! `Settings` and `amend_slf4j_config` and renders it with the oracle's line format.

use rakka_actor::config::{
    Config, ConfigError, SLF4J_LOGGER, Settings, SettingsError, SuspendDuration, amend_slf4j_config,
};
use rakka_actor::util::TimeUnit;

/// Oracle ids Rust does not produce on purpose: `(id, reason)`. Each must exist in the oracle.
#[allow(dead_code)]
pub const PENDING: &[(&str, &str)] = &[
    (
        "SET-SER serialize-messages = on",
        "Q-CFG-11: Settings::new rejects serialize-messages=on until AC-SER exists (Pekko accepts it); tested in config/settings.rs",
    ),
    (
        "SET-SER serialize-creators = on",
        "Q-CFG-11: Settings::new rejects serialize-creators=on until AC-SER exists (Pekko accepts it); tested in config/settings.rs",
    ),
    (
        "SET-SER serialize-messages and serialize-creators = on",
        "Q-CFG-11: Settings::new rejects both keys on until AC-SER exists (Pekko accepts them); tested in config/settings.rs",
    ),
];

fn esc(s: &str) -> String {
    let mut out = String::new();
    for unit in s.encode_utf16() {
        if (0x20..=0x7e).contains(&unit) {
            out.push(char::from(u8::try_from(unit).expect("ASCII")));
        } else {
            out.push_str(&format!("\\u{unit:04X}"));
        }
    }
    out
}

/// The exception the oracle prints: its simple class name and message.
struct Failure {
    name: &'static str,
    message: String,
}

impl From<ConfigError> for Failure {
    fn from(e: ConfigError) -> Failure {
        Failure {
            name: e.kind().java_name(),
            message: e.message().to_owned(),
        }
    }
}

impl From<SettingsError> for Failure {
    fn from(e: SettingsError) -> Failure {
        Failure {
            name: e.java_name(),
            message: e.message().to_owned(),
        }
    }
}

type R = Result<String, Failure>;

fn parse(text: &str) -> Result<Config, ConfigError> {
    Config::parse_string(text)
}

fn list(items: &[String]) -> String {
    format!("[{}]", items.join(", "))
}

fn ms(t: rakka_actor::util::Timeout) -> String {
    format!("{}ms", t.as_nanos() / 1_000_000)
}

fn suspend(d: SuspendDuration) -> String {
    match d {
        SuspendDuration::Infinite => "Inf".to_owned(),
        SuspendDuration::Finite(t) => ms(t),
    }
}

fn java_double(v: f64) -> String {
    format!("{v:?}")
}

/// Settings over `app` merged on the base (typed over actor reference.conf, resolved).
fn set(app: &str) -> Result<Settings, Failure> {
    let config = parse(app)?
        .with_fallback(&Config::default_reference_with_typed())
        .resolve()?;
    Ok(Settings::new("oracle", config)?)
}

fn show(s: &Settings) -> String {
    let prefix: Vec<String> = s
        .no_serialization_verification_needed_class_prefix
        .iter()
        .cloned()
        .collect();
    format!(
        "ConfigVersion={} SupervisorStrategyClass={} CreationTimeout={} UnstartedPushTimeout={} AllowJavaSerialization={} \
         SerializeAllMessages={} SerializeAllCreators={} NoSerializationVerificationNeededClassPrefix={} LogLevel={} \
         StdoutLogLevel={} Loggers={} LoggersDispatcher={} LoggingFilter={} LoggerStartTimeout={} LogConfigOnStart={} \
         LogDeadLetters={} LogDeadLettersDuringShutdown={} LogDeadLettersSuspendDuration={} AddLoggingReceive={} \
         DebugAutoReceive={} DebugLifecycle={} FsmDebugEvent={} DebugEventStream={} DebugUnhandledMessage={} \
         DebugRouterMisconfiguration={} Home={} SchedulerClass={} Daemonicity={} JvmExitOnFatalError={} JvmShutdownHooks={} \
         FailMixedVersions={} CoordinatedShutdownTerminateActorSystem={} CoordinatedShutdownRunByActorSystemTerminate={} \
         DefaultVirtualNodesFactor={}",
        s.config_version,
        s.supervisor_strategy_class,
        ms(s.creation_timeout),
        ms(s.unstarted_push_timeout),
        s.allow_java_serialization,
        s.serialize_all_messages,
        s.serialize_all_creators,
        list(&prefix),
        s.log_level,
        s.stdout_log_level,
        list(&s.loggers),
        s.loggers_dispatcher,
        s.logging_filter,
        ms(s.logger_start_timeout),
        s.log_config_on_start,
        s.log_dead_letters,
        s.log_dead_letters_during_shutdown,
        suspend(s.log_dead_letters_suspend_duration),
        s.add_logging_receive,
        s.debug_auto_receive,
        s.debug_lifecycle,
        s.fsm_debug_event,
        s.debug_event_stream,
        s.debug_unhandled_message,
        s.debug_router_misconfiguration,
        match &s.home {
            Some(h) => format!("Some({h})"),
            None => "None".to_owned(),
        },
        s.scheduler_class,
        s.daemonicity,
        s.jvm_exit_on_fatal_error,
        s.jvm_shutdown_hooks,
        s.fail_mixed_versions,
        s.coordinated_shutdown_terminate_actor_system,
        s.coordinated_shutdown_run_by_actor_system_terminate,
        s.default_virtual_nodes_factor,
    )
}

fn amend(app: &str, available: bool) -> R {
    let input = parse(app)?.with_fallback(&Config::default_reference());
    let out = amend_slf4j_config(&input, |_| available)?;
    Ok(format!(
        "loggers={} logging-filter={}",
        list(&out.get_string_list("pekko.loggers")?),
        out.get_string("pekko.logging-filter")?
    ))
}

type Cases = Vec<(String, String)>;

fn add(lines: &mut Cases, id: String, r: R) {
    let line = match r {
        Ok(v) => format!("ok {v}"),
        Err(e) => format!("ERR {} [{}]", e.name, e.message),
    };
    lines.push((esc(&id), esc(&format!("{id} -> {line}"))));
}

fn defaults(lines: &mut Cases) {
    add(
        lines,
        "SET-DEFAULT the reference configuration".into(),
        set("").map(|s| show(&s)),
    );
    add(
        lines,
        "SET-A9 dispatcher and mailbox keys of ConfigSpec".into(),
        (|| {
            let s = set("")?;
            let root = s.config();
            let c = root.get_config("pekko.actor.default-dispatcher")?;
            let pool = c.get_config("default-executor")?;
            let fj = c.get_config("fork-join-executor")?;
            let tp = c.get_config("thread-pool-executor")?;
            let mb = root.get_config("pekko.actor.default-mailbox")?;
            let msec = TimeUnit::Milliseconds;
            Ok(format!(
                "type={} executor={} shutdown-timeout={} throughput={} throughput-deadline-time={} attempt-teamwork={} \
                 fallback={} fj-parallelism-min={} fj-parallelism-factor={} fj-parallelism-max={} fj-task-peeking-mode={} \
                 tp-keep-alive-time={} tp-core-pool-size-factor={} tp-max-pool-size-factor={} tp-task-queue-size={} \
                 tp-task-queue-type={} tp-allow-core-timeout={} tp-fixed-pool-size={} mailbox-capacity={} \
                 mailbox-push-timeout-time={} mailbox-type={}",
                c.get_string("type")?,
                c.get_string("executor")?,
                c.get_duration("shutdown-timeout", msec)?,
                c.get_int("throughput")?,
                c.get_duration("throughput-deadline-time", msec)?,
                c.get_boolean("attempt-teamwork")?,
                pool.get_string("fallback")?,
                fj.get_int("parallelism-min")?,
                java_double(fj.get_double("parallelism-factor")?),
                fj.get_int("parallelism-max")?,
                fj.get_string("task-peeking-mode")?,
                tp.get_duration("keep-alive-time", msec)?,
                java_double(tp.get_double("core-pool-size-factor")?),
                java_double(tp.get_double("max-pool-size-factor")?),
                tp.get_int("task-queue-size")?,
                tp.get_string("task-queue-type")?,
                tp.get_boolean("allow-core-timeout")?,
                tp.get_string("fixed-pool-size")?,
                mb.get_int("mailbox-capacity")?,
                mb.get_duration("mailbox-push-timeout-time", msec)?,
                mb.get_string("mailbox-type")?,
            ))
        })(),
    );
    let ext = |app: &str, keys: &[(&str, &str)]| -> R {
        let s = set(app)?;
        let parts: Result<Vec<String>, ConfigError> = keys
            .iter()
            .map(|(label, key)| {
                Ok(format!(
                    "{label}={}",
                    list(&s.config().get_string_list(key)?)
                ))
            })
            .collect();
        Ok(parts?.join(" "))
    };
    let both = [
        ("library-extensions", "pekko.library-extensions"),
        ("extensions", "pekko.extensions"),
    ];
    add(
        lines,
        "SET-EXT default library-extensions and extensions".into(),
        ext("", &both),
    );
    add(
        lines,
        "SET-EXT library-extensions += and extensions set by the application".into(),
        ext(
            "pekko.library-extensions += \"my.Ext\"\npekko.extensions = [\"a.B\", \"c.D$\"]",
            &both,
        ),
    );
    add(
        lines,
        "SET-EXT library-extensions replaced by the application".into(),
        ext("pekko.library-extensions = [\"only.One\"]", &both[..1]),
    );
    add(
        lines,
        "SET-EXT extensions of a wrong type".into(),
        set("pekko.extensions = 5").map(|s| format!("{:?}", s.config())),
    );
    add(
        lines,
        "SET-PROVIDER default provider key".into(),
        (|| Ok(set("")?.config().get_string("pekko.actor.provider")?))(),
    );
}

fn dead_letters(lines: &mut Cases) {
    for v in [
        "on",
        "off",
        "true",
        "false",
        "ON",
        "Off",
        "TRUE",
        "False",
        "\"ON\"",
        "\"off\"",
        "10",
        "0",
        "5",
        "-1",
        "2147483647",
        "\"5\"",
        "yes",
        "no",
        "1.5",
        "abc",
        "3000000000",
        "\"\"",
        "[1]",
    ] {
        add(
            lines,
            format!("SET-LDL log-dead-letters = {v}"),
            set(&format!("pekko.log-dead-letters = {v}"))
                .map(|s| format!("LogDeadLetters={}", s.log_dead_letters)),
        );
    }
    for v in [
        "infinite",
        "Infinite",
        "INFINITE",
        "inf",
        "5s",
        "1 minute",
        "0",
        "100",
        "-1s",
        "off",
        "unlimited",
        "1.5s",
        "5 foo",
        "106752 d",
        "106751 d",
        "9223372036854775807 d",
        "9223372036854775808 d",
        "-9223372036854775808 d",
        "1ns",
        "999999ns",
        "\"infinite\"",
        "\"\"",
        "[1]",
        "on",
        "2562047 h",
    ] {
        add(
            lines,
            format!("SET-LDSD log-dead-letters-suspend-duration = {v}"),
            set(&format!("pekko.log-dead-letters-suspend-duration = {v}")).map(|s| {
                format!(
                    "LogDeadLettersSuspendDuration={}",
                    suspend(s.log_dead_letters_suspend_duration)
                )
            }),
        );
    }
    for v in ["on", "off", "true", "false", "yes", "abc", "1", "\"on\""] {
        add(
            lines,
            format!("SET-LDDS log-dead-letters-during-shutdown = {v}"),
            set(&format!("pekko.log-dead-letters-during-shutdown = {v}")).map(|s| {
                format!(
                    "LogDeadLettersDuringShutdown={}",
                    s.log_dead_letters_during_shutdown
                )
            }),
        );
    }
}

fn debug_and_jvm(lines: &mut Cases) {
    let debug = [
        "receive",
        "autoreceive",
        "lifecycle",
        "fsm",
        "event-stream",
        "unhandled",
        "router-misconfiguration",
    ];
    for k in debug {
        add(
            lines,
            format!("SET-DEBUG {k} = on"),
            set(&format!("pekko.actor.debug.{k} = on")).map(|s| show(&s)),
        );
    }
    let all: String = debug
        .iter()
        .map(|k| format!("pekko.actor.debug.{k} = true\n"))
        .collect();
    add(
        lines,
        "SET-DEBUG all seven on".into(),
        set(&all).map(|s| show(&s)),
    );
    add(
        lines,
        "SET-DEBUG receive of a wrong type".into(),
        set("pekko.actor.debug.receive = abc").map(|s| show(&s)),
    );
    add(
        lines,
        "SET-DEBUG lifecycle quoted".into(),
        set("pekko.actor.debug.lifecycle = \"yes\"").map(|s| show(&s)),
    );
    for k in [
        "daemonic = on",
        "jvm-exit-on-fatal-error = off",
        "jvm-shutdown-hooks = off",
        "fail-mixed-versions = off",
        "daemonic = abc",
        "jvm-exit-on-fatal-error = 3",
    ] {
        add(
            lines,
            format!("SET-JVM {k}"),
            set(&format!("pekko.{k}")).map(|s| show(&s)),
        );
    }
    add(
        lines,
        "SET-JVM all flipped".into(),
        set("pekko.daemonic = on\npekko.jvm-exit-on-fatal-error = off\npekko.jvm-shutdown-hooks = off\npekko.fail-mixed-versions = off")
            .map(|s| show(&s)),
    );
    let home = |s: Settings| match s.home {
        Some(h) => format!("Home=Some({h})"),
        None => "Home=None".to_owned(),
    };
    for h in ["\"/opt/pekko\"", "\"\"", "\" \"", "x", "[1]", "5"] {
        add(
            lines,
            format!("SET-HOME home = {h}"),
            set(&format!("pekko.home = {h}")).map(home),
        );
    }
    add(
        lines,
        "SET-HOME home = null".into(),
        set("pekko.home = null").map(home),
    );
    add(
        lines,
        "SET-SCHEDULER implementation = my.Sched".into(),
        set("pekko.scheduler.implementation = my.Sched")
            .map(|s| format!("SchedulerClass={}", s.scheduler_class)),
    );
    add(
        lines,
        "SET-SCHEDULER implementation = [1]".into(),
        set("pekko.scheduler.implementation = [1]")
            .map(|s| format!("SchedulerClass={}", s.scheduler_class)),
    );
}

fn timeouts_and_logging(lines: &mut Cases) {
    for t in [
        "1s",
        "2500ms",
        "0",
        "-1s",
        "abc",
        "1.5 minutes",
        "off",
        "1 d",
        "100",
    ] {
        add(
            lines,
            format!("SET-TO creation-timeout = {t}"),
            set(&format!("pekko.actor.creation-timeout = {t}"))
                .map(|s| format!("CreationTimeout={}", ms(s.creation_timeout))),
        );
    }
    for t in ["500ms", "1 minute", "-5ms", "x"] {
        add(
            lines,
            format!("SET-TO unstarted-push-timeout = {t}"),
            set(&format!("pekko.actor.unstarted-push-timeout = {t}"))
                .map(|s| format!("UnstartedPushTimeout={}", ms(s.unstarted_push_timeout))),
        );
    }
    for t in ["7s", "1ms", "0", "never"] {
        add(
            lines,
            format!("SET-TO logger-startup-timeout = {t}"),
            set(&format!("pekko.logger-startup-timeout = {t}"))
                .map(|s| format!("LoggerStartTimeout={}", ms(s.logger_start_timeout))),
        );
    }
    let loggers = |app: &str| set(app).map(|s| format!("Loggers={}", list(&s.loggers)));
    add(
        lines,
        "SET-LOG loggers = []".into(),
        loggers("pekko.loggers = []"),
    );
    add(
        lines,
        "SET-LOG loggers = two names".into(),
        loggers("pekko.loggers = [\"a.B\", \"c.D$E\"]"),
    );
    add(
        lines,
        "SET-LOG loggers with a repeated name".into(),
        loggers("pekko.loggers = [\"a.B\", \"a.B\"]"),
    );
    add(
        lines,
        "SET-LOG loggers = a string".into(),
        loggers("pekko.loggers = \"a.B\""),
    );
    add(
        lines,
        "SET-LOG loggers = numbers".into(),
        loggers("pekko.loggers = [1, 2]"),
    );
    add(
        lines,
        "SET-LOG loggers = list of objects".into(),
        loggers("pekko.loggers = [{a = 1}]"),
    );
    add(
        lines,
        "SET-LOG logging-filter = x.Filter".into(),
        set("pekko.logging-filter = x.Filter")
            .map(|s| format!("LoggingFilter={}", s.logging_filter)),
    );
    add(
        lines,
        "SET-LOG logging-filter = a number".into(),
        set("pekko.logging-filter = 5").map(|s| format!("LoggingFilter={}", s.logging_filter)),
    );
    add(
        lines,
        "SET-LOG loggers-dispatcher = my-dispatcher".into(),
        set("pekko.loggers-dispatcher = my-dispatcher")
            .map(|s| format!("LoggersDispatcher={}", s.loggers_dispatcher)),
    );
    for l in ["DEBUG", "OFF", "debug", "\"WARNING\"", "5", "off"] {
        add(
            lines,
            format!("SET-LOG loglevel = {l}"),
            set(&format!("pekko.loglevel = {l}")).map(|s| format!("LogLevel={}", s.log_level)),
        );
        add(
            lines,
            format!("SET-LOG stdout-loglevel = {l}"),
            set(&format!("pekko.stdout-loglevel = {l}"))
                .map(|s| format!("StdoutLogLevel={}", s.stdout_log_level)),
        );
    }
    for l in ["on", "off", "true", "abc", "1"] {
        add(
            lines,
            format!("SET-LOG log-config-on-start = {l}"),
            set(&format!("pekko.log-config-on-start = {l}"))
                .map(|s| format!("LogConfigOnStart={}", s.log_config_on_start)),
        );
    }
    add(
        lines,
        "SET-GUARDIAN guardian-supervisor-strategy = StoppingSupervisorStrategy".into(),
        set("pekko.actor.guardian-supervisor-strategy = \"org.apache.pekko.actor.StoppingSupervisorStrategy\"")
            .map(|s| format!("SupervisorStrategyClass={}", s.supervisor_strategy_class)),
    );
    add(
        lines,
        "SET-GUARDIAN guardian-supervisor-strategy = a list".into(),
        set("pekko.actor.guardian-supervisor-strategy = [1]")
            .map(|s| format!("SupervisorStrategyClass={}", s.supervisor_strategy_class)),
    );
}

fn serialization_and_shutdown(lines: &mut Cases) {
    add(
        lines,
        "SET-SER allow-java-serialization = on".into(),
        set("pekko.actor.allow-java-serialization = on")
            .map(|s| format!("AllowJavaSerialization={}", s.allow_java_serialization)),
    );
    // SET-SER serialize-messages = on, serialize-creators = on and both: PENDING (Q-CFG-11).
    add(
        lines,
        "SET-SER serialize-messages = off with prefix []".into(),
        set("pekko.actor.serialize-messages = off\npekko.actor.no-serialization-verification-needed-class-prefix = []").map(|s| {
            format!(
                "SerializeAllMessages={} prefix={}",
                s.serialize_all_messages,
                list(&s.no_serialization_verification_needed_class_prefix.iter().cloned().collect::<Vec<_>>())
            )
        }),
    );
    add(
        lines,
        "SET-SER prefix with a repeated entry".into(),
        set("pekko.actor.no-serialization-verification-needed-class-prefix = [\"b.\", \"a.\", \"b.\"]").map(|s| {
            format!(
                "prefix={}",
                list(&s.no_serialization_verification_needed_class_prefix.iter().cloned().collect::<Vec<_>>())
            )
        }),
    );
    add(
        lines,
        "SET-SER prefix = a string".into(),
        set("pekko.actor.no-serialization-verification-needed-class-prefix = \"x.\"").map(|s| {
            format!(
                "prefix={}",
                list(
                    &s.no_serialization_verification_needed_class_prefix
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                )
            )
        }),
    );
    add(
        lines,
        "SET-SER serialize-messages = abc".into(),
        set("pekko.actor.serialize-messages = abc").map(|s| show(&s)),
    );
    for a in ["on", "off"] {
        for b in ["on", "off"] {
            add(
                lines,
                format!("SET-CS terminate-actor-system = {a} run-by-actor-system-terminate = {b}"),
                set(&format!(
                    "pekko.coordinated-shutdown.terminate-actor-system = {a}\npekko.coordinated-shutdown.run-by-actor-system-terminate = {b}"
                ))
                .map(|s| {
                    format!(
                        "CoordinatedShutdownTerminateActorSystem={} CoordinatedShutdownRunByActorSystemTerminate={}",
                        s.coordinated_shutdown_terminate_actor_system, s.coordinated_shutdown_run_by_actor_system_terminate
                    )
                }),
            );
        }
    }
    add(
        lines,
        "SET-CS terminate-actor-system = off with the default run-by-actor-system-terminate".into(),
        set("pekko.coordinated-shutdown.terminate-actor-system = off").map(|s| show(&s)),
    );
    add(
        lines,
        "SET-CS run-by-actor-system-terminate = off alone".into(),
        set("pekko.coordinated-shutdown.run-by-actor-system-terminate = off").map(|s| show(&s)),
    );
    add(
        lines,
        "SET-CS terminate-actor-system = abc".into(),
        set("pekko.coordinated-shutdown.terminate-actor-system = abc").map(|s| show(&s)),
    );
    add(
        lines,
        "SET-CS the unsupported combination is reported before the version check".into(),
        set("pekko.coordinated-shutdown.terminate-actor-system = off\npekko.version = \"1.0.0\"")
            .map(|s| show(&s)),
    );
    add(
        lines,
        "SET-CS the unsupported combination is reported before a bad virtual-nodes-factor".into(),
        set("pekko.coordinated-shutdown.terminate-actor-system = off\npekko.actor.deployment.default.virtual-nodes-factor = abc")
            .map(|s| show(&s)),
    );
    for v in ["3", "0", "-2", "abc", "1.5", "\"7\""] {
        add(
            lines,
            format!("SET-VNF virtual-nodes-factor = {v}"),
            set(&format!(
                "pekko.actor.deployment.default.virtual-nodes-factor = {v}"
            ))
            .map(|s| {
                format!(
                    "DefaultVirtualNodesFactor={}",
                    s.default_virtual_nodes_factor
                )
            }),
        );
    }
    for v in ["\"2.0.0\"", "\"1.7.1\"", "\"\"", "\"2.0.0-M1\"", "2", "[1]"] {
        add(
            lines,
            format!("SET-VERSION pekko.version = {v}"),
            set(&format!("pekko.version = {v}"))
                .map(|s| format!("ConfigVersion={}", s.config_version)),
        );
    }
}

fn invalid(lines: &mut Cases) {
    let base = Config::default_reference_with_typed();
    let shown = |app: &str| set(app).map(|s| show(&s));
    add(
        lines,
        "SET-INVALID a key of the reference is an object in the application".into(),
        shown("pekko.loglevel { x = 1 }"),
    );
    add(
        lines,
        "SET-INVALID creation-timeout is a list".into(),
        shown("pekko.actor.creation-timeout = [1]"),
    );
    add(
        lines,
        "SET-INVALID creation-timeout is an object".into(),
        shown("pekko.actor.creation-timeout { x = 1 }"),
    );
    add(
        lines,
        "SET-INVALID several problems are reported together".into(),
        shown("pekko.loglevel { x = 1 }\npekko.loggers = 5\npekko.daemonic = [1]"),
    );
    add(
        lines,
        "SET-INVALID internal-dispatcher as a dispatcher name is allowed".into(),
        shown("pekko.actor.internal-dispatcher = \"pekko.actor.default-dispatcher\""),
    );
    add(
        lines,
        "SET-INVALID internal-dispatcher removed from the application".into(),
        (|| {
            Ok(show(&Settings::new(
                "oracle",
                base.without_path("pekko.actor.internal-dispatcher")?,
            )?))
        })(),
    );
    add(
        lines,
        "SET-INVALID an unresolved configuration".into(),
        (|| {
            Ok(show(&Settings::new(
                "oracle",
                parse("pekko.loglevel = ${x}")?.with_fallback(&base),
            )?))
        })(),
    );
    add(
        lines,
        "SET-INVALID the application adds unknown keys".into(),
        shown("pekko.unknown-key = 1\nmy-app.x = 2"),
    );
}

fn amend_cases(lines: &mut Cases) {
    const SLF4J_FILTER: &str = "org.apache.pekko.event.slf4j.Slf4jLoggingFilter";
    const DEFAULT_LOGGER: &str = "org.apache.pekko.event.Logging$DefaultLogger";
    let configs: Vec<(&str, String)> = vec![
        ("the reference configuration", String::new()),
        ("use-slf4j = on", "pekko.use-slf4j = on".into()),
        ("use-slf4j = off", "pekko.use-slf4j = off".into()),
        ("use-slf4j = true", "pekko.use-slf4j = true".into()),
        ("use-slf4j = yes", "pekko.use-slf4j = yes".into()),
        ("use-slf4j = quoted on", "pekko.use-slf4j = \"on\"".into()),
        ("use-slf4j = maybe", "pekko.use-slf4j = maybe".into()),
        ("use-slf4j = a list", "pekko.use-slf4j = [1]".into()),
        ("use-slf4j = null", "pekko.use-slf4j = null".into()),
        (
            "slf4j logger already configured with the default filter",
            format!("pekko.loggers = [\"{SLF4J_LOGGER}\"]"),
        ),
        (
            "slf4j logger already configured with the slf4j filter",
            format!(
                "pekko.loggers = [\"{SLF4J_LOGGER}\"]\npekko.logging-filter = \"{SLF4J_FILTER}\""
            ),
        ),
        (
            "slf4j logger already configured with a custom filter",
            format!("pekko.loggers = [\"{SLF4J_LOGGER}\"]\npekko.logging-filter = my.Filter"),
        ),
        (
            "slf4j logger and the default logger",
            format!("pekko.loggers = [\"{SLF4J_LOGGER}\", \"{DEFAULT_LOGGER}\"]"),
        ),
        (
            "use-slf4j = on with the default logger and a custom one",
            format!("pekko.use-slf4j = on\npekko.loggers = [\"{DEFAULT_LOGGER}\", \"my.Logger\"]"),
        ),
        (
            "use-slf4j = on with a custom filter",
            "pekko.use-slf4j = on\npekko.logging-filter = my.Filter".into(),
        ),
        (
            "use-slf4j = on with the slf4j filter",
            format!("pekko.use-slf4j = on\npekko.logging-filter = \"{SLF4J_FILTER}\""),
        ),
        (
            "use-slf4j = on with no loggers",
            "pekko.use-slf4j = on\npekko.loggers = []".into(),
        ),
        (
            "use-slf4j = on with only a custom logger",
            "pekko.use-slf4j = on\npekko.loggers = [\"my.Logger\"]".into(),
        ),
        (
            "use-slf4j = on with the default logger twice",
            format!(
                "pekko.use-slf4j = on\npekko.loggers = [\"{DEFAULT_LOGGER}\", \"{DEFAULT_LOGGER}\"]"
            ),
        ),
        (
            "use-slf4j = off with the slf4j logger",
            format!("pekko.use-slf4j = off\npekko.loggers = [\"{SLF4J_LOGGER}\"]"),
        ),
        (
            "loggers is a string",
            "pekko.use-slf4j = on\npekko.loggers = \"x\"".into(),
        ),
        (
            "logging-filter is a list",
            "pekko.use-slf4j = on\npekko.logging-filter = [1]".into(),
        ),
    ];
    for (label, app) in &configs {
        for available in [true, false] {
            let suffix = if available {
                ", pekko-slf4j on the classpath"
            } else {
                ", pekko-slf4j not on the classpath"
            };
            add(
                lines,
                format!("AMEND {label}{suffix}"),
                amend(app, available),
            );
        }
    }
    add(
        lines,
        "AMEND the amended configuration is amended again without change".into(),
        (|| {
            let input = parse("pekko.use-slf4j = on")?.with_fallback(&Config::default_reference());
            let once = amend_slf4j_config(&input, |_| true)?;
            let twice = amend_slf4j_config(&once, |_| true)?;
            Ok(format!(
                "loggers={} logging-filter={}",
                list(&twice.get_string_list("pekko.loggers")?),
                twice.get_string("pekko.logging-filter")?
            ))
        })(),
    );
}

pub fn rust_lines() -> Vec<(String, String)> {
    let mut lines: Cases = Vec::new();
    defaults(&mut lines);
    dead_letters(&mut lines);
    debug_and_jvm(&mut lines);
    timeouts_and_logging(&mut lines);
    serialization_and_shutdown(&mut lines);
    invalid(&mut lines);
    amend_cases(&mut lines);
    lines
}
