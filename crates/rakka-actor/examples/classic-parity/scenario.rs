//! Scenario runner: executes a JSON scenario against the Rust Classic runtime and prints the same
//! trace as the Java oracle (`oracle/.../Scenario.java`) does for real Pekko.
//!
//! ```text
//! {"system": "name", "dispatcher": {"parallelism": 1, "throughput": 5}, "steps": [ {"op": "spawn", ...} ]}
//! ```
//!
//! Every step awaits whatever it observes (events, termination), or reports only invariants for
//! races, so traces are deterministic and never depend on a JVM-vs-Rust interleaving.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::json::{Value, parse_value, write_str, write_text};
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{
    Actor, ActorContext, ActorIdentity, AnyMessage, Identify, Kill, PoisonPill, Props,
};
use rakka_actor::actor::supervision::{
    AllForOneStrategy, Directive, OneForOneStrategy, SupervisorStrategy,
};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::actor::{ActorError, ActorRef};
use rakka_actor::dispatch::{Failed, Terminated};
use rakka_actor::event::{Channel, DeadLetter, ErrorEvent, UnhandledMessage, WarningEvent};

/// How long an `await-*` step waits for its condition (override with `SCENARIO_AWAIT_MS`, e.g. when debugging).
fn await_timeout() -> Duration {
    Duration::from_millis(
        std::env::var("SCENARIO_AWAIT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000),
    )
}

/// What an actor records; the JSON form is shared with the oracle.
#[derive(Clone)]
enum Event {
    PreStart,
    /// `sender`: the `,"sender":…` field of an actor with `log-sender`, else empty
    PostStop {
        sender: String,
    },
    Received {
        msg: Msg,
        sender: String,
        n: Option<u64>,
    },
    PreRestart {
        reason: String,
        msg: Option<Msg>,
        sender: String,
    },
    PostRestart {
        reason: String,
        sender: String,
    },
    /// `supervisor.log-decisions`: the cause a decider was given and the directive it chose
    Decide {
        cause: String,
        directive: String,
    },
    DeadLetter {
        msg: Msg,
        sender: String,
        recipient: String,
    },
    Unhandled {
        msg: Msg,
        sender: String,
        recipient: String,
    },
    /// `errors` channel: a published `Error`, already rendered
    Error(String),
    Terminated {
        actor: String,
        is: Option<String>,
        existence_confirmed: bool,
        address_terminated: bool,
        sender: String,
    },
    /// `post-stop-spawn` result, already rendered (`"ok":true,...` or the exception fields)
    PostStopSpawn(String),
}

/// A message as the oracle renders it: strings verbatim, anything else by JVM class name.
#[derive(Clone, PartialEq)]
enum Msg {
    Text(String),
    Class(&'static str),
    /// an `ActorIdentity`: correlation id and the path of its reference
    Identity(String, Option<String>),
    /// a [`WatchMsg`]
    Custom(String),
    Unknown,
}

fn msg_of(m: &AnyMessage) -> Msg {
    if let Some(t) = m.downcast_ref::<String>() {
        Msg::Text(t.clone())
    } else if let Some(i) = m.downcast_ref::<ActorIdentity>() {
        let correlation = i.correlation_id.downcast_ref::<String>().cloned();
        let path = i.actor_ref.as_ref().map(path_str);
        Msg::Identity(correlation.unwrap_or_default(), path)
    } else if m.is::<Kill>() {
        Msg::Class("org.apache.pekko.actor.Kill$")
    } else if let Some(custom) = m.downcast_ref::<WatchMsg>() {
        Msg::Custom(custom.0.clone())
    } else if m.is::<PoisonPill>() {
        Msg::Class("org.apache.pekko.actor.PoisonPill$")
    } else if m.is::<Failed>() {
        Msg::Class("org.apache.pekko.dispatch.sysmsg.Failed")
    } else {
        Msg::Unknown
    }
}

fn msg_json(out: &mut String, m: &Msg) {
    match m {
        Msg::Text(t) => write_text(out, t),
        Msg::Class(c) => out.push_str(&format!("{{\"class\":\"{c}\"}}")),
        Msg::Identity(correlation, path) => {
            out.push_str("{\"identity\":");
            write_text(out, correlation);
            out.push_str(",\"ref\":");
            match path {
                Some(p) => write_text(out, p),
                None => out.push_str("null"),
            }
            out.push('}');
        }
        Msg::Custom(name) => {
            out.push_str("{\"custom\":");
            write_text(out, name);
            out.push('}');
        }
        Msg::Unknown => out.push_str("{\"class\":\"?\"}"),
    }
}

impl Event {
    fn json(&self) -> String {
        let mut o = String::new();
        let q = |o: &mut String, t: &str| write_text(o, t);
        match self {
            Event::PreStart => o.push_str("{\"ev\":\"pre-start\"}"),
            Event::PostStop { sender } => o.push_str(&format!("{{\"ev\":\"post-stop\"{sender}}}")),
            Event::Received { msg, sender, n } => {
                o.push_str("{\"ev\":\"received\",\"msg\":");
                msg_json(&mut o, msg);
                o.push_str(",\"sender\":");
                q(&mut o, sender);
                if let Some(n) = n {
                    o.push_str(&format!(",\"n\":{n}"));
                }
                o.push('}');
            }
            Event::PreRestart {
                reason,
                msg,
                sender,
            } => {
                o.push_str("{\"ev\":\"pre-restart\",\"reason\":");
                q(&mut o, reason);
                o.push_str(",\"msg\":");
                match msg {
                    Some(m) => msg_json(&mut o, m),
                    None => o.push_str("null"),
                }
                o.push_str(sender);
                o.push('}');
            }
            Event::PostRestart { reason, sender } => {
                o.push_str("{\"ev\":\"post-restart\",\"reason\":");
                q(&mut o, reason);
                o.push_str(sender);
                o.push('}');
            }
            Event::Decide { cause, directive } => {
                o.push_str("{\"ev\":\"decide\",\"cause\":");
                q(&mut o, cause);
                o.push_str(",\"directive\":");
                q(&mut o, directive);
                o.push('}');
            }
            Event::Terminated {
                actor,
                is,
                existence_confirmed,
                address_terminated,
                sender,
            } => {
                o.push_str("{\"ev\":\"terminated\",\"actor\":");
                q(&mut o, actor);
                o.push_str(",\"is\":");
                match is {
                    Some(l) => q(&mut o, l),
                    None => o.push_str("null"),
                }
                o.push_str(&format!(",\"existenceConfirmed\":{existence_confirmed},\"addressTerminated\":{address_terminated},\"sender\":"));
                q(&mut o, sender);
                o.push('}');
            }
            Event::Error(rendered) => o.push_str(rendered),
            Event::PostStopSpawn(r) => o.push_str(&format!("{{\"ev\":\"post-stop-spawn\",{r}}}")),
            Event::DeadLetter {
                msg,
                sender,
                recipient,
            }
            | Event::Unhandled {
                msg,
                sender,
                recipient,
            } => {
                o.push_str(if matches!(self, Event::DeadLetter { .. }) {
                    "{\"ev\":\"dead-letter\""
                } else {
                    "{\"ev\":\"unhandled\""
                });
                o.push_str(",\"msg\":");
                msg_json(&mut o, msg);
                o.push_str(",\"sender\":");
                q(&mut o, sender);
                o.push_str(",\"recipient\":");
                q(&mut o, recipient);
                o.push('}');
            }
        }
        o
    }
}

/// A named gate a supervisor's decider waits on.
type Hold = Arc<(Mutex<bool>, Condvar)>;

/// `supervisor: {...}` of a scripted actor.
#[derive(Clone)]
struct SupervisorCfg {
    max: i32,
    decider: HashMap<String, String>,
    log_decisions: bool,
    hold: Option<String>,
    all_for_one: bool,
    /// `decider-throws`: the decider itself fails (IllegalStateException "decider") while deciding
    decider_throws: bool,
    /// the strategy's `loggingEnabled` (default true)
    logging: bool,
}

/// Everything the actors record, shared with the harness thread.
#[derive(Default)]
struct Ctx {
    events: Mutex<HashMap<String, Vec<Event>>>,
    order: Mutex<Vec<String>>,
    refs: Mutex<HashMap<String, ActorRef>>,
    /// "label:pre-start", "label:post-stop", "label:terminated:<is>" in global order
    life: Mutex<Vec<String>>,
    /// command id -> rendered result fields
    results: Mutex<HashMap<u64, String>>,
    cmd_ids: AtomicU64,
    gate_open: Mutex<bool>,
    gate_cv: Condvar,
    /// named holds a supervisor's decider waits on (`supervisor.hold`)
    holds: Mutex<HashMap<String, Hold>>,
    /// labels whose `postRestart` already threw (`throw-in-post-restart: "once"`)
    failed_post_restart: Mutex<HashSet<String>>,
}

impl Ctx {
    fn add(&self, label: &str, e: Event) {
        self.events
            .lock()
            .unwrap()
            .entry(label.to_string())
            .or_default()
            .push(e);
    }

    fn of(&self, label: &str) -> Vec<Event> {
        self.events
            .lock()
            .unwrap()
            .get(label)
            .cloned()
            .unwrap_or_default()
    }

    /// The label (alphabetically first) of a registered ref equal to `r` (same path and uid).
    fn label_of(&self, r: &ActorRef) -> Option<String> {
        let refs = self.refs.lock().unwrap();
        refs.iter()
            .filter(|(_, v)| *v == r)
            .map(|(k, _)| k.clone())
            .min()
    }

    fn hold(&self, name: &str) -> Hold {
        self.holds
            .lock()
            .unwrap()
            .entry(name.to_string())
            .or_default()
            .clone()
    }

    fn life(&self, e: String) {
        self.life.lock().unwrap().push(e);
    }

    fn release(&self) {
        *self.gate_open.lock().unwrap() = true;
        self.gate_cv.notify_all();
    }

    fn wait_gate(&self) {
        let g = self.gate_open.lock().unwrap();
        drop(self.gate_cv.wait_while(g, |open| !*open).unwrap());
    }
}

fn text(m: &AnyMessage) -> Option<String> {
    m.downcast_ref::<String>().cloned()
}

fn path_str(r: &ActorRef) -> String {
    r.path().to_string()
}

/// An actor that does nothing (slice 2a).
struct Noop;
impl Actor for Noop {}

/// The behaviour config of a scripted actor (`{"kind": "echo", ...}`).
#[derive(Clone)]
struct Behavior {
    kind: String,
    handles: Option<Vec<String>>,
    to: Option<String>,
    msg: Option<String>,
    prefix: Option<String>,
    count: i64,
    post_stop_spawn: Option<String>,
    throw_on: Vec<String>,
    /// `supervisor: {"max": n, "log-decisions": bool, "decider": {<exception message>: "resume"|"restart"|"stop"|"escalate"}}`
    supervisor: Option<SupervisorCfg>,
    throw_in_prestart: bool,
    /// `throw-in-post-restart`: `postRestart` records its event and then throws IllegalStateException("post-restart")
    throw_in_post_restart: bool,
    /// `throw-in-post-restart: "once"`: only the first `postRestart` of the label throws
    throw_in_first_post_restart: bool,
    /// `keep-children`: `preRestart` only records its event (neither stops the children nor runs `postStop`)
    keep_children: bool,
    /// `log-sender`: the lifecycle hooks also record `sender()`
    log_sender: bool,
    /// `default-pre-restart`: the actor does not override `preRestart` (see `DefaultPreRestart`)
    default_pre_restart: bool,
    /// `stop-without-unwatch`: `preRestart` stops the children and runs `postStop` without `unwatch`, unlike the default
    stop_without_unwatch: bool,
    /// `pre-restart-throws: <message or null>`: `preRestart` fails before its default part
    pre_restart_throws: Option<Option<String>>,
    /// `post-stop-throws: <message or null>`: `postStop` records its event and then fails
    post_stop_throws: Option<Option<String>>,
    /// `constructor-fails-from: n`: the actor factory throws IllegalStateException("ctor") from the n-th instance on
    constructor_fails_from: Option<u64>,
    ignore_terminated: bool,
    numbered: bool,
}

/// A command the harness asks an actor to run inside its own context (`do` op).
struct Cmd {
    id: u64,
    spec: Value,
}

impl Behavior {
    fn parse(v: &Value) -> Behavior {
        let s = |k: &str| v.get(k).and_then(Value::as_str);
        Behavior {
            kind: s("kind").unwrap_or_else(|| "log".into()),
            handles: v
                .get("handles")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect()),
            to: s("to"),
            msg: s("msg"),
            prefix: s("prefix"),
            post_stop_spawn: s("post-stop-spawn"),
            throw_on: match v.get("throw-on") {
                Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).collect(),
                Some(t) => t.as_str().into_iter().collect(),
                None => Vec::new(),
            },
            supervisor: v.get("supervisor").map(|sp| {
                let max = match sp.get("max") {
                    Some(Value::Number(n)) => *n as i32,
                    _ => -1,
                };
                let decider = match sp.get("decider") {
                    Some(Value::Object(o)) => o
                        .iter()
                        .filter_map(|(k, d)| d.as_str().map(|d| (k.clone(), d)))
                        .collect(),
                    _ => HashMap::new(),
                };
                let hold = sp.get("hold").and_then(Value::as_str);
                SupervisorCfg {
                    max,
                    decider,
                    log_decisions: matches!(sp.get("log-decisions"), Some(Value::Bool(true))),
                    hold,
                    all_for_one: sp.get("strategy").and_then(Value::as_str).as_deref()
                        == Some("all-for-one"),
                    decider_throws: matches!(sp.get("decider-throws"), Some(Value::Bool(true))),
                    logging: !matches!(sp.get("logging"), Some(Value::Bool(false))),
                }
            }),
            throw_in_prestart: matches!(v.get("throw-in-prestart"), Some(Value::Bool(true))),
            throw_in_post_restart: matches!(
                v.get("throw-in-post-restart"),
                Some(Value::Bool(true))
            ),
            throw_in_first_post_restart: v
                .get("throw-in-post-restart")
                .and_then(Value::as_str)
                .as_deref()
                == Some("once"),
            keep_children: matches!(v.get("keep-children"), Some(Value::Bool(true))),
            log_sender: matches!(v.get("log-sender"), Some(Value::Bool(true))),
            default_pre_restart: matches!(v.get("default-pre-restart"), Some(Value::Bool(true))),
            stop_without_unwatch: matches!(v.get("stop-without-unwatch"), Some(Value::Bool(true))),
            pre_restart_throws: v.get("pre-restart-throws").map(Value::as_str),
            post_stop_throws: v.get("post-stop-throws").map(Value::as_str),
            constructor_fails_from: match v.get("constructor-fails-from") {
                Some(Value::Number(n)) => Some(*n as u64),
                _ => None,
            },
            ignore_terminated: matches!(v.get("ignore-terminated"), Some(Value::Bool(true))),
            numbered: matches!(v.get("numbered"), Some(Value::Bool(true))),
            count: match v.get("count") {
                Some(Value::Number(n)) => *n as i64,
                _ => 0,
            },
        }
    }
}

