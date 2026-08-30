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
use crate::types::{CpoFlag, NUL};
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

/// Size of a [`MapArguments`] LHS buffer: `MAXMAPLEN` characters plus a NUL.
const MAXMAPLEN_BUF: size_t = MAXMAPLEN as size_t + 1;

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
/// `mapargs`.
///
/// `rhs` and `orig_rhs` both come back pointing at fresh allocations.  If the
/// final LHS is longer than `MAXMAPLEN`, `lhs_len` holds the larger original
/// length and `lhs` is truncated, which is how the caller detects it.
///
/// If `<C-H>`-style simplification happened, `alt_lhs` holds the
/// unsimplified spelling and `alt_lhs_len` is non-zero; otherwise
/// `alt_lhs_len` is 0.
///
/// # Safety
/// The three input strings must be live, and `mapargs` writable.
pub(crate) unsafe fn set_maparg_lhs_rhs(
    orig_lhs: *const c_char,
    orig_lhs_len: size_t,
    orig_rhs: *const c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) -> bool {
    // SAFETY: the caller's promise — `mapargs` is a writable `MapArguments`.
    let mut args = unsafe { Ma::new(mapargs) };
    args.rhs_lua = rhs_lua;

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
    // calls, which is where `replace_termcodes` may leave its answer.
    let mut replaced =
        unsafe { replace_termcodes(orig_lhs, orig_lhs_len, buf, 0, flags, simplify, cpo_val) };
    if replaced.is_null() {
        return false;
    }
    // SAFETY: `replace_termcodes` answers a NUL-terminated string.
    args.lhs_len = unsafe { cstr::bytes_at(replaced) }.len();
    let lhs = args.field_ptr(offset_of!(MapArguments, lhs));
    // SAFETY: the field's own address, and `MAXMAPLEN_BUF` is its size.
    unsafe { xstrlcpy(lhs, replaced, MAXMAPLEN_BUF) };
    if did_simplify {
        // SAFETY: as the first call.
        replaced =
            unsafe { replace_termcodes(orig_lhs, orig_lhs_len, buf, 0, nosimp, plain, cpo_val) };
        if replaced.is_null() {
            return false;
        }
        // SAFETY: as above.
        args.alt_lhs_len = unsafe { cstr::bytes_at(replaced) }.len();
        let alt = args.field_ptr(offset_of!(MapArguments, alt_lhs));
        // SAFETY: as the `lhs` copy above.
        unsafe { xstrlcpy(alt, replaced, MAXMAPLEN_BUF) };
    } else {
        args.alt_lhs_len = 0;
    }

    // SAFETY: as above — the caller's live strings and `MapArguments`.
    unsafe { set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, 0, cpo_val, mapargs) };
    true
}

