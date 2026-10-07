//! `'completefunc'`, `'omnifunc'`, `'thesaurusfunc'` and the `'complete'` `F` flag.
//!
//! The `did_set_*` halves are the option callbacks that compile a funcname
//! into a `Callback`; [`expand_by_function`] is the call itself, which runs
//! the function twice (`findstart` then the matches) exactly as upstream
//! does.  The `cpt_sources_*` half tracks the per-`'complete'`-entry state
//! those functions need.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::option::vars::P_TSRFU;
use crate::optionstr::{OptString, local_or_global};

use crate::guard::Lock;
use crate::option::next_option_part;
use crate::optionstr::OptStringRef;
use crate::semsg;
use crate::types::{DictRef, Failed, ListRef, NUL, OptError, OptionSetFlags, VAR_DICT, VAR_LIST};
use crate::winlayer::{Buf, Win};

/// One of the three global completion-function callbacks.
///
/// `'completefunc'`, `'omnifunc'` and `'thesaurusfunc'` each compile to a
/// `Callback` the editor *calls*, so `get`/`set` cannot own one: the value
/// holds a funcref or a Lua reference, and a copy would be a second owner of
/// it. Every helper that touches a callback -- `option_set_callback_func`,
/// `callback_copy`, `callback_free`, `set_ref_in_callback` -- is C-shaped
/// and takes the *slot's* address so it can free what is there and write the
/// new value in place; the buffer-local twin of each of these lives in a
/// `Buffer` field, and the same helpers serve both.
///
/// `CompleteFuncCb` names the cell rather than pointing into it, so it is
/// `Copy` and needs no `unsafe` to make. The one thing it cannot avoid is
/// the address, and that is produced at exactly one place --
/// [`slot`](Self::slot) -- rather than at nine call sites.
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
    /// The slot's address, which is what the C-shaped callback helpers take.
    /// It stands where a buffer-local callback would answer
    /// `&raw mut buf.b_cfu_cb`.
    pub(crate) fn slot(self) -> *mut Callback {
        self.0.ptr()
    }

    /// Compile `value` into this callback, freeing what was there. C's
    /// `option_set_callback_func`, answering whether the value was accepted.
    ///
    /// # Safety
    /// `value` must be a live NUL-terminated option string, or null.
    pub(crate) unsafe fn set_from_option(self, value: *mut c_char) -> Result<(), Failed> {
        // SAFETY: the caller's promise; the slot is this cell's own.
        unsafe { option_set_callback_func(value, self.slot()) }
    }

    /// Copy this callback into a buffer-local slot, freeing what was there.
    ///
    /// # Safety
    /// `bufcb` must point at a live buffer's callback field.
    pub(crate) unsafe fn copy_to_buflocal(self, bufcb: *mut Callback) {
        // SAFETY: the caller's promise.
        unsafe { copy_global_to_buflocal_cb(self.slot(), bufcb) };
    }

    /// Mark what this callback references with `copy_id`, so the garbage
    /// collector leaves it alone. Answers whether to abort.
    pub(crate) fn set_ref(self, copy_id: c_int) -> bool {
        // SAFETY: the caller's promise; the slot is this cell's own.
        set_ref_in_callback(unsafe { &*self.slot() }, copy_id, None, None)
    }
}

