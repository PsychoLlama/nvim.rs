#![deny(unsafe_op_in_unsafe_fn)]

//! `vim.diff()`: the Lua binding over the bundled xdiff.
//!
//! # Boundary
//!
//! `nlua_xdl_diff` is a Lua C function and the three emit callbacks are
//! `xdl_diff`'s, so all four keep the C ABI and their raw `lua_State` /
//! `void *` arguments. Everything the callbacks reach — the linematch
//! refinement, the decision walk — is safe and lives elsewhere.
//!
//! The three shapes the binding can answer in are [`Mode`]: a unified-diff
//! *string* (the default, assembled in a `luaL_Buffer`), *nothing* with an
//! `on_hunk` callback run per hunk, or a *list* of `{start_a, count_a, //! start_b, count_b}` tuples (`result_type = 'indices'`).

use core::ffi::{c_char, c_int, c_long, c_void};
use core::{ptr, slice};

use crate::api::private::dispatch::key_dict_xdl_diff_get_field;
use crate::api::private::helpers::{api_clear_error, api_free_string, api_set_error};
use crate::linematch::{block_from_lnum, linematch_nbuffers};
use crate::lua::converter::nlua_pop_keydict;
use crate::lua::executor::{api_free_luaref, nlua_pushref};
use crate::lua::ffi::{
    LUA_TFUNCTION, LUA_TSTRING, LUA_TTABLE, lua_concat, lua_createtable, lua_error, lua_gettop,
    lua_isnumber, lua_objlen, lua_pcall, lua_pushinteger, lua_pushstring, lua_pushvalue,
    lua_rawseti, lua_settop, lua_tolstring, lua_tonumber, lua_type, luaL_argerror, luaL_buffinit,
    luaL_error, luaL_prepbuffer, luaL_pushresult, luaL_where,
};
use crate::memory::strequal;
use crate::types::{
    Arena, Error, KeyDict_xdl_diff, Object, OptionalKeys, String_0, int64_t, kErrorTypeException,
    kErrorTypeNone, kErrorTypeValidation, kObjectTypeBoolean, kObjectTypeInteger, kObjectTypeNil,
    linenr_T, lua_Integer, lua_State, luaL_Buffer, mmbuffer_t, mmfile_t, object_data, size_t,
    xdemitcb_t, xdemitconf_t, xpparam_t,
};
use crate::xdiff::ffi::xdl_diff;
use crate::xdiff::xtypes::{
    XDF_HISTOGRAM_DIFF, XDF_IGNORE_BLANK_LINES, XDF_IGNORE_CR_AT_EOL, XDF_IGNORE_WHITESPACE,
    XDF_IGNORE_WHITESPACE_AT_EOL, XDF_IGNORE_WHITESPACE_CHANGE, XDF_INDENT_HEURISTIC,
    XDF_NEED_MINIMAL, XDF_PATIENCE_DIFF,
};

/// What `vim.diff` was asked to produce.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// A unified-diff string.
    Unified,
    /// Nothing: `on_hunk` is called once per hunk instead.
    OnHunk,
    /// A list of `{start_a, count_a, start_b, count_b}` tuples.
    Locations,
}

/// A hunk's four coordinates: where the change starts in each document and
/// how many lines it covers there. Zero-based, as `xdl_diff` reports them;
/// the `+ 1` that makes a non-empty side one-based happens on the way out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hunk {
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
}

impl Hunk {
    fn from_cb(start_a: c_int, count_a: c_int, start_b: c_int, count_b: c_int) -> Self {
        Hunk {
            start_a,
            count_a,
            start_b,
            count_b,
        }
    }

    /// The four numbers as Lua sees them: a non-empty side is one-based,
    /// an empty one keeps its start where it is.
    fn one_based(self) -> [c_int; 4] {
        [
            if self.count_a > 0 {
                self.start_a + 1
            } else {
                self.start_a
            },
            self.count_a,
            if self.count_b > 0 {
                self.start_b + 1
            } else {
                self.start_b
            },
            self.count_b,
        ]
    }
}

