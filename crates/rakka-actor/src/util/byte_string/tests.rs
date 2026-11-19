// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/util/{ByteStringSpec,ByteStringBuilderSpec,ByteIteratorSpec,ByteStringInputStreamSpec}.scala and actor-tests/src/test/scala/org/apache/pekko/io/UnsynchronizedByteArrayInputStreamSpec.scala (Apache-2.0).
//! Ports of the Pekko 2.0.0 specs of the byte-string family (`ByteStringSpec`, `ByteStringBuilderSpec`,
//! `ByteIteratorSpec`, `ByteStringInputStreamSpec`, `UnsynchronizedByteArrayInputStreamSpec`) and the
//! checks of the Rust-only conveniences. The Scala properties run here over inputs from a fixed
//! xorshift generator, so a failure always reproduces.

use std::io::{Read, Write};

use bytes::{Bytes, BytesMut};

use super::*;
use crate::util::swar::ByteOrder::{BigEndian, LittleEndian};
use crate::util::test_random::XorshiftStar;

struct Generator(XorshiftStar);

impl Generator {
    fn new(seed: u64) -> Self {
        Self(XorshiftStar::new(seed))
    }

    fn next(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn below(&mut self, bound: usize) -> usize {
        self.0.below(bound as u64) as usize
    }

    fn bytes(&mut self, max_len: usize) -> Vec<u8> {
        let len = self.below(max_len + 1);
        (0..len).map(|_| self.next() as u8).collect()
    }

    fn int(&mut self) -> i32 {
        match self.below(6) {
            0 => i32::MIN,
            1 => i32::MAX,
            2 => -1,
            _ => self.below(40) as i32 - 5,
        }
    }
}

fn bs(text: &str) -> ByteString {
    ByteString::from_string(text)
}

fn text(value: &ByteString) -> String {
    value.utf8_string()
}

/// A value built by concatenating several fragments, as the Scala generators do.
fn fragments(parts: &[&[u8]]) -> ByteString {
    parts.iter().fold(ByteString::empty(), |acc, p| {
        acc.concat(&ByteString::from_array(p))
    })
}

// ---------------------------------------------------------------- ByteStringSpec: trimming

#[test]
fn drop_take_and_drop_right_saturate_on_one_fragment() {
    let empty = ByteString::empty();
    assert_eq!(empty.drop(-1), bs(""));
    assert_eq!(bs("abc").drop(1), bs("bc"));
    assert_eq!(bs("0123456789").drop(5).take(4).drop(1).take(2), bs("67"));
    let ten = bs("0123456789");
    for n in [i32::MIN, -1, 0] {
        assert_eq!(ten.drop(n), ten);
        assert_eq!(ten.drop_right(n), ten);
        assert_eq!(ten.take(n), ByteString::empty());
    }
    for n in [10, 11, i32::MAX] {
        assert_eq!(ten.drop(n), ByteString::empty());
        assert_eq!(ten.drop_right(n), ByteString::empty());
        assert_eq!(ten.take(n), ten);
    }
    assert_eq!(text(&ten.drop_right(3)), "0123456");
    assert_eq!(text(&ten.take_right(3)), "789");
    assert_eq!(
        ten.take_right(i32::MIN),
        ten,
        "length - Int.MinValue overflows to the whole value"
    );
}

#[test]
fn drop_and_take_on_concatenations_and_slices() {
    let rope = fragments(&[b"01", b"23", b"45", b"67", b"89"]);
    assert_eq!(text(&rope.drop(5).take(2)), "56");
    assert_eq!(text(&rope.drop(i32::MIN)), "0123456789");
    assert!(rope.drop(i32::MAX).is_empty());
    assert_eq!(text(&rope.drop_right(4)), "012345");
    assert_eq!(text(&rope.take(100)), "0123456789");
    assert!(rope.take(-1).is_empty());
}

#[test]
fn slice_patterns_clamp_and_empty_when_from_is_after_until() {
    let ten = bs("0123456789");
    assert_eq!(ten.slice(-10, 10), ten);
    assert!(ten.slice(-10, 0).is_empty());
    assert!(ten.slice(1, -2).is_empty());
    assert!(ten.slice(1, 0).is_empty());
    assert!(ten.slice(-100, -10).is_empty());
    assert_eq!(text(&ten.slice(2, 4)), "23");
    assert!(ten.slice(4, 2).is_empty());
    assert!(ten.slice(10, 12).is_empty());
    assert_eq!(text(&ten.slice(-1, i32::MAX)), "0123456789");
    assert_eq!(text(&ten.slice(3, 100)), "3456789");
}

#[test]
fn sizes_and_sequencing_of_concatenations() {
    let mut g = Generator::new(0x1234);
    for _ in 0..200 {
        let (a, b) = (g.bytes(20), g.bytes(20));
        let (x, y) = (ByteString::from_array(&a), ByteString::from_array(&b));
        let joined = x.concat(&y);
        assert_eq!(joined.len(), a.len() + b.len());
        assert_eq!(joined.take(x.len() as i32), x);
        assert_eq!(joined.drop(x.len() as i32), y);
        assert_eq!(joined.take_right(y.len() as i32), y);
        let sandwich = y.concat(&x).concat(&y);
        assert_eq!(sandwich.drop(y.len() as i32).take(x.len() as i32), x);
        let (left, right) = joined.split_at(i32::MAX);
        assert_eq!(left.concat(&right), joined);
        let (left, right) = joined.split_at(i32::MIN);
        assert_eq!(left.concat(&right), joined);
    }
}

#[test]
fn behaves_like_a_vector() {
    let mut g = Generator::new(77);
    for _ in 0..300 {
        let data = g.bytes(30);
        let value = ByteString::from_array(&data);
        assert_eq!(value.to_vec(), data);
        assert_eq!(value.iter().collect::<Vec<_>>(), data);
        assert_eq!(value.len(), data.len());
        let (n, m) = (g.int(), g.int());
        let start = n.clamp(0, data.len() as i32) as usize;
        assert_eq!(value.drop(n).to_vec(), data[start..]);
        let upto = n.clamp(0, data.len() as i32) as usize;
        assert_eq!(value.take(n).to_vec(), data[..upto]);
        let from = n.clamp(0, data.len() as i32) as usize;
        let until = m.clamp(0, data.len() as i32) as usize;
        let expected: Vec<u8> = if from < until {
            data[from..until].to_vec()
        } else {
            vec![]
        };
        assert_eq!(value.slice(n, m).to_vec(), expected);
        if let Some(&first) = data.first() {
            assert_eq!(value.head(), Ok(first));
            assert_eq!(value.last(), Ok(*data.last().unwrap()));
            assert_eq!(value.tail().to_vec(), data[1..]);
            assert_eq!(value.init().to_vec(), data[..data.len() - 1]);
        }
        let limit = 128;
        let (prefix, rest) = value.span(|b| b < limit);
        let count = data.iter().take_while(|&&b| b < limit).count();
        assert_eq!(prefix.to_vec(), data[..count]);
        assert_eq!(rest.to_vec(), data[count..]);
        assert_eq!(value.take_while(|b| b < limit), prefix);
        assert_eq!(value.drop_while(|b| b < limit), rest);
        let sum: u32 = value.iter().map(u32::from).sum();
        assert_eq!(sum, data.iter().map(|&b| u32::from(b)).sum::<u32>());
        let size = g.below(7) as i32 + 1;
        let groups = value.grouped(size).unwrap();
        let expected: Vec<Vec<u8>> = data.chunks(size as usize).map(<[u8]>::to_vec).collect();
        assert_eq!(
            groups.iter().map(ByteString::to_vec).collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn apply_head_and_last_fail_outside_the_value() {
    let value = bs("abc");
    assert_eq!(value.apply(2), Ok(b'c'));
    for index in [3, -1, i32::MAX, i32::MIN] {
        let error = value.apply(index).unwrap_err();
        assert_eq!(
            error,
            ByteStringError::IndexOutOfBounds(Some(index.to_string()))
        );
        assert_eq!(error.java_class(), "java.lang.IndexOutOfBoundsException");
        assert_eq!(error.to_string(), index.to_string());
    }
    assert!(ByteString::empty().head().is_err());
    assert!(ByteString::empty().last().is_err());
    assert!(ByteString::empty().tail().is_empty());
    assert!(ByteString::empty().init().is_empty());
}

#[test]
fn grouped_rejects_a_size_that_is_not_positive() {
    let error = bs("abc").grouped(0).unwrap_err();
    assert_eq!(error.message(), Some("size=0 must be positive"));
    assert_eq!(error.java_class(), "java.lang.IllegalArgumentException");
    assert_eq!(
        bs("abc").grouped(-3).unwrap_err().to_string(),
        "size=-3 must be positive"
    );
    assert!(ByteString::empty().grouped(2).unwrap().is_empty());
}

#[test]
fn display_cuts_after_one_hundred_bytes() {
    assert_eq!(ByteString::empty().to_string(), "ByteString()");
    assert_eq!(
        ByteString::from_array(&[1, 0xff, 0x80]).to_string(),
        "ByteString(1, -1, -128)"
    );
    let long = ByteString::from_array(&[0; 101]);
    assert_eq!(
        long.to_string(),
        format!("ByteString({})... and [1] more", ["0"; 100].join(", "))
    );
    assert_eq!(format!("{:?}", bs("a")), "ByteString(97)");
}

#[test]
fn equality_hash_and_conversions() {
    use std::collections::HashSet;
    let a = bs("abc");
    let rope = fragments(&[b"a", b"bc"]);
    assert_eq!(a, rope);
    assert_ne!(a, bs("abd"));
    let set: HashSet<ByteString> = [a.clone(), rope.clone()].into_iter().collect();
    assert_eq!(set.len(), 1);
    assert_eq!(ByteString::from(vec![97u8, 98, 99]), a);
    assert_eq!(ByteString::from(&b"abc"[..]), a);
    assert_eq!(ByteString::from("abc"), a);
    assert_eq!(ByteString::from(Bytes::from_static(b"abc")), a);
    assert_eq!((b'a'..=b'c').collect::<ByteString>(), a);
    assert_eq!(a.as_ref(), b"abc");
    assert_eq!(a.to_bytes(), Bytes::from_static(b"abc"));
    assert_eq!(ByteString::from_array_unsafe(b"abc".to_vec()), a);
    assert_eq!(ByteString::UTF_8, "UTF-8");
    assert!(ByteString::default().is_empty());
}

#[test]
fn compact_keeps_the_content() {
    let rope = fragments(&[b"ab", b"c", b"de"]);
    let compact = rope.compact();
    assert_eq!(compact, rope);
    assert!(compact.is_compact());
    assert_eq!(compact.compact(), compact);
    let view = bs("0123456789").slice(2, 5);
    assert_eq!(view.compact().as_slice(), b"234");
}

// ---------------------------------------------------------------- factories and conversions

#[test]
fn from_array_range_saturates() {
    let array = [1, 2, 3, 4];
    assert_eq!(ByteString::from_array_range(&array, 2, 10).to_vec(), [3, 4]);
    assert_eq!(
        ByteString::from_array_range(&array, -2, 3).to_vec(),
        [1, 2, 3]
    );
    assert!(ByteString::from_array_range(&array, 4, 1).is_empty());
    assert!(ByteString::from_array_range(&array, 9, 2).is_empty());
    assert!(ByteString::from_array_range(&array, 1, -3).is_empty());
    assert_eq!(
        ByteString::from_array_range(&array, i32::MIN, i32::MAX).to_vec(),
        array
    );
}

#[test]
fn from_ints_truncates_and_from_buf_consumes() {
    assert_eq!(
        ByteString::from_ints(&[255, 256, -1, 128]).to_vec(),
        [255, 0, 255, 128]
    );
    let mut buffer = &b"\x01\x02\x03"[1..];
    assert_eq!(ByteString::from_buf(&mut buffer).to_vec(), [2, 3]);
    assert!(buffer.is_empty());
}

#[test]
fn strings_round_trip_through_every_charset_and_fail_on_unknown_names() {
    let text_value = "h\u{e9}llo \u{20ac} \u{1F600}";
    assert_eq!(bs(text_value).utf8_string(), text_value);
    assert_eq!(bs(text_value).decode_string("utf-8").unwrap(), text_value);
    assert_eq!(
        ByteString::from_string_with_charset("\u{e9}\u{20ac}", "latin1")
            .unwrap()
            .to_vec(),
        [0xe9, b'?']
    );
    assert_eq!(
        ByteString::from_string_in("a\u{e9}\u{1F600}", Charset::UsAscii).to_vec(),
        b"a??"
    );
    assert_eq!(
        ByteString::from_array(&[0xe9, 0x80]).decode_string_in(Charset::Iso8859_1),
        "\u{e9}\u{80}"
    );
    assert_eq!(
        ByteString::from_array(&[b'a', 0xe9])
            .decode_string("US-ASCII")
            .unwrap(),
        "a\u{fffd}"
    );
    let error = bs("a").decode_string("nope").unwrap_err();
    assert_eq!(
        error,
        ByteStringError::UnsupportedEncoding("nope".to_owned())
    );
    assert_eq!(error.java_class(), "java.io.UnsupportedEncodingException");
    assert!(ByteString::from_string_with_charset("a", "nope").is_err());
    for (name, charset) in [
        ("UTF8", Charset::Utf8),
        ("ascii", Charset::UsAscii),
        ("ISO646-US", Charset::UsAscii),
        ("ISO8859_1", Charset::Iso8859_1),
        ("L1", Charset::Iso8859_1),
    ] {
        assert_eq!(Charset::for_name(name), Some(charset));
    }
}

#[test]
fn invalid_utf8_is_replaced_like_the_jvm_does() {
    let cases: [(&[u8], &str); 8] = [
        (&[0xff, 0xfe], "\u{fffd}\u{fffd}"),
        (&[0xe2, 0x82], "\u{fffd}"),
        (&[0xed, 0xa0, 0x80], "\u{fffd}"),
        (&[0xed, 0xa0, 0x41], "\u{fffd}A"),
        (&[0xed, 0xbf], "\u{fffd}"),
        (&[0xed, 0xa0, 0xa0, 0xa0], "\u{fffd}\u{fffd}"),
        (
            &[0xf4, 0x90, 0x80, 0x80],
            "\u{fffd}\u{fffd}\u{fffd}\u{fffd}",
        ),
        (&[b'a', 0xe2, 0x82, 0xac], "a\u{20ac}"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            ByteString::from_array(input).utf8_string(),
            expected,
            "{input:x?}"
        );
    }
}

#[test]
fn decode_string_range_is_clamped() {
    let value = bs("hello world");
    assert_eq!(value.decode_string_range(Charset::Utf8, 0, 5), "hello");
    assert_eq!(value.decode_string_range(Charset::Utf8, -3, 4), "hell");
    assert_eq!(value.decode_string_range(Charset::Utf8, 6, 100), "world");
    assert_eq!(value.decode_string_range(Charset::Utf8, 5, 2), "");
    assert_eq!(value.decode_string_range(Charset::Utf8, 3, 3), "");
}

#[test]
fn base64_round_trips_and_reports_malformed_input() {
    let mut g = Generator::new(5);
    for _ in 0..200 {
        let value = ByteString::from_array(&g.bytes(40));
        let encoded = value.encode_base64();
        assert_eq!(encoded.decode_base64().unwrap(), value);
        assert!(encoded.len().is_multiple_of(4));
    }
    assert_eq!(bs("a").encode_base64(), bs("YQ=="));
    assert_eq!(bs("ab").encode_base64(), bs("YWI="));
    assert_eq!(bs("YQ").decode_base64().unwrap(), bs("a"));
    assert_eq!(bs("YR==").decode_base64().unwrap(), bs("a"));
    let failures = [
        (
            "Y",
            "Input byte[] should at least have 2 bytes for base64 bytes",
        ),
        ("a!", "Illegal base64 character 21"),
        ("YQ=", "Input byte array has wrong 4-byte ending unit"),
        ("====", "Input byte array has wrong 4-byte ending unit"),
        (
            "YQ==YQ==",
            "Input byte array has incorrect ending byte at 4",
        ),
        ("YWJjY", "Last unit does not have enough valid bits"),
        ("Y=Q=", "Last unit does not have enough valid bits"),
    ];
    for (input, message) in failures {
        let error = bs(input).decode_base64().unwrap_err();
        assert_eq!(error.message(), Some(message), "{input}");
        assert_eq!(error.java_class(), "java.lang.IllegalArgumentException");
    }
    let high = ByteString::from_array(&[0xda, b'A'])
        .decode_base64()
        .unwrap_err();
    assert_eq!(high.message(), Some("Illegal base64 character -26"));
    assert!(ByteString::empty().decode_base64().unwrap().is_empty());
    assert!(ByteString::empty().encode_base64().is_empty());
}

#[test]
fn copy_to_array_and_buffer_follow_the_table_of_the_spec() {
    let value = bs("01234");
    let table: [(usize, usize, i32, usize); 8] = [
        (4, 0, 2, 2),
        (4, 0, 10, 4),
        (4, 2, 10, 2),
        (4, 4, 1, 0),
        (4, 5, 1, 0),
        (4, 1, -1, 0),
        (10, 6, 100, 4),
        (0, 0, 5, 0),
    ];
    for (size, start, len, copied) in table {
        let mut destination = vec![0u8; size];
        assert_eq!(value.copy_to_array(&mut destination, start, len), copied);
        assert_eq!(
            &destination[start.min(size)..start.min(size) + copied],
            &value.as_slice()[..copied]
        );
    }
    let mut small = BytesMut::zeroed(2);
    let mut window = &mut small[..];
    assert_eq!(value.copy_to_buffer(&mut window), 2);
    assert_eq!(&small[..], b"01");
    let mut growing = Vec::new();
    assert_eq!(value.copy_to_buffer(&mut growing), 5);
    assert_eq!(growing, b"01234");
}

#[test]
fn map_i_truncates_to_bytes() {
    assert_eq!(
        bs("abc").map_i(|b| i32::from(b) + 300).to_vec(),
        [97 + 44, 98 + 44, 99 + 44]
    );
}

#[test]
fn concat_with_an_empty_side_keeps_the_other_value() {
    let a = bs("ab");
    assert_eq!(a.concat(&ByteString::empty()), a);
    assert_eq!(ByteString::empty().concat(&a), a);
    assert_eq!(a.concat(&bs("c")), bs("abc"));
}

// ---------------------------------------------------------------- search

#[test]
fn index_of_family_over_slices_and_concatenations() {
    let rope = fragments(&[b"abc", b"efg"]);
    assert_eq!(rope.index_of_from(b'c', 3), None);
    assert_eq!(rope.index_of_from(b'g', 5), Some(5));
    assert_eq!(rope.index_of(b'e'), Some(3));
    assert_eq!(rope.index_of_range(b'a', 0, 0), None);
    assert_eq!(rope.index_of_range(b'a', -1, 1), Some(0));
    assert_eq!(rope.index_of_range(b'c', 2, 3), Some(2));
    assert_eq!(rope.index_of_range(b'c', 2, 2), None);
    assert_eq!(rope.index_of_range(b'g', 0, 5), None);
    assert_eq!(rope.index_of_range(b'a', 0, i32::MAX), Some(0));
    assert_eq!(rope.index_of_range(b'g', 0, i32::MIN), None);
    assert_eq!(ByteString::empty().index_of(b'a'), None);
    let view = bs("abcdefgh").slice(2, 7);
    assert_eq!(view.index_of(b'c'), Some(0));
    assert_eq!(view.index_of(b'h'), None);
    assert!(view.contains(b'g'));
    assert!(!view.contains(b'a'));
    let repeated = bs("abcab");
    assert_eq!(repeated.last_index_of(b'a'), Some(3));
    assert_eq!(repeated.last_index_of_end(b'a', 2), Some(0));
    assert_eq!(repeated.last_index_of_end(b'b', -1), None);
    assert_eq!(repeated.last_index_of_end(b'b', i32::MAX), Some(4));
    assert_eq!(ByteString::empty().last_index_of(b'a'), None);
}

#[test]
fn slices_are_found_prefixes_and_suffixes_match() {
    let value = bs("abcabcd");
    assert_eq!(value.index_of_slice(b"", 0), Some(0));
    assert_eq!(value.index_of_slice(b"", 100), None);
    assert_eq!(value.index_of_slice(b"abc", 1), Some(3));
    assert_eq!(value.index_of_slice(b"abc", 4), None);
    assert_eq!(value.index_of_slice(b"d", -3), Some(6));
    assert_eq!(value.index_of_slice(b"abcabcdx", 0), None);
    assert_eq!(value.index_of_slice(b"bcd", 0), Some(4));
    assert!(value.contains_slice(b"cab"));
    assert!(!value.contains_slice(b"cd0"));
    assert_eq!(value.last_index_of_slice(b"abc", 7), Some(3));
    assert_eq!(value.last_index_of_slice(b"abc", 2), Some(0));
    assert_eq!(value.last_index_of_slice(b"abc", -1), None);
    assert_eq!(value.last_index_of_slice(b"", 3), Some(3));
    assert_eq!(value.last_index_of_slice(b"", -1), None);
    assert_eq!(value.last_index_of_slice(b"d", 100), Some(6));
    assert_eq!(value.last_index_of_slice(b"abcabcdx", 100), None);
    assert!(value.starts_with(b"abc"));
    assert!(value.starts_with_at(b"cab", 2));
    assert!(!value.starts_with_at(b"abc", 5));
    assert!(value.starts_with_at(b"", 7));
    assert!(!value.starts_with_at(b"", 8));
    assert!(value.ends_with(b"bcd"));
    assert!(!value.ends_with(b"abcabcdd"));
    assert!(value.ends_with(b""));
}

#[test]
fn searches_agree_with_the_standard_library() {
    let mut g = Generator::new(99);
    for _ in 0..300 {
        let data: Vec<u8> = (0..g.below(25)).map(|_| g.below(4) as u8).collect();
        let needle: Vec<u8> = (0..g.below(4)).map(|_| g.below(4) as u8).collect();
        let value = ByteString::from_array(&data);
        let from = g.below(30) as i32 - 3;
        let from_index = from.max(0) as usize;
        let expected = if needle.is_empty() {
            (from_index <= data.len()).then_some(from_index)
        } else {
            (from_index..data.len()).find(|&i| data[i..].starts_with(&needle))
        };
        assert_eq!(
            value.index_of_slice(&needle, from),
            expected,
            "{data:?} {needle:?} {from}"
        );
        let end = g.below(30) as i32 - 3;
        let expected_last = if needle.is_empty() {
            (end >= 0).then(|| (end as usize).min(data.len()))
        } else {
            (0..=data
                .len()
                .saturating_sub(needle.len())
                .min(end.max(-1) as usize))
                .rev()
                .find(|&i| end >= 0 && data.len() >= needle.len() && data[i..].starts_with(&needle))
        };
        assert_eq!(
            value.last_index_of_slice(&needle, end),
            expected_last,
            "{data:?} {needle:?} {end}"
        );
    }
}

// ---------------------------------------------------------------- ByteStringBuilderSpec

#[test]
fn builder_appends_collections_and_byte_strings() {
    let mut builder = ByteStringBuilder::new();
    assert!(builder.is_empty());
    assert!(!builder.non_empty());
    builder.add_all(vec![b'a']);
    builder.append(&ByteString::empty());
    assert_eq!(builder.result(), bs("a"));
    builder.append(&bs("bc"));
    builder.add_one(b'd');
    builder.put_byte(b'e');
    builder.put_bytes(b"fg");
    assert_eq!(builder.len(), 7);
    assert!(builder.non_empty());
    assert_eq!(text(&builder.result()), "abcdefg");
    builder.size_hint(100);
    builder.size_hint(-5);
    builder.clear();
    assert!(builder.is_empty());
    assert_eq!(builder.result(), ByteString::empty());
    assert_eq!(ByteString::new_builder().result(), ByteString::empty());
}

#[test]
fn builder_writes_primitives_like_a_byte_buffer() {
    let mut g = Generator::new(31);
    for _ in 0..100 {
        let (short, int, long) = (g.next() as i16, g.next() as i32, g.next() as i64);
        let (float, double) = (f32::from_bits(g.next() as u32), f64::from_bits(g.next()));
        let mut be = ByteStringBuilder::new();
        let mut le = ByteStringBuilder::new();
        be.put_short(i32::from(short), BigEndian);
        le.put_short(i32::from(short), LittleEndian);
        be.put_int(int, BigEndian);
        le.put_int(int, LittleEndian);
        be.put_long(long, BigEndian);
        le.put_long(long, LittleEndian);
        be.put_float(float, BigEndian);
        le.put_float(float, LittleEndian);
        be.put_double(double, BigEndian);
        le.put_double(double, LittleEndian);
        let mut expected_be = Vec::new();
        expected_be.extend(short.to_be_bytes());
        expected_be.extend(int.to_be_bytes());
        expected_be.extend(long.to_be_bytes());
        expected_be.extend(float.to_bits().to_be_bytes());
        expected_be.extend(double.to_bits().to_be_bytes());
        let mut expected_le = Vec::new();
        expected_le.extend(short.to_le_bytes());
        expected_le.extend(int.to_le_bytes());
        expected_le.extend(long.to_le_bytes());
        expected_le.extend(float.to_bits().to_le_bytes());
        expected_le.extend(double.to_bits().to_le_bytes());
        assert_eq!(be.result().to_vec(), expected_be);
        assert_eq!(le.result().to_vec(), expected_le);
    }
}

#[test]
fn put_long_part_writes_the_low_bytes_and_wraps_its_shifts() {
    let mut builder = ByteStringBuilder::new();
    builder.put_long_part(0x01_0203_0405, 3, BigEndian);
    builder.put_long_part(0x01_0203_0405, 3, LittleEndian);
    builder.put_long_part(7, 0, BigEndian);
    builder.put_long_part(7, -2, LittleEndian);
    assert_eq!(builder.result().to_vec(), [3, 4, 5, 5, 4, 3]);
    let mut wide = ByteStringBuilder::new();
    wide.put_long_part(0x0102_0304_0506_0708, 9, BigEndian);
    assert_eq!(wide.result().to_vec(), [8, 1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn builder_arrays_and_ranges() {
    let mut builder = ByteStringBuilder::new();
    builder.put_shorts(&[1, -2], BigEndian);
    builder.put_ints(&[1], LittleEndian);
    builder.put_longs(&[1], BigEndian);
    builder.put_floats(&[1.0], BigEndian);
    builder.put_doubles(&[1.0], LittleEndian);
    assert_eq!(builder.len(), 4 + 4 + 8 + 4 + 8);
    let mut ranged = ByteStringBuilder::new();
    ranged
        .put_ints_range(&[10, 20, 30, 40], 1, 2, BigEndian)
        .unwrap();
    assert_eq!(ranged.result().to_vec(), [0, 0, 0, 20, 0, 0, 0, 30]);
    ranged.clear();
    ranged
        .put_shorts_range(&[1, 2, 3], 2, 1, LittleEndian)
        .unwrap();
    ranged.put_longs_range(&[9], 0, 1, LittleEndian).unwrap();
    ranged
        .put_floats_range(&[1.0, 2.0], 1, 1, BigEndian)
        .unwrap();
    ranged.put_doubles_range(&[1.0], 0, 0, BigEndian).unwrap();
    assert_eq!(ranged.len(), 2 + 8 + 4);
    let error = ranged.put_ints_range(&[1, 2], 1, 5, BigEndian).unwrap_err();
    assert_eq!(
        error.java_class(),
        "java.lang.ArrayIndexOutOfBoundsException"
    );
    assert!(ranged.put_ints_range(&[1, 2], -1, 1, BigEndian).is_err());
}

#[test]
fn builder_byte_ranges_and_the_output_stream() {
    let mut builder = ByteStringBuilder::new();
    builder.put_bytes_range(&[1, 2, 3, 4, 5], 1, 3).unwrap();
    builder.put_bytes_range(&[1, 2, 3], 3, 0).unwrap();
    assert_eq!(builder.result().to_vec(), [2, 3, 4]);
    for (start, len) in [(2, 5), (-1, 1), (0, -1), (i32::MAX, i32::MAX)] {
        let error = builder.put_bytes_range(&[1, 2, 3], start, len).unwrap_err();
        assert_eq!(
            error.java_class(),
            "java.lang.ArrayIndexOutOfBoundsException"
        );
    }
    assert_eq!(builder.len(), 3);
    builder.write_int(300);
    builder.write_int(-1);
    assert_eq!(builder.write(b"xy").unwrap(), 2);
    builder.flush().unwrap();
    assert_eq!(builder.result().to_vec(), [2, 3, 4, 44, 255, b'x', b'y']);
    let copy = builder.clone();
    assert_eq!(copy.result(), builder.result());
    assert!(format!("{builder:?}").contains("ByteStringBuilder"));
}

// ---------------------------------------------------------------- ByteIteratorSpec

fn array_iterator(bytes: &[u8]) -> ByteIterator {
    ByteString::from_array(bytes).iterator()
}

#[test]
fn index_of_matches_the_spec() {
    let fresh = || array_iterator(&[0x20, 0x20, 0x10, 0x20, 0x20, 0x10]);
    assert_eq!(fresh().index_of(0x20, 0), Some(0));
    assert_eq!(fresh().index_of(0x10, 0), Some(2));
    assert_eq!(fresh().index_of(0x20, 1), Some(1));
    assert_eq!(fresh().index_of(0x10, 1), Some(2));
    assert_eq!(fresh().index_of(0x10, 3), Some(5));
    assert_eq!(fresh().index_of(0x10, -4), Some(2));
    assert_eq!(fresh().index_of(0x99, 0), None);
    let mut consumed = fresh();
    consumed.index_of(0x10, 0);
    assert!(consumed.is_empty());
}

#[test]
fn reading_across_every_split_matches_the_array_reader() {
    let data: Vec<u8> = (0..16u32).map(|i| ((i * 31 + 7) & 0xff) as u8).collect();
    for split in 1..data.len() {
        let joined =
            ByteString::from_array(&data[..split]).concat(&ByteString::from_array(&data[split..]));
        for order in [BigEndian, LittleEndian] {
            for offset in 0..=data.len() - 8 {
                let mut reference = array_iterator(&data);
                reference.drop(offset as i32);
                let mut combined = joined.iterator();
                combined.drop(offset as i32);
                assert_eq!(
                    combined.clone().get_short(order),
                    reference.clone().get_short(order)
                );
                assert_eq!(
                    combined.clone().get_int(order),
                    reference.clone().get_int(order)
                );
                assert_eq!(combined.get_long(order), reference.get_long(order));
            }
        }
    }
}

#[test]
fn iterator_reads_primitives_in_both_orders() {
    let mut g = Generator::new(8);
    for _ in 0..100 {
        let data: Vec<u8> = (0..27).map(|_| g.next() as u8).collect();
        for order in [BigEndian, LittleEndian] {
            let mut it = array_iterator(&data);
            let mut slice = &data[..];
            let take = |slice: &mut &[u8], n: usize| -> Vec<u8> {
                let head = slice[..n].to_vec();
                *slice = &slice[n..];
                head
            };
            let from = |bytes: Vec<u8>| -> Vec<u8> {
                if order == BigEndian {
                    bytes
                } else {
                    bytes.into_iter().rev().collect()
                }
            };
            assert_eq!(it.get_byte(), Ok(take(&mut slice, 1)[0]));
            let short = from(take(&mut slice, 2));
            assert_eq!(
                it.get_short(order),
                Ok(i16::from_be_bytes(short.try_into().unwrap()))
            );
            let int = from(take(&mut slice, 4));
            assert_eq!(
                it.get_int(order),
                Ok(i32::from_be_bytes(int.try_into().unwrap()))
            );
            let long = from(take(&mut slice, 8));
            assert_eq!(
                it.get_long(order),
                Ok(i64::from_be_bytes(long.try_into().unwrap()))
            );
            let float = from(take(&mut slice, 4));
            assert_eq!(
                it.get_float(order).map(f32::to_bits),
                Ok(u32::from_be_bytes(float.try_into().unwrap()))
            );
            let double = from(take(&mut slice, 8));
            assert_eq!(
                it.get_double(order).map(f64::to_bits),
                Ok(u64::from_be_bytes(double.try_into().unwrap()))
            );
            assert!(it.is_empty());
        }
    }
}

#[test]
fn get_long_part_reads_a_variable_number_of_bytes() {
    assert_eq!(
        array_iterator(&[1, 2, 3]).get_long_part(3, BigEndian),
        Ok(66051)
    );
    assert_eq!(
        array_iterator(&[1, 2, 3]).get_long_part(3, LittleEndian),
        Ok(197121)
    );
    let mut it = array_iterator(&[1, 2]);
    assert_eq!(it.get_long_part(0, BigEndian), Ok(0));
    assert_eq!(it.get_long_part(-1, LittleEndian), Ok(0));
    assert_eq!(it.len(), 2);
    assert_eq!(
        array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]).get_long_part(8, LittleEndian),
        Ok(201_786_885)
    );
    assert_eq!(
        array_iterator(&[0xff, 0xff]).get_long_part(2, BigEndian),
        Ok(65535)
    );
    let mut short = array_iterator(&[1, 2]);
    assert!(short.get_long_part(3, BigEndian).is_err());
    assert!(
        short.is_empty(),
        "get_long_part consumes what there is before it fails"
    );
}

#[test]
fn exhausted_iterators_fail_with_the_message_of_the_empty_scala_iterator() {
    let mut it = array_iterator(&[1]);
    assert_eq!(it.head(), Some(1));
    assert_eq!(it.next_byte(), Ok(1));
    let error = it.next_byte().unwrap_err();
    assert_eq!(
        error,
        ByteStringError::NoSuchElement("next on empty iterator".to_owned())
    );
    assert_eq!(error.java_class(), "java.util.NoSuchElementException");
    assert_eq!(it.head(), None);
    assert!(!it.has_next());
    let mut short = array_iterator(&[1, 2]);
    assert!(short.get_int(BigEndian).is_err());
    assert_eq!(
        short.len(),
        2,
        "a primitive read that finds too few bytes consumes none"
    );
    assert_eq!(ByteIterator::default().size(), 0);
}

#[test]
fn iterator_trimming_mutates_the_cursor() {
    let mut it = array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]);
    it.take(5);
    assert_eq!(it.len(), 5);
    it.take(7);
    assert_eq!(it.len(), 5);
    it.take(2);
    assert_eq!(it.to_byte_string().to_vec(), [1, 2]);
    let mut it = array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]);
    it.drop(-3);
    assert_eq!(it.len(), 8);
    it.drop(2);
    assert_eq!(it.len(), 6);
    it.drop(100);
    assert!(it.is_empty());
    let mut it = array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]);
    it.slice(2, 5);
    assert_eq!(it.to_byte_string().to_vec(), [3, 4, 5]);
    for (from, until, left) in [(5, 2, 0), (-3, 100, 8), (3, 3, 0), (i32::MIN, 2, 2)] {
        let mut it = array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]);
        it.slice(from, until);
        assert_eq!(it.len(), left, "{from}..{until}");
    }
    let mut it = array_iterator(&[1, 2, 3, 4, 5]);
    assert_eq!(it.size(), 5);
    assert!(it.is_empty());
    assert_eq!(it.bytes().count(), 0);
    let mut it = array_iterator(&[9, 8]);
    assert_eq!(it.bytes().collect::<Vec<_>>(), [9, 8]);
}

