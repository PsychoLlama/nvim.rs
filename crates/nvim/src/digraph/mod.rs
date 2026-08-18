//! Digraphs: CTRL-K input, the `:digraph[s]` ex command, the
//! `digraph_get()`/`digraph_set()` family of eval functions, and — sharing
//! this file like upstream `digraph.c` — the 'keymap' option's language
//! mapping loader (`:loadkeymap`).
//!
//! The default table lives in [`tables`]; user-defined digraphs are an
//! ordered list that shadows it (`:digraph`, `digraph_set()`). Lookups scan
//! the user list first, in insertion order, then the default table.

#![deny(unsafe_op_in_unsafe_fn)]

mod tables;

use crate::charset::char2cells;
use crate::drawscreen::status_redraw_curbuf;
use crate::eval::eval_to_string;
use crate::eval::typval::{
    tv_check_for_opt_bool_arg, tv_get_bool, tv_get_string_buf_chk, tv_get_string_chk,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_list, tv_list_append_string,
};
use crate::ex_docmd::{do_cmdline_cmd, getline_equal};
use crate::ex_getln::putcmdline;
use crate::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::getchar::plain_vgetc;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_8, HLF_CM};
use crate::keycodes::K_BS;
use crate::main::{
    Columns, allow_keys, cmdline_star, curbuf, curwin, emsg_skip, got_int, msg_col, no_mapping,
    p_cpo, p_dg, p_enc,
};
use crate::mapping::do_map;
use crate::mbyte::{mb_cptr2char_adv, utf_char2bytes, utf_iscomposing_first};
use crate::memory::{xfree, xmemdupz};
use crate::message::{msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar};
use crate::normal::add_to_showcmd;
use crate::os::cshim::gettext;
use crate::os::input::fast_breakcheck;
use crate::runtime::{getsourceline, source_runtime};
use crate::state::MODE_LANGMAP;
use crate::types::{
    BoolVarValue, EvalFuncData, OptInt, VAR_BOOL, VAR_LIST, VAR_STRING, VAR_UNKNOWN, buf_T,
    exarg_T, garray_T, int16_t, list_T, typval_T, varnumber_T, win_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;

const NUL: c_int = 0;
const ESC: c_int = 27;
const CTRL_H: c_int = 8;
const OK: c_int = 1;
const FAIL: c_int = 0;

const K_BOOL_VAR_FALSE: BoolVarValue = 0;
const K_BOOL_VAR_TRUE: BoolVarValue = 1;

const E_DIGRAPH_SETLIST: &str =
    "E1216: digraph_setlist() argument must be a list of lists with two items";

/// A digraph mapping: two input characters and the resulting codepoint.
///
/// The input characters are single bytes, as in Vim: multibyte characters
/// given to `digraph_set()` are truncated when stored.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Digraph {
    pub char1: u8,
    pub char2: u8,
    pub result: c_int,
}

/// User-defined digraphs, in insertion order. Shadows [`tables::DEFAULT_DIGRAPHS`].
static USER_DIGRAPHS: GlobalCell<Vec<Digraph>> = GlobalCell::new(Vec::new());

/// The default digraph table, for table-integrity tests.
pub fn default_digraphs() -> &'static [Digraph] {
    &tables::DEFAULT_DIGRAPHS
}

/// Look up `char1`/`char2` in the default table only.
pub fn lookup_default(char1: c_int, char2: c_int) -> Option<c_int> {
    tables::DEFAULT_DIGRAPHS
        .iter()
        .find(|d| d.char1 as c_int == char1 && d.char2 as c_int == char2)
        .map(|d| d.result)
}

/// Look up `char1`/`char2` in the user list, then the default table. When
/// nothing matches, the result is `char2` itself — with the eighth bit set
/// if `meta_char` is given and `char1` is a space.
fn get_exact_digraph(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    if char1 < 0 || char2 < 0 {
        return char2;
    }
    let mut retval = USER_DIGRAPHS.with(|user| {
        user.iter()
            .find(|d| d.char1 as c_int == char1 && d.char2 as c_int == char2)
            .map_or(0, |d| d.result)
    });
    if retval == 0 {
        retval = lookup_default(char1, char2).unwrap_or(0);
    }
    if retval == 0 {
        if char1 == ' ' as c_int && meta_char {
            return char2 | 0x80;
        }
        return char2;
    }
    retval
}

