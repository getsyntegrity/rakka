// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! Cases of the AC-UTIL (bytes) oracle (`BytesOracle.java`): the Rust twin of each case. The cases are the
//! scripts of `bytes.tsv`, run by an interpreter per family that mirrors the Java one operation by
//! operation and prints one token per operation. Slice P04.

use std::collections::HashMap;

use bytes::{Buf, BytesMut};
use rakka_actor::util::byte_string::{
    ByteIterator, ByteIteratorInputStream, ByteString, ByteStringBuilder, ByteStringError, Charset,
    UnsynchronizedByteArrayInputStream,
};
use rakka_actor::util::swar::ByteOrder;

const SCRIPTS: &str = include_str!("bytes.tsv");

type Out = Result<String, ByteStringError>;

fn ok(text: impl Into<String>) -> Out {
    Ok(text.into())
}

fn flag(x: bool) -> &'static str {
    if x { "t" } else { "f" }
}

fn parse<T: std::str::FromStr>(text: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    text.parse()
        .unwrap_or_else(|e| panic!("bad number {text}: {e:?}"))
}

/// A data word `xHEX` as bytes.
fn hex(word: &str) -> Vec<u8> {
    let digits = word.strip_prefix('x').expect("a hex word");
    (0..digits.len() / 2)
        .map(|i| u8::from_str_radix(&digits[2 * i..2 * i + 2], 16).expect("hex digits"))
        .collect()
}

fn hexs(bytes: &[u8]) -> String {
    let digits: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("<{digits}>")
}

fn hexs_of(bs: &ByteString) -> String {
    hexs(bs.as_slice())
}

fn order(word: &str) -> ByteOrder {
    match word {
        "BE" => ByteOrder::BigEndian,
        "LE" => ByteOrder::LittleEndian,
        other => panic!("order {other}"),
    }
}

/// A single byte given as a decimal that is cast like `(byte) n`.
fn sb(word: &str) -> u8 {
    parse::<i32>(word) as u8
}

fn signed_below(k: i32) -> impl FnMut(u8) -> bool {
    move |b| i32::from(b as i8) < k
}

fn rope(t: &[&str], from: usize) -> ByteString {
    t[from..].iter().fold(ByteString::empty(), |acc, w| {
        acc.concat(&ByteString::from_array(&hex(w)))
    })
}

fn charset(word: &str) -> Charset {
    Charset::for_name(word).unwrap_or_else(|| panic!("charset {word}"))
}

fn ints<T: std::str::FromStr>(t: &[&str], from: usize) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    t[from..].iter().map(|w| parse(w)).collect()
}

/// xorshift64*, as in the oracle.
struct Rng(u64);

impl Rng {
    fn new(seed: i64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed as u64
        })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        ((self.next() >> 33) % bound as u64) as usize
    }
}

trait Interp {
    fn op(&mut self, t: &[&str]) -> Out;
    fn random(&mut self, r: &mut Rng) -> Vec<String>;
    fn epilogue(&self) -> Vec<Vec<String>>;
}

fn step(interp: &mut dyn Interp, t: &[&str]) -> String {
    match interp.op(t) {
        Ok(text) => text,
        Err(e) => {
            let class = e.java_class().rsplit('.').next().unwrap_or_default();
            format!("!{class}[{}]", e.message().unwrap_or("null"))
        }
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
            let mut r = Rng::new(parse(t[1]));
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

fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn interpreter(family: &str) -> Box<dyn Interp> {
    match family {
        "bytestring" => Box::new(StringInterp::default()),
        "builder" => Box::new(BuilderInterp::default()),
        "iterator" => Box::new(IteratorInterp::default()),
        "stream" => Box::new(StreamInterp::default()),
        other => panic!("unknown family {other}"),
    }
}

/// The lines of the bytes oracle, in the order of `bytes.tsv`: `(id, result)` where the result is what
/// follows `ok ` in the oracle line. The cases Pekko answers with a JVM error text that Rust does not
/// reproduce (Q-UTIL-23) are not here.
pub fn cases() -> Vec<(String, String)> {
    SCRIPTS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.splitn(3, '\t');
            let (id, family, script) = (
                fields.next().expect("an id"),
                fields.next().expect("a family"),
                fields.next().expect("a script"),
            );
            (id.to_owned(), run_script(&mut *interpreter(family), script))
        })
        .collect()
}

