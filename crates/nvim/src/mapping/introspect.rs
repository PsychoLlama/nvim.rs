//! Reporting mappings to Vimscript and to the API.
//!
//! [`mapblock_fill_dict`] renders one [`mapblock_T`] as the twenty-key dict
//! that `maparg()`, `maplist()` and `nvim_get_keymap` all answer with;
//! [`get_maparg`] backs `maparg()`/`mapcheck()` and [`keymap_array`] backs
//! `nvim_get_keymap`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::kvec::InitVec;
use crate::types::builders::static_cstring;
use crate::types::{NUL, VAR_DICT, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, kListLenUnknown};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Size of the scratch buffer `tv_get_string_buf` may answer with.
const NUMBUFLEN: usize = 65;

/// `hasmapto()`: whether any mapping in the named modes has `{name}` in its
/// RHS.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_hasmapto(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe {
        let name = numbuf.string(argvars);
        let mut buf = [0 as c_char; NUMBUFLEN];
        let mut abbr = false;
        let mode = if (*argvars.add(1)).v_type == VAR_UNKNOWN as _ {
            c"nvo".as_ptr()
        } else {
            let mode = tv_get_string_buf(argvars.add(1), buf.as_mut_ptr());
            if (*argvars.add(2)).v_type != VAR_UNKNOWN as _ {
                abbr = tv_get_number(argvars.add(2)) != 0;
            }
            mode
        };
        (*rettv).vval.v_number = varnumber_T::from(map_to_exists(name, mode, abbr));
    }
}

/// C's `PUT_C`: append `key: value` to a dict whose storage `arena_dict` has
/// already reserved.
///
/// # Safety
/// `dict` must have room for one more entry.
unsafe fn put(dict: &mut Dict, key: &'static CStr, value: Object) {
    unsafe {
        dict.items.add(dict.size).write(key_value_pair {
            key: static_cstring(key),
            value,
        });
        dict.size += 1;
    }
}

/// How many entries [`mapblock_fill_dict`] can write.
const MAPARG_DICT_KEYS: size_t = 20;

/// Render `mp` as the dict `maparg()`, `maplist()` and `nvim_get_keymap` all
/// answer with.
///
/// `compatible` selects the old `maparg()` shape: the RHS comes back as the
/// text the user typed rather than as a `str2special` rendering, `<script>`
/// is not told apart from `:noremap`, and there is no `"buf"` key.
/// `lhsrawalt` is the other spelling of a simplified LHS, or null.
///
/// # Safety
/// `mp` must be a live mapblock and `arena` a live arena.
pub(crate) unsafe fn mapblock_fill_dict(
    mp: *const mapblock_T,
    lhsrawalt: *const c_char,
    buffer_value: c_int,
    abbr: bool,
    compatible: bool,
    arena: *mut Arena,
) -> Dict {
    unsafe {
        let mut dict = arena_dict(arena, MAPARG_DICT_KEYS);
        let lhs = str2special_arena((*mp).m_keys, compatible, !compatible, arena);
        let mapmode: *mut c_char = arena_alloc(arena, 7, false).cast();
        mapmode.copy_from_nonoverlapping(map_mode_to_chars((*mp).m_mode).as_ptr(), 7);

        let noremap_value = if compatible {
            // Keep the old compatible behaviour, which cannot tell a
            // <script> mapping apart.
            c_int::from((*mp).m_noremap != 0)
        } else if (*mp).m_noremap == REMAP_SCRIPT {
            2
        } else {
            c_int::from((*mp).m_noremap != 0)
        };

        if (*mp).m_luaref != LUA_NOREF {
            put(
                &mut dict,
                c"callback",
                Object {
                    type_0: kObjectTypeLuaRef,
                    data: object_data {
                        luaref: api_new_luaref((*mp).m_luaref),
                    },
                },
            );
        } else {
            let rhs = cstr_as_string(if compatible {
                (*mp).m_orig_str
            } else {
                str2special_arena((*mp).m_str, false, true, arena)
            });
            put(&mut dict, c"rhs", Object::string(rhs));
        }
        if !(*mp).m_desc.is_null() {
            put(
                &mut dict,
                c"desc",
                Object::string(cstr_as_string((*mp).m_desc)),
            );
        }
        put(&mut dict, c"lhs", Object::string(cstr_as_string(lhs)));
        put(
            &mut dict,
            c"lhsraw",
            Object::string(cstr_as_string((*mp).m_keys)),
        );
        if !lhsrawalt.is_null() {
            // Also add the value for the simplified entry.
            put(
                &mut dict,
                c"lhsrawalt",
                Object::string(cstr_as_string(lhsrawalt)),
            );
        }
        put(&mut dict, c"noremap", Object::integer(noremap_value.into()));
        put(
            &mut dict,
            c"script",
            Object::integer(Integer::from((*mp).m_noremap == REMAP_SCRIPT)),
        );
        put(
            &mut dict,
            c"expr",
            Object::integer(Integer::from((*mp).m_expr != 0)),
        );
        put(
            &mut dict,
            c"silent",
            Object::integer(Integer::from((*mp).m_silent != 0)),
        );
        put(
            &mut dict,
            c"sid",
            Object::integer((*mp).m_script_ctx.sc_sid.into()),
        );
        put(&mut dict, c"scriptversion", Object::integer(1));
        put(
            &mut dict,
            c"lnum",
            Object::integer((*mp).m_script_ctx.sc_lnum.into()),
        );
        put(&mut dict, c"buffer", Object::integer(buffer_value.into()));
        if !compatible {
            put(&mut dict, c"buf", Object::integer(buffer_value.into()));
        }
        put(
            &mut dict,
            c"nowait",
            Object::integer(Integer::from((*mp).m_nowait != 0)),
        );
        put(
            &mut dict,
            c"replace_keycodes",
            Object::integer(Integer::from((*mp).m_replace_keycodes)),
        );
        put(&mut dict, c"mode", Object::string(cstr_as_string(mapmode)));
        put(&mut dict, c"abbr", Object::integer(Integer::from(abbr)));
        put(
            &mut dict,
            c"mode_bits",
            Object::integer((*mp).m_mode.into()),
        );

        dict
    }
}

