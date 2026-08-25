#![deny(unsafe_op_in_unsafe_fn)]

//! Decoration providers: the Lua callbacks a plugin registers with
//! `nvim_set_decoration_provider` to decorate a redraw as it happens.
//!
//! A provider is a bundle of callbacks keyed by namespace. The redraw calls
//! them in a fixed order — `on_start` once per redraw, then `on_buf` per
//! buffer, `on_win` per window, then `on_line`/`on_range` per line or span
//! while `win_line` is drawing, and `on_end` at the end — and each of them
//! can turn the provider off for the rest of the redraw by answering `false`
//! or by raising an error.
//!
//! Two rules run through the whole file:
//!
//! * **Every callback pumps.** `nlua_call_ref` runs arbitrary Lua, which can
//!   register another provider and so grow [`PROVIDERS`]. Nothing here holds
//!   a reference or an index-derived pointer across a call: the provider is
//!   re-resolved by index afterwards, which is exactly what the C does with
//!   its `kv_A(decor_providers, i)` re-fetch and the comment explaining it.
//! * **Errors are budgeted.** A provider that raises is reported at most
//!   [`CB_MAX_ERROR`] times and then disabled, so a broken plugin cannot
//!   flood the message area on every redraw.

use crate::api::extmark::describe_ns;
use crate::api::private::helpers::{
    api_clear_error, api_free_array, api_free_object, api_object_to_bool,
};
use crate::decoration::{DecorStateRef, decor_check_to_be_deleted, decor_range_count};
use crate::global_cell::GlobalCell;
use crate::guard::Lock;
use crate::highlight::hl_check_ns;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::lua::executor::{api_free_luaref, nlua_call_ref};
use crate::main::{display_tick, ns_hl_active};
use crate::r#move::validate_botline_win;
use crate::msg_schedule_semsg_multiline_c;
use crate::types::builders::ArrayBuf;
use crate::types::{
    Array, DecorProvider, DecorProvider_state, Error, Integer, LuaRef, LuaRetMode, NS, Object,
    buf_T, kErrorTypeNone, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeInteger, linenr_T,
    win_T,
};

use core::ffi::{c_char, c_int};
use core::ptr;

// The provider's own lifecycle state. `Disabled` is sticky (only a fresh
// `nvim_set_decoration_provider` clears it); the other two are per-redraw.
/// Set up for this redraw and taking calls.
pub(crate) const kDecorProviderActive: DecorProvider_state = 1;
/// Turned off for the rest of *this window* — `on_win` or `on_line` declined.
pub(crate) const kDecorProviderWinDisabled: DecorProvider_state = 2;
/// Turned off for the rest of *this redraw* — `on_start` declined.
pub(crate) const kDecorProviderRedrawDisabled: DecorProvider_state = 3;
/// Turned off until it is registered again.
pub(crate) const kDecorProviderDisabled: DecorProvider_state = 4;

/// `LuaRef` value meaning "no callback".
const LUA_NOREF: LuaRef = -2;
/// Errors reported from one provider before it is disabled.
const CB_MAX_ERROR: u8 = 3;
/// Ask `nlua_call_ref` for all return values (an `Array`).
const kRetMulti: LuaRetMode = 3;
/// Ask `nlua_call_ref` for one value, interpreted as a boolean.
const kRetNilBool: LuaRetMode = 1;

const ERROR_INIT: Error = Error {
    type_0: kErrorTypeNone,
    msg: ptr::null_mut(),
};

/// The registered providers, in registration order — which is the order every
/// callback runs in. Entries are never removed: unregistering clears the
/// callbacks and marks the provider disabled, because `w_ns_hl_winhl` and the
/// namespace highlight cache keep referring to it by namespace id.
static PROVIDERS: GlobalCell<Vec<DecorProvider>> = GlobalCell::new(Vec::new());

/// How many providers are registered. Read fresh on every loop iteration: a
/// callback can register one more.
fn provider_count() -> usize {
    PROVIDERS.with(Vec::len)
}

