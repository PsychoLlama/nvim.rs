//! Character classes, and what is printable.
//!
//! `utf_class_tab` answers which class a codepoint belongs to -- the number
//! `charclass()` reports and the equivalence `w`/`b` and `iskeyword` matching are
//! built on.  `utf_printable` is the separate question of whether a codepoint has
//! a visible glyph, which drives whether `strtrans()` escapes it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct clinterval {
    pub first: ::core::ffi::c_uint,
    pub last: ::core::ffi::c_uint,
    pub cls: ::core::ffi::c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct interval {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
}

pub unsafe extern "C" fn mb_get_class(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        return mb_get_class_tab(p, &raw mut (*curbuf.get()).b_chartab as *mut uint64_t);
    }
}

pub unsafe extern "C" fn mb_get_class_tab(
    mut p: *const ::core::ffi::c_char,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    unsafe {
        if (*utf8len_tab.ptr())[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                return 0 as ::core::ffi::c_int;
            }
            if vim_iswordc_tab(
                *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
                chartab,
            ) {
                return 2 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        return utf_class_tab(utf_ptr2char(p), chartab);
    }
}

pub(crate) unsafe extern "C" fn prop_is_emojilike(mut prop: *const utf8proc_property_t) -> bool {
    unsafe {
        return (*prop).boundclass as ::core::ffi::c_int
            == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
            || (*prop).boundclass as ::core::ffi::c_int
                == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn intable(
    mut table: *const interval,
    mut n_items: size_t,
    mut c: ::core::ffi::c_int,
) -> bool {
    unsafe {
        assert!(n_items > 0 as size_t, "n_items > 0");
        if c < (*table.offset(0 as ::core::ffi::c_int as isize)).first {
            return false_0 != 0;
        }
        assert!(
            n_items <= (18446744073709551615 as size_t).wrapping_div(2 as size_t),
            "n_items <= SIZE_MAX / 2"
        );
        let mut bot: size_t = 0 as size_t;
        let mut top: size_t = n_items;
        loop {
            let mut mid: size_t = bot.wrapping_add(top) >> 1 as ::core::ffi::c_int;
            if (*table.offset(mid as isize)).last < c {
                bot = mid.wrapping_add(1 as size_t);
            } else if (*table.offset(mid as isize)).first > c {
                top = mid;
            } else {
                return true_0 != 0;
            }
            if top <= bot {
                break;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn utf_printable(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        static nonprint: GlobalCell<[interval; 9]> = GlobalCell::new([
            interval {
                first: 0x70f as ::core::ffi::c_int,
                last: 0x70f as ::core::ffi::c_int,
            },
            interval {
                first: 0x180b as ::core::ffi::c_int,
                last: 0x180e as ::core::ffi::c_int,
            },
            interval {
                first: 0x200b as ::core::ffi::c_int,
                last: 0x200f as ::core::ffi::c_int,
            },
            interval {
                first: 0x202a as ::core::ffi::c_int,
                last: 0x202e as ::core::ffi::c_int,
            },
            interval {
                first: 0x2060 as ::core::ffi::c_int,
                last: 0x206f as ::core::ffi::c_int,
            },
            interval {
                first: 0xd800 as ::core::ffi::c_int,
                last: 0xdfff as ::core::ffi::c_int,
            },
            interval {
                first: 0xfeff as ::core::ffi::c_int,
                last: 0xfeff as ::core::ffi::c_int,
            },
            interval {
                first: 0xfff9 as ::core::ffi::c_int,
                last: 0xfffb as ::core::ffi::c_int,
            },
            interval {
                first: 0xfffe as ::core::ffi::c_int,
                last: 0xffff as ::core::ffi::c_int,
            },
        ]);
        return !intable(
            (nonprint.ptr() as *const _) as *const interval,
            ::core::mem::size_of::<[interval; 9]>()
                .wrapping_div(::core::mem::size_of::<interval>())
                .wrapping_div(
                    (::core::mem::size_of::<[interval; 9]>()
                        .wrapping_rem(::core::mem::size_of::<interval>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            c,
        );
    }
}

pub unsafe extern "C" fn utf_class(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return utf_class_tab(c, &raw mut (*curbuf.get()).b_chartab as *mut uint64_t);
    }
}

pub unsafe extern "C" fn utf_class_tab(
    c: ::core::ffi::c_int,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    unsafe {
        static classes: GlobalCell<[clinterval; 71]> = GlobalCell::new([
            clinterval {
                first: 0x37e as ::core::ffi::c_uint,
                last: 0x37e as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x387 as ::core::ffi::c_uint,
                last: 0x387 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x55a as ::core::ffi::c_uint,
                last: 0x55f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x589 as ::core::ffi::c_uint,
                last: 0x589 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x5be as ::core::ffi::c_uint,
                last: 0x5be as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x5c0 as ::core::ffi::c_uint,
                last: 0x5c0 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x5c3 as ::core::ffi::c_uint,
                last: 0x5c3 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x5f3 as ::core::ffi::c_uint,
                last: 0x5f4 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x60c as ::core::ffi::c_uint,
                last: 0x60c as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x61b as ::core::ffi::c_uint,
                last: 0x61b as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x61f as ::core::ffi::c_uint,
                last: 0x61f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x66a as ::core::ffi::c_uint,
                last: 0x66d as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x6d4 as ::core::ffi::c_uint,
                last: 0x6d4 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x700 as ::core::ffi::c_uint,
                last: 0x70d as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x964 as ::core::ffi::c_uint,
                last: 0x965 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x970 as ::core::ffi::c_uint,
                last: 0x970 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xdf4 as ::core::ffi::c_uint,
                last: 0xdf4 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xe4f as ::core::ffi::c_uint,
                last: 0xe4f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xe5a as ::core::ffi::c_uint,
                last: 0xe5b as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xf04 as ::core::ffi::c_uint,
                last: 0xf12 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xf3a as ::core::ffi::c_uint,
                last: 0xf3d as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xf85 as ::core::ffi::c_uint,
                last: 0xf85 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x104a as ::core::ffi::c_uint,
                last: 0x104f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x10fb as ::core::ffi::c_uint,
                last: 0x10fb as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1361 as ::core::ffi::c_uint,
                last: 0x1368 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x166d as ::core::ffi::c_uint,
                last: 0x166e as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1680 as ::core::ffi::c_uint,
                last: 0x1680 as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x169b as ::core::ffi::c_uint,
                last: 0x169c as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x16eb as ::core::ffi::c_uint,
                last: 0x16ed as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1735 as ::core::ffi::c_uint,
                last: 0x1736 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x17d4 as ::core::ffi::c_uint,
                last: 0x17dc as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1800 as ::core::ffi::c_uint,
                last: 0x180a as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2000 as ::core::ffi::c_uint,
                last: 0x200b as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x200c as ::core::ffi::c_uint,
                last: 0x2027 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2028 as ::core::ffi::c_uint,
                last: 0x2029 as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x202a as ::core::ffi::c_uint,
                last: 0x202e as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x202f as ::core::ffi::c_uint,
                last: 0x202f as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2030 as ::core::ffi::c_uint,
                last: 0x205e as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x205f as ::core::ffi::c_uint,
                last: 0x205f as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2060 as ::core::ffi::c_uint,
                last: 0x206f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2070 as ::core::ffi::c_uint,
                last: 0x207f as ::core::ffi::c_uint,
                cls: 0x2070 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2080 as ::core::ffi::c_uint,
                last: 0x2094 as ::core::ffi::c_uint,
                cls: 0x2080 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x20a0 as ::core::ffi::c_uint,
                last: 0x27ff as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2800 as ::core::ffi::c_uint,
                last: 0x28ff as ::core::ffi::c_uint,
                cls: 0x2800 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2900 as ::core::ffi::c_uint,
                last: 0x2998 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x29d8 as ::core::ffi::c_uint,
                last: 0x29db as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x29fc as ::core::ffi::c_uint,
                last: 0x29fd as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2e00 as ::core::ffi::c_uint,
                last: 0x2e7f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x3000 as ::core::ffi::c_uint,
                last: 0x3000 as ::core::ffi::c_uint,
                cls: 0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x3001 as ::core::ffi::c_uint,
                last: 0x3020 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x3030 as ::core::ffi::c_uint,
                last: 0x3030 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x303d as ::core::ffi::c_uint,
                last: 0x303d as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x3040 as ::core::ffi::c_uint,
                last: 0x309f as ::core::ffi::c_uint,
                cls: 0x3040 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x30a0 as ::core::ffi::c_uint,
                last: 0x30ff as ::core::ffi::c_uint,
                cls: 0x30a0 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x3300 as ::core::ffi::c_uint,
                last: 0x9fff as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xac00 as ::core::ffi::c_uint,
                last: 0xd7a3 as ::core::ffi::c_uint,
                cls: 0xac00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xf900 as ::core::ffi::c_uint,
                last: 0xfaff as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xfd3e as ::core::ffi::c_uint,
                last: 0xfd3f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xfe30 as ::core::ffi::c_uint,
                last: 0xfe6b as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xff00 as ::core::ffi::c_uint,
                last: 0xff0f as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xff1a as ::core::ffi::c_uint,
                last: 0xff20 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xff3b as ::core::ffi::c_uint,
                last: 0xff40 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0xff5b as ::core::ffi::c_uint,
                last: 0xff65 as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1d000 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x1d24f as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1d400 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x1d7ff as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1f000 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x1f2ff as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x1f300 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x1f9ff as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 1 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x20000 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x2a6df as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2a700 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x2b73f as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2b740 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x2b81f as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
            clinterval {
                first: 0x2f800 as ::core::ffi::c_int as ::core::ffi::c_uint,
                last: 0x2fa1f as ::core::ffi::c_int as ::core::ffi::c_uint,
                cls: 0x4e00 as ::core::ffi::c_uint,
            },
        ]);
        let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut top: ::core::ffi::c_int = ::core::mem::size_of::<[clinterval; 71]>()
            .wrapping_div(::core::mem::size_of::<clinterval>())
            .wrapping_div(
                (::core::mem::size_of::<[clinterval; 71]>()
                    .wrapping_rem(::core::mem::size_of::<clinterval>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize)
            as ::core::ffi::c_int;
        if c < 0x100 as ::core::ffi::c_int {
            if c == ' ' as ::core::ffi::c_int
                || c == '\t' as ::core::ffi::c_int
                || c == NUL
                || c == 0xa0 as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
            if vim_iswordc_tab(c, chartab) {
                return 2 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
        if prop_is_emojilike(prop) {
            return 3 as ::core::ffi::c_int;
        }
        while top >= bot {
            let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
            if (*classes.ptr())[mid as usize].last < c as ::core::ffi::c_uint {
                bot = mid + 1 as ::core::ffi::c_int;
            } else if (*classes.ptr())[mid as usize].first > c as ::core::ffi::c_uint {
                top = mid - 1 as ::core::ffi::c_int;
            } else {
                return (*classes.ptr())[mid as usize].cls as ::core::ffi::c_int;
            }
        }
        return 2 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_charclass(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string
                .is_null()
        {
            return;
        }
        (*rettv).vval.v_number = mb_get_class(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string,
        ) as varnumber_T;
    }
}
