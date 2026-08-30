//! RAII guards over the editor's suppression counters and script context.
//!
//! `textlock`, `sandbox`, `msg_silent`, `emsg_off`, `emsg_skip`,
//! `no_wait_return`, `no_mapping`, `allow_keys` and `RedrawingDisabled` are
//! the globals that say "for the duration of this operation, don't show
//! errors / don't redraw / don't let anything touch the buffer". C bumps
//! them by hand and unbumps them at every exit — which is fine until an
//! exit is missed, and in Rust a `?` or a panic between the two halves
//! leaks the suppression for the rest of the session.
//!
//! Everything here is the same two moves the C makes, with the release
//! attached to a scope instead of to the programmer's memory:
//!
//! - [`Bump`] — the counter shape. `+= 1` on acquire, `-= 1` on drop, so
//!   nesting composes exactly as it does in C.
//! - [`Saved`] — the save/restore shape. Overwrite with a fixed value on
//!   acquire, put the old value back on drop. Not the same thing as a
//!   `Bump`: restoring a *saved* value also undoes whatever the scope's
//!   callees did to the counter, which is deliberate at the sites that use
//!   it (a `:silent` that ends at a prompt, an autocmd window that must
//!   redraw regardless of nesting).
//!
//! Neither constructor is public. The vocabulary is the named constructors
//! on [`Suppress`], [`Allow`], [`Lock`] and [`Keys`], which is what makes
//! the counter and its intended direction greppable.
//!
//! ```ignore
//! let _guard = Suppress::emsg();          // emsg_off += 1 for this scope
//! let _guard = cond.then(Suppress::emsg); // ... only when `cond`
//! let _guard = Allow::messages();         // msg_silent = 0, restored after
//! ```
//!
//! [`Script`] is the odd one out: not a counter but the whole `sctx_T`
//! saying which script the running code belongs to, overwritten for a
//! scope and put back. It is the same save/restore shape as [`Saved`],
//! over a value rather than an `int`.
//!
//! Bind to a named `_guard`, never to `let _ =`: `_` drops immediately and
//! the scope runs unguarded. Where the C released the counter in the
//! middle of a long body, bind without the underscore and `drop(guard)` at
//! that exact point rather than inventing a block — the release point is
//! load-bearing (an error raised after it is meant to be seen) and the
//! guard is still what runs it on an early exit.
//!
//! # The counters that are still bumped by hand, and why
//!
//! Not every `x += 1` … `x -= 1` pair is a guard. These stay written out,
//! and a reviewer who "finishes the job" by converting one changes
//! behaviour:
//!
//! - **The restore is not the inverse of the bump.** `do_cmdline` raises
//!   `ex_nesting_level` when the line came from a function and lowers it
//!   again only if a *re-evaluated* test still says so; `open_scriptin`
//!   and `openscript` raise `curscript` and lower it only on the failure
//!   path, because on success the script stays on the stack past the
//!   return.
//! - **The scope is a state machine, not a block.** `RedrawingDisabled`
//!   under `disabled_redraw` spans Insert mode's entry and its exit;
//!   `debug_backtrace_level` is what `:up` and `:down` move; `compl_pending`
//!   and `pum_selected` are positions, not depths.
//! - **The lifetime belongs to a caller several frames up.** The
//!   `enter`/`leave` pairs — `try_enter`/`try_leave`,
//!   `block_autocmds`/`unblock_autocmds` (a C-ABI pair a plugin can call),
//!   `apply_cmdmod`/`undo_cmdmod`, `do_cmdline_start`/`do_cmdline_end`,
//!   `verbose_enter`/`verbose_leave`, `incr_quickfix_busy`/`decr_quickfix_busy`,
//!   `save_last_search_pattern`/`restore_last_search_pattern`,
//!   `ui_busy_start`/`ui_busy_stop`, `window_lock`/`window_unlock` — are
//!   the *public* form of a guard already; wrapping the counter would just
//!   move the pairing problem to their own callers.
//! - **`buf_write` releases `no_wait_return` inside a callee.** The
//!   contract is written on `buf_write_do_autocmds`; a guard here would
//!   have to be handed across the call.
//! - **Upstream leaks it on purpose.** `win_float_setup_preview` returns
//!   with `RedrawingDisabled` and `no_u_sync` still raised and says so.
//!
//! Everything else — every counter whose release is the arithmetic inverse
//! of its bump, taken and released inside one body — is a guard, and the
//! ratchet's job is to keep it that way.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::main::{
    RedrawingDisabled, allbuf_lock, allow_keys, autocmd_no_enter, autocmd_no_leave, cmdline_star,
    curbuf_splice_pending, current_sctx, disable_fold_update, emsg_off, emsg_silent, emsg_skip,
    expr_map_lock, inhibit_delete_count, msg_listdo_overwrite, msg_silent, no_check_timestamps,
    no_mapping, no_u_sync, no_wait_return, no_zero_mapping, sandbox, tabpage_move_disallowed,
    textlock,
};
use crate::types::{scid_T, sctx_T};
use core::ffi::c_int;

