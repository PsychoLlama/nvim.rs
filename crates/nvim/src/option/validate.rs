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

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::main::{
    Rows, curwin, e_invarg, e_positive, e_scroll, e_winheight, e_winwidth, full_screen, p_wh,
    p_wiw, p_wmh, p_wmw,
};
use crate::memory::xfree;
use crate::options::*;
use crate::os::cshim::{gettext, snprintf};
use crate::strings::vim_snprintf;
use crate::types::{
    IOSIZE, MAX_MCO, OptIndex, OptInt, OptVal, OptionSetFlags, size_t, vimoption_T,
};
use crate::window::{min_rows_for_all_tabpages, win_default_scroll};

use super::{
    INT_MAX, INT_MIN, MAX_NUMBERWIDTH, MIN_COLUMNS, SB_MAX, TABSTOP_MAX, get_option_unset_value,
    kOptValTypeNil, kOptValTypeNumber, option_has_type, option_is_global_local, optval_copy,
    optval_equal, optval_to_cstr, optval_type_name,
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
fn too_small() -> *const c_char {
    e_positive.as_ptr()
}

/// "E474: Invalid argument", for a value the option cannot hold at all.
fn invalid() -> *const c_char {
    e_invarg.as_ptr()
}

/// The bound test almost every numeric option shares. `low` and `high` are
/// inclusive; each side names its own message, because "too small" is
/// usually E487 but is E474 for the options where a small value is not
/// wrong so much as meaningless.
fn bounded(
    value: OptInt,
    low: OptInt,
    below: *const c_char,
    high: OptInt,
    above: *const c_char,
) -> *const c_char {
    if value < low {
        below
    } else if value > high {
        above
    } else {
        ptr::null()
    }
}

/// Clamp the options whose limit is the size of the screen. Unlike
/// [`validate_num_option`], a message here comes with a corrected value: it
/// is a warning, not a rejection.
///
/// # Safety
///
/// `errbuf` must have room for `errbuflen` bytes.
pub(crate) unsafe fn check_num_option_bounds(
    opt_idx: OptIndex,
    newval: &mut OptInt,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ptr::null();
    // SAFETY: the caller's `errbuf` has room for `errbuflen` bytes, and
    // `curwin` is live.
    unsafe {
        match opt_idx {
            kOptLines => {
                let least = min_rows_for_all_tabpages();
                if *newval < least as OptInt && full_screen.get() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        gettext(c"E593: Need at least %d lines".as_ptr()),
                        least,
                    );
                    errmsg = errbuf;
                    *newval = least as OptInt;
                }
                *newval = (*newval).min(INT_MAX as OptInt);
            }
            kOptColumns => {
                if *newval < MIN_COLUMNS as OptInt && full_screen.get() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        gettext(c"E594: Need at least %d columns".as_ptr()),
                        MIN_COLUMNS as c_int,
                    );
                    errmsg = errbuf;
                    *newval = MIN_COLUMNS as OptInt;
                }
                *newval = (*newval).min(INT_MAX as OptInt);
            }
            // 'pumblend' saturates silently rather than reporting.
            kOptPumblend => *newval = (*newval).clamp(0, 100),
            kOptScrolljump => {
                if (*newval < -100 || *newval >= Rows.get() as OptInt) && full_screen.get() {
                    errmsg = e_scroll.as_ptr();
                    *newval = 1;
                }
            }
            kOptScroll => {
                let height = (*curwin.get()).w_view_height;
                if (*newval <= 0 || (*newval > height as OptInt && height > 0)) && full_screen.get()
                {
                    // Zero is how `:set scroll=0` asks for the default, so
                    // it is corrected without a message.
                    if *newval != 0 {
                        errmsg = e_scroll.as_ptr();
                    }
                    *newval = win_default_scroll(curwin.get());
                }
            }
            _ => {}
        }
    }
    errmsg
}

