//! The register array, and who is allowed to reach into it.
//!
//! `y_regs` is 39 slots, and [`op_reg_index`] is the only place their layout
//! is written down:
//!
//! | slot | register |
//! | --- | --- |
//! | 0..=9 | `"0`..`"9` |
//! | 10..=35 | `"a`..`"z` (an uppercase name is the same slot, appending) |
//! | 36 | `"-`, the small-delete register |
//! | 37 | `"*`, the primary selection |
//! | 38 | `"+`, the clipboard |
//!
//! [`get_yank_register`] is the front door, and the reason this file is not a
//! plain accessor list: the same call means three different things
//! (`YREG_PASTE`, `YREG_YANK`, `YREG_PUT`), an unnamed request may be
//! redirected to the clipboard by 'clipboard', a `"*`/`"+` read has to ask the
//! provider first, and an unnamed *paste* reads whatever was last written
//! rather than slot 0. [`op_reg_iter`] and [`op_reg_set`] are shada's view of
//! the same array.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::types::NUL;

/// Whether `c` is an ASCII letter, in either case.
#[inline]
fn is_ascii_letter(c: c_int) -> bool {
    (b'a' as c_int..=b'z' as c_int).contains(&c) || (b'A' as c_int..=b'Z' as c_int).contains(&c)
}

/// The index of the slot `""` currently points at, or -1 when nothing has
/// been written yet.
///
/// # Safety
/// Reads the register store; main thread only.
pub unsafe fn get_unname_register() -> c_int {
    unsafe {
        if y_previous.get().is_null() {
            -1
        } else {
            y_previous.get().offset_from(get_y_register(0)) as c_int
        }
    }
}

/// Slot `reg` of the register array.
///
/// # Safety
/// `reg` must be a valid index, as [`op_reg_index`] answers.
pub unsafe fn get_y_register(reg: c_int) -> *mut yankreg_T {
    unsafe { (y_regs.ptr() as *mut yankreg_T).offset(reg as isize) }
}

/// The register `""` points at, or null.
pub fn get_y_previous() -> *mut yankreg_T {
    y_previous.get()
}

/// Whether `regname` names a register.
///
/// With `writing`, the read-only registers (`"/ ". "% ": "=`) are rejected.
/// 0 -- the default register -- is *not* handled here; the caller must check
/// for it. The black hole `"_` counts as valid.
///
/// # Safety
/// Reads 'comments'-independent globals only; main thread only.
pub unsafe fn valid_yank_reg(regname: c_int, writing: bool) -> bool {
    unsafe {
        regname > 0 && (is_ascii_letter(regname) || ascii_isdigit(regname))
            || !writing && !vim_strchr(c"/.%:=".as_ptr(), regname).is_null()
            || regname == '#' as c_int
            || regname == '"' as c_int
            || regname == '-' as c_int
            || regname == '_' as c_int
            || regname == '*' as c_int
            || regname == '+' as c_int
    }
}

/// Which clipboard register an unnamed paste should use, or `NUL` when
/// 'clipboard' does not ask for one (or no provider is available).
///
/// # Safety
/// May call the clipboard provider, which runs Lua.
pub unsafe fn get_default_register_name() -> c_int {
    unsafe {
        let mut name = NUL;
        clipboard::adjust_clipboard_name(&mut name, true, false);
        name
    }
}

/// Iterate over the non-empty registers of `regs`.
///
/// Pass a null `iter` to start; the answer is what to pass next, or null when
/// the iteration is over. Only the shada-saved slots (below `"*`) are walked.
///
/// # Safety
/// `regs` must point at an array of at least `NUM_SAVED_REGISTERS` registers;
/// `name`, `reg` and `is_unnamed` must be writable.
pub unsafe fn op_reg_iter(
    iter: *const c_void,
    regs: *const yankreg_T,
    name: *mut c_char,
    reg: *mut yankreg_T,
    is_unnamed: *mut bool,
) -> *const c_void {
    unsafe {
        *name = NUL as c_char;
        let mut iter_reg = if iter.is_null() {
            regs
        } else {
            iter as *const yankreg_T
        };
        while iter_reg.offset_from(regs) < NUM_SAVED_REGISTERS as isize && reg_empty(iter_reg) {
            iter_reg = iter_reg.add(1);
        }
        if iter_reg.offset_from(regs) == NUM_SAVED_REGISTERS as isize || reg_empty(iter_reg) {
            return ::core::ptr::null();
        }

        *name = get_register_name(iter_reg.offset_from(regs) as c_int) as c_char;
        *reg = *iter_reg;
        *is_unnamed = core::ptr::eq(iter_reg, y_previous.get());

        // Look ahead for the next non-empty one, which is what the caller
        // passes back in.
        loop {
            iter_reg = iter_reg.add(1);
            if iter_reg.offset_from(regs) >= NUM_SAVED_REGISTERS as isize {
                return ::core::ptr::null();
            }
            if !reg_empty(iter_reg) {
                return iter_reg as *const c_void;
            }
        }
    }
}