/// The body of `maparg()` and `mapcheck()`: `exact` is what tells them apart.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
unsafe fn get_maparg(argvars: *mut typval_T, rettv: *mut typval_T, exact: bool) {
    let mut numbuf = NumBuf::new();
    unsafe {
        // Return an empty string on failure.
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();

        let keys = numbuf.string(argvars).cast_mut();
        if c_int::from(*keys) == NUL {
            return;
        }

        let mut buf = [0 as c_char; NUMBUFLEN];
        let mut abbr = false;
        let mut get_dict = false;
        let mut which: *mut c_char = if (*argvars.add(1)).v_type != VAR_UNKNOWN as _ {
            let which = tv_get_string_buf_chk(argvars.add(1), buf.as_mut_ptr());
            if (*argvars.add(2)).v_type != VAR_UNKNOWN as _ {
                abbr = tv_get_number(argvars.add(2)) != 0;
                if (*argvars.add(3)).v_type != VAR_UNKNOWN as _ {
                    get_dict = tv_get_number(argvars.add(3)) != 0;
                }
            }
            which.cast_mut()
        } else {
            c"".as_ptr().cast_mut()
        };
        if which.is_null() {
            return;
        }

        let mut keys_buf: *mut c_char = ptr::null_mut();
        let mut alt_keys_buf: *mut c_char = ptr::null_mut();
        let mut did_simplify = false;
        let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
        let mode = get_map_mode(&raw mut which, false);

        let keys_simplified = replace_termcodes(
            keys,
            strlen(keys),
            &raw mut keys_buf,
            0,
            flags,
            &raw mut did_simplify,
            p_cpo.get(),
        );
        let mut found = check_map(keys_simplified, mode, exact, false, abbr);
        if did_simplify {
            // When the lhs is being simplified the not-simplified keys are
            // preferred for printing, like in do_map(). Upstream leaves the
            // previous `mp` in place when this second look-up fails, but it
            // clears both `rhs` and `rhs_lua`, and every reader of `mp` is
            // behind a test on one of those -- so dropping the whole match
            // is the same answer.
            replace_termcodes(
                keys,
                strlen(keys),
                &raw mut alt_keys_buf,
                0,
                flags | REPTERM_NO_SIMPLIFY as c_int,
                ptr::null_mut(),
                p_cpo.get(),
            );
            found = check_map(alt_keys_buf, mode, exact, false, abbr);
        }

        if !get_dict {
            // Return a string.
            if let Some(found) = &found {
                if !found.rhs.is_null() {
                    (*rettv).vval.v_string = if c_int::from(*found.rhs) == NUL {
                        xstrdup(c"<Nop>".as_ptr())
                    } else {
                        str2special_save(found.rhs, false, false)
                    };
                } else if found.rhs_lua != LUA_NOREF {
                    (*rettv).vval.v_string =
                        nlua_funcref_str((*found.mp).m_luaref, ptr::null_mut());
                }
            }
        } else if let Some(found) = found.filter(|f| !f.rhs.is_null() || f.rhs_lua != LUA_NOREF) {
            // Return a dictionary.
            let mut arena = ARENA_EMPTY;
            let dict = mapblock_fill_dict(
                found.mp,
                if did_simplify {
                    keys_simplified
                } else {
                    ptr::null_mut()
                },
                c_int::from(found.local),
                abbr,
                true,
                &raw mut arena,
            );
            let mut obj = Object::dict(dict);
            object_to_vim_take_luaref(&raw mut obj, rettv, true, ptr::null_mut());
            arena_mem_free(arena_finish(&raw mut arena));
        } else {
            // Return an empty dictionary.
            tv_dict_alloc_ret(rettv);
        }

        xfree(keys_buf.cast());
        xfree(alt_keys_buf.cast());
    }
}

