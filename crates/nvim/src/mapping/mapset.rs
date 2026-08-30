//! Setting a mapping from a dict: `mapset()` and `nvim_set_keymap`.
//!
//! Both take an already-built description rather than a command line —
//! [`f_mapset`] a `maparg()` dict, [`modify_keymap`] an API keyset — and both
//! end in [`buf_do_map`] or [`map_add`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api_error;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{VAR_DICT, VAR_FUNC, kErrorTypeException, kErrorTypeValidation};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Size of the scratch buffer `tv_get_string_buf_chk` may answer with.
const NUMBUFLEN: usize = 65;

/// The two `nvim_set_keymap` validation messages that carry a quote, hoisted
/// out of the bodies that raise them.
const REQUIRES_EXPR: &CStr = c"\"replace_keycodes\" requires \"expr\"";

/// `mapset()`: replace a mapping from a `maparg()`-shaped dict.
///
/// Two call shapes: one dict argument carrying `"mode"` and `"abbr"` as well,
/// or a mode string, an abbreviation flag and the dict.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_mapset(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    if check_secure() {
        return;
    }

    let mut buf = [0 as c_char; NUMBUFLEN];
    let which: *const c_char;
    let is_abbr: bool;
    let d: *mut dict_T;

    // If the first argument is a dict, then that is the only one allowed.
    // SAFETY (this block): the Vimscript call convention — `argvars` is a live
    // argument vector running to a `VAR_UNKNOWN`, so every slot tested here is
    // there, and `buf` is the scratch `tv_get_string_buf_chk` may answer with.
    if unsafe { (*argvars).v_type } == VAR_DICT as _ {
        d = unsafe { (*argvars).vval.v_dict };
        // SAFETY: `d` is the dict just taken off the argument.
        let abbr = unsafe {
            which = numbuf.dict_string(d, c"mode".as_ptr());
            tv_dict_get_bool(d, c"abbr".as_ptr(), -1)
        };
        if which.is_null() || abbr < 0 {
            emsg(gettext(E_ENTRIES_MISSING_IN_MAPSET_DICT_ARGUMENT));
            return;
        }
        is_abbr = abbr != 0;
    } else {
        // SAFETY: as above.
        which = unsafe { tv_get_string_buf_chk(argvars, buf.as_mut_ptr()) };
        if which.is_null() {
            return;
        }
        // SAFETY: as above.
        is_abbr = unsafe { tv_get_bool(argvars.add(1)) } != 0;
        // SAFETY: as above.
        if unsafe { tv_check_for_dict_arg(argvars, 2) }.is_err() {
            return;
        }
        // SAFETY: `tv_check_for_dict_arg` just said slot 2 is a dict.
        d = unsafe { (*argvars.add(2)).vval.v_dict };
    }

    // SAFETY: `which` is a NUL-terminated mode string.
    let mode = unsafe { get_map_mode_string(which, is_abbr) };
    if mode == 0 {
        // SAFETY: a static format whose one conversion is `which`.
        let which = unsafe { c_str(which) };
        semsg!("E1276: Illegal map mode string: '{which}'");
        return;
    }

    // Get the values in the same order as get_maparg() writes them.
    // SAFETY: `d` is a live dict, and each `NumBuf` outlives the string it
    // lends back.
    let (lhs, lhsraw, lhsrawalt, mut orig_rhs) = unsafe {
        (
            numbuf2.dict_string(d, c"lhs".as_ptr()),
            numbuf3.dict_string(d, c"lhsraw".as_ptr()),
            numbuf4.dict_string(d, c"lhsrawalt".as_ptr()),
            numbuf5.dict_string(d, c"rhs".as_ptr()),
        )
    };
    let mut rhs_lua = LUA_NOREF;
    // SAFETY: as above; `callback_di` is null or one of `d`'s own items, and
    // `find_func` answers null or a live `ufunc_T`.
    unsafe {
        let key = c"callback".count_bytes() as _;
        let callback_di = tv_dict_find(d, c"callback".as_ptr(), key);
        if !callback_di.is_null() && (*callback_di).di_tv.v_type == VAR_FUNC as _ {
            let fp = find_func((*callback_di).di_tv.vval.v_string);
            if !fp.is_null() && (*fp).uf_flags & FC_LUAREF != 0 {
                rhs_lua = api_new_luaref((*fp).uf_luaref);
                orig_rhs = c"".as_ptr().cast_mut();
            }
        }
    }
    if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
        // SAFETY: a static NUL-terminated message, and `rhs_lua` is the
        // reference taken just above, if any.
        unsafe {
            emsg(gettext(E_ENTRIES_MISSING_IN_MAPSET_DICT_ARGUMENT));
            api_free_luaref(rhs_lua);
        }
        return;
    }

    // SAFETY: `d` is a live dict.
    let mut noremap = if unsafe { tv_dict_get_number(d, c"noremap".as_ptr()) } != 0 {
        REMAP_NONE
    } else {
        0
    };
    // SAFETY: as above.
    if unsafe { tv_dict_get_number(d, c"script".as_ptr()) } != 0 {
        noremap = REMAP_SCRIPT;
    }

    // Upstream's designated initialiser, which leaves everything it does not
    // name zeroed -- including `rhs_lua`, which `set_maparg_rhs` overwrites
    // below.  `MAP_ARGUMENTS_INIT` cannot stand in on its own: it seeds
    // `rhs_lua` with `LUA_NOREF`, which is not zero.
    let mut args = MapArguments {
        rhs_lua: 0,
        ..MAP_ARGUMENTS_INIT
    };
    // SAFETY: `d` is a live dict; the `desc` allocation becomes the mapping's.
    unsafe {
        args.expr = tv_dict_get_number(d, c"expr".as_ptr()) != 0;
        args.silent = tv_dict_get_number(d, c"silent".as_ptr()) != 0;
        args.nowait = tv_dict_get_number(d, c"nowait".as_ptr()) != 0;
        args.replace_keycodes = tv_dict_get_number(d, c"replace_keycodes".as_ptr()) != 0;
        args.desc = tv_dict_get_string_alloc(d, c"desc".as_ptr());
    }

    // SAFETY: as above.
    let (sid, lnum, buffer) = unsafe {
        (
            tv_dict_get_number(d, c"sid".as_ptr()) as scid_T,
            tv_dict_get_number(d, c"lnum".as_ptr()) as linenr_T,
            tv_dict_get_number(d, c"buffer".as_ptr()) != 0,
        )
    };
    // The dict's "mode" is not used past get_map_mode_string.

    let cpo = p_cpo.get();
    let parsed = &raw mut args;
    // SAFETY: `orig_rhs` is NUL-terminated and `args` this frame's own struct.
    unsafe {
        let rhs_len = cstr::bytes_at(orig_rhs).len();
        set_maparg_rhs(orig_rhs, rhs_len, rhs_lua, sid, cpo, parsed);
    }

    // SAFETY: `curbuf` is set from startup to exit; `&raw` reads nothing, and
    // both addresses come off the one cell pointer rather than off a `&mut`.
    let (buf_maps, buf_abbrs) = unsafe {
        let cur = curbuf.get();
        (
            (&raw mut (*cur).b_maphash).cast::<*mut mapblock_T>(),
            &raw mut (*cur).b_first_abbr,
        )
    };
    let map_table = if buffer { buf_maps } else { global_map_heads() };
    let abbr_table = if buffer {
        buf_abbrs
    } else {
        global_abbr_head()
    };

    // Delete any existing mapping for this lhs and mode.
    let mut unmap_args = MAP_ARGUMENTS_INIT;
    let unmap = &raw mut unmap_args;
    // SAFETY: `lhs` is NUL-terminated and `unmap_args` this frame's struct.
    unsafe {
        let lhs_len = cstr::bytes_at(lhs).len();
        set_maparg_lhs_rhs(lhs, lhs_len, c"".as_ptr(), 0, LUA_NOREF, cpo, unmap);
    }
    unmap_args.buffer = buffer;
    let unmap_lhs = MAPTYPE_UNMAP_LHS as c_int;
    // SAFETY: `curbuf` is live, and `unmap_args` is this frame's struct.
    unsafe {
        let cur = Buf::current();
        buf_do_map(unmap_lhs, unmap, mode, is_abbr, cur);
        xfree(unmap_args.rhs.cast());
        xfree(unmap_args.orig_rhs.cast());
    }

    let mut mp_result: [*mut mapblock_T; 2] = [ptr::null_mut(); 2];
    // SAFETY: `curbuf` is set from startup to exit.
    let cur = unsafe { Buf::current() };
    let add = |keys, simplified| {
        // SAFETY: both tables name live storage, `keys` is one of the dict's
        // NUL-terminated LHS strings, and `args` is this frame's struct.
        unsafe {
            let (m, a) = (map_table, abbr_table);
            map_add(
                cur, m, a, keys, parsed, noremap, mode, is_abbr, sid, lnum, simplified,
            )
        }
    };
    mp_result[0] = add(lhsraw, false);
    if !lhsrawalt.is_null() {
        mp_result[1] = add(lhsrawalt, true);
    }

    if !mp_result[0].is_null() && !mp_result[1].is_null() {
        // SAFETY: both are entries `map_add` just linked into a live table.
        unsafe {
            (*mp_result[0]).m_alt = mp_result[1];
            (*mp_result[1]).m_alt = mp_result[0];
        }
    }
}

