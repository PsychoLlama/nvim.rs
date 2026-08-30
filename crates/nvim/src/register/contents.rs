//! A register as text: the API's way in and out.
//!
//! [`get_reg_contents`] and the `write_reg_contents*` family are what
//! `getreg()`, `setreg()`, `nvim_paste`, the clipboard provider and shada all
//! use, so these are the functions that turn a `yankreg_T` into lines or one
//! string and back.
//!
//! [`str_to_reg`] is the interesting direction. It splits an incoming string
//! on newlines; an embedded **NUL is a newline** the way the rest of the
//! editor spells one, so it is translated on the way in; a charwise register
//! written to again *appends to its last line* rather than starting a new
//! one; and a blockwise write has to measure the widest line in **cells**
//! rather than bytes.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::types::NUL;

/// The motion type of register `regname`, and its width if blockwise.
///
/// `kMTUnknown` for an invalid or empty register.
///
/// # Safety
/// `reg_width` must be null or writable. May run the clipboard provider.
pub unsafe fn get_reg_type(regname: c_int, reg_width: *mut colnr_T) -> MotionType {
    // Every computed register reads as charwise.
    match regname {
        Ctrl_F | Ctrl_P | Ctrl_W | Ctrl_A => return kMTCharWise,
        c if c == '#' as c_int
            || c == '=' as c_int
            || c == ':' as c_int
            || c == '/' as c_int
            || c == '.' as c_int
            || c == '%' as c_int
            || c == '_' as c_int =>
        {
            return kMTCharWise;
        }
        _ => {}
    }

    // SAFETY: `valid_yank_reg` only looks the name up.
    if regname != NUL && !unsafe { valid_yank_reg(regname, false) } {
        return kMTUnknown;
    }
    // SAFETY: a valid register name, so this answers a live register.
    let reg = unsafe { get_yank_register(regname, YREG_PASTE) };
    // SAFETY: `reg` is that live register; these are three of its fields.
    let (y_array, y_type, y_width) = unsafe { ((*reg).y_array, (*reg).y_type, (*reg).y_width) };
    if y_array.is_null() {
        return kMTUnknown;
    }
    if !reg_width.is_null() && y_type == kMTBlockWise {
        // SAFETY: the caller promises a writable `reg_width`.
        unsafe { *reg_width = y_width };
    }
    y_type
}

/// Hand back `s` as either the string itself or a one-element list, depending
/// on `kGRegList`.
///
/// Takes ownership of `s` either way.
///
/// # Safety
/// `s` must be an allocated, NUL-terminated string.
unsafe fn get_reg_wrap_one_line(s: *mut c_char, flags: c_int) -> *mut c_void {
    if flags & kGRegList as c_int == 0 {
        return s as *mut c_void;
    }
    // SAFETY: a fresh one-element list, which then takes ownership of `s` --
    // an allocated NUL-terminated string, as the caller promises.
    let list = unsafe { tv_list_alloc(1) };
    // SAFETY: as above.
    unsafe { tv_list_append_allocated_string(list, s) };
    list as *mut c_void
}

