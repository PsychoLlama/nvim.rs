//! Display width: how many screen cells a character occupies.
//!
//! `utf_char2cells` is the answer for a codepoint -- 0 for a combining mark, 2
//! for a wide or fullwidth one, and for an ambiguous-width one whatever
//! `'ambiwidth'` says -- and the `ptr`/`string` spellings sum it over a buffer.
//! `setcellwidths()` overrides the table for chosen ranges; `cw_table` is that
//! override, kept sorted so `cw_value` can binary-search it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cw_interval_T {
    pub first: int64_t,
    pub last: int64_t,
    pub width: ::core::ffi::c_char,
}

static e_list_item_nr_is_not_list: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E1109: List item %d is not a List\0",
        )
    });

static e_list_item_nr_does_not_contain_3_numbers: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E1110: List item %d does not contain 3 numbers\0",
        )
    });

static e_list_item_nr_range_invalid: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E1111: List item %d range invalid\0",
        )
    });

static e_list_item_nr_cell_width_invalid: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1112: List item %d cell width invalid\0",
        )
    });

static e_overlapping_ranges_for_nr: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1113: Overlapping ranges for 0x%lx\0",
        )
    });

static e_only_values_of_0x80_and_higher_supported: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1114: Only values of 0x80 and higher supported\0",
        )
    });

pub unsafe extern "C" fn utf_char2cells(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if c < 0x80 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        if !vim_isprintc(c) {
            debug_assert!(c <= 0xffff as ::core::ffi::c_int, "c <= 0xFFFF");
            return if c > 0xff as ::core::ffi::c_int {
                6 as ::core::ffi::c_int
            } else {
                4 as ::core::ffi::c_int
            };
        }
        let mut n: ::core::ffi::c_int = cw_value(c);
        if n != 0 as ::core::ffi::c_int {
            return n;
        }
        let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
        if (*prop).charwidth as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            return 2 as ::core::ffi::c_int;
        }
        if *p_ambw.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int
            && (*prop).ambiguous_width
        {
            return 2 as ::core::ffi::c_int;
        }
        if p_emoji.get() != 0
            && c >= 0x1f000 as ::core::ffi::c_int
            && !(*prop).ambiguous_width
            && prop_is_emojilike(&*prop) as ::core::ffi::c_int != 0
        {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn utf_ptr2cells(mut p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const uint8_t = p_in as *const uint8_t;
        if *p as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
            let mut len: ::core::ffi::c_int = utf8len_tab[*p as usize] as ::core::ffi::c_int;
            let mut c: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
            if c <= 0 as int32_t {
                return 4 as ::core::ffi::c_int;
            }
            if c < 0x80 as int32_t {
                return char2cells(c as ::core::ffi::c_int);
            }
            let mut cells: ::core::ffi::c_int = utf_char2cells(c as ::core::ffi::c_int);
            if cells == 1 as ::core::ffi::c_int
                && p_emoji.get() != 0
                && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t))
                    as ::core::ffi::c_int
                    != 0
            {
                let mut c2: ::core::ffi::c_int = utf_ptr2char(p_in.offset(len as isize));
                if c2 == 0xfe0f as ::core::ffi::c_int {
                    return 2 as ::core::ffi::c_int;
                }
            }
            return cells;
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn utf_ptr2cells_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if size > 0 as ::core::ffi::c_int
            && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        {
            let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
            if len < utf8len_tab[*p as uint8_t as usize] as ::core::ffi::c_int {
                return 1 as ::core::ffi::c_int;
            }
            let mut c: ::core::ffi::c_int = utf_ptr2char(p);
            if utf_ptr2len(p) == 1 as ::core::ffi::c_int || c == NUL {
                return 4 as ::core::ffi::c_int;
            }
            if c < 0x80 as ::core::ffi::c_int {
                return char2cells(c);
            }
            let mut cells: ::core::ffi::c_int = utf_char2cells(c);
            if cells == 1 as ::core::ffi::c_int
                && p_emoji.get() != 0
                && size > len
                && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t))
                    as ::core::ffi::c_int
                    != 0
                && utf_ptr2len_len(p.offset(len as isize), size - len)
                    == utf8len_tab[*p.offset(len as isize) as uint8_t as usize]
                        as ::core::ffi::c_int
            {
                let mut c2: ::core::ffi::c_int = utf_ptr2char(p.offset(len as isize));
                if c2 == 0xfe0f as ::core::ffi::c_int {
                    return 2 as ::core::ffi::c_int;
                }
            }
            return cells;
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn mb_string2cells(mut str: *const ::core::ffi::c_char) -> size_t {
    unsafe {
        let mut clen: size_t = 0 as size_t;
        let mut p: *const ::core::ffi::c_char = str;
        while *p as ::core::ffi::c_int != NUL {
            clen = clen.wrapping_add(utf_ptr2cells(p) as size_t);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        return clen;
    }
}

pub unsafe extern "C" fn mb_string2cells_len(
    mut str: *const ::core::ffi::c_char,
    mut size: size_t,
) -> size_t {
    unsafe {
        let mut clen: size_t = 0 as size_t;
        let mut p: *const ::core::ffi::c_char = str;
        while *p as ::core::ffi::c_int != NUL && p < str.offset(size as isize) {
            clen = clen.wrapping_add(utf_ptr2cells_len(
                p,
                size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
            ) as size_t);
            p = p.offset(utfc_ptr2len_len(
                p,
                size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
            ) as isize);
        }
        return clen;
    }
}

pub unsafe extern "C" fn utf_ambiguous_width(mut p: *const ::core::ffi::c_char) -> bool {
    unsafe {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return false_0 != 0;
        }
        let mut info: CharInfo = utf_ptr2CharInfo(p);
        if info.value >= 0x80 as int32_t {
            let mut prop: *const utf8proc_property_t =
                utf8proc_get_property(info.value as utf8proc_int32_t);
            if (*prop).ambiguous_width || prop_is_emojilike(&*prop) as ::core::ffi::c_int != 0 {
                return true_0 != 0;
            }
        }
        return memcmp(
            p.offset(info.len as isize) as *const ::core::ffi::c_void,
            b"\xEF\xB8\x8F\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int;
    }
}

static cw_table: GlobalCell<*mut cw_interval_T> =
    GlobalCell::new(::core::ptr::null_mut::<cw_interval_T>());

static cw_table_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);

