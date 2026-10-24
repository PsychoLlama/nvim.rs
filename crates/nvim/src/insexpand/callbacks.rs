//! `'completefunc'`, `'omnifunc'`, `'thesaurusfunc'` and the `'complete'` `F` flag.
//!
//! The `did_set_*` halves are the option callbacks that compile a funcname
//! into a `Callback`; [`expand_by_function`] is the call itself, which runs
//! the function twice (`findstart` then the matches) exactly as upstream
//! does.  The `cpt_sources_*` half tracks the per-`'complete'`-entry state
//! those functions need.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::option::vars::P_TSRFU;
use crate::optionstr::{OptString, local_or_global};

use crate::guard::Lock;
use crate::option::next_option_part;
use crate::optionstr::OptStringRef;
use crate::semsg;
use crate::types::{DictRef, Failed, ListRef, OptError, OptionSetFlags, VAR_DICT, VAR_LIST};
use crate::winlayer::{Buf, Win};

/// A second reference to a completion callback, released when it drops.
///
/// Every call of a completion function goes through one: the function may
/// reset the option whose callback it is (`:set completefunc=` from inside
/// the completefunc), which frees the original while the call is still
/// running. The copy keeps the function alive for the length of the call,
/// as `'operatorfunc'` does.
pub(crate) struct HeldCallback(Callback);

impl HeldCallback {
    /// A second owner of what `cb` holds.
    pub(crate) fn of(cb: &Callback) -> HeldCallback {
        HeldCallback(cb.duplicate())
    }
}

impl ::core::ops::Deref for HeldCallback {
    type Target = Callback;

    fn deref(&self) -> &Callback {
        &self.0
    }
}

impl Drop for HeldCallback {
    fn drop(&mut self) {
        self.0.clear();
    }
}

