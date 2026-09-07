//! What the key stream is in the middle of.
//!
//! Where the last key came from and what came with it (`KeyTyped`,
//! `KeyStuffed`, `mod_mask`, `vgetc_char`), whether mapping and abbreviation
//! are switched off for this read (`no_mapping`, `no_zero_mapping`,
//! `allow_keys`, `expr_map_lock`), what the typeahead buffer did last
//! (`typebuf_was_empty`, `typebuf_was_filled`, `maptick`), and the register
//! being recorded into or replayed from (`reg_recording`, `reg_executing`).
//!
//! `got_int` is here because this is where it is raised: `os_breakcheck`
//! reads the input for a pending CTRL-C, and every long loop in the tree
//! polls the flag that read sets.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`test_disable_char_avail`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::keycodes::ModMask;
use crate::types::{FILE, LuaRef, uint8_t};
use core::ffi::c_int;

#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static mod_mask: GlobalCell<ModMask> = GlobalCell::new(ModMask::NONE);
pub(crate) static vgetc_mod_mask: GlobalCell<ModMask> = GlobalCell::new(ModMask::NONE);
pub(crate) static vgetc_char: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static vgetc_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static reg_recording: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static reg_executing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static pending_end_reg_executing: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static reg_recorded: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static no_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static no_zero_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static allow_keys: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static mapped_ctrl_c: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static ctrl_c_interrupts: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static typebuf_was_empty: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static expr_map_lock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static ignore_script: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static KeyTyped: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static KeyStuffed: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static maptick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static scriptout: GlobalCell<*mut FILE> =
    GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub(crate) static got_int: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static bangredo: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static langmap_mapchar: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);
pub(crate) static typebuf_was_filled: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static repeat_luaref: GlobalCell<LuaRef> = GlobalCell::new(-2 as LuaRef);
