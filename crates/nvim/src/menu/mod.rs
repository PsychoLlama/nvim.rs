//! Menus: `:menu` and the tree it builds.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`define`] | `:menu` and `add_menu_path()` |
//! | [`tree`] | the `vimmenu_T` tree -- find, list, dump, remove, free |
//! | [`complete`] | command-line completion of a menu path |
//! | [`name`] | names as text: path components, mode letters, accelerators |
//! | [`exec`] | `:emenu`, `:popup` and running a right-hand side |
//! | [`info`] | `:menutranslate` and `menu_info()` |
//!
//! What stays here is what the six share: the mode alphabet (`MENU_*_MODE`,
//! `MENU_INDEX_*`, [`MODE_CHARS`]), the `menus_locked` guard, the
//! `menu_is_*` predicates that classify a node, [`get_menu_mode`] -- "which
//! mode is the editor in?" for the executing side -- and the two newtypes
//! everything else is written in terms of.
//!
//! [`Menu`] wraps a `*mut vimmenu_T` and [`Link`] wraps the `next`,
//! `children` or root slot that points at one. Each has a single unsafe
//! constructor carrying the invariant; every other method, and so almost
//! all of the six children, is safe code. [`CText`] does the same for the
//! NUL-terminated buffers a menu path is taken apart in.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_long};
use core::ops::{Deref, DerefMut};
use core::ptr;
use std::ffi::CString;

use crate::autocmd::{EVENT_MENUPOPUP, apply_autocmds};
use crate::charset::skipwhite;
use crate::eval::typval::{
    tv_dict_add_allocated_str, tv_dict_add_bool, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_len, tv_list_alloc,
    tv_list_append_dict, tv_list_append_string,
};
use crate::global_cell::GlobalCell;
use crate::main::{
    State, VIsual_active, VIsual_select, curbuf, e_cannot_change_menus_while_listing, finish_op,
    root_menu,
};
use crate::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::message::{emsg, str2special_save};
use crate::os::cshim::gettext;
use crate::popupmenu::pum_show_popupmenu;
use crate::semsg_c;
use crate::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_TERMINAL,
};
use crate::types::{dict_T, kListLenMayKnow, list_T, ptrdiff_t, varnumber_T, vimmenu_T};

// The carve of the transpiled module; see each child's docs.
mod complete;
mod define;
mod exec;
mod info;
mod name;
mod tree;

pub use self::complete::*;
pub use self::define::*;
pub use self::exec::*;
pub use self::info::*;
pub use self::name::*;
pub use self::tree::*;

/// How deep a menu path may go, and so how many priority components
/// `:menu 10.20.30 …` can carry.
pub(crate) const MENUDEPTH: usize = 10;

/// A name starting with this is never shown in the menubar or the listing.
const MNU_HIDDEN_CHAR: u8 = b']';

pub(crate) const MENU_MODES: usize = 8;

pub(crate) const MENU_INDEX_INVALID: c_int = -1;
pub(crate) const MENU_INDEX_NORMAL: c_int = 0;
pub(crate) const MENU_INDEX_VISUAL: c_int = 1;
pub(crate) const MENU_INDEX_SELECT: c_int = 2;
pub(crate) const MENU_INDEX_OP_PENDING: c_int = 3;
pub(crate) const MENU_INDEX_INSERT: c_int = 4;
pub(crate) const MENU_INDEX_CMDLINE: c_int = 5;
pub(crate) const MENU_INDEX_TERMINAL: c_int = 6;
pub(crate) const MENU_INDEX_TIP: c_int = 7;

pub(crate) const MENU_NORMAL_MODE: c_int = 1 << MENU_INDEX_NORMAL;
pub(crate) const MENU_VISUAL_MODE: c_int = 1 << MENU_INDEX_VISUAL;
pub(crate) const MENU_SELECT_MODE: c_int = 1 << MENU_INDEX_SELECT;
pub(crate) const MENU_OP_PENDING_MODE: c_int = 1 << MENU_INDEX_OP_PENDING;
pub(crate) const MENU_INSERT_MODE: c_int = 1 << MENU_INDEX_INSERT;
pub(crate) const MENU_CMDLINE_MODE: c_int = 1 << MENU_INDEX_CMDLINE;
pub(crate) const MENU_TERMINAL_MODE: c_int = 1 << MENU_INDEX_TERMINAL;
pub(crate) const MENU_TIP_MODE: c_int = 1 << MENU_INDEX_TIP;
/// Every mode but the tooltip pseudo-mode.
pub(crate) const MENU_ALL_MODES: c_int = MENU_TIP_MODE - 1;

