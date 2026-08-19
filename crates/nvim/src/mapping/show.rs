//! Listing mappings: `:map` output and command-line completion.
//!
//! [`showmap`] prints one mapping in the four-column `:map` form.
//! [`translate_mapping`] is the same rendering for completion, which
//! [`ExpandMappings`] runs over the whole table for `:map <Tab>`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_J, Ctrl_V, key_unescape};
use crate::types::{CMD_map, CMD_unmap, FAIL, OK};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// What [`set_context_in_map_cmd`] worked out for the completion that
/// follows: which modes to list, whether the command was an abbreviation
/// one, and whether `<buffer>` was given.
static EXPAND_MAPMODES: GlobalCell<c_int> = GlobalCell::new(0);
static EXPAND_ISABBREV: GlobalCell<bool> = GlobalCell::new(false);
static EXPAND_BUFFER: GlobalCell<bool> = GlobalCell::new(false);

/// Print one mapping in `:map`'s four columns: modes, LHS, flags, RHS.
///
/// `local` marks a buffer-local mapping with `@`.
///
/// # Safety
/// `mp` must be a live mapblock.
pub(crate) unsafe fn showmap(mp: *mut mapblock_T, local: bool) {
    unsafe {
        if message_filtered((*mp).m_keys)
            && message_filtered((*mp).m_str)
            && ((*mp).m_desc.is_null() || message_filtered((*mp).m_desc))
        {
            return;
        }

        if msg_col.get() > 0 || msg_silent.get() != 0 {
            msg_putchar(c_int::from(b'\n'));
            if got_int.get() {
                return; // 'q' typed at the MORE prompt
            }
        }

        let mut mapchars = map_mode_to_chars((*mp).m_mode);
        msg_puts(mapchars.as_ptr());
        let mut len = strlen(mapchars.as_mut_ptr());
        len += 1;
        while len <= 3 {
            msg_putchar(c_int::from(b' '));
            len += 1;
        }

        // Display the LHS, and pad to at least twelve columns.
        len = msg_outtrans_special((*mp).m_keys, true, 0) as size_t;
        loop {
            msg_putchar(c_int::from(b' '));
            len += 1;
            if len >= 12 {
                break;
            }
        }

        if (*mp).m_noremap == REMAP_NONE {
            msg_puts_hl(c"*".as_ptr(), HLF_8, false);
        } else if (*mp).m_noremap == REMAP_SCRIPT {
            msg_puts_hl(c"&".as_ptr(), HLF_8, false);
        } else {
            msg_putchar(c_int::from(b' '));
        }

        msg_putchar(c_int::from(if local { b'@' } else { b' ' }));

        // `false` below would show only things like <Up> as such on the rhs
        // and not M-x etc; `true` gets both -- webb
        if (*mp).m_luaref != LUA_NOREF {
            let str = nlua_funcref_str((*mp).m_luaref, ptr::null_mut());
            msg_puts_hl(str, HLF_8, false);
            xfree(str.cast());
        } else if c_int::from(*(*mp).m_str) == NUL {
            msg_puts_hl(c"<Nop>".as_ptr(), HLF_8, false);
        } else {
            msg_outtrans_special((*mp).m_str, false, 0);
        }

        if !(*mp).m_desc.is_null() {
            msg_puts(c"\n                 ".as_ptr()); // shift to the rhs column
            msg_puts((*mp).m_desc);
        }
        if p_verbose.get() > 0 {
            last_set_msg((*mp).m_script_ctx);
        }
        msg_clr_eos();
    }
}

