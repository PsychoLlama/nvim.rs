//! Ex command and completion functions for command history
//!
//! ex_history (`:history` command), get_history_arg (completion helper)

use std::ffi::{c_char, c_int};

use crate::ffi::{self, ExargPtr, ExpandPtr};
use crate::helpers::HISTORY_NAMES;
use crate::{HIST_COUNT, HIST_INVALID};

extern "C" {
    // -- Message output --
    fn nvim_cmdhist_msg_puts_title(buf: *const c_char);
    fn nvim_cmdhist_msg_putchar(c: c_int);
    fn nvim_cmdhist_msg_outtrans(buf: *const c_char);
    fn nvim_cmdhist_message_filtered(s: *const c_char) -> c_int;

    // -- Display --
    fn nvim_cmdhist_get_Columns() -> c_int;
    fn nvim_cmdhist_get_got_int() -> c_int;
    fn nvim_cmdhist_vim_strsize(s: *const c_char) -> c_int;
    fn nvim_cmdhist_trunc_string(s: *const c_char, buf: *mut c_char, len: c_int, buflen: c_int);
    fn nvim_cmdhist_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);

    // -- Formatting (snprintf wrappers using IObuff) --
    fn nvim_cmdhist_format_hist_header(name: *const c_char) -> *mut c_char;
    fn nvim_cmdhist_format_hist_entry(is_current: c_int, hisnum: c_int) -> c_int;
    fn nvim_cmdhist_get_IObuff() -> *mut c_char;
    fn nvim_cmdhist_get_IOSIZE() -> c_int;

    // -- Parsing --
    fn nvim_cmdhist_get_list_range(
        end: *mut *mut c_char,
        val1: *mut c_int,
        val2: *mut c_int,
    ) -> c_int;

    // -- Errors --
    fn nvim_cmdhist_semsg_trailing_arg(s: *const c_char);
    fn nvim_cmdhist_semsg_val_too_large(s: *const c_char);
    fn nvim_cmdhist_msg_history_zero();

    // -- Exarg/expand accessors --
    fn nvim_cmdhist_eap_get_arg(eap: ExargPtr) -> *mut c_char;
    fn nvim_cmdhist_xp_buf_set(xp: ExpandPtr, idx: c_int, c: c_char);
    fn nvim_cmdhist_xp_buf_ptr(xp: ExpandPtr) -> *mut c_char;

    // -- Character classification --
    fn nvim_cmdhist_ascii_isdigit(c: c_int) -> c_int;
    fn nvim_cmdhist_ascii_isalpha(c: c_int) -> c_int;
}

/// Short names for history argument completion: `:=@>?/`
const SHORT_NAMES: &[u8] = b":=@>?/";

// =============================================================================
// get_history_arg
// =============================================================================

/// Function given to `ExpandGeneric()` to obtain the possible first
/// arguments of the `:history` command.
///
/// # Safety
/// `xp` must be a valid `expand_T *`. `idx` must be a non-negative index.
#[export_name = "get_history_arg"]
#[must_use]
pub unsafe extern "C" fn rs_get_history_arg(xp: ExpandPtr, idx: c_int) -> *mut c_char {
    let short_names_count = SHORT_NAMES.len() as c_int;
    // HISTORY_NAMES has 6 entries (5 names + 1 sentinel), so 5 valid names
    let history_name_count = (HISTORY_NAMES.len() - 1) as c_int;

    if idx < short_names_count {
        let c = SHORT_NAMES[idx as usize];
        nvim_cmdhist_xp_buf_set(xp, 0, c as c_char);
        nvim_cmdhist_xp_buf_set(xp, 1, 0);
        return nvim_cmdhist_xp_buf_ptr(xp);
    }
    if idx < short_names_count + history_name_count {
        let name_idx = (idx - short_names_count) as usize;
        return HISTORY_NAMES[name_idx].as_ptr().cast::<c_char>().cast_mut();
    }
    if idx == short_names_count + history_name_count {
        return c"all".as_ptr().cast_mut();
    }
    std::ptr::null_mut()
}

// =============================================================================
// ex_history
// =============================================================================

