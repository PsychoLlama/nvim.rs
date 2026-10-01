//! An option's variable in one scope: [`OptSlot`] and the three kinds of
//! variable it can name.
//!
//! A global value is a field of the option record, named by the
//! [`Field`] selector the table's row holds. A local one is a field of a
//! buffer, of one of a window's two `WinOpt`s, or of a window's syntax
//! block, and is named the same way: the **holder** — a [`Buf`] or [`Win`]
//! handle — and a `const` selector for the field. Neither half is an
//! address into the holder, so a slot is resolved *per access*: each read
//! or write borrows the field for that one operation and lets go. A slot
//! kept across a `did_set_*` callback — which can run user code, close the
//! window, or `:set` the very option — holds no reference while it waits.
//!
//! [`crate::option::scope`] decides which slot an option reads from;
//! [`crate::option::value`] reads and writes an [`OptVal`] through one.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int};

use crate::global_cell::{Field, field};
use crate::memory::XString;
use crate::options::vars::{BoolOpt, NumOpt, StrOpt};
use crate::optionstr::{empty_option, is_empty_option};
use crate::types::{Buffer, OptIndex, OptInt, OptStr, OptVal, SynBlock, WinOpt};
use crate::winlayer::{Buf, Win};

use super::{option_default, store_option_default, tristate};

/// Which of a window's two copies of its window-local options.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum WinOptSet {
    /// `w_onebuf_opt`: the values in effect for the buffer the window shows
    /// — what `:setlocal` writes.
    One,
    /// `w_allbuf_opt`: the window's own "global" copy, which a buffer
    /// entered later starts from — what `:setglobal` writes.
    All,
}

impl Win {
    /// One of the window's two copies of its window-local options, borrowed
    /// for the caller's one access.
    #[inline(always)]
    pub(crate) fn options_mut(&mut self, set: WinOptSet) -> &mut WinOpt {
        match set {
            WinOptSet::One => &mut self.w_onebuf_opt,
            WinOptSet::All => &mut self.w_allbuf_opt,
        }
    }
}

/// A window's, buffer's or syntax block's own copy of an option: the
/// handle that holds it and the selector naming the field.
///
/// The syntax block is reached through the window (`w_s`) at each access,
/// not remembered: an `:ownsyntax` in between moves it, and the C read
/// `win->w_s` at the moment of the read too.
pub(crate) enum Local<T> {
    /// A field of a buffer.
    Buf(Buf, Field<Buffer, T>),
    /// A field of one of a window's two `WinOpt`s.
    Win(Win, WinOptSet, Field<WinOpt, T>),
    /// A field of the syntax block a window uses — the four 'spell*'
    /// options live there.
    Syn(Win, Field<SynBlock, T>),
}

// Hand-written rather than derived: `derive` would demand `T: Copy`, and
// `Option<XString>` is not -- but a *name* for one is.
impl<T> Clone for Local<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Local<T> {}

/// Two locals are the same variable when they name the same field of the
/// same holder — what the C's address comparison asked.
impl<T> PartialEq for Local<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Local::Buf(a, f), Local::Buf(b, g)) => a == b && f == g,
            (Local::Win(a, s, f), Local::Win(b, t, g)) => a == b && s == t && f == g,
            (Local::Syn(a, f), Local::Syn(b, g)) => a == b && f == g,
            _ => false,
        }
    }
}

impl<T> Eq for Local<T> {}

impl<T> Local<T> {
    /// Run `f` on the field, borrowed for exactly that long.
    ///
    /// `f` is one read or one write. It must not call back into the editor:
    /// the borrow is of the whole holder, which is what [`Buf`]'s and
    /// [`Win`]'s `DerefMut` hand out.
    #[inline]
    fn with<R>(self, f: impl FnOnce(&mut T) -> R) -> R {
        match self {
            Local::Buf(mut buf, field) => f(field.of(&mut buf)),
            Local::Win(mut win, set, field) => f(field.of(win.options_mut(set))),
            Local::Syn(win, field) => f(field.of(&mut win.syntax())),
        }
    }

    /// The same field of the window's other `WinOpt`. Only a window-local
    /// variable has one.
    fn in_set(self, set: WinOptSet) -> Self {
        match self {
            Local::Win(win, _, field) => Local::Win(win, set, field),
            Local::Buf(..) | Local::Syn(..) => unreachable!("only a window has two option copies"),
        }
    }
}

impl<T: Copy> Local<T> {
    /// What the field holds.
    #[inline]
    fn get(self) -> T {
        self.with(|value| *value)
    }

    /// Overwrite the field.
    #[inline]
    fn set(self, value: T) {
        self.with(|field| *field = value);
    }
}

/// One variable of one kind, wherever it lives.
///
/// [`NumVar`], [`StrVar`] and [`BoolVar`] are the three; the third is
/// spelled out separately because its two halves disagree about the type.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum NumVar {
    /// The global value: a field of the option record.
    Global(NumOpt),
    /// A window's or buffer's own copy, or a negative sentinel meaning "not
    /// set here".
    Local(Local<OptInt>),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// current default in place. See [`BoolVar::OwnDefault`].
    OwnDefault(OptIndex),
}

