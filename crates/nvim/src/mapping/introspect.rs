//! Reporting mappings to Vimscript and to the API.
//!
//! [`mapblock_fill_dict`] renders one [`MapBlock`] as the twenty-key dict
//! that `maparg()`, `maplist()` and `nvim_get_keymap` all answer with;
//! [`get_maparg`] backs `maparg()`/`mapcheck()` and [`keymap_array`] backs
//! `nvim_get_keymap`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::memory::handoff::owned_cstr;
use crate::types::{NUL, VAR_DICT, kListLenUnknown};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// `hasmapto()`: whether any mapping in the named modes has `{name}` in its
/// RHS.
pub fn f_hasmapto(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    // SAFETY: the Vimscript call convention — `args` is a live argument
    // vector, and `numbuf` outlives the string it lends back.
    let name = numbuf.string_ptr(&args[0]);
    let mode = match args.get(1) {
        // SAFETY: `buf` is the scratch `tv_get_string_buf` may answer with.
        Some(tv) => buf.string_ptr(tv),
        None => c"nvo".as_ptr(),
    };
    let number = |n: usize| args.get(n).map(tv_get_number);
    let abbr = number(2).is_some_and(|n| n != 0);
    // SAFETY: both strings are NUL-terminated, and `result` is the caller's
    // writable answer slot.
    unsafe {
        let found = map_to_exists(name, mode, abbr);
        (*result).write_number(VarNumber::from(found));
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
struct Filling(ApiDict);

impl Filling {
    fn new(dict: ApiDict) -> Self {
        Self(dict)
    }

    /// C's `PUT_C`: append `key: value`.
    fn put(&mut self, key: &'static CStr, value: Object) {
        debug_assert!(self.0.len() < MAPARG_DICT_KEYS);
        self.0.insert(key, value);
    }

    /// The finished dict.
    fn finish(self) -> ApiDict {
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
/// The arena here is this call's own scratch: `str2special_arena` and the
/// mode buffer are intermediates the dictionary copies out of, so the whole
/// chain is released before the answer leaves.
pub(crate) fn mapblock_fill_dict(
    mp: Mb,
    lhsrawalt: Option<&MapStr>,
    buffer_value: c_int,
    abbr: bool,
    compatible: bool,
) -> ApiDict {
    let mut scratch = ARENA_EMPTY;
    let arena = &raw mut scratch;
    // SAFETY: the caller's promise — `mp` is a live mapblock, so `m_keys` is
    // its NUL-terminated LHS.  `arena_alloc` answers seven writable bytes,
    // which is the width `map_mode_to_chars` fills.
    let (dict, lhs, mapmode) = unsafe {
        let dict = ApiDict::with_capacity(MAPARG_DICT_KEYS);
        let lhs = str2special_arena(mp.m_keys.as_ptr(), compatible, !compatible, arena);
        let mapmode: *mut c_char = arena_alloc(arena, 7, false).cast();
        mapmode.copy_from_nonoverlapping(map_mode_to_chars(mp.m_mode).as_ptr(), 7);
        (dict, lhs, mapmode)
    };
    let rhs = &mp.m_rhs;
    let mut out = Filling::new(dict);

    let noremap_value = if compatible {
        // Keep the old compatible behaviour, which cannot tell a
        // <script> mapping apart.
        c_int::from(mp.m_noremap != 0)
    } else if mp.m_noremap == REMAP_SCRIPT {
        2
    } else {
        c_int::from(mp.m_noremap != 0)
    };

    if rhs.luaref() != LUA_NOREF {
        let luaref = api_new_luaref(rhs.luaref());
        out.put(c"callback", Object::LuaRef(luaref));
    } else {
        // SAFETY: `orig_str` and `str` are the mapping's own NUL-terminated
        // strings, and `arena` is this call's scratch.
        let text = unsafe {
            cstr_to_string(if compatible {
                rhs.orig_str.as_ptr()
            } else {
                str2special_arena(rhs.str.as_ptr(), false, true, arena)
            })
        };
        out.put(c"rhs", Object::string(text));
    }
    if let Some(desc) = &rhs.desc {
        // SAFETY: the mapping's own NUL-terminated text.
        let desc = unsafe { cstr_to_string(desc.as_ptr()) };
        out.put(c"desc", Object::string(desc));
    }
    // SAFETY: `lhs` is `str2special_arena`'s answer and `m_keys` the mapping's
    // own LHS; both are NUL-terminated.
    let (lhs, lhsraw) = unsafe { (cstr_to_string(lhs), cstr_to_string(mp.m_keys.as_ptr())) };
    out.put(c"lhs", Object::string(lhs));
    out.put(c"lhsraw", Object::string(lhsraw));
    if let Some(alt) = lhsrawalt {
        // Also add the value for the simplified entry.
        // SAFETY: a `MapStr` is NUL-terminated by its own invariant.
        let alt = unsafe { cstr_to_string(alt.as_ptr()) };
        out.put(c"lhsrawalt", Object::string(alt));
    }
    out.put(c"noremap", Object::integer(noremap_value.into()));
    out.put(
        c"script",
        Object::integer(Integer::from(mp.m_noremap == REMAP_SCRIPT)),
    );
    out.put(c"expr", Object::integer(Integer::from(mp.m_expr)));
    out.put(c"silent", Object::integer(Integer::from(mp.m_silent)));
    out.put(c"sid", Object::integer(mp.m_script_ctx.sc_sid.into()));
    out.put(c"scriptversion", Object::integer(1));
    out.put(c"lnum", Object::integer(mp.m_script_ctx.sc_lnum.into()));
    out.put(c"buffer", Object::integer(buffer_value.into()));
    if !compatible {
        out.put(c"buf", Object::integer(buffer_value.into()));
    }
    out.put(c"nowait", Object::integer(Integer::from(mp.m_nowait)));
    out.put(
        c"replace_keycodes",
        Object::integer(Integer::from(mp.m_replace_keycodes)),
    );
    // SAFETY: `mapmode` is the seven-byte NUL-terminated copy made above.
    let mode = unsafe { cstr_to_string(mapmode) };
    out.put(c"mode", Object::string(mode));
    out.put(c"abbr", Object::integer(Integer::from(abbr)));
    out.put(c"mode_bits", Object::integer(mp.m_mode.into()));

    // Every string above was copied out of the scratch, so nothing the
    // answer holds points into it.
    // SAFETY: the chain is this call's own.
    unsafe { arena_mem_free(arena_finish(arena)) };
    out.finish()
}

/// The body of `maparg()` and `mapcheck()`: `exact` is what tells them apart.
fn get_maparg(args: &[TypVal], result: &mut TypVal, exact: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise — `result` is the writable answer slot.
    let mut ret = unsafe { Live::new(result) };
    // Return an empty string on failure.
    ret.write_string(ptr::null_mut());

    // SAFETY: the Vimscript call convention — `args` is a live argument
    // vector whose first entry is the keys, NUL-terminated.
    let keys = numbuf.string_ptr(&args[0]).cast_mut();
    // SAFETY: as above.
    if unsafe { c_int::from(*keys) } == NUL {
        return;
    }

    let mut buf = NumBuf::new();
    // SAFETY: as above.
    let number = |n: usize| args.get(n).map(tv_get_number);
    let abbr = number(2).is_some_and(|n| n != 0);
    let get_dict = number(3).is_some_and(|n| n != 0);
    let mut which: *mut c_char = match args.get(1) {
        // SAFETY: `buf` is `tv_get_string_buf_chk`'s scratch.
        Some(tv) => buf.string_ptr_chk(tv).cast_mut(),
        None => c"".as_ptr().cast_mut(),
    };
    if which.is_null() {
        return;
    }

    let mut keys_buf: *mut c_char = ptr::null_mut();
    let mut alt_keys_buf: *mut c_char = ptr::null_mut();
    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
    let nosimp = flags | REPTERM_NO_SIMPLIFY as c_int;
    let cpo = p_cpo();
    let plain = ptr::null_mut();
    let simplify = &raw mut did_simplify;
    let out = &raw mut keys_buf;
    let alt_out = &raw mut alt_keys_buf;
    // SAFETY: `which` is a NUL-terminated mode string.
    let mode = unsafe { get_map_mode(&raw mut which, false) };

    // SAFETY: `keys` is NUL-terminated, and both `*_buf` slots are locals that
    // outlive the calls; the allocations they take over are the guards'.
    let (keys_simplified, _owned, mut found) = unsafe {
        let len = cstr::bytes_at(keys).len();
        let simplified = replace_termcodes(keys, len, out, 0, flags, simplify, cpo);
        let owned = COwned::new(keys_buf);
        let found = check_map(simplified, mode, exact, false, abbr);
        (simplified, owned, found)
    };
    // SAFETY: as above.
    let _alt_owned = unsafe {
        if did_simplify {
            // When the lhs is being simplified the not-simplified keys are
            // preferred for printing, like in do_map(). Upstream leaves the
            // previous `mp` in place when this second look-up fails, but it
            // clears both `rhs` and `rhs_lua`, and every reader of `mp` is
            // behind a test on one of those -- so dropping the whole match
            // is the same answer.
            let len = cstr::bytes_at(keys).len();
            replace_termcodes(keys, len, alt_out, 0, nosimp, plain, cpo);
            found = check_map(alt_keys_buf, mode, exact, false, abbr);
        }
        COwned::new(alt_keys_buf)
    };

    // SAFETY: a match names a still-linked mapping.
    let found = found.map(|found| (unsafe { Mb::new(found.mp) }, found.local));
    if !get_dict {
        // Return a string.
        if let Some((mp, _)) = found {
            let rhs = &mp.m_rhs;
            ret.write_string(if rhs.luaref() != LUA_NOREF {
                // SAFETY: `mp` is the matching mapping, still linked.
                unsafe { nlua_funcref_str(rhs.luaref()) }
            } else if rhs.str.is_empty() {
                owned_cstr(b"<Nop>".to_vec())
            } else {
                // SAFETY: the matching mapping's NUL-terminated RHS.
                unsafe { str2special_save(rhs.str.as_ptr(), false, false) }
            });
        }
    } else if let Some((mp, local)) = found {
        // Return a dictionary.
        // SAFETY: `keys_simplified` is `replace_termcodes`'s NUL-terminated
        // answer and `mp` a live mapping.
        let dict = unsafe {
            let alt = did_simplify.then(|| MapStr::new(cstr::bytes_at(keys_simplified)));
            mapblock_fill_dict(mp, alt.as_ref(), c_int::from(local), abbr, true)
        };
        *result = TypVal::from(Object::dict(dict));
    } else {
        // Return an empty dictionary.
        // SAFETY: the caller's writable answer slot.
        tv_dict_alloc_ret(result);
    }
}

/// `maplist()`: every mapping, global then buffer-local, as a list of dicts.
pub fn f_maplist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
    let cpo = p_cpo();
    let abbr = args.first().is_some_and(|tv| tv_get_bool(tv) != 0);
    // SAFETY: as above.
    tv_list_alloc_ret(result, kListLenUnknown as ptrdiff_t);
    let cur = Buf::current();

    // Do it twice: once for global maps and once for local maps.
    for (buffer_local, table) in [(0, MapTable::Global), (1, MapTable::Buffer(cur))] {
        let collect = |mp: Mb| {
            if mp.m_simplified {
                return None;
            }
            let mut keys_buf: *mut c_char = ptr::null_mut();
            let mut did_simplify = false;
            let out = &raw mut keys_buf;
            let simplify = &raw mut did_simplify;

            let mut arena = ARENA_EMPTY;
            // SAFETY: `m_keys` is the mapping's own NUL-terminated LHS, and
            // `arena`, `keys_buf` and `did_simplify` are this frame's locals;
            // the allocation `keys_buf` takes over is the guard's.
            let (alt, _owned) = unsafe {
                let lhs = str2special_arena(mp.m_keys.as_ptr(), true, false, &raw mut arena);
                let len = cstr::bytes_at(lhs).len();
                replace_termcodes(lhs, len, out, 0, flags, simplify, cpo);
                let alt = (did_simplify && !keys_buf.is_null())
                    .then(|| MapStr::new(cstr::bytes_at(keys_buf)));
                (alt, COwned::new(keys_buf))
            };

            // SAFETY: `mp` is a live entry of the table being walked and
            // `result`'s list was allocated above.
            unsafe {
                let dict = mapblock_fill_dict(mp, alt.as_ref(), buffer_local, abbr, true);
                let mut d = TypVal::from(Object::dict(dict));
                debug_assert_eq!(d.v_type(), VAR_DICT);
                (*(*result).list_or_null()).push_dict(d.take_dict());
                arena_mem_free(arena_finish(&raw mut arena));
            }
            None
        };
        // SAFETY: the tables are live, and `collect` neither unlinks nor frees
        // an entry.
        unsafe { map_walk::<()>(table, abbr, collect) };
    }
}

/// `maparg()`.
pub fn f_maparg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_maparg(args, result, true)
}

/// `mapcheck()`.
pub fn f_mapcheck(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_maparg(args, result, false)
}

/// The mode a mode-shortname string names, and how much of it was consumed.
///
/// `nvim_get_keymap` and `nvim_set_keymap` take the same shortnames as
/// `:map`'s command names, optionally with a `!` in front and an `a` after
/// for the abbreviation form.
///
/// # Safety
/// `mode` must be a live API string.
pub(crate) unsafe fn parse_shortname_mode(mode: &String_0) -> (c_int, bool, *mut c_char) {
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
pub fn keymap_array(mode: String_0, buffer: Option<Buf>) -> Array {
    // SAFETY: the caller's promise — `mode` is a live API string.
    let (int_mode, is_abbrev, _) = unsafe { parse_shortname_mode(&mode) };
    let buffer_value = buffer.map_or(0, |buf| buf.handle as c_int);
    let table = match buffer {
        Some(buf) => MapTable::Buffer(buf),
        None => MapTable::Global,
    };

    let mut mappings = Array::EMPTY;
    {
        let collect = |mp: Mb| {
            if mp.m_simplified || int_mode & mp.m_mode == 0 {
                return None;
            }
            let alt = mp.m_alt;
            // SAFETY: a non-null `m_alt` is the live twin of this entry, whose
            // `m_keys` is its own LHS.
            let dict = unsafe {
                let twin = (!alt.is_null()).then(|| Mb::new(alt));
                let lhsrawalt = twin.as_ref().map(|twin| &twin.m_keys);
                mapblock_fill_dict(mp, lhsrawalt, buffer_value, is_abbrev, false)
            };
            mappings.push(Object::dict(dict));
            None
        };
        // SAFETY: the tables are live, and `collect` neither unlinks nor frees
        // an entry.
        unsafe { map_walk::<()>(table, is_abbrev, collect) };
    }
    mappings
}