// ---------------------------------------------------------------- ByteString

#[derive(Default)]
struct StringInterp {
    vars: HashMap<String, ByteString>,
}

impl StringInterp {
    fn v(&self, name: &str) -> &ByteString {
        self.vars
            .get(name)
            .unwrap_or_else(|| panic!("unbound {name}"))
    }

    fn bind(&mut self, name: &str, value: ByteString) -> Out {
        let text = hexs_of(&value);
        self.vars.insert(name.to_owned(), value);
        ok(text)
    }
}

fn index(found: Option<usize>) -> Out {
    ok(found.map_or_else(|| "-1".to_owned(), |i| i.to_string()))
}

impl Interp for StringInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "arr" => self.bind(t[1], ByteString::from_array(&hex(t[2]))),
            "view" => {
                let whole = ByteString::from_array(&hex(t[2]));
                self.bind(t[1], whole.slice(parse(t[3]), parse(t[4])))
            }
            "rope" => self.bind(t[1], rope(t, 2)),
            "ints" => self.bind(t[1], ByteString::from_ints(&ints::<i32>(t, 2))),
            "fromarr" => self.bind(
                t[1],
                ByteString::from_array_range(&hex(t[2]), parse(t[3]), parse(t[4])),
            ),
            "str" => {
                let text = String::from_utf8(hex(t[2])).expect("valid UTF-8 script text");
                let value = if t.len() > 3 {
                    ByteString::from_string_with_charset(&text, t[3])?
                } else {
                    ByteString::from_string(&text)
                };
                self.bind(t[1], value)
            }
            "buf" => {
                let data = hex(t[2]);
                let position: usize = parse(t[3]);
                let mut buffer = &data[position..];
                let value = ByteString::from_buf(&mut buffer);
                let consumed = data.len() - buffer.remaining();
                let text = self.bind(t[1], value)?;
                ok(format!("{text} pos={consumed}"))
            }
            "unsafe" => self.bind(t[1], ByteString::from_array_unsafe(hex(t[2]))),
            "rnd" => {
                let mut r = Rng::new(parse(t[2]));
                let pool = hex(t[4]);
                let count: usize = parse(t[3]);
                let bytes = (0..count)
                    .map(|_| {
                        if pool.is_empty() {
                            r.below(256) as u8
                        } else {
                            pool[r.below(pool.len())]
                        }
                    })
                    .collect();
                self.bind(t[1], ByteString::from_array_unsafe(bytes))
            }
            "compact" => {
                let compacted = self.v(t[1]).compact();
                self.bind(t[2], compacted)
            }
            "show" => ok(self.v(t[1]).to_string()),
            "len" => ok(self.v(t[1]).len().to_string()),
            "empty" => ok(flag(self.v(t[1]).is_empty())),
            "apply" => ok((self.v(t[1]).apply(parse(t[2]))? as i8).to_string()),
            "head" => ok((self.v(t[1]).head()? as i8).to_string()),
            "last" => ok((self.v(t[1]).last()? as i8).to_string()),
            "eq" => ok(flag(self.v(t[1]) == self.v(t[2]))),
            "take" => {
                let r = self.v(t[2]).take(parse(t[3]));
                self.bind(t[1], r)
            }
            "drop" => {
                let r = self.v(t[2]).drop(parse(t[3]));
                self.bind(t[1], r)
            }
            "dropr" => {
                let r = self.v(t[2]).drop_right(parse(t[3]));
                self.bind(t[1], r)
            }
            "taker" => {
                let r = self.v(t[2]).take_right(parse(t[3]));
                self.bind(t[1], r)
            }
            "sl" => {
                let r = self.v(t[2]).slice(parse(t[3]), parse(t[4]));
                self.bind(t[1], r)
            }
            "cat" => {
                let r = self.v(t[2]).concat(self.v(t[3]));
                self.bind(t[1], r)
            }
            "tail" => {
                let r = self.v(t[2]).tail();
                self.bind(t[1], r)
            }
            "init" => {
                let r = self.v(t[2]).init();
                self.bind(t[1], r)
            }
            "splitat" => {
                let (a, b) = self.v(t[1]).split_at(parse(t[2]));
                ok(format!("{}{}", hexs_of(&a), hexs_of(&b)))
            }
            "span" => {
                let (a, b) = self.v(t[1]).span(signed_below(parse(t[2])));
                ok(format!("{}{}", hexs_of(&a), hexs_of(&b)))
            }
            "takew" => {
                let r = self.v(t[2]).take_while(signed_below(parse(t[3])));
                self.bind(t[1], r)
            }
            "dropw" => {
                let r = self.v(t[2]).drop_while(signed_below(parse(t[3])));
                self.bind(t[1], r)
            }
            "grouped" => {
                let groups = self.v(t[1]).grouped(parse(t[2]))?;
                let parts: String = groups.iter().map(hexs_of).collect();
                ok(format!("[{parts}]"))
            }
            "mapi" => {
                let k: i32 = parse(t[3]);
                let r = self.v(t[2]).map_i(|b| i32::from(b as i8) + k);
                self.bind(t[1], r)
            }
            "sum" => ok(self
                .v(t[1])
                .iter()
                .map(|b| i32::from(b as i8))
                .sum::<i32>()
                .to_string()),
            "toarr" => ok(hexs(&self.v(t[1]).to_vec())),
            "idx" => {
                let bs = self.v(t[1]);
                let byte = sb(t[2]);
                index(match t.len() {
                    3 => bs.index_of(byte),
                    4 => bs.index_of_from(byte, parse(t[3])),
                    _ => bs.index_of_range(byte, parse(t[3]), parse(t[4])),
                })
            }
            "lidx" => {
                let bs = self.v(t[1]);
                index(if t.len() == 3 {
                    bs.last_index_of(sb(t[2]))
                } else {
                    bs.last_index_of_end(sb(t[2]), parse(t[3]))
                })
            }
            "has" => ok(flag(self.v(t[1]).contains(sb(t[2])))),
            "idxs" => {
                let bs = self.v(t[1]);
                let from = if t.len() == 3 { 0 } else { parse(t[3]) };
                index(bs.index_of_slice(&hex(t[2]), from))
            }
            "lidxs" => {
                let bs = self.v(t[1]);
                let end = if t.len() == 3 {
                    bs.len() as i32
                } else {
                    parse(t[3])
                };
                index(bs.last_index_of_slice(&hex(t[2]), end))
            }
            "hass" => ok(flag(self.v(t[1]).contains_slice(&hex(t[2])))),
            "starts" => {
                let bs = self.v(t[1]);
                let offset = if t.len() == 3 { 0 } else { parse(t[3]) };
                ok(flag(bs.starts_with_at(&hex(t[2]), offset)))
            }
            "ends" => ok(flag(self.v(t[1]).ends_with(&hex(t[2])))),
            "utf8" => ok(self.v(t[1]).utf8_string()),
            "dec" => {
                let bs = self.v(t[1]);
                if t.len() == 3 {
                    Ok(bs.decode_string(t[2])?)
                } else {
                    ok(bs.decode_string_range(charset(t[2]), parse(t[3]), parse(t[4])))
                }
            }
            "b64e" => {
                let r = self.v(t[2]).encode_base64();
                self.bind(t[1], r)
            }
            "b64d" => {
                let r = self.v(t[2]).decode_base64()?;
                self.bind(t[1], r)
            }
            "copyarr" => {
                let mut dest = vec![0u8; parse(t[2])];
                let n = self
                    .v(t[1])
                    .copy_to_array(&mut dest, parse(t[3]), parse(t[4]));
                ok(format!("n={n} dest={}", hexs(&dest)))
            }
            "copybuf" => {
                let capacity: usize = parse(t[2]);
                let mut buffer = BytesMut::zeroed(capacity);
                let mut window = &mut buffer[..];
                let n = self.v(t[1]).copy_to_buffer(&mut window);
                ok(format!("n={n} pos={n} dest={}", hexs(&buffer)))
            }
            "tobuf" => ok(hexs(&self.v(t[1]).to_vec())),
            "iter" => ok(hexs_of(&self.v(t[1]).iterator().to_byte_string())),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut Rng) -> Vec<String> {
        let names = ["a", "b", "c"];
        let (x, y, z) = (names[r.below(3)], names[r.below(3)], names[r.below(3)]);
        let n = (r.below(16) as i32 - 3).to_string();
        let m = (r.below(16) as i32 - 3).to_string();
        match r.below(9) {
            0 => words(&["cat", x, y, z]),
            1 => words(&["take", x, y, &n]),
            2 => words(&["drop", x, y, &n]),
            3 => words(&["sl", x, y, &n, &m]),
            4 => words(&["dropr", x, y, &n]),
            5 => words(&["taker", x, y, &n]),
            6 => words(&["splitat", y, &n]),
            7 => words(&["idx", y, &r.below(6).to_string(), &n, &m]),
            _ => {
                let pattern = format!("x0{}0{}", r.below(6), r.below(6));
                words(&["idxs", y, &pattern, &n])
            }
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![
            words(&["show", "a"]),
            words(&["show", "b"]),
            words(&["show", "c"]),
            words(&["len", "a"]),
            words(&["eq", "a", "b"]),
            words(&["toarr", "c"]),
        ]
    }
}

