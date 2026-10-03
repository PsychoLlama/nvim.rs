//! Listing mappings: `:map` output and command-line completion.
//!
//! [`showmap`] prints one mapping in the four-column `:map` form.
//! [`translate_mapping`] is the same rendering for completion, which
//! [`expand_mappings`] runs over the whole table for `:map <Tab>`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cmdexpand::Scored;
use crate::cmdexpand::fuzzy_sorted;
use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_J, Ctrl_V, Key, key_unescape};
use crate::memory::XString;
use crate::strings::has_char;
use crate::types::CmdIdx;
use crate::types::{CpoFlag, ExpandContext, Failed, NUL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

/// What [`set_context_in_map_cmd`] worked out for the completion that
/// follows: which modes to list, whether the command was an abbreviation
/// one, and whether `<buffer>` was given.
static EXPAND_MAPMODES: GlobalCell<c_int> = GlobalCell::new(0);
static EXPAND_ISABBREV: GlobalCell<bool> = GlobalCell::new(false);
static EXPAND_BUFFER: GlobalCell<bool> = GlobalCell::new(false);

/// Print one mapping in `:map`'s four columns: modes, LHS, flags, RHS.
///
/// `local` marks a buffer-local mapping with `@`.
pub(crate) fn showmap(mp: Mb, local: bool) {
    let rhs = &mp.m_rhs;
    let filtered = message_filtered(mp.m_keys.as_cstr())
        && message_filtered(rhs.str.as_cstr())
        && rhs
            .desc
            .as_ref()
            .is_none_or(|desc| message_filtered(desc.as_cstr()));
    if filtered {
        return;
    }

    if msg_col.get() > 0 || msg_silent.get() != 0 {
        msg_putchar(c_int::from(b'\n'));
        if got_int.get() {
            return; // 'q' typed at the MORE prompt
        }
    }

    let mapchars = map_mode_to_chars(mp.m_mode);
    let modes = cstr::in_chars(&mapchars);
    msg_str(modes);
    let mut len = modes.count_bytes();
    len += 1;
    while len <= 3 {
        msg_putchar(c_int::from(b' '));
        len += 1;
    }

    // Display the LHS, and pad to at least twelve columns.
    len = msg_display_keys(mp.m_keys.as_cstr(), true, 0) as size_t;
    loop {
        msg_putchar(c_int::from(b' '));
        len += 1;
        if len >= 12 {
            break;
        }
    }

    if mp.m_noremap == REMAP_NONE {
        msg_str_hl(c"*", HLF_8, false);
    } else if mp.m_noremap == REMAP_SCRIPT {
        msg_str_hl(c"&", HLF_8, false);
    } else {
        msg_putchar(c_int::from(b' '));
    }

    msg_putchar(c_int::from(if local { b'@' } else { b' ' }));

    // `false` below would show only things like <Up> as such on the rhs
    // and not M-x etc; `true` gets both -- webb
    if rhs.luaref() != LUA_NOREF {
        // SAFETY: the mapping's own reference; the rendering is the guard's.
        let text = unsafe { COwned::new(nlua_funcref_str(rhs.luaref())) };
        // SAFETY: a NUL-terminated rendering that outlives the call.
        msg_str_hl(unsafe { cstr::at(text.as_c_ptr()) }, HLF_8, false);
    } else if rhs.str.is_empty() {
        msg_str_hl(c"<Nop>", HLF_8, false);
    } else {
        msg_display_keys(rhs.str.as_cstr(), false, 0);
    }

    if let Some(desc) = &rhs.desc {
        msg_str(c"\n                 "); // shift to the rhs column
        msg_str(desc.as_cstr());
    }
    if p_verbose() > 0 {
        last_set_msg(mp.m_script_ctx);
    }
    msg_clr_eos();
}

/// Translate a mapping's internal LHS into the external form `:map` and
/// `:abbrev` accept, which is what command-line completion offers.
///
/// The answer can be wider than the original, so it is built in a `Vec`.
pub(crate) fn translate_mapping(str_in: &[u8], cpo: &CStr) -> Vec<u8> {
    let mut out = Vec::<u8>::new();

    let cpo_bslash = has_char(cpo, CpoFlag::BSLASH.as_c_int());
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

/// Work out what to complete when completing a mapping or abbreviation name,
/// for command `cmd` whose argument starts at `arg` in the completion's
/// line. Answers no next command.
#[allow(clippy::too_many_arguments)] // upstream's `set_context_in_*` shape
pub fn set_context_in_map_cmd(
    expand: &mut Expand,
    cmd: &CStr,
    arg: usize,
    forceit: bool,
    isabbrev: bool,
    isunmap: bool,
    cmdidx: CmdIdx,
) -> Option<usize> {
    if forceit && cmdidx != CmdIdx::map && cmdidx != CmdIdx::unmap {
        expand.context = ExpandContext::Nothing;
        return None;
    }

    if isunmap {
        let mut name = cmd.as_ptr().cast_mut();
        // SAFETY: a NUL-terminated command name, which the walk only reads.
        let mode = unsafe { get_map_mode(&raw mut name, forceit || isabbrev) };
        EXPAND_MAPMODES.set(mode);
    } else {
        let mut modes = MODE_INSERT | MODE_CMDLINE;
        if !isabbrev {
            modes |= MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
        }
        EXPAND_MAPMODES.set(modes);
    }
    EXPAND_ISABBREV.set(isabbrev);
    expand.context = ExpandContext::Mappings;
    EXPAND_BUFFER.set(false);

    // Skip the map arguments; only `<buffer>` changes what is offered.
    let all = expand.line_cstr().to_bytes().get(arg..).unwrap_or_default();
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
    expand.pattern = arg + all.len() - rest.len();

    None
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
pub fn expand_mappings(pat: &CStr, regmatch: &mut RegMatch) -> Result<Vec<XString>, Failed> {
    let fuzzy = cmdline_fuzzy_complete(pat.to_bytes());

    // Exactly one of these fills: `fuzzy` is fixed for the whole call.
    let mut scored = Vec::<Scored>::new();
    let mut plain = Vec::<XString>::new();

    // Whether `p` matches, and with what fuzzy score.
    let matched = |regex_match: &mut RegMatch, p: &CStr| -> Option<c_int> {
        if fuzzy {
            let score = fuzzy_match_str(p, pat);
            (score != FUZZY_SCORE_NONE).then_some(score)
        } else {
            vim_regexec(regex_match, p, 0).then_some(0)
        }
    };
    // In whichever of the two shapes is in use. The two vectors are
    // parameters rather than captures so the loops below can still read
    // them.
    let push = |scored: &mut Vec<Scored>, plain: &mut Vec<XString>, text: XString, score| {
        if fuzzy {
            let idx = scored.len();
            scored.push(Scored { text, score, idx });
        } else {
            plain.push(text);
        }
    };

    // First search in map modifier arguments.
    for (i, word) in EXPAND_ARGS.into_iter().enumerate() {
        if i == EXPAND_ARG_BUFFER && EXPAND_BUFFER.get() {
            continue;
        }
        if let Some(score) = matched(regmatch, word) {
            push(&mut scored, &mut plain, XString::from_cstr(word), score);
        }
    }

    // Then the mapping names themselves. Note that `<buffer>` only
    // redirects the *mapping* lookup: upstream reads the global
    // abbreviation list either way.
    let abbr = EXPAND_ISABBREV.get();
    let table = if !abbr && EXPAND_BUFFER.get() {
        MapTable::Buffer(Buf::current())
    } else {
        MapTable::Global
    };
    let collect = |mp: Mb| {
        if mp.m_simplified || mp.m_mode & EXPAND_MAPMODES.get() == 0 {
            return None;
        }
        let mut rendering = p_cpo(|cpo| translate_mapping(mp.keys(), cpo));
        if rendering.is_empty() {
            return None; // nothing to match against
        }
        // Matched as a C string, which stops at a NUL the keys may hold.
        rendering.push(0);
        let text = XString::from_cstr(cstr::in_bytes(&rendering));
        if let Some(score) = matched(regmatch, text.as_cstr()) {
            push(&mut scored, &mut plain, text, score);
        }
        None
    };
    // SAFETY: the tables are live and `collect` neither unlinks nor frees an
    // entry.
    unsafe { map_walk::<()>(table, abbr, collect) };

    let mut found = if fuzzy {
        // Fuzzy matching sorts them by score.
        fuzzy_sorted(scored, false)
    } else {
        // Sort the matches.
        plain.sort_unstable_by(|a: &XString, b: &XString| a[..].cmp(&b[..]));
        plain
    };
    // Remove duplicate entries, keeping the first of each run.
    found.dedup_by(|a, b| a == b);

    if found.is_empty() {
        Err(Failed)
    } else {
        Ok(found)
    }
}
