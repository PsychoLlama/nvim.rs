//! `:map` and friends: the command layer over the table.
//!
//! [`buf_do_map`] does all of it — list, add, replace and delete — for one
//! already-parsed [`MapArguments`]; [`do_map`] and [`do_exmap`] are the
//! parsing wrappers, and the `ex_*` entry points are what the command table
//! dispatches to.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::charset::is_word_at;
use crate::ex_docmd::sourcing_lnum;
use crate::keycodes::Ctrl_C;
use crate::lua::executor::nlua_set_sctx_in;
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::tr_c;
use crate::types::{MapTables, MapTwin};
use crate::winlayer::Buf;
use core::ffi::c_int;

/// Whether an abbreviation's LHS is one vi can find the start of.
///
/// If it ends in a keyword character, everything before it must be all
/// keyword characters or all non-keyword ones -- `#i` for `#include` is the
/// point of the rule -- and it may never contain white space.
fn abbrev_lhs_ok(lhs: &MapStr) -> bool {
    let bytes = lhs.as_bytes();
    // Whether the character starting at `at` is a keyword character.
    let is_word = |at: usize| c_int::from(is_word_at(&bytes[at..], Buf::current()));

    let first = is_word(0);
    let mut last = first;
    let mut same = -1; // count of characters of the same type at the start
    let mut n = 1; // number of (multi-byte) characters
    let mut at = cluster_len(bytes);
    while at < bytes.len() {
        n += 1;
        last = is_word(at); // type of the last character
        if same == -1 && last != first {
            same = n - 1;
        }
        at += cluster_len(&bytes[at..]).max(1);
    }
    if last != 0 && n > 2 && same >= 0 && same < n - 1 {
        return false;
    }
    // An abbreviation cannot contain white space.
    !bytes.iter().any(|&byte| ascii_iswhite(c_int::from(byte)))
}

/// Whether a *global* mapping already claims exactly these keys in any of
/// `mode`, which is what makes a new `<unique>` buffer-local one fail.
fn global_map_exists(mode: c_int, lhs: &[u8], is_abbrev: bool) -> bool {
    let clashes = |mp: &MapBlock| {
        if got_int.get() {
            return Some(false);
        }
        // Check entries with the same mode.
        (mp.m_mode & mode != 0 && mp.keys() == lhs).then_some(true)
    };
    map_walk(MapTable::Global, is_abbrev, clashes).unwrap_or(false)
}

/// The entries a listing shows, copied out of the table so that printing --
/// which can stop at a `--More--` prompt -- holds no borrow of it.
fn listed(table: MapTable, is_abbrev: bool, keep: impl Fn(&MapBlock) -> bool) -> Vec<MapBlock> {
    table.with(|tables| {
        tables
            .walk(is_abbrev)
            .filter(|mp| keep(mp))
            .cloned()
            .collect()
    })
}

/// Print `entries`, stopping when `q` was typed at the `--More--` prompt.
fn show_all(entries: &[MapBlock], local: bool) {
    for mp in entries {
        if got_int.get() {
            break;
        }
        showmap(mp, local);
    }
}

/// List the buffer-local mappings that a *global* listing should also show,
/// and answer whether any were printed.
///
/// Without `has_lhs` every entry in `mode` is shown; with it, every entry
/// whose LHS and `lhs` agree as far as the shorter of the two.
fn show_buffer_local(buffer: Buf, mode: c_int, lhs: &[u8], has_lhs: bool, is_abbrev: bool) -> bool {
    if got_int.get() {
        return false;
    }
    let entries = listed(MapTable::Buffer(buffer), is_abbrev, |mp| {
        !mp.m_simplified && mp.m_mode & mode != 0 && (!has_lhs || agrees(mp.keys(), lhs))
    });
    show_all(&entries, true);
    !entries.is_empty()
}

/// Whether two keys agree as far as the shorter of the two, which is the
/// `strncmp(a, b, MIN(alen, blen))` every listing and `:unmap` match uses.
fn agrees(a: &[u8], b: &[u8]) -> bool {
    let minlen = a.len().min(b.len());
    a[..minlen] == b[..minlen]
}