/// A scripted actor: records lifecycle and received messages, then acts according to its behaviour.
struct Scripted {
    label: String,
    cfg: Behavior,
    ctx: Arc<Ctx>,
    /// receives counted by this instance (a restart starts again at 0)
    received: u64,
}

/// `,"strategy":<class>` when a supervisor strategy is the `logClass` of an event, nothing for any other reporter.
fn strategy_of(o: &mut String, log_class: &str) {
    if log_class.ends_with("ForOneStrategy") {
        o.push_str(",\"strategy\":");
        js(o, log_class);
    }
}

/// `throw new IllegalStateException(message)`, where `message` may be null.
fn throw_illegal_state(message: Option<&str>) -> ! {
    throw(Throwable::new("java.lang.IllegalStateException", message))
}

fn reason_str(t: &Throwable) -> String {
    t.to_string()
}

impl Scripted {
    fn logged_sender(&self, c: &ActorContext) -> String {
        let mut field = String::new();
        if self.cfg.log_sender {
            field.push_str(",\"sender\":");
            write_text(&mut field, &path_str(&c.sender()));
        }
        field
    }
}

impl Actor for Scripted {
    fn pre_start(&mut self, _c: &mut ActorContext) {
        self.ctx.add(&self.label, Event::PreStart);
        self.ctx.life(format!("{}:pre-start", self.label));
        if self.cfg.throw_in_prestart {
            throw(Throwable::illegal_state("init"));
        }
    }

