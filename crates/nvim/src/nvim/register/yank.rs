//! Yanking text into a register.
//!
//! [`op_yank_reg`] is the whole of `y`: it copies the operator's region into a
//! `yankreg_T` line by line. The blockwise case goes through `block_prep` per
//! line, so that a short line is padded out and a tab straddling the edge of
//! the block is split into spaces; the charwise case is the same walk with
//! `charwise_block_prep`, which is how the first and last lines of a charwise
//! yank get their partial extents.
//!
//! [`format_reg_type`] renders the `v` / `V` / `CTRL-V width` string the API
//! and `:registers` both show, and [`do_autocmd_textyankpost`] builds the
//! `v:event` dictionary TextYankPost sees.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::smsg_c;
use ::core::ffi::{c_char, c_int, c_ulong, c_void};

use super::*;

/// Copy one line of a block into slot `y_idx` of `reg`, padding both ends
/// with the spaces `block_prep` measured.
///
/// With `exclude_trailing_space` -- 'formatoptions' `y`'s blockwise yank --
/// the padding at the end is dropped and any white space the text itself ends
/// with is trimmed off.
///
/// # Safety
/// `bd` must describe a region of the current line, and `reg.y_array` hold at
/// least `y_idx + 1` slots.
unsafe fn yank_copy_line(
    reg: *mut yankreg_T,
    bd: *mut block_def,
    y_idx: size_t,
    exclude_trailing_space: bool,
) {
    unsafe {
        if exclude_trailing_space {
            (*bd).endspaces = 0;
        }
        let size = (*bd).startspaces + (*bd).endspaces + (*bd).textlen;
        debug_assert!(size >= 0);
        let start = xmallocz(size as size_t) as *mut c_char;
        (*(*reg).y_array.add(y_idx)).data = start;

        let mut pnew = start;
        memset(
            pnew as *mut c_void,
            ' ' as c_int,
            (*bd).startspaces as size_t,
        );
        pnew = pnew.offset((*bd).startspaces as isize);
        memmove(
            pnew as *mut c_void,
            (*bd).textstart as *const c_void,
            (*bd).textlen as size_t,
        );
        pnew = pnew.offset((*bd).textlen as isize);
        memset(pnew as *mut c_void, ' ' as c_int, (*bd).endspaces as size_t);
        pnew = pnew.offset((*bd).endspaces as isize);

        if exclude_trailing_space {
            // Walk back over the trailing white space, a character at a time
            // so that a multi-byte character is not cut in half.
            let mut s = (*bd).textlen + (*bd).endspaces;
            while s > 0 && ascii_iswhite(c_int::from(*(*bd).textstart.offset((s - 1) as isize))) {
                s -= utf_head_off((*bd).textstart, (*bd).textstart.offset((s - 1) as isize)) + 1;
                pnew = pnew.offset(-1);
            }
        }
        *pnew = NUL as c_char;
        (*(*reg).y_array.add(y_idx)).size = pnew.offset_from(start) as size_t;
    }
}

/// Move the lines of `reg` onto the end of `curr`, and free `reg`'s array.
///
/// This is what an uppercase register name does. A charwise append joins the
/// last old line and the first new one into a single line, unless 'cpoptions'
/// has `>`.
///
/// # Safety
/// Both registers must own their arrays; `reg` must hold at least one line
/// when the charwise join runs.
unsafe fn append_to_register(curr: *mut yankreg_T, reg: *mut yankreg_T, yank_type: MotionType) {
    unsafe {
        let new_ptr = xmalloc(
            ::core::mem::size_of::<String_0>()
                .wrapping_mul((*curr).y_size.wrapping_add((*reg).y_size)),
        ) as *mut String_0;
        let mut j: size_t = 0;
        while j < (*curr).y_size {
            *new_ptr.add(j) = *(*curr).y_array.add(j);
            j = j.wrapping_add(1);
        }
        xfree((*curr).y_array as *mut c_void);
        (*curr).y_array = new_ptr;

        // Appending linewise text makes the whole register linewise.
        if yank_type == kMTLineWise {
            (*curr).y_type = kMTLineWise;
        }

        let mut y_idx: size_t = 0;
        if (*curr).y_type == kMTCharWise && vim_strchr(p_cpo.get(), CPO_REGAPPEND).is_null() {
            // Join the last old line and the first new one.
            let first_new = *(*reg).y_array;
            j = j.wrapping_sub(1);
            let last_old = &mut *(*curr).y_array.add(j);
            let joined_size = last_old.size.wrapping_add(first_new.size);
            let pnew = xmalloc(joined_size.wrapping_add(1)) as *mut c_char;
            strcpy(pnew, last_old.data);
            strcpy(pnew.add(last_old.size), first_new.data);
            xfree(last_old.data as *mut c_void);
            *last_old = String_0 {
                data: pnew,
                size: joined_size,
            };
            j = j.wrapping_add(1);

            xfree(first_new.data as *mut c_void);
            (*(*reg).y_array).data = ::core::ptr::null_mut();
            (*(*reg).y_array).size = 0;
            y_idx = 1;
        }

        while y_idx < (*reg).y_size {
            *(*curr).y_array.add(j) = *(*reg).y_array.add(y_idx);
            y_idx = y_idx.wrapping_add(1);
            j = j.wrapping_add(1);
        }
        (*curr).y_size = j;
        xfree((*reg).y_array as *mut c_void);
    }
}

