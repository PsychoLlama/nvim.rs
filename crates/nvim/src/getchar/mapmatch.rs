//! Matching the typeahead against the mapping table.
//!
//! [`handle_mapping`] is the heart of it: it walks the maphash bucket for the
//! first typeahead byte looking for the longest mapping whose LHS is a prefix
//! of what is buffered, decides between waiting for more input and giving up
//! (`'timeout'`/`'timeoutlen'`), and on a match replaces the LHS with the RHS
//! in the typeahead.

#![forbid(unsafe_code)]

use super::*;
use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_N, Ctrl_P, KE_IGNORE, KE_PLUG, KE_SNR, key_escape};
use crate::mbyte::{char_len, encode_char, unescape_at};
use crate::normal::{set_visual_select, visual_active, visual_select};
use crate::option::vars::P_LANGMAP;
use crate::types::MapStr;
use crate::types::{Failed, MB_MAXBYTES, NUL};
use core::ffi::c_int;

/// Whether C's `LANGMAP_ADJUST` maps keys through `'langmap'` now: the
/// half of the macro that does not depend on the key, which nothing in a
/// mapping walk changes -- so the walk asks once, not per typeahead byte
/// per candidate mapping.
///
/// `'langmap'` only applies to *typed* keys, and `'langremap'` decides
/// whether it also applies to the result of a mapping. `maplen` is the
/// typeahead's.
fn langmap_applies(maplen: c_int) -> bool {
    // `first_byte` and not the projecting `p_langmap(|value| ...)` reader:
    // the projection has to find the value's NUL to hand out a `&CStr`, and
    // asking that here cost 616 M instructions on `inbench` (`mapresolve`
    // +24%). This is upstream's `*p_langmap`.
    P_LANGMAP.first_byte() != 0
        && (p_lrm()
            || if vgetc_busy.get() != 0 {
                maplen == 0
            } else {
                KeyTyped.get()
            })
        && KeyStuffed.get() == 0
}

/// The other half of `LANGMAP_ADJUST`: `c` through `'langmap'`, for a key
/// [`langmap_applies`] said it applies to.
#[inline(always)]
fn langmap_map(c: c_int) -> c_int {
    match usize::try_from(c) {
        Ok(byte) if byte < 256 => langmap_mapchar.with(|map| c_int::from(map[byte])),
        Ok(_) => langmap_adjust_mb(c),
        Err(_) => c,
    }
}

/// Replace `slen` bytes at `offset` in the typeahead with `string`.
///
/// `offset + slen` must be within the typeahead.
pub(crate) fn put_string_in_typebuf(
    offset: c_int,
    slen: c_int,
    string: &[u8],
) -> Result<(), Failed> {
    let new_slen = c_int::try_from(string.len()).expect("a key is a few bytes");
    let extra = new_slen - slen;
    if extra < 0 {
        // Remove the matched characters, taking care of tb_noremap.
        del_typebuf(-extra, offset);
    } else if extra > 0 {
        // Insert the extra space we need.
        let from = usize::try_from(slen).expect("a positive length");
        ins_typebuf(&string[from..], REMAP_YES, offset, false, false)?;
    }
    // The storage may have moved under the delete or the insert; the copy
    // goes to wherever the bytes are now.
    typeahead().set_bytes(offset, string);
    Ok(())
}

/// Whether the typeahead starts with a key that Insert-mode completion uses,
/// including the form with a Ctrl modifier.
pub(crate) fn at_ins_compl_key() -> bool {
    let tb = typeahead();
    let mut c = tb.byte(0);

    if tb.len() > 3
        && c == K_SPECIAL
        && tb.byte(1) == KS_MODIFIER
        && ModMask::from_bits(tb.byte(2)).has(ModMask::CTRL)
    {
        c = tb.byte(3) & 0x1f;
    }
    (ctrl_x_mode_not_default() && vim_is_ctrl_x_key(c))
        || (compl_status_local() && (c == Ctrl_N || c == Ctrl_P))
}

