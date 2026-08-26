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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::guard::Suppress;
use crate::regexp::{RE_BOTH, RE_LAST, RE_MAGIC, RE_SEARCH, RE_SUBST};
use crate::search::{SEARCH_HIS, SEARCH_KEEP, SEARCH_START};
use crate::types::{CmdModFlags, FAIL, NUL, OK, Vv};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// The offset ShaDa does not store, and the one a zeroed slot has.
const NO_OFFSET: SearchOffset = SearchOffset {
    dir: 0,
    line: false,
    end: false,
    off: 0,
};

/// An empty pattern slot.
///
/// `magic` and `dir` are the only fields whose "nothing remembered yet"
/// value is not zero: [`spats`] starts out magic and searching forwards,
/// while the saved copies start out as C's zero-initialised statics.
const fn no_pattern(magic: bool, dir: c_char) -> SearchPattern {
    SearchPattern {
        pat: ptr::null_mut(),
        patlen: 0,
        magic,
        no_scs: false,
        timestamp: 0,
        off: SearchOffset { dir, ..NO_OFFSET },
        additional_data: ptr::null_mut(),
    }
}

/// The two remembered patterns, indexed by `RE_SEARCH` and `RE_SUBST`.
///
/// `pat` and `additional_data` are owned allocations; [`free_spat`] is
/// what releases them. The pair is also the interchange with ShaDa, which
/// hands a slot's ownership over wholesale
/// ([`set_search_pattern`]/[`set_substitute_pattern`]).
pub(crate) static spats: GlobalCell<[SearchPattern; 2]> =
    GlobalCell::new([no_pattern(true, b'/' as c_char); 2]);

/// Which of [`spats`] was used most recently — the one `RE_LAST` means.
pub(crate) static last_idx: GlobalCell<c_int> = GlobalCell::new(RE_SEARCH);

/// The pattern [`search_regcomp`] last compiled, as the user wrote it.
///
/// Kept separately from [`spats`] because `'rightleft'` reverses it and
/// because `SEARCH_KEEP` compiles patterns that are never remembered. The
/// "not found" messages quote this one.
static compiled_pat: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static compiled_patlen: GlobalCell<size_t> = GlobalCell::new(0);

/// Copies of the above, kept while autocommands and user functions run.
static saved_spats: GlobalCell<[SearchPattern; 2]> = GlobalCell::new([no_pattern(false, 0); 2]);
static saved_compiled_pat: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static saved_compiled_patlen: GlobalCell<size_t> = GlobalCell::new(0);
static saved_spats_last_idx: GlobalCell<c_int> = GlobalCell::new(0);
static saved_spats_no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
/// Nesting depth of [`save_search_patterns`]; only the outermost saves.
static save_level: GlobalCell<c_int> = GlobalCell::new(0);

/// A second, independent copy of `spats[RE_SEARCH]`, for incremental
/// search — which has to be able to put the pattern back even when it was
/// cancelled from inside a user function.
static saved_last_search_spat: GlobalCell<SearchPattern> = GlobalCell::new(no_pattern(false, 0));
static did_save_last_search_spat: GlobalCell<c_int> = GlobalCell::new(0);
static saved_last_idx: GlobalCell<c_int> = GlobalCell::new(0);
static saved_no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
static saved_search_match_endcol: GlobalCell<colnr_T> = GlobalCell::new(0);
static saved_search_match_lines: GlobalCell<linenr_T> = GlobalCell::new(0);

/// The remembered pattern at `idx`, copied out.
///
/// The copy shares `pat` and `additional_data` with the slot; it is a
/// borrow of those, not ownership.
fn spat(idx: c_int) -> SearchPattern {
    spats.get()[idx as usize]
}

/// Store `pat` at `idx`. Whatever was there is dropped on the floor —
/// callers that owned it call [`free_spat`] first.
fn put_spat(idx: c_int, pat: SearchPattern) {
    spats.with_mut(|slots| slots[idx as usize] = pat);
}

/// Release a slot's owned string and its ShaDa extras.
///
/// # Safety
/// `pat` and `additional_data` must be owned allocations or null, and must
/// not be used afterwards.
unsafe fn free_spat(spat: &SearchPattern) {
    unsafe { xfree(spat.pat as *mut c_void) };
    unsafe { xfree(spat.additional_data as *mut c_void) };
}

