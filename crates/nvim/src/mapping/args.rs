//! Parsing `:map` arguments into a [`MapArguments`].
//!
//! [`str_to_mapargs`] splits `<buffer><expr>… {lhs} {rhs}` and hands both
//! halves to [`set_maparg_lhs_rhs`], which runs them through
//! `replace_termcodes`.  [`get_map_mode`], [`get_map_mode_string`] and
//! [`map_mode_to_chars`] are the other direction of the same question — which
//! modes a command name or a `maparg()`-style mode string names, and how a
//! set of modes is spelled back.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::keycodes::{Ctrl_V, KE_LUA};
use crate::option::cpo_has;
use crate::types::CpoFlag;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The modes `:map!` covers.
const MASK_BANG: c_int = MODE_INSERT | MODE_CMDLINE;
/// The modes plain `:map` covers.
const MASK_MAP: c_int = MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
/// The modes `:vmap` covers.
const MASK_V: c_int = MODE_VISUAL | MODE_SELECT;

/// The mode letters for `mode`, NUL-terminated.
///
/// This is what `:map`'s first column and `maparg()`'s `"mode"` key show, and
/// it is at most six letters plus the NUL: `!`, `i`, `l` and `c` each stand
/// alone, a plain `:map` mode set is one space, and anything else is spelled
/// out letter by letter.
pub(crate) fn map_mode_to_chars(mode: c_int) -> [c_char; 7] {
    let mut buf = [0 as c_char; 7];
    let mut at = 0;
    let mut put = |ch: u8| {
        buf[at] = ch as c_char;
        at += 1;
    };

    if mode & MASK_BANG == MASK_BANG {
        put(b'!'); // :map!
    } else if mode & MODE_INSERT != 0 {
        put(b'i'); // :imap
    } else if mode & MODE_LANGMAP != 0 {
        put(b'l'); // :lmap
    } else if mode & MODE_CMDLINE != 0 {
        put(b'c'); // :cmap
    } else if mode & MASK_MAP == MASK_MAP {
        put(b' '); // :map
    } else {
        if mode & MODE_NORMAL != 0 {
            put(b'n'); // :nmap
        }
        if mode & MODE_OP_PENDING != 0 {
            put(b'o'); // :omap
        }
        if mode & MODE_TERMINAL != 0 {
            put(b't'); // :tmap
        }
        if mode & MASK_V == MASK_V {
            put(b'v'); // :vmap
        } else {
            if mode & MODE_VISUAL != 0 {
                put(b'x'); // :xmap
            }
            if mode & MODE_SELECT != 0 {
                put(b's'); // :smap
            }
        }
    }
    buf
}

/// Replace termcodes in `orig_lhs` and `orig_rhs` and store the results in
/// `args`.
///
/// If the final LHS is longer than `MAXMAPLEN`, `lhs_len` holds the larger
/// original length and `lhs` is truncated, which is how the caller detects
/// it.
///
/// If `<C-H>`-style simplification happened, `alt_lhs` holds the
/// unsimplified spelling and `alt_lhs_len` is non-zero; otherwise
/// `alt_lhs_len` is 0.
///
/// # Safety
/// The three input strings must be live.
pub(crate) unsafe fn set_maparg_lhs_rhs(
    orig_lhs: *const c_char,
    orig_lhs_len: size_t,
    orig_rhs: *const c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    cpo_val: *const c_char,
    args: &mut MapArguments,
) -> bool {
    // If the mapping was given as ^V<C_UP>, replace the term codes with the
    // appropriate two bytes; if it is a shifted special key, unshift it too,
    // giving another two. `replace_termcodes` may answer with `lhs_buf` or
    // with memory of its own, which is why it takes the buffer's address; it
    // also removes CTRL-Vs and sometimes backslashes. If something like <C-H>
    // simplifies to 0x08 we take a second, unsimplified copy and mark the
    // pair.
    let mut lhs_buf = [0 as c_char; 128];
    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
    let mut bufarg = lhs_buf.as_mut_ptr();
    let buf = &raw mut bufarg;
    let simplify = &raw mut did_simplify;
    let plain = ptr::null_mut();
    let nosimp = flags | REPTERM_NO_SIMPLIFY as c_int;
    // SAFETY: the caller's promise — `orig_lhs` is `orig_lhs_len` live bytes
    // and `cpo_val` a NUL-terminated `'cpoptions'`.  `lhs_buf` outlives both
    // calls, which is where `replace_termcodes` may leave its answer, and its
    // answer is NUL-terminated.
    let replaced = unsafe {
        let at = replace_termcodes(orig_lhs, orig_lhs_len, buf, 0, flags, simplify, cpo_val);
        if at.is_null() {
            return false;
        }
        cstr::bytes_at(at)
    };
    args.lhs_len = replaced.len();
    args.lhs = MapStr::new(&replaced[..replaced.len().min(MAXMAPLEN as size_t)]);
    if did_simplify {
        // SAFETY: as the first call.
        let alt = unsafe {
            let at = replace_termcodes(orig_lhs, orig_lhs_len, buf, 0, nosimp, plain, cpo_val);
            if at.is_null() {
                return false;
            }
            cstr::bytes_at(at)
        };
        args.alt_lhs_len = alt.len();
        args.alt_lhs = MapStr::new(&alt[..alt.len().min(MAXMAPLEN as size_t)]);
    } else {
        args.alt_lhs_len = 0;
    }

    // SAFETY: as above — the caller's live strings.
    unsafe { set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, 0, cpo_val, args) };
    true
}

