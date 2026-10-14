//! ShaDa: the msgpack encoding `:wshada` chooses for each kind of entry, and
//! what `:rshada` makes of a file packed by hand.
//!
//! The writer hand-rolls its packing, so the *choice* of encoding (fixint or
//! uint8, fixstr or str8, bin8 for every string that is not a map key) is
//! part of the format, and a decoded comparison would not see it. Each case
//! writes a file, then reads it back token by token in the format of
//! `test/battery/shadatok.py` — the same lines that script prints, so an
//! expectation here can be checked against it by hand — and compares the
//! entries of the kind it set up. Timestamps are masked as the script masks
//! them; so are the header's pid and version, and the sandbox's path.
//!
//! Written against the behaviour before the writer leaves `PackerBuffer` for
//! `mpack/token.rs` and the reader its `Cursor` over raw chunks; a row that
//! moves is a format change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// A cursor over one msgpack buffer.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        out
    }

    fn uint(&mut self, n: usize) -> u64 {
        self.take(n)
            .iter()
            .fold(0, |acc, &b| (acc << 8) | u64::from(b))
    }
}

/// `shadatok.py`'s `show_bytes` for the bytes these cases use: printable
/// ASCII as is, the usual escapes, `\xNN` for the rest.
fn show(raw: &[u8]) -> String {
    let mut out = String::from("'");
    for &b in raw {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\'' => out.push_str("\\'"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(char::from(b)),
            _ => out.push_str(&format!("\\x{b:02x}")),
        }
    }
    out.push('\'');
    out
}

/// One element, as `shadatok.py`'s `tokenise` prints it, nested elements
/// indented. Answers the element's bytes when it is a string, for the map
/// key that decides whether the next value is masked.
fn tokenise(r: &mut Reader, out: &mut Vec<String>, depth: usize, mask: Option<&str>) -> Vec<u8> {
    let pad = "  ".repeat(depth);
    let c = r.take(1)[0];
    let mut emit = |token: &str, text: String| {
        let token = if mask.is_some() { "masked" } else { token };
        let text = mask.map_or(text, str::to_owned);
        out.push(format!("{pad}{token} {text}"));
    };
    let (token, len): (&str, usize) = match c {
        0x00..=0x7f => {
            emit("fixint", c.to_string());
            return Vec::new();
        }
        0xe0..=0xff => {
            emit("negfixint", (i16::from(c) - 0x100).to_string());
            return Vec::new();
        }
        0x80..=0x8f | 0xde => {
            let (token, n) = match c {
                0xde => ("map16", r.uint(2)),
                _ => ("fixmap", u64::from(c & 0x0f)),
            };
            emit(token, format!("{{{n}}}"));
            for _ in 0..n {
                let key = tokenise(r, out, depth + 1, None);
                let mask = (key == b"pid").then_some("<PID>");
                let mask = mask.or((key == b"version").then_some("<VERSION>"));
                tokenise(r, out, depth + 1, mask);
            }
            return Vec::new();
        }
        0x90..=0x9f | 0xdc => {
            let (token, n) = match c {
                0xdc => ("array16", r.uint(2)),
                _ => ("fixarray", u64::from(c & 0x0f)),
            };
            emit(token, format!("[{n}]"));
            for _ in 0..n {
                tokenise(r, out, depth + 1, None);
            }
            return Vec::new();
        }
        0xa0..=0xbf => ("fixstr", usize::from(c & 0x1f)),
        0xc4 => ("bin8", usize::try_from(r.uint(1)).expect("small")),
        0xc5 => ("bin16", usize::try_from(r.uint(2)).expect("small")),
        0xd9 => ("str8", usize::try_from(r.uint(1)).expect("small")),
        0xda => ("str16", usize::try_from(r.uint(2)).expect("small")),
        0xc0 => {
            emit("nil", "None".into());
            return Vec::new();
        }
        0xc2 | 0xc3 => {
            emit(
                if c == 0xc3 { "true" } else { "false" },
                (if c == 0xc3 { "True" } else { "False" }).into(),
            );
            return Vec::new();
        }
        0xcb => {
            let bits = r.uint(8);
            emit("float64", format!("{:?}", f64::from_bits(bits)));
            return Vec::new();
        }
        0xcc..=0xcf => {
            let width = 1 << (c - 0xcc);
            let v = r.uint(width);
            emit(
                ["uint8", "uint16", "uint32", "uint64"][usize::from(c - 0xcc)],
                v.to_string(),
            );
            return Vec::new();
        }
        0xd0..=0xd3 => {
            let width = 1usize << (c - 0xd0);
            let v = r.uint(width);
            let shift = 64 - 8 * width;
            #[allow(clippy::cast_possible_wrap)]
            let v = ((v << shift) as i64) >> shift;
            emit(
                ["int8", "int16", "int32", "int64"][usize::from(c - 0xd0)],
                v.to_string(),
            );
            return Vec::new();
        }
        _ => {
            emit(&format!("reserved{c:#x}"), "None".into());
            return Vec::new();
        }
    };
    let raw = r.take(len).to_vec();
    emit(token, show(&raw));
    raw
}

