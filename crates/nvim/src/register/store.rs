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
//!
//! A slot is reached through [`Reg::read`]/[`Reg::write`], whose closure is
//! the whole of the borrow: nothing that runs user code may run inside one.
//! A caller that has to hold a register across such code -- a put, a replay,
//! the listing -- takes [`Reg::snapshot`], which shares the lines rather than
//! copying them (see [`RegLines`]).

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::mbyte::c_string_cells_len;
use crate::shada::ShadaExtra;
use crate::strings::has_char;
use crate::types::{ColNr, MotionType, RegLines, Timestamp};
use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// Whether `c` is an ASCII letter, in either case.
#[inline]
fn is_ascii_letter(c: c_int) -> bool {
    (b'a' as c_int..=b'z' as c_int).contains(&c) || (b'A' as c_int..=b'Z' as c_int).contains(&c)
}

/// A register a name resolved to: one of the 39 slots, or the empty register
/// a `YREG_PUT` of `"*`/`"+` reports without asking the provider (upstream's
/// `static yankreg_T empty_reg`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reg {
    Slot(usize),
    Empty,
}

impl Reg {
    /// Run `f` on the register.
    ///
    /// `f` must not reach the register store again -- no user code, no
    /// clipboard provider: the borrow is the cell's (a debug build panics on
    /// a nested one).
    pub fn read<R>(self, f: impl FnOnce(&YankReg) -> R) -> R {
        match self {
            Reg::Slot(slot) => y_regs.with(|regs| f(&regs[slot])),
            Reg::Empty => f(&YankReg::EMPTY),
        }
    }

    /// Run `f` on the register, to change it. As [`read`](Self::read); a
    /// write to the empty register is dropped, as nothing reads it back.
    pub fn write<R>(self, f: impl FnOnce(&mut YankReg) -> R) -> R {
        match self {
            Reg::Slot(slot) => y_regs.with_mut(|regs| f(&mut regs[slot])),
            Reg::Empty => f(&mut YankReg::EMPTY.clone()),
        }
    }

    /// The register as it is now, to hold across code that may write it:
    /// the lines are shared, not copied.
    pub fn snapshot(self) -> YankReg {
        self.read(YankReg::clone)
    }
}

/// The index of the slot `""` currently points at, or -1 when nothing has
/// been written yet.
pub(crate) fn get_unname_register() -> c_int {
    y_previous.get().unwrap_or(-1)
}

/// Slot `reg` of the register array.
///
/// Panics unless `reg` is a valid index, as [`op_reg_index`] answers.
pub fn get_y_register(reg: c_int) -> Reg {
    let reg = usize::try_from(reg).expect("a register index");
    assert!(reg < NUM_REGISTERS as usize, "a register index");
    Reg::Slot(reg)
}

/// The register `""` points at, if anything has been written.
pub fn get_y_previous() -> Option<Reg> {
    y_previous.get().map(get_y_register)
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
    let _ = clipboard::adjust_clipboard_name(&mut name, true, false);
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
    let previous = y_previous.get();
    y_regs.with(|regs| {
        (0..NUM_SAVED_REGISTERS)
            .filter_map(|i| {
                let reg = &regs[i as usize];
                if reg_empty(reg) || max_lines.is_some_and(|max| reg.lines.len() > max) {
                    return None;
                }
                let yank = ShadaYank {
                    contents: reg
                        .lines
                        .iter()
                        .map(|line| String_0::from_bytes(line))
                        .collect(),
                    motion: reg.kind,
                    width: reg.width,
                    timestamp: reg.timestamp,
                    extra: reg.extra.clone(),
                };
                let name = get_register_name(i) as c_char;
                Some((name, previous == Some(i), yank))
            })
            .collect()
    })
}

/// Look at the register named `name`, or answer `None` for a name with no
/// slot.
fn with_named_register<R>(name: c_char, f: impl FnOnce(&YankReg) -> R) -> Option<R> {
    let i = op_reg_index(c_int::from(name));
    (i != -1).then(|| get_y_register(i).read(f))
}

/// When the register named `name` was set, or `None` for a name with no
/// slot.
pub(crate) fn shada_register_timestamp(name: c_char) -> Option<Timestamp> {
    with_named_register(name, |reg| reg.timestamp)
}

/// When the register named `name` was emptied, or `None` when it holds
/// something (or the name has no slot) -- so that ShaDa does not bring back
/// a register this session cleared after the file was written.
pub(crate) fn shada_cleared_register_timestamp(name: c_char) -> Option<Timestamp> {
    with_named_register(name, |reg| reg_empty(reg).then_some(reg.timestamp)).flatten()
}