/// One of the three global completion-function callbacks.
///
/// `'completefunc'`, `'omnifunc'` and `'thesaurusfunc'` each compile to a
/// `Callback` the editor *calls*. The cell is only ever borrowed for the
/// length of one helper; a call takes a [`HeldCallback`] of it, so nothing
/// borrows the cell while user code runs.
#[derive(Clone, Copy)]
pub(crate) struct CompleteFuncCb(&'static GlobalCell<Callback>);

/// The global `'completefunc'` callback.
pub(crate) fn cfu_cb() -> CompleteFuncCb {
    CompleteFuncCb(&CFU_CB)
}

/// The global `'omnifunc'` callback.
pub(crate) fn ofu_cb() -> CompleteFuncCb {
    CompleteFuncCb(&OFU_CB)
}

/// The global `'thesaurusfunc'` callback.
pub(crate) fn tsrfu_cb() -> CompleteFuncCb {
    CompleteFuncCb(&TSRFU_CB)
}

impl CompleteFuncCb {
    /// A second reference to the callback, for a call.
    pub(crate) fn held(self) -> HeldCallback {
        self.0.with(HeldCallback::of)
    }

    /// Compile `value` into this callback, freeing what was there. C's
    /// `option_set_callback_func`, answering whether the value was accepted.
    pub(crate) fn set_from_option(self, value: &CStr) -> Result<(), Failed> {
        let cb = callback_from_option(value)?;
        self.0.with_mut(|slot| install_callback(slot, cb));
        Ok(())
    }

    /// Copy this callback into the buffer-local slot `bufcb`, freeing what
    /// was there.
    pub(crate) fn copy_to_buflocal(self, bufcb: &mut Callback) {
        let copy = self.0.with(Callback::duplicate);
        bufcb.clear();
        *bufcb = copy;
    }

    /// Mark what this callback references with `copy_id`, so the garbage
    /// collector leaves it alone. Answers whether to abort.
    pub(crate) fn set_ref(self, copy_id: c_int) -> bool {
        self.0
            .with(|cb| set_ref_in_callback(cb, copy_id, None, None))
    }
}

/// The global `'complete'` `F{func}` callbacks: one slot per entry of the
/// option, empty for every entry that is not an `F{func}`.
///
/// This is the cached copy `:set complete=` leaves behind so a buffer that
/// has never had `:setlocal complete=` can be given the same array; the
/// live one a completion reads is always `curbuf`'s `b_p_cpt_cb`. Upstream
/// keeps both as a `Callback *` with a count beside it; here both are a
/// `Vec`, and a `Callback` owns what it names without a destructor, so
/// replacing either frees the old slots by hand.
#[derive(Clone, Copy)]
pub(crate) struct CptCallbacks(());

/// The cached global `'complete'` `F{func}` callbacks. See [`CptCallbacks`].
pub(crate) fn cpt_cb() -> CptCallbacks {
    CptCallbacks(())
}

impl CptCallbacks {
    /// The number of slots; zero when `'complete'` has never been set
    /// globally.
    pub(crate) fn count(self) -> c_int {
        c_int::try_from(CPT_CB.with(Vec::len)).unwrap_or(c_int::MAX)
    }

    /// Replace the cached array with a copy of `src`. An empty `src`
    /// leaves the cache alone, as upstream's did.
    pub(crate) fn replace_from(self, src: &[Callback]) {
        if src.is_empty() {
            return;
        }
        let fresh: Vec<Callback> = src.iter().map(Callback::duplicate).collect();
        for mut old in CPT_CB.replace(fresh) {
            old.clear();
        }
    }

    /// Copy the cache into `buffer`'s array, freeing what was there. An
    /// empty cache leaves the buffer's array alone.
    pub(crate) fn copy_to_buffer(self, mut buffer: Buf) {
        let fresh: Vec<Callback> =
            CPT_CB.with(|slots| slots.iter().map(Callback::duplicate).collect());
        if fresh.is_empty() {
            return;
        }
        clear_buffer_cpt_callbacks(buffer);
        buffer.b_p_cpt_cb = fresh;
    }

    /// Mark what the cache references with `copy_id`. Answers whether to
    /// abort.
    pub(crate) fn set_ref(self, copy_id: c_int) -> bool {
        CPT_CB.with_mut(|slots| {
            // No containing list or dict to mark.
            let mark = |slot: &mut Callback| set_ref_in_callback(slot, copy_id, None, None);
            slots.iter_mut().any(mark)
        })
    }
}

/// The per-`'complete'`-entry state: one row for every comma-separated
/// segment of the option, plus the index of the segment being collected.
///
/// Upstream keeps this as a `CptSource *` with a hand-rolled
/// `cpt_sources_count` beside it and `cpt_sources_index` as a third global,
/// `xcalloc`'d by `setup_cpt_sources` and `xfree`'d by `cpt_sources_clear`.
/// Here the rows are a `Vec` in [`ComplState`], empty for upstream's NULL.
///
/// Rows are addressed by index rather than by a pointer held across a call,
/// which matters: `prepare_cpt_compl_funcs` and `cpt_compl_refresh` write a
/// row *after* running a user's completion function, and that function can
/// `:set complete=...` and rebuild the array underneath. Upstream writes
/// through the stale pointer; an out-of-range index here is ignored, and an
/// out-of-range read answers [`CPT_SOURCE_INIT`], the zeroed row `xcalloc`
/// used to hand back.
#[derive(Clone, Copy)]
pub(crate) struct CptSources(());

/// The per-`'complete'`-entry state. See [`CptSources`].
pub(crate) fn cpt_sources() -> CptSources {
    CptSources(())
}

impl CptSources {
    /// Whether `'complete'` has not been parsed into rows at all — upstream's
    /// `cpt_sources_array == NULL`, which is how the completion asks whether
    /// it is running a `'complete'`-driven scan.
    pub(crate) fn is_unset(self) -> bool {
        CPT_SOURCES.with(Vec::is_empty)
    }

    /// The number of rows.
    pub(crate) fn len(self) -> usize {
        CPT_SOURCES.with(Vec::len)
    }

    /// The generation of the rows: bumped by every rebuild and clear.
    pub(crate) fn generation(self) -> u64 {
        CPT_GENERATION.get()
    }

    /// Row `idx` of generation `generation` as an index into the rows that
    /// are live now: `None` when it is −1, out of range, or names a row of
    /// an earlier generation (a user function can `:set complete=` and the
    /// completion rebuild the rows while older matches are in the list).
    pub(crate) fn live_index(self, idx: c_int, generation: u64) -> Option<usize> {
        let idx = usize::try_from(idx).ok()?;
        (generation == self.generation() && idx < self.len()).then_some(idx)
    }

    /// Row `idx` by value, or the zeroed row when `idx` is out of range.
    pub(crate) fn row(self, idx: c_int) -> CptSource {
        let idx = usize::try_from(idx).ok();
        CPT_SOURCES
            .with(|rows| idx.and_then(|idx| rows.get(idx).copied()))
            .unwrap_or(CPT_SOURCE_INIT)
    }

    /// The row the scan is collecting from, by value.
    pub(crate) fn current(self) -> CptSource {
        self.row(self.index())
    }

    /// Whether any source has `refresh` set to `always`.
    pub(crate) fn any_refresh_always(self) -> bool {
        CPT_SOURCES.with(|rows| rows.iter().any(|row| row.cs_refresh_always))
    }

    /// Change row `idx` in place; out of range does nothing.
    pub(crate) fn update(self, idx: c_int, f: impl FnOnce(&mut CptSource)) {
        let Ok(idx) = usize::try_from(idx) else {
            return;
        };
        CPT_SOURCES.with_mut(|rows| rows.get_mut(idx).map(f));
    }

    /// Take `rows` as the new state, dropping whatever was there. The index
    /// is left alone: the caller sets it when the scan starts.
    pub(crate) fn set_rows(self, rows: Vec<CptSource>) {
        CPT_SOURCES.set(rows);
        CPT_GENERATION.update(|generation| *generation += 1);
    }

    /// Drop the rows and forget where the scan was — C's
    /// `cpt_sources_clear()`.
    pub(crate) fn clear(self) {
        CPT_SOURCES.set(Vec::new());
        CPT_SOURCES_INDEX.set(-1);
        CPT_GENERATION.update(|generation| *generation += 1);
    }

    /// The `'complete'` entry the scan is collecting from, or −1 between
    /// scans.
    pub(crate) fn index(self) -> c_int {
        CPT_SOURCES_INDEX.get()
    }

    /// Point the scan at entry `idx`.
    pub(crate) fn set_index(self, idx: c_int) {
        CPT_SOURCES_INDEX.set(idx);
    }
}

/// One entry of `'complete'`, as the walks over the option see it.
pub(crate) struct CptEntry<'a> {
    /// The option from this entry on.
    pub(crate) at: &'a [u8],
    /// The entry's text, as `copy_option_part` copies it into an `LSIZE`
    /// buffer: a backslash before a comma dropped, cut at `LSIZE - 1` bytes.
    pub(crate) part: Vec<u8>,
    /// The option after this entry.
    pub(crate) after: &'a [u8],
}