/// The "N lines yanked" message.
///
/// # Safety
/// `oap` must be the operator that was just applied.
unsafe fn report_yank(oap: *mut oparg_T, yank_type: MotionType, yanklines: size_t) {
    unsafe {
        let mut namebuf: [c_char; 100] = [0; 100];
        if (*oap).regname == NUL {
            namebuf[0] = NUL as c_char;
        } else {
            vim_snprintf(
                namebuf.as_mut_ptr(),
                namebuf.len(),
                gettext(c" into \"%c".as_ptr()),
                (*oap).regname,
            );
        }

        // The message may be the first thing that scrolls, so make sure the
        // window is up to date before it is written.
        update_topline(curwin.get());
        if must_redraw.get() != 0 {
            update_screen();
        }

        let (one, many) = if yank_type == kMTBlockWise {
            (
                c"block of %ld line yanked%s".as_ptr(),
                c"block of %ld lines yanked%s".as_ptr(),
            )
        } else {
            (
                c"%ld line yanked%s".as_ptr(),
                c"%ld lines yanked%s".as_ptr(),
            )
        };
        smsg_c!(
            0,
            ngettext(one, many, yanklines as c_ulong),
            yanklines as int64_t,
            namebuf.as_mut_ptr(),
        );
    }
}

/// Yank the operator's region into `reg`.
///
/// With `append`, the new text goes onto the end of whatever `reg` already
/// holds; with `message`, the "N lines yanked" report is given.
///
/// # Safety
/// `oap` must describe a region of the current buffer and `reg` be a live
/// register.
pub unsafe fn op_yank_reg(oap: *mut oparg_T, message: bool, mut reg: *mut yankreg_T, append: bool) {
    unsafe {
        let mut newreg = EMPTY_YANKREG;
        let mut yank_type = (*oap).motion_type;
        let mut yanklines = (*oap).line_count as size_t;
        let mut yankendlnum = (*oap).end.lnum;
        let mut bd = block_def {
            startspaces: 0,
            endspaces: 0,
            textlen: 0,
            textstart: ::core::ptr::null_mut(),
            textcol: 0,
            start_vcol: 0,
            end_vcol: 0,
            is_short: 0,
            is_MAX: 0,
            is_oneChar: 0,
            pre_whitesp: 0,
            pre_whitesp_c: 0,
            end_char_vcols: 0,
            start_char_vcols: 0,
        };

        // Appending yanks into a scratch register first, then merges.
        let curr = reg;
        if append && !(*reg).y_array.is_null() {
            reg = &raw mut newreg;
        } else {
            free_register(reg);
        }

        // A charwise yank that starts in column 0 and ends before column 0 of
        // a later line is really a linewise one.
        if (*oap).motion_type == kMTCharWise
            && (*oap).start.col == 0
            && !(*oap).inclusive
            && (!(*oap).is_VIsual || c_int::from(*p_sel.get()) == 'o' as c_int)
            && (*oap).end.col == 0
            && yanklines > 1
        {
            yank_type = kMTLineWise;
            yankendlnum -= 1;
            yanklines = yanklines.wrapping_sub(1);
        }

        (*reg).y_size = yanklines;
        (*reg).y_type = yank_type;
        (*reg).y_width = 0;
        (*reg).y_array = xcalloc(yanklines, ::core::mem::size_of::<String_0>()) as *mut String_0;
        (*reg).additional_data = ::core::ptr::null_mut();
        (*reg).timestamp = os_time();

        if yank_type == kMTBlockWise {
            (*reg).y_width = (*oap).end_vcol - (*oap).start_vcol;
            // A `$`-extended block has no fixed width.
            if (*curwin.get()).w_curswant == MAXCOL && (*reg).y_width > 0 {
                (*reg).y_width -= 1;
            }
        }

        let mut y_idx: size_t = 0;
        let mut lnum = (*oap).start.lnum;
        while lnum <= yankendlnum {
            match (*reg).y_type {
                kMTBlockWise => {
                    block_prep(oap, &raw mut bd, lnum, false);
                    yank_copy_line(reg, &raw mut bd, y_idx, (*oap).excl_tr_ws);
                }
                kMTLineWise => {
                    *(*reg).y_array.add(y_idx) =
                        cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
                }
                kMTCharWise => {
                    charwise_block_prep(
                        (*oap).start,
                        (*oap).end,
                        &raw mut bd,
                        lnum,
                        (*oap).inclusive,
                    );
                    // The region may reach past the end of a short line.
                    let tmp = strlen(bd.textstart) as c_int;
                    if tmp < bd.textlen {
                        bd.textlen = tmp;
                    }
                    yank_copy_line(reg, &raw mut bd, y_idx, false);
                }
                kMTUnknown => abort(),
                _ => {}
            }
            lnum += 1;
            y_idx = y_idx.wrapping_add(1);
        }

        if curr != reg {
            append_to_register(curr, reg, yank_type);
        }

        if message {
            // A single charwise line is not worth reporting.
            if yank_type == kMTCharWise && yanklines == 1 {
                yanklines = 0;
            }
            if yanklines > p_report.get() as size_t {
                report_yank(oap, yank_type, yanklines);
            }
        }

        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
            if yank_type == kMTLineWise {
                (*curbuf.get()).b_op_start.col = 0;
                (*curbuf.get()).b_op_end.col = MAXCOL;
            }
            if yank_type != kMTLineWise && !(*oap).inclusive {
                // An exclusive region's `']` is the character *before* the end.
                decl(&raw mut (*curbuf.get()).b_op_end);
            }
        }
    }
}