/// `bytes` as a register line. A line holds no NUL -- in memory a NUL in the
/// text is the newline that stands for it -- and ShaDa's own lines never have
/// one, since they are written from memory; a file that does is read with
/// the NUL as that newline, as `setreg()` reads it.
fn shada_line(bytes: &[u8]) -> XString {
    XString::from_vec(
        bytes
            .iter()
            .map(|&b| if b == 0 { b'\n' } else { b })
            .collect(),
    )
}

/// Put `yank` in the register named `name`, replacing whatever was there,
/// and point `""` at it when `is_unnamed`. Answers false for a name with no
/// slot.
pub(crate) fn shada_set_register(name: c_char, yank: ShadaYank, is_unnamed: bool) -> bool {
    let i = op_reg_index(c_int::from(name));
    if i == -1 {
        return false;
    }
    let lines = yank
        .contents
        .iter()
        .map(|line| shada_line(line.as_bytes()))
        .collect();
    let reg = YankReg {
        lines: RegLines::new(lines),
        kind: yank.motion,
        width: yank.width,
        timestamp: yank.timestamp,
        extra: yank.extra,
    };
    get_y_register(i).write(|slot| *slot = reg);
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

impl YankReg {
    /// Widen a blockwise register's `width` to its widest line, in *cells*.
    ///
    /// Does nothing to a register that is not blockwise.
    pub fn update_width(&mut self) {
        if self.kind != kMTBlockWise {
            return;
        }
        let widest = self
            .lines
            .iter()
            .map(|line| c_string_cells_len(line, line.len()))
            .max()
            .unwrap_or(0);
        let widest = c_int::try_from(widest).expect("a register line is narrower than c_int");
        self.width = self.width.max(widest - 1);
    }
}

/// The register `regname` names, for one of the three `YREG_*` purposes.
///
/// Cannot handle the black hole `"_`, and must only be called with a valid
/// register name (see [`valid_yank_reg`]).
///
/// - `YREG_PASTE` -- about to paste it. With no name, read whatever was
///   written last, or the unnamed clipboard if 'clipboard' says so; a
///   `"*`/`"+` read queries the provider, and so may run arbitrary Lua.
/// - `YREG_YANK` -- about to yank into it. With no name, yank into `"0`, and
///   move `""` to it.
/// - `YREG_PUT` -- only report where a paste *would* read from; a `"*`/`"+`
///   the provider does not answer is [`Reg::Empty`].
pub fn get_yank_register(regname: c_int, mode: c_int) -> Reg {
    if (mode == YREG_PASTE || mode == YREG_PUT)
        && let Some((slot, true)) = clipboard::get_clipboard(regname, false)
    {
        return Reg::Slot(slot);
    }
    if mode == YREG_PUT && (regname == '*' as c_int || regname == '+' as c_int) {
        // Reporting only: hand back an empty register rather than asking
        // the provider what the clipboard holds.
        return Reg::Empty;
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

/// Whether the register `regname` holds linewise text, and the register
/// as it is, prepared for pasting -- `None` for an invalid or black-hole
/// name. May run the clipboard provider.
pub fn yank_register_mline(regname: c_int) -> (bool, Option<YankReg>) {
    if regname != 0 && !valid_yank_reg(regname, false) {
        return (false, None);
    }
    if regname == '_' as c_int {
        return (false, None); // black hole
    }
    let reg = get_yank_register(regname, YREG_PASTE).snapshot();
    (reg.kind == kMTLineWise, Some(reg))
}

/// Register `name` as it is now, for `do_put` to work on after something has
/// overwritten it. May run the clipboard provider.
pub fn copy_register(name: c_int) -> YankReg {
    get_yank_register(name, YREG_PASTE).snapshot()
}

/// Shift the numbered delete registers `"1`..`"8` up one, dropping `"9`, so
/// that a new delete can go into `"1`.
///
/// With `y_append` the caller is appending to an uppercase register, so `""`
/// is left where it is.
pub fn shift_delete_registers(y_append: bool) {
    y_regs.with_mut(|regs| {
        regs[1..=9].rotate_right(1);
        // `"1` is left unset, keeping the type, width and time of the
        // register that moved to `"2` -- upstream copies the struct and nulls
        // only the array. The extra data moved with it: upstream leaves `"1`
        // and `"2` sharing one block, which the yank into `"1` then frees.
        let moved = &regs[2];
        regs[1] = YankReg {
            lines: RegLines::NONE,
            kind: moved.kind,
            width: moved.width,
            timestamp: moved.timestamp,
            extra: None,
        };
    });
    if !y_append {
        y_previous.set(Some(1));
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

/// Whether a register holds nothing -- either no lines at all, or the one
/// empty charwise line an empty yank leaves behind.
#[inline]
fn reg_empty(reg: &YankReg) -> bool {
    match reg.lines.as_slice() {
        [] => true,
        [only] => reg.kind == kMTCharWise && only.is_empty(),
        _ => false,
    }
}
