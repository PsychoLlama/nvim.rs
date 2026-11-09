//! What the key stream is in the middle of.
//!
//! Where the last key came from and what came with it (`KEY_TYPED`,
//! `KEY_STUFFED`, `MOD_MASK`, `VGETC_CHAR`), whether mapping and abbreviation
//! are switched off for this read (`NO_MAPPING`, `NO_ZERO_MAPPING`,
//! `ALLOW_KEYS`, `EXPR_MAP_LOCK`), what the typeahead buffer did last
//! (`TYPEBUF_WAS_EMPTY`, `TYPEBUF_WAS_FILLED`, `maptick`), and the register
//! being recorded into or replayed from (`REG_RECORDING`, `REG_EXECUTING`).
//!
//! `GOT_INT` is here because this is where it is raised: `os_breakcheck`
//! reads the input for a pending CTRL-C, and every long loop in the tree
//! polls the flag that read sets.
//!
//! # One record
//!
//! Upstream declares these as `EXTERN`s in `globals.h` and statics in
//! `getchar.c`. They are one [`GetcharState`] here, behind one cell, with a
//! `const` selector per field under the old name -- see
//! [`state_record`] for the access rules.
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
//! symbol name (so it is defined in `harness.rs` and re-exported here).
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::GotcharsState;
use crate::getchar::{KeyBuffer, TypeAhead};
use crate::global_cell::{GlobalCell, state_record};
use crate::keycodes::ModMask;
use crate::os::fs::CFile;
use crate::os::libc::{size_t, uint8_t};
use crate::types::{Key, LuaRef};
use core::ffi::c_int;

pub use crate::harness::test_disable_char_avail;
pub(crate) static LANGMAP_MAPCHAR: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);

/// The key [`vungetc`](super::vungetc) put back, and what came with it.
///
/// The next `vgetc` answers it -- with these modifiers and this mouse
/// position -- instead of reading the typeahead; a typed key waits for the
/// stuff buffer to empty first, a stuffed one does not.
#[derive(Debug)]
pub(crate) struct UngotKey {
    pub(crate) c: Key,
    pub(crate) mod_mask: ModMask,
    pub(crate) mouse_grid: c_int,
    pub(crate) mouse_row: c_int,
    pub(crate) mouse_col: c_int,
    /// `KEY_STUFFED` when it was put back.
    pub(crate) stuffed: bool,
}

