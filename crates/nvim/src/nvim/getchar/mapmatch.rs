//! Matching the typeahead against the mapping table.
//!
//! [`handle_mapping`] is the heart of it: it walks the maphash bucket for the
//! first typeahead byte looking for the longest mapping whose LHS is a prefix
//! of what is buffered, decides between waiting for more input and giving up
//! (`'timeout'`/`'timeoutlen'`), and on a match replaces the LHS with the RHS
//! in the typeahead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::key_escape;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Longest UTF-8 sequence a character can occupy.
const MB_MAXBYTES: usize = 21;

/// C's `LANGMAP_ADJUST`: map `c` through `'langmap'` when `condition` holds
/// and the key is one `'langmap'` applies to.
///
/// `'langmap'` only applies to *typed* keys, and `'langremap'` decides
/// whether it also applies to the result of a mapping.
///
/// # Safety
/// Callable at any time.
unsafe fn langmap_adjust(c: c_int, condition: bool) -> c_int {
    unsafe {
        // Upstream's operand order, short-circuiting exactly as the macro
        // does. Evaluating `typebuf_maplen()` up front would be pure, but it
        // is a call in the innermost loop of the mapping match and this
        // runs once per typeahead byte per candidate mapping: measured at
        // +5.6..7.6% on `inbench`'s `mapresolve` before it was put back.
        if *p_langmap.get() != 0
            && condition
            && (p_lrm.get() != 0
                || if vgetc_busy.get() != 0 {
                    typebuf_maplen() == 0
                } else {
                    KeyTyped.get()
                })
            && KeyStuffed.get() == 0
            && c >= 0
        {
            if c < 256 {
                c_int::from((*langmap_mapchar.ptr())[c as usize])
            } else {
                langmap_adjust_mb(c)
            }
        } else {
            c
        }
    }
}

/// Replace `slen` bytes at `offset` in the typeahead with `string`.
///
/// # Safety
/// `string` must point at `new_slen + 1` writable bytes, and `offset + slen`
/// must be within the typeahead.
pub(crate) unsafe fn put_string_in_typebuf(
    offset: c_int,
    slen: c_int,
    string: *mut u8,
    new_slen: c_int,
) -> c_int {
    unsafe {
        let extra = new_slen - slen;
        *string.offset(new_slen as isize) = 0;
        if extra < 0 {
            // Remove the matched characters, taking care of tb_noremap.
            del_typebuf(-extra, offset);
        } else if extra > 0 {
            // Insert the extra space we need.
            if ins_typebuf(
                string.offset(slen as isize).cast(),
                REMAP_YES,
                offset,
                false,
                false,
            ) == FAIL
            {
                return FAIL;
            }
        }
        // Careful: del_typebuf() and ins_typebuf() may have reallocated
        // typebuf.tb_buf, so the destination is re-derived here.
        let tb = typebuf.ptr();
        ptr::copy(
            string,
            (*tb).tb_buf.offset(((*tb).tb_off + offset) as isize),
            new_slen as usize,
        );
        OK
    }
}

/// Whether the typeahead starts with a key that Insert-mode completion uses,
/// including the form with a Ctrl modifier.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn at_ins_compl_key() -> bool {
    unsafe {
        let tb = typebuf.ptr();
        let p = (*tb).tb_buf.offset((*tb).tb_off as isize);
        let mut c = c_int::from(*p);

        if (*tb).tb_len > 3
            && c == K_SPECIAL
            && c_int::from(*p.add(1)) == KS_MODIFIER
            && c_int::from(*p.add(2)) & MOD_MASK_CTRL != 0
        {
            c = c_int::from(*p.add(3)) & 0x1f;
        }
        (ctrl_x_mode_not_default() && vim_is_ctrl_x_key(c))
            || (compl_status_local() && (c == Ctrl_N || c == Ctrl_P))
    }
}

