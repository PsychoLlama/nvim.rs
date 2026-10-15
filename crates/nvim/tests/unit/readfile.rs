//! Reading a file into a buffer: what `readfile` makes of the bytes.
//!
//! The read guesses twice -- the encoding from `'fileencodings'` (or takes
//! `++enc=`), the line ending from `'fileformats'` (or takes `++ff=`) -- and
//! may read the whole file again when a guess was wrong. It reads in chunks
//! (64 KiB, less the room a conversion needs), so a character can be split
//! across two reads and has to be carried over; it recognises a byte-order
//! mark, keeps a NUL as a newline inside the line, and hands an encoding it
//! cannot convert itself to iconv or to `'charconvert'`.
//!
//! Written against the behaviour before the read loop moved from pointers
//! into the read buffer onto indices into an owned one; a row that moves is a
//! behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// What a case saw, one row per observation, compared whole against the
/// recording -- or, with `NVIM_UNIT_RECORD` naming a directory, written
/// there instead, which is how the recording was made on the old code.
struct Seen<'a> {
    fx: &'a Probe<'a>,
    rows: Vec<String>,
}

impl<'a> Seen<'a> {
    fn new(fx: &'a Probe<'a>) -> Self {
        Seen {
            fx,
            rows: Vec::new(),
        }
    }

    /// A message the read printed, or anything else already in hand.
    fn note(&mut self, label: &str, text: &str) {
        self.rows.push(format!("{label} => {text:?}"));
    }

    /// `expr`'s value as `string()` renders it, with long runs collapsed.
    fn eval(&mut self, expr: &str) {
        let got = collapse(&self.fx.eval(expr));
        self.rows.push(format!("{expr} => {got}"));
    }

    #[track_caller]
    fn verify(self, name: &str, want: &str) {
        let got = self.rows.join("\n");
        if let Some(dir) = std::env::var_os("NVIM_UNIT_RECORD") {
            std::fs::write(std::path::Path::new(&dir).join(name), &got).expect("recording");
            return;
        }
        assert_eq!(got, want.trim_start_matches('\n').trim_end());
    }
}

/// `text` with every run of more than eight of one character written
/// `<Nxc>`, so a row about a 64 KiB line stays readable, and every control
/// character as `^X`.
fn collapse(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text
        .chars()
        .flat_map(|c| match u8::try_from(c) {
            Ok(b) if b < 0x20 => vec!['^', char::from(b + 0x40)],
            _ => vec![c],
        })
        .peekable();
    while let Some(c) = chars.next() {
        let mut n = 1;
        while chars.peek() == Some(&c) {
            chars.next();
            n += 1;
        }
        if n > 8 {
            out.push_str(&format!("<{n}x{c}>"));
        } else {
            out.extend(std::iter::repeat_n(c, n));
        }
    }
    out
}

/// Read `name` into a fresh buffer with `opts` (`++enc=…` and the like),
/// answering what the read printed. A buffer an earlier read left for the
/// same name is wiped first: `:edit` would only switch to it.
///
/// The read lands in a buffer of its own, never buffer 1: `:enew` and
/// `:edit` would reuse an empty unnamed buffer 1 and name it, and wiping
/// it then would move every later case's buffer numbers.
fn read(fx: &Probe, opts: &str, name: &str) -> String {
    fresh_buffer(fx, name);
    fx.output(&format!("edit! {opts} {name}"))
}

/// Wipe what an earlier read left for `name`, and stand in a new empty
/// buffer in the only window.
fn fresh_buffer(fx: &Probe, name: &str) {
    fx.run(&format!("silent! bwipeout! {name}"));
    fx.run("new");
    fx.run("only!");
}

fn restore(fx: &Probe) {
    fx.home();
    fx.run(
        "set fileencodings& fileformats& charconvert& shortmess& binary& fileencoding& \
         fileformat& bomb& readonly& endofline& endoffile&",
    );
    fx.run("unlet! g:convlog");
}

#[test]
fn a_utf8_character_split_by_the_first_read_is_carried_over() {
    let sandbox = Sandbox::dir("readfile_utf8_split");
    // The first read is 64 KiB: the two bytes of "é" straddle it.
    let mut text = vec![b'a'; 65535];
    text.extend_from_slice("éz\nsecond\n".as_bytes());
    sandbox.write("f.txt", &text);
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    seen.note("edit", &read(&fx, "", "f.txt"));
    seen.eval("strlen(getline(1))");
    seen.eval("getline(1)[65535:]");
    seen.eval("getline(2)");
    seen.eval("&fileencoding");
    restore(&fx);
    seen.verify("utf8_split", UTF8_SPLIT);
}