    fn supervisor_strategy(&self) -> SupervisorStrategy {
        match &self.cfg.supervisor {
            None => OneForOneStrategy::default_strategy().into(),
            Some(SupervisorCfg {
                max,
                decider,
                log_decisions: log,
                hold,
                all_for_one,
                decider_throws,
                logging,
            }) => {
                let logging = *logging;
                let decider_throws = *decider_throws;
                let (decider, log, ctx, label, hold) = (
                    decider.clone(),
                    *log,
                    self.ctx.clone(),
                    self.label.clone(),
                    hold.clone(),
                );
                let decide = move |e: &Throwable| {
                    if decider_throws {
                        throw(Throwable::illegal_state("decider"));
                    }
                    let chosen = match e
                        .message
                        .as_ref()
                        .and_then(|m| decider.get(m))
                        .map(String::as_str)
                    {
                        Some("resume") => Directive::Resume,
                        Some("restart") => Directive::Restart,
                        Some("stop") => Directive::Stop,
                        Some("escalate") => Directive::Escalate,
                        _ => rakka_actor::actor::supervision::default_decider(e),
                    };
                    if log {
                        ctx.add(
                            &label,
                            Event::Decide {
                                cause: reason_str(e),
                                directive: format!("{chosen:?}"),
                            },
                        );
                    }
                    if let Some(h) = &hold {
                        // the decision stays pending until the harness opens the hold
                        ctx.life(format!("{label}:holding"));
                        let gate = ctx.hold(h);
                        let open = gate.0.lock().unwrap();
                        drop(
                            gate.1
                                .wait_timeout_while(open, Duration::from_secs(30), |o| !*o)
                                .unwrap(),
                        );
                    }
                    chosen
                };
                if *all_for_one {
                    AllForOneStrategy::new(*max, decide)
                        .with_logging_enabled(logging)
                        .into()
                } else {
                    OneForOneStrategy::new(*max, decide)
                        .with_logging_enabled(logging)
                        .into()
                }
            }
        }
    }

    fn pre_restart(
        &mut self,
        c: &mut ActorContext,
        reason: &Throwable,
        message: Option<&AnyMessage>,
    ) {
        self.ctx.add(
            &self.label,
            Event::PreRestart {
                reason: reason_str(reason),
                msg: message.map(msg_of),
                sender: self.logged_sender(c),
            },
        );
        self.ctx.life(format!("{}:pre-restart", self.label));
        if self.cfg.keep_children {
            return;
        }
        if let Some(message) = &self.cfg.pre_restart_throws {
            throw_illegal_state(message.as_deref());
        }
        for child in c.children() {
            if !self.cfg.stop_without_unwatch {
                c.unwatch(&child);
            }
            c.stop(&child);
        }
        self.post_stop(c); // default preRestart, unless `stop-without-unwatch`
    }