/// A copy of provider `idx`. `DecorProvider` is `Copy` and small, so the
/// loops here take a snapshot rather than hold a borrow across a Lua call.
fn provider(idx: usize) -> DecorProvider {
    PROVIDERS.with(|providers| providers[idx])
}

/// Runs `f` on provider `idx`. The borrow must not outlive `f` — in
/// particular `f` must not call Lua.
fn with_provider<R>(idx: usize, f: impl FnOnce(&mut DecorProvider) -> R) -> R {
    PROVIDERS.with_mut(|providers| f(&mut providers[idx]))
}

fn set_state(idx: usize, state: DecorProvider_state) {
    with_provider(idx, |p| p.state = state);
}

/// Report a provider's error, both to the log and to the message area.
///
/// # Safety
/// `name` and `msg` must be NUL-terminated.
unsafe fn decor_provider_error(ns_id: NS, name: *const c_char, msg: *const c_char) {
    // SAFETY: the caller's NUL-terminated strings, plus the editor's own
    // namespace table.
    unsafe {
        let ns = describe_ns(ns_id, c"(UNKNOWN PLUGIN)".as_ptr());
        logmsg_c!(
            LOGLVL_ERR,
            ptr::null(),
            c"decor_provider_error".as_ptr(),
            29,
            true,
            c"Error in decoration provider \"%s\" (ns=%s):\n%s".as_ptr(),
            name,
            ns,
            msg,
        );
        msg_schedule_semsg_multiline_c!(
            c"Decoration provider \"%s\" (ns=%s):\n%s".as_ptr(),
            name,
            ns,
            msg,
        );
    }
}

/// Call one provider callback and answer whether the provider is still good.
///
/// `res`, when given, asks for the callback's whole return list and receives
/// it; otherwise the single return value is read as a boolean, with
/// `default_true` standing in for `nil`.
///
/// The provider is named by *index*, not by pointer: the call below can
/// register another provider and move the vector out from under us.
///
/// # Safety
/// `name` must be NUL-terminated; `args` must be a live `Array` the callee
/// may consume.
unsafe fn decor_provider_invoke(
    idx: usize,
    name: *const c_char,
    callback: LuaRef,
    args: Array,
    default_true: bool,
    res: Option<&mut Array>,
) -> bool {
    // SAFETY: the caller's arguments; `nlua_call_ref` owns `args` from here.
    unsafe {
        let mut err = ERROR_INIT;
        let want_list = res.is_some();

        let locked = Lock::text();
        let ret = nlua_call_ref(
            callback,
            name,
            args,
            if want_list { kRetMulti } else { kRetNilBool },
            ptr::null_mut(),
            &raw mut err,
        );
        drop(locked);

        if err.type_0 == kErrorTypeNone {
            with_provider(idx, |p| p.error_count = 0);
            if let Some(res) = res {
                debug_assert!(ret.type_0 == kObjectTypeArray);
                *res = ret.data.array;
                return true;
            }
            if api_object_to_bool(
                ret,
                c"provider %s retval".as_ptr(),
                default_true,
                &raw mut err,
            ) {
                return true;
            }
        }

        if err.type_0 != kErrorTypeNone {
            let (ns_id, count) = with_provider(idx, |p| (p.ns_id, p.error_count));
            if count < CB_MAX_ERROR {
                decor_provider_error(ns_id, name, err.msg);
                // The report can reach Lua through the message area, so the
                // count is bumped through a fresh borrow.
                with_provider(idx, |p| {
                    p.error_count = count + 1;
                    if p.error_count >= CB_MAX_ERROR {
                        p.state = kDecorProviderDisabled;
                    }
                });
            }
        }

        api_clear_error(&raw mut err);
        // TODO(bfredl): wants to be on an arena
        api_free_object(ret);
        false
    }
}