/// The bits `linematch_nbuffers` reports a decision with: which of the two
/// buffers the step consumed a line from.
const COMPARED_BUFFER0: c_int = 1 << 0;
const COMPARED_BUFFER1: c_int = 1 << 1;

/// What the emit callbacks are handed as `cb_data`.
///
/// `ma`/`mb` are only set for [`Mode::Locations`], which needs the two
/// documents again to refine a hunk; `err` only for [`Mode::OnHunk`], which
/// is the only one that can fail inside the walk.
struct HunkContext {
    lstate: *mut lua_State,
    err: *mut Error,
    ma: *mut mmfile_t,
    mb: *mut mmfile_t,
    linematch: int64_t,
    iwhite: bool,
}

pub const KEYDICT_INIT: KeyDict_xdl_diff = KeyDict_xdl_diff {
    is_set__xdl_diff_: 0 as OptionalKeys,
    on_hunk: 0,
    result_type: String_0::NULL,
    algorithm: String_0::NULL,
    ctxlen: 0,
    interhunkctxlen: 0,
    linematch: Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    },
    ignore_whitespace: false,
    ignore_whitespace_change: false,
    ignore_whitespace_change_at_eol: false,
    ignore_cr_at_eol: false,
    ignore_blank_lines: false,
    indent_heuristic: false,
};

/// Append one `{start_a, count_a, start_b, count_b}` tuple to the list on top
/// of the stack. A zero count keeps its start where it is; a non-empty side
/// is reported one-based.
///
/// # Safety
/// `lstate` must be live with the result list on top and two free slots.
unsafe fn lua_pushhunk(lstate: *mut lua_State, hunk: Hunk) {
    // SAFETY: the caller's state; the tuple is built on top of the list and
    // appended to it, so the stack ends level.
    unsafe {
        lua_createtable(lstate, 0, 0);
        for (slot, value) in hunk.one_based().into_iter().enumerate() {
            lua_pushinteger(lstate, value as lua_Integer);
            lua_rawseti(lstate, -2, slot as c_int + 1);
        }
        let at = lua_objlen(lstate, -2) as c_int + 1;
        lua_rawseti(lstate, -2, at);
    }
}

/// Split one hunk into the finer hunks `linematch` finds inside it, and push
/// each. Runs only when `linematch` is on and the hunk is small enough.
///
/// # Safety
/// As [`lua_pushhunk`]; `ma`/`mb` must point at the two live documents.
unsafe fn get_linematch_results(
    lstate: *mut lua_State,
    ma: *mut mmfile_t,
    mb: *mut mmfile_t,
    hunk: Hunk,
    iwhite: bool,
) {
    // The two blocks as bytes; `xdl_diff` only hands over non-empty ones.
    // SAFETY: the caller's documents, which outlive this call.
    let (bytes_a, bytes_b) = unsafe {
        (
            slice::from_raw_parts((*ma).ptr.cast::<u8>(), (*ma).size as usize),
            slice::from_raw_parts((*mb).ptr.cast::<u8>(), (*mb).size as usize),
        )
    };
    let block_a = block_from_lnum(bytes_a, hunk.start_a as linenr_T + 1).unwrap_or_default();
    let block_b = block_from_lnum(bytes_b, hunk.start_b as linenr_T + 1).unwrap_or_default();
    let decisions = linematch_nbuffers(&[block_a, block_b], &[hunk.count_a, hunk.count_b], iwhite);
    // SAFETY: the caller's state, with the list still on top for each push.
    let push = |finer: Hunk| unsafe { lua_pushhunk(lstate, finer) };
    walk_decisions(hunk, &decisions, push);
}

