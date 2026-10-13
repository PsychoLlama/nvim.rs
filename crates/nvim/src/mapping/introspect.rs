//! Reporting mappings to Vimscript and to the API.
//!
//! [`mapblock_fill_dict`] renders one [`MapBlock`] as the twenty-key dict
//! that `maparg()`, `maplist()` and `nvim_get_keymap` all answer with;
//! [`get_maparg`] backs `maparg()`/`mapcheck()` and [`keymap_array`] backs
//! `nvim_get_keymap`.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::lua::executor::funcref_description;
use crate::memory::ThinCString;
use crate::option::vars::P_CPO;
use crate::types::{VAR_DICT, kListLenUnknown};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

/// `hasmapto()`: whether any mapping in the named modes has `{name}` in its
/// RHS.
pub fn f_hasmapto(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    let name = numbuf.string(&args[0]).to_bytes();
    let mode = match args.get(1) {
        // `buf` is the scratch `tv_get_string_buf` may answer with.
        Some(tv) => buf.string(tv).to_bytes(),
        None => b"nvo",
    };
    let number = |n: usize| args.get(n).map(tv_get_number);
    let abbr = number(2).is_some_and(|n| n != 0);
    let found = map_to_exists(name, mode, abbr);
    result.write_number(VarNumber::from(found));
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
/// `lhsrawalt` is the other spelling of a simplified LHS, if there is one.
pub(crate) fn mapblock_fill_dict(
    mp: &MapBlock,
    lhsrawalt: Option<&MapStr>,
    buffer_value: c_int,
    abbr: bool,
    compatible: bool,
) -> ApiDict {
    let dict = ApiDict::with_capacity(MAPARG_DICT_KEYS);
    let lhs = special_keys_text(mp.m_keys.as_cstr().to_bytes(), compatible, !compatible);
    let mapmode = map_mode_to_chars(mp.m_mode);
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
        let text = if compatible {
            String_0::from_cstr(rhs.orig_str.as_cstr())
        } else {
            String_0::from_bytes(&special_keys_text(
                rhs.str.as_cstr().to_bytes(),
                false,
                true,
            ))
        };
        out.put(c"rhs", Object::string(text));
    }
    if let Some(desc) = &rhs.desc {
        let desc = String_0::from_cstr(desc.as_cstr());
        out.put(c"desc", Object::string(desc));
    }
    let (lhs, lhsraw) = (
        String_0::from_bytes(&lhs),
        String_0::from_cstr(mp.m_keys.as_cstr()),
    );
    out.put(c"lhs", Object::string(lhs));
    out.put(c"lhsraw", Object::string(lhsraw));
    if let Some(alt) = lhsrawalt {
        // Also add the value for the simplified entry.
        let alt = String_0::from_cstr(alt.as_cstr());
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
    let mode = String_0::from_cstr(mapmode.as_cstr());
    out.put(c"mode", Object::string(mode));
    out.put(c"abbr", Object::integer(Integer::from(abbr)));
    out.put(c"mode_bits", Object::integer(mp.m_mode.into()));

    out.finish()
}

/// The body of `maparg()` and `mapcheck()`: `exact` is what tells them apart.
fn get_maparg(args: &[TypVal], result: &mut TypVal, exact: bool) {
    let mut numbuf = NumBuf::new();
    // Return an empty string on failure.
    result.write_string(None);

    let keys = numbuf.string(&args[0]).to_bytes();
    if keys.is_empty() {
        return;
    }

    let mut buf = NumBuf::new();
    let number = |n: usize| args.get(n).map(tv_get_number);
    let abbr = number(2).is_some_and(|n| n != 0);
    let get_dict = number(3).is_some_and(|n| n != 0);
    let which = match args.get(1) {
        // `buf` is `tv_get_string_buf_chk`'s scratch.
        Some(tv) => buf.string_chk(tv),
        None => Some(c""),
    };
    let Some(which) = which else {
        return;
    };

    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART | REPTERM_DO_LT;
    let nosimp = flags | REPTERM_NO_SIMPLIFY;
    let (mode, _) = get_map_mode(which.to_bytes(), false);

    // A copy, because the second `replace_termcodes` below is past the end
    // of a projection's borrow.
    let cpo = P_CPO.get();
    let c_prefix = |bytes: Option<Vec<u8>>| {
        let mut bytes = bytes.unwrap_or_default();
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        bytes.truncate(end);
        bytes
    };
    let simplify = Some(&mut did_simplify);
    let keys_simplified = c_prefix(replace_termcodes(
        keys,
        0,
        flags,
        simplify,
        cpo.as_cstr(),
        None,
    ));
    // When the lhs is being simplified the not-simplified keys are preferred
    // for printing, like in do_map(). Upstream looks the simplified keys up
    // first and then replaces the match with this one's -- leaving the
    // previous `mp` in place when it fails, but clearing both `rhs` and
    // `rhs_lua`, and every reader of `mp` is behind a test on one of those --
    // so looking up only the one it ends with is the same answer.
    let lookup = if did_simplify {
        c_prefix(replace_termcodes(
            keys,
            0,
            nosimp,
            None,
            cpo.as_cstr(),
            None,
        ))
    } else {
        keys_simplified.clone()
    };

    let found = check_map(&lookup, mode, exact, false, abbr, |mp, local| {
        if !get_dict {
            // Return a string.
            let rhs = &mp.m_rhs;
            let text = if rhs.luaref() != LUA_NOREF {
                funcref_description(rhs.luaref())
            } else if rhs.str.is_empty() {
                Some(ThinCString::from_bytes(b"<Nop>"))
            } else {
                let text = special_keys_text(rhs.str.as_bytes(), false, false);
                Some(ThinCString::from_bytes(&text))
            };
            TypVal::string(text)
        } else {
            // Return a dictionary.
            let alt = did_simplify.then(|| MapStr::new(&keys_simplified));
            let dict = mapblock_fill_dict(mp, alt.as_ref(), c_int::from(local), abbr, true);
            TypVal::from(Object::dict(dict))
        }
    });
    match found {
        Some(answer) => *result = answer,
        None if get_dict => {
            // Return an empty dictionary.
            tv_dict_alloc_ret(result);
        }
        None => {}
    }
}

/// `maplist()`: every mapping, global then buffer-local, as a list of dicts.
pub fn f_maplist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let flags = REPTERM_FROM_PART | REPTERM_DO_LT;
    // A copy: the walk below hands each mapping to a closure.
    let cpo = P_CPO.get();
    let abbr = args.first().is_some_and(|tv| tv_get_bool(tv) != 0);
    let list = tv_list_alloc_ret(result, kListLenUnknown as ptrdiff_t);
    let cur = Buf::current();

    // Do it twice: once for global maps and once for local maps.
    for (buffer_local, table) in [(0, MapTable::Global), (1, MapTable::Buffer(cur))] {
        let collect = |mp: &MapBlock| {
            if mp.m_simplified {
                return None;
            }
            let mut did_simplify = false;

            let lhs = special_keys_text(mp.m_keys.as_bytes(), true, false);
            let simplify = Some(&mut did_simplify);
            let keys = replace_termcodes(&lhs, 0, flags, simplify, cpo.as_cstr(), None);
            let alt = keys
                .filter(|_| did_simplify)
                .map(|keys| MapStr::new(keys.split(|&b| b == 0).next().unwrap_or_default()));

            let dict = mapblock_fill_dict(mp, alt.as_ref(), buffer_local, abbr, true);
            let mut d = TypVal::from(Object::dict(dict));
            debug_assert_eq!(d.v_type(), VAR_DICT);
            list.push_dict(d.take_dict());
            None::<()>
        };
        map_walk(table, abbr, collect);
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

/// The mode a mode-shortname string names, whether it asked for the
/// abbreviation form, and how much of it was consumed.
///
/// `nvim_get_keymap` and `nvim_set_keymap` take the same shortnames as
/// `:map`'s command names, optionally with a `!` in front and an `a` after
/// for the abbreviation form.  The `a` is not counted as consumed.
pub(crate) fn parse_shortname_mode(mode: &String_0) -> (c_int, bool, usize) {
    let text: &[u8] = if mode.is_empty() {
        b"m"
    } else {
        mode.as_bytes()
    };
    let forceit = text.first() == Some(&b'!');
    let (int_mode, mut used) = get_map_mode(text, forceit);
    if forceit {
        debug_assert_eq!(used, 0);
        // `get_map_mode` left the `!` it did not consume.
        used = 1;
    }
    let is_abbrev = int_mode & (MODE_INSERT | MODE_CMDLINE) != 0 && text.get(used) == Some(&b'a');
    (int_mode, is_abbrev, used)
}

/// Every mapping in `mode`, as `maparg()`-like dicts: `nvim_get_keymap`.
///
/// `buf` is the buffer whose local mappings to report, or `None` for the
/// global ones.
pub fn keymap_array(mode: String_0, buffer: Option<Buf>) -> Array {
    let (int_mode, is_abbrev, _) = parse_shortname_mode(&mode);
    let buffer_value = buffer.map_or(0, |buf| buf.handle as c_int);
    let table = match buffer {
        Some(buf) => MapTable::Buffer(buf),
        None => MapTable::Global,
    };

    let mut mappings = Array::EMPTY;
    table.with(|tables| {
        for mp in tables.walk(is_abbrev) {
            if mp.m_simplified || int_mode & mp.m_mode == 0 {
                continue;
            }
            let lhsrawalt = tables.twin_keys(is_abbrev, mp);
            let dict = mapblock_fill_dict(mp, lhsrawalt, buffer_value, is_abbrev, false);
            mappings.push(Object::dict(dict));
        }
    });
    mappings
}
