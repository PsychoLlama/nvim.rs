//! Setting a mapping from a dict: `mapset()` and `nvim_set_keymap`.
//!
//! Both take an already-built description rather than a command line —
//! [`f_mapset`] a `maparg()` dict, [`modify_keymap`] an API keyset — and both
//! end in [`buf_do_map`] or [`map_add`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, VAR_DICT, VAR_FUNC, kErrorTypeException, kErrorTypeValidation};
use core::ffi::{c_char, c_int};
use core::ptr;

/// Size of the scratch buffer `tv_get_string_buf_chk` may answer with.
const NUMBUFLEN: usize = 65;

/// `mapset()`: replace a mapping from a `maparg()`-shaped dict.
///
/// Two call shapes: one dict argument carrying `"mode"` and `"abbr"` as well,
/// or a mode string, an abbreviation flag and the dict.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_mapset(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if check_secure() {
            return;
        }

        let mut buf = [0 as c_char; NUMBUFLEN];
        let which: *const c_char;
        let is_abbr: bool;
        let d: *mut dict_T;

        // If the first argument is a dict, then that is the only one allowed.
        if (*argvars).v_type == VAR_DICT as _ {
            d = (*argvars).vval.v_dict;
            which = tv_dict_get_string(d, c"mode".as_ptr(), false);
            let abbr = tv_dict_get_bool(d, c"abbr".as_ptr(), -1);
            if which.is_null() || abbr < 0 {
                emsg(gettext(E_ENTRIES_MISSING_IN_MAPSET_DICT_ARGUMENT.as_ptr()));
                return;
            }
            is_abbr = abbr != 0;
        } else {
            which = tv_get_string_buf_chk(argvars, buf.as_mut_ptr());
            if which.is_null() {
                return;
            }
            is_abbr = tv_get_bool(argvars.add(1)) != 0;
            if tv_check_for_dict_arg(argvars, 2) == FAIL {
                return;
            }
            d = (*argvars.add(2)).vval.v_dict;
        }

        let mode = get_map_mode_string(which, is_abbr);
        if mode == 0 {
            semsg_c!(gettext(E_ILLEGAL_MAP_MODE_STRING_STR.as_ptr()), which);
            return;
        }

        // Get the values in the same order as get_maparg() writes them.
        let lhs = tv_dict_get_string(d, c"lhs".as_ptr(), false);
        let lhsraw = tv_dict_get_string(d, c"lhsraw".as_ptr(), false);
        let lhsrawalt = tv_dict_get_string(d, c"lhsrawalt".as_ptr(), false);
        let mut orig_rhs = tv_dict_get_string(d, c"rhs".as_ptr(), false);
        let mut rhs_lua = LUA_NOREF;
        let callback_di = tv_dict_find(d, c"callback".as_ptr(), c"callback".count_bytes() as _);
        if !callback_di.is_null() && (*callback_di).di_tv.v_type == VAR_FUNC as _ {
            let fp = find_func((*callback_di).di_tv.vval.v_string);
            if !fp.is_null() && (*fp).uf_flags & FC_LUAREF != 0 {
                rhs_lua = api_new_luaref((*fp).uf_luaref);
                orig_rhs = c"".as_ptr().cast_mut();
            }
        }
        if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
            emsg(gettext(E_ENTRIES_MISSING_IN_MAPSET_DICT_ARGUMENT.as_ptr()));
            api_free_luaref(rhs_lua);
            return;
        }

        let mut noremap = if tv_dict_get_number(d, c"noremap".as_ptr()) != 0 {
            REMAP_NONE
        } else {
            0
        };
        if tv_dict_get_number(d, c"script".as_ptr()) != 0 {
            noremap = REMAP_SCRIPT;
        }

        // Upstream's designated initialiser, which leaves everything it does
        // not name zeroed -- including `rhs_lua`, which `set_maparg_rhs`
        // overwrites below.
        let mut args: MapArguments = core::mem::zeroed();
        args.expr = tv_dict_get_number(d, c"expr".as_ptr()) != 0;
        args.silent = tv_dict_get_number(d, c"silent".as_ptr()) != 0;
        args.nowait = tv_dict_get_number(d, c"nowait".as_ptr()) != 0;
        args.replace_keycodes = tv_dict_get_number(d, c"replace_keycodes".as_ptr()) != 0;
        args.desc = tv_dict_get_string(d, c"desc".as_ptr(), true);

        let sid = tv_dict_get_number(d, c"sid".as_ptr()) as scid_T;
        let lnum = tv_dict_get_number(d, c"lnum".as_ptr()) as linenr_T;
        let buffer = tv_dict_get_number(d, c"buffer".as_ptr()) != 0;
        // The dict's "mode" is not used past get_map_mode_string.

        set_maparg_rhs(
            orig_rhs,
            strlen(orig_rhs),
            rhs_lua,
            sid,
            p_cpo.get(),
            &raw mut args,
        );

        let map_table: *mut *mut mapblock_T = if buffer {
            (&raw mut (*curbuf.get()).b_maphash).cast()
        } else {
            MAPHASH.ptr().cast()
        };
        let abbr_table: *mut *mut mapblock_T = if buffer {
            &raw mut (*curbuf.get()).b_first_abbr
        } else {
            FIRST_ABBR.ptr()
        };

        // Delete any existing mapping for this lhs and mode.
        let mut unmap_args = MAP_ARGUMENTS_INIT;
        set_maparg_lhs_rhs(
            lhs,
            strlen(lhs),
            c"".as_ptr(),
            0,
            LUA_NOREF,
            p_cpo.get(),
            &raw mut unmap_args,
        );
        unmap_args.buffer = buffer;
        buf_do_map(
            MAPTYPE_UNMAP_LHS as c_int,
            &raw mut unmap_args,
            mode,
            is_abbr,
            curbuf.get(),
        );
        xfree(unmap_args.rhs.cast());
        xfree(unmap_args.orig_rhs.cast());

        let mut mp_result: [*mut mapblock_T; 2] = [ptr::null_mut(); 2];
        mp_result[0] = map_add(
            curbuf.get(),
            map_table,
            abbr_table,
            lhsraw,
            &raw mut args,
            noremap,
            mode,
            is_abbr,
            sid,
            lnum,
            false,
        );
        if !lhsrawalt.is_null() {
            mp_result[1] = map_add(
                curbuf.get(),
                map_table,
                abbr_table,
                lhsrawalt,
                &raw mut args,
                noremap,
                mode,
                is_abbr,
                sid,
                lnum,
                true,
            );
        }

        if !mp_result[0].is_null() && !mp_result[1].is_null() {
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
    err: *mut Error,
) {
    unsafe {
        let mut lua_funcref = LUA_NOREF;
        let global = buffer == -1;
        if global {
            buffer = 0;
        }
        let target_buf = find_buffer_by_handle(buffer, err);
        if target_buf.is_null() {
            return;
        }

        let save_current_sctx = api_set_sctx(channel_id);

        let mut parsed_args = MAP_ARGUMENTS_INIT;
        if !opts.is_null() {
            parsed_args.nowait = (*opts).nowait;
            parsed_args.noremap = (*opts).noremap;
            parsed_args.silent = (*opts).silent;
            parsed_args.script = (*opts).script;
            parsed_args.expr = (*opts).expr;
            parsed_args.unique = (*opts).unique;
            parsed_args.replace_keycodes = (*opts).replace_keycodes;
            if (*opts).is_set__keymap_ & 1 << KEYSET_OPTIDX_keymap__callback != 0 {
                lua_funcref = (*opts).callback;
                (*opts).callback = LUA_NOREF;
            }
            if (*opts).is_set__keymap_ & 1 << KEYSET_OPTIDX_keymap__desc != 0 {
                parsed_args.desc = string_to_cstr((*opts).desc);
            }
        }
        parsed_args.buffer = !global;

        'fail_and_free: {
            if parsed_args.replace_keycodes && !parsed_args.expr {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"\"replace_keycodes\" requires \"expr\"".as_ptr(),
                );
                break 'fail_and_free;
            }

            if !set_maparg_lhs_rhs(
                lhs.data,
                lhs.size,
                rhs.data,
                rhs.size,
                lua_funcref,
                p_cpo.get(),
                &raw mut parsed_args,
            ) || parsed_args.lhs_len > MAXMAPLEN as size_t
                || parsed_args.alt_lhs_len > MAXMAPLEN as size_t
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"LHS exceeds maximum map length: %s".as_ptr(),
                    lhs.data,
                );
                break 'fail_and_free;
            }

            let (mode_val, is_abbrev, mut p) = parse_shortname_mode(mode);
            if is_abbrev {
                p = p.add(1);
            }
            if mode.size > 0 && p.offset_from(mode.data) as size_t != mode.size {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Invalid mode shortname: \"%s\"".as_ptr(),
                    mode.data,
                );
                break 'fail_and_free;
            }
            if parsed_args.lhs_len == 0 {
                api_set_error(err, kErrorTypeValidation, c"Invalid (empty) LHS".as_ptr());
                break 'fail_and_free;
            }

            let is_noremap = parsed_args.noremap;
            debug_assert!(!(is_unmap && is_noremap));

            if !is_unmap
                && lua_funcref == LUA_NOREF
                && parsed_args.rhs_len == 0
                && !parsed_args.rhs_is_noop
            {
                if rhs.size == 0 {
                    // Assume the caller wants the RHS to be a <Nop>.
                    parsed_args.rhs_is_noop = true;
                } else {
                    abort(); // should never happen
                }
            } else if is_unmap && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF) {
                if parsed_args.rhs_len != 0 {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Gave nonempty RHS in unmap command: %s".as_ptr(),
                        parsed_args.rhs,
                    );
                } else {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Gave nonempty RHS for unmap".as_ptr(),
                    );
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

            match buf_do_map(
                maptype_val,
                &raw mut parsed_args,
                mode_val,
                is_abbrev,
                target_buf,
            ) {
                1 => api_set_error(
                    err,
                    kErrorTypeException,
                    (&raw const e_invarg).cast::<c_char>(),
                    0,
                ),
                2 => api_set_error(
                    err,
                    kErrorTypeException,
                    (&raw const e_nomap).cast::<c_char>(),
                    0,
                ),
                5 => api_set_error(
                    err,
                    kErrorTypeException,
                    if is_abbrev {
                        E_ABBREVIATION_ALREADY_EXISTS_FOR_STR.as_ptr()
                    } else {
                        E_MAPPING_ALREADY_EXISTS_FOR_STR.as_ptr()
                    },
                    lhs.data,
                ),
                6 => api_set_error(
                    err,
                    kErrorTypeException,
                    if is_abbrev {
                        E_GLOBAL_ABBREVIATION_ALREADY_EXISTS_FOR_STR.as_ptr()
                    } else {
                        E_GLOBAL_MAPPING_ALREADY_EXISTS_FOR_STR.as_ptr()
                    },
                    lhs.data,
                ),
                _ => {}
            }
        }

        current_sctx.set(save_current_sctx);
        if parsed_args.rhs_lua != LUA_NOREF {
            api_free_luaref(parsed_args.rhs_lua);
            parsed_args.rhs_lua = LUA_NOREF;
        }
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
        xfree(parsed_args.desc.cast());
    }
}
