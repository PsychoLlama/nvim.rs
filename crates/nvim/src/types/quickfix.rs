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
use crate::file_search::Name;
use crate::memory::XString;

/// A stack's place in the quickfix stack table: what a window holds for its
/// location list, and what the quickfix code re-finds a stack by after
/// anything that can run user code.
pub(crate) type QfId = crate::id_table::TableId<QfStack>;

/// A stack of quickfix (or location) lists.
///
/// There is one quickfix stack for the editor and one location list stack
/// per window that has asked for one; the quickfix code owns all of them in
/// one table and everything else names one by [`QfId`]. A location list
/// stack is shared by reference between the window that owns it and the
/// location list window showing it, and freed at the last reference.
pub struct QfStack {
    /// How many windows hold this stack. Meaningless for the quickfix
    /// stack, which is never freed.
    pub refcount: Refcount,
    /// How many of [`lists`](Self::lists) hold a list. The rest are unused.
    pub list_count: ::core::ffi::c_int,
    /// Which list `:cc` and friends work on.
    pub current: ::core::ffi::c_int,
    /// Room for `'chistory'` (or `'lhistory'`) lists, oldest first.
    pub lists: Vec<QfList>,
    pub kind: QfListType,
    /// The buffer the quickfix window shows this stack in, or
    /// `INVALID_QFBUFNR`.
    pub bufnr: ::core::ffi::c_int,
}

impl QfStack {
    /// An empty stack of `kind` with room for no list.
    pub const fn new(kind: QfListType) -> Self {
        QfStack {
            refcount: Refcount::ZERO,
            list_count: 0,
            current: 0,
            lists: Vec::new(),
            kind,
            bufnr: 0,
        }
    }

    /// How many lists the stack has room for — `'chistory'` for the
    /// quickfix stack, `'lhistory'` for a location list stack.
    pub fn max_count(&self) -> ::core::ffi::c_int {
        // 'chistory'/'lhistory' cap the stack three orders of magnitude below
        // `c_int::MAX`, so the saturation is unreachable.
        ::core::ffi::c_int::try_from(self.lists.len()).unwrap_or(::core::ffi::c_int::MAX)
    }

    /// Whether the stack holds no list at all.
    pub fn is_empty(&self) -> bool {
        self.list_count <= 0
    }

    /// Whether this is the quickfix stack, rather than a location list one.
    pub fn is_quickfix(&self) -> bool {
        self.kind == QFLT_QUICKFIX
    }

    /// The list in slot `idx`, which must be one the stack has room for.
    pub fn list(&self, idx: ::core::ffi::c_int) -> &QfList {
        &self.lists[slot(idx)]
    }

    /// [`list`](Self::list), writable.
    pub fn list_mut(&mut self, idx: ::core::ffi::c_int) -> &mut QfList {
        &mut self.lists[slot(idx)]
    }

    /// The list `:cc` and friends work on.
    pub fn current_list(&self) -> &QfList {
        self.list(self.current)
    }

    /// [`current_list`](Self::current_list), writable.
    pub fn current_list_mut(&mut self) -> &mut QfList {
        self.list_mut(self.current)
    }

    /// The slot of the list whose id is `id`, if it is still on the stack.
    pub fn find_list(&self, id: ::core::ffi::c_uint) -> Option<::core::ffi::c_int> {
        let count = slot(self.list_count);
        let at = self.lists[..count].iter().position(|list| list.id == id)?;
        ::core::ffi::c_int::try_from(at).ok()
    }
}

/// A list or entry number as an index. The stack never holds more than
/// `'chistory'` lists and a list's numbers come from a `c_int` count, so a
/// negative one is a caller's bug.
fn slot(idx: ::core::ffi::c_int) -> usize {
    usize::try_from(idx).expect("a list slot is never negative")
}

pub type QfListType = ::core::ffi::c_uint;
pub const QFLT_INTERNAL: QfListType = 2;
pub const QFLT_LOCATION: QfListType = 1;
pub const QFLT_QUICKFIX: QfListType = 0;