/// `:amenu` -- the six modes a menu entry can be typed in.
pub(crate) const MENU_AMENU_MODES: c_int = MENU_INSERT_MODE
    | MENU_CMDLINE_MODE
    | MENU_NORMAL_MODE
    | MENU_VISUAL_MODE
    | MENU_SELECT_MODE
    | MENU_OP_PENDING_MODE;

/// `:menu` without a bang.
pub(crate) const MENU_PLAIN_MODES: c_int =
    MENU_NORMAL_MODE | MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE;

/// The letter each mode is spelled with, indexed by `MENU_INDEX_*`. It is
/// both the `:menu` listing's column and the key `menu_get()` files a
/// mapping under, so the order is observable.
pub(crate) const MODE_CHARS: [&CStr; MENU_MODES] =
    [c"n", c"v", c"s", c"o", c"i", c"c", c"tl", c"t"];

/// `noremap` values stored per mode in a `vimmenu_T`.
pub(crate) const REMAP_SCRIPT: c_int = -2;
pub(crate) const REMAP_NONE: c_int = -1;
pub(crate) const REMAP_YES: c_int = 0;

/// `replace_termcodes()`: also translate `<lt>`.
pub(crate) const REPTERM_DO_LT: c_int = 2;

pub(crate) static E_NOTSUBMENU: &CStr = c"E327: Part of menu-item path is not sub-menu";
pub(crate) static E_NOMENU: &CStr = c"E329: No menu \"%s\"";

/// The translated text of one of the shared `e_*` message constants. The
/// `_c` message macros want a `printf` format string, which a
/// `format_args!` literal cannot be.
pub(crate) fn message<const N: usize>(msg: &'static [c_char; N]) -> *const c_char {
    // SAFETY: gettext answers either its argument or a pointer into the
    // loaded message catalog; both are `'static`.
    unsafe { gettext(msg.as_ptr()) }
}

/// [`message`] for a message this module owns.
pub(crate) fn message_str(msg: &'static CStr) -> *const c_char {
    // SAFETY: as [`message`].
    unsafe { gettext(msg.as_ptr()) }
}

/// `emsg(_(msg))`.
pub(crate) fn emsg_c(msg: &'static CStr) {
    // SAFETY: a `'static` NUL-terminated string; emsg copies what it keeps.
    unsafe { emsg(message_str(msg)) };
}

/// `emsg(_(msg))` for one of the shared `e_*` constants.
pub(crate) fn emsg_shared<const N: usize>(msg: &'static [c_char; N]) {
    // SAFETY: as [`emsg_c`].
    unsafe { emsg(message(msg)) };
}

/// `semsg(fmt, arg)` for the five messages that interpolate a menu name.
///
/// A menu name is arbitrary bytes -- the sweep defines one out of invalid
/// UTF-8 -- so these stay on vim's own `printf`, which copies them through,
/// rather than moving to a `format_args!` literal that would have to lose
/// the bytes it cannot decode.
pub(crate) fn semsg_name(fmt: *const c_char, name: *const c_char) {
    // SAFETY: `fmt` is a `'static` format with a single `%s`, and `name` is
    // a NUL-terminated string that outlives the call.
    unsafe { semsg_c!(fmt, name) };
}

/// While non-zero no menu may be added or removed, so that the listing
/// cannot walk a tree that is changing under it.
static MENUS_LOCKED: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether the menus are locked, reporting it if they are.
pub(crate) fn is_menus_locked() -> bool {
    if MENUS_LOCKED.get() > 0 {
        emsg_shared(&e_cannot_change_menus_while_listing);
        return true;
    }
    false
}

/// Holds the menu tree still for the duration of `f`.
pub(crate) fn with_menus_locked<R>(f: impl FnOnce() -> R) -> R {
    MENUS_LOCKED.set(MENUS_LOCKED.get() + 1);
    let result = f();
    MENUS_LOCKED.set(MENUS_LOCKED.get() - 1);
    result
}

// ---------------------------------------------------------------------------
// The tree as a pair of newtypes
// ---------------------------------------------------------------------------

