//! Buffers: the list of them, and the state of one.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`open`] | reading a file into a buffer, and the scratch forms |
//! | [`close`] | unloading, deleting and wiping one |
//! | [`switch`] | `:buffer`, `:bnext`, `:bdelete` |
//! | [`enter`] | making a buffer current |
//! | [`list`] | creating an entry in the buffer list, and finding one |
//! | [`expand`] | completing a buffer name |
//! | [`pos`] | the per-window remembered cursor position |
//! | [`info`] | `:ls`, CTRL-G, `'title'` |
//! | [`name`] | a buffer's file name and the alternate file |
//! | [`all`] | `:ball` |
//! | [`modeline`] | `chk_modeline()` and `'modelines'` |
//! | [`type`] | the `'buftype'` predicates |
//!
//! What stays here is the flag alphabet the twelve share (`DOBUF_*`,
//! `BLN_*`, `BFA_*`, `READ_*`), the `bufref_T` layer every one of them uses to
//! survive an autocommand (`set_bufref`, `bufref_valid`, `buf_valid`), the
//! `b:changedtick` and buffer-number counters, and `buf_meta_total`, the
//! marktree accessor `buffer.h` had as a `static inline`.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{c_bytes, lastbuf};
use crate::src::nvim::map::{map_put_ref_int_ptr_t, mh_get_int};
use crate::src::nvim::types::{
    AlignTextPos, CdCause, ExtmarkOp, Map_int_ptr_t, MarkAdjustMode, MarkTree, MetaIndex,
    OptValType, UndoObjectType, WinSplit, WinStyle, bfa_values, bln_values, buf_T, bufref_T,
    cmd_addr_T, dobuf_action_values, dobuf_start_values, etype_T, getf_values, ptr_t, uint32_t,
    varnumber_T, win_T,
};
use crate::src::nvim::window::{window_layout_lock, window_layout_unlock};

// The carve of the transpiled module; see each child's docs.
mod all;
mod close;
mod enter;
mod expand;
mod info;
mod list;
mod modeline;
mod name;
mod open;
mod pos;
mod switch;
mod r#type;

pub use self::all::*;
pub use self::close::*;
pub use self::enter::*;
pub use self::expand::*;
pub use self::info::*;
pub use self::list::*;
pub use self::modeline::*;
pub use self::name::*;
pub use self::open::*;
pub use self::pos::*;
pub use self::switch::*;
pub use self::r#type::*;

pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_16 = 1073741823;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_17 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_17 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_17 = 1;
pub const kCdCauseAuto: CdCause = 2;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type dobuf_flags_value = ::core::ffi::c_uint;
pub const DOBUF_SKIPHELP: dobuf_flags_value = 4;
pub const DOBUF_FORCEIT: dobuf_flags_value = 1;
pub const BFA_IGNORE_ABORT: bfa_values = 8;
pub const BFA_KEEP_UNDO: bfa_values = 4;
pub const BFA_WIPE: bfa_values = 2;
pub const BFA_DEL: bfa_values = 1;
pub const READ_NOWINENTER: C2Rust_Unnamed_29 = 128;
pub const OPT_LOCAL: C2Rust_Unnamed_33 = 2;
pub const OPT_MODELINE: C2Rust_Unnamed_33 = 4;
pub const ETYPE_MODELINE: etype_T = 4;
pub const SHM_FILEINFO: C2Rust_Unnamed_24 = 70;
pub const READ_BUFFER: C2Rust_Unnamed_29 = 8;
pub const READ_STDIN: C2Rust_Unnamed_29 = 4;
pub const READ_NEW: C2Rust_Unnamed_29 = 1;
pub const READ_FIFO: C2Rust_Unnamed_29 = 64;
pub const READ_NOFILE: C2Rust_Unnamed_29 = 256;
pub const BCO_NOHELP: C2Rust_Unnamed_32 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_32 = 1;
pub const kBffInitChangedtick: C2Rust_Unnamed_35 = 2;
pub const kBffClearWinInfo: C2Rust_Unnamed_35 = 1;
pub const BCO_ALWAYS: C2Rust_Unnamed_32 = 2;
pub const ECMD_FORCEIT: C2Rust_Unnamed_27 = 8;
pub const ECMD_ONE: C2Rust_Unnamed_28 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufmatch_T {
    pub buf: *mut buf_T,
    pub match_0: *mut ::core::ffi::c_char,
}
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_25 = 4096;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_25 = 2;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_30 = -2147483648;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_25 = 8192;
pub const SHM_RO: C2Rust_Unnamed_24 = 114;
pub const SHM_MOD: C2Rust_Unnamed_24 = 109;
pub const READ_DUMMY: C2Rust_Unnamed_29 = 16;
pub const ECMD_HIDE: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const ECMD_OLDBUF: C2Rust_Unnamed_27 = 4;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const ECMD_LAST: C2Rust_Unnamed_28 = -1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const OPT_GLOBAL: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NEVERLOADED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BF_WRITE_MASK: ::core::ffi::c_int = BF_NOTEDITED + BF_NEW + BF_READERR;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    unsafe {
        let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_ptr_t.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    unsafe {
        let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
            map,
            key,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
pub unsafe fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    unsafe {
        return (*buf).changedtick_di.di_tv.vval.v_number;
    }
}
static e_attempt_to_delete_buffer_that_is_in_use_str: [::core::ffi::c_char; 52] =
    c_bytes(b"E937: Attempt to delete a buffer that is in use: %s\0");
static buf_free_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static top_file_num: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
unsafe extern "C" fn trigger_undo_ftplugin(mut buf: *mut buf_T, mut win: *mut win_T) {
    unsafe {
        let win_was_locked: bool = (*win).w_locked;
        window_layout_lock();
        (*buf).b_locked += 1;
        (*win).w_locked = true_0 != 0;
        do_cmdline_cmd(c"if exists('b:undo_ftplugin') | exe b:undo_ftplugin | endif".as_ptr());
        (*buf).b_locked -= 1;
        (*win).w_locked = win_was_locked;
        window_layout_unlock();
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_bufref(mut bufref: *mut bufref_T, mut buf: *mut buf_T) {
    unsafe {
        (*bufref).br_buf = buf;
        (*bufref).br_fnum = if buf.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*buf).handle as ::core::ffi::c_int
        };
        (*bufref).br_buf_free_count = buf_free_count.get();
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bufref_valid(mut bufref: *mut bufref_T) -> bool {
    unsafe {
        return if (*bufref).br_buf_free_count == buf_free_count.get() {
            true_0
        } else {
            (buf_valid((*bufref).br_buf) as ::core::ffi::c_int != 0
                && (*bufref).br_fnum == (*(*bufref).br_buf).handle)
                as ::core::ffi::c_int
        } != 0;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buf_valid(mut buf: *mut buf_T) -> bool {
    unsafe {
        if buf.is_null() {
            return false_0 != 0;
        }
        let mut bp: *mut buf_T = lastbuf.get();
        while !bp.is_null() {
            if bp == buf {
                return true_0 != 0;
            }
            bp = (*bp).b_prev;
        }
        return false_0 != 0;
    }
}
static lasttitle: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static lasticon: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CPO_INTMOD: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const NO_LOCAL_UNDOLEVEL: ::core::ffi::c_int = -123456 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_MODELINE: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;

/// The buffer's running total of one kind of extmark metadata, kept at the
/// root of its marktree. `buffer.h` had this as a `static inline`.
pub unsafe fn buf_meta_total(b: *const buf_T, m: MetaIndex) -> uint32_t {
    unsafe { (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize] }
}