/// One quickfix list within a stack.
pub struct QfList {
    /// The list's identity, which `getqflist()` reports and a command that
    /// ran user code compares to tell whether it is still on the same list.
    /// Never reused; 0 for an unused slot.
    pub id: ::core::ffi::c_uint,
    pub kind: QfListType,
    /// The entries, in order. Users number them from 1.
    pub entries: Vec<QfEntry>,
    /// The current entry, as an index into [`entries`](Self::entries); 0
    /// when there are none.
    pub cursor: usize,
    /// The current entry's number, counted from 1 — or 0 before the first
    /// valid entry of a list being built has been seen.
    pub index: ::core::ffi::c_int,
    /// No entry names a real position, so every one counts as valid.
    pub no_valid: bool,
    /// Some entry carries a `user_data` value the collector has to see.
    pub has_user_data: bool,
    pub title: Option<XString>,
    /// What `setqflist()` attached under `context`.
    pub context: Option<Box<TypVal>>,
    /// This list's own `'quickfixtextfunc'`.
    pub text_func: Callback,
    /// The directories `%D`/`%X` pushed while parsing; the top is what a
    /// relative file name is resolved against.
    pub dir_stack: DirStack,
    /// The files `%P`/`%Q` pushed while parsing.
    pub file_stack: DirStack,
    /// A `%A`…`%N` message is open, for `%C` and `%Z` lines to continue.
    pub multiline: bool,
    /// The open multi-line message is being dropped (`%-`).
    pub multiignore: bool,
    /// The tail of the last line is to be re-scanned (`%O`/`%P`/`%Q`).
    pub multiscan: bool,
    /// Bumped on every change, for `getqflist({'changedtick': 1})` and for
    /// a command that ran user code to tell whether the list moved.
    pub changedtick: ::core::ffi::c_int,
}

impl Default for QfList {
    fn default() -> Self {
        QfList::new()
    }
}

impl QfList {
    /// An unused slot.
    pub const fn new() -> Self {
        QfList {
            id: 0,
            kind: QFLT_QUICKFIX,
            entries: Vec::new(),
            cursor: 0,
            index: 0,
            no_valid: false,
            has_user_data: false,
            title: None,
            context: None,
            text_func: Callback::None,
            dir_stack: DirStack { dirs: Vec::new() },
            file_stack: DirStack { dirs: Vec::new() },
            multiline: false,
            multiignore: false,
            multiscan: false,
            changedtick: 0,
        }
    }

    /// How many entries the list holds.
    pub fn count(&self) -> ::core::ffi::c_int {
        // A list is built one entry at a time from a `c_int` count, so this
        // saturation is unreachable.
        ::core::ffi::c_int::try_from(self.entries.len()).unwrap_or(::core::ffi::c_int::MAX)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether at least one entry names a real position.
    pub fn has_valid_entries(&self) -> bool {
        !self.is_empty() && !self.no_valid
    }

    /// The current entry, if the list has any.
    pub fn current(&self) -> Option<&QfEntry> {
        self.entries.get(self.cursor)
    }

    /// Entry number `nr`, counted from 1.
    pub fn nth(&self, nr: ::core::ffi::c_int) -> Option<&QfEntry> {
        let at = usize::try_from(nr).ok()?.checked_sub(1)?;
        self.entries.get(at)
    }

    /// The directory a relative file name is resolved against: the last one
    /// `%D` pushed, if any.
    pub(crate) fn directory(&self) -> Option<&Name> {
        self.dir_stack.dirs.last()
    }

    /// The file a `%P` claimed the following lines for, if any.
    pub(crate) fn current_file(&self) -> Option<&Name> {
        self.file_stack.dirs.last()
    }

    /// Note that the list changed.
    pub fn changed(&mut self) {
        self.changedtick += 1;
    }
}

impl Drop for QfList {
    fn drop(&mut self) {
        // Everything else the list holds releases itself; the callback is
        // the one value that has to be asked to.
        self.text_func.clear();
    }
}

/// The directories `%D`/`%X` (or `%O`/`%P`/`%Q`) pushed while parsing, the
/// one most recently entered last. See `quickfix::entry` for the operations.
#[derive(Default)]
pub struct DirStack {
    pub(crate) dirs: Vec<Name>,
}

/// One entry in a quickfix list.
pub struct QfEntry {
    /// The line the entry names, or 0 for none.
    pub lnum: LineNr,
    pub end_lnum: LineNr,
    /// The buffer the entry names, or 0 for none.
    pub fnum: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
    /// The error number, or -1/0 for none.
    pub nr: ::core::ffi::c_int,
    /// Shown instead of the file name.
    pub module: Option<XString>,
    /// The file name as the entry was given it, when it differs from the
    /// buffer's own.
    pub fname: Option<XString>,
    /// A search pattern that finds the position, instead of `lnum`.
    pub pattern: Option<XString>,
    pub text: XString,
    /// Non-zero when `col` is a screen column. Wider than a bool because
    /// `setqflist()` keeps whatever number it was given.
    pub viscol: ::core::ffi::c_char,
    /// The line it named was deleted.
    pub cleared: bool,
    /// `e`, `w`, `i`, `n`, 1 for a help entry, or 0.
    pub kind: ::core::ffi::c_char,
    pub user_data: TypVal,
    /// The entry names a real position and can be jumped to.
    pub valid: bool,
}
