//! Digraphs: CTRL-K input, the `:digraph[s]` ex command, the
//! `digraph_get()`/`digraph_set()` family of eval functions, and — sharing
//! this file like upstream `digraph.c` — the 'keymap' option's language
//! mapping loader (`:loadkeymap`).
//!
//! The default table lives in [`tables`]; user-defined digraphs are an
//! ordered list that shadows it (`:digraph`, `digraph_set()`). Lookups scan
//! the user list first, in insertion order, then the default table.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod tables;

use crate::buffer::KEYMAP_INIT;
use crate::charset::char2cells;
use crate::cstr;
use crate::drawscreen::status_redraw_curbuf;
use crate::eval::eval_to_string;
use crate::eval::typval::{
    NumBuf, list_items, list_iter, list_len, tv_check_for_opt_bool_arg, tv_get_bool, tv_list_alloc,
    tv_list_alloc_ret,
};
use crate::ex_docmd::{do_cmdline_cmd, exarg_getline, reads_sourced_lines};
use crate::ex_getln::putcmdline;
use crate::ex_getln::state::cmdline_star;
use crate::getchar::plain_vgetc;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::guard::{Keys, Suppress};
use crate::highlight_group::{HLF_8, HLF_CM};
use crate::keycodes::SpecialKey;
use crate::mapping::{MAPTYPE_MAP, MAPTYPE_UNMAP, do_map};
use crate::mbyte::{char_at, char_len, encode_char, utf_iscomposing_first};
use crate::memory::ThinCString;
use crate::message::state::msg_col;
use crate::message::{msg_advance, msg_display, msg_ext_set_kind, msg_putchar};
use crate::message_fmt::emsg_text;
use crate::normal::add_to_showcmd;
use crate::option::SavedCpo;
use crate::option::vars::{p_dg, p_enc};
use crate::optionstr::OptStringRef;
use crate::os::cshim::gettext;
use crate::os::input::fast_breakcheck;
use crate::runtime::{RuntimeOpts, source_runtime};
use crate::state::MODE_LANGMAP;
use crate::tr;
use crate::types::Key;
use crate::types::{
    BoolVarValue, EvalFuncData, ExArg, Failed, KeymapEntry, List, OptError, OptInt, TypVal,
    VAR_LIST, int16_t,
};
use crate::ui::state::Columns;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use crate::winlayer::graph::switch_to;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

const K_BOOL_VAR_FALSE: BoolVarValue = 0;
const K_BOOL_VAR_TRUE: BoolVarValue = 1;

/// Why a digraph could not be defined, and the message that says so --
/// upstream's texts. Reported once, by `:digraphs` or the builtin.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DigraphError {
    /// What was given for the pair, decoded lossily as upstream's `%s`
    /// showed it -- `[NULL]` for a value that is not a string.
    #[error("{}", tr!("E1214: Digraph must be just two characters: {}", String::from_utf8_lossy(.0.as_bytes())))]
    NotTwoChars(ThinCString),
    #[error("{}", tr!("E104: Escape not allowed in digraph"))]
    Escape,
    #[error("{}", tr!("E39: Number expected"))]
    NumberExpected,
    /// What was given for the result, which is not one character.
    #[error("{}", tr!("E1215: Digraph must be one character: {}", String::from_utf8_lossy(.0.as_bytes())))]
    NotOneChar(ThinCString),
    #[error("{}", tr!("E1216: digraph_setlist() argument must be a list of lists with two items"))]
    NotAListOfPairs,
    /// A string argument that did not read as one: its message is out.
    #[error(transparent)]
    Reported(#[from] Failed),
}

// Two words, so a `Result` carrying one is no wider than it has to be.
const _: () = assert!(size_of::<DigraphError>() <= 16);

impl DigraphError {
    /// Puts the message out, unless it is already out.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Reported(_)) {
            emsg_text(self.to_string());
        }
    }
}

/// A digraph mapping: two input characters and the key they produce.
///
/// The input characters are single bytes, as in Vim: multibyte characters
/// given to `digraph_set()` are truncated when stored.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Digraph {
    pub char1: u8,
    pub char2: u8,
    pub result: Key,
}

impl Digraph {
    /// Whether this digraph is typed as `char1` `char2`.
    fn is_typed_as(&self, char1: Key, char2: Key) -> bool {
        Key::byte(self.char1) == char1 && Key::byte(self.char2) == char2
    }
}

