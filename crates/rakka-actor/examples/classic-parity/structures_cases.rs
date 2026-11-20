// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/{Index,SubclassifiedIndex,WildcardIndex,MessageBuffer,TokenBucket,BoundedBlockingQueue,StablePriorityQueue,TypedMultiMap,LockUtil,ImmutableIntMap,FrequencySketch,FrequencyList,RecencyList,SegmentedRecencyList,DoubleLinkedList}.scala (Apache-2.0).
//! Cases of the AC-UTIL (structures) oracle (`StructuresOracle.java` and its siblings): the Rust twin of
//! each case. The sequence cases are the scripts of `structures.tsv`, run by an interpreter per
//! family that mirrors the Java one operation by operation; the concurrent cases are coded here and in
//! the oracle and print invariants only. Slice P03.

use crate::xorshift::XorshiftStar;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use rakka_actor::util::bounded_blocking_queue::{
    BackingQueue, BoundedBlockingQueue, QueueEvent, QueueIterator,
};
use rakka_actor::util::clock::Clock;
use rakka_actor::util::double_linked_list::{DoubleLinkedList, LinkTable};
use rakka_actor::util::frequency_list::FrequencyList;
use rakka_actor::util::frequency_sketch::{
    FastFrequencySketch, FrequencySketch, JavaHashCode, StringHasher, bits, murmur3,
};
use rakka_actor::util::immutable_int_map::ImmutableIntMap;
use rakka_actor::util::index::Index;
use rakka_actor::util::message_buffer::{MessageBuffer, MessageBufferMap};
use rakka_actor::util::recency_list::RecencyList;
use rakka_actor::util::segmented_recency_list::SegmentedRecencyList;
use rakka_actor::util::stable_priority_queue::{StablePriorityBlockingQueue, StablePriorityQueue};
use rakka_actor::util::subclassified_index::{Changes, Subclassification, SubclassifiedIndex};
use rakka_actor::util::switch::Switch;
use rakka_actor::util::token_bucket::TokenBucket;
use rakka_actor::util::typed_multi_map::{TypedKey, TypedMultiMap};
use rakka_actor::util::wildcard_index::WildcardIndex;

const SCRIPTS: &str = include_str!("structures.tsv");

// ---------------------------------------------------------------- shared helpers

fn b(x: bool) -> &'static str {
    if x { "t" } else { "f" }
}

fn list<T: std::fmt::Display>(items: impl IntoIterator<Item = T>) -> String {
    let parts: Vec<String> = items.into_iter().map(|i| i.to_string()).collect();
    format!("[{}]", parts.join(","))
}

fn opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |v| format!("some({v})"))
}

/// The token of an operation that threw: `!<Class>[<message>]`.
fn exception(class: &str, message: Option<&str>) -> String {
    format!("!{class}[{}]", message.unwrap_or("null"))
}

type Out = Result<String, String>;

fn ok(text: impl Into<String>) -> Out {
    Ok(text.into())
}

fn parse<T: std::str::FromStr>(text: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    text.parse()
        .unwrap_or_else(|e| panic!("bad number {text}: {e:?}"))
}

trait Interp {
    fn op(&mut self, t: &[&str]) -> Out;
    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String>;
    fn epilogue(&self) -> Vec<Vec<String>>;
}

fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn step(interp: &mut dyn Interp, t: &[&str]) -> String {
    match interp.op(t) {
        Ok(text) | Err(text) => text,
    }
}

fn run_script(interp: &mut dyn Interp, script: &str) -> String {
    let mut out = Vec::new();
    for raw in script.split(';') {
        let text = raw.trim();
        if text.is_empty() {
            continue;
        }
        let t: Vec<&str> = text.split(' ').collect();
        if t[0] == "rand" {
            let mut r = XorshiftStar::new(parse(t[1]));
            for _ in 0..parse::<usize>(t[2]) {
                let op = interp.random(&mut r);
                let refs: Vec<&str> = op.iter().map(String::as_str).collect();
                out.push(step(interp, &refs));
            }
            for op in interp.epilogue() {
                let refs: Vec<&str> = op.iter().map(String::as_str).collect();
                out.push(step(interp, &refs));
            }
        } else {
            out.push(step(interp, &t));
        }
    }
    out.join(" ")
}

fn interpreter(family: &str) -> Box<dyn Interp> {
    match family {
        "index" => Box::new(IndexInterp::new(false)),
        "index-reversed" => Box::new(IndexInterp::new(true)),
        "int-map" => Box::new(IntMapInterp::default()),
        "typed-multimap" => Box::new(TypedMultiMapInterp::default()),
        "wildcard" => Box::new(WildcardInterp::default()),
        "subclassified" => Box::new(SubclassifiedInterp::new()),
        "message-buffer" => Box::new(MessageBufferInterp::default()),
        "switch" => Box::new(SwitchInterp::default()),
        "token-bucket" => Box::new(TokenBucketInterp::default()),
        "stable-queue" => Box::new(StableInterp::new(false)),
        "stable-blocking-queue" => Box::new(StableInterp::new(true)),
        "bounded-queue" => Box::new(BoundedInterp::default()),
        "sketch" => Box::new(SketchInterp::default()),
        "recency" => Box::new(RecencyInterp::new()),
        "frequency" => Box::new(FrequencyInterp::new()),
        "segmented" => Box::new(SegmentedInterp::new()),
        "double-list" => Box::new(DoubleListInterp::default()),
        other => panic!("unknown family {other}"),
    }
}

/// The lines of the structures oracle, in the order of `structures.tsv` and then the concurrent
/// cases: `(id, result)` where the result is what follows `ok ` in the oracle line.
pub fn cases() -> Vec<(String, String)> {
    // A panicking action is part of the cases: keep it off the standard error.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut out = Vec::new();
    for line in SCRIPTS.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let (id, family, script) = (
            fields.next().expect("an id"),
            fields.next().expect("a family"),
            fields.next().expect("a script"),
        );
        out.push((id.to_owned(), run_script(&mut *interpreter(family), script)));
    }
    panic::set_hook(previous_hook);
    out.extend(concurrent_cases());
    // `ReentrantGuard` (AC-UTIL-015.2) no longer exists in 2.0.0 and never existed here: `Mutex` replaces it.
    out.push((
        "RG-01 ReentrantGuard is absent from the 2.0.0 jar".to_owned(),
        "class=absent".to_owned(),
    ));
    out
}

// ---------------------------------------------------------------- Index

struct IndexInterp {
    index: Index<String, i32>,
}

impl IndexInterp {
    fn new(reversed: bool) -> Self {
        let index = if reversed {
            Index::new(100, |a: &i32, b: &i32| b.cmp(a))
        } else {
            Index::new(100, |a: &i32, b: &i32| a.cmp(b))
        };
        Self { index }
    }
}

impl Interp for IndexInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let key = || t[1].to_owned();
        match t[0] {
            "put" => ok(b(self.index.put(key(), parse(t[2])))),
            "rm" => ok(b(self.index.remove(&key(), &parse(t[2])))),
            "rmk" => ok(self.index.remove_key(&key()).map_or_else(
                || "none".to_owned(),
                |values| format!("some{}", list(values)),
            )),
            "rmv" => {
                self.index.remove_value(&parse(t[1]));
                ok("ok")
            }
            "vals" => ok(list(self.index.value_iterator(&key()))),
            "find" => {
                let n: i32 = parse(t[2]);
                ok(opt(self.index.find_value(&key(), |v| *v >= n)))
            }
            "allv" => {
                let mut values: Vec<i32> = self.index.values().into_iter().collect();
                values.sort_unstable();
                ok(list(values))
            }
            "keys" => {
                let mut keys = self.index.keys();
                keys.sort();
                ok(list(keys))
            }
            "empty" => ok(b(self.index.is_empty())),
            "clear" => {
                self.index.clear();
                ok("ok")
            }
            "each" => {
                let mut seen: BTreeMap<String, Vec<i32>> = BTreeMap::new();
                self.index
                    .foreach(|k, v| seen.entry(k.clone()).or_default().push(*v));
                let items: Vec<String> = seen
                    .iter()
                    .map(|(k, v)| format!("{k}={}", list(v)))
                    .collect();
                ok(format!("{{{}}}", items.join(" ")))
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let key = format!("k{}", r.below(4));
        let value = r.below(6).to_string();
        match r.below(6) {
            0 => vec!["put".into(), key, value],
            1 => vec!["rm".into(), key, value],
            2 => vec!["rmk".into(), key],
            3 => vec!["rmv".into(), value],
            4 => vec!["vals".into(), key],
            _ => vec!["find".into(), key, value],
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["each"]),
            words(&["allv"]),
            words(&["keys"]),
            words(&["empty"]),
        ]
    }
}

// ---------------------------------------------------------------- ImmutableIntMap

const INT_KEYS: [i32; 9] = [i32::MIN, -5, -1, 0, 1, 2, 7, 100, i32::MAX];

#[derive(Default)]
struct IntMapInterp {
    map: ImmutableIntMap,
    marks: [ImmutableIntMap; 4],
}