/// The finer hunks a run of `linematch` decisions carves `hunk` into.
///
/// Each decision says which of the two buffers its step consumed a line
/// from, so runs of the same decision are one hunk: the walk closes a hunk
/// when the decision changes and once more at the end. It therefore always
/// emits at least one hunk, even for no decisions at all.
fn walk_decisions(hunk: Hunk, decisions: &[c_int], mut emit: impl FnMut(Hunk)) {
    let mut lnuma = hunk.start_a;
    let mut lnumb = hunk.start_b;
    let mut current = Hunk::from_cb(lnuma, 0, lnumb, 0);
    for (i, &decision) in decisions.iter().enumerate() {
        if i != 0 && decisions[i - 1] != decision {
            emit(current);
            current = Hunk::from_cb(lnuma, 0, lnumb, 0);
        }
        if decision & COMPARED_BUFFER0 != 0 {
            lnuma += 1;
            current.count_a += 1;
        }
        if decision & COMPARED_BUFFER1 != 0 {
            lnumb += 1;
            current.count_b += 1;
        }
    }
    emit(current);
}

/// `xdemitcb_t::out_line` for [`Mode::Unified`]: append the emitted text to
/// the `luaL_Buffer` behind `cb_data`.
///
/// # Safety
/// `cb_data` must be a live `luaL_Buffer *`, and `mb` must point at `nbuf`
/// buffers whose `ptr` holds `size` bytes.
unsafe extern "C" fn write_string(cb_data: *mut c_void, mb: *mut mmbuffer_t, nbuf: c_int) -> c_int {
    // `luaL_Buffer`'s inline capacity, which is also how much
    // `luaL_prepbuffer` guarantees. Upstream spells it `MIN(BUFSIZ, 16384)`
    // and `BUFSIZ` is 8192. Function-local so it stays out of the FFI golden.
    const LUAL_BUFFERSIZE: c_int = 8192;

    let buf = cb_data.cast::<luaL_Buffer>();
    for i in 0..nbuf {
        // SAFETY: the caller promises `nbuf` buffers.
        let (bytes, size) =
            unsafe { ((*mb.offset(i as isize)).ptr, (*mb.offset(i as isize)).size) };
        let mut total = 0;
        while total < size {
            // `MIN(size - total, LUAL_BUFFERSIZE)`; c2rust inlined the
            // constant's own `BUFSIZ > 16384` arms into both sides.
            let tocopy = (size - total).min(LUAL_BUFFERSIZE);
            // SAFETY: `buf` is the live Lua buffer, and `luaL_prepbuffer`
            // answers room for `LUAL_BUFFERSIZE` bytes -- at least `tocopy`.
            unsafe {
                let room = luaL_prepbuffer(buf);
                if room.is_null() {
                    return -1;
                }
                room.copy_from_nonoverlapping(bytes.offset(total as isize), tocopy as usize);
                // `luaL_addsize`, which the header spells as a macro.
                (*buf).p = (*buf).p.offset(tocopy as isize);
            }
            total += LUAL_BUFFERSIZE;
        }
    }
    0
}

/// `xdemitconf_t::hunk_func` for [`Mode::Locations`].
///
/// # Safety
/// `cb_data` must be a live `HunkContext *` whose `lstate` has the result
/// list on top.
unsafe extern "C" fn hunk_locations_cb(
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
    cb_data: *mut c_void,
) -> c_int {
    let hunk = Hunk::from_cb(start_a, count_a, start_b, count_b);
    // SAFETY: the caller's context, live for the whole `xdl_diff` call.
    let ctx = unsafe { &*cb_data.cast::<HunkContext>() };
    // A hunk wider than `linematch` is not worth refining.
    if ctx.linematch > 0 && int64_t::from(count_a + count_b) <= ctx.linematch {
        // SAFETY: as above; `ma`/`mb` are set for this mode.
        unsafe { get_linematch_results(ctx.lstate, ctx.ma, ctx.mb, hunk, ctx.iwhite) };
    } else {
        // SAFETY: as above.
        unsafe { lua_pushhunk(ctx.lstate, hunk) };
    }
    0
}