/// Give the entry at `at` of list `hash` the right-hand side and flags in
/// `args`, reusing the block a `:map` of an existing LHS would otherwise have
/// to allocate.
///
/// The old RHS bundle is released here — by the assignment, not by hand — and
/// the twin link is cut, because the pair is no longer two spellings of one
/// mapping.
#[allow(clippy::too_many_arguments)] // the walk's state, spelled out
fn reuse_mapblock(
    tables: &mut MapTables,
    (hash, abbr, at): (usize, bool, usize),
    args: &MapArguments,
    noremap: c_int,
    mode: c_int,
    simplified: bool,
    sctx: ScriptCtx,
) -> MapTwin {
    tables.cut_pair(hash, abbr, at);
    let mp = &mut tables.list_mut(hash, abbr)[at];
    mp.m_rhs = args.rhs().dup();
    mp.m_noremap = noremap;
    mp.m_nowait = args.nowait;
    mp.m_silent = args.silent;
    mp.m_mode = mode;
    mp.m_simplified = simplified;
    mp.m_expr = args.expr;
    mp.m_replace_keycodes = args.replace_keycodes;
    mp.m_script_ctx = sctx;
    mp.as_twin()
}

/// What a walk of one bucket for a definition or an `:unmap` ended with.
enum Walked {
    /// Every entry was looked at.
    Done,
    /// A `<unique>` definition found its LHS taken.
    Clash,
}

