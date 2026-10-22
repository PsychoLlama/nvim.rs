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
        // The header's length follows its version string and pid, which are
        // masked inside it, so it is masked too.
        let len_text = if typ == 1 { "<LEN>" } else { &head[2] };
        let mut lines = vec![format!("type={typ} {} len={len_text}", head[1])];
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
        .map(|(_, text)| mask_dir(&text, sandbox.as_str()))
        .collect()
}

/// `text` with the sandbox directory written `<DIR>` -- and, since the
/// entry's length counts the directory's bytes, its `len=fixint N` written
/// `len=<DIR>+M` with the directory taken out, so the answer does not
/// depend on what `$TMPDIR` is called.
fn mask_dir(text: &str, dir: &str) -> String {
    let uses = text.matches(dir).count();
    let mut masked = text.replace(dir, "<DIR>");
    if uses > 0 {
        for token in ["len=fixint ", "len=uint8 ", "len=uint16 "] {
            let Some(at) = masked.find(token) else {
                continue;
            };
            let start = at + token.len();
            let end = masked[start..]
                .find(|c: char| !c.is_ascii_digit())
                .map_or(masked.len(), |n| start + n);
            let len: usize = masked[start..end].parse().expect("a length");
            let rest = len - uses * dir.len();
            masked.replace_range(at..end, &format!("len=<DIR>+{rest}"));
            break;
        }
    }
    masked
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
            "type=1 masked <TS> len=<LEN>",
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
            "type=7 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 81",
            "type=7 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'n'",
            "    fixint 48",
            "type=10 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'n'",
            "    fixint 97",
            "type=10 masked <TS> len=<DIR>+14",
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

/// One entry of a hand-built file: `typ`, the timestamp `ts` as a uint32,
/// and the payload's length as a fixint, uint8 or uint16.
pub(crate) fn entry_bytes(typ: u8, ts: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![typ, 0xce];
    out.extend_from_slice(&ts.to_be_bytes());
    let len = payload.len();
    if len < 0x80 {
        out.push(u8::try_from(len).expect("checked"));
    } else if len < 0x100 {
        out.extend_from_slice(&[0xcc, u8::try_from(len).expect("checked")]);
    } else {
        out.push(0xcd);
        out.extend_from_slice(&u16::try_from(len).expect("short").to_be_bytes());
    }
    out.extend_from_slice(payload);
    out
}

/// A msgpack bin8 holding `bytes`.
pub(crate) fn bin(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![0xc4, u8::try_from(bytes.len()).expect("short")];
    out.extend_from_slice(bytes);
    out
}

/// A fixmap of `pairs`, each already encoded.
pub(crate) fn map(pairs: &[(&[u8], Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0x80 | u8::try_from(pairs.len()).expect("few")];
    for (key, value) in pairs {
        out.push(0xa0 | u8::try_from(key.len()).expect("short"));
        out.extend_from_slice(key);
        out.extend_from_slice(value);
    }
    out
}

/// `nvim --headless -u NONE <args>` in `dir`, with `$HOME` and the XDG
/// directories inside it; waits at most ten seconds and answers stderr.
fn child(dir: &std::path::Path, args: &[&str]) -> String {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let runtime = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runtime");
    let mut argv = vec!["--headless", "-u", "NONE"];
    argv.extend_from_slice(args);
    let mut proc = Command::new(env!("CARGO_BIN_EXE_nvim"))
        .args(&argv)
        .current_dir(dir)
        .env_clear()
        .env("HOME", dir)
        .env("PATH", "/usr/bin:/bin")
        .env("VIMRUNTIME", runtime)
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .env("XDG_DATA_HOME", dir.join("data"))
        .env("XDG_STATE_HOME", dir.join("state"))
        .env("XDG_CACHE_HOME", dir.join("cache"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("nvim starts");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if proc.try_wait().expect("the child").is_some() {
            break;
        }
        if Instant::now() > deadline {
            let _ = proc.kill();
            panic!("nvim {argv:?} did not exit");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut stderr = String::new();
    {
        use std::io::Read;
        let _ = proc
            .stderr
            .take()
            .expect("a stderr pipe")
            .read_to_string(&mut stderr);
    }
    stderr
}

/// Every entry of `path` but the header, one string each, the sandbox's
/// directory written `<DIR>`.
fn file_entries(path: &std::path::Path, dir: &str) -> Vec<String> {
    let bytes = std::fs::read(path).expect("the written file");
    entries(&bytes)
        .into_iter()
        .filter(|(typ, _)| *typ != 1)
        .map(|(_, lines)| mask_dir(&lines.join("\n"), dir))
        .collect()
}

/// Every kind of entry that carries a map or an array, each with a key or
/// element this Nvim has no field for, read at startup and written back:
/// what reaches the editor's state keeps its extra data, and a variable's
/// is dropped.
#[test]
fn extra_data_rides_along_with_every_kind() {
    let sandbox = Sandbox::dir("shada_extra");
    let dir = sandbox.as_str().to_owned();
    let fname = format!("{dir}/f.txt");
    sandbox.write("f.txt", b"one\ntwo\nthree\n");
    let extra = (&b"zz"[..], vec![0x07]);
    let ts = 1_000_000_000;
    let mut file = Vec::new();
    // Buffer list first, so that the local marks below find its buffer.
    let mut buflist = vec![0x91];
    buflist.extend(map(&[
        (b"f", bin(fname.as_bytes())),
        (b"l", vec![0x02]),
        extra.clone(),
    ]));
    file.extend(entry_bytes(9, ts, &buflist));
    file.extend(entry_bytes(
        2,
        ts,
        &map(&[(b"sp", bin(b"pat")), (b"sc", vec![0xc3]), extra.clone()]),
    ));
    let mut sub = vec![0x92];
    sub.extend(bin(b"rep"));
    sub.push(0x07);
    file.extend(entry_bytes(3, ts, &sub));
    let mut hist = vec![0x93, 0x00];
    hist.extend(bin(b"cmdline"));
    hist.push(0x07);
    file.extend(entry_bytes(4, ts, &hist));
    let mut shist = vec![0x94, 0x01];
    shist.extend(bin(b"spat"));
    shist.extend([b'?', 0x07]);
    file.extend(entry_bytes(4, ts, &shist));
    file.extend(entry_bytes(
        5,
        ts,
        &map(&[
            (b"n", vec![b'r']),
            (b"rc", [vec![0x91], bin(b"x")].concat()),
            extra.clone(),
        ]),
    ));
    let mut var = vec![0x93];
    var.extend(bin(b"XV"));
    var.extend([0x05, 0x07]);
    file.extend(entry_bytes(6, ts, &var));
    file.extend(entry_bytes(
        7,
        ts,
        &map(&[
            (b"n", vec![b'G']),
            (b"f", bin(b"/nonexistent/other")),
            (b"l", vec![0x02]),
            extra.clone(),
        ]),
    ));
    file.extend(entry_bytes(
        8,
        ts,
        &map(&[
            (b"f", bin(b"/nonexistent/other")),
            (b"l", vec![0x03]),
            extra.clone(),
        ]),
    ));
    file.extend(entry_bytes(
        10,
        ts,
        &map(&[
            (b"n", vec![b'b']),
            (b"f", bin(fname.as_bytes())),
            (b"l", vec![0x02]),
            extra.clone(),
        ]),
    ));
    file.extend(entry_bytes(
        11,
        ts,
        &map(&[
            (b"f", bin(fname.as_bytes())),
            (b"l", vec![0x03]),
            (b"c", vec![0x01]),
            extra.clone(),
        ]),
    ));
    sandbox.write("in.shada", &file);
    let stderr = child(
        sandbox.root(),
        &[
            "-i",
            "in.shada",
            "--cmd",
            "set shada=%,!,'10,<50,:10,/10,@10,f1,s10",
            "-c",
            "wshada! out.shada",
            "-c",
            "set shada=",
            "-c",
            "qa!",
        ],
    );
    assert_eq!(stderr, "");
    let got = file_entries(&sandbox.path("out.shada"), &dir);
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=9 masked <TS> len=<DIR>+19",
            "  fixarray [1]",
            "    fixmap {3}",
            "      fixstr 'f'",
            "      bin8 '<DIR>/f.txt'",
            "      fixstr 'l'",
            "      fixint 2",
            "      fixstr 'zz'",
            "      fixint 7",
            "type=6 masked <TS> len=fixint 6",
            "  fixarray [2]",
            "    bin8 'XV'",
            "    fixint 5",
            "type=7 masked <TS> len=fixint 33",
            "  fixmap {4}",
            "    fixstr 'f'",
            "    bin8 '/nonexistent/other'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 71",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=5 masked <TS> len=fixint 15",
            "  fixmap {3}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'x'",
            "    fixstr 'n'",
            "    fixint 114",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=8 masked <TS> len=fixint 30",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '/nonexistent/other'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=2 masked <TS> len=fixint 17",
            "  fixmap {3}",
            "    fixstr 'sp'",
            "    bin8 'pat'",
            "    fixstr 'sc'",
            "    true True",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=3 masked <TS> len=fixint 7",
            "  fixarray [2]",
            "    bin8 'rep'",
            "    fixint 7",
            "type=10 masked <TS> len=<DIR>+21",
            "  fixmap {4}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 98",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=10 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "type=11 masked <TS> len=<DIR>+21",
            "  fixmap {4}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'c'",
            "    fixint 1",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=4 masked <TS> len=fixint 12",
            "  fixarray [3]",
            "    fixint 0",
            "    bin8 'cmdline'",
            "    fixint 7",
            "type=4 masked <TS> len=fixint 10",
            "  fixarray [4]",
            "    fixint 1",
            "    bin8 'spat'",
            "    fixint 63",
            "    fixint 7"
        ]
    );
}

/// What the merge does with entries this Nvim has no slot for (an unknown
/// type, a register, mark or history it does not know), with a jump both
/// sides hold, and with a numbered mark the old file lists twice.
#[test]
fn the_merge_keeps_what_it_cannot_place_and_one_of_each_jump() {
    let sandbox = Sandbox::dir("shada_merge");
    let dir = sandbox.as_str().to_owned();
    let fname = format!("{dir}/f.txt");
    sandbox.write("f.txt", b"one\ntwo\nthree\nfour\n");
    let old = 1_000_000_000;
    let future = 4_000_000_000;
    let mut file = Vec::new();
    // An entry type this Nvim does not know, kept byte for byte.
    file.extend(entry_bytes(100, old, &[0x92, 0x01, 0xc4, 0x01, b'u']));
    file.extend(entry_bytes(
        5,
        old,
        &map(&[
            (b"n", vec![b'!']),
            (b"rc", [vec![0x91], bin(b"bang")].concat()),
        ]),
    ));
    file.extend(entry_bytes(
        7,
        old,
        &map(&[(b"n", vec![b'!']), (b"f", bin(b"/nonexistent/g"))]),
    ));
    let mut hist = vec![0x92, 0x07];
    hist.extend(bin(b"seventh"));
    file.extend(entry_bytes(4, old, &hist));
    let mut var = vec![0x93];
    var.extend(bin(b"OLDVAR"));
    var.extend([0x05, 0x07]);
    file.extend(entry_bytes(6, old, &var));
    for _ in 0..2 {
        file.extend(entry_bytes(
            7,
            old,
            &map(&[
                (b"n", vec![b'1']),
                (b"f", bin(b"/nonexistent/n")),
                (b"l", vec![0x04]),
            ]),
        ));
    }
    // The jump the editor below also holds: once newer than it, once
    // older.
    for ts in [future, old] {
        file.extend(entry_bytes(
            8,
            ts,
            &map(&[(b"f", bin(fname.as_bytes())), (b"l", vec![0x03])]),
        ));
    }
    file.extend(entry_bytes(
        10,
        old,
        &map(&[
            (b"n", vec![b'!']),
            (b"f", bin(fname.as_bytes())),
            (b"l", vec![0x02]),
        ]),
    ));
    sandbox.write("merge.shada", &file);
    let stderr = child(
        sandbox.root(),
        &[
            "-i",
            "NONE",
            "--cmd",
            "set shada=!,'10,<50,:10,/0,@0,f1,s10",
            "f.txt",
            "-c",
            "normal! 3G",
            "-c",
            "wshada merge.shada",
            "-c",
            "qa!",
        ],
    );
    assert_eq!(stderr, "");
    let got = file_entries(&sandbox.path("merge.shada"), &dir);
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=100 masked <TS> len=fixint 5",
            "  fixarray [2]",
            "    fixint 1",
            "    bin8 'u'",
            "type=5 masked <TS> len=fixint 14",
            "  fixmap {2}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'bang'",
            "    fixstr 'n'",
            "    fixint 33",
            "type=7 masked <TS> len=fixint 22",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '/nonexistent/g'",
            "    fixstr 'n'",
            "    fixint 33",
            "type=4 masked <TS> len=fixint 11",
            "  fixarray [2]",
            "    fixint 7",
            "    bin8 'seventh'",
            "type=6 masked <TS> len=fixint 11",
            "  fixarray [3]",
            "    bin8 'OLDVAR'",
            "    fixint 5",
            "    fixint 7",
            "type=7 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "    fixstr 'n'",
            "    fixint 48",
            "type=7 masked <TS> len=fixint 25",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '/nonexistent/n'",
            "    fixstr 'l'",
            "    fixint 4",
            "    fixstr 'n'",
            "    fixint 49",
            "type=8 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "type=8 masked <TS> len=<DIR>+11",
            "  fixmap {1}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "type=8 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "type=10 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 3",
            "type=10 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 33"
        ]
    );
}

/// Every motion type, block widths across the fixint/uint8/uint16 cuts,
/// a line on each side of bin8's 255-byte step, and an array16.
#[test]
fn registers_of_every_type_and_width() {
    let sandbox = Sandbox::dir("shada_reg_widths");
    let fx = Probe::new(sandbox.editor());
    fx.run("call setreg('e', ['ab'], 'b')");
    fx.run("call setreg('f', ['ab'], 'b200')");
    fx.run("call setreg('g', ['ab'], 'b300')");
    fx.run("call setreg('h', [repeat('y', 254), repeat('z', 255)], 'l')");
    fx.run("call setreg('i', map(range(17), 'string(v:val)'), 'c')");
    let names = [
        "'n'\n    fixint 101",
        "'n'\n    fixint 102",
        "'n'\n    fixint 103",
        "'n'\n    fixint 104",
        "'n'\n    fixint 105",
    ];
    let got = written(&fx, &sandbox, "'0,<50,f0,:0,/0,@0,s10,h", |typ, text| {
        typ == 5 && names.iter().any(|n| text.contains(n))
    });
    for r in ["e", "f", "g", "h", "i"] {
        fx.run(&format!("call setreg('{r}', [])"));
    }
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=5 masked <TS> len=fixint 20",
            "  fixmap {4}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'ab'",
            "    fixstr 'n'",
            "    fixint 101",
            "    fixstr 'rt'",
            "    fixint 2",
            "    fixstr 'rw'",
            "    fixint 1",
            "type=5 masked <TS> len=fixint 21",
            "  fixmap {4}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'ab'",
            "    fixstr 'n'",
            "    fixint 102",
            "    fixstr 'rt'",
            "    fixint 2",
            "    fixstr 'rw'",
            "    uint8 199",
            "type=5 masked <TS> len=fixint 22",
            "  fixmap {4}",
            "    fixstr 'rc'",
            "    fixarray [1]",
            "      bin8 'ab'",
            "    fixstr 'n'",
            "    fixint 103",
            "    fixstr 'rt'",
            "    fixint 2",
            "    fixstr 'rw'",
            "    uint16 299",
            "type=5 masked <TS> len=uint16 526",
            "  fixmap {3}",
            "    fixstr 'rc'",
            "    fixarray [2]",
            "      bin8 'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy'",
            "      bin16 'zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz'",
            "    fixstr 'n'",
            "    fixint 104",
            "    fixstr 'rt'",
            "    fixint 1",
            "type=5 masked <TS> len=fixint 68",
            "  fixmap {2}",
            "    fixstr 'rc'",
            "    array16 [17]",
            "      bin8 '0'",
            "      bin8 '1'",
            "      bin8 '2'",
            "      bin8 '3'",
            "      bin8 '4'",
            "      bin8 '5'",
            "      bin8 '6'",
            "      bin8 '7'",
            "      bin8 '8'",
            "      bin8 '9'",
            "      bin8 '10'",
            "      bin8 '11'",
            "      bin8 '12'",
            "      bin8 '13'",
            "      bin8 '14'",
            "      bin8 '15'",
            "      bin8 '16'",
            "    fixstr 'n'",
            "    fixint 105"
        ]
    );
    restore(&fx);
}