/// User-defined digraphs, in insertion order. Shadows [`tables::DEFAULT_DIGRAPHS`].
static USER_DIGRAPHS: GlobalCell<Vec<Digraph>> = GlobalCell::new(Vec::new());

/// The default digraph table, for table-integrity tests.
pub fn default_digraphs() -> &'static [Digraph] {
    &tables::DEFAULT_DIGRAPHS
}

/// Look up `char1`/`char2` in the default table only.
pub fn lookup_default(char1: u8, char2: u8) -> Option<Key> {
    tables::DEFAULT_DIGRAPHS
        .iter()
        .find(|d| d.char1 == char1 && d.char2 == char2)
        .map(|d| d.result)
}

/// Look up `char1`/`char2` in the user list, then the default table. When
/// nothing matches, the result is `char2` itself — with the eighth bit set
/// if `meta_char` is given and `char1` is a space.
fn get_exact_digraph(char1: Key, char2: Key, meta_char: bool) -> Key {
    if char1.is_special() || char2.is_special() {
        return char2;
    }
    let user = USER_DIGRAPHS.with(|user| {
        user.iter()
            .find(|d| d.is_typed_as(char1, char2))
            .map(|d| d.result)
    });
    let found = user
        .filter(|result| !result.is_nul())
        .or_else(|| {
            let (char1, char2) = (char1.as_byte()?, char2.as_byte()?);
            lookup_default(char1, char2)
        })
        .filter(|result| !result.is_nul());
    match found {
        Some(result) => result,
        None if char1.is(' ') && meta_char => Key::from_code(char2.code() | 0x80),
        None => char2,
    }
}

/// Get the digraph for `char1` and `char2`, trying the reversed pair too.
/// Falls back to `char2` when no digraph is defined.
pub fn digraph_get(char1: Key, char2: Key, meta_char: bool) -> Key {
    let mut retval = get_exact_digraph(char1, char2, meta_char);
    if retval == char2 && char1 != char2 {
        retval = get_exact_digraph(char2, char1, meta_char);
        if retval == char1 {
            return char2;
        }
    }
    retval
}

/// What 'digraph' is in the middle of: the character typed before the last
/// backspace, and the last character typed. `None` is upstream's `-1`.
struct Typed {
    backspaced: Option<Key>,
    last: Option<Key>,
}

static TYPED: GlobalCell<Typed> = GlobalCell::new(Typed {
    backspaced: Some(Key::NUL),
    last: Some(Key::NUL),
});

/// Forget a half-typed 'digraph' char1-BS-char2 sequence, as entering Insert
/// mode does.
pub(crate) fn reset_digraph() {
    TYPED.set(Typed {
        backspaced: None,
        last: None,
    });
}

/// Handle typed characters for the 'digraph' option: entering
/// char1-BS-char2 produces the digraph.
pub fn do_digraph(c: Key) -> Key {
    let mut c = c;
    if p_dg() {
        let (backspaced, last) = TYPED.with(|typed| (typed.backspaced, typed.last));
        // Upstream tests `>= 0`: a special key never starts one.
        if let Some(char1) = backspaced.filter(|k| !k.is_special()) {
            c = digraph_get(char1, c, false);
        }
        let backspaced = last.filter(|k| !k.is_special() && (c == SpecialKey::Bs || c == Key::BS));
        TYPED.set(Typed {
            backspaced,
            last: Some(c),
        });
    } else {
        TYPED.with_mut(|typed| typed.last = Some(c));
    }
    c
}

/// Find a digraph that produces `val`, preferring user-defined ones.
/// Returns the two characters NUL-terminated, ready for `%s`.
pub fn get_digraph_for_char(val: Key) -> Option<[u8; 3]> {
    USER_DIGRAPHS
        .with(|user| user.iter().find(|d| d.result == val).copied())
        .or_else(|| {
            tables::DEFAULT_DIGRAPHS
                .iter()
                .find(|d| d.result == val)
                .copied()
        })
        .map(|d| [d.char1, d.char2, 0])
}