// ---------------------------------------------------------------- ByteStringBuilder

#[derive(Default)]
struct BuilderInterp {
    builder: ByteStringBuilder,
}

fn float_bits(word: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(word, 16).expect("hex bits"))
}

fn double_bits(word: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(word, 16).expect("hex bits"))
}

impl Interp for BuilderInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        let b = &mut self.builder;
        match t[0] {
            "byte" => b.put_byte(sb(t[1])),
            "plus" => b.add_one(sb(t[1])),
            "short" => b.put_short(parse(t[1]), order(t[2])),
            "int" => b.put_int(parse(t[1]), order(t[2])),
            "long" => b.put_long(parse(t[1]), order(t[2])),
            "longpart" => b.put_long_part(parse(t[1]), parse(t[2]), order(t[3])),
            "float" => b.put_float(float_bits(t[1]), order(t[2])),
            "double" => b.put_double(double_bits(t[1]), order(t[2])),
            "bytes" => b.put_bytes(&hex(t[1])),
            "bytesr" => b.put_bytes_range(&hex(t[1]), parse(t[2]), parse(t[3]))?,
            "append" => b.append(&rope(t, 1)),
            "addall" => b.add_all(hex(t[1])),
            "shorts" => b.put_shorts(&ints::<i16>(t, 2), order(t[1])),
            "ints" => b.put_ints(&ints::<i32>(t, 2), order(t[1])),
            "intsr" => {
                b.put_ints_range(&ints::<i32>(t, 4), parse(t[2]), parse(t[3]), order(t[1]))?
            }
            "longs" => b.put_longs(&ints::<i64>(t, 2), order(t[1])),
            "floats" => {
                let values: Vec<f32> = t[2..].iter().map(|w| float_bits(w)).collect();
                b.put_floats(&values, order(t[1]));
            }
            "doubles" => {
                let values: Vec<f64> = t[2..].iter().map(|w| double_bits(w)).collect();
                b.put_doubles(&values, order(t[1]));
            }
            "write" => b.write_int(parse(t[1])),
            "writeb" => b.put_bytes_range(&hex(t[1]), parse(t[2]), parse(t[3]))?,
            "len" => return ok(b.len().to_string()),
            "isempty" => return ok(flag(b.is_empty())),
            "nonempty" => return ok(flag(b.non_empty())),
            "clear" => b.clear(),
            "hint" => b.size_hint(parse(t[1])),
            "result" => {
                let r = b.result();
                return ok(format!("{} len={}", hexs_of(&r), r.len()));
            }
            other => panic!("unknown op {other}"),
        }
        ok("ok")
    }

    fn random(&mut self, r: &mut Rng) -> Vec<String> {
        let endian = |r: &mut Rng| if r.below(2) == 0 { "BE" } else { "LE" };
        match r.below(7) {
            0 => words(&["byte", &(r.below(256) as i32 - 128).to_string()]),
            1 => {
                let v = (r.below(70000) as i32 - 3000).to_string();
                words(&["short", &v, endian(r)])
            }
            2 => {
                let v = ((r.next() as i64) >> 33).to_string();
                words(&["int", &v, endian(r)])
            }
            3 => {
                let v = ((r.next() as i64) >> 10).to_string();
                let n = r.below(9).to_string();
                words(&["longpart", &v, &n, endian(r)])
            }
            4 => {
                let first = format!("{:x}", 0x100 + r.below(256));
                let second = format!("{:x}", 0x100 + r.below(256));
                words(&["bytes", &format!("x{}{}", &first[1..], &second[1..])])
            }
            5 => words(&["result"]),
            _ => words(&["len"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["result"]), words(&["len"]), words(&["isempty"])]
    }
}

