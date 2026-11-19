// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Lexer and parser of HOCON text into the unresolved value tree.

use super::error::{ConfigError, ConfigErrorKind};
use super::model::{Kind, Number, Obj, Piece, Subst, Val, merge_obj, normalize};

/// Resources an `include "name"` (or `classpath("name")`) can find: `(name, text)` pairs.
pub type Resources<'a> = &'a [(&'a str, &'a str)];

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Open,
    Close,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Equals,
    PlusEquals,
    Nl,
    Ws(String),
    Quoted(String),
    Unquoted(String),
    Number(Number),
    Bool(bool),
    Null,
    Subst(Vec<String>, bool),
    Eof,
}

#[derive(Clone, Debug)]
struct T {
    tok: Tok,
    line: u32,
}

fn parse_err(line: u32, msg: impl Into<String>) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::Parse,
        format!("String: {line}: {}", msg.into()),
    )
}

const FORBIDDEN: &str = "$\"{}[]:=,+#`^?!@*&\\";

fn lex(src: &str) -> Result<Vec<T>, ConfigError> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut line = 1u32;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\n' => {
                out.push(T { tok: Tok::Nl, line });
                line += 1;
                i += 1;
            }
            c if c.is_whitespace() || c == '\u{feff}' => {
                let s = i;
                while i < chars.len()
                    && chars[i] != '\n'
                    && (chars[i].is_whitespace() || chars[i] == '\u{feff}')
                {
                    i += 1;
                }
                out.push(T {
                    tok: Tok::Ws(chars[s..i].iter().collect()),
                    line,
                });
            }
            '#' => skip_comment(&chars, &mut i),
            '/' if chars.get(i + 1) == Some(&'/') => skip_comment(&chars, &mut i),
            '{' => push1(&mut out, &mut i, Tok::Open, line),
            '}' => push1(&mut out, &mut i, Tok::Close, line),
            '[' => push1(&mut out, &mut i, Tok::LBracket, line),
            ']' => push1(&mut out, &mut i, Tok::RBracket, line),
            ',' => push1(&mut out, &mut i, Tok::Comma, line),
            ':' => push1(&mut out, &mut i, Tok::Colon, line),
            '=' => push1(&mut out, &mut i, Tok::Equals, line),
            '+' if chars.get(i + 1) == Some(&'=') => {
                out.push(T {
                    tok: Tok::PlusEquals,
                    line,
                });
                i += 2;
            }
            '"' => {
                let start_line = line;
                let s = if chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                    i += 3;
                    let s = i;
                    // The string ends at the last quote of a run of three or more quotes.
                    loop {
                        if i >= chars.len() {
                            return Err(parse_err(start_line, "Unterminated triple-quoted string"));
                        }
                        if chars[i] == '"'
                            && chars.get(i + 1) == Some(&'"')
                            && chars.get(i + 2) == Some(&'"')
                        {
                            let mut j = i;
                            while chars.get(j + 1) == Some(&'"') {
                                j += 1;
                            }
                            let text: String = chars[s..j - 2].iter().collect();
                            line += text.matches('\n').count() as u32;
                            i = j + 1;
                            break text;
                        }
                        i += 1;
                    }
                } else {
                    i += 1;
                    let mut text = String::new();
                    loop {
                        match chars.get(i) {
                            None | Some('\n') => {
                                return Err(parse_err(
                                    line,
                                    "Unbalanced quotes: unterminated string",
                                ));
                            }
                            Some('"') => {
                                i += 1;
                                break;
                            }
                            Some('\\') => {
                                i += 1;
                                text.push(unescape(&chars, &mut i, line)?);
                            }
                            Some(&ch) => {
                                text.push(ch);
                                i += 1;
                            }
                        }
                    }
                    text
                };
                out.push(T {
                    tok: Tok::Quoted(s),
                    line: start_line,
                });
            }
            '$' => {
                if chars.get(i + 1) != Some(&'{') {
                    return Err(parse_err(line, "Expecting '{' after '$' in a substitution"));
                }
                i += 2;
                let optional = chars.get(i) == Some(&'?');
                if optional {
                    i += 1;
                }
                let s = i;
                let mut in_quote = false;
                while i < chars.len() && (in_quote || chars[i] != '}') {
                    if chars[i] == '"' {
                        in_quote = !in_quote;
                    }
                    if chars[i] == '\n' {
                        return Err(parse_err(line, "Unterminated substitution"));
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(parse_err(line, "Unterminated substitution"));
                }
                let text: String = chars[s..i].iter().collect();
                i += 1;
                let path = parse_path(&text).map_err(|m| parse_err(line, m))?;
                out.push(T {
                    tok: Tok::Subst(path, optional),
                    line,
                });
            }
            c if c.is_ascii_digit() || c == '-' => {
                let s = i;
                while i < chars.len() && "0123456789eE+-.".contains(chars[i]) {
                    i += 1;
                }
                let text: String = chars[s..i].iter().collect();
                let tok = match Number::parse(&text) {
                    Some(n) => Tok::Number(n),
                    None => Tok::Unquoted(text),
                };
                out.push(T { tok, line });
            }
            _ => {
                let s = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && !FORBIDDEN.contains(chars[i])
                    && !(chars[i] == '/' && chars.get(i + 1) == Some(&'/'))
                {
                    i += 1;
                }
                if i == s {
                    return Err(parse_err(
                        line,
                        format!("Reserved character '{c}' is not allowed outside quotes"),
                    ));
                }
                let text: String = chars[s..i].iter().collect();
                let tok = match text.as_str() {
                    "true" => Tok::Bool(true),
                    "false" => Tok::Bool(false),
                    "null" => Tok::Null,
                    _ => Tok::Unquoted(text),
                };
                out.push(T { tok, line });
            }
        }
    }
    out.push(T {
        tok: Tok::Eof,
        line,
    });
    Ok(out)
}