/// `maplist()`: every mapping, global then buffer-local, as a list of dicts.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_maplist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
        let abbr = (*argvars).v_type != VAR_UNKNOWN as _ && tv_get_bool(argvars) != 0;

        tv_list_alloc_ret(rettv, kListLenUnknown as ptrdiff_t);

        // Do it twice: once for global maps and once for local maps.
        for (buffer_local, table) in [(0, MapTable::Global), (1, MapTable::Buffer(curbuf.get()))] {
            map_walk::<()>(table, abbr, |mp| {
                if (*mp).m_simplified != 0 {
                    return None;
                }
                let mut keys_buf: *mut c_char = ptr::null_mut();
                let mut did_simplify = false;

                let mut arena = ARENA_EMPTY;
                let lhs = str2special_arena((*mp).m_keys, true, false, &raw mut arena);
                replace_termcodes(
                    lhs,
                    strlen(lhs),
                    &raw mut keys_buf,
                    0,
                    flags,
                    &raw mut did_simplify,
                    p_cpo.get(),
                );

                let dict = mapblock_fill_dict(
                    mp,
                    if did_simplify {
                        keys_buf
                    } else {
                        ptr::null_mut()
                    },
                    buffer_local,
                    abbr,
                    true,
                    &raw mut arena,
                );
                let mut d = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                let mut obj = Object::dict(dict);
                object_to_vim_take_luaref(&raw mut obj, &raw mut d, true, ptr::null_mut());
                debug_assert_eq!(d.v_type, VAR_DICT);
                tv_list_append_dict((*rettv).vval.v_list, d.vval.v_dict);
                arena_mem_free(arena_finish(&raw mut arena));
                xfree(keys_buf.cast());
                None
            });
        }
    }
}

/// `maparg()`.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_maparg(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_maparg(argvars, rettv, true) }
}

/// `mapcheck()`.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_mapcheck(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_maparg(argvars, rettv, false) }
}

/// The mode a mode-shortname string names, and how much of it was consumed.
///
/// `nvim_get_keymap` and `nvim_set_keymap` take the same shortnames as
/// `:map`'s command names, optionally with a `!` in front and an `a` after
/// for the abbreviation form.
///
/// # Safety
/// `mode` must be a live API string.
pub(crate) unsafe fn parse_shortname_mode(mode: String_0) -> (c_int, bool, *mut c_char) {
    unsafe {
        let mut p = if !mode.is_empty() {
            mode.data()
        } else {
            c"m".as_ptr().cast_mut()
        };
        let forceit = c_int::from(*p) == c_int::from(b'!');
        let int_mode = get_map_mode(&raw mut p, forceit);
        if forceit {
            debug_assert_eq!(p, mode.data());
            p = p.add(1);
        }
        let is_abbrev = int_mode & (MODE_INSERT | MODE_CMDLINE) != 0 && *p == b'a' as c_char;
        (int_mode, is_abbrev, p)
    }
}

/// Every mapping in `mode`, as `maparg()`-like dicts: `nvim_get_keymap`.
///
/// `buf` is the buffer whose local mappings to report, or null for the
/// global ones.
///
/// # Safety
/// `arena` must be live and `buf` null or a live buffer.
pub unsafe fn keymap_array(mode: String_0, buf: *mut buf_T, arena: *mut Arena) -> Array {
    unsafe {
        let (int_mode, is_abbrev, _) = parse_shortname_mode(mode);
        let buffer_value = if buf.is_null() {
            0
        } else {
            (*buf).handle as c_int
        };
        let table = if buf.is_null() {
            MapTable::Global
        } else {
            MapTable::Buffer(buf)
        };

        let mut mappings = ArrayBuilder {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
            init_array: [Object::NIL; 16],
        };
        {
            let mut items = InitVec::new(
                &mut mappings.size,
                &mut mappings.capacity,
                &mut mappings.items,
                &mut mappings.init_array,
            );
            items.init();
            map_walk::<()>(table, is_abbrev, |mp| {
                if (*mp).m_simplified != 0 || int_mode & (*mp).m_mode == 0 {
                    return None;
                }
                let dict = mapblock_fill_dict(
                    mp,
                    if (*mp).m_alt.is_null() {
                        ptr::null_mut()
                    } else {
                        (*(*mp).m_alt).m_keys
                    },
                    buffer_value,
                    is_abbrev,
                    false,
                    arena,
                );
                items.push(Object::dict(dict));
                None
            });
        }

        arena_take_arraybuilder(arena, &raw mut mappings)
    }
}