/// `xdemitconf_t::hunk_func` for [`Mode::OnHunk`]: call the user's function
/// with the hunk's four coordinates. A number it answers stops the walk.
///
/// # Safety
/// `cb_data` must be a live `HunkContext *` whose `lstate` has the callback
/// on top and whose `err` is writable.
unsafe extern "C" fn call_on_hunk_cb(
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
    cb_data: *mut c_void,
) -> c_int {
    let coordinates = Hunk::from_cb(start_a, count_a, start_b, count_b).one_based();
    // SAFETY: the caller's context, live for the whole `xdl_diff` call.
    let (lstate, err) = unsafe {
        let ctx = &*cb_data.cast::<HunkContext>();
        (ctx.lstate, ctx.err)
    };
    // SAFETY: the callback is at the top of the stack, and `lua_pcall`
    // consumes the copy plus its four arguments.
    let (fidx, failed) = unsafe {
        let fidx = lua_gettop(lstate);
        lua_pushvalue(lstate, fidx);
        for value in coordinates {
            lua_pushinteger(lstate, value as lua_Integer);
        }
        (fidx, lua_pcall(lstate, 4, 1, 0) != 0)
    };
    if failed {
        // SAFETY: the error message is on top; `api_set_error` copies it.
        unsafe {
            api_set_error(
                err,
                kErrorTypeException,
                c"on_hunk: %s".as_ptr(),
                lua_tolstring(lstate, -1, ptr::null_mut::<size_t>()),
            );
        }
        return -1;
    }
    // SAFETY: the result is on top and is dropped before returning.
    unsafe {
        let r = if lua_isnumber(lstate, -1) != 0 {
            lua_tonumber(lstate, -1) as c_int
        } else {
            0
        };
        lua_settop(lstate, fidx);
        r
    }
}

/// One of the two document arguments, as a borrowed `mmfile_t`.
///
/// # Safety
/// `lstate` must be a live Lua state.
unsafe fn get_string_arg(lstate: *mut lua_State, idx: c_int) -> mmfile_t {
    // SAFETY: the caller's state. `luaL_argerror` does not return, so past
    // the two checks the argument is a string of at most `INT_MAX` bytes,
    // and it stays on the stack for the whole call.
    unsafe {
        if lua_type(lstate, idx) != LUA_TSTRING {
            luaL_argerror(lstate, idx, c"expected string".as_ptr());
        }
        let mut size: size_t = 0;
        let ptr = lua_tolstring(lstate, idx, &raw mut size).cast_mut();
        if size > c_int::MAX as size_t {
            luaL_argerror(lstate, idx, c"string too long".as_ptr());
        }
        mmfile_t {
            ptr,
            size: size as c_int,
        }
    }
}

/// Read the options table at index 3 into `cfg`/`params`/`linematch`, and
/// answer which shape the caller asked for. A bad option sets `err` and
/// stops the read where it was.
///
/// # Safety
/// `lstate` must be a live Lua state with a table at index 3.
unsafe fn process_xdl_diff_opts(
    lstate: *mut lua_State,
    cfg: &mut xdemitconf_t,
    params: &mut xpparam_t,
    linematch: &mut int64_t,
    err: &mut Error,
) -> Mode {
    let mut opts: KeyDict_xdl_diff = KEYDICT_INIT;
    let mut err_param: *mut c_char = ptr::null_mut::<c_char>();
    // SAFETY: the caller's state and table; `opts` is a live keydict and
    // owns whatever the pop puts in it, which is freed at the end.
    unsafe {
        nlua_pop_keydict(
            lstate,
            (&raw mut opts).cast::<c_void>(),
            Some(key_dict_xdl_diff_get_field),
            &raw mut err_param,
            ptr::null_mut::<Arena>(),
            err,
        );
    }

    // SAFETY: the keydict's two string fields are NUL-terminated or null.
    let mode = unsafe { apply_opts(lstate, &opts, cfg, params, linematch, err) };

    // SAFETY: the keydict owns these; `opts` is not read again.
    unsafe {
        api_free_string(opts.result_type);
        api_free_string(opts.algorithm);
        api_free_luaref(opts.on_hunk);
    }
    mode
}