/// Tell every provider with an `_on_spell_nav` callback that the spell
/// checker is looking at this span.
///
/// # Safety
/// `wp` must point to a live window.
pub(crate) unsafe fn decor_providers_invoke_spell(
    wp: *mut win_T,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) {
    // SAFETY: the caller's window; the callbacks re-enter the editor.
    unsafe {
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state != kDecorProviderDisabled && p.spell_nav != LUA_NOREF {
                let mut args = ArrayBuf::<6>::new();
                args.push(Object::integer((*wp).handle.into()));
                args.push(Object::integer((*(*wp).w_buffer).handle.into()));
                args.push(Object::integer(start_row.into()));
                args.push(Object::integer(start_col.into()));
                args.push(Object::integer(end_row.into()));
                args.push(Object::integer(end_col.into()));
                decor_provider_invoke(
                    idx,
                    c"spell".as_ptr(),
                    p.spell_nav,
                    args.array(),
                    true,
                    None,
                );
            }
        }
    }
}

/// Ask every `_on_conceal_line` callback about `row`.
///
/// # Safety
/// `wp` must point to a live window.
///
/// @return whether a provider placed any marks in the callback.
pub(crate) unsafe fn decor_providers_invoke_conceal_line(wp: *mut win_T, row: c_int) -> bool {
    // SAFETY: the caller's window; the callbacks re-enter the editor.
    unsafe {
        let keys = (*(*wp).w_buffer).b_marktree[0].n_keys;
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state != kDecorProviderDisabled && p.conceal_line != LUA_NOREF {
                let mut args = ArrayBuf::<4>::new();
                args.push(Object::integer((*wp).handle.into()));
                args.push(Object::integer((*(*wp).w_buffer).handle.into()));
                args.push(Object::integer(row.into()));
                decor_provider_invoke(
                    idx,
                    c"conceal_line".as_ptr(),
                    p.conceal_line,
                    args.array(),
                    true,
                    None,
                );
            }
        }
        (*(*wp).w_buffer).b_marktree[0].n_keys > keys
    }
}

/// Start a redraw: run every `on_start` callback and put the providers that
/// did not decline into the active state.
///
/// # Safety
/// Runs Lua; main thread only.
pub(crate) unsafe fn decor_providers_start() {
    // SAFETY: the callbacks re-enter the editor.
    unsafe {
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state == kDecorProviderDisabled {
                continue;
            }
            if p.redraw_start != LUA_NOREF {
                let mut args = ArrayBuf::<2>::new();
                args.push(Object::integer(display_tick.get() as c_int as Integer));
                let active = decor_provider_invoke(
                    idx,
                    c"start".as_ptr(),
                    p.redraw_start,
                    args.array(),
                    true,
                    None,
                );
                set_state(
                    idx,
                    if active {
                        kDecorProviderActive
                    } else {
                        kDecorProviderRedrawDisabled
                    },
                );
            } else {
                set_state(idx, kDecorProviderActive);
            }
        }
    }
}

/// Whether a decoration provider's Lua callback is on the stack, which is
/// what tells [`decor_free`](crate::decoration::decor_free) that a decoration
/// it is asked to delete may still be referenced by the ranges being drawn.
///
/// Upstream keeps this in `DecorState`, and it is the one field there that is
/// not about the window being drawn: it is read from every extmark deletion,
/// which is nowhere near the draw pass, and it stays true across the whole
/// callback rather than across a row. Leaving it in the state would have
/// forced the state handle down every path that can delete a mark; a cell of
/// its own, reached by `get`/`set`, costs nothing and says what it is.
static PROVIDER_RUNNING: GlobalCell<bool> = GlobalCell::new(false);

/// Whether a decoration provider's callback is on the stack; see
/// [`PROVIDER_RUNNING`].
pub(crate) fn decor_provider_running() -> bool {
    PROVIDER_RUNNING.get()
}

/// Announce that a decoration provider's callback is (no longer) running.
fn set_provider_running(running: bool) {
    PROVIDER_RUNNING.set(running);
}