/// The contents of register `regname`, as an allocated string or -- with
/// `kGRegList` -- a `list_T` of lines.
///
/// `kGRegNoExpr` refuses `"=` outright and `kGRegExprSrc` answers its source
/// rather than evaluating it, which is what `getreg('=', 1, ...)` wants.
///
/// # Safety
/// May run arbitrary Vimscript through `"=`.
pub unsafe fn get_reg_contents(regname: c_int, flags: c_int) -> *mut c_void {
    let mut regname = regname;
    if regname == '=' as c_int {
        if flags & kGRegNoExpr as c_int != 0 {
            return ::core::ptr::null_mut();
        }
        // SAFETY: both hand back an allocated NUL-terminated string, which is
        // what the wrapper takes ownership of.
        if flags & kGRegExprSrc as c_int != 0 {
            return unsafe { get_reg_wrap_one_line(get_expr_line_src(), flags) };
        }
        // SAFETY: as above; evaluating the expression may run Vimscript,
        // which this function's own caller already allows for.
        return unsafe { get_reg_wrap_one_line(get_expr_line(), flags) };
    }
    if regname == '@' as c_int {
        regname = '"' as c_int; // `getreg('@')` means the unnamed register
    }
    // SAFETY: `valid_yank_reg` only looks the name up.
    if regname != NUL && !unsafe { valid_yank_reg(regname, false) } {
        return ::core::ptr::null_mut();
    }

    let mut retval: *mut c_char = ::core::ptr::null_mut();
    let mut allocated = false;
    // SAFETY: two writable locals, which is all `get_spec_reg` writes to.
    if unsafe { get_spec_reg(regname, &raw mut retval, &raw mut allocated, false) } {
        if retval.is_null() {
            return ::core::ptr::null_mut();
        }
        // The caller always owns the answer.
        // SAFETY: `get_spec_reg` answered a NUL-terminated string; when it is
        // not already ours, `xstrdup` makes a copy that is.
        let owned = if allocated {
            retval
        } else {
            unsafe { xstrdup(retval) }
        };
        // SAFETY: `owned` is that allocated, NUL-terminated string.
        return unsafe { get_reg_wrap_one_line(owned, flags) };
    }

    // SAFETY: a valid register name, so this answers a live register.
    let reg = unsafe { get_yank_register(regname, YREG_PUT) };
    // SAFETY: `reg` is that live register; these are three of its fields.
    let (y_array, y_type, y_size) = unsafe { ((*reg).y_array, (*reg).y_type, (*reg).y_size) };
    if y_array.is_null() {
        return ::core::ptr::null_mut();
    }

    if flags & kGRegList as c_int != 0 {
        // SAFETY: a non-null `y_array` holds `y_size` NUL-terminated lines,
        // so every index below is one of them; the list copies each.
        return unsafe {
            let list = tv_list_alloc(y_size as ptrdiff_t);
            for i in 0..y_size {
                let line = *y_array.add(i);
                tv_list_append_string(list, line.data(), line.len() as c_int as ssize_t);
            }
            list as *mut c_void
        };
    }

    // One string, with a newline between lines and after the last one if
    // the register is linewise.
    let needs_nl = |i: size_t| y_type == kMTLineWise || i < y_size.wrapping_sub(1);
    let mut len: size_t = 0;
    for i in 0..y_size {
        // SAFETY: `i` is below `y_size`, so this is one of the register's
        // lines.
        len = len.wrapping_add(unsafe { (*y_array.add(i)).len() });
        if needs_nl(i) {
            len = len.wrapping_add(1);
        }
    }
    // SAFETY: `len` is the sum of the lines and the newlines between them,
    // and the +1 is the terminating NUL, so every copy below lands inside.
    let retval = unsafe { xmalloc(len.wrapping_add(1)) } as *mut c_char;
    let mut at: size_t = 0;
    for i in 0..y_size {
        // SAFETY: `i` is below `y_size`, as above.
        let line = unsafe { *y_array.add(i) };
        // SAFETY: `at` is the offset the loop above measured this line at,
        // and the line is NUL-terminated.
        unsafe { strcpy(retval.add(at), line.data()) };
        at = at.wrapping_add(line.len());
        if needs_nl(i) {
            // SAFETY: the loop above counted this newline into `len`.
            unsafe { *retval.add(at) = '\n' as c_char };
            at = at.wrapping_add(1);
        }
    }
    // SAFETY: `at` has reached `len`, the last byte of the allocation.
    unsafe { *retval.add(at) = NUL as c_char };
    retval as *mut c_void
}

/// Prepare register `name` to be written: check the name, remember `""`, and
/// empty the register unless the write is an append.
///
/// Answers null for an invalid name, having given E354.
///
/// # Safety
/// Unless the write is an append, the register's current contents are freed,
/// so nothing may still be pointing at them.
unsafe fn init_write_reg(
    name: c_int,
    old_y_previous: &mut *mut yankreg_T,
    must_append: bool,
) -> *mut yankreg_T {
    // SAFETY: `valid_yank_reg` only looks the name up.
    if !unsafe { valid_yank_reg(name, true) } {
        // SAFETY: reports the name, which is all it reads.
        unsafe { emsg_invreg(name) };
        return ::core::ptr::null_mut();
    }
    // `get_yank_register` moves `""`, which a write to a *named* register
    // must not do; `finish_write_reg` puts it back.
    *old_y_previous = y_previous.get();
    // SAFETY: a valid register name, so this answers a live register.
    let reg = unsafe { get_yank_register(name, YREG_YANK) };
    if !is_append_register(name) && !must_append {
        // SAFETY: `reg` is that live register, and the caller promises
        // nothing else holds on to the lines being dropped here.
        unsafe { free_register(reg) };
    }
    reg
}

