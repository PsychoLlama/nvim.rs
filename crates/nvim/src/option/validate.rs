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

use crate::message_fmt::msg_cstr;
use crate::option::B_IMODE_LAST;
use crate::tr;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::message::{E_INVARG, E_POSITIVE, E_SCROLL, E_WINHEIGHT, E_WINWIDTH};
use crate::option::defs::{MAX_MCO, OptError, OptIndex, OptVal, OptionSetFlags};
use crate::option::vars::{p_wh, p_wiw, p_wmh, p_wmw};
use crate::options::*;
use crate::os::cshim::gettext;
use crate::startup::FULL_SCREEN;
use crate::types::OptInt;
use crate::ui::state::ROWS;
use crate::window::{min_rows_for_all_tabpages, win_default_scroll};

use super::{
    INT_MAX, INT_MIN, MAX_NUMBERWIDTH, MIN_COLUMNS, SB_MAX, TABSTOP_MAX, get_option,
    get_option_unset_value, option_has_type, option_is_global_local, optval_type_name,
};

/// The two messages the quickfix-stack bounds report.
const E_QUICKFIX_TOO_FEW: &CStr =
    c"E1542: Cannot have a negative or zero number of quickfix/location lists";
const E_QUICKFIX_TOO_MANY: &CStr =
    c"E1543: Cannot have more than a hundred quickfix/location lists";

/// The most matches 'maxsearchcount' may ask the search-count display for.
const MAX_SEARCH_COUNT: c_int = 9999;

/// "E487: Argument must be positive", for a value below its floor.
fn too_small() -> OptError {
    E_POSITIVE.into()
}

/// "E474: Invalid argument", for a value the option cannot hold at all.
fn invalid() -> OptError {
    E_INVARG.into()
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
        OPT_LINES => {
            let least = min_rows_for_all_tabpages();
            if *newval < OptInt::from(least) && FULL_SCREEN.get() {
                errmsg = Err(OptError::formatted(tr!(
                    "E593: Need at least {least} lines"
                )));
                *newval = OptInt::from(least);
            }
            *newval = (*newval).min(OptInt::from(INT_MAX));
        }
        OPT_COLUMNS => {
            if *newval < OptInt::from(MIN_COLUMNS) && FULL_SCREEN.get() {
                errmsg = Err(OptError::formatted(tr!(
                    "E594: Need at least {MIN_COLUMNS} columns"
                )));
                *newval = OptInt::from(MIN_COLUMNS);
            }
            *newval = (*newval).min(OptInt::from(INT_MAX));
        }
        // 'pumblend' saturates silently rather than reporting.
        OPT_PUMBLEND => *newval = (*newval).clamp(0, 100),
        OPT_SCROLLJUMP => {
            if (*newval < -100 || *newval >= OptInt::from(ROWS.get())) && FULL_SCREEN.get() {
                errmsg = Err(E_SCROLL.into());
                *newval = 1;
            }
        }
        OPT_SCROLL => {
            let height = Win::current().w_view_height;
            if (*newval <= 0 || (*newval > OptInt::from(height) && height > 0)) && FULL_SCREEN.get()
            {
                // Zero is how `:set scroll=0` asks for the default, so
                // it is corrected without a message.
                if *newval != 0 {
                    errmsg = Err(E_SCROLL.into());
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
        OPT_HELPHEIGHT | OPT_TITLELEN | OPT_UPDATECOUNT | OPT_REPORT | OPT_UPDATETIME
        | OPT_SIDESCROLL | OPT_FOLDLEVEL | OPT_SHIFTWIDTH | OPT_TEXTWIDTH | OPT_WRITEDELAY
        | OPT_TIMEOUTLEN | OPT_CMDHEIGHT => bounded(value, 0, too_small(), OptInt::MAX, invalid()),
        OPT_CMDWINHEIGHT => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        // The four window-size options each cross-check their partner.
        OPT_WINHEIGHT if value >= 1 && p_wmh() > value => Err(E_WINHEIGHT.into()),
        OPT_WINHEIGHT => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        OPT_WINMINHEIGHT => bounded(value, 0, too_small(), p_wh(), E_WINHEIGHT.into()),
        OPT_WINWIDTH if value >= 1 && p_wmw() > value => Err(E_WINWIDTH.into()),
        OPT_WINWIDTH => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        OPT_WINMINWIDTH => bounded(value, 0, too_small(), p_wiw(), E_WINWIDTH.into()),
        // 'maxcombine' is fixed: whatever is asked for, this is the answer.
        OPT_MAXCOMBINE => {
            *newval = OptInt::from(MAX_MCO);
            Ok(())
        }
        OPT_HISTORY => bounded(value, 0, too_small(), 10000, invalid()),
        // 'pyxversion' only ever means Python 3; 0 asks for the default.
        OPT_PYXVERSION => match value {
            0 => {
                *newval = 3;
                Ok(())
            }
            3 => Ok(()),
            _ => Err(invalid()),
        },
        OPT_REGEXPENGINE => bounded(value, 0, invalid(), 2, invalid()),
        // The two offsets may be negative before the screen exists: that is
        // how a window says it does not override the global value.
        OPT_SCROLLOFF | OPT_SIDESCROLLOFF => {
            if value < 0 && FULL_SCREEN.get() {
                Err(too_small())
            } else {
                Ok(())
            }
        }
        OPT_CONCEALLEVEL => bounded(value, 0, too_small(), 3, invalid()),
        OPT_NUMBERWIDTH => bounded(
            value,
            1,
            too_small(),
            OptInt::from(MAX_NUMBERWIDTH),
            invalid(),
        ),
        OPT_IMINSERT => bounded(value, 0, invalid(), B_IMODE_LAST, invalid()),
        // 'imsearch' has one value more than 'iminsert': -1 is "follow it".
        OPT_IMSEARCH => bounded(value, -1, invalid(), B_IMODE_LAST, invalid()),
        // 'channel' is read-only; every value is refused.
        OPT_CHANNEL => Err(invalid()),
        OPT_SCROLLBACK => bounded(value, -1, invalid(), OptInt::from(SB_MAX), invalid()),
        OPT_TABSTOP => bounded(value, 1, too_small(), OptInt::from(TABSTOP_MAX), invalid()),
        OPT_CHISTORY | OPT_LHISTORY => bounded(
            value,
            1,
            E_QUICKFIX_TOO_FEW.into(),
            100,
            E_QUICKFIX_TOO_MANY.into(),
        ),
        OPT_MAXSEARCHCOUNT => bounded(
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
        let name = msg_cstr(opt.fullname);
        let want = msg_cstr(optval_type_name(opt.type_0));
        let got = msg_cstr(optval_type_name(newval.kind()));
        let rep = &*newval;
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