    fn post_restart(&mut self, c: &mut ActorContext, reason: &Throwable) {
        self.ctx.add(
            &self.label,
            Event::PostRestart {
                reason: reason_str(reason),
                sender: self.logged_sender(c),
            },
        );
        self.ctx.life(format!("{}:post-restart", self.label));
        let first = self.cfg.throw_in_first_post_restart
            && self
                .ctx
                .failed_post_restart
                .lock()
                .unwrap()
                .insert(self.label.clone());
        if self.cfg.throw_in_post_restart || first {
            throw(Throwable::illegal_state("post-restart"));
        }
        self.pre_start(c); // default postRestart
    }

    fn post_stop(&mut self, c: &mut ActorContext) {
        let sender = self.logged_sender(c);
        self.ctx.add(&self.label, Event::PostStop { sender });
        self.ctx.life(format!("{}:post-stop", self.label));
        if let Some(n) = &self.cfg.post_stop_spawn {
            // creating a child while terminating
            let mut r = String::new();
            match c.actor_of_named(Props::new(|| Noop), n) {
                Ok(k) => {
                    r.push_str("\"ok\":true,");
                    ref_json(&mut r, &k);
                }
                Err(e) => err_json(&mut r, &e),
            }
            self.ctx.add(&self.label, Event::PostStopSpawn(r));
        }
        if let Some(message) = &self.cfg.post_stop_throws {
            throw_illegal_state(message.as_deref());
        }
    }

    fn receive(&mut self, c: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(cmd) = msg.downcast_ref::<Cmd>() {
            let res = command(&self.ctx, c, &cmd.spec);
            self.ctx.results.lock().unwrap().insert(cmd.id, res);
            return true;
        }
        if let Some(t) = msg.downcast_ref::<Terminated>() {
            if self.cfg.ignore_terminated {
                return false; // unhandled: DeathPactException
            }
            let is = self.ctx.label_of(&t.actor);
            self.ctx.add(
                &self.label,
                Event::Terminated {
                    actor: path_str(&t.actor),
                    is: is.clone(),
                    existence_confirmed: t.existence_confirmed,
                    address_terminated: t.address_terminated,
                    sender: path_str(&c.sender()),
                },
            );
            self.ctx.life(format!(
                "{}:terminated:{}",
                self.label,
                is.unwrap_or_else(|| "null".into())
            ));
            return true;
        }
        let m = text(msg);
        if let Some(h) = &self.cfg.handles {
            match &m {
                Some(t) if h.contains(t) => {}
                _ => return false,
            }
        }
        let sender = c.sender();
        self.received += 1;
        let n = self.cfg.numbered.then_some(self.received);
        self.ctx.add(
            &self.label,
            Event::Received {
                msg: msg_of(msg),
                sender: path_str(&sender),
                n,
            },
        );
        self.ctx.order.lock().unwrap().push(format!(
            "{}:{}",
            self.label,
            m.clone().unwrap_or_default()
        ));
        if m.as_ref().is_some_and(|t| self.cfg.throw_on.contains(t)) {
            throw(Throwable::illegal_state(m.as_deref().unwrap_or_default()));
        }
        let target = || {
            self.cfg
                .to
                .as_ref()
                .and_then(|l| self.ctx.refs.lock().unwrap().get(l).cloned())
        };
        match self.cfg.kind.as_str() {
            "echo" => sender.tell_any(msg.clone(), Some(c.self_ref())),
            "forward" => {
                if let Some(t) = target() {
                    c.forward(&t, msg.clone());
                }
            }
            "stop-on" => {
                if m.is_some() && m == self.cfg.msg {
                    let me = c.self_ref();
                    c.stop(&me);
                }
            }
            "blast" => {
                if let Some(t) = target() {
                    let me = c.self_ref();
                    for i in 0..self.cfg.count {
                        t.tell(
                            format!("{}-{i}", self.cfg.prefix.clone().unwrap_or_default()),
                            Some(&me),
                        );
                    }
                }
            }
            "gate" if m.as_deref() == Some("block") => self.ctx.wait_gate(),
            _ => {}
        }
        true
    }
}

/// A scripted actor that leaves `pre_restart` to the trait, so the runtime's own default runs.
struct DefaultPreRestart(Scripted);

impl Actor for DefaultPreRestart {
    fn pre_start(&mut self, c: &mut ActorContext) {
        self.0.pre_start(c);
    }
    fn post_restart(&mut self, c: &mut ActorContext, reason: &Throwable) {
        self.0.post_restart(c, reason);
    }
    fn post_stop(&mut self, c: &mut ActorContext) {
        self.0.post_stop(c);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        self.0.supervisor_strategy()
    }
    fn receive(&mut self, c: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0.receive(c, msg)
    }
}

fn props_for(ctx: &Arc<Ctx>, behavior: Option<&Value>, label: Option<String>) -> Props {
    match behavior {
        None => Props::new(|| Noop),
        Some(b) => {
            let (cfg, ctx2, l) = (
                Behavior::parse(b),
                ctx.clone(),
                label.unwrap_or_else(|| "_".into()),
            );
            let instances = AtomicU64::new(0);
            let default_pre_restart = cfg.default_pre_restart;
            let scripted = move || {
                let instance = instances.fetch_add(1, Ordering::SeqCst) + 1;
                if cfg
                    .constructor_fails_from
                    .is_some_and(|from| instance >= from)
                {
                    throw(Throwable::illegal_state("ctor"));
                }
                Scripted {
                    label: l.clone(),
                    cfg: cfg.clone(),
                    ctx: ctx2.clone(),
                    received: 0,
                }
            };
            if default_pre_restart {
                Props::new(move || DefaultPreRestart(scripted()))
            } else {
                Props::new(scripted)
            }
        }
    }
}

/// A user message type with value equality (`{"custom": name}`), for `watch-with`.
#[derive(PartialEq)]
struct WatchMsg(String);

fn watch_with(c: &mut ActorContext, target: &ActorRef, message: &Value) {
    match message {
        Value::Str(text) => c.watch_with(target, text.clone()),
        Value::Number(number) => c.watch_with(target, *number as i64),
        other => match other.get("custom").and_then(Value::as_str) {
            Some(name) => c.watch_with(target, WatchMsg(name)),
            None => c.watch_with(target, PoisonPill),
        },
    }
}

