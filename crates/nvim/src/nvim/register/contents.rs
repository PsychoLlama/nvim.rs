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

use crate::semsg_c;
use ::core::ffi::{c_char, c_int, c_void};

use super::*;

/// The motion type of register `regname`, and its width if blockwise.
///
/// `kMTUnknown` for an invalid or empty register.
///
/// # Safety
/// `reg_width` must be null or writable. May run the clipboard provider.
pub unsafe fn get_reg_type(regname: c_int, reg_width: *mut colnr_T) -> MotionType {
    unsafe {
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

        if regname != NUL && !valid_yank_reg(regname, false) {
            return kMTUnknown;
        }
        let reg = get_yank_register(regname, YREG_PASTE);
        if (*reg).y_array.is_null() {
            return kMTUnknown;
        }
        if !reg_width.is_null() && (*reg).y_type == kMTBlockWise {
            *reg_width = (*reg).y_width;
        }
        (*reg).y_type
    }
}

/// Hand back `s` as either the string itself or a one-element list, depending
/// on `kGRegList`.
///
/// Takes ownership of `s` either way.
///
/// # Safety
/// `s` must be an allocated, NUL-terminated string.
unsafe fn get_reg_wrap_one_line(s: *mut c_char, flags: c_int) -> *mut c_void {
    unsafe {
        if flags & kGRegList as c_int == 0 {
            return s as *mut c_void;
        }
        let list = tv_list_alloc(1);
        tv_list_append_allocated_string(list, s);
        list as *mut c_void
    }
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
    unsafe {
        let mut regname = regname;
        if regname == '=' as c_int {
            if flags & kGRegNoExpr as c_int != 0 {
                return ::core::ptr::null_mut();
            }
            if flags & kGRegExprSrc as c_int != 0 {
                return get_reg_wrap_one_line(get_expr_line_src(), flags);
            }
            return get_reg_wrap_one_line(get_expr_line(), flags);
        }
        if regname == '@' as c_int {
            regname = '"' as c_int; // `getreg('@')` means the unnamed register
        }
        if regname != NUL && !valid_yank_reg(regname, false) {
            return ::core::ptr::null_mut();
        }

        let mut retval: *mut c_char = ::core::ptr::null_mut();
        let mut allocated = false;
        if get_spec_reg(regname, &raw mut retval, &raw mut allocated, false) {
            if retval.is_null() {
                return ::core::ptr::null_mut();
            }
            // The caller always owns the answer.
            let owned = if allocated { retval } else { xstrdup(retval) };
            return get_reg_wrap_one_line(owned, flags);
        }

        let reg = get_yank_register(regname, YREG_PUT);
        if (*reg).y_array.is_null() {
            return ::core::ptr::null_mut();
        }

        if flags & kGRegList as c_int != 0 {
            let list = tv_list_alloc((*reg).y_size as ptrdiff_t);
            for i in 0..(*reg).y_size {
                let line = *(*reg).y_array.add(i);
                tv_list_append_string(list, line.data, line.size as c_int as ssize_t);
            }
            return list as *mut c_void;
        }

        // One string, with a newline between lines and after the last one if
        // the register is linewise.
        let needs_nl =
            |i: size_t| (*reg).y_type == kMTLineWise || i < (*reg).y_size.wrapping_sub(1);
        let mut len: size_t = 0;
        for i in 0..(*reg).y_size {
            len = len.wrapping_add((*(*reg).y_array.add(i)).size);
            if needs_nl(i) {
                len = len.wrapping_add(1);
            }
        }
        let retval = xmalloc(len.wrapping_add(1)) as *mut c_char;
        let mut at: size_t = 0;
        for i in 0..(*reg).y_size {
            let line = *(*reg).y_array.add(i);
            strcpy(retval.add(at), line.data);
            at = at.wrapping_add(line.size);
            if needs_nl(i) {
                *retval.add(at) = '\n' as c_char;
                at = at.wrapping_add(1);
            }
        }
        *retval.add(at) = NUL as c_char;
        retval as *mut c_void
    }
}