/// Set or remove a mapping or abbreviation in `buffer`, or display matching
/// ones.
///
/// `maptype` is one of the `MAPTYPE_*` values and `args` is already parsed
/// and termcode-replaced: whitespace, `<` and `>` in the two halves are
/// literal by the time this sees them.  Any mapping created *shares* the
/// parse's right-hand side, so `args` needs nothing done to it afterwards.
///
/// Answers 0 on success, or 1 for invalid arguments, 2 for no match, 5 for a
/// `<unique>` clash and 6 for a buffer-local `<unique>` entry clashing with a
/// global one.
pub(crate) fn buf_do_map(
    mut maptype: c_int,
    args: &MapArguments,
    mode: c_int,
    is_abbrev: bool,
    buffer: Buf,
) -> c_int {
    let mut retval = 0;

    // If <buffer> was given we search the buffer's mappings, not the
    // global ones.
    let table = MapTable::of(args.buffer, buffer);
    let local = table.is_local();
    let mut mp_result: [Option<MapTwin>; 2] = [None; 2];

    let unmap_lhs_only = maptype == MAPTYPE_UNMAP_LHS;
    if unmap_lhs_only {
        maptype = MAPTYPE_UNMAP;
    }
    let is_unmap = maptype == MAPTYPE_UNMAP;

    // For ":noremap" don't remap, otherwise do remap.
    let noremap = if args.script {
        REMAP_SCRIPT
    } else if maptype == MAPTYPE_NOREMAP {
        REMAP_NONE
    } else {
        REMAP_YES
    };

    let has_lhs = !args.lhs.is_empty();
    let has_rhs = args.rhs_lua() != LUA_NOREF || args.rhs_len() != 0 || args.rhs_is_noop;
    let do_print = !has_lhs || (!is_unmap && !has_rhs);
    if do_print {
        msg_ext_set_kind(c"list_cmd");
    }
    // The script context a replaced entry takes, worked out once and outside
    // the table's borrow: it asks Lua where the definition came from.
    let sctx = (has_rhs && !is_unmap).then(|| {
        let mut ctx = current_sctx.get();
        ctx.sc_lnum += sourcing_lnum();
        nlua_set_sctx_in(&mut ctx);
        ctx
    });

    'theend: {
        // Check for :unmap without argument.
        if is_unmap && !has_lhs {
            retval = 1;
            break 'theend;
        }

        let did_simplify = args.alt_lhs_len != 0;

        // The following is done twice if we have two versions of the keys.
        for keyround in 1..=2 {
            let mut did_it = false;
            let mut did_local = false;
            let keyround1_simplified = keyround == 1 && did_simplify;

            if keyround == 2 && !did_simplify {
                break;
            }
            // Which spelling of the LHS this round works on.  Printing always
            // uses the not-simplified one.
            let (lhs, lhs_len) = if keyround == 2 || (did_simplify && do_print) {
                (&args.alt_lhs, args.alt_lhs_len)
            } else {
                (&args.lhs, args.lhs_len)
            };

            // Check arguments and translate function keys.
            if has_lhs {
                if lhs_len > MAXMAPLEN as size_t {
                    retval = 1;
                    break 'theend;
                }
                if is_abbrev && !is_unmap && !abbrev_lhs_ok(lhs) {
                    retval = 1;
                    break 'theend;
                }
            }
            // Past the length check the truncated copy *is* the whole LHS.
            let lhs = lhs.as_bytes();

            if has_lhs && has_rhs && is_abbrev {
                // We are adding an abbreviation, so reset the flag that
                // says there are none.
                no_abbr.set(false);
            }

            if do_print {
                msg_start();
            }

            // Check that a new local mapping was not already defined
            // globally.
            let clash = args.unique
                && local
                && has_lhs
                && has_rhs
                && !is_unmap
                && global_map_exists(mode, lhs, is_abbrev);
            if clash {
                retval = 6;
                break 'theend;
            }

            // When listing global mappings, also list buffer-local ones.
            if !local && !has_rhs && !is_unmap {
                did_local = show_buffer_local(buffer, mode, lhs, has_lhs, is_abbrev);
            }

            if do_print {
                // A listing: every entry in the mode, or every one whose LHS
                // agrees with `lhs` in the one bucket it hashes to.
                let hash = map_hash(mode, c_int::from(lhs.first().copied().unwrap_or(0)));
                let entries = if got_int.get() {
                    Vec::new()
                } else if has_lhs {
                    table.with(|tables| {
                        let list = tables.list(if is_abbrev { 0 } else { hash }, is_abbrev);
                        let keep = |mp: &&MapBlock| {
                            mp.m_mode & mode != 0 && !mp.m_simplified && agrees(mp.keys(), lhs)
                        };
                        list.iter()
                            .rev()
                            .map(|mp| &**mp)
                            .filter(keep)
                            .cloned()
                            .collect()
                    })
                } else {
                    listed(table, is_abbrev, |mp| {
                        mp.m_mode & mode != 0 && !mp.m_simplified
                    })
                };
                show_all(&entries, local);
                did_it = !entries.is_empty();
            } else {
                // Find a matching entry. For :unmap we may loop twice: once
                // for an entry with a matching "from" part, and if that fails
                // once for one with a matching "to" part, so that ":ab foo
                // bar" can be undone by ":unab foo" -- where "foo" has itself
                // been replaced by "bar".
                let num_rounds = if is_unmap && !unmap_lhs_only { 2 } else { 1 };
                let mut round = 0;
                while round < num_rounds && !did_it && !got_int.get() {
                    let (hash_start, hash_end) = if round == 0 || is_abbrev {
                        // Just one hash.
                        let start = if is_abbrev {
                            0
                        } else {
                            map_hash(mode, c_int::from(lhs.first().copied().unwrap_or(0)))
                        };
                        (start, start + 1)
                    } else {
                        (0, MAX_MAPHASH)
                    };
                    let step = Step {
                        lhs,
                        args,
                        mode,
                        noremap,
                        is_abbrev,
                        is_unmap,
                        keyround1_simplified,
                        round,
                        sctx,
                    };
                    let walked = table.with_mut(|tables| {
                        for hash in hash_start..hash_end {
                            if got_int.get() {
                                break;
                            }
                            let found = step.bucket(tables, hash, &mut did_it);
                            if let Some(twin) = found.1 {
                                mp_result[keyround - 1] = Some(twin);
                            }
                            if let Walked::Clash = found.0 {
                                return Walked::Clash;
                            }
                        }
                        Walked::Done
                    });
                    if let Walked::Clash = walked {
                        retval = 5;
                        break 'theend;
                    }
                    round += 1;
                }
            }

            if is_unmap {
                if !did_it {
                    if !keyround1_simplified {
                        retval = 2; // no match
                    }
                } else if lhs.first().copied().map(c_int::from) == Some(Ctrl_C) {
                    // CTRL-C has been unmapped, reuse it for Interrupting.
                    match table {
                        MapTable::Buffer(mut buf) => buf.b_mapped_ctrl_c &= !mode,
                        MapTable::Global => mapped_ctrl_c.set(mapped_ctrl_c.get() & !mode),
                    }
                }
                continue;
            }

            if !has_lhs || !has_rhs {
                // Print entries.
                if !did_it && !did_local {
                    let text = if is_abbrev {
                        c"No abbreviation found"
                    } else {
                        c"No mapping found"
                    };
                    msg(gettext(text), 0);
                }
                break 'theend; // listing finished
            }

            if did_it {
                continue; // the new entry has been added already
            }

            // Get here when adding a new entry to the maphash list or the
            // abbrlist.
            mp_result[keyround - 1] = Some(map_add(
                table,
                lhs,
                args,
                noremap,
                mode,
                is_abbrev,
                0, // sid
                0, // lnum
                keyround1_simplified,
            ));
        }

        if let [Some(a), Some(b)] = mp_result {
            table.with_mut(|tables| tables.pair(is_abbrev, a, b));
        }
    }

    retval
}

/// What one bucket walk of [`buf_do_map`] needs to know: the LHS, how to
/// match it, and what to do with a match.
struct Step<'a> {
    lhs: &'a [u8],
    args: &'a MapArguments,
    mode: c_int,
    noremap: c_int,
    is_abbrev: bool,
    is_unmap: bool,
    keyround1_simplified: bool,
    round: c_int,
    sctx: Option<ScriptCtx>,
}