/// Read the two characters of a digraph from the user (after CTRL-K, or
/// CTRL-R CTRL-K on the cmdline) and return the resulting character.
/// Returns NUL when ESC cancels the sequence.
pub fn get_digraph(cmdline: bool) -> Key {
    let raw_key = Keys::unmapped_with_codes();
    let c = plain_vgetc();
    drop(raw_key);
    if c == Key::ESC {
        // Special keys or ESC cancel CTRL-K.
        return Key::NUL;
    }
    if c.is_special() {
        return c;
    }
    if cmdline {
        if let Some(byte) = c.as_byte().filter(|&b| b < 128)
            && char2cells(c.code()) == 1
            && cmdline_star.get() == 0
        {
            putcmdline(byte.cast_signed(), true);
        }
    } else {
        add_to_showcmd(c);
    }
    let raw_key = Keys::unmapped_with_codes();
    let cc = plain_vgetc();
    drop(raw_key);
    if cc != Key::ESC {
        return digraph_get(c, cc, true);
    }
    Key::NUL
}

/// Add (or overwrite) a user digraph.
fn register_digraph(char1: Key, char2: Key, n: Key) {
    USER_DIGRAPHS.with_mut(|user| {
        for d in user.iter_mut() {
            if d.is_typed_as(char1, char2) {
                d.result = n;
                return;
            }
        }
        // Upstream's `(char)`: a multibyte character keeps its low byte.
        user.push(Digraph {
            char1: char1.code().to_le_bytes()[0],
            char2: char2.code().to_le_bytes()[0],
            result: n,
        });
    });
}

/// Check the characters of a prospective digraph: there must be exactly
/// two, and ESC is not allowed.
fn check_digraph_chars(char1: Key, char2: Key) -> Result<(), DigraphError> {
    if char2.is_nul() {
        let mut msg = [0u8; 7];
        let len = encode_char(char1.code(), &mut msg);
        return Err(DigraphError::NotTwoChars(msg[..len].into()));
    }
    if char1 == Key::ESC || char2 == Key::ESC {
        return Err(DigraphError::Escape);
    }
    Ok(())
}

/// Skip over ' ' and '\t', like `skipwhite`.
fn skip_white(s: &[u8]) -> &[u8] {
    let n = s.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
    &s[n..]
}

/// Split at the first ' ' or '\t', like `skiptowhite`.
fn split_at_white(s: &[u8]) -> (&[u8], &[u8]) {
    let n = s
        .iter()
        .position(|&b| b == b' ' || b == b'\t')
        .unwrap_or(s.len());
    s.split_at(n)
}

/// Parse a decimal number like strict `getdigits_int`, which saturates at
/// `INT_MAX` rather than failing: the digits come straight from a
/// `:digraph a: 4294967296` command line, so an out-of-range value is
/// ordinary user input and must not end the process.
fn parse_digits(s: &[u8]) -> (c_int, &[u8]) {
    let n = s.iter().take_while(|b| b.is_ascii_digit()).count();
    let (digits, rest) = s.split_at(n);
    let mut value: c_int = 0;
    for &d in digits {
        value = value
            .saturating_mul(10)
            .saturating_add(c_int::from(d - b'0'));
    }
    (value, rest)
}

/// Add digraphs from a `:digraph {char1}{char2} {number} ...` argument.
pub fn putdigraph(s: &[u8]) {
    if let Err(refused) = put_digraphs(s) {
        refused.report();
    }
}

/// Every `{char1}{char2} {number}` of `:digraphs`' argument, defined in
/// turn up to the first that is refused.
fn put_digraphs(mut s: &[u8]) -> Result<(), DigraphError> {
    loop {
        s = skip_white(s);
        if s.is_empty() {
            return Ok(());
        }
        let char1 = s[0];
        let char2 = s.get(1).copied().unwrap_or(0);
        s = &s[s.len().min(2)..];
        check_digraph_chars(Key::byte(char1), Key::byte(char2))?;
        s = skip_white(s);
        if !s.first().is_some_and(u8::is_ascii_digit) {
            return Err(DigraphError::NumberExpected);
        }
        let (n, rest) = parse_digits(s);
        s = rest;
        register_digraph(Key::byte(char1), Key::byte(char2), Key::from_code(n));
    }
}

/// NUL-terminate `bytes` and print them with `msg_display`.
fn outtrans(bytes: &[u8], hl_id: c_int) {
    let mut buf = [0u8; 32];
    buf[..bytes.len()].copy_from_slice(bytes);
    // `bytes` is always shorter than `buf`, so it is terminated.
    msg_display(cstr::in_bytes(&buf), hl_id, false);
}