/// Start a window: run every `on_win` callback. A provider that declines is
/// skipped for the rest of this window.
///
/// # Safety
/// `wp` must point to a live window; runs Lua.
pub(crate) unsafe fn decor_providers_invoke_win(wp: *mut win_T, state: DecorStateRef) {
    // SAFETY: the caller's window; the callbacks re-enter the editor.
    unsafe {
        // This might change in the future; then this would need
        // `set_provider_running` just like "on_line" below.
        debug_assert!(state.current_end == 0 && state.future_begin == decor_range_count(state));

        if provider_count() > 0 {
            validate_botline_win(wp);
        }
        let botline: linenr_T = (*wp).w_botline.min((*(*wp).w_buffer).b_ml.ml_line_count);

        for idx in 0..provider_count() {
            let p = with_provider(idx, |p| {
                if p.state == kDecorProviderWinDisabled {
                    p.state = kDecorProviderActive;
                }
                p.win_skip_row = 0;
                p.win_skip_col = 0;
                *p
            });

            if p.state == kDecorProviderActive && p.redraw_win != LUA_NOREF {
                let mut args = ArrayBuf::<4>::new();
                args.push(Object::window((*wp).handle));
                args.push(Object::buffer((*(*wp).w_buffer).handle));
                // TODO(bfredl): we are not using this, but should be first drawn line?
                args.push(Object::integer(((*wp).w_topline - 1).into()));
                args.push(Object::integer((botline - 1).into()));
                // TODO(bfredl): could skip a call if retval was interpreted like range?
                if !decor_provider_invoke(
                    idx,
                    c"win".as_ptr(),
                    p.redraw_win,
                    args.array(),
                    true,
                    None,
                ) {
                    set_state(idx, kDecorProviderWinDisabled);
                }
            }
        }
    }
}

/// Run every `on_line` callback for one window row.
///
/// # Safety
/// `wp` must point to a live window; runs Lua.
pub(crate) unsafe fn decor_providers_invoke_line(wp: *mut win_T, row: c_int) {
    // SAFETY: the caller's window; the callbacks re-enter the editor and may
    // place ephemeral decorations, which is what the flag below announces.
    unsafe {
        set_provider_running(true);
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state == kDecorProviderActive && p.redraw_line != LUA_NOREF {
                let mut args = ArrayBuf::<3>::new();
                args.push(Object::window((*wp).handle));
                args.push(Object::buffer((*(*wp).w_buffer).handle));
                args.push(Object::integer(row.into()));
                if !decor_provider_invoke(
                    idx,
                    c"line".as_ptr(),
                    p.redraw_line,
                    args.array(),
                    true,
                    None,
                ) {
                    // returned 'false' or errored: skip the rest of this window
                    set_state(idx, kDecorProviderWinDisabled);
                }
                hl_check_ns();
            }
        }
        set_provider_running(false);
    }
}

