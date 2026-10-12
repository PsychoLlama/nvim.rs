//! Vetting a string option's value, and the sweeps that re-vet every
//! buffer's copy of one.
//!
//! Two things live here that the rest of the tree leans on.
//!
//! The **string options a buffer holds**, [`BUF_STRING_OPTIONS`]: the
//! fields a freshly allocated buffer has to have written before anything
//! reads them.
//!
//! The **rejection message**. A check that has to name what it disliked
//! formats an owned [`OptError`]; upstream formatted into a caller-supplied
//! `errbuf` and let the caller decline one by passing null, which is how a
//! set could fail and report nothing.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::formatted;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::ascii::ascii_isdigit;
use crate::charset::transchar;
use crate::global_cell::{Field, field};
use crate::guard::secure;
use crate::indent_c::parse_cino_for;
use crate::memory::XString;
use crate::option::{kOptFlagNDname, kOptFlagNFname, valid_name};
use crate::options::{
    kOptBackupcopy, kOptBelloff, kOptCasemap, kOptClipboard, kOptCompleteopt, kOptDisplay,
    kOptFoldopen, kOptJumpoptions, kOptRedrawdebug, kOptSessionoptions, kOptSwitchbuf,
    kOptTabclose, kOptTagcase, kOptTermpastefilter, kOptViewoptions, kOptVirtualedit,
    kOptWildoptions, opt_scl_values,
};
use crate::os::cshim::gettext;
use crate::types::{Buffer, Failed, NUL, OptError, StlOpt, uint32_t};

use super::OptStringRef;

use super::{
    SCL_NO, check_str_opt, e_illegal_character_after_chr, e_unbalanced_groups,
    e_unclosed_expression_sequence, opt_strings_ok,
};
use crate::decoration::SCL_NUM;

use crate::winlayer::Buf;
/// The options whose bitmask is derived from a value the startup sequence
/// may have installed without going through `:set`.
pub fn didset_string_options() {
    for idx in [
        kOptCasemap,
        kOptBackupcopy,
        kOptBelloff,
        kOptCompleteopt,
        kOptSessionoptions,
        kOptViewoptions,
        kOptFoldopen,
        kOptDisplay,
        kOptJumpoptions,
        kOptRedrawdebug,
        kOptTagcase,
        kOptTermpastefilter,
        kOptVirtualedit,
        kOptSwitchbuf,
        kOptTabclose,
        kOptWildoptions,
        kOptClipboard,
    ] {
        let _ = check_str_opt(idx, None);
    }
}

/// "E539: Illegal character <x>", naming the byte it did not like.
pub fn illegal_char(c: c_int) -> OptError {
    let fmt = gettext(c"E539: Illegal character <%s>");
    let shown = transchar(c);
    let message = formatted!(OptError::ROOM, fmt.to_bytes(), cstr::in_chars(&shown));
    OptError::Owned(XString::from_vec(message))
}

/// "E535: Illegal character after <%c>", for the options that spell a field
/// as a character followed by a value.
pub(crate) fn illegal_char_after_chr(c: c_int) -> OptError {
    let fmt = gettext(e_illegal_character_after_chr);
    let message = formatted!(OptError::ROOM, fmt.to_bytes(), c);
    OptError::Owned(XString::from_vec(message))
}

