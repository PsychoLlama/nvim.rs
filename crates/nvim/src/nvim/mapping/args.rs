//! Parsing `:map` arguments into a [`MapArguments`].
//!
//! [`str_to_mapargs`] splits `<buffer><expr>… {lhs} {rhs}` and hands both
//! halves to [`set_maparg_lhs_rhs`], which runs them through
//! `replace_termcodes`.  [`get_map_mode`], [`get_map_mode_string`] and
//! [`map_mode_to_chars`] are the other direction of the same question — which
//! modes a command name or a `maparg()`-style mode string names, and how a
//! set of modes is spelled back.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{c_char, c_int};
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
    unsafe {
        (*mapargs).rhs_lua = rhs_lua;

        // If the mapping was given as ^V<C_UP>, replace the term codes with
        // the appropriate two bytes; if it is a shifted special key, unshift
        // it too, giving another two. `replace_termcodes` may answer with
        // `lhs_buf` or with memory of its own, which is why it takes the
        // buffer's address; it also removes CTRL-Vs and sometimes
        // backslashes. If something like <C-H> simplifies to 0x08 we take a
        // second, unsimplified copy and mark the pair.
        let mut lhs_buf = [0 as c_char; 128];
        let mut did_simplify = false;
        let flags = REPTERM_FROM_PART as c_int | REPTERM_DO_LT as c_int;
        let mut bufarg = lhs_buf.as_mut_ptr();
        let mut replaced = replace_termcodes(
            orig_lhs,
            orig_lhs_len,
            &raw mut bufarg,
            0,
            flags,
            &raw mut did_simplify,
            cpo_val,
        );
        if replaced.is_null() {
            return false;
        }
        (*mapargs).lhs_len = strlen(replaced);
        xstrlcpy((&raw mut (*mapargs).lhs).cast(), replaced, MAXMAPLEN_BUF);
        if did_simplify {
            replaced = replace_termcodes(
                orig_lhs,
                orig_lhs_len,
                &raw mut bufarg,
                0,
                flags | REPTERM_NO_SIMPLIFY as c_int,
                ptr::null_mut(),
                cpo_val,
            );
            if replaced.is_null() {
                return false;
            }
            (*mapargs).alt_lhs_len = strlen(replaced);
            xstrlcpy(
                (&raw mut (*mapargs).alt_lhs).cast(),
                replaced,
                MAXMAPLEN_BUF,
            );
        } else {
            (*mapargs).alt_lhs_len = 0;
        }

        set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, 0, cpo_val, mapargs);
        true
    }
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
    unsafe {
        (*mapargs).rhs_lua = rhs_lua;

        if rhs_lua != LUA_NOREF {
            // orig_rhs is not used for Lua mappings, but still has to be a
            // string; the rhs stores <Lua>ref_no<CR> for the key loop.
            let mut tmp_buf = [0 as c_char; 64];
            (*mapargs).orig_rhs = xcalloc(1, size_of::<c_char>()).cast();
            (*mapargs).orig_rhs_len = 0;
            (*mapargs).rhs_len = vim_snprintf(
                tmp_buf.as_mut_ptr(),
                (tmp_buf.len() - 1) as size_t,
                c"%c%c%c%d\r".as_ptr(),
                K_SPECIAL,
                KS_EXTRA,
                KE_LUA as c_int,
                rhs_lua,
            ) as size_t;
            (*mapargs).rhs = xstrdup(tmp_buf.as_ptr());
            return;
        }

        (*mapargs).orig_rhs_len = orig_rhs_len;
        (*mapargs).orig_rhs = xcalloc(orig_rhs_len + 1, size_of::<c_char>()).cast();
        xmemcpyz((*mapargs).orig_rhs.cast(), orig_rhs.cast(), orig_rhs_len);
        if strcasecmp(orig_rhs, c"<nop>".as_ptr()) == 0 {
            // "<Nop>" means nothing.
            (*mapargs).rhs = xcalloc(1, size_of::<c_char>()).cast(); // single NUL char
            (*mapargs).rhs_len = 0;
            (*mapargs).rhs_is_noop = true;
        } else {
            let mut rhs_buf: *mut c_char = ptr::null_mut();
            let replaced = replace_termcodes(
                orig_rhs,
                orig_rhs_len,
                &raw mut rhs_buf,
                sid,
                REPTERM_DO_LT as c_int,
                ptr::null_mut(),
                cpo_val,
            );
            (*mapargs).rhs_len = strlen(replaced);
            // replace_termcodes may produce an empty string even when
            // orig_rhs is not empty -- a single ^V, see :h map-empty-rhs.
            (*mapargs).rhs_is_noop = orig_rhs_len != 0 && (*mapargs).rhs_len == 0;
            (*mapargs).rhs = replaced;
        }
    }
}