/// A counter held one higher for the lifetime of the guard.
///
/// Drop subtracts what the constructor added, so a `Bump` nests: the
/// counter is only back at rest once the outermost one is gone.
#[must_use = "the counter is released as soon as the guard is dropped"]
pub struct Bump {
    cell: &'static GlobalCell<c_int>,
    by: c_int,
}

impl Bump {
    fn new(cell: &'static GlobalCell<c_int>) -> Self {
        cell.set(cell.get() + 1);
        Bump { cell, by: 1 }
    }

    /// A bump of `by` rather than 1, for the two sites that add a boolean.
    fn by(cell: &'static GlobalCell<c_int>, by: c_int) -> Self {
        cell.set(cell.get() + by);
        Bump { cell, by }
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        self.cell.set(self.cell.get() - self.by);
    }
}

/// A counter overwritten for the lifetime of the guard, then put back.
///
/// Unlike [`Bump`] this does not nest — dropping restores the value the
/// constructor saw, whatever happened in between. That is what the C does
/// at these sites and why they are spelled differently.
#[must_use = "the old value is restored as soon as the guard is dropped"]
pub struct Saved {
    cell: &'static GlobalCell<c_int>,
    saved: c_int,
}

impl Saved {
    fn new(cell: &'static GlobalCell<c_int>, value: c_int) -> Self {
        Saved {
            cell,
            saved: cell.replace(value),
        }
    }

    /// Save unconditionally, overwrite only when `cond`.
    ///
    /// Not the same as `cond.then(…)`: the restore happens either way,
    /// which is what a scope that hands control to arbitrary Lua wants.
    fn when(cond: bool, cell: &'static GlobalCell<c_int>, value: c_int) -> Self {
        let saved = cell.get();
        if cond {
            cell.set(value);
        }
        Saved { cell, saved }
    }
}

impl Drop for Saved {
    fn drop(&mut self) {
        self.cell.set(self.saved);
    }
}

/// Both halves of the "read a key literally" pair, released together.
#[must_use = "the counters are released as soon as the guard is dropped"]
pub struct RawKeys {
    _no_mapping: Bump,
    _allow_keys: Bump,
}

/// `emsg_off` and `msg_silent` together: "run this and tell me nothing".
#[must_use = "the counters are released as soon as the guard is dropped"]
pub struct Quiet {
    _emsg: Bump,
    _messages: Bump,
}

/// Guards that turn some part of the editor's output off for a scope.
pub struct Suppress;

impl Suppress {
    /// `emsg_off` — error messages are not displayed at all.
    ///
    /// For the caller who evaluates something it expects to fail and does
    /// not want the user to hear about it.
    pub fn emsg() -> Bump {
        Bump::new(&emsg_off)
    }

    /// `emsg_skip` — errors from an expression that is only being parsed,
    /// not executed (a skipped `:if` branch, a `:for` over a bad list).
    pub fn emsg_skip() -> Bump {
        Bump::new(&emsg_skip)
    }

    /// `msg_silent` — messages are computed but not shown, as under
    /// `:silent`.
    pub fn messages() -> Bump {
        Bump::new(&msg_silent)
    }

    /// [`Suppress::messages`] whose release *restores* the level it found
    /// rather than subtracting one.
    ///
    /// The API's two output-capturing entry points do it this way: they
    /// put the whole message state back in one block afterwards, so a
    /// script that leaked a `:silent` cannot escape through them.
    pub fn messages_saved() -> Saved {
        Saved::new(&msg_silent, msg_silent.get() + 1)
    }