/// Fold a modifier at the front of the typeahead into the key it modifies,
/// where a single code stands for the combination.
///
/// Looks at offsets 0 through `max_offset - 1`. Answers how many bytes the
/// replacement occupies, 0 when nothing changed, or -1 on failure.
pub(crate) fn check_simplify_modifier(max_offset: c_int) -> c_int {
    // Terminal mode wants the full modifiers, so that the key can be
    // encoded for the child process.
    if State.get() & MODE_TERMINAL != 0 || no_reduce_keys.get() > 0 {
        return 0;
    }

    for offset in 0..max_offset {
        let tb = typeahead();
        if offset + 3 >= tb.len() {
            break;
        }
        if tb.byte(offset) != K_SPECIAL || tb.byte(offset + 1) != KS_MODIFIER {
            continue;
        }

        // A modifier that was not used for a mapping: apply it to the
        // ASCII key. Shift would already have been applied.
        let mut modifier = ModMask::from_bits(tb.byte(offset + 2));
        let c = tb.byte(offset + 3);
        let new_c = merge_modifiers(c, &mut modifier);
        if new_c == c {
            continue;
        }

        if offset == 0 {
            // At the start: remember the character and mod_mask from
            // before the merge. In some cases -- at the hit-return
            // prompt, say -- they are put back into the typeahead.
            vgetc_char.set(c);
            vgetc_mod_mask.set(ModMask::from_bits(tb.byte(2)));
        }

        let mut new_string = [0u8; MB_MAXBYTES];
        let len = if new_c < 0 {
            new_string[..3].copy_from_slice(&key_escape(new_c));
            3
        } else {
            encode_char(new_c, &mut new_string)
        };
        let new_string = &new_string[..len];

        // The offsets below are within the typeahead: `offset + 3` was just
        // tested against its length.
        let ok = if modifier.is_empty() {
            // The whole three-byte prefix plus the key becomes the key.
            put_string_in_typebuf(offset, 4, new_string)
        } else {
            // Some of the modifier is left over; keep the prefix.
            tb.set_byte(offset + 2, modifier.bits() as u8);
            put_string_in_typebuf(offset + 3, 1, new_string)
        };
        if ok.is_err() {
            return -1;
        }
        return c_int::try_from(len).expect("a key is a few bytes");
    }
    0
}

/// A mapping the walk is considering: where it is, and the two flags the
/// walk compares against later candidates.
#[derive(Copy, Clone)]
struct Candidate {
    at: MapAt,
    nowait: bool,
    mode: c_int,
}

/// What the maphash walk found.
struct MapSearch {
    /// The best candidate, or `None` when there is none.
    mp: Option<Candidate>,
    /// The match length, or `KEYLEN_PART_MAP` when a longer mapping might
    /// still match once more is typed.
    keylen: c_int,
    /// The length of the longest *full* match.
    mp_match_len: c_int,
    /// The longest prefix any non-matching entry agreed on, which is how far
    /// `check_simplify_modifier` has to look.
    max_mlen: c_int,
}