unsafe extern "C" fn cw_value(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if (*cw_table.ptr()).is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (c as int64_t) < (*(*cw_table.ptr()).offset(0 as ::core::ffi::c_int as isize)).first {
            return 0 as ::core::ffi::c_int;
        }
        let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut top: ::core::ffi::c_int =
            cw_table_size.get() as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while top >= bot {
            let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
            if (*(*cw_table.ptr()).offset(mid as isize)).last < c as int64_t {
                bot = mid + 1 as ::core::ffi::c_int;
            } else if (*(*cw_table.ptr()).offset(mid as isize)).first > c as int64_t {
                top = mid - 1 as ::core::ffi::c_int;
            } else {
                return (*(*cw_table.ptr()).offset(mid as isize)).width as ::core::ffi::c_int;
            }
        }
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn tv_nr_compare(
    mut a1: *const ::core::ffi::c_void,
    mut a2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let li1: *const listitem_T = tv_list_first(*(a1 as *mut *const list_T));
        let li2: *const listitem_T = tv_list_first(*(a2 as *mut *const list_T));
        let n1: varnumber_T = (*li1).li_tv.vval.v_number;
        let n2: varnumber_T = (*li2).li_tv.vval.v_number;
        return if n1 == n2 {
            0 as ::core::ffi::c_int
        } else if n1 > n2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn f_setcellwidths(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut ptrs: *mut *const list_T = ::core::ptr::null_mut::<*const list_T>();
        let mut item: ::core::ffi::c_int = 0;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list
                .is_null()
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        let l: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        let mut table: *mut cw_interval_T = ::core::ptr::null_mut::<cw_interval_T>();
        let table_size: size_t = tv_list_len(l) as size_t;
        if table_size != 0 as size_t {
            ptrs = xmalloc(::core::mem::size_of::<*const list_T>().wrapping_mul(table_size))
                as *mut *const list_T;
            item = 0 as ::core::ffi::c_int;
            let l_: *const list_T = l;
            if !l_.is_null() {
                let mut li: *const listitem_T = (*l_).lv_first;
                while !li.is_null() {
                    let li_tv: *const typval_T = &raw const (*li).li_tv;
                    if (*li_tv).v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*li_tv).vval.v_list.is_null()
                    {
                        semsg(
                            gettext(
                                (e_list_item_nr_is_not_list.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    }
                    let li_l: *const list_T = (*li_tv).vval.v_list;
                    *ptrs.offset(item as isize) = li_l;
                    let mut lili: *const listitem_T = tv_list_first(li_l);
                    let mut i: ::core::ffi::c_int = 0;
                    let mut n1: varnumber_T = 0;
                    i = 0 as ::core::ffi::c_int;
                    while !lili.is_null() {
                        let lili_tv: *const typval_T = &raw const (*lili).li_tv;
                        if (*lili_tv).v_type as ::core::ffi::c_uint
                            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        if i == 0 as ::core::ffi::c_int {
                            n1 = (*lili_tv).vval.v_number;
                            if n1 < 0x80 as varnumber_T {
                                emsg(gettext(
                                    (e_only_values_of_0x80_and_higher_supported.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ));
                                xfree(ptrs as *mut ::core::ffi::c_void);
                                return;
                            }
                        } else if i == 1 as ::core::ffi::c_int && (*lili_tv).vval.v_number < n1 {
                            semsg(
                                gettext(
                                    (e_list_item_nr_range_invalid.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                item,
                            );
                            xfree(ptrs as *mut ::core::ffi::c_void);
                            return;
                        } else if i == 2 as ::core::ffi::c_int
                            && ((*lili_tv).vval.v_number < 1 as varnumber_T
                                || (*lili_tv).vval.v_number > 2 as varnumber_T)
                        {
                            semsg(
                                gettext(
                                    (e_list_item_nr_cell_width_invalid.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                item,
                            );
                            xfree(ptrs as *mut ::core::ffi::c_void);
                            return;
                        }
                        lili = (*lili).li_next;
                        i += 1;
                    }
                    if i != 3 as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                (e_list_item_nr_does_not_contain_3_numbers.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    }
                    item += 1;
                    li = (*li).li_next;
                }
            }
            qsort(
                ptrs as *mut ::core::ffi::c_void,
                table_size,
                ::core::mem::size_of::<*const list_T>(),
                Some(
                    tv_nr_compare
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            table = xmalloc(::core::mem::size_of::<cw_interval_T>().wrapping_mul(table_size))
                as *mut cw_interval_T;
            item = 0 as ::core::ffi::c_int;
            while (item as size_t) < table_size {
                let li_l_0: *const list_T = *ptrs.offset(item as isize);
                let mut lili_0: *const listitem_T = tv_list_first(li_l_0);
                let n1_0: varnumber_T = (*lili_0).li_tv.vval.v_number;
                if item > 0 as ::core::ffi::c_int
                    && n1_0 <= (*table.offset((item - 1 as ::core::ffi::c_int) as isize)).last
                {
                    semsg(
                        gettext(
                            (e_overlapping_ranges_for_nr.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        n1_0 as size_t,
                    );
                    xfree(ptrs as *mut ::core::ffi::c_void);
                    xfree(table as *mut ::core::ffi::c_void);
                    return;
                }
                (*table.offset(item as isize)).first = n1_0 as int64_t;
                lili_0 = (*lili_0).li_next;
                (*table.offset(item as isize)).last = (*lili_0).li_tv.vval.v_number as int64_t;
                lili_0 = (*lili_0).li_next;
                (*table.offset(item as isize)).width =
                    (*lili_0).li_tv.vval.v_number as ::core::ffi::c_char;
                item += 1;
            }
            xfree(ptrs as *mut ::core::ffi::c_void);
        }
        let cw_table_save: *mut cw_interval_T = cw_table.get();
        let cw_table_size_save: size_t = cw_table_size.get();
        cw_table.set(table);
        cw_table_size.set(table_size);
        let error: *const ::core::ffi::c_char = check_chars_options();
        if !error.is_null() {
            emsg(gettext(error));
            cw_table.set(cw_table_save);
            cw_table_size.set(cw_table_size_save);
            xfree(table as *mut ::core::ffi::c_void);
            return;
        }
        xfree(cw_table_save as *mut ::core::ffi::c_void);
        changed_window_setting_all();
        redraw_all_later(UPD_NOT_VALID);
    }
}

pub unsafe extern "C" fn f_getcellwidths(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, cw_table_size.get() as ptrdiff_t);
        let mut i: size_t = 0 as size_t;
        while i < cw_table_size.get() {
            let mut entry: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
            tv_list_append_number(entry, (*(*cw_table.ptr()).offset(i as isize)).first);
            tv_list_append_number(entry, (*(*cw_table.ptr()).offset(i as isize)).last);
            tv_list_append_number(
                entry,
                (*(*cw_table.ptr()).offset(i as isize)).width as varnumber_T,
            );
            tv_list_append_list((*rettv).vval.v_list, entry);
            i = i.wrapping_add(1);
        }
    }
}