#[test]
fn iterator_predicates_clone_and_duplicate() {
    let mut it = array_iterator(&[1, 2, 3, 4, 5]);
    it.take_while(|b| b < 3);
    assert_eq!(it.to_byte_string().to_vec(), [1, 2]);
    let mut it = array_iterator(&[1, 2, 3, 4, 5]);
    it.drop_while(|b| b < 3);
    assert_eq!(it.to_byte_string().to_vec(), [3, 4, 5]);
    let (mut front, mut back) = array_iterator(&[1, 2, 3, 4, 5]).span(|b| b < 3);
    assert_eq!(
        (
            front.to_byte_string().to_vec(),
            back.to_byte_string().to_vec()
        ),
        (vec![1, 2], vec![3, 4, 5])
    );
    let mut original = array_iterator(&[1, 2, 3]);
    let mut copy = original.clone();
    original.next_byte().unwrap();
    assert_eq!((original.len(), copy.len()), (2, 3));
    copy.next_byte().unwrap();
    let (mut first, second) = original.duplicate();
    first.next_byte().unwrap();
    assert_eq!((first.len(), second.len()), (1, 2));
}

#[test]
fn byte_arrays_and_byte_strings_are_read_with_the_spec_semantics() {
    let mut it = array_iterator(&[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(it.get_bytes(3), Ok(vec![1, 2, 3]));
    assert_eq!(it.get_bytes(0), Ok(vec![]));
    assert!(it.get_bytes(6).is_err());
    assert_eq!(
        it.len(),
        5,
        "a read that asks for too many bytes copies and consumes nothing"
    );
    let mut target = [0u8; 6];
    it.get_bytes_into(&mut target, 1, 4).unwrap();
    assert_eq!(target, [0, 4, 5, 6, 7, 0]);
    let error = it.get_bytes_into(&mut target, 6, 1).unwrap_err();
    assert_eq!(
        error.java_class(),
        "java.lang.ArrayIndexOutOfBoundsException"
    );
    assert_eq!(it.get_byte_string(5).to_vec(), [8]);
    let mut it = array_iterator(&[1, 2, 3]);
    assert_eq!(it.get_byte_string(2).to_vec(), [1, 2]);
    assert_eq!(it.get_byte_string(-1), ByteString::empty());
    assert_eq!(it.get_byte_string(9).to_vec(), [3]);
}

#[test]
fn arrays_of_primitives_are_read_whole_or_not_at_all() {
    let mut it = array_iterator(&[0, 1, 0, 2, 0xff, 0xfe]);
    let mut shorts = [0i16; 3];
    it.get_shorts(&mut shorts, BigEndian).unwrap();
    assert_eq!(shorts, [1, 2, -2]);
    let mut it = array_iterator(&[0, 0, 0, 1, 0, 0, 0, 2]);
    let mut ints = [0i32; 2];
    it.get_ints(&mut ints, BigEndian).unwrap();
    assert_eq!(ints, [1, 2]);
    let mut it = array_iterator(&[1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
    let mut longs = [0i64; 2];
    it.get_longs(&mut longs, LittleEndian).unwrap();
    assert_eq!(longs, [1, 2]);
    let mut it = array_iterator(&[0x3f, 0x80, 0, 0, 0x40, 0, 0, 0]);
    let mut floats = [0f32; 2];
    it.get_floats(&mut floats, BigEndian).unwrap();
    assert_eq!(floats, [1.0, 2.0]);
    let mut it = array_iterator(&[0x3f, 0xf0, 0, 0, 0, 0, 0, 0]);
    let mut doubles = [0f64; 1];
    it.get_doubles(&mut doubles, BigEndian).unwrap();
    assert_eq!(doubles, [1.0]);
    let mut short = array_iterator(&[0, 1]);
    assert!(short.get_shorts(&mut [0; 3], BigEndian).is_err());
    assert_eq!(short.len(), 2);
    let mut destination = [0i32; 2];
    let error = array_iterator(&[0; 16])
        .get_ints_range(&mut destination, 1, 2, BigEndian)
        .unwrap_err();
    assert_eq!(
        error.java_class(),
        "java.lang.ArrayIndexOutOfBoundsException"
    );
    array_iterator(&[0, 0, 0, 9])
        .get_ints_range(&mut destination, 1, 1, BigEndian)
        .unwrap();
    assert_eq!(destination, [0, 9]);
    assert!(
        array_iterator(&[0; 16])
            .get_longs_range(&mut [0; 1], usize::MAX, 2, BigEndian)
            .is_err()
    );
}

#[test]
fn copy_to_array_and_buffer_consume_what_they_copy() {
    let mut it = array_iterator(&[1, 2, 3, 4]);
    let mut destination = [0u8; 3];
    assert_eq!(it.copy_to_array(&mut destination, 2, 10), 1);
    assert_eq!(destination, [0, 0, 1]);
    assert_eq!(it.copy_to_array(&mut destination, 0, 10), 3);
    assert_eq!(destination, [2, 3, 4]);
    assert_eq!(it.copy_to_array(&mut destination, 9, 1), 0);
    assert_eq!(
        array_iterator(&[1, 2]).copy_to_array(&mut destination, 0, -1),
        0
    );
    let mut it = array_iterator(&[1, 2, 3]);
    let mut small = BytesMut::zeroed(2);
    let mut window = &mut small[..];
    assert_eq!(it.copy_to_buffer(&mut window), 2);
    assert_eq!(it.len(), 1);
    let mut growing = Vec::new();
    assert_eq!(it.copy_to_buffer(&mut growing), 1);
    assert_eq!(growing, [3]);
}

#[test]
fn iterator_input_stream_follows_the_pekko_adapter() {
    let mut stream = array_iterator(b"abc").into_input_stream();
    assert_eq!(stream.available(), 3);
    assert_eq!(stream.skip(2), 2);
    assert_eq!(stream.read(), 99);
    assert_eq!(stream.read(), -1);
    let mut stream = array_iterator(b"abc").into_input_stream();
    let mut destination = [0u8; 5];
    assert_eq!(stream.read_range(&mut destination, 1, 3), Ok(3));
    assert_eq!(destination, [0, 97, 98, 99, 0]);
    assert_eq!(stream.read_range(&mut destination, 0, 3), Ok(-1));
    assert_eq!(stream.read_range(&mut destination, 0, 0), Ok(0));
    for (off, len) in [(3, 3), (-1, 1), (0, -1), (i32::MAX, 1)] {
        assert_eq!(
            stream.read_range(&mut destination, off, len),
            Err(ByteStringError::IndexOutOfBounds(None))
        );
    }
    let mut stream = array_iterator(b"abc").into_input_stream();
    assert_eq!(stream.skip(100), 3);
    assert_eq!(stream.skip(-2), -2);
    assert_eq!(stream.skip(i64::from(u32::MAX) + 2), 0);
    let mut stream = array_iterator(b"hello").into_input_stream();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    assert_eq!(text, "hello");
}

// ---------------------------------------------------------------- input streams

fn read_everything(mut stream: impl Read) -> String {
    let mut out = String::new();
    stream.read_to_string(&mut out).unwrap();
    out
}

#[test]
fn byte_string_input_streams_read_every_shape_of_value() {
    assert_eq!(ByteString::empty().as_input_stream().read(), -1);
    let mut one = [1u8];
    assert_eq!(
        ByteString::empty()
            .as_input_stream()
            .read_range(&mut one, 0, 1),
        Ok(-1)
    );
    assert_eq!(read_everything(ByteString::empty().as_input_stream()), "");
    assert_eq!(read_everything(bs("abc").as_input_stream()), "abc");
    assert_eq!(
        read_everything(bs("abcdef").slice(1, 4).as_input_stream()),
        "bcd"
    );
    assert_eq!(
        read_everything(fragments(&[b"a", b"bc"]).as_input_stream()),
        "abc"
    );
}

#[test]
fn unsynchronized_stream_supports_mark_reset_and_skip() {
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abc");
    assert!(stream.mark_supported());
    assert_eq!(stream.read(), i32::from(b'a'));
    stream.mark(1);
    assert_eq!(stream.read(), i32::from(b'b'));
    stream.reset();
    assert_eq!(stream.read(), i32::from(b'b'));
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abc");
    assert_eq!(stream.skip(1), Ok(1));
    assert_eq!(stream.read(), i32::from(b'b'));
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abc");
    assert_eq!(stream.skip(50), Ok(3));
    assert_eq!(stream.available(), 0);
    assert_eq!(stream.read(), -1);
    assert_eq!(stream.skip(i64::MAX), Ok(0), "must not overflow");
    assert_eq!(
        UnsynchronizedByteArrayInputStream::new(b"abc")
            .skip(-1)
            .unwrap_err()
            .to_string(),
        "Skipping backward is not supported"
    );
    stream.reset();
    assert_eq!(stream.available(), 3);
}

#[test]
fn unsynchronized_stream_skip_n_bytes_and_read_n_bytes() {
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abcdef");
    stream.skip_n_bytes(0).unwrap();
    stream.skip_n_bytes(-1).unwrap();
    assert_eq!(stream.read(), i32::from(b'a'));
    stream.skip_n_bytes(2).unwrap();
    assert_eq!(stream.read(), i32::from(b'd'));
    let error = stream.skip_n_bytes(3).unwrap_err();
    assert_eq!(error, ByteStringError::EndOfFile);
    assert_eq!(
        (error.java_class(), error.message()),
        ("java.io.EOFException", None)
    );
    assert_eq!(error.to_string(), "");
    assert_eq!(stream.available(), 0);
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abcdef");
    assert_eq!(stream.read_n_bytes(2).unwrap(), b"ab");
    assert_eq!(stream.read_n_bytes(100).unwrap(), b"cdef");
    assert_eq!(stream.read_n_bytes(1).unwrap(), b"");
    assert_eq!(
        stream.read_n_bytes(-1).unwrap_err().message(),
        Some("len < 0")
    );
}

#[test]
fn unsynchronized_stream_reads_into_arrays() {
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abcdef");
    let mut buffer = [0u8; 4];
    assert_eq!(stream.read_range(&mut buffer, 0, 4), Ok(4));
    assert_eq!(&buffer, b"abcd");
    assert_eq!(stream.read_range(&mut buffer, 1, 0), Ok(0));
    assert_eq!(stream.read_range(&mut buffer, 1, 3), Ok(2));
    assert_eq!(&buffer, b"aefd");
    assert_eq!(stream.read_range(&mut buffer, 0, 4), Ok(-1));
    assert_eq!(stream.read_n_bytes_into(&mut buffer, 0, 4), Ok(0));
    assert_eq!(stream.read_range(&mut buffer, 0, 0), Ok(0));
    let error = stream.read_range(&mut buffer, 2, 3).unwrap_err();
    assert_eq!(
        error.message(),
        Some("Range [2, 2 + 3) out of bounds for length 4")
    );
    assert!(stream.read_range(&mut buffer, -1, 1).is_err());
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abcdef");
    assert_eq!(stream.read_n_bytes_into(&mut buffer, 0, 2), Ok(2));
    assert_eq!(stream.read_all_bytes(), b"cdef");
    assert_eq!(stream.read_all_bytes(), b"");
    let mut stream = UnsynchronizedByteArrayInputStream::new(b"abc");
    stream.read();
    let mut sink = Vec::new();
    assert_eq!(stream.transfer_to(&mut sink).unwrap(), 2);
    assert_eq!(sink, b"bc");
    assert_eq!(stream.transfer_to(&mut sink).unwrap(), 0);
    let mut stream = UnsynchronizedByteArrayInputStream::from_bytes(Bytes::from_static(b"xyz"));
    let mut two = [0u8; 2];
    assert_eq!(Read::read(&mut stream, &mut two).unwrap(), 2);
    assert_eq!(two, *b"xy");
    assert_eq!(stream.clone().read(), i32::from(b'z'));
}

#[test]
fn unsynchronized_stream_range_constructor() {
    let stream = UnsynchronizedByteArrayInputStream::with_range(&[1, 2, 3], 5, 2).unwrap();
    assert_eq!(stream.available(), 0);
    let mut stream = UnsynchronizedByteArrayInputStream::with_range(&[1, 2, 3], 1, 5).unwrap();
    assert_eq!(stream.available(), 2);
    stream.skip(1).unwrap();
    stream.mark(0);
    stream.read();
    stream.reset();
    assert_eq!(stream.read(), 3);
    assert_eq!(
        UnsynchronizedByteArrayInputStream::with_range(&[1], -1, 1)
            .unwrap_err()
            .to_string(),
        "offset cannot be negative"
    );
    assert_eq!(
        UnsynchronizedByteArrayInputStream::with_range(&[1], 0, -1)
            .unwrap_err()
            .to_string(),
        "length cannot be negative"
    );
    assert_eq!(
        UnsynchronizedByteArrayInputStream::with_range(&[], 5, 2)
            .unwrap()
            .available(),
        0
    );
    assert_eq!(
        UnsynchronizedByteArrayInputStream::with_range(&[1, 2], i32::MAX, i32::MAX)
            .unwrap()
            .available(),
        0
    );
}

#[test]
fn errors_convert_to_io_errors() {
    let error: std::io::Error = ByteStringError::IllegalArgument("bad".to_owned()).into();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(error.to_string(), "bad");
}
