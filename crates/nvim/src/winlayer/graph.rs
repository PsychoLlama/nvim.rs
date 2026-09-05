//! The window, buffer and tabpage graph: which of each exists, in what
//! order, and which one is current.
//!
//! The transpiler parked these beside `main()` because upstream declares them
//! in `globals.h`, but every one of them is a node of the tree [`winlayer`]
//! wraps: the doubly-linked window list (`firstwin`/`lastwin`), the buffer
//! list (`firstbuf`/`lastbuf`), the tabpage list (`first_tabpage`), the frame
//! tree's root (`topframe`), and the `cur*` cursors into all three. The
//! command-line window's saved graph (`cmdwin_*`) belongs with them: it is a
//! second, temporary editor tree that the first one is swapped out for.
//!
//! Only the `cur*` pointers are raw. The list heads already carry the
//! [`WinId`]/[`BufId`]/[`TabId`] handles this module's registries hand out,
//! which is the shape the rest of them are headed for.
//!
//! [`winlayer`]: super
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// Not `forbid(unsafe_code)`: that lint rejects the name-mangling override on
// `curwin`, whose symbol plugins and the functional suite read directly.
#![deny(unsafe_op_in_unsafe_fn)]

use super::{BufId, TabId, WinId};
use crate::global_cell::GlobalCell;
use crate::types::{Buffer, Frame, Tabpage, Window};
use core::ffi::c_int;

pub(crate) static firstwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static lastwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static prevwin: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
#[unsafe(no_mangle)]
pub static curwin: GlobalCell<*mut Window> = GlobalCell::new(::core::ptr::null_mut::<Window>());
pub(crate) static topframe: GlobalCell<*mut Frame> =
    GlobalCell::new(::core::ptr::null_mut::<Frame>());
pub(crate) static first_tabpage: GlobalCell<Option<TabId>> = GlobalCell::new(None);
pub(crate) static curtab: GlobalCell<*mut Tabpage> =
    GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub(crate) static lastused_tabpage: GlobalCell<*mut Tabpage> =
    GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub(crate) static firstbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static lastbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub static curbuf: GlobalCell<*mut Buffer> = GlobalCell::new(::core::ptr::null_mut::<Buffer>());
pub(crate) static cmdwin_type: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_result: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_buf: GlobalCell<*mut Buffer> =
    GlobalCell::new(::core::ptr::null_mut::<Buffer>());
pub(crate) static cmdwin_win: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub(crate) static cmdwin_old_curwin: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub(crate) static cmdline_win: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