/// How many lines `str` will become, and whether the first of them joins the
/// register's current last line.
///
/// `extraline` is the line a string not ending in a newline still contributes.
///
/// # Safety
/// `y_ptr` must be a live register, and `str` must hold `len` bytes, or --
/// with `str_list` -- be a null-terminated array of NUL-terminated strings.
unsafe fn count_lines(
    y_ptr: *mut yankreg_T,
    yank_type: MotionType,
    str: *const c_char,
    len: size_t,
    str_list: bool,
) -> (size_t, bool, bool) {
    if str_list {
        let mut newlines: size_t = 0;
        // SAFETY: with `str_list`, `str` is a null-terminated array of string
        // pointers, so the walk stops on its terminator without leaving it.
        let mut ss = str as *mut *mut c_char;
        while !unsafe { *ss }.is_null() {
            newlines = newlines.wrapping_add(1);
            ss = unsafe { ss.add(1) };
        }
        return (newlines, false, false);
    }

    // SAFETY: `str` holds `len` bytes, which is the span searched.
    let mut newlines = unsafe { memcnt(str as *const c_void, '\n' as c_char, len) };
    let mut extraline = false;
    // SAFETY: the `len == 0` arm in front is the bounds proof -- the read is
    // of the last of `str`'s `len` bytes -- so the chain stays whole.
    if yank_type == kMTCharWise
        || len == 0
        || unsafe { c_int::from(*str.add(len.wrapping_sub(1))) } != '\n' as c_int
    {
        extraline = true;
        newlines = newlines.wrapping_add(1);
    }
    // A charwise register's last line has no newline of its own, so the
    // first new line continues it.
    // SAFETY: `y_ptr` is a live register.
    let append = unsafe { (*y_ptr).y_size > 0 && (*y_ptr).y_type == kMTCharWise };
    if append {
        newlines = newlines.wrapping_sub(1);
    }
    (newlines, extraline, append)
}

