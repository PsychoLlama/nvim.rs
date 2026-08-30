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

use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::guard::Lock;
use crate::message_fmt::{c_str, report_msg};
use crate::tr_plural;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_ulong, c_void};

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, NUL};

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
    if exclude_trailing_space {
        // SAFETY: the caller promises `bd` describes a region of the line.
        unsafe { (*bd).endspaces = 0 };
    }
    // Nothing below writes through `bd`, so its measurements are read once.
    //
    // SAFETY: as above.
    let def = unsafe { *bd };
    let size = def.startspaces + def.endspaces + def.textlen;
    debug_assert!(size >= 0);
    // SAFETY: `size` is not negative, so it is a length; `xmallocz` adds the
    // terminating NUL's byte itself.
    let start = unsafe { xmallocz(size as size_t) } as *mut c_char;
    // SAFETY: the caller promises `y_array` has more than `y_idx` slots.
    unsafe { (*(*reg).y_array.add(y_idx)).set_data(start) };

    // Lay the line out: the leading padding, the text, the trailing padding.
    //
    // SAFETY: the three runs are together exactly the `size` bytes `start`
    // owns, and `textstart` points at `textlen` bytes of the current line.
    let mut pnew = start;
    let into = pnew.cast::<u8>();
    unsafe { into.write_bytes(b' ', def.startspaces as size_t) };
    pnew = unsafe { pnew.offset(def.startspaces as isize) };
    let text = def.textstart as *const c_void;
    let into = pnew.cast::<u8>();
    unsafe { into.copy_from(text.cast(), def.textlen as size_t) };
    pnew = unsafe { pnew.offset(def.textlen as isize) };
    unsafe { pnew.cast::<u8>().write_bytes(b' ', def.endspaces as size_t) };
    pnew = unsafe { pnew.offset(def.endspaces as isize) };

    if exclude_trailing_space {
        // Walk back over the trailing white space, a character at a time
        // so that a multi-byte character is not cut in half.
        //
        // SAFETY: `s` starts at the end of the text `textstart` points at and
        // only ever moves back inside it; the `s > 0` test in front of the
        // read is what proves the byte at `s - 1` is one of them.  `pnew`
        // steps back over bytes it just wrote.
        let mut s = def.textlen + def.endspaces;
        while s > 0
            && ascii_iswhite(c_int::from(unsafe {
                *def.textstart.offset((s - 1) as isize)
            }))
        {
            s -= unsafe { utf_head_off(def.textstart, def.textstart.offset((s - 1) as isize)) } + 1;
            pnew = unsafe { pnew.offset(-1) };
        }
    }
    // SAFETY: `pnew` is at most one past the text, and `xmallocz` left room
    // for a NUL there.
    unsafe { *pnew = NUL as c_char };
    // SAFETY: the slot the data went into; `pnew` is at or past `start`.
    unsafe { (*(*reg).y_array.add(y_idx)).set_len(pnew.offset_from(start) as size_t) };
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
    // Grow `curr`'s array to hold both registers' lines, moving the old ones
    // over.  `j` is where the appended lines start.
    //
    // SAFETY: the caller promises both registers own arrays of `y_size`
    // strings, and the new array is allocated for both counts together.
    let mut j = unsafe {
        let old = (*curr).y_size;
        let room = ::core::mem::size_of::<String_0>().wrapping_mul(old.wrapping_add((*reg).y_size));
        let new_ptr = xmalloc(room) as *mut String_0;
        for i in 0..old {
            *new_ptr.add(i) = *(*curr).y_array.add(i);
        }
        xfree((*curr).y_array as *mut c_void);
        (*curr).y_array = new_ptr;
        old
    };

    // Appending linewise text makes the whole register linewise.
    if yank_type == kMTLineWise {
        // SAFETY: `curr` is a live register.
        unsafe { (*curr).y_type = kMTLineWise };
    }

    let mut y_idx: size_t = 0;
    // SAFETY: as above.
    if unsafe { (*curr).y_type } == kMTCharWise && !cpo_has(CpoFlag::REGAPPEND) {
        // Join the last old line and the first new one.
        //
        // SAFETY: `j` is `curr`'s line count, which is at least one here
        // because a charwise register holds a line, and the caller promises
        // `reg` holds one too.  Both strings are NUL-terminated and carry
        // their own lengths, and the joined allocation is the sum plus a NUL.
        let first_new = unsafe { *(*reg).y_array };
        j = j.wrapping_sub(1);
        let last_old = unsafe { &mut *(*curr).y_array.add(j) };
        let joined_size = last_old.len().wrapping_add(first_new.len());
        let pnew = unsafe { xmalloc(joined_size.wrapping_add(1)) } as *mut c_char;
        unsafe { strcpy(pnew, last_old.data()) };
        unsafe { strcpy(pnew.add(last_old.len()), first_new.data()) };
        unsafe { xfree(last_old.data() as *mut c_void) };
        *last_old = String_0::from_raw_parts(pnew, joined_size);
        j = j.wrapping_add(1);

        unsafe { xfree(first_new.data() as *mut c_void) };
        unsafe { *(*reg).y_array }.set_data(::core::ptr::null_mut());
        unsafe { *(*reg).y_array }.set_len(0);
        y_idx = 1;
    }

    // The rest of `reg`'s lines are *moved*, so `reg`'s array is freed but
    // the strings in it are not.
    //
    // SAFETY: the space for them was made above, and `y_idx` walks `reg`'s
    // own `y_size` strings.
    while y_idx < unsafe { (*reg).y_size } {
        unsafe { *(*curr).y_array.add(j) = *(*reg).y_array.add(y_idx) };
        y_idx = y_idx.wrapping_add(1);
        j = j.wrapping_add(1);
    }
    unsafe { (*curr).y_size = j };
    unsafe { xfree((*reg).y_array as *mut c_void) };
}