/// Prepare register `name` to be written: check the name, remember `""`, and
/// empty the register unless the write is an append.
///
/// Answers null for an invalid name, having given E354.
///
/// # Safety
/// `old_y_previous` must be writable.
unsafe fn init_write_reg(
    name: c_int,
    old_y_previous: *mut *mut yankreg_T,
    must_append: bool,
) -> *mut yankreg_T {
    unsafe {
        if !valid_yank_reg(name, true) {
            emsg_invreg(name);
            return ::core::ptr::null_mut();
        }
        // `get_yank_register` moves `""`, which a write to a *named* register
        // must not do; `finish_write_reg` puts it back.
        *old_y_previous = y_previous.get();
        let reg = get_yank_register(name, YREG_YANK);
        if !is_append_register(name) && !must_append {
            free_register(reg);
        }
        reg
    }
}

/// How many lines `str` will become, and whether the first of them joins the
/// register's current last line.
///
/// `extraline` is the line a string not ending in a newline still contributes.
///
/// # Safety
/// `str` must hold `len` bytes, or -- with `str_list` -- be a
/// null-terminated array of NUL-terminated strings.
unsafe fn count_lines(
    y_ptr: *mut yankreg_T,
    yank_type: MotionType,
    str: *const c_char,
    len: size_t,
    str_list: bool,
) -> (size_t, bool, bool) {
    unsafe {
        if str_list {
            let mut newlines: size_t = 0;
            let mut ss = str as *mut *mut c_char;
            while !(*ss).is_null() {
                newlines = newlines.wrapping_add(1);
                ss = ss.add(1);
            }
            return (newlines, false, false);
        }

        let mut newlines = memcnt(str as *const c_void, '\n' as c_char, len);
        let mut extraline = false;
        if yank_type == kMTCharWise
            || len == 0
            || c_int::from(*str.add(len.wrapping_sub(1))) != '\n' as c_int
        {
            extraline = true;
            newlines = newlines.wrapping_add(1);
        }
        // A charwise register's last line has no newline of its own, so the
        // first new line continues it.
        let append = (*y_ptr).y_size > 0 && (*y_ptr).y_type == kMTCharWise;
        if append {
            newlines = newlines.wrapping_sub(1);
        }
        (newlines, extraline, append)
    }
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
    unsafe {
        if (*y_ptr).y_array.is_null() {
            (*y_ptr).y_size = 0;
        }
        if yank_type == kMTUnknown {
            yank_type = if str_list
                || len > 0
                    && (c_int::from(*str.add(len.wrapping_sub(1))) == NL
                        || c_int::from(*str.add(len.wrapping_sub(1))) == CAR)
            {
                kMTLineWise
            } else {
                kMTCharWise
            };
        }

        let (newlines, extraline, mut append) = count_lines(y_ptr, yank_type, str, len, str_list);
        if (*y_ptr).y_size.wrapping_add(newlines) == 0 {
            xfree((*y_ptr).y_array as *mut c_void);
            (*y_ptr).y_array = ::core::ptr::null_mut();
            return;
        }

        let pp = xrealloc(
            (*y_ptr).y_array as *mut c_void,
            (*y_ptr)
                .y_size
                .wrapping_add(newlines)
                .wrapping_mul(::core::mem::size_of::<String_0>()),
        ) as *mut String_0;
        (*y_ptr).y_array = pp;

        let mut lnum = (*y_ptr).y_size;
        let mut maxlen: size_t = 0;
        if str_list {
            let mut ss = str as *mut *mut c_char;
            while !(*ss).is_null() {
                *pp.add(lnum) = cstr_to_string(*ss);
                if yank_type == kMTBlockWise {
                    maxlen = maxlen.max(mb_string2cells(*ss));
                }
                ss = ss.add(1);
                lnum = lnum.wrapping_add(1);
            }
        } else {
            let mut start = str;
            let end = str.add(len);
            // `+ extraline` so that a string with no trailing newline still
            // produces its last (possibly empty) line.
            while start < end.offset(extraline as c_int as isize) {
                let mut charlen = 0;
                let mut line_end = start;
                while line_end < end {
                    if c_int::from(*line_end) == '\n' as c_int {
                        break;
                    }
                    if yank_type == kMTBlockWise {
                        charlen += utf_ptr2cells_len(line_end, end.offset_from(line_end) as c_int);
                    }
                    // A NUL is one byte and is translated below, so it must
                    // not be handed to the UTF-8 length.
                    if c_int::from(*line_end) == NUL {
                        line_end = line_end.add(1);
                    } else {
                        line_end = line_end.offset(utf_ptr2len_len(
                            line_end,
                            end.offset_from(line_end) as c_int,
                        ) as isize);
                    }
                }
                debug_assert!(line_end.offset_from(start) >= 0);
                let line_len = line_end.offset_from(start) as size_t;
                maxlen = maxlen.max(charlen as size_t);

                // Appending continues the register's current last line.
                let extra = if append {
                    lnum = lnum.wrapping_sub(1);
                    (*pp.add(lnum)).size
                } else {
                    0
                };
                let s = xmallocz(line_len.wrapping_add(extra)) as *mut c_char;
                if extra > 0 {
                    memcpy(
                        s as *mut c_void,
                        (*pp.add(lnum)).data as *const c_void,
                        extra,
                    );
                }
                if line_len > 0 {
                    memcpy(
                        s.add(extra) as *mut c_void,
                        start as *const c_void,
                        line_len,
                    );
                }
                let s_len = extra.wrapping_add(line_len);
                if append {
                    xfree((*pp.add(lnum)).data as *mut c_void);
                    append = false;
                }
                *pp.add(lnum) = String_0 {
                    data: s,
                    size: s_len,
                };
                // A NUL in the text is how the editor spells a newline.
                memchrsub(s as *mut c_void, NUL as c_char, '\n' as c_char, s_len);

                start = start.add(line_len.wrapping_add(1));
                lnum = lnum.wrapping_add(1);
            }
        }

        (*y_ptr).y_type = yank_type;
        (*y_ptr).y_size = lnum;
        xfree((*y_ptr).additional_data as *mut c_void);
        (*y_ptr).additional_data = ::core::ptr::null_mut();
        (*y_ptr).timestamp = os_time();
        (*y_ptr).y_width = if yank_type == kMTBlockWise {
            if blocklen == -1 {
                maxlen as c_int - 1
            } else {
                blocklen
            }
        } else {
            0
        };
    }
}