/// Put `str` in `y_ptr`, appending to whatever is already there.
///
/// `yank_type` may be `kMTUnknown`, in which case it is worked out from the
/// text: a trailing newline (or carriage return) means linewise. `blocklen`
/// is the block width, or -1 to measure it.
///
/// # Safety
/// `str` must hold `len` bytes, or -- with `str_list` -- be a
/// null-terminated array of NUL-terminated strings.
unsafe fn str_to_reg(
    y_ptr: *mut yankreg_T,
    mut yank_type: MotionType,
    str: *const c_char,
    len: size_t,
    blocklen: colnr_T,
    str_list: bool,
) {
    // SAFETY: `y_ptr` is a live register.
    if unsafe { (*y_ptr).y_array.is_null() } {
        // SAFETY: as above.
        unsafe { (*y_ptr).y_size = 0 };
    }
    if yank_type == kMTUnknown {
        // SAFETY: the `len > 0` in front of the reads is their bounds proof
        // -- they are of the last of `str`'s `len` bytes -- so the chain
        // stays whole. `str_list` short-circuits before any of it.
        let ends_line = str_list
            || len > 0
                && unsafe {
                    c_int::from(*str.add(len.wrapping_sub(1))) == NL
                        || c_int::from(*str.add(len.wrapping_sub(1))) == CAR
                };
        yank_type = if ends_line { kMTLineWise } else { kMTCharWise };
    }

    // SAFETY: `y_ptr` is a live register and the caller's promise about
    // `str`/`len`/`str_list` carries straight through.
    let (newlines, extraline, mut append) =
        unsafe { count_lines(y_ptr, yank_type, str, len, str_list) };
    // SAFETY: `y_ptr` is a live register.
    let y_size = unsafe { (*y_ptr).y_size };
    if y_size.wrapping_add(newlines) == 0 {
        // SAFETY: `y_array` is the register's own allocation, and the field
        // is nulled in the same breath so nothing reads the freed block.
        unsafe { xfree((*y_ptr).y_array as *mut c_void) };
        unsafe { (*y_ptr).y_array = ::core::ptr::null_mut() };
        return;
    }

    let room = y_size
        .wrapping_add(newlines)
        .wrapping_mul(::core::mem::size_of::<String_0>());
    // SAFETY: `y_array` is the register's own allocation, grown here to hold
    // the lines it already has plus the new ones; the field is updated to the
    // block `xrealloc` answered before anything reads it again.
    let pp = unsafe { xrealloc((*y_ptr).y_array as *mut c_void, room) } as *mut String_0;
    // SAFETY: as above.
    unsafe { (*y_ptr).y_array = pp };

    let mut lnum = y_size;
    let mut maxlen: size_t = 0;
    if str_list {
        // SAFETY: with `str_list`, `str` is a null-terminated array of
        // NUL-terminated strings, so the walk stops on its terminator and
        // every string it hands on is readable; `count_lines` counted the
        // same elements, so `pp` has room for all of them from `lnum` on.
        let mut ss = str as *mut *mut c_char;
        while !unsafe { *ss }.is_null() {
            unsafe { *pp.add(lnum) = cstr_to_string(*ss) };
            if yank_type == kMTBlockWise {
                maxlen = maxlen.max(unsafe { mb_string2cells(*ss) });
            }
            ss = unsafe { ss.add(1) };
            lnum = lnum.wrapping_add(1);
        }
    } else {
        let mut start = str;
        // SAFETY: `str` holds `len` bytes, so this is its one-past-the-end.
        let end = unsafe { str.add(len) };
        // `+ extraline` so that a string with no trailing newline still
        // produces its last (possibly empty) line. Both operands are loop
        // invariants, so the limit is worked out once; it is only ever
        // compared against, never read through, hence `wrapping_offset`.
        let limit = end.wrapping_offset(extraline as c_int as isize);
        while start < limit {
            let mut charlen = 0;
            let mut line_end = start;
            // SAFETY: `line_end` walks from `start` to `end`, so it stays
            // inside `str`'s `len` bytes; the two UTF-8 helpers are handed
            // the number of bytes left, so neither reads past `end`, and a
            // NUL byte is stepped over singly rather than measured.
            while line_end < end {
                if c_int::from(unsafe { *line_end }) == '\n' as c_int {
                    break;
                }
                if yank_type == kMTBlockWise {
                    charlen +=
                        unsafe { utf_ptr2cells_len(line_end, end.offset_from(line_end) as c_int) };
                }
                // A NUL is one byte and is translated below, so it must
                // not be handed to the UTF-8 length.
                if c_int::from(unsafe { *line_end }) == NUL {
                    line_end = unsafe { line_end.add(1) };
                } else {
                    let left = unsafe { end.offset_from(line_end) } as c_int;
                    line_end = unsafe { line_end.offset(utf_ptr2len_len(line_end, left) as isize) };
                }
            }
            debug_assert!(line_end >= start);
            // SAFETY: both point into `str`, `line_end` at or after `start`.
            let line_len = unsafe { line_end.offset_from(start) } as size_t;
            maxlen = maxlen.max(charlen as size_t);

            // Appending continues the register's current last line.
            let extra = if append {
                lnum = lnum.wrapping_sub(1);
                // SAFETY: `append` means the register had a last line, which
                // `lnum` now indexes.
                unsafe { (*pp.add(lnum)).len() }
            } else {
                0
            };
            // SAFETY: room for the line the register already had and the one
            // just read off `str`, plus the NUL `xmallocz` adds.
            let s = unsafe { xmallocz(line_len.wrapping_add(extra)) } as *mut c_char;
            if extra > 0 {
                // SAFETY: `extra` is the length of the line at `lnum`, and
                // `s` was sized to take it first.
                let into = s.cast::<u8>();
                unsafe { into.copy_from_nonoverlapping((*pp.add(lnum)).data().cast(), extra) };
            }
            if line_len > 0 {
                // SAFETY: `line_len` bytes from `start` lie inside `str`, and
                // `s` was sized to take them after `extra`.
                let into = unsafe { s.add(extra) }.cast::<u8>();
                unsafe { into.copy_from_nonoverlapping(start.cast(), line_len) };
            }
            let s_len = extra.wrapping_add(line_len);
            if append {
                // SAFETY: the old line's text has just been copied into `s`,
                // and the array slot is overwritten below.
                unsafe { xfree((*pp.add(lnum)).data() as *mut c_void) };
                append = false;
            }
            // SAFETY: `lnum` is inside the array `xrealloc` sized above, and
            // `s` is an allocation of `s_len` bytes plus a NUL.
            unsafe { *pp.add(lnum) = String_0::from_raw_parts(s, s_len) };
            // A NUL in the text is how the editor spells a newline.
            // SAFETY: `s` holds those `s_len` bytes.
            unsafe { memchrsub(s as *mut c_void, NUL as c_char, '\n' as c_char, s_len) };

            // Past the newline `line_end` stopped on -- which on the last
            // line is one past `end`, so this cannot be a plain `add`.
            start = start.wrapping_add(line_len.wrapping_add(1));
            lnum = lnum.wrapping_add(1);
        }
    }

    let y_width = if yank_type == kMTBlockWise {
        if blocklen == -1 {
            maxlen as c_int - 1
        } else {
            blocklen
        }
    } else {
        0
    };
    // SAFETY: `y_ptr` is a live register, now holding `lnum` lines; its
    // `additional_data` is its own allocation and described the text that has
    // just been replaced, so it is dropped and the field nulled.
    unsafe { (*y_ptr).y_type = yank_type };
    unsafe { (*y_ptr).y_size = lnum };
    unsafe { xfree((*y_ptr).additional_data as *mut c_void) };
    unsafe { (*y_ptr).additional_data = ::core::ptr::null_mut() };
    unsafe { (*y_ptr).timestamp = os_time() };
    unsafe { (*y_ptr).y_width = y_width };
}

