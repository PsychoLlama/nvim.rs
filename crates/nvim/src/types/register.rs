#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::memory::XString;
use crate::shada::ShadaExtra;
use std::rc::Rc;

pub type GRegFlags = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct BlockDef {
    pub startspaces: ::core::ffi::c_int,
    pub endspaces: ::core::ffi::c_int,
    pub textlen: ::core::ffi::c_int,
    pub textcol: ColNr,
    pub start_vcol: ColNr,
    pub end_vcol: ColNr,
    pub is_short: ::core::ffi::c_int,
    pub is_max: ::core::ffi::c_int,
    pub is_one_char: ::core::ffi::c_int,
    pub pre_whitesp: ::core::ffi::c_int,
    pub pre_whitesp_c: ::core::ffi::c_int,
    pub end_char_vcols: ColNr,
    pub start_char_vcols: ColNr,
}

impl BlockDef {
    /// All zeros — the state `block_prep` and `charwise_block_prep` overwrite.
    ///
    /// C declares these uninitialised and fills them in; every caller in the
    /// tree does exactly that, so the zeros are never read.
    pub const ZERO: Self = BlockDef {
        startspaces: 0,
        endspaces: 0,
        textlen: 0,
        textcol: 0,
        start_vcol: 0,
        end_vcol: 0,
        is_short: 0,
        is_max: 0,
        is_one_char: 0,
        pre_whitesp: 0,
        pre_whitesp_c: 0,
        end_char_vcols: 0,
        start_char_vcols: 0,
    };
}

impl Default for BlockDef {
    fn default() -> Self {
        Self::ZERO
    }
}
/// The lines a register holds.
///
/// Upstream's `y_array`/`y_size` pair, with its null kept as a state of its
/// own: a register that was never set (or was freed) holds no array at all,
/// which is not the same as one holding an array of no lines -- `@r` fails on
/// the first and replays nothing on the second, and the clipboard provider
/// can answer the second.
///
/// The vector is shared. A put, a replay or a `:registers` listing reads a
/// register while the editor runs code that may write it -- an `on_bytes`
/// callback, the clipboard provider, an autocommand -- and upstream's answer
/// is to hold a pointer into the array and hope. Here the reader takes a
/// counted share ([`Clone`] is a reference count, not a copy of the text)
/// and a writer that finds the vector shared copies it first
/// ([`lines_mut`](Self::lines_mut)), so the reader's lines stay what they
/// were and the register gets the new ones.
///
/// No line holds a NUL: a NUL in the text is stored as the newline that
/// stands for it, as in a buffer line, and every way into a register keeps
/// that (`str_to_reg` translates, the clipboard and the API hand over C
/// strings or translate, ShaDa's reader translates). So a line is the C
/// string a reader of upstream's `String` saw, and the puts hand its bytes
/// on without a scan.
#[derive(Clone, Default, Debug)]
pub struct RegLines(Option<Rc<Vec<XString>>>);

impl RegLines {
    /// No array: the register is unset.
    pub const NONE: Self = RegLines(None);

    /// An array holding `lines` -- set, even when `lines` is empty.
    pub fn new(lines: Vec<XString>) -> Self {
        RegLines(Some(Rc::new(lines)))
    }

    /// Whether there is an array, empty or not: upstream's `y_array != NULL`.
    pub fn is_set(&self) -> bool {
        self.0.is_some()
    }

    /// The lines; none for an unset register.
    pub fn as_slice(&self) -> &[XString] {
        self.0.as_deref().map_or(&[], Vec::as_slice)
    }

    /// The lines, to change: the array is created when there is none, and
    /// copied first when a reader still holds a share of it.
    pub fn lines_mut(&mut self) -> &mut Vec<XString> {
        Rc::make_mut(self.0.get_or_insert_with(Rc::default))
    }
}

impl core::ops::Deref for RegLines {
    type Target = [XString];

    fn deref(&self) -> &[XString] {
        self.as_slice()
    }
}