/// Fold a modifier at the front of the typeahead into the key it modifies,
/// where a single code stands for the combination.
///
/// Looks at offsets 0 through `max_offset - 1`. Answers how many bytes the
/// replacement occupies, 0 when nothing changed, or -1 on failure.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn check_simplify_modifier(max_offset: c_int) -> c_int {
    unsafe {
        // Terminal mode wants the full modifiers, so that the key can be
        // encoded for the child process.
        if State.get() & MODE_TERMINAL != 0 || no_reduce_keys.get() > 0 {
            return 0;
        }

        for offset in 0..max_offset {
            let tb = typebuf.ptr();
            if offset + 3 >= (*tb).tb_len {
                break;
            }
            let tp = (*tb).tb_buf.offset(((*tb).tb_off + offset) as isize);
            if c_int::from(*tp) != K_SPECIAL || c_int::from(*tp.add(1)) != KS_MODIFIER {
                continue;
            }

            // A modifier that was not used for a mapping: apply it to the
            // ASCII key. Shift would already have been applied.
            let mut modifier = c_int::from(*tp.add(2));
            let c = c_int::from(*tp.add(3));
            let new_c = merge_modifiers(c, &raw mut modifier);
            if new_c == c {
                continue;
            }

            if offset == 0 {
                // At the start: remember the character and mod_mask from
                // before the merge. In some cases -- at the hit-return
                // prompt, say -- they are put back into the typeahead.
                vgetc_char.set(c);
                vgetc_mod_mask.set(c_int::from(*tp.add(2)));
            }

            let mut new_string = [0u8; MB_MAXBYTES];
            let len = if new_c < 0 {
                new_string[..3].copy_from_slice(&key_escape(new_c));
                3
            } else {
                utf_char2bytes(new_c, new_string.as_mut_ptr().cast())
            };

            let ok = if modifier == 0 {
                // The whole three-byte prefix plus the key becomes the key.
                put_string_in_typebuf(offset, 4, new_string.as_mut_ptr(), len)
            } else {
                // Some of the modifier is left over; keep the prefix.
                *tp.add(2) = modifier as u8;
                put_string_in_typebuf(offset + 3, 1, new_string.as_mut_ptr(), len)
            };
            if ok == FAIL {
                return -1;
            }
            return len;
        }
        0
    }
}