/// Every string option a buffer holds, by selector: what a freshly
/// allocated buffer has to have written before anything reads or drops it.
///
/// A buffer is allocated **zeroed**, and all-zero bytes are not a valid
/// `Option<XString>`: the niche the `Option` uses is the vector's capacity,
/// so a zeroed field reads as `Some` over a null pointer. The allocator
/// (`buffer::list`) writes `None` into each of these -- with `write`, which
/// drops nothing -- before the block is a `Buffer` at all. That includes the
/// five in the buffer's own syntax block, which is part of the buffer;
/// `crate::syntax::init_synblock` writes the same five for the standalone
/// block `:ownsyntax` allocates, and writing `None` twice costs nothing.
///
/// This is where upstream's "replace each null option value with the shared
/// empty string" sweep went: `None` *is* that shared empty string, so the
/// value a fresh buffer starts with is written once instead of patched up
/// on every `buf_copy_options`. A selector, not an address, so the list is
/// checked against the fields' types and costs nothing to hold.
pub const BUF_STRING_OPTIONS: [Field<Buffer, Option<XString>>; 58] = [
    field!(Buffer, b_p_bkc),
    field!(Buffer, b_p_bh),
    field!(Buffer, b_p_bt),
    field!(Buffer, b_p_cino),
    field!(Buffer, b_p_cink),
    field!(Buffer, b_p_cinw),
    field!(Buffer, b_p_cinsd),
    field!(Buffer, b_p_com),
    field!(Buffer, b_p_cms),
    field!(Buffer, b_p_cot),
    field!(Buffer, b_p_cpt),
    field!(Buffer, b_p_cfu),
    field!(Buffer, b_p_ofu),
    field!(Buffer, b_p_tfu),
    field!(Buffer, b_p_ffu),
    field!(Buffer, b_p_fenc),
    field!(Buffer, b_p_ff),
    field!(Buffer, b_p_ft),
    field!(Buffer, b_p_fo),
    field!(Buffer, b_p_flp),
    field!(Buffer, b_p_isk),
    field!(Buffer, b_p_def),
    field!(Buffer, b_p_inc),
    field!(Buffer, b_p_inex),
    field!(Buffer, b_p_inde),
    field!(Buffer, b_p_indk),
    field!(Buffer, b_p_fp),
    field!(Buffer, b_p_fex),
    field!(Buffer, b_p_kp),
    field!(Buffer, b_p_lop),
    field!(Buffer, b_p_menc),
    field!(Buffer, b_p_mps),
    field!(Buffer, b_p_nf),
    field!(Buffer, b_p_qe),
    field!(Buffer, b_p_sua),
    field!(Buffer, b_p_syn),
    field!(Buffer, b_p_vsts),
    field!(Buffer, b_p_vsts_nopaste),
    field!(Buffer, b_p_vts),
    field!(Buffer, b_p_keymap),
    field!(Buffer, b_p_gefm),
    field!(Buffer, b_p_gp),
    field!(Buffer, b_p_mp),
    field!(Buffer, b_p_efm),
    field!(Buffer, b_p_ep),
    field!(Buffer, b_p_path),
    field!(Buffer, b_p_tags),
    field!(Buffer, b_p_tc),
    field!(Buffer, b_p_dict),
    field!(Buffer, b_p_dia),
    field!(Buffer, b_p_tsr),
    field!(Buffer, b_p_tsrfu),
    field!(Buffer, b_p_lw),
    field!(Buffer, b_s.b_p_spc),
    field!(Buffer, b_s.b_p_spf),
    field!(Buffer, b_s.b_p_spl),
    field!(Buffer, b_s.b_p_spo),
    field!(Buffer, b_s.b_syn_isk),
];

/// Rebuild what a buffer derives from its string options.
///
/// Upstream spelled this "replace every null option value with the shared
/// empty string", and then re-derived the 'cinoptions' cache from the value
/// it had just made dereferenceable. A buffer's string options own their
/// bytes now and `None` is the shared empty string, so nothing is left to
/// patch up and the cache is the whole of it.
pub fn check_buf_options(buffer: Buf) {
    parse_cino_for(buffer);
}

/// Is `val` a name 'filetype', 'syntax' or 'keymap' will accept?
pub(crate) fn valid_filetype(val: &CStr) -> bool {
    valid_name(val, b".-_")
}

/// [`check_signcolumn`] over the window's own 'signcolumn'.
pub(crate) fn check_window_signcolumn(window: Win) -> Result<(), Failed> {
    check_signcolumn(None, Some(window))
}

/// Parse 'signcolumn' and, given a window, store the width range it asks
/// for.
///
/// `scl` overrides the window's own value; no `window` only validates. The
/// two halves are separate grammars: everything the generated table lists
/// ("no", "yes", "yes:1".."yes:9", "auto", "auto:1".."auto:9", "number"),
/// and then the `auto:<min>-<max>` range, which the table cannot enumerate
/// and which is parsed by hand.
pub fn check_signcolumn(scl: Option<&[u8]>, window: Option<Win>) -> Result<(), Failed> {
    // The range is worked out while the value is borrowed -- from the
    // caller, or from the window itself -- and the window written after:
    // `None` for a value that is not good, `Some(None)` for one that is
    // with no window to store it in.
    let range = |val: &[u8]| -> Option<Option<(c_int, c_int)>> {
        if val.is_empty() {
            return None;
        }
        if opt_strings_ok(val, &opt_scl_values, false) {
            let Some(w) = window else {
                return Some(None);
            };
            // 'number' only wins when the window is actually showing numbers.
            let numbered = w.w_onebuf_opt.wo_nu != 0 || w.w_onebuf_opt.wo_rnu != 0;
            return Some(Some(match val {
                [b'n', b'o', ..] => (SCL_NO, SCL_NO),
                [b'n', b'u', ..] if numbered => (SCL_NUM, SCL_NUM),
                [b'y', b'e', b's', b':', n, ..] => (digit(*n), digit(*n)),
                [b'y', ..] => (1, 1),
                [b'a', b'u', b't', b'o', b':', n, ..] => (0, digit(*n)),
                _ => (0, 1),
            }));
        }
        // "auto:<min>-<max>", the one spelling the table cannot list.
        let [b'a', b'u', b't', b'o', b':', min, b'-', max] = val else {
            return None;
        };
        if !ascii_isdigit(c_int::from(*min)) || !ascii_isdigit(c_int::from(*max)) {
            return None;
        }
        let (min, max) = (digit(*min), digit(*max));
        if min < 1 || max < 2 || min > 8 || min >= max {
            return None;
        }
        Some(window.is_some().then_some((min, max)))
    };
    let found = match (scl, window) {
        (Some(scl), _) => range(scl),
        (None, Some(w)) => range(w.w_onebuf_opt.wo_scl.bytes()),
        (None, None) => range(b""),
    };
    let Some(found) = found else {
        return Err(Failed);
    };
    // A range is only worked out for a window.
    let (Some((min, max)), Some(mut window)) = (found, window) else {
        return Ok(());
    };
    window.w_minscwidth = min;
    window.w_maxscwidth = max;
    // Keep the width the window is currently drawing inside the new
    // range, without widening it on its own.
    let held = if min <= 0 {
        0
    } else {
        max.min(window.w_scwidth)
    };
    window.w_scwidth = min.max(held);
    Ok(())
}

