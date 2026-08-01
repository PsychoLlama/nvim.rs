//! The sixteen cterm colour names, per `'t_Co'`.
//!
//! `ctermfg=`/`ctermbg=` accept a small set of names rather than the RGB
//! table, and which number each one means depends on how many colours the
//! terminal claims ([`lookup_color`]). The 8-colour case also has to fake
//! the light half by setting `bold`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) static color_names: GlobalCell<[*mut ::core::ffi::c_char; 28]> = GlobalCell::new([
    c"Black".as_ptr().cast_mut(),
    c"DarkBlue".as_ptr().cast_mut(),
    c"DarkGreen".as_ptr().cast_mut(),
    c"DarkCyan".as_ptr().cast_mut(),
    c"DarkRed".as_ptr().cast_mut(),
    c"DarkMagenta".as_ptr().cast_mut(),
    c"Brown".as_ptr().cast_mut(),
    c"DarkYellow".as_ptr().cast_mut(),
    c"Gray".as_ptr().cast_mut(),
    c"Grey".as_ptr().cast_mut(),
    c"LightGray".as_ptr().cast_mut(),
    c"LightGrey".as_ptr().cast_mut(),
    c"DarkGray".as_ptr().cast_mut(),
    c"DarkGrey".as_ptr().cast_mut(),
    c"Blue".as_ptr().cast_mut(),
    c"LightBlue".as_ptr().cast_mut(),
    c"Green".as_ptr().cast_mut(),
    c"LightGreen".as_ptr().cast_mut(),
    c"Cyan".as_ptr().cast_mut(),
    c"LightCyan".as_ptr().cast_mut(),
    c"Red".as_ptr().cast_mut(),
    c"LightRed".as_ptr().cast_mut(),
    c"Magenta".as_ptr().cast_mut(),
    c"LightMagenta".as_ptr().cast_mut(),
    c"Yellow".as_ptr().cast_mut(),
    c"LightYellow".as_ptr().cast_mut(),
    c"White".as_ptr().cast_mut(),
    c"NONE".as_ptr().cast_mut(),
]);

pub(crate) static color_numbers_16: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);

pub(crate) static color_numbers_88: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    32 as ::core::ffi::c_int,
    72 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    43 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    61 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    63 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    74 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    75 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    78 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);

pub(crate) static color_numbers_256: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    130 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    81 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    121 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    159 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    224 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    225 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    229 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);

pub(crate) static color_numbers_8: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);

pub(crate) unsafe extern "C" fn lookup_color(
    idx: ::core::ffi::c_int,
    foreground: bool,
    boldp: *mut TriState,
) -> ::core::ffi::c_int {
    unsafe {
        let mut color: ::core::ffi::c_int = (*color_numbers_16.ptr())[idx as usize];
        if color < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        if t_colors.get() == 8 as ::core::ffi::c_int {
            color = (*color_numbers_8.ptr())[idx as usize];
            if foreground {
                if color & 8 as ::core::ffi::c_int != 0 {
                    *boldp = kTrue;
                } else {
                    *boldp = kFalse;
                }
            }
            color &= 7 as ::core::ffi::c_int;
        } else if t_colors.get() == 16 as ::core::ffi::c_int {
            color = (*color_numbers_8.ptr())[idx as usize];
        } else if t_colors.get() == 88 as ::core::ffi::c_int {
            color = (*color_numbers_88.ptr())[idx as usize];
        } else if t_colors.get() >= 256 as ::core::ffi::c_int {
            color = (*color_numbers_256.ptr())[idx as usize];
        }
        return color;
    }
}

pub unsafe extern "C" fn name_to_ctermcolor(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut off: ::core::ffi::c_int = if (*name as ::core::ffi::c_int)
            < 'a' as ::core::ffi::c_int
            || *name as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
        {
            *name as ::core::ffi::c_int
        } else {
            *name as ::core::ffi::c_int - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        i = ::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if off
                == *(*color_names.ptr())[i as usize].offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                && strcasecmp(
                    name.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                    (*color_names.ptr())[i as usize].offset(1 as ::core::ffi::c_int as isize),
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
        }
        if i < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        let mut bold: TriState = kNone;
        return lookup_color(i, false_0 != 0, &raw mut bold);
    }
}
