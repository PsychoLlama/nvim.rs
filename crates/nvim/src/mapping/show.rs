//! Listing mappings: `:map` output and command-line completion.
//!
//! [`showmap`] prints one mapping in the four-column `:map` form.
//! [`translate_mapping`] is the same rendering for completion, which
//! [`expand_mappings`] runs over the whole table for `:map <Tab>`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_J, Ctrl_V, Key, key_unescape};
use crate::memory::handoff::owned_cstr;
use crate::types::CmdIdx;
use crate::types::{CpoFlag, ExpandContext, Failed, NUL};
use crate::winlayer::Buf;
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
pub(crate) unsafe fn showmap(mp: Mb, local: bool) {
    let rhs = &mp.m_rhs;
    // SAFETY: the three strings a live mapblock owns are NUL-terminated by
    // `MapStr`'s own invariant.
    let filtered = unsafe {
        message_filtered(mp.m_keys.as_ptr())
            && message_filtered(rhs.str.as_ptr())
            && rhs
                .desc
                .as_ref()
                .is_none_or(|desc| message_filtered(desc.as_ptr()))
    };
    if filtered {
        return;
    }

    if msg_col.get() > 0 || msg_silent.get() != 0 {
        // SAFETY: a message primitive that reads nothing of ours.
        unsafe { msg_putchar(c_int::from(b'\n')) };
        if got_int.get() {
            return; // 'q' typed at the MORE prompt
        }
    }

    let mapchars = map_mode_to_chars(mp.m_mode);
    // SAFETY: `map_mode_to_chars` answers a NUL-terminated seven-byte array
    // that lives until the end of this body.
    let mut len = unsafe {
        msg_puts(mapchars.as_ptr());
        cstr::bytes_at(mapchars.as_ptr()).len()
    };
    len += 1;
    while len <= 3 {
        // SAFETY: as above.
        unsafe { msg_putchar(c_int::from(b' ')) };
        len += 1;
    }

    // Display the LHS, and pad to at least twelve columns.
    // SAFETY: `m_keys` is the mapping's own NUL-terminated LHS.
    len = unsafe { msg_outtrans_special(mp.m_keys.as_ptr(), true, 0) } as size_t;
    loop {
        // SAFETY: as above.
        unsafe { msg_putchar(c_int::from(b' ')) };
        len += 1;
        if len >= 12 {
            break;
        }
    }

    // SAFETY: static NUL-terminated markers, and the message primitives.
    unsafe {
        if mp.m_noremap == REMAP_NONE {
            msg_puts_hl(c"*".as_ptr(), HLF_8, false);
        } else if mp.m_noremap == REMAP_SCRIPT {
            msg_puts_hl(c"&".as_ptr(), HLF_8, false);
        } else {
            msg_putchar(c_int::from(b' '));
        }

        msg_putchar(c_int::from(if local { b'@' } else { b' ' }));
    }

    // `false` below would show only things like <Up> as such on the rhs
    // and not M-x etc; `true` gets both -- webb
    if rhs.luaref != LUA_NOREF {
        // SAFETY: the mapping's own reference; the rendering is the guard's.
        let text = unsafe { COwned::new(nlua_funcref_str(rhs.luaref, ptr::null_mut())) };
        // SAFETY: a NUL-terminated rendering that outlives the call.
        unsafe { msg_puts_hl(text.as_c_ptr(), HLF_8, false) };
    } else if rhs.str.is_empty() {
        // SAFETY: a static NUL-terminated marker.
        unsafe { msg_puts_hl(c"<Nop>".as_ptr(), HLF_8, false) };
    } else {
        // SAFETY: `m_str` is the mapping's own NUL-terminated RHS.
        unsafe { msg_outtrans_special(rhs.str.as_ptr(), false, 0) };
    }

    if let Some(desc) = &rhs.desc {
        // SAFETY: a static text, then the mapping's own NUL-terminated `desc`.
        unsafe {
            msg_puts(c"\n                 ".as_ptr()); // shift to the rhs column
            msg_puts(desc.as_ptr());
        }
    }
    if p_verbose.get() > 0 {
        // SAFETY: a plain copy of the mapping's script context.
        unsafe { last_set_msg(mp.m_script_ctx) };
    }
    // SAFETY: a message primitive that reads nothing of ours.
    unsafe { msg_clr_eos() };
}