fn digraph_header(name: &'static [u8]) {
    if msg_col.get() > 0 {
        newline();
    }
    let header = gettext(cstr::in_bytes(name));
    msg_display(header, HLF_CM, false);
    newline();
}

/// `msg_putchar('\n')`: end the current message line.
fn newline() {
    msg_putchar(c_int::from(b'\n'));
}

/// Print one digraph. With `previous`, print a section header when this
/// digraph starts a new Unicode block (`:digraphs!`).
fn printdigraph(dp: &Digraph, previous: Option<&mut c_int>) {
    const LIST_WIDTH: c_int = 13;
    let result = dp.result.code();
    if result == 0 {
        return;
    }
    if let Some(previous) = previous {
        for (i, header) in tables::BLOCK_HEADERS.iter().enumerate() {
            let next_start = tables::BLOCK_HEADERS
                .get(i + 1)
                .map_or(tables::BLOCK_END, |h| h.start);
            if *previous < header.start && result >= header.start && result < next_start {
                digraph_header(header.name);
                break;
            }
        }
        *previous = result;
    }
    if msg_col.get() > Columns.get() - LIST_WIDTH {
        newline();
    }
    if msg_col.get() % LIST_WIDTH != 0 {
        msg_advance((msg_col.get() / LIST_WIDTH + 1) * LIST_WIDTH);
    }
    outtrans(&[dp.char1, dp.char2, b' '], 0);
    let mut buf = [0u8; 12];
    let mut len = 0;
    // buf has room for the longest UTF-8 sequence plus the leading space.
    if utf_iscomposing_first(result) {
        buf[0] = b' ';
        len = 1;
    }
    // `buf` has room for the longest sequence after the space.
    len += encode_char(result, &mut buf[len..]);
    outtrans(&buf[..len], HLF_8);
    let mut num = Vec::with_capacity(8);
    if char2cells(result) == 1 {
        num.push(b' ');
    }
    num.extend_from_slice(format!(" {result:3}").as_bytes());
    outtrans(&num, 0);
}

/// `:digraphs[!]` — list the active digraphs, with `use_headers` grouping
/// them under Unicode block headers.
pub fn listdigraphs(use_headers: bool) {
    // SAFETY: static string argument, message state only.
    msg_ext_set_kind(c"list_cmd");
    newline();
    let mut previous: c_int = 0;
    for dp in tables::DEFAULT_DIGRAPHS.iter() {
        if got_int.get() {
            break;
        }
        // getexactdigraph() so that user-defined digraphs override the
        // default; the entry is skipped if the user removed it.
        let result = get_exact_digraph(Key::byte(dp.char1), Key::byte(dp.char2), false);
        if !result.is_nul() && result != Key::byte(dp.char2) {
            let tmp = Digraph { result, ..*dp };
            let previous = use_headers.then_some(&mut previous);
            printdigraph(&tmp, previous);
        }
        // SAFETY: may set got_int; no digraph state is borrowed.
        fast_breakcheck();
    }
    let users = USER_DIGRAPHS.with(|user| user.clone());
    for dp in &users {
        if got_int.get() {
            break;
        }
        if previous >= 0 && use_headers {
            digraph_header(b"Custom\0");
        }
        previous = -1;
        printdigraph(dp, None);
        // SAFETY: may set got_int; no digraph state is borrowed.
        fast_breakcheck();
    }
}

/// Append `[chars, result]` to the `digraph_getlist()` result.
fn getlist_append_pair(dp: &Digraph, list: &mut List) {
    let chars = [dp.char1, dp.char2, 0];
    let mut buf = [0u8; 7];
    encode_char(dp.result.code(), &mut buf);
    let pair = tv_list_alloc(2);
    pair.edit().push_str(Some(cstr::in_bytes(&chars)));
    pair.edit().push_str(Some(cstr::in_bytes(&buf)));
    list.push_list(Some(pair));
}

