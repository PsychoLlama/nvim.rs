//! The register array, and who is allowed to reach into it.
//!
//! `Y_REGS` is 39 slots, and [`op_reg_index`] is the only place their layout
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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::c_string_cells_len;
use crate::normal::defs::MotionType;
use crate::register::defs::RegLines;
use crate::shada::ShadaExtra;
use crate::types::{Key, Timestamp};
use core::ffi::{c_char, c_int};

use super::*;

/// A register's name: the character typed after `"`, `@`, `q` or CTRL-R, or
/// [`RegName::UNNAMED`] (0) for none -- the default register.
///
/// [`RegName::valid`] is the constructor that checks the name is one (what
/// upstream's `valid_yank_reg` asks); [`RegName::from_byte`] carries a byte
/// that arrived from text (`:registers` arguments, ShaDa, `getreg()`), whose
/// readers check or answer for an unknown name themselves. A key that is not
/// a byte -- a special key, a character past U+00FF -- names no register,
/// which [`register_name`] says with `None`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct RegName(u8);

impl RegName {
    /// No name: the unnamed register, or "not given".
    pub const UNNAMED: RegName = RegName(0);
    /// `""`: the unnamed register, named.
    pub const QUOTE: RegName = RegName(b'"');
    /// `"*`: the selection.
    pub const STAR: RegName = RegName(b'*');
    /// `"+`: the clipboard.
    pub const PLUS: RegName = RegName(b'+');
    /// `"_`: the black hole.
    pub const BLACK_HOLE: RegName = RegName(b'_');
    /// `"-`: the small delete register.
    pub const SMALL_DELETE: RegName = RegName(b'-');
    /// `".`: the last inserted text.
    pub const LAST_INSERTED: RegName = RegName(b'.');
    /// `":`: the last command line.
    pub const LAST_COMMAND: RegName = RegName(b':');
    /// `"%`: the current file name.
    pub const FILE_NAME: RegName = RegName(b'%');
    /// `"/`: the last search pattern.
    pub const LAST_SEARCH: RegName = RegName(b'/');
    /// `"=`: the expression register.
    pub const EXPRESSION: RegName = RegName(b'=');

    /// The register `name` names, if it is one. With `writing`, the
    /// read-only registers (`"/ ". "% ": "=`) are refused. The unnamed
    /// register (0) is not a name typed and is refused too; the black hole
    /// `"_` is a register.
    pub fn valid(name: u8, writing: bool) -> Option<RegName> {
        let ok = name.is_ascii_alphanumeric()
            || !writing && b"/.%:=".contains(&name)
            || b"#\"-_*+".contains(&name);
        ok.then_some(RegName(name))
    }

    /// The byte as a name, whether or not it names a register.
    pub const fn from_byte(name: u8) -> RegName {
        RegName(name)
    }

    /// The character that types the name; 0 for the unnamed register.
    pub const fn byte(self) -> u8 {
        self.0
    }

    /// Whether this is no name at all.
    pub const fn is_unnamed(self) -> bool {
        self.0 == 0
    }

    /// Whether this names a register: [`valid`](Self::valid) for a name
    /// already held.
    pub fn is_valid(self, writing: bool) -> bool {
        RegName::valid(self.0, writing).is_some()
    }
}