/// Push the write out to the clipboard and put `""` back where it was.
///
/// # Safety
/// `reg` must be the register [`init_write_reg`] answered.
unsafe fn finish_write_reg(name: c_int, reg: *mut yankreg_T, old_y_previous: *mut yankreg_T) {
    // SAFETY: `reg` is the register `init_write_reg` answered, which the
    // provider is handed the contents of.
    unsafe { clipboard::set_clipboard(name, reg) };
    // Only an explicit write to `""` moves it.
    if name != '"' as c_int {
        y_previous.set(old_y_previous);
    }
}

/// Write `str` to register `name`, working out the motion type from the text.
///
/// `len` may be -1 for a NUL-terminated string.
///
/// # Safety
/// `str` must hold `len` bytes, or be NUL-terminated when `len` is negative.
pub unsafe fn write_reg_contents(
    name: c_int,
    str: *const c_char,
    len: ssize_t,
    must_append: c_int,
) {
    // SAFETY: the caller's promise about `str` and `len` carries through.
    unsafe { write_reg_contents_ex(name, str, len, must_append != 0, kMTUnknown, 0) };
}

/// Write a null-terminated array of lines to register `name`.
///
/// # Safety
/// `strings` must be a null-terminated array of NUL-terminated strings.
pub unsafe fn write_reg_contents_lst(
    name: c_int,
    strings: *mut *mut c_char,
    must_append: bool,
    yank_type: MotionType,
    block_len: colnr_T,
) {
    if name == '/' as c_int || name == '=' as c_int {
        // Neither register can hold more than one line.
        // SAFETY: `strings` is a null-terminated array, so the first element
        // is there; the second is only read once the first is not the
        // terminator, so it is there too.
        let first = unsafe { *strings };
        let s = if first.is_null() {
            c"".as_ptr().cast_mut()
        } else if !(unsafe { *strings.add(1) }).is_null() {
            emsg(gettext(
                e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines,
            ));
            return;
        } else {
            first
        };
        // SAFETY: `s` is NUL-terminated, which is what `len == -1` means.
        unsafe { write_reg_contents_ex(name, s, -1, must_append, yank_type, block_len) };
        return;
    }
    if name == '_' as c_int {
        return; // black hole
    }

    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut();
    // SAFETY: a plain write, so nothing is still holding the old contents.
    let reg = unsafe { init_write_reg(name, &mut old_y_previous, must_append) };
    if reg.is_null() {
        return;
    }
    // The length is meaningless for a list write and `str_to_reg` ignores
    // it; upstream passes `strlen((char *)strings)` all the same.
    // SAFETY: `strings` is the null-terminated array of NUL-terminated
    // strings `str_list` asks for, and `reg` is the register just prepared.
    let len = unsafe { cstr::bytes_at(strings as *mut c_char) }.len();
    unsafe { str_to_reg(reg, yank_type, strings as *mut c_char, len, block_len, true) };
    unsafe { finish_write_reg(name, reg, old_y_previous) };
}

