//! A register as text: the API's way in and out.
//!
//! `get_reg_contents` and `write_reg_contents*` are what `getreg()`,
//! `setreg()`, `nvim_paste`, the clipboard provider and shada all use, so
//! these are the functions that have to turn a `yankreg_T` into lines or a
//! string and back.  `str_to_reg` is the interesting direction: it splits an
//! incoming string on newlines, treats an embedded NUL as a newline the way
//! the rest of the editor does, and for a blockwise write has to measure the
//! widest line in *cells* rather than bytes.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_reg_type(
    mut regname: ::core::ffi::c_int,
    mut reg_width: *mut colnr_T,
) -> MotionType {
    unsafe {
        's_19: {
            'c_46756: {
                'c_46754: {
                    'c_46752: {
                        'c_46750: {
                            'c_46748: {
                                'c_46746: {
                                    'c_46744: {
                                        'c_46742: {
                                            match regname {
                                                35 => {}
                                                61 => {}
                                                58 => {
                                                    break 'c_46742;
                                                }
                                                47 => {
                                                    break 'c_46744;
                                                }
                                                46 => {
                                                    break 'c_46746;
                                                }
                                                Ctrl_F => {
                                                    break 'c_46748;
                                                }
                                                Ctrl_P => {
                                                    break 'c_46750;
                                                }
                                                Ctrl_W => {
                                                    break 'c_46752;
                                                }
                                                Ctrl_A => {
                                                    break 'c_46754;
                                                }
                                                37 | 95 => {
                                                    break 'c_46756;
                                                }
                                                _ => {
                                                    break 's_19;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return kMTCharWise;
        }
        if regname != NUL && !valid_yank_reg(regname, false) {
            return kMTUnknown;
        }
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE);
        if !(*reg).y_array.is_null() {
            if !reg_width.is_null()
                && (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
            {
                *reg_width = (*reg).y_width;
            }
            return (*reg).y_type;
        }
        return kMTUnknown;
    }
}

unsafe extern "C" fn get_reg_wrap_one_line(
    mut s: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    unsafe {
        if flags & kGRegList as ::core::ffi::c_int == 0 {
            return s as *mut ::core::ffi::c_void;
        }
        let list: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
        tv_list_append_allocated_string(list, s);
        return list as *mut ::core::ffi::c_void;
    }
}

pub unsafe extern "C" fn get_reg_contents(
    mut regname: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    unsafe {
        if regname == '=' as ::core::ffi::c_int {
            if flags & kGRegNoExpr as ::core::ffi::c_int != 0 {
                return NULL_0;
            }
            if flags & kGRegExprSrc as ::core::ffi::c_int != 0 {
                return get_reg_wrap_one_line(get_expr_line_src(), flags);
            }
            return get_reg_wrap_one_line(get_expr_line(), flags);
        }
        if regname == '@' as ::core::ffi::c_int {
            regname = '"' as ::core::ffi::c_int;
        }
        if regname != NUL && !valid_yank_reg(regname, false) {
            return NULL_0;
        }
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut allocated: bool = false;
        if get_spec_reg(regname, &raw mut retval, &raw mut allocated, false) {
            if retval.is_null() {
                return NULL_0;
            }
            if allocated {
                return get_reg_wrap_one_line(retval, flags);
            }
            return get_reg_wrap_one_line(xstrdup(retval), flags);
        }
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PUT);
        if (*reg).y_array.is_null() {
            return NULL_0;
        }
        if flags & kGRegList as ::core::ffi::c_int != 0 {
            let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
            let mut i: size_t = 0 as size_t;
            while i < (*reg).y_size {
                tv_list_append_string(
                    list,
                    (*(*reg).y_array.offset(i as isize)).data,
                    (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
                );
                i = i.wrapping_add(1);
            }
            return list as *mut ::core::ffi::c_void;
        }
        let mut len: size_t = 0 as size_t;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*reg).y_size {
            len = len.wrapping_add((*(*reg).y_array.offset(i_0 as isize)).size);
            if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                || i_0 < (*reg).y_size.wrapping_sub(1 as size_t)
            {
                len = len.wrapping_add(1);
            }
            i_0 = i_0.wrapping_add(1);
        }
        retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        len = 0 as size_t;
        let mut i_1: size_t = 0 as size_t;
        while i_1 < (*reg).y_size {
            strcpy(
                retval.offset(len as isize),
                (*(*reg).y_array.offset(i_1 as isize)).data,
            );
            len = len.wrapping_add((*(*reg).y_array.offset(i_1 as isize)).size);
            if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                || i_1 < (*reg).y_size.wrapping_sub(1 as size_t)
            {
                let c2rust_fresh5 = len;
                len = len.wrapping_add(1);
                *retval.offset(c2rust_fresh5 as isize) = '\n' as ::core::ffi::c_char;
            }
            i_1 = i_1.wrapping_add(1);
        }
        *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
        return retval as *mut ::core::ffi::c_void;
    }
}