/// Get the digraph for `char1` and `char2`, trying the reversed pair too.
/// Falls back to `char2` when no digraph is defined.
pub fn digraph_get(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    let mut retval = get_exact_digraph(char1, char2, meta_char);
    if retval == char2 && char1 != char2 {
        retval = get_exact_digraph(char2, char1, meta_char);
        if retval == char1 {
            return char2;
        }
    }
    retval
}

/// Handle typed characters for the 'digraph' option: entering
/// char1-BS-char2 produces the digraph. Called with -1 to reset the state
/// (e.g. when entering insert mode).
pub fn do_digraph(c: c_int) -> c_int {
    /// Character typed before the last BS, or -1.
    static BACKSPACED: GlobalCell<c_int> = GlobalCell::new(0);
    /// Last typed character.
    static LASTCHAR: GlobalCell<c_int> = GlobalCell::new(0);

    let mut c = c;
    if c == -1 {
        BACKSPACED.set(-1);
    } else if p_dg.get() != 0 {
        if BACKSPACED.get() >= 0 {
            c = digraph_get(BACKSPACED.get(), c, false);
        }
        BACKSPACED.set(-1);
        if (c == K_BS || c == CTRL_H) && LASTCHAR.get() >= 0 {
            BACKSPACED.set(LASTCHAR.get());
        }
    }
    LASTCHAR.set(c);
    c
}

