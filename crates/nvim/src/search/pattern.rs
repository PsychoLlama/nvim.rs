//! The search patterns themselves, and everything that remembers one.
//!
//! Two patterns are live at any time — the last one searched for and the
//! last one substituted with — and both are kept in the module-private
//! [`SPATS`] pair. [`search_regcomp`] is the compiler every caller goes
//! through: it fills in the remembered pattern when handed an empty one,
//! records the new one, and applies the `'ignorecase'`/`'smartcase'` rule
//! ([`pat_has_uppercase`]). The save/restore families exist because
//! incremental search, `:substitute` and the tag code all have to run a
//! search of their own without disturbing what the user last typed.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::guard::Suppress;
use crate::mbyte::Codepoint;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memory::XString;
use crate::optionstr::OptString;
use crate::regexp::low_byte;
use crate::regexp::{MAGIC_ALL, MAGIC_ON, RE_BOTH, RE_LAST, RE_MAGIC, RE_SEARCH, RE_SUBST};
use crate::search::{SEARCH_HIS, SEARCH_KEEP, SEARCH_START};
use crate::shada::ShadaExtra;
use crate::types::{ApiString, CmdModFlags, Timestamp, Vv};
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
#[derive(Clone, Debug)]
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
/// value is not zero: [`SPATS`] starts out magic and searching forwards,
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
pub(crate) static SPATS: GlobalCell<[SearchPattern; 2]> = GlobalCell::new([
    no_pattern(true, b'/'.cast_signed()),
    no_pattern(true, b'/'.cast_signed()),
]);

/// Which of [`SPATS`] was used most recently — the one `RE_LAST` means.
pub(crate) static LAST_IDX: GlobalCell<c_int> = GlobalCell::new(RE_SEARCH);

/// The pattern [`search_regcomp`] last compiled, as the user wrote it.
///
/// Kept separately from [`SPATS`] because `'rightleft'` reverses it and
/// because `SEARCH_KEEP` compiles patterns that are never remembered. The
/// "not found" messages quote this one.
static COMPILED_PAT: GlobalCell<Option<XString>> = GlobalCell::new(None);

state_record! {
    /// What the save/restore pairs put aside while autocommands, user
    /// functions and incremental search run a search of their own.
    struct SavedSearch in SAVED as SavedField;
    /// [`SPATS`], kept while autocommands and user functions run.
    SAVED_SPATS: [SearchPattern; 2] = [no_pattern(false, 0), no_pattern(false, 0)];
    /// [`COMPILED_PAT`], kept alongside.
    SAVED_COMPILED_PAT: Option<XString> = None;
    SAVED_SPATS_LAST_IDX: c_int = 0;
    SAVED_SPATS_NO_HLSEARCH: bool = false;
    /// Nesting depth of [`save_search_patterns`]; only the outermost saves.
    SAVE_LEVEL: c_int = 0;
    /// A second, independent copy of `SPATS[RE_SEARCH]`, for incremental
    /// search — which has to be able to put the pattern back even when it
    /// was cancelled from inside a user function.
    SAVED_LAST_SEARCH_SPAT: SearchPattern = no_pattern(false, 0);
    /// Nesting depth of [`save_last_search_pattern`].
    DID_SAVE_LAST_SEARCH_SPAT: c_int = 0;
    SAVED_LAST_IDX: c_int = 0;
    SAVED_NO_HLSEARCH: bool = false;
    /// The incremental-search highlight range, kept across `searchcount()`.
    SAVED_SEARCH_MATCH_ENDCOL: ColNr = ColNr::ZERO;
    SAVED_SEARCH_MATCH_LINES: i32 = 0;
}

/// A slot index as the array index it is.
fn slot(idx: c_int) -> usize {
    usize::from(idx != RE_SEARCH)
}

/// Look at the remembered pattern at `idx`.
fn with_spat<R>(idx: c_int, f: impl FnOnce(&SearchPattern) -> R) -> R {
    SPATS.with(|slots| f(&slots[slot(idx)]))
}

