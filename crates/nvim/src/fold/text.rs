//! Rendering the one line a closed fold is drawn as.
//!
//! This is display, not tree: 'foldtext' is an expression the user controls,
//! so most of the work here is evaluating it safely (a failed evaluation
//! latches, so a broken 'foldtext' does not raise an error per fold) and
//! sanitising whatever it answers.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
use crate::api::extmark::parse_virt_text;
use crate::winlayer::WinId;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{ptr2cells_at, transstr_owned, vim_isprintc};
use crate::eval::eval_foldtext;
use crate::eval::vars::{set_vim_var_nr, set_vim_var_string};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memory::XString;
use crate::message::state::did_emsg;
use crate::os::cshim::ngettext;
use crate::runtime::state::current_sctx;
use crate::strings::{FmtArg, format_into};
use crate::types::Vv;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_ulong};

use super::marker::*;
use super::*;
use crate::options::kWinOptFoldtext;
use crate::optionstr::OptString;

/// What [`get_foldtext`] rendered a closed fold as.
pub(crate) enum FoldText {
    /// The default text, NUL-terminated in the caller's scratch buffer.
    Scratch,
    /// The string 'foldtext' answered, made printable: the caller's.
    Owned(XString),
    /// The `[text, hl]` chunks 'foldtext' answered, which the caller draws;
    /// the scratch buffer holds the empty string.
    Chunks(VirtText),
}

/// Generates text to display
///
/// `buf` — scratch for the default text, used when 'foldtext' is empty or
///            failed.
/// `lnum` — first line of the fold, with last line `lnume`.
pub(crate) fn get_foldtext(
    window: Win,
    lnum: LineNr,
    lnume: LineNr,
    foldinfo: FoldInfo,
    buf: &mut [u8; FOLD_TEXT_LEN as usize],
) -> FoldText {
    let mut answer: Option<FoldText> = None;
    // A 'foldtext' that errored is not evaluated again until the window or
    // the direction of travel changes, so one broken expression does not
    // raise one error per drawn fold.
    static GOT_FDT_ERROR: GlobalCell<bool> = GlobalCell::new(false);
    static LAST_WIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
    static LAST_LNUM: GlobalCell<LineNr> = GlobalCell::new(0);
    let save_did_emsg = did_emsg.get();
    if LAST_WIN.get() != Some(window.id()) || LAST_LNUM.get() > lnum || LAST_LNUM.get() == 0 {
        GOT_FDT_ERROR.set(false);
    }
    if !GOT_FDT_ERROR.get() {
        did_emsg.set(0);
    }
    let win = window;
    if c_int::from(win.w_onebuf_opt.wo_fdt.first_byte()) != NUL {
        let level = foldinfo.fi_level.clamp(0, 21);
        let dashes = vec![b'-'; usize::try_from(level).unwrap_or(0)];
        set_vim_var_nr(Vv::Foldstart, lnum as VarNumber);
        set_vim_var_nr(Vv::Foldend, lnume as VarNumber);
        set_vim_var_string(Vv::Folddashes, Some(&dashes));
        set_vim_var_nr(Vv::Foldlevel, level as VarNumber);
        if !GOT_FDT_ERROR.get() {
            let saved = switch_to(window);
            let saved_sctx = current_sctx.get();
            current_sctx.set(win.w_onebuf_opt.wo_script_ctx[kWinOptFoldtext as usize]);
            let no_emsg = Suppress::emsg();
            let obj: Object = eval_foldtext(window);
            match obj.kind() {
                kObjectTypeArray => {
                    // A list of `[text, hl]` chunks: the caller draws them,
                    // and the text is empty.
                    let chunks = obj.into_array().expect("the tag says Array");
                    if let Ok(parsed) = parse_virt_text(&chunks, None) {
                        buf[0] = 0;
                        answer = Some(FoldText::Chunks(parsed));
                    }
                }
                kObjectTypeString => {
                    let string = obj.into_string().expect("the tag says String");
                    // The null string is what a failed evaluation answers.
                    if !string.is_null() {
                        let bytes = string.as_bytes();
                        // Read as the C string it was: up to the first NUL.
                        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                        answer = Some(FoldText::Owned(printable(bytes[..end].to_vec())));
                    }
                }
                _ => drop(obj),
            }
            drop(no_emsg);
            if answer.is_none() || did_emsg.get() != 0 {
                GOT_FDT_ERROR.set(true);
            }
            saved.restore();
            current_sctx.set(saved_sctx);
        }
        LAST_LNUM.set(lnum);
        LAST_WIN.set(Some(window.id()));
        set_vim_var_string(Vv::Folddashes, None);
        if did_emsg.get() == 0 && save_did_emsg != 0 {
            did_emsg.set(save_did_emsg);
        }
    }
    answer.unwrap_or_else(|| {
        let count = lnume - lnum + 1;
        let one = c"+--%3d line folded";
        let many = c"+--%3d lines folded ";
        let fmt = ngettext(one, many, count as c_ulong);
        format_into(&mut buf[..], fmt.to_bytes(), &[FmtArg::from(count)]);
        FoldText::Scratch
    })
}

