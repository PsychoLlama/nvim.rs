//! The register array, and who is allowed to reach into it.
//!
//! `y_regs` is 39 slots -- `"a`..`"z`, `"0`..`"9`, the small-delete `"-`, and
//! `"*`/`"+` -- indexed by `op_reg_index`, which is the only place the layout
//! is written down.  `get_yank_register` is the front door and the reason this
//! file is not a plain accessor list: the same call has to mean three different
//! things (`YREG_PASTE`, `YREG_YANK`, `YREG_PUT`), an uppercase name means
//! "append to the lowercase one", an unnamed request may be redirected to the
//! clipboard by 'clipboard', and a `"*`/`"+` read has to ask the provider
//! first.  `op_reg_iter`/`op_reg_set` are shada's view of the same array.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_unname_register() -> ::core::ffi::c_int {
    unsafe {
        return if (*y_previous.ptr()).is_null() {
            -1 as ::core::ffi::c_int
        } else {
            (*y_previous.ptr()).offset_from(
                (y_regs.ptr() as *mut yankreg_T).offset(0 as ::core::ffi::c_int as isize),
            ) as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn get_y_register(mut reg: ::core::ffi::c_int) -> *mut yankreg_T {
    unsafe {
        return (y_regs.ptr() as *mut yankreg_T).offset(reg as isize);
    }
}

pub unsafe extern "C" fn get_y_previous() -> *mut yankreg_T {
    return y_previous.get();
}

pub unsafe extern "C" fn valid_yank_reg(
    mut regname: ::core::ffi::c_int,
    mut writing: bool,
) -> bool {
    unsafe {
        if regname > 0 as ::core::ffi::c_int
            && (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(regname) as ::core::ffi::c_int != 0)
            || !writing
                && !vim_strchr(b"/.%:=\0".as_ptr() as *const ::core::ffi::c_char, regname).is_null()
            || regname == '#' as ::core::ffi::c_int
            || regname == '"' as ::core::ffi::c_int
            || regname == '-' as ::core::ffi::c_int
            || regname == '_' as ::core::ffi::c_int
            || regname == '*' as ::core::ffi::c_int
            || regname == '+' as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn get_default_register_name() -> ::core::ffi::c_int {
    unsafe {
        let mut name: ::core::ffi::c_int = NUL;
        clipboard::adjust_clipboard_name(&mut name, true, false);
        return name;
    }
}

pub unsafe extern "C" fn op_reg_iter(
    iter: *const ::core::ffi::c_void,
    regs: *const yankreg_T,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    unsafe {
        *name = NUL as ::core::ffi::c_char;
        let mut iter_reg: *const yankreg_T = if iter.is_null() {
            regs.offset(0 as ::core::ffi::c_int as isize)
        } else {
            iter as *const yankreg_T
        };
        while iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
            < NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
            && reg_empty(iter_reg) as ::core::ffi::c_int != 0
        {
            iter_reg = iter_reg.offset(1);
        }
        if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
            == NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
            || reg_empty(iter_reg) as ::core::ffi::c_int != 0
        {
            return ::core::ptr::null::<::core::ffi::c_void>();
        }
        let mut iter_off: ::core::ffi::c_int = iter_reg
            .offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
            as ::core::ffi::c_int;
        *name = get_register_name(iter_off) as ::core::ffi::c_char;
        *reg = *iter_reg;
        *is_unnamed = iter_reg == y_previous.get() as *const yankreg_T;
        loop {
            iter_reg = iter_reg.offset(1);
            if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
                >= NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
            {
                break;
            }
            if !reg_empty(iter_reg) {
                return iter_reg as *mut ::core::ffi::c_void;
            }
        }
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn op_global_reg_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    unsafe {
        return op_reg_iter(iter, y_regs.ptr() as *mut yankreg_T, name, reg, is_unnamed);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn op_reg_set(
    name: ::core::ffi::c_char,
    reg: yankreg_T,
    mut is_unnamed: bool,
) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
        if i == -1 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        free_register((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
        (*y_regs.ptr())[i as usize] = reg;
        if is_unnamed {
            y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn op_reg_get(name: ::core::ffi::c_char) -> *const yankreg_T {
    unsafe {
        let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
        if i == -1 as ::core::ffi::c_int {
            return ::core::ptr::null::<yankreg_T>();
        }
        return (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
    }
}

pub unsafe extern "C" fn op_reg_set_previous(name: ::core::ffi::c_char) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
        if i == -1 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn update_yankreg_width(mut reg: *mut yankreg_T) {
    unsafe {
        if (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            let mut maxlen: size_t = 0 as size_t;
            let mut i: size_t = 0 as size_t;
            while i < (*reg).y_size {
                let mut rowlen: size_t = mb_string2cells_len(
                    (*(*reg).y_array.offset(i as isize)).data,
                    (*(*reg).y_array.offset(i as isize)).size,
                );
                maxlen = if maxlen > rowlen { maxlen } else { rowlen };
                i = i.wrapping_add(1);
            }
            '_c2rust_label: {
                if maxlen <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"maxlen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        295 as ::core::ffi::c_uint,
                        b"void update_yankreg_width(yankreg_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*reg).y_width =
                (if (*reg).y_width > maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
                    (*reg).y_width as ::core::ffi::c_int
                } else {
                    maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                }) as colnr_T;
        }
    }
}

pub unsafe extern "C" fn get_yank_register(
    mut regname: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> *mut yankreg_T {
    unsafe {
        let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        if (mode == YREG_PASTE as ::core::ffi::c_int || mode == YREG_PUT as ::core::ffi::c_int)
            && clipboard::get_clipboard(regname, &mut reg, false)
        {
            return reg;
        } else if mode == YREG_PUT as ::core::ffi::c_int
            && (regname == '*' as ::core::ffi::c_int || regname == '+' as ::core::ffi::c_int)
        {
            static empty_reg: GlobalCell<yankreg_T> = GlobalCell::new(yankreg_T {
                y_array: ::core::ptr::null_mut::<String_0>(),
                y_size: 0,
                y_type: kMTCharWise,
                y_width: 0,
                timestamp: 0,
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            });
            return empty_reg.ptr();
        } else if mode != YREG_YANK as ::core::ffi::c_int
            && (regname == 0 as ::core::ffi::c_int
                || regname == '"' as ::core::ffi::c_int
                || regname == '*' as ::core::ffi::c_int
                || regname == '+' as ::core::ffi::c_int)
            && !(*y_previous.ptr()).is_null()
        {
            return y_previous.get();
        }
        let mut i: ::core::ffi::c_int = op_reg_index(regname);
        if i == -1 as ::core::ffi::c_int {
            i = 0 as ::core::ffi::c_int;
        }
        reg = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
        if mode == YREG_YANK as ::core::ffi::c_int {
            y_previous.set(reg);
        }
        return reg;
    }
}

pub unsafe extern "C" fn yank_register_mline(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut *mut yankreg_T,
) -> bool {
    unsafe {
        *reg = ::core::ptr::null_mut::<yankreg_T>();
        if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, false_0 != 0) {
            return false_0 != 0;
        }
        if regname == '_' as ::core::ffi::c_int {
            return false_0 != 0;
        }
        *reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        return (**reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn copy_register(mut name: ::core::ffi::c_int) -> *mut yankreg_T {
    unsafe {
        let mut reg: *mut yankreg_T = get_yank_register(name, YREG_PASTE as ::core::ffi::c_int);
        let mut copy: *mut yankreg_T =
            xmalloc(::core::mem::size_of::<yankreg_T>()) as *mut yankreg_T;
        *copy = *reg;
        if (*copy).y_size == 0 as size_t {
            (*copy).y_array = ::core::ptr::null_mut::<String_0>();
        } else {
            (*copy).y_array =
                xcalloc((*copy).y_size, ::core::mem::size_of::<String_0>()) as *mut String_0;
            let mut i: size_t = 0 as size_t;
            while i < (*copy).y_size {
                *(*copy).y_array.offset(i as isize) = copy_string(
                    *(*reg).y_array.offset(i as isize),
                    ::core::ptr::null_mut::<Arena>(),
                );
                i = i.wrapping_add(1);
            }
        }
        return copy;
    }
}

pub unsafe extern "C" fn shift_delete_registers(mut y_append: bool) {
    unsafe {
        free_register((y_regs.ptr() as *mut yankreg_T).offset(9 as ::core::ffi::c_int as isize));
        let mut n: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
        while n > 1 as ::core::ffi::c_int {
            (*y_regs.ptr())[n as usize] = (*y_regs.ptr())[(n - 1 as ::core::ffi::c_int) as usize];
            n -= 1;
        }
        if !y_append {
            y_previous
                .set((y_regs.ptr() as *mut yankreg_T).offset(1 as ::core::ffi::c_int as isize));
        }
        (*y_regs.ptr())[1 as ::core::ffi::c_int as usize].y_array =
            ::core::ptr::null_mut::<String_0>();
    }
}

pub unsafe extern "C" fn free_register(mut reg: *mut yankreg_T) {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*reg).additional_data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        if (*reg).y_array.is_null() {
            return;
        }
        let mut i: size_t = (*reg).y_size;
        loop {
            let c2rust_fresh0 = i;
            i = i.wrapping_sub(1);
            if c2rust_fresh0 <= 0 as size_t {
                break;
            }
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*(*reg).y_array.offset(i as isize)).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
            (*(*reg).y_array.offset(i as isize)).size = 0 as size_t;
        }
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*reg).y_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        let _ = *ptr__1;
    }
}

#[inline]
pub unsafe fn is_literal_register(regname: ::core::ffi::c_int) -> bool {
    return regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
        || (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0);
}

#[inline]
pub unsafe fn op_reg_index(regname: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if ascii_isdigit(regname) {
        return regname - '0' as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname == '-' as ::core::ffi::c_int {
        return DELETION_REGISTER as ::core::ffi::c_int;
    } else if regname == '*' as ::core::ffi::c_int {
        return STAR_REGISTER as ::core::ffi::c_int;
    } else if regname == '+' as ::core::ffi::c_int {
        return PLUS_REGISTER as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}

#[inline]
pub(crate) unsafe extern "C" fn is_append_register(mut regname: ::core::ffi::c_int) -> bool {
    return regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint;
}

#[inline]
pub unsafe fn get_register_name(mut num: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if num == -1 as ::core::ffi::c_int {
        return '"' as ::core::ffi::c_int;
    } else if num < 10 as ::core::ffi::c_int {
        return num + '0' as ::core::ffi::c_int;
    } else if num == DELETION_REGISTER as ::core::ffi::c_int {
        return '-' as ::core::ffi::c_int;
    } else if num == STAR_REGISTER as ::core::ffi::c_int {
        return '*' as ::core::ffi::c_int;
    } else if num == PLUS_REGISTER as ::core::ffi::c_int {
        return '+' as ::core::ffi::c_int;
    } else {
        return num + 'a' as ::core::ffi::c_int - 10 as ::core::ffi::c_int;
    };
}

#[inline]
unsafe extern "C" fn reg_empty(reg: *const yankreg_T) -> bool {
    unsafe {
        return (*reg).y_array.is_null()
            || (*reg).y_size == 0 as size_t
            || (*reg).y_size == 1 as size_t
                && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                && (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size == 0 as size_t;
    }
}
