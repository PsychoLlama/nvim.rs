//! A reader for the subset of Lua that the vendored metadata is written in.
//!
//! `crates/nvim/src/nvim/options.lua` and `crates/nvim/src/nvim/eval.lua` are
//! data, not programs: one nested table constructor of literals, plus a
//! handful of helper calls (`N_` for the short descriptions, `macros` for
//! defaults that a C macro supplies). Upstream fed them to a real Lua
//! interpreter; doing the same here would make regenerating the tables depend
//! on a *built* nvim, which is the one thing a bootstrap step must not need —
//! and `just apigen --check` has to run whether or not the tree compiles.
//!
//! So the files are read directly. The grammar accepted is deliberately tiny:
//! literals, table constructors, and calls left unevaluated as
//! [`Value::Call`]. Anything else — an operator, a method call, a closure in
//! the data — is a parse error rather than a silent misreading, which is
//! what keeps this honest as the metadata follows upstream.

use std::collections::BTreeMap;

/// A Lua value, as far as the data in `options.lua` goes.
#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Str(String),
    Table(Table),
    /// `f(a, b)` — recorded, not evaluated. The caller decides what a call
    /// means (`macros('DFLT_COLS', 'number')` names a C constant).
    Call(String, Vec<Value>),
    /// A bare name: a reference to something declared elsewhere in the file.
    Ref(String),
}

impl Value {
    /// How to name this value in a diagnostic.
    pub fn describe(&self) -> String {
        match self {
            Value::Nil => "nil".into(),
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Str(s) => format!("{s:?}"),
            Value::Table(_) => "a table".into(),
            Value::Call(f, _) => format!("a call to `{f}`"),
            Value::Ref(name) => format!("`{name}`"),
        }
    }
}

/// A Lua table: the positional part and the keyed part, both in source order.
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub array: Vec<Value>,
    pub map: BTreeMap<String, Value>,
}