impl NumVar {
    /// What the variable holds.
    pub(crate) fn get(self) -> OptInt {
        match self {
            NumVar::Global(field) => field.get(),
            NumVar::OwnDefault(idx) => option_default(idx)
                .as_number()
                .expect("an immutable number option's default is a number"),
            NumVar::Local(local) => local.get(),
        }
    }

    /// Overwrite the variable.
    pub(crate) fn set(self, value: OptInt) {
        match self {
            NumVar::Global(field) => field.set(value),
            NumVar::OwnDefault(idx) => store_option_default(idx, OptVal::Number(value)),
            NumVar::Local(local) => local.set(value),
        }
    }
}

/// A string option's variable.
///
/// Both halves own their bytes: the global value is a field of the option
/// record, a local copy an `Option<XString>` field of a window, a buffer or
/// a syntax block, and `None` on either side is upstream's shared empty
/// string -- the option holding no value of its own. [`get`](Self::get) and
/// [`replace`](Self::replace) are the seam where that owned storage meets
/// the `char *` the option protocol and [`OptVal`] still speak.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum StrVar {
    /// The global value: a field of the option record.
    Global(StrOpt),
    /// A window's, buffer's or syntax block's own copy.
    Local(Local<Option<XString>>),
    /// An immutable option reads its own default in place. See
    /// [`BoolVar::OwnDefault`].
    OwnDefault(OptIndex),
}

impl StrVar {
    /// The value's bytes, as the pointer the option protocol reads.
    ///
    /// A variable that owns nothing answers the shared empty string, which
    /// is what upstream kept in the variable itself. **The pointer is the
    /// variable's own buffer and lives until the variable is written** --
    /// the promise [`StrOpt::value_ptr`] makes for a global, and the reason
    /// a caller that keeps it across anything that can `:set` takes a copy.
    pub(crate) fn get(self) -> *mut c_char {
        match self {
            // Installing the defaults reads each option's old value before
            // it has one -- upstream reads its `NULL` there -- and that is
            // the protocol's own business, so it answers the empty string
            // rather than tripping `StrOpt`'s read-before-defaults check.
            StrVar::Global(field) if field.is_uninit() => empty_option(),
            StrVar::Global(field) => field.value_ptr(),
            StrVar::OwnDefault(idx) => option_default(idx)
                .as_string()
                .expect("an immutable string option's default is a string")
                .data(),
            StrVar::Local(local) => local.with(|value| {
                value
                    .as_ref()
                    .map_or_else(empty_option, |value| value.as_ptr().cast_mut())
            }),
        }
    }

    /// Whether the value is empty — upstream's `*p == NUL`, which is also
    /// what "not set here" looks like in a global-local option's local copy:
    /// a field that owns nothing and one that owns `""` both answer true.
    pub(crate) fn is_empty(self) -> bool {
        match self {
            StrVar::Global(field) => field.is_uninit() || field.first_byte() == 0,
            StrVar::Local(local) => {
                local.with(|value| value.as_deref().is_none_or(<[u8]>::is_empty))
            }
            StrVar::OwnDefault(idx) => option_default(idx)
                .as_string()
                .expect("an immutable string option's default is a string")
                .is_empty(),
        }
    }

    /// Give the variable a value of its own -- or none -- and answer what it
    /// held. The move out and the move in are one step.
    ///
    /// Only for a variable that owns its bytes; an immutable option's
    /// default is not one, and goes through [`replace`](Self::replace).
    pub(crate) fn swap(self, value: Option<XString>) -> Option<XString> {
        match self {
            StrVar::Global(field) => field.swap(value),
            StrVar::Local(local) => local.with(|field| core::mem::replace(field, value)),
            StrVar::OwnDefault(_) => unreachable!("an immutable option's default is not swapped"),
        }
    }

    /// Overwrite the variable, taking over `value`, and answer the block
    /// that was there — which the caller now owns and must release.
    ///
    /// # Safety
    ///
    /// `value` must be the shared empty string or an allocation with one
    /// owner, which this takes over.
    pub(crate) unsafe fn replace(self, value: *mut c_char) -> *mut c_char {
        if let StrVar::OwnDefault(idx) = self {
            // SAFETY: an option value is NUL-terminated.
            let len = unsafe { crate::cstr::bytes_at(value) }.len();
            let old = option_default(idx)
                .as_string()
                .expect("an immutable string option's default is a string")
                .data();
            store_option_default(idx, OptVal::String(OptStr::from_raw_parts(value, len)));
            return old;
        }
        // SAFETY: the caller's promise -- one owner. The shared empty string
        // is nobody's allocation, so it becomes the variable owning nothing
        // rather than a block to adopt, and comes back out as itself.
        let owned = (!is_empty_option(value)).then(|| unsafe { XString::from_raw(value) });
        self.swap(owned)
            .map_or_else(empty_option, XString::into_raw)
    }
}

