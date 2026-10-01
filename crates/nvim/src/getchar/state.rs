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
//!
//! # One record
//!
//! Upstream declares these as `EXTERN`s in `globals.h` and statics in
//! `getchar.c`. They are one [`GetcharState`] here, behind one cell, with a
//! `const` selector per field under the old name -- see
//! [`state_record`](crate::global_cell::state_record) for the access rules.
//! The fields below the editor-wide ones are `getchar/`'s own: the key
//! `vungetc` put back ([`UngotKey`]), the half-assembled keys `gotchars` and
//! the 'showcmd' echo are holding, and the counters the `getchar.c` statics
//! were.
//!
//! What stays a cell of its own: the five key buffers and the typeahead
//! (each is mutated through a short `with_mut` of its own, and the
//! typeahead's static initial storage is compared by address), the
//! `:source!` stream stack, the `vim.on_key()` byte buffer, the 'langmap'
//! table, and `test_disable_char_avail`, which the oldtest harness writes by
//! symbol name.
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

use super::GotcharsState;
use crate::global_cell::{GlobalCell, state_record};
use crate::keycodes::ModMask;
use crate::os::fs::CFile;
use crate::types::{LuaRef, size_t, uint8_t};
use core::ffi::c_int;

#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static langmap_mapchar: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);

/// The key [`vungetc`](super::vungetc) put back, and what came with it.
///
/// The next `vgetc` answers it -- with these modifiers and this mouse
/// position -- instead of reading the typeahead; a typed key waits for the
/// stuff buffer to empty first, a stuffed one does not.
pub(crate) struct UngotKey {
    pub(crate) c: c_int,
    pub(crate) mod_mask: ModMask,
    pub(crate) mouse_grid: c_int,
    pub(crate) mouse_row: c_int,
    pub(crate) mouse_col: c_int,
    /// `KeyStuffed` when it was put back.
    pub(crate) stuffed: bool,
}

state_record! {
    /// What the key stream is in the middle of. See the [module docs](self).
    pub(crate) struct GetcharState in GETCHAR as GetcharField;

    // -- the key just read --
    /// The modifiers of the key `vgetc` answered last.
    pub(crate) mod_mask: ModMask = ModMask::NONE;
    /// The modifiers and the key as `vgetc` read them, before the
    /// modifiers were folded into the key: what a terminal or a hit-enter
    /// prompt puts back to be read again.
    pub(crate) vgetc_mod_mask: ModMask = ModMask::NONE;
    pub(crate) vgetc_char: c_int = 0;
    /// Nesting depth of `vgetc`.
    pub(crate) vgetc_busy: c_int = 0;
    /// The key was typed, not mapped or stuffed.
    pub(crate) KeyTyped: bool = false;
    /// Nonzero: the key came from the stuff buffer.
    pub(crate) KeyStuffed: c_int = 0;
    /// What remapping the key is still allowed (`RM_*`).
    pub(crate) KeyNoremap: c_int = 0;
    /// CTRL-C was typed, or an operation was interrupted.
    pub(crate) got_int: bool = false;

    // -- what may be done with the next keys --
    /// Nonzero: no mappings at all.
    pub(crate) no_mapping: c_int = 0;
    /// Nonzero: no mapping for a `0` typed as a count.
    pub(crate) no_zero_mapping: c_int = 0;
    /// Nonzero: special keys are recognised even with `no_mapping`.
    pub(crate) allow_keys: c_int = 0;
    /// Nonzero while an `<expr>` mapping is evaluated: no text changes.
    pub(crate) expr_map_lock: c_int = 0;
    /// Nonzero: keys are not simplified (`getchar()`'s `simplify: false`).
    pub(crate) no_reduce_keys: c_int = 0;
    /// The modes CTRL-C is mapped in; it does not interrupt there.
    pub(crate) mapped_ctrl_c: c_int = 0;
    /// CTRL-C interrupts (sets `got_int`) rather than being a key.
    pub(crate) ctrl_c_interrupts: bool = true;
    /// Don't read from the `:source!` streams.
    pub(crate) ignore_script: bool = false;

    // -- the typeahead --
    /// `:normal` ran out of keys, and the last one was made up to end it.
    pub(crate) typebuf_was_empty: bool = false;
    /// The typeahead was filled from a client or `feedkeys()`.
    pub(crate) typebuf_was_filled: bool = false;
    /// Bumped whenever keys are read or mapped, for the cmdline's
    /// "was this typed since" tests.
    pub(crate) maptick: c_int = 0;
    /// A character `flush_buffers` must not throw away.
    pub(super) typeahead_char: c_int = 0;
    /// Bytes `ins_char_typebuf` put back that `vim.on_key()` must not see
    /// again.
    pub(super) on_key_ignore_len: size_t = 0;
    /// The key `vungetc` put back, until `vgetc` answers it.
    pub(super) ungot: Option<UngotKey> = None;
    /// The key the previous forced answer to `:normal` used, so that the
    /// cmdline window alternates between ESC and CTRL-C.
    pub(super) normal_busy_last_key: c_int = 0;

    // -- recording and replaying --
    /// The register `q` is recording into, or 0.
    pub(crate) reg_recording: c_int = 0;
    /// The register `@` is executing, or 0.
    pub(crate) reg_executing: c_int = 0;
    /// Clear `reg_executing` at the next advancing read.
    pub(crate) pending_end_reg_executing: bool = false;
    /// The register last recorded into.
    pub(crate) reg_recorded: c_int = 0;
    /// How many bytes the last `gotchars` recorded, so that `get_recorded`
    /// can drop the keys that stopped the recording.
    ///
    /// Every arithmetic on this counter is **wrapping**, as the C's
    /// `size_t` is: `vgetc` subtracts what the previous call recorded and
    /// `ungetchars` subtracts what it took back, and either can take it
    /// below zero. A huge value then makes `get_recorded`'s
    /// `len >= last_recorded_len` fail and nothing is trimmed, which is
    /// what upstream does. `test_registers`' Test_recording_with_select_mode
    /// reaches it.
    pub(super) last_recorded_len: size_t = 0;
    /// How many bytes the last `vgetc` recorded. Peeking can record more,
    /// so `last_recorded_len` may have grown past it since.
    pub(super) last_vgetc_recorded_len: size_t = 0;
    /// What `gotchars` has half a key of, between calls.
    pub(super) gotchars_pending: GotcharsState = GotcharsState::new();
    /// What the 'showcmd' echo of a partial mapping has half a key of.
    pub(super) showcmd_pending: GotcharsState = GotcharsState::new();
    /// The redo buffer may not change: an insert repeat is reading it.
    pub(super) block_redo: bool = false;
    /// `!` was typed with the command being made redoable.
    pub(crate) bangredo: bool = false;
    /// The Lua callback a `K_LUA` key ran, which `.` repeats.
    pub(crate) repeat_luaref: LuaRef = -2;

    // -- scripts --
    /// The `:source!` stream being read, or -1.
    pub(super) curscript: c_int = -1;
    /// Typed characters since the swap files were last synced.
    pub(super) typed_since_sync: c_int = 0;
    /// The `-w`/`-W` file every typed character is copied to.
    pub(crate) scriptout: Option<CFile> = None;
}