/// One ASCII digit as its value. Every caller has already established that
/// the byte is a digit, or is in the half of 'signcolumn' the option table
/// vetted.
fn digit(byte: u8) -> c_int {
    c_int::from(byte) - c_int::from(b'0')
}

/// Check a 'statusline'-format value. Answers an untranslated message when
/// the format is bad.
///
/// Upstream formats the message into a function-local static, because the
/// answer has to outlive the call and the caller passes no buffer. The
/// answer is owned instead, so two checks in flight cannot collide.
///
/// `format` is the value's bytes; its end is where the C string's NUL was.
pub(crate) fn check_stl_option(format: &[u8]) -> Result<(), OptError> {
    let illegal = |c: c_int| Err(illegal_char(c));

    let mut rest = format;
    let mut groupdepth: c_int = 0;

    while let Some(at) = rest.iter().position(|&b| b == b'%') {
        // Past the `%`. The value may end here, in which case the item is
        // the terminator and the membership test below rejects it.
        rest = &rest[at + 1..];
        match rest.first() {
            // "%%", the truncation mark and the item separator take no
            // width, no precision and no argument.
            Some(&b'%' | &b'<' | &b'=') => {
                rest = &rest[1..];
                continue;
            }
            Some(&b')') => {
                rest = &rest[1..];
                groupdepth -= 1;
                if groupdepth < 0 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        // A minimum width, optionally left-aligned.
        rest = rest.strip_prefix(b"-").unwrap_or(rest);
        rest = &rest[rest.iter().take_while(|b| b.is_ascii_digit()).count()..];
        // A user highlight group takes the width as its number and stops
        // there.
        if rest.first() == Some(&b'*') {
            continue;
        }
        // A maximum width.
        if let Some(after) = rest.strip_prefix(b".") {
            rest = &after[after.iter().take_while(|b| b.is_ascii_digit()).count()..];
        }
        if rest.first() == Some(&b'(') {
            groupdepth += 1;
            continue;
        }
        let Some(&item) = rest.first() else {
            return illegal(NUL);
        };
        if StlOpt::from_byte(item).is_none() {
            return illegal(c_int::from(item));
        }
        if item == b'{' {
            rest = &rest[1..];
            // "%{%…%}" re-evaluates its result as another format, and its
            // terminator is "%}" rather than a bare "}".
            let reevaluate = rest.first() == Some(&b'%');
            if reevaluate {
                rest = &rest[1..];
                if rest.first() == Some(&b'}') {
                    return illegal(c_int::from(b'}'));
                }
            }
            let close = if reevaluate {
                rest.windows(2).position(|w| w == b"%}").map(|at| at + 1)
            } else {
                rest.iter().position(|&b| b == b'}')
            };
            let Some(close) = close else {
                return Err(e_unclosed_expression_sequence.into());
            };
            rest = &rest[close..];
        }
    }

    if groupdepth != 0 {
        return Err(e_unbalanced_groups.into());
    }
    Ok(())
}

/// Does `val` hold a character an option marked as a file or directory name
/// refuses? The set is wider while 'secure' is on.
pub fn check_illegal_path_names(val: &CStr, flags: uint32_t) -> bool {
    let val = val.to_bytes();
    let holds = |set: &[u8]| val.iter().any(|b| set.contains(b));
    (flags & kOptFlagNFname as uint32_t != 0
        && holds(if secure.get() != 0 {
            b"/\\*?[|;&<>\r\n"
        } else {
            b"/\\*?[<>\r\n"
        }))
        || (flags & kOptFlagNDname as uint32_t != 0 && holds(b"*?[|;&<>\r\n"))
}