/// Walk the maphash buckets for `tb_c1` looking for the longest mapping whose
/// LHS is a prefix of the typeahead.
///
/// The longest *full* match is remembered, but a full match is only accepted
/// when nothing partly matches — which is what lets `aa` and `aaa` both be
/// mapped. `waited` is whether the wait for more input already timed out.
fn search_maphash(
    keylen: c_int,
    waited: bool,
    local_state: c_int,
    is_plug_map: bool,
    mut tb_c1: c_int,
) -> MapSearch {
    let mut found = MapSearch {
        mp: None,
        keylen,
        mp_match_len: 0,
        max_mlen: 0,
    };
    // Nothing in the walk changes the typeahead or the 'langmap' state:
    // read them once.
    let maplen = typeahead().maplen();
    let langmap = langmap_applies(maplen);

    // How many bytes at the start of a key are not subject to 'langmap':
    // the two that follow a K_SPECIAL.
    let nolmaplen = if tb_c1 == K_SPECIAL {
        2
    } else {
        if langmap
            && State.get() & (MODE_CMDLINE | MODE_INSERT) == 0
            && get_real_state() != MODE_SELECT
        {
            tb_c1 = langmap_map(tb_c1);
        }
        0
    };

    let mut mp_match: Option<Candidate> = None;
    // Buffer-local mappings first, then the global ones; each bucket newest
    // first, as upstream's lists run. The typeahead is lent for the walk,
    // which runs no user code.
    let partial = typeahead().with_valid(|tb, noremaps| {
        let tb_len = c_int::try_from(tb.len()).expect("the typeahead is shorter than an int");
        with_maphash_lists(local_state, tb_c1, |hash, lists| {
            for (table, list) in lists {
                for (index, mp) in list.iter().enumerate().rev() {
                    // The LHS with its NUL: the walk below stops there,
                    // because no typeahead byte is one.
                    let lhs = mp.m_keys.as_bytes_with_nul();
                    let keylen = c_int::try_from(lhs.len() - 1).expect("a short LHS");
                    // Only consider an entry whose first character matches
                    // and that is for the current state. Skip `:lmap`
                    // mappings when keys were mapped.
                    if c_int::from(lhs[0]) != tb_c1
                        || mp.m_mode & local_state == 0
                        || (mp.m_mode & MODE_LANGMAP != 0 && maplen != 0)
                    {
                        continue;
                    }

                    // How many bytes of the typeahead this mapping agrees
                    // with.
                    let mut nomap = nolmaplen;
                    let mut modifiers = ModMask::NONE;
                    // Upstream keeps `modifiers == 1` as the marker for "the
                    // byte after this one is the modifier byte", which is
                    // not a modifier bit at all (they start at 0x02); it is
                    // a separate flag here.
                    let mut modifier_next = false;
                    // The typeahead against the LHS, the NUL included, from
                    // their second bytes; both cut to the shorter, so that
                    // neither index is bounds-checked per byte.
                    let limit = tb.len().min(lhs.len());
                    let (typed_bytes, lhs_bytes) = (&tb[..limit], &lhs[..limit]);
                    let mut at = 1;
                    while at < limit {
                        let (typed, key) = (typed_bytes[at], lhs_bytes[at]);
                        let mut c2 = c_int::from(typed);
                        if nomap > 0 {
                            if nomap == 2 && c2 == KS_MODIFIER {
                                modifier_next = true;
                            } else if nomap == 1 && modifier_next {
                                modifiers = ModMask::from_bits(c2);
                            }
                            nomap -= 1;
                        } else {
                            if c2 == K_SPECIAL {
                                nomap = 2;
                            } else if langmap
                                && (!modifiers.has(ModMask::CTRL)
                                    || merge_modifiers(c2, &mut modifiers) == c2)
                            {
                                // Only apply 'langmap' when merging the
                                // modifiers into the key would not produce
                                // another character, so that 'langmap'
                                // behaves the same in different terminals and
                                // GUIs. (Only CTRL merges into anything, and
                                // `modifiers` is reset below whatever the
                                // merge left in it.)
                                c2 = langmap_map(c2);
                            }
                            modifiers = ModMask::NONE;
                            modifier_next = false;
                        }
                        // Upstream's `(uint8_t)c2`: a 'langmap' result
                        // wider than a byte compares by its low byte.
                        if key != c2.to_le_bytes()[0] {
                            break;
                        }
                        at += 1;
                    }
                    let mut mlen = at as c_int;

                    // Don't allow mapping the first byte(s) of a multibyte
                    // character, which happens after mapping <M-a> and then
                    // changing 'encoding'. Beware that 0x80 is escaped.
                    if lhs[0] >= 0x80
                        && let Some((ch, _)) = unescape_at(&lhs[..lhs.len() - 1])
                        && usize::from(utf8len_tab[usize::from(lhs[0])]) > lead_len(&ch)
                    {
                        mlen = 0;
                    }

                    // A full match is `mlen == keylen`; a partial one is the
                    // whole typeahead agreeing with a longer mapping.
                    found.keylen = keylen;
                    if mlen != found.keylen && !(mlen == tb_len && tb_len < found.keylen) {
                        // No match; a termcode may still match at the next
                        // character, so remember how far this one agreed.
                        found.max_mlen = found.max_mlen.max(mlen);
                        continue;
                    }

                    // When only script-local mappings are allowed, the
                    // mapping has to start with K_SNR.
                    let snr = || {
                        matches!(mp.keys(), [a, b, c, ..] if c_int::from(*a) == K_SPECIAL
                            && c_int::from(*b) == KS_EXTRA
                            && c_int::from(*c) == KE_SNR as c_int)
                    };
                    if c_int::from(noremaps[0]) == RM_SCRIPT as c_int && !snr() {
                        continue;
                    }

                    // Skip the entry when one of the typed keys may not be
                    // remapped.
                    let matched = usize::try_from(mlen).expect("a positive length");
                    let blocked = noremaps[..matched].iter().any(|&flags| {
                        c_int::from(flags) & (RM_NONE as c_int | RM_ABBR as c_int) != 0
                    });
                    if !is_plug_map && blocked {
                        continue;
                    }

                    let here = Candidate {
                        at: MapAt::new(table, hash, index, mp),
                        nowait: mp.m_nowait,
                        mode: mp.m_mode,
                    };
                    if found.keylen > tb_len {
                        // `<nowait>` is order-dependent, and upstream is the
                        // same: `mp_match` is only set by an *earlier*
                        // iteration of this walk, and `map_add` pushes each
                        // new mapping to the front of its bucket. So a
                        // `<nowait>` mapping only cuts the wait short when it
                        // was defined *after* the longer candidate it is
                        // ambiguous with. Measured both ways -- see the
                        // divergence docket, O-B13-1.
                        if !waited && !mp_match.is_some_and(|m| m.nowait) {
                            // Stop at a partial match and wait for more
                            // input.
                            found.keylen = KEYLEN_PART_MAP;
                            found.mp = Some(here);
                            return true;
                        }
                    } else if found.keylen > found.mp_match_len
                        || (found.keylen == found.mp_match_len
                            && mp_match.is_some_and(|m| m.mode & MODE_LANGMAP == 0)
                            && mp.m_mode & MODE_LANGMAP != 0)
                    {
                        // A longer match, or a langmap one at the same
                        // length.
                        mp_match = Some(here);
                        found.mp_match_len = found.keylen;
                    }
                }
            }
            false
        })
    });
    if partial {
        return finish_search(found, mp_match);
    }

    found.mp = None; // the lists are exhausted
    finish_search(found, mp_match)
}