/// Change the remembered pattern at `idx` in place.
fn update_spat<R>(idx: c_int, f: impl FnOnce(&mut SearchPattern) -> R) -> R {
    SPATS.with_mut(|slots| f(&mut slots[slot(idx)]))
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
/// - `pat_save == RE_SEARCH`: save `pat` in `SPATS[RE_SEARCH]` (a normal
///   search command); `RE_SUBST`: in `SPATS[RE_SUBST]` (`:substitute`);
///   `RE_BOTH`: in both (`:global`).
/// - `pat_use == RE_SEARCH`/`RE_SUBST`/`RE_LAST`: which remembered pattern
///   to use when `pat` is empty.
/// - `options & SEARCH_HIS`: put the pattern in the search history.
/// - `options & SEARCH_KEEP`: do not remember the pattern at all.
///
/// `regmatch` gets the compiled pattern and the ignore-case flag to match
/// it with. Answers the pattern that was compiled -- `pat` itself, or the
/// remembered one an empty `pat` stood for -- or, when it does not compile
/// or there is no remembered one to fall back on, what was said about it.
pub(crate) fn search_regcomp(
    pat: &[u8],
    pat_save: c_int,
    pat_use: c_int,
    options: c_int,
    regmatch: &mut RegMMatch,
) -> Result<XString, Refused> {
    let mut magic = magic_isset();
    // The pattern stops at a NUL, as upstream's copies of it did.
    let typed = cstr::until_nul(pat);

    // Which slot the pattern came out of, when it was a remembered one:
    // remembering it again in that same slot changes nothing.
    let mut from_slot = None;
    let source = if typed.is_empty() {
        // No pattern given: use a previously defined one.
        let idx = if pat_use == RE_LAST {
            LAST_IDX.get()
        } else {
            pat_use
        };
        let found = with_spat(idx, |spat| {
            spat.pat.clone().map(|pat| (pat, spat.magic, spat.no_scs))
        });
        let Some((pat, spat_magic, no_scs)) = found else {
            // Never defined.
            let msg = if pat_use == RE_SUBST {
                E_NOPRESUB
            } else {
                E_NOPREVRE
            };
            emsg(gettext(msg));
            return Err(Refused::Invalid);
        };
        magic = spat_magic;
        NO_SMARTCASE.set(no_scs);
        from_slot = Some(idx);
        pat
    } else {
        if options & SEARCH_HIS != 0 {
            add_to_history(HIST_SEARCH, typed, true, 0);
        }
        XString::from_bytes(typed)
    };
    let text: &[u8] = &source;

    let rightleft_reverse = Win::current().w_onebuf_opt.wo_rl != 0
        && Win::current().w_onebuf_opt.wo_rlc.first_byte() == b's';
    COMPILED_PAT.set(Some(if rightleft_reverse {
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
    regmatch.rmm_maxcol = ColNr::ZERO;
    regmatch.regprog = Some(vim_regcomp_ext(
        source.as_cstr(),
        if magic { RE_MAGIC } else { 0 },
        0,
    )?);
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
pub(crate) fn get_search_pat() -> XString {
    COMPILED_PAT.with(|pat| pat.clone().unwrap_or_default())
}

/// Remember `pat` as the pattern at `idx`, and as the last one used. The
/// text stops at a NUL.
pub(crate) fn save_re_pat(idx: c_int, pat: &[u8], magic: bool) {
    let text = XString::from_bytes(cstr::until_nul(pat));
    update_spat(idx, |spat| {
        spat.pat = Some(text);
        spat.magic = magic;
        spat.no_scs = NO_SMARTCASE.get();
        spat.timestamp = os_time();
        spat.additional_data = None;
    });
    LAST_IDX.set(idx);
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls() {
        redraw_all_later(UPD_SOME_VALID);
    }
    set_no_hlsearch(false);
}

/// Save the search patterns so they can be restored later. Used around
/// autocommands and user functions; only the outermost call saves.
pub fn save_search_patterns() {
    if SAVE_LEVEL.replace(SAVE_LEVEL.get() + 1) != 0 {
        return;
    }
    SAVED_SPATS.set(SPATS.with(Clone::clone));
    SAVED_COMPILED_PAT.set(COMPILED_PAT.with(Clone::clone));
    SAVED_SPATS_LAST_IDX.set(LAST_IDX.get());
    SAVED_SPATS_NO_HLSEARCH.set(NO_HLSEARCH.get());
}

/// Undo one [`save_search_patterns`]; the outermost call restores.
pub fn restore_search_patterns() {
    SAVE_LEVEL.set(SAVE_LEVEL.get() - 1);
    if SAVE_LEVEL.get() != 0 {
        return;
    }
    let saved = SAVED_SPATS.replace([no_pattern(false, 0), no_pattern(false, 0)]);
    // Dropped after the cell is released: nothing here runs, but the old
    // pair is the caller's no longer.
    let _replaced = SPATS.with_mut(|slots| core::mem::replace(slots, saved));
    set_vv_searchforward();
    COMPILED_PAT.set(SAVED_COMPILED_PAT.take());
    LAST_IDX.set(SAVED_SPATS_LAST_IDX.get());
    set_no_hlsearch(SAVED_SPATS_NO_HLSEARCH.get());
}

/// Save the search pattern for incremental search.
///
/// Similar to but separate from [`save_search_patterns`]: the pattern has
/// to be restorable when incremental search is cancelled even if that
/// happens inside a user function. Only the outermost call saves.
pub fn save_last_search_pattern() {
    DID_SAVE_LAST_SEARCH_SPAT.set(DID_SAVE_LAST_SEARCH_SPAT.get() + 1);
    if DID_SAVE_LAST_SEARCH_SPAT.get() != 1 {
        return;
    }
    SAVED_LAST_SEARCH_SPAT.set(with_spat(RE_SEARCH, Clone::clone));
    SAVED_LAST_IDX.set(LAST_IDX.get());
    SAVED_NO_HLSEARCH.set(NO_HLSEARCH.get());
}

/// Undo one [`save_last_search_pattern`]; the outermost call restores.
pub fn restore_last_search_pattern() {
    DID_SAVE_LAST_SEARCH_SPAT.set(DID_SAVE_LAST_SEARCH_SPAT.get() - 1);
    if DID_SAVE_LAST_SEARCH_SPAT.get() > 0 {
        return;
    }
    if DID_SAVE_LAST_SEARCH_SPAT.get() != 0 {
        let msg =
            c"restore_last_search_pattern() called more often than save_last_search_pattern()";
        iemsg(msg);
        return;
    }
    let saved = SAVED_LAST_SEARCH_SPAT.replace(no_pattern(false, 0));
    put_spat(RE_SEARCH, saved);
    set_vv_searchforward();
    LAST_IDX.set(SAVED_LAST_IDX.get());
    set_no_hlsearch(SAVED_NO_HLSEARCH.get());
}

/// Save the incremental-search highlighting variables, so that calling
/// `searchcount()` does not invalidate the highlighting.
pub(crate) fn save_incsearch_state() {
    SAVED_SEARCH_MATCH_ENDCOL.set(SEARCH_MATCH_ENDCOL.get());
    SAVED_SEARCH_MATCH_LINES.set(SEARCH_MATCH_LINES.get());
}

pub(crate) fn restore_incsearch_state() {
    SEARCH_MATCH_ENDCOL.set(SAVED_SEARCH_MATCH_ENDCOL.get());
    SEARCH_MATCH_LINES.set(SAVED_SEARCH_MATCH_LINES.get());
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
    spat_text(LAST_IDX.get())
}

/// The `:substitute` pattern, copied out — what `/` falls back on when no
/// search pattern has been used yet.
pub(crate) fn substitute_pattern() -> Option<XString> {
    spat_text(RE_SUBST)
}

/// The offset `/` and `?` remember alongside the search pattern: the
/// `e`/`s`/`b` suffix and its `+n`/`-n`.
///
/// It lives in `SPATS[RE_SEARCH]` rather than beside the command that
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
    with_spat(LAST_IDX.get(), |spat| {
        spat.pat.as_deref().is_some_and(|live| live == pat)
    })
}

/// Replace the last used pattern's string, keeping its flags and offset.
/// Only `searchcount()` does this, and only under
/// [`save_last_search_pattern`], which puts the real one back.
pub(crate) fn replace_last_used_pattern(pat: &[u8]) {
    let text = XString::from_bytes(cstr::until_nul(pat));
    update_spat(LAST_IDX.get(), |spat| spat.pat = Some(text));
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
        && !NO_SMARTCASE.get()
        && scs
        && !(ctrl_x_mode_not_default() && Buf::current().b_p_inf != 0)
    {
        ic = !pat_has_uppercase(pat);
    }
    NO_SMARTCASE.set(false);
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
        skip_regexp_magic(pat.to_bytes_with_nul(), 0, is_magic).1
    });
    let at = |i: usize| cstr::byte_at(pat, i);
    let mut i = 0;
    while i < pat.len() {
        let len = c_cluster_len(&pat[i..]);
        if len > 1 {
            if char_at(&pat[i..]).is_upper() {
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
        } else if Codepoint::from(pat[i]).is_upper() {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

/// Reset the search direction to forwards. For `gd` and `gD`.
pub fn reset_search_dir() {
    update_spat(RE_SEARCH, |spat| spat.off.dir = b'/'.cast_signed());
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
                dir: b'/'.cast_signed(),
                ..NO_OFFSET
            },
            additional_data: None,
        },
    );
    set_vv_searchforward();
    if setlast {
        LAST_IDX.set(idx);
    }
    if SAVE_LEVEL.get() != 0 {
        // Upstream takes the flags from slot 0 whichever slot is being
        // set, then overwrites only the string. Preserved.
        let mut saved = with_spat(RE_SEARCH, Clone::clone);
        saved.pat = spat_text(idx);
        SAVED_SPATS.with_mut(|slots| slots[slot(idx)] = saved);
        SAVED_SPATS_LAST_IDX.set(LAST_IDX.get());
    }
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls() && idx == LAST_IDX.get() && !NO_HLSEARCH.get() {
        redraw_all_later(UPD_SOME_VALID);
    }
}

/// Compile the last used search pattern, for highlighting every match in a
/// window. Leaves no program in `regmatch` when there is no pattern.
pub fn last_pat_prog(regmatch: &mut RegMMatch) {
    if !has_spat(LAST_IDX.get()) {
        regmatch.regprog = None;
        return;
    }
    // So it doesn't beep if the pattern is bad.
    let _no_emsg = Suppress::emsg();
    let _ = search_regcomp(b"", 0, LAST_IDX.get(), SEARCH_KEEP, regmatch);
}

/// Set the direction `n` repeats in, for `:let v:searchforward =`.
pub fn set_search_direction(cdir: c_int) {
    // The direction is `/` or `?`, a byte.
    let dir = low_byte(cdir).cast_signed();
    update_spat(RE_SEARCH, |spat| spat.off.dir = dir);
}

/// Publish the search direction as `v:searchforward`.
pub(crate) fn set_vv_searchforward() {
    let forward = with_spat(RE_SEARCH, |spat| spat.off.dir == b'/'.cast_signed());
    set_vim_var_nr(Vv::Searchforward, VarNumber::from(forward));
}

/// Whether `pattern` matches zero-width.
///
/// With `move_to_match` the search starts at the top of the buffer, else
/// at `cur`. `direction` is `FORWARD` or `BACKWARD`.
///
/// `None` when there is no telling: the pattern does not compile (which has
/// been said), does not match, or matching it reported an error.
pub(crate) fn is_zero_width(
    pattern: &[u8],
    move_to_match: bool,
    cur: Pos,
    direction: Direction,
) -> Option<bool> {
    let mut regmatch = RegMMatch::default();
    let called_emsg_before = CALLED_EMSG.get();
    search_regcomp(pattern, RE_SEARCH, RE_SEARCH, SEARCH_KEEP, &mut regmatch).ok()?;

    // Init startcol correctly.
    regmatch.startpos[0].col = ColNr::NONE;
    // Searching from the top starts at the zeroed position; searching
    // from the cursor accepts a match at the cursor itself.
    let mut pos = if move_to_match { Pos::default() } else { cur };
    let flag = if move_to_match { 0 } else { SEARCH_START };

    let (w, b) = (Some(Win::current()), Buf::current());
    searchit(
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
    )
    .ok()?;
    // A zero-width pattern matches somewhere; find where, then ask whether
    // its start and end are the same position.
    let nmatched = loop {
        regmatch.startpos[0].col += 1;
        let col = regmatch.startpos[0].col;
        let nmatched = vim_regexec_multi(&mut regmatch, w, b, pos.lnum, col, None, None);
        let short_of_the_match = if direction == FORWARD {
            regmatch.startpos[0].col < pos.col
        } else {
            regmatch.startpos[0].col > pos.col
        };
        if nmatched != LineNr::ZERO || regmatch.regprog.is_none() || !short_of_the_match {
            break nmatched;
        }
    };
    (CALLED_EMSG.get() == called_emsg_before).then(|| {
        nmatched != LineNr::ZERO
            && regmatch.startpos[0].lnum == regmatch.endpos[0].lnum
            && regmatch.startpos[0].col == regmatch.endpos[0].col
    })
}

/// One remembered pattern as ShaDa reads and writes it, owned.
#[derive(Debug)]
pub(crate) struct ShadaPattern {
    pub(crate) pat: ApiString,
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
            pat: ApiString::from_bytes(text),
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

/// When the last search (or substitute) pattern was cleared, or `None`
/// when there is one -- so that ShaDa does not bring back a pattern this
/// session cleared (`:let @/ = ""`) after the file was written.
pub(crate) fn shada_cleared_pattern_timestamp(is_substitute: bool) -> Option<Timestamp> {
    with_spat(shada_slot(is_substitute), |slot| {
        slot.pat.is_none().then_some(slot.timestamp)
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
    LAST_IDX.set(if is_substitute_pattern {
        RE_SUBST
    } else {
        RE_SEARCH
    });
}

/// Whether the search pattern, rather than the substitute pattern, was the
/// one used last.
pub fn search_was_last_used() -> bool {
    LAST_IDX.get() == RE_SEARCH
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_fixture::Fixture;
    use crate::ex_docmd::state::CMDMOD;

    /// Compile `a\xc0` with `'rightleft'` on and `'rightleftcmd'` set to
    /// `rlc`, under `:keeppatterns` so the remembered search patterns of
    /// whatever runs next are not disturbed, and answer the pattern
    /// [`get_search_pat`] then holds.
    fn compiled_rightleft(rlc: &str) -> XString {
        let _fixture = Fixture::new();
        let mut win = Win::current();
        win.w_onebuf_opt.wo_rl = 1;
        win.w_onebuf_opt.wo_rlc = Some(XString::from(rlc));
        let flags = CMDMOD.with(|c| c.cmod_flags);
        CMDMOD.with_mut(|c| c.cmod_flags |= CmdModFlags::KEEPPATTERNS);
        let mut regmatch = RegMMatch::default();
        let rc = search_regcomp(b"a\xc0", 0, 0, 0, &mut regmatch);
        CMDMOD.with_mut(|c| c.cmod_flags = flags);
        assert!(rc.is_ok(), "the pattern compiles");
        get_search_pat()
    }

    #[test]
    fn a_right_to_left_pattern_is_reversed_bytewise() {
        // `a\xc0` is deliberately not valid UTF-8: `\xc0` announces a
        // two-byte sequence that is not there. Reversing it has to move
        // bytes, not characters, or the reversal reads past the end of the
        // pattern.
        assert_eq!(
            &compiled_rightleft("s")[..],
            b"\xc0a",
            "the pattern was reversed"
        );
    }

    #[test]
    fn only_a_rightleftcmd_of_s_reverses_the_pattern() {
        // `'rightleft'` alone is not the trigger: `'rightleftcmd'` says
        // which command lines are typed right to left, and only `s`
        // (search) is a value the reversal applies to.
        assert_eq!(
            &compiled_rightleft("")[..],
            b"a\xc0",
            "the pattern was kept as typed"
        );
    }
}
