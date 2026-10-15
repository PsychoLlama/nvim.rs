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
//! rather than slot 0. [`shada_registers`] and [`shada_set_register`] are
//! shada's view of the same array.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::shada::ShadaExtra;
use crate::strings::has_char;
use crate::types::{ColNr, MotionType, Timestamp};
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
pub(crate) fn get_unname_register() -> c_int {
    y_previous.get().unwrap_or(-1)
}

/// Slot `reg` of the register array.
///
/// Panics unless `reg` is a valid index, as [`op_reg_index`] answers.
pub fn get_y_register(reg: c_int) -> *mut YankReg {
    let reg = usize::try_from(reg).expect("a register index");
    assert!(reg < NUM_REGISTERS as usize, "a register index");
    y_regs.ptr().cast::<YankReg>().wrapping_add(reg)
}

/// The register `""` points at, or null.
pub fn get_y_previous() -> *mut YankReg {
    y_previous
        .get()
        .map_or(::core::ptr::null_mut(), get_y_register)
}

/// Whether `regname` names a register.
///
/// With `writing`, the read-only registers (`"/ ". "% ": "=`) are rejected.
/// 0 -- the default register -- is *not* handled here; the caller must check
/// for it. The black hole `"_` counts as valid.
pub fn valid_yank_reg(regname: c_int, writing: bool) -> bool {
    regname > 0 && (is_ascii_letter(regname) || ascii_isdigit(regname))
        || !writing && has_char(c"/.%:=", regname)
        || regname == '#' as c_int
        || regname == '"' as c_int
        || regname == '-' as c_int
        || regname == '_' as c_int
        || regname == '*' as c_int
        || regname == '+' as c_int
}

/// Which clipboard register an unnamed paste should use, or `NUL` when
/// 'clipboard' does not ask for one (or no provider is available).
pub fn get_default_register_name() -> c_int {
    let mut name = NUL;
    unsafe { clipboard::adjust_clipboard_name(&mut name, true, false) };
    name
}

/// One register as ShaDa reads and writes it: its lines, how it is put
/// back, when it was set, and the extra data the file gave it.
pub(crate) struct ShadaYank {
    pub(crate) contents: Vec<String_0>,
    pub(crate) motion: MotionType,
    pub(crate) width: ColNr,
    pub(crate) timestamp: Timestamp,
    pub(crate) extra: Option<ShadaExtra>,
}

/// A copy of every non-empty register shada saves (those below `"*`), with
/// its name and whether `""` points at it, in slot order. A register of
/// more than `max_lines` lines is left out.
pub(crate) fn shada_registers(max_lines: Option<usize>) -> Vec<(c_char, bool, ShadaYank)> {
    let mut found = Vec::new();
    for i in 0..NUM_SAVED_REGISTERS {
        let reg = get_y_register(i);
        // SAFETY: a slot of the register array, which owns `y_size` strings
        // at `y_array` and its extra data; both are only read here.
        unsafe {
            if reg_empty(reg) || max_lines.is_some_and(|max| (*reg).y_size > max) {
                continue;
            }
            let contents = core::slice::from_raw_parts((*reg).y_array, (*reg).y_size).to_vec();
            let yank = ShadaYank {
                contents,
                motion: (*reg).y_type,
                width: (*reg).y_width,
                timestamp: (*reg).timestamp,
                extra: crate::mark::extra_copy_raw((*reg).additional_data),
            };
            let name = get_register_name(i) as c_char;
            found.push((name, core::ptr::eq(reg, get_y_previous()), yank));
        }
    }
    found
}

/// When the register named `name` was set, or `None` for a name with no
/// slot.
pub(crate) fn shada_register_timestamp(name: c_char) -> Option<Timestamp> {
    let i = op_reg_index(c_int::from(name));
    // SAFETY: a slot of the register array, read.
    (i != -1).then(|| unsafe { (*get_y_register(i)).timestamp })
}

/// Put `yank` in the register named `name`, freeing whatever was there, and
/// point `""` at it when `is_unnamed`. Answers false for a name with no
/// slot.
pub(crate) fn shada_set_register(name: c_char, yank: ShadaYank, is_unnamed: bool) -> bool {
    let i = op_reg_index(c_int::from(name));
    if i == -1 {
        return false;
    }
    let size = yank.contents.len();
    // SAFETY: an `xmalloc` array of exactly `size` strings, each written
    // before the register can read it; the register owns it and the extra
    // data from here, and frees both in `free_register`.
    let reg = unsafe {
        let array = xmalloc(size * size_of::<String_0>()).cast::<String_0>();
        for (at, line) in yank.contents.into_iter().enumerate() {
            array.add(at).write(line);
        }
        YankReg {
            y_array: array,
            y_size: size,
            y_type: yank.motion,
            y_width: yank.width,
            timestamp: yank.timestamp,
            additional_data: crate::mark::extra_into_raw(yank.extra),
        }
    };
    // SAFETY: the slot owns its array and extra data, released before the
    // new register is written over it.
    unsafe { free_register(get_y_register(i)) };
    unsafe { *get_y_register(i) = reg };
    if is_unnamed {
        y_previous.set(Some(i));
    }
    true
}

/// Point `""` at the register named `name`.
///
/// Answers false for a name with no slot.
pub fn op_reg_set_previous(name: c_char) -> bool {
    let i = op_reg_index(c_int::from(name));
    if i == -1 {
        return false;
    }
    y_previous.set(Some(i));
    true
}

/// Point `""` at register `regname`'s slot -- the first slot, for a name
/// that has none -- the way a yank into it would.
pub(crate) fn point_unnamed_at(regname: c_int) {
    y_previous.set(Some(op_reg_index(regname).max(0)));
}