// ---------------------------------------------------------------- ByteIterator

#[derive(Default)]
struct IteratorInterp {
    it: ByteIterator,
    it2: ByteIterator,
    stream: Option<ByteIteratorInputStream>,
}

fn floats_text(values: &[f32]) -> String {
    let parts: String = values
        .iter()
        .map(|v| format!("{:x},", v.to_bits()))
        .collect();
    format!("[{parts}]")
}

fn doubles_text(values: &[f64]) -> String {
    let parts: String = values
        .iter()
        .map(|v| format!("{:x},", v.to_bits()))
        .collect();
    format!("[{parts}]")
}

fn array_text<T: std::fmt::Display>(values: &[T]) -> String {
    let parts: Vec<String> = values.iter().map(ToString::to_string).collect();
    format!("[{}]", parts.join(", "))
}

impl Interp for IteratorInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "iter" => {
                self.it = ByteString::from_array(&hex(t[1])).iterator();
                return ok("ok");
            }
            "iterview" => {
                self.it = ByteString::from_array(&hex(t[1]))
                    .slice(parse(t[2]), parse(t[3]))
                    .iterator();
                return ok("ok");
            }
            "iterrope" => {
                self.it = rope(t, 1).iterator();
                return ok("ok");
            }
            "clone" => {
                self.it2 = self.it.clone();
                return ok("ok");
            }
            "dup" => {
                let (first, second) = std::mem::take(&mut self.it).duplicate();
                self.it = first;
                self.it2 = second;
                return ok("ok");
            }
            _ => {}
        }
        let (name, i) = match t[0].strip_prefix("2:") {
            Some(name) => (name, &mut self.it2),
            None => (t[0], &mut self.it),
        };
        match name {
            "next" => ok((i.next_byte()? as i8).to_string()),
            "head" => ok(i
                .head()
                .map_or_else(|| "none".to_owned(), |b| (b as i8).to_string())),
            "hasnext" => ok(flag(i.has_next())),
            "len" => ok(i.len().to_string()),
            "size" => ok(i.size().to_string()),
            "take" => {
                i.take(parse(t[1]));
                ok(format!("len={}", i.len()))
            }
            "drop" => {
                i.drop(parse(t[1]));
                ok(format!("len={}", i.len()))
            }
            "slice" => {
                i.slice(parse(t[1]), parse(t[2]));
                ok(format!("len={}", i.len()))
            }
            "getbyte" => ok((i.get_byte()? as i8).to_string()),
            "getshort" => ok(i.get_short(order(t[1]))?.to_string()),
            "getint" => ok(i.get_int(order(t[1]))?.to_string()),
            "getlong" => ok(i.get_long(order(t[1]))?.to_string()),
            "getlongpart" => ok(i.get_long_part(parse(t[1]), order(t[2]))?.to_string()),
            "getfloat" => ok(format!("{:x}", i.get_float(order(t[1]))?.to_bits())),
            "getdouble" => ok(format!("{:x}", i.get_double(order(t[1]))?.to_bits())),
            "getbytes" => ok(hexs(&i.get_bytes(parse(t[1]))?)),
            "getbytesinto" => {
                let mut dest = vec![0u8; parse(t[1])];
                i.get_bytes_into(&mut dest, parse(t[2]), parse(t[3]))?;
                ok(format!("{} len={}", hexs(&dest), i.len()))
            }
            "getbs" => ok(hexs_of(&i.get_byte_string(parse(t[1])))),
            "getshorts" => {
                let mut values = vec![0i16; parse(t[2])];
                i.get_shorts(&mut values, order(t[1]))?;
                ok(array_text(&values))
            }
            "getints" => {
                let mut values = vec![0i32; parse(t[2])];
                i.get_ints(&mut values, order(t[1]))?;
                ok(array_text(&values))
            }
            "getlongs" => {
                let mut values = vec![0i64; parse(t[2])];
                i.get_longs(&mut values, order(t[1]))?;
                ok(array_text(&values))
            }
            "getfloats" => {
                let mut values = vec![0f32; parse(t[2])];
                i.get_floats(&mut values, order(t[1]))?;
                ok(floats_text(&values))
            }
            "getdoubles" => {
                let mut values = vec![0f64; parse(t[2])];
                i.get_doubles(&mut values, order(t[1]))?;
                ok(doubles_text(&values))
            }
            "copyarr" => {
                let mut dest = vec![0u8; parse(t[1])];
                let n = i.copy_to_array(&mut dest, parse(t[2]), parse(t[3]));
                ok(format!("n={n} dest={} len={}", hexs(&dest), i.len()))
            }
            "copybuf" => {
                let capacity: usize = parse(t[1]);
                let mut buffer = BytesMut::zeroed(capacity);
                let mut window = &mut buffer[..];
                let n = i.copy_to_buffer(&mut window);
                ok(format!(
                    "n={n} pos={n} dest={} len={}",
                    hexs(&buffer),
                    i.len()
                ))
            }
            "tobs" => {
                let bs = i.to_byte_string();
                ok(format!("{} len={}", hexs_of(&bs), i.len()))
            }
            "idxof" => {
                let from = if t.len() == 2 { 0 } else { parse(t[2]) };
                index(i.index_of(sb(t[1]), from))
            }
            "takew" => {
                i.take_while(signed_below(parse(t[1])));
                ok(format!("len={}", i.len()))
            }
            "dropw" => {
                i.drop_while(signed_below(parse(t[1])));
                ok(format!("len={}", i.len()))
            }
            "spanw" => {
                let (mut front, mut back) = std::mem::take(i).span(signed_below(parse(t[1])));
                ok(format!(
                    "{}{}",
                    hexs_of(&front.to_byte_string()),
                    hexs_of(&back.to_byte_string())
                ))
            }
            "ins" => {
                self.stream = Some(std::mem::take(i).into_input_stream());
                ok("ok")
            }
            "rd" => ok(self.stream().read().to_string()),
            "rdb" => {
                let mut dest = vec![0u8; parse(t[1])];
                let n = self
                    .stream()
                    .read_range(&mut dest, parse(t[2]), parse(t[3]))?;
                ok(format!("n={n} dest={}", hexs(&dest)))
            }
            "skip" => ok(self.stream().skip(parse(t[1])).to_string()),
            other => panic!("unknown op {other}"),
        }
    }

    fn random(&mut self, r: &mut Rng) -> Vec<String> {
        match r.below(8) {
            0 => words(&["next"]),
            1 => words(&["take", &(r.below(10) as i32 - 2).to_string()]),
            2 => words(&["drop", &(r.below(10) as i32 - 2).to_string()]),
            3 => words(&["getshort", if r.below(2) == 0 { "BE" } else { "LE" }]),
            4 => words(&["getbs", &r.below(6).to_string()]),
            5 => {
                let start = r.below(5).to_string();
                let len = r.below(6).to_string();
                words(&["copyarr", "4", &start, &len])
            }
            6 => words(&["len"]),
            _ => words(&["hasnext"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["len"]), words(&["tobs"]), words(&["hasnext"])]
    }
}

