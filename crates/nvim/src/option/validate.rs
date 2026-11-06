//! Vetting a new value before anything is allowed to see it.
//!
//! Two rounds, and the difference between them matters.
//! [`validate_num_option`] rejects a number the option can never hold —
//! that is a user error and the set fails. [`check_num_option_bounds`] then
//! clamps the handful of options whose limit is the size of the screen
//! ('lines', 'columns', 'scroll', 'scrolljump', 'pumblend'): those report a
//! message *and* take the corrected value, because the screen can shrink
//! under a value that was legal when it was set.
//!
//! Every message comes back as a `*const c_char` the caller shows; a null
//! one means the value is good. The two that need a number in the text are
//! formatted into the caller's `errbuf`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::message_fmt::msg_cstr;
use crate::tr;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::message::{e_invarg, e_positive, e_scroll, e_winheight, e_winwidth};
use crate::option::vars::{p_wh, p_wiw, p_wmh, p_wmw};
use crate::options::*;
use crate::os::cshim::gettext;
use crate::startup::full_screen;
use crate::types::{MAX_MCO, OptError, OptIndex, OptInt, OptVal, OptionSetFlags};
use crate::ui::state::Rows;
use crate::window::{min_rows_for_all_tabpages, win_default_scroll};

use super::{
    INT_MAX, INT_MIN, MAX_NUMBERWIDTH, MIN_COLUMNS, SB_MAX, TABSTOP_MAX, get_option,
    get_option_unset_value, option_has_type, option_is_global_local, optval_to_text,
    optval_type_name,
};

/// The two messages the quickfix-stack bounds report.
const E_QUICKFIX_TOO_FEW: &CStr =
    c"E1542: Cannot have a negative or zero number of quickfix/location lists";
const E_QUICKFIX_TOO_MANY: &CStr =
    c"E1543: Cannot have more than a hundred quickfix/location lists";

/// The most matches 'maxsearchcount' may ask the search-count display for.
const MAX_SEARCH_COUNT: c_int = 9999;
/// The largest 'iminsert'/'imsearch' value: the language-mapping mode.
const B_IMODE_LAST: c_int = 1;

/// "E487: Argument must be positive", for a value below its floor.
fn too_small() -> OptError {
    e_positive.into()
}

/// "E474: Invalid argument", for a value the option cannot hold at all.
fn invalid() -> OptError {
    e_invarg.into()
}

/// The bound test almost every numeric option shares. `low` and `high` are
/// inclusive; each side names its own message, because "too small" is
/// usually E487 but is E474 for the options where a small value is not
/// wrong so much as meaningless.
fn bounded(
    value: OptInt,
    low: OptInt,
    below: OptError,
    high: OptInt,
    above: OptError,
) -> Result<(), OptError> {
    if value < low {
        Err(below)
    } else if value > high {
        Err(above)
    } else {
        Ok(())
    }
}

/// Clamp the options whose limit is the size of the screen. Unlike
/// [`validate_num_option`], a message here comes with a corrected value: it
/// is a warning, not a rejection.
///
pub(crate) fn check_num_option_bounds(
    opt_idx: OptIndex,
    newval: &mut OptInt,
) -> Result<(), OptError> {
    let mut errmsg = Ok(());
    match opt_idx {
        kOptLines => {
            let least = min_rows_for_all_tabpages();
            if *newval < OptInt::from(least) && full_screen.get() {
                errmsg = Err(OptError::formatted(tr!(
                    "E593: Need at least {least} lines"
                )));
                *newval = OptInt::from(least);
            }
            *newval = (*newval).min(OptInt::from(INT_MAX));
        }
        kOptColumns => {
            if *newval < OptInt::from(MIN_COLUMNS) && full_screen.get() {
                errmsg = Err(OptError::formatted(tr!(
                    "E594: Need at least {MIN_COLUMNS} columns"
                )));
                *newval = OptInt::from(MIN_COLUMNS);
            }
            *newval = (*newval).min(OptInt::from(INT_MAX));
        }
        // 'pumblend' saturates silently rather than reporting.
        kOptPumblend => *newval = (*newval).clamp(0, 100),
        kOptScrolljump => {
            if (*newval < -100 || *newval >= OptInt::from(Rows.get())) && full_screen.get() {
                errmsg = Err(e_scroll.into());
                *newval = 1;
            }
        }
        kOptScroll => {
            let height = Win::current().w_view_height;
            if (*newval <= 0 || (*newval > OptInt::from(height) && height > 0)) && full_screen.get()
            {
                // Zero is how `:set scroll=0` asks for the default, so
                // it is corrected without a message.
                if *newval != 0 {
                    errmsg = Err(e_scroll.into());
                }
                *newval = win_default_scroll(Win::current());
            }
        }
        _ => {}
    }
    errmsg
}