/// Whether the optional key at bit `optidx` was given.
fn is_set(opts: &KeyDict_xdl_diff, optidx: c_int) -> bool {
    opts.is_set__xdl_diff_ & (1 << optidx) != 0
}

/// The body of [`process_xdl_diff_opts`], split out so the keydict is freed
/// on every path — upstream's `goto exit`.
///
/// # Safety
/// `lstate` must be live; `opts` must be a populated keydict.
unsafe fn apply_opts(
    lstate: *mut lua_State,
    opts: &KeyDict_xdl_diff,
    cfg: &mut xdemitconf_t,
    params: &mut xpparam_t,
    linematch: &mut int64_t,
    err: &mut Error,
) -> Mode {
    // The bit index of each optional key in `is_set__xdl_diff_`, as apigen
    // names them. Function-local so they stay out of the FFI golden.
    const KEYSET_OPTIDX_xdl_diff__ctxlen: c_int = 1;
    const KEYSET_OPTIDX_xdl_diff__on_hunk: c_int = 2;
    const KEYSET_OPTIDX_xdl_diff__algorithm: c_int = 3;
    const KEYSET_OPTIDX_xdl_diff__linematch: c_int = 4;
    const KEYSET_OPTIDX_xdl_diff__result_type: c_int = 5;
    const KEYSET_OPTIDX_xdl_diff__interhunkctxlen: c_int = 6;

    let mut had_result_type_indices = false;
    // SAFETY: `result_type`/`algorithm` are NUL-terminated or null, which is
    // what `strequal` takes.
    if is_set(opts, KEYSET_OPTIDX_xdl_diff__result_type)
        && !unsafe { strequal(c"unified".as_ptr(), opts.result_type.data()) }
    {
        if unsafe { strequal(c"indices".as_ptr(), opts.result_type.data()) } {
            had_result_type_indices = true;
        } else {
            // SAFETY: `err` is the caller's.
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"not a valid result_type".as_ptr(),
                )
            };
            return Mode::Unified;
        }
    }

    // SAFETY: as above.
    if is_set(opts, KEYSET_OPTIDX_xdl_diff__algorithm)
        && !unsafe { strequal(c"myers".as_ptr(), opts.algorithm.data()) }
    {
        // SAFETY: as above.
        let algorithm = unsafe {
            [
                (c"minimal", XDF_NEED_MINIMAL),
                (c"patience", XDF_PATIENCE_DIFF),
                (c"histogram", XDF_HISTOGRAM_DIFF),
            ]
            .into_iter()
            .find(|(name, _)| strequal(name.as_ptr(), opts.algorithm.data()))
        };
        match algorithm {
            Some((_, flag)) => params.flags |= flag,
            None => {
                // SAFETY: `err` is the caller's.
                unsafe {
                    api_set_error(err, kErrorTypeValidation, c"not a valid algorithm".as_ptr())
                };
                return Mode::Unified;
            }
        }
    }

    if is_set(opts, KEYSET_OPTIDX_xdl_diff__ctxlen) {
        cfg.ctxlen = opts.ctxlen as c_long;
    }
    if is_set(opts, KEYSET_OPTIDX_xdl_diff__interhunkctxlen) {
        cfg.interhunkctxlen = opts.interhunkctxlen as c_long;
    }
    if is_set(opts, KEYSET_OPTIDX_xdl_diff__linematch) {
        // SAFETY: the union arm is the one `type_0` names.
        match opts.linematch.type_0 {
            kObjectTypeBoolean => {
                *linematch = if unsafe { opts.linematch.data.boolean } {
                    int64_t::MAX
                } else {
                    0
                };
            }
            kObjectTypeInteger => *linematch = unsafe { opts.linematch.data.integer },
            _ => {
                // SAFETY: `err` is the caller's.
                unsafe {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"linematch must be a boolean or integer".as_ptr(),
                    );
                }
                return Mode::Unified;
            }
        }
    }

    for (given, flag) in [
        (opts.ignore_whitespace, XDF_IGNORE_WHITESPACE),
        (opts.ignore_whitespace_change, XDF_IGNORE_WHITESPACE_CHANGE),
        (
            opts.ignore_whitespace_change_at_eol,
            XDF_IGNORE_WHITESPACE_AT_EOL,
        ),
        (opts.ignore_cr_at_eol, XDF_IGNORE_CR_AT_EOL),
        (opts.ignore_blank_lines, XDF_IGNORE_BLANK_LINES),
        (opts.indent_heuristic, XDF_INDENT_HEURISTIC),
    ] {
        if given {
            params.flags |= flag;
        }
    }

    if is_set(opts, KEYSET_OPTIDX_xdl_diff__on_hunk) {
        // SAFETY: `lstate` is live; the callback is left on the stack for
        // `call_on_hunk_cb` to copy.
        let is_function = unsafe {
            nlua_pushref(lstate, opts.on_hunk);
            lua_type(lstate, -1) == LUA_TFUNCTION
        };
        if !is_function {
            // SAFETY: `err` is the caller's.
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"on_hunk is not a function".as_ptr(),
                )
            };
        }
        return Mode::OnHunk;
    }
    if had_result_type_indices {
        return Mode::Locations;
    }
    Mode::Unified
}