/// The entries of `'complete'` value `option`, in order: every non-empty
/// comma-separated segment, the `,` and ` ` between them skipped.
pub(crate) fn cpt_entries(mut option: &[u8]) -> impl Iterator<Item = CptEntry<'_>> {
    ::core::iter::from_fn(move || {
        let start = option.iter().position(|&b| b != b',' && b != b' ')?;
        let at = &option[start..];
        let mut part = Vec::new();
        let after = next_option_part(at, &mut part);
        part.truncate(LSIZE as usize - 1);
        option = after;
        Some(CptEntry { at, part, after })
    })
}

/// C's `atoi` over the start of `text`: blanks, a sign, then digits.
pub(crate) fn leading_number(text: &[u8]) -> c_int {
    let start = text
        .iter()
        .position(|b| !matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .unwrap_or(text.len());
    let text = &text[start..];
    let (negative, digits) = match text.first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let value = digits
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .fold(0 as c_int, |n, &d| {
            n.wrapping_mul(10).wrapping_add(c_int::from(d - b'0'))
        });
    if negative {
        value.wrapping_neg()
    } else {
        value
    }
}

/// The number of entries in `'complete'` — every non-empty comma-separated
/// segment counts as one.
pub(crate) fn get_cpt_sources_count() -> c_int {
    c_int::try_from(cpt_entries(Buf::current().b_p_cpt.bytes()).count()).unwrap_or(c_int::MAX)
}

/// Parse the `'completefunc'` value and set the callback function; the value
/// may be a function name, `function(<name>)`, `funcref(<name>)` or a lambda.
///
/// This is an `opt_did_set_cb` row in the generated option table.
pub fn did_set_completefunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let value = args
        .os_newval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .as_cstr();
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        callback_from_option(value).map(|cb| install_callback(&mut buf.b_cfu_cb, cb))
    } else {
        let retval = cfu_cb().set_from_option(value);
        if retval.is_ok() && !args.os_flags.has(OptionSetFlags::GLOBAL) {
            set_buflocal_cfu_callback(buf);
        }
        retval
    };
    if retval.is_err() {
        Err(e_invarg.into())
    } else {
        Ok(())
    }
}