/// A mark whose file name is not ASCII, and the header's integer widths
/// past the 28-bit cut the writer keeps for unsigned values.
#[test]
fn a_multibyte_file_name_and_a_wide_max_kbyte() {
    let sandbox = Sandbox::dir("shada_multibyte");
    sandbox.write("été.txt", b"one\ntwo\n");
    let fx = Probe::new(sandbox.editor());
    fx.run("delmarks A-Z");
    fx.run("badd été.txt");
    fx.run("buffer été.txt");
    fx.run("call cursor(2, 1)");
    fx.run("mark M");
    let got = written(
        &fx,
        &sandbox,
        "'0,<0,f1,:0,/0,@0,s300000000,h",
        |typ, text| typ == 1 || (typ == 7 && text.contains("'n'\n    fixint 77")),
    );
    fx.run("delmarks M");
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=1 masked <TS> len=<LEN>",
            "  fixmap {5}",
            "    fixstr 'generator'",
            "    bin8 'nvim'",
            "    fixstr 'version'",
            "    masked <VERSION>",
            "    fixstr 'max_kbyte'",
            "    uint64 300000000",
            "    fixstr 'pid'",
            "    masked <PID>",
            "    fixstr 'encoding'",
            "    bin8 'utf-8'",
            "type=7 masked <TS> len=<DIR>+21",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/\\xc3\\xa9t\\xc3\\xa9.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 77"
        ]
    );
    restore(&fx);
}