/// Translate a mapping's internal LHS into the external form `:map` and
/// `:abbrev` accept, which is what command-line completion offers.
///
/// The answer can be wider than the original, so it is built in a growarray;
/// the caller owns the string that comes back.
///
/// # Safety
/// Both strings must be live and NUL-terminated.
pub(crate) unsafe fn translate_mapping(
    str_in: *const c_char,
    cpo_val: *const c_char,
) -> *mut c_char {
    unsafe {
        let mut ga: garray_T = core::mem::zeroed();
        ga_init(&raw mut ga, 1, 40);

        let cpo_bslash = !vim_strchr(cpo_val, CPO_BSLASH).is_null();
        let mut str = str_in.cast::<u8>();
        while *str != 0 {
            let mut c = c_int::from(*str);
            'next: {
                if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
                    let mut modifiers = 0;
                    if c_int::from(*str.add(1)) == KS_MODIFIER {
                        str = str.add(2);
                        modifiers = c_int::from(*str);
                        str = str.add(1);
                        c = c_int::from(*str);
                    }

                    if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
                        c = key_unescape(*str.add(1), *str.add(2));
                        if c == K_ZERO {
                            c = NUL; // display <Nul> as ^@
                        }
                        str = str.add(2);
                    }
                    if c < 0 || modifiers != 0 {
                        // A special key.
                        ga_concat(&raw mut ga, get_special_key_name(c, modifiers));
                        break 'next;
                    }
                }

                if c == c_int::from(b' ')
                    || c == c_int::from(b'\t')
                    || c == Ctrl_J
                    || c == Ctrl_V
                    || c == c_int::from(b'<')
                    || (c == c_int::from(b'\\') && !cpo_bslash)
                {
                    ga_append(
                        &raw mut ga,
                        if cpo_bslash { Ctrl_V } else { b'\\'.into() } as u8,
                    );
                }
                if c != 0 {
                    ga_append(&raw mut ga, c as u8);
                }
            }
            str = str.add(1);
        }
        ga_append(&raw mut ga, NUL as u8);
        ga.ga_data.cast()
    }
}

/// The `:map-arguments` that may precede the `{lhs}` on a completed command
/// line, in the order upstream tries them.
const CONTEXT_ARGS: [&[u8]; 7] = [
    b"<buffer>",
    b"<unique>",
    b"<nowait>",
    b"<silent>",
    b"<special>",
    b"<script>",
    b"<expr>",
];

/// Index of `<buffer>` in [`CONTEXT_ARGS`], the one that changes what is
/// offered.
const CONTEXT_ARG_BUFFER: usize = 0;

/// Work out what to complete when completing a mapping or abbreviation name.
///
/// # Safety
/// `xp`, `cmd` and `arg` must be live.
#[allow(clippy::too_many_arguments)] // upstream's `set_context_in_*` shape
pub unsafe fn set_context_in_map_cmd(
    xp: *mut expand_T,
    mut cmd: *mut c_char,
    mut arg: *mut c_char,
    forceit: bool,
    isabbrev: bool,
    isunmap: bool,
    cmdidx: cmdidx_T,
) -> *mut c_char {
    unsafe {
        if forceit && cmdidx != CMD_map && cmdidx != CMD_unmap {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return ptr::null_mut();
        }

        if isunmap {
            EXPAND_MAPMODES.set(get_map_mode(&raw mut cmd, forceit || isabbrev));
        } else {
            let mut modes = MODE_INSERT | MODE_CMDLINE;
            if !isabbrev {
                modes |= MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
            }
            EXPAND_MAPMODES.set(modes);
        }
        EXPAND_ISABBREV.set(isabbrev);
        (*xp).xp_context = EXPAND_MAPPINGS as c_int;
        EXPAND_BUFFER.set(false);

        // Skip the map arguments; only `<buffer>` changes what is offered.
        'skip: loop {
            for (i, word) in CONTEXT_ARGS.into_iter().enumerate() {
                if take_map_arg(&mut arg, word) {
                    if i == CONTEXT_ARG_BUFFER {
                        EXPAND_BUFFER.set(true);
                    }
                    continue 'skip;
                }
            }
            break;
        }
        (*xp).xp_pattern = arg;

        ptr::null_mut()
    }
}

/// The map arguments `:map <Tab>` offers, in upstream's order.  `<buffer>` is
/// dropped once it has already been given.
const EXPAND_ARGS: [&CStr; 7] = [
    c"<silent>",
    c"<unique>",
    c"<script>",
    c"<expr>",
    c"<buffer>",
    c"<nowait>",
    c"<special>",
];