/// Copy the global `'completefunc'` callback into `buffer`'s local one.
///
/// Safe: [`Buf`] is the live buffer whose own callback field this writes.
pub fn set_buflocal_cfu_callback(mut buffer: Buf) {
    cfu_cb().copy_to_buflocal(&mut buffer.b_cfu_cb);
}

/// Parse the `'omnifunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub fn did_set_omnifunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let value = args
        .os_newval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .as_cstr();
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        callback_from_option(value).map(|cb| install_callback(&mut buf.b_ofu_cb, cb))
    } else {
        let retval = ofu_cb().set_from_option(value);
        if retval.is_ok() && !args.os_flags.has(OptionSetFlags::GLOBAL) {
            set_buflocal_ofu_callback(buf);
        }
        retval
    };
    if retval.is_err() {
        Err(e_invarg.into())
    } else {
        Ok(())
    }
}

/// Copy the global `'omnifunc'` callback into `buffer`'s local one.
///
/// Safe: [`Buf`] is the live buffer whose own callback field this writes.
pub fn set_buflocal_ofu_callback(mut buffer: Buf) {
    ofu_cb().copy_to_buflocal(&mut buffer.b_ofu_cb);
}

/// Free `buffer`'s `'complete'` `F{func}` callbacks.
pub(crate) fn clear_buffer_cpt_callbacks(mut buffer: Buf) {
    for mut cb in ::core::mem::take(&mut buffer.b_p_cpt_cb) {
        cb.clear();
    }
}

/// Copy the global `'complete'` `F{func}` callbacks into `buffer`'s local array,
/// clearing any existing buffer-local callbacks first.
///
/// Safe: [`Buf`] is the live buffer whose own callback array this rebuilds --
/// which is also what retires upstream's NULL check.
pub fn set_buflocal_cpt_callbacks(buffer: Buf) {
    if cpt_cb().count() == 0 {
        return;
    }
    cpt_cb().copy_to_buffer(buffer);
}

/// Parse `'complete'` and (re)build the `F{func}` callbacks; entries other
/// than `F{func}` are counted but leave their slot empty.
///
/// `local` says the set was a `:setlocal`, which leaves the cached global
/// array alone.
pub fn set_cpt_callbacks(local: bool) -> Result<(), Failed> {
    if Buf::current_or_none().is_none() {
        return Err(Failed);
    }

    clear_buffer_cpt_callbacks(Buf::current());

    let count = get_cpt_sources_count();
    if count == 0 {
        return Ok(());
    }
    // An entry nothing fills reads back as `Callback::None`.
    Buf::current().b_p_cpt_cb = (0..count).map(|_| Callback::None).collect();

    // A copy: the walk writes the buffer's callback array.
    let option = Buf::current().b_p_cpt.bytes().to_vec();
    for (idx, entry) in cpt_entries(&option).enumerate() {
        let part = entry.part;
        if part.first() == Some(&b'F') && part.len() > 1 {
            // Drop the `^N` max-matches suffix.
            let end = part.iter().position(|&b| b == b'^').unwrap_or(part.len());
            let name = XString::from_bytes(&part[1..end]);
            let cb = callback_from_option(name.as_cstr());
            // `idx` counts the entries, which is the array's length.
            let mut buf = Buf::current();
            if let Some(slot) = buf.b_p_cpt_cb.get_mut(idx) {
                match cb {
                    Ok(cb) => install_callback(slot, cb),
                    Err(_) => *slot = Callback::None,
                }
            }
        }
    }

    if !local {
        // ':set' was used instead of ':setlocal': cache the callback array.
        let buf = Buf::current();
        cpt_cb().replace_from(&buf.b_p_cpt_cb);
    }
    Ok(())
}