impl Interp for IntMapInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "u" => {
                self.map = self.map.updated(parse(t[1]), parse(t[2]));
                ok("ok")
            }
            "ua" => {
                let mut evaluated = false;
                let value: i32 = parse(t[2]);
                let next = self.map.update_if_absent(parse(t[1]), || {
                    evaluated = true;
                    value
                });
                let same = next == self.map;
                self.map = next;
                ok(format!(
                    "{}/{}",
                    if evaluated { "evaluated" } else { "lazy" },
                    if same { "same" } else { "new" }
                ))
            }
            "rm" => {
                let next = self.map.remove(parse(t[1]));
                let same = next == self.map;
                self.map = next;
                ok(if same { "same" } else { "new" })
            }
            "get" => ok(self.map.get(parse(t[1])).to_string()),
            "has" => ok(b(self.map.contains(parse(t[1])))),
            "keys" => ok(list(self.map.keys_iterator())),
            "str" => ok(self.map.to_string()),
            "hash" => ok(self.map.hash_code().to_string()),
            "size" => ok(self.map.size().to_string()),
            "mark" => {
                self.marks[parse::<usize>(t[1])] = self.map.clone();
                ok("ok")
            }
            "eqm" => {
                let other = &self.marks[parse::<usize>(t[1])];
                ok(format!(
                    "eq={} hashEq={}",
                    b(self.map == *other),
                    b(self.map.hash_code() == other.hash_code())
                ))
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let key = INT_KEYS[r.below(INT_KEYS.len())].to_string();
        let value = (r.below(5) as i32 - 2).to_string();
        match r.below(7) {
            0 | 1 => vec!["u".into(), key, value],
            2 => vec!["ua".into(), key, value],
            3 => vec!["rm".into(), key],
            4 => vec!["get".into(), key],
            5 => vec!["has".into(), key],
            _ => vec!["hash".into()],
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["keys"]),
            words(&["str"]),
            words(&["size"]),
            words(&["hash"]),
        ]
    }
}

// ---------------------------------------------------------------- TypedMultiMap

#[derive(Clone, PartialEq, Eq, Hash)]
struct Name(String);

impl TypedKey for Name {
    type Value = i32;
}

#[derive(Default)]
struct TypedMultiMapInterp {
    map: TypedMultiMap<Name>,
    mark: TypedMultiMap<Name>,
}

fn ints(csv: &str) -> HashSet<i32> {
    if csv == "-" {
        HashSet::new()
    } else {
        csv.split(',').map(parse).collect()
    }
}

fn sorted_ints(values: impl IntoIterator<Item = i32>) -> String {
    let mut values: Vec<i32> = values.into_iter().collect();
    values.sort_unstable();
    list(values)
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl Interp for TypedMultiMapInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let key = || Name(t[1].to_owned());
        match t[0] {
            "ins" => {
                self.map = self.map.inserted(key(), parse(t[2]));
                ok("ok")
            }
            "rm" => {
                self.map = self.map.removed(&key(), &parse(t[2]));
                ok("ok")
            }
            "vrm" => {
                self.map = self.map.value_removed(&parse(t[1]));
                ok("ok")
            }
            "krm" => {
                self.map = self.map.key_removed(&key());
                ok("ok")
            }
            "setall" => {
                self.map = self.map.set_all(key(), ints(t[2]));
                ok("ok")
            }
            "get" => ok(sorted_ints(self.map.get(&key()))),
            "keys" => {
                let mut keys: Vec<String> = self.map.key_set().into_iter().map(|k| k.0).collect();
                keys.sort();
                ok(list(keys))
            }
            "mark" => {
                self.mark = self.map.clone();
                ok("ok")
            }
            "plus" => {
                self.map = self.map.plus(&self.mark);
                ok("ok")
            }
            "eqm" => ok(format!(
                "eq={} hashEq={}",
                b(self.map == self.mark),
                b(hash_of(&self.map) == hash_of(&self.mark))
            )),
            "reset" => {
                self.map = TypedMultiMap::empty();
                ok("ok")
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let key = format!("k{}", r.below(3));
        let value = r.below(4).to_string();
        match r.below(9) {
            0..=2 => vec!["ins".into(), key, value],
            3 => vec!["rm".into(), key, value],
            4 => vec!["vrm".into(), value],
            5 => vec!["krm".into(), key],
            6 => {
                let values = if r.below(2) == 0 {
                    "-".to_owned()
                } else {
                    format!("{value},{}", r.below(4))
                };
                vec!["setall".into(), key, values]
            }
            7 => vec!["get".into(), key],
            _ => vec![if r.below(3) == 0 { "mark" } else { "plus" }.into()],
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["keys"]),
            words(&["get", "k0"]),
            words(&["get", "k1"]),
            words(&["get", "k2"]),
            words(&["eqm"]),
        ]
    }
}

// ---------------------------------------------------------------- WildcardIndex

fn path(text: &str) -> Vec<&str> {
    if text == "-" {
        Vec::new()
    } else {
        text.split('/').collect()
    }
}

#[derive(Default)]
struct WildcardInterp {
    index: WildcardIndex<i32>,
}

const WILDCARD_ELEMENTS: [&str; 10] = ["a", "ab", "b", "c", "x", "*", "**", "a*", "b*", "c*"];
const WILDCARD_CONCRETE: [&str; 9] = ["a", "ab", "abc", "b", "bc", "c", "cd", "x", "y"];

impl Interp for WildcardInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "ins" => match self.index.insert(&path(t[1]), parse(t[2])) {
                Ok(index) => {
                    self.index = index;
                    ok("ok")
                }
                Err(e) => Err(exception(e.java_class(), Some(e.message()))),
            },
            "find" => ok(opt(self.index.find(path(t[1])))),
            "empty" => ok(b(self.index.is_empty())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        if r.below(3) == 0 {
            let len = r.below(4);
            let inserted = if len == 0 {
                "-".to_owned()
            } else {
                (0..len)
                    .map(|_| WILDCARD_ELEMENTS[r.below(WILDCARD_ELEMENTS.len())])
                    .collect::<Vec<_>>()
                    .join("/")
            };
            let value = r.below(9).to_string();
            return vec!["ins".into(), inserted, value];
        }
        let len = r.below(4);
        if len == 0 {
            return words(&["find", "-"]);
        }
        let found = (0..len)
            .map(|_| WILDCARD_CONCRETE[r.below(WILDCARD_CONCRETE.len())])
            .collect::<Vec<_>>()
            .join("/");
        vec!["find".into(), found]
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["empty"])]
    }
}

// ---------------------------------------------------------------- SubclassifiedIndex

struct Divisibility;

impl Subclassification<u32> for Divisibility {
    fn is_equal(&self, x: &u32, y: &u32) -> bool {
        x == y
    }

    fn is_subclass(&self, x: &u32, y: &u32) -> bool {
        x.is_multiple_of(*y)
    }
}

const SUBCLASS_KEYS: [u32; 15] = [1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 18, 24, 30, 36];

struct SubclassifiedInterp {
    index: SubclassifiedIndex<u32, String, Divisibility>,
}

impl SubclassifiedInterp {
    fn new() -> Self {
        Self {
            index: SubclassifiedIndex::new(Divisibility),
        }
    }
}

fn sorted_strings(values: impl IntoIterator<Item = String>) -> String {
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort();
    list(values)
}

fn changes(changes: Changes<u32, String>) -> String {
    let mut sorted: BTreeMap<u32, String> = BTreeMap::new();
    for (key, values) in changes {
        assert!(
            sorted.insert(key, sorted_strings(values)).is_none(),
            "duplicate key in changes {key}"
        );
    }
    let items: Vec<String> = sorted.iter().map(|(k, v)| format!("{k}:{v}")).collect();
    format!("{{{}}}", items.join(" "))
}

impl Interp for SubclassifiedInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "addKey" => ok(changes(self.index.add_key(&parse(t[1])))),
            "addValue" => ok(changes(
                self.index.add_value(&parse(t[1]), &t[2].to_owned()),
            )),
            "rmValue" => ok(changes(
                self.index.remove_value_of(&parse(t[1]), &t[2].to_owned()),
            )),
            "rmAll" => ok(changes(self.index.remove_value(&t[1].to_owned()))),
            "find" => ok(sorted_strings(self.index.find_values(&parse(t[1])))),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let key = SUBCLASS_KEYS[r.below(SUBCLASS_KEYS.len())].to_string();
        let value = format!("v{}", r.below(3));
        match r.below(7) {
            0 => vec!["addKey".into(), key],
            1..=3 => vec!["addValue".into(), key, value],
            4 => vec!["rmValue".into(), key, value],
            5 => vec!["rmAll".into(), value],
            _ => vec!["find".into(), key],
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["find", "1"]),
            words(&["find", "6"]),
            words(&["find", "36"]),
            words(&["find", "5"]),
        ]
    }
}

// ---------------------------------------------------------------- MessageBuffer

#[derive(Default)]
struct MessageBufferInterp {
    buffer: MessageBuffer<String, String>,
    map: MessageBufferMap<String, String, String>,
}