/// `utfc_ptr2len` of the character [`unescape_at`] answered, NUL-padded:
/// nothing composes with the padding, so it is the character's own length.
fn lead_len(ch: &[u8]) -> usize {
    char_len(&ch[..ch.iter().position(|&b| b == 0).unwrap_or(ch.len())])
}

/// With no partial match, the longest full match is the answer.
fn finish_search(mut found: MapSearch, mp_match: Option<Candidate>) -> MapSearch {
    if found.keylen != KEYLEN_PART_MAP && mp_match.is_some() {
        found.mp = mp_match;
        found.keylen = found.mp_match_len;
    }
    found
}

/// What [`apply_mapping`] reads off the mapping it applies, copied out
/// before anything runs: evaluating an `<expr>` mapping can invoke a function
/// that redefines the mapping, which deletes it.
struct Applied {
    mode: c_int,
    noremap: c_int,
    silent: bool,
    /// The expression of an `<expr>` mapping.
    expr: Option<MapExpr>,
    /// The RHS of any other.
    rhs: MapStr,
    /// The LHS spellings the inserted text is compared with: this one, cut
    /// to the `keylen` bytes that matched, and the twin's whole, if there is
    /// one.
    keys: Vec<u8>,
    alt_keys: Option<Vec<u8>>,
}

impl Applied {
    /// Whether `text` starts with one of the two LHS spellings.
    fn starts_with_lhs(&self, text: &MapStr) -> bool {
        let text = text.as_bytes_with_nul();
        let starts_with = |lhs: &[u8]| lhs.len() <= text.len() && text[..lhs.len()] == *lhs;
        starts_with(&self.keys) || self.alt_keys.as_deref().is_some_and(starts_with)
    }
}