/// One register: its lines, how a put lays them out, when it was set, and
/// the ShaDa data this editor does not read but writes back.
#[derive(Clone, Debug)]
pub struct YankReg {
    pub lines: RegLines,
    /// `kMTCharWise`, `kMTLineWise`, `kMTBlockWise`, or `kMTUnknown` while a
    /// write is still deciding.
    pub kind: MotionType,
    /// A blockwise register's width in cells, less one.
    pub width: ColNr,
    pub timestamp: Timestamp,
    pub(crate) extra: Option<ShadaExtra>,
}

impl YankReg {
    /// The empty register every slot starts as: upstream's
    /// `static yankreg_T y_regs[NUM_REGISTERS] = { 0 }`.
    pub const EMPTY: YankReg = YankReg {
        lines: RegLines::NONE,
        kind: 0, // kMTCharWise
        width: 0,
        timestamp: 0,
        extra: None,
    };

    /// Drop the lines and the extra data, leaving the register unset --
    /// upstream's `free_register`. The type, width and timestamp stay, as
    /// they do there.
    pub fn clear(&mut self) {
        self.lines = RegLines::NONE;
        self.extra = None;
    }
}

impl Default for YankReg {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// `do_put()` flags — upstream's anonymous enum in `register_defs.h`.
///
/// c2rust typed this `c_uint`, so every use site is `PUT_X as c_int`;
/// retyping belongs to the slice that deletes those casts.
pub type PutFlags = ::core::ffi::c_uint;

/// make the indent look nice
pub const PUT_FIXINDENT: PutFlags = 1;
/// leave the cursor after the end of the new text
pub const PUT_CURSEND: PutFlags = 2;
/// leave the cursor on the last line of the new text
pub const PUT_CURSLINE: PutFlags = 4;
/// put the register as lines
pub const PUT_LINE: PutFlags = 8;
/// split the line for a linewise register
pub const PUT_LINE_SPLIT: PutFlags = 16;
/// put a linewise register below the Visual selection
pub const PUT_LINE_FORWARD: PutFlags = 32;
/// in block mode, do not add trailing spaces
pub const PUT_BLOCK_INNER: PutFlags = 64;

#[cfg(test)]
mod tests {
    use super::{RegLines, YankReg};
    use crate::memory::XString;

    fn lines(text: &[&str]) -> Vec<XString> {
        text.iter().map(|&line| XString::from(line)).collect()
    }

    #[test]
    fn an_unset_register_is_not_an_empty_one() {
        assert!(!RegLines::NONE.is_set());
        assert!(RegLines::NONE.is_empty());
        let none = RegLines::new(Vec::new());
        assert!(none.is_set());
        assert!(none.is_empty());
        // Writing to an unset register sets it.
        let mut unset = RegLines::NONE;
        unset.lines_mut().push(XString::from("x"));
        assert!(unset.is_set());
        assert_eq!(unset.as_slice(), &lines(&["x"])[..]);
    }

    #[test]
    fn a_share_keeps_its_lines_when_the_register_is_written() {
        let mut reg = YankReg::EMPTY;
        reg.lines = RegLines::new(lines(&["one", "two"]));
        let held = reg.clone();
        // The writer copies the shared vector; the share keeps the old one.
        reg.lines.lines_mut()[0].push_bytes(b"!");
        reg.lines.lines_mut().push(XString::from("three"));
        assert_eq!(held.lines.as_slice(), &lines(&["one", "two"])[..]);
        assert_eq!(reg.lines.as_slice(), &lines(&["one!", "two", "three"])[..]);
        // Clearing drops only the register's own share.
        reg.clear();
        assert!(!reg.lines.is_set());
        assert_eq!(held.lines.len(), 2);
    }

    #[test]
    fn an_unshared_write_is_in_place() {
        let mut reg = RegLines::new(lines(&["a"]));
        let before = reg.as_slice().as_ptr();
        reg.lines_mut()[0].push_bytes(b"b");
        assert_eq!(reg.as_slice().as_ptr(), before);
        assert_eq!(reg.as_slice(), &lines(&["ab"])[..]);
    }
}