/// A copy of `spats[idx]` owning its own `pat`, for the saved arrays.
///
/// `additional_data` is *not* copied — the original keeps it, exactly as
/// upstream's struct assignment leaves it aliased. Nothing frees the copy.
unsafe fn clone_spat(idx: c_int) -> SearchPattern {
    let mut copy = spat(idx);
    if !copy.pat.is_null() {
        copy.pat = unsafe { xstrnsave(copy.pat, copy.patlen) };
    }
    copy
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
/// `regmatch` is an out parameter: the compiled pattern and the
/// ignore-case flag to match it with. Answers `OK`, or `FAIL` when the
/// pattern does not compile or there is no remembered one to fall back on.
///
/// # Safety
/// `pat` must be null, or readable for `patlen` bytes and NUL-terminated;
/// it is borrowed only for the call. `used_pat` may be null, and is
/// otherwise written with a pointer into `pat` or into the remembered
/// pattern, which lives until the next call. `regmatch` must be writable,
/// and the caller owns the `regprog` this leaves in it.
pub unsafe fn search_regcomp(
    mut pat: *mut c_char,
    mut patlen: size_t,
    used_pat: *mut *mut c_char,
    pat_save: c_int,
    pat_use: c_int,
    options: c_int,
    regmatch: *mut regmmatch_T,
) -> c_int {
    rc_did_emsg.set(false);
    let mut magic = magic_isset();

    if pat.is_null() || unsafe { *pat } as c_int == NUL {
        // No pattern given: use a previously defined one.
        let idx = if pat_use == RE_LAST {
            last_idx.get()
        } else {
            pat_use
        };
        let remembered = spat(idx);
        if remembered.pat.is_null() {
            // Never defined.
            let msg = if pat_use == RE_SUBST {
                e_nopresub.as_ptr()
            } else {
                e_noprevre.as_ptr()
            };
            unsafe { emsg(gettext(msg)) };
            rc_did_emsg.set(true);
            return FAIL;
        }
        pat = remembered.pat;
        patlen = remembered.patlen;
        magic = remembered.magic;
        no_smartcase.set(remembered.no_scs);
    } else if options & SEARCH_HIS as c_int != 0 {
        add_to_history(
            HIST_SEARCH as c_int,
            unsafe { core::slice::from_raw_parts(pat as *const u8, patlen) },
            true,
            NUL as u8,
        );
    }

    if !used_pat.is_null() {
        unsafe { *used_pat = pat };
    }

    unsafe { xfree(compiled_pat.get() as *mut c_void) };
    let rightleft_reverse = cur_win().w_onebuf_opt.wo_rl != 0
        && unsafe { *cur_win().w_onebuf_opt.wo_rlc } as c_int == 's' as c_int;
    compiled_pat.set(if rightleft_reverse {
        unsafe { reverse_text(pat) }
    } else {
        unsafe { xstrnsave(pat, patlen) }
    });
    compiled_patlen.set(patlen);

    // Remember the pattern, unless the caller or `:keeppatterns` asked
    // us not to.
    if options & SEARCH_KEEP as c_int == 0 && !cmdmod_has(CmdModFlags::KEEPPATTERNS) {
        if pat_save == RE_SEARCH || pat_save == RE_BOTH {
            unsafe { save_re_pat(RE_SEARCH, pat, patlen, magic) };
        }
        if pat_save == RE_SUBST || pat_save == RE_BOTH {
            unsafe { save_re_pat(RE_SUBST, pat, patlen, magic) };
        }
    }

    unsafe { (*regmatch).rmm_ic = ignorecase(pat) };
    unsafe { (*regmatch).rmm_maxcol = 0 };
    unsafe { (*regmatch).regprog = vim_regcomp(pat, if magic { RE_MAGIC } else { 0 }) };
    if unsafe { (*regmatch).regprog.is_null() } {
        FAIL
    } else {
        OK
    }
}

/// The pattern [`search_regcomp`] last compiled, valid until the next call
/// to it.
pub fn get_search_pat() -> *mut c_char {
    compiled_pat.get()
}

/// Remember `pat` as the pattern at `idx`, and as the last one used.
///
/// # Safety
/// `pat` must be a readable string of at least `patlen` bytes.
pub unsafe fn save_re_pat(idx: c_int, pat: *mut c_char, patlen: size_t, magic: bool) {
    let old = spat(idx);
    if old.pat == pat {
        return;
    }
    unsafe { free_spat(&old) };
    put_spat(
        idx,
        SearchPattern {
            pat: unsafe { xstrnsave(pat, patlen) },
            patlen,
            magic,
            no_scs: no_smartcase.get(),
            timestamp: os_time(),
            off: old.off,
            additional_data: ptr::null_mut(),
        },
    );
    last_idx.set(idx);
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls.get() != 0 {
        unsafe { redraw_all_later(UPD_SOME_VALID) };
    }
    unsafe { set_no_hlsearch(false) };
}

/// Save the search patterns so they can be restored later. Used around
/// autocommands and user functions; only the outermost call saves.
pub fn save_search_patterns() {
    if save_level.replace(save_level.get() + 1) != 0 {
        return;
    }
    // SAFETY: the clones are fresh allocations; nothing else is borrowing
    // the cells.
    for idx in [RE_SEARCH, RE_SUBST] {
        let clone = unsafe { clone_spat(idx) };
        saved_spats.with_mut(|slots| slots[idx as usize] = clone);
    }
    if compiled_pat.get().is_null() {
        saved_compiled_pat.set(ptr::null_mut());
        saved_compiled_patlen.set(0);
    } else {
        saved_compiled_pat.set(unsafe { xstrnsave(compiled_pat.get(), compiled_patlen.get()) });
        saved_compiled_patlen.set(compiled_patlen.get());
    }
    saved_spats_last_idx.set(last_idx.get());
    saved_spats_no_hlsearch.set(no_hlsearch.get());
}

/// Undo one [`save_search_patterns`]; the outermost call restores.
pub fn restore_search_patterns() {
    save_level.set(save_level.get() - 1);
    if save_level.get() != 0 {
        return;
    }
    // SAFETY: the saved copies hand their allocations back to `spats`.
    for idx in [RE_SEARCH, RE_SUBST] {
        unsafe { free_spat(&spat(idx)) };
        put_spat(idx, saved_spats.get()[idx as usize]);
    }
    set_vv_searchforward();
    unsafe { xfree(compiled_pat.get() as *mut c_void) };
    compiled_pat.set(saved_compiled_pat.get());
    compiled_patlen.set(saved_compiled_patlen.get());
    last_idx.set(saved_spats_last_idx.get());
    unsafe { set_no_hlsearch(saved_spats_no_hlsearch.get()) };
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
    // SAFETY: the clone is a fresh allocation.
    saved_last_search_spat.set(unsafe { clone_spat(RE_SEARCH) });
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
        // SAFETY: a literal message.
        let msg =
            c"restore_last_search_pattern() called more often than save_last_search_pattern()";
        unsafe { iemsg(msg.as_ptr()) };
        return;
    }
    // SAFETY: the saved copy hands its allocation back. `additional_data`
    // is deliberately not freed — the saved copy aliases the live slot's,
    // which the assignment below puts back.
    unsafe { xfree(spat(RE_SEARCH).pat as *mut c_void) };
    let saved = saved_last_search_spat.replace(no_pattern(false, 0));
    put_spat(RE_SEARCH, saved);
    set_vv_searchforward();
    last_idx.set(saved_last_idx.get());
    unsafe { set_no_hlsearch(saved_no_hlsearch.get()) };
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

/// The pattern `/` and `?` last searched for.
pub fn last_search_pattern() -> *mut c_char {
    spat(RE_SEARCH).pat
}

pub fn last_search_pattern_len() -> size_t {
    spat(RE_SEARCH).patlen
}

/// Whichever of the two patterns was used last.
pub fn last_search_pat() -> *mut c_char {
    spat(last_idx.get()).pat
}

/// The `:substitute` pattern, borrowed — what `/` falls back on when no
/// search pattern has been used yet.
pub(crate) fn substitute_pattern() -> SearchPattern {
    spat(RE_SUBST)
}

/// The offset `/` and `?` remember alongside the search pattern: the
/// `e`/`s`/`b` suffix and its `+n`/`-n`.
///
/// It lives in `spats[RE_SEARCH]` rather than beside the command that
/// parsed it because `n` and `N` re-apply it, and because `'cpoptions'`
/// "o" makes a *line* offset one-shot. [`do_search`](super::do_search)
/// parses into it and puts the old one back for `SEARCH_KEEP`.
pub(crate) fn search_offset() -> SearchOffset {
    spat(RE_SEARCH).off
}

pub(crate) fn set_search_offset(off: SearchOffset) {
    let mut pat = spat(RE_SEARCH);
    pat.off = off;
    put_spat(RE_SEARCH, pat);
}

/// Whichever of the two patterns was used last, with its length — what
/// the search-count cache compares itself against.
pub(crate) fn last_used_pattern() -> SearchPattern {
    spat(last_idx.get())
}

/// Replace the last used pattern's string, keeping its flags and offset.
/// Only `searchcount()` does this, and only under
/// [`save_last_search_pattern`], which puts the real one back.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub(crate) unsafe fn replace_last_used_pattern(s: *const c_char) {
    let idx = last_idx.get();
    let mut pat = spat(idx);
    unsafe { xfree(pat.pat as *mut c_void) };
    pat.patlen = unsafe { strlen(s) };
    pat.pat = unsafe { xstrnsave(s, pat.patlen) };
    put_spat(idx, pat);
}