    /// [`Suppress::messages_saved`] that only raises the level when `cond`,
    /// but restores it either way — `execute()`'s `{silent}` argument, where
    /// an explicit empty value asks for output *and* still resets whatever
    /// the executed commands left behind.
    pub fn messages_saved_when(cond: bool) -> Saved {
        Saved::when(cond, &msg_silent, msg_silent.get() + 1)
    }

    /// `no_wait_return` — a message shown in this scope does not stop for
    /// the hit-enter prompt.
    pub fn wait_return() -> Bump {
        Bump::new(&no_wait_return)
    }

    /// `RedrawingDisabled` — the screen is not updated while this is held.
    pub fn redraw() -> Bump {
        Bump::new(&RedrawingDisabled)
    }

    /// `emsg_off = 1` — errors off for the scope, restoring the level that
    /// was in effect rather than decrementing it. The one site spelled this
    /// way compiles a user's tag pattern, where an error raised by a
    /// *nested* evaluation must not survive the scope either.
    pub fn emsg_outright() -> Saved {
        Saved::new(&emsg_off, 1)
    }

    /// [`Suppress::emsg`] + [`Suppress::messages`]: the recurring "compile
    /// and run this pattern, and say nothing whatever it does" pair.
    pub fn output() -> Quiet {
        Quiet {
            _emsg: Self::emsg(),
            _messages: Self::messages(),
        }
    }

    /// `emsg_silent` — an error raised in this scope still sets `v:errmsg`
    /// and still aborts, it is only not *shown*.
    ///
    /// Weaker than [`Suppress::emsg`], which stops the error being raised
    /// at all: `:silent!`, `'debug'` and `assert_fails()` all read this
    /// counter rather than that one.
    pub fn emsg_silent() -> Bump {
        Bump::new(&emsg_silent)
    }

    /// `no_u_sync` — undo does not start a new change while this is held.
    ///
    /// Everything typed inside the scope joins the change that was already
    /// open, which is what makes an expression register, a `CTRL-V` and a
    /// `:s///c` prompt one undoable edit rather than several.
    pub fn undo_sync() -> Bump {
        Bump::new(&no_u_sync)
    }

    /// `curbuf_splice_pending` — the individual line edits in this scope do
    /// not each announce themselves.
    ///
    /// The scope's caller sends one `extmark_splice` for the whole thing
    /// afterwards, so extmarks and the buffer-update RPC see one change
    /// where the code made three or four.
    pub fn splice() -> Bump {
        Bump::new(&curbuf_splice_pending)
    }

    /// `disable_fold_update` — folds are not recomputed in this scope.
    ///
    /// For the operation that leaves the buffer half-moved part way
    /// through, where a `'foldexpr'` re-evaluated on it would see lines
    /// that are about to move again.
    pub fn fold_update() -> Bump {
        Bump::new(&disable_fold_update)
    }

    /// `inhibit_delete_count` — a line deleted in this scope does not count
    /// towards the "N fewer lines" report.
    pub fn delete_count() -> Bump {
        Bump::new(&inhibit_delete_count)
    }

    /// `msg_listdo_overwrite` — a file message in this scope does not
    /// overwrite the line above it, whatever `'shortmess'` says.
    ///
    /// `:argdo` and its siblings hold this so that the per-file header
    /// stays on screen above the command's own output.
    pub fn message_overwrite() -> Bump {
        Bump::new(&msg_listdo_overwrite)
    }

    /// `cmdline_star` — the command line shows `*` for each character
    /// instead of what was typed, and nothing typed in the scope is
    /// recorded, echoed or completed. `inputsecret()`'s guard.
    pub fn cmdline_echo() -> Bump {
        Bump::new(&cmdline_star)
    }

    /// `no_check_timestamps` — a file whose timestamp changed under the
    /// editor during this scope raises no warning.
    pub fn timestamp_checks() -> Bump {
        Bump::new(&no_check_timestamps)
    }

    /// `no_zero_mapping` — a `0` read in this scope is a count digit, not
    /// the "go to column 0" command, so it must not resolve a mapping.
    pub fn zero_mapping() -> Bump {
        Bump::new(&no_zero_mapping)
    }

    /// `autocmd_no_enter` — no `WinEnter`/`BufEnter` fires in this scope.
    pub fn win_enter_autocmds() -> Bump {
        Bump::new(&autocmd_no_enter)
    }

    /// `autocmd_no_leave` — no `WinLeave`/`BufLeave` fires in this scope.
    pub fn win_leave_autocmds() -> Bump {
        Bump::new(&autocmd_no_leave)
    }

