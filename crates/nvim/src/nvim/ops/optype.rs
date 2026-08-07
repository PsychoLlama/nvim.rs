//! The operator vocabulary: which keys name an operator, and what it does
//! to a region.
//!
//! `opchars` is the table upstream keeps in lock-step with the `OP_*` order,
//! one row per operator: the first character, the optional second (`g~`, `zf`,
//! `g@`) and two flags -- `OPF_LINES`, meaning the operator always works on
//! whole lines, and `OPF_CHANGE`, meaning it modifies the buffer.  Everything
//! else here reads one column of it.  `get_op_type` is the reverse lookup
//! normal mode does on the keys it just read, with five special cases (`r`,
//! `~`, `g CTRL-A`, `g CTRL-X`, `zy`) that the table cannot express.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static opchars: GlobalCell<[[::core::ffi::c_char; 3]; 30]> = GlobalCell::new([
    [
        NUL as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'd' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'y' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'c' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        '<' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        '>' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        '!' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '~' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        '=' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'q' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        ':' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'U' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'u' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'J' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'J' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '?' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'r' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'I' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'A' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'f' as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'o' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'O' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'c' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'C' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'd' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'D' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'w' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '@' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        Ctrl_A as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        Ctrl_X as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
]);

pub unsafe extern "C" fn get_op_type(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        if char1 == 'r' as ::core::ffi::c_int {
            return OP_REPLACE;
        }
        if char1 == '~' as ::core::ffi::c_int {
            return OP_TILDE;
        }
        if char1 == 'g' as ::core::ffi::c_int && char2 == Ctrl_A {
            return OP_NR_ADD;
        }
        if char1 == 'g' as ::core::ffi::c_int && char2 == Ctrl_X {
            return OP_NR_SUB;
        }
        if char1 == 'z' as ::core::ffi::c_int && char2 == 'y' as ::core::ffi::c_int {
            return OP_YANK;
        }
        i = 0 as ::core::ffi::c_int;
        while !((*opchars.ptr())[i as usize][0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            == char1
            && (*opchars.ptr())[i as usize][1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == char2)
        {
            if i == ::core::mem::size_of::<[[::core::ffi::c_char; 3]; 30]>()
                .wrapping_div(::core::mem::size_of::<[::core::ffi::c_char; 3]>())
                .wrapping_div(
                    (::core::mem::size_of::<[[::core::ffi::c_char; 3]; 30]>()
                        .wrapping_rem(::core::mem::size_of::<[::core::ffi::c_char; 3]>())
                        == 0) as ::core::ffi::c_int as usize,
                )
                .wrapping_sub(1 as usize) as ::core::ffi::c_int
            {
                internal_error(b"get_op_type()\0".as_ptr() as *const ::core::ffi::c_char);
                break;
            } else {
                i += 1;
            }
        }
        return i;
    }
}

pub unsafe extern "C" fn op_on_lines(mut op: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return (*opchars.ptr())[op as usize][2 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            & OPF_LINES;
    }
}

pub unsafe extern "C" fn op_is_change(mut op: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return (*opchars.ptr())[op as usize][2 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            & OPF_CHANGE;
    }
}

pub unsafe extern "C" fn get_op_char(mut optype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return (*opchars.ptr())[optype as usize][0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn get_extra_op_char(mut optype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return (*opchars.ptr())[optype as usize][1 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
    }
}