/// One node of the menu tree.
///
/// # Invariant
/// The pointer names a live `vimmenu_T` -- one `add_menu_path` allocated and
/// `free_menu` has not yet released -- whose `name` and `dname` are
/// NUL-terminated strings, whose `en_name`/`en_dname`/`actext` are null or
/// NUL-terminated strings, and whose `next`, `children` and `parent` are
/// null or name another such node.
#[derive(Copy, Clone)]
pub(crate) struct Menu(*mut vimmenu_T);

impl Menu {
    /// # Safety
    /// `ptr` must satisfy the invariant above.
    pub(crate) const unsafe fn new(ptr: *mut vimmenu_T) -> Self {
        Menu(ptr)
    }

    /// The node `ptr` names, or `None` for C's `NULL`.
    ///
    /// # Safety
    /// A non-null `ptr` must satisfy the invariant above.
    pub(crate) unsafe fn opt(ptr: *const vimmenu_T) -> Option<Self> {
        (!ptr.is_null()).then(|| {
            // SAFETY: the caller's obligation, minus the null case.
            unsafe { Menu::new(ptr.cast_mut()) }
        })
    }

    pub(crate) fn raw(self) -> *mut vimmenu_T {
        self.0
    }

    /// Whether both name the same node.
    pub(crate) fn same(self, other: Menu) -> bool {
        ptr::eq(self.0, other.0)
    }

    pub(crate) fn next(self) -> Option<Menu> {
        // SAFETY: the invariant covers the whole sibling list.
        unsafe { Menu::opt(self.next) }
    }

    pub(crate) fn children(self) -> Option<Menu> {
        // SAFETY: the invariant covers the whole subtree.
        unsafe { Menu::opt(self.children) }
    }

    pub(crate) fn parent(self) -> Option<Menu> {
        // SAFETY: the invariant covers the parent chain.
        unsafe { Menu::opt(self.parent) }
    }

    /// This node and every sibling after it.
    pub(crate) fn siblings(self) -> impl Iterator<Item = Menu> {
        core::iter::successors(Some(self), |m| m.next())
    }

    /// The slot holding the pointer to the next sibling.
    pub(crate) fn next_link(self) -> Link {
        // SAFETY: the invariant; the field is live for as long as the node.
        unsafe { Link::new(&raw mut (*self.0).next) }
    }

    /// The slot holding the pointer to the first child.
    pub(crate) fn children_link(self) -> Link {
        // SAFETY: as [`Menu::next_link`].
        unsafe { Link::new(&raw mut (*self.0).children) }
    }

    /// The name as `:menu` spelled it, mnemonic and accelerator included.
    pub(crate) fn name(&self) -> &CStr {
        // SAFETY: the invariant; `name` is never null on a live node.
        unsafe { CStr::from_ptr(self.name) }
    }

    /// The displayed name: no `&` mnemonic marker, no accelerator.
    pub(crate) fn dname(&self) -> &CStr {
        // SAFETY: the invariant; `dname` is never null on a live node.
        unsafe { CStr::from_ptr(self.dname) }
    }

    /// The pre-translation name, for a node `:menutranslate` renamed.
    pub(crate) fn en_name(&self) -> Option<&CStr> {
        // SAFETY: the invariant; null unless a translation applied.
        unsafe { cstr_opt(self.en_name) }
    }

    /// The pre-translation displayed name.
    pub(crate) fn en_dname(&self) -> Option<&CStr> {
        // SAFETY: as [`Menu::en_name`]; the two are set together.
        unsafe { cstr_opt(self.en_dname) }
    }

    /// The accelerator text, i.e. whatever followed the name's `<Tab>`.
    pub(crate) fn actext(&self) -> Option<&CStr> {
        // SAFETY: the invariant; null unless the name carried a TAB.
        unsafe { cstr_opt(self.actext) }
    }

    /// The right-hand side stored for one mode.
    pub(crate) fn rhs(&self, idx: usize) -> Option<&CStr> {
        // SAFETY: the invariant; unset modes hold null.
        unsafe { cstr_opt(self.strings[idx]) }
    }

    /// Whether the node is available in any of `modes`.
    pub(crate) fn in_modes(&self, modes: c_int) -> bool {
        self.modes & modes != 0
    }
}

impl Deref for Menu {
    type Target = vimmenu_T;

