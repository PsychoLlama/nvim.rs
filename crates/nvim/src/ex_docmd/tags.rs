//! `:tag` and the identifier searches that share its argument handling.
//!
//! Both families are dispatched by *spelling*: every one of the twenty-odd
//! commands runs the same handler, which reads its own name out of
//! `cmdnames` to decide what it does. `:djump`, `:dlist`, `:dsearch` and
//! `:dsplit` differ only in their third letter; `:tnext`, `:tprevious`,
//! `:tselect` and the rest only in their second.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::regexp::skip_regexp_at;
use core::ffi::{c_char, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int_at;

use crate::ex_docmd::scan::ends_excmd;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::{
    ACTION_GOTO, ACTION_SHOW, ACTION_SHOW_ALL, ACTION_SPLIT, DT_FIRST, DT_JUMP, DT_LAST, DT_LTAG,
    DT_NEXT, DT_POP, DT_PREV, DT_SELECT, DT_TAG, FIND_ANY, FIND_DEFINE, cmdmod_split, cmdmod_tab,
    cmdnames, kDirectionNotSet,
};
use crate::message::e_trailing_arg;
use crate::option::magic_isset;
use crate::option::vars::p_pvh;
use crate::search::find_pattern_in_path;
use crate::tag::do_tag;
use crate::tag::state::{
    g_do_tagpreview, postponed_split, postponed_split_flags, postponed_split_tab,
};
use crate::types::ExArg;

/// `:isearch`, `:ilist`, `:ijump`, `:isplit` and their `:d…` twins.
///
/// The third letter of the name says what to do with what is found, and
/// the first says whether the search is for a *definition* or for any
/// occurrence.
pub(crate) fn ex_findpat(excmd: &mut ExArg) {
    let name = cmdnames[excmd.cmdidx.index()].cmd_name;
    let action = match ubyte_at(name, 2) {
        // `:isearch`/`:dsearch` show the first match; `:psearch` goes
        // to it in the preview window.
        b'e' => {
            if byte(name) == 'p' as c_int {
                ACTION_GOTO
            } else {
                ACTION_SHOW
            }
        }
        b'i' => ACTION_SHOW_ALL, // `:ilist`
        b'u' => ACTION_GOTO,     // `:ijump`
        _ => ACTION_SPLIT,       // `:isplit`
    } as c_int;

    // A leading count is which match to take.
    let mut n = 1;
    if ascii_isdigit(c_int::from(excmd.line.byte_at(excmd.line.arg))) {
        let arg = excmd.line.arg;
        let (count, at) = getdigits_int_at(excmd.line.buffer_mut(), arg, false, 0);
        n = count;
        excmd.line.arg = excmd.line.skip_white(at);
    }

    // `/pat/` searches for a pattern rather than for a whole word, and
    // the rest of the line after it may be another command.
    let mut whole = true;
    if c_int::from(excmd.line.byte_at(excmd.line.arg)) == '/' as c_int {
        whole = false;
        excmd.line.arg += 1;
        let magic = magic_isset() as c_int;
        let mut at =
            excmd.line.arg + skip_regexp_at(excmd.line.tail(excmd.line.arg), '/' as c_int, magic);
        if excmd.line.byte_at(at) != 0 {
            excmd.line.terminate_at(at);
            at = excmd.line.skip_white(at + 1);
            if ends_excmd(c_int::from(excmd.line.byte_at(at))) == 0 {
                excmd.errmsg = Some(ex_errmsg(e_trailing_arg, excmd.line.cstr_from(at)));
            } else {
                excmd.line.next = excmd.line.check_next(at);
            }
        }
    }

    if !excmd.skip {
        let define = excmd.line.byte_at(excmd.line.cmd) == b'd';
        find_pattern_in_path(
            excmd.line.arg(),
            kDirectionNotSet,
            whole,
            !excmd.forceit,
            if define {
                FIND_DEFINE as c_int
            } else {
                FIND_ANY as c_int
            },
            n,
            action,
            excmd.line1,
            excmd.line2,
            excmd.forceit,
            false,
        );
    }
}

/// `:ptag` and friends — the same as `:tag`, in the preview window.
pub(crate) fn ex_ptag(excmd: &mut ExArg) {
    g_do_tagpreview.set(p_pvh() as c_int);
    unsafe { ex_tag_cmd(excmd, cmdnames[excmd.cmdidx.index()].cmd_name.add(1)) };
}

/// `:stag` and friends — the same as `:tag`, in a new window.
pub(crate) fn ex_stag(excmd: &mut ExArg) {
    // `-1` means "split, and let the tag code choose the size".
    postponed_split.set(-1);
    postponed_split_flags.set(cmdmod_split());
    postponed_split_tab.set(cmdmod_tab());
    unsafe { ex_tag_cmd(excmd, cmdnames[excmd.cmdidx.index()].cmd_name.add(1)) };
    postponed_split_flags.set(0);
    postponed_split_tab.set(0);
}

/// `:tag`, `:tnext`, `:tselect`, `:tjump`, `:tprevious`, `:tpop`, …
pub(crate) fn ex_tag(excmd: &mut ExArg) {
    unsafe { ex_tag_cmd(excmd, cmdnames[excmd.cmdidx.index()].cmd_name) };
}

/// Run a tag command named by `name`, whose *second* letter says which one
/// it is.
///
/// `ex_ptag` and `ex_stag` pass the name one byte in, so that `:ptnext`
/// and `:stselect` read the same letter `:tnext` and `:tselect` do. A
/// leading `l` overrides everything: it is the location-list form.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
unsafe fn ex_tag_cmd(excmd: &mut ExArg, name: *const c_char) {
    let mut cmd = match ubyte_at(name, 1) {
        b'j' => DT_JUMP,
        b's' => DT_SELECT,
        b'p' | b'N' => DT_PREV,
        b'n' => DT_NEXT,
        b'o' => DT_POP,
        b'f' | b'r' => DT_FIRST,
        b'l' => DT_LAST,
        _ => DT_TAG,
    } as c_int;
    if byte(name) == 'l' as c_int {
        cmd = DT_LTAG as c_int;
    }
    let tag = excmd.line.cstr_from(excmd.line.arg);
    let count = if excmd.addr_count > 0 {
        excmd.line2 as c_int
    } else {
        1
    };
    do_tag(tag, cmd, count, c_int::from(excmd.forceit), true);
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// The byte at `p[i]`, unsigned, as the C's `(uint8_t)*(p + i)` reads it.
fn ubyte_at(p: *const c_char, i: isize) -> u8 {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as u8 }
}
