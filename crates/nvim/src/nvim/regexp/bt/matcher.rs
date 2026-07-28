//! The backtracking match loop itself.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regmatch(
    mut scan: *mut uint8_t,
    mut tm: *const proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> bool {
    let mut next: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut op: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut rp: *mut regitem_T = ::core::ptr::null_mut::<regitem_T>();
    let mut no: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut tm_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*regstack.ptr()).ga_len = 0 as ::core::ffi::c_int;
    (*backpos.ptr()).ga_len = 0 as ::core::ffi::c_int;
    loop {
        reg_breakcheck();
        loop {
            if got_int.get() as ::core::ffi::c_int != 0 || scan.is_null() {
                status = RA_FAIL;
                break;
            } else {
                if !tm.is_null() && {
                    tm_count += 1;
                    tm_count == 100 as ::core::ffi::c_int
                } {
                    tm_count = 0 as ::core::ffi::c_int;
                    if profile_passed_limit(*tm) {
                        if !timed_out.is_null() {
                            *timed_out = true_0;
                        }
                        status = RA_FAIL;
                        break;
                    }
                }
                status = RA_CONT;
                next = regnext(scan);
                op = *scan as ::core::ffi::c_int;
                if !(*rex.ptr()).reg_line_lbr
                    && (op >= FIRST_NL && op <= LAST_NL)
                    && (*rex.ptr()).reg_match.is_null()
                    && *(*rex.ptr()).input as ::core::ffi::c_int == NUL
                    && (*rex.ptr()).lnum <= (*rex.ptr()).reg_maxline
                {
                    reg_nextline();
                } else if (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                    && (op >= FIRST_NL && op <= LAST_NL)
                    && *(*rex.ptr()).input as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                {
                    (*rex.ptr()).input =
                        (*rex.ptr())
                            .input
                            .offset(utfc_ptr2len((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                as isize);
                } else {
                    if op >= FIRST_NL && op <= LAST_NL {
                        op -= ADD_NL;
                    }
                    c = utf_ptr2char((*rex.ptr()).input as *mut ::core::ffi::c_char);
                    's_2509: {
                        match op {
                            BOL => {
                                if (*rex.ptr()).input != (*rex.ptr()).line {
                                    status = RA_NOMATCH;
                                }
                            }
                            EOL => {
                                if c != NUL {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_BOF => {
                                if (*rex.ptr()).lnum != 0 as linenr_T
                                    || (*rex.ptr()).input != (*rex.ptr()).line
                                    || (*rex.ptr()).reg_match.is_null()
                                        && (*rex.ptr()).reg_firstlnum > 1 as linenr_T
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_EOF => {
                                if (*rex.ptr()).lnum != (*rex.ptr()).reg_maxline || c != NUL {
                                    status = RA_NOMATCH;
                                }
                            }
                            CURSOR => {
                                if (*rex.ptr()).reg_win.is_null()
                                    || (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                        != (*(*rex.ptr()).reg_win).w_cursor.lnum
                                    || (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T
                                        != (*(*rex.ptr()).reg_win).w_cursor.col
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_MARK => {
                                let mut mark: ::core::ffi::c_int = *scan
                                    .offset(3 as ::core::ffi::c_int as isize)
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int;
                                let mut cmp: ::core::ffi::c_int = *scan
                                    .offset(3 as ::core::ffi::c_int as isize)
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int;
                                let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
                                let mut col: size_t = if (*rex.ptr()).reg_match.is_null() {
                                    (*rex.ptr()).input.offset_from((*rex.ptr()).line) as size_t
                                } else {
                                    0 as size_t
                                };
                                let mut fm: *mut fmark_T = mark_get(
                                    (*rex.ptr()).reg_buf,
                                    curwin.get(),
                                    ::core::ptr::null_mut::<fmark_T>(),
                                    kMarkBufLocal,
                                    mark,
                                );
                                if (*rex.ptr()).reg_match.is_null() {
                                    (*rex.ptr()).line =
                                        reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                                    (*rex.ptr()).input = (*rex.ptr()).line.offset(col as isize);
                                }
                                if fm.is_null() || (*fm).mark.lnum <= 0 as linenr_T {
                                    status = RA_NOMATCH;
                                } else {
                                    pos = &raw mut (*fm).mark;
                                    let pos_col: colnr_T = if (*pos).lnum
                                        == (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                        && (*pos).col == MAXCOL as ::core::ffi::c_int
                                    {
                                        reg_getline_len((*pos).lnum - (*rex.ptr()).reg_firstlnum)
                                    } else {
                                        (*pos).col
                                    };
                                    if if (*pos).lnum
                                        == (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                    {
                                        if pos_col
                                            == (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                as colnr_T
                                        {
                                            (cmp == '<' as ::core::ffi::c_int
                                                || cmp == '>' as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else if pos_col
                                            < (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                as colnr_T
                                        {
                                            (cmp != '>' as ::core::ffi::c_int) as ::core::ffi::c_int
                                        } else {
                                            (cmp != '<' as ::core::ffi::c_int) as ::core::ffi::c_int
                                        }
                                    } else if (*pos).lnum
                                        < (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                    {
                                        (cmp != '>' as ::core::ffi::c_int) as ::core::ffi::c_int
                                    } else {
                                        (cmp != '<' as ::core::ffi::c_int) as ::core::ffi::c_int
                                    } != 0
                                    {
                                        status = RA_NOMATCH;
                                    }
                                }
                            }
                            RE_VISUAL => {
                                if !reg_match_visual() {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_LNUM => {
                                '_c2rust_label: {
                                    if (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                        >= 0 as linenr_T
                                        && ((*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum)
                                            as uintmax_t
                                            <= 4294967295 as uintmax_t
                                    {
                                    } else {
                                        __assert_fail(
                                            b"rex.lnum + rex.reg_firstlnum >= 0 && (uintmax_t)(rex.lnum + rex.reg_firstlnum) <= UINT32_MAX\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/regexp.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            6288 as ::core::ffi::c_uint,
                                            b"_Bool regmatch(uint8_t *, const proftime_T *, int *)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if !(*rex.ptr()).reg_match.is_null()
                                    || re_num_cmp(
                                        ((*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum)
                                            as uint32_t,
                                        scan,
                                    ) == 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_COL => {
                                '_c2rust_label_0: {
                                    if (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                        + 1 as isize
                                        >= 0 as isize
                                        && ((*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                            + 1 as isize)
                                            as uintmax_t
                                            <= 4294967295 as uintmax_t
                                    {
                                    } else {
                                        __assert_fail(
                                            b"rex.input - rex.line + 1 >= 0 && (uintmax_t)(rex.input - rex.line + 1) <= UINT32_MAX\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/regexp.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            6297 as ::core::ffi::c_uint,
                                            b"_Bool regmatch(uint8_t *, const proftime_T *, int *)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if re_num_cmp(
                                    ((*rex.ptr()).input.offset_from((*rex.ptr()).line) + 1 as isize)
                                        as uint32_t,
                                    scan,
                                ) == 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            RE_VCOL => {
                                let mut wp: *mut win_T = if (*rex.ptr()).reg_win.is_null() {
                                    curwin.get()
                                } else {
                                    (*rex.ptr()).reg_win
                                };
                                let mut lnum: linenr_T = if (*rex.ptr()).reg_match.is_null() {
                                    (*rex.ptr()).reg_firstlnum + (*rex.ptr()).lnum
                                } else {
                                    1 as linenr_T
                                };
                                if (*rex.ptr()).reg_match.is_null()
                                    && (lnum <= 0 as linenr_T
                                        || lnum > (*(*wp).w_buffer).b_ml.ml_line_count)
                                {
                                    lnum = 1 as ::core::ffi::c_int as linenr_T;
                                }
                                let mut vcol: ::core::ffi::c_int = win_linetabsize(
                                    wp,
                                    lnum,
                                    (*rex.ptr()).line as *mut ::core::ffi::c_char,
                                    (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T,
                                );
                                if re_num_cmp((vcol as uint32_t).wrapping_add(1 as uint32_t), scan)
                                    == 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            BOW => {
                                if c == NUL {
                                    status = RA_NOMATCH;
                                } else {
                                    let this_class: ::core::ffi::c_int = mb_get_class_tab(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut uint64_t,
                                    );
                                    if this_class <= 1 as ::core::ffi::c_int {
                                        status = RA_NOMATCH;
                                    } else if reg_prev_class() == this_class {
                                        status = RA_NOMATCH;
                                    }
                                }
                            }
                            EOW => {
                                if (*rex.ptr()).input == (*rex.ptr()).line {
                                    status = RA_NOMATCH;
                                } else {
                                    let mut this_class_0: ::core::ffi::c_int = 0;
                                    let mut prev_class: ::core::ffi::c_int = 0;
                                    this_class_0 = mb_get_class_tab(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut uint64_t,
                                    );
                                    prev_class = reg_prev_class();
                                    if this_class_0 == prev_class
                                        || prev_class == 0 as ::core::ffi::c_int
                                        || prev_class == 1 as ::core::ffi::c_int
                                    {
                                        status = RA_NOMATCH;
                                    }
                                }
                            }
                            ANY => {
                                if c == NUL {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            IDENT => {
                                if !vim_isIDc(c) {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            SIDENT => {
                                if ascii_isdigit(*(*rex.ptr()).input as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    || !vim_isIDc(c)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            KWORD => {
                                if !vim_iswordp_buf(
                                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                    (*rex.ptr()).reg_buf,
                                ) {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            SKWORD => {
                                if ascii_isdigit(*(*rex.ptr()).input as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    || !vim_iswordp_buf(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        (*rex.ptr()).reg_buf,
                                    )
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            FNAME => {
                                if !vim_isfilec(c) {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            SFNAME => {
                                if ascii_isdigit(*(*rex.ptr()).input as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    || !vim_isfilec(c)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            PRINT => {
                                if !vim_isprintc(utf_ptr2char(
                                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                )) {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            SPRINT => {
                                if ascii_isdigit(*(*rex.ptr()).input as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    || !vim_isprintc(utf_ptr2char(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                    ))
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            WHITE => {
                                if !ascii_iswhite(c) {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NWHITE => {
                                if c == NUL || ascii_iswhite(c) as ::core::ffi::c_int != 0 {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            DIGIT => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_DIGIT
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NDIGIT => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_DIGIT
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            HEX => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_HEX
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NHEX => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_HEX
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            OCTAL => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_OCTAL
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NOCTAL => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_OCTAL
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            WORD => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_WORD
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NWORD => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_WORD
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            HEAD => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_HEAD
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NHEAD => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_HEAD
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            ALPHA => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_ALPHA
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NALPHA => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_ALPHA
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            LOWER => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_LOWER
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NLOWER => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_LOWER
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            UPPER => {
                                if !(c < 0x100 as ::core::ffi::c_int
                                    && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                        & RI_UPPER
                                        != 0)
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NUPPER => {
                                if c == NUL
                                    || c < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[c as usize] as ::core::ffi::c_int
                                            & RI_UPPER
                                            != 0
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            EXACTLY => {
                                let mut len: ::core::ffi::c_int = 0;
                                let mut opnd: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                                opnd = scan.offset(3 as ::core::ffi::c_int as isize);
                                if *opnd as ::core::ffi::c_int
                                    != *(*rex.ptr()).input as ::core::ffi::c_int
                                    && !(*rex.ptr()).reg_ic
                                {
                                    status = RA_NOMATCH;
                                } else if *opnd as ::core::ffi::c_int != NUL {
                                    if *opnd.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == NUL
                                        && !(*rex.ptr()).reg_ic
                                    {
                                        len = 1 as ::core::ffi::c_int;
                                    } else {
                                        len = strlen(opnd as *mut ::core::ffi::c_char)
                                            as ::core::ffi::c_int;
                                        if cstrncmp(
                                            opnd as *mut ::core::ffi::c_char,
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            &raw mut len,
                                        ) != 0 as ::core::ffi::c_int
                                        {
                                            status = RA_NOMATCH;
                                        }
                                    }
                                    if status != RA_NOMATCH
                                        && utf_composinglike(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                .offset(len as isize),
                                            ::core::ptr::null_mut::<GraphemeState>(),
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                        && !(*rex.ptr()).reg_icombine
                                        && *next as ::core::ffi::c_int != RE_COMPOSING
                                    {
                                        status = RA_NOMATCH;
                                    }
                                    if status != RA_NOMATCH {
                                        (*rex.ptr()).input =
                                            (*rex.ptr()).input.offset(len as isize);
                                    }
                                }
                            }
                            ANYOF | ANYBUT => {
                                let mut q: *mut uint8_t =
                                    scan.offset(3 as ::core::ffi::c_int as isize);
                                if c == NUL {
                                    status = RA_NOMATCH;
                                } else if cstrchr(q as *mut ::core::ffi::c_char, c).is_null()
                                    as ::core::ffi::c_int
                                    == (op == ANYOF) as ::core::ffi::c_int
                                {
                                    status = RA_NOMATCH;
                                } else {
                                    let mut len_0: ::core::ffi::c_int =
                                        utfc_ptr2len(q as *mut ::core::ffi::c_char)
                                            - utf_ptr2len(q as *mut ::core::ffi::c_char);
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utf_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                    q =
                                        q.offset(
                                            utf_ptr2len(q as *mut ::core::ffi::c_char) as isize
                                        );
                                    if len_0 != 0 as ::core::ffi::c_int {
                                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while i < len_0 {
                                            if *q.offset(i as isize) as ::core::ffi::c_int
                                                != *(*rex.ptr()).input.offset(i as isize)
                                                    as ::core::ffi::c_int
                                            {
                                                status = RA_NOMATCH;
                                                break;
                                            } else {
                                                i += 1;
                                            }
                                        }
                                        (*rex.ptr()).input =
                                            (*rex.ptr()).input.offset(len_0 as isize);
                                    }
                                }
                            }
                            MULTIBYTECODE => {
                                let mut i_0: ::core::ffi::c_int = 0;
                                let mut len_1: ::core::ffi::c_int = 0;
                                let mut opnd_0: *const uint8_t =
                                    scan.offset(3 as ::core::ffi::c_int as isize);
                                len_1 = utfc_ptr2len(opnd_0 as *mut ::core::ffi::c_char);
                                if len_1 < 2 as ::core::ffi::c_int {
                                    status = RA_NOMATCH;
                                } else {
                                    let opndc: ::core::ffi::c_int =
                                        utf_ptr2char(opnd_0 as *mut ::core::ffi::c_char);
                                    if utf_iscomposing_legacy(opndc) {
                                        status = RA_NOMATCH;
                                        i_0 = 0 as ::core::ffi::c_int;
                                        while *(*rex.ptr()).input.offset(i_0 as isize)
                                            as ::core::ffi::c_int
                                            != NUL
                                        {
                                            let inpc: ::core::ffi::c_int = utf_ptr2char(
                                                ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                    .offset(i_0 as isize),
                                            );
                                            if !utf_iscomposing_legacy(inpc) {
                                                if i_0 > 0 as ::core::ffi::c_int {
                                                    break;
                                                }
                                            } else if opndc == inpc {
                                                len_1 = i_0
                                                    + utfc_ptr2len(
                                                        ((*rex.ptr()).input
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(i_0 as isize),
                                                    );
                                                status = RA_MATCH;
                                                break;
                                            }
                                            i_0 += utf_ptr2len(
                                                ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                    .offset(i_0 as isize),
                                            );
                                        }
                                    } else if cstrncmp(
                                        opnd_0 as *mut ::core::ffi::c_char,
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        &raw mut len_1,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        status = RA_NOMATCH;
                                        break 's_2509;
                                    }
                                    (*rex.ptr()).input = (*rex.ptr()).input.offset(len_1 as isize);
                                }
                            }
                            RE_COMPOSING => {
                                while utf_iscomposing_legacy(utf_ptr2char(
                                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                )) {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utf_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                }
                            }
                            NOTHING => {}
                            BACK => {
                                let mut i_1: ::core::ffi::c_int = 0;
                                let mut bp: *mut backpos_T =
                                    (*backpos.ptr()).ga_data as *mut backpos_T;
                                i_1 = 0 as ::core::ffi::c_int;
                                while i_1 < (*backpos.ptr()).ga_len {
                                    if (*bp.offset(i_1 as isize)).bp_scan == scan {
                                        break;
                                    }
                                    i_1 += 1;
                                }
                                if i_1 == (*backpos.ptr()).ga_len {
                                    let mut p: *mut backpos_T = ga_append_via_ptr(
                                        backpos.ptr(),
                                        ::core::mem::size_of::<backpos_T>(),
                                    )
                                        as *mut backpos_T;
                                    (*p).bp_scan = scan;
                                } else if reg_save_equal(&raw mut (*bp.offset(i_1 as isize)).bp_pos)
                                {
                                    status = RA_NOMATCH;
                                }
                                '_c2rust_label_1: {
                                    if status != 1 as ::core::ffi::c_int {
                                    } else {
                                        __assert_fail(
                                            b"status != RA_FAIL\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            b"src/nvim/regexp.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            6708 as ::core::ffi::c_uint,
                                            b"_Bool regmatch(uint8_t *, const proftime_T *, int *)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if status != RA_NOMATCH {
                                    reg_save(
                                        &raw mut (*bp.offset(i_1 as isize)).bp_pos,
                                        backpos.ptr(),
                                    );
                                }
                            }
                            80 | 81 | 82 | 83 | 84 | 85 | 86 | 87 | 88 | 89 => {
                                no = op - MOPEN;
                                cleanup_subexpr();
                                rp = regstack_push(RS_MOPEN, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as int16_t;
                                    if (*rex.ptr()).reg_match.is_null() {
                                        save_se_multi(
                                            &raw mut (*rp).rs_un.sesave,
                                            (*rex.ptr()).reg_startpos.offset(no as isize),
                                        );
                                    } else {
                                        save_se_one(
                                            &raw mut (*rp).rs_un.sesave,
                                            (*rex.ptr()).reg_startp.offset(no as isize),
                                        );
                                    };
                                }
                            }
                            NOPEN | NCLOSE => {
                                if regstack_push(RS_NOPEN, scan).is_null() {
                                    status = RA_FAIL;
                                }
                            }
                            111 | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 => {
                                no = op - ZOPEN;
                                cleanup_zsubexpr();
                                rp = regstack_push(RS_ZOPEN, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as int16_t;
                                    if (*rex.ptr()).reg_match.is_null() {
                                        save_se_multi(
                                            &raw mut (*rp).rs_un.sesave,
                                            (reg_startzpos.ptr() as *mut lpos_T)
                                                .offset(no as isize),
                                        );
                                    } else {
                                        save_se_one(
                                            &raw mut (*rp).rs_un.sesave,
                                            (reg_startzp.ptr() as *mut *mut uint8_t)
                                                .offset(no as isize),
                                        );
                                    };
                                }
                            }
                            90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 98 | 99 => {
                                no = op - MCLOSE;
                                cleanup_subexpr();
                                rp = regstack_push(RS_MCLOSE, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as int16_t;
                                    if (*rex.ptr()).reg_match.is_null() {
                                        save_se_multi(
                                            &raw mut (*rp).rs_un.sesave,
                                            (*rex.ptr()).reg_endpos.offset(no as isize),
                                        );
                                    } else {
                                        save_se_one(
                                            &raw mut (*rp).rs_un.sesave,
                                            (*rex.ptr()).reg_endp.offset(no as isize),
                                        );
                                    };
                                }
                            }
                            121 | 122 | 123 | 124 | 125 | 126 | 127 | 128 | 129 => {
                                no = op - ZCLOSE;
                                cleanup_zsubexpr();
                                rp = regstack_push(RS_ZCLOSE, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as int16_t;
                                    if (*rex.ptr()).reg_match.is_null() {
                                        save_se_multi(
                                            &raw mut (*rp).rs_un.sesave,
                                            (reg_endzpos.ptr() as *mut lpos_T).offset(no as isize),
                                        );
                                    } else {
                                        save_se_one(
                                            &raw mut (*rp).rs_un.sesave,
                                            (reg_endzp.ptr() as *mut *mut uint8_t)
                                                .offset(no as isize),
                                        );
                                    };
                                }
                            }
                            101 | 102 | 103 | 104 | 105 | 106 | 107 | 108 | 109 => {
                                let mut len_2: ::core::ffi::c_int = 0;
                                no = op - BACKREF;
                                cleanup_subexpr();
                                if !(*rex.ptr()).reg_match.is_null() {
                                    if (*(*rex.ptr()).reg_startp.offset(no as isize)).is_null()
                                        || (*(*rex.ptr()).reg_endp.offset(no as isize)).is_null()
                                    {
                                        len_2 = 0 as ::core::ffi::c_int;
                                    } else {
                                        len_2 = (*(*rex.ptr()).reg_endp.offset(no as isize))
                                            .offset_from(
                                                *(*rex.ptr()).reg_startp.offset(no as isize),
                                            )
                                            as ::core::ffi::c_int;
                                        if cstrncmp(
                                            *(*rex.ptr()).reg_startp.offset(no as isize)
                                                as *mut ::core::ffi::c_char,
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            &raw mut len_2,
                                        ) != 0 as ::core::ffi::c_int
                                        {
                                            status = RA_NOMATCH;
                                        }
                                    }
                                } else if (*(*rex.ptr()).reg_startpos.offset(no as isize)).lnum
                                    < 0 as linenr_T
                                    || (*(*rex.ptr()).reg_endpos.offset(no as isize)).lnum
                                        < 0 as linenr_T
                                {
                                    len_2 = 0 as ::core::ffi::c_int;
                                } else if (*(*rex.ptr()).reg_startpos.offset(no as isize)).lnum
                                    == (*rex.ptr()).lnum
                                    && (*(*rex.ptr()).reg_endpos.offset(no as isize)).lnum
                                        == (*rex.ptr()).lnum
                                {
                                    len_2 = ((*(*rex.ptr()).reg_endpos.offset(no as isize)).col
                                        - (*(*rex.ptr()).reg_startpos.offset(no as isize)).col)
                                        as ::core::ffi::c_int;
                                    if cstrncmp(
                                        ((*rex.ptr()).line as *mut ::core::ffi::c_char).offset(
                                            (*(*rex.ptr()).reg_startpos.offset(no as isize)).col
                                                as isize,
                                        ),
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        &raw mut len_2,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        status = RA_NOMATCH;
                                    }
                                } else {
                                    let mut r: ::core::ffi::c_int = match_with_backref(
                                        (*(*rex.ptr()).reg_startpos.offset(no as isize)).lnum,
                                        (*(*rex.ptr()).reg_startpos.offset(no as isize)).col,
                                        (*(*rex.ptr()).reg_endpos.offset(no as isize)).lnum,
                                        (*(*rex.ptr()).reg_endpos.offset(no as isize)).col,
                                        &raw mut len_2,
                                    );
                                    if r != RA_MATCH {
                                        status = r;
                                    }
                                }
                                (*rex.ptr()).input = (*rex.ptr()).input.offset(len_2 as isize);
                            }
                            131 | 132 | 133 | 134 | 135 | 136 | 137 | 138 | 139 => {
                                cleanup_zsubexpr();
                                no = op - ZREF;
                                if !(*re_extmatch_in.ptr()).is_null()
                                    && !(*re_extmatch_in.get()).matches[no as usize].is_null()
                                {
                                    let mut len_3: ::core::ffi::c_int = strlen(
                                        (*re_extmatch_in.get()).matches[no as usize]
                                            as *mut ::core::ffi::c_char,
                                    )
                                        as ::core::ffi::c_int;
                                    if cstrncmp(
                                        (*re_extmatch_in.get()).matches[no as usize]
                                            as *mut ::core::ffi::c_char,
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        &raw mut len_3,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        status = RA_NOMATCH;
                                    } else {
                                        (*rex.ptr()).input =
                                            (*rex.ptr()).input.offset(len_3 as isize);
                                    }
                                }
                            }
                            BRANCH => {
                                if *next as ::core::ffi::c_int != BRANCH {
                                    next = scan.offset(3 as ::core::ffi::c_int as isize);
                                } else {
                                    rp = regstack_push(RS_BRANCH, scan);
                                    if rp.is_null() {
                                        status = RA_FAIL;
                                    } else {
                                        status = RA_BREAK;
                                    }
                                }
                            }
                            BRACE_LIMITS => {
                                if *next as ::core::ffi::c_int == BRACE_SIMPLE {
                                    bl_minval.set(
                                        ((*scan.offset(3 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 24 as ::core::ffi::c_int)
                                            + ((*scan.offset(4 as ::core::ffi::c_int as isize)
                                                as int64_t)
                                                << 16 as ::core::ffi::c_int)
                                            + ((*scan.offset(5 as ::core::ffi::c_int as isize)
                                                as int64_t)
                                                << 8 as ::core::ffi::c_int)
                                            + *scan.offset(6 as ::core::ffi::c_int as isize)
                                                as int64_t,
                                    );
                                    bl_maxval.set(
                                        ((*scan
                                            .offset(4 as ::core::ffi::c_int as isize)
                                            .offset(3 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 24 as ::core::ffi::c_int)
                                            + ((*scan
                                                .offset(4 as ::core::ffi::c_int as isize)
                                                .offset(4 as ::core::ffi::c_int as isize)
                                                as int64_t)
                                                << 16 as ::core::ffi::c_int)
                                            + ((*scan
                                                .offset(4 as ::core::ffi::c_int as isize)
                                                .offset(5 as ::core::ffi::c_int as isize)
                                                as int64_t)
                                                << 8 as ::core::ffi::c_int)
                                            + *scan
                                                .offset(4 as ::core::ffi::c_int as isize)
                                                .offset(6 as ::core::ffi::c_int as isize)
                                                as int64_t,
                                    );
                                } else if *next as ::core::ffi::c_int >= BRACE_COMPLEX
                                    && (*next as ::core::ffi::c_int)
                                        < BRACE_COMPLEX + 10 as ::core::ffi::c_int
                                {
                                    no = *next as ::core::ffi::c_int - BRACE_COMPLEX;
                                    (*brace_min.ptr())[no as usize] = ((*scan
                                        .offset(3 as ::core::ffi::c_int as isize)
                                        as int64_t)
                                        << 24 as ::core::ffi::c_int)
                                        + ((*scan.offset(4 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 16 as ::core::ffi::c_int)
                                        + ((*scan.offset(5 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 8 as ::core::ffi::c_int)
                                        + *scan.offset(6 as ::core::ffi::c_int as isize) as int64_t;
                                    (*brace_max.ptr())[no as usize] = ((*scan
                                        .offset(4 as ::core::ffi::c_int as isize)
                                        .offset(3 as ::core::ffi::c_int as isize)
                                        as int64_t)
                                        << 24 as ::core::ffi::c_int)
                                        + ((*scan
                                            .offset(4 as ::core::ffi::c_int as isize)
                                            .offset(4 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 16 as ::core::ffi::c_int)
                                        + ((*scan
                                            .offset(4 as ::core::ffi::c_int as isize)
                                            .offset(5 as ::core::ffi::c_int as isize)
                                            as int64_t)
                                            << 8 as ::core::ffi::c_int)
                                        + *scan
                                            .offset(4 as ::core::ffi::c_int as isize)
                                            .offset(6 as ::core::ffi::c_int as isize)
                                            as int64_t;
                                    (*brace_count.ptr())[no as usize] = 0 as ::core::ffi::c_int;
                                } else {
                                    internal_error(
                                        b"BRACE_LIMITS\0".as_ptr() as *const ::core::ffi::c_char
                                    );
                                    status = RA_FAIL;
                                }
                            }
                            140 | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 => {
                                no = op - BRACE_COMPLEX;
                                (*brace_count.ptr())[no as usize] += 1;
                                if (*brace_count.ptr())[no as usize] as int64_t
                                    <= (if (*brace_min.ptr())[no as usize]
                                        <= (*brace_max.ptr())[no as usize]
                                    {
                                        (*brace_min.ptr())[no as usize]
                                    } else {
                                        (*brace_max.ptr())[no as usize]
                                    })
                                {
                                    rp = regstack_push(RS_BRCPLX_MORE, scan);
                                    if rp.is_null() {
                                        status = RA_FAIL;
                                    } else {
                                        (*rp).rs_no = no as int16_t;
                                        reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                        next = scan.offset(3 as ::core::ffi::c_int as isize);
                                    }
                                } else if (*brace_min.ptr())[no as usize]
                                    <= (*brace_max.ptr())[no as usize]
                                {
                                    if (*brace_count.ptr())[no as usize] as int64_t
                                        <= (*brace_max.ptr())[no as usize]
                                    {
                                        rp = regstack_push(RS_BRCPLX_LONG, scan);
                                        if rp.is_null() {
                                            status = RA_FAIL;
                                        } else {
                                            (*rp).rs_no = no as int16_t;
                                            reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                            next = scan.offset(3 as ::core::ffi::c_int as isize);
                                        }
                                    }
                                } else if (*brace_count.ptr())[no as usize] as int64_t
                                    <= (*brace_min.ptr())[no as usize]
                                {
                                    rp = regstack_push(RS_BRCPLX_SHORT, scan);
                                    if rp.is_null() {
                                        status = RA_FAIL;
                                    } else {
                                        reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                    }
                                }
                            }
                            BRACE_SIMPLE | STAR | PLUS => {
                                let mut rst: regstar_T = regstar_T {
                                    nextb: 0,
                                    nextb_ic: 0,
                                    count: 0,
                                    minval: 0,
                                    maxval: 0,
                                };
                                if *next as ::core::ffi::c_int == EXACTLY {
                                    rst.nextb = *next.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int;
                                    if (*rex.ptr()).reg_ic {
                                        if mb_isupper(rst.nextb) {
                                            rst.nextb_ic = mb_tolower(rst.nextb);
                                        } else {
                                            rst.nextb_ic = mb_toupper(rst.nextb);
                                        }
                                    } else {
                                        rst.nextb_ic = rst.nextb;
                                    }
                                } else {
                                    rst.nextb = NUL;
                                    rst.nextb_ic = NUL;
                                }
                                if op != BRACE_SIMPLE {
                                    rst.minval = (if op == STAR {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        1 as ::core::ffi::c_int
                                    }) as int64_t;
                                    rst.maxval = MAX_LIMIT as int64_t;
                                } else {
                                    rst.minval = bl_minval.get();
                                    rst.maxval = bl_maxval.get();
                                }
                                rst.count = regrepeat(
                                    scan.offset(3 as ::core::ffi::c_int as isize),
                                    rst.maxval,
                                ) as int64_t;
                                if got_int.get() {
                                    status = RA_FAIL;
                                } else if if rst.minval <= rst.maxval {
                                    (rst.count >= rst.minval) as ::core::ffi::c_int
                                } else {
                                    (rst.count >= rst.maxval) as ::core::ffi::c_int
                                } != 0
                                {
                                    if ((*regstack.ptr()).ga_len as ::core::ffi::c_uint
                                        >> 10 as ::core::ffi::c_int)
                                        as int64_t
                                        >= p_mmp.get()
                                    {
                                        emsg(gettext(
                                            (e_pattern_uses_more_memory_than_maxmempattern.ptr()
                                                as *const _)
                                                as *const ::core::ffi::c_char,
                                        ));
                                        status = RA_FAIL;
                                    } else {
                                        ga_grow(
                                            regstack.ptr(),
                                            ::core::mem::size_of::<regstar_T>()
                                                as ::core::ffi::c_int,
                                        );
                                        (*regstack.ptr()).ga_len +=
                                            ::core::mem::size_of::<regstar_T>()
                                                as ::core::ffi::c_int;
                                        rp = regstack_push(
                                            (if rst.minval <= rst.maxval {
                                                RS_STAR_LONG as ::core::ffi::c_int
                                            } else {
                                                RS_STAR_SHORT as ::core::ffi::c_int
                                            })
                                                as regstate_T,
                                            scan,
                                        );
                                        if rp.is_null() {
                                            status = RA_FAIL;
                                        } else {
                                            *(rp as *mut regstar_T)
                                                .offset(-(1 as ::core::ffi::c_int as isize)) = rst;
                                            status = RA_BREAK;
                                        }
                                    }
                                } else {
                                    status = RA_NOMATCH;
                                }
                            }
                            NOMATCH | MATCH | SUBPAT => {
                                rp = regstack_push(RS_NOMATCH, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = op as int16_t;
                                    reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                    next = scan.offset(3 as ::core::ffi::c_int as isize);
                                }
                            }
                            BEHIND | NOBEHIND => {
                                if ((*regstack.ptr()).ga_len as ::core::ffi::c_uint
                                    >> 10 as ::core::ffi::c_int)
                                    as int64_t
                                    >= p_mmp.get()
                                {
                                    emsg(gettext(
                                        (e_pattern_uses_more_memory_than_maxmempattern.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ));
                                    status = RA_FAIL;
                                } else {
                                    ga_grow(
                                        regstack.ptr(),
                                        ::core::mem::size_of::<regbehind_T>() as ::core::ffi::c_int,
                                    );
                                    (*regstack.ptr()).ga_len +=
                                        ::core::mem::size_of::<regbehind_T>() as ::core::ffi::c_int;
                                    rp = regstack_push(RS_BEHIND1, scan);
                                    if rp.is_null() {
                                        status = RA_FAIL;
                                    } else {
                                        save_subexpr(
                                            (rp as *mut regbehind_T)
                                                .offset(-(1 as ::core::ffi::c_int as isize)),
                                        );
                                        (*rp).rs_no = op as int16_t;
                                        reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                    }
                                }
                            }
                            BHPOS => {
                                if (*rex.ptr()).reg_match.is_null() {
                                    if (*behind_pos.ptr()).rs_u.pos.col
                                        != (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                            as colnr_T
                                        || (*behind_pos.ptr()).rs_u.pos.lnum != (*rex.ptr()).lnum
                                    {
                                        status = RA_NOMATCH;
                                    }
                                } else if (*behind_pos.ptr()).rs_u.ptr != (*rex.ptr()).input {
                                    status = RA_NOMATCH;
                                }
                            }
                            NEWL => {
                                if (c != NUL
                                    || !(*rex.ptr()).reg_match.is_null()
                                    || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                                    || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0)
                                    && (c != '\n' as ::core::ffi::c_int
                                        || !(*rex.ptr()).reg_line_lbr)
                                {
                                    status = RA_NOMATCH;
                                } else if (*rex.ptr()).reg_line_lbr {
                                    (*rex.ptr()).input = (*rex.ptr())
                                        .input
                                        .offset(utfc_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ) as isize);
                                } else {
                                    reg_nextline();
                                }
                            }
                            END => {
                                status = RA_MATCH;
                            }
                            _ => {
                                iemsg(gettext(&raw const e_re_corr as *const ::core::ffi::c_char));
                                status = RA_FAIL;
                            }
                        }
                    }
                }
                if status != RA_CONT {
                    break;
                }
                scan = next;
            }
        }
        while !((*regstack.ptr()).ga_len <= 0 as ::core::ffi::c_int) && status != RA_FAIL {
            rp = (((*regstack.ptr()).ga_data as *mut ::core::ffi::c_char)
                .offset((*regstack.ptr()).ga_len as isize) as *mut regitem_T)
                .offset(-(1 as ::core::ffi::c_int as isize));
            match (*rp).rs_state as ::core::ffi::c_uint {
                0 => {
                    regstack_pop(&raw mut scan);
                }
                1 => {
                    if status == RA_NOMATCH {
                        if (*rex.ptr()).reg_match.is_null() {
                            *(*rex.ptr()).reg_startpos.offset((*rp).rs_no as isize) =
                                (*rp).rs_un.sesave.se_u.pos;
                        } else {
                            *(*rex.ptr()).reg_startp.offset((*rp).rs_no as isize) =
                                (*rp).rs_un.sesave.se_u.ptr;
                        }
                    }
                    regstack_pop(&raw mut scan);
                }
                3 => {
                    if status == RA_NOMATCH {
                        if (*rex.ptr()).reg_match.is_null() {
                            (*reg_startzpos.ptr())[(*rp).rs_no as usize] =
                                (*rp).rs_un.sesave.se_u.pos;
                        } else {
                            (*reg_startzp.ptr())[(*rp).rs_no as usize] =
                                (*rp).rs_un.sesave.se_u.ptr;
                        }
                    }
                    regstack_pop(&raw mut scan);
                }
                2 => {
                    if status == RA_NOMATCH {
                        if (*rex.ptr()).reg_match.is_null() {
                            *(*rex.ptr()).reg_endpos.offset((*rp).rs_no as isize) =
                                (*rp).rs_un.sesave.se_u.pos;
                        } else {
                            *(*rex.ptr()).reg_endp.offset((*rp).rs_no as isize) =
                                (*rp).rs_un.sesave.se_u.ptr;
                        }
                    }
                    regstack_pop(&raw mut scan);
                }
                4 => {
                    if status == RA_NOMATCH {
                        if (*rex.ptr()).reg_match.is_null() {
                            (*reg_endzpos.ptr())[(*rp).rs_no as usize] =
                                (*rp).rs_un.sesave.se_u.pos;
                        } else {
                            (*reg_endzp.ptr())[(*rp).rs_no as usize] = (*rp).rs_un.sesave.se_u.ptr;
                        }
                    }
                    regstack_pop(&raw mut scan);
                }
                5 => {
                    if status == RA_MATCH {
                        regstack_pop(&raw mut scan);
                    } else {
                        if status != RA_BREAK {
                            reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                            scan = (*rp).rs_scan;
                        }
                        if scan.is_null() || *scan as ::core::ffi::c_int != BRANCH {
                            status = RA_NOMATCH;
                            regstack_pop(&raw mut scan);
                        } else {
                            (*rp).rs_scan = regnext(scan);
                            reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                            scan = scan.offset(3 as ::core::ffi::c_int as isize);
                        }
                    }
                }
                6 => {
                    if status == RA_NOMATCH {
                        reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                        (*brace_count.ptr())[(*rp).rs_no as usize] -= 1;
                    }
                    regstack_pop(&raw mut scan);
                }
                7 => {
                    if status == RA_NOMATCH {
                        reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                        (*brace_count.ptr())[(*rp).rs_no as usize] -= 1;
                        status = RA_CONT;
                    }
                    regstack_pop(&raw mut scan);
                    if status == RA_CONT {
                        scan = regnext(scan);
                    }
                }
                8 => {
                    if status == RA_NOMATCH {
                        reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                    }
                    regstack_pop(&raw mut scan);
                    if status == RA_NOMATCH {
                        scan = scan.offset(3 as ::core::ffi::c_int as isize);
                        status = RA_CONT;
                    }
                }
                9 => {
                    if status
                        == (if (*rp).rs_no as ::core::ffi::c_int == NOMATCH {
                            RA_MATCH
                        } else {
                            RA_NOMATCH
                        })
                    {
                        status = RA_NOMATCH;
                    } else {
                        status = RA_CONT;
                        if (*rp).rs_no as ::core::ffi::c_int != SUBPAT {
                            reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                        }
                    }
                    regstack_pop(&raw mut scan);
                    if status == RA_CONT {
                        scan = regnext(scan);
                    }
                }
                10 => {
                    if status == RA_NOMATCH {
                        regstack_pop(&raw mut scan);
                        (*regstack.ptr()).ga_len -=
                            ::core::mem::size_of::<regbehind_T>() as ::core::ffi::c_int;
                    } else {
                        reg_save(
                            &raw mut (*(rp as *mut regbehind_T)
                                .offset(-(1 as ::core::ffi::c_int as isize)))
                            .save_after,
                            backpos.ptr(),
                        );
                        (*(rp as *mut regbehind_T).offset(-(1 as ::core::ffi::c_int as isize)))
                            .save_behind = behind_pos.get();
                        behind_pos.set((*rp).rs_un.regsave);
                        (*rp).rs_state = RS_BEHIND2;
                        reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                        scan = (*rp)
                            .rs_scan
                            .offset(3 as ::core::ffi::c_int as isize)
                            .offset(4 as ::core::ffi::c_int as isize);
                    }
                }
                11 => {
                    if status == RA_MATCH
                        && reg_save_equal(behind_pos.ptr()) as ::core::ffi::c_int != 0
                    {
                        behind_pos.set(
                            (*(rp as *mut regbehind_T).offset(-(1 as ::core::ffi::c_int as isize)))
                                .save_behind,
                        );
                        if (*rp).rs_no as ::core::ffi::c_int == BEHIND {
                            reg_restore(
                                &raw mut (*(rp as *mut regbehind_T)
                                    .offset(-(1 as ::core::ffi::c_int as isize)))
                                .save_after,
                                backpos.ptr(),
                            );
                        } else {
                            status = RA_NOMATCH;
                            restore_subexpr(
                                (rp as *mut regbehind_T)
                                    .offset(-(1 as ::core::ffi::c_int as isize)),
                            );
                        }
                        regstack_pop(&raw mut scan);
                        (*regstack.ptr()).ga_len -=
                            ::core::mem::size_of::<regbehind_T>() as ::core::ffi::c_int;
                    } else {
                        let mut limit: int64_t = 0;
                        no = OK;
                        limit = ((*(*rp).rs_scan.offset(3 as ::core::ffi::c_int as isize)
                            as int64_t)
                            << 24 as ::core::ffi::c_int)
                            + ((*(*rp).rs_scan.offset(4 as ::core::ffi::c_int as isize)
                                as int64_t)
                                << 16 as ::core::ffi::c_int)
                            + ((*(*rp).rs_scan.offset(5 as ::core::ffi::c_int as isize)
                                as int64_t)
                                << 8 as ::core::ffi::c_int)
                            + *(*rp).rs_scan.offset(6 as ::core::ffi::c_int as isize) as int64_t;
                        if (*rex.ptr()).reg_match.is_null() {
                            if limit > 0 as int64_t
                                && ((if (*rp).rs_un.regsave.rs_u.pos.lnum
                                    < (*behind_pos.ptr()).rs_u.pos.lnum
                                {
                                    strlen((*rex.ptr()).line as *mut ::core::ffi::c_char) as colnr_T
                                } else {
                                    (*behind_pos.ptr()).rs_u.pos.col
                                }) - (*rp).rs_un.regsave.rs_u.pos.col)
                                    as int64_t
                                    >= limit
                            {
                                no = FAIL;
                            } else if (*rp).rs_un.regsave.rs_u.pos.col == 0 as ::core::ffi::c_int {
                                if (*rp).rs_un.regsave.rs_u.pos.lnum
                                    < (*behind_pos.ptr()).rs_u.pos.lnum
                                    || {
                                        (*rp).rs_un.regsave.rs_u.pos.lnum -= 1;
                                        reg_getline((*rp).rs_un.regsave.rs_u.pos.lnum).is_null()
                                    }
                                {
                                    no = FAIL;
                                } else {
                                    reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                                    (*rp).rs_un.regsave.rs_u.pos.col =
                                        strlen((*rex.ptr()).line as *mut ::core::ffi::c_char)
                                            as colnr_T;
                                }
                            } else {
                                let line: *const uint8_t =
                                    reg_getline((*rp).rs_un.regsave.rs_u.pos.lnum) as *mut uint8_t;
                                (*rp).rs_un.regsave.rs_u.pos.col -= utf_head_off(
                                    line as *mut ::core::ffi::c_char,
                                    (line as *mut ::core::ffi::c_char)
                                        .offset((*rp).rs_un.regsave.rs_u.pos.col as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                ) + 1 as ::core::ffi::c_int;
                            }
                        } else if (*rp).rs_un.regsave.rs_u.ptr == (*rex.ptr()).line {
                            no = FAIL;
                        } else {
                            (*rp).rs_un.regsave.rs_u.ptr = (*rp).rs_un.regsave.rs_u.ptr.offset(
                                -((utf_head_off(
                                    (*rex.ptr()).line as *mut ::core::ffi::c_char,
                                    ((*rp).rs_un.regsave.rs_u.ptr as *mut ::core::ffi::c_char)
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                ) + 1 as ::core::ffi::c_int)
                                    as isize),
                            );
                            if limit > 0 as int64_t
                                && (*behind_pos.ptr())
                                    .rs_u
                                    .ptr
                                    .offset_from((*rp).rs_un.regsave.rs_u.ptr)
                                    > limit as ptrdiff_t
                            {
                                no = FAIL;
                            }
                        }
                        if no == OK {
                            reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                            scan = (*rp)
                                .rs_scan
                                .offset(3 as ::core::ffi::c_int as isize)
                                .offset(4 as ::core::ffi::c_int as isize);
                            if status == RA_MATCH {
                                status = RA_NOMATCH;
                                restore_subexpr(
                                    (rp as *mut regbehind_T)
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                );
                            }
                        } else {
                            behind_pos.set(
                                (*(rp as *mut regbehind_T)
                                    .offset(-(1 as ::core::ffi::c_int as isize)))
                                .save_behind,
                            );
                            if (*rp).rs_no as ::core::ffi::c_int == NOBEHIND {
                                reg_restore(
                                    &raw mut (*(rp as *mut regbehind_T)
                                        .offset(-(1 as ::core::ffi::c_int as isize)))
                                    .save_after,
                                    backpos.ptr(),
                                );
                                status = RA_MATCH;
                            } else if status == RA_MATCH {
                                status = RA_NOMATCH;
                                restore_subexpr(
                                    (rp as *mut regbehind_T)
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                );
                            }
                            regstack_pop(&raw mut scan);
                            (*regstack.ptr()).ga_len -=
                                ::core::mem::size_of::<regbehind_T>() as ::core::ffi::c_int;
                        }
                    }
                }
                12 | 13 => {
                    let mut rst_0: *mut regstar_T =
                        (rp as *mut regstar_T).offset(-(1 as ::core::ffi::c_int as isize));
                    if status == RA_MATCH {
                        regstack_pop(&raw mut scan);
                        (*regstack.ptr()).ga_len -=
                            ::core::mem::size_of::<regstar_T>() as ::core::ffi::c_int;
                    } else {
                        if status != RA_BREAK {
                            reg_restore(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                        }
                        loop {
                            if status != RA_BREAK {
                                if (*rp).rs_state as ::core::ffi::c_uint
                                    == RS_STAR_LONG as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    (*rst_0).count -= 1;
                                    if (*rst_0).count < (*rst_0).minval {
                                        break;
                                    }
                                    if (*rex.ptr()).input == (*rex.ptr()).line {
                                        if (*rex.ptr()).lnum == 0 as linenr_T {
                                            status = RA_NOMATCH;
                                            break;
                                        } else {
                                            (*rex.ptr()).lnum -= 1;
                                            (*rex.ptr()).line =
                                                reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                                            if (*rex.ptr()).line.is_null() {
                                                break;
                                            }
                                            (*rex.ptr()).input =
                                                (*rex.ptr())
                                                    .line
                                                    .offset(
                                                        reg_getline_len((*rex.ptr()).lnum) as isize
                                                    );
                                            reg_breakcheck();
                                        }
                                    } else {
                                        (*rex.ptr()).input = (*rex.ptr()).input.offset(
                                            -((utf_head_off(
                                                (*rex.ptr()).line as *mut ::core::ffi::c_char,
                                                ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                    .offset(-(1 as ::core::ffi::c_int as isize)),
                                            ) + 1 as ::core::ffi::c_int)
                                                as isize),
                                        );
                                    }
                                } else {
                                    if (*rst_0).count == (*rst_0).minval
                                        || regrepeat(
                                            (*rp).rs_scan.offset(3 as ::core::ffi::c_int as isize),
                                            1 as int64_t,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        break;
                                    }
                                    (*rst_0).count += 1;
                                }
                                if got_int.get() {
                                    break;
                                }
                            } else {
                                status = RA_NOMATCH;
                            }
                            if !((*rst_0).nextb == NUL
                                || *(*rex.ptr()).input as ::core::ffi::c_int == (*rst_0).nextb
                                || *(*rex.ptr()).input as ::core::ffi::c_int == (*rst_0).nextb_ic)
                            {
                                continue;
                            }
                            reg_save(&raw mut (*rp).rs_un.regsave, backpos.ptr());
                            scan = regnext((*rp).rs_scan);
                            status = RA_CONT;
                            break;
                        }
                        if status != RA_CONT {
                            regstack_pop(&raw mut scan);
                            (*regstack.ptr()).ga_len -=
                                ::core::mem::size_of::<regstar_T>() as ::core::ffi::c_int;
                            status = RA_NOMATCH;
                        }
                    }
                }
                _ => {}
            }
            if status == RA_CONT
                || rp
                    == (((*regstack.ptr()).ga_data as *mut ::core::ffi::c_char)
                        .offset((*regstack.ptr()).ga_len as isize)
                        as *mut regitem_T)
                        .offset(-(1 as ::core::ffi::c_int as isize))
            {
                break;
            }
        }
        if status == RA_CONT {
            continue;
        }
        if (*regstack.ptr()).ga_len <= 0 as ::core::ffi::c_int || status == RA_FAIL {
            if scan.is_null() {
                iemsg(gettext(&raw const e_re_corr as *const ::core::ffi::c_char));
            }
            return status == RA_MATCH;
        }
    }
}