/// Index of `<buffer>` in [`EXPAND_ARGS`].
const EXPAND_ARG_BUFFER: usize = 4;

/// Find all mapping/abbreviation names matching `regmatch`, for command-line
/// completion of `:[un]map` and `:[un]abbrev` in all modes.
///
/// Answers `OK` if any matched, `FAIL` otherwise.
///
/// # Safety
/// Every pointer argument must be live; `matches` and `numMatches` are
/// written unconditionally.
pub unsafe fn ExpandMappings(
    pat: *mut c_char,
    regmatch: *mut regmatch_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe {
        let fuzzy = cmdline_fuzzy_complete(pat);

        *numMatches = 0; // return values in case of FAIL
        *matches = ptr::null_mut();

        let mut ga: garray_T = core::mem::zeroed();
        let itemsize = if fuzzy {
            size_of::<fuzmatch_str_T>()
        } else {
            size_of::<*mut c_char>()
        };
        ga_init(&raw mut ga, itemsize as c_int, 3);

        // Whether `p` matches, and with what fuzzy score.
        let matched = |p: *mut c_char| -> Option<c_int> {
            if fuzzy {
                let score = fuzzy_match_str(p, pat);
                (score != FUZZY_SCORE_NONE).then_some(score)
            } else {
                vim_regexec(regmatch, p, 0).then_some(0)
            }
        };
        // C's `GA_APPEND`, in whichever of the two element shapes is in use.
        // `ga` is a parameter rather than a capture so the loops below can
        // still read it.
        let push = |ga: &mut garray_T, s: *mut c_char, score: c_int| {
            ga_grow(ga, 1);
            if fuzzy {
                *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) = fuzmatch_str_T {
                    idx: ga.ga_len,
                    str: s,
                    score,
                };
            } else {
                *(ga.ga_data as *mut *mut c_char).offset(ga.ga_len as isize) = s;
            }
            ga.ga_len += 1;
        };

        // First search in map modifier arguments.
        for (i, word) in EXPAND_ARGS.into_iter().enumerate() {
            if i == EXPAND_ARG_BUFFER && EXPAND_BUFFER.get() {
                continue;
            }
            let p = word.as_ptr().cast_mut();
            if let Some(score) = matched(p) {
                push(&mut ga, xstrdup(p), score);
            }
        }

        // Then the mapping names themselves. Note that `<buffer>` only
        // redirects the *mapping* lookup: upstream reads the global
        // abbreviation list either way.
        let abbr = EXPAND_ISABBREV.get();
        let table = if !abbr && EXPAND_BUFFER.get() {
            MapTable::Buffer(curbuf.get())
        } else {
            MapTable::Global
        };
        map_walk::<()>(table, abbr, |mp| {
            if (*mp).m_simplified != 0 || (*mp).m_mode & EXPAND_MAPMODES.get() == 0 {
                return None;
            }
            let p = translate_mapping((*mp).m_keys, p_cpo.get());
            if p.is_null() {
                return None;
            }
            match matched(p) {
                Some(score) => push(&mut ga, p, score),
                None => xfree(p.cast()),
            }
            None
        });

        if ga.ga_len == 0 {
            return FAIL;
        }

        if fuzzy {
            fuzzymatches_to_strmatches(ga.ga_data.cast(), matches, ga.ga_len, false);
        } else {
            *matches = ga.ga_data.cast();
        }
        *numMatches = ga.ga_len;

        let mut count = *numMatches;
        if count > 1 {
            // Sort the matches; fuzzy matching already sorted them.
            if !fuzzy {
                sort_strings(*matches, count);
            }
            // Remove duplicate entries, keeping the first of each run.
            let items = core::slice::from_raw_parts_mut(*matches, count as usize);
            let mut kept = 0;
            for read in 1..items.len() {
                if strcmp(items[kept], items[read]) != 0 {
                    kept += 1;
                    items[kept] = items[read];
                } else {
                    xfree(items[read].cast());
                    count -= 1;
                }
            }
        }

        *numMatches = count;
        if count == 0 { FAIL } else { OK }
    }
}