/// `:history` command - print a history
///
/// # Safety
/// `eap` must be a valid `exarg_T *`. Accesses C history arrays via FFI.
#[export_name = "ex_history"]
pub unsafe extern "C" fn rs_ex_history(eap: ExargPtr) {
    let hislen = ffi::nvim_get_hislen();
    if hislen == 0 {
        nvim_cmdhist_msg_history_zero();
        return;
    }

    let mut histype1: c_int = 0; // HIST_CMD
    let mut histype2: c_int = 0; // HIST_CMD
    let mut hisidx1: c_int = 1;
    let mut hisidx2: c_int = -1;

    let arg = nvim_cmdhist_eap_get_arg(eap);
    let mut end = arg;

    // Parse history type if not starting with digit, '-', or ','
    let first_byte = *arg;
    if nvim_cmdhist_ascii_isdigit(c_int::from(first_byte)) == 0
        && first_byte != b'-' as c_char
        && first_byte != b',' as c_char
    {
        end = arg;
        while nvim_cmdhist_ascii_isalpha(c_int::from(*end)) != 0
            || !ffi::nvim_cmdhist_vim_strchr(c":=@>/?".as_ptr(), c_int::from(*end as u8)).is_null()
        {
            end = end.add(1);
        }
        let len = end.offset_from(arg) as usize;
        histype1 = crate::helpers::rs_get_histtype(arg, len, 0);
        if histype1 == HIST_INVALID {
            if ffi::nvim_cmdhist_strnicmp(arg, c"all".as_ptr(), len) == 0 {
                histype1 = 0;
                histype2 = HIST_COUNT - 1;
            } else {
                nvim_cmdhist_semsg_trailing_arg(arg);
                return;
            }
        } else {
            histype2 = histype1;
        }
    }

    if nvim_cmdhist_get_list_range(&mut end, &mut hisidx1, &mut hisidx2) == 0 || *end != 0 {
        if *end != 0 {
            nvim_cmdhist_semsg_trailing_arg(end);
        } else {
            nvim_cmdhist_semsg_val_too_large(arg);
        }
        return;
    }

    while nvim_cmdhist_get_got_int() == 0 && histype1 <= histype2 {
        // Format header: "\n      #  <name> history"
        let name = HISTORY_NAMES[histype1 as usize].as_ptr().cast::<c_char>();
        let header_buf = nvim_cmdhist_format_hist_header(name);
        nvim_cmdhist_msg_puts_title(header_buf);

        let idx = *ffi::get_hisidx(histype1);
        let hist = ffi::get_histentry(histype1);

        let mut j = hisidx1;
        let mut k = hisidx2;
        if j < 0 {
            if -j > hislen {
                j = 0;
            } else {
                let slot = (hislen + j + idx + 1) % hislen;
                j = ffi::nvim_cmdhist_he_get_hisnum(ffi::nvim_cmdhist_he_at(hist, slot));
            }
        }
        if k < 0 {
            if -k > hislen {
                k = 0;
            } else {
                let slot = (hislen + k + idx + 1) % hislen;
                k = ffi::nvim_cmdhist_he_get_hisnum(ffi::nvim_cmdhist_he_at(hist, slot));
            }
        }

        if idx >= 0 && j <= k {
            let mut i = idx + 1;
            loop {
                if nvim_cmdhist_get_got_int() != 0 {
                    break;
                }
                if i == hislen {
                    i = 0;
                }
                let entry = ffi::nvim_cmdhist_he_at(hist, i);
                let hisstr = ffi::nvim_cmdhist_he_get_hisstr(entry);
                let hisnum = ffi::nvim_cmdhist_he_get_hisnum(entry);
                if !hisstr.is_null()
                    && hisnum >= j
                    && hisnum <= k
                    && nvim_cmdhist_message_filtered(hisstr) == 0
                {
                    nvim_cmdhist_msg_putchar(b'\n' as c_int);
                    let is_current = c_int::from(i == idx);
                    let len = nvim_cmdhist_format_hist_entry(is_current, hisnum);
                    let iobuff = nvim_cmdhist_get_IObuff();
                    let iosize = nvim_cmdhist_get_IOSIZE();
                    if nvim_cmdhist_vim_strsize(hisstr) > nvim_cmdhist_get_Columns() - 10 {
                        nvim_cmdhist_trunc_string(
                            hisstr,
                            iobuff.add(len as usize),
                            nvim_cmdhist_get_Columns() - 10,
                            iosize - len,
                        );
                    } else {
                        nvim_cmdhist_xstrlcpy(
                            iobuff.add(len as usize),
                            hisstr,
                            (iosize - len) as usize,
                        );
                    }
                    nvim_cmdhist_msg_outtrans(iobuff);
                }
                if i == idx {
                    break;
                }
                i += 1;
            }
        }

        histype1 += 1;
    }
}