/// The global `'complete'` `F{func}` callbacks: one slot per entry of the
/// option, empty for every entry that is not an `F{func}`.
///
/// This is the cached copy `:set complete=` leaves behind so a buffer that
/// has never had `:setlocal complete=` can be given the same array; the
/// live one a completion reads is always `curbuf`'s `b_p_cpt_cb`. Upstream
/// keeps it as a `Callback *` with `cpt_cb_count` beside it; here it is a
/// `Vec` in [`ComplState`], and a `Callback` owns what it names without a
/// destructor, so replacing the cache frees the old slots by hand.
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
        CPT_CB.with(Vec::len) as c_int
    }

    /// Replace the cached array with a copy of `src`'s `count` callbacks.
    /// A `count` of zero leaves the cache alone, as upstream's did.
    ///
    /// # Safety
    /// `src` must hold `count` live callbacks.
    pub(crate) unsafe fn replace_from(self, src: *mut Callback, count: c_int) {
        if count == 0 {
            return;
        }
        let copy = |i: usize| {
            let mut slot = Callback::None;
            // SAFETY: the caller's `count` live callbacks.
            unsafe {
                let from = src.add(i);
                if (*from).is_set() {
                    callback_copy(&mut slot, &*from);
                }
            }
            slot
        };
        let fresh: Vec<Callback> = (0..count as usize).map(copy).collect();
        for mut old in CPT_CB.replace(fresh) {
            callback_free(&mut old);
        }
    }

    /// Copy the cache into `buffer`'s array, freeing what was there.
    pub(crate) fn copy_to_buffer(self, buffer: Buf) {
        let raw = buffer.raw();
        CPT_CB.with_mut(|slots| {
            let count = slots.len() as c_int;
            // SAFETY: a live buffer owns its callback array and count; the
            // two fields are addressed from its raw pointer rather than
            // through `DerefMut`, so taking the second does not invalidate
            // the first; the cache's own slots are only read.
            unsafe {
                let (dst, dst_count) = (&raw mut (*raw).b_p_cpt_cb, &raw mut (*raw).b_p_cpt_count);
                copy_cpt_callbacks(dst, dst_count, slots.as_mut_ptr(), count);
            }
        });
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
    cpt_entries(Buf::current().b_p_cpt.bytes()).count() as c_int
}

/// Copy a global callback function to a buffer-local callback.
///
/// # Safety
///
/// `globcb` must point at an initialized callback, unaliased for the call.
/// `bufcb` must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn copy_global_to_buflocal_cb(globcb: *mut Callback, bufcb: *mut Callback) {
    unsafe { callback_free(&mut *bufcb) };
    if unsafe { &*globcb }.is_set() {
        unsafe { callback_copy(&mut *bufcb, &*globcb) };
    }
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
        .data();
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        unsafe { option_set_callback_func(value, &raw mut buf.b_cfu_cb) }
    } else {
        let retval = unsafe { cfu_cb().set_from_option(value) };
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
    // SAFETY: a live buffer owns its callback field.
    unsafe { cfu_cb().copy_to_buflocal(&raw mut buffer.b_cfu_cb) }
}

/// Parse the `'omnifunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub fn did_set_omnifunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let value = args
        .os_newval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .data();
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        unsafe { option_set_callback_func(value, &raw mut buf.b_ofu_cb) }
    } else {
        let retval = unsafe { ofu_cb().set_from_option(value) };
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
    // SAFETY: a live buffer owns its callback field.
    unsafe { ofu_cb().copy_to_buflocal(&raw mut buffer.b_ofu_cb) }
}

/// Free an array of `'complete'` `F{func}` callbacks and null the pointer.
///
/// # Safety
///
/// `callbacks` must point at a writable `*mut Callback` slot the caller owns
/// for the call.
pub unsafe fn clear_cpt_callbacks(callbacks: *mut *mut Callback, count: c_int) {
    if callbacks.is_null() || unsafe { *callbacks }.is_null() {
        return;
    }
    for i in 0..count as isize {
        unsafe { callback_free(&mut *(*callbacks).offset(i)) };
    }
    unsafe { xfree((*callbacks).cast::<c_void>()) };
    unsafe { *callbacks = ptr::null_mut() };
}