/// Translate a mapping's internal LHS into the external form `:map` and
/// `:abbrev` accept, which is what command-line completion offers.
///
/// The answer can be wider than the original, so it is built in a `Vec`.
///
/// # Safety
/// `cpo_val` must be live and NUL-terminated.
pub(crate) unsafe fn translate_mapping(str_in: &[u8], cpo_val: *const c_char) -> Vec<u8> {
    let mut out = Vec::<u8>::new();

    // SAFETY: the caller's promise — `cpo_val` is NUL-terminated.
    let cpo_bslash = !unsafe { vim_strchr(cpo_val, CpoFlag::BSLASH.as_c_int()) }.is_null();
    let mut at = 0;
    while at < str_in.len() {
        let mut c = c_int::from(str_in[at]);
        // A `K_SPECIAL` escape is three bytes; upstream's tests spell that as
        // "the two bytes after this one are not the NUL".
        let three_at =
            |at: usize| matches!(str_in.get(at + 1..at + 3), Some([a, b]) if *a != 0 && *b != 0);
        'next: {
            if c == K_SPECIAL && three_at(at) {
                let mut modifiers = ModMask::NONE;
                if c_int::from(str_in[at + 1]) == KS_MODIFIER {
                    at += 2;
                    modifiers = ModMask::from_bits(c_int::from(str_in[at]));
                    at += 1;
                    c = c_int::from(str_in[at]);
                }

                if c == K_SPECIAL && three_at(at) {
                    c = key_unescape(str_in[at + 1], str_in[at + 2]);
                    if c == Key::Zero.code() {
                        c = NUL; // display <Nul> as ^@
                    }
                    at += 2;
                }
                if c < 0 || !modifiers.is_empty() {
                    // A special key.
                    let name = get_special_key_name(c, modifiers);
                    // SAFETY: `name` is a NUL-terminated rendering that
                    // outlives the call.
                    out.extend_from_slice(unsafe { cstr::bytes_at(name.as_ptr()) });
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
                let escape = if cpo_bslash { Ctrl_V } else { b'\\'.into() } as u8;
                out.push(escape);
            }
            if c != 0 {
                out.push(c as u8);
            }
        }
        at += 1;
    }
    out
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
    cmdidx: CmdIdx,
) -> *mut c_char {
    // SAFETY: the caller's promise — `xp` is a live `expand_T`.
    let mut xp = unsafe { Live::new(xp) };
    if forceit && cmdidx != CmdIdx::map && cmdidx != CmdIdx::unmap {
        xp.xp_context = ExpandContext::Nothing;
        return ptr::null_mut();
    }

    if isunmap {
        // SAFETY: the caller's promise — `cmd` is a live command name.
        let mode = unsafe { get_map_mode(&raw mut cmd, forceit || isabbrev) };
        EXPAND_MAPMODES.set(mode);
    } else {
        let mut modes = MODE_INSERT | MODE_CMDLINE;
        if !isabbrev {
            modes |= MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
        }
        EXPAND_MAPMODES.set(modes);
    }
    EXPAND_ISABBREV.set(isabbrev);
    xp.xp_context = ExpandContext::Mappings;
    EXPAND_BUFFER.set(false);

    // Skip the map arguments; only `<buffer>` changes what is offered.
    // SAFETY: the caller's promise — `arg` is NUL-terminated.
    let all = unsafe { cstr::bytes_at(arg) };
    let mut rest = all;
    'skip: loop {
        for (i, word) in CONTEXT_ARGS.into_iter().enumerate() {
            if take_map_arg(&mut rest, word) {
                if i == CONTEXT_ARG_BUFFER {
                    EXPAND_BUFFER.set(true);
                }
                continue 'skip;
            }
        }
        break;
    }
    // SAFETY: `rest` is a tail of `arg`'s own bytes.
    xp.xp_pattern = unsafe { arg.add(all.len() - rest.len()) };

    ptr::null_mut()
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
/// Answers `Ok` if any matched, `Err` otherwise.
///
/// # Safety
/// Every pointer argument must be live; `matches` and `numMatches` are
/// written unconditionally.
pub unsafe fn expand_mappings(
    pat: *mut c_char,
    regmatch: *mut regmatch_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise — `pat` is a live, NUL-terminated pattern.
    let fuzzy = unsafe { cmdline_fuzzy_complete(pat) };

    // SAFETY: the caller's promise — both out-parameters are writable.
    unsafe {
        *numMatches = 0; // return values in case of FAIL
        *matches = ptr::null_mut();
    }

    // Exactly one of these fills: `fuzzy` is fixed for the whole call.
    let mut scored = Vec::<fuzmatch_str_T>::new();
    let mut plain = Vec::<*mut c_char>::new();

    // Whether `p` matches, and with what fuzzy score.
    let matched = |p: *mut c_char| -> Option<c_int> {
        if fuzzy {
            // SAFETY: `p` and `pat` are both live and NUL-terminated.
            let score = unsafe { fuzzy_match_str(cstr::at(p), cstr::at(pat)) };
            (score != FUZZY_SCORE_NONE).then_some(score)
        } else {
            // SAFETY: the caller's promise — `regmatch` is a live, compiled
            // match — and `p` is NUL-terminated.
            unsafe { vim_regexec(regmatch, p, 0) }.then_some(0)
        }
    };
    // C's `GA_APPEND`, in whichever of the two element shapes is in use.
    // The two vectors are parameters rather than captures so the loops below
    // can still read them.
    let push = |scored: &mut Vec<fuzmatch_str_T>, plain: &mut Vec<*mut c_char>, s, score| {
        if fuzzy {
            let idx = c_int::try_from(scored.len()).expect("a match count fits a c_int");
            scored.push(fuzmatch_str_T { idx, str: s, score });
        } else {
            plain.push(s);
        }
    };

    // First search in map modifier arguments.
    for (i, word) in EXPAND_ARGS.into_iter().enumerate() {
        if i == EXPAND_ARG_BUFFER && EXPAND_BUFFER.get() {
            continue;
        }
        let p = word.as_ptr().cast_mut();
        if let Some(score) = matched(p) {
            // The copy is owned by the growarray from here on.
            push(
                &mut scored,
                &mut plain,
                owned_cstr(word.to_bytes().to_vec()),
                score,
            );
        }
    }

    // Then the mapping names themselves. Note that `<buffer>` only
    // redirects the *mapping* lookup: upstream reads the global
    // abbreviation list either way.
    let abbr = EXPAND_ISABBREV.get();
    let table = if !abbr && EXPAND_BUFFER.get() {
        // SAFETY: `curbuf` is set from startup to exit.
        MapTable::Buffer(unsafe { Buf::current() })
    } else {
        MapTable::Global
    };
    let collect = |mp: Mb| {
        if mp.m_simplified || mp.m_mode & EXPAND_MAPMODES.get() == 0 {
            return None;
        }
        // SAFETY: `'cpoptions'` is NUL-terminated.
        let mut rendering = unsafe { translate_mapping(mp.keys(), p_cpo.get()) };
        if rendering.is_empty() {
            return None; // nothing to match against
        }
        // Matched as a C string out of this frame's own buffer, and only
        // handed to the growarray -- which owns it from then on -- if it hit.
        rendering.push(0);
        if let Some(score) = matched(rendering.as_mut_ptr().cast()) {
            rendering.pop();
            push(&mut scored, &mut plain, owned_cstr(rendering), score);
        }
        None
    };
    // SAFETY: the tables are live and `collect` neither unlinks nor frees an
    // entry.
    unsafe { map_walk::<()>(table, abbr, collect) };

    let found = if fuzzy { scored.len() } else { plain.len() };
    if found == 0 {
        return Err(Failed);
    }
    let found = c_int::try_from(found).expect("a match count fits a c_int");

    // Both handovers give the receiver a boxed slice, which is `xfree`-able
    // because the tree's allocator is libc's (`allocator.rs`).
    // SAFETY: both out-parameters are the caller's writable slots.
    let mut count = unsafe {
        if fuzzy {
            let raw = Box::into_raw(scored.into_boxed_slice()).cast::<fuzmatch_str_T>();
            fuzzymatches_to_strmatches(raw, matches, found, false);
        } else {
            *matches = Box::into_raw(plain.into_boxed_slice()).cast::<*mut c_char>();
        }
        *numMatches = found;
        found
    };
    if count > 1 {
        // SAFETY: `*matches` now holds `count` NUL-terminated strings, which
        // is what the sort and the comparison below read.
        unsafe {
            // Sort the matches; fuzzy matching already sorted them.
            if !fuzzy {
                sort_strings(*matches, count);
            }
            // Remove duplicate entries, keeping the first of each run.  The
            // one `xfree` this module still makes: past the handover above the
            // array and its strings belong to the *caller*, and `xfree` is the
            // release its own code will use on the rest of them.
            let items = core::slice::from_raw_parts_mut(*matches, count as usize);
            let mut kept = 0;
            for read in 1..items.len() {
                if !cstr::eq(items[kept], items[read]) {
                    kept += 1;
                    items[kept] = items[read];
                } else {
                    xfree(items[read].cast());
                    count -= 1;
                }
            }
        }
    }

    // SAFETY: the caller's writable out-parameter.
    unsafe { *numMatches = count };
    if count == 0 { Err(Failed) } else { Ok(()) }
}
