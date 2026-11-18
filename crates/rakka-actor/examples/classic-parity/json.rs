//! Minimal JSON string reader/writer matching the oracle's wire format (ASCII-only output).

/// Why the harness refuses an input with a lone UTF-16 surrogate (decision Q-X-17).
pub const OUT_OF_DOMAIN: &str =
    "input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)";

/// Writes the UTF-16 code units `s` as a JSON string literal; everything outside 0x20..=0x7e becomes `\uxxxx`.
/// Output may hold lone surrogates (Java messages can); only inputs are restricted to `&str`.
pub fn write_str(out: &mut String, s: &[u16]) {
    out.push('"');
    for &c in s {
        match c {
            0x22 => out.push_str("\\\""),
            0x5c => out.push_str("\\\\"),
            c if !(0x20..=0x7e).contains(&c) => out.push_str(&format!("\\u{c:04x}")),
            c => out.push(c as u8 as char),
        }
    }
    out.push('"');
}

/// Writes the Rust string `s` as a JSON string literal (see [`write_str`]).
pub fn write_text(out: &mut String, s: &str) {
    write_str(out, &s.encode_utf16().collect::<Vec<u16>>());
}

/// Parses a JSON string literal. Escapes are decoded as UTF-16 code units; an unpaired surrogate is
/// rejected with [`OUT_OF_DOMAIN`] and never replaced (no lossy decoding).
pub fn parse_str(lit: &str) -> Result<String, String> {
    let chars: Vec<char> = lit.chars().collect();
    if chars.len() < 2 || chars[0] != '"' || chars[chars.len() - 1] != '"' {
        return Err(format!("stdin must be a JSON string literal, got: {lit}"));
    }
    let mut out: Vec<u16> = Vec::new();
    let mut i = 1;
    while i < chars.len() - 1 {
        let c = chars[i];
        if c != '\\' {
            let mut buf = [0u16; 2];
            out.extend_from_slice(c.encode_utf16(&mut buf));
            i += 1;
            continue;
        }
        i += 1;
        let e = *chars.get(i).ok_or("dangling backslash")?;
        match e {
            '"' => out.push(0x22),
            '\\' => out.push(0x5c),
            '/' => out.push(0x2f),
            'b' => out.push(0x08),
            'f' => out.push(0x0c),
            'n' => out.push(0x0a),
            'r' => out.push(0x0d),
            't' => out.push(0x09),
            'u' => {
                let hex: String = chars
                    .get(i + 1..i + 5)
                    .ok_or("short \\u escape")?
                    .iter()
                    .collect();
                out.push(u16::from_str_radix(&hex, 16).map_err(|e| e.to_string())?);
                i += 4;
            }
            other => return Err(format!("bad escape \\{other}")),
        }
        i += 1;
    }
    String::from_utf16(&out).map_err(|_| OUT_OF_DOMAIN.to_string())
}

/// A parsed JSON value. Objects keep insertion order.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }
}

/// Parses a complete JSON document.
pub fn parse_value(text: &str) -> Result<Value, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = ValueParser { c: &chars, i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != chars.len() {
        return Err(format!("trailing JSON at {}", p.i));
    }
    Ok(v)
}

struct ValueParser<'a> {
    c: &'a [char],
    i: usize,
}

impl ValueParser<'_> {
    fn ws(&mut self) {
        while self.i < self.c.len() && " \t\r\n".contains(self.c[self.i]) {
            self.i += 1;
        }
    }

    fn peek(&self) -> Result<char, String> {
        self.c
            .get(self.i)
            .copied()
            .ok_or_else(|| "unexpected end of JSON".to_string())
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.peek()? {
            '{' => {
                self.i += 1;
                self.ws();
                let mut kv = Vec::new();
                if self.peek()? == '}' {
                    self.i += 1;
                    return Ok(Value::Object(kv));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    if self.peek()? != ':' {
                        return Err("expected ':'".into());
                    }
                    self.i += 1;
                    self.ws();
                    kv.push((k, self.value()?));
                    self.ws();
                    match self.peek()? {
                        ',' => self.i += 1,
                        '}' => {
                            self.i += 1;
                            return Ok(Value::Object(kv));
                        }
                        c => return Err(format!("unexpected '{c}' in object")),
                    }
                }
            }
            '[' => {
                self.i += 1;
                self.ws();
                let mut a = Vec::new();
                if self.peek()? == ']' {
                    self.i += 1;
                    return Ok(Value::Array(a));
                }
                loop {
                    self.ws();
                    a.push(self.value()?);
                    self.ws();
                    match self.peek()? {
                        ',' => self.i += 1,
                        ']' => {
                            self.i += 1;
                            return Ok(Value::Array(a));
                        }
                        c => return Err(format!("unexpected '{c}' in array")),
                    }
                }
            }
            '"' => Ok(Value::Str(self.string()?)),
            't' => self.word("true", Value::Bool(true)),
            'f' => self.word("false", Value::Bool(false)),
            'n' => self.word("null", Value::Null),
            _ => {
                let st = self.i;
                while self.i < self.c.len() && "+-0123456789.eE".contains(self.c[self.i]) {
                    self.i += 1;
                }
                let t: String = self.c[st..self.i].iter().collect();
                t.parse::<f64>()
                    .map(Value::Number)
                    .map_err(|e| format!("bad number {t:?}: {e}"))
            }
        }
    }

    fn word(&mut self, w: &str, v: Value) -> Result<Value, String> {
        if self.c[self.i..].iter().take(w.len()).collect::<String>() == w {
            self.i += w.len();
            Ok(v)
        } else {
            Err(format!("expected {w}"))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        let st = self.i;
        self.i += 1;
        while self.peek()? != '"' {
            if self.c[self.i] == '\\' {
                self.i += 1;
            }
            self.i += 1;
        }
        self.i += 1;
        parse_str(&self.c[st..self.i].iter().collect::<String>())
    }
}