    fn deref(&self) -> &vimmenu_T {
        // SAFETY: the invariant; the reference never spans a free.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Menu {
    fn deref_mut(&mut self) -> &mut vimmenu_T {
        // SAFETY: as [`Menu::deref`]; each write finishes in its statement.
        unsafe { &mut *self.0 }
    }
}

/// The slot a node is reached through: C's `vimmenu_T **`.
///
/// Unlinking a node means writing its successor into the slot that named it,
/// which is either the root list head or another node's `next`/`children`.
///
/// # Invariant
/// The pointer names a live slot -- the root cell, or a field of a node
/// satisfying [`Menu`]'s invariant.
#[derive(Copy, Clone)]
pub(crate) struct Link(*mut *mut vimmenu_T);

impl Link {
    /// # Safety
    /// `slot` must satisfy the invariant above.
    pub(crate) const unsafe fn new(slot: *mut *mut vimmenu_T) -> Self {
        Link(slot)
    }

    /// The node in the slot, if any.
    pub(crate) fn get(self) -> Option<Menu> {
        // SAFETY: the invariant; the slot holds null or a live node.
        unsafe { Menu::opt(*self.0) }
    }

    /// Put `menu` in the slot.
    pub(crate) fn set(self, menu: Option<Menu>) {
        let value = menu.map_or(ptr::null_mut(), Menu::raw);
        // SAFETY: the invariant; the write finishes in this statement.
        unsafe { *self.0 = value }
    }
}

/// The list head every menu path is resolved from.
///
/// C's `get_root_menu()` takes the path and ignores it: there was one root
/// per window toolbar once, and there has been a single tree since. The
/// escape hatch stays because the layer *writes* through the slot when the
/// first top-level menu is created or the last one is removed.
pub(crate) fn root_link() -> Link {
    // SAFETY: a static list head, live for the whole process.
    unsafe { Link::new(root_menu.ptr()) }
}

/// The first top-level menu, if there is one.
pub(crate) fn root_first() -> Option<Menu> {
    root_link().get()
}

// ---------------------------------------------------------------------------
// Menu names as C text
// ---------------------------------------------------------------------------

/// A position in a NUL-terminated buffer the menu code is taking apart.
///
/// A menu path arrives as one string and is consumed component by component:
/// `\`-escapes are squeezed out in place, and the `.` ending a component is
/// overwritten with a NUL so the component can be handed on as its own C
/// string. Callers keep several of these into one buffer at once, which is
/// why this is a raw pointer rather than a slice.
///
/// # Invariant
/// The pointer names a byte of a live, writable, NUL-terminated buffer, at
/// or before that buffer's terminator.
#[derive(Copy, Clone)]
pub(crate) struct CText(*mut c_char);

impl CText {
    /// # Safety
    /// `ptr` must satisfy the invariant above.
    pub(crate) const unsafe fn new(ptr: *mut c_char) -> Self {
        CText(ptr)
    }

    pub(crate) fn raw(self) -> *mut c_char {
        self.0
    }

    /// The `i`th byte from here, which may be the terminator itself.
    pub(crate) fn byte(self, i: usize) -> u8 {
        // SAFETY: the invariant; callers index at or before the terminator,
        // exactly as the C read `p[1]` only after testing `*p`.
        unsafe { *self.0.add(i) as u8 }
    }

    /// Everything from here to the terminator.
    pub(crate) fn bytes(&self) -> &[u8] {
        // SAFETY: the invariant; the terminator bounds the read.
        unsafe { CStr::from_ptr(self.0).to_bytes() }
    }

    /// Everything from here to the terminator, terminator included.
    pub(crate) fn as_cstr(&self) -> &CStr {
        // SAFETY: the invariant.
        unsafe { CStr::from_ptr(self.0) }
    }

    pub(crate) fn is_empty(self) -> bool {
        self.byte(0) == 0
    }

    /// The position `i` bytes further on.
    pub(crate) fn at(self, i: usize) -> CText {
        // SAFETY: the invariant; callers only step within the buffer.
        unsafe { CText::new(self.0.add(i)) }
    }

    /// How far apart two positions in one buffer are.
    pub(crate) fn offset_from(self, start: CText) -> usize {
        // SAFETY: both name bytes of the same buffer, `self` the later.
        unsafe { self.0.offset_from(start.0) as usize }
    }

    /// Overwrite the `i`th byte.
    pub(crate) fn set(self, i: usize, byte: u8) {
        // SAFETY: the invariant; `i` is at or before the terminator, and the
        // buffer is one the caller owns (a `xstrdup` of the command line, or
        // the command line itself, which `ex_docmd` lets commands edit).
        unsafe { *self.0.add(i) = byte as c_char }
    }