impl IteratorInterp {
    fn stream(&mut self) -> &mut ByteIteratorInputStream {
        self.stream.as_mut().expect("an input stream")
    }
}

// ---------------------------------------------------------------- java.io adapters

struct StreamInterp {
    input: UnsynchronizedByteArrayInputStream,
}

impl Default for StreamInterp {
    fn default() -> Self {
        Self {
            input: UnsynchronizedByteArrayInputStream::new(&[]),
        }
    }
}

impl Interp for StreamInterp {
    fn op(&mut self, t: &[&str]) -> Out {
        match t[0] {
            "bsin" => self.input = ByteString::from_array(&hex(t[1])).as_input_stream(),
            "bsview" => {
                self.input = ByteString::from_array(&hex(t[1]))
                    .slice(parse(t[2]), parse(t[3]))
                    .as_input_stream();
            }
            "bsrope" => self.input = rope(t, 1).as_input_stream(),
            "uin" => {
                self.input = if t.len() == 2 {
                    UnsynchronizedByteArrayInputStream::new(&hex(t[1]))
                } else {
                    UnsynchronizedByteArrayInputStream::with_range(
                        &hex(t[1]),
                        parse(t[2]),
                        parse(t[3]),
                    )?
                };
            }
            "read" => return ok(self.input.read().to_string()),
            "readb" => {
                let mut dest = vec![0u8; parse(t[1])];
                let n = self.input.read_range(&mut dest, parse(t[2]), parse(t[3]))?;
                return ok(format!("n={n} dest={}", hexs(&dest)));
            }
            "avail" => return ok(self.input.available().to_string()),
            "skip" => return ok(self.input.skip(parse(t[1]))?.to_string()),
            "mark" => self.input.mark(parse(t[1])),
            "reset" => self.input.reset(),
            "marksup" => return ok(flag(self.input.mark_supported())),
            "readall" => return ok(hexs(&self.input.read_all_bytes())),
            "readn" => return ok(hexs(&self.input.read_n_bytes(parse(t[1]))?)),
            "readnb" => {
                let mut dest = vec![0u8; parse(t[1])];
                let n = self
                    .input
                    .read_n_bytes_into(&mut dest, parse(t[2]), parse(t[3]))?;
                return ok(format!("n={n} dest={}", hexs(&dest)));
            }
            "skipn" => self.input.skip_n_bytes(parse(t[1]))?,
            "transfer" => {
                let mut sink = Vec::new();
                let n = self
                    .input
                    .transfer_to(&mut sink)
                    .expect("a Vec never fails");
                return ok(format!("n={n} out={}", hexs(&sink)));
            }
            other => panic!("unknown op {other}"),
        }
        ok("ok")
    }

    fn random(&mut self, r: &mut Rng) -> Vec<String> {
        match r.below(4) {
            0 => words(&["read"]),
            1 => {
                let off = r.below(3).to_string();
                let len = r.below(4).to_string();
                words(&["readb", "5", &off, &len])
            }
            2 => words(&["skip", &r.below(4).to_string()]),
            _ => words(&["avail"]),
        }
    }

    fn epilogue(&self) -> Vec<Vec<String>> {
        vec![words(&["read"]), words(&["avail"])]
    }
}