fn render_buffer(buffer: &MessageBuffer<String, String>) -> String {
    let mut items = Vec::new();
    buffer.foreach(|m, s| items.push(format!("{m}->{s}")));
    list(items)
}

impl Interp for MessageBufferInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let id = || t[1].to_owned();
        match t[0] {
            "app" => {
                self.buffer.append(t[1].to_owned(), t[2].to_owned());
                ok(format!("size={}", self.buffer.size()))
            }
            "dh" => {
                self.buffer.drop_head();
                ok(format!("size={}", self.buffer.size()))
            }
            "head" | "gethead" => ok(self
                .buffer
                .head()
                .map_or_else(|| "(null,null)".to_owned(), |(m, s)| format!("({m},{s})"))),
            "each" => ok(render_buffer(&self.buffer)),
            "fn" => {
                let drop: Vec<&str> = t[1].split(',').collect();
                self.buffer.filter_not(|m, _| drop.contains(&m.as_str()));
                ok(format!("size={}", self.buffer.size()))
            }
            "size" => ok(self.buffer.size().to_string()),
            "empty" => ok(format!(
                "{}{}",
                b(self.buffer.is_empty()),
                b(self.buffer.non_empty())
            )),
            "ma" => {
                self.map.add(id());
                ok("ok")
            }
            "mp" => {
                self.map.append(id(), t[2].to_owned(), t[3].to_owned());
                ok("ok")
            }
            "mr" => {
                self.map.remove(&id());
                ok("ok")
            }
            "mc" => ok(b(self.map.contains(&id()))),
            "mg" => ok(render_buffer(self.map.get_or_empty(&id()))),
            "mt" => ok(self.map.total_size().to_string()),
            "ms" => ok(format!(
                "{}{}{}",
                self.map.size(),
                b(self.map.is_empty()),
                b(self.map.non_empty())
            )),
            "md" => {
                let mut sent = Vec::new();
                let dropped = self.map.drop(&id(), t[2], |d| {
                    sent.push(format!("{}/{}/{}/noSender", d.message, d.reason, d.sender));
                });
                ok(format!("dropped={dropped} sent={}", list(sent)))
            }
            "me" => {
                let mut seen: BTreeMap<String, String> = BTreeMap::new();
                self.map.foreach(|id, buffer| {
                    seen.insert(id.clone(), render_buffer(buffer));
                });
                let items: Vec<String> = seen.iter().map(|(k, v)| format!("{k}={v}")).collect();
                ok(format!("{{{}}}", items.join(", ")))
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let id = format!("id{}", r.below(3));
        let m = format!("m{}", r.below(6));
        let s = format!("s{}", r.below(3));
        match r.below(11) {
            0 | 1 => vec!["app".into(), m, s],
            2 => words(&["dh"]),
            3 => {
                let second = r.below(6);
                vec!["fn".into(), format!("{m},m{second}")]
            }
            4 | 5 => vec!["mp".into(), id, m, s],
            6 => vec!["mr".into(), id],
            7 => vec!["ma".into(), id],
            8 => vec!["mg".into(), id],
            9 => {
                if self.map.contains(&id) {
                    vec!["md".into(), id, "why".into()]
                } else {
                    vec!["mg".into(), id]
                }
            }
            _ => words(&["head"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["each"]),
            words(&["size"]),
            words(&["mt"]),
            words(&["ms"]),
            words(&["me"]),
        ]
    }
}

// ---------------------------------------------------------------- Switch

struct SwitchInterp {
    switch: Switch,
    log: RefCell<Vec<String>>,
}

impl Default for SwitchInterp {
    fn default() -> Self {
        Self {
            switch: Switch::new(false),
            log: RefCell::new(Vec::new()),
        }
    }
}

impl SwitchInterp {
    fn note(&self, what: &str) {
        self.log.borrow_mut().push(what.to_owned());
    }

    fn boom(&self) {
        self.note("boom");
        panic!("boom");
    }

    fn threw(outcome: std::thread::Result<bool>) -> String {
        match outcome {
            Ok(_) => "returned".to_owned(),
            Err(payload) => format!(
                "threw:{}",
                payload.downcast_ref::<&str>().copied().unwrap_or("?")
            ),
        }
    }

    fn yielding(&self, value: &'static str) -> &'static str {
        self.note("y");
        value
    }
}

const SWITCH_OPS: [&str; 19] = [
    "on", "off", "onA", "offA", "onX", "offX", "ifon", "ifoff", "ifonY", "ifoffY", "whon", "whoff",
    "whonY", "whoffY", "fold", "locked", "lockedOn", "ison", "isoff",
];

impl Interp for SwitchInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        if t[0] == "start" {
            self.switch = Switch::new(t[1] == "1");
            return ok("ok");
        }
        let s = &self.switch;
        match t[0] {
            "on" => ok(b(s.switch_on())),
            "off" => ok(b(s.switch_off())),
            "onA" => ok(b(s.switch_on_with(|| self.note("on")))),
            "offA" => ok(b(s.switch_off_with(|| self.note("off")))),
            "onX" => ok(Self::threw(panic::catch_unwind(AssertUnwindSafe(|| {
                s.switch_on_with(|| self.boom())
            })))),
            "offX" => ok(Self::threw(panic::catch_unwind(AssertUnwindSafe(|| {
                s.switch_off_with(|| self.boom())
            })))),
            "ifon" => ok(b(s.if_on(|| self.note("ifon")))),
            "ifoff" => ok(b(s.if_off(|| self.note("ifoff")))),
            "ifonY" => ok(opt(s.if_on_yield(|| self.yielding("yes")))),
            "ifoffY" => ok(opt(s.if_off_yield(|| self.yielding("yes")))),
            "whon" => ok(b(s.while_on(|| self.note("whon")))),
            "whoff" => ok(b(s.while_off(|| self.note("whoff")))),
            "whonY" => ok(opt(s.while_on_yield(|| self.yielding("yes")))),
            "whoffY" => ok(opt(s.while_off_yield(|| self.yielding("yes")))),
            "fold" => ok(s.fold(|| self.yielding("on"), || self.yielding("off"))),
            "locked" => ok(s.locked(|| self.yielding("locked"))),
            "lockedOn" => ok(s.locked(|| {
                format!(
                    "inner={}{}{}",
                    b(s.switch_on()),
                    b(s.while_on(|| self.note("nested"))),
                    b(s.is_on())
                )
            })),
            "ison" => ok(b(s.is_on())),
            "isoff" => ok(b(s.is_off())),
            "log" => {
                let shown = list(self.log.borrow().iter());
                self.log.borrow_mut().clear();
                ok(shown)
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        vec![SWITCH_OPS[r.below(SWITCH_OPS.len())].to_owned()]
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["ison"]), words(&["log"])]
    }
}

// ---------------------------------------------------------------- TokenBucket

type Bucket = TokenBucket<Box<dyn Fn() -> i64>>;

struct TokenBucketInterp {
    bucket: Option<Bucket>,
    now: Rc<Cell<i64>>,
    pending_time: i64,
    started: bool,
}

impl Default for TokenBucketInterp {
    fn default() -> Self {
        Self {
            bucket: None,
            now: Rc::new(Cell::new(0)),
            pending_time: 0,
            started: false,
        }
    }
}

impl TokenBucketInterp {
    fn make(&mut self, capacity: i64, period: i64, now: i64) -> Result<(), String> {
        let clock = Rc::new(Cell::new(now));
        let reader = clock.clone();
        let bucket = TokenBucket::new(
            capacity,
            period,
            Box::new(move || reader.get()) as Box<dyn Fn() -> i64>,
        )
        .map_err(|e| exception(e.java_class(), Some(e.message())))?;
        self.now = clock;
        self.bucket = Some(bucket);
        Ok(())
    }
}

const BUCKET_CAPACITIES: [i64; 6] = [0, 1, 2, 5, 10, 100];
const BUCKET_PERIODS: [i64; 5] = [1, 2, 3, 7, i64::MAX >> 10];
const BUCKET_STARTS: [i64; 7] = [0, 1, -100, i64::MIN, i64::MAX, i64::MAX - 5, 123_456_789];
const BUCKET_STEPS: [i64; 8] = [0, 1, 2, 3, 5, 20, 100, 1_000_000];
const BUCKET_COSTS: [i64; 6] = [0, 1, 2, 5, 10, 20];

impl Interp for TokenBucketInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "new" => {
                let now = self.pending_time;
                self.make(parse(t[1]), parse(t[2]), now)?;
                ok("ok")
            }
            "time" => {
                self.pending_time = parse(t[1]);
                self.now.set(self.pending_time);
                ok("ok")
            }
            "init" => {
                self.bucket.as_mut().expect("a bucket").init();
                ok("ok")
            }
            "setup" => {
                self.make(parse(t[1]), parse(t[2]), parse(t[3]))?;
                self.bucket.as_mut().expect("a bucket").init();
                ok("ok")
            }
            "add" => {
                self.now.set(self.now.get().wrapping_add(parse(t[1])));
                ok("ok")
            }
            "o" => self
                .bucket
                .as_mut()
                .expect("a bucket")
                .offer(parse(t[1]))
                .map(|delay| delay.to_string())
                .map_err(|e| exception(e.java_class(), Some(e.message()))),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        if !self.started {
            self.started = true;
            return vec![
                "setup".into(),
                BUCKET_CAPACITIES[r.below(BUCKET_CAPACITIES.len())].to_string(),
                BUCKET_PERIODS[r.below(BUCKET_PERIODS.len())].to_string(),
                BUCKET_STARTS[r.below(BUCKET_STARTS.len())].to_string(),
            ];
        }
        if r.below(3) == 0 {
            return vec![
                "add".into(),
                BUCKET_STEPS[r.below(BUCKET_STEPS.len())].to_string(),
            ];
        }
        vec![
            "o".into(),
            BUCKET_COSTS[r.below(BUCKET_COSTS.len())].to_string(),
        ]
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        Vec::new()
    }
}