unsafe extern "C" fn init_write_reg(
    mut name: ::core::ffi::c_int,
    mut old_y_previous: *mut *mut yankreg_T,
    mut must_append: bool,
) -> *mut yankreg_T {
    unsafe {
        if !valid_yank_reg(name, true) {
            emsg_invreg(name);
            return ::core::ptr::null_mut::<yankreg_T>();
        }
        *old_y_previous = y_previous.get();
        let mut reg: *mut yankreg_T = get_yank_register(name, YREG_YANK);
        if !is_append_register(name) && !must_append {
            free_register(reg);
        }
        return reg;
    }
}

unsafe extern "C" fn str_to_reg(
    mut y_ptr: *mut yankreg_T,
    mut yank_type: MotionType,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    mut blocklen: colnr_T,
    mut str_list: bool,
) {
    unsafe {
        if (*y_ptr).y_array.is_null() {
            (*y_ptr).y_size = 0 as size_t;
        }
        if yank_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
            yank_type = (if str_list as ::core::ffi::c_int != 0
                || len > 0 as size_t
                    && (*str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == NL
                        || *str.offset(len.wrapping_sub(1 as size_t) as isize)
                            as ::core::ffi::c_int
                            == CAR)
            {
                kMTLineWise as ::core::ffi::c_int
            } else {
                kMTCharWise as ::core::ffi::c_int
            }) as MotionType;
        }
        let mut newlines: size_t = 0 as size_t;
        let mut extraline: bool = false;
        let mut append: bool = false;
        if str_list {
            let mut ss: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
            while !(*ss).is_null() {
                newlines = newlines.wrapping_add(1);
                ss = ss.offset(1);
            }
        } else {
            newlines = memcnt(
                str as *const ::core::ffi::c_void,
                '\n' as ::core::ffi::c_char,
                len,
            );
            if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                || len == 0 as size_t
                || *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\n' as ::core::ffi::c_int
            {
                extraline = true;
                newlines = newlines.wrapping_add(1);
            }
            if (*y_ptr).y_size > 0 as size_t
                && (*y_ptr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            {
                append = true;
                newlines = newlines.wrapping_sub(1);
            }
        }
        if (*y_ptr).y_size.wrapping_add(newlines) == 0 as size_t {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*y_ptr).y_array as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            return;
        }
        let mut pp: *mut String_0 = xrealloc(
            (*y_ptr).y_array as *mut ::core::ffi::c_void,
            (*y_ptr)
                .y_size
                .wrapping_add(newlines)
                .wrapping_mul(::core::mem::size_of::<String_0>()),
        ) as *mut String_0;
        (*y_ptr).y_array = pp;
        let mut lnum: size_t = (*y_ptr).y_size;
        let mut maxlen: size_t = 0 as size_t;
        if str_list {
            let mut ss_0: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
            while !(*ss_0).is_null() {
                *pp.offset(lnum as isize) = cstr_to_string(*ss_0);
                if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    let mut charlen: size_t = mb_string2cells(*ss_0);
                    maxlen = if maxlen > charlen { maxlen } else { charlen };
                }
                ss_0 = ss_0.offset(1);
                lnum = lnum.wrapping_add(1);
            }
        } else {
            let mut line_len: size_t = 0;
            let mut start: *const ::core::ffi::c_char = str;
            let mut end: *const ::core::ffi::c_char = str.offset(len as isize);
            while start < end.offset(extraline as ::core::ffi::c_int as isize) {
                let mut charlen_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut line_end: *const ::core::ffi::c_char = start;
                while line_end < end {
                    if *line_end as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                        break;
                    }
                    if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                        charlen_0 += utf_ptr2cells_len(
                            line_end,
                            end.offset_from(line_end) as ::core::ffi::c_int,
                        );
                    }
                    if *line_end as ::core::ffi::c_int == NUL {
                        line_end = line_end.offset(1);
                    } else {
                        line_end = line_end.offset(utf_ptr2len_len(
                            line_end,
                            end.offset_from(line_end) as ::core::ffi::c_int,
                        ) as isize);
                    }
                }
                '_c2rust_label: {
                    if line_end.offset_from(start) >= 0 as isize {
                    } else {
                        __assert_fail(
                        c"line_end - start >= 0".as_ptr(),
                        c"src/nvim/register.rs".as_ptr(),
                        2491 as ::core::ffi::c_uint,
                        c"void str_to_reg(yankreg_T *, MotionType, const char *, size_t, colnr_T, _Bool)".as_ptr(),
                    );
                    }
                };
                line_len = line_end.offset_from(start) as size_t;
                maxlen = if maxlen > charlen_0 as size_t {
                    maxlen
                } else {
                    charlen_0 as size_t
                };
                let mut extra: size_t = if append as ::core::ffi::c_int != 0 {
                    lnum = lnum.wrapping_sub(1);
                    (*pp.offset(lnum as isize)).size
                } else {
                    0 as size_t
                };
                let mut s: *mut ::core::ffi::c_char =
                    xmallocz(line_len.wrapping_add(extra)) as *mut ::core::ffi::c_char;
                if extra > 0 as size_t {
                    memcpy(
                        s as *mut ::core::ffi::c_void,
                        (*pp.offset(lnum as isize)).data as *const ::core::ffi::c_void,
                        extra,
                    );
                }
                if line_len > 0 as size_t {
                    memcpy(
                        s.offset(extra as isize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        line_len,
                    );
                }
                let mut s_len: size_t = extra.wrapping_add(line_len);
                if append {
                    xfree((*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void);
                    append = false;
                }
                *pp.offset(lnum as isize) = String_0 {
                    data: s,
                    size: s_len,
                };
                memchrsub(
                    (*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void,
                    NUL as ::core::ffi::c_char,
                    '\n' as ::core::ffi::c_char,
                    s_len,
                );
                start = start.offset(line_len.wrapping_add(1 as size_t) as isize);
                lnum = lnum.wrapping_add(1);
            }
        }
        (*y_ptr).y_type = yank_type;
        (*y_ptr).y_size = lnum;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*y_ptr).additional_data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        (*y_ptr).timestamp = os_time();
        if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            (*y_ptr).y_width = (if blocklen == -1 as ::core::ffi::c_int {
                maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                blocklen as ::core::ffi::c_int
            }) as colnr_T;
        } else {
            (*y_ptr).y_width = 0 as ::core::ffi::c_int as colnr_T;
        };
    }
}