/// What the maphash walk found.
struct MapSearch {
    /// The best candidate, or null when there is none.
    mp: *mut mapblock_T,
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
/// mapped.
///
/// # Safety
/// `timedout` must point at a readable flag.
unsafe fn search_maphash(
    keylen: c_int,
    timedout: *const bool,
    local_state: c_int,
    is_plug_map: bool,
    mut tb_c1: c_int,
) -> MapSearch {
    unsafe {
        let mut found = MapSearch {
            mp: ptr::null_mut(),
            keylen,
            mp_match_len: 0,
            max_mlen: 0,
        };

        // How many bytes at the start of a key are not subject to 'langmap':
        // the two that follow a K_SPECIAL.
        let nolmaplen = if tb_c1 == K_SPECIAL {
            2
        } else {
            tb_c1 = langmap_adjust(
                tb_c1,
                State.get() & (MODE_CMDLINE | MODE_INSERT) == 0 && get_real_state() != MODE_SELECT,
            );
            0
        };

        // Buffer-local mappings first, then the global ones.
        let mut mp = get_buf_maphash_list(local_state, tb_c1);
        let mut mp2 = get_maphash_list(local_state, tb_c1);
        if mp.is_null() {
            mp = mp2;
            mp2 = ptr::null_mut();
        }

        let mut mp_match: *mut mapblock_T = ptr::null_mut();
        while !mp.is_null() {
            'entry: {
                // Only consider an entry whose first character matches and
                // that is for the current state. Skip `:lmap` mappings when
                // keys were mapped.
                if c_int::from(*(*mp).m_keys as u8) != tb_c1
                    || (*mp).m_mode & local_state == 0
                    || ((*mp).m_mode & MODE_LANGMAP != 0 && (*typebuf.ptr()).tb_maplen != 0)
                {
                    break 'entry;
                }

                // How many bytes of the typeahead this mapping agrees with.
                let mut nomap = nolmaplen;
                let mut modifiers = 0;
                let tb = typebuf.ptr();
                let mut mlen = 1;
                while mlen < (*tb).tb_len {
                    let mut c2 = c_int::from(*(*tb).tb_buf.offset(((*tb).tb_off + mlen) as isize));
                    if nomap > 0 {
                        if nomap == 2 && c2 == KS_MODIFIER {
                            modifiers = 1;
                        } else if nomap == 1 && modifiers == 1 {
                            modifiers = c2;
                        }
                        nomap -= 1;
                    } else {
                        if c2 == K_SPECIAL {
                            nomap = 2;
                        } else if merge_modifiers(c2, &raw mut modifiers) == c2 {
                            // Only apply 'langmap' when merging the modifiers
                            // into the key would not produce another
                            // character, so that 'langmap' behaves the same
                            // in different terminals and GUIs.
                            c2 = langmap_adjust(c2, true);
                        }
                        modifiers = 0;
                    }
                    if c_int::from(*(*mp).m_keys.offset(mlen as isize) as u8) != c2 {
                        break;
                    }
                    mlen += 1;
                }

                // Don't allow mapping the first byte(s) of a multibyte
                // character, which happens after mapping <M-a> and then
                // changing 'encoding'. Beware that 0x80 is escaped.
                let mut p1: *const c_char = (*mp).m_keys;
                let p2 = mb_unescape(&raw mut p1);
                if !p2.is_null()
                    && c_int::from((*utf8len_tab.ptr())[tb_c1 as usize]) > utfc_ptr2len(p2)
                {
                    mlen = 0;
                }

                // A full match is `mlen == keylen`; a partial one is the
                // whole typeahead agreeing with a longer mapping.
                found.keylen = (*mp).m_keylen;
                if mlen != found.keylen && !(mlen == (*tb).tb_len && (*tb).tb_len < found.keylen) {
                    // No match; a termcode may still match at the next
                    // character, so remember how far this one agreed.
                    found.max_mlen = found.max_mlen.max(mlen);
                    break 'entry;
                }

                let mut s = (*tb).tb_noremap.offset((*tb).tb_off as isize);
                // When only script-local mappings are allowed, the mapping
                // has to start with K_SNR.
                if c_int::from(*s) == RM_SCRIPT as c_int
                    && (c_int::from(*(*mp).m_keys as u8) != K_SPECIAL
                        || c_int::from(*(*mp).m_keys.add(1) as u8) != KS_EXTRA
                        || c_int::from(*(*mp).m_keys.add(2)) != KE_SNR as c_int)
                {
                    break 'entry;
                }

                // Skip the entry when one of the typed keys may not be
                // remapped. `n` is left at the index that stopped the scan,
                // or -1 when every byte was remappable.
                let mut n = mlen;
                loop {
                    n -= 1;
                    if n < 0 {
                        break;
                    }
                    let flags = *s;
                    s = s.add(1);
                    if c_int::from(flags) & (RM_NONE as c_int | RM_ABBR as c_int) != 0 {
                        break;
                    }
                }
                if !is_plug_map && n >= 0 {
                    break 'entry;
                }

                if found.keylen > (*tb).tb_len {
                    if !*timedout && !(!mp_match.is_null() && (*mp_match).m_nowait != 0) {
                        // Stop at a partial match and wait for more input.
                        found.keylen = KEYLEN_PART_MAP;
                        found.mp = mp;
                        return finish_search(found, mp_match);
                    }
                } else if found.keylen > found.mp_match_len
                    || (found.keylen == found.mp_match_len
                        && !mp_match.is_null()
                        && (*mp_match).m_mode & MODE_LANGMAP == 0
                        && (*mp).m_mode & MODE_LANGMAP != 0)
                {
                    // A longer match, or a langmap one at the same length.
                    mp_match = mp;
                    found.mp_match_len = found.keylen;
                }
            }

            // Advance: the buffer-local list first, then the global one.
            if (*mp).m_next.is_null() {
                mp = mp2;
                mp2 = ptr::null_mut();
            } else {
                mp = (*mp).m_next;
            }
        }

        found.mp = mp; // null: the lists are exhausted
        finish_search(found, mp_match)
    }
}

/// With no partial match, the longest full match is the answer.
fn finish_search(mut found: MapSearch, mp_match: *mut mapblock_T) -> MapSearch {
    if found.keylen != KEYLEN_PART_MAP && !mp_match.is_null() {
        found.mp = mp_match;
        found.keylen = found.mp_match_len;
    }
    found
}