/// `vim.diff(a, b[, opts])`.
///
/// # Safety
/// Called by Lua with a live `lua_State`.
pub unsafe extern "C-unwind" fn nlua_xdl_diff(lstate: *mut lua_State) -> c_int {
    // SAFETY: Lua's own state.
    if unsafe { lua_gettop(lstate) } < 2 {
        // SAFETY: as above.
        return unsafe { luaL_error(lstate, c"Expected at least 2 arguments".as_ptr()) };
    }
    // SAFETY: as above; both stay on the stack for the whole call.
    let (mut ma, mut mb) = unsafe { (get_string_arg(lstate, 1), get_string_arg(lstate, 2)) };

    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut::<c_char>(),
    };
    let mut cfg = xdemitconf_t {
        ctxlen: 0,
        interhunkctxlen: 0,
        flags: 0,
        find_func: None,
        find_func_priv: ptr::null_mut::<c_void>(),
        hunk_func: None,
    };
    let mut params = xpparam_t {
        flags: 0,
        anchors: ptr::null_mut::<*mut c_char>(),
        anchors_nr: 0,
    };
    let mut linematch: int64_t = 0;
    let mut mode = Mode::Unified;

    // SAFETY: as above.
    if unsafe { lua_gettop(lstate) } == 3 {
        // SAFETY: as above.
        if unsafe { lua_type(lstate, 3) } != LUA_TTABLE {
            // SAFETY: as above.
            return unsafe { luaL_argerror(lstate, 3, c"expected table".as_ptr()) };
        }
        // SAFETY: as above, with a table at index 3.
        mode = unsafe {
            process_xdl_diff_opts(lstate, &mut cfg, &mut params, &mut linematch, &mut err)
        };
    }

    // Both of these are addressed by the callbacks through `ecb`, so they
    // have to outlive the `xdl_diff` call below and nothing may move them.
    let mut buf = luaL_Buffer {
        p: ptr::null_mut::<c_char>(),
        lvl: 0,
        L: ptr::null_mut::<lua_State>(),
        buffer: [0; 8192],
    };
    let mut ctx = HunkContext {
        lstate,
        err: &raw mut err,
        ma: &raw mut ma,
        mb: &raw mut mb,
        linematch,
        iwhite: params.flags & XDF_IGNORE_WHITESPACE > 0,
    };
    let mut ecb = xdemitcb_t {
        priv_0: ptr::null_mut::<c_void>(),
        out_hunk: None,
        out_line: None,
    };

    if err.type_0 == kErrorTypeNone {
        match mode {
            Mode::Unified => {
                // SAFETY: `lstate` is live and `buf` outlives the walk.
                unsafe { luaL_buffinit(lstate, &raw mut buf) };
                ecb.priv_0 = (&raw mut buf).cast::<c_void>();
                ecb.out_line = Some(write_string);
            }
            Mode::OnHunk => {
                cfg.hunk_func = Some(call_on_hunk_cb);
                ecb.priv_0 = (&raw mut ctx).cast::<c_void>();
            }
            Mode::Locations => {
                cfg.hunk_func = Some(hunk_locations_cb);
                ecb.priv_0 = (&raw mut ctx).cast::<c_void>();
                // The result list the callback appends to.
                // SAFETY: `lstate` is live.
                unsafe { lua_createtable(lstate, 0, 0) };
            }
        }

        // SAFETY: the four structures are live for the call, and each mode
        // wired up the callback that matches what `ecb.priv_0` points at.
        let failed = unsafe {
            xdl_diff(
                &raw mut ma,
                &raw mut mb,
                &raw mut params,
                &raw mut cfg,
                &raw mut ecb,
            ) == -1
        };
        if failed && err.type_0 == kErrorTypeNone {
            // SAFETY: `err` is this frame's.
            unsafe {
                api_set_error(
                    &raw mut err,
                    kErrorTypeException,
                    c"diff operation failed".as_ptr(),
                );
            }
        }
    }

    if err.type_0 != kErrorTypeNone {
        // SAFETY: `lstate` is live; `luaL_where` and the message become one
        // string, and `lua_error` raises it and does not return.
        return unsafe {
            luaL_where(lstate, 1);
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            lua_concat(lstate, 2);
            lua_error(lstate)
        };
    }
    match mode {
        // SAFETY: `luaL_buffinit` ran above, so the buffer is Lua's.
        Mode::Unified => {
            unsafe { luaL_pushresult(&raw mut buf) };
            1
        }
        Mode::Locations => 1,
        Mode::OnHunk => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both buffers consumed a line: the step is an unchanged line.
    const BOTH: c_int = COMPARED_BUFFER0 | COMPARED_BUFFER1;

    /// The hunks `walk_decisions` carves out of `hunk`, collected.
    fn walk(hunk: Hunk, decisions: &[c_int]) -> Vec<Hunk> {
        let mut hunks = Vec::new();
        walk_decisions(hunk, decisions, |finer| hunks.push(finer));
        hunks
    }

    /// A non-empty side is reported one-based; an empty one keeps its start,
    /// because a zero count means "before this line" rather than "at it".
    #[test]
    fn only_a_non_empty_side_shifts_to_one_based() {
        assert_eq!(Hunk::from_cb(3, 2, 7, 4).one_based(), [4, 2, 8, 4]);
        // A pure deletion: nothing in the second document.
        assert_eq!(Hunk::from_cb(3, 2, 7, 0).one_based(), [4, 2, 7, 0]);
        // A pure addition: nothing in the first.
        assert_eq!(Hunk::from_cb(3, 0, 7, 4).one_based(), [3, 0, 8, 4]);
        assert_eq!(Hunk::from_cb(0, 0, 0, 0).one_based(), [0, 0, 0, 0]);
    }

    /// A negative count cannot come out of `xdl_diff`, but the rule is
    /// `> 0` rather than `!= 0` and this is what says so.
    #[test]
    fn a_zero_count_is_the_only_thing_that_holds_the_start() {
        assert_eq!(Hunk::from_cb(5, 1, 5, 1).one_based(), [6, 1, 6, 1]);
        assert_eq!(Hunk::from_cb(5, 0, 5, 0).one_based(), [5, 0, 5, 0]);
    }

    /// No decisions still closes a hunk: `linematch` answering nothing
    /// leaves the empty hunk the walk started with, at the original start.
    #[test]
    fn an_empty_decision_run_still_emits_one_hunk() {
        assert_eq!(
            walk(Hunk::from_cb(4, 0, 9, 0), &[]),
            [Hunk::from_cb(4, 0, 9, 0)]
        );
    }

    /// One run of one decision is one hunk, and the counts are the run's
    /// length in whichever documents it consumed from.
    #[test]
    fn one_run_is_one_hunk() {
        assert_eq!(
            walk(Hunk::from_cb(0, 3, 0, 3), &[BOTH; 3]),
            [Hunk::from_cb(0, 3, 0, 3)]
        );
        assert_eq!(
            walk(Hunk::from_cb(0, 2, 0, 0), &[COMPARED_BUFFER0; 2]),
            [Hunk::from_cb(0, 2, 0, 0)]
        );
        assert_eq!(
            walk(Hunk::from_cb(0, 0, 0, 2), &[COMPARED_BUFFER1; 2]),
            [Hunk::from_cb(0, 0, 0, 2)]
        );
    }

    /// A change of decision closes a hunk and opens the next one where the
    /// closed one left the two line numbers -- which advance only in the
    /// document the decision consumed from.
    #[test]
    fn a_changed_decision_closes_the_hunk_at_the_lines_consumed() {
        let hunk = Hunk::from_cb(10, 2, 20, 2);
        let deletion_then_addition = [COMPARED_BUFFER0, COMPARED_BUFFER0, 2, 2];
        assert_eq!(
            walk(hunk, &deletion_then_addition),
            [Hunk::from_cb(10, 2, 20, 0), Hunk::from_cb(12, 0, 20, 2),]
        );
    }

    /// Three runs, each one step long: every hunk carries exactly the one
    /// line its decision consumed, and the starts chain through.
    #[test]
    fn every_alternation_is_its_own_hunk() {
        let alternating = [COMPARED_BUFFER0, COMPARED_BUFFER1, COMPARED_BUFFER0];
        assert_eq!(
            walk(Hunk::from_cb(0, 2, 0, 1), &alternating),
            [
                Hunk::from_cb(0, 1, 0, 0),
                Hunk::from_cb(1, 0, 0, 1),
                Hunk::from_cb(1, 1, 1, 0),
            ]
        );
    }

    /// The finer hunks cover the outer one exactly: the counts add up to
    /// what `xdl_diff` reported, and nothing overlaps.
    #[test]
    fn the_finer_hunks_tile_the_original() {
        let hunk = Hunk::from_cb(7, 3, 11, 3);
        let decisions = [BOTH, COMPARED_BUFFER0, COMPARED_BUFFER1, BOTH];
        let hunks = walk(hunk, &decisions);
        assert_eq!(hunks.iter().map(|h| h.count_a).sum::<c_int>(), hunk.count_a);
        assert_eq!(hunks.iter().map(|h| h.count_b).sum::<c_int>(), hunk.count_b);
        let (mut lnuma, mut lnumb) = (hunk.start_a, hunk.start_b);
        for finer in hunks {
            assert_eq!((finer.start_a, finer.start_b), (lnuma, lnumb));
            lnuma += finer.count_a;
            lnumb += finer.count_b;
        }
    }

    /// The keydict's optional-key bitfield is read bit by bit, and the
    /// initializer says nothing was given.
    #[test]
    fn an_optional_key_is_one_bit_of_the_keydict() {
        assert!((0..8).all(|bit| !is_set(&KEYDICT_INIT, bit)));
        // The six `KEYSET_OPTIDX_xdl_diff__*` values `apply_opts` names.
        for optidx in 1..=6 {
            let mut opts = KEYDICT_INIT;
            opts.is_set__xdl_diff_ = 1 << optidx;
            assert!(is_set(&opts, optidx));
            assert!((0..8).filter(|&b| b != optidx).all(|b| !is_set(&opts, b)));
        }
        // Bit 0 is not one of the six, and reading it must not spill into
        // its neighbour.
        let mut opts = KEYDICT_INIT;
        opts.is_set__xdl_diff_ = 0b101;
        assert!(is_set(&opts, 0));
        assert!(!is_set(&opts, 1));
        assert!(is_set(&opts, 2));
    }
}