/// The `rhs` half of [`set_maparg_lhs_rhs`], also used on its own by
/// `mapset()`, which brings its own already-parsed LHS.
///
/// This is where `args` stops being a parse and becomes an owner: the three
/// RHS strings and the Lua reference are bundled into the one [`MapRhs`] every
/// mapblock the parse goes on to create will share.  `args.desc` is folded in
/// here, so a caller that wants a `desc` must set it *first*.
///
/// # Safety
/// `orig_rhs` and `cpo_val` must be live.
pub(crate) unsafe fn set_maparg_rhs(
    orig_rhs: *const c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    sid: scid_T,
    cpo_val: *const c_char,
    args: &mut MapArguments,
) {
    debug_assert!(args.rhs.is_none(), "an RHS is parsed once");
    let desc = args.desc.take();

    if rhs_lua != LUA_NOREF {
        // orig_rhs is not used for Lua mappings, but still has to be a
        // string; the rhs stores <Lua>ref_no<CR> for the key loop.
        let mut tmp_buf = [0 as c_char; 64];
        let at = tmp_buf.as_mut_ptr();
        let cap = (tmp_buf.len() - 1) as size_t;
        let fmt = c"%c%c%c%d\r".as_ptr();
        // SAFETY: `tmp_buf` is 64 bytes and `cap` leaves room for the NUL; the
        // format string names exactly the four arguments that follow it, and
        // the answer is NUL-terminated.
        let str = unsafe {
            let ke_lua = KE_LUA as c_int;
            vim_snprintf(at, cap, fmt, K_SPECIAL, KS_EXTRA, ke_lua, rhs_lua);
            MapStr::new(cstr::bytes_at(at))
        };
        args.rhs = Some(Rc::new(MapRhs {
            str,
            orig_str: MapStr::empty(),
            desc,
            luaref: rhs_lua,
        }));
        return;
    }

    // SAFETY: the caller's promise — `orig_rhs` is `orig_rhs_len` live bytes.
    let orig_str =
        unsafe { MapStr::new(core::slice::from_raw_parts(orig_rhs.cast(), orig_rhs_len)) };
    // SAFETY: `orig_rhs` is NUL-terminated.
    let str = if unsafe { strcasecmp(orig_rhs, c"<nop>".as_ptr()) } == 0 {
        // "<Nop>" means nothing.
        MapStr::empty()
    } else {
        let mut rhs_buf: *mut c_char = ptr::null_mut();
        let buf = &raw mut rhs_buf;
        let dolt = REPTERM_DO_LT as c_int;
        let plain = ptr::null_mut();
        // SAFETY: as above, plus `cpo_val` NUL-terminated; `rhs_buf` is only a
        // scratch slot `replace_termcodes` may take over, which the guard
        // releases once the answer has been copied out of it.
        unsafe {
            let at = replace_termcodes(orig_rhs, orig_rhs_len, buf, sid, dolt, plain, cpo_val);
            let _owned = COwned::new(rhs_buf);
            MapStr::new(cstr::bytes_at(at))
        }
    };
    // "<Nop>" is a noop, and so is a single ^V: `replace_termcodes` may
    // produce an empty string even when `orig_rhs` is not -- see
    // :h map-empty-rhs.
    args.rhs_is_noop = orig_rhs_len != 0 && str.is_empty();
    args.rhs = Some(Rc::new(MapRhs {
        str,
        orig_str,
        desc,
        luaref: LUA_NOREF,
    }));
}

