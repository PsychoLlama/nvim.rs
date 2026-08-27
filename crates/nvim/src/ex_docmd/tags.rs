//! `:tag` and the identifier searches that share its argument handling.
//!
//! Both families are dispatched by *spelling*: every one of the twenty-odd
//! commands runs the same handler, which reads its own name out of
//! `cmdnames` to decide what it does. `:djump`, `:dlist`, `:dsearch` and
//! `:dsplit` differ only in their third letter; `:tnext`, `:tprevious`,
//! `:tselect` and the rest only in their second.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;

use crate::ex_docmd::scan::{check_nextcmd, ends_excmd};
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::{
    ACTION_GOTO, ACTION_SHOW, ACTION_SHOW_ALL, ACTION_SPLIT, DT_FIRST, DT_JUMP, DT_LAST, DT_LTAG,
    DT_NEXT, DT_POP, DT_PREV, DT_SELECT, DT_TAG, FIND_ANY, FIND_DEFINE, cmdmod_split, cmdmod_tab,
    cmdnames, kDirectionNotSet,
};
use crate::main::{
    e_trailing_arg, g_do_tagpreview, p_pvh, postponed_split, postponed_split_flags,
    postponed_split_tab,
};
use crate::option::magic_isset;
use crate::regexp::skip_regexp;
use crate::search::find_pattern_in_path;
use crate::tag::do_tag;
use crate::types::{NUL, exarg_T};
use crate::winlayer::Ea;
use ::libc::strlen;

/// `:isearch`, `:ilist`, `:ijump`, `:isplit` and their `:d…` twins.
///
/// The third letter of the name says what to do with what is found, and
/// the first says whether the search is for a *definition* or for any
/// occurrence.
pub(crate) unsafe fn ex_findpat(eap: *mut exarg_T) {
    let mut ea = unsafe { Ea::new(eap) };
    let name = cmdnames[ea.cmdidx as usize].cmd_name;
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
    if ascii_isdigit(byte(ea.arg)) {
        n = unsafe { getdigits_int(ea.arg_ptr(), false, 0) };
        ea.arg = skipwhite(ea.arg);
    }

    // `/pat/` searches for a pattern rather than for a whole word, and
    // the rest of the line after it may be another command.
    let mut whole = true;
    if byte(ea.arg) == '/' as c_int {
        whole = false;
        ea.arg = unsafe { ea.arg.add(1) };
        let mut p = unsafe { skip_regexp(ea.arg, '/' as c_int, magic_isset() as c_int) };
        if unsafe { *p } != 0 {
            unsafe { *p = NUL as c_char };
            p = unsafe { skipwhite(p.add(1)) };
            if ends_excmd(byte(p)) == 0 {
                ea.errmsg = Some(unsafe { ex_errmsg(e_trailing_arg.as_ptr(), p) });
            } else {
                ea.nextcmd = unsafe { check_nextcmd(p) };
            }
        }
    }

    if ea.skip == 0 {
        unsafe {
            find_pattern_in_path(
                ea.arg,
                kDirectionNotSet,
                strlen(ea.arg),
                whole,
                ea.forceit == 0,
                if *ea.cmd as c_int == 'd' as c_int {
                    FIND_DEFINE as c_int
                } else {
                    FIND_ANY as c_int
                },
                n,
                action,
                ea.line1,
                ea.line2,
                ea.forceit != 0,
                false,
            )
        };
    }
}

/// `:ptag` and friends — the same as `:tag`, in the preview window.
pub(crate) unsafe fn ex_ptag(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    g_do_tagpreview.set(p_pvh.get() as c_int);
    unsafe { ex_tag_cmd(eap, cmdnames[eap.cmdidx as usize].cmd_name.add(1)) };
}

/// `:stag` and friends — the same as `:tag`, in a new window.
pub(crate) unsafe fn ex_stag(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    // `-1` means "split, and let the tag code choose the size".
    postponed_split.set(-1);
    postponed_split_flags.set(cmdmod_split());
    postponed_split_tab.set(cmdmod_tab());
    unsafe { ex_tag_cmd(eap, cmdnames[eap.cmdidx as usize].cmd_name.add(1)) };
    postponed_split_flags.set(0);
    postponed_split_tab.set(0);
}

/// `:tag`, `:tnext`, `:tselect`, `:tjump`, `:tprevious`, `:tpop`, …
pub(crate) unsafe fn ex_tag(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    unsafe { ex_tag_cmd(eap, cmdnames[eap.cmdidx as usize].cmd_name) };
}

/// Run a tag command named by `name`, whose *second* letter says which one
/// it is.
///
/// `ex_ptag` and `ex_stag` pass the name one byte in, so that `:ptnext`
/// and `:stselect` read the same letter `:tnext` and `:tselect` do. A
/// leading `l` overrides everything: it is the location-list form.
unsafe fn ex_tag_cmd(eap: Ea, name: *const c_char) {
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
    unsafe {
        do_tag(
            eap.arg,
            cmd,
            if eap.addr_count > 0 {
                eap.line2 as c_int
            } else {
                1
            },
            eap.forceit,
            true,
        )
    };
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
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
