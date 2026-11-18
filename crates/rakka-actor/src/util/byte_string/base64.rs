// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! The basic Base64 alphabet (RFC 4648 section 4) with padding on output; the decoder accepts missing
//! padding, as `java.util.Base64.getDecoder` does, and fails with the messages the oracle records.

use super::ByteStringError;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let group = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, &b)| acc | (u32::from(b) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[((group >> (18 - 6 * i)) & 0x3F) as usize]);
        }
        out.extend(std::iter::repeat_n(b'=', 3 - chunk.len()));
    }
    out
}

fn value(symbol: u8) -> Option<u32> {
    match symbol {
        b'A'..=b'Z' => Some(u32::from(symbol - b'A')),
        b'a'..=b'z' => Some(u32::from(symbol - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(symbol - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn fail(message: impl Into<String>) -> ByteStringError {
    ByteStringError::IllegalArgument(message.into())
}

const WRONG_UNIT: &str = "Input byte array has wrong 4-byte ending unit";
const LAST_UNIT: &str = "Last unit does not have enough valid bits";

pub(crate) fn decode(input: &[u8]) -> Result<Vec<u8>, ByteStringError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    if input.len() < 2 {
        return Err(fail(
            "Input byte[] should at least have 2 bytes for base64 bytes",
        ));
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3 + 2);
    let mut bits = 0u32;
    let mut count = 0;
    let mut position = 0;
    while position < input.len() {
        let symbol = input[position];
        position += 1;
        if let Some(v) = value(symbol) {
            bits = (bits << 6) | v;
            count += 1;
            if count == 4 {
                out.extend_from_slice(&[(bits >> 16) as u8, (bits >> 8) as u8, bits as u8]);
                bits = 0;
                count = 0;
            }
            continue;
        }
        if symbol != b'=' {
            // The character is shown as the signed value of the byte in hexadecimal.
            let signed = i32::from(symbol as i8);
            let sign = if signed < 0 { "-" } else { "" };
            return Err(fail(format!(
                "Illegal base64 character {sign}{:x}",
                signed.abs()
            )));
        }
        match count {
            2 => {
                if input.get(position) != Some(&b'=') {
                    return Err(fail(WRONG_UNIT));
                }
                position += 1;
                out.push((bits >> 4) as u8);
            }
            3 => out.extend_from_slice(&[(bits >> 10) as u8, (bits >> 2) as u8]),
            1 => return Err(fail(LAST_UNIT)),
            _ => return Err(fail(WRONG_UNIT)),
        }
        if position < input.len() {
            return Err(fail(format!(
                "Input byte array has incorrect ending byte at {position}"
            )));
        }
        return Ok(out);
    }
    match count {
        0 => {}
        1 => return Err(fail(LAST_UNIT)),
        2 => out.push((bits >> 4) as u8),
        _ => out.extend_from_slice(&[(bits >> 10) as u8, (bits >> 2) as u8]),
    }
    Ok(out)
}