/// Every entry of a ShaDa file: its type and its token lines, the header
/// line first (`entry N type=T(...) masked <TS> len=...` without the offset,
/// which moves with everything before it).
fn entries(bytes: &[u8]) -> Vec<(u64, Vec<String>)> {
    let mut r = Reader { buf: bytes, pos: 0 };
    let mut found = Vec::new();
    while r.pos < bytes.len() {
        let mut head = Vec::new();
        tokenise(&mut r, &mut head, 0, None);
        let typ: u64 = head[0]
            .rsplit(' ')
            .next()
            .and_then(|v| v.parse().ok())
            .expect("a type");
        tokenise(&mut r, &mut head, 0, Some("<TS>"));
        let len_at = r.pos;
        tokenise(&mut r, &mut head, 0, None);
        let mut len_reader = Reader {
            buf: bytes,
            pos: len_at,
        };
        let len = match len_reader.take(1)[0] {
            c @ 0..=0x7f => u64::from(c),
            0xcc => len_reader.uint(1),
            0xcd => len_reader.uint(2),
            _ => len_reader.uint(4),
        };
        let len = usize::try_from(len).expect("small");
        let payload = r.take(len).to_vec();
        let mut lines = vec![format!("type={typ} {} len={}", head[1], head[2])];
        tokenise(
            &mut Reader {
                buf: &payload,
                pos: 0,
            },
            &mut lines,
            1,
            None,
        );
        found.push((typ, lines));
    }
    found
}

/// `:wshada!` the editor's state under the 'shada' given, and answer the
/// entries `keep` selects, one string each, with the sandbox's path written
/// `<DIR>`.
fn written(
    fx: &Probe,
    sandbox: &Sandbox,
    shada: &str,
    keep: impl Fn(u64, &str) -> bool,
) -> Vec<String> {
    fx.run(&format!("set shada={shada}"));
    let path = sandbox.path("out.shada");
    let _ = std::fs::remove_file(&path);
    fx.check_error(&format!("wshada! {}", path.display()), "none");
    let bytes = std::fs::read(&path).expect("the written file");
    entries(&bytes)
        .into_iter()
        .map(|(typ, lines)| (typ, lines.join("\n")))
        .filter(|(typ, text)| keep(*typ, text))
        .map(|(_, text)| text.replace(sandbox.as_str(), "<DIR>"))
        .collect()
}

fn restore(fx: &Probe) {
    fx.home();
    fx.run("set shada&");
}

#[test]
fn the_header_names_its_writer() {
    let sandbox = Sandbox::dir("shada_header");
    let fx = Probe::new(sandbox.editor());
    let got = written(&fx, &sandbox, "'0,<0,f0,:0,/0,@0,s1,h", |typ, _| typ == 1);
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=1 masked <TS> len=fixint 91",
            "  fixmap {5}",
            "    fixstr 'generator'",
            "    bin8 'nvim'",
            "    fixstr 'version'",
            "    masked <VERSION>",
            "    fixstr 'max_kbyte'",
            "    fixint 1",
            "    fixstr 'pid'",
            "    masked <PID>",
            "    fixstr 'encoding'",
            "    bin8 'utf-8'",
        ]
    );
    restore(&fx);
}