/// Write `str` to register `name` as `yank_type`.
///
/// `"/`, `"#` and `"=` are not real registers and are handled here: the first
/// sets the last search pattern, the second the alternate file, and the third
/// the expression source.
///
/// # Safety
/// `str` must hold `len` bytes, or be NUL-terminated when `len` is negative.
pub unsafe fn write_reg_contents_ex(
    name: c_int,
    str: *const c_char,
    len: ssize_t,
    must_append: bool,
    yank_type: MotionType,
    block_len: colnr_T,
) {
    // SAFETY: a negative `len` means `str` is NUL-terminated.
    let len = if len < 0 {
        unsafe { cstr::bytes_at(str).len() as ssize_t }
    } else {
        len
    };

    if name == '/' as c_int {
        // SAFETY: `str` holds `len` bytes, which is what the pattern is read
        // from; it is copied, not kept.
        unsafe { set_last_search_pat(str, RE_SEARCH as c_int, true, true) };
        return;
    }

    if name == '#' as c_int {
        // The alternate file, by number or by name.
        // SAFETY: `str` is the buffer number or name, NUL-terminated or
        // `len` bytes long; `atoi` stops at the first non-digit, which the
        // test in front proves is not the first byte, and `str + len` is the
        // one-past-the-end `buflist_findpat` wants.
        let buf = unsafe {
            if ascii_isdigit(c_int::from(*str)) {
                let num = atoi(str);
                let buf = find_buf(num);
                if buf.is_none() {
                    semsg!("E86: Buffer {} does not exist", int64_t::from(num));
                }
                buf
            } else {
                let end = str.offset(len as isize);
                find_buf(buflist_findpat(str, end, true, false, false))
            }
        };
        if let Some(buf) = buf {
            cur_win().w_alt_fnum = buf.handle;
        }
        return;
    }

    if name == '=' as c_int {
        // The expression register keeps its source, not a yankreg.
        let mut offset: size_t = 0;
        let mut totlen = len as size_t;
        if must_append && !expr_line.get().is_null() {
            // SAFETY: the `is_null` in front proves there is a string, and
            // `expr_line` is always NUL-terminated.
            let exprlen = unsafe { cstr::bytes_at(expr_line.get()) }.len();
            totlen = totlen.wrapping_add(exprlen);
            offset = exprlen;
        }
        // SAFETY: `expr_line` is our own allocation, regrown to hold the text
        // being kept (`offset` bytes), the `len` new ones, and a NUL.
        let grown = unsafe { xrealloc(expr_line.get() as *mut c_void, totlen.wrapping_add(1)) };
        expr_line.set(grown as *mut c_char);
        let dst = expr_line.get();
        // SAFETY: as above -- `offset + len` is `totlen`, and `str` holds
        // those `len` bytes.
        let into = unsafe { dst.add(offset) }.cast::<u8>();
        unsafe { into.copy_from_nonoverlapping(str.cast(), len as size_t) };
        // SAFETY: `totlen` is the last byte of the allocation.
        unsafe { *dst.add(totlen) = NUL as c_char };
        return;
    }

    if name == '_' as c_int {
        return; // black hole
    }

    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut();
    // SAFETY: a plain write, so nothing is still holding the old contents.
    let reg = unsafe { init_write_reg(name, &mut old_y_previous, must_append) };
    if reg.is_null() {
        return;
    }
    // SAFETY: `str` holds `len` bytes -- `strlen`'s answer, when the caller
    // gave -1 -- and `reg` is the register just prepared.
    unsafe { str_to_reg(reg, yank_type, str, len as size_t, block_len, false) };
    unsafe { finish_write_reg(name, reg, old_y_previous) };
}