impl Table {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self.map.get(key) {
            Some(Value::Nil) | None => None,
            some => some,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Value::Str(s)) => Some(s),
            _ => None,
        }
    }

    /// Whether `if t.key then` would take the branch — `false` and `nil` are
    /// the only falsy values in Lua, and `options.lua` does write
    /// `deny_duplicates = false`.
    pub fn truthy(&self, key: &str) -> bool {
        !matches!(self.get(key), None | Some(Value::Bool(false)))
    }

    pub fn table(&self, key: &str) -> Option<&Table> {
        match self.get(key) {
            Some(Value::Table(t)) => Some(t),
            _ => None,
        }
    }

    /// The values of a list-shaped field, as strings. `None` when absent;
    /// an error when present but not a list of strings.
    pub fn str_list(&self, key: &str) -> Result<Option<Vec<String>>, String> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        let items = match value {
            Value::Str(s) => return Ok(Some(vec![s.clone()])),
            Value::Table(t) => &t.array,
            _ => return Err(format!("`{key}` is not a list of strings")),
        };
        items
            .iter()
            .map(|v| match v {
                Value::Str(s) => Ok(s.clone()),
                _ => Err(format!("`{key}` is not a list of strings")),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Name(String),
    Str(String),
    Int(i64),
    Punct(char),
}

struct Lexer<'a> {
    /// The file name diagnostics quote.
    file: &'a str,
    src: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn err(&self, what: &str) -> String {
        format!("{}:{}: {what}", self.file, self.line)
    }

    fn bump(&mut self) -> u8 {
        let c = self.src[self.pos];
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
        }
        c
    }

    fn at(&self, s: &str) -> bool {
        self.src[self.pos..].starts_with(s.as_bytes())
    }

    /// The `=` count of a long bracket opening here (`[[` is 0, `[=[` is 1),
    /// or `None` if this is not one.
    fn long_bracket(&self) -> Option<usize> {
        if self.src[self.pos] != b'[' {
            return None;
        }
        let mut n = 0;
        while self.src.get(self.pos + 1 + n) == Some(&b'=') {
            n += 1;
        }
        (self.src.get(self.pos + 1 + n) == Some(&b'[')).then_some(n)
    }

    fn read_long(&mut self, level: usize) -> Result<String, String> {
        for _ in 0..level + 2 {
            self.bump();
        }
        // A newline immediately after the opening bracket is dropped.
        if self.src.get(self.pos) == Some(&b'\n') {
            self.bump();
        }
        let close = format!("]{}]", "=".repeat(level));
        let start = self.pos;
        while !self.at(&close) {
            if self.pos >= self.src.len() {
                return Err(self.err("unterminated long bracket"));
            }
            self.bump();
        }
        let text = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
        for _ in 0..close.len() {
            self.bump();
        }
        Ok(text)
    }

    fn read_quoted(&mut self) -> Result<String, String> {
        let quote = self.bump();
        let mut out = Vec::new();
        loop {
            if self.pos >= self.src.len() {
                return Err(self.err("unterminated string"));
            }
            let c = self.bump();
            if c == quote {
                return Ok(String::from_utf8_lossy(&out).into_owned());
            }
            if c != b'\\' {
                out.push(c);
                continue;
            }
            let e = self.bump();
            out.push(match e {
                b'n' => b'\n',
                b't' => b'\t',
                b'r' => b'\r',
                b'a' => 7,
                b'b' => 8,
                b'f' => 12,
                b'v' => 11,
                b'\\' | b'"' | b'\'' => e,
                _ => return Err(self.err(&format!("unsupported string escape `\\{}`", e as char))),
            });
        }
    }

    fn skip_trivia(&mut self) -> Result<(), String> {
        loop {
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
                self.bump();
            }
            if !self.at("--") {
                return Ok(());
            }
            self.bump();
            self.bump();
            match self.long_bracket() {
                Some(level) => {
                    self.read_long(level)?;
                }
                None => {
                    while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                        self.bump();
                    }
                }
            }
        }
    }

    fn next(&mut self) -> Result<Option<Tok>, String> {
        self.skip_trivia()?;
        if self.pos >= self.src.len() {
            return Ok(None);
        }
        let c = self.src[self.pos];
        if c == b'"' || c == b'\'' {
            return Ok(Some(Tok::Str(self.read_quoted()?)));
        }
        if let Some(level) = self.long_bracket() {
            return Ok(Some(Tok::Str(self.read_long(level)?)));
        }
        if c.is_ascii_digit() {
            return Ok(Some(Tok::Int(self.read_number()?)));
        }
        if c == b'_' || c.is_ascii_alphabetic() {
            let start = self.pos;
            while self
                .src
                .get(self.pos)
                .is_some_and(|c| *c == b'_' || c.is_ascii_alphanumeric())
            {
                self.bump();
            }
            let name = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
            return Ok(Some(Tok::Name(name)));
        }
        Ok(Some(Tok::Punct(self.bump() as char)))
    }

    fn read_number(&mut self) -> Result<i64, String> {
        let start = self.pos;
        let hex = self.at("0x") || self.at("0X");
        if hex {
            self.bump();
            self.bump();
        }
        while self
            .src
            .get(self.pos)
            .is_some_and(|c| c.is_ascii_alphanumeric())
        {
            self.bump();
        }
        let text = String::from_utf8_lossy(&self.src[start..self.pos]).into_owned();
        let parsed = if hex {
            i64::from_str_radix(&text[2..], 16)
        } else {
            text.parse()
        };
        parsed.map_err(|_| self.err(&format!("unsupported number `{text}`")))
    }
}

struct Parser<'a> {
    file: &'a str,
    toks: Vec<(Tok, usize)>,
    pos: usize,
}