    /// C's `STRMOVE(p + i, p + i + count)`: drop `count` bytes at `i`,
    /// shifting the rest of the string -- terminator included -- down over
    /// them.
    pub(crate) fn squeeze(self, i: usize, count: usize) {
        let rest = self.at(i + count).bytes().len() + 1;
        // SAFETY: the invariant; source and destination are `rest` bytes of
        // one buffer, and `copy` allows them to overlap as `memmove` did.
        unsafe { ptr::copy(self.0.add(i + count), self.0.add(i), rest) }
    }

    /// The length of the character at `i`, composing characters included --
    /// C's `MB_PTR_ADV` step. Zero at the terminator, which is what stops
    /// the walks that squeeze the string as they go.
    pub(crate) fn char_len(self, i: usize) -> usize {
        // SAFETY: the invariant; `utfc_ptr2len` stops at the terminator.
        unsafe { utfc_ptr2len(self.0.add(i)) as usize }
    }

    /// Whether both name the same position.
    pub(crate) fn same(self, other: CText) -> bool {
        ptr::eq(self.0, other.0)
    }

    /// Whether the text starts with `prefix` -- C's `strncmp(p, …, n) == 0`.
    pub(crate) fn starts_with(&self, prefix: &[u8]) -> bool {
        self.bytes().starts_with(prefix)
    }
}

/// `ptr` as a string, or `None` for C's `NULL`.
///
/// # Safety
/// A non-null `ptr` must name a NUL-terminated string outliving `'a`.
unsafe fn cstr_opt<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    // SAFETY: the caller's obligation, minus the null case.
    (!ptr.is_null()).then(|| unsafe { CStr::from_ptr(ptr) })
}

/// A writable copy of `s` for the parsers that take a name apart in place.
/// Hold the `Vec` for as long as the [`CText`] over it is used.
pub(crate) fn scratch(s: &CStr) -> Vec<u8> {
    s.to_bytes_with_nul().to_vec()
}

/// [`scratch`] over bytes that are not yet a C string.
pub(crate) fn scratch_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bytes.len() + 1);
    buf.extend_from_slice(bytes);
    buf.push(0);
    buf
}

/// A [`CText`] over a [`scratch`] buffer.
pub(crate) fn text_of(buf: &mut [u8]) -> CText {
    debug_assert_eq!(buf.last(), Some(&0), "scratch buffers are NUL-terminated");
    // SAFETY: a live, writable, NUL-terminated buffer.
    unsafe { CText::new(buf.as_mut_ptr().cast()) }
}

/// Skip the whitespace at `p`.
pub(crate) fn skip_white(p: CText) -> CText {
    // SAFETY: `skipwhite` walks within the string, stopping at its
    // terminator at the latest.
    unsafe { CText::new(skipwhite(p.raw())) }
}

/// An `xfree`-able copy of `s`, for the fields of a node.
pub(crate) fn dup(s: &CStr) -> *mut c_char {
    // SAFETY: `s` is NUL-terminated and live for the call.
    unsafe { xstrdup(s.as_ptr()) }
}

/// [`dup`] for bytes that are not yet a C string.
pub(crate) fn dup_bytes(bytes: &[u8]) -> *mut c_char {
    // SAFETY: a live slice; `xmemdupz` copies it and adds the terminator.
    unsafe { xmemdupz(bytes.as_ptr().cast(), bytes.len()) as *mut c_char }
}

/// `xfree()` for a string field, which may be null.
pub(crate) fn free_str(s: *mut c_char) {
    // SAFETY: null or a pointer this module got from the same allocator.
    unsafe { xfree(s.cast()) }
}

// ---------------------------------------------------------------------------
// The Dicts `menu_get()` and `menu_info()` answer with
// ---------------------------------------------------------------------------
//
// One wrapper per `tv_dict_*`/`tv_list_*` entry point, so that the two
// builders themselves are safe code. Every `dict`/`list` here is one this
// module has just allocated and still owns; the keys are `'static` literals
// and the values are copied or taken over by the callee.

pub(crate) fn dict_alloc() -> *mut dict_T {
    // SAFETY: allocates a fresh Dict and never answers null.
    unsafe { tv_dict_alloc() }
}