fn push1(out: &mut Vec<T>, i: &mut usize, tok: Tok, line: u32) {
    out.push(T { tok, line });
    *i += 1;
}

fn skip_comment(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i] != '\n' {
        *i += 1;
    }
}

fn unescape(chars: &[char], i: &mut usize, line: u32) -> Result<char, ConfigError> {
    let Some(&c) = chars.get(*i) else {
        return Err(parse_err(line, "Unterminated escape"));
    };
    *i += 1;
    Ok(match c {
        '"' => '"',
        '\\' => '\\',
        '/' => '/',
        'b' => '\u{8}',
        'f' => '\u{c}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'u' => {
            let hex: String = chars.iter().skip(*i).take(4).collect();
            let code = u32::from_str_radix(&hex, 16)
                .ok()
                .filter(|_| hex.len() == 4)
                .ok_or_else(|| parse_err(line, format!("Malformed escape \\u{hex}")))?;
            *i += 4;
            char::from_u32(code).unwrap_or('\u{fffd}')
        }
        other => {
            return Err(parse_err(
                line,
                format!("backslash followed by '{other}', this is not a valid escape sequence"),
            ));
        }
    })
}

/// Splits a path expression (`a.b."c.d"`) into its segments.
pub(crate) fn parse_path(text: &str) -> Result<Vec<String>, String> {
    let chars: Vec<char> = text.trim().chars().collect();
    let mut segments = Vec::new();
    let mut cur = String::new();
    let mut had_quoted = false;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '"' => {
                i += 1;
                had_quoted = true;
                loop {
                    match chars.get(i) {
                        None => return Err("Unterminated quote in path expression".to_owned()),
                        Some('"') => {
                            i += 1;
                            break;
                        }
                        Some('\\') => {
                            i += 1;
                            cur.push(
                                unescape(&chars, &mut i, 0).map_err(|e| e.message().to_owned())?,
                            );
                        }
                        Some(&c) => {
                            cur.push(c);
                            i += 1;
                        }
                    }
                }
            }
            '.' => {
                if cur.is_empty() && !had_quoted {
                    return Err(format!("Invalid path '{text}': empty path element"));
                }
                segments.push(std::mem::take(&mut cur));
                had_quoted = false;
                i += 1;
            }
            c => {
                cur.push(c);
                i += 1;
            }
        }
    }
    if cur.is_empty() && !had_quoted {
        return Err(format!("Invalid path '{text}': empty path element"));
    }
    segments.push(cur);
    Ok(segments)
}

