//! The search patterns themselves, and everything that remembers one.
//!
//! Two patterns are live at any time — the last one searched for and the
//! last one substituted with — and both are kept in the module-private
//! [`spats`] pair. [`search_regcomp`] is the compiler every caller goes
//! through: it fills in the remembered pattern when handed an empty one,
//! records the new one, and applies the `'ignorecase'`/`'smartcase'` rule
//! ([`pat_has_uppercase`]). The save/restore families exist because
//! incremental search, `:substitute` and the tag code all have to run a
//! search of their own without disturbing what the user last typed.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::guard::Suppress;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memory::XString;
use crate::optionstr::OptString;
use crate::regexp::{RE_BOTH, RE_LAST, RE_MAGIC, RE_SEARCH, RE_SUBST};
use crate::search::{SEARCH_HIS, SEARCH_KEEP, SEARCH_START};
use crate::shada::ShadaExtra;
use crate::types::{CmdModFlags, FAIL, Failed, NUL, String_0, Timestamp, Vv};
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use core::ffi::{c_char, c_int};

/// The offset ShaDa does not store, and the one a zeroed slot has.
const NO_OFFSET: SearchOffset = SearchOffset {
    dir: 0,
    line: false,
    end: false,
    off: 0,
};

/// One remembered search pattern, with the flags and offset it was used
/// with.
#[derive(Clone)]
pub(crate) struct SearchPattern {
    /// The pattern's text, or `None` when there is no pattern -- which is
    /// also what setting an empty one (`:let @/ = ""`) leaves.
    pub(crate) pat: Option<XString>,
    /// `'magic'` as it was when the pattern was used.
    pub(crate) magic: bool,
    /// `'smartcase'` was not to apply to it.
    pub(crate) no_scs: bool,
    /// When it was set, for ShaDa's merge.
    pub(crate) timestamp: Timestamp,
    /// The search offset, `n`/`N`'s to re-apply.
    pub(crate) off: SearchOffset,
    /// The ShaDa entry's keys this Nvim has no field for, written back
    /// verbatim.
    pub(crate) additional_data: Option<ShadaExtra>,
}

/// An empty pattern slot.
///
/// `magic` and `dir` are the only fields whose "nothing remembered yet"
/// value is not zero: [`spats`] starts out magic and searching forwards,
/// while the saved copies start out as C's zero-initialised statics.
const fn no_pattern(magic: bool, dir: c_char) -> SearchPattern {
    SearchPattern {
        pat: None,
        magic,
        no_scs: false,
        timestamp: 0,
        off: SearchOffset { dir, ..NO_OFFSET },
        additional_data: None,
    }
}

/// The two remembered patterns, indexed by `RE_SEARCH` and `RE_SUBST`.
///
/// The pair is also the interchange with ShaDa, which hands a slot over
/// wholesale ([`shada_set_pattern`]).
pub(crate) static spats: GlobalCell<[SearchPattern; 2]> = GlobalCell::new([
    no_pattern(true, b'/' as c_char),
    no_pattern(true, b'/' as c_char),
]);

/// Which of [`spats`] was used most recently — the one `RE_LAST` means.
pub(crate) static last_idx: GlobalCell<c_int> = GlobalCell::new(RE_SEARCH);

/// The pattern [`search_regcomp`] last compiled, as the user wrote it.
///
/// Kept separately from [`spats`] because `'rightleft'` reverses it and
/// because `SEARCH_KEEP` compiles patterns that are never remembered. The
/// "not found" messages quote this one.
static compiled_pat: GlobalCell<Option<XString>> = GlobalCell::new(None);

state_record! {
    /// What the save/restore pairs put aside while autocommands, user
    /// functions and incremental search run a search of their own.
    struct SavedSearch in SAVED as SavedField;
    /// [`spats`], kept while autocommands and user functions run.
    saved_spats: [SearchPattern; 2] = [no_pattern(false, 0), no_pattern(false, 0)];
    /// [`compiled_pat`], kept alongside.
    saved_compiled_pat: Option<XString> = None;
    saved_spats_last_idx: c_int = 0;
    saved_spats_no_hlsearch: bool = false;
    /// Nesting depth of [`save_search_patterns`]; only the outermost saves.
    save_level: c_int = 0;
    /// A second, independent copy of `spats[RE_SEARCH]`, for incremental
    /// search — which has to be able to put the pattern back even when it
    /// was cancelled from inside a user function.
    saved_last_search_spat: SearchPattern = no_pattern(false, 0);
    /// Nesting depth of [`save_last_search_pattern`].
    did_save_last_search_spat: c_int = 0;
    saved_last_idx: c_int = 0;
    saved_no_hlsearch: bool = false;
    /// The incremental-search highlight range, kept across `searchcount()`.
    saved_search_match_endcol: ColNr = 0;
    saved_search_match_lines: LineNr = 0;
}