state_record! {
    /// What the key stream is in the middle of. See the [module docs](self).
    pub(crate) struct GetcharState in GETCHAR as GetcharField;

    // -- the key just read --
    /// The modifiers of the key `vgetc` answered last.
    pub(crate) MOD_MASK: ModMask = ModMask::NONE;
    /// The modifiers and the key as `vgetc` read them, before the
    /// modifiers were folded into the key: what a terminal or a hit-enter
    /// prompt puts back to be read again.
    pub(crate) VGETC_MOD_MASK: ModMask = ModMask::NONE;
    pub(crate) VGETC_CHAR: Key = Key::NUL;
    /// Nesting depth of `vgetc`.
    pub(crate) VGETC_BUSY: c_int = 0;
    /// The key was typed, not mapped or stuffed.
    pub(crate) KEY_TYPED: bool = false;
    /// Nonzero: the key came from the stuff buffer.
    pub(crate) KEY_STUFFED: bool = false;
    /// What remapping the key is still allowed (`RM_*`).
    pub(crate) KEY_NOREMAP: u8 = 0;
    /// CTRL-C was typed, or an operation was interrupted.
    pub(crate) GOT_INT: bool = false;

    // -- what may be done with the next keys --
    /// Nonzero: no mappings at all.
    pub(crate) NO_MAPPING: c_int = 0;
    /// Nonzero: no mapping for a `0` typed as a count.
    pub(crate) NO_ZERO_MAPPING: c_int = 0;
    /// Nonzero: special keys are recognised even with `NO_MAPPING`.
    pub(crate) ALLOW_KEYS: c_int = 0;
    /// Nonzero while an `<expr>` mapping is evaluated: no text changes.
    pub(crate) EXPR_MAP_LOCK: c_int = 0;
    /// Nonzero: keys are not simplified (`getchar()`'s `simplify: false`).
    pub(crate) NO_REDUCE_KEYS: c_int = 0;
    /// The modes CTRL-C is mapped in; it does not interrupt there.
    pub(crate) MAPPED_CTRL_C: c_int = 0;
    /// CTRL-C interrupts (sets `GOT_INT`) rather than being a key.
    pub(crate) CTRL_C_INTERRUPTS: bool = true;
    /// Don't read from the `:source!` streams.
    pub(crate) IGNORE_SCRIPT: bool = false;

    // -- the typeahead --
    /// `:normal` ran out of keys, and the last one was made up to end it.
    pub(crate) TYPEBUF_WAS_EMPTY: bool = false;
    /// The typeahead was filled from a client or `feedkeys()`.
    pub(crate) TYPEBUF_WAS_FILLED: bool = false;
    /// Bumped whenever keys are read or mapped, for the cmdline's
    /// "was this typed since" tests.
    pub(crate) MAPTICK: c_int = 0;
    /// A character `flush_buffers` must not throw away.
    pub(super) TYPEAHEAD_CHAR: u8 = 0;
    /// Bytes `ins_char_typebuf` put back that `vim.on_key()` must not see
    /// again.
    pub(super) ON_KEY_IGNORE_LEN: size_t = 0;
    /// The key `vungetc` put back, until `vgetc` answers it.
    pub(super) UNGOT: Option<UngotKey> = None;
    /// The key the previous forced answer to `:normal` used, so that the
    /// cmdline window alternates between ESC and CTRL-C.
    pub(super) NORMAL_BUSY_LAST_KEY: u8 = 0;

    // -- recording and replaying --
    /// The register `q` is recording into, or 0.
    pub(crate) REG_RECORDING: u8 = 0;
    /// The register `@` is executing, or 0.
    pub(crate) REG_EXECUTING: u8 = 0;
    /// Clear `REG_EXECUTING` at the next advancing read.
    pub(crate) PENDING_END_REG_EXECUTING: bool = false;
    /// The register last recorded into.
    pub(crate) REG_RECORDED: u8 = 0;
    /// How many bytes the last `gotchars` recorded, so that `get_recorded`
    /// can drop the keys that stopped the recording.
    ///
    /// Every arithmetic on this counter is **wrapping**, as the C's
    /// `size_t` is: `vgetc` subtracts what the previous call recorded and
    /// `ungetchars` subtracts what it took back, and either can take it
    /// below zero. A huge value then makes `get_recorded`'s
    /// `len >= LAST_RECORDED_LEN` fail and nothing is trimmed, which is
    /// what upstream does. `test_registers`' Test_recording_with_select_mode
    /// reaches it.
    pub(super) LAST_RECORDED_LEN: size_t = 0;
    /// How many bytes the last `vgetc` recorded. Peeking can record more,
    /// so `LAST_RECORDED_LEN` may have grown past it since.
    pub(super) LAST_VGETC_RECORDED_LEN: size_t = 0;
    /// What `gotchars` has half a key of, between calls. Changed through
    /// `with_mut`: the state machine is a leaf, and moving the key buffer
    /// out and back per typed byte showed in the input benchmark.
    pub(super) GOTCHARS_PENDING: GotcharsState = GotcharsState::new();
    /// What the 'showcmd' echo of a partial mapping has half a key of.
    pub(super) SHOWCMD_PENDING: GotcharsState = GotcharsState::new();
    /// The redo buffer may not change: an insert repeat is reading it.
    pub(super) BLOCK_REDO: bool = false;
    /// `!` was typed with the command being made redoable.
    pub(crate) BANGREDO: bool = false;
    /// The Lua callback a `K_LUA` key ran, which `.` repeats.
    pub(crate) REPEAT_LUAREF: LuaRef = -2;

    // -- scripts --
    /// The `:source!` stream being read, or -1.
    pub(super) CURSCRIPT: c_int = -1;
    /// Typed characters since the swap files were last synced.
    pub(super) TYPED_SINCE_SYNC: c_int = 0;
    /// The `-w`/`-W` file every typed character is copied to.
    pub(crate) SCRIPTOUT: Option<CFile> = None;
}

pub type FlushBuffers = ::core::ffi::c_uint;
/// The redo pair a user function or autocommand set aside. Not `Copy`: each
/// field owns a block chain, and `save_redobuff` *moves* them here.
/// [`Default`] is the "nothing saved yet" state its callers declare it in --
/// `mem::zeroed` is not usable on a struct holding an enum with a niche.
#[derive(Default, Debug)]
pub struct SaveRedo {
    pub(crate) sr_redobuff: KeyBuffer,
    pub(crate) sr_old_redobuff: KeyBuffer,
}
/// All three kinds of typeahead, set aside so that a prompt has to be
/// answered by the user. Not `Copy`, and `Default` rather than zeroed, for
/// the same reasons as [`SaveRedo`].
#[derive(Default, Debug)]
pub struct TypeaheadSave {
    pub(crate) save_typebuf: TypeAhead,
    pub(crate) typebuf_valid: bool,
    /// The key `vungetc` had put back, which the prompt must not answer.
    pub(crate) ungot: Option<UngotKey>,
    pub(crate) save_readbuf1: KeyBuffer,
    pub(crate) save_readbuf2: KeyBuffer,
}