impl Step<'_> {
    /// Walk one bucket newest first, unmapping or redefining whatever
    /// matches; answer how it ended and the entry a definition reused.
    ///
    /// Upstream's two `break`s leave *this* walk and resume at the next
    /// bucket, not at the next round.
    fn bucket(
        &self,
        tables: &mut MapTables,
        hash: usize,
        did_it: &mut bool,
    ) -> (Walked, Option<MapTwin>) {
        let abbr = self.is_abbrev;
        let lhs = self.lhs;
        let mut reused = None;
        // An index below the one being looked at is unmoved by a removal or
        // a re-hash at it, so the walk can go on from there.
        let mut at = tables.list(hash, abbr).len();
        while at > 0 {
            // Find the next entry, newest first, in the mode whose keys
            // agree: the scan reads the list once, so a long bucket that
            // holds no match costs one pass.
            let list = tables.list(hash, abbr);
            let found = list[..at].iter().rposition(|entry| {
                // 'q' typed at a `--More--` prompt stops the walk where it
                // is, as upstream tests it per entry.
                got_int.get()
                    || (entry.m_mode & self.mode != 0
                        && agrees(
                            if self.round != 0 {
                                entry.rhs()
                            } else {
                                entry.keys()
                            },
                            lhs,
                        ))
            });
            let Some(found) = found else {
                break;
            };
            at = found;
            if got_int.get() {
                break;
            }
            let entry = &mut tables.list_mut(hash, abbr)[at];
            // Do we have a match? On the second round, try to unmap the
            // "rhs" string.
            let keys = if self.round != 0 {
                entry.rhs()
            } else {
                entry.keys()
            };
            let n = keys.len();

            if self.is_unmap {
                // Delete the entry, but only on a full match. For
                // abbreviations we ignore trailing space when matching the
                // "lhs", since an abbreviation cannot have any.
                let trailing =
                    n <= lhs.len() && lhs[n..].iter().all(|&b| ascii_iswhite(c_int::from(b)));
                if n != lhs.len() && !(abbr && self.round == 0 && trailing) {
                    continue;
                }
                // In the keyround for simplified keys, don't unmap a
                // mapping without the m_simplified flag.
                if self.keyround1_simplified && !entry.m_simplified {
                    break;
                }
                // Reset the indicated mode bits; if nothing is left the
                // entry is deleted below.
                entry.m_mode &= !self.mode;
                *did_it = true;
            } else if n != lhs.len() {
                continue; // the new entry is ambiguous
            } else if self.keyround1_simplified && !entry.m_simplified {
                // In the keyround for simplified keys, don't replace a
                // mapping without the m_simplified flag.
                *did_it = true;
                break;
            } else if self.args.unique {
                return (Walked::Clash, reused);
            } else {
                // A new rhs for an existing entry.
                entry.m_mode &= !self.mode; // remove mode bits
                if entry.m_mode == 0 && !*did_it {
                    let sctx = self.sctx.expect("a definition has a script context");
                    let place = (hash, abbr, at);
                    let twin = reuse_mapblock(
                        tables,
                        place,
                        self.args,
                        self.noremap,
                        self.mode,
                        self.keyround1_simplified,
                        sctx,
                    );
                    reused = Some(twin);
                    *did_it = true;
                }
            }

            let entry = &tables.list(hash, abbr)[at];
            if entry.m_mode == 0 {
                tables.remove_at(hash, abbr, at); // the entry can go
                continue;
            }

            // May need to put this entry into another hash list.
            let first = entry.keys().first().copied().unwrap_or(0);
            let new_hash = map_hash(entry.m_mode, c_int::from(first));
            if !abbr && new_hash != hash {
                tables.relink(hash, at, new_hash);
            }
        }
        (Walked::Done, reused)
    }
}