/// Copy `cnt` `Callback`s from `src` to `*dest`, clearing what was there and
/// allocating the destination.
///
/// # Safety
///
/// `dest` must point at a writable `*mut Callback` slot the caller owns for
/// the call. `dest_cnt` must point at a writable `int` the caller owns. `src`
/// must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn copy_cpt_callbacks(
    dest: *mut *mut Callback,
    dest_cnt: *mut c_int,
    src: *mut Callback,
    cnt: c_int,
) {
    if cnt == 0 {
        return;
    }
    unsafe { clear_cpt_callbacks(dest, *dest_cnt) };
    // `xcalloc`, not `xmalloc`: an entry nothing fills has to read back as
    // `Callback::None`, which is discriminant 0.
    unsafe { *dest = xcalloc(cnt as size_t, size_of::<Callback>()).cast::<Callback>() };
    unsafe { *dest_cnt = cnt };
    for i in 0..cnt as isize {
        if unsafe { &*src.offset(i) }.is_set() {
            unsafe { callback_copy(&mut *(*dest).offset(i), &*src.offset(i)) };
        }
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
/// # Safety
///
/// `args` must point at a live `OptSet`, unaliased for the call.
pub unsafe fn set_cpt_callbacks(args: *mut OptSet) -> Result<(), Failed> {
    let local = unsafe { (*args).os_flags }.has(OptionSetFlags::LOCAL);
    if Buf::current_or_none().is_none() {
        return Err(Failed);
    }

    let (buffer, count) = (Buf::current_raw(), Buf::current().b_p_cpt_count);
    unsafe { clear_cpt_callbacks(&raw mut (*buffer).b_p_cpt_cb, count) };
    Buf::current().b_p_cpt_count = 0;

    let count = get_cpt_sources_count();
    if count == 0 {
        return Ok(());
    }
    // Zeroed for the reason `copy_cpt_callbacks` zeroes.
    Buf::current().b_p_cpt_cb =
        unsafe { xcalloc(count as size_t, size_of::<Callback>()) }.cast::<Callback>();
    Buf::current().b_p_cpt_count = count;

    // A copy: the walk writes the buffer's callback array.
    let option = Buf::current().b_p_cpt.bytes().to_vec();
    for (idx, entry) in cpt_entries(&option).enumerate() {
        let part = entry.part;
        if part.first() == Some(&b'F') && part.len() > 1 {
            // Drop the `^N` max-matches suffix.
            let end = part.iter().position(|&b| b == b'^').unwrap_or(part.len());
            let name = XString::from_bytes(&part[1..end]);
            // SAFETY: `idx` counts the entries, which is the array's length.
            let slot = unsafe { Buf::current().b_p_cpt_cb.add(idx) };
            // SAFETY: a NUL-terminated name, and a slot of the array just
            // allocated.
            if unsafe { option_set_callback_func(name.as_ptr().cast_mut(), slot) }.is_err() {
                // SAFETY: as above.
                unsafe { *slot = Callback::None };
            }
        }
    }

    if !local {
        // ':set' was used instead of ':setlocal': cache the callback array.
        unsafe { cpt_cb().replace_from(Buf::current().b_p_cpt_cb, Buf::current().b_p_cpt_count) };
    }
    Ok(())
}

/// Parse the `'thesaurusfunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub fn did_set_thesaurusfunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        // Buffer-local option set.
        {
            let value = buf.b_p_tsrfu.value_ptr();
            unsafe { option_set_callback_func(value, &raw mut buf.b_tsrfu_cb) }
        }
    } else {
        // Global option set.
        let retval =
            p_tsrfu(|value| unsafe { tsrfu_cb().set_from_option(value.as_ptr().cast_mut()) });
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
    let (callbacks, count) = (buffer.b_p_cpt_cb, buffer.b_p_cpt_count);
    if callbacks.is_null() {
        return false;
    }
    let mut abort = false;
    for i in 0..count as isize {
        // SAFETY: the buffer's own array of `count` live callbacks.
        let slot = unsafe { &*callbacks.offset(i) };
        abort = abort || set_ref_in_callback(slot, copy_id, None, None);
    }
    abort
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

/// The callback to use for insert-mode completion of `type_0`.
pub(crate) fn get_insert_callback(type_0: c_int) -> *mut Callback {
    if type_0 == CTRL_X_FUNCTION {
        return unsafe { &raw mut (*Buf::current_raw()).b_cfu_cb };
    }
    if type_0 == CTRL_X_OMNI {
        return unsafe { &raw mut (*Buf::current_raw()).b_ofu_cb };
    }
    // CTRL_X_THESAURUS
    if !Buf::current().b_p_tsrfu.bytes().is_empty() {
        unsafe { &raw mut (*Buf::current_raw()).b_tsrfu_cb }
    } else {
        tsrfu_cb().slot()
    }
}

/// Call `'completefunc'`, `'omnifunc'` or `'thesaurusfunc'` and add whatever
/// it answers to the match list.
///
/// `type_0` is one of `CTRL_X_OMNI`, `CTRL_X_FUNCTION` or `CTRL_X_THESAURUS`;
/// `cb` is set when a function in `'complete'` triggered this, null otherwise.
///
/// # Safety
///
/// `cb` must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn expand_by_function(type_0: c_int, base: ComplStr, mut cb: *mut Callback) {
    debug_assert!(Buf::current_or_none().is_some());

    let is_cpt_function = !cb.is_null();
    if !is_cpt_function {
        if get_complete_funcname(type_0).is_empty() {
            return;
        }
        cb = get_insert_callback(type_0);
    }

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
    if unsafe { callback_call(&*cb, &args, &mut rettv) } {
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
        unsafe { ins_compl_add_list(list.as_ptr()) };
    } else if let Some(dict) = &matchdict {
        unsafe { ins_compl_add_dict(dict.as_ptr()) };
    }

    // Restore State, it might have been changed.
    State.set(save_state);
    drop(matchdict);
    drop(matchlist);
}

/// The attribute of the named highlight group, or `-1` for no name.
///
/// # Safety
///
/// `hlname` must point at a NUL-terminated string.
#[inline]
pub(crate) unsafe fn get_user_highlight_attr(hlname: *const c_char) -> c_int {
    if !hlname.is_null() && unsafe { *hlname } as c_int != NUL {
        return unsafe { syn_name2attr(hlname) };
    }
    -1
}

/// The callback the `'complete'` entry at the start of `entry` names, if it
/// refers to a user-defined function; `idx` indexes the callback array.
pub(crate) fn get_callback_if_cpt_func(entry: &[u8], idx: c_int) -> *mut Callback {
    let buffer = Buf::current_raw();
    match entry.first() {
        // SAFETY: the current buffer's own callback field.
        Some(b'o') => unsafe { &raw mut (*buffer).b_ofu_cb },
        Some(b'F') if !matches!(entry.get(1), None | Some(b',')) => {
            // 'F{func}' case.
            // SAFETY: the entry's index is in range of the array
            // `set_cpt_callbacks` sized to the entries.
            let slot = unsafe { Buf::current().b_p_cpt_cb.offset(idx as isize) };
            // SAFETY: a slot of that array.
            if unsafe { &*slot }.is_set() {
                slot
            } else {
                ptr::null_mut()
            }
        }
        // SAFETY: as for `o`.
        Some(b'F') => unsafe { &raw mut (*buffer).b_cfu_cb }, // 'cfu'
        _ => ptr::null_mut(),
    }
}

/// Call the functions named in `'complete'` with `findstart=1` and record the
/// start column each answers.
pub(crate) fn prepare_cpt_compl_funcs() {
    // Make a copy of 'cpt' in case the buffer gets wiped out.
    let cpt = strip_caret_numbers(Buf::current().b_p_cpt.bytes());
    for (idx, entry) in cpt_entries(&cpt).enumerate() {
        let idx = idx as c_int;
        let cb = get_callback_if_cpt_func(entry.at, idx);
        if cb.is_null() {
            cpt_sources().update(idx, |source| source.cs_startcol = -3);
            continue;
        }
        let mut startcol = 0;
        let col = Win::current().w_cursor.col;
        // SAFETY: a live callback, and `startcol` is this frame's own.
        if unsafe { get_userdefined_compl_info(col, cb, &raw mut startcol) }.is_err() {
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
    if idx >= 0 && idx < cpt_sources().len() as c_int - 1 {
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
                cs_flag: entry.at[0] as c_char,
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

/// Collect matches through `cb` and record its `refresh:always` flag.
///
/// # Safety
///
/// `cb` must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn get_cpt_func_completion_matches(cb: *mut Callback) {
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

    unsafe { expand_by_function(0, cpt_compl_pattern(), cb) };

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
            ptr::null_mut()
        };
        if !cb.is_null() {
            remove_old_matches();
            let mut startcol = 0;
            let col = Win::current().w_cursor.col;
            // SAFETY: a live callback, and `startcol` is this frame's own.
            let ret = unsafe { get_userdefined_compl_info(col, cb, &raw mut startcol) };
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
                // SAFETY: as above.
                unsafe { get_cpt_func_completion_matches(cb) };
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