/// History of each of the five kinds, a search with a `?` separator, and a
/// line long enough for a bin8 with a uint8 entry length.
#[test]
fn history_of_each_kind_with_separators() {
    let sandbox = Sandbox::dir("shada_history_kinds");
    let fx = Probe::new(sandbox.editor());
    for kind in ["cmd", "search", "expr", "input", "debug"] {
        fx.run(&format!("call histdel('{kind}')"));
    }
    fx.run(&format!("call histadd('cmd', '{}')", "c".repeat(140)));
    fx.run("call histadd('search', 'fwd')");
    fx.run("call histadd('debug', 'step')");
    let got = written(&fx, &sandbox, "'0,<0,f0,:10,/10,@10,s10,h", |typ, _| {
        typ == 4
    });
    for kind in ["cmd", "search", "debug"] {
        fx.run(&format!("call histdel('{kind}')"));
    }
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=4 masked <TS> len=uint8 144",
            "  fixarray [2]",
            "    fixint 0",
            "    bin8 'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'",
            "type=4 masked <TS> len=fixint 8",
            "  fixarray [3]",
            "    fixint 1",
            "    bin8 'fwd'",
            "    fixint 0",
            "type=4 masked <TS> len=fixint 8",
            "  fixarray [2]",
            "    fixint 4",
            "    bin8 'step'"
        ]
    );
    restore(&fx);
}