impl Parser<'_> {
    fn err(&self, what: &str) -> String {
        let line = self.toks.get(self.pos).map_or(0, |t| t.1);
        format!("{}:{line}: {what}", self.file)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.0)
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        let hit = self.peek() == Some(tok);
        self.pos += usize::from(hit);
        hit
    }

    fn expect(&mut self, tok: Tok) -> Result<(), String> {
        if self.eat(&tok) {
            return Ok(());
        }
        Err(self.err(&format!("expected {tok:?}, found {:?}", self.peek())))
    }

    fn value(&mut self) -> Result<Value, String> {
        if self.eat(&Tok::Punct('-')) {
            return match self.value()? {
                Value::Int(n) => Ok(Value::Int(-n)),
                _ => Err(self.err("unary minus on a non-number")),
            };
        }
        let mut tok = self
            .peek()
            .cloned()
            .ok_or_else(|| self.err("expected a value"))?;
        self.pos += 1;
        // A dotted name (`table.concat`) is still just a name here.
        while let Tok::Name(name) = &tok {
            if self.peek() != Some(&Tok::Punct('.')) {
                break;
            }
            self.pos += 1;
            let Some(Tok::Name(field)) = self.peek().cloned() else {
                return Err(self.err("expected a name after `.`"));
            };
            self.pos += 1;
            tok = Tok::Name(format!("{name}.{field}"));
        }
        match tok {
            Tok::Str(s) => Ok(Value::Str(s)),
            Tok::Int(n) => Ok(Value::Int(n)),
            Tok::Punct('{') => self.table().map(Value::Table),
            Tok::Name(name) => match name.as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                "nil" => Ok(Value::Nil),
                _ if self.eat(&Tok::Punct('(')) => {
                    let mut args = Vec::new();
                    while !self.eat(&Tok::Punct(')')) {
                        args.push(self.value()?);
                        self.eat(&Tok::Punct(','));
                    }
                    Ok(Value::Call(name, args))
                }
                _ => Ok(Value::Ref(name)),
            },
            other => Err(self.err(&format!("expected a value, found {other:?}"))),
        }
    }

    /// The body of a table constructor; the opening `{` is already eaten.
    fn table(&mut self) -> Result<Table, String> {
        let mut out = Table::default();
        while !self.eat(&Tok::Punct('}')) {
            // `key = value`, `['key'] = value`, or a positional item. A
            // comment-shaped `--[[@as T]]` annotation is already gone.
            let key = match (
                self.peek().cloned(),
                self.toks.get(self.pos + 1).map(|t| &t.0),
            ) {
                (Some(Tok::Name(name)), Some(Tok::Punct('='))) => {
                    self.pos += 2;
                    Some(name)
                }
                (Some(Tok::Punct('[')), _) => {
                    self.pos += 1;
                    let Value::Str(key) = self.value()? else {
                        return Err(self.err("only string table keys are supported"));
                    };
                    self.expect(Tok::Punct(']'))?;
                    self.expect(Tok::Punct('='))?;
                    Some(key)
                }
                _ => None,
            };
            let value = self.value()?;
            match key {
                Some(key) => {
                    if out.map.insert(key.clone(), value).is_some() {
                        return Err(self.err(&format!("duplicate table key `{key}`")));
                    }
                }
                None => out.array.push(value),
            }
            if !self.eat(&Tok::Punct(',')) && !self.eat(&Tok::Punct(';')) {
                self.expect(Tok::Punct('}'))?;
                break;
            }
        }
        Ok(out)
    }
}

/// Tokenize a whole file, so that long strings and comments cannot be
/// mistaken for data wherever the wanted table happens to sit.
fn lex(file: &str, source: &str) -> Result<Vec<(Tok, usize)>, String> {
    let mut lexer = Lexer {
        file,
        src: source.as_bytes(),
        pos: 0,
        line: 1,
    };
    let mut toks = Vec::new();
    while let Some(tok) = lexer.next()? {
        toks.push((tok, lexer.line));
    }
    Ok(toks)
}

/// Read the table a Lua source file assigns to `binding` — `local options`
/// in `options.lua`, `M.funcs` in `eval.lua`. The binding is spelled as Lua
/// and lexed with the same rules as the file, so a dotted name works.
pub fn read_table(file: &str, source: &str, binding: &str) -> Result<Table, String> {
    let toks = lex(file, source)?;
    let mut want: Vec<Tok> = lex(file, binding)?.into_iter().map(|(t, _)| t).collect();
    want.push(Tok::Punct('='));
    want.push(Tok::Punct('{'));
    let start = toks
        .windows(want.len())
        .position(|w| w.iter().map(|t| &t.0).eq(want.iter()))
        .ok_or_else(|| format!("{file}: no `{binding} = {{`"))?;
    let mut parser = Parser {
        file,
        toks,
        pos: start + want.len(),
    };
    parser.table()
}