/// Replace a matched mapping's LHS in the typeahead with its RHS.
///
/// Answers `map_result_retry` so that the RHS is matched against the mapping
/// table in turn, or `map_result_fail`. `at` names a mapping that matched
/// `keylen` bytes of the typeahead, and `mapdepth` is the caller's count of
/// the mappings applied since the last typed key.
fn apply_mapping(at: MapAt, keylen: c_int, mapdepth: &mut c_int) -> c_int {
    let applied = at.with(|tables, mp| {
        // Upstream compares `keylen` bytes, which is never more than the LHS
        // it matched. Only a `:map` asks, so a `:noremap` copies nothing.
        let compares = mp.m_noremap == REMAP_YES;
        let keys = mp.keys();
        Applied {
            mode: mp.m_mode,
            noremap: mp.m_noremap,
            silent: mp.m_silent,
            expr: mp.m_expr.then(|| MapExpr::of(mp)),
            rhs: if mp.m_expr {
                MapStr::empty()
            } else {
                mp.m_rhs.str.clone()
            },
            keys: if compares {
                keys[..keys.len().min(keylen as usize)].to_vec()
            } else {
                Vec::new()
            },
            alt_keys: tables
                .twin_keys(false, mp)
                .filter(|_| compares)
                .map(|keys| keys.as_bytes().to_vec()),
        }
    });
    // The match ran no user code, so the mapping is still there.
    let Some(applied) = applied else {
        return map_result_fail as c_int;
    };
    let tb = typeahead();

    // Write the keys to the script file(s). Note that `:lmap` mappings
    // are written *after* being applied. #5658
    if keylen > tb.maplen() && applied.mode & MODE_LANGMAP == 0 {
        // A copy: recording them reaches `'scriptout'` and the swap files.
        let typed = usize::try_from(keylen - tb.maplen()).expect("a positive length");
        gotchars(tb.copy_out(tb.maplen(), typed).as_slice());
    }

    cmd_silent.set(tb.silent() > 0);
    del_typebuf(keylen, 0); // remove the mapped keys

    // The depth check catches `:map x y` plus `:map y x`.
    *mapdepth += 1;
    if *mapdepth >= p_mmd() as c_int {
        emsg(gettext(e_recursive_mapping));
        if State.get() & MODE_CMDLINE != 0 {
            redrawcmdline();
        } else {
            setcursor();
        }
        flush_buffers(FLUSH_MINIMAL);
        *mapdepth = 0; // for the next one
        return map_result_fail as c_int;
    }

    // In Select mode with a Visual-mode mapping: switch to Visual mode
    // for the duration, and append K_SELECT to switch back.
    if visual_active() && visual_select() && applied.mode & MODE_VISUAL != 0 {
        set_visual_select(false);
        let _ = ins_typebuf(K_SELECT_STRING.to_bytes(), REMAP_NONE, 0, true, false);
    }

    // `:map <expr>`: the RHS is an expression to evaluate.
    let evaluated = if let Some(expr) = &applied.expr {
        let save_vgetc_busy = vgetc_busy.get();
        let save_may_garbage_collect = may_garbage_collect.get();
        let prev_did_emsg = did_emsg.get();

        vgetc_busy.set(0);
        may_garbage_collect.set(false);

        let mut map_str = eval_map_expr(expr, NUL);

        if map_str.as_ref().is_none_or(MapStr::is_empty) {
            if prev_did_emsg != did_emsg.get() {
                // An error was displayed and the expression answered
                // nothing: generate a <Nop> so that a redraw can happen.
                let nop = [K_SPECIAL as u8, KS_EXTRA as u8, KE_IGNORE as u8];
                map_str = Some(MapStr::new(&nop));
                if State.get() & MODE_CMDLINE != 0 {
                    // Redraw the command below the error.
                    msg_didout.set(true);
                    msg_row.set(msg_row.get().max(cmdline_row.get()));
                    redrawcmd();
                }
            } else if State.get() & (MODE_NORMAL | MODE_INSERT) != 0 {
                // Otherwise just put the cursor back.
                setcursor();
            }
        }

        vgetc_busy.set(save_vgetc_busy);
        may_garbage_collect.set(save_may_garbage_collect);
        map_str
    } else {
        None
    };
    let map_str = if applied.expr.is_some() {
        evaluated.as_ref()
    } else {
        Some(&applied.rhs)
    };

    // Insert the RHS into the typeahead. When the LHS is a prefix of the
    // RHS the first character is not remapped (but abbreviations still
    // apply); when m_noremap says so, none of it is.
    let inserted = if let Some(map_str) = map_str {
        // A LANGMAP mapping's keys were not recorded above, so they are
        // recorded here instead. The mode is the copy taken before the
        // expression ran, which may have deleted the mapping.
        if keylen > tb.maplen() && applied.mode & MODE_LANGMAP != 0 {
            gotchars(map_str.as_bytes());
        }

        // Whether the RHS starts with the LHS, which is what decides
        // between remapping all of it and skipping the first byte; upstream
        // only asks for a `:map` -- for a `:noremap` the two `strncmp`s are
        // skipped, and `mapresolve` notices.
        let noremap = if applied.noremap != REMAP_YES {
            applied.noremap
        } else if applied.starts_with_lhs(map_str) {
            REMAP_SKIP
        } else {
            REMAP_YES
        };

        ins_typebuf(
            map_str.as_bytes(),
            noremap,
            0,
            true,
            cmd_silent.get() || applied.silent,
        )
    } else {
        Err(Failed)
    };

    if inserted.is_err() {
        map_result_fail as c_int
    } else {
        map_result_retry as c_int
    }
}