/// A name compares with the character that types it: `name == b'_'`.
impl PartialEq<u8> for RegName {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

/// The register a typed key names, or `None` when the key is not even a
/// byte. The name still has to pass [`RegName::valid`].
#[inline]
pub(crate) fn register_name(key: Key) -> Option<RegName> {
    key.as_byte().map(RegName)
}

/// The key that types register `name`, for the redo buffer and the
/// typeahead.
#[inline]
pub(crate) fn register_key(name: RegName) -> Key {
    Key::byte(name.0)
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
    ///
    /// Crate-private, as [`write`](Self::write) is: a `pub` generic is
    /// instantiated by whichever crate calls it, which makes `Y_REGS` an
    /// exported symbol (`just state-exports`).
    pub(crate) fn read<R>(self, f: impl FnOnce(&YankReg) -> R) -> R {
        match self {
            Reg::Slot(slot) => Y_REGS.with(|regs| f(&regs[slot])),
            Reg::Empty => f(&YankReg::EMPTY),
        }
    }

    /// Run `f` on the register, to change it. As [`read`](Self::read); a
    /// write to the empty register is dropped, as nothing reads it back.
    pub(crate) fn write<R>(self, f: impl FnOnce(&mut YankReg) -> R) -> R {
        match self {
            Reg::Slot(slot) => Y_REGS.with_mut(|regs| f(&mut regs[slot])),
            Reg::Empty => f(&mut YankReg::EMPTY.clone()),
        }
    }

    /// The register as it is now, to hold across code that may write it:
    /// the lines are shared, not copied.
    pub fn snapshot(self) -> YankReg {
        self.read(YankReg::clone)
    }
}

/// The slot `""` currently points at, or `None` when nothing has been
/// written yet.
pub(crate) fn get_unname_register() -> Option<usize> {
    Y_PREVIOUS.get()
}

/// Slot `reg` of the register array.
///
/// Panics unless `reg` is a valid index, as [`op_reg_index`] answers.
pub fn get_y_register(reg: usize) -> Reg {
    assert!(reg < NUM_REGISTERS, "a register index");
    Reg::Slot(reg)
}

/// The register `""` points at, if anything has been written.
pub fn get_y_previous() -> Option<Reg> {
    Y_PREVIOUS.get().map(get_y_register)
}

/// Which clipboard register an unnamed paste should use, or 0 when
/// 'clipboard' does not ask for one (or no provider is available).
pub fn get_default_register_name() -> RegName {
    let mut name = RegName::UNNAMED;
    let _ = clipboard::adjust_clipboard_name(&mut name, true, false);
    name
}

/// One register as ShaDa reads and writes it: its lines, how it is put
/// back, when it was set, and the extra data the file gave it.
#[derive(Debug)]
pub(crate) struct ShadaYank {
    pub(crate) contents: Vec<ApiString>,
    pub(crate) motion: MotionType,
    pub(crate) width: i32,
    pub(crate) timestamp: Timestamp,
    pub(crate) extra: Option<ShadaExtra>,
}

/// A copy of every non-empty register shada saves (those below `"*`), with
/// its name and whether `""` points at it, in slot order. A register of
/// more than `max_lines` lines is left out.
pub(crate) fn shada_registers(max_lines: Option<usize>) -> Vec<(c_char, bool, ShadaYank)> {
    let previous = Y_PREVIOUS.get();
    Y_REGS.with(|regs| {
        (0..NUM_SAVED_REGISTERS)
            .filter_map(|i| {
                let reg = &regs[i];
                if reg_empty(reg) || max_lines.is_some_and(|max| reg.lines.len() > max) {
                    return None;
                }
                let yank = ShadaYank {
                    contents: reg
                        .lines
                        .iter()
                        .map(|line| ApiString::from_bytes(line))
                        .collect(),
                    motion: reg.kind,
                    width: reg.width,
                    timestamp: reg.timestamp,
                    extra: reg.extra.clone(),
                };
                let name = get_register_name(Some(i)).byte().cast_signed();
                Some((name, previous == Some(i), yank))
            })
            .collect()
    })
}

/// Look at the register named `name`, or answer `None` for a name with no
/// slot.
fn with_named_register<R>(name: c_char, f: impl FnOnce(&YankReg) -> R) -> Option<R> {
    op_reg_index(RegName::from_byte(name.cast_unsigned())).map(|i| get_y_register(i).read(f))
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
    let Some(i) = op_reg_index(RegName::from_byte(name.cast_unsigned())) else {
        return false;
    };
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
        Y_PREVIOUS.set(Some(i));
    }
    true
}

/// Point `""` at the register named `name`.
///
/// Answers false for a name with no slot.
pub fn op_reg_set_previous(name: RegName) -> bool {
    let Some(i) = op_reg_index(name) else {
        return false;
    };
    Y_PREVIOUS.set(Some(i));
    true
}

/// Point `""` at register `regname`'s slot -- the first slot, for a name
/// that has none -- the way a yank into it would.
pub(crate) fn point_unnamed_at(regname: RegName) {
    Y_PREVIOUS.set(Some(op_reg_index(regname).unwrap_or(0)));
}