    /// Both of the above: the scope walks the window list and enters
    /// windows as bookkeeping, which the user's autocommands must not see.
    ///
    /// Where the two are released at *different* points — the release
    /// order is load-bearing at three sites, which enter a window with one
    /// of the pair already down — take them separately and `drop` each at
    /// the point the C decremented it.
    pub fn win_enter_leave_autocmds() -> WinAutocmds {
        WinAutocmds {
            _enter: Self::win_enter_autocmds(),
            _leave: Self::win_leave_autocmds(),
        }
    }

    /// A suppression counter that belongs to one module rather than to the
    /// editor as a whole, named by its own `static`.
    ///
    /// The constructors above exist so that a *global* counter's intended
    /// direction is greppable from one file; a `static` private to the
    /// module that reads it is already that, and does not earn a name here.
    pub fn counter(cell: &'static GlobalCell<c_int>) -> Bump {
        Bump::new(cell)
    }
}

/// A recursion or nesting counter, held one higher for a scope.
///
/// The odd guard out: what it protects is not a suppression but a *count of
/// how deep in this operation we are* — the `static RECURSE: GlobalCell<c_int>`
/// a recursive parser keeps inside itself so that a self-referential
/// expression cannot exhaust the C stack, the editor's `ex_nesting_level`,
/// and the `..._busy` flags a callee reads to find out that it is being
/// re-entered. The cell is usually the caller's own rather than one of the
/// editor's globals, which is why this one constructor is generic where the
/// rest are named. The shape is [`Bump`]'s; what it buys is that the un-bump
/// can no longer be skipped by a `?` on the way out, which is exactly what
/// happens the moment such a body starts answering `Result`.
pub struct Depth;

impl Depth {
    /// Hold `cell` one higher until the guard is dropped.
    pub fn of(cell: &'static GlobalCell<c_int>) -> Bump {
        Bump::new(cell)
    }
}

/// Guards that lift a suppression for a scope and put it back afterwards.
///
/// The mirror of [`Suppress`]: these are the sites where the editor has to
/// reach the user *despite* an enclosing `:silent` or a redraw-disabled
/// operation — a prompt, a dialog, a swap-file question.
pub struct Allow;

impl Allow {
    /// `msg_silent = 0` — this scope's messages reach the user even inside
    /// `:silent`.
    pub fn messages() -> Saved {
        Saved::new(&msg_silent, 0)
    }

    /// `RedrawingDisabled = 0` — this scope redraws even inside an
    /// operation that had disabled it.
    pub fn redraw() -> Saved {
        Saved::new(&RedrawingDisabled, 0)
    }

    /// `no_wait_return = 0` — the hit-enter prompt is armed again for this
    /// scope, whatever the caller had asked for.
    pub fn wait_return() -> Saved {
        Saved::new(&no_wait_return, 0)
    }

    /// `textlock = 0` — the callback about to run is allowed to change
    /// text even though the caller was inside a text-locked operation.
    pub fn text_changes() -> Saved {
        Saved::new(&textlock, 0)
    }

    /// [`Allow::text_changes`], lifting the lock only when `cond`, but
    /// restoring it either way.
    pub fn text_changes_when(cond: bool) -> Saved {
        Saved::when(cond, &textlock, 0)
    }

    /// `expr_map_lock = 0` — [`Allow::text_changes`]'s companion; the two
    /// sites that invite arbitrary Lua in lift both locks together.
    pub fn expr_map() -> Saved {
        Saved::new(&expr_map_lock, 0)
    }

    /// [`Allow::expr_map`], lifting the lock only when `cond`, but
    /// restoring it either way.
    pub fn expr_map_when(cond: bool) -> Saved {
        Saved::when(cond, &expr_map_lock, 0)
    }

    /// `allow_keys = 0` — key codes are *not* recognised in this scope, so
    /// a raw `<BS>` byte stays a byte.
    pub fn no_key_codes() -> Saved {
        Saved::new(&allow_keys, 0)
    }

    /// `no_mapping -= 1` — the inverse of [`Keys::unmapped`], for the
    /// callee that has to read a *mapped* key back out of a caller that
    /// had suppressed mapping (`'langmap'`, composing characters).
    pub fn mapping() -> Bump {
        Bump::by(&no_mapping, -1)
    }

