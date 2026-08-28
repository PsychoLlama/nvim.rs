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
use crate::types::{NUL, VAR_DICT, VAR_STRING, VAR_UNKNOWN, VarLock, kListLenUnknown};
use crate::winlayer::Buf;
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
    let mut buf = [0 as c_char; NUMBUFLEN];
    let mut abbr = false;
    // SAFETY: the Vimscript call convention — `argvars` names at least the
    // three slots read here, terminated by a `VAR_UNKNOWN`, and `buf` is the
    // scratch `tv_get_string_buf` may answer with.
    let (name, mode) = unsafe {
        let name = numbuf.string(argvars);
        let mode = if (*argvars.add(1)).v_type == VAR_UNKNOWN as _ {
            c"nvo".as_ptr()
        } else {
            let mode = tv_get_string_buf(argvars.add(1), buf.as_mut_ptr());
            if (*argvars.add(2)).v_type != VAR_UNKNOWN as _ {
                abbr = tv_get_number(argvars.add(2)) != 0;
            }
            mode
        };
        (name, mode)
    };
    // SAFETY: both strings are NUL-terminated, and `rettv` is the caller's
    // writable answer slot.
    unsafe {
        let found = map_to_exists(name, mode, abbr);
        (*rettv).vval.v_number = varnumber_T::from(found);
    }
}

/// How many entries [`mapblock_fill_dict`] can write.
const MAPARG_DICT_KEYS: size_t = 20;

/// A dict being filled in, whose caller has promised the arena reserved room
/// for [`MAPARG_DICT_KEYS`] entries.
///
/// C spells each append `PUT_C`, which writes one past the end and bumps the
/// size with nothing checking that the reservation was big enough.  The
/// promise is made once, at construction; every [`Filling::put`] after it is
/// ordinary checked code, which is what makes the twenty appends below safe.
struct Filling(Dict);

impl Filling {
    /// # Safety
    /// `dict`'s storage must have room for [`MAPARG_DICT_KEYS`] entries.
    unsafe fn new(dict: Dict) -> Self {
        Self(dict)
    }

    /// C's `PUT_C`: append `key: value`.
    fn put(&mut self, key: &'static CStr, value: Object) {
        debug_assert!(self.0.size < MAPARG_DICT_KEYS);
        // SAFETY: the constructor's promise — room for `MAPARG_DICT_KEYS`
        // entries — and `size` counts the ones written so far.
        unsafe {
            self.0.items.add(self.0.size).write(key_value_pair {
                key: static_cstring(key),
                value,
            });
        }
        self.0.size += 1;
    }