// ---------------------------------------------------------------- StablePriorityQueue

enum Stable {
    Plain(StablePriorityQueue<(i32, String)>),
    Blocking(StablePriorityBlockingQueue<(i32, String)>),
}

struct StableInterp {
    queue: Stable,
    sequence: usize,
}

impl StableInterp {
    fn new(blocking: bool) -> Self {
        let cmp = |a: &(i32, String), b: &(i32, String)| a.0.cmp(&b.0);
        let queue = if blocking {
            Stable::Blocking(StablePriorityBlockingQueue::new(11, cmp))
        } else {
            Stable::Plain(StablePriorityQueue::new(11, cmp))
        };
        Self { queue, sequence: 0 }
    }

    fn label(item: Option<(i32, String)>) -> String {
        item.map_or_else(|| "null".to_owned(), |(_, label)| label)
    }

    fn poll(&mut self) -> Option<(i32, String)> {
        match &mut self.queue {
            Stable::Plain(q) => q.poll(),
            Stable::Blocking(q) => q.poll(),
        }
    }
}

impl Interp for StableInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "offer" => {
                let item = (parse::<i32>(t[1]), t[2].to_owned());
                ok(b(match &mut self.queue {
                    Stable::Plain(q) => q.offer(item),
                    Stable::Blocking(q) => q.offer(item),
                }))
            }
            "poll" => {
                let polled = self.poll();
                ok(Self::label(polled))
            }
            "peek" => match &self.queue {
                Stable::Plain(q) => ok(q
                    .peek()
                    .map_or_else(|| "null".to_owned(), |(_, l)| l.clone())),
                Stable::Blocking(_) => panic!("peek is not part of the blocking twin"),
            },
            "size" => ok(match &self.queue {
                Stable::Plain(q) => q.len(),
                Stable::Blocking(q) => q.len(),
            }
            .to_string()),
            "drain" => {
                let mut out = Vec::new();
                while let Some((_, label)) = self.poll() {
                    out.push(label);
                }
                ok(list(out))
            }
            "iterall" => match &self.queue {
                Stable::Plain(q) => {
                    let mut labels: Vec<String> = q.iter().map(|(_, l)| l.clone()).collect();
                    labels.sort();
                    ok(list(labels))
                }
                Stable::Blocking(_) => panic!("iteration is not part of the blocking twin"),
            },
            "iterrm" => match &mut self.queue {
                Stable::Plain(q) => {
                    let label = q.iter().nth(parse(t[1])).expect("an element").1.clone();
                    q.retain(|(_, l)| *l != label);
                    ok(format!("size={}", q.len()))
                }
                Stable::Blocking(_) => panic!("iteration is not part of the blocking twin"),
            },
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        match r.below(3) {
            0 | 1 => {
                let prio = r.below(5).to_string();
                self.sequence += 1;
                vec!["offer".into(), prio, format!("e{}", self.sequence - 1)]
            }
            _ => words(&["poll"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["size"]), words(&["drain"])]
    }
}

// ---------------------------------------------------------------- BoundedBlockingQueue

type Item = Arc<String>;

/// A `LinkedBlockingQueue(limit)`: refuses a `maxCapacity` above its limit.
struct Limited(VecDeque<Item>, usize);

impl BackingQueue<Item> for Limited {
    fn offer(&mut self, element: Item) -> bool {
        self.0.offer(element)
    }
    fn poll(&mut self) -> Option<Item> {
        self.0.poll()
    }
    fn peek(&self) -> Option<&Item> {
        self.0.peek()
    }
    fn len(&self) -> usize {
        BackingQueue::len(&self.0)
    }
    fn remaining_capacity(&self) -> usize {
        self.1 - BackingQueue::len(&self.0)
    }
    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&Item) -> bool) -> bool {
        self.0.remove_first_where(matches)
    }
    fn retain_where(&mut self, keep: &mut dyn FnMut(&Item) -> bool) -> bool {
        self.0.retain_where(keep)
    }
    fn clear(&mut self) {
        BackingQueue::clear(&mut self.0);
    }
    fn snapshot(&self) -> Vec<Item> {
        self.0.snapshot()
    }
}

fn item(text: &str) -> Item {
    Arc::new(text.to_owned())
}

fn new_bounded(capacity: i32, kind: &str) -> Result<BoundedBlockingQueue<Item>, String> {
    let rejected =
        |e: rakka_actor::actor::IllegalArgumentError| exception(e.java_class(), Some(e.message()));
    match kind {
        "linked" => BoundedBlockingQueue::new(capacity, VecDeque::<Item>::new()).map_err(rejected),
        "priority" => BoundedBlockingQueue::new(
            capacity,
            StablePriorityQueue::new(11, |a: &Item, b: &Item| a.cmp(b)),
        )
        .map_err(rejected),
        "blocking2" => {
            BoundedBlockingQueue::new(capacity, Limited(VecDeque::new(), 2)).map_err(rejected)
        }
        "full" => {
            let mut full = VecDeque::new();
            full.push_back(item("Hello"));
            BoundedBlockingQueue::new(capacity, full).map_err(rejected)
        }
        other => panic!("unknown backing {other}"),
    }
}

fn csv_items(csv: &str) -> Vec<Item> {
    if csv == "-" {
        Vec::new()
    } else {
        csv.split(',').map(item).collect()
    }
}

fn show_items(items: impl IntoIterator<Item = Item>) -> String {
    list(items.into_iter().map(|i| i.to_string()))
}

#[derive(Default)]
struct BoundedInterp {
    queue: Option<BoundedBlockingQueue<Item>>,
}

const BOUNDED_ITEMS: [&str; 5] = ["a", "b", "c", "d", "e"];

fn remove_error(error: &rakka_actor::util::bounded_blocking_queue::IteratorRemoveError) -> String {
    exception(error.java_class(), error.message())
}

fn iterator_next(it: &mut QueueIterator<'_, Item>) -> Result<Item, String> {
    it.next()
        .ok_or_else(|| exception("NoSuchElementException", None))
}

