//! `:tag` and the identifier searches that share its argument handling.
//!
//! Both families are dispatched by *spelling*: every one of the twenty-odd
//! commands runs the same handler, which reads its own name out of
//! `cmdnames` to decide what it does. `:djump`, `:dlist`, `:dsearch` and
//! `:dsplit` differ only in their third letter; `:tnext`, `:tprevious`,
//! `:tselect` and the rest only in their second.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::number_as_int;
use crate::regexp::skip_regexp_at;
use crate::search::ACTION_GOTO;
use crate::search::ACTION_SHOW;
use crate::search::ACTION_SHOW_ALL;
use crate::search::ACTION_SPLIT;
use crate::search::FIND_ANY;
use crate::search::FIND_DEFINE;
use core::ffi::c_int;

use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int_at;

use crate::ex_docmd::scan::ends_excmd;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::{DIRECTION_NOT_SET, cmdmod_split, cmdmod_tab, cmdnames};
use crate::message::E_TRAILING_ARG;
use crate::option::magic_isset;
use crate::option::vars::p_pvh;
use crate::search::find_pattern_in_path;
use crate::tag::state::{
    g_do_tagpreview, postponed_split, postponed_split_flags, postponed_split_tab,
};
use crate::tag::{TagCmd, do_tag};
use crate::types::ExArg;

/// `:isearch`, `:ilist`, `:ijump`, `:isplit` and their `:d…` twins.
///
/// The third letter of the name says what to do with what is found, and
/// the first says whether the search is for a *definition* or for any
/// occurrence.
pub(crate) fn ex_findpat(excmd: &mut ExArg) {
    let name = cmdnames[excmd.cmdidx.index()].cmd_name.to_bytes_with_nul();
    let action = match name[2] {
        // `:isearch`/`:dsearch` show the first match; `:psearch` goes
        // to it in the preview window.
        b'e' => {
            if name[0] == b'p' {
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
    if excmd.line.byte_at(excmd.line.arg) == b'/' {
        whole = false;
        excmd.line.arg += 1;
        let magic = c_int::from(magic_isset());
        let mut at = excmd.line.arg
            + skip_regexp_at(excmd.line.tail(excmd.line.arg), c_int::from(b'/'), magic);
        if excmd.line.byte_at(at) != 0 {
            excmd.line.terminate_at(at);
            at = excmd.line.skip_white(at + 1);
            if ends_excmd(c_int::from(excmd.line.byte_at(at))) == 0 {
                excmd.errmsg = Some(ex_errmsg(E_TRAILING_ARG, excmd.line.cstr_from(at)));
            } else {
                excmd.line.next = excmd.line.check_next(at);
            }
        }
    }

    if !excmd.skip {
        let define = excmd.line.byte_at(excmd.line.cmd) == b'd';
        find_pattern_in_path(
            excmd.line.arg(),
            DIRECTION_NOT_SET,
            whole,
            !excmd.forceit,
            if define { FIND_DEFINE } else { FIND_ANY },
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
    g_do_tagpreview.set(number_as_int(p_pvh()));
    ex_tag_cmd(
        excmd,
        &cmdnames[excmd.cmdidx.index()].cmd_name.to_bytes()[1..],
    );
}

/// `:stag` and friends — the same as `:tag`, in a new window.
pub(crate) fn ex_stag(excmd: &mut ExArg) {
    // `-1` means "split, and let the tag code choose the size".
    postponed_split.set(-1);
    postponed_split_flags.set(cmdmod_split());
    postponed_split_tab.set(cmdmod_tab());
    ex_tag_cmd(
        excmd,
        &cmdnames[excmd.cmdidx.index()].cmd_name.to_bytes()[1..],
    );
    postponed_split_flags.set(0);
    postponed_split_tab.set(0);
}

/// `:tag`, `:tnext`, `:tselect`, `:tjump`, `:tprevious`, `:tpop`, …
pub(crate) fn ex_tag(excmd: &mut ExArg) {
    ex_tag_cmd(excmd, cmdnames[excmd.cmdidx.index()].cmd_name.to_bytes());
}

/// Run a tag command named by `name`, whose *second* letter says which one
/// it is.
///
/// `ex_ptag` and `ex_stag` pass the name one byte in, so that `:ptnext`
/// and `:stselect` read the same letter `:tnext` and `:tselect` do. A
/// leading `l` overrides everything: it is the location-list form.
fn ex_tag_cmd(excmd: &mut ExArg, name: &[u8]) {
    let mut cmd = match name.get(1).copied().unwrap_or(0) {
        b'j' => TagCmd::Jump,
        b's' => TagCmd::Select,
        b'p' | b'N' => TagCmd::Prev,
        b'n' => TagCmd::Next,
        b'o' => TagCmd::Pop,
        b'f' | b'r' => TagCmd::First,
        b'l' => TagCmd::Last,
        _ => TagCmd::Tag,
    };
    if name.first() == Some(&b'l') {
        cmd = TagCmd::LTag;
    }
    let tag = excmd.line.cstr_from(excmd.line.arg);
    let count = if excmd.addr_count > 0 {
        excmd.number2
    } else {
        1
    };
    do_tag(tag, cmd, count, c_int::from(excmd.forceit), true);
}