    /// The finished dict.
    fn finish(self) -> Dict {
        self.0
    }
}

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
    mp: Mb,
    lhsrawalt: *const c_char,
    buffer_value: c_int,
    abbr: bool,
    compatible: bool,
    arena: *mut Arena,
) -> Dict {
    // SAFETY: the caller's promise — `arena` is live and `mp` a live mapblock,
    // so `m_keys` is its NUL-terminated LHS.  `arena_alloc` answers seven
    // writable bytes, which is the width `map_mode_to_chars` fills.
    let (dict, lhs, mapmode) = unsafe {
        let dict = arena_dict(arena, MAPARG_DICT_KEYS);
        let lhs = str2special_arena(mp.m_keys, compatible, !compatible, arena);
        let mapmode: *mut c_char = arena_alloc(arena, 7, false).cast();
        mapmode.copy_from_nonoverlapping(map_mode_to_chars(mp.m_mode).as_ptr(), 7);
        (dict, lhs, mapmode)
    };
    // SAFETY: `arena_dict` just reserved `MAPARG_DICT_KEYS` entries.
    let mut out = unsafe { Filling::new(dict) };

    let noremap_value = if compatible {
        // Keep the old compatible behaviour, which cannot tell a
        // <script> mapping apart.
        c_int::from(mp.m_noremap != 0)
    } else if mp.m_noremap == REMAP_SCRIPT {
        2
    } else {
        c_int::from(mp.m_noremap != 0)
    };

    if mp.m_luaref != LUA_NOREF {
        // SAFETY: the mapping's own reference, of which this takes a new one
        // for the caller to own.
        let luaref = unsafe { api_new_luaref(mp.m_luaref) };
        out.put(
            c"callback",
            Object {
                type_0: kObjectTypeLuaRef,
                data: object_data { luaref },
            },
        );
    } else {
        // SAFETY: `m_orig_str` and `m_str` are the mapping's own
        // NUL-terminated strings, and `arena` is live.
        let rhs = unsafe {
            cstr_as_string(if compatible {
                mp.m_orig_str
            } else {
                str2special_arena(mp.m_str, false, true, arena)
            })
        };
        out.put(c"rhs", Object::string(rhs));
    }
    if !mp.m_desc.is_null() {
        // SAFETY: a non-null `m_desc` is the mapping's own NUL-terminated text.
        let desc = unsafe { cstr_as_string(mp.m_desc) };
        out.put(c"desc", Object::string(desc));
    }
    // SAFETY: `lhs` is `str2special_arena`'s answer and `m_keys` the mapping's
    // own LHS; both are NUL-terminated.
    let (lhs, lhsraw) = unsafe { (cstr_as_string(lhs), cstr_as_string(mp.m_keys)) };
    out.put(c"lhs", Object::string(lhs));
    out.put(c"lhsraw", Object::string(lhsraw));
    if !lhsrawalt.is_null() {
        // Also add the value for the simplified entry.
        // SAFETY: the caller's promise — a NUL-terminated alternative LHS.
        let alt = unsafe { cstr_as_string(lhsrawalt) };
        out.put(c"lhsrawalt", Object::string(alt));
    }
    out.put(c"noremap", Object::integer(noremap_value.into()));
    out.put(
        c"script",
        Object::integer(Integer::from(mp.m_noremap == REMAP_SCRIPT)),
    );
    out.put(c"expr", Object::integer(Integer::from(mp.m_expr != 0)));
    out.put(c"silent", Object::integer(Integer::from(mp.m_silent != 0)));
    out.put(c"sid", Object::integer(mp.m_script_ctx.sc_sid.into()));
    out.put(c"scriptversion", Object::integer(1));
    out.put(c"lnum", Object::integer(mp.m_script_ctx.sc_lnum.into()));
    out.put(c"buffer", Object::integer(buffer_value.into()));
    if !compatible {
        out.put(c"buf", Object::integer(buffer_value.into()));
    }
    out.put(c"nowait", Object::integer(Integer::from(mp.m_nowait != 0)));
    out.put(
        c"replace_keycodes",
        Object::integer(Integer::from(mp.m_replace_keycodes)),
    );
    // SAFETY: `mapmode` is the seven-byte NUL-terminated copy made above.
    let mode = unsafe { cstr_as_string(mapmode) };
    out.put(c"mode", Object::string(mode));
    out.put(c"abbr", Object::integer(Integer::from(abbr)));
    out.put(c"mode_bits", Object::integer(mp.m_mode.into()));

    out.finish()
}