/// The `rhs` half of [`set_maparg_lhs_rhs`], also used on its own by
/// `mapset()`, which brings its own already-parsed LHS.
///
/// # Safety
/// `orig_rhs` and `cpo_val` must be live, and `mapargs` writable.
pub(crate) unsafe fn set_maparg_rhs(
    orig_rhs: *const c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    sid: scid_T,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) {
    // SAFETY: the caller's promise — `mapargs` is a writable `MapArguments`.
    let mut args = unsafe { Ma::new(mapargs) };
    args.rhs_lua = rhs_lua;

    if rhs_lua != LUA_NOREF {
        // orig_rhs is not used for Lua mappings, but still has to be a
        // string; the rhs stores <Lua>ref_no<CR> for the key loop.
        let mut tmp_buf = [0 as c_char; 64];
        let at = tmp_buf.as_mut_ptr();
        let cap = (tmp_buf.len() - 1) as size_t;
        let fmt = c"%c%c%c%d\r".as_ptr();
        // SAFETY: `tmp_buf` is 64 bytes and `cap` leaves room for the NUL; the
        // format string names exactly the four arguments that follow it.
        unsafe {
            args.orig_rhs = xcalloc(1, size_of::<c_char>()).cast();
            args.orig_rhs_len = 0;
            let ke_lua = KE_LUA as c_int;
            let n = vim_snprintf(at, cap, fmt, K_SPECIAL, KS_EXTRA, ke_lua, rhs_lua);
            args.rhs_len = n as size_t;
            args.rhs = xstrdup(tmp_buf.as_ptr());
        }
        return;
    }

    args.orig_rhs_len = orig_rhs_len;
    // SAFETY: the caller's promise — `orig_rhs` is `orig_rhs_len` live bytes,
    // which is exactly what the fresh allocation has room for plus a NUL.
    unsafe {
        args.orig_rhs = xcalloc(orig_rhs_len + 1, size_of::<c_char>()).cast();
        xmemcpyz(args.orig_rhs.cast(), orig_rhs.cast(), orig_rhs_len);
    }
    // SAFETY: `orig_rhs` is NUL-terminated.
    if unsafe { strcasecmp(orig_rhs, c"<nop>".as_ptr()) } == 0 {
        // "<Nop>" means nothing.
        // SAFETY: a one-byte zeroed allocation is a single NUL char.
        args.rhs = unsafe { xcalloc(1, size_of::<c_char>()) }.cast();
        args.rhs_len = 0;
        args.rhs_is_noop = true;
    } else {
        let mut rhs_buf: *mut c_char = ptr::null_mut();
        let buf = &raw mut rhs_buf;
        let dolt = REPTERM_DO_LT as c_int;
        let plain = ptr::null_mut();
        // SAFETY: as above, plus `cpo_val` NUL-terminated; `rhs_buf` is only a
        // scratch slot `replace_termcodes` may take over.
        let replaced =
            unsafe { replace_termcodes(orig_rhs, orig_rhs_len, buf, sid, dolt, plain, cpo_val) };
        // SAFETY: `replace_termcodes` answers a NUL-terminated string.
        args.rhs_len = unsafe { cstr::bytes_at(replaced) }.len();
        // replace_termcodes may produce an empty string even when orig_rhs is
        // not empty -- a single ^V, see :h map-empty-rhs.
        args.rhs_is_noop = orig_rhs_len != 0 && args.rhs_len == 0;
        args.rhs = replaced;
    }
}

/// If `to_parse` starts with `word`, step it past `word` and the whitespace
/// after it.
///
/// # Safety
/// `*to_parse` must be live and NUL-terminated.
pub(crate) unsafe fn take_map_arg(to_parse: &mut *mut c_char, word: &[u8]) -> bool {
    // SAFETY: the caller's promise — `*to_parse` is NUL-terminated, so the
    // comparison stops inside it, and on a match the `word.len()` bytes it
    // just matched are there to step over.
    unsafe {
        if !(cstr::prefix_eq(*to_parse, word.as_ptr().cast(), word.len() as size_t)) {
            return false;
        }
        *to_parse = skipwhite(to_parse.add(word.len()));
    }
    true
}