/// A slot index as the array index it is.
fn slot(idx: c_int) -> usize {
    usize::from(idx != RE_SEARCH)
}

/// Look at the remembered pattern at `idx`.
fn with_spat<R>(idx: c_int, f: impl FnOnce(&SearchPattern) -> R) -> R {
    spats.with(|slots| f(&slots[slot(idx)]))
}

/// Change the remembered pattern at `idx` in place.
fn update_spat<R>(idx: c_int, f: impl FnOnce(&mut SearchPattern) -> R) -> R {
    spats.with_mut(|slots| f(&mut slots[slot(idx)]))
}

/// A copy of the remembered pattern's text at `idx`.
fn spat_text(idx: c_int) -> Option<XString> {
    with_spat(idx, |spat| spat.pat.clone())
}

/// Whether there is a pattern at `idx`.
fn has_spat(idx: c_int) -> bool {
    with_spat(idx, |spat| spat.pat.is_some())
}

/// Store `pat` at `idx`, answering what was there.
fn put_spat(idx: c_int, pat: SearchPattern) -> SearchPattern {
    update_spat(idx, |old| core::mem::replace(old, pat))
}

/// Compile a search pattern, remembering it as the caller asks.
///
/// - `pat_save == RE_SEARCH`: save `pat` in `spats[RE_SEARCH]` (a normal
///   search command); `RE_SUBST`: in `spats[RE_SUBST]` (`:substitute`);
///   `RE_BOTH`: in both (`:global`).
/// - `pat_use == RE_SEARCH`/`RE_SUBST`/`RE_LAST`: which remembered pattern
///   to use when `pat` is empty.
/// - `options & SEARCH_HIS`: put the pattern in the search history.
/// - `options & SEARCH_KEEP`: do not remember the pattern at all.
///
/// `regmatch` gets the compiled pattern and the ignore-case flag to match
/// it with. Answers the pattern that was compiled -- `pat` itself, or the
/// remembered one an empty `pat` stood for -- or `Err` when it does not
/// compile or there is no remembered one to fall back on.
pub fn search_regcomp(
    pat: &[u8],
    pat_save: c_int,
    pat_use: c_int,
    options: c_int,
    regmatch: &mut RegMMatch,
) -> Result<XString, Failed> {
    rc_did_emsg.set(false);
    let mut magic = magic_isset();
    // The pattern stops at a NUL, as upstream's copies of it did.
    let typed = cstr::until_nul(pat);

    // Which slot the pattern came out of, when it was a remembered one:
    // remembering it again in that same slot changes nothing.
    let mut from_slot = None;
    let source = if typed.is_empty() {
        // No pattern given: use a previously defined one.
        let idx = if pat_use == RE_LAST {
            last_idx.get()
        } else {
            pat_use
        };
        let found = with_spat(idx, |spat| {
            spat.pat.clone().map(|pat| (pat, spat.magic, spat.no_scs))
        });
        let Some((pat, spat_magic, no_scs)) = found else {
            // Never defined.
            let msg = if pat_use == RE_SUBST {
                e_nopresub
            } else {
                e_noprevre
            };
            emsg(gettext(msg));
            rc_did_emsg.set(true);
            return Err(Failed);
        };
        magic = spat_magic;
        no_smartcase.set(no_scs);
        from_slot = Some(idx);
        pat
    } else {
        if options & SEARCH_HIS != 0 {
            add_to_history(HIST_SEARCH, typed, true, NUL as u8);
        }
        XString::from_bytes(typed)
    };
    let text: &[u8] = &source;

    let rightleft_reverse = Win::current().w_onebuf_opt.wo_rl != 0
        && Win::current().w_onebuf_opt.wo_rlc.first_byte() == b's';
    compiled_pat.set(Some(if rightleft_reverse {
        XString::from_vec(reversed_text(text))
    } else {
        source.clone()
    }));

    // Remember the pattern, unless the caller or `:keeppatterns` asked
    // us not to.
    if options & SEARCH_KEEP == 0 && !cmdmod_has(CmdModFlags::KEEPPATTERNS) {
        for idx in [RE_SEARCH, RE_SUBST] {
            let wanted = pat_save == idx || pat_save == RE_BOTH;
            if wanted && from_slot != Some(idx) {
                save_re_pat(idx, text, magic);
            }
        }
    }

    regmatch.rmm_ic = c_int::from(ignorecase(text));
    regmatch.rmm_maxcol = 0;
    regmatch.regprog = vim_regcomp(source.as_cstr(), if magic { RE_MAGIC } else { 0 });
    if regmatch.regprog.is_none() {
        return Err(Failed);
    }
    Ok(source)
}