/// Set or remove a mapping or an abbreviation in the current buffer, or
/// display the matching ones.
///
/// ```vim
/// map[!]                          " show all key mappings
/// map[!] {lhs}                    " show key mapping for {lhs}
/// map[!] {lhs} {rhs}              " set key mapping for {lhs} to {rhs}
/// noremap[!] {lhs} {rhs}          " same, but no remapping for {rhs}
/// unmap[!] {lhs}                  " remove key mapping for {lhs}
/// abbr                            " show all abbreviations
/// abbr {lhs}                      " show abbreviations for {lhs}
/// abbr {lhs} {rhs}                " set abbreviation for {lhs} to {rhs}
/// noreabbr {lhs} {rhs}            " same, but no remapping for {rhs}
/// unabbr {lhs}                    " remove abbreviation for {lhs}
/// ```
///
/// `arg` is everything after the initial `:[x][nore]map`, up to its NUL.
/// `mode` is a set of mode bits; see `get_map_mode`.  For the answer, see
/// [`buf_do_map`].
pub(crate) fn do_map(maptype: c_int, arg: &[u8], mode: c_int, is_abbrev: bool) -> c_int {
    let mut args = MapArguments::default();
    let is_unmap = maptype == MAPTYPE_UNMAP;
    let mut result = str_to_mapargs(arg, is_unmap, &mut args);
    if result == 0 {
        result = buf_do_map(maptype, &args, mode, is_abbrev, Buf::current());
    }
    result
}

/// Clear all mappings (`:mapclear`) or abbreviations (`:abclear`).
fn do_mapclear(excmd: &mut ExArg, forceit: bool, abbr: bool) {
    let (mode, _) = get_map_mode(excmd.line.cmd(), forceit);
    let arg = excmd.line.arg();
    let local = arg == b"<buffer>";
    if !local && !arg.is_empty() {
        emsg(gettext(e_invarg));
        return;
    }
    map_clear_mode(Buf::current(), mode, local, abbr);
}

/// Add a mapping, copying both strings so that read-only ones can be used.
pub fn add_map(lhs: &[u8], rhs: &[u8], mode: c_int, buffer: bool) {
    let mut args = MapArguments::default();
    let noremap = MAPTYPE_NOREMAP;
    p_cpo(|cpo| set_maparg_lhs_rhs(lhs, rhs, LUA_NOREF, cpo, &mut args));
    args.buffer = buffer;

    buf_do_map(noremap, &args, mode, false, Buf::current());
}

/// `:map`, `:abbrev` and every prefixed variant of either, from the command
/// table's point of view: work out the mode and the map type from the command
/// name, then report whatever [`buf_do_map`] answers.
fn do_exmap(excmd: &mut ExArg, isabbrev: bool) {
    let cmd = excmd.line.cmd();
    let (mode, consumed) = get_map_mode(cmd, excmd.forceit || isabbrev);
    let maptype = match cmd.get(consumed).copied().unwrap_or(0) {
        b'n' => MAPTYPE_NOREMAP,
        b'u' => MAPTYPE_UNMAP,
        _ => MAPTYPE_MAP,
    };
    let mut parsed = MapArguments::default();
    let is_unmap = maptype == MAPTYPE_UNMAP;
    if str_to_mapargs(excmd.line.arg(), is_unmap, &mut parsed) != 0 {
        emsg(gettext(e_invarg)); // invalid arguments
        return;
    }
    let answer = buf_do_map(maptype, &parsed, mode, isabbrev, Buf::current());
    match answer {
        1 => {
            emsg(gettext(e_invarg));
        }
        2 => {
            emsg(gettext(if isabbrev { e_noabbr } else { e_nomap }));
        }
        5 | 6 => {
            let which = match (answer, isabbrev) {
                (5, true) => E_ABBREVIATION_ALREADY_EXISTS_FOR_STR,
                (5, false) => E_MAPPING_ALREADY_EXISTS_FOR_STR,
                (_, true) => E_GLOBAL_ABBREVIATION_ALREADY_EXISTS_FOR_STR,
                (_, false) => E_GLOBAL_MAPPING_ALREADY_EXISTS_FOR_STR,
            };
            let lhs = msg_bytes(parsed.lhs.as_bytes());
            emsg_text(tr_c!(which, lhs));
        }
        _ => {}
    }
}

/// `:abbreviate` and friends.
pub fn ex_abbreviate(excmd: &mut ExArg) {
    do_exmap(excmd, true) // almost the same as mapping
}

/// `:map` and friends.
pub fn ex_map(excmd: &mut ExArg) {
    // In a secure mode we print the mappings, for security reasons.
    if secure.get() != 0 {
        secure.set(2);
        msg_display(excmd.line.cstr_from(excmd.line.cmd), 0, false);
        msg_putchar(c_int::from(b'\n'));
    }
    do_exmap(excmd, false);
}

/// `:unmap` and friends.
pub fn ex_unmap(excmd: &mut ExArg) {
    do_exmap(excmd, false)
}

/// `:mapclear` and friends.
pub fn ex_mapclear(excmd: &mut ExArg) {
    let forceit = excmd.forceit;
    do_mapclear(excmd, forceit, false);
}

/// `:abclear` and friends.
pub fn ex_abclear(excmd: &mut ExArg) {
    do_mapclear(excmd, true, true);
}