/// Handle mappings at the front of the typeahead buffer.
///
/// Answers `map_result_retry` when something was mapped (the RHS has to be
/// matched again, for a recursive mapping), `map_result_get` when nothing
/// mapped and the typeahead has a character to hand out, `map_result_nomatch`
/// when more input is needed to decide, and `map_result_fail` on failure.
///
/// `keylenp` is the match length, in and out; `timedout` is whether the wait
/// for more input timed out; `mapdepth` counts the mappings applied since
/// the last typed key.
pub(crate) fn handle_mapping(keylenp: &mut c_int, timedout: bool, mapdepth: &mut c_int) -> c_int {
    let mut keylen = *keylenp;
    let local_state = get_real_state();
    let tb = typeahead();

    // Typeahead starting with <Plug> is remapped even by a `noremap`
    // mapping: it can only have come from a mapping in the first place.
    let is_plug_map = tb.len() >= 3
        && tb.byte(0) == K_SPECIAL
        && tb.byte(1) == KS_EXTRA
        && tb.byte(2) == KE_PLUG as c_int;

    let tb_c1 = tb.byte(0);

    // Don't look for mappings when
    // - `no_mapping` is set: mappings are disabled, e.g. for CTRL-V;
    // - this byte may not be remapped;
    // - 'paste' is set and we are in Insert or Cmdline mode;
    // - a hit-return prompt is up and CR or space was typed;
    // - a --More-- prompt is up;
    // - we are in CTRL-X mode and this is a key that mode uses.
    let mappable = no_mapping.get() == 0
        && (no_zero_mapping.get() == 0 || tb_c1 != '0' as c_int)
        && (tb.maplen() == 0
            || is_plug_map
            || tb.noremap(0) & (RM_NONE as c_int | RM_ABBR as c_int) == 0)
        && !(p_paste() && State.get() & (MODE_INSERT | MODE_CMDLINE) != 0)
        && !(State.get() == MODE_HITRETURN && (tb_c1 == CAR || tb_c1 == ' ' as c_int))
        && State.get() != MODE_ASKMORE
        && !at_ins_compl_key();

    let mut mp: Option<Candidate> = None;
    let mut mp_match_len = 0;
    let mut max_mlen = 0;
    if mappable {
        let found = search_maphash(keylen, timedout, local_state, is_plug_map, tb_c1);
        mp = found.mp;
        keylen = found.keylen;
        mp_match_len = found.mp_match_len;
        max_mlen = found.max_mlen;
    }

    if (mp.is_none() || max_mlen > mp_match_len) && keylen != KEYLEN_PART_MAP {
        // No mapping matched, or one matched but a non-matching entry
        // agreed on at least as much: try folding the modifier into the
        // key, where mappings are allowed at all.
        if no_mapping.get() == 0 || allow_keys.get() != 0 {
            if tb_c1 == K_SPECIAL && (tb.len() < 2 || (tb.byte(1) == KS_MODIFIER && tb.len() < 4)) {
                // An incomplete modifier sequence: it is not yet possible
                // to decide whether to simplify.
                keylen = KEYLEN_PART_KEY;
            } else {
                keylen = check_simplify_modifier(max_mlen + 1);
                if keylen < 0 {
                    return map_result_fail as c_int; // ins_typebuf() failed
                }
            }
        } else {
            keylen = 0;
        }

        if keylen == 0 && mp.is_none() {
            // No simplification and no mapping at all: hand out the
            // character in the typeahead as it stands.
            *keylenp = keylen;
            return map_result_get as c_int;
        }
        if keylen > 0 {
            // Keys were simplified; match the result again.
            *keylenp = keylen;
            return map_result_retry as c_int;
        }
        if keylen < 0 {
            // An incomplete key sequence: get some more characters.
            debug_assert!(keylen == KEYLEN_PART_KEY);
        } else {
            debug_assert!(mp.is_some());
            keylen = mp_match_len;
        }
    }

    *keylenp = keylen;
    match mp {
        // A non-negative `keylen` only survives the block above with a
        // mapping to apply; the `else` is the `KEYLEN_PART_*` cases.
        Some(mp) if keylen >= 0 && keylen <= tb.len() => apply_mapping(mp.at, keylen, mapdepth),
        _ => map_result_nomatch as c_int,
    }
}