#[test]
fn a_surrogate_pair_split_by_the_read_is_carried_over() {
    let sandbox = Sandbox::dir("readfile_utf16_split");
    // UTF-16 reads two thirds of 64 KiB, rounded to even: 43,690 bytes. The
    // pair for U+1F600 starts two bytes before that.
    let mut text = Vec::new();
    for _ in 0..21844 {
        text.extend_from_slice(b"a\0");
    }
    text.extend_from_slice(&[0x3d, 0xd8, 0x00, 0xde]);
    text.extend_from_slice(b"\n\0b\0\n\0");
    sandbox.write("f.txt", &text);
    // An odd byte left at the end of the file.
    sandbox.write("odd.txt", b"a\0b\0\n\0c");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    seen.note("edit", &read(&fx, "++enc=utf-16le", "f.txt"));
    seen.eval("strchars(getline(1))");
    seen.eval("char2nr(strcharpart(getline(1), 21844, 1))");
    seen.eval("getline(2)");
    seen.eval("&fileencoding");
    seen.note("odd", &read(&fx, "++enc=utf-16le", "odd.txt"));
    seen.eval("[getline(1, '$'), &readonly]");
    seen.note("ucs-2", &read(&fx, "++enc=ucs-2le", "odd.txt"));
    seen.eval("[getline(1, '$'), &readonly]");
    seen.note("ucs-4", &read(&fx, "++enc=ucs-4le", "odd.txt"));
    seen.eval("[getline(1, '$'), &readonly]");
    restore(&fx);
    seen.verify("utf16_split", UTF16_SPLIT);
}

#[test]
fn an_iconv_sequence_split_by_the_read_is_carried_over() {
    let sandbox = Sandbox::dir("readfile_iconv_split");
    // iconv reads an eighth of 64 KiB: 8,192 bytes. Shift-JIS "あ" (82 A0)
    // straddles that.
    let mut text = vec![b'a'; 8191];
    text.extend_from_slice(&[0x82, 0xa0]);
    text.extend_from_slice(b"\nb\x82\xa0c\n");
    sandbox.write("f.txt", &text);
    // A byte that is no Shift-JIS at all, and a lead byte the file ends on.
    sandbox.write("bad.txt", b"x\x82\x20y\n\x82");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    seen.note("edit", &read(&fx, "++enc=sjis", "f.txt"));
    seen.eval("strchars(getline(1))");
    seen.eval("char2nr(strcharpart(getline(1), 8191, 1))");
    seen.eval("getline(2)");
    seen.eval("&fileencoding");
    for bad in ["", "++bad=keep", "++bad=drop", "++bad=X"] {
        seen.note(bad, &read(&fx, &format!("++enc=sjis {bad}"), "bad.txt"));
        seen.eval("[getline(1, '$'), &readonly]");
    }
    // Through 'fileencodings', a bad byte moves on to the next guess.
    fx.run("set fileencodings=sjis,latin1");
    seen.note("fencs", &read(&fx, "", "bad.txt"));
    seen.eval("[getline(1, '$'), &fileencoding]");
    restore(&fx);
    seen.verify("iconv_split", ICONV_SPLIT);
}