/// Run every `on_range` callback for one span.
///
/// A callback may answer `false` to be skipped for the rest of the window, or
/// a `(row, col)` pair saying "everything up to here is already decorated",
/// which the next call for an earlier span skips on.
///
/// # Safety
/// `wp` must point to a live window; runs Lua.
pub(crate) unsafe fn decor_providers_invoke_range(
    wp: *mut win_T,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) {
    // SAFETY: the caller's window; the callbacks re-enter the editor.
    unsafe {
        set_provider_running(true);
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state != kDecorProviderActive || p.redraw_range == LUA_NOREF {
                continue;
            }
            if p.win_skip_row > end_row || (p.win_skip_row == end_row && p.win_skip_col >= end_col)
            {
                continue;
            }

            let mut args = ArrayBuf::<6>::new();
            args.push(Object::window((*wp).handle));
            args.push(Object::buffer((*(*wp).w_buffer).handle));
            args.push(Object::integer(start_row.into()));
            args.push(Object::integer(start_col.into()));
            args.push(Object::integer(end_row.into()));
            args.push(Object::integer(end_col.into()));

            let mut res = Array {
                size: 0,
                capacity: 0,
                items: ptr::null_mut(),
            };
            let status = decor_provider_invoke(
                idx,
                c"range".as_ptr(),
                p.redraw_range,
                args.array(),
                true,
                Some(&mut res),
            );

            // The Lua call may have reallocated the provider vector, so
            // everything below goes through the index again.
            if !status {
                // errored: skip the rest of this window
                set_state(idx, kDecorProviderWinDisabled);
            } else if res.size >= 1 {
                let first = *res.items;
                if first.type_0 == kObjectTypeBoolean {
                    if !first.data.boolean {
                        set_state(idx, kDecorProviderWinDisabled);
                    }
                } else if first.type_0 == kObjectTypeInteger {
                    let row = first.data.integer;
                    let mut col = 0;
                    if res.size >= 2 {
                        let second = *res.items.add(1);
                        if second.type_0 == kObjectTypeInteger {
                            col = second.data.integer;
                        }
                    }
                    with_provider(idx, |p| {
                        p.win_skip_row = row as c_int;
                        p.win_skip_col = col as c_int;
                    });
                }
            }

            api_free_array(res);
            hl_check_ns();
        }
        set_provider_running(false);
    }
}

/// Run every `on_buf` callback for one buffer.
///
/// # Safety
/// `buf` must point to a live buffer; runs Lua.
pub(crate) unsafe fn decor_providers_invoke_buf(buf: *mut buf_T) {
    // SAFETY: the caller's buffer; the callbacks re-enter the editor.
    unsafe {
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state == kDecorProviderActive && p.redraw_buf != LUA_NOREF {
                let mut args = ArrayBuf::<2>::new();
                args.push(Object::buffer((*buf).handle));
                args.push(Object::integer(display_tick.get() as Integer));
                decor_provider_invoke(idx, c"buf".as_ptr(), p.redraw_buf, args.array(), true, None);
            }
        }
    }
}

/// Finish a redraw: run every `on_end` callback, then free the decorations a
/// callback asked to delete while they were still being drawn.
///
/// # Safety
/// Runs Lua; main thread only.
pub(crate) unsafe fn decor_providers_invoke_end() {
    // SAFETY: the callbacks re-enter the editor.
    unsafe {
        for idx in 0..provider_count() {
            let p = provider(idx);
            if p.state != kDecorProviderDisabled && p.redraw_end != LUA_NOREF {
                let mut args = ArrayBuf::<1>::new();
                args.push(Object::integer(display_tick.get() as c_int as Integer));
                decor_provider_invoke(idx, c"end".as_ptr(), p.redraw_end, args.array(), true, None);
            }
        }
        decor_check_to_be_deleted();
    }
}

/// Mark all cached state of per-namespace highlights as invalid, and
/// revalidate the current namespace.
///
/// Expensive! Should only be called by an already throttled validity check
/// like `highlight_changed()` (throttled to the next redraw or mode change).
///
/// # Safety
/// Reaches the highlight tables; main thread only.
pub(crate) unsafe fn decor_provider_invalidate_hl() {
    PROVIDERS.with_mut(|providers| {
        for p in providers.iter_mut() {
            p.hl_cached = false;
        }
    });

    if ns_hl_active.get() != 0 {
        ns_hl_active.set(-1);
        // SAFETY: the editor's own highlight tables.
        unsafe { hl_check_ns() };
    }
}