/// [`search_regcomp`] over a pattern the caller holds -- empty for the
/// remembered one `pat_use` names: the compiled match, owned, and when
/// `want_used` the pattern that was compiled, copied out (what `:global`
/// reports when nothing matched). `None` when it does not compile or there
/// is no remembered pattern, which has been reported.
pub(crate) fn search_regcomp_owned(
    pat: &CStr,
    pat_save: c_int,
    pat_use: c_int,
    options: c_int,
    want_used: bool,
) -> Option<(RegMMatch, Option<XString>)> {
    let mut regmatch = RegMMatch::default();
    let pat = pat.to_bytes();
    let used = search_regcomp(pat, pat_save, pat_use, options, &mut regmatch).ok()?;
    Some((regmatch, want_used.then_some(used)))
}

/// The pattern [`search_regcomp`] last compiled, copied out; empty before
/// the first compile.
pub fn get_search_pat() -> XString {
    compiled_pat.with(|pat| pat.clone().unwrap_or_default())
}

/// Remember `pat` as the pattern at `idx`, and as the last one used. The
/// text stops at a NUL.
pub(crate) fn save_re_pat(idx: c_int, pat: &[u8], magic: bool) {
    let text = XString::from_bytes(cstr::until_nul(pat));
    update_spat(idx, |spat| {
        spat.pat = Some(text);
        spat.magic = magic;
        spat.no_scs = no_smartcase.get();
        spat.timestamp = os_time();
        spat.additional_data = None;
    });
    last_idx.set(idx);
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls() {
        redraw_all_later(UPD_SOME_VALID);
    }
    set_no_hlsearch(false);
}

/// Save the search patterns so they can be restored later. Used around
/// autocommands and user functions; only the outermost call saves.
pub fn save_search_patterns() {
    if save_level.replace(save_level.get() + 1) != 0 {
        return;
    }
    saved_spats.set(spats.with(Clone::clone));
    saved_compiled_pat.set(compiled_pat.with(Clone::clone));
    saved_spats_last_idx.set(last_idx.get());
    saved_spats_no_hlsearch.set(no_hlsearch.get());
}

/// Undo one [`save_search_patterns`]; the outermost call restores.
pub fn restore_search_patterns() {
    save_level.set(save_level.get() - 1);
    if save_level.get() != 0 {
        return;
    }
    let saved = saved_spats.replace([no_pattern(false, 0), no_pattern(false, 0)]);
    // Dropped after the cell is released: nothing here runs, but the old
    // pair is the caller's no longer.
    let _replaced = spats.with_mut(|slots| core::mem::replace(slots, saved));
    set_vv_searchforward();
    compiled_pat.set(saved_compiled_pat.take());
    last_idx.set(saved_spats_last_idx.get());
    set_no_hlsearch(saved_spats_no_hlsearch.get());
}

/// Save the search pattern for incremental search.
///
/// Similar to but separate from [`save_search_patterns`]: the pattern has
/// to be restorable when incremental search is cancelled even if that
/// happens inside a user function. Only the outermost call saves.
pub fn save_last_search_pattern() {
    did_save_last_search_spat.set(did_save_last_search_spat.get() + 1);
    if did_save_last_search_spat.get() != 1 {
        return;
    }
    saved_last_search_spat.set(with_spat(RE_SEARCH, Clone::clone));
    saved_last_idx.set(last_idx.get());
    saved_no_hlsearch.set(no_hlsearch.get());
}