/// Renders `ok` or the exception a `watch` / `watch-with` threw, uids redacted, as the oracle does.
fn registration(o: &mut String, sp: &Value, register: impl FnOnce()) {
    let Err(thrown) = catch_unwind(AssertUnwindSafe(register)) else {
        o.push_str("\"ok\":true");
        return;
    };
    let thrown = thrown.downcast::<Throwable>().expect("a Throwable");
    o.push_str("\"ok\":false,\"exception\":");
    s(o, &thrown.class);
    if sp.get("no-text").is_none() {
        let text = thrown.message.unwrap_or_default();
        let redacted: Vec<String> = text
            .split(']')
            .map(|part| match part.rsplit_once('#') {
                Some((head, uid)) if uid.parse::<i64>().is_ok() => format!("{head}#uid"),
                _ => part.to_owned(),
            })
            .collect();
        o.push_str(",\"message\":");
        s(o, &redacted.join("]"));
    }
}

/// Runs a `do` command inside the actor and renders the result fields.
fn command(ctx: &Arc<Ctx>, c: &mut ActorContext, sp: &Value) -> String {
    let mut o = String::new();
    let target = |k: &str| {
        sp.get(k)
            .and_then(Value::as_str)
            .and_then(|l| ctx.refs.lock().unwrap().get(&l).cloned())
    };
    let is_json = |o: &mut String, r: &ActorRef| match ctx.label_of(r) {
        Some(l) => {
            o.push_str(",\"is\":");
            s(o, &l);
        }
        None => o.push_str(",\"is\":null"),
    };
    match sp
        .get("c")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .as_str()
    {
        "spawn-child" => {
            let lbl = sp.get("as").and_then(Value::as_str);
            let props = props_for(ctx, sp.get("behavior"), lbl.clone());
            let res = match sp.get("name").and_then(Value::as_str) {
                Some(n) => c.actor_of_named(props, &n),
                None => c.actor_of(props),
            };
            match res {
                Ok(r) => {
                    o.push_str("\"ok\":true,");
                    ref_json(&mut o, &r);
                    if let Some(l) = lbl {
                        ctx.refs.lock().unwrap().insert(l, r);
                    }
                }
                Err(e) => err_json(&mut o, &e),
            }
        }
        "child" => match c.child(&sp.get("name").and_then(Value::as_str).unwrap_or_default()) {
            Some(r) => {
                o.push_str("\"ok\":true,\"found\":true,");
                ref_json(&mut o, &r);
                is_json(&mut o, &r);
            }
            None => o.push_str("\"ok\":true,\"found\":false"),
        },
        "children" => {
            let mut ps: Vec<String> = c.children().iter().map(path_str).collect();
            ps.sort();
            o.push_str("\"ok\":true,\"children\":[");
            for (i, p) in ps.iter().enumerate() {
                if i > 0 {
                    o.push(',');
                }
                s(&mut o, p);
            }
            o.push(']');
        }
        "watch" => {
            let subject = target("target").expect("known ref");
            registration(&mut o, sp, || c.watch(&subject));
        }
        "watch-with" => {
            let subject = target("target").expect("known ref");
            let message = sp.get("msg").expect("a message");
            registration(&mut o, sp, || watch_with(c, &subject, message));
        }
        "unwatch" => {
            c.unwatch(&target("target").expect("known ref"));
            o.push_str("\"ok\":true");
        }
        "stop" => {
            c.stop(&target("target").expect("known ref"));
            o.push_str("\"ok\":true");
        }
        other => o.push_str(&format!(
            "\"ok\":false,\"error\":\"unknown command {other}\""
        )),
    }
    o
}

/// Subscribed to the event stream; records dead letters, unhandled messages and errors.
struct Collector {
    label: String,
    ctx: Arc<Ctx>,
    /// `errors` channel: only the errors whose message starts with this are recorded
    prefix: String,
    /// `errors` channel: an actor the collector also watches
    watch: Option<ActorRef>,
}

impl Actor for Collector {
    fn pre_start(&mut self, c: &mut ActorContext) {
        if let Some(subject) = &self.watch {
            c.watch(subject);
        }
    }

    fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(d) = msg.downcast_ref::<DeadLetter>() {
            self.ctx.add(
                &self.label,
                Event::DeadLetter {
                    msg: msg_of(&d.message),
                    sender: path_str(&d.sender),
                    recipient: path_str(&d.recipient),
                },
            );
            true
        } else if let Some(e) = msg.downcast_ref::<ErrorEvent>() {
            let text = e.message.as_deref().unwrap_or("null");
            if text.starts_with(&self.prefix) {
                let mut o = String::from("{\"ev\":\"error\",\"source\":");
                js(&mut o, &e.log_source);
                o.push_str(",\"message\":");
                match &e.message {
                    Some(message) => js(&mut o, message),
                    None => o.push_str("null"),
                }
                let of_actor_class = e.log_class == std::any::type_name::<Scripted>();
                o.push_str(&format!(",\"ofActorClass\":{of_actor_class},\"cause\":"));
                js(&mut o, &e.cause.class);
                if !e.cause.class.ends_with("NoCause$") {
                    o.push_str(",\"causeText\":");
                    js(&mut o, &e.cause.to_string());
                }
                strategy_of(&mut o, &e.log_class);
                o.push('}');
                self.ctx.add(&self.label, Event::Error(o));
            }
            true
        } else if let Some(w) = msg.downcast_ref::<WarningEvent>() {
            let text = w.message.as_deref().unwrap_or("null");
            if text.starts_with(&self.prefix) {
                let mut o = String::from("{\"ev\":\"warning\",\"source\":");
                js(&mut o, &w.log_source);
                o.push_str(",\"message\":");
                match &w.message {
                    Some(message) => js(&mut o, message),
                    None => o.push_str("null"),
                }
                strategy_of(&mut o, &w.log_class);
                o.push('}');
                self.ctx.add(&self.label, Event::Error(o));
            }
            true
        } else if let Some(t) = msg.downcast_ref::<Terminated>() {
            let actor = path_str(&t.actor); // a path of this system: nothing to escape
            let o = format!("{{\"ev\":\"watched-terminated\",\"actor\":\"{actor}\"}}");
            self.ctx.add(&self.label, Event::Error(o));
            true
        } else if let Some(u) = msg.downcast_ref::<UnhandledMessage>() {
            self.ctx.add(
                &self.label,
                Event::Unhandled {
                    msg: msg_of(&u.message),
                    sender: path_str(&u.sender),
                    recipient: path_str(&u.recipient),
                },
            );
            true
        } else {
            false
        }
    }
}