pub(crate) fn dict_len(dict: *const dict_T) -> c_long {
    // SAFETY: a live Dict, or null (which answers 0).
    unsafe { tv_dict_len(dict) }
}

pub(crate) fn dict_add_str(dict: *mut dict_T, key: &CStr, value: &CStr) {
    dict_add_str_raw(dict, key, value.as_ptr());
}

/// [`dict_add_str`] for a value that is still a raw pointer.
pub(crate) fn dict_add_str_raw(dict: *mut dict_T, key: &CStr, value: *const c_char) {
    // SAFETY: see the section note; `tv_dict_add_str` copies `value`.
    unsafe { tv_dict_add_str(dict, key.as_ptr(), key.count_bytes(), value) };
}

/// [`dict_add_str`] handing over an allocation the Dict then owns.
pub(crate) fn dict_add_allocated_str(dict: *mut dict_T, key: &CStr, value: *mut c_char) {
    // SAFETY: see the section note; the Dict takes over `value`.
    unsafe { tv_dict_add_allocated_str(dict, key.as_ptr(), key.count_bytes(), value) };
}

pub(crate) fn dict_add_nr(dict: *mut dict_T, key: &CStr, value: varnumber_T) {
    // SAFETY: see the section note.
    unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
}

pub(crate) fn dict_add_bool(dict: *mut dict_T, key: &CStr, value: bool) {
    // SAFETY: see the section note.
    unsafe { tv_dict_add_bool(dict, key.as_ptr(), key.count_bytes(), value.into()) };
}

pub(crate) fn dict_add_list(dict: *mut dict_T, key: &CStr, value: *mut list_T) {
    // SAFETY: see the section note; the Dict takes a reference to the list.
    unsafe { tv_dict_add_list(dict, key.as_ptr(), key.count_bytes(), value) };
}

/// A nested Dict under a key given as raw bytes -- `menu_get()` files each
/// mapping under a mode letter, and takes only the *first* byte of one, so
/// terminal mode lands under `t` rather than `tl`.
pub(crate) fn dict_add_dict(dict: *mut dict_T, key: &[u8], value: *mut dict_T) {
    // SAFETY: see the section note; `key` is a live slice of `key.len()`.
    unsafe { tv_dict_add_dict(dict, key.as_ptr().cast(), key.len(), value) };
}

pub(crate) fn list_alloc() -> *mut list_T {
    // SAFETY: allocates a fresh List and never answers null.
    unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) }
}

pub(crate) fn list_append_dict(list: *mut list_T, dict: *mut dict_T) {
    // SAFETY: see the section note.
    unsafe { tv_list_append_dict(list, dict) };
}

pub(crate) fn list_append_str(list: *mut list_T, value: &CStr) {
    // SAFETY: see the section note; a negative length means "to the NUL".
    unsafe { tv_list_append_string(list, value.as_ptr(), -1) };
}

/// A right-hand side with its special keys spelled out, as an allocation the
/// Dict takes over.
pub(crate) fn special_text(rhs: *const c_char) -> *mut c_char {
    // SAFETY: a node's rhs: NUL-terminated and live.
    unsafe { str2special_save(rhs, false, false) }
}

/// One codepoint as an owned string -- the mnemonic, for the two Dicts.
/// Answers the empty string for a node with no mnemonic, as C's
/// `buf[utf_char2bytes(0, buf)] = NUL` leaves an empty buffer.
pub(crate) fn char_as_text(c: c_int) -> CString {
    let mut buf = [0u8; 8];
    // SAFETY: `utf_char2bytes` writes at most `MB_MAXCHAR` (6) bytes.
    let len = unsafe { utf_char2bytes(c, buf.as_mut_ptr().cast()) } as usize;
    let bytes = &buf[..len];
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(len);
    CString::new(&bytes[..end]).expect("truncated at the first NUL")
}

// ---------------------------------------------------------------------------
// Classifying a name
// ---------------------------------------------------------------------------

/// Whether `name` can appear in the menubar -- everything that is not one of
/// the three special roots or explicitly hidden.
pub(crate) fn is_menubar(name: &CStr) -> bool {
    !is_popup(name)
        && !is_toolbar(name)
        && !is_winbar(name)
        && name.to_bytes().first() != Some(&MNU_HIDDEN_CHAR)
}

/// Whether `name` names the right-click menu, whose per-mode copies
/// `PopUpn`, `PopUpi`, ... all match too.
pub(crate) fn is_popup(name: &CStr) -> bool {
    name.to_bytes().starts_with(b"PopUp")
}