/// Render a register's type as the string `getregtype()` and `:registers`
/// show: `v`, `V`, or `CTRL-V` followed by the block width.
///
/// # Safety
/// `buf` must hold at least `buf_len` bytes, and `buf_len` be more than 1.
pub unsafe fn format_reg_type(
    reg_type: MotionType,
    reg_width: colnr_T,
    buf: *mut c_char,
    buf_len: size_t,
) {
    unsafe {
        debug_assert!(buf_len > 1);
        match reg_type {
            kMTLineWise => {
                *buf = 'V' as c_char;
                *buf.add(1) = NUL as c_char;
            }
            kMTCharWise => {
                *buf = 'v' as c_char;
                *buf.add(1) = NUL as c_char;
            }
            kMTBlockWise => {
                snprintf(buf, buf_len, c"\x16%d".as_ptr(), reg_width + 1);
            }
            kMTUnknown => {
                *buf = NUL as c_char;
            }
            _ => {}
        }
    }
}

/// Fire TextYankPost with the `v:event` dictionary describing the yank.
///
/// Guarded against recursion: an autocommand that yanks would otherwise fire
/// this again.
///
/// # Safety
/// `oap` and `reg` must describe the yank that just happened. Runs arbitrary
/// autocommands, under `textlock`.
pub unsafe fn do_autocmd_textyankpost(oap: *mut oparg_T, reg: *mut yankreg_T) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false);

        if recursive.get() || !has_event(EVENT_TEXTYANKPOST) {
            return;
        }
        recursive.set(true);

        let mut save_v_event = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut(),
                }; 16],
            },
        };
        let dict = get_v_event(&raw mut save_v_event);

        let list = tv_list_alloc((*reg).y_size as ptrdiff_t);
        for i in 0..(*reg).y_size {
            let line = *(*reg).y_array.add(i);
            tv_list_append_string(list, line.data, line.size as c_int as ssize_t);
        }
        tv_list_set_lock(list, VAR_FIXED);
        tv_dict_add_list(dict, c"regcontents".as_ptr(), 11, list);

        let mut buf: [c_char; 67] = [0; 67];
        format_reg_type((*reg).y_type, (*reg).y_width, buf.as_mut_ptr(), buf.len());
        tv_dict_add_str(dict, c"regtype".as_ptr(), 7, buf.as_mut_ptr());

        buf[0] = (*oap).regname as c_char;
        buf[1] = NUL as c_char;
        tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_mut_ptr());

        tv_dict_add_bool(
            dict,
            c"inclusive".as_ptr(),
            9,
            if (*oap).inclusive {
                kBoolVarTrue
            } else {
                kBoolVarFalse
            },
        );

        buf[0] = get_op_char((*oap).op_type) as c_char;
        buf[1] = NUL as c_char;
        tv_dict_add_str(dict, c"operator".as_ptr(), 8, buf.as_mut_ptr());

        tv_dict_add_bool(
            dict,
            c"visual".as_ptr(),
            6,
            if (*oap).is_VIsual {
                kBoolVarTrue
            } else {
                kBoolVarFalse
            },
        );
        tv_dict_set_keys_readonly(dict);

        // The buffer must not change under the yank that is still in flight.
        *textlock.ptr() += 1;
        apply_autocmds(
            EVENT_TEXTYANKPOST,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        *textlock.ptr() -= 1;

        restore_v_event(dict, &raw mut save_v_event);
        recursive.set(false);
    }
}

/// `y`: yank the operator's region into the register it names.
///
/// Answers false for an invalid register name, having beeped.
///
/// # Safety
/// `oap` must describe a region of the current buffer. Runs the clipboard
/// provider and TextYankPost, and so arbitrary Lua.
pub unsafe fn op_yank(oap: *mut oparg_T, message: bool) -> bool {
    unsafe {
        if (*oap).regname != 0 && !valid_yank_reg((*oap).regname, true) {
            beep_flush();
            return false;
        }
        if (*oap).regname == '_' as c_int {
            return true; // black hole: nothing to do
        }

        let reg = get_yank_register((*oap).regname, YREG_YANK);
        op_yank_reg(oap, message, reg, is_append_register((*oap).regname));
        clipboard::set_clipboard((*oap).regname, reg);
        do_autocmd_textyankpost(oap, reg);
        true
    }
}