/// Widen a blockwise register's `y_width` to its widest line, in *cells*.
///
/// Does nothing to a register that is not blockwise.
///
/// # Safety
/// `reg` must point at a register whose `y_array` holds `y_size` strings.
pub unsafe fn update_yankreg_width(reg: *mut YankReg) {
    if unsafe { (*reg).y_type } != kMTBlockWise {
        return;
    }
    let mut maxlen: size_t = 0;
    for i in 0..unsafe { (*reg).y_size } {
        let line = unsafe { &*(*reg).y_array.add(i) };
        maxlen = maxlen.max(unsafe { mb_string2cells_len(line.data(), line.len()) });
    }
    debug_assert!(maxlen <= c_int::MAX as size_t);
    unsafe { (*reg).y_width = (*reg).y_width.max(maxlen as c_int - 1) };
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
pub unsafe fn get_yank_register(regname: c_int, mode: c_int) -> *mut YankReg {
    let mut reg: *mut YankReg = ::core::ptr::null_mut();
    if (mode == YREG_PASTE || mode == YREG_PUT)
        && unsafe { clipboard::get_clipboard(regname, &mut reg, false) }
    {
        return reg;
    }
    if mode == YREG_PUT && (regname == '*' as c_int || regname == '+' as c_int) {
        // Reporting only: hand back an empty register rather than asking
        // the provider what the clipboard holds.
        static empty_reg: GlobalCell<YankReg> = GlobalCell::new(EMPTY_YANKREG);
        return empty_reg.ptr();
    }
    if mode != YREG_YANK
        && (regname == 0
            || regname == '"' as c_int
            || regname == '*' as c_int
            || regname == '+' as c_int)
        && let Some(previous) = y_previous.get()
    {
        // An unnamed paste reads the last register written.
        return get_y_register(previous);
    }

    let i = op_reg_index(regname).max(0);
    if mode == YREG_YANK {
        y_previous.set(Some(i));
    }
    get_y_register(i)
}

/// Whether the register `regname` holds linewise text, also handing back the
/// register prepared for pasting.
///
/// `*reg` is set to null for an invalid or black-hole register.
///
/// # Safety
/// `reg` must be writable. May run the clipboard provider.
pub unsafe fn yank_register_mline(regname: c_int, reg: *mut *mut YankReg) -> bool {
    unsafe { *reg = ::core::ptr::null_mut() };
    if regname != 0 && !valid_yank_reg(regname, false) {
        return false;
    }
    if regname == '_' as c_int {
        return false; // black hole
    }
    unsafe { *reg = get_yank_register(regname, YREG_PASTE) };
    unsafe { (**reg).y_type == kMTLineWise }
}

/// A deep copy of register `name`, for `do_put` to work on.
///
/// The caller frees it with [`free_register`] and `xfree`.
///
/// # Safety
/// `name` must be a valid register name. May run the clipboard provider.
pub unsafe fn copy_register(name: c_int) -> *mut YankReg {
    let reg = unsafe { get_yank_register(name, YREG_PASTE) };
    let copy = unsafe { xmalloc(::core::mem::size_of::<YankReg>()) } as *mut YankReg;
    unsafe { *copy = *reg };
    if unsafe { (*copy).y_size } == 0 {
        unsafe { (*copy).y_array = ::core::ptr::null_mut() };
    } else {
        unsafe {
            (*copy).y_array =
                xcalloc((*copy).y_size, ::core::mem::size_of::<String_0>()) as *mut String_0
        };
        for i in 0..unsafe { (*copy).y_size } {
            // SAFETY: `xcalloc` zeroed the destination, which is the null
            // string, so the assignment has nothing to release.
            unsafe { *(*copy).y_array.add(i) = (*(*reg).y_array.add(i)).clone() };
        }
    }
    copy
}

/// Shift the numbered delete registers `"1`..`"8` up one, freeing `"9`, so
/// that a new delete can go into `"1`.
///
/// With `y_append` the caller is appending to an uppercase register, so `""`
/// is left where it is.
pub fn shift_delete_registers(y_append: bool) {
    unsafe { free_register(get_y_register(9)) };
    for n in (2..=9).rev() {
        unsafe { *get_y_register(n) = *get_y_register(n - 1) };
    }
    if !y_append {
        y_previous.set(Some(1));
    }
    // `"1`'s array now belongs to `"2`: forget it rather than free it.
    unsafe { (*get_y_register(1)).y_array = ::core::ptr::null_mut() };
}

/// Free everything a register owns and leave it empty.
///
/// # Safety
/// `reg` must own its `y_array` and additional data.
pub unsafe fn free_register(reg: *mut YankReg) {
    unsafe { xfree((*reg).additional_data as *mut c_void) };
    unsafe { (*reg).additional_data = ::core::ptr::null_mut() };
    if unsafe { (*reg).y_array }.is_null() {
        return;
    }
    for i in (0..unsafe { (*reg).y_size }).rev() {
        // SAFETY: the register owns `y_size` strings at `y_array`, each of
        // them initialised.
        unsafe { (*reg).y_array.add(i).drop_in_place() };
    }
    unsafe { xfree((*reg).y_array as *mut c_void) };
    unsafe { (*reg).y_array = ::core::ptr::null_mut() };
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
unsafe fn reg_empty(reg: *const YankReg) -> bool {
    unsafe { (*reg).y_array }.is_null()
        || unsafe { (*reg).y_size } == 0
        || unsafe { (*reg).y_size } == 1
            && unsafe { (*reg).y_type } == kMTCharWise
            && unsafe { (*(*reg).y_array).is_empty() }
}
