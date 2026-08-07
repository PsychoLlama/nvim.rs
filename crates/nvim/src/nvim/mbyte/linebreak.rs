//! Where a line may be broken between two characters.
//!
//! `'linebreak'` needs to know whether a break is allowed between a given pair,
//! which is not symmetric: CJK punctuation that may not *start* a line is a
//! different set from the punctuation that may not *end* one.  `utf_allow_break`
//! asks both halves.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn utf_eat_space(mut cc: ::core::ffi::c_int) -> bool {
    return cc >= 0x2000 as ::core::ffi::c_int && cc <= 0x206f as ::core::ffi::c_int
        || cc >= 0x2e00 as ::core::ffi::c_int && cc <= 0x2e7f as ::core::ffi::c_int
        || cc >= 0x3000 as ::core::ffi::c_int && cc <= 0x303f as ::core::ffi::c_int
        || cc >= 0xff01 as ::core::ffi::c_int && cc <= 0xff0f as ::core::ffi::c_int
        || cc >= 0xff1a as ::core::ffi::c_int && cc <= 0xff20 as ::core::ffi::c_int
        || cc >= 0xff3b as ::core::ffi::c_int && cc <= 0xff40 as ::core::ffi::c_int
        || cc >= 0xff5b as ::core::ffi::c_int && cc <= 0xff65 as ::core::ffi::c_int;
}

pub unsafe extern "C" fn utf_allow_break_before(mut cc: ::core::ffi::c_int) -> bool {
    unsafe {
        static BOL_prohibition_punct: GlobalCell<[::core::ffi::c_int; 43]> = GlobalCell::new([
            '!' as ::core::ffi::c_int,
            '%' as ::core::ffi::c_int,
            ')' as ::core::ffi::c_int,
            ',' as ::core::ffi::c_int,
            ':' as ::core::ffi::c_int,
            ';' as ::core::ffi::c_int,
            '>' as ::core::ffi::c_int,
            '?' as ::core::ffi::c_int,
            ']' as ::core::ffi::c_int,
            '}' as ::core::ffi::c_int,
            0x2019 as ::core::ffi::c_int,
            0x201d as ::core::ffi::c_int,
            0x2020 as ::core::ffi::c_int,
            0x2021 as ::core::ffi::c_int,
            0x2026 as ::core::ffi::c_int,
            0x2030 as ::core::ffi::c_int,
            0x2031 as ::core::ffi::c_int,
            0x203c as ::core::ffi::c_int,
            0x2047 as ::core::ffi::c_int,
            0x2048 as ::core::ffi::c_int,
            0x2049 as ::core::ffi::c_int,
            0x2103 as ::core::ffi::c_int,
            0x2109 as ::core::ffi::c_int,
            0x3001 as ::core::ffi::c_int,
            0x3002 as ::core::ffi::c_int,
            0x3009 as ::core::ffi::c_int,
            0x300b as ::core::ffi::c_int,
            0x300d as ::core::ffi::c_int,
            0x300f as ::core::ffi::c_int,
            0x3011 as ::core::ffi::c_int,
            0x3015 as ::core::ffi::c_int,
            0x3017 as ::core::ffi::c_int,
            0x3019 as ::core::ffi::c_int,
            0x301b as ::core::ffi::c_int,
            0xff01 as ::core::ffi::c_int,
            0xff09 as ::core::ffi::c_int,
            0xff0c as ::core::ffi::c_int,
            0xff0e as ::core::ffi::c_int,
            0xff1a as ::core::ffi::c_int,
            0xff1b as ::core::ffi::c_int,
            0xff1f as ::core::ffi::c_int,
            0xff3d as ::core::ffi::c_int,
            0xff5d as ::core::ffi::c_int,
        ]);
        let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 43]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_int; 43]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize)
            as ::core::ffi::c_int;
        while first < last {
            let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
            if cc == (*BOL_prohibition_punct.ptr())[mid as usize] {
                return false_0 != 0;
            } else if cc > (*BOL_prohibition_punct.ptr())[mid as usize] {
                first = mid + 1 as ::core::ffi::c_int;
            } else {
                last = mid - 1 as ::core::ffi::c_int;
            }
        }
        return cc != (*BOL_prohibition_punct.ptr())[first as usize];
    }
}

pub unsafe extern "C" fn utf_allow_break_after(mut cc: ::core::ffi::c_int) -> bool {
    unsafe {
        static EOL_prohibition_punct: GlobalCell<[::core::ffi::c_int; 19]> = GlobalCell::new([
            '(' as ::core::ffi::c_int,
            '<' as ::core::ffi::c_int,
            '[' as ::core::ffi::c_int,
            '`' as ::core::ffi::c_int,
            '{' as ::core::ffi::c_int,
            0x2018 as ::core::ffi::c_int,
            0x201c as ::core::ffi::c_int,
            0x3008 as ::core::ffi::c_int,
            0x300a as ::core::ffi::c_int,
            0x300c as ::core::ffi::c_int,
            0x300e as ::core::ffi::c_int,
            0x3010 as ::core::ffi::c_int,
            0x3014 as ::core::ffi::c_int,
            0x3016 as ::core::ffi::c_int,
            0x3018 as ::core::ffi::c_int,
            0x301a as ::core::ffi::c_int,
            0xff08 as ::core::ffi::c_int,
            0xff3b as ::core::ffi::c_int,
            0xff5b as ::core::ffi::c_int,
        ]);
        let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 19]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_int; 19]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize)
            as ::core::ffi::c_int;
        while first < last {
            let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
            if cc == (*EOL_prohibition_punct.ptr())[mid as usize] {
                return false_0 != 0;
            } else if cc > (*EOL_prohibition_punct.ptr())[mid as usize] {
                first = mid + 1 as ::core::ffi::c_int;
            } else {
                last = mid - 1 as ::core::ffi::c_int;
            }
        }
        return cc != (*EOL_prohibition_punct.ptr())[first as usize];
    }
}

pub unsafe extern "C" fn utf_allow_break(
    mut cc: ::core::ffi::c_int,
    mut ncc: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if cc == ncc && (cc == 0x2014 as ::core::ffi::c_int || cc == 0x2026 as ::core::ffi::c_int) {
            return false_0 != 0;
        }
        return utf_allow_break_after(cc) as ::core::ffi::c_int != 0
            && utf_allow_break_before(ncc) as ::core::ffi::c_int != 0;
    }
}