impl YankReg {
    /// Widen a blockwise register's `width` to its widest line, in *cells*.
    ///
    /// Does nothing to a register that is not blockwise.
    pub fn update_width(&mut self) {
        if self.kind != MotionType::BlockWise {
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
/// register name (see [`RegName::valid`]).
///
/// - `YREG_PASTE` -- about to paste it. With no name, read whatever was
///   written last, or the unnamed clipboard if 'clipboard' says so; a
///   `"*`/`"+` read queries the provider, and so may run arbitrary Lua.
/// - `YREG_YANK` -- about to yank into it. With no name, yank into `"0`, and
///   move `""` to it.
/// - `YREG_PUT` -- only report where a paste *would* read from; a `"*`/`"+`
///   the provider does not answer is [`Reg::Empty`].
pub fn get_yank_register(regname: RegName, mode: c_int) -> Reg {
    if (mode == YREG_PASTE || mode == YREG_PUT)
        && let Some((slot, true)) = clipboard::get_clipboard(regname, false)
    {
        return Reg::Slot(slot);
    }
    if mode == YREG_PUT && (regname == b'*' || regname == b'+') {
        // Reporting only: hand back an empty register rather than asking
        // the provider what the clipboard holds.
        return Reg::Empty;
    }
    if mode != YREG_YANK
        && matches!(regname.byte(), 0 | b'"' | b'*' | b'+')
        && let Some(previous) = Y_PREVIOUS.get()
    {
        // An unnamed paste reads the last register written.
        return get_y_register(previous);
    }

    let i = op_reg_index(regname).unwrap_or(0);
    if mode == YREG_YANK {
        Y_PREVIOUS.set(Some(i));
    }
    get_y_register(i)
}

/// Whether the register `regname` holds linewise text, and the register
/// as it is, prepared for pasting -- `None` for an invalid or black-hole
/// name. May run the clipboard provider.
pub fn yank_register_mline(regname: RegName) -> (bool, Option<YankReg>) {
    if !regname.is_unnamed() && !regname.is_valid(false) {
        return (false, None);
    }
    if regname == b'_' {
        return (false, None); // black hole
    }
    let reg = get_yank_register(regname, YREG_PASTE).snapshot();
    (reg.kind == MotionType::LineWise, Some(reg))
}

/// Register `name` as it is now, for `do_put` to work on after something has
/// overwritten it. May run the clipboard provider.
pub fn copy_register(name: RegName) -> YankReg {
    get_yank_register(name, YREG_PASTE).snapshot()
}

/// Shift the numbered delete registers `"1`..`"8` up one, dropping `"9`, so
/// that a new delete can go into `"1`.
///
/// With `y_append` the caller is appending to an uppercase register, so `""`
/// is left where it is.
pub fn shift_delete_registers(y_append: bool) {
    Y_REGS.with_mut(|regs| {
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
        Y_PREVIOUS.set(Some(1));
    }
}

/// Whether text put in `regname` is taken literally -- that is, whether the
/// register is one of the real slots rather than a computed one.
#[inline]
pub fn is_literal_register(regname: RegName) -> bool {
    regname == b'*' || regname == b'+' || regname.byte().is_ascii_alphanumeric()
}

/// The slot `regname` names, or `None` for a name with no slot.
#[inline]
pub fn op_reg_index(regname: RegName) -> Option<usize> {
    let index = |base: u8| usize::from(regname.byte() - base);
    match regname.byte() {
        b'0'..=b'9' => Some(index(b'0')),
        b'a'..=b'z' => Some(index(b'a') + 10),
        // An uppercase name is the same slot, appending rather than replacing.
        b'A'..=b'Z' => Some(index(b'A') + 10),
        b'-' => Some(DELETION_REGISTER),
        b'*' => Some(STAR_REGISTER),
        b'+' => Some(PLUS_REGISTER),
        _ => None,
    }
}

/// Whether `regname` is an uppercase name, which appends to its register
/// rather than replacing it.
#[inline]
pub(crate) fn is_append_register(regname: RegName) -> bool {
    regname.byte().is_ascii_uppercase()
}

/// The name of slot `num`; `None` is the unnamed register.
#[inline]
pub fn get_register_name(num: Option<usize>) -> RegName {
    let Some(num) = num else {
        return RegName::QUOTE;
    };
    // The slots are fewer than 39, so the offsets fit a byte.
    let offset =
        |base: u8, minus: usize| base + u8::try_from(num - minus).expect("a register slot");
    match num {
        0..=9 => RegName::from_byte(offset(b'0', 0)),
        DELETION_REGISTER => RegName::SMALL_DELETE,
        STAR_REGISTER => RegName::STAR,
        PLUS_REGISTER => RegName::PLUS,
        _ => RegName::from_byte(offset(b'a', 10)),
    }
}

/// Whether a register holds nothing -- either no lines at all, or the one
/// empty charwise line an empty yank leaves behind.
#[inline]
fn reg_empty(reg: &YankReg) -> bool {
    match reg.lines.as_slice() {
        [] => true,
        [only] => reg.kind == MotionType::CharWise && only.is_empty(),
        _ => false,
    }
}
