//! How far a simple item can repeat from here.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regrepeat(
    mut p: *mut uint8_t,
    mut maxcount: int64_t,
) -> ::core::ffi::c_int {
    let mut count: int64_t = 0 as int64_t;
    let mut opnd: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut mask: ::core::ffi::c_int = 0;
    let mut testval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scan: *mut uint8_t = (*rex.ptr()).input;
    opnd = p.offset(3 as ::core::ffi::c_int as isize);
    's_965: {
        '_do_class: {
            'c_98954: {
                'c_99840: {
                    'c_100039: {
                        'c_100236: {
                            match *p as ::core::ffi::c_int {
                                ANY | 50 => {
                                    while count < maxcount {
                                        while *scan as ::core::ffi::c_int != NUL && count < maxcount
                                        {
                                            count += 1;
                                            scan = scan.offset(utfc_ptr2len(
                                                scan as *mut ::core::ffi::c_char,
                                            )
                                                as isize);
                                        }
                                        if !(*rex.ptr()).reg_match.is_null()
                                            || !(*p as ::core::ffi::c_int >= FIRST_NL
                                                && *p as ::core::ffi::c_int <= LAST_NL)
                                            || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                                            || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                            || count == maxcount
                                        {
                                            break;
                                        }
                                        count += 1;
                                        reg_nextline();
                                        scan = (*rex.ptr()).input;
                                        if got_int.get() {
                                            break;
                                        }
                                    }
                                    break 's_965;
                                }
                                IDENT | 53 => {
                                    testval = 1 as ::core::ffi::c_int;
                                }
                                SIDENT | 54 => {}
                                KWORD | 55 => {
                                    testval = 1 as ::core::ffi::c_int;
                                    break 'c_100236;
                                }
                                SKWORD | 56 => {
                                    break 'c_100236;
                                }
                                FNAME | 57 => {
                                    testval = 1 as ::core::ffi::c_int;
                                    break 'c_100039;
                                }
                                SFNAME | 58 => {
                                    break 'c_100039;
                                }
                                PRINT | 59 => {
                                    testval = 1 as ::core::ffi::c_int;
                                    break 'c_99840;
                                }
                                SPRINT | 60 => {
                                    break 'c_99840;
                                }
                                WHITE | 61 => {
                                    mask = RI_WHITE;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NWHITE | 62 => {
                                    mask = RI_WHITE;
                                    break '_do_class;
                                }
                                DIGIT | 63 => {
                                    mask = RI_DIGIT;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NDIGIT | 64 => {
                                    mask = RI_DIGIT;
                                    break '_do_class;
                                }
                                HEX | 65 => {
                                    mask = RI_HEX;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NHEX | 66 => {
                                    mask = RI_HEX;
                                    break '_do_class;
                                }
                                OCTAL | 67 => {
                                    mask = RI_OCTAL;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NOCTAL | 68 => {
                                    mask = RI_OCTAL;
                                    break '_do_class;
                                }
                                WORD | 69 => {
                                    mask = RI_WORD;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NWORD | 70 => {
                                    mask = RI_WORD;
                                    break '_do_class;
                                }
                                HEAD | 71 => {
                                    mask = RI_HEAD;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NHEAD | 72 => {
                                    mask = RI_HEAD;
                                    break '_do_class;
                                }
                                ALPHA | 73 => {
                                    mask = RI_ALPHA;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NALPHA | 74 => {
                                    mask = RI_ALPHA;
                                    break '_do_class;
                                }
                                LOWER | 75 => {
                                    mask = RI_LOWER;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NLOWER | 76 => {
                                    mask = RI_LOWER;
                                    break '_do_class;
                                }
                                UPPER | 77 => {
                                    mask = RI_UPPER;
                                    testval = mask;
                                    break '_do_class;
                                }
                                NUPPER | 78 => {
                                    mask = RI_UPPER;
                                    break '_do_class;
                                }
                                EXACTLY => {
                                    let mut cu: ::core::ffi::c_int = 0;
                                    let mut cl: ::core::ffi::c_int = 0;
                                    if (*rex.ptr()).reg_ic {
                                        cu = mb_toupper(*opnd as ::core::ffi::c_int);
                                        cl = mb_tolower(*opnd as ::core::ffi::c_int);
                                        while count < maxcount
                                            && (*scan as ::core::ffi::c_int == cu
                                                || *scan as ::core::ffi::c_int == cl)
                                        {
                                            count += 1;
                                            scan = scan.offset(1);
                                        }
                                    } else {
                                        cu = *opnd as ::core::ffi::c_int;
                                        while count < maxcount && *scan as ::core::ffi::c_int == cu
                                        {
                                            count += 1;
                                            scan = scan.offset(1);
                                        }
                                    }
                                    break 's_965;
                                }
                                MULTIBYTECODE => {
                                    let mut i: ::core::ffi::c_int = 0;
                                    let mut len: ::core::ffi::c_int = 0;
                                    let mut cf: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    len = utfc_ptr2len(opnd as *mut ::core::ffi::c_char);
                                    if len > 1 as ::core::ffi::c_int {
                                        if (*rex.ptr()).reg_ic {
                                            cf = utf_fold(utf_ptr2char(
                                                opnd as *mut ::core::ffi::c_char,
                                            ));
                                        }
                                        while count < maxcount
                                            && utfc_ptr2len(scan as *mut ::core::ffi::c_char) >= len
                                        {
                                            i = 0 as ::core::ffi::c_int;
                                            while i < len {
                                                if *opnd.offset(i as isize) as ::core::ffi::c_int
                                                    != *scan.offset(i as isize)
                                                        as ::core::ffi::c_int
                                                {
                                                    break;
                                                }
                                                i += 1;
                                            }
                                            if i < len
                                                && (!(*rex.ptr()).reg_ic
                                                    || utf_fold(utf_ptr2char(
                                                        scan as *mut ::core::ffi::c_char,
                                                    )) != cf)
                                            {
                                                break;
                                            }
                                            scan = scan.offset(len as isize);
                                            count += 1;
                                        }
                                    }
                                    break 's_965;
                                }
                                ANYOF | 51 => {
                                    testval = 1 as ::core::ffi::c_int;
                                    break 'c_98954;
                                }
                                ANYBUT | 52 => {
                                    break 'c_98954;
                                }
                                NEWL => {
                                    while count < maxcount
                                        && (*scan as ::core::ffi::c_int == NUL
                                            && (*rex.ptr()).lnum <= (*rex.ptr()).reg_maxline
                                            && !(*rex.ptr()).reg_line_lbr
                                            && (*rex.ptr()).reg_match.is_null()
                                            || *scan as ::core::ffi::c_int
                                                == '\n' as ::core::ffi::c_int
                                                && (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int
                                                    != 0)
                                    {
                                        count += 1;
                                        if (*rex.ptr()).reg_line_lbr {
                                            (*rex.ptr()).input =
                                                (*rex.ptr()).input.offset(utfc_ptr2len(
                                                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                                )
                                                    as isize);
                                        } else {
                                            reg_nextline();
                                        }
                                        scan = (*rex.ptr()).input;
                                        if got_int.get() {
                                            break;
                                        }
                                    }
                                    break 's_965;
                                }
                                _ => {
                                    iemsg(gettext(
                                        &raw const e_re_corr as *const ::core::ffi::c_char,
                                    ));
                                    break 's_965;
                                }
                            }
                            while count < maxcount {
                                if vim_isIDc(utf_ptr2char(scan as *mut ::core::ffi::c_char))
                                    as ::core::ffi::c_int
                                    != 0
                                    && (testval != 0 || !ascii_isdigit(*scan as ::core::ffi::c_int))
                                {
                                    scan = scan
                                        .offset(
                                            utfc_ptr2len(scan as *mut ::core::ffi::c_char) as isize
                                        );
                                } else if *scan as ::core::ffi::c_int == NUL {
                                    if !(*rex.ptr()).reg_match.is_null()
                                        || !(*p as ::core::ffi::c_int >= FIRST_NL
                                            && *p as ::core::ffi::c_int <= LAST_NL)
                                        || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                                        || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                    {
                                        break;
                                    }
                                    reg_nextline();
                                    scan = (*rex.ptr()).input;
                                    if got_int.get() {
                                        break;
                                    }
                                } else {
                                    if !((*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                        && *scan as ::core::ffi::c_int
                                            == '\n' as ::core::ffi::c_int
                                        && (*p as ::core::ffi::c_int >= FIRST_NL
                                            && *p as ::core::ffi::c_int <= LAST_NL))
                                    {
                                        break;
                                    }
                                    scan = scan.offset(1);
                                }
                                count += 1;
                            }
                            break 's_965;
                        }
                        while count < maxcount {
                            if vim_iswordp_buf(
                                scan as *mut ::core::ffi::c_char,
                                (*rex.ptr()).reg_buf,
                            ) as ::core::ffi::c_int
                                != 0
                                && (testval != 0 || !ascii_isdigit(*scan as ::core::ffi::c_int))
                            {
                                scan =
                                    scan.offset(
                                        utfc_ptr2len(scan as *mut ::core::ffi::c_char) as isize
                                    );
                            } else if *scan as ::core::ffi::c_int == NUL {
                                if !(*rex.ptr()).reg_match.is_null()
                                    || !(*p as ::core::ffi::c_int >= FIRST_NL
                                        && *p as ::core::ffi::c_int <= LAST_NL)
                                    || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                                    || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                {
                                    break;
                                }
                                reg_nextline();
                                scan = (*rex.ptr()).input;
                                if got_int.get() {
                                    break;
                                }
                            } else {
                                if !((*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                    && *scan as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                                    && (*p as ::core::ffi::c_int >= FIRST_NL
                                        && *p as ::core::ffi::c_int <= LAST_NL))
                                {
                                    break;
                                }
                                scan = scan.offset(1);
                            }
                            count += 1;
                        }
                        break 's_965;
                    }
                    while count < maxcount {
                        if vim_isfilec(utf_ptr2char(scan as *mut ::core::ffi::c_char))
                            as ::core::ffi::c_int
                            != 0
                            && (testval != 0 || !ascii_isdigit(*scan as ::core::ffi::c_int))
                        {
                            scan = scan
                                .offset(utfc_ptr2len(scan as *mut ::core::ffi::c_char) as isize);
                        } else if *scan as ::core::ffi::c_int == NUL {
                            if !(*rex.ptr()).reg_match.is_null()
                                || !(*p as ::core::ffi::c_int >= FIRST_NL
                                    && *p as ::core::ffi::c_int <= LAST_NL)
                                || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                                || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                            {
                                break;
                            }
                            reg_nextline();
                            scan = (*rex.ptr()).input;
                            if got_int.get() {
                                break;
                            }
                        } else {
                            if !((*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                && *scan as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                                && (*p as ::core::ffi::c_int >= FIRST_NL
                                    && *p as ::core::ffi::c_int <= LAST_NL))
                            {
                                break;
                            }
                            scan = scan.offset(1);
                        }
                        count += 1;
                    }
                    break 's_965;
                }
                while count < maxcount {
                    if *scan as ::core::ffi::c_int == NUL {
                        if !(*rex.ptr()).reg_match.is_null()
                            || !(*p as ::core::ffi::c_int >= FIRST_NL
                                && *p as ::core::ffi::c_int <= LAST_NL)
                            || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                            || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                        {
                            break;
                        }
                        reg_nextline();
                        scan = (*rex.ptr()).input;
                        if got_int.get() {
                            break;
                        }
                    } else if vim_isprintc(utf_ptr2char(scan as *mut ::core::ffi::c_char))
                        as ::core::ffi::c_int
                        == 1 as ::core::ffi::c_int
                        && (testval != 0 || !ascii_isdigit(*scan as ::core::ffi::c_int))
                    {
                        scan = scan.offset(utfc_ptr2len(scan as *mut ::core::ffi::c_char) as isize);
                    } else {
                        if !((*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                            && *scan as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                            && (*p as ::core::ffi::c_int >= FIRST_NL
                                && *p as ::core::ffi::c_int <= LAST_NL))
                        {
                            break;
                        }
                        scan = scan.offset(1);
                    }
                    count += 1;
                }
                break 's_965;
            }
            while count < maxcount {
                let mut len_0: ::core::ffi::c_int = 0;
                if *scan as ::core::ffi::c_int == NUL {
                    if !(*rex.ptr()).reg_match.is_null()
                        || !(*p as ::core::ffi::c_int >= FIRST_NL
                            && *p as ::core::ffi::c_int <= LAST_NL)
                        || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                        || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                    {
                        break;
                    }
                    reg_nextline();
                    scan = (*rex.ptr()).input;
                    if got_int.get() {
                        break;
                    }
                } else if (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                    && *scan as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                    && (*p as ::core::ffi::c_int >= FIRST_NL && *p as ::core::ffi::c_int <= LAST_NL)
                {
                    scan = scan.offset(1);
                } else {
                    len_0 = utfc_ptr2len(scan as *mut ::core::ffi::c_char);
                    if len_0 > 1 as ::core::ffi::c_int {
                        if cstrchr(
                            opnd as *mut ::core::ffi::c_char,
                            utf_ptr2char(scan as *mut ::core::ffi::c_char),
                        )
                        .is_null() as ::core::ffi::c_int
                            == testval
                        {
                            break;
                        }
                        scan = scan.offset(len_0 as isize);
                    } else {
                        if cstrchr(
                            opnd as *mut ::core::ffi::c_char,
                            *scan as ::core::ffi::c_int,
                        )
                        .is_null() as ::core::ffi::c_int
                            == testval
                        {
                            break;
                        }
                        scan = scan.offset(1);
                    }
                }
                count += 1;
            }
            break 's_965;
        }
        while count < maxcount {
            let mut l: ::core::ffi::c_int = 0;
            if *scan as ::core::ffi::c_int == NUL {
                if !(*rex.ptr()).reg_match.is_null()
                    || !(*p as ::core::ffi::c_int >= FIRST_NL
                        && *p as ::core::ffi::c_int <= LAST_NL)
                    || (*rex.ptr()).lnum > (*rex.ptr()).reg_maxline
                    || (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                {
                    break;
                }
                reg_nextline();
                scan = (*rex.ptr()).input;
                if got_int.get() {
                    break;
                }
            } else {
                l = utfc_ptr2len(scan as *mut ::core::ffi::c_char);
                if l > 1 as ::core::ffi::c_int {
                    if testval != 0 as ::core::ffi::c_int {
                        break;
                    }
                    scan = scan.offset(l as isize);
                } else if RI_FLAGS[*scan as usize] as ::core::ffi::c_int & mask == testval {
                    scan = scan.offset(1);
                } else {
                    if !((*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                        && *scan as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                        && (*p as ::core::ffi::c_int >= FIRST_NL
                            && *p as ::core::ffi::c_int <= LAST_NL))
                    {
                        break;
                    }
                    scan = scan.offset(1);
                }
            }
            count += 1;
        }
    }
    (*rex.ptr()).input = scan;
    return count as ::core::ffi::c_int;
}