impl Interp for BoundedInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        if t[0] == "new" {
            self.queue = Some(new_bounded(parse(t[1]), t[2])?);
            return ok("ok");
        }
        let q = self.queue.as_ref().expect("a queue");
        match t[0] {
            "offer" => ok(b(q.offer(item(t[1])))),
            "put" => {
                q.put(item(t[1]));
                ok("ok")
            }
            "poll" => ok(q
                .poll()
                .map_or_else(|| "null".to_owned(), |i| i.to_string())),
            "take" => ok(q.take().to_string()),
            "peek" => ok(q
                .peek()
                .map_or_else(|| "null".to_owned(), |i| i.to_string())),
            "rem" => ok(b(q.remove(&item(t[1])))),
            "has" => ok(b(q.contains(&item(t[1])))),
            "rc" => ok(q.remaining_capacity().to_string()),
            "size" => ok(q.len().to_string()),
            "empty" => ok(b(q.is_empty())),
            "clear" => {
                q.clear();
                ok("ok")
            }
            "drain" => {
                let mut sink = Vec::new();
                let max: i64 = if t[1] == "all" { i64::MAX } else { parse(t[1]) };
                let n = q.drain_to(&mut sink, usize::try_from(max).unwrap_or(0));
                ok(format!("{n}{}", show_items(sink)))
            }
            "containsAll" => ok(b(q.contains_all(&csv_items(t[1])))),
            "removeAll" => ok(b(q.remove_all(&csv_items(t[1])))),
            "retainAll" => ok(b(q.retain_all(&csv_items(t[1])))),
            "array" => ok(show_items(q.to_vec())),
            "itall" => ok(show_items(q.iter())),
            "itrm" => {
                let mut it = q.iter();
                for _ in 0..parse::<usize>(t[1]) {
                    iterator_next(&mut it)?;
                }
                it.remove().map_err(|e| remove_error(&e))?;
                ok("removed")
            }
            "itrm2" => {
                let mut it = q.iter();
                iterator_next(&mut it)?;
                it.remove().map_err(|e| remove_error(&e))?;
                it.remove().map_err(|e| remove_error(&e))?;
                ok("removed twice")
            }
            "itnext" => {
                let mut it = q.iter();
                while it.next().is_some() {}
                iterator_next(&mut it)?;
                ok("next")
            }
            "offerT" => ok(b(q.offer_timeout(item(t[1]), Duration::from_millis(1)))),
            "pollT" => ok(q
                .poll_timeout(Duration::from_millis(1))
                .map_or_else(|| "null".to_owned(), |i| i.to_string())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let x = BOUNDED_ITEMS[r.below(BOUNDED_ITEMS.len())].to_owned();
        match r.below(14) {
            0..=2 => vec!["offer".into(), x],
            3 | 4 => words(&["poll"]),
            5 => words(&["peek"]),
            6 => vec!["rem".into(), x],
            7 => vec!["has".into(), x],
            8 => vec!["drain".into(), (1 + r.below(3)).to_string()],
            9 => {
                let other = BOUNDED_ITEMS[r.below(BOUNDED_ITEMS.len())];
                vec!["removeAll".into(), format!("{x},{other}")]
            }
            10 => {
                let (first, second) = (
                    BOUNDED_ITEMS[r.below(BOUNDED_ITEMS.len())],
                    BOUNDED_ITEMS[r.below(BOUNDED_ITEMS.len())],
                );
                vec!["retainAll".into(), format!("{x},{first},{second}")]
            }
            11 => vec!["offerT".into(), x],
            12 => {
                let other = BOUNDED_ITEMS[r.below(BOUNDED_ITEMS.len())];
                vec!["containsAll".into(), format!("{x},{other}")]
            }
            _ => words(&["rc"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["size"]), words(&["array"]), words(&["rc"])]
    }
}

// ---------------------------------------------------------------- FrequencySketch

enum Word {
    Text(String),
    Number(i32),
}

impl JavaHashCode for Word {
    fn java_hash_code(&self) -> i32 {
        match self {
            Word::Text(text) => text.java_hash_code(),
            Word::Number(n) => *n,
        }
    }
}

fn word(text: &str) -> Word {
    text.strip_prefix('#')
        .map_or_else(|| Word::Text(text.to_owned()), |n| Word::Number(parse(n)))
}

fn raw_text(word: &Word) -> &str {
    match word {
        Word::Text(text) => text,
        Word::Number(_) => panic!("a numeric item in the generic sketch"),
    }
}

#[derive(Default)]
struct SketchInterp {
    generic: Option<FrequencySketch<str, StringHasher>>,
    fast: Option<FastFrequencySketch<Word>>,
}

fn rejected(e: &rakka_actor::actor::IllegalArgumentError) -> String {
    exception(e.java_class(), Some(e.message()))
}

impl Interp for SketchInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "murmur" => ok(
                murmur3::string_hash(if t[1] == "-" { "" } else { t[1] }, 135_283_237).to_string(),
            ),
            "murmurseed" => ok(murmur3::string_hash(t[2], parse(t[1])).to_string()),
            "fs" => {
                let sketch = FrequencySketch::new(
                    parse(t[1]),
                    parse(t[2]),
                    parse(t[3]),
                    parse(t[4]),
                    parse(t[5]),
                    StringHasher::default(),
                )
                .map_err(|e| rejected(&e))?;
                self.generic = Some(sketch);
                self.fast = None;
                ok("ok")
            }
            "fsraw" => {
                let sketch = FrequencySketch::from_parts(
                    parse(t[1]),
                    parse(t[2]),
                    parse(t[3]),
                    parse(t[4]),
                    StringHasher::default(),
                )
                .map_err(|e| rejected(&e))?;
                self.generic = Some(sketch);
                self.fast = None;
                ok("ok")
            }
            "ffs" => {
                let sketch = FastFrequencySketch::new(parse(t[1]), parse(t[2]), parse(t[3]))
                    .map_err(|e| rejected(&e))?;
                self.fast = Some(sketch);
                self.generic = None;
                ok("ok")
            }
            "ffsraw" => {
                let sketch = FastFrequencySketch::from_parts(parse(t[1]), parse(t[2]))
                    .map_err(|e| rejected(&e))?;
                self.fast = Some(sketch);
                self.generic = None;
                ok("ok")
            }
            "inc" => {
                let times = t.get(2).map_or(1, |n| parse::<usize>(n));
                let item = word(t[1]);
                for _ in 0..times {
                    match (&mut self.generic, &mut self.fast) {
                        (Some(sketch), _) => sketch.increment(raw_text(&item)),
                        (None, Some(sketch)) => sketch.increment(&item),
                        (None, None) => panic!("no sketch"),
                    }
                }
                ok("ok")
            }
            "freq" => {
                let item = word(t[1]);
                ok(match (&self.generic, &self.fast) {
                    (Some(sketch), _) => sketch.frequency(raw_text(&item)),
                    (None, Some(sketch)) => sketch.frequency(&item),
                    (None, None) => panic!("no sketch"),
                }
                .to_string())
            }
            "size" => ok(match (&self.generic, &self.fast) {
                (Some(sketch), _) => sketch.size(),
                (None, Some(sketch)) => sketch.size(),
                (None, None) => panic!("no sketch"),
            }
            .to_string()),
            "dbg" => ok(match (&self.generic, &self.fast) {
                (Some(sketch), _) => sketch.to_debug_string(),
                (None, Some(sketch)) => sketch.to_debug_string(),
                (None, None) => panic!("no sketch"),
            }),
            "bits" => {
                let i: i32 = parse(t[1]);
                ok(format!(
                    "pow2={} exp={} ceil={}",
                    b(bits::is_power_of_two(i)),
                    bits::power_of_two_exponent(i),
                    bits::ceiling_power_of_two(i)
                ))
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let pool = 40;
        let rank = {
            let bound = r.below(pool) + 1;
            r.below(bound)
        };
        let prefix = if self.fast.is_some() && r.below(5) == 0 {
            "#"
        } else {
            "k"
        };
        let text = format!("{prefix}{rank}");
        if r.below(4) == 0 {
            vec!["freq".into(), text]
        } else {
            vec!["inc".into(), text]
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        let mut out = vec![words(&["size"])];
        for i in 0..8 {
            out.push(vec!["freq".into(), format!("k{i}")]);
        }
        if self.fast.is_some() {
            for i in 0..4 {
                out.push(vec!["freq".into(), format!("#{i}")]);
            }
        }
        out
    }
}

// ---------------------------------------------------------------- RecencyList, FrequencyList, SegmentedRecencyList

/// The spec's `TestClock`: one tick is one second.
#[derive(Clone, Default)]
struct Ticks(Arc<AtomicI64>);

impl Ticks {
    fn tick(&self) -> i64 {
        self.0.fetch_add(1, Ordering::SeqCst) + 1
    }
}

impl Clock for Ticks {
    fn current_time(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        self.current_time() - duration_nanos / 1_000_000_000
    }
}

fn csv(text: &str) -> Vec<String> {
    if text == "-" {
        Vec::new()
    } else {
        text.split(',').map(str::to_owned).collect()
    }
}

fn seconds(text: &str) -> Duration {
    Duration::from_secs(parse(text))
}

const POOL: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

fn pick(r: &mut XorshiftStar) -> String {
    POOL[r.below(POOL.len())].to_owned()
}

fn unsupported(e: &rakka_actor::actor::UnsupportedOperationError) -> String {
    exception(e.java_class(), Some(e.message()))
}

struct RecencyInterp {
    clock: Ticks,
    list: RecencyList<String, Ticks>,
}

impl RecencyInterp {
    fn new() -> Self {
        let clock = Ticks::default();
        Self {
            list: RecencyList::new(clock.clone()),
            clock,
        }
    }
}

impl Interp for RecencyInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let l = &mut self.list;
        match t[0] {
            "tick" => ok(format!("t={}", self.clock.tick())),
            "u" => {
                for x in csv(t[1]) {
                    l.update(x);
                }
                ok(format!("size={}", l.size()))
            }
            "rm" => {
                for x in csv(t[1]) {
                    l.remove(&x);
                }
                ok(format!("size={}", l.size()))
            }
            "lr" => ok(list(l.remove_least_recent_n(parse(t[1])))),
            "lrs" => ok(list(
                l.remove_least_recent_skipping(parse(t[1]), parse(t[2])),
            )),
            "lr1" => ok(list(l.remove_least_recent())),
            "mr" => ok(list(l.remove_most_recent_n(parse(t[1])))),
            "mrs" => ok(list(
                l.remove_most_recent_skipping(parse(t[1]), parse(t[2])),
            )),
            "mr1" => ok(list(l.remove_most_recent())),
            "lo" => ok(list(l.remove_least_recent_outside(seconds(t[1])))),
            "mw" => ok(list(l.remove_most_recent_within(seconds(t[1])))),
            "l" => ok(list(l.least_to_most_recent())),
            "m" => ok(list(l.most_to_least_recent())),
            "has" => ok(b(l.contains(&t[1].to_owned()))),
            "size" => ok(l.size().to_string()),
            "least" => ok(opt(l.least_recent())),
            "most" => ok(opt(l.most_recent())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        match r.below(16) {
            0 => words(&["tick"]),
            1..=4 => vec!["u".into(), pick(r)],
            5 => vec!["rm".into(), pick(r)],
            6 => vec!["lr".into(), r.below(4).to_string()],
            7 => {
                let (n, skip) = (r.below(3), r.below(3));
                vec!["lrs".into(), n.to_string(), skip.to_string()]
            }
            8 => vec!["mr".into(), r.below(4).to_string()],
            9 => {
                let (n, skip) = (r.below(3), r.below(3));
                vec!["mrs".into(), n.to_string(), skip.to_string()]
            }
            10 => vec!["lo".into(), r.below(4).to_string()],
            11 => vec!["mw".into(), r.below(4).to_string()],
            12 => words(&["lr1"]),
            13 => words(&["mr1"]),
            14 => vec!["has".into(), pick(r)],
            _ => words(&["l"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["l"]),
            words(&["m"]),
            words(&["size"]),
            words(&["least"]),
            words(&["most"]),
        ]
    }
}

struct FrequencyInterp {
    clock: Ticks,
    list: FrequencyList<String, Ticks>,
}

impl FrequencyInterp {
    fn new() -> Self {
        Self {
            clock: Ticks::default(),
            list: FrequencyList::empty(false),
        }
    }
}

impl Interp for FrequencyInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let skip = |text: &str| (text != "-").then(|| text.to_owned());
        let l = &mut self.list;
        match t[0] {
            "new" => {
                let aging = t[1].contains("aging");
                self.list = if t[1].contains("clock") {
                    FrequencyList::with_overall_recency(self.clock.clone(), aging)
                } else {
                    FrequencyList::empty(aging)
                };
                ok("ok")
            }
            "tick" => ok(format!("t={}", self.clock.tick())),
            "u" => {
                for x in csv(t[1]) {
                    l.update(x);
                }
                ok(format!("size={}", l.size()))
            }
            "rm" => {
                for x in csv(t[1]) {
                    l.remove(&x);
                }
                ok(format!("size={}", l.size()))
            }
            "rlf" => ok(list(l.remove_least_frequent_n(parse(t[1])))),
            "rlf1" => ok(list(l.remove_least_frequent())),
            "rlfs" => ok(list(
                l.remove_least_frequent_skipping(parse(t[1]), skip(t[2]).as_ref()),
            )),
            "rmf" => ok(list(l.remove_most_frequent_n(parse(t[1])))),
            "rmf1" => ok(list(l.remove_most_frequent())),
            "rmfs" => ok(list(
                l.remove_most_frequent_skipping(parse(t[1]), skip(t[2]).as_ref()),
            )),
            "lf" => ok(list(l.least_to_most_frequent())),
            "mf" => ok(list(l.most_to_least_frequent())),
            "has" => ok(b(l.contains(&t[1].to_owned()))),
            "size" => ok(l.size().to_string()),
            "least" => ok(opt(l.least_frequent())),
            "most" => ok(opt(l.most_frequent())),
            "olr" => l
                .remove_overall_least_recent(parse(t[1]))
                .map(list)
                .map_err(|e| unsupported(&e)),
            "omr" => l
                .remove_overall_most_recent(parse(t[1]))
                .map(list)
                .map_err(|e| unsupported(&e)),
            "olo" => l
                .remove_overall_least_recent_outside(seconds(t[1]))
                .map(list)
                .map_err(|e| unsupported(&e)),
            "omw" => l
                .remove_overall_most_recent_within(seconds(t[1]))
                .map(list)
                .map_err(|e| unsupported(&e)),
            "oall" => ok(list(l.overall_least_to_most_recent())),
            "omall" => ok(list(l.overall_most_to_least_recent())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        match r.below(18) {
            0 => words(&["tick"]),
            1..=5 => vec!["u".into(), pick(r)],
            6 => vec!["rm".into(), pick(r)],
            7 => vec!["rlf".into(), r.below(4).to_string()],
            8 => {
                let n = r.below(3).to_string();
                let skip = if r.below(2) == 0 {
                    "-".to_owned()
                } else {
                    pick(r)
                };
                vec!["rlfs".into(), n, skip]
            }
            9 => vec!["rmf".into(), r.below(4).to_string()],
            10 => {
                let n = r.below(3).to_string();
                let skip = if r.below(2) == 0 {
                    "-".to_owned()
                } else {
                    pick(r)
                };
                vec!["rmfs".into(), n, skip]
            }
            11 => words(&["rlf1"]),
            12 => words(&["rmf1"]),
            13 => vec!["olr".into(), r.below(3).to_string()],
            14 => vec!["omr".into(), r.below(3).to_string()],
            15 => vec!["olo".into(), r.below(4).to_string()],
            16 => vec!["omw".into(), r.below(4).to_string()],
            _ => words(&["lf"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["lf"]),
            words(&["mf"]),
            words(&["size"]),
            words(&["least"]),
            words(&["most"]),
            words(&["oall"]),
        ]
    }
}

struct SegmentedInterp {
    clock: Ticks,
    list: SegmentedRecencyList<String, Ticks>,
    levels: usize,
}

fn limits(text: &str) -> Vec<i32> {
    text.split(',').map(parse).collect()
}

impl SegmentedInterp {
    fn new() -> Self {
        Self {
            clock: Ticks::default(),
            list: SegmentedRecencyList::empty(&[2, 3]),
            levels: 2,
        }
    }
}

impl Interp for SegmentedInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let l = &mut self.list;
        match t[0] {
            "new" => {
                self.levels = t[1].split(',').count();
                self.list = if t.get(2) == Some(&"clock") {
                    SegmentedRecencyList::with_overall_recency(self.clock.clone(), &limits(t[1]))
                } else {
                    SegmentedRecencyList::empty(&limits(t[1]))
                };
                ok("ok")
            }
            "tick" => ok(format!("t={}", self.clock.tick())),
            "u" => {
                for x in csv(t[1]) {
                    l.update(x);
                }
                ok(format!("size={}", l.size()))
            }
            "rm" => {
                for x in csv(t[1]) {
                    l.remove(&x);
                }
                ok(format!("size={}", l.size()))
            }
            "rol" => ok(list(l.remove_least_recent_over_limit())),
            "lr" => ok(list(l.remove_least_recent())),
            "oo" => l
                .remove_overall_least_recent_outside(seconds(t[1]))
                .map(list)
                .map_err(|e| unsupported(&e)),
            "limits" => {
                l.update_limits(&limits(t[1]));
                self.levels = t[1].split(',').count();
                ok("ok")
            }
            "lvl" => ok(list(l.least_to_most_recent_of(parse(t[1])))),
            "sizeof" => ok(l.size_of(parse(t[1])).to_string()),
            "levels" => ok((0..self.levels)
                .map(|i| list(l.least_to_most_recent_of(i)))
                .collect::<Vec<_>>()
                .join("/")),
            "has" => ok(b(l.contains(&t[1].to_owned()))),
            "size" => ok(l.size().to_string()),
            "least" => ok(opt(l.least_recent())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        match r.below(10) {
            0 => words(&["tick"]),
            1..=4 => vec!["u".into(), pick(r)],
            5 => vec!["rm".into(), pick(r)],
            6 => words(&["rol"]),
            7 => words(&["lr"]),
            8 => vec!["oo".into(), r.below(4).to_string()],
            _ => words(&["levels"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["levels"]),
            words(&["size"]),
            words(&["least"]),
            words(&["sizeof", "0"]),
        ]
    }
}

// ---------------------------------------------------------------- DoubleLinkedList

#[derive(Default)]
struct DoubleListInterp {
    list: DoubleLinkedList<usize>,
    links: LinkTable,
    names: Vec<String>,
    linked: Vec<String>,
    created: usize,
}

impl DoubleListInterp {
    fn node(&mut self, name: &str) -> usize {
        match self.names.iter().position(|n| n == name) {
            Some(at) => at,
            None => {
                self.names.push(name.to_owned());
                self.names.len() - 1
            }
        }
    }

    fn name(&self, node: usize) -> String {
        self.names[node].clone()
    }

    fn name_of(&self, node: Option<&usize>) -> String {
        node.map_or_else(|| "null".to_owned(), |n| self.name(*n))
    }
}

impl Interp for DoubleListInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let starts =
            |names: Vec<String>, prefix: String| move |n: &usize| names[*n].starts_with(&prefix);
        match t[0] {
            "pre" => {
                let n = self.node(t[1]);
                let r = self.list.prepend(&mut self.links, n);
                ok(self.name(r))
            }
            "app" => {
                let n = self.node(t[1]);
                let r = self.list.append(&mut self.links, n);
                ok(self.name(r))
            }
            "rm" => {
                let n = self.node(t[1]);
                self.list.remove(&mut self.links, &n);
                ok("ok")
            }
            "front" => {
                let n = self.node(t[1]);
                let r = self.list.move_to_front(&mut self.links, n);
                ok(self.name(r))
            }
            "back" => {
                let n = self.node(t[1]);
                let r = self.list.move_to_back(&mut self.links, n);
                ok(self.name(r))
            }
            "gfp" => {
                let fresh = self.node(t[2]);
                let r = self.list.get_first_or_else_prepend(
                    &mut self.links,
                    starts(self.names.clone(), t[1].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "glp" => {
                let fresh = self.node(t[2]);
                let r = self.list.get_last_or_else_append(
                    &mut self.links,
                    starts(self.names.clone(), t[1].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "gno" => {
                let (at, fresh) = (self.node(t[1]), self.node(t[3]));
                let r = self.list.get_next_or_else_insert(
                    &mut self.links,
                    &at,
                    starts(self.names.clone(), t[2].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "gpo" => {
                let (at, fresh) = (self.node(t[1]), self.node(t[3]));
                let r = self.list.get_previous_or_else_insert(
                    &mut self.links,
                    &at,
                    starts(self.names.clone(), t[2].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "fno" => {
                let (at, fresh) = (self.node(t[1]), self.node(t[4]));
                let r = self.list.find_next_or_else_insert(
                    &mut self.links,
                    &at,
                    starts(self.names.clone(), t[2].to_owned()),
                    starts(self.names.clone(), t[3].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "fpo" => {
                let (at, fresh) = (self.node(t[1]), self.node(t[4]));
                let r = self.list.find_previous_or_else_insert(
                    &mut self.links,
                    &at,
                    starts(self.names.clone(), t[2].to_owned()),
                    starts(self.names.clone(), t[3].to_owned()),
                    || fresh,
                );
                ok(self.name(r))
            }
            "check" => {
                let forward: Vec<String> = self
                    .list
                    .forward_iterator(&self.links)
                    .map(|n| self.name(n))
                    .collect();
                let mut backward: Vec<String> = self
                    .list
                    .backward_iterator(&self.links)
                    .map(|n| self.name(n))
                    .collect();
                backward.reverse();
                ok(format!(
                    "empty={} first={} last={} fwd={} backReversed={}",
                    b(self.list.is_empty()),
                    self.name_of(self.list.first()),
                    self.name_of(self.list.last()),
                    list(forward),
                    list(backward)
                ))
            }
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut XorshiftStar) -> Vec<String> {
        let choice = r.below(8);
        if self.linked.is_empty() || choice < 2 {
            let name = format!("n{}", self.created);
            self.created += 1;
            self.linked.push(name.clone());
            let op = if r.below(2) == 0 { "pre" } else { "app" };
            return vec![op.into(), name];
        }
        let existing = self.linked[r.below(self.linked.len())].clone();
        match choice {
            2 => {
                self.linked.retain(|n| *n != existing);
                vec!["rm".into(), existing]
            }
            3 => vec!["front".into(), existing],
            4 => vec!["back".into(), existing],
            _ => words(&["check"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["check"])]
    }
}

// ---------------------------------------------------------------- concurrent cases (invariants only)

fn concurrent_cases() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, result: String| out.push((id.to_owned(), result));
    add(
        "CC-01 Index in parallel: the upper half is never lost",
        index_in_parallel(),
    );
    add(
        "CC-02 Switch: exactly one of 8 racing switchOn wins",
        racing_switch_on(),
    );
    add(
        "CC-03 Switch: switchOff waits for locked",
        switch_off_waits_for_locked(),
    );
    add(
        "BQ-C1 one producer, one consumer through capacity 3 keep FIFO",
        one_producer_one_consumer(),
    );
    add(
        "BQ-C2 four producers and four consumers lose and duplicate nothing",
        four_and_four(),
    );
    add(
        "BQ-C3 take blocks on an empty queue until an element arrives",
        take_blocks(),
    );
    add(
        "BQ-C4 put blocks on a full queue until there is room",
        put_blocks(),
    );
    add(
        "BQ-C5 offer with a timeout gives up on a full queue and succeeds when room appears",
        offer_with_timeout(),
    );
    add(
        "BQ-C6 poll with a timeout returns null on an empty queue and the element when one arrives",
        poll_with_timeout(),
    );
    add(
        "BQ-C9 poll with Long.MAX_VALUE nanoseconds waits until an element arrives",
        poll_with_unbounded_timeout(Duration::from_nanos(i64::MAX as u64)),
    );
    add(
        "BQ-C10 poll with Long.MAX_VALUE days saturates and waits until an element arrives",
        poll_with_unbounded_timeout(Duration::MAX),
    );
    add(
        "BQ-C11 offer with Long.MAX_VALUE nanoseconds on a full queue waits until there is room",
        offer_with_unbounded_timeout(Duration::from_nanos(i64::MAX as u64)),
    );
    add(
        "BQ-C12 offer with Long.MAX_VALUE days saturates and waits until there is room",
        offer_with_unbounded_timeout(Duration::MAX),
    );
    add(
        "BQ-C7 iterator remove drops the element it returned, by identity",
        iterator_remove_by_identity(),
    );
    add("BQ-C8 the iterator is a snapshot", iterator_snapshot());
    out
}

fn index_in_parallel() -> String {
    let (keys, values, threads, tasks_per_thread) = (10_usize, 10_usize, 8_i64, 1500);
    let index = Arc::new(Index::<i32, i32>::new(100, |a, b| a.cmp(b)));
    for k in 0..keys {
        for v in 0..values {
            index.put(k as i32, v as i32);
        }
    }
    let go = Arc::new(Barrier::new(threads as usize));
    let upper_always_filled = Arc::new(AtomicBool::new(true));
    let workers: Vec<_> = (0..threads)
        .map(|i| {
            let (index, go, upper) = (index.clone(), go.clone(), upper_always_filled.clone());
            thread::spawn(move || {
                let mut r = XorshiftStar::new(1000 + i);
                go.wait();
                for _ in 0..tasks_per_thread {
                    match r.below(4) {
                        0 => {
                            let (k, v) = (r.below(keys) as i32, r.below(values) as i32);
                            index.put(k, v);
                        }
                        1 => {
                            let (k, v) = (r.below(keys / 2) as i32, r.below(values) as i32);
                            index.remove(&k, &v);
                        }
                        2 => {
                            index.remove_key(&(r.below(keys / 2) as i32));
                        }
                        _ => {
                            let key = r.below(keys);
                            if key >= keys / 2
                                && index.value_iterator(&(key as i32)).next().is_none()
                            {
                                upper.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("no worker panicked");
    }
    let upper: Vec<String> = (keys / 2..keys)
        .map(|k| list(index.value_iterator(&(k as i32))))
        .collect();
    let lower_within_range = (0..keys / 2).all(|k| {
        index
            .value_iterator(&(k as i32))
            .all(|v| (0..values as i32).contains(&v))
    });
    format!(
        "upperAlwaysFilled={} upper={} lowerWithinRange={}",
        upper_always_filled.load(Ordering::SeqCst),
        upper.join(" "),
        lower_within_range
    )
}

fn racing_switch_on() -> String {
    let switch = Arc::new(Switch::new(false));
    let go = Arc::new(Barrier::new(8));
    let winners: usize = (0..8)
        .map(|_| {
            let (switch, go) = (switch.clone(), go.clone());
            thread::spawn(move || {
                go.wait();
                switch.switch_on()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|t| usize::from(t.join().expect("no panic")))
        .sum();
    format!("winners={winners} on={}", b(switch.is_on()))
}

fn switch_off_waits_for_locked() -> String {
    let switch = Arc::new(Switch::new(true));
    let off_done = Arc::new(AtomicBool::new(false));
    let other: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);
    let while_locked = switch.locked(|| {
        let (to_switch, to_flag) = (switch.clone(), off_done.clone());
        *other.lock().expect("lock") = Some(thread::spawn(move || {
            to_switch.switch_off();
            to_flag.store(true, Ordering::SeqCst);
        }));
        // whatever the other thread has reached, it cannot have changed the switch while we hold the lock
        format!(
            "on={} offDone={}",
            b(switch.is_on()),
            b(off_done.load(Ordering::SeqCst))
        )
    });
    other
        .lock()
        .expect("lock")
        .take()
        .expect("spawned")
        .join()
        .expect("no panic");
    format!(
        "whileLocked[{while_locked}] after: on={} offDone={}",
        b(switch.is_on()),
        b(off_done.load(Ordering::SeqCst))
    )
}

fn bounded_ints(capacity: i32) -> Arc<BoundedBlockingQueue<Arc<i32>>> {
    Arc::new(BoundedBlockingQueue::new(capacity, VecDeque::<Arc<i32>>::new()).expect("valid"))
}

fn one_producer_one_consumer() -> String {
    let queue = bounded_ints(3);
    let n = 2000;
    let overfull = Arc::new(AtomicBool::new(false));
    let producer = {
        let queue = queue.clone();
        thread::spawn(move || {
            for i in 0..n {
                queue.put(Arc::new(i));
            }
        })
    };
    let consumer = {
        let (queue, overfull) = (queue.clone(), overfull.clone());
        thread::spawn(move || {
            (0..n)
                .map(|_| {
                    if queue.len() > 3 {
                        overfull.store(true, Ordering::SeqCst);
                    }
                    *queue.take()
                })
                .collect::<Vec<_>>()
        })
    };
    producer.join().expect("no panic");
    let got = consumer.join().expect("no panic");
    let fifo = got.iter().enumerate().all(|(i, v)| *v == i as i32);
    format!(
        "received={} fifo={} neverOverfull={} left={}",
        got.len(),
        b(fifo),
        b(!overfull.load(Ordering::SeqCst)),
        queue.len()
    )
}

fn four_and_four() -> String {
    let queue = bounded_ints(5);
    let (producers, consumers, each) = (4_i32, 4, 500);
    let mut handles = Vec::new();
    for p in 0..producers {
        let queue = queue.clone();
        handles.push(thread::spawn(move || {
            for i in 0..each {
                queue.put(Arc::new(p * 1_000_000 + i));
            }
            Vec::new()
        }));
    }
    for _ in 0..consumers {
        let queue = queue.clone();
        handles.push(thread::spawn(move || {
            (0..each).map(|_| *queue.take()).collect::<Vec<_>>()
        }));
    }
    let received: Vec<Vec<i32>> = handles
        .into_iter()
        .map(|h| h.join().expect("no panic"))
        .collect();
    let mut all = HashSet::new();
    let mut total = 0;
    let mut per_producer_order = true;
    for mine in received.iter().filter(|r| !r.is_empty()) {
        let mut last = vec![-1_i32; producers as usize];
        for v in mine {
            total += 1;
            all.insert(*v);
            let (p, i) = ((v / 1_000_000) as usize, v % 1_000_000);
            if i <= last[p] {
                per_producer_order = false;
            }
            last[p] = i;
        }
    }
    format!(
        "total={total} distinct={} perProducerOrderPerConsumer={} left={}",
        all.len(),
        b(per_producer_order),
        queue.len()
    )
}

/// A queue of strings that counts the times a thread went to sleep waiting for the queue to be not empty
/// or not full, to know that the thread under test is parked without sleeping.
struct Parked {
    queue: Arc<BoundedBlockingQueue<Item>>,
    not_empty: Arc<AtomicUsize>,
    not_full: Arc<AtomicUsize>,
}

impl Parked {
    fn new(capacity: i32) -> Self {
        let (not_empty, not_full) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let (ne, nf) = (not_empty.clone(), not_full.clone());
        let listener: Arc<dyn Fn(QueueEvent) + Send + Sync> = Arc::new(move |event| match event {
            QueueEvent::AwaitNotEmpty => {
                ne.fetch_add(1, Ordering::SeqCst);
            }
            QueueEvent::AwaitNotFull => {
                nf.fetch_add(1, Ordering::SeqCst);
            }
            QueueEvent::SignalNotEmpty | QueueEvent::SignalNotFull => {}
        });
        let queue =
            BoundedBlockingQueue::with_listener(capacity, VecDeque::<Item>::new(), Some(listener))
                .expect("valid");
        Self {
            queue: Arc::new(queue),
            not_empty,
            not_full,
        }
    }

    fn not_empty_waiters(&self) -> usize {
        self.not_empty.load(Ordering::SeqCst)
    }

    fn not_full_waiters(&self) -> usize {
        self.not_full.load(Ordering::SeqCst)
    }

    fn wait_not_empty_above(&self, before: usize) {
        while self.not_empty.load(Ordering::SeqCst) <= before {
            thread::yield_now();
        }
    }

    fn wait_not_full_above(&self, before: usize) {
        while self.not_full.load(Ordering::SeqCst) <= before {
            thread::yield_now();
        }
    }
}

fn take_blocks() -> String {
    let parked = Parked::new(1);
    let consumer = {
        let queue = parked.queue.clone();
        thread::spawn(move || queue.take())
    };
    parked.wait_not_empty_above(0);
    let before = "null";
    parked.queue.put(item("x"));
    let after = consumer.join().expect("no panic");
    format!("before={before} after={after} left={}", parked.queue.len())
}

fn put_blocks() -> String {
    let parked = Parked::new(1);
    parked.queue.put(item("a"));
    let done = Arc::new(AtomicBool::new(false));
    let producer = {
        let (queue, done) = (parked.queue.clone(), done.clone());
        thread::spawn(move || {
            queue.put(item("b"));
            done.store(true, Ordering::SeqCst);
        })
    };
    parked.wait_not_full_above(0);
    let during = format!(
        "done={} size={}",
        b(done.load(Ordering::SeqCst)),
        parked.queue.len()
    );
    let first = parked.queue.take();
    producer.join().expect("no panic");
    let then = parked
        .queue
        .poll()
        .map_or_else(|| "null".to_owned(), |i| i.to_string());
    format!(
        "{during} first={first} then={then} done={}",
        b(done.load(Ordering::SeqCst))
    )
}

fn offer_with_timeout() -> String {
    let parked = Parked::new(1);
    parked.queue.put(item("a"));
    let offered_on_full = parked
        .queue
        .offer_timeout(item("b"), Duration::from_millis(20));
    let before = parked.not_full.load(Ordering::SeqCst);
    let producer = {
        let queue = parked.queue.clone();
        thread::spawn(move || queue.offer_timeout(item("c"), Duration::from_secs(1000 * 86_400)))
    };
    parked.wait_not_full_above(before);
    parked.queue.take();
    let later = producer.join().expect("no panic");
    format!(
        "gaveUp={} later={} content={}",
        b(!offered_on_full),
        b(later),
        show_items(parked.queue.to_vec())
    )
}

fn poll_with_timeout() -> String {
    let parked = Parked::new(1);
    let empty = parked
        .queue
        .poll_timeout(Duration::from_millis(20))
        .map_or_else(|| "null".to_owned(), |i| i.to_string());
    let before = parked.not_empty.load(Ordering::SeqCst);
    let consumer = {
        let queue = parked.queue.clone();
        thread::spawn(move || queue.poll_timeout(Duration::from_secs(1000 * 86_400)))
    };
    parked.wait_not_empty_above(before);
    parked.queue.put(item("z"));
    let got = consumer
        .join()
        .expect("no panic")
        .map_or_else(|| "null".to_owned(), |i| i.to_string());
    format!("empty={empty} got={got}")
}

/// A timeout past what an `Instant` can hold is "no deadline" (Java saturates it to `Long.MAX_VALUE` nanoseconds).
fn poll_with_unbounded_timeout(timeout: Duration) -> String {
    let parked = Parked::new(1);
    let consumer = {
        let queue = parked.queue.clone();
        thread::spawn(move || queue.poll_timeout(timeout))
    };
    // A consumer that died (panicked on the deadline arithmetic) is reported, not waited for.
    while parked.not_empty_waiters() == 0 && !consumer.is_finished() {
        thread::yield_now();
    }
    parked.queue.put(item("z"));
    match consumer.join() {
        Ok(taken) => format!(
            "got={}",
            taken.map_or_else(|| "null".to_owned(), |i| i.to_string())
        ),
        Err(_) => "consumer panicked".to_owned(),
    }
}

fn offer_with_unbounded_timeout(timeout: Duration) -> String {
    offered_to_full_queue_outcome(move |queue| queue.offer_timeout(item("c"), timeout))
}

/// Runs `offer` on a producer thread against a full queue of one, frees a slot once the producer is parked (or has
/// died) and reports what the producer returned; a producer that panicked is reported as such instead of being waited
/// for forever.
fn offered_to_full_queue_outcome(
    offer: impl FnOnce(&BoundedBlockingQueue<Item>) -> bool + Send + 'static,
) -> String {
    let parked = Parked::new(1);
    parked.queue.put(item("a"));
    let producer = {
        let queue = parked.queue.clone();
        thread::spawn(move || offer(&queue))
    };
    while parked.not_full_waiters() == 0 && !producer.is_finished() {
        thread::yield_now();
    }
    parked.queue.take();
    match producer.join() {
        Ok(later) => format!(
            "later={} content={}",
            b(later),
            show_items(parked.queue.to_vec())
        ),
        Err(_) => "producer panicked".to_owned(),
    }
}

fn iterator_remove_by_identity() -> String {
    let queue = BoundedBlockingQueue::new(3, VecDeque::<Item>::new()).expect("valid");
    let (first, second) = (item("a"), item("a"));
    queue.put(first.clone());
    queue.put(second);
    let mut it = queue.iter();
    it.next();
    it.next();
    it.remove().expect("remove");
    let left = queue.to_vec();
    format!(
        "size={} keptFirst={} rc={}",
        queue.len(),
        b(Arc::ptr_eq(&left[0], &first)),
        queue.remaining_capacity()
    )
}

fn iterator_snapshot() -> String {
    let queue = BoundedBlockingQueue::new(4, VecDeque::<Item>::new()).expect("valid");
    queue.put(item("a"));
    queue.put(item("b"));
    let it = queue.iter();
    queue.put(item("c"));
    queue.poll();
    let seen: Vec<Item> = it.collect();
    format!(
        "seen={} now={}",
        show_items(seen),
        show_items(queue.to_vec())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_producer_that_panics_is_reported_instead_of_waited_for() {
        let outcome = offered_to_full_queue_outcome(|_| panic!("deadline arithmetic overflowed"));
        assert_eq!(outcome, "producer panicked");
    }

    #[test]
    fn a_producer_that_waits_for_room_reports_its_offer_and_the_content() {
        let outcome =
            offered_to_full_queue_outcome(|queue| queue.offer_timeout(item("c"), Duration::MAX));
        assert_eq!(outcome, "later=t content=[c]");
    }
}