/// 'foldtext''s answer as it can be drawn: tabs become spaces, and anything
/// unprintable or wide sends the whole string through `transstr`.
fn printable(mut text: Vec<u8>) -> XString {
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let len = c_cluster_len(rest);
        if len > 1 {
            if !vim_isprintc(char_at(rest)) {
                break;
            }
            at += len - 1;
        } else if c_int::from(rest[0]) == TAB {
            text[at] = b' ';
        } else if ptr2cells_at(rest) > 1 {
            break;
        }
        at += 1;
    }
    if at < text.len() {
        transstr_owned(&text, true)
    } else {
        XString::from_vec(text)
    }
}

/// Remove 'foldmarker' and 'commentstring' from `text[from..]`, in place.
pub(super) fn foldtext_cleanup(text: &mut Vec<u8>, from: usize) {
    // 'commentstring' split around its `%s`, with the padding trimmed; a copy
    // of the current buffer's value.
    let cms = Buf::current().b_p_cms.get();
    let cms = &cms[skip_white(&cms)..];
    let mut cms_slen = trimmed_len(cms);
    let mut cms_end: Option<&[u8]> = None;
    if let Some(at) = cms.windows(2).position(|w| w == b"%s") {
        cms_slen = trimmed_len(&cms[..at.min(cms_slen)]);
        let after = &cms[at + 2..];
        let after = &after[skip_white(after)..];
        let end_len = trimmed_len(after);
        cms_end = Some(&after[..end_len]);
    }
    let cms_start = &cms[..cms_slen];
    parse_marker(Win::current());
    // A copy: the window's option is not held while the text is edited.
    let foldmarker = Win::current().w_onebuf_opt.wo_fmr.get();
    let (start_marker, end_marker) = markers(&foldmarker);

    // Each half of 'commentstring' is only removed once.
    let mut did1 = false;
    let mut did2 = false;
    let mut s = from;
    while s < text.len() {
        let mut len = 0;
        let rest = &text[s..];
        if rest.starts_with(start_marker) {
            len = start_marker.len();
        } else if rest.starts_with(end_marker) {
            len = end_marker.len();
        }
        if len > 0 {
            // A numbered marker, and any comment opener it sits behind.
            if text
                .get(s + len)
                .is_some_and(|&b| ascii_isdigit(c_int::from(b)))
            {
                len += 1;
            }
            let mut p = s;
            while p > from && ascii_iswhite(c_int::from(text[p - 1])) {
                p -= 1;
            }
            if p >= from + cms_slen && text[p - cms_slen..].starts_with(cms_start) {
                len += s - p + cms_slen;
                s = p - cms_slen;
            }
        } else if let Some(cms_end) = cms_end {
            if !did1 && cms_slen > 0 && rest.starts_with(cms_start) {
                len = cms_slen;
                did1 = true;
            } else if !did2 && !cms_end.is_empty() && rest.starts_with(cms_end) {
                len = cms_end.len();
                did2 = true;
            }
        }
        if len != 0 {
            while text
                .get(s + len)
                .is_some_and(|&b| ascii_iswhite(c_int::from(b)))
            {
                len += 1;
            }
            text.drain(s..s + len);
        } else {
            s += c_cluster_len(&text[s..]).max(1);
        }
    }
}

/// How many leading blanks `bytes` has.
fn skip_white(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .take_while(|&&b| ascii_iswhite(c_int::from(b)))
        .count()
}

/// `bytes.len()` without its trailing blanks.
fn trimmed_len(bytes: &[u8]) -> usize {
    bytes.len()
        - bytes
            .iter()
            .rev()
            .take_while(|&&b| ascii_iswhite(c_int::from(b)))
            .count()
}