/// `:wshada!` and `:wshada` over a file this process may not open: the
/// open fails and says so, and nothing is renamed over it.
#[test]
fn writing_over_an_unreadable_file() {
    let sandbox = Sandbox::dir("shada_unreadable");
    let fx = Probe::new(sandbox.editor());
    let path = sandbox.write("locked.shada", b"");
    let mut perm = std::fs::metadata(&path).expect("the file").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o000);
    std::fs::set_permissions(&path, perm).expect("chmod");
    fx.run("set shada='0,<0,f0,:0,/0,@0,s10,h");
    let messages = |cmd: String| {
        fx.run("messages clear");
        fx.run(&cmd);
        fx.output("messages")
            .trim()
            .replace(sandbox.as_str(), "<DIR>")
    };
    let forced = messages(format!("wshada! {}", path.display()));
    let merged = messages(format!("wshada {}", path.display()));
    let left = std::fs::read_dir(sandbox.root())
        .expect("the sandbox")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .into_string()
                .expect("text")
        })
        .collect::<Vec<_>>();
    let mut perm = std::fs::metadata(&path).expect("the file").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o600);
    std::fs::set_permissions(&path, perm).expect("chmod");
    assert_eq!(
        [forced, merged],
        [
            "E886: System error while opening ShaDa file <DIR>/locked.shada for writing: permission denied",
            "E886: System error while opening ShaDa file <DIR>/locked.shada for reading to merge before writing it: permission denied\nE886: System error while opening ShaDa file <DIR>/locked.shada for writing: permission denied"
        ]
    );
    assert_eq!(left, ["locked.shada"]);
    restore(&fx);
}