/// Set `reg`'s type from an API `regtype` string (`""`, `v`/`c`, `V`/`l`,
/// `b`/CTRL-V, optionally followed by a block width).
///
/// Answers false for a `regtype` that does not parse.
///
/// # Safety
/// `reg` must be writable and `regtype` describe readable bytes.
pub unsafe fn prepare_yankreg_from_object(
    reg: *mut yankreg_T,
    regtype: String_0,
    _lines: size_t,
) -> bool {
    // SAFETY: a non-null `regtype` describes readable bytes, so its first is
    // there to be read.
    let type_0 = if regtype.data().is_null() {
        NUL
    } else {
        unsafe { c_int::from(*regtype.data()) }
    };
    let y_type = match type_0 {
        0 => kMTUnknown, // "" means "work it out from the text"
        c if c == 'v' as c_int || c == 'c' as c_int => kMTCharWise,
        c if c == 'V' as c_int || c == 'l' as c_int => kMTLineWise,
        c if c == 'b' as c_int || c == Ctrl_V => kMTBlockWise,
        _ => return false,
    };
    // SAFETY: the caller promises a writable `reg`. As upstream, both fields
    // are settled before the width is parsed, so a `regtype` that fails below
    // still leaves the type behind.
    unsafe { (*reg).y_type = y_type };
    unsafe { (*reg).y_width = 0 };

    if regtype.len() > 1 {
        // A width only means something for a block.
        if y_type != kMTBlockWise {
            return false;
        }
        // SAFETY: `regtype` is longer than one byte, so its second is there.
        if !unsafe { ascii_isdigit(c_int::from(*regtype.data().add(1))) } {
            return false;
        }
        // SAFETY: as above.
        let mut p: *const c_char = unsafe { regtype.data().add(1) };
        // SAFETY: `p` is a writable local, pointing at the digit just seen;
        // `getdigits_int` walks it to the end of the run and no further, so
        // it stays inside `regtype`, and `reg` is writable.
        unsafe { (*reg).y_width = getdigits_int(&raw mut p as *mut *mut c_char, false, 1) - 1 };
        // Nothing may follow the width.
        // SAFETY: `p` and `regtype.data()` are into the same string.
        if regtype.len() > unsafe { p.offset_from(regtype.data()) } as size_t {
            return false;
        }
    }
    // SAFETY: a writable `reg`, as above.
    unsafe { (*reg).additional_data = ::core::ptr::null_mut() };
    unsafe { (*reg).timestamp = 0 };
    true
}

/// Settle a register built from an API object: drop the empty last line a
/// linewise write ends with, and decide the type when it was left unknown.
///
/// `clipboard_adjust` is for the clipboard provider, whose lists always carry
/// that trailing empty line.
///
/// # Safety
/// `reg` must own an array of `y_size` lines.
pub unsafe fn finish_yankreg_from_object(reg: *mut yankreg_T, clipboard_adjust: bool) {
    // SAFETY: `reg` owns an array of `y_size` lines; these are its fields.
    let (y_type, y_size, y_array) = unsafe { ((*reg).y_type, (*reg).y_size, (*reg).y_array) };
    // SAFETY: the `y_size > 0` in front is the bounds proof for the read, so
    // the chain stays whole -- it names the array's last line.
    let ends_empty = y_size > 0 && unsafe { (*y_array.add(y_size.wrapping_sub(1))).is_empty() };
    if ends_empty {
        if y_type != kMTCharWise {
            if y_type == kMTUnknown || clipboard_adjust {
                // SAFETY: `reg` is writable; the empty last line is dropped
                // from the count, its `String_0` staying in the array for
                // `free_register` to release.
                unsafe { (*reg).y_size = y_size.wrapping_sub(1) };
            }
            if y_type == kMTUnknown {
                // SAFETY: as above.
                unsafe { (*reg).y_type = kMTLineWise };
            }
        }
    } else if y_type == kMTUnknown {
        // SAFETY: as above.
        unsafe { (*reg).y_type = kMTCharWise };
    }
    // SAFETY: `reg` is the register whose lines were just settled.
    unsafe { update_yankreg_width(reg) };
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