struct Parser<'a> {
    toks: Vec<T>,
    pos: usize,
    prefix: Vec<String>,
    resources: Resources<'a>,
    depth: u32,
}

/// Parses HOCON text into the unresolved root object.
pub(crate) fn parse(src: &str, resources: Resources<'_>) -> Result<Obj, ConfigError> {
    parse_depth(src, resources, 0)
}

fn parse_depth(src: &str, resources: Resources<'_>, depth: u32) -> Result<Obj, ConfigError> {
    let toks = lex(src)?;
    let mut p = Parser {
        toks,
        pos: 0,
        prefix: Vec::new(),
        resources,
        depth,
    };
    p.skip_blank();
    if matches!(p.peek(), Tok::LBracket) {
        return Err(parse_err(
            p.line(),
            "Document has a list at its root; expecting an object (config files must have an object at the root)",
        ));
    }
    let braced = matches!(p.peek(), Tok::Open);
    if braced {
        p.pos += 1;
    }
    let obj = p.parse_body(braced)?;
    p.skip_blank();
    if !matches!(p.peek(), Tok::Eof) {
        return Err(parse_err(
            p.line(),
            "Document has trailing tokens after the closing brace of the root object",
        ));
    }
    Ok(obj)
}

impl Parser<'_> {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn line(&self) -> u32 {
        self.toks[self.pos].line
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Tok::Ws(_)) {
            self.pos += 1;
        }
    }

    fn skip_blank(&mut self) {
        while matches!(self.peek(), Tok::Ws(_) | Tok::Nl) {
            self.pos += 1;
        }
    }

    /// Object body after `{` (or the root); consumes the closing brace when `braced`.
    fn parse_body(&mut self, braced: bool) -> Result<Obj, ConfigError> {
        let mut obj = Obj::new();
        loop {
            self.skip_blank();
            while matches!(self.peek(), Tok::Comma) {
                self.pos += 1;
                self.skip_blank();
            }
            match self.peek() {
                Tok::Close if braced => {
                    self.pos += 1;
                    return Ok(obj);
                }
                Tok::Eof if !braced => return Ok(obj),
                Tok::Eof => {
                    return Err(parse_err(
                        self.line(),
                        "Expecting a close brace '}' but got end of file",
                    ));
                }
                Tok::Close => return Err(parse_err(self.line(), "Unbalanced close brace '}'")),
                _ => self.parse_field(&mut obj)?,
            }
        }
    }

    fn parse_field(&mut self, obj: &mut Obj) -> Result<(), ConfigError> {
        let line = self.line();
        if matches!(self.peek(), Tok::Unquoted(t) if t == "include") && self.looks_like_include() {
            return self.parse_include(obj);
        }
        let key = self.parse_key()?;
        self.skip_ws();
        let plus = match self.peek() {
            Tok::Colon | Tok::Equals => {
                self.pos += 1;
                false
            }
            Tok::PlusEquals => {
                self.pos += 1;
                true
            }
            Tok::Open => false,
            other => {
                return Err(parse_err(
                    self.line(),
                    format!(
                        "Key '{}' may not be followed by token: {other:?}",
                        key.join(".")
                    ),
                ));
            }
        };
        self.skip_blank();
        let mut full = self.prefix.clone();
        full.extend(key.iter().cloned());
        let saved = self.prefix.clone();
        self.prefix = full.clone();
        let value = self.parse_value()?;
        self.prefix = saved;
        let value = if plus {
            Val::new(
                line,
                Kind::Concat(vec![
                    Piece::V(Val::new(
                        line,
                        Kind::Subst(Subst {
                            path: full,
                            optional: true,
                        }),
                    )),
                    Piece::V(Val::new(line, Kind::List(vec![value]))),
                ]),
            )
        } else {
            value
        };
        assign(obj, &key, value, line);
        Ok(())
    }

    fn parse_key(&mut self) -> Result<Vec<String>, ConfigError> {
        let mut segments: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut had_quoted = false;
        let mut pending_ws = String::new();
        let mut any = false;
        loop {
            match self.peek().clone() {
                Tok::Ws(w) => {
                    if any {
                        pending_ws.push_str(&w);
                    }
                    self.pos += 1;
                }
                Tok::Quoted(s) => {
                    cur.push_str(&std::mem::take(&mut pending_ws));
                    cur.push_str(&s);
                    had_quoted = true;
                    any = true;
                    self.pos += 1;
                }
                Tok::Unquoted(_) | Tok::Number(_) | Tok::Bool(_) | Tok::Null => {
                    let text = match self.peek().clone() {
                        Tok::Unquoted(t) => t,
                        Tok::Number(n) => n.text().to_owned(),
                        Tok::Bool(b) => b.to_string(),
                        _ => "null".to_owned(),
                    };
                    if !cur.is_empty() || had_quoted {
                        cur.push_str(&std::mem::take(&mut pending_ws));
                    } else {
                        pending_ws.clear();
                    }
                    for ch in text.chars() {
                        if ch == '.' {
                            if cur.is_empty() && !had_quoted {
                                return Err(parse_err(self.line(), "Empty path element in key"));
                            }
                            segments.push(std::mem::take(&mut cur));
                            had_quoted = false;
                        } else {
                            cur.push(ch);
                        }
                    }
                    any = true;
                    self.pos += 1;
                }
                Tok::Colon | Tok::Equals | Tok::PlusEquals | Tok::Open => break,
                other => {
                    return Err(parse_err(
                        self.line(),
                        format!("Expecting a key but got token: {other:?}"),
                    ));
                }
            }
        }
        if !any || (cur.is_empty() && !had_quoted) {
            return Err(parse_err(self.line(), "Empty key or path element"));
        }
        segments.push(cur);
        Ok(segments)
    }

    fn looks_like_include(&self) -> bool {
        let mut j = self.pos + 1;
        while matches!(self.toks[j].tok, Tok::Ws(_)) {
            j += 1;
        }
        match &self.toks[j].tok {
            Tok::Quoted(_) => j > self.pos + 1,
            Tok::Unquoted(t) => t.ends_with('('),
            _ => false,
        }
    }

    fn parse_include(&mut self, obj: &mut Obj) -> Result<(), ConfigError> {
        let line = self.line();
        self.pos += 1;
        self.skip_ws();
        let mut required = false;
        let mut kind = "heuristic";
        let mut parens = 0;
        while let Tok::Unquoted(t) = self.peek().clone() {
            for name in t.split('(').filter(|n| !n.is_empty()) {
                match name {
                    "required" => required = true,
                    "file" | "url" | "classpath" => kind = name_static(name),
                    other => {
                        return Err(parse_err(
                            line,
                            format!(
                                "include keyword is not followed by a quoted string, but by: {other}"
                            ),
                        ));
                    }
                }
                parens += 1;
            }
            self.pos += 1;
            self.skip_ws();
        }
        let Tok::Quoted(name) = self.peek().clone() else {
            return Err(parse_err(
                line,
                "include keyword is not followed by a quoted string",
            ));
        };
        self.pos += 1;
        let mut closed = 0;
        while closed < parens {
            self.skip_ws();
            match self.peek() {
                Tok::Unquoted(t) if t.chars().all(|c| c == ')') => {
                    closed += t.len();
                    self.pos += 1;
                }
                _ => {
                    return Err(parse_err(
                        line,
                        "include with a wrapper is missing its closing parenthesis",
                    ));
                }
            }
        }
        if let Some(included) = self.load_include(&name, kind, required, line)? {
            let merged = merge_obj(&included, obj);
            *obj = merged;
        }
        Ok(())
    }

    fn load_include(
        &self,
        name: &str,
        kind: &str,
        required: bool,
        line: u32,
    ) -> Result<Option<Obj>, ConfigError> {
        if self.depth > 50 {
            return Err(parse_err(
                line,
                "include statements nested more than 50 times, you probably have a cycle in your includes",
            ));
        }
        let found: Option<String> = if kind == "file" {
            std::fs::read_to_string(name).ok()
        } else {
            [name.to_owned(), format!("{name}.conf")]
                .iter()
                .find_map(|n| {
                    self.resources
                        .iter()
                        .find(|(k, _)| k == n)
                        .map(|(_, t)| (*t).to_owned())
                })
        };
        match found {
            Some(text) => parse_depth(&text, self.resources, self.depth + 1).map(Some),
            None if !required => Ok(None),
            None if kind == "file" => Err(ConfigError::new(
                ConfigErrorKind::Io,
                format!(
                    "{name}: java.io.FileNotFoundException: {name} (No such file or directory)"
                ),
            )),
            None => {
                let nf = |ext: &str| {
                    format!(
                        "{name}.{ext}: java.io.IOException: resource not found on classpath: {name}.{ext}"
                    )
                };
                Err(ConfigError::new(
                    ConfigErrorKind::Io,
                    format!(
                        "{name}: {}, {}, {}",
                        nf("conf"),
                        nf("json"),
                        nf("properties")
                    ),
                ))
            }
        }
    }

    /// One value: a concatenation of values on the same line.
    fn parse_value(&mut self) -> Result<Val, ConfigError> {
        let line = self.line();
        let mut pieces: Vec<Piece> = Vec::new();
        loop {
            let line_here = self.line();
            match self.peek().clone() {
                Tok::Ws(w) => {
                    self.pos += 1;
                    if !pieces.is_empty() {
                        pieces.push(Piece::Ws(w));
                    }
                }
                Tok::Quoted(s) => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Str(s))));
                }
                Tok::Unquoted(s) => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Str(s))));
                }
                Tok::Number(n) => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Num(n))));
                }
                Tok::Bool(b) => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Bool(b))));
                }
                Tok::Null => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Null)));
                }
                Tok::Subst(path, optional) => {
                    self.pos += 1;
                    pieces.push(Piece::V(Val::new(
                        line_here,
                        Kind::Subst(Subst { path, optional }),
                    )));
                }
                Tok::Open => {
                    self.pos += 1;
                    let obj = self.parse_body(true)?;
                    pieces.push(Piece::V(Val::new(line_here, Kind::Obj(obj))));
                }
                Tok::LBracket => {
                    self.pos += 1;
                    let list = self.parse_list()?;
                    pieces.push(Piece::V(Val::new(line_here, Kind::List(list))));
                }
                Tok::Nl | Tok::Comma | Tok::Close | Tok::RBracket | Tok::Eof => break,
                other => {
                    return Err(parse_err(
                        self.line(),
                        format!("Expecting a value but got wrong token: {other:?}"),
                    ));
                }
            }
        }
        while matches!(pieces.last(), Some(Piece::Ws(_))) {
            pieces.pop();
        }
        if pieces.is_empty() {
            return Err(parse_err(
                line,
                "Expecting a value but got wrong token: end of line",
            ));
        }
        super::resolve::combine(pieces, line, false)
    }

    fn parse_list(&mut self) -> Result<Vec<Val>, ConfigError> {
        let mut items = Vec::new();
        loop {
            self.skip_blank();
            while matches!(self.peek(), Tok::Comma) {
                self.pos += 1;
                self.skip_blank();
            }
            match self.peek() {
                Tok::RBracket => {
                    self.pos += 1;
                    return Ok(items);
                }
                Tok::Eof => {
                    return Err(parse_err(
                        self.line(),
                        "List not closed: expecting a close bracket ']' but got end of file",
                    ));
                }
                _ => items.push(self.parse_value()?),
            }
        }
    }
}

fn name_static(name: &str) -> &'static str {
    match name {
        "file" => "file",
        "url" => "url",
        _ => "classpath",
    }
}

/// `key.path = value` into the object, as a higher-priority layer than what is already there.
fn assign(obj: &mut Obj, key: &[String], value: Val, line: u32) {
    let mut v = value;
    for seg in key[1..].iter().rev() {
        let mut inner = Obj::new();
        inner.insert(seg.clone(), vec![v]);
        v = Val::new(line, Kind::Obj(inner));
    }
    let mut stack = vec![v];
    if let Some(existing) = obj.remove(&key[0]) {
        stack.extend(existing);
    }
    obj.insert(key[0].clone(), normalize(stack));
}