/// Marks read with extra data (a global mark, a local one, a change and a
/// jump, all in one file), then moved by `:move`, undo, redo and a
/// deletion before the file is written back: which extras survive, and
/// where each mark ends up. A local mark's extra is dropped the first time
/// an undo step is saved -- the undo header keeps a copy of the marks, and
/// upstream will not let two copies own one payload.
#[test]
fn mark_extra_data_under_edits() {
    let sandbox = Sandbox::dir("shada_mark_extra");
    let dir = sandbox.as_str().to_owned();
    let fname = format!("{dir}/f.txt");
    sandbox.write("f.txt", b"l1\nl2\nl3\nl4\nl5\nl6\n");
    let extra = (&b"zz"[..], vec![0x07]);
    let ts = 1_000_000_000;
    let mut file = Vec::new();
    for (typ, name, line) in [
        (7, Some(b'G'), 4),
        (10, Some(b'b'), 4),
        (10, Some(b'c'), 5),
        (11, None, 4),
        (8, None, 5),
    ] {
        let mut pairs = vec![(&b"f"[..], bin(fname.as_bytes())), (b"l", vec![line])];
        if let Some(name) = name {
            pairs.push((b"n", vec![name]));
        }
        pairs.push(extra.clone());
        file.extend(entry_bytes(typ, ts, &map(&pairs)));
    }
    sandbox.write("in.shada", &file);
    let sync = "let &undolevels = &undolevels";
    let stderr = child(
        sandbox.root(),
        &[
            "-i",
            "in.shada",
            "--cmd",
            "set shada='10,f1",
            "-c",
            "edit f.txt",
            "-c",
            "3,4move 6",
            "-c",
            sync,
            "-c",
            "silent undo",
            "-c",
            "silent redo",
            "-c",
            "1delete",
            "-c",
            sync,
            "-c",
            "wshada! out.shada",
            "-c",
            "set shada=",
            "-c",
            "qa!",
        ],
    );
    assert_eq!(stderr, "");
    let got: Vec<String> = file_entries(&sandbox.path("out.shada"), &dir)
        .into_iter()
        .filter(|entry| entry.contains("<DIR>/f.txt"))
        .collect();
    assert_eq!(
        got.join("\n").lines().collect::<Vec<_>>(),
        [
            "type=7 masked <TS> len=<DIR>+21",
            "  fixmap {4}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 6",
            "    fixstr 'n'",
            "    fixint 71",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=7 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'n'",
            "    fixint 48",
            "type=8 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 6",
            "type=8 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "type=8 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 5",
            "type=8 masked <TS> len=<DIR>+11",
            "  fixmap {1}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "type=10 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 5",
            "    fixstr 'n'",
            "    fixint 98",
            "type=10 masked <TS> len=<DIR>+17",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "    fixstr 'n'",
            "    fixint 99",
            "type=10 masked <TS> len=<DIR>+11",
            "  fixmap {1}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "type=10 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'n'",
            "    fixint 46",
            "type=11 masked <TS> len=<DIR>+18",
            "  fixmap {3}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 6",
            "    fixstr 'zz'",
            "    fixint 7",
            "type=11 masked <TS> len=<DIR>+14",
            "  fixmap {2}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
            "    fixstr 'l'",
            "    fixint 2",
            "type=11 masked <TS> len=<DIR>+11",
            "  fixmap {1}",
            "    fixstr 'f'",
            "    bin8 '<DIR>/f.txt'",
        ]
    );
}