/// A boolean option's variable.
///
/// The two halves disagree about the type: the global value is a `bool`,
/// and a local copy is the tri-state word upstream gave every boolean — 0
/// false, 1 true, **-1 not set in this scope**.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum BoolVar {
    /// The global value: a field of the option record.
    Global(BoolOpt),
    /// A window's or buffer's own tri-state copy.
    Local(Local<c_int>),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// current default in place — the `def_val` of its own row, as
    /// `crate::option::state` currently holds it. Upstream reached the same
    /// bytes through a `void *` into the defaults table.
    OwnDefault(OptIndex),
}

impl BoolVar {
    /// What the variable holds, as the tri-state word: 0, 1 or -1.
    pub(crate) fn get(self) -> c_int {
        match self {
            BoolVar::Global(field) => c_int::from(field.get()),
            BoolVar::OwnDefault(idx) => option_default(idx)
                .tristate()
                .expect("an immutable boolean option's default is a boolean"),
            BoolVar::Local(local) => local.get(),
        }
    }

    /// Overwrite the variable with a tri-state word.
    ///
    /// A global value has no third state, and **-1 reaches here**: upstream's
    /// `TRISTATE_FROM_INT` turns any negative number into `kNone`, so
    /// `:let &ignorecase = -3` arrives as -1 even though 'ignorecase' has no
    /// scope to be unset in. Upstream stores that -1 in the option's `int`
    /// and every test of the option then reads it as true; a `bool` cannot
    /// hold it, so the word is coerced the same way round. The one
    /// observable difference is the number `:echo &ignorecase` prints back:
    /// 1 where upstream prints -1. Restoring the exact word means a
    /// tri-state global boolean, which is a change to what apigen emits.
    ///
    /// This used to be a `debug_assert!` that the word was 0 or 1. It was
    /// wrong, and only a debug build could see it: `:let &ignorecase = -3`
    /// aborted the editor where a release build coerced and carried on.
    pub(crate) fn set(self, word: c_int) {
        match self {
            BoolVar::Global(field) => field.set(word != 0),
            BoolVar::OwnDefault(idx) => {
                store_option_default(idx, OptVal::Boolean(tristate(word)));
            }
            BoolVar::Local(local) => local.set(word),
        }
    }
}

/// An option's storage in one scope, with the type its row declares.
///
/// The arm carries the type, so a field whose type disagrees with its
/// option's does not compile, and "which variable is this" is a comparison
/// of holders and selectors rather than of addresses.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum OptSlot {
    /// The option has no variable in this scope.
    None,
    /// A boolean option's variable.
    Boolean(BoolVar),
    /// A number option's variable.
    Number(NumVar),
    /// A string option's variable.
    String(StrVar),
}

impl From<Local<c_int>> for OptSlot {
    fn from(local: Local<c_int>) -> Self {
        OptSlot::Boolean(BoolVar::Local(local))
    }
}

impl From<Local<OptInt>> for OptSlot {
    fn from(local: Local<OptInt>) -> Self {
        OptSlot::Number(NumVar::Local(local))
    }
}

impl From<Local<Option<XString>>> for OptSlot {
    fn from(local: Local<Option<XString>>) -> Self {
        OptSlot::String(StrVar::Local(local))
    }
}

impl OptSlot {
    /// Whether the option has no variable in this scope.
    pub(crate) fn is_none(self) -> bool {
        matches!(self, OptSlot::None)
    }

    /// A boolean option's variable. The callers that reach for one have
    /// already established the option's type — from `option_has_type`, from
    /// the `did_set_*` they are, or from the row itself — and the table's
    /// compile-time assertion is what ties that type to this arm.
    pub(crate) fn boolean_var(self) -> BoolVar {
        match self {
            OptSlot::Boolean(var) => var,
            _ => unreachable!("the option is not a boolean option"),
        }
    }

    /// A number option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn number_var(self) -> NumVar {
        match self {
            OptSlot::Number(var) => var,
            _ => unreachable!("the option is not a number option"),
        }
    }

    /// A string option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn string_var(self) -> StrVar {
        match self {
            OptSlot::String(var) => var,
            _ => unreachable!("the option is not a string option"),
        }
    }

    /// Whether this is the current buffer's 'modified' flag -- `b_changed`,
    /// which is not the whole answer: a buffer whose undo state says it is
    /// back where it was saved is unchanged whatever the flag says.
    pub(crate) fn is_current_modified(self) -> bool {
        self == OptSlot::from(Local::Buf(
            Buf::current(),
            const { field!(Buffer, b_changed) },
        ))
    }

    /// The same field of the window's other `WinOpt`: what `:setglobal`
    /// reaches for a window-local option.
    pub(crate) fn in_set(self, set: WinOptSet) -> Self {
        match self {
            OptSlot::None => OptSlot::None,
            OptSlot::Boolean(BoolVar::Local(local)) => local.in_set(set).into(),
            OptSlot::Number(NumVar::Local(local)) => local.in_set(set).into(),
            OptSlot::String(StrVar::Local(local)) => local.in_set(set).into(),
            _ => unreachable!("a global value has no second copy"),
        }
    }
}