/// Reject a number the option can never hold. A message here fails the set;
/// `newval` is only written by the two options that answer a legal but
/// meaningless value with a fixed one.
///
/// # Safety
///
/// `errbuf` must have room for `errbuflen` bytes.
pub(crate) unsafe fn validate_num_option(
    opt_idx: OptIndex,
    newval: &mut OptInt,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    let value = *newval;
    // Every numeric option ends up in an `int` somewhere down the line.
    if value < INT_MIN as OptInt || value > INT_MAX as OptInt {
        return invalid();
    }
    let errmsg = match opt_idx {
        kOptHelpheight | kOptTitlelen | kOptUpdatecount | kOptReport | kOptUpdatetime
        | kOptSidescroll | kOptFoldlevel | kOptShiftwidth | kOptTextwidth | kOptWritedelay
        | kOptTimeoutlen | kOptCmdheight => bounded(value, 0, too_small(), OptInt::MAX, invalid()),
        kOptCmdwinheight => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        // The four window-size options each cross-check their partner.
        kOptWinheight if value >= 1 && p_wmh.get() > value => e_winheight.as_ptr(),
        kOptWinheight => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        kOptWinminheight => bounded(value, 0, too_small(), p_wh.get(), e_winheight.as_ptr()),
        kOptWinwidth if value >= 1 && p_wmw.get() > value => e_winwidth.as_ptr(),
        kOptWinwidth => bounded(value, 1, too_small(), OptInt::MAX, invalid()),
        kOptWinminwidth => bounded(value, 0, too_small(), p_wiw.get(), e_winwidth.as_ptr()),
        // 'maxcombine' is fixed: whatever is asked for, this is the answer.
        kOptMaxcombine => {
            *newval = MAX_MCO as OptInt;
            ptr::null()
        }
        kOptHistory => bounded(value, 0, too_small(), 10000, invalid()),
        // 'pyxversion' only ever means Python 3; 0 asks for the default.
        kOptPyxversion => match value {
            0 => {
                *newval = 3;
                ptr::null()
            }
            3 => ptr::null(),
            _ => invalid(),
        },
        kOptRegexpengine => bounded(value, 0, invalid(), 2, invalid()),
        // The two offsets may be negative before the screen exists: that is
        // how a window says it does not override the global value.
        kOptScrolloff | kOptSidescrolloff => {
            if value < 0 && full_screen.get() {
                too_small()
            } else {
                ptr::null()
            }
        }
        kOptConceallevel => bounded(value, 0, too_small(), 3, invalid()),
        kOptNumberwidth => bounded(value, 1, too_small(), MAX_NUMBERWIDTH as OptInt, invalid()),
        kOptIminsert => bounded(value, 0, invalid(), B_IMODE_LAST as OptInt, invalid()),
        // 'imsearch' has one value more than 'iminsert': -1 is "follow it".
        kOptImsearch => bounded(value, -1, invalid(), B_IMODE_LAST as OptInt, invalid()),
        // 'channel' is read-only; every value is refused.
        kOptChannel => invalid(),
        kOptScrollback => bounded(value, -1, invalid(), SB_MAX as OptInt, invalid()),
        kOptTabstop => bounded(value, 1, too_small(), TABSTOP_MAX as OptInt, invalid()),
        kOptChistory | kOptLhistory => bounded(
            value,
            1,
            E_QUICKFIX_TOO_FEW.as_ptr(),
            100,
            E_QUICKFIX_TOO_MANY.as_ptr(),
        ),
        kOptMaxsearchcount => bounded(value, 1, too_small(), MAX_SEARCH_COUNT as OptInt, invalid()),
        _ => ptr::null(),
    };
    if !errmsg.is_null() {
        return errmsg;
    }
    // SAFETY: the caller's `errbuf` has room for `errbuflen` bytes.
    unsafe { check_num_option_bounds(opt_idx, newval, errbuf, errbuflen) }
}

/// Vet a whole value: the right type for the option, and within bounds if it
/// is a number. `newval` may be rewritten — an unset value becomes the
/// option's "not set here" sentinel, and a number may be clamped.
///
/// # Safety
///
/// `errbuf` must have room for `IOSIZE` bytes, which is what the
/// type-mismatch message writes regardless of `errbuflen`.
pub(crate) unsafe fn validate_option_value(
    opt_idx: OptIndex,
    newval: &mut OptVal,
    opt_flags: OptionSetFlags,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    // SAFETY: the caller's `errbuf` has room, and the option table is a
    // plain array.
    unsafe {
        // `:setlocal` writing a global-local option's sentinel is how it is
        // unset; nothing else needs to look at the value.
        if option_is_global_local(opt_idx)
            && opt_flags.has(OptionSetFlags::LOCAL)
            && optval_equal(*newval, get_option_unset_value(opt_idx))
        {
            return ptr::null();
        }
        let opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        if newval.type_0 == kOptValTypeNil {
            // A global value has no "unset" state to fall back to.
            if opt_flags == OptionSetFlags::GLOBAL {
                return gettext(c"Cannot unset global option value".as_ptr());
            }
            *newval = optval_copy(get_option_unset_value(opt_idx));
            ptr::null()
        } else if !option_has_type(opt_idx, newval.type_0) {
            let rep = optval_to_cstr(*newval);
            snprintf(
                errbuf,
                IOSIZE as size_t,
                gettext(c"Invalid value for option '%s': expected %s, got %s %s".as_ptr()),
                (*opt).fullname,
                optval_type_name((*opt).type_0).as_ptr(),
                optval_type_name(newval.type_0).as_ptr(),
                rep,
            );
            xfree(rep.cast::<c_void>());
            errbuf
        } else if newval.type_0 == kOptValTypeNumber {
            validate_num_option(opt_idx, &mut newval.data.number, errbuf, errbuflen)
        } else {
            ptr::null()
        }
    }
}