/// If `to_parse` starts with `word`, step it past `word` and the whitespace
/// after it.
///
/// # Safety
/// `*to_parse` must be live and NUL-terminated.
unsafe fn take_map_arg(to_parse: &mut *mut c_char, word: &[u8]) -> bool {
    unsafe {
        if strncmp(*to_parse, word.as_ptr().cast(), word.len() as size_t) != 0 {
            return false;
        }
        *to_parse = skipwhite(to_parse.add(word.len()));
        true
    }
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
    unsafe {
        let mut to_parse = skipwhite(strargs);
        mapargs.write_bytes(0, 1);

        // Accept <buffer>, <nowait>, <silent>, <expr>, <script> and <unique>
        // in any order.
        loop {
            if take_map_arg(&mut to_parse, b"<buffer>") {
                (*mapargs).buffer = true;
            } else if take_map_arg(&mut to_parse, b"<nowait>") {
                (*mapargs).nowait = true;
            } else if take_map_arg(&mut to_parse, b"<silent>") {
                (*mapargs).silent = true;
            } else if take_map_arg(&mut to_parse, b"<special>") {
                // Obsolete modifier, accepted and ignored.
            } else if take_map_arg(&mut to_parse, b"<script>") {
                (*mapargs).script = true;
            } else if take_map_arg(&mut to_parse, b"<expr>") {
                (*mapargs).expr = true;
            } else if take_map_arg(&mut to_parse, b"<unique>") {
                (*mapargs).unique = true;
            } else {
                break;
            }
        }

        // The next whitespace character ends {lhs} -- unless it is preceded
        // by a CTRL-V, or by a backslash when 'cpoptions' has no 'B'.
        let mut lhs_end = to_parse;
        let do_backslash = vim_strchr(p_cpo.get(), CPO_BSLASH).is_null();
        while *lhs_end != 0 && (is_unmap || !ascii_iswhite(c_int::from(*lhs_end))) {
            let escape =
                c_int::from(*lhs_end) == Ctrl_V || (do_backslash && *lhs_end == b'\\' as c_char);
            if escape && c_int::from(*lhs_end.add(1)) != NUL {
                lhs_end = lhs_end.add(1); // skip CTRL-V or backslash
            }
            lhs_end = lhs_end.add(1);
        }
        let rhs_start = skipwhite(lhs_end);

        // The given {lhs} may be longer than MAXMAPLEN before termcodes are
        // replaced ("<Space>" is longer than ' '), so copy it out first.
        let orig_lhs_len = lhs_end.offset_from(to_parse) as size_t;
        if orig_lhs_len >= 256 {
            return 1;
        }
        let mut lhs_to_replace = [0 as c_char; 256];
        xmemcpyz(
            lhs_to_replace.as_mut_ptr().cast(),
            to_parse.cast(),
            orig_lhs_len,
        );

        let orig_rhs_len = strlen(rhs_start);
        if !set_maparg_lhs_rhs(
            lhs_to_replace.as_ptr(),
            orig_lhs_len,
            rhs_start,
            orig_rhs_len,
            LUA_NOREF,
            p_cpo.get(),
            mapargs,
        ) {
            return 1;
        }
        if (*mapargs).lhs_len > MAXMAPLEN as size_t {
            return 1;
        }
        0
    }
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
    unsafe {
        let mut p = *cmdp;
        let modec = *p as u8;
        p = p.add(1);
        let mode = match modec {
            b'i' => MODE_INSERT,                         // :imap
            b'l' => MODE_LANGMAP,                        // :lmap
            b'c' => MODE_CMDLINE,                        // :cmap
            b'n' if *p != b'o' as c_char => MODE_NORMAL, // :nmap, avoiding :noremap
            b'v' => MASK_V,                              // :vmap
            b'x' => MODE_VISUAL,                         // :xmap
            b's' => MODE_SELECT,                         // :smap
            b'o' => MODE_OP_PENDING,                     // :omap
            b't' => MODE_TERMINAL,                       // :tmap
            _ => {
                p = p.sub(1);
                if forceit {
                    MASK_BANG // :map!
                } else {
                    MASK_MAP // :map
                }
            }
        };
        *cmdp = p;
        mode
    }
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
    unsafe {
        let mut p = mode_string;
        if c_int::from(*p) == NUL {
            p = c" ".as_ptr(); // compatibility
        }

        let mut mode = 0;
        loop {
            let modec = *p as u8;
            p = p.add(1);
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
}
