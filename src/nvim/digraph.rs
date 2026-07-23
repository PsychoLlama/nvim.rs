//! Digraphs: CTRL-K input, the `:digraph[s]` ex command, the
//! `digraph_get()`/`digraph_set()` family of eval functions, and — sharing
//! this file like upstream `digraph.c` — the 'keymap' option's language
//! mapping loader (`:loadkeymap`).
//!
//! The default table lives in [`tables`]; user-defined digraphs are an
//! ordered list that shadows it (`:digraph`, `digraph_set()`). Lookups scan
//! the user list first, in insertion order, then the default table.

mod tables;

use crate::src::nvim::charset::char2cells;
use crate::src::nvim::drawscreen::status_redraw_curbuf;
use crate::src::nvim::eval::typval::{
    tv_check_for_opt_bool_arg, tv_get_bool, tv_get_string_buf_chk, tv_get_string_chk,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_list, tv_list_append_string,
};
use crate::src::nvim::eval_1::eval_to_string;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, getline_equal};
use crate::src::nvim::ex_getln::putcmdline;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::src::nvim::getchar::plain_vgetc;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    allow_keys, cmdline_star, curbuf, curwin, e_number_exp, emsg_skip, got_int, msg_col,
    no_mapping, p_cpo, p_dg, p_enc, Columns,
};
use crate::src::nvim::mapping::do_map;
use crate::src::nvim::mbyte::{mb_cptr2char_adv, utf_char2bytes, utf_iscomposing_first};
use crate::src::nvim::memory::{xfree, xmemdupz};
use crate::src::nvim::message::{
    emsg, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar, semsg,
};
use crate::src::nvim::normal::add_to_showcmd;
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::runtime::{getsourceline, source_runtime};
use crate::src::nvim::types::{
    exarg_T, garray_T, int16_t, list_T, typval_T, varnumber_T, win_T, BoolVarValue, EvalFuncData,
    OptInt, VarType,
};
use core::ffi::{c_char, c_int, c_void, CStr};
use std::ffi::CString;

const NUL: c_int = 0;
const ESC: c_int = 27;
const CTRL_H: c_int = 8;
/// Special key code for `<BS>` (`K_BS` upstream).
const K_BS: c_int = -(('k' as c_int) + (('b' as c_int) << 8));
const OK: c_int = 1;
const FAIL: c_int = 0;
/// Highlight ids used by the `:digraphs` listing.
const HLF_CM: c_int = 11;
const HLF_8: c_int = 1;

const VAR_UNKNOWN: VarType = 0;
const VAR_STRING: VarType = 2;
const VAR_LIST: VarType = 4;
const VAR_BOOL: VarType = 7;
const K_BOOL_VAR_FALSE: BoolVarValue = 0;
const K_BOOL_VAR_TRUE: BoolVarValue = 1;