#[test]
fn registers_keep_their_type_width_and_lines() {
    let sandbox = Sandbox::dir("shada_registers");
    let fx = Probe::new(sandbox.editor());
    fx.run("call setreg('a', 'chars', 'c')");
    fx.run("call setreg('b', ['line one', 'line two'], 'l')");
    fx.run("call setreg('c', ['ab', 'cde'], 'b5')");
    fx.run("call setreg('d', repeat('x', 40), 'c')");
    let names = [
        "'n'\n    fixint 97",
        "'n'\n    fixint 98",
        "'n'\n    fixint 99",
        "'n'\n    fixint 100",
    ];
    let got = written(&fx, &sandbox, "'0,<50,f0,:0,/0,@0,s10,h", |typ, text| {
        typ == 5 && names.iter().any(|n| text.contains(n))
    });
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=5 masked <TS> len=fixint 15",
            "  fixmap {2}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'chars'",
            "    fixstr 'n'",
            "    fixint 97",
            "type=5 masked <TS> len=fixint 32",
            "  fixmap {3}",
            "    fixstr 'rc'",
            "    fixarray [2]",
            "      bin8 'line one'",
            "      bin8 'line two'",
            "    fixstr 'n'",
            "    fixint 98",
            "    fixstr 'rt'",
            "    fixint 1",
            "type=5 masked <TS> len=fixint 25",
            "  fixmap {4}",
            "    fixstr 'rc'",
            "    fixarray [2]",
            "      bin8 'ab'",
            "      bin8 'cde'",
            "    fixstr 'n'",
            "    fixint 99",
            "    fixstr 'rt'",
            "    fixint 2",
            "    fixstr 'rw'",
            "    fixint 4",
            "type=5 masked <TS> len=fixint 50",
            "  fixmap {2}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'",
            "    fixstr 'n'",
            "    fixint 100",
        ]
    );
    restore(&fx);
}

#[test]
fn history_entries_of_every_kind() {
    let sandbox = Sandbox::dir("shada_history");
    let fx = Probe::new(sandbox.editor());
    for kind in ["cmd", "search", "expr", "input", "debug"] {
        fx.run(&format!("call histdel('{kind}')"));
    }
    fx.run("call histadd('cmd', 'echo \"one\"')");
    fx.run("call histadd('search', 'needle')");
    fx.run("call histadd('expr', '1 + 2')");
    fx.run("call histadd('input', 'typed')");
    let got = written(&fx, &sandbox, "'0,<0,f0,:10,/10,@10,s1,h", |typ, _| {
        typ == 4
    });
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=4 masked <TS> len=fixint 14",
            "  fixarray [2]",
            "    fixint 0",
            "    bin8 'echo \"one\"'",
            "type=4 masked <TS> len=fixint 11",
            "  fixarray [3]",
            "    fixint 1",
            "    bin8 'needle'",
            "    fixint 0",
            "type=4 masked <TS> len=fixint 9",
            "  fixarray [2]",
            "    fixint 2",
            "    bin8 '1 + 2'",
            "type=4 masked <TS> len=fixint 9",
            "  fixarray [2]",
            "    fixint 3",
            "    bin8 'typed'",
        ]
    );
    restore(&fx);
}

#[test]
fn variables_of_every_type() {
    let sandbox = Sandbox::dir("shada_variables");
    let fx = Probe::new(sandbox.editor());
    for (name, value) in [
        ("NUM", "42"),
        ("BIG", "70000"),
        ("NEG", "-5"),
        ("STR", "'text'"),
        ("FLOAT", "1.5"),
        ("LIST", "[1, 'two', [3]]"),
        ("DICT", "{'a': 1}"),
        ("BLOB", "0z0011"),
        ("TRUE", "v:true"),
    ] {
        fx.run(&format!("let g:SHADA_{name} = {value}"));
    }
    let mut got = written(&fx, &sandbox, "!,'0,<0,f0,:0,/0,@0,s1,h", |typ, text| {
        typ == 6 && text.contains("'SHADA_")
    });
    // Written in the order the variables' hash table walks, which depends
    // on every other global the shared editor holds: compared sorted.
    got.sort();
    fx.run("unlet g:SHADA_NUM g:SHADA_BIG g:SHADA_NEG g:SHADA_STR g:SHADA_FLOAT g:SHADA_LIST g:SHADA_DICT g:SHADA_BLOB g:SHADA_TRUE");
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=6 masked <TS> len=fixint 13",
            "  fixarray [2]",
            "    bin8 'SHADA_NEG'",
            "    negfixint -5",
            "type=6 masked <TS> len=fixint 13",
            "  fixarray [2]",
            "    bin8 'SHADA_NUM'",
            "    fixint 42",
            "type=6 masked <TS> len=fixint 14",
            "  fixarray [2]",
            "    bin8 'SHADA_TRUE'",
            "    true True",
            "type=6 masked <TS> len=fixint 17",
            "  fixarray [2]",
            "    bin8 'SHADA_BIG'",
            "    uint32 70000",
            "type=6 masked <TS> len=fixint 17",
            "  fixarray [2]",
            "    bin8 'SHADA_DICT'",
            "    fixmap {1}",
            "      fixstr 'a'",
            "      fixint 1",
            "type=6 masked <TS> len=fixint 18",
            "  fixarray [2]",
            "    bin8 'SHADA_STR'",
            "    bin8 'text'",
            "type=6 masked <TS> len=fixint 18",
            "  fixarray [3]",
            "    bin8 'SHADA_BLOB'",
            "    bin8 '\\x00\\x11'",
            "    fixint 10",
            "type=6 masked <TS> len=fixint 22",
            "  fixarray [2]",
            "    bin8 'SHADA_LIST'",
            "    fixarray [3]",
            "      fixint 1",
            "      bin8 'two'",
            "      fixarray [1]",
            "        fixint 3",
            "type=6 masked <TS> len=fixint 23",
            "  fixarray [2]",
            "    bin8 'SHADA_FLOAT'",
            "    float64 1.5",
        ]
    );
    restore(&fx);
}

