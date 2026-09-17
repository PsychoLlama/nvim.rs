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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::winlayer::Win;
use core::ffi::{CStr, c_int, c_void};

use crate::memory::{XString, xfree};
use crate::message::{e_invarg, e_positive, e_scroll, e_winheight, e_winwidth};
use crate::option::vars::{p_wh, p_wiw, p_wmh, p_wmw};
use crate::options::*;
use crate::os::cshim::{gettext, snprintf};
use crate::startup::full_screen;
use crate::strings::vim_snprintf;
use crate::types::{MAX_MCO, OptError, OptIndex, OptInt, OptVal, OptionSetFlags, size_t};
use crate::ui::state::Rows;
use crate::window::{min_rows_for_all_tabpages, win_default_scroll};

use super::{
    INT_MAX, INT_MIN, MAX_NUMBERWIDTH, MIN_COLUMNS, SB_MAX, TABSTOP_MAX, get_option,
    get_option_unset_value, option_has_type, option_is_global_local, optval_copy, optval_equal,
    optval_to_cstr, optval_type_name,
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
    // SAFETY (both `vim_snprintf`s): `message` is the buffer the formatter
    // is told the size of, and `curwin` is live.
    match opt_idx {
        kOptLines => {
            let least = min_rows_for_all_tabpages();
            if *newval < least as OptInt && full_screen.get() {
                let fmt = gettext(c"E593: Need at least %d lines");
                let message = XString::filled(OptError::ROOM, |buf| unsafe {
                    vim_snprintf(buf, OptError::ROOM as size_t, fmt.as_ptr(), least);
                });
                errmsg = Err(message.into());
                *newval = least as OptInt;
            }
            *newval = (*newval).min(INT_MAX as OptInt);
        }
        kOptColumns => {
            if *newval < MIN_COLUMNS as OptInt && full_screen.get() {
                let fmt = gettext(c"E594: Need at least %d columns");
                let message = XString::filled(OptError::ROOM, |buf| unsafe {
                    vim_snprintf(
                        buf,
                        OptError::ROOM as size_t,
                        fmt.as_ptr(),
                        MIN_COLUMNS as c_int,
                    );
                });
                errmsg = Err(message.into());
                *newval = MIN_COLUMNS as OptInt;
            }
            *newval = (*newval).min(INT_MAX as OptInt);
        }
        // 'pumblend' saturates silently rather than reporting.
        kOptPumblend => *newval = (*newval).clamp(0, 100),
        kOptScrolljump => {
            if (*newval < -100 || *newval >= Rows.get() as OptInt) && full_screen.get() {
                errmsg = Err(e_scroll.into());
                *newval = 1;
            }
        }
        kOptScroll => {
            let height = Win::current().w_view_height;
            if (*newval <= 0 || (*newval > height as OptInt && height > 0)) && full_screen.get() {
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
    if value < INT_MIN as OptInt || value > INT_MAX as OptInt {
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
            *newval = MAX_MCO as OptInt;
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
        kOptNumberwidth => bounded(value, 1, too_small(), MAX_NUMBERWIDTH as OptInt, invalid()),
        kOptIminsert => bounded(value, 0, invalid(), B_IMODE_LAST as OptInt, invalid()),
        // 'imsearch' has one value more than 'iminsert': -1 is "follow it".
        kOptImsearch => bounded(value, -1, invalid(), B_IMODE_LAST as OptInt, invalid()),
        // 'channel' is read-only; every value is refused.
        kOptChannel => Err(invalid()),
        kOptScrollback => bounded(value, -1, invalid(), SB_MAX as OptInt, invalid()),
        kOptTabstop => bounded(value, 1, too_small(), TABSTOP_MAX as OptInt, invalid()),
        kOptChistory | kOptLhistory => bounded(
            value,
            1,
            E_QUICKFIX_TOO_FEW.into(),
            100,
            E_QUICKFIX_TOO_MANY.into(),
        ),
        kOptMaxsearchcount => bounded(value, 1, too_small(), MAX_SEARCH_COUNT as OptInt, invalid()),
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
        && optval_equal(newval, &get_option_unset_value(opt_idx))
    {
        return Ok(());
    }
    let opt = get_option(opt_idx);
    if newval.is_nil() {
        // A global value has no "unset" state to fall back to.
        if opt_flags == OptionSetFlags::GLOBAL {
            return Err(gettext(c"Cannot unset global option value").into());
        }
        *newval = optval_copy(&get_option_unset_value(opt_idx));
        Ok(())
    } else if !option_has_type(opt_idx, newval.kind()) {
        let rep = optval_to_cstr(newval);
        let fmt = c"Invalid value for option '%s': expected %s, got %s %s";
        let fmt = gettext(fmt);
        let want = optval_type_name(opt.type_0).as_ptr();
        let got = optval_type_name(newval.kind()).as_ptr();
        let name = opt.fullname;
        // SAFETY: `message` is the buffer the formatter is told the size
        // of, and every argument is a NUL-terminated string.
        let message = XString::filled(OptError::ROOM, |buf| unsafe {
            snprintf(
                buf,
                OptError::ROOM as size_t,
                fmt.as_ptr(),
                name,
                want,
                got,
                rep,
            );
        });
        // SAFETY: `optval_to_cstr` answered an allocation this owns.
        unsafe { xfree(rep.cast::<c_void>()) };
        Err(message.into())
    } else if let OptVal::Number(number) = newval {
        // The check clamps in place, so it is handed the value's own word.
        validate_num_option(opt_idx, number)
    } else {
        Ok(())
    }
}