/// If `to_parse` starts with `word`, step it past `word` and the whitespace
/// after it.
pub(crate) fn take_map_arg(to_parse: &mut &[u8], word: &[u8]) -> bool {
    let Some(rest) = to_parse.strip_prefix(word) else {
        return false;
    };
    *to_parse = skip_white(rest);
    true
}

/// Parse a string of `:map-arguments` into a [`MapArguments`].
///
/// Termcodes, backslashes and CTRL-Vs inside the extracted `{lhs}` and
/// `{rhs}` are replaced by [`set_maparg_lhs_rhs`].
///
/// `is_unmap` makes everything right of the last map argument the `{lhs}`,
/// spaces included: `:unmap` has no separate `{rhs}`.
///
/// Answers 0 on success and 1 if the arguments are invalid.
///
/// # Safety
/// `strargs` must be live and NUL-terminated.
pub(crate) unsafe fn str_to_mapargs(
    strargs: *const c_char,
    is_unmap: bool,
    args: &mut MapArguments,
) -> c_int {
    // SAFETY: the caller's promise — `strargs` is NUL-terminated, and so is
    // whatever `skipwhite` leaves inside it.
    let (base, all) = unsafe {
        let base = skipwhite(strargs);
        (base, cstr::bytes_at(base))
    };
    let mut rest = all;

    // Accept <buffer>, <nowait>, <silent>, <expr>, <script> and <unique>
    // in any order.
    loop {
        if take_map_arg(&mut rest, b"<buffer>") {
            args.buffer = true;
        } else if take_map_arg(&mut rest, b"<nowait>") {
            args.nowait = true;
        } else if take_map_arg(&mut rest, b"<silent>") {
            args.silent = true;
        } else if take_map_arg(&mut rest, b"<special>") {
            // Obsolete modifier, accepted and ignored.
        } else if take_map_arg(&mut rest, b"<script>") {
            args.script = true;
        } else if take_map_arg(&mut rest, b"<expr>") {
            args.expr = true;
        } else if take_map_arg(&mut rest, b"<unique>") {
            args.unique = true;
        } else {
            break;
        }
    }
    let consumed = all.len() - rest.len();

    // The next whitespace character ends {lhs} -- unless it is preceded
    // by a CTRL-V, or by a backslash when 'cpoptions' has no 'B'.
    let do_backslash = !cpo_has(CpoFlag::BSLASH);
    let mut at = 0;
    while let Some(&ch) = rest.get(at) {
        if !is_unmap && ascii_iswhite(c_int::from(ch)) {
            break;
        }
        let escape = c_int::from(ch) == Ctrl_V || ch == b'\\' && do_backslash;
        // Skip the CTRL-V or backslash *and* whatever it escapes, unless the
        // escape is the last byte.
        if escape && rest.get(at + 1).is_some_and(|&next| next != 0) {
            at += 1;
        }
        at += 1;
    }
    let orig_lhs_len = at;

    // The given {lhs} may be longer than MAXMAPLEN before termcodes are
    // replaced ("<Space>" is longer than ' '), so copy it out first.
    if orig_lhs_len >= 256 {
        return 1;
    }
    let mut lhs_to_replace = [0 as c_char; 256];
    for (slot, &byte) in lhs_to_replace.iter_mut().zip(&rest[..orig_lhs_len]) {
        *slot = byte as c_char;
    }

    // SAFETY: `consumed + orig_lhs_len` is an index inside `base`'s own
    // NUL-terminated bytes, and `lhs` names the local copy just made.
    let ok = unsafe {
        let rhs_start = skipwhite(base.add(consumed + orig_lhs_len));
        let orig_rhs_len = cstr::bytes_at(rhs_start).len();
        set_maparg_lhs_rhs(
            lhs_to_replace.as_ptr(),
            orig_lhs_len,
            rhs_start,
            orig_rhs_len,
            LUA_NOREF,
            p_cpo.get(),
            args,
        )
    };
    if !ok {
        return 1;
    }
    if args.lhs_len > MAXMAPLEN as size_t {
        return 1;
    }
    0
}