/// The "N lines yanked" message.
///
/// # Safety
/// `oap` must be the operator that was just applied.
unsafe fn report_yank(oap: *mut oparg_T, yank_type: MotionType, yanklines: size_t) {
    let mut namebuf: [c_char; 100] = [0; 100];
    // SAFETY: the caller promises `oap` is the operator just applied.
    let regname = unsafe { (*oap).regname };
    if regname == NUL {
        namebuf[0] = NUL as c_char;
    } else {
        let fmt = gettext(c" into \"%c");
        // SAFETY: `namebuf` is writable for the length given, and the format
        // takes exactly the one `%c` argument handed over.
        unsafe { vim_snprintf(namebuf.as_mut_ptr(), namebuf.len(), fmt.as_ptr(), regname) };
    }

    // The message may be the first thing that scrolls, so make sure the
    // window is up to date before it is written.
    //
    // SAFETY: main thread, with a current window and buffer.
    update_topline(unsafe { Win::current() });
    if must_redraw.get() != 0 {
        // SAFETY: as above.
        let _ = unsafe { update_screen() };
    }

    let (one, many) = if yank_type == kMTBlockWise {
        (
            c"block of %ld line yanked%s",
            c"block of %ld lines yanked%s",
        )
    } else {
        (c"%ld line yanked%s", c"%ld lines yanked%s")
    };
    let fmt = ngettext(one, many, yanklines as c_ulong);
    // SAFETY: `namebuf` is this frame's, NUL-terminated by the writes above.
    let name = unsafe { c_str(namebuf.as_mut_ptr()) };
    let _: bool = report_msg(0, || tr_plural!(fmt, yanklines as int64_t, name));
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
    let mut newreg = EMPTY_YANKREG;
    // Nothing this function reaches writes through `oap`, so the operator is
    // read once and worked from.
    //
    // SAFETY: the caller promises `oap` describes a region of the buffer.
    let op = unsafe { *oap };
    let mut yank_type = op.motion_type;
    let mut yanklines = op.line_count as size_t;
    let mut yankendlnum = op.end.lnum;
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
    // SAFETY: the caller promises `reg` is a live register.
    if append && !unsafe { (*reg).y_array }.is_null() {
        reg = &raw mut newreg;
    } else {
        // SAFETY: as above; it owns whatever it holds.
        unsafe { free_register(reg) };
    }

    // A charwise yank that starts in column 0 and ends before column 0 of
    // a later line is really a linewise one.
    //
    // SAFETY: 'selection' is a NUL-terminated option string.
    let sel_old = op.is_VIsual && unsafe { c_int::from(*p_sel.get()) } != 'o' as c_int;
    if op.motion_type == kMTCharWise
        && op.start.col == 0
        && !op.inclusive
        && !sel_old
        && op.end.col == 0
        && yanklines > 1
    {
        yank_type = kMTLineWise;
        yankendlnum -= 1;
        yanklines = yanklines.wrapping_sub(1);
    }

    // SAFETY: `reg` is live and its array is being replaced wholesale; the
    // new one has a slot per line the walk below fills in.
    unsafe { (*reg).y_size = yanklines };
    unsafe { (*reg).y_type = yank_type };
    unsafe { (*reg).y_width = 0 };
    unsafe {
        (*reg).y_array = xcalloc(yanklines, ::core::mem::size_of::<String_0>()) as *mut String_0
    };
    unsafe { (*reg).additional_data = ::core::ptr::null_mut() };
    unsafe { (*reg).timestamp = os_time() };

    if yank_type == kMTBlockWise {
        // A `$`-extended block has no fixed width.
        let narrow = cur_win().w_curswant == MAXCOL;
        // SAFETY: `reg` is live.
        unsafe { (*reg).y_width = op.end_vcol - op.start_vcol };
        if narrow && unsafe { (*reg).y_width } > 0 {
            unsafe { (*reg).y_width -= 1 };
        }
    }

    let mut y_idx: size_t = 0;
    let mut lnum = op.start.lnum;
    while lnum <= yankendlnum {
        // SAFETY: `reg` is live and holds the type just written.
        match unsafe { (*reg).y_type } {
            kMTBlockWise => {
                // SAFETY: `lnum` is a line of the region `oap` describes, and
                // `bd` is this walk's own measurement block.
                unsafe { block_prep(oap, &raw mut bd, lnum, false) };
                // SAFETY: `bd` now describes a region of that line, and the
                // array has a slot for every line of the region.
                unsafe { yank_copy_line(reg, &raw mut bd, y_idx, op.excl_tr_ws) };
            }
            kMTLineWise => {
                // SAFETY: `lnum` is a line of the current buffer, so `ml_get`
                // hands back its NUL-terminated text and `ml_get_len` its
                // length; the slot is this walk's own.
                let text = unsafe { cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t) };
                unsafe { *(*reg).y_array.add(y_idx) = text };
            }
            kMTCharWise => {
                // SAFETY: `lnum` is a line of the region, and `bd` is this
                // walk's own block.
                unsafe { charwise_block_prep(op.start, op.end, &raw mut bd, lnum, op.inclusive) };
                // The region may reach past the end of a short line.
                //
                // SAFETY: `textstart` points into that line, NUL-terminated.
                let tmp = unsafe { cstr::bytes_at(bd.textstart) }.len() as c_int;
                if tmp < bd.textlen {
                    bd.textlen = tmp;
                }
                // SAFETY: as in the blockwise arm.
                unsafe { yank_copy_line(reg, &raw mut bd, y_idx, false) };
            }
            // SAFETY: `abort` never returns and touches nothing.
            kMTUnknown => unsafe { abort() },
            _ => {}
        }
        lnum += 1;
        y_idx = y_idx.wrapping_add(1);
    }

    if curr != reg {
        // SAFETY: `curr` is the caller's register and `reg` the scratch one
        // just filled; both own their arrays, and a charwise `reg` holds at
        // least the one line the walk above put in it.
        unsafe { append_to_register(curr, reg, yank_type) };
    }

    if message {
        // A single charwise line is not worth reporting.
        if yank_type == kMTCharWise && yanklines == 1 {
            yanklines = 0;
        }
        if yanklines > p_report.get() as size_t {
            // SAFETY: `oap` is still the operator that was just applied.
            unsafe { report_yank(oap, yank_type, yanklines) };
        }
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        cur_buf().b_op_start = op.start;
        cur_buf().b_op_end = op.end;
        if yank_type == kMTLineWise {
            cur_buf().b_op_start.col = 0;
            cur_buf().b_op_end.col = MAXCOL;
        }
        if yank_type != kMTLineWise && !op.inclusive {
            // An exclusive region's `']` is the character *before* the end.
            //
            // SAFETY: the mark is a position in the current buffer, and the
            // borrow is taken through the root because `dec` reads `curbuf`
            // itself -- handing it a borrow of the whole `buf_T` would alias.
            unsafe { decl(&mut (*curbuf.get()).b_op_end) };
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
    debug_assert!(buf_len > 1);
    // Every type but the blockwise one answers a single character, which
    // `buf_len > 1` leaves room for along with its NUL.
    let short = match reg_type {
        kMTLineWise => 'V' as c_char,
        kMTCharWise => 'v' as c_char,
        kMTUnknown => NUL as c_char,
        kMTBlockWise => {
            // SAFETY: `buf` holds the `buf_len` bytes `snprintf` is told
            // about, and the format takes the single `%d` given.
            unsafe { snprintf(buf, buf_len, c"\x16%d".as_ptr(), reg_width + 1) };
            return;
        }
        _ => return,
    };
    // SAFETY: `buf_len` is more than one, so the first byte is writable.
    unsafe { *buf = short };
    // `kMTUnknown` answers the empty string, whose NUL is that first byte
    // already; upstream leaves the second one alone.
    if reg_type != kMTUnknown {
        // SAFETY: `buf_len` is more than one, so the second byte is too.
        unsafe { *buf.add(1) = NUL as c_char };
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
    static recursive: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: main thread, reading the autocommand table.
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
    // SAFETY: `save_v_event` is a writable local that outlives the matching
    // `restore_v_event` below.
    let dict = unsafe { get_v_event(&raw mut save_v_event) };

    // `regcontents`: the register's lines, as a locked list.
    //
    // SAFETY: the caller promises `reg` describes the yank, so its `y_array`
    // holds `y_size` strings, each NUL-terminated and carrying its length.
    let list = unsafe {
        let list = tv_list_alloc((*reg).y_size as ptrdiff_t);
        for i in 0..(*reg).y_size {
            let line = *(*reg).y_array.add(i);
            tv_list_append_string(list, line.data(), line.len() as c_int as ssize_t);
        }
        tv_list_set_lock(list, VarLock::Fixed);
        list
    };
    // SAFETY: `dict` is `v:event`'s, the key is a literal of the length given.
    let _ = unsafe { tv_dict_add_list(dict, c"regcontents".as_ptr(), 11, list) };

    let mut buf: [c_char; 67] = [0; 67];
    // SAFETY: `reg` is live, and `buf` is 67 writable bytes -- more than one.
    unsafe { format_reg_type((*reg).y_type, (*reg).y_width, buf.as_mut_ptr(), buf.len()) };
    // SAFETY: `buf` is NUL-terminated, and the key is a literal of length 7.
    let _ = unsafe { tv_dict_add_str(dict, c"regtype".as_ptr(), 7, buf.as_mut_ptr()) };

    // SAFETY: the caller promises `oap` is the yank's operator.
    let op = unsafe { *oap };
    buf[0] = op.regname as c_char;
    buf[1] = NUL as c_char;
    // SAFETY: as above.
    let _ = unsafe { tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_mut_ptr()) };

    let flag = |set| if set { kBoolVarTrue } else { kBoolVarFalse };
    // SAFETY: `dict` is `v:event`'s, the key a literal of the length given.
    let _ = unsafe { tv_dict_add_bool(dict, c"inclusive".as_ptr(), 9, flag(op.inclusive)) };

    buf[0] = get_op_char(op.op_type) as c_char;
    buf[1] = NUL as c_char;
    // SAFETY: `buf` is NUL-terminated, and the key is a literal of length 8.
    let _ = unsafe { tv_dict_add_str(dict, c"operator".as_ptr(), 8, buf.as_mut_ptr()) };

    // SAFETY: as for `inclusive`.
    let _ = unsafe { tv_dict_add_bool(dict, c"visual".as_ptr(), 6, flag(op.is_VIsual)) };
    // SAFETY: `dict` is the one just filled in.
    unsafe { tv_dict_set_keys_readonly(dict) };

    // The buffer must not change under the yank that is still in flight.
    let locked = Lock::text();
    let none = ::core::ptr::null_mut();
    // SAFETY: main thread, with a current buffer; null pattern and file name
    // ask for the event's own defaults.
    unsafe { apply_autocmds(EVENT_TEXTYANKPOST, none, none, false, curbuf.get()) };
    drop(locked);

    // SAFETY: `save_v_event` is the one `get_v_event` was given.
    unsafe { restore_v_event(dict, &raw mut save_v_event) };
    recursive.set(false);
}

/// `y`: yank the operator's region into the register it names.
///
/// Answers false for an invalid register name, having beeped.
///
/// # Safety
/// `oap` must describe a region of the current buffer. Runs the clipboard
/// provider and TextYankPost, and so arbitrary Lua.
pub unsafe fn op_yank(oap: *mut oparg_T, message: bool) -> bool {
    // SAFETY: the caller promises `oap` describes a region of the buffer.
    let regname = unsafe { (*oap).regname };
    // SAFETY: main thread, reading the register store.
    if regname != 0 && !unsafe { valid_yank_reg(regname, true) } {
        // SAFETY: as above.
        beep_flush();
        return false;
    }
    if regname == '_' as c_int {
        return true; // black hole: nothing to do
    }

    // SAFETY: `regname` is a valid register name, checked above.
    let reg = unsafe { get_yank_register(regname, YREG_YANK) };
    // SAFETY: `oap` describes a region of the buffer and `reg` is live.
    unsafe { op_yank_reg(oap, message, reg, is_append_register(regname)) };
    // SAFETY: `reg` holds what was just yanked; this is what runs the
    // clipboard provider's Lua.
    unsafe { clipboard::set_clipboard(regname, reg) };
    // SAFETY: `oap` and `reg` describe the yank that just happened.
    unsafe { do_autocmd_textyankpost(oap, reg) };
    true
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