/// Parse the `'thesaurusfunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub fn did_set_thesaurusfunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        // Buffer-local option set.
        // A copy: evaluating the value may set the option again.
        let value = buf.b_p_tsrfu.get();
        callback_from_option(value.as_cstr()).map(|cb| install_callback(&mut buf.b_tsrfu_cb, cb))
    } else {
        // Global option set. A copy, as above.
        let retval = tsrfu_cb().set_from_option(P_TSRFU.get().as_cstr());
        // When using :set, free the local callback.
        if !args.os_flags.has(OptionSetFlags::GLOBAL) {
            callback_free(&mut buf.b_tsrfu_cb);
        }
        retval
    };
    if retval.is_err() {
        Err(e_invarg.into())
    } else {
        Ok(())
    }
}

/// Mark `copy_id` references in `buffer`'s `F{func}` `'complete'` callbacks
/// so they are not garbage collected.
pub fn mark_cpt_callbacks(buffer: Buf, copy_id: c_int) -> bool {
    let mark = |slot: &Callback| set_ref_in_callback(slot, copy_id, None, None);
    buffer.b_p_cpt_cb.iter().any(mark)
}

/// Mark the global `'completefunc'`, `'omnifunc'` and `'thesaurusfunc'`
/// callbacks with `copy_id` so they are not garbage collected.
pub fn set_ref_in_insexpand_funcs(copy_id: c_int) -> bool {
    let mut abort = cfu_cb().set_ref(copy_id);
    abort = abort || ofu_cb().set_ref(copy_id);
    abort = abort || tsrfu_cb().set_ref(copy_id);
    abort = abort || cpt_cb().set_ref(copy_id);
    abort
}

/// The user-defined completion function name for completion `type_0`.
pub(crate) fn get_complete_funcname(type_0: c_int) -> XString {
    let buf = Buf::current();
    let local = match type_0 {
        CTRL_X_FUNCTION => &buf.b_p_cfu,
        CTRL_X_OMNI => &buf.b_p_ofu,
        // The only one of the three with a global value to fall back to.
        CTRL_X_THESAURUS => return local_or_global(&buf.b_p_tsrfu, P_TSRFU).get(),
        _ => return XString::new(),
    };
    local.clone().unwrap_or_default()
}

/// The callback to use for insert-mode completion of `type_0`, held for a
/// call.
pub(crate) fn get_insert_callback(type_0: c_int) -> HeldCallback {
    let buf = Buf::current();
    if type_0 == CTRL_X_FUNCTION {
        return HeldCallback::of(&buf.b_cfu_cb);
    }
    if type_0 == CTRL_X_OMNI {
        return HeldCallback::of(&buf.b_ofu_cb);
    }
    // CTRL_X_THESAURUS
    if !buf.b_p_tsrfu.bytes().is_empty() {
        HeldCallback::of(&buf.b_tsrfu_cb)
    } else {
        tsrfu_cb().held()
    }
}