/// The provider for namespace `ns_id`, registering an empty one if `force`
/// and there is none.
///
/// The pointer is into the provider vector and is invalidated by anything
/// that can register a provider — that is, by any Lua call. Callers that keep
/// it across one must use [`with_decor_provider`] instead.
///
/// Namespace 0 is the global one and has no provider, so a caller asking for
/// a non-positive id is a bug — but only a `debug_assert!`, because upstream
/// spells it `assert(ns_id > 0)` (`v0.12.4:src/nvim/decoration_provider.c:305`)
/// and that vanishes under `NDEBUG`. A release upstream nvim reached through
/// `nvim_set_hl(-2, …)` appends a provider with a negative id and carries on;
/// aborting instead would be a divergence, not a fix.
///
/// # Safety
/// The answer must not outlive the next registration.
pub(crate) unsafe fn get_decor_provider(ns_id: NS, force: bool) -> *mut DecorProvider {
    debug_assert!(ns_id > 0);
    match provider_index(ns_id, force) {
        Some(idx) => PROVIDERS.with(|providers| providers.as_ptr().cast_mut().wrapping_add(idx)),
        None => ptr::null_mut(),
    }
}

/// Runs `f` on the provider for namespace `ns_id`, registering an empty one
/// if `force` and there is none. Answers `None` when there is no provider and
/// `force` is false.
///
/// This is the form to use around anything that can run Lua: it resolves the
/// provider afresh, so a callback that registered another provider in between
/// cannot leave a stale pointer behind.
///
/// Carries [`get_decor_provider`]'s `ns_id > 0` assertion, and for the same
/// reason it is a `debug_assert!` there.
pub(crate) fn with_decor_provider<R>(
    ns_id: NS,
    force: bool,
    f: impl FnOnce(&mut DecorProvider) -> R,
) -> Option<R> {
    debug_assert!(ns_id > 0);
    let idx = provider_index(ns_id, force)?;
    Some(with_provider(idx, f))
}

/// The index of `ns_id`'s provider, appending a fresh one if `force`.
fn provider_index(ns_id: NS, force: bool) -> Option<usize> {
    PROVIDERS.with_mut(|providers| {
        if let Some(idx) = providers.iter().position(|p| p.ns_id == ns_id) {
            return Some(idx);
        }
        if !force {
            return None;
        }
        providers.push(new_provider(ns_id));
        Some(providers.len() - 1)
    })
}

/// A provider with no callbacks, disabled until one is registered.
///
/// `conceal_line` starts at `LUA_REFNIL` (-1) rather than `LUA_NOREF` (-2),
/// which `DECORATION_PROVIDER_INIT` has done since the field was added. It is
/// harmless because a provider born here is `Disabled` and every caller tests
/// that first, and because registering callbacks runs `decor_provider_clear`
/// beforehand, which sets the slot properly. Preserved rather than corrected:
/// changing it would be a semantic change with no observable trigger.
const fn new_provider(ns_id: NS) -> DecorProvider {
    DecorProvider {
        ns_id,
        state: kDecorProviderDisabled,
        win_skip_row: 0,
        win_skip_col: 0,
        redraw_start: LUA_NOREF,
        redraw_buf: LUA_NOREF,
        redraw_win: LUA_NOREF,
        redraw_line: LUA_NOREF,
        redraw_range: LUA_NOREF,
        redraw_end: LUA_NOREF,
        hl_def: LUA_NOREF,
        spell_nav: LUA_NOREF,
        conceal_line: -1,
        hl_valid: 0,
        hl_cached: false,
        error_count: 0,
    }
}

/// Drop every callback `p` holds and disable it.
///
/// # Safety
/// `p` must be null or point to a live provider.
pub(crate) unsafe fn decor_provider_clear(p: *mut DecorProvider) {
    if p.is_null() {
        return;
    }
    // SAFETY: the caller's provider. `api_free_luaref` only touches the Lua
    // registry, so the provider vector cannot move under this.
    unsafe {
        for slot in [
            &raw mut (*p).redraw_start,
            &raw mut (*p).redraw_buf,
            &raw mut (*p).redraw_win,
            &raw mut (*p).redraw_line,
            &raw mut (*p).redraw_range,
            &raw mut (*p).redraw_end,
            &raw mut (*p).spell_nav,
            &raw mut (*p).conceal_line,
        ] {
            if *slot != LUA_NOREF {
                api_free_luaref(*slot);
                *slot = LUA_NOREF;
            }
        }
        (*p).state = kDecorProviderDisabled;
    }
}