#[test]
fn marks_name_their_file_and_position() {
    let sandbox = Sandbox::dir("shada_marks");
    sandbox.write("f.txt", b"one\ntwo\nthree\n");
    let fx = Probe::new(sandbox.editor());
    fx.run("delmarks A-Z");
    fx.run("edit f.txt");
    fx.run("call cursor(2, 2)");
    fx.run("mark Q");
    fx.run("call cursor(3, 1)");
    fx.run("mark a");
    let got = written(&fx, &sandbox, "'10,<0,f1,:0,/0,@0,s1,h", |typ, text| {
        (typ == 7 || typ == 10) && text.contains("f.txt")
    });
    fx.run("delmarks Q a");
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=7 masked <TS> len=fixint 60",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 81",
            "type=7 masked <TS> len=fixint 60",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'n'",
            "    fixint 48",
            "type=10 masked <TS> len=fixint 60",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'n'",
            "    fixint 97",
            "type=10 masked <TS> len=fixint 57",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
        ]
    );
    restore(&fx);
}

#[test]
fn the_search_pattern_and_substitute_string() {
    let sandbox = Sandbox::dir("shada_search");
    let fx = Probe::new(sandbox.editor());
    fx.run("call setline(1, 'a needle here')");
    fx.run("s/needle/pin/");
    fx.run("let @/ = 'hay'");
    let got = written(&fx, &sandbox, "'0,<0,f0,:0,/10,@0,s1,h", |typ, _| {
        typ == 2 || typ == 3
    });
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=2 masked <TS> len=fixint 13",
            "  fixmap {2}",
            "    fixstr 'sp'",
            "    bin8 'hay'",
            "    fixstr 'sc'",
            "    true True",
            "type=2 masked <TS> len=fixint 24",
            "  fixmap {4}",
            "    fixstr 'sp'",
            "    bin8 'needle'",
            "    fixstr 'su'",
            "    false False",
            "    fixstr 'sc'",
            "    true True",
            "    fixstr 'ss'",
            "    true True",
            "type=3 masked <TS> len=fixint 6",
            "  fixarray [1]",
            "    bin8 'pin'",
        ]
    );
    restore(&fx);
}

/// A ShaDa file built byte by byte, with fixed timestamps: a register, a
/// variable and a history entry. `rshada!` has to restore each.
#[test]
fn reading_a_file_packed_by_hand() {
    let sandbox = Sandbox::dir("shada_read");
    let fx = Probe::new(sandbox.editor());
    let entry = |typ: u8, payload: &[u8]| -> Vec<u8> {
        let mut out = vec![typ, 0xce, 0x3b, 0x9a, 0xca, 0x00];
        out.push(u8::try_from(payload.len()).expect("a short payload"));
        out.extend_from_slice(payload);
        out
    };
    let mut file = Vec::new();
    // Register `q`: {n: 'q', rc: ['hand', 'made'], rt: 1}.
    file.extend(entry(
        5,
        b"\x83\xa1n\x71\xa2rc\x92\xc4\x04hand\xc4\x04made\xa2rt\x01",
    ));
    // Variable HANDVAR = 'packed'.
    file.extend(entry(6, b"\x92\xc4\x07HANDVAR\xc4\x06packed"));
    // Command-line history: [0, 'echo "by hand"'].
    file.extend(entry(4, b"\x92\x00\xc4\x0eecho \"by hand\""));
    let path = sandbox.write("in.shada", &file);
    fx.run("call histdel('cmd')");
    fx.run("set shada=!,'10,<50,:10,s10");
    fx.check_error(&format!("rshada! {}", path.display()), "none");
    fx.expect_all(&[
        (
            "[getreg('q', 1, 1), getregtype('q')]",
            "[['hand', 'made'], 'V']",
        ),
        ("get(g:, 'HANDVAR', 'unset')", "'packed'"),
        ("histget('cmd', -1)", "'echo \"by hand\"'"),
    ]);
    fx.run("call setreg('q', [])");
    fx.run("unlet! g:HANDVAR");
    fx.run("call histdel('cmd')");
    restore(&fx);
}