/// Find a digraph that produces codepoint `val`, preferring user-defined
/// ones. Returns the two characters NUL-terminated, ready for `%s`.
pub fn get_digraph_for_char(val: c_int) -> Option<[u8; 3]> {
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
pub fn get_digraph(cmdline: bool) -> c_int {
    no_mapping.set(no_mapping.get() + 1);
    allow_keys.set(allow_keys.get() + 1);
    // SAFETY: transpiled input machinery; no digraph state is borrowed.
    let c = unsafe { plain_vgetc() };
    no_mapping.set(no_mapping.get() - 1);
    allow_keys.set(allow_keys.get() - 1);
    if c == ESC {
        // Special keys or ESC cancel CTRL-K.
        return NUL;
    }
    if c < 0 {
        return c;
    }
    if cmdline {
        // SAFETY: transpiled display helpers, plain value arguments.
        if unsafe { char2cells(c) } == 1 && c < 128 && cmdline_star.get() == 0 {
            unsafe { putcmdline(c as c_char, true) };
        }
    } else {
        // SAFETY: same as above.
        add_to_showcmd(c);
    }
    no_mapping.set(no_mapping.get() + 1);
    allow_keys.set(allow_keys.get() + 1);
    // SAFETY: as for the first read.
    let cc = unsafe { plain_vgetc() };
    no_mapping.set(no_mapping.get() - 1);
    allow_keys.set(allow_keys.get() - 1);
    if cc != ESC {
        return digraph_get(c, cc, true);
    }
    NUL
}

/// Add (or overwrite) a user digraph.
fn register_digraph(char1: c_int, char2: c_int, n: c_int) {
    USER_DIGRAPHS.with_mut(|user| {
        for d in user.iter_mut() {
            if d.char1 as c_int == char1 && d.char2 as c_int == char2 {
                d.result = n;
                return;
            }
        }
        user.push(Digraph {
            char1: char1 as u8,
            char2: char2 as u8,
            result: n,
        });
    });
}

/// Check the characters of a prospective digraph: there must be exactly
/// two, and ESC is not allowed. Emits the error message itself.
fn check_digraph_chars_valid(char1: c_int, char2: c_int) -> bool {
    if char2 == 0 {
        let mut msg = [0u8; 7];
        // SAFETY: utf_char2bytes writes at most 6 bytes.
        let len = unsafe { utf_char2bytes(char1, msg.as_mut_ptr() as *mut c_char) };
        let msg = String::from_utf8_lossy(&msg[..len as usize]);
        crate::semsg!("E1214: Digraph must be just two characters: {msg}");
        return false;
    }
    if char1 == ESC || char2 == ESC {
        crate::semsg!("E104: Escape not allowed in digraph");
        return false;
    }
    true
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
    let mut value: i64 = 0;
    for &d in digits {
        value = value.saturating_mul(10).saturating_add((d - b'0') as i64);
    }
    (value.min(c_int::MAX as i64) as c_int, rest)
}

/// Add digraphs from a `:digraph {char1}{char2} {number} ...` argument.
pub fn putdigraph(mut s: &[u8]) {
    loop {
        s = skip_white(s);
        if s.is_empty() {
            return;
        }
        let char1 = s[0];
        let char2 = s.get(1).copied().unwrap_or(0);
        s = &s[s.len().min(2)..];
        if !check_digraph_chars_valid(char1 as c_int, char2 as c_int) {
            return;
        }
        s = skip_white(s);
        if !s.first().is_some_and(u8::is_ascii_digit) {
            crate::semsg!("E39: Number expected");
            return;
        }
        let (n, rest) = parse_digits(s);
        s = rest;
        register_digraph(char1 as c_int, char2 as c_int, n);
    }
}

/// NUL-terminate `bytes` and print them with `msg_outtrans`.
fn outtrans(bytes: &[u8], hl_id: c_int) {
    let mut buf = [0u8; 32];
    buf[..bytes.len()].copy_from_slice(bytes);
    // SAFETY: buf is NUL-terminated (bytes is always shorter than buf).
    unsafe { msg_outtrans(buf.as_ptr() as *const c_char, hl_id, false) };
}

fn digraph_header(name: &[u8]) {
    if msg_col.get() > 0 {
        newline();
    }
    // SAFETY: `name` is NUL-terminated and outlives the call; gettext hands
    // back a valid C string.
    unsafe { msg_outtrans(gettext(name.as_ptr() as *const c_char), HLF_CM, false) };
    newline();
}

/// `msg_putchar('\n')`: end the current message line.
fn newline() {
    // SAFETY: plain message output, no arguments.
    unsafe { msg_putchar('\n' as c_int) };
}

/// Print one digraph. With `previous`, print a section header when this
/// digraph starts a new Unicode block (`:digraphs!`).
fn printdigraph(dp: &Digraph, previous: Option<&mut c_int>) {
    const LIST_WIDTH: c_int = 13;
    if dp.result == 0 {
        return;
    }
    if let Some(previous) = previous {
        for (i, header) in tables::BLOCK_HEADERS.iter().enumerate() {
            let next_start = tables::BLOCK_HEADERS
                .get(i + 1)
                .map_or(tables::BLOCK_END, |h| h.start);
            if *previous < header.start && dp.result >= header.start && dp.result < next_start {
                digraph_header(header.name);
                break;
            }
        }
        *previous = dp.result;
    }
    if msg_col.get() > Columns.get() - LIST_WIDTH {
        newline();
    }
    if msg_col.get() % LIST_WIDTH != 0 {
        // SAFETY: plain message output with a plain value argument.
        unsafe { msg_advance((msg_col.get() / LIST_WIDTH + 1) * LIST_WIDTH) };
    }
    outtrans(&[dp.char1, dp.char2, b' '], 0);
    let mut buf = [0u8; 12];
    let mut len = 0;
    // buf has room for the longest UTF-8 sequence plus the leading space.
    if utf_iscomposing_first(dp.result) {
        buf[0] = b' ';
        len = 1;
    }
    // SAFETY: utf_char2bytes writes at most 6 bytes at offset <= 1.
    len += unsafe { utf_char2bytes(dp.result, buf[len..].as_mut_ptr() as *mut c_char) } as usize;
    outtrans(&buf[..len], HLF_8);
    let mut num = Vec::with_capacity(8);
    // SAFETY: value check only.
    if unsafe { char2cells(dp.result) } == 1 {
        num.push(b' ');
    }
    num.extend_from_slice(format!(" {:3}", dp.result).as_bytes());
    outtrans(&num, 0);
}

/// `:digraphs[!]` — list the active digraphs, with `use_headers` grouping
/// them under Unicode block headers.
pub fn listdigraphs(use_headers: bool) {
    // SAFETY: static string argument, message state only.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    newline();
    let mut previous: c_int = 0;
    for dp in tables::DEFAULT_DIGRAPHS.iter() {
        if got_int.get() {
            break;
        }
        // getexactdigraph() so that user-defined digraphs override the
        // default; the entry is skipped if the user removed it.
        let result = get_exact_digraph(dp.char1 as c_int, dp.char2 as c_int, false);
        if result != 0 && result != dp.char2 as c_int {
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
///
/// # Safety
///
/// `l` must be a valid list.
unsafe fn getlist_append_pair(dp: &Digraph, l: *mut list_T) {
    let chars = [dp.char1, dp.char2, 0];
    let mut buf = [0u8; 7];
    // SAFETY: `l` is a valid list; `utf_char2bytes` writes at most six bytes
    // into `buf`; both local buffers are NUL-terminated and outlive the
    // appends, which copy what they keep.
    unsafe {
        let l2 = tv_list_alloc(2);
        tv_list_append_list(l, l2);
        tv_list_append_string(l2, chars.as_ptr() as *const c_char, -1);
        utf_char2bytes(dp.result, buf.as_mut_ptr() as *mut c_char);
        tv_list_append_string(l2, buf.as_ptr() as *const c_char, -1);
    }
}

/// Build the `digraph_getlist()` result: user digraphs, plus the effective
/// defaults when `list_all` is given.
///
/// # Safety
///
/// `rettv` must be a valid return-value slot.
unsafe fn digraph_getlist_common(list_all: bool, rettv: *mut typval_T) {
    let user_len = USER_DIGRAPHS.with(|user| user.len());
    let capacity = (tables::DEFAULT_DIGRAPHS.len() + user_len) as isize;
    // SAFETY: `rettv` is a valid return slot, so the list it is given owns
    // itself from here on.
    let list = unsafe {
        tv_list_alloc_ret(rettv, capacity);
        (*rettv).vval.v_list
    };
    if list_all {
        for dp in tables::DEFAULT_DIGRAPHS.iter() {
            if got_int.get() {
                break;
            }
            let result = get_exact_digraph(dp.char1 as c_int, dp.char2 as c_int, false);
            if result != 0 && result != dp.char2 as c_int {
                // SAFETY: `list` is the list just allocated into `rettv`.
                unsafe { getlist_append_pair(&Digraph { result, ..*dp }, list) };
            }
        }
    }
    let users = USER_DIGRAPHS.with(|user| user.clone());
    for dp in &users {
        if got_int.get() {
            break;
        }
        // SAFETY: as above.
        unsafe { getlist_append_pair(dp, list) };
    }
}

/// Decode the leading character of `s`, and answer it with what follows —
/// `mb_cptr2char_adv` over a slice. A NUL first byte decodes to 0 and
/// consumes nothing, exactly as the C did.
fn next_char(s: &[u8]) -> (c_int, &[u8]) {
    let start = s.as_ptr() as *const c_char;
    let mut p = start;
    // SAFETY: `s` is a prefix of a NUL-terminated string, so the decoder
    // stops at the terminator at the latest and advances `p` by the length
    // of exactly the character it decoded.
    let c = unsafe { mb_cptr2char_adv(&raw mut p) };
    let used = (p as usize - start as usize).min(s.len());
    (c, &s[used..])
}

/// The string value of `arg`, `None` when the typval is not one (which is
/// where `tv_get_string_buf_chk` reports its own error).
///
/// # Safety
///
/// `arg` must be a valid typval, and `buf` — the scratch space a non-string
/// value is rendered into — must outlive the returned slice.
unsafe fn tv_string(arg: *const typval_T, buf: &mut [c_char; 65]) -> Option<&[u8]> {
    // SAFETY: caller contract; the result is null or a NUL-terminated string
    // owned by the typval or by `buf`, both of which outlive the borrow.
    unsafe {
        let s = tv_get_string_buf_chk(arg, buf.as_mut_ptr());
        (!s.is_null()).then(|| CStr::from_ptr(s).to_bytes())
    }
}

/// The two digraph characters of `chars`. Reports E1214 on anything but
/// exactly two characters, `None` included: a non-string argument reads as
/// the string "[NULL]", which is what the C `%s` format path printed.
fn digraph_chars(chars: Option<&[u8]>) -> Option<(c_int, c_int)> {
    if let Some(bytes) = chars.filter(|b| !b.is_empty()) {
        let (char1, rest) = next_char(bytes);
        if !rest.is_empty() {
            let (char2, rest) = next_char(rest);
            if rest.is_empty() {
                return check_digraph_chars_valid(char1, char2).then_some((char1, char2));
            }
        }
    }
    let text = chars.map_or("[NULL]".into(), String::from_utf8_lossy);
    crate::semsg!("E1214: Digraph must be just two characters: {text}");
    None
}

/// Shared body of `digraph_set()` and `digraph_setlist()`. The digraph
/// argument is only read once the characters check out, so a bad pair
/// reports its own error and nothing else.
///
/// # Safety
///
/// Both arguments must be valid typvals.
unsafe fn digraph_set_common(argchars: *const typval_T, argdigraph: *const typval_T) -> bool {
    let mut buf_chars = [0 as c_char; 65];
    // SAFETY: caller contract; `buf_chars` outlives the borrow.
    let chars = unsafe { tv_string(argchars, &mut buf_chars) };
    let Some((char1, char2)) = digraph_chars(chars) else {
        return false;
    };
    let mut buf_digraph = [0 as c_char; 65];
    // SAFETY: caller contract; `buf_digraph` outlives the borrow.
    let Some(digraph) = (unsafe { tv_string(argdigraph, &mut buf_digraph) }) else {
        return false;
    };
    let (n, rest) = next_char(digraph);
    if !rest.is_empty() {
        let text = String::from_utf8_lossy(digraph);
        crate::semsg!("E1215: Digraph must be one character: {text}");
        return false;
    }
    register_digraph(char1, char2, n);
    true
}

/// Store a `v:true`/`v:false` result.
///
/// # Safety
///
/// `rettv` must be a valid return-value slot.
unsafe fn set_bool_ret(rettv: *mut typval_T, value: bool) {
    // SAFETY: caller contract.
    unsafe {
        (*rettv).v_type = VAR_BOOL;
        (*rettv).vval.v_bool = if value {
            K_BOOL_VAR_TRUE
        } else {
            K_BOOL_VAR_FALSE
        };
    }
}

/// `digraph_get()`.
///
/// # Safety
///
/// Standard eval-function contract: `argvars` and `rettv` are valid.
pub unsafe fn f_digraph_get(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: caller contract; the result slot starts out empty.
    let digraphs = unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = core::ptr::null_mut();
        tv_get_string_chk(argvars)
    };
    if digraphs.is_null() {
        return;
    }
    // SAFETY: a non-null `tv_get_string_chk` result is a NUL-terminated
    // string owned by the typval, which outlives this call.
    let bytes = unsafe { CStr::from_ptr(digraphs).to_bytes() };
    if bytes.len() != 2 {
        crate::semsg!(
            "E1214: Digraph must be just two characters: {}",
            String::from_utf8_lossy(bytes)
        );
        return;
    }
    // The chars go through `char` in C, hence the sign extension.
    let code = digraph_get(bytes[0] as i8 as c_int, bytes[1] as i8 as c_int, false);
    let mut buf = [0u8; 7];
    // SAFETY: `utf_char2bytes` writes at most six bytes into `buf`, and
    // `xmemdupz` copies exactly the `len` it wrote.
    unsafe {
        let len = utf_char2bytes(code, buf.as_mut_ptr() as *mut c_char) as usize;
        (*rettv).vval.v_string = xmemdupz(buf.as_ptr() as *const c_void, len) as *mut c_char;
    }
}

/// `digraph_getlist()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_digraph_getlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: caller contract.
    if unsafe { tv_check_for_opt_bool_arg(argvars, 0) } == FAIL {
        return;
    }
    // SAFETY: caller contract; the optional argument was just type-checked.
    let list_all =
        unsafe { (*argvars).v_type != VAR_UNKNOWN && tv_get_bool(argvars) != 0 as varnumber_T };
    // SAFETY: caller contract.
    unsafe { digraph_getlist_common(list_all, rettv) };
}

/// `digraph_set()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_digraph_set(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: caller contract; `digraph_set()` takes two arguments.
    let set = unsafe { digraph_set_common(argvars, argvars.offset(1)) };
    // SAFETY: caller contract.
    unsafe { set_bool_ret(rettv, set) };
}

/// `digraph_setlist()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_digraph_setlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: caller contract.
    let set = unsafe { digraph_setlist_common(argvars) };
    // SAFETY: caller contract.
    unsafe { set_bool_ret(rettv, set) };
}