const E_DIGRAPH_TWO_CHARS: &[u8] = b"E1214: Digraph must be just two characters: %s\0";
const E_DIGRAPH_ONE_CHAR: &[u8] = b"E1215: Digraph must be one character: %s\0";
const E_DIGRAPH_SETLIST: &[u8] =
    b"E1216: digraph_setlist() argument must be a list of lists with two items\0";

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
        unsafe { add_to_showcmd(c) };
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
        // SAFETY: utf_char2bytes writes at most 6 bytes; msg stays
        // NUL-terminated.
        unsafe {
            utf_char2bytes(char1, msg.as_mut_ptr() as *mut c_char);
            semsg(
                gettext(E_DIGRAPH_TWO_CHARS.as_ptr() as *const c_char),
                msg.as_ptr(),
            );
        }
        return false;
    }
    if char1 == ESC || char2 == ESC {
        // SAFETY: plain message call with a static string.
        unsafe {
            emsg(gettext(
                b"E104: Escape not allowed in digraph\0".as_ptr() as *const c_char
            ));
        }
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

/// Parse a decimal number like strict `getdigits_int`: values that don't
/// fit in an int abort in C, so panicking here preserves behavior.
fn parse_digits(s: &[u8]) -> (c_int, &[u8]) {
    let n = s.iter().take_while(|b| b.is_ascii_digit()).count();
    let (digits, rest) = s.split_at(n);
    let mut value: i64 = 0;
    for &d in digits {
        value = value.saturating_mul(10).saturating_add((d - b'0') as i64);
    }
    assert!(value <= c_int::MAX as i64, "digraph value out of range");
    (value as c_int, rest)
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
        if !s.first().map_or(false, u8::is_ascii_digit) {
            // SAFETY: plain message call with a static string.
            unsafe { emsg(gettext(e_number_exp.ptr() as *const c_char)) };
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
    // SAFETY: name is NUL-terminated; gettext returns a valid C string.
    unsafe {
        if msg_col.get() > 0 {
            msg_putchar('\n' as c_int);
        }
        msg_outtrans(gettext(name.as_ptr() as *const c_char), HLF_CM, false);
        msg_putchar('\n' as c_int);
    }
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
    // SAFETY: plain message output.
    unsafe {
        if msg_col.get() > Columns.get() - LIST_WIDTH {
            msg_putchar('\n' as c_int);
        }
        if msg_col.get() % LIST_WIDTH != 0 {
            msg_advance((msg_col.get() / LIST_WIDTH + 1) * LIST_WIDTH);
        }
    }
    outtrans(&[dp.char1, dp.char2, b' '], 0);
    let mut buf = [0u8; 12];
    let mut len = 0;
    // SAFETY: value check on the composing property; buf has room for the
    // longest UTF-8 sequence plus the leading space.
    if unsafe { utf_iscomposing_first(dp.result) } {
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
    // SAFETY: plain message output.
    unsafe {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
        msg_putchar('\n' as c_int);
    }
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
        unsafe { fast_breakcheck() };
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
        unsafe { fast_breakcheck() };
    }
}

/// Append `[chars, result]` to the `digraph_getlist()` result.
///
/// # Safety
///
/// `l` must be a valid list.
unsafe fn getlist_append_pair(dp: &Digraph, l: *mut list_T) {
    let l2 = tv_list_alloc(2);
    tv_list_append_list(l, l2);
    let chars = [dp.char1, dp.char2, 0];
    tv_list_append_string(l2, chars.as_ptr() as *const c_char, -1);
    let mut buf = [0u8; 7];
    utf_char2bytes(dp.result, buf.as_mut_ptr() as *mut c_char);
    tv_list_append_string(l2, buf.as_ptr() as *const c_char, -1);
}

/// Build the `digraph_getlist()` result: user digraphs, plus the effective
/// defaults when `list_all` is given.
///
/// # Safety
///
/// `rettv` must be a valid return-value slot.
unsafe fn digraph_getlist_common(list_all: bool, rettv: *mut typval_T) {
    let user_len = USER_DIGRAPHS.with(|user| user.len());
    tv_list_alloc_ret(rettv, (tables::DEFAULT_DIGRAPHS.len() + user_len) as isize);
    let list = (*rettv).vval.v_list;
    if list_all {
        for dp in tables::DEFAULT_DIGRAPHS.iter() {
            if got_int.get() {
                break;
            }
            let result = get_exact_digraph(dp.char1 as c_int, dp.char2 as c_int, false);
            if result != 0 && result != dp.char2 as c_int {
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

/// Get the two digraph characters from a typval argument. Emits E1214 on
/// anything but exactly two characters.
///
/// # Safety
///
/// `arg` must be a valid typval.
unsafe fn get_digraph_chars(arg: *const typval_T, char1: &mut c_int, char2: &mut c_int) -> c_int {
    let mut buf_chars = [0 as c_char; 65];
    let chars = tv_get_string_buf_chk(arg, buf_chars.as_mut_ptr());
    let mut p = chars;
    if !p.is_null() && *p as c_int != NUL {
        *char1 = mb_cptr2char_adv(&mut p);
        if *p as c_int != NUL {
            *char2 = mb_cptr2char_adv(&mut p);
            if *p as c_int == NUL {
                if check_digraph_chars_valid(*char1, *char2) {
                    return OK;
                }
                return FAIL;
            }
        }
    }
    semsg(
        gettext(E_DIGRAPH_TWO_CHARS.as_ptr() as *const c_char),
        chars,
    );
    FAIL
}

/// Shared body of `digraph_set()` and `digraph_setlist()`.
///
/// # Safety
///
/// Both arguments must be valid typvals.
unsafe fn digraph_set_common(argchars: *const typval_T, argdigraph: *const typval_T) -> bool {
    let mut char1 = 0;
    let mut char2 = 0;
    if get_digraph_chars(argchars, &mut char1, &mut char2) == FAIL {
        return false;
    }
    let mut buf_digraph = [0 as c_char; 65];
    let digraph = tv_get_string_buf_chk(argdigraph, buf_digraph.as_mut_ptr());
    if digraph.is_null() {
        return false;
    }
    let mut p = digraph;
    let n = mb_cptr2char_adv(&mut p);
    if *p as c_int != NUL {
        semsg(
            gettext(E_DIGRAPH_ONE_CHAR.as_ptr() as *const c_char),
            digraph,
        );
        return false;
    }
    register_digraph(char1, char2, n);
    true
}

/// `digraph_get()`.
///
/// # Safety
///
/// Standard eval-function contract: `argvars` and `rettv` are valid.
pub unsafe extern "C" fn f_digraph_get(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = core::ptr::null_mut();
    let digraphs = tv_get_string_chk(argvars);
    if digraphs.is_null() {
        return;
    }
    let bytes = CStr::from_ptr(digraphs).to_bytes();
    if bytes.len() != 2 {
        semsg(
            gettext(E_DIGRAPH_TWO_CHARS.as_ptr() as *const c_char),
            digraphs,
        );
        return;
    }
    // The chars go through `char` in C, hence the sign extension.
    let code = digraph_get(bytes[0] as i8 as c_int, bytes[1] as i8 as c_int, false);
    let mut buf = [0u8; 7];
    let len = utf_char2bytes(code, buf.as_mut_ptr() as *mut c_char) as usize;
    (*rettv).vval.v_string = xmemdupz(buf.as_ptr() as *const c_void, len) as *mut c_char;
}

/// `digraph_getlist()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe extern "C" fn f_digraph_getlist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    if tv_check_for_opt_bool_arg(argvars, 0) == FAIL {
        return;
    }
    let list_all = (*argvars).v_type != VAR_UNKNOWN && tv_get_bool(argvars) != 0 as varnumber_T;
    digraph_getlist_common(list_all, rettv);
}

/// `digraph_set()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe extern "C" fn f_digraph_set(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_BOOL;
    (*rettv).vval.v_bool = K_BOOL_VAR_FALSE;
    if !digraph_set_common(argvars, argvars.offset(1)) {
        return;
    }
    (*rettv).vval.v_bool = K_BOOL_VAR_TRUE;
}

/// `digraph_setlist()`.
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe extern "C" fn f_digraph_setlist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_BOOL;
    (*rettv).vval.v_bool = K_BOOL_VAR_FALSE;
    if (*argvars).v_type != VAR_LIST {
        emsg(gettext(E_DIGRAPH_SETLIST.as_ptr() as *const c_char));
        return;
    }
    let pl = (*argvars).vval.v_list;
    if pl.is_null() {
        (*rettv).vval.v_bool = K_BOOL_VAR_TRUE;
        return;
    }
    let mut pli = (*pl).lv_first;
    while !pli.is_null() {
        if (*pli).li_tv.v_type != VAR_LIST {
            emsg(gettext(E_DIGRAPH_SETLIST.as_ptr() as *const c_char));
            return;
        }
        let l = (*pli).li_tv.vval.v_list;
        if l.is_null() || (*l).lv_len != 2 {
            emsg(gettext(E_DIGRAPH_SETLIST.as_ptr() as *const c_char));
            return;
        }
        let first = (*l).lv_first;
        if !digraph_set_common(&(*first).li_tv, &(*(*first).li_next).li_tv) {
            return;
        }
        pli = (*pli).li_next;
    }
    (*rettv).vval.v_bool = K_BOOL_VAR_TRUE;
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
const MODE_LANGMAP: c_int = 0x20;

/// One `:loadkeymap` entry: heap C strings owned by the buffer's
/// `b_kmap_ga` (freed by [`keymap_ga_clear`]).
#[repr(C)]
struct kmap_T {
    from: *mut c_char,
    to: *mut c_char,
}

/// Source the keymap file for the current buffer's 'keymap' (or unload
/// language mappings when it is empty). Returns an error message or null.
pub fn keymap_init() -> *const c_char {
    // SAFETY: curbuf is valid; the sourced script may change buffer options
    // but not free the buffer out from under us.
    unsafe {
        let buf = curbuf.get();
        (*buf).b_kmap_state &= !(KEYMAP_INIT as int16_t);
        if *(*buf).b_p_keymap as c_int == NUL {
            // Stop any active keymap and clear the b:keymap_name variable.
            keymap_unload();
            do_cmdline_cmd(b"unlet! b:keymap_name\0".as_ptr() as *const c_char);
        } else {
            // Source the keymap file; snapshot the names first because the
            // script can set 'keymap' itself.
            let keymap = CStr::from_ptr((*buf).b_p_keymap).to_bytes().to_vec();
            let enc = CStr::from_ptr(p_enc.get()).to_bytes().to_vec();
            let mut name = Vec::with_capacity(keymap.len() + enc.len() + 13);
            name.extend_from_slice(b"keymap/");
            name.extend_from_slice(&keymap);
            name.push(b'_');
            name.extend_from_slice(&enc);
            name.extend_from_slice(b".vim\0");
            if source_runtime(name.as_mut_ptr() as *mut c_char, 0) == FAIL {
                // Try without encoding.
                let mut name = Vec::with_capacity(keymap.len() + 12);
                name.extend_from_slice(b"keymap/");
                name.extend_from_slice(&keymap);
                name.extend_from_slice(b".vim\0");
                if source_runtime(name.as_mut_ptr() as *mut c_char, 0) == FAIL {
                    return b"E544: Keymap file not found\0".as_ptr() as *const c_char;
                }
            }
        }
    }
    core::ptr::null()
}

/// `:loadkeymap` — read language mappings from the file being sourced.
///
/// # Safety
///
/// `eap` must be a valid command block (ex-command contract).
pub unsafe extern "C" fn ex_loadkeymap(eap: *mut exarg_T) {
    if !getline_equal(
        (*eap).ea_getline,
        (*eap).cookie,
        Some(getsourceline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    ) {
        emsg(gettext(
            b"E105: Using :loadkeymap not in a sourced file\0".as_ptr() as *const c_char,
        ));
        return;
    }
    // Stop any active keymap and load the new entries.
    keymap_unload();
    let buf = curbuf.get();
    (*buf).b_kmap_state = 0;
    ga_init(
        &raw mut (*buf).b_kmap_ga,
        core::mem::size_of::<kmap_T>() as c_int,
        20,
    );
    // Set 'cpoptions' to "C" to avoid line continuation.
    let save_cpo = p_cpo.get();
    p_cpo.set(b"C\0".as_ptr() as *const c_char as *mut c_char);
    // Get each line of the sourced file, break at the end.
    loop {
        let line = (*eap).ea_getline.expect("non-null line getter")(0, (*eap).cookie, 0, true);
        if line.is_null() {
            break;
        }
        let bytes = CStr::from_ptr(line).to_bytes();
        let s = skip_white(bytes);
        if !s.is_empty() && s[0] != b'"' {
            let (from, rest) = split_at_white(s);
            let (to, _) = split_at_white(skip_white(rest));
            if from.len() + to.len() >= KMAP_LLEN || from.is_empty() || to.is_empty() {
                if to.is_empty() {
                    emsg(gettext(
                        b"E791: Empty keymap entry\0".as_ptr() as *const c_char
                    ));
                }
            } else {
                let kp =
                    ga_append_via_ptr(&raw mut (*buf).b_kmap_ga, core::mem::size_of::<kmap_T>())
                        as *mut kmap_T;
                (*kp).from = xmemdupz(from.as_ptr() as *const c_void, from.len()) as *mut c_char;
                (*kp).to = xmemdupz(to.as_ptr() as *const c_void, to.len()) as *mut c_char;
            }
        }
        xfree(line as *mut c_void);
    }
    // Setup the mappings for the current entries.
    for i in 0..(*buf).b_kmap_ga.ga_len {
        let kp = ((*buf).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize);
        let mut cmd = Vec::with_capacity(KMAP_LLEN + 11);
        cmd.extend_from_slice(b"<buffer> ");
        cmd.extend_from_slice(CStr::from_ptr((*kp).from).to_bytes());
        cmd.push(b' ');
        cmd.extend_from_slice(CStr::from_ptr((*kp).to).to_bytes());
        cmd.push(0);
        do_map(
            MAPTYPE_MAP,
            cmd.as_mut_ptr() as *mut c_char,
            MODE_LANGMAP,
            false,
        );
    }
    p_cpo.set(save_cpo);
    (*buf).b_kmap_state |= KEYMAP_LOADED as int16_t;
    status_redraw_curbuf();
}

/// Free the string entries of a keymap garray (the garray itself is the
/// caller's to clear).
///
/// # Safety
///
/// `kmap_ga` must be a valid keymap garray (`buf_T::b_kmap_ga`).
pub unsafe fn keymap_ga_clear(kmap_ga: *mut garray_T) {
    for i in 0..(*kmap_ga).ga_len {
        let kp = ((*kmap_ga).ga_data as *mut kmap_T).offset(i as isize);
        xfree((*kp).from as *mut c_void);
        xfree((*kp).to as *mut c_void);
    }
}

/// Stop using 'keymap': remove the language mappings and free the entries.
fn keymap_unload() {
    // SAFETY: curbuf is valid; do_map only parses the given command.
    unsafe {
        let buf = curbuf.get();
        if (*buf).b_kmap_state as c_int & KEYMAP_LOADED == 0 {
            return;
        }
        // Set 'cpoptions' to "C" to avoid line continuation.
        let save_cpo = p_cpo.get();
        p_cpo.set(b"C\0".as_ptr() as *const c_char as *mut c_char);
        for i in 0..(*buf).b_kmap_ga.ga_len {
            let kp = ((*buf).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize);
            let mut cmd = Vec::with_capacity(KMAP_LLEN + 10);
            cmd.extend_from_slice(b"<buffer> ");
            cmd.extend_from_slice(CStr::from_ptr((*kp).from).to_bytes());
            cmd.push(0);
            do_map(
                MAPTYPE_UNMAP,
                cmd.as_mut_ptr() as *mut c_char,
                MODE_LANGMAP,
                false,
            );
        }
        keymap_ga_clear(&raw mut (*buf).b_kmap_ga);
        p_cpo.set(save_cpo);
        ga_clear(&raw mut (*buf).b_kmap_ga);
        (*buf).b_kmap_state &= !(KEYMAP_LOADED as int16_t);
        status_redraw_curbuf();
    }
}

/// The keymap name to show in the status line ('statusline' `%k`/`%K` and
/// the mode message): `b:keymap_name`, the 'keymap' value, or "lang".
/// `None` unless language mappings are active for `wp`'s buffer.
pub fn keymap_str(wp: *mut win_T) -> Option<CString> {
    // SAFETY: wp and its buffer are valid; curwin/curbuf are restored
    // before returning.
    unsafe {
        if (*(*wp).w_buffer).b_p_iminsert != B_IMODE_LMAP {
            return None;
        }
        let old_curbuf = curbuf.get();
        let old_curwin = curwin.get();
        // Evaluate b:keymap_name in wp's buffer.
        curbuf.set((*wp).w_buffer);
        curwin.set(wp);
        emsg_skip.set(emsg_skip.get() + 1);
        let mut expr = *b"b:keymap_name\0";
        let s = eval_to_string(expr.as_mut_ptr() as *mut c_char, false, false);
        emsg_skip.set(emsg_skip.get() - 1);
        curbuf.set(old_curbuf);
        curwin.set(old_curwin);
        let name = if !s.is_null() && *s as c_int != NUL {
            CStr::from_ptr(s).to_owned()
        } else if (*(*wp).w_buffer).b_kmap_state as c_int & KEYMAP_LOADED != 0 {
            CStr::from_ptr((*(*wp).w_buffer).b_p_keymap).to_owned()
        } else {
            CString::new("lang").expect("no interior NUL")
        };
        xfree(s as *mut c_void);
        Some(name)
    }
}