/// Undo one [`save_last_search_pattern`]; the outermost call restores.
pub fn restore_last_search_pattern() {
    did_save_last_search_spat.set(did_save_last_search_spat.get() - 1);
    if did_save_last_search_spat.get() > 0 {
        return;
    }
    if did_save_last_search_spat.get() != 0 {
        let msg =
            c"restore_last_search_pattern() called more often than save_last_search_pattern()";
        iemsg(msg);
        return;
    }
    let saved = saved_last_search_spat.replace(no_pattern(false, 0));
    put_spat(RE_SEARCH, saved);
    set_vv_searchforward();
    last_idx.set(saved_last_idx.get());
    set_no_hlsearch(saved_no_hlsearch.get());
}

/// Save the incremental-search highlighting variables, so that calling
/// `searchcount()` does not invalidate the highlighting.
pub(crate) fn save_incsearch_state() {
    saved_search_match_endcol.set(search_match_endcol.get());
    saved_search_match_lines.set(search_match_lines.get());
}

pub(crate) fn restore_incsearch_state() {
    search_match_endcol.set(saved_search_match_endcol.get());
    search_match_lines.set(saved_search_match_lines.get());
}

/// The pattern `/` and `?` last searched for, copied out, or `None` when
/// there is none yet.
pub fn last_search_pattern() -> Option<XString> {
    spat_text(RE_SEARCH)
}

/// Whether `/` and `?` have searched for anything yet.
pub(crate) fn has_last_search_pattern() -> bool {
    has_spat(RE_SEARCH)
}

/// Whichever of the two patterns was used last, copied out, or `None` when
/// there is none yet.
pub fn last_search_pat() -> Option<XString> {
    spat_text(last_idx.get())
}

/// The `:substitute` pattern, copied out — what `/` falls back on when no
/// search pattern has been used yet.
pub(crate) fn substitute_pattern() -> Option<XString> {
    spat_text(RE_SUBST)
}

/// The offset `/` and `?` remember alongside the search pattern: the
/// `e`/`s`/`b` suffix and its `+n`/`-n`.
///
/// It lives in `spats[RE_SEARCH]` rather than beside the command that
/// parsed it because `n` and `N` re-apply it, and because `'cpoptions'`
/// "o" makes a *line* offset one-shot. [`do_search`](super::do_search)
/// parses into it and puts the old one back for `SEARCH_KEEP`.
pub(crate) fn search_offset() -> SearchOffset {
    with_spat(RE_SEARCH, |spat| spat.off)
}

pub(crate) fn set_search_offset(off: SearchOffset) {
    update_spat(RE_SEARCH, |spat| spat.off = off);
}

/// Whether the last used pattern's text is `pat` — what the search-count
/// cache compares itself against.
pub(crate) fn last_used_pattern_is(pat: &[u8]) -> bool {
    with_spat(last_idx.get(), |spat| {
        spat.pat.as_deref().is_some_and(|live| live == pat)
    })
}

/// Replace the last used pattern's string, keeping its flags and offset.
/// Only `searchcount()` does this, and only under
/// [`save_last_search_pattern`], which puts the real one back.
pub(crate) fn replace_last_used_pattern(pat: &[u8]) {
    let text = XString::from_bytes(cstr::until_nul(pat));
    update_spat(last_idx.get(), |spat| spat.pat = Some(text));
}

/// Whether case should be ignored for pattern `pat`, per `'ignorecase'`
/// and `'smartcase'`. The pattern stops at a NUL.
pub fn ignorecase(pat: &[u8]) -> bool {
    ignorecase_opt(pat, p_ic(), p_scs())
}

/// As [`ignorecase`] but with the `'ignorecase'`/`'smartcase'` values
/// passed in.
pub fn ignorecase_opt(pat: &[u8], ic_in: bool, scs: bool) -> bool {
    let mut ic = ic_in;
    // 'infercase' completion does its own case handling.
    if ic
        && !no_smartcase.get()
        && scs
        && !(ctrl_x_mode_not_default() && Buf::current().b_p_inf != 0)
    {
        ic = !pat_has_uppercase(pat);
    }
    no_smartcase.set(false);
    ic
}