    /// [`Allow::mapping`] for both halves of the pair — the inverse of
    /// [`Keys::unmapped_with_codes`].
    pub fn mapping_with_codes() -> RawKeys {
        RawKeys {
            _no_mapping: Bump::by(&no_mapping, -1),
            _allow_keys: Bump::by(&allow_keys, -1),
        }
    }

    /// `autocmd_no_enter -= 1` — the inverse of
    /// [`Suppress::win_enter_autocmds`], for the callee that has to let one
    /// `BufEnter` through a caller that had switched them off.
    pub fn win_enter_autocmds() -> Bump {
        Bump::by(&autocmd_no_enter, -1)
    }

    /// `autocmd_no_leave -= 1` — the inverse of
    /// [`Suppress::win_leave_autocmds`].
    pub fn win_leave_autocmds() -> Bump {
        Bump::by(&autocmd_no_leave, -1)
    }

    /// `no_check_timestamps = 0` — this scope checks file timestamps even
    /// inside an operation that had switched the check off, restoring the
    /// level afterwards.
    pub fn timestamp_checks() -> Saved {
        Saved::new(&no_check_timestamps, 0)
    }
}

/// Guards over the two locks that say what the code running inside them is
/// allowed to do.
pub struct Lock;

impl Lock {
    /// `textlock` — buffer text, window layout and the current
    /// buffer/window must not change while this is held.
    pub fn text() -> Bump {
        Bump::new(&textlock)
    }

    /// `sandbox` — the code about to run came from somewhere untrusted
    /// (a modeline, a `'foldexpr'`, a tag command) and the operations
    /// marked unsafe-in-sandbox are refused.
    pub fn sandbox() -> Bump {
        Bump::new(&sandbox)
    }

    /// `expr_map_lock` — [`Lock::text`]'s companion for the `<expr>`
    /// mapping and abbreviation expansions, which additionally must not
    /// change the mapping tables they are being read from.
    pub fn expr_map() -> Bump {
        Bump::new(&expr_map_lock)
    }

    /// `allbuf_lock` — no buffer may be added, removed or renamed while
    /// this is held, and `:cd` is refused.
    ///
    /// The scopes that take it hand control to an autocommand while
    /// holding a raw `buf_T *`.
    pub fn all_buffers() -> Bump {
        Bump::new(&allbuf_lock)
    }

    /// `tabpage_move_disallowed` — an autocommand fired in this scope may
    /// not reorder the tab pages the scope is walking.
    pub fn tabpage_move() -> Bump {
        Bump::new(&tabpage_move_disallowed)
    }

    /// A lock counter that belongs to one module rather than to the editor
    /// as a whole, named by its own `static` — [`Suppress::counter`]'s
    /// sibling, for a counter whose sense is "refuse this operation".
    pub fn held(cell: &'static GlobalCell<c_int>) -> Bump {
        Bump::new(cell)
    }
}

/// Both halves of the "walk the windows without the user noticing" pair,
/// released together.
#[must_use = "the counters are released as soon as the guard is dropped"]
pub struct WinAutocmds {
    _enter: Bump,
    _leave: Bump,
}

/// Guards over how the next key read is decoded.
pub struct Keys;

impl Keys {
    /// `no_mapping` — the keys read in this scope do not go through
    /// mappings or abbreviations.
    pub fn unmapped() -> Bump {
        Bump::new(&no_mapping)
    }

    /// `no_mapping` + `allow_keys`: the recurring pair. Read the next key
    /// literally, but still decode the multi-byte key codes.
    pub fn unmapped_with_codes() -> RawKeys {
        RawKeys {
            _no_mapping: Self::unmapped(),
            _allow_keys: Bump::new(&allow_keys),
        }
    }
}

/// The script context put back when the guard is dropped.
#[must_use = "the previous script context is restored as soon as the guard is dropped"]
pub(crate) struct SavedSctx {
    saved: sctx_T,
}

impl Drop for SavedSctx {
    fn drop(&mut self) {
        current_sctx.set(self.saved);
    }
}

/// Guards over `current_sctx` — which script the code running now belongs
/// to.
///
/// Every site that hands control to code written somewhere else — an
/// autocommand, a mapping, an option's expression, a sourced file, an API
/// call — points `current_sctx` at whoever *wrote* that code, so that
/// `:verbose`, `<SID>`, `<sfile>` and the error messages name the script
/// rather than whatever happened to be running when it fired.
pub(crate) struct Script;