/// Parse a string of `:map-arguments` into a [`MapArguments`].
///
/// Termcodes, backslashes and CTRL-Vs inside the extracted `{lhs}` and
/// `{rhs}` are replaced by [`set_maparg_lhs_rhs`].  `rhs` and `orig_rhs` come
/// back either null or owning, and must be freed even on failure.
///
/// `is_unmap` makes everything right of the last map argument the `{lhs}`,
/// spaces included: `:unmap` has no separate `{rhs}`.
///
/// Answers 0 on success and 1 if the arguments are invalid.
///
/// # Safety
/// `strargs` must be live and NUL-terminated, and `mapargs` writable.
pub(crate) unsafe fn str_to_mapargs(
    strargs: *const c_char,
    is_unmap: bool,
    mapargs: *mut MapArguments,
) -> c_int {
    // SAFETY: the caller's promise — `strargs` is NUL-terminated and
    // `mapargs` a writable `MapArguments`, which this zeroes before filling.
    let mut to_parse = unsafe { skipwhite(strargs) };
    // SAFETY: as above.
    unsafe { mapargs.write_bytes(0, 1) };
    let mut args = unsafe { Ma::new(mapargs) };

    // Accept <buffer>, <nowait>, <silent>, <expr>, <script> and <unique>
    // in any order.
    loop {
        // SAFETY: `to_parse` walks the caller's NUL-terminated string.
        unsafe {
            if take_map_arg(&mut to_parse, b"<buffer>") {
                args.buffer = true;
            } else if take_map_arg(&mut to_parse, b"<nowait>") {
                args.nowait = true;
            } else if take_map_arg(&mut to_parse, b"<silent>") {
                args.silent = true;
            } else if take_map_arg(&mut to_parse, b"<special>") {
                // Obsolete modifier, accepted and ignored.
            } else if take_map_arg(&mut to_parse, b"<script>") {
                args.script = true;
            } else if take_map_arg(&mut to_parse, b"<expr>") {
                args.expr = true;
            } else if take_map_arg(&mut to_parse, b"<unique>") {
                args.unique = true;
            } else {
                break;
            }
        }
    }

    // The next whitespace character ends {lhs} -- unless it is preceded
    // by a CTRL-V, or by a backslash when 'cpoptions' has no 'B'.
    let mut lhs_end = to_parse;
    let do_backslash = !cpo_has(CpoFlag::BSLASH);
    loop {
        // SAFETY: `lhs_end` walks the same NUL-terminated string, and the loop
        // stops at its NUL.
        let ch = unsafe { *lhs_end };
        if ch == 0 || (!is_unmap && ascii_iswhite(c_int::from(ch))) {
            break;
        }
        let escape = c_int::from(ch) == Ctrl_V || (do_backslash && ch == b'\\' as c_char);
        // SAFETY: `lhs_end` is on a non-NUL byte, so the byte after it is
        // readable and stepping over it stays inside the string.
        unsafe {
            if escape && c_int::from(*lhs_end.add(1)) != NUL {
                lhs_end = lhs_end.add(1); // skip CTRL-V or backslash
            }
            lhs_end = lhs_end.add(1);
        }
    }
    // SAFETY: `lhs_end` is inside the same NUL-terminated string, and both
    // pointers come from it, so the difference is its `{lhs}` length.
    let (rhs_start, orig_lhs_len) =
        unsafe { (skipwhite(lhs_end), lhs_end.offset_from(to_parse) as size_t) };

    // The given {lhs} may be longer than MAXMAPLEN before termcodes are
    // replaced ("<Space>" is longer than ' '), so copy it out first.
    if orig_lhs_len >= 256 {
        return 1;
    }
    let mut lhs_to_replace = [0 as c_char; 256];
    let dst = lhs_to_replace.as_mut_ptr().cast();
    // SAFETY: `orig_lhs_len` is below 256, the size of `lhs_to_replace`, and
    // is the length of the `{lhs}` at `to_parse`.
    unsafe { xmemcpyz(dst, to_parse.cast(), orig_lhs_len) };

    let lhs = lhs_to_replace.as_ptr();
    let cpo = p_cpo.get();
    // SAFETY: `rhs_start` is inside the caller's NUL-terminated string, `lhs`
    // names the local copy just made, and `mapargs` is the caller's struct.
    let ok = unsafe {
        let orig_rhs_len = cstr::bytes_at(rhs_start).len();
        let rhs = rhs_start;
        set_maparg_lhs_rhs(
            lhs,
            orig_lhs_len,
            rhs,
            orig_rhs_len,
            LUA_NOREF,
            cpo,
            mapargs,
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
/// # Safety
/// `mode_string` must be live and NUL-terminated.
pub(crate) unsafe fn get_map_mode_string(mode_string: *const c_char, abbr: bool) -> c_int {
    let mut p = mode_string;
    // SAFETY: the caller's promise — `mode_string` is NUL-terminated.
    if c_int::from(unsafe { *p }) == NUL {
        p = c" ".as_ptr(); // compatibility
    }

    let mut mode = 0;
    loop {
        // SAFETY: `p` walks a NUL-terminated string and the loop stops on the
        // NUL, so the step past it never happens twice.
        let modec = unsafe {
            let modec = *p as u8;
            p = p.add(1);
            modec
        };
        if modec == 0 {
            break;
        }
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