fn s(out: &mut String, t: &str) {
    write_text(out, t);
}

fn js(out: &mut String, t: &str) {
    write_text(out, t);
}

fn err_json(out: &mut String, e: &ActorError) {
    out.push_str("\"ok\":false,\"exception\":");
    s(out, e.class().java_name());
    out.push_str(",\"message\":");
    write_str(out, &e.message_utf16());
}

fn ref_json(out: &mut String, r: &ActorRef) {
    let p = r.path();
    out.push_str("\"path\":");
    js(out, &p.to_string());
    out.push_str(",\"name\":");
    js(out, p.name());
    out.push_str(",\"parent\":");
    js(out, &p.parent().to_string());
    out.push_str(&format!(",\"hasUid\":{}", p.uid() != 0));
}

/// Polls `cond` until it holds or the timeout elapses; returns whether it holds.
fn wait_for(cond: impl Fn() -> bool) -> bool {
    let end = Instant::now() + await_timeout();
    while !cond() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(2));
    }
    cond()
}

/// (sender, msg) of the string `received` events of an actor, in order.
fn received(ctx: &Ctx, label: &str) -> Vec<(String, String)> {
    ctx.of(label)
        .into_iter()
        .filter_map(|e| match e {
            Event::Received {
                msg: Msg::Text(m),
                sender,
                ..
            } => Some((sender, m)),
            _ => None,
        })
        .collect()
}

fn dead_letters_for(ctx: &Ctx, dl: &str, recipient: &str) -> Vec<String> {
    ctx.of(dl)
        .into_iter()
        .filter_map(|e| match e {
            Event::DeadLetter {
                msg: Msg::Text(m),
                recipient: r,
                ..
            } if r == recipient => Some(m),
            _ => None,
        })
        .collect()
}

/// Runs a scenario and returns the one-line JSON trace.
pub fn run(text: &str) -> Result<String, String> {
    let sc = parse_value(text)?;
    let name = sc
        .get("system")
        .and_then(Value::as_str)
        .ok_or("scenario needs a \"system\" name")?;
    let mut settings = Settings::default();
    if let Some(d) = sc.get("dispatcher") {
        let num = |k: &str| match d.get(k) {
            Some(Value::Number(n)) => Some(*n as usize),
            _ => None,
        };
        settings.parallelism = num("parallelism").unwrap_or(settings.parallelism);
        settings.throughput = num("throughput").unwrap_or(settings.throughput);
    }
    let mut out = String::from("{\"system\":");
    let system = match ActorSystem::create_with_settings(&name, settings) {
        Ok(sys) => sys,
        Err(e) => {
            out.push('{');
            err_json(&mut out, &e);
            out.push_str("},\"steps\":[]}");
            return Ok(out);
        }
    };
    out.push_str("{\"ok\":true,\"name\":");
    s(&mut out, system.name());
    out.push_str(",\"toString\":");
    s(&mut out, &system.to_string());
    for (key, path) in [
        ("root", system.root_path().to_string()),
        ("user", system.guardian().path().to_string()),
        (
            "systemGuardian",
            system.system_guardian().path().to_string(),
        ),
        ("deadLetters", system.dead_letters().path().to_string()),
    ] {
        out.push_str(&format!(",\"{key}\":"));
        js(&mut out, &path);
    }
    out.push_str("},\"steps\":[");

    let ctx = Arc::new(Ctx::default());
    let mut sent: HashMap<String, Vec<String>> = HashMap::new();
    let steps = sc
        .get("steps")
        .and_then(Value::as_array)
        .ok_or("scenario needs \"steps\"")?;
    let mut failure = None;
    for (i, st) in steps.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if let Err(e) = step(&system, &ctx, &mut sent, st, &mut out) {
            failure = Some(e);
            break;
        }
    }
    // best-effort cleanup, like the oracle
    ctx.release();
    system.terminate().wait(await_timeout());
    if let Some(e) = failure {
        return Err(e);
    }
    out.push_str("]}");
    Ok(out)
}

fn lookup(system: &ActorSystem, ctx: &Ctx, label: &str) -> Option<ActorRef> {
    if label == "deadLetters" {
        Some(system.dead_letters())
    } else {
        ctx.refs.lock().unwrap().get(label).cloned()
    }
}

fn message(v: &Value) -> Result<AnyMessage, String> {
    if matches!(v, Value::Object(_))
        && v.get("special").and_then(Value::as_str).as_deref() == Some("PoisonPill")
    {
        return Ok(Arc::new(PoisonPill));
    }
    if matches!(v, Value::Object(_))
        && v.get("special").and_then(Value::as_str).as_deref() == Some("Kill")
    {
        return Ok(Arc::new(Kill));
    }
    if matches!(v, Value::Object(_))
        && v.get("special").and_then(Value::as_str).as_deref() == Some("Identify")
    {
        let correlation = v.get("id").and_then(Value::as_str);
        return Ok(Arc::new(Identify {
            message_id: Arc::new(correlation.ok_or("Identify needs an id")?),
        }));
    }
    Ok(Arc::new(v.as_str().ok_or(
        "message must be a string or {\"special\":\"PoisonPill\"}",
    )?))
}