impl Script {
    /// Run this scope as `sctx`.
    pub(crate) fn context(sctx: sctx_T) -> SavedSctx {
        SavedSctx {
            saved: current_sctx.replace(sctx),
        }
    }

    /// [`Script::context`] changing only the script id, leaving the
    /// sequence number and the line where they were.
    pub(crate) fn sid(sid: scid_T) -> SavedSctx {
        Self::context(current_sctx.get().with_sid(sid))
    }

    /// Save the context and leave it alone, for the scope that fills it in
    /// itself or only sometimes.
    pub(crate) fn saved() -> SavedSctx {
        SavedSctx {
            saved: current_sctx.get(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialises the tests below.
    ///
    /// They assert *absolute* values of process-wide counters, and `cargo
    /// test` runs a binary's tests in parallel threads, so two of them that
    /// touch the same cell -- `a_negative_bump_adds_back` and
    /// `raw_keys_releases_both` both drive `no_mapping` -- interleave and one
    /// of them sees the other's state.
    static COUNTERS: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// The lock, ignoring a poisoning left by an earlier failure: these tests
    /// restore what they change, and a second report of the first failure is
    /// noise.
    fn counters() -> std::sync::MutexGuard<'static, ()> {
        COUNTERS.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn bump_nests_and_unwinds() {
        let _serial = counters();
        assert_eq!(emsg_off.get(), 0);
        {
            let _outer = Suppress::emsg();
            assert_eq!(emsg_off.get(), 1);
            {
                let _inner = Suppress::emsg();
                assert_eq!(emsg_off.get(), 2);
            }
            assert_eq!(emsg_off.get(), 1);
        }
        assert_eq!(emsg_off.get(), 0);
    }

    #[test]
    fn saved_restores_what_it_saw() {
        let _serial = counters();
        msg_silent.set(3);
        {
            let _guard = Allow::messages();
            assert_eq!(msg_silent.get(), 0);
            // A callee bumping it does not survive the restore.
            let inner = Suppress::messages();
            assert_eq!(msg_silent.get(), 1);
            core::mem::forget(inner);
        }
        assert_eq!(msg_silent.get(), 3);
        msg_silent.set(0);
    }

    #[test]
    fn a_panic_releases_the_guard() {
        let _serial = counters();
        // The whole point: no path out of the scope can leak the counter.
        let caught = std::panic::catch_unwind(|| {
            let _guard = Lock::text();
            assert_eq!(textlock.get(), 1);
            panic!("the callee failed");
        });
        assert!(caught.is_err());
        assert_eq!(textlock.get(), 0);
    }

    #[test]
    fn a_negative_bump_adds_back() {
        let _serial = counters();
        no_mapping.set(2);
        {
            let _lifted = Allow::mapping();
            assert_eq!(no_mapping.get(), 1);
        }
        assert_eq!(no_mapping.get(), 2);
        no_mapping.set(0);
    }

    #[test]
    fn win_autocmds_releases_both() {
        let _serial = counters();
        {
            let _guard = Suppress::win_enter_leave_autocmds();
            assert_eq!((autocmd_no_enter.get(), autocmd_no_leave.get()), (1, 1));
        }
        assert_eq!((autocmd_no_enter.get(), autocmd_no_leave.get()), (0, 0));
    }

    #[test]
    fn allow_is_the_inverse_of_suppress() {
        let _serial = counters();
        let _outer = Suppress::win_enter_autocmds();
        assert_eq!(autocmd_no_enter.get(), 1);
        {
            let _lifted = Allow::win_enter_autocmds();
            assert_eq!(autocmd_no_enter.get(), 0);
        }
        assert_eq!(autocmd_no_enter.get(), 1);
    }

    #[test]
    fn a_module_counter_needs_no_name_here() {
        let _serial = counters();
        static PRIVATE: GlobalCell<c_int> = GlobalCell::new(0);
        {
            let _held = Lock::held(&PRIVATE);
            let _deeper = Suppress::counter(&PRIVATE);
            assert_eq!(PRIVATE.get(), 2);
        }
        assert_eq!(PRIVATE.get(), 0);
    }

    #[test]
    fn raw_keys_releases_both() {
        let _serial = counters();
        {
            let _guard = Keys::unmapped_with_codes();
            assert_eq!((no_mapping.get(), allow_keys.get()), (1, 1));
        }
        assert_eq!((no_mapping.get(), allow_keys.get()), (0, 0));
    }
}