/// Call `'completefunc'`, `'omnifunc'` or `'thesaurusfunc'` and add whatever
/// it answers to the match list.
///
/// `type_0` is one of `CTRL_X_OMNI`, `CTRL_X_FUNCTION` or `CTRL_X_THESAURUS`;
/// `cb` is set when a function in `'complete'` triggered this (the caller
/// holds it for the call), `None` otherwise.
pub(crate) fn expand_by_function(type_0: c_int, base: ComplStr, cb: Option<&Callback>) {
    debug_assert!(Buf::current_or_none().is_some());

    let held;
    let cb = match cb {
        Some(cb) => cb,
        None => {
            if get_complete_funcname(type_0).is_empty() {
                return;
            }
            held = get_insert_callback(type_0);
            &*held
        }
    };

    // Call the function to obtain the list of matches.
    // The argument owns a copy of the base, released with it.
    let args = [TypVal::Number(0), base.with_bytes(TypVal::string_from)];

    let mut matchlist: Option<ListRef> = None;
    let mut matchdict: Option<DictRef> = None;
    let mut rettv = TYPVAL_T_INIT;
    let save_state = State.get();
    let pos = Win::current().w_cursor;

    // Lock the text to avoid weird things from happening.  Also disallow
    // switching to another window: it should not be needed and may end up
    // in Insert mode in another buffer.
    let locked = Lock::text();
    if callback_call(cb, &args, &mut rettv) {
        // The two container arms take the reference out of `rettv`; it is
        // given back when the handle drops below.
        match rettv.v_type() {
            VAR_LIST => matchlist = rettv.take_list(),
            VAR_DICT => matchdict = rettv.take_dict(),
            // VAR_SPECIAL falls through to the default.
            // TODO(brammool): Give error message?
            _ => tv_clear(&mut rettv),
        }
    }
    drop(locked);

    Win::current().w_cursor = pos; // restore the cursor position
    check_cursor(Win::current()); // make sure the position is valid, just in case
    validate_cursor(Win::current());
    if !equalpos(Win::current().w_cursor, pos) {
        emsg(gettext(E_COMPLDEL));
    } else if let Some(list) = &matchlist {
        ins_compl_add_list(list);
    } else if let Some(dict) = &matchdict {
        ins_compl_add_dict(dict);
    }

    // Restore State, it might have been changed.
    State.set(save_state);
    drop(matchdict);
    drop(matchlist);
}

/// The attribute of the named highlight group, or `-1` for no name.
#[inline]
pub(crate) fn get_user_highlight_attr(hlname: Option<&CStr>) -> c_int {
    match hlname {
        Some(name) if !name.is_empty() => syn_name2attr(name),
        _ => -1,
    }
}

/// The callback the `'complete'` entry at the start of `entry` names, held
/// for a call, if it refers to a user-defined function; `idx` indexes the
/// callback array.
pub(crate) fn get_callback_if_cpt_func(entry: &[u8], idx: c_int) -> Option<HeldCallback> {
    let buffer = Buf::current();
    match entry.first() {
        Some(b'o') => Some(HeldCallback::of(&buffer.b_ofu_cb)),
        Some(b'F') if !matches!(entry.get(1), None | Some(b',')) => {
            // 'F{func}' case. An index past the array (the option changed
            // since `set_cpt_callbacks` sized it) names no function.
            let slot = usize::try_from(idx)
                .ok()
                .and_then(|i| buffer.b_p_cpt_cb.get(i));
            slot.filter(|cb| cb.is_set()).map(HeldCallback::of)
        }
        Some(b'F') => Some(HeldCallback::of(&buffer.b_cfu_cb)), // 'cfu'
        _ => None,
    }
}

/// Call the functions named in `'complete'` with `findstart=1` and record the
/// start column each answers.
pub(crate) fn prepare_cpt_compl_funcs() {
    // Make a copy of 'cpt' in case the buffer gets wiped out.
    let cpt = strip_caret_numbers(Buf::current().b_p_cpt.bytes());
    for (idx, entry) in cpt_entries(&cpt).enumerate() {
        let idx = c_int::try_from(idx).unwrap_or(c_int::MAX);
        let Some(cb) = get_callback_if_cpt_func(entry.at, idx) else {
            cpt_sources().update(idx, |source| source.cs_startcol = -3);
            continue;
        };
        let col = Win::current().w_cursor.col;
        let (ret, mut startcol) = get_userdefined_compl_info(col, Some(&cb));
        if ret.is_err() {
            if startcol == -3 {
                cpt_sources().update(idx, |source| source.cs_refresh_always = false);
            } else {
                startcol = -2;
            }
        } else if startcol < 0 || startcol > Win::current().w_cursor.col {
            startcol = Win::current().w_cursor.col;
        }
        cpt_sources().update(idx, |source| source.cs_startcol = startcol);
    }
}