/// Push the write out to the clipboard and put `""` back where it was.
///
/// # Safety
/// `reg` must be the register [`init_write_reg`] answered.
unsafe fn finish_write_reg(name: c_int, reg: *mut yankreg_T, old_y_previous: *mut yankreg_T) {
    unsafe {
        clipboard::set_clipboard(name, reg);
        // Only an explicit write to `""` moves it.
        if name != '"' as c_int {
            y_previous.set(old_y_previous);
        }
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
    unsafe {
        write_reg_contents_ex(name, str, len, must_append != 0, kMTUnknown, 0);
    }
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
    unsafe {
        if name == '/' as c_int || name == '=' as c_int {
            // Neither register can hold more than one line.
            let s = if (*strings).is_null() {
                c"".as_ptr().cast_mut()
            } else if !(*strings.add(1)).is_null() {
                emsg(gettext(
                    e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines
                        .as_ptr(),
                ));
                return;
            } else {
                *strings
            };
            write_reg_contents_ex(name, s, -1, must_append, yank_type, block_len);
            return;
        }
        if name == '_' as c_int {
            return; // black hole
        }

        let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut();
        let reg = init_write_reg(name, &raw mut old_y_previous, must_append);
        if reg.is_null() {
            return;
        }
        // The length is meaningless for a list write and `str_to_reg` ignores
        // it; upstream passes `strlen((char *)strings)` all the same.
        str_to_reg(
            reg,
            yank_type,
            strings as *mut c_char,
            strlen(strings as *mut c_char),
            block_len,
            true,
        );
        finish_write_reg(name, reg, old_y_previous);
    }
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
    unsafe {
        let len = if len < 0 { strlen(str) as ssize_t } else { len };

        if name == '/' as c_int {
            set_last_search_pat(str, RE_SEARCH as c_int, true, true);
            return;
        }

        if name == '#' as c_int {
            // The alternate file, by number or by name.
            let buf = if ascii_isdigit(c_int::from(*str)) {
                let num = atoi(str);
                let buf = buflist_findnr(num);
                if buf.is_null() {
                    semsg_c!(
                        gettext(&raw const e_nobufnr as *const c_char),
                        int64_t::from(num),
                    );
                }
                buf
            } else {
                buflist_findnr(buflist_findpat(
                    str,
                    str.offset(len as isize),
                    true,
                    false,
                    false,
                ))
            };
            if !buf.is_null() {
                (*curwin.get()).w_alt_fnum = (*buf).handle;
            }
            return;
        }

        if name == '=' as c_int {
            // The expression register keeps its source, not a yankreg.
            let mut offset: size_t = 0;
            let mut totlen = len as size_t;
            if must_append && !expr_line.get().is_null() {
                let exprlen = strlen(expr_line.get());
                totlen = totlen.wrapping_add(exprlen);
                offset = exprlen;
            }
            expr_line.set(
                xrealloc(expr_line.get() as *mut c_void, totlen.wrapping_add(1)) as *mut c_char,
            );
            memcpy(
                expr_line.get().add(offset) as *mut c_void,
                str as *const c_void,
                len as size_t,
            );
            *expr_line.get().add(totlen) = NUL as c_char;
            return;
        }

        if name == '_' as c_int {
            return; // black hole
        }

        let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut();
        let reg = init_write_reg(name, &raw mut old_y_previous, must_append);
        if reg.is_null() {
            return;
        }
        str_to_reg(reg, yank_type, str, len as size_t, block_len, false);
        finish_write_reg(name, reg, old_y_previous);
    }
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
    unsafe {
        let type_0 = if regtype.data.is_null() {
            NUL
        } else {
            c_int::from(*regtype.data)
        };
        (*reg).y_type = match type_0 {
            0 => kMTUnknown, // "" means "work it out from the text"
            c if c == 'v' as c_int || c == 'c' as c_int => kMTCharWise,
            c if c == 'V' as c_int || c == 'l' as c_int => kMTLineWise,
            c if c == 'b' as c_int || c == Ctrl_V => kMTBlockWise,
            _ => return false,
        };

        (*reg).y_width = 0;
        if regtype.size > 1 {
            // A width only means something for a block.
            if (*reg).y_type != kMTBlockWise {
                return false;
            }
            if !ascii_isdigit(c_int::from(*regtype.data.add(1))) {
                return false;
            }
            let mut p: *const c_char = regtype.data.add(1);
            (*reg).y_width = getdigits_int(&raw mut p as *mut *mut c_char, false, 1) - 1;
            // Nothing may follow the width.
            if regtype.size > p.offset_from(regtype.data) as size_t {
                return false;
            }
        }
        (*reg).additional_data = ::core::ptr::null_mut();
        (*reg).timestamp = 0;
        true
    }
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
    unsafe {
        let ends_empty =
            (*reg).y_size > 0 && (*(*reg).y_array.add((*reg).y_size.wrapping_sub(1))).size == 0;
        if ends_empty {
            if (*reg).y_type != kMTCharWise {
                if (*reg).y_type == kMTUnknown || clipboard_adjust {
                    (*reg).y_size = (*reg).y_size.wrapping_sub(1);
                }
                if (*reg).y_type == kMTUnknown {
                    (*reg).y_type = kMTLineWise;
                }
            }
        } else if (*reg).y_type == kMTUnknown {
            (*reg).y_type = kMTCharWise;
        }
        update_yankreg_width(reg);
    }
}