/// Set, tweak or remove a mapping in a mode: the implementation behind
/// `nvim_set_keymap`, `nvim_del_keymap` and their `buf_` variants.
///
/// `buffer` is a buffer handle, 0 for the current buffer, or -1 for "all
/// buffers", i.e. the global tables.  `is_unmap` removes the mapping matching
/// `lhs` instead of adding one.
///
/// # Safety
/// Every pointer argument must be live; `err` is written on failure.
#[allow(clippy::too_many_arguments)] // the API dispatcher's own signature
pub unsafe fn modify_keymap(
    channel_id: uint64_t,
    mut buffer: Buffer,
    is_unmap: bool,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: *mut KeyDict_keymap,
    err: &mut Error,
) {
    let mut lua_funcref = LUA_NOREF;
    let global = buffer == -1;
    if global {
        buffer = 0;
    }
    // SAFETY: the caller's promise -- `err` is a live, writable error slot.
    let target_buf = unsafe { find_buffer_by_handle(buffer, err) };
    if target_buf.is_null() {
        return;
    }

    // The guard restores the previous script context when it is dropped
    // below.
    let sctx = api_set_sctx(channel_id);

    let mut parsed_args = MAP_ARGUMENTS_INIT;
    if !opts.is_null() {
        // SAFETY: the caller's promise -- a non-null `opts` is a live keyset,
        // whose `desc` string this copies and whose `callback` it takes over.
        unsafe {
            let mut o = Live::new(opts);
            parsed_args.nowait = o.nowait;
            parsed_args.noremap = o.noremap;
            parsed_args.silent = o.silent;
            parsed_args.script = o.script;
            parsed_args.expr = o.expr;
            parsed_args.unique = o.unique;
            parsed_args.replace_keycodes = o.replace_keycodes;
            if o.is_set__keymap_ & 1 << KEYSET_OPTIDX_keymap__callback != 0 {
                lua_funcref = o.callback;
                o.callback = LUA_NOREF;
            }
            if o.is_set__keymap_ & 1 << KEYSET_OPTIDX_keymap__desc != 0 {
                parsed_args.desc = string_to_cstr(o.desc);
            }
        }
    }
    parsed_args.buffer = !global;

    'fail_and_free: {
        if parsed_args.replace_keycodes && !parsed_args.expr {
            *err = Error::validation(REQUIRES_EXPR);
            break 'fail_and_free;
        }

        let parsed = &raw mut parsed_args;
        let cpo = p_cpo.get();
        // SAFETY: `lhs` and `rhs` are live API strings, and `parsed_args` is
        // this frame's own struct.
        let ok = unsafe {
            let (l, ll) = (lhs.data(), lhs.len());
            let (r, rl) = (rhs.data(), rhs.len());
            set_maparg_lhs_rhs(l, ll, r, rl, lua_funcref, cpo, parsed)
        };
        if !ok
            || parsed_args.lhs_len > MAXMAPLEN as size_t
            || parsed_args.alt_lhs_len > MAXMAPLEN as size_t
        {
            // SAFETY: `lhs` is a live API string.
            let lhs = unsafe { c_str(lhs.data()) };
            *err = api_error!(
                kErrorTypeValidation,
                "LHS exceeds maximum map length: {lhs}"
            );
            break 'fail_and_free;
        }

        // SAFETY: `mode` is a live API string.
        let (mode_val, is_abbrev, mut p) = unsafe { parse_shortname_mode(mode) };
        if is_abbrev {
            // SAFETY: `parse_shortname_mode` left `p` on the `a` it found.
            p = unsafe { p.add(1) };
        }
        // SAFETY: `p` walks `mode`'s own bytes, so the difference is how much
        // of it was consumed.
        let consumed = unsafe { p.offset_from(mode.data()) } as size_t;
        if !mode.is_empty() && consumed != mode.len() {
            // SAFETY: `mode` is a live API string.
            let mode = unsafe { c_str(mode.data()) };
            *err = api_error!(kErrorTypeValidation, "Invalid mode shortname: \"{mode}\"");
            break 'fail_and_free;
        }
        if parsed_args.lhs_len == 0 {
            *err = Error::validation(c"Invalid (empty) LHS");
            break 'fail_and_free;
        }

        let is_noremap = parsed_args.noremap;
        debug_assert!(!(is_unmap && is_noremap));

        if !is_unmap
            && lua_funcref == LUA_NOREF
            && parsed_args.rhs_len == 0
            && !parsed_args.rhs_is_noop
        {
            if rhs.is_empty() {
                // Assume the caller wants the RHS to be a <Nop>.
                parsed_args.rhs_is_noop = true;
            } else {
                // SAFETY: `abort` never returns and reads nothing of ours.
                unsafe { abort() }; // should never happen
            }
        } else if is_unmap && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF) {
            // SAFETY: `parsed_args.rhs` is this frame's own NUL-terminated
            // string, and `err` the caller's slot.
            unsafe {
                *err = if parsed_args.rhs_len != 0 {
                    let rhs = c_str(parsed_args.rhs);
                    api_error!(
                        kErrorTypeValidation,
                        "Gave nonempty RHS in unmap command: {rhs}"
                    )
                } else {
                    Error::validation(c"Gave nonempty RHS for unmap")
                };
            }
            break 'fail_and_free;
        }

        // buf_do_map() reads noremap/unmap as its own argument.
        let maptype_val = if is_unmap {
            MAPTYPE_UNMAP as c_int
        } else if is_noremap {
            MAPTYPE_NOREMAP as c_int
        } else {
            MAPTYPE_MAP as c_int
        };

        // SAFETY: `target_buf` is the live buffer `find_buffer_by_handle`
        // answered, and `parsed_args` is this frame's struct.
        let answer = unsafe {
            let target = Buf::new(target_buf);
            buf_do_map(maptype_val, parsed, mode_val, is_abbrev, target)
        };
        // The four "already exists" texts hold a `%s`, so their literals are
        // written out here rather than shared with `domap`'s copies, which
        // still hand them to a `printf`.
        // SAFETY: `lhs` is a live API string.
        let lhs = unsafe { c_str(lhs.data()) };
        let refused = match (answer, is_abbrev) {
            (1, _) => Some(Error::exception(e_invarg)),
            (2, _) => Some(Error::exception(e_nomap)),
            (5, true) => Some(api_error!(
                kErrorTypeException,
                "E226: Abbreviation already exists for {lhs}"
            )),
            (5, false) => Some(api_error!(
                kErrorTypeException,
                "E227: Mapping already exists for {lhs}"
            )),
            (6, true) => Some(api_error!(
                kErrorTypeException,
                "E224: Global abbreviation already exists for {lhs}"
            )),
            (6, false) => Some(api_error!(
                kErrorTypeException,
                "E225: Global mapping already exists for {lhs}"
            )),
            _ => None,
        };
        if let Some(e) = refused {
            *err = e;
        }
    }

    drop(sctx);
    if parsed_args.rhs_lua != LUA_NOREF {
        // SAFETY: a reference this frame took over and nothing else owns.
        unsafe { api_free_luaref(parsed_args.rhs_lua) };
        parsed_args.rhs_lua = LUA_NOREF;
    }
    // SAFETY: the three allocations this frame made; whatever a mapblock took
    // ownership of was nulled out by `buf_do_map`.
    unsafe {
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
        xfree(parsed_args.desc.cast());
    }
}