/// [`op_reg_iter`] over the global register array.
///
/// # Safety
/// `name`, `reg` and `is_unnamed` must be writable.
pub unsafe fn op_global_reg_iter(
    iter: *const c_void,
    name: *mut c_char,
    reg: *mut yankreg_T,
    is_unnamed: *mut bool,
) -> *const c_void {
    unsafe { op_reg_iter(iter, get_y_register(0), name, reg, is_unnamed) }
}

/// Put `reg` in the register named `name`, freeing whatever was there.
///
/// Answers false for a name with no slot.
///
/// # Safety
/// `reg` must own its `y_array` and additional data; the store takes them.
pub unsafe fn op_reg_set(name: c_char, reg: yankreg_T, is_unnamed: bool) -> bool {
    unsafe {
        let i = op_reg_index(c_int::from(name));
        if i == -1 {
            return false;
        }
        free_register(get_y_register(i));
        *get_y_register(i) = reg;
        if is_unnamed {
            y_previous.set(get_y_register(i));
        }
        true
    }
}

/// The contents of the register named `name`, or null for a name with no slot.
///
/// # Safety
/// Reads the register store; main thread only.
pub unsafe fn op_reg_get(name: c_char) -> *const yankreg_T {
    unsafe {
        let i = op_reg_index(c_int::from(name));
        if i == -1 {
            return ::core::ptr::null();
        }
        get_y_register(i)
    }
}

/// Point `""` at the register named `name`.
///
/// Answers false for a name with no slot.
///
/// # Safety
/// Writes the register store; main thread only.
pub unsafe fn op_reg_set_previous(name: c_char) -> bool {
    unsafe {
        let i = op_reg_index(c_int::from(name));
        if i == -1 {
            return false;
        }
        y_previous.set(get_y_register(i));
        true
    }
}

/// Widen a blockwise register's `y_width` to its widest line, in *cells*.
///
/// Does nothing to a register that is not blockwise.
///
/// # Safety
/// `reg` must point at a register whose `y_array` holds `y_size` strings.
pub unsafe fn update_yankreg_width(reg: *mut yankreg_T) {
    unsafe {
        if (*reg).y_type != kMTBlockWise {
            return;
        }
        let mut maxlen: size_t = 0;
        for i in 0..(*reg).y_size {
            let line = *(*reg).y_array.add(i);
            maxlen = maxlen.max(mb_string2cells_len(line.data(), line.len()));
        }
        debug_assert!(maxlen <= c_int::MAX as size_t);
        (*reg).y_width = (*reg).y_width.max(maxlen as c_int - 1);
    }
}

/// The register `regname` names, for one of the three `YREG_*` purposes.
///
/// Cannot handle the black hole `"_`, and must only be called with a valid
/// register name.
///
/// - `YREG_PASTE` -- about to paste it. With no name, read whatever was
///   written last, or the unnamed clipboard if 'clipboard' says so; a
///   `"*`/`"+` read queries the provider.
/// - `YREG_YANK` -- about to yank into it. With no name, yank into `"0`, and
///   move `""` to it.
/// - `YREG_PUT` -- only report where a paste *would* read from, without
///   querying the provider.
///
/// # Safety
/// `regname` must be a valid register name (see [`valid_yank_reg`]). May run
/// the clipboard provider, and so arbitrary Lua.
pub unsafe fn get_yank_register(regname: c_int, mode: c_int) -> *mut yankreg_T {
    unsafe {
        let mut reg: *mut yankreg_T = ::core::ptr::null_mut();
        if (mode == YREG_PASTE || mode == YREG_PUT)
            && clipboard::get_clipboard(regname, &mut reg, false)
        {
            return reg;
        }
        if mode == YREG_PUT && (regname == '*' as c_int || regname == '+' as c_int) {
            // Reporting only: hand back an empty register rather than asking
            // the provider what the clipboard holds.
            static empty_reg: GlobalCell<yankreg_T> = GlobalCell::new(EMPTY_YANKREG);
            return empty_reg.ptr();
        }
        if mode != YREG_YANK
            && (regname == 0
                || regname == '"' as c_int
                || regname == '*' as c_int
                || regname == '+' as c_int)
            && !y_previous.get().is_null()
        {
            // An unnamed paste reads the last register written.
            return y_previous.get();
        }

        let i = op_reg_index(regname);
        let reg = get_y_register(if i == -1 { 0 } else { i });
        if mode == YREG_YANK {
            y_previous.set(reg);
        }
        reg
    }
}

/// Whether the register `regname` holds linewise text, also handing back the
/// register prepared for pasting.
///
/// `*reg` is set to null for an invalid or black-hole register.
///
/// # Safety
/// `reg` must be writable. May run the clipboard provider.
pub unsafe fn yank_register_mline(regname: c_int, reg: *mut *mut yankreg_T) -> bool {
    unsafe {
        *reg = ::core::ptr::null_mut();
        if regname != 0 && !valid_yank_reg(regname, false) {
            return false;
        }
        if regname == '_' as c_int {
            return false; // black hole
        }
        *reg = get_yank_register(regname, YREG_PASTE);
        (**reg).y_type == kMTLineWise
    }
}