/// Advance `cpt_sources_index` by one, or report E684 and fail.
pub(crate) fn advance_cpt_sources_index_safe() -> Result<(), Failed> {
    let idx = cpt_sources().index();
    if idx >= 0 && idx < c_int::try_from(cpt_sources().len()).unwrap_or(c_int::MAX) - 1 {
        cpt_sources().set_index(idx + 1);
        return Ok(());
    }
    semsg!("E684: List index out of range: {}", idx);
    Err(Failed)
}

/// Build the per-`'complete'`-entry state: the source letter and its `^N`
/// max-matches limit.
pub(crate) fn setup_cpt_sources() {
    let option = Buf::current().b_p_cpt.bytes().to_vec();
    let rows = cpt_entries(&option)
        .map(|entry| {
            let mut source = CptSource {
                cs_flag: entry.at[0].cast_signed(),
                ..CPT_SOURCE_INIT
            };
            if let Some(caret) = entry.part.iter().position(|&b| b == b'^') {
                source.cs_max_matches = leading_number(&entry.part[caret + 1..]);
            }
            source
        })
        .collect();
    // No rows leaves the state unset, as clearing it does.
    cpt_sources().clear();
    cpt_sources().set_rows(rows);
}

/// Whether any completion source has `refresh` set to `always`.
pub(crate) fn is_cpt_func_refresh_always() -> bool {
    cpt_sources().any_refresh_always()
}

/// Collect matches through `cb` (held by the caller for the call) and record
/// its `refresh:always` flag.
pub(crate) fn get_cpt_func_completion_matches(cb: &Callback) {
    let idx = cpt_sources().index();
    let startcol = cpt_sources().row(idx).cs_startcol;
    if startcol == -2 || startcol == -3 {
        return;
    }

    set_compl_globals(startcol, Win::current().w_cursor.col, true);

    // Insert the leader string (previously removed) before expansion.
    // This prevents flicker when `func` (e.g. an LSP client) is slow and
    // calls 'sleep', which triggers ui_flush().
    if !cpt_sources().row(idx).cs_refresh_always {
        ins_compl_insert_text(&ins_compl_leader_str().to_vec());
    }

    expand_by_function(0, cpt_compl_pattern(), Some(cb));

    if !cpt_sources().row(idx).cs_refresh_always {
        ins_compl_delete(false);
    }

    let refresh_always = compl_opt_refresh_always.get();
    cpt_sources().update(idx, |source| source.cs_refresh_always = refresh_always);
    compl_opt_refresh_always.set(false);
}

/// Re-collect matches from the `'complete'` functions that set
/// `refresh:always`.
pub(crate) fn cpt_compl_refresh() {
    // Make the completion list linear (non-cyclic).
    ins_compl_make_linear();
    // Make a copy of 'cpt' in case the buffer gets wiped out.
    let cpt = strip_caret_numbers(Buf::current().b_p_cpt.bytes());

    cpt_sources().set_index(0);
    for entry in cpt_entries(&cpt) {
        let idx = cpt_sources().index();
        let cb = if cpt_sources().row(idx).cs_refresh_always {
            get_callback_if_cpt_func(entry.at, idx)
        } else {
            None
        };
        if let Some(cb) = cb {
            remove_old_matches();
            let col = Win::current().w_cursor.col;
            let (ret, mut startcol) = get_userdefined_compl_info(col, Some(&cb));
            if ret.is_err() {
                if startcol == -3 {
                    cpt_sources().update(idx, |source| source.cs_refresh_always = false);
                } else {
                    startcol = -2;
                }
            } else if startcol < 0 || startcol > Win::current().w_cursor.col {
                startcol = Win::current().w_cursor.col;
            }
            cpt_sources().update(idx, |source| source.cs_startcol = startcol);
            if ret.is_ok() {
                compl_source_start_timer(idx);
                get_cpt_func_completion_matches(&cb);
            }
        }
        if may_advance_cpt_index(entry.after) {
            let _ = advance_cpt_sources_index_safe();
        }
    }
    cpt_sources().set_index(-1);

    // Make the list cyclic.
    compl_matches.set(ins_compl_make_cyclic());
}