fn step(
    system: &ActorSystem,
    ctx: &Arc<Ctx>,
    sent: &mut HashMap<String, Vec<String>>,
    st: &Value,
    out: &mut String,
) -> Result<(), String> {
    let op = st
        .get("op")
        .and_then(Value::as_str)
        .ok_or("step needs \"op\"")?;
    out.push_str("{\"op\":");
    s(out, &op);
    let label = |key: &str| st.get(key).and_then(Value::as_str);
    let num = |key: &str| match st.get(key) {
        Some(Value::Number(n)) => *n as i64,
        _ => 0,
    };
    let from = || label("from").and_then(|l| ctx.refs.lock().unwrap().get(&l).cloned());
    match op.as_str() {
        "spawn" => {
            let lbl = label("as");
            let props = props_for(ctx, st.get("behavior"), lbl.clone());
            let result = match st.get("name") {
                Some(Value::Null) => {
                    return Err("null actor names cannot be expressed in Rust".into());
                }
                Some(v) => {
                    system.actor_of_named(props, &v.as_str().ok_or("name must be a string")?)
                }
                None => system.actor_of(props),
            };
            match result {
                Ok(r) => {
                    out.push_str(",\"ok\":true,");
                    ref_json(out, &r);
                    if let Some(l) = lbl {
                        ctx.refs.lock().unwrap().insert(l, r);
                    }
                }
                Err(e) => {
                    out.push(',');
                    err_json(out, &e);
                }
            }
        }
        "subscribe" => {
            let lbl = label("as").ok_or("subscribe needs as")?;
            let (c2, l2) = (ctx.clone(), lbl.clone());
            let prefix = label("prefix").unwrap_or_default();
            let watch = label("watch").and_then(|l| lookup(system, ctx, &l));
            let r = system
                .actor_of(Props::new(move || Collector {
                    label: l2.clone(),
                    ctx: c2.clone(),
                    prefix: prefix.clone(),
                    watch: watch.clone(),
                }))
                .map_err(|e| e.to_string())?;
            ctx.refs.lock().unwrap().insert(lbl, r.clone());
            if label("channel").as_deref() == Some("logs") {
                system.event_stream().subscribe(&r, Channel::Warning);
            }
            let channel = match label("channel").as_deref() {
                Some("dead-letters") => Channel::DeadLetter,
                Some("errors" | "logs") => Channel::Error,
                _ => Channel::UnhandledMessage,
            };
            system.event_stream().subscribe(&r, channel);
            out.push_str(",\"ok\":true");
        }
        "tell" => {
            let target = label("ref").ok_or("tell needs ref")?;
            let msg = message(st.get("msg").ok_or("tell needs msg")?)?;
            sent.entry(target.clone()).or_default();
            if let Some(t) = text(&msg) {
                sent.get_mut(&target).unwrap().push(t);
            }
            lookup(system, ctx, &target)
                .ok_or("unknown ref")?
                .tell_any(msg, from());
            out.push_str(",\"ok\":true");
        }
        "tell-seq" => {
            let target = label("ref").ok_or("tell-seq needs ref")?;
            let r = lookup(system, ctx, &target).ok_or("unknown ref")?;
            let sender = from();
            let entry = sent.entry(target.clone()).or_default();
            for i in 0..num("count") {
                let m = format!("{}-{i}", label("prefix").unwrap_or_default());
                entry.push(m.clone());
                r.tell(m, sender.as_ref());
            }
            out.push_str(",\"ok\":true");
        }
        "release" => {
            ctx.release();
            out.push_str(",\"ok\":true");
        }
        "stop" => {
            let r =
                lookup(system, ctx, &label("ref").ok_or("stop needs ref")?).ok_or("unknown ref")?;
            system.stop(&r);
            out.push_str(",\"ok\":true");
        }
        "await-stopped" => {
            let r = lookup(system, ctx, &label("ref").ok_or("await-stopped needs ref")?)
                .ok_or("unknown ref")?;
            wait_for(|| r.is_terminated());
            out.push_str(&format!(
                ",\"ok\":true,\"terminated\":{}",
                r.is_terminated()
            ));
        }
        "terminate" => {
            system.terminate();
            out.push_str(",\"ok\":true");
        }
        "await-system-terminated" => match system.when_terminated().wait(await_timeout()) {
            Some(Err(failure)) => {
                return Err(format!("the system ended with a failure: {failure}"));
            }
            Some(Ok(t)) => {
                out.push_str(",\"ok\":true,\"actor\":");
                js(out, &t.actor.path().to_string());
                out.push_str(&format!(
                    ",\"existenceConfirmed\":{},\"addressTerminated\":{}",
                    t.existence_confirmed, t.address_terminated
                ));
            }
            None => return Err("timed out waiting for system termination".into()),
        },
        "await-events" => {
            let target = label("target").ok_or("await-events needs target")?;
            let n = num("count") as usize;
            let ok = wait_for(|| ctx.of(&target).len() >= n);
            let ev = ctx.of(&target);
            let mut shown: Vec<String> = ev.iter().take(n).map(Event::json).collect();
            if matches!(st.get("sorted"), Some(Value::Bool(true))) {
                shown.sort(); // events whose order is a race (two Terminated)
            }
            out.push_str(&format!(",\"ok\":{ok},\"events\":[{}]", shown.join(",")));
        }
        "open-hold" => {
            let h = label("hold").ok_or("open-hold needs hold")?;
            ctx.life(format!("hold-open:{h}"));
            let gate = ctx.hold(&h);
            *gate.0.lock().unwrap() = true;
            gate.1.notify_all();
            out.push_str(",\"ok\":true");
        }
        "await-quiet" => {
            let target = label("target").ok_or("await-quiet needs target")?;
            let n = num("count") as usize;
            let end = Instant::now() + Duration::from_millis(num("ms") as u64);
            while ctx.of(&target).len() <= n && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(2));
            }
            let ev = ctx.of(&target);
            let shown: Vec<String> = ev.iter().take(n).map(Event::json).collect();
            out.push_str(&format!(
                ",\"ok\":true,\"quiet\":{},\"events\":[{}]",
                ev.len() <= n,
                shown.join(",")
            ));
        }
        "await-order" => {
            let n = num("count") as usize;
            let ok = wait_for(|| ctx.order.lock().unwrap().len() >= n);
            let ord = ctx.order.lock().unwrap().clone();
            let mut sb = String::new();
            for (i, o) in ord.iter().take(n).enumerate() {
                if i > 0 {
                    sb.push(',');
                }
                s(&mut sb, o);
            }
            out.push_str(&format!(",\"ok\":{ok},\"order\":[{sb}]"));
        }
        "await-received" => {
            let target = label("target").ok_or("await-received needs target")?;
            let total = num("total") as usize;
            let ok = wait_for(|| received(ctx, &target).len() >= total);
            let rcv: Vec<_> = received(ctx, &target).into_iter().take(total).collect();
            let mut per: BTreeMap<String, (i64, bool)> = BTreeMap::new();
            let mut seen = HashSet::new();
            let mut dup = false;
            for (sender, m) in &rcv {
                dup |= !seen.insert((sender.clone(), m.clone()));
                let p = per.entry(sender.clone()).or_insert((0, true));
                let idx: i64 = m
                    .rsplit('-')
                    .next()
                    .and_then(|x| x.parse().ok())
                    .ok_or("await-received needs prefix-<n> messages")?;
                if idx != p.0 {
                    p.1 = false;
                }
                p.0 += 1;
            }
            let mut sb = String::new();
            for (k, (count, in_order)) in &per {
                if !sb.is_empty() {
                    sb.push(',');
                }
                s(&mut sb, k);
                sb.push_str(&format!(":{{\"count\":{count},\"inOrder\":{in_order}}}"));
            }
            out.push_str(&format!(
                ",\"ok\":{ok},\"total\":{},\"duplicates\":{dup},\"senders\":{{{sb}}}",
                rcv.len()
            ));
        }
        "await-accounted" => {
            let target = label("target").ok_or("await-accounted needs target")?;
            let dl = label("dl").ok_or("await-accounted needs dl")?;
            let total = num("total") as usize;
            let recipient = path_str(&lookup(system, ctx, &target).ok_or("unknown ref")?);
            let ok = wait_for(|| {
                received(ctx, &target).len() + dead_letters_for(ctx, &dl, &recipient).len() >= total
            });
            let rcv: Vec<String> = received(ctx, &target).into_iter().map(|(_, m)| m).collect();
            let dls = dead_letters_for(ctx, &dl, &recipient);
            let sent_to = sent.get(&target).cloned().unwrap_or_default();
            let prefix = rcv.len() <= sent_to.len() && rcv[..] == sent_to[..rcv.len()];
            let mut rest: Vec<String> = sent_to
                .get(rcv.len().min(sent_to.len())..)
                .unwrap_or(&[])
                .to_vec();
            rest.sort();
            let mut dsorted = dls.clone();
            dsorted.sort();
            let counts = if matches!(st.get("counts"), Some(Value::Bool(true))) {
                format!(
                    ",\"receivedCount\":{},\"deadLetterCount\":{}",
                    rcv.len(),
                    dls.len()
                )
            } else {
                String::new()
            };
            out.push_str(&format!(
                ",\"ok\":{ok},\"receivedIsPrefixOfSent\":{prefix},\"deadLettersAreTheRest\":{},\"accounted\":{}{counts}",
                rest == dsorted,
                rcv.len() + dls.len() == total
            ));
        }
        "do" => {
            let target =
                lookup(system, ctx, &label("ref").ok_or("do needs ref")?).ok_or("unknown ref")?;
            let id = ctx.cmd_ids.fetch_add(1, Ordering::SeqCst) + 1;
            target.tell(
                Cmd {
                    id,
                    spec: st.get("cmd").cloned().ok_or("do needs cmd")?,
                },
                None,
            );
            if matches!(st.get("async"), Some(Value::Bool(true))) {
                out.push_str(",\"ok\":true,\"queued\":true"); // result is never read
            } else if wait_for(|| ctx.results.lock().unwrap().contains_key(&id)) {
                out.push(',');
                out.push_str(&ctx.results.lock().unwrap()[&id]);
            } else {
                out.push_str(",\"ok\":false,\"timeout\":true");
            }
        }
        "await-child-gone" => {
            let target = lookup(
                system,
                ctx,
                &label("ref").ok_or("await-child-gone needs ref")?,
            )
            .ok_or("unknown ref")?;
            let spec = Value::Object(vec![
                ("c".to_string(), Value::Str("child".to_string())),
                (
                    "name".to_string(),
                    st.get("name")
                        .cloned()
                        .ok_or("await-child-gone needs name")?,
                ),
            ]);
            let gone = wait_for(|| {
                let id = ctx.cmd_ids.fetch_add(1, Ordering::SeqCst) + 1;
                target.tell(
                    Cmd {
                        id,
                        spec: spec.clone(),
                    },
                    None,
                );
                wait_for(|| ctx.results.lock().unwrap().contains_key(&id));
                ctx.results
                    .lock()
                    .unwrap()
                    .get(&id)
                    .is_some_and(|r| r == "\"ok\":true,\"found\":false")
            });
            out.push_str(&format!(",\"ok\":true,\"gone\":{gone}"));
        }
        "events-now" => {
            let target = label("target").ok_or("events-now needs target")?;
            let shown: Vec<String> = ctx.of(&target).iter().map(Event::json).collect();
            out.push_str(&format!(",\"ok\":true,\"events\":[{}]", shown.join(",")));
        }
        "events-sorted" => {
            let target = label("target").ok_or("events-sorted needs target")?;
            let mut shown: Vec<String> = ctx.of(&target).iter().map(Event::json).collect();
            shown.sort();
            out.push_str(&format!(",\"ok\":true,\"events\":[{}]", shown.join(",")));
        }
        "compare-refs" => {
            let a = ctx
                .refs
                .lock()
                .unwrap()
                .get(&label("a").ok_or("compare-refs needs a")?)
                .cloned()
                .ok_or("unknown ref a")?;
            let b = ctx
                .refs
                .lock()
                .unwrap()
                .get(&label("b").ok_or("compare-refs needs b")?)
                .cloned()
                .ok_or("unknown ref b")?;
            let (pa, pb) = (a.path(), b.path());
            out.push_str(&format!(
                ",\"ok\":true,\"pathEqual\":{},\"refEqual\":{},\"uidEqual\":{},\"pathStringEqual\":{},\"aHasUid\":{},\"bHasUid\":{}",
                pa == pb,
                a == b,
                pa.uid() == pb.uid(),
                path_str(&a) == path_str(&b),
                pa.uid() != 0,
                pb.uid() != 0
            ));
        }
        "await-life" => {
            let n = num("count") as usize;
            let ok = wait_for(|| ctx.life.lock().unwrap().len() >= n);
            let snap = ctx.life.lock().unwrap().clone();
            let mut sorted = snap.clone();
            sorted.sort();
            let ev: Vec<String> = sorted
                .iter()
                .map(|e| {
                    let mut t = String::new();
                    s(&mut t, e);
                    t
                })
                .collect();
            let mut cs = Vec::new();
            if let Some(Value::Array(before)) = st.get("before") {
                for pr in before {
                    let names: Vec<String> = pr
                        .as_array()
                        .map(|a| a.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    let i = snap.iter().position(|e| *e == names[0]);
                    let j = snap.iter().position(|e| *e == names[1]);
                    cs.push(matches!((i, j), (Some(i), Some(j)) if i < j).to_string());
                }
            }
            out.push_str(&format!(
                ",\"ok\":{ok},\"events\":[{}],\"before\":[{}]",
                ev.join(","),
                cs.join(",")
            ));
        }
        other => return Err(format!("unknown op {other}")),
    }
    out.push('}');
    Ok(())
}