/// Build the `digraph_getlist()` result: user digraphs, plus the effective
/// defaults when `list_all` is given.
fn digraph_getlist_common(list_all: bool, result: &mut TypVal) {
    let user_len = USER_DIGRAPHS.with(|user| user.len());
    let capacity =
        isize::try_from(tables::DEFAULT_DIGRAPHS.len() + user_len).expect("a short table");
    // SAFETY: `result` is a valid return slot, so the list it is given owns
    // itself from here on.
    let list = tv_list_alloc_ret(result, capacity);
    if list_all {
        for dp in tables::DEFAULT_DIGRAPHS.iter() {
            if got_int.get() {
                break;
            }
            let result = get_exact_digraph(Key::byte(dp.char1), Key::byte(dp.char2), false);
            if !result.is_nul() && result != Key::byte(dp.char2) {
                getlist_append_pair(&Digraph { result, ..*dp }, list);
            }
        }
    }
    let users = USER_DIGRAPHS.with(|user| user.clone());
    for dp in &users {
        if got_int.get() {
            break;
        }
        getlist_append_pair(dp, list);
    }
}

/// Decode the leading character of `s`, and answer it with what follows,
/// stepping over the base character only. A NUL first byte decodes to 0 and
/// consumes nothing, as the C did.
fn next_char(s: &[u8]) -> (Key, &[u8]) {
    if s.first().is_none_or(|&b| b == 0) {
        return (Key::NUL, s);
    }
    (Key::from_code(char_at(s)), &s[char_len(s)..])
}

/// The string value of `arg`, `None` when the typval is not one (which is
/// where `tv_get_string_buf_chk` reports its own error).
fn tv_string<'a>(arg: &'a TypVal, buf: &'a mut NumBuf) -> Option<&'a [u8]> {
    buf.string_chk(arg).ok().map(CStr::to_bytes)
}

/// The two digraph characters of `chars`: E1214 on anything but exactly
/// two characters, `None` included -- a non-string argument reads as the
/// string "[NULL]", which is what the C `%s` format path printed.
fn digraph_chars(chars: Option<&[u8]>) -> Result<(Key, Key), DigraphError> {
    if let Some(bytes) = chars.filter(|b| !b.is_empty()) {
        let (char1, rest) = next_char(bytes);
        if !rest.is_empty() {
            let (char2, rest) = next_char(rest);
            if rest.is_empty() {
                check_digraph_chars(char1, char2)?;
                return Ok((char1, char2));
            }
        }
    }
    Err(DigraphError::NotTwoChars(chars.unwrap_or(b"[NULL]").into()))
}

/// Shared body of `digraph_set()` and `digraph_setlist()`. The digraph
/// argument is only read once the characters check out, so a bad pair
/// reports its own error and nothing else.
fn digraph_set_common(argchars: &TypVal, argdigraph: &TypVal) -> Result<(), DigraphError> {
    let mut buf_chars = NumBuf::new();
    let chars = tv_string(argchars, &mut buf_chars);
    let (char1, char2) = digraph_chars(chars)?;
    let mut buf_digraph = NumBuf::new();
    let digraph = tv_string(argdigraph, &mut buf_digraph).ok_or(Failed)?;
    let (n, rest) = next_char(digraph);
    if !rest.is_empty() {
        return Err(DigraphError::NotOneChar(digraph.into()));
    }
    register_digraph(char1, char2, n);
    Ok(())
}

/// A builtin's `v:true`, or its refusal reported and `v:false`.
fn set_or_report(set: Result<(), DigraphError>) -> bool {
    set.inspect_err(DigraphError::report).is_ok()
}

/// Store a `v:true`/`v:false` result.
fn set_bool_ret(result: &mut TypVal, value: bool) {
    result.write_boolean(if value {
        K_BOOL_VAR_TRUE
    } else {
        K_BOOL_VAR_FALSE
    });
}

/// `digraph_get()`.
pub fn f_digraph_get(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    let Some(bytes) = numbuf.bytes_chk(&args[0]) else {
        return;
    };
    if bytes.len() != 2 {
        DigraphError::NotTwoChars(bytes.into()).report();
        return;
    }
    // The chars go through `char` in C, hence the sign extension.
    let sign_extended = |b: u8| Key::from_code(c_int::from(b.cast_signed()));
    let code = digraph_get(sign_extended(bytes[0]), sign_extended(bytes[1]), false);
    let mut buf = [0u8; 7];
    let len = encode_char(code.code(), &mut buf);
    result.write_string(Some(ThinCString::from_bytes(&buf[..len])));
}