/// A deep copy of register `name`, for `do_put` to work on.
///
/// The caller frees it with [`free_register`] and `xfree`.
///
/// # Safety
/// `name` must be a valid register name. May run the clipboard provider.
pub unsafe fn copy_register(name: c_int) -> *mut yankreg_T {
    unsafe {
        let reg = get_yank_register(name, YREG_PASTE);
        let copy = xmalloc(::core::mem::size_of::<yankreg_T>()) as *mut yankreg_T;
        *copy = *reg;
        if (*copy).y_size == 0 {
            (*copy).y_array = ::core::ptr::null_mut();
        } else {
            (*copy).y_array =
                xcalloc((*copy).y_size, ::core::mem::size_of::<String_0>()) as *mut String_0;
            for i in 0..(*copy).y_size {
                *(*copy).y_array.add(i) =
                    copy_string(*(*reg).y_array.add(i), ::core::ptr::null_mut());
            }
        }
        copy
    }
}

/// Shift the numbered delete registers `"1`..`"8` up one, freeing `"9`, so
/// that a new delete can go into `"1`.
///
/// With `y_append` the caller is appending to an uppercase register, so `""`
/// is left where it is.
///
/// # Safety
/// Writes the register store; main thread only.
pub unsafe fn shift_delete_registers(y_append: bool) {
    unsafe {
        free_register(get_y_register(9));
        for n in (2..=9).rev() {
            *get_y_register(n) = *get_y_register(n - 1);
        }
        if !y_append {
            y_previous.set(get_y_register(1));
        }
        // `"1`'s array now belongs to `"2`: forget it rather than free it.
        (*get_y_register(1)).y_array = ::core::ptr::null_mut();
    }
}

/// Free everything a register owns and leave it empty.
///
/// # Safety
/// `reg` must own its `y_array` and additional data.
pub unsafe fn free_register(reg: *mut yankreg_T) {
    unsafe {
        xfree((*reg).additional_data as *mut c_void);
        (*reg).additional_data = ::core::ptr::null_mut();
        if (*reg).y_array.is_null() {
            return;
        }
        for i in (0..(*reg).y_size).rev() {
            let line = &mut *(*reg).y_array.add(i);
            xfree(line.data() as *mut c_void);
            line.set_data(::core::ptr::null_mut());
            line.set_len(0);
        }
        xfree((*reg).y_array as *mut c_void);
        (*reg).y_array = ::core::ptr::null_mut();
    }
}

/// Whether text put in `regname` is taken literally -- that is, whether the
/// register is one of the real slots rather than a computed one.
#[inline]
pub fn is_literal_register(regname: c_int) -> bool {
    regname == '*' as c_int
        || regname == '+' as c_int
        || is_ascii_letter(regname)
        || ascii_isdigit(regname)
}

/// The slot `regname` names, or -1 for a name with no slot.
#[inline]
pub fn op_reg_index(regname: c_int) -> c_int {
    if ascii_isdigit(regname) {
        regname - '0' as c_int
    } else if (b'a' as c_int..=b'z' as c_int).contains(&regname) {
        regname - 'a' as c_int + 10
    } else if (b'A' as c_int..=b'Z' as c_int).contains(&regname) {
        // An uppercase name is the same slot, appending rather than replacing.
        regname - 'A' as c_int + 10
    } else if regname == '-' as c_int {
        DELETION_REGISTER
    } else if regname == '*' as c_int {
        STAR_REGISTER
    } else if regname == '+' as c_int {
        PLUS_REGISTER
    } else {
        -1
    }
}

/// Whether `regname` is an uppercase name, which appends to its register
/// rather than replacing it.
#[inline]
pub(crate) fn is_append_register(regname: c_int) -> bool {
    (b'A' as c_int..=b'Z' as c_int).contains(&regname)
}

/// The name of slot `num`; -1 is the unnamed register.
#[inline]
pub fn get_register_name(num: c_int) -> c_int {
    if num == -1 {
        '"' as c_int
    } else if num < 10 {
        num + '0' as c_int
    } else if num == DELETION_REGISTER {
        '-' as c_int
    } else if num == STAR_REGISTER {
        '*' as c_int
    } else if num == PLUS_REGISTER {
        '+' as c_int
    } else {
        num + 'a' as c_int - 10
    }
}

/// Whether a register holds nothing -- either no array at all, or the one
/// empty charwise line an empty yank leaves behind.
///
/// # Safety
/// `reg` must point at a register whose `y_array` holds `y_size` strings.
#[inline]
unsafe fn reg_empty(reg: *const yankreg_T) -> bool {
    unsafe {
        (*reg).y_array.is_null()
            || (*reg).y_size == 0
            || (*reg).y_size == 1 && (*reg).y_type == kMTCharWise && (*(*reg).y_array).is_empty()
    }
}