/// The body of `maparg()` and `mapcheck()`: `exact` is what tells them apart.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
unsafe fn get_maparg(argvars: *mut typval_T, rettv: *mut typval_T, exact: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise — `rettv` is the writable answer slot.
    let mut ret = unsafe { Live::new(rettv) };
    // Return an empty string on failure.
    ret.v_type = VAR_STRING;
    ret.vval.v_string = ptr::null_mut();

    // SAFETY: the Vimscript call convention — `argvars` is a live argument
    // vector whose first entry is the keys, NUL-terminated.
    let keys = unsafe { numbuf.string(argvars) }.cast_mut();
    // SAFETY: as above.
    if unsafe { c_int::from(*keys) } == NUL {
        return;
    }

    let mut buf = [0 as c_char; NUMBUFLEN];
    let mut abbr = false;
    let mut get_dict = false;
    // SAFETY: as above — the vector runs to a `VAR_UNKNOWN`, so every slot
    // tested here is there, and `buf` is `tv_get_string_buf_chk`'s scratch.
    let mut which: *mut c_char = unsafe {
        if (*argvars.add(1)).v_type != VAR_UNKNOWN as _ {
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
        }
    };
    if which.is_null() {
        return;
    }

    let mut keys_buf: *mut c_char = ptr::null_mut();
    let mut alt_keys_buf: *mut c_char = ptr::null_mut();
    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
    let nosimp = flags | REPTERM_NO_SIMPLIFY as c_int;
    let cpo = p_cpo.get();
    let plain = ptr::null_mut();
    let simplify = &raw mut did_simplify;
    let out = &raw mut keys_buf;
    let alt_out = &raw mut alt_keys_buf;
    // SAFETY: `which` is a NUL-terminated mode string.
    let mode = unsafe { get_map_mode(&raw mut which, false) };

    // SAFETY: `keys` is NUL-terminated, and both `*_buf` slots are locals that
    // outlive the calls; the allocations they take over are freed below.
    let (keys_simplified, mut found) = unsafe {
        let len = strlen(keys);
        let simplified = replace_termcodes(keys, len, out, 0, flags, simplify, cpo);
        (simplified, check_map(simplified, mode, exact, false, abbr))
    };
    if did_simplify {
        // When the lhs is being simplified the not-simplified keys are
        // preferred for printing, like in do_map(). Upstream leaves the
        // previous `mp` in place when this second look-up fails, but it
        // clears both `rhs` and `rhs_lua`, and every reader of `mp` is
        // behind a test on one of those -- so dropping the whole match
        // is the same answer.
        // SAFETY: as above.
        found = unsafe {
            let len = strlen(keys);
            replace_termcodes(keys, len, alt_out, 0, nosimp, plain, cpo);
            check_map(alt_keys_buf, mode, exact, false, abbr)
        };
    }

    if !get_dict {
        // Return a string.
        if let Some(found) = &found {
            if !found.rhs.is_null() {
                // SAFETY: `rhs` is the matching mapping's NUL-terminated RHS.
                ret.vval.v_string = unsafe {
                    if c_int::from(*found.rhs) == NUL {
                        xstrdup(c"<Nop>".as_ptr())
                    } else {
                        str2special_save(found.rhs, false, false)
                    }
                };
            } else if found.rhs_lua != LUA_NOREF {
                // SAFETY: `mp` is the matching mapping, still linked.
                ret.vval.v_string =
                    unsafe { nlua_funcref_str((*found.mp).m_luaref, ptr::null_mut()) };
            }
        }
    } else if let Some(found) = found.filter(|f| !f.rhs.is_null() || f.rhs_lua != LUA_NOREF) {
        // Return a dictionary.
        let mut arena = ARENA_EMPTY;
        let alt = if did_simplify {
            keys_simplified
        } else {
            ptr::null_mut()
        };
        // SAFETY: `found.mp` is still linked, `arena` is this frame's own, and
        // `rettv` is the caller's writable slot.
        unsafe {
            let mp = Mb::new(found.mp);
            let local = c_int::from(found.local);
            let dict = mapblock_fill_dict(mp, alt, local, abbr, true, &raw mut arena);
            let mut obj = Object::dict(dict);
            object_to_vim_take_luaref(&raw mut obj, rettv, true, ptr::null_mut());
            arena_mem_free(arena_finish(&raw mut arena));
        }
    } else {
        // Return an empty dictionary.
        // SAFETY: the caller's writable answer slot.
        unsafe { tv_dict_alloc_ret(rettv) };
    }

    // SAFETY: the two allocations `replace_termcodes` may have made above.
    unsafe {
        xfree(keys_buf.cast());
        xfree(alt_keys_buf.cast());
    }
}