/// `digraph_getlist()`.
pub fn f_digraph_getlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if tv_check_for_opt_bool_arg(args, 0).is_err() {
        return;
    }
    let list_all = !args.is_empty() && tv_get_bool(&args[0]) != 0;
    digraph_getlist_common(list_all, result);
}

/// `digraph_set()`.
pub fn f_digraph_set(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let set = digraph_set_common(&args[0], &args[1]);
    set_bool_ret(result, set_or_report(set));
}

/// `digraph_setlist()`.
pub fn f_digraph_setlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let set = digraph_setlist_common(&args[0]);
    set_bool_ret(result, set_or_report(set));
}

/// Body of `digraph_setlist()`: the argument must be a list of two-item
/// `[chars, digraph]` lists. Stops at the first bad entry, keeping the
/// digraphs registered before it.
fn digraph_setlist_common(arg: &TypVal) -> Result<(), DigraphError> {
    if arg.v_type() != VAR_LIST {
        return Err(DigraphError::NotAListOfPairs);
    }
    let Some(pl) = arg.list_ref() else {
        return Ok(());
    };
    // `digraph_set_common` does not touch the list it is reading from.
    for pli in list_iter(Some(pl)) {
        if pli.li_tv.list_ref().is_none() || list_len(pli.li_tv.list_ref()) != 2 {
            return Err(DigraphError::NotAListOfPairs);
        }
        // SAFETY: a live list of exactly two items.
        let pair = list_items(pli.li_tv.list_ref());
        digraph_set_common(&pair[0].li_tv, &pair[1].li_tv)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 'keymap' option: language mappings loaded from a keymap file.

const KEYMAP_LOADED: int16_t = 2;
const B_IMODE_LMAP: OptInt = 1;
/// Maximum length of `from` plus `to` in one keymap entry.
const KMAP_LLEN: usize = 200;

/// Source the keymap file for the current buffer's 'keymap' (or unload
/// language mappings when it is empty). Answers an error message.
pub fn keymap_init() -> Result<(), OptError> {
    let mut buf = Buf::current();
    buf.b_kmap_state &= !KEYMAP_INIT;
    let keymap = buf.b_p_keymap.bytes().to_vec();
    if keymap.is_empty() {
        // Stop any active keymap and clear the b:keymap_name variable.
        keymap_unload();
        // SAFETY: a static command string, run like any other ex command.
        let _ = do_cmdline_cmd(c"unlet! b:keymap_name");
        return Ok(());
    }
    // Source the keymap file, first for this encoding and then without it.
    // The name is snapshotted above because the script can set 'keymap'.
    let enc = p_enc(|value| value.to_bytes().to_vec());
    if source_keymap_file(&keymap, Some(&enc)) || source_keymap_file(&keymap, None) {
        return Ok(());
    }
    Err((c"E544: Keymap file not found").into())
}

/// Source `keymap/{name}_{enc}.vim` from the runtime path — or
/// `keymap/{name}.vim` without an encoding — and report whether it was found.
fn source_keymap_file(keymap: &[u8], enc: Option<&[u8]>) -> bool {
    let mut name = Vec::with_capacity(keymap.len() + enc.map_or(0, |e| e.len() + 1) + 12);
    name.extend_from_slice(b"keymap/");
    name.extend_from_slice(keymap);
    if let Some(enc) = enc {
        name.push(b'_');
        name.extend_from_slice(enc);
    }
    name.extend_from_slice(b".vim\0");
    source_runtime(cstr::in_bytes(&name), RuntimeOpts::NONE).is_ok()
}

/// `:loadkeymap` — read language mappings from the file being sourced.
pub fn ex_loadkeymap(excmd: &mut ExArg) {
    if !reads_sourced_lines(excmd) {
        crate::semsg!("E105: Using :loadkeymap not in a sourced file");
        return;
    }
    // Stop any active keymap and load the new entries.
    keymap_unload();
    // `keymap_unload` left the keymap garray cleared.
    let mut buf = Buf::current();
    buf.b_kmap_state = 0;
    buf.b_kmap_ga.clear();
    // Set 'cpoptions' to "C" to avoid line continuation.
    let _cpo = SavedCpo::held(c"C");
    // SAFETY: caller contract; the line getter was just checked to be the
    // sourcing one, and `buf`'s entry list was just emptied.
    read_keymap_entries(excmd, buf);
    apply_keymap_entries(buf);
    buf.b_kmap_state |= KEYMAP_LOADED;
    status_redraw_curbuf();
}

/// Read `{from} {to}` pairs from the file being sourced into `buffer`'s keymap
/// entries, until the line getter runs out. Blank lines and `"` comments are
/// skipped; an over-long or half-empty entry is dropped, and an empty `to`
/// reports E791.
fn read_keymap_entries(excmd: &mut ExArg, mut buffer: Buf) {
    while let Some(line) = exarg_getline(excmd, Some(0), 0, true) {
        let s = skip_white(&line);
        if !s.is_empty() && s[0] != b'"' {
            let (from, rest) = split_at_white(s);
            let (to, _) = split_at_white(skip_white(rest));
            if from.len() + to.len() >= KMAP_LLEN || from.is_empty() || to.is_empty() {
                if to.is_empty() {
                    crate::semsg!("E791: Empty keymap entry");
                }
            } else {
                buffer.b_kmap_ga.push(KeymapEntry {
                    from: from.to_vec(),
                    to: to.to_vec(),
                });
            }
        }
    }
}

/// Make every entry of `buffer`'s keymap a buffer-local language mapping.
fn apply_keymap_entries(buffer: Buf) {
    // SAFETY: the caller's buffer. The commands are built before any of them
    // runs, so `do_map` cannot be reading the list it is driven by.
    let cmds: Vec<Vec<u8>> = buffer
        .b_kmap_ga
        .iter()
        .map(|entry| keymap_map_cmd(&entry.from, Some(&entry.to)))
        .collect();
    for cmd in cmds {
        do_map(MAPTYPE_MAP, &cmd, MODE_LANGMAP, false);
    }
}

/// The `:lmap`/`:lunmap` argument for one keymap entry: `<buffer> {from}`,
/// plus ` {to}` when mapping.
fn keymap_map_cmd(from: &[u8], to: Option<&[u8]>) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(KMAP_LLEN + 11);
    cmd.extend_from_slice(b"<buffer> ");
    cmd.extend_from_slice(from);
    if let Some(to) = to {
        cmd.push(b' ');
        cmd.extend_from_slice(to);
    }
    cmd
}

/// Stop using 'keymap': remove the language mappings and free the entries.
fn keymap_unload() {
    let mut buf = Buf::current();
    if buf.b_kmap_state & KEYMAP_LOADED == 0 {
        return;
    }
    // Set 'cpoptions' to "C" to avoid line continuation.
    let _cpo = SavedCpo::held(c"C");
    // The commands are built before any of them runs, so `do_map` cannot be
    // reading the list it is driven by.
    let cmds: Vec<Vec<u8>> = buf
        .b_kmap_ga
        .iter()
        .map(|entry| keymap_map_cmd(&entry.from, None))
        .collect();
    for cmd in cmds {
        do_map(MAPTYPE_UNMAP, &cmd, MODE_LANGMAP, false);
    }
    // The entries own their two strings.
    buf.b_kmap_ga = Vec::new();
    buf.b_kmap_state &= !(KEYMAP_LOADED);
    status_redraw_curbuf();
}

/// The keymap name to show in the status line ('statusline' `%k`/`%K` and
/// the mode message): `b:keymap_name`, the 'keymap' value, or "lang".
/// `None` unless language mappings are active for `window`'s buffer.
pub fn keymap_str(window: Win) -> Option<CString> {
    // SAFETY: caller contract -- a live window whose buffer is valid.
    let buf = window.buffer();
    if buf.b_p_iminsert != B_IMODE_LMAP {
        return None;
    }
    // Evaluate b:keymap_name in wp's buffer.
    let saved = switch_to(window);
    let skipping = Suppress::emsg_skip();
    let s = eval_to_string(b"b:keymap_name", false, false);
    drop(skipping);
    saved.restore();
    let name = match s {
        Some(s) if !s.is_empty() => s.as_cstr().to_owned(),
        _ if buf.b_kmap_state & KEYMAP_LOADED != 0 => buf.b_p_keymap.cstr().to_owned(),
        _ => CString::new("lang").expect("no interior NUL"),
    };
    Some(name)
}