fn is_toolbar(name: &CStr) -> bool {
    name.to_bytes().starts_with(b"ToolBar")
}

/// Whether `name` names a window toolbar menu.
fn is_winbar(name: &CStr) -> bool {
    name.to_bytes().starts_with(b"WinBar")
}

/// Whether `name` is a separator: it both starts and ends with `-`.
///
/// A one-byte `"-"` satisfies both, as it does in C.
pub(crate) fn is_separator(name: &CStr) -> bool {
    let bytes = name.to_bytes();
    bytes.first() == Some(&b'-') && bytes.last() == Some(&b'-')
}

/// Whether `name` is kept out of the menubar and the listing: the hidden
/// marker, or one of `PopUp`'s per-mode copies (plain `PopUp` is shown).
pub(crate) fn is_hidden(name: &CStr) -> bool {
    let bytes = name.to_bytes();
    bytes.first() == Some(&MNU_HIDDEN_CHAR) || (is_popup(name) && bytes.len() > 5)
}

/// [`is_separator`] for the callers outside this module, which hold a node's
/// raw `dname` or a completion candidate.
///
/// # Safety
/// `name` must name a NUL-terminated string.
pub unsafe fn menu_is_separator(name: *const c_char) -> bool {
    // SAFETY: the caller's obligation.
    is_separator(unsafe { CStr::from_ptr(name) })
}

// ---------------------------------------------------------------------------
// The mode the editor is in
// ---------------------------------------------------------------------------

/// Which mode's right-hand side a menu invoked right now should run, or
/// [`MENU_INDEX_INVALID`] if the editor is in none of them.
pub(crate) fn get_menu_mode() -> c_int {
    if State.get() & MODE_TERMINAL != 0 {
        return MENU_INDEX_TERMINAL;
    }
    if VIsual_active.get() {
        return if VIsual_select.get() {
            MENU_INDEX_SELECT
        } else {
            MENU_INDEX_VISUAL
        };
    }
    if State.get() & MODE_INSERT != 0 {
        return MENU_INDEX_INSERT;
    }
    if State.get() & MODE_CMDLINE != 0
        || State.get() == MODE_ASKMORE
        || State.get() == MODE_HITRETURN
    {
        return MENU_INDEX_CMDLINE;
    }
    if finish_op.get() {
        return MENU_INDEX_OP_PENDING;
    }
    if State.get() & MODE_NORMAL != 0 {
        return MENU_INDEX_NORMAL;
    }
    if State.get() & MODE_LANGMAP != 0 {
        // Must be an "r" command, which behaves like Insert mode.
        return MENU_INDEX_INSERT;
    }
    MENU_INDEX_INVALID
}

/// [`get_menu_mode`] as a `MENU_*_MODE` bit, or 0 for no mode at all.
pub fn get_menu_mode_flag() -> c_int {
    let mode = get_menu_mode();
    if mode == MENU_INDEX_INVALID {
        return 0;
    }
    1 << mode
}

/// Show the `PopUp` menu for the mode the editor is in -- `PopUpn` in
/// Normal mode, `PopUpi` in Insert mode, and so on.
///
/// # Safety
/// Must run from the main loop: the popup takes over key input.
pub unsafe fn show_popupmenu() {
    let menu_mode = get_menu_mode();
    if menu_mode == MENU_INDEX_INVALID {
        return;
    }
    let mode = MODE_CHARS[menu_mode as usize];

    // SAFETY: `mode` is a `'static` string and the event takes no buffer of
    // its own; the autocmd may redefine menus, which is why the search below
    // runs afterwards.
    unsafe {
        apply_autocmds(
            EVENT_MENUPOPUP,
            mode.as_ptr().cast_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        )
    };

    // "PopUp" followed by this mode's letters; a longer name still matches,
    // as the C prefix compare did.
    let found = root_first()
        .into_iter()
        .flat_map(Menu::siblings)
        .find(|menu| {
            let name = menu.name().to_bytes();
            name.starts_with(b"PopUp") && name[5..].starts_with(mode.to_bytes())
        });

    // Only show a popup when it is defined and has entries.
    let Some(menu) = found.filter(|menu| menu.children().is_some()) else {
        return;
    };
    // SAFETY: a live node, and the caller's main-loop obligation.
    unsafe { pum_show_popupmenu(menu.raw()) };
}