/// Reject a number the option can never hold. A message here fails the set;
/// `newval` is only written by the two options that answer a legal but
/// meaningless value with a fixed one.
///
pub(crate) fn validate_num_option(opt_idx: OptIndex, newval: &mut OptInt) -> Result<(), OptError> {
    let value = *newval;
    // Every numeric option ends up in an `int` somewhere down the line.
    if value < OptInt::from(INT_MIN) || value > OptInt::from(INT_MAX) {
        return Err(invalid());
    }
    let errmsg = match opt_idx {
        kOptHelpheight | kOptTitlelen | kOptUpdatecount | kOptReport | kOptUpdatetime
        | kOptSidescroll | kOptFoldlevel | kOptShiftwidth | kOptTextwidth | kOptWritedelay
        | kOptTimeoutlen | kOptCmdheight => bounded(value, 0, too_small(), OptInt::MAX, invalid()),
        kOptCmdwinheight => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        // The four window-size options each cross-check their partner.
        kOptWinheight if value >= 1 && p_wmh() > value => Err(e_winheight.into()),
        kOptWinheight => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        kOptWinminheight => bounded(value, 0, too_small(), p_wh(), e_winheight.into()),
        kOptWinwidth if value >= 1 && p_wmw() > value => Err(e_winwidth.into()),
        kOptWinwidth => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        kOptWinminwidth => bounded(value, 0, too_small(), p_wiw(), e_winwidth.into()),
        // 'maxcombine' is fixed: whatever is asked for, this is the answer.
        kOptMaxcombine => {
            *newval = OptInt::from(MAX_MCO);
            Ok(())
        }
        kOptHistory => bounded(value, 0, too_small(), 10000, invalid()),
        // 'pyxversion' only ever means Python 3; 0 asks for the default.
        kOptPyxversion => match value {
            0 => {
                *newval = 3;
                Ok(())
            }
            3 => Ok(()),
            _ => Err(invalid()),
        },
        kOptRegexpengine => bounded(value, 0, invalid(), 2, invalid()),
        // The two offsets may be negative before the screen exists: that is
        // how a window says it does not override the global value.
        kOptScrolloff | kOptSidescrolloff => {
            if value < 0 && full_screen.get() {
                Err(too_small())
            } else {
                Ok(())
            }
        }
        kOptConceallevel => bounded(value, 0, too_small(), 3, invalid()),
        kOptNumberwidth => bounded(
            value,
            1,
            too_small(),
            OptInt::from(MAX_NUMBERWIDTH),
            invalid(),
        ),
        kOptIminsert => bounded(value, 0, invalid(), OptInt::from(B_IMODE_LAST), invalid()),
        // 'imsearch' has one value more than 'iminsert': -1 is "follow it".
        kOptImsearch => bounded(value, -1, invalid(), OptInt::from(B_IMODE_LAST), invalid()),
        // 'channel' is read-only; every value is refused.
        kOptChannel => Err(invalid()),
        kOptScrollback => bounded(value, -1, invalid(), OptInt::from(SB_MAX), invalid()),
        kOptTabstop => bounded(value, 1, too_small(), OptInt::from(TABSTOP_MAX), invalid()),
        kOptChistory | kOptLhistory => bounded(
            value,
            1,
            E_QUICKFIX_TOO_FEW.into(),
            100,
            E_QUICKFIX_TOO_MANY.into(),
        ),
        kOptMaxsearchcount => bounded(
            value,
            1,
            too_small(),
            OptInt::from(MAX_SEARCH_COUNT),
            invalid(),
        ),
        _ => Ok(()),
    };
    errmsg?;
    check_num_option_bounds(opt_idx, newval)
}

/// Vet a whole value: the right type for the option, and within bounds if it
/// is a number. `newval` may be rewritten — an unset value becomes the
/// option's "not set here" sentinel, and a number may be clamped.
///
pub(crate) fn validate_option_value(
    opt_idx: OptIndex,
    newval: &mut OptVal,
    opt_flags: OptionSetFlags,
) -> Result<(), OptError> {
    // `:setlocal` writing a global-local option's sentinel is how it is
    // unset; nothing else needs to look at the value.
    if option_is_global_local(opt_idx)
        && opt_flags.has(OptionSetFlags::LOCAL)
        && *newval == get_option_unset_value(opt_idx)
    {
        return Ok(());
    }
    let opt = get_option(opt_idx);
    if newval.is_nil() {
        // A global value has no "unset" state to fall back to.
        if opt_flags == OptionSetFlags::GLOBAL {
            return Err(gettext(c"Cannot unset global option value").into());
        }
        *newval = get_option_unset_value(opt_idx);
        Ok(())
    } else if !option_has_type(opt_idx, newval.kind()) {
        let rep = optval_to_text(newval);
        let name = msg_cstr(opt.fullname);
        let want = msg_cstr(optval_type_name(opt.type_0));
        let got = msg_cstr(optval_type_name(newval.kind()));
        let rep = msg_cstr(rep.as_cstr());
        Err(OptError::formatted(tr!(
            "Invalid value for option '{name}': expected {want}, got {got} {rep}"
        )))
    } else if let OptVal::Number(number) = newval {
        // The check clamps in place, so it is handed the value's own word.
        validate_num_option(opt_idx, number)
    } else {
        Ok(())
    }
}
