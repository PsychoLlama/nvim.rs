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

use core::ffi::{CStr, c_int};
use std::ffi::CString;
use std::rc::Rc;

/// A menu node's identity, which no other node ever takes, even after it is
/// deleted: what `:popup` and `:emenu` hold across the user code they run,
/// re-found afterwards.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct MenuId(pub(crate) u64);

/// One node of the menu tree: a submenu when it has children, a menu item
/// otherwise.
///
/// The node owns its names, its right-hand sides and its children, which
/// are kept in display order (by priority, then by when they were added).
#[derive(Clone)]
pub struct VimMenu {
    pub(crate) id: MenuId,
    /// The modes the node is available in, `MENU_*_MODE` bits.
    pub modes: c_int,
    /// The modes it is enabled in.
    pub enabled: c_int,
    /// The name as `:menu` spelled it, mnemonic and accelerator included.
    pub name: CString,
    /// The displayed name: no `&` mnemonic marker, no accelerator.
    pub dname: CString,
    /// The pre-translation name and displayed name, for a node
    /// `:menutranslate` renamed; set together.
    pub en_name: Option<CString>,
    pub en_dname: Option<CString>,
    /// The mnemonic character, or 0.
    pub mnemonic: c_int,
    /// The accelerator text, i.e. whatever followed the name's `<Tab>`.
    pub actext: Option<CString>,
    pub priority: c_int,
    /// The right-hand side per mode index. Several modes commonly share one
    /// string, which is why it is counted.
    pub strings: [Option<Rc<CStr>>; 8],
    pub noremap: [c_int; 8],
    pub silent: [bool; 8],
    pub children: Vec<VimMenu>,
}