/// Body of `digraph_setlist()`: the argument must be a list of two-item
/// `[chars, digraph]` lists. Stops at the first bad entry, keeping the
/// digraphs registered before it.
///
/// # Safety
///
/// `arg` must be a valid typval.
unsafe fn digraph_setlist_common(arg: *const typval_T) -> bool {
    // SAFETY: caller contract; the list is only read once its type is known.
    let pl = unsafe {
        if (*arg).v_type != VAR_LIST {
            crate::semsg!("{E_DIGRAPH_SETLIST}");
            return false;
        }
        (*arg).vval.v_list
    };
    if pl.is_null() {
        return true;
    }
    // SAFETY: `pl` is a valid list; the walk only follows its links, and
    // `digraph_set_common` does not touch the list it is reading from.
    unsafe {
        let mut pli = (*pl).lv_first;
        while !pli.is_null() {
            if (*pli).li_tv.v_type != VAR_LIST {
                crate::semsg!("{E_DIGRAPH_SETLIST}");
                return false;
            }
            let l = (*pli).li_tv.vval.v_list;
            if l.is_null() || (*l).lv_len != 2 {
                crate::semsg!("{E_DIGRAPH_SETLIST}");
                return false;
            }
            let first = (*l).lv_first;
            if !digraph_set_common(&(*first).li_tv, &(*(*first).li_next).li_tv) {
                return false;
            }
            pli = (*pli).li_next;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// 'keymap' option: language mappings loaded from a keymap file.

const KEYMAP_INIT: c_int = 1;
const KEYMAP_LOADED: c_int = 2;
const B_IMODE_LMAP: OptInt = 1;
/// Maximum length of `from` plus `to` in one keymap entry.
const KMAP_LLEN: usize = 200;
const MAPTYPE_MAP: c_int = 0;
const MAPTYPE_UNMAP: c_int = 1;

/// One `:loadkeymap` entry: heap C strings owned by the buffer's
/// `b_kmap_ga` (freed by [`keymap_ga_clear`]).
struct kmap_T {
    from: *mut c_char,
    to: *mut c_char,
}

/// Source the keymap file for the current buffer's 'keymap' (or unload
/// language mappings when it is empty). Returns an error message or null.
pub fn keymap_init() -> *const c_char {
    let buf = curbuf.get();
    // SAFETY: curbuf is valid, and the 'keymap' value it holds is a
    // NUL-terminated option string.
    let keymap = unsafe {
        (*buf).b_kmap_state &= !(KEYMAP_INIT as int16_t);
        CStr::from_ptr((*buf).b_p_keymap).to_bytes().to_vec()
    };
    if keymap.is_empty() {
        // Stop any active keymap and clear the b:keymap_name variable.
        keymap_unload();
        // SAFETY: a static command string, run like any other ex command.
        unsafe { do_cmdline_cmd(c"unlet! b:keymap_name".as_ptr()) };
        return core::ptr::null();
    }
    // Source the keymap file, first for this encoding and then without it.
    // The name is snapshotted above because the script can set 'keymap'.
    // SAFETY: 'encoding' is a NUL-terminated option string.
    let enc = unsafe { CStr::from_ptr(p_enc.get()).to_bytes().to_vec() };
    if source_keymap_file(&keymap, Some(&enc)) || source_keymap_file(&keymap, None) {
        return core::ptr::null();
    }
    c"E544: Keymap file not found".as_ptr()
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
    // SAFETY: `name` is NUL-terminated and outlives the call, which only
    // reads it.
    unsafe { source_runtime(name.as_mut_ptr() as *mut c_char, 0) != FAIL }
}

/// `:loadkeymap` — read language mappings from the file being sourced.
///
/// # Safety
///
/// `eap` must be a valid command block (ex-command contract).
pub unsafe fn ex_loadkeymap(eap: *mut exarg_T) {
    // SAFETY: caller contract; the getter and its cookie are the sourcing
    // machinery's, and `getline_equal` only compares them.
    let sourced = unsafe {
        getline_equal(
            (*eap).ea_getline,
            (*eap).cookie,
            Some(getsourceline as unsafe fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
        )
    };
    if !sourced {
        crate::semsg!("E105: Using :loadkeymap not in a sourced file");
        return;
    }
    // Stop any active keymap and load the new entries.
    keymap_unload();
    let buf = curbuf.get();
    // SAFETY: curbuf is valid and `keymap_unload` left its keymap garray
    // cleared.
    unsafe {
        (*buf).b_kmap_state = 0;
        ga_init(
            &raw mut (*buf).b_kmap_ga,
            core::mem::size_of::<kmap_T>() as c_int,
            20,
        );
    }
    // Set 'cpoptions' to "C" to avoid line continuation.
    let save_cpo = p_cpo.get();
    p_cpo.set(c"C".as_ptr() as *mut c_char);
    // SAFETY: caller contract; the line getter was just checked to be the
    // sourcing one, and `buf`'s garray was just initialised for `kmap_T`.
    unsafe { read_keymap_entries(eap, buf) };
    // SAFETY: the entries just read own two NUL-terminated strings each.
    unsafe { apply_keymap_entries(buf) };
    p_cpo.set(save_cpo);
    // SAFETY: curbuf is still valid.
    unsafe {
        (*buf).b_kmap_state |= KEYMAP_LOADED as int16_t;
        status_redraw_curbuf();
    }
}

/// Read `{from} {to}` pairs from the file being sourced into `buf`'s keymap
/// garray, until the line getter runs out. Blank lines and `"` comments are
/// skipped; an over-long or half-empty entry is dropped, and an empty `to`
/// reports E791.
///
/// # Safety
///
/// `eap` must be a live command block whose line getter is the sourcing one,
/// and `buf` a valid buffer whose `b_kmap_ga` is initialised for `kmap_T`.
unsafe fn read_keymap_entries(eap: *mut exarg_T, buf: *mut buf_T) {
    loop {
        // SAFETY: caller contract; the getter answers an owned heap line or
        // null at end of file.
        let line =
            unsafe { (*eap).ea_getline.expect("non-null line getter")(0, (*eap).cookie, 0, true) };
        if line.is_null() {
            break;
        }
        // SAFETY: the line is NUL-terminated and owned until freed below.
        let bytes = unsafe { CStr::from_ptr(line).to_bytes() };
        let s = skip_white(bytes);
        if !s.is_empty() && s[0] != b'"' {
            let (from, rest) = split_at_white(s);
            let (to, _) = split_at_white(skip_white(rest));
            if from.len() + to.len() >= KMAP_LLEN || from.is_empty() || to.is_empty() {
                if to.is_empty() {
                    crate::semsg!("E791: Empty keymap entry");
                }
            } else {
                // SAFETY: the garray is sized for `kmap_T`, so the appended
                // slot is one; `xmemdupz` copies both slices out of `line`.
                unsafe {
                    let kp = ga_append_via_ptr(
                        &raw mut (*buf).b_kmap_ga,
                        core::mem::size_of::<kmap_T>(),
                    ) as *mut kmap_T;
                    (*kp).from =
                        xmemdupz(from.as_ptr() as *const c_void, from.len()) as *mut c_char;
                    (*kp).to = xmemdupz(to.as_ptr() as *const c_void, to.len()) as *mut c_char;
                }
            }
        }
        // SAFETY: the line is ours to free, and nothing borrows it now.
        unsafe { xfree(line as *mut c_void) };
    }
}

/// Make every entry of `buf`'s keymap garray a buffer-local language mapping.
///
/// # Safety
///
/// `buf` must be a valid buffer whose `b_kmap_ga` holds live `kmap_T`s.
unsafe fn apply_keymap_entries(buf: *mut buf_T) {
    // SAFETY: caller contract; the garray holds `ga_len` entries with
    // NUL-terminated strings, and `do_map` only reads the command it is
    // given.
    unsafe {
        for i in 0..(*buf).b_kmap_ga.ga_len {
            let kp = ((*buf).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize);
            let from = CStr::from_ptr((*kp).from).to_bytes();
            let to = CStr::from_ptr((*kp).to).to_bytes();
            let mut cmd = keymap_map_cmd(from, Some(to));
            do_map(
                MAPTYPE_MAP,
                cmd.as_mut_ptr() as *mut c_char,
                MODE_LANGMAP,
                false,
            );
        }
    }
}

/// The `:lmap`/`:lunmap` argument for one keymap entry: `<buffer> {from}`,
/// plus ` {to}` when mapping. NUL-terminated for `do_map`.
fn keymap_map_cmd(from: &[u8], to: Option<&[u8]>) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(KMAP_LLEN + 11);
    cmd.extend_from_slice(b"<buffer> ");
    cmd.extend_from_slice(from);
    if let Some(to) = to {
        cmd.push(b' ');
        cmd.extend_from_slice(to);
    }
    cmd.push(0);
    cmd
}

/// Free the string entries of a keymap garray (the garray itself is the
/// caller's to clear).
///
/// # Safety
///
/// `kmap_ga` must be a valid keymap garray (`buf_T::b_kmap_ga`).
pub unsafe fn keymap_ga_clear(kmap_ga: *mut garray_T) {
    // SAFETY: caller contract; the garray holds `ga_len` live entries, and
    // each owns its two strings.
    unsafe {
        for i in 0..(*kmap_ga).ga_len {
            let kp = ((*kmap_ga).ga_data as *mut kmap_T).offset(i as isize);
            xfree((*kp).from as *mut c_void);
            xfree((*kp).to as *mut c_void);
        }
    }
}

/// Stop using 'keymap': remove the language mappings and free the entries.
fn keymap_unload() {
    let buf = curbuf.get();
    // SAFETY: curbuf is valid.
    if unsafe { (*buf).b_kmap_state } as c_int & KEYMAP_LOADED == 0 {
        return;
    }
    // Set 'cpoptions' to "C" to avoid line continuation.
    let save_cpo = p_cpo.get();
    p_cpo.set(c"C".as_ptr() as *mut c_char);
    // SAFETY: the garray holds `ga_len` entries with NUL-terminated strings;
    // `do_map` only reads the command, and the entries own what is freed.
    unsafe {
        for i in 0..(*buf).b_kmap_ga.ga_len {
            let kp = ((*buf).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize);
            let mut cmd = keymap_map_cmd(CStr::from_ptr((*kp).from).to_bytes(), None);
            do_map(
                MAPTYPE_UNMAP,
                cmd.as_mut_ptr() as *mut c_char,
                MODE_LANGMAP,
                false,
            );
        }
        keymap_ga_clear(&raw mut (*buf).b_kmap_ga);
    }
    p_cpo.set(save_cpo);
    // SAFETY: the garray's entries were just freed, so clearing it is safe.
    unsafe {
        ga_clear(&raw mut (*buf).b_kmap_ga);
        (*buf).b_kmap_state &= !(KEYMAP_LOADED as int16_t);
        status_redraw_curbuf();
    }
}

/// The keymap name to show in the status line ('statusline' `%k`/`%K` and
/// the mode message): `b:keymap_name`, the 'keymap' value, or "lang".
/// `None` unless language mappings are active for `wp`'s buffer.
///
/// # Safety
///
/// `wp` and its buffer must be valid; curwin/curbuf are restored before
/// returning.
pub unsafe fn keymap_str(wp: *mut win_T) -> Option<CString> {
    // SAFETY: caller contract; the window's buffer is valid.
    let buf = unsafe { (*wp).w_buffer };
    // SAFETY: as above.
    if unsafe { (*buf).b_p_iminsert } != B_IMODE_LMAP {
        return None;
    }
    let old_curbuf = curbuf.get();
    let old_curwin = curwin.get();
    // Evaluate b:keymap_name in wp's buffer.
    curbuf.set(buf);
    curwin.set(wp);
    emsg_skip.set(emsg_skip.get() + 1);
    let mut expr = *b"b:keymap_name\0";
    // SAFETY: `expr` is NUL-terminated and outlives the call; the result is
    // an owned heap string or null.
    let s = unsafe { eval_to_string(expr.as_mut_ptr() as *mut c_char, false, false) };
    emsg_skip.set(emsg_skip.get() - 1);
    curbuf.set(old_curbuf);
    curwin.set(old_curwin);
    // SAFETY: `s` is null or NUL-terminated, and 'keymap' is an option
    // string; both are copied here, and `s` is ours to free afterwards.
    let name = unsafe {
        let name = if !s.is_null() && *s as c_int != NUL {
            CStr::from_ptr(s).to_owned()
        } else if (*buf).b_kmap_state as c_int & KEYMAP_LOADED != 0 {
            CStr::from_ptr((*buf).b_p_keymap).to_owned()
        } else {
            CString::new("lang").expect("no interior NUL")
        };
        xfree(s as *mut c_void);
        name
    };
    Some(name)
}
