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
use core::fmt;
use core::num::TryFromIntError;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub type ColNr = ::core::ffi::c_int;

/// `linenr_T`: a line number in a buffer, 1 for the first line.
///
/// A newtype rather than C's `int32_t`, so a line cannot be mistaken for a
/// column, a count or a window number. Moving along lines is `+`/`-` with an
/// `i32` count, and the distance between two lines is `LineNr - LineNr`, an
/// `i32`; nothing adds two line numbers.
///
/// [`ZERO`](Self::ZERO) is not a line. What it stands for is the caller's:
/// "no line" in a mark or a cached position, "before the first line" in
/// `:0`, `appended_lines(0, ..)` and an `:append` target. The newtype keeps
/// the value, not a reading of it.
///
/// `#[repr(transparent)]` over `i32`, so `tools/ffigen` still renders it as
/// `typedef int32_t LineNr` and the structs that carry one keep their layout.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineNr(i32);

impl LineNr {
    /// 0: no line, or the position before the first one.
    pub const ZERO: Self = Self(0);

    /// The first line.
    pub const ONE: Self = Self(1);

    /// `MAXLNUM`: one past any line a buffer can have; what `$` in an open
    /// range and a deleted mark resolve to.
    pub const MAX: Self = Self(i32::MAX);

    /// The line numbered `n`.
    pub const fn new(n: i32) -> Self {
        Self(n)
    }

    /// The number, as C's `int32_t`.
    pub const fn get(self) -> i32 {
        self.0
    }

    /// Whether this is [`ZERO`](Self::ZERO).
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// The line's 0-based index into a list of the buffer's lines.
    ///
    /// Debug builds assert that this is a line (at least 1).
    #[allow(
        clippy::cast_sign_loss,
        reason = "asserted positive; the release build keeps C's conversion"
    )]
    pub fn index(self) -> usize {
        debug_assert!(self.0 >= 1, "line {} has no index", self.0);
        (self.0 - 1) as usize
    }

    /// The line at 0-based `index`.
    ///
    /// # Panics
    /// If the line number does not fit in an `i32`.
    pub fn from_index(index: usize) -> Self {
        i32::try_from(index)
            .ok()
            .and_then(|i| i.checked_add(1))
            .map(Self)
            .unwrap_or_else(|| panic!("line index {index} past MAXLNUM"))
    }

    /// The line after this one.
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// The line before this one.
    pub const fn prev(self) -> Self {
        Self(self.0 - 1)
    }

    /// `count` lines on, or `None` past `i32`.
    pub const fn checked_add(self, count: i32) -> Option<Self> {
        match self.0.checked_add(count) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }

    /// `count` lines back, stopping at `i32::MIN` rather than wrapping.
    pub const fn saturating_sub(self, count: i32) -> Self {
        Self(self.0.saturating_sub(count))
    }

    /// The lines from this one through `last`, in order: `first..=last` for
    /// a type `Range` cannot step.
    pub fn through(self, last: Self) -> impl DoubleEndedIterator<Item = Self> + Clone {
        (self.0..=last.0).map(Self)
    }

    /// The lines from this one up to, not including, `end`.
    pub fn up_to(self, end: Self) -> impl DoubleEndedIterator<Item = Self> + Clone {
        (self.0..end.0).map(Self)
    }

    /// `count` lines on, stopping at [`MAX`](Self::MAX) rather than wrapping.
    pub const fn saturating_add(self, count: i32) -> Self {
        Self(self.0.saturating_add(count))
    }
}

impl Add<i32> for LineNr {
    type Output = Self;

    fn add(self, count: i32) -> Self {
        Self(self.0 + count)
    }
}

impl Sub<i32> for LineNr {
    type Output = Self;

    fn sub(self, count: i32) -> Self {
        Self(self.0 - count)
    }
}

/// How many lines `other` is before `self`.
impl Sub for LineNr {
    type Output = i32;

    fn sub(self, other: Self) -> i32 {
        self.0 - other.0
    }
}

impl AddAssign<i32> for LineNr {
    fn add_assign(&mut self, count: i32) {
        self.0 += count;
    }
}

impl SubAssign<i32> for LineNr {
    fn sub_assign(&mut self, count: i32) {
        self.0 -= count;
    }
}

impl From<LineNr> for i32 {
    fn from(lnum: LineNr) -> Self {
        lnum.0
    }
}

impl From<LineNr> for i64 {
    fn from(lnum: LineNr) -> Self {
        Self::from(lnum.0)
    }
}

impl From<LineNr> for f64 {
    fn from(lnum: LineNr) -> Self {
        Self::from(lnum.0)
    }
}

/// The number as a `usize`, for a length or a capacity, if it is not
/// negative.
impl TryFrom<LineNr> for usize {
    type Error = TryFromIntError;

    fn try_from(lnum: LineNr) -> Result<Self, Self::Error> {
        Self::try_from(lnum.0)
    }
}

/// An API `Integer` or a Vimscript `Number` as a line number, if it is one
/// `i32` can hold.
impl TryFrom<i64> for LineNr {
    type Error = TryFromIntError;

    fn try_from(n: i64) -> Result<Self, Self::Error> {
        i32::try_from(n).map(Self)
    }
}

/// A count of lines as the last line number, if `i32` can hold it.
impl TryFrom<usize> for LineNr {
    type Error = TryFromIntError;

    fn try_from(n: usize) -> Result<Self, Self::Error> {
        i32::try_from(n).map(Self)
    }
}

impl fmt::Display for LineNr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The bare number, as the alias printed.
impl fmt::Debug for LineNr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct LPos {
    pub lnum: LineNr,
    pub col: ColNr,
}
#[derive(Copy, Clone, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Pos {
    pub lnum: LineNr,
    pub col: ColNr,
    pub coladd: ColNr,
}

impl Pos {
    /// The same position moved to `col`, for the read-modify-write of a
    /// position held in a [`crate::global_cell::GlobalCell`].
    pub fn with_col(self, col: ColNr) -> Pos {
        Pos { col, ..self }
    }

    /// The same position moved to `lnum`.
    pub fn with_lnum(self, lnum: LineNr) -> Pos {
        Pos { lnum, ..self }
    }
}