/// Replace a matched mapping's LHS in the typeahead with its RHS.
///
/// Answers `map_result_retry` so that the RHS is matched against the mapping
/// table in turn, or `map_result_fail`.
///
/// # Safety
/// `mp` must be a live mapping that matched `keylen` bytes of the typeahead.
unsafe fn apply_mapping(mp: *mut mapblock_T, keylen: c_int, mapdepth: *mut c_int) -> c_int {
    unsafe {
        let tb = typebuf.ptr();

        // Write the keys to the script file(s). Note that `:lmap` mappings
        // are written *after* being applied. #5658
        if keylen > (*tb).tb_maplen && (*mp).m_mode & MODE_LANGMAP == 0 {
            gotchars(
                (*tb)
                    .tb_buf
                    .offset(((*tb).tb_off + (*tb).tb_maplen) as isize),
                (keylen - (*tb).tb_maplen) as usize,
            );
        }

        cmd_silent.set((*tb).tb_silent > 0);
        del_typebuf(keylen, 0); // remove the mapped keys

        // The depth check catches `:map x y` plus `:map y x`.
        *mapdepth += 1;
        if *mapdepth >= p_mmd.get() as c_int {
            emsg(gettext(e_recursive_mapping.ptr().cast()));
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
        if VIsual_active.get() && VIsual_select.get() && (*mp).m_mode & MODE_VISUAL != 0 {
            VIsual_select.set(false);
            ins_typebuf(
                K_SELECT_STRING.as_ptr().cast_mut(),
                REMAP_NONE,
                0,
                true,
                false,
            );
        }

        // Copy the fields of *mp that are used below: evaluating an <expr>
        // mapping can invoke a function that redefines the mapping, which
        // frees *mp.
        let save_m_expr = (*mp).m_expr != 0;
        let save_m_noremap = (*mp).m_noremap;
        let save_m_silent = (*mp).m_silent != 0;
        let mut save_m_keys: *mut c_char = ptr::null_mut();
        let mut save_alt_m_keys: *mut c_char = ptr::null_mut();
        let save_alt_m_keylen = if (*mp).m_alt.is_null() {
            0
        } else {
            (*(*mp).m_alt).m_keylen
        };

        // `:map <expr>`: the RHS is an expression to evaluate.
        let map_str = if save_m_expr {
            let save_vgetc_busy = vgetc_busy.get();
            let save_may_garbage_collect = may_garbage_collect.get();
            let prev_did_emsg = did_emsg.get();

            vgetc_busy.set(0);
            may_garbage_collect.set(false);

            save_m_keys = xmemdupz((*mp).m_keys.cast(), (*mp).m_keylen as usize).cast();
            if !(*mp).m_alt.is_null() {
                save_alt_m_keys =
                    xmemdupz((*(*mp).m_alt).m_keys.cast(), save_alt_m_keylen as usize).cast();
            }
            let mut map_str = eval_map_expr(mp, NUL);

            if map_str.is_null() || c_int::from(*map_str) == NUL {
                if prev_did_emsg != did_emsg.get() {
                    // An error was displayed and the expression answered
                    // nothing: generate a <Nop> so that a redraw can happen.
                    xfree(map_str.cast());
                    let nop = [K_SPECIAL as u8, KS_EXTRA as u8, KE_IGNORE as u8];
                    map_str = xmemdupz(nop.as_ptr().cast(), 3).cast();
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
            (*mp).m_str
        };

        // Insert the RHS into the typeahead. When the LHS is a prefix of the
        // RHS the first character is not remapped (but abbreviations still
        // apply); when m_noremap says so, none of it is.
        let inserted = if map_str.is_null() {
            FAIL
        } else {
            // A LANGMAP mapping's keys were not recorded above, so they are
            // recorded here instead.
            if keylen > (*tb).tb_maplen && (*mp).m_mode & MODE_LANGMAP != 0 {
                gotchars(map_str.cast(), strlen(map_str));
            }

            // Whether the RHS starts with the LHS, which is what decides
            // between remapping all of it and skipping the first byte. Kept
            // as a closure rather than a `let`, because upstream only
            // evaluates it for a `:map` -- for a `:noremap` the two
            // `strncmp`s are skipped, and `mapresolve` notices.
            let starts_with_lhs = || {
                if save_m_expr {
                    strncmp(map_str, save_m_keys, keylen as usize) == 0
                        || (!save_alt_m_keys.is_null()
                            && strncmp(map_str, save_alt_m_keys, save_alt_m_keylen as usize) == 0)
                } else {
                    strncmp(map_str, (*mp).m_keys, keylen as usize) == 0
                        || (!(*mp).m_alt.is_null()
                            && strncmp(
                                map_str,
                                (*(*mp).m_alt).m_keys,
                                (*(*mp).m_alt).m_keylen as usize,
                            ) == 0)
                }
            };
            let noremap = if save_m_noremap != REMAP_YES {
                save_m_noremap
            } else if starts_with_lhs() {
                REMAP_SKIP
            } else {
                REMAP_YES
            };

            let inserted =
                ins_typebuf(map_str, noremap, 0, true, cmd_silent.get() || save_m_silent);
            if save_m_expr {
                xfree(map_str.cast());
            }
            inserted
        };
        xfree(save_m_keys.cast());
        xfree(save_alt_m_keys.cast());

        if inserted == FAIL {
            map_result_fail as c_int
        } else {
            map_result_retry as c_int
        }
    }
}

/// Handle mappings at the front of the typeahead buffer.
///
/// Answers `map_result_retry` when something was mapped (the RHS has to be
/// matched again, for a recursive mapping), `map_result_get` when nothing
/// mapped and the typeahead has a character to hand out, `map_result_nomatch`
/// when more input is needed to decide, and `map_result_fail` on failure.
///
/// # Safety
/// `keylenp`, `timedout` and `mapdepth` must point at valid storage.
pub(crate) unsafe fn handle_mapping(
    keylenp: *mut c_int,
    timedout: *const bool,
    mapdepth: *mut c_int,
) -> c_int {
    unsafe {
        let mut keylen = *keylenp;
        let local_state = get_real_state();
        let tb = typebuf.ptr();

        // Typeahead starting with <Plug> is remapped even by a `noremap`
        // mapping: it can only have come from a mapping in the first place.
        let is_plug_map = (*tb).tb_len >= 3
            && c_int::from(*(*tb).tb_buf.offset((*tb).tb_off as isize)) == K_SPECIAL
            && c_int::from(*(*tb).tb_buf.offset(((*tb).tb_off + 1) as isize)) == KS_EXTRA
            && c_int::from(*(*tb).tb_buf.offset(((*tb).tb_off + 2) as isize)) == KE_PLUG as c_int;

        let tb_c1 = c_int::from(*(*tb).tb_buf.offset((*tb).tb_off as isize));

        // Don't look for mappings when
        // - `no_mapping` is set: mappings are disabled, e.g. for CTRL-V;
        // - this byte may not be remapped;
        // - 'paste' is set and we are in Insert or Cmdline mode;
        // - a hit-return prompt is up and CR or space was typed;
        // - a --More-- prompt is up;
        // - we are in CTRL-X mode and this is a key that mode uses.
        let mappable = no_mapping.get() == 0
            && (no_zero_mapping.get() == 0 || tb_c1 != '0' as c_int)
            && ((*tb).tb_maplen == 0
                || is_plug_map
                || c_int::from(*(*tb).tb_noremap.offset((*tb).tb_off as isize))
                    & (RM_NONE as c_int | RM_ABBR as c_int)
                    == 0)
            && !(p_paste.get() != 0 && State.get() & (MODE_INSERT | MODE_CMDLINE) != 0)
            && !(State.get() == MODE_HITRETURN && (tb_c1 == CAR || tb_c1 == ' ' as c_int))
            && State.get() != MODE_ASKMORE
            && !at_ins_compl_key();

        let mut mp: *mut mapblock_T = ptr::null_mut();
        let mut mp_match_len = 0;
        let mut max_mlen = 0;
        if mappable {
            let found = search_maphash(keylen, timedout, local_state, is_plug_map, tb_c1);
            mp = found.mp;
            keylen = found.keylen;
            mp_match_len = found.mp_match_len;
            max_mlen = found.max_mlen;
        }

        if (mp.is_null() || max_mlen > mp_match_len) && keylen != KEYLEN_PART_MAP {
            // No mapping matched, or one matched but a non-matching entry
            // agreed on at least as much: try folding the modifier into the
            // key, where mappings are allowed at all.
            if no_mapping.get() == 0 || allow_keys.get() != 0 {
                if tb_c1 == K_SPECIAL
                    && ((*tb).tb_len < 2
                        || (c_int::from(*(*tb).tb_buf.offset(((*tb).tb_off + 1) as isize))
                            == KS_MODIFIER
                            && (*tb).tb_len < 4))
                {
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

            if keylen == 0 && mp.is_null() {
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
                debug_assert!(!mp.is_null());
                keylen = mp_match_len;
            }
        }

        *keylenp = keylen;
        if keylen >= 0 && keylen <= (*tb).tb_len {
            apply_mapping(mp, keylen, mapdepth)
        } else {
            map_result_nomatch as c_int
        }
    }
}