#[test]
fn fileencodings_falls_back_to_latin1() {
    let sandbox = Sandbox::dir("readfile_fencs");
    sandbox.write("f.txt", b"caf\xe9\nna\xefve\n");
    sandbox.write("hi.txt", &(0x80u8..=0xff).chain(*b"\n").collect::<Vec<_>>());
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    seen.note("default", &read(&fx, "", "f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding]");
    // Without latin1 to fall back to, the last guess keeps the bytes and
    // marks the illegal one.
    fx.run("set fileencodings=utf-8");
    seen.note("utf-8 only", &read(&fx, "", "f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding, &readonly]");
    // `++enc=latin1` converts every high byte.
    seen.note("latin1", &read(&fx, "++enc=latin1", "hi.txt"));
    seen.eval("[strchars(getline(1)), strlen(getline(1)), &fileencoding]");
    seen.eval("char2nr(strcharpart(getline(1), 127, 1))");
    // An encoding nobody can convert: read as it is.
    seen.note("unknown", &read(&fx, "++enc=no-such-encoding", "f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding]");
    // An empty 'fileencodings' takes the buffer's own 'fileencoding'.
    fx.run("set fileencodings=");
    fresh_buffer(&fx, "f.txt");
    fx.run("setlocal fileencoding=latin1");
    seen.note("empty fencs", &fx.output("edit! f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding]");
    restore(&fx);
    seen.verify("fencs", FENCS);
}

#[test]
fn byte_order_marks_are_recognised_and_removed() {
    let sandbox = Sandbox::dir("readfile_bom");
    sandbox.write("u8.txt", b"\xef\xbb\xbfhi\n");
    sandbox.write("u16.txt", b"\xff\xfeh\0i\0\n\0");
    sandbox.write("u16be.txt", b"\xfe\xff\0h\0i\0\n");
    sandbox.write("u32.txt", b"\xff\xfe\0\0h\0\0\0\n\0\0\0");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    for name in ["u8.txt", "u16.txt", "u16be.txt", "u32.txt"] {
        seen.note(name, &read(&fx, "", name));
        seen.eval("[getline(1, '$'), &fileencoding, &bomb]");
    }
    // `++bin` does not look for one.
    seen.note("bin", &read(&fx, "++bin", "u8.txt"));
    seen.eval("[getline(1, '$'), &bomb, &binary]");
    // `++enc=utf-8` keeps looking.
    seen.note("enc", &read(&fx, "++enc=utf-8", "u8.txt"));
    seen.eval("[getline(1, '$'), &bomb]");
    restore(&fx);
    seen.verify("bom", BOM);
}

#[test]
fn nul_bytes_and_line_endings() {
    let sandbox = Sandbox::dir("readfile_eol");
    sandbox.write("nul.txt", b"a\0b\nc\0\n\0");
    sandbox.write("crlf.txt", b"a\r\nb\r\n");
    sandbox.write("mixed.txt", b"a\r\nb\nc\r\n");
    sandbox.write("mac.txt", b"a\rb\nc\0d\r");
    sandbox.write("noeol.txt", b"a\nb");
    sandbox.write("ctrlz.txt", b"a\r\nb\r\n\x1a");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    let cases = [
        ("", "nul.txt"),
        ("", "crlf.txt"),
        ("++bin", "crlf.txt"),
        ("", "mixed.txt"),
        ("++ff=dos", "mixed.txt"),
        ("++ff=mac", "mac.txt"),
        ("", "noeol.txt"),
        ("", "ctrlz.txt"),
        ("++ff=unix", "ctrlz.txt"),
    ];
    for (opts, name) in cases {
        seen.note(&format!("{opts} {name}"), &read(&fx, opts, name));
        seen.eval("[getline(1, '$'), &ff, &eol, &eof, &bin]");
    }
    // 'fileformats' with mac in it counts the carriage returns.
    fx.run("set fileformats=unix,mac");
    seen.note("unix,mac", &read(&fx, "", "mac.txt"));
    seen.eval("[getline(1, '$'), &ff]");
    restore(&fx);
    seen.verify("eol", EOL);
}

#[test]
fn a_line_longer_than_a_read_grows_the_buffer() {
    let sandbox = Sandbox::dir("readfile_long");
    let mut text = vec![b'x'; 1_500_000];
    text.extend_from_slice(b"\nend\n");
    sandbox.write("f.txt", &text);
    // And through latin1, which reads half as much at a time.
    let mut latin = vec![0xe9u8; 200_000];
    latin.extend_from_slice(b"\nend\n");
    sandbox.write("l.txt", &latin);
    let fx = Probe::new(sandbox.editor());
    let mut seen = Seen::new(&fx);
    read(&fx, "", "f.txt");
    seen.eval("[line('$'), strlen(getline(1)), getline(2)]");
    read(&fx, "++enc=latin1", "l.txt");
    seen.eval("[line('$'), strchars(getline(1)), strlen(getline(1)), getline(2)]");
    restore(&fx);
    seen.verify("long", LONG);
}

#[test]
fn charconvert_converts_what_nothing_else_can() {
    let sandbox = Sandbox::dir("readfile_charconvert");
    sandbox.write("f.txt", b"one\ntwo\n");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    // Upper-cases the file, and notes what it was asked to convert.
    fx.run("let g:convlog = []");
    fx.run(
        "set charconvert=writefile(map(readfile(v:fname_in),'toupper(v:val)'),v:fname_out)\
         +len(add(g:convlog,[v:charconvert_from,v:charconvert_to]))*0",
    );
    let mut seen = Seen::new(&fx);
    seen.note("converted", &read(&fx, "++enc=no-such-encoding", "f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding, g:convlog]");
    // A failing conversion falls back to reading the bytes as they are.
    fx.run("set charconvert=1");
    seen.note("failed", &read(&fx, "++enc=no-such-encoding", "f.txt"));
    seen.eval("[getline(1, '$'), &fileencoding]");
    restore(&fx);
    seen.verify("charconvert", CHARCONVERT);
}

#[test]
fn read_appends_after_the_cursor_with_its_own_options() {
    let sandbox = Sandbox::dir("readfile_read");
    sandbox.write("base.txt", b"one\ntwo\n");
    sandbox.write("l.txt", b"caf\xe9\n");
    sandbox.write("d.txt", b"x\r\ny\r\n");
    let fx = Probe::new(sandbox.editor());
    fx.run("set shortmess=");
    let mut seen = Seen::new(&fx);
    read(&fx, "", "base.txt");
    fx.run("1");
    seen.note("latin1", &fx.output("read ++enc=latin1 l.txt"));
    seen.note("dos", &fx.output("$read d.txt"));
    seen.eval("[getline(1, '$'), &fileencoding, &ff, line(\"'[\"), line(\"']\")]");
    restore(&fx);
    seen.verify("read", READ);
}

const UTF8_SPLIT: &str = r#"
edit => "\n\"f.txt\" 2 lines, 65546 bytes"
strlen(getline(1)) => 65538
getline(1)[65535:] => 'éz'
getline(2) => 'second'
&fileencoding => 'utf-8'
"#;
const UTF16_SPLIT: &str = r#"
edit => "\n\"f.txt\" [converted] 2 lines, 21851 bytes"
strchars(getline(1)) => 21845
char2nr(strcharpart(getline(1), 21844, 1)) => 128512
getline(2) => 'b'
&fileencoding => 'utf-16le'
odd => "\n\"odd.txt\" [converted][CONVERSION ERROR in line 2] 1 line, 3 bytes"
[getline(1, '$'), &readonly] => [['ab'], 1]
ucs-2 => "\n\"odd.txt\" [converted][CONVERSION ERROR in line 2] 1 line, 3 bytes"
[getline(1, '$'), &readonly] => [['ab'], 1]
ucs-4 => "\n\"odd.txt\" [noeol][converted][CONVERSION ERROR in line 1] 1 line, 5 bytes"
[getline(1, '$'), &readonly] => [['�����'], 1]
"#;
const ICONV_SPLIT: &str = r#"
edit => "\n\"f.txt\" [converted] 2 lines, 8201 bytes"
strchars(getline(1)) => 8192
char2nr(strcharpart(getline(1), 8191, 1)) => 12354
getline(2) => 'bあc'
&fileencoding => 'sjis'
 => "\n\"bad.txt\" [converted][CONVERSION ERROR in line 1] 1 line, 5 bytes"
[getline(1, '$'), &readonly] => [['x? y'], 1]
++bad=keep => "\n\"bad.txt\" [noeol][converted][CONVERSION ERROR in line 1] 2 lines, 6 bytes"
[getline(1, '$'), &readonly] => [['x� y', '�'], 1]
++bad=drop => "\n\"bad.txt\" [converted][CONVERSION ERROR in line 1] 1 line, 4 bytes"
[getline(1, '$'), &readonly] => [['x y'], 1]
++bad=X => "\n\"bad.txt\" [converted][CONVERSION ERROR in line 1] 1 line, 5 bytes"
[getline(1, '$'), &readonly] => [['xX y'], 1]
fencs => "\n\"bad.txt\" [noeol][converted] 2 lines, 8 bytes"
[getline(1, '$'), &fileencoding] => [['x y', ''], 'latin1']
"#;
const FENCS: &str = r#"
default => "\n\"f.txt\" [converted] 2 lines, 13 bytes"
[getline(1, '$'), &fileencoding] => [['café', 'naïve'], 'latin1']
utf-8 only => "\n\"f.txt\" [ILLEGAL BYTE in line 1] 2 lines, 11 bytes"
[getline(1, '$'), &fileencoding, &readonly] => [['caf?', 'na?ve'], '', 1]
latin1 => "\n\"hi.txt\" [converted] 1 line, 257 bytes"
[strchars(getline(1)), strlen(getline(1)), &fileencoding] => [128, 256, 'latin1']
char2nr(strcharpart(getline(1), 127, 1)) => 255
unknown => "\n\"f.txt\" [NOT converted][ILLEGAL BYTE in line 1] 2 lines, 11 bytes"
[getline(1, '$'), &fileencoding] => [['caf?', 'na?ve'], '']
empty fencs => "\n\"f.txt\" [ILLEGAL BYTE in line 1] 2 lines, 11 bytes"
[getline(1, '$'), &fileencoding] => [['caf?', 'na?ve'], '']
"#;
const BOM: &str = r#"
u8.txt => "\n\"u8.txt\" 1 line, 6 bytes"
[getline(1, '$'), &fileencoding, &bomb] => [['hi'], 'utf-8', 1]
u16.txt => "\n\"u16.txt\" [converted] 1 line, 5 bytes"
[getline(1, '$'), &fileencoding, &bomb] => [['hi'], 'utf-16le', 1]
u16be.txt => "\n\"u16be.txt\" [converted] 1 line, 5 bytes"
[getline(1, '$'), &fileencoding, &bomb] => [['hi'], 'utf-16', 1]
u32.txt => "\n\"u32.txt\" [converted] 1 line, 6 bytes"
[getline(1, '$'), &fileencoding, &bomb] => [['h'], 'ucs-4le', 1]
bin => "\n\"u8.txt\" 1 line, 6 bytes"
[getline(1, '$'), &bomb, &binary] => [['﻿hi'], 0, 1]
enc => "\n\"u8.txt\" 1 line, 6 bytes"
[getline(1, '$'), &bomb] => [['hi'], 1]
"#;
const EOL: &str = r#"
 nul.txt => "\n\"nul.txt\" [noeol] 3 lines, 8 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a^Jb', 'c^J', '^J'], 'unix', 0, 0, 0]
 crlf.txt => "\n\"crlf.txt\" [dos] 2 lines, 6 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a', 'b'], 'dos', 1, 0, 0]
++bin crlf.txt => "\n\"crlf.txt\" 2 lines, 6 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a^M', 'b^M'], 'unix', 1, 0, 1]
 mixed.txt => "\n\"mixed.txt\" 3 lines, 8 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a^M', 'b', 'c^M'], 'unix', 1, 0, 0]
++ff=dos mixed.txt => "\n\"mixed.txt\" [CR missing][dos] 3 lines, 8 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a', 'b', 'c'], 'dos', 1, 0, 0]
++ff=mac mac.txt => "\n\"mac.txt\" [mac] 2 lines, 8 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a', 'b^Mc^Jd'], 'mac', 1, 0, 0]
 noeol.txt => "\n\"noeol.txt\" [noeol] 2 lines, 3 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a', 'b'], 'unix', 0, 0, 0]
 ctrlz.txt => "\n\"ctrlz.txt\" [dos] 2 lines, 7 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a', 'b'], 'dos', 1, 1, 0]
++ff=unix ctrlz.txt => "\n\"ctrlz.txt\" [noeol] 3 lines, 7 bytes"
[getline(1, '$'), &ff, &eol, &eof, &bin] => [['a^M', 'b^M', '^Z'], 'unix', 0, 0, 0]
unix,mac => "\n\"mac.txt\" [mac] 2 lines, 8 bytes"
[getline(1, '$'), &ff] => [['a', 'b^Mc^Jd'], 'mac']
"#;
const LONG: &str = r#"
[line('$'), strlen(getline(1)), getline(2)] => [2, 1500000, 'end']
[line('$'), strchars(getline(1)), strlen(getline(1)), getline(2)] => [2, 200000, 400000, 'end']
"#;
const CHARCONVERT: &str = r#"
converted => "\n\"f.txt\" [converted] 2 lines, 8 bytes"
[getline(1, '$'), &fileencoding, g:convlog] => [['ONE', 'TWO'], 'no-such-encoding', [['no-such-encoding', 'utf-8']]]
failed => "\nConversion with 'charconvert' failed\r\n\"f.txt\" [NOT converted] 2 lines, 8 bytes"
[getline(1, '$'), &fileencoding] => [['one', 'two'], '']
"#;
const READ: &str = r#"
latin1 => "\n\"l.txt\" [converted] 1 line, 6 bytes"
dos => "\n\"d.txt\" [dos] 2 lines, 6 bytes"
[getline(1, '$'), &fileencoding, &ff, line("'["), line("']")] => [['one', 'café', 'two', 'x', 'y'], 'utf-8', 'unix', 4, 5]
"#;