/// `maplist()`: every mapping, global then buffer-local, as a list of dicts.
///
/// # Safety
/// The Vimscript call convention: `argvars` is a live argument vector.
pub unsafe fn f_maplist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
    let cpo = p_cpo.get();
    // SAFETY: the Vimscript call convention — `argvars` is a live argument
    // vector and `rettv` the writable answer slot.
    let abbr = unsafe { (*argvars).v_type != VAR_UNKNOWN as _ && tv_get_bool(argvars) != 0 };
    // SAFETY: as above.
    unsafe { tv_list_alloc_ret(rettv, kListLenUnknown as ptrdiff_t) };
    // SAFETY: `curbuf` is set from startup to exit.
    let cur = unsafe { Buf::current() };

    // Do it twice: once for global maps and once for local maps.
    for (buffer_local, table) in [(0, MapTable::Global), (1, MapTable::Buffer(cur))] {
        let collect = |mp: Mb| {
            if mp.m_simplified != 0 {
                return None;
            }
            let mut keys_buf: *mut c_char = ptr::null_mut();
            let mut did_simplify = false;
            let out = &raw mut keys_buf;
            let simplify = &raw mut did_simplify;

            let mut arena = ARENA_EMPTY;
            // SAFETY: `m_keys` is the mapping's own NUL-terminated LHS, and
            // `arena`, `keys_buf` and `did_simplify` are this frame's locals.
            let lhs = unsafe { str2special_arena(mp.m_keys, true, false, &raw mut arena) };
            // SAFETY: as above.
            unsafe {
                let len = strlen(lhs);
                replace_termcodes(lhs, len, out, 0, flags, simplify, cpo);
            }

            let alt = if did_simplify {
                keys_buf
            } else {
                ptr::null_mut()
            };
            let mut d = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union { v_number: 0 },
            };
            // SAFETY: `mp` is a live entry of the table being walked, `arena`
            // is this frame's own, and `rettv`'s list was allocated above.
            unsafe {
                let dict = mapblock_fill_dict(mp, alt, buffer_local, abbr, true, &raw mut arena);
                let mut obj = Object::dict(dict);
                object_to_vim_take_luaref(&raw mut obj, &raw mut d, true, ptr::null_mut());
                debug_assert_eq!(d.v_type, VAR_DICT);
                tv_list_append_dict((*rettv).vval.v_list, d.vval.v_dict);
                arena_mem_free(arena_finish(&raw mut arena));
                xfree(keys_buf.cast());
            }
            None
        };
        // SAFETY: the tables are live, and `collect` neither unlinks nor frees
        // an entry.
        unsafe { map_walk::<()>(table, abbr, collect) };
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
    let mut p = if !mode.is_empty() {
        mode.data()
    } else {
        c"m".as_ptr().cast_mut()
    };
    // SAFETY: the caller's promise — `mode` is a live API string, so `p` is
    // NUL-terminated and every step below stays inside it.
    let forceit = unsafe { c_int::from(*p) } == c_int::from(b'!');
    // SAFETY: as above.
    let int_mode = unsafe { get_map_mode(&raw mut p, forceit) };
    if forceit {
        debug_assert_eq!(p, mode.data());
        // SAFETY: `get_map_mode` put `p` back at the `!` it did not consume.
        p = unsafe { p.add(1) };
    }
    // SAFETY: as above.
    let is_abbrev = int_mode & (MODE_INSERT | MODE_CMDLINE) != 0 && unsafe { *p } == b'a' as c_char;
    (int_mode, is_abbrev, p)
}

/// Every mapping in `mode`, as `maparg()`-like dicts: `nvim_get_keymap`.
///
/// `buf` is the buffer whose local mappings to report, or `None` for the
/// global ones.
///
/// # Safety
/// `arena` must be live.
pub unsafe fn keymap_array(mode: String_0, buf: Option<Buf>, arena: *mut Arena) -> Array {
    // SAFETY: the caller's promise — `mode` is a live API string.
    let (int_mode, is_abbrev, _) = unsafe { parse_shortname_mode(mode) };
    let buffer_value = buf.map_or(0, |buf| buf.handle as c_int);
    let table = match buf {
        Some(buf) => MapTable::Buffer(buf),
        None => MapTable::Global,
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
        let collect = |mp: Mb| {
            if mp.m_simplified != 0 || int_mode & mp.m_mode == 0 {
                return None;
            }
            let alt = mp.m_alt;
            // SAFETY: a non-null `m_alt` is the live twin of this entry, whose
            // `m_keys` is its own NUL-terminated LHS; `arena` is the caller's.
            let dict = unsafe {
                let lhsrawalt = if alt.is_null() {
                    ptr::null_mut()
                } else {
                    (*alt).m_keys
                };
                mapblock_fill_dict(mp, lhsrawalt, buffer_value, is_abbrev, false, arena)
            };
            items.push(Object::dict(dict));
            None
        };
        // SAFETY: the tables are live, and `collect` neither unlinks nor frees
        // an entry.
        unsafe { map_walk::<()>(table, is_abbrev, collect) };
    }

    // SAFETY: the caller's promise — `arena` is live — and `mappings` is this
    // frame's own builder.
    unsafe { arena_take_arraybuilder(arena, &raw mut mappings) }
}