unsafe extern "C" fn finish_write_reg(
    mut name: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut old_y_previous: *mut yankreg_T,
) {
    unsafe {
        clipboard::set_clipboard(name, reg);
        if name != '"' as ::core::ffi::c_int {
            y_previous.set(old_y_previous);
        }
    }
}

pub unsafe extern "C" fn write_reg_contents(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: ::core::ffi::c_int,
) {
    unsafe {
        write_reg_contents_ex(name, str, len, must_append != 0, kMTUnknown, 0 as colnr_T);
    }
}

pub unsafe extern "C" fn write_reg_contents_lst(
    mut name: ::core::ffi::c_int,
    mut strings: *mut *mut ::core::ffi::c_char,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    unsafe {
        if name == '/' as ::core::ffi::c_int || name == '=' as ::core::ffi::c_int {
            let mut s: *mut ::core::ffi::c_char = *strings.offset(0 as ::core::ffi::c_int as isize);
            if (*strings.offset(0 as ::core::ffi::c_int as isize)).is_null() {
                s = c"".as_ptr() as *mut ::core::ffi::c_char;
            } else if !(*strings.offset(1 as ::core::ffi::c_int as isize)).is_null() {
                emsg(gettext(
                    (e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines
                        .ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                return;
            }
            write_reg_contents_ex(name, s, -1 as ssize_t, must_append, yank_type, block_len);
            return;
        }
        if name == '_' as ::core::ffi::c_int {
            return;
        }
        let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        reg = init_write_reg(name, &raw mut old_y_previous, must_append);
        if reg.is_null() {
            return;
        }
        str_to_reg(
            reg,
            yank_type,
            strings as *mut ::core::ffi::c_char,
            strlen(strings as *mut ::core::ffi::c_char),
            block_len,
            true,
        );
        finish_write_reg(name, reg, old_y_previous);
    }
}

pub unsafe extern "C" fn write_reg_contents_ex(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    unsafe {
        if len < 0 as ssize_t {
            len = strlen(str) as ssize_t;
        }
        if name == '/' as ::core::ffi::c_int {
            set_last_search_pat(str, RE_SEARCH as ::core::ffi::c_int, true, true);
            return;
        }
        if name == '#' as ::core::ffi::c_int {
            let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
            if ascii_isdigit(*str as ::core::ffi::c_int) {
                let mut num: ::core::ffi::c_int = atoi(str);
                buf = buflist_findnr(num);
                if buf.is_null() {
                    semsg(
                        gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                        num as int64_t,
                    );
                }
            } else {
                buf = buflist_findnr(buflist_findpat(
                    str,
                    str.offset(len as isize),
                    true,
                    false,
                    false,
                ));
            }
            if buf.is_null() {
                return;
            }
            (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
            return;
        }
        if name == '=' as ::core::ffi::c_int {
            let mut offset: size_t = 0 as size_t;
            let mut totlen: size_t = len as size_t;
            if must_append as ::core::ffi::c_int != 0 && !(*expr_line.ptr()).is_null() {
                let mut exprlen: size_t = strlen(expr_line.get());
                totlen = totlen.wrapping_add(exprlen);
                offset = exprlen;
            }
            expr_line.set(xrealloc(
                expr_line.get() as *mut ::core::ffi::c_void,
                totlen.wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char);
            memcpy(
                (*expr_line.ptr()).offset(offset as isize) as *mut ::core::ffi::c_void,
                str as *const ::core::ffi::c_void,
                len as size_t,
            );
            *(*expr_line.ptr()).offset(totlen as isize) = NUL as ::core::ffi::c_char;
            return;
        }
        if name == '_' as ::core::ffi::c_int {
            return;
        }
        let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        reg = init_write_reg(name, &raw mut old_y_previous, must_append);
        if reg.is_null() {
            return;
        }
        str_to_reg(reg, yank_type, str, len as size_t, block_len, false);
        finish_write_reg(name, reg, old_y_previous);
    }
}

pub unsafe extern "C" fn prepare_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut regtype: String_0,
    mut _lines: size_t,
) -> bool {
    unsafe {
        let mut type_0: ::core::ffi::c_char = (if !regtype.data.is_null() {
            *regtype.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            NUL
        }) as ::core::ffi::c_char;
        match type_0 as ::core::ffi::c_int {
            0 => {
                (*reg).y_type = kMTUnknown;
            }
            118 | 99 => {
                (*reg).y_type = kMTCharWise;
            }
            86 | 108 => {
                (*reg).y_type = kMTLineWise;
            }
            98 | Ctrl_V => {
                (*reg).y_type = kMTBlockWise;
            }
            _ => return false,
        }
        (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
        if regtype.size > 1 as size_t {
            if (*reg).y_type as ::core::ffi::c_int != kMTBlockWise as ::core::ffi::c_int {
                return false;
            }
            if !ascii_isdigit(
                *regtype.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) {
                return false;
            }
            let mut p: *const ::core::ffi::c_char =
                regtype.data.offset(1 as ::core::ffi::c_int as isize);
            (*reg).y_width = (getdigits_int(
                &raw mut p as *mut *mut ::core::ffi::c_char,
                false,
                1 as ::core::ffi::c_int,
            ) - 1 as ::core::ffi::c_int) as colnr_T;
            if regtype.size > p.offset_from(regtype.data) as size_t {
                return false;
            }
        }
        (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        (*reg).timestamp = 0 as Timestamp;
        return true;
    }
}

pub unsafe extern "C" fn finish_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut clipboard_adjust: bool,
) {
    unsafe {
        if (*reg).y_size > 0 as size_t
            && (*(*reg)
                .y_array
                .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize))
            .size
                == 0 as size_t
        {
            if (*reg).y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int {
                if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int
                    || clipboard_adjust as ::core::ffi::c_int != 0
                {
                    (*reg).y_size = (*reg).y_size.wrapping_sub(1);
                }
                if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
                    (*reg).y_type = kMTLineWise;
                }
            }
        } else if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
            (*reg).y_type = kMTCharWise;
        }
        update_yankreg_width(reg);
    }
}