/// Whether pattern `pat` has an uppercase character in it — the
/// `'smartcase'` test. The pattern stops at a NUL.
pub fn pat_has_uppercase(pat: &[u8]) -> bool {
    let pat = cstr::until_nul(pat);
    // Which of `\`, `%` and `_` introduce an escape depends on the
    // pattern's own magicness, which only a parse can tell us.
    let is_magic = c_int::from(magic_isset());
    let magic_val = cstr::with_terminated(pat, |pat| {
        skip_regexp_magic(pat.to_bytes_with_nul(), c_int::from(NUL), is_magic).1
    });
    let at = |i: usize| cstr::byte_at(pat, i);
    let mut i = 0;
    while i < pat.len() {
        let len = c_cluster_len(&pat[i..]);
        if len > 1 {
            if mb_isupper(char_at(&pat[i..])) {
                return true;
            }
            i += len;
        } else if pat[i] == b'\\' && magic_val <= MAGIC_ON {
            // Skip "\_X" and "\%X", else "\X".
            i += match at(i + 1) {
                0 => 1,
                b'_' | b'%' if at(i + 2) != 0 => 3,
                _ => 2,
            };
        } else if (pat[i] == b'%' || pat[i] == b'_') && magic_val == MAGIC_ALL {
            // Skip "%X" and "_X".
            i += if at(i + 1) != 0 { 2 } else { 1 };
        } else if mb_isupper(c_int::from(pat[i])) {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

/// Reset the search direction to forwards. For `gd` and `gD`.
pub fn reset_search_dir() {
    update_spat(RE_SEARCH, |spat| spat.off.dir = b'/' as c_char);
    set_vv_searchforward();
}

/// Set the last search pattern, for `:let @/ =` and the ShaDa file. The
/// saved copy is set too, so that this works inside an autocommand. The
/// pattern stops at a NUL; an empty one means that nothing should be
/// matched.
pub fn set_last_search_pat(s: &[u8], idx: c_int, magic: bool, setlast: bool) {
    let text = cstr::until_nul(s);
    let pat = (!text.is_empty()).then(|| XString::from_bytes(text));
    put_spat(
        idx,
        SearchPattern {
            pat,
            magic,
            no_scs: false,
            timestamp: os_time(),
            off: SearchOffset {
                dir: b'/' as c_char,
                ..NO_OFFSET
            },
            additional_data: None,
        },
    );
    set_vv_searchforward();
    if setlast {
        last_idx.set(idx);
    }
    if save_level.get() != 0 {
        // Upstream takes the flags from slot 0 whichever slot is being
        // set, then overwrites only the string. Preserved.
        let mut saved = with_spat(RE_SEARCH, Clone::clone);
        saved.pat = spat_text(idx);
        saved_spats.with_mut(|slots| slots[slot(idx)] = saved);
        saved_spats_last_idx.set(last_idx.get());
    }
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls() && idx == last_idx.get() && !no_hlsearch.get() {
        redraw_all_later(UPD_SOME_VALID);
    }
}

/// Compile the last used search pattern, for highlighting every match in a
/// window. Leaves no program in `regmatch` when there is no pattern.
pub fn last_pat_prog(regmatch: &mut RegMMatch) {
    if !has_spat(last_idx.get()) {
        regmatch.regprog = None;
        return;
    }
    // So it doesn't beep if the pattern is bad.
    let _no_emsg = Suppress::emsg();
    let _ = search_regcomp(b"", 0, last_idx.get(), SEARCH_KEEP, regmatch);
}

/// Set the direction `n` repeats in, for `:let v:searchforward =`.
pub fn set_search_direction(cdir: c_int) {
    // The direction is `/` or `?`, a byte.
    let dir = cdir as c_char;
    update_spat(RE_SEARCH, |spat| spat.off.dir = dir);
}

/// Publish the search direction as `v:searchforward`.
pub(crate) fn set_vv_searchforward() {
    let forward = with_spat(RE_SEARCH, |spat| spat.off.dir == b'/' as c_char);
    set_vim_var_nr(Vv::Searchforward, VarNumber::from(forward));
}

/// Whether `pattern` matches zero-width.
///
/// With `move_to_match` the search starts at the top of the buffer, else
/// at `cur`. `direction` is `FORWARD` or `BACKWARD`.
///
/// Answers 1, 0, or -1 for failure.
pub(crate) fn is_zero_width(
    pattern: &[u8],
    move_to_match: bool,
    cur: Pos,
    direction: Direction,
) -> c_int {
    let mut regmatch = RegMMatch::default();
    let called_emsg_before = called_emsg.get();
    if search_regcomp(pattern, RE_SEARCH, RE_SEARCH, SEARCH_KEEP, &mut regmatch).is_err() {
        return -1;
    }

    // Init startcol correctly.
    regmatch.startpos[0].col = -1;
    // Searching from the top starts at the zeroed position; searching
    // from the cursor accepts a match at the cursor itself.
    let mut pos = if move_to_match { Pos::default() } else { cur };
    let flag = if move_to_match { 0 } else { SEARCH_START };

    let mut result = -1;
    let (w, b) = (Some(Win::current()), Buf::current());
    let found = searchit(
        w,
        b,
        &mut pos,
        None,
        direction,
        pattern,
        1,
        SEARCH_KEEP + flag,
        RE_SEARCH,
        None,
    );
    if found != FAIL {
        // A zero-width pattern matches somewhere; find where, then ask
        // whether its start and end are the same position.
        let nmatched = loop {
            regmatch.startpos[0].col += 1;
            let col = regmatch.startpos[0].col;
            let nmatched = vim_regexec_multi(&mut regmatch, w, b, pos.lnum, col, None, None);
            let short_of_the_match = if direction == FORWARD {
                regmatch.startpos[0].col < pos.col
            } else {
                regmatch.startpos[0].col > pos.col
            };
            if nmatched != 0 || regmatch.regprog.is_none() || !short_of_the_match {
                break nmatched;
            }
        };
        if called_emsg.get() == called_emsg_before {
            result = c_int::from(
                nmatched != 0
                    && regmatch.startpos[0].lnum == regmatch.endpos[0].lnum
                    && regmatch.startpos[0].col == regmatch.endpos[0].col,
            );
        }
    }
    result
}

/// One remembered pattern as ShaDa reads and writes it, owned.
pub(crate) struct ShadaPattern {
    pub(crate) pat: String_0,
    pub(crate) magic: bool,
    pub(crate) no_scs: bool,
    pub(crate) timestamp: Timestamp,
    pub(crate) off: SearchOffset,
    pub(crate) extra: Option<ShadaExtra>,
}

/// The slot ShaDa means: the substitute pattern or the search pattern.
fn shada_slot(is_substitute: bool) -> c_int {
    if is_substitute { RE_SUBST } else { RE_SEARCH }
}

/// A copy of the last search (or, with `is_substitute`, substitute)
/// pattern, or `None` when there is none. A substitute pattern's offset is
/// not part of what ShaDa stores.
pub(crate) fn shada_pattern(is_substitute: bool) -> Option<ShadaPattern> {
    with_spat(shada_slot(is_substitute), |slot| {
        let text = slot.pat.as_ref()?;
        Some(ShadaPattern {
            pat: String_0::from_bytes(text),
            magic: slot.magic,
            no_scs: slot.no_scs,
            timestamp: slot.timestamp,
            off: if is_substitute { NO_OFFSET } else { slot.off },
            extra: slot.additional_data.clone(),
        })
    })
}

/// When the last search (or substitute) pattern was set, or `None` when
/// there is none.
pub(crate) fn shada_pattern_timestamp(is_substitute: bool) -> Option<Timestamp> {
    with_spat(shada_slot(is_substitute), |slot| {
        slot.pat.is_some().then_some(slot.timestamp)
    })
}

/// Make `pattern` the last search (or substitute) pattern, taking it over.
pub(crate) fn shada_set_pattern(is_substitute: bool, pattern: ShadaPattern) {
    let idx = shada_slot(is_substitute);
    let spat_new = SearchPattern {
        // The text stops at a NUL, as the C string it was handed over as.
        pat: Some(XString::from_bytes(cstr::until_nul(pattern.pat.as_bytes()))),
        magic: pattern.magic,
        no_scs: pattern.no_scs,
        timestamp: pattern.timestamp,
        off: if is_substitute {
            NO_OFFSET
        } else {
            pattern.off
        },
        additional_data: pattern.extra,
    };
    put_spat(idx, spat_new);
    if !is_substitute {
        set_vv_searchforward();
    }
}

/// Record which of the two patterns was used last.
pub fn set_last_used_pattern(is_substitute_pattern: bool) {
    last_idx.set(if is_substitute_pattern {
        RE_SUBST
    } else {
        RE_SEARCH
    });
}

/// Whether the search pattern, rather than the substitute pattern, was the
/// one used last.
pub fn search_was_last_used() -> bool {
    last_idx.get() == RE_SEARCH
}