/// The mapping mode a command name asks for, stepping `cmdp` past the mode
/// letter it consumed.
///
/// A leading `n` only means `:nmap` when the next letter is not `o`, which is
/// what keeps `:noremap` out of Normal mode.
///
/// # Safety
/// `cmdp` must point at a live, NUL-terminated command name.
pub(crate) unsafe fn get_map_mode(cmdp: *mut *mut c_char, forceit: bool) -> c_int {
    // SAFETY: the caller's promise — `cmdp` holds a live, NUL-terminated
    // command name, so its first byte and the one after it are readable.
    let (mut p, modec) = unsafe {
        let p = *cmdp;
        (p.add(1), *p as u8)
    };
    let mode = match modec {
        b'i' => MODE_INSERT,  // :imap
        b'l' => MODE_LANGMAP, // :lmap
        b'c' => MODE_CMDLINE, // :cmap
        // SAFETY: `modec` is `n`, so `p` is still inside the name.
        b'n' if unsafe { *p } != b'o' as c_char => MODE_NORMAL, // :nmap, avoiding :noremap
        b'v' => MASK_V,                                         // :vmap
        b'x' => MODE_VISUAL,                                    // :xmap
        b's' => MODE_SELECT,                                    // :smap
        b'o' => MODE_OP_PENDING,                                // :omap
        b't' => MODE_TERMINAL,                                  // :tmap
        _ => {
            // SAFETY: `p` was stepped forward once off the name's first byte.
            p = unsafe { p.sub(1) };
            if forceit {
                MASK_BANG // :map!
            } else {
                MASK_MAP // :map
            }
        }
    };
    // SAFETY: the caller's writable slot.
    unsafe { *cmdp = p };
    mode
}

/// The mapping mode a `maparg()`-style mode string asks for, or 0 if it is
/// not a legal one.
///
/// The string may name several modes at once ("nox"), and `!` and ' ' are
/// each a whole set.  A combination is only legal when it is one bit, or when
/// every bit of it lies inside a single one of those two sets; an
/// abbreviation may only ask for Insert and Cmdline.
///
pub(crate) fn get_map_mode_string(mode_string: &[u8], abbr: bool) -> c_int {
    let chars = if mode_string.is_empty() {
        b" ".as_slice() // compatibility
    } else {
        mode_string
    };

    let mut mode = 0;
    for &modec in chars {
        mode |= match modec {
            b'i' => MODE_INSERT,
            b'l' => MODE_LANGMAP,
            b'c' => MODE_CMDLINE,
            b'n' => MODE_NORMAL,
            b'x' => MODE_VISUAL,
            b's' => MODE_SELECT,
            b'o' => MODE_OP_PENDING,
            b't' => MODE_TERMINAL,
            b'v' => MASK_V,
            b'!' => MASK_BANG,
            b' ' => MASK_MAP,
            _ => return 0, // error, unknown mode character
        };
    }

    // True when `mode` is non-empty and lies wholly inside `mask`.
    let fits = |mask: c_int| mode & mask != 0 && mode & !mask == 0;
    if abbr {
        if mode & !MASK_BANG != 0 {
            return 0;
        }
    } else if mode & (mode - 1) != 0 && !(fits(MASK_BANG) || fits(MASK_MAP)) {
        // More than one bit set, and not contained in either mask.
        return 0;
    }

    mode
}