/// Whether case should be ignored for pattern `pat`, per `'ignorecase'`
/// and `'smartcase'`.
///
/// # Safety
/// `pat` must be a NUL-terminated string.
pub unsafe fn ignorecase(pat: *mut c_char) -> c_int {
    unsafe { ignorecase_opt(pat, p_ic.get(), p_scs.get()) }
}

/// As [`ignorecase`] but with the `'ignorecase'`/`'smartcase'` values
/// passed in.
///
/// # Safety
/// `pat` must be a NUL-terminated string.
pub unsafe fn ignorecase_opt(pat: *mut c_char, ic_in: c_int, scs: c_int) -> c_int {
    let mut ic = ic_in;
    // 'infercase' completion does its own case handling.
    // SAFETY: the caller's NUL-terminated pattern.
    if ic != 0
        && !no_smartcase.get()
        && scs != 0
        && !(ctrl_x_mode_not_default() && cur_buf().b_p_inf != 0)
    {
        ic = !unsafe { pat_has_uppercase(pat) } as c_int;
    }
    no_smartcase.set(false);
    ic
}

/// Whether pattern `pat` has an uppercase character in it — the
/// `'smartcase'` test.
///
/// # Safety
/// `pat` must be a NUL-terminated string, and not null.
pub unsafe fn pat_has_uppercase(pat: *mut c_char) -> bool {
    // Which of `\`, `%` and `_` introduce an escape depends on the
    // pattern's own magicness, which only a parse can tell us.
    let mut magic_val: magic_T = MAGIC_ON;
    let is_magic = magic_isset() as c_int;
    let (no_str, no_int) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: `pat` is a live NUL-terminated pattern.
    unsafe { skip_regexp_ex(pat, NUL, is_magic, no_str, no_int, &raw mut magic_val) };

    let pat = unsafe { CStr::from_ptr(pat) }.to_bytes_with_nul();
    let mut i = 0;
    while pat[i] != 0 {
        let at = pat[i..].as_ptr().cast::<c_char>();
        let len = unsafe { utfc_ptr2len(at) };
        if len > 1 {
            if mb_isupper(unsafe { utf_ptr2char(at) }) {
                return true;
            }
            i += len as usize;
        } else if pat[i] == b'\\' && magic_val <= MAGIC_ON {
            // Skip "\_X" and "\%X", else "\X".
            i += match pat[i + 1] {
                0 => 1,
                b'_' | b'%' if pat[i + 2] != 0 => 3,
                _ => 2,
            };
        } else if (pat[i] == b'%' || pat[i] == b'_') && magic_val == MAGIC_ALL {
            // Skip "%X" and "_X".
            i += if pat[i + 1] != 0 { 2 } else { 1 };
        } else if mb_isupper(pat[i] as c_int) {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

/// Reset the search direction to forwards. For `gd` and `gD`.
pub fn reset_search_dir() {
    let mut pat = spat(RE_SEARCH);
    pat.off.dir = b'/' as c_char;
    put_spat(RE_SEARCH, pat);
    set_vv_searchforward();
}

/// Set the last search pattern, for `:let @/ =` and the ShaDa file. The
/// saved copy is set too, so that this works inside an autocommand.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn set_last_search_pat(s: *const c_char, idx: c_int, magic: bool, setlast: bool) {
    unsafe { free_spat(&spat(idx)) };
    // An empty string means that nothing should be matched.
    let patlen = if unsafe { *s } as c_int == NUL {
        0
    } else {
        unsafe { strlen(s) }
    };
    put_spat(
        idx,
        SearchPattern {
            pat: if patlen == 0 {
                ptr::null_mut()
            } else {
                unsafe { xstrnsave(s, patlen) }
            },
            patlen,
            magic,
            no_scs: false,
            timestamp: os_time(),
            off: SearchOffset {
                dir: b'/' as c_char,
                ..NO_OFFSET
            },
            additional_data: ptr::null_mut(),
        },
    );
    set_vv_searchforward();
    if setlast {
        last_idx.set(idx);
    }
    if save_level.get() != 0 {
        unsafe { free_spat(&saved_spats.get()[idx as usize]) };
        // Upstream takes the flags from slot 0 whichever slot is being
        // set, then overwrites only the string. Preserved.
        let mut saved = spat(RE_SEARCH);
        saved.pat = if spat(idx).pat.is_null() {
            ptr::null_mut()
        } else {
            unsafe { xstrnsave(spat(idx).pat, spat(idx).patlen) }
        };
        saved.patlen = if spat(idx).pat.is_null() {
            0
        } else {
            spat(idx).patlen
        };
        saved_spats.with_mut(|slots| slots[idx as usize] = saved);
        saved_spats_last_idx.set(last_idx.get());
    }
    // With 'hlsearch' a changed pattern means a redraw.
    if p_hls.get() != 0 && idx == last_idx.get() && !no_hlsearch.get() {
        unsafe { redraw_all_later(UPD_SOME_VALID) };
    }
}

/// Compile the last used search pattern, for highlighting every match in a
/// window. Answers `regmatch->regprog == NULL` when there is no pattern.
///
/// # Safety
/// `regmatch` must be writable.
pub unsafe fn last_pat_prog(regmatch: *mut regmmatch_T) {
    if spat(last_idx.get()).pat.is_null() {
        unsafe { (*regmatch).regprog = ptr::null_mut() };
        return;
    }
    // So it doesn't beep if the pattern is bad.
    let _no_emsg = Suppress::emsg();
    let empty = c"".as_ptr() as *mut c_char;
    let (used, keep) = (last_idx.get(), SEARCH_KEEP as c_int);
    // SAFETY: an empty pattern re-compiled into the caller's `regmatch`.
    unsafe { search_regcomp(empty, 0, ptr::null_mut(), 0, used, keep, regmatch) };
}

/// Set the direction `n` repeats in, for `:let v:searchforward =`.
pub fn set_search_direction(cdir: c_int) {
    let mut pat = spat(RE_SEARCH);
    pat.off.dir = cdir as c_char;
    put_spat(RE_SEARCH, pat);
}

/// Publish the search direction as `v:searchforward`.
pub(crate) fn set_vv_searchforward() {
    // SAFETY: setting a `v:` variable to a number.
    let forward = (spat(RE_SEARCH).off.dir as c_int == '/' as c_int) as varnumber_T;
    unsafe { set_vim_var_nr(Vv::Searchforward, forward) };
}

/// Whether `pattern` matches zero-width.
///
/// With `move_to_match` the search starts at the top of the buffer, else
/// at `cur`. `direction` is `FORWARD` or `BACKWARD`.
///
/// # Safety
/// `pattern` must be null or a readable string of `patternlen` bytes, and
/// `cur` must be readable unless `move_to_match`.
///
/// @return  1, 0, or -1 for failure.
pub(crate) unsafe fn is_zero_width(
    mut pattern: *mut c_char,
    mut patternlen: size_t,
    move_to_match: bool,
    cur: *mut pos_T,
    direction: Direction,
) -> c_int {
    let mut regmatch = regmmatch_T::default();
    let called_emsg_before = called_emsg.get();
    if pattern.is_null() {
        pattern = spat(last_idx.get()).pat;
        patternlen = spat(last_idx.get()).patlen;
    }
    let (rm, keep) = (&raw mut regmatch, SEARCH_KEEP as c_int);
    // SAFETY: `pattern` is a live pattern of `patternlen` bytes, and
    // `regmatch` is this frame's.
    let compiled = unsafe {
        search_regcomp(
            pattern,
            patternlen,
            ptr::null_mut(),
            RE_SEARCH,
            RE_SEARCH,
            keep,
            rm,
        )
    };
    if compiled == FAIL {
        return -1;
    }

    // Init startcol correctly.
    regmatch.startpos[0].col = -1;
    // Searching from the top starts at the zeroed position; searching
    // from the cursor accepts a match at the cursor itself.
    let mut pos = if move_to_match {
        pos_T::default()
    } else {
        unsafe { *cur }
    };
    let flag = if move_to_match {
        0
    } else {
        SEARCH_START as c_int
    };

    let mut result = -1;
    let (w, b) = (curwin.get(), curbuf.get());
    let (at, end) = (&raw mut pos, ptr::null_mut());
    let (opts, none) = (SEARCH_KEEP as c_int + flag, ptr::null_mut());
    // SAFETY: `pos` is this frame's, `pattern` a live pattern of
    // `patternlen` bytes, and the window and buffer are the current ones.
    let found = unsafe {
        searchit(
            w, b, at, end, direction, pattern, patternlen, 1, opts, RE_SEARCH, none,
        )
    };
    if found != FAIL {
        // A zero-width pattern matches somewhere; find where, then ask
        // whether its start and end are the same position.
        let nmatched = loop {
            regmatch.startpos[0].col += 1;
            let (rm, col) = (&raw mut regmatch, regmatch.startpos[0].col);
            let (no_tm, no_to) = (ptr::null_mut(), ptr::null_mut());
            // SAFETY: `regmatch` is this frame's compiled program, matched
            // against a line of the current buffer.
            let nmatched = unsafe { vim_regexec_multi(rm, w, b, pos.lnum, col, no_tm, no_to) };
            let short_of_the_match = if direction == FORWARD {
                regmatch.startpos[0].col < pos.col
            } else {
                regmatch.startpos[0].col > pos.col
            };
            if nmatched != 0 || regmatch.regprog.is_null() || !short_of_the_match {
                break nmatched;
            }
        };
        if called_emsg.get() == called_emsg_before {
            result = (nmatched != 0
                && regmatch.startpos[0].lnum == regmatch.endpos[0].lnum
                && regmatch.startpos[0].col == regmatch.endpos[0].col)
                as c_int;
        }
    }

    unsafe { vim_regfree(regmatch.regprog) };
    result
}

/// Get the last search pattern, for ShaDa.
///
/// # Safety
/// `pat` must be writable. The answer borrows the live slot's string.
pub unsafe fn get_search_pattern(pat: *mut SearchPattern) {
    unsafe { *pat = spat(RE_SEARCH) }
}

/// Get the last substitute pattern, for ShaDa. Its offset is not part of
/// what ShaDa stores.
///
/// # Safety
/// `pat` must be writable. The answer borrows the live slot's string.
pub unsafe fn get_substitute_pattern(pat: *mut SearchPattern) {
    unsafe { *pat = spat(RE_SUBST) };
    unsafe { (*pat).off = NO_OFFSET };
}

/// Set the last search pattern, taking `pat`'s allocations over.
///
/// # Safety
/// `pat.pat` and `pat.additional_data` must be owned allocations or null.
pub unsafe fn set_search_pattern(pat: SearchPattern) {
    unsafe { free_spat(&spat(RE_SEARCH)) };
    put_spat(RE_SEARCH, pat);
    set_vv_searchforward();
}

/// Set the last substitute pattern, taking `pat`'s allocations over. Its
/// offset is not part of what ShaDa stores.
///
/// # Safety
/// `pat.pat` and `pat.additional_data` must be owned allocations or null.
pub unsafe fn set_substitute_pattern(mut pat: SearchPattern) {
    unsafe { free_spat(&spat(RE_SUBST)) };
    pat.off = NO_OFFSET;
    put_spat(RE_SUBST, pat);
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

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
