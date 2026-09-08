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
use crate::guard::Lock;
use crate::semsg;
use crate::strings::vim_strchr;
use crate::types::{
    Failed, IOSIZE, NUL, OptionSetFlags, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
};
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
    ///
    /// # Safety
    /// Runs the `set_ref_in_*` walk over a live typval graph.
    pub(crate) unsafe fn set_ref(self, copy_id: c_int) -> bool {
        // SAFETY: the caller's promise; the slot is this cell's own.
        unsafe { set_ref_in_callback(self.slot(), copy_id, ptr::null_mut(), ptr::null_mut()) }
    }
}

/// The global `'complete'` `F{func}` callbacks: one slot per entry of the
/// option, empty for every entry that is not an `F{func}`.
///
/// This is the cached copy `:set complete=` leaves behind so a buffer that
/// has never had `:setlocal complete=` can be given the same array; the
/// live one a completion reads is always `curbuf`'s `b_p_cpt_cb`. Upstream
/// keeps it as a `Callback *` with `cpt_cb_count` beside it and passes
/// *both cells' addresses* to `copy_cpt_callbacks` as out-parameters, which
/// is the only reason those two `.ptr()`s existed.
#[derive(Clone, Copy)]
pub(crate) struct CptCallbacks(());

/// The cached global `'complete'` `F{func}` callbacks. See [`CptCallbacks`].
pub(crate) fn cpt_cb() -> CptCallbacks {
    CptCallbacks(())
}

impl CptCallbacks {
    /// The array, null when `'complete'` has never been set globally.
    pub(crate) fn slots(self) -> *mut Callback {
        CPT_CB.get()
    }

    /// The number of slots.
    pub(crate) fn count(self) -> c_int {
        CPT_CB_COUNT.get()
    }

    /// Replace the cached array with a copy of `src`'s `count` callbacks.
    /// A `count` of zero leaves the cache alone, as upstream's did.
    ///
    /// # Safety
    /// `src` must hold `count` live callbacks.
    pub(crate) unsafe fn replace_from(self, src: *mut Callback, count: c_int) {
        let mut slots = CPT_CB.get();
        let mut slot_count = CPT_CB_COUNT.get();
        // SAFETY: the caller's promise; the out-parameters are locals, so
        // nothing observes the cache half-written.
        unsafe { copy_cpt_callbacks(&raw mut slots, &raw mut slot_count, src, count) };
        CPT_CB.set(slots);
        CPT_CB_COUNT.set(slot_count);
    }
}

/// The per-`'complete'`-entry state: one row for every comma-separated
/// segment of the option, plus the index of the segment being collected.
///
/// Upstream keeps this as a `CptSource *` with a hand-rolled
/// `cpt_sources_count` beside it and `cpt_sources_index` as a third global,
/// `xcalloc`'d by `setup_cpt_sources` and `xfree`'d by `cpt_sources_clear`;
/// eleven sites reached a row by writing `(*cpt_sources_array.ptr()).offset(i)`
/// and dereferencing, with the bounds carried in the reader's head.
///
/// `CptSources` is the one owner, in `ComplStr`'s shape: it names the cells
/// rather than pointing into them, so it is `Copy` and forms no reference
/// into a global. The rows are a boxed slice; `set_rows` installs the
/// pointer and the count *together* (upstream published the array first and
/// the count last, so the two disagreed for the length of the parse loop),
/// and every row is read out by value or written through [`update`].
///
/// [`update`]: CptSources::update
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
        CPT_SOURCES.get().is_null()
    }

    /// The rows, empty while [`is_unset`](Self::is_unset).
    pub(crate) fn rows(self) -> &'static [CptSource] {
        let rows = CPT_SOURCES.get();
        if rows.is_null() {
            return &[];
        }
        // SAFETY: `set_rows` stored a boxed slice of exactly this length and
        // only `clear` drops it.
        unsafe { ::core::slice::from_raw_parts(rows, CPT_SOURCES_COUNT.get() as usize) }
    }

    /// Row `idx` by value, or the zeroed row when `idx` is out of range.
    pub(crate) fn row(self, idx: c_int) -> CptSource {
        usize::try_from(idx)
            .ok()
            .and_then(|idx| self.rows().get(idx).copied())
            .unwrap_or(CPT_SOURCE_INIT)
    }

    /// The row the scan is collecting from, by value.
    pub(crate) fn current(self) -> CptSource {
        self.row(self.index())
    }

    /// Change row `idx` in place; out of range does nothing.
    pub(crate) fn update(self, idx: c_int, f: impl FnOnce(&mut CptSource)) {
        let rows = CPT_SOURCES.get();
        let Ok(idx) = usize::try_from(idx) else {
            return;
        };
        if rows.is_null() || idx >= CPT_SOURCES_COUNT.get() as usize {
            return;
        }
        // SAFETY: `idx` is in range of the boxed slice `set_rows` stored, and
        // `f` only writes fields of the row.
        f(unsafe { &mut *rows.add(idx) });
    }

    /// Take `rows` as the new state, dropping whatever was there. The index
    /// is left alone: the caller sets it when the scan starts. Empty `rows`
    /// leave the state unset -- a zero-length boxed slice is a *dangling*
    /// pointer, not a null one, and `is_unset` is the null check upstream's
    /// callers do.
    pub(crate) fn set_rows(self, rows: Vec<CptSource>) {
        self.free_rows();
        if rows.is_empty() {
            return;
        }
        let count = rows.len() as c_int;
        CPT_SOURCES.set(Box::into_raw(rows.into_boxed_slice()).cast::<CptSource>());
        CPT_SOURCES_COUNT.set(count);
    }

    /// Drop the rows and forget where the scan was — C's
    /// `cpt_sources_clear()`.
    pub(crate) fn clear(self) {
        self.free_rows();
        CPT_SOURCES_INDEX.set(-1);
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

    fn free_rows(self) {
        let rows = CPT_SOURCES.get();
        let count = CPT_SOURCES_COUNT.replace(0) as usize;
        CPT_SOURCES.set(ptr::null_mut());
        if rows.is_null() {
            return;
        }
        // SAFETY: the allocation is this owner's own boxed slice, of exactly
        // `count` rows, and `CptSource` owns nothing.
        drop(unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(rows, count)) });
    }
}

/// Step over the `,` and ` ` that separate two `'complete'` entries.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, unaliased for the call.
unsafe fn skip_cpt_delims(mut p: *mut c_char) -> *mut c_char {
    while unsafe { *p } as c_int == ',' as c_int || unsafe { *p } as c_int == ' ' as c_int {
        p = unsafe { p.offset(1) };
    }
    p
}

/// The number of entries in `'complete'` — every non-empty comma-separated
/// segment counts as one.
pub(crate) fn get_cpt_sources_count() -> c_int {
    let mut dummy = [0 as c_char; LSIZE as usize];
    let mut count = 0;
    let mut p = Buf::current().b_p_cpt;
    while unsafe { *p } as c_int != NUL {
        p = unsafe { skip_cpt_delims(p) };
        if unsafe { *p } as c_int != NUL {
            // Advance p.
            // SAFETY: `p` walks `'complete'` and `dummy` has `LSIZE` bytes.
            unsafe { next_cpt_part(&raw mut p, dummy.as_mut_ptr(), LSIZE as size_t) };
            count += 1;
        }
    }
    count
}

/// Copy a global callback function to a buffer-local callback.
///
/// # Safety
///
/// `globcb` must point at an initialized callback, unaliased for the call.
/// `bufcb` must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn copy_global_to_buflocal_cb(globcb: *mut Callback, bufcb: *mut Callback) {
    unsafe { callback_free(bufcb) };
    if unsafe { &*globcb }.is_set() {
        unsafe { callback_copy(bufcb, globcb) };
    }
}

/// Parse the `'completefunc'` value and set the callback function; the value
/// may be a function name, `function(<name>)`, `funcref(<name>)` or a lambda.
///
/// This is an `opt_did_set_cb` row in the generated option table.
pub fn did_set_completefunc(args: &mut OptSet) -> Option<&CStr> {
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
        Some(e_invarg)
    } else {
        None
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
pub fn did_set_omnifunc(args: &mut OptSet) -> Option<&CStr> {
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
        Some(e_invarg)
    } else {
        None
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
        unsafe { callback_free((*callbacks).offset(i)) };
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
            unsafe { callback_copy((*dest).offset(i), src.offset(i)) };
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
    // SAFETY: a live buffer owns its callback array, and the cache hands back
    // its own slots.
    let raw = buffer.raw();
    let (slots, count) = (cpt_cb().slots(), cpt_cb().count());
    // SAFETY: the two fields are addressed from the buffer's raw pointer
    // rather than through `DerefMut`, so taking the second does not
    // invalidate the first, and the cache hands back its own slots.
    unsafe {
        let (dst, dst_count) = (&raw mut (*raw).b_p_cpt_cb, &raw mut (*raw).b_p_cpt_count);
        copy_cpt_callbacks(dst, dst_count, slots, count);
    };
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

    let mut part = [0 as c_char; LSIZE as usize];
    let mut idx: isize = 0;
    let mut p = Buf::current().b_p_cpt;
    while unsafe { *p } as c_int != NUL {
        p = unsafe { skip_cpt_delims(p) };
        if unsafe { *p } as c_int != NUL {
            // Advance p.
            // SAFETY: `p` walks `'complete'` and `part` has `LSIZE` bytes.
            let slen = unsafe { next_cpt_part(&raw mut p, part.as_mut_ptr(), LSIZE as size_t) };
            if slen > 0 && part[0] as c_int == 'F' as c_int && part[1] as c_int != NUL {
                // Drop the `^N` max-matches suffix.
                let caret = unsafe { vim_strchr(part.as_mut_ptr(), '^' as c_int) };
                if !caret.is_null() {
                    unsafe { *caret = NUL as c_char };
                }
                let slot = unsafe { Buf::current().b_p_cpt_cb.offset(idx) };
                if unsafe { option_set_callback_func(part.as_mut_ptr().offset(1), slot) }.is_err() {
                    unsafe { *slot = Callback::None };
                }
            }
            idx += 1;
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
pub fn did_set_thesaurusfunc(args: &mut OptSet) -> Option<&CStr> {
    let mut buf = args.os_buf;
    let retval = if args.os_flags.has(OptionSetFlags::LOCAL) {
        // Buffer-local option set.
        unsafe { option_set_callback_func(buf.b_p_tsrfu, &raw mut buf.b_tsrfu_cb) }
    } else {
        // Global option set.
        let retval = unsafe { tsrfu_cb().set_from_option(p_tsrfu.get()) };
        // When using :set, free the local callback.
        if !args.os_flags.has(OptionSetFlags::GLOBAL) {
            unsafe { callback_free(&raw mut buf.b_tsrfu_cb) };
        }
        retval
    };
    if retval.is_err() {
        Some(e_invarg)
    } else {
        None
    }
}

/// Mark `copy_id` references in an array of `F{func}` callbacks so they are not
/// garbage collected.
///
/// # Safety
///
/// `callbacks` must point at an initialized callback, unaliased for the call.
pub unsafe fn set_ref_in_cpt_callbacks(
    callbacks: *mut Callback,
    count: c_int,
    copy_id: c_int,
) -> bool {
    if callbacks.is_null() {
        return false;
    }
    let mut abort = false;
    let (no_list, no_dict) = (ptr::null_mut(), ptr::null_mut());
    for i in 0..count as isize {
        // SAFETY: `callbacks` holds `count` live callbacks.
        let slot = unsafe { callbacks.offset(i) };
        // SAFETY: as above; the two nulls say there is no containing list or
        // dict to mark.
        abort = abort || unsafe { set_ref_in_callback(slot, copy_id, no_list, no_dict) };
    }
    abort
}

/// Mark the global `'completefunc'`, `'omnifunc'` and `'thesaurusfunc'`
/// callbacks with `copy_id` so they are not garbage collected.
pub fn set_ref_in_insexpand_funcs(copy_id: c_int) -> bool {
    let mut abort = unsafe { cfu_cb().set_ref(copy_id) };
    abort = abort || unsafe { ofu_cb().set_ref(copy_id) };
    abort = abort || unsafe { tsrfu_cb().set_ref(copy_id) };
    abort =
        abort || unsafe { set_ref_in_cpt_callbacks(cpt_cb().slots(), cpt_cb().count(), copy_id) };
    abort
}

/// The user-defined completion function name for completion `type_0`.
pub(crate) fn get_complete_funcname(type_0: c_int) -> *mut c_char {
    match type_0 {
        CTRL_X_FUNCTION => Buf::current().b_p_cfu,
        CTRL_X_OMNI => Buf::current().b_p_ofu,
        CTRL_X_THESAURUS => {
            if unsafe { *Buf::current().b_p_tsrfu } as c_int == NUL {
                p_tsrfu.get()
            } else {
                Buf::current().b_p_tsrfu
            }
        }
        _ => c"".as_ptr().cast_mut(),
    }
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
    if unsafe { *Buf::current().b_p_tsrfu } as c_int != NUL {
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
/// `base` must point at a NUL-terminated string, unaliased for the call. `cb`
/// must point at an initialized callback, unaliased for the call.
pub(crate) unsafe fn expand_by_function(type_0: c_int, base: *mut c_char, mut cb: *mut Callback) {
    debug_assert!(Buf::current_or_none().is_some());

    let is_cpt_function = !cb.is_null();
    if !is_cpt_function {
        if unsafe { *get_complete_funcname(type_0) } as c_int == NUL {
            return;
        }
        cb = get_insert_callback(type_0);
    }

    // Call the function to obtain the list of matches.
    let mut args = [TYPVAL_T_INIT; 3];
    args[0].v_type = VAR_NUMBER;
    args[1].v_type = VAR_STRING;
    args[2].v_type = VAR_UNKNOWN;
    args[0].vval.v_number = 0;
    args[1].vval.v_string = if base.is_null() {
        c"".as_ptr().cast_mut()
    } else {
        base
    };

    let mut matchlist: *mut List = ptr::null_mut();
    let mut matchdict: *mut Dict = ptr::null_mut();
    let mut rettv = TYPVAL_T_INIT;
    let save_state = State.get();
    let pos = Win::current().w_cursor;

    // Lock the text to avoid weird things from happening.  Also disallow
    // switching to another window: it should not be needed and may end up
    // in Insert mode in another buffer.
    let locked = Lock::text();
    if unsafe { callback_call(cb, 2, args.as_mut_ptr(), &raw mut rettv) } {
        match rettv.v_type {
            VAR_LIST => matchlist = unsafe { rettv.vval.v_list },
            VAR_DICT => matchdict = unsafe { rettv.vval.v_dict },
            // VAR_SPECIAL falls through to the default.
            // TODO(brammool): Give error message?
            _ => unsafe { tv_clear(&raw mut rettv) },
        }
    }
    drop(locked);

    Win::current().w_cursor = pos; // restore the cursor position
    check_cursor(Win::current()); // make sure the position is valid, just in case
    validate_cursor(Win::current());
    if !equalpos(Win::current().w_cursor, pos) {
        emsg(gettext(E_COMPLDEL));
    } else if !matchlist.is_null() {
        unsafe { ins_compl_add_list(matchlist) };
    } else if !matchdict.is_null() {
        unsafe { ins_compl_add_dict(matchdict) };
    }

    // Restore State, it might have been changed.
    State.set(save_state);
    if !matchdict.is_null() {
        unsafe { tv_dict_unref(matchdict) };
    }
    if !matchlist.is_null() {
        unsafe { tv_list_unref(matchlist) };
    }
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

/// The callback `p` names if it refers to a user-defined function in
/// `'complete'`; `idx` indexes the callback array.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn get_callback_if_cpt_func(mut p: *mut c_char, idx: c_int) -> *mut Callback {
    if unsafe { *p } as c_int == 'o' as c_int {
        return unsafe { &raw mut (*Buf::current_raw()).b_ofu_cb };
    }
    if unsafe { *p } as c_int == 'F' as c_int {
        p = unsafe { p.offset(1) };
        if unsafe { *p } as c_int != ',' as c_int && unsafe { *p } as c_int != NUL {
            // 'F{func}' case.
            let slot = unsafe { Buf::current().b_p_cpt_cb.offset(idx as isize) };
            return if unsafe { &*slot }.is_set() {
                slot
            } else {
                ptr::null_mut()
            };
        }
        return unsafe { &raw mut (*Buf::current_raw()).b_cfu_cb }; // 'cfu'
    }
    ptr::null_mut()
}

/// Call the functions named in `'complete'` with `findstart=1` and record the
/// start column each answers.
pub(crate) fn prepare_cpt_compl_funcs() {
    // The throwaway `copy_option_part` steps the entry into.
    let mut skipped = [0 as c_char; IOSIZE as usize];
    // Make a copy of 'cpt' in case the buffer gets wiped out.
    let cpt = unsafe { xstrdup(Buf::current().b_p_cpt) };
    unsafe { strip_caret_numbers_in_place(cpt) };

    let mut idx = 0;
    let mut p = cpt;
    while unsafe { *p } != 0 {
        p = unsafe { skip_cpt_delims(p) };
        if unsafe { *p } as c_int == NUL {
            break;
        }

        let cb = unsafe { get_callback_if_cpt_func(p, idx) };
        if cb.is_null() {
            cpt_sources().update(idx, |source| source.cs_startcol = -3);
        } else {
            let mut startcol = 0;
            let col = Win::current().w_cursor.col;
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

        // Advance p.
        // SAFETY: `p` walks `'complete'` and `skipped` has `IOSIZE` bytes.
        unsafe { next_cpt_part(&raw mut p, skipped.as_mut_ptr(), IOSIZE as size_t) };
        idx += 1;
    }
    unsafe { xfree(cpt.cast::<c_void>()) };
}

/// Advance `cpt_sources_index` by one, or report E684 and fail.
pub(crate) fn advance_cpt_sources_index_safe() -> Result<(), Failed> {
    let idx = cpt_sources().index();
    if idx >= 0 && idx < cpt_sources().rows().len() as c_int - 1 {
        cpt_sources().set_index(idx + 1);
        return Ok(());
    }
    semsg!("E684: List index out of range: {}", idx);
    Err(Failed)
}

/// Build the per-`'complete'`-entry state: the source letter and its `^N`
/// max-matches limit.
pub(crate) fn setup_cpt_sources() {
    cpt_sources().clear();

    let count = get_cpt_sources_count();
    if count == 0 {
        return;
    }

    let mut rows = Vec::with_capacity(count as usize);
    let mut part = [0 as c_char; LSIZE as usize];
    let mut p = Buf::current().b_p_cpt;
    while unsafe { *p } != 0 {
        p = unsafe { skip_cpt_delims(p) };
        if unsafe { *p } != 0 {
            // If not end of string, count this segment.
            let mut source = CptSource {
                cs_flag: unsafe { *p },
                ..CPT_SOURCE_INIT
            };
            part.fill(0);
            // Advance p.
            // SAFETY: `p` walks `'complete'` and `part` has `LSIZE` bytes.
            let slen = unsafe { next_cpt_part(&raw mut p, part.as_mut_ptr(), LSIZE as size_t) };
            if slen > 0 {
                let caret = unsafe { vim_strchr(part.as_mut_ptr(), '^' as c_int) };
                if !caret.is_null() {
                    source.cs_max_matches = unsafe { atoi(caret.offset(1)) };
                }
            }
            rows.push(source);
        }
    }
    debug_assert_eq!(rows.len(), count as usize);
    cpt_sources().set_rows(rows);
}

/// Whether any completion source has `refresh` set to `always`.
pub(crate) fn is_cpt_func_refresh_always() -> bool {
    cpt_sources().rows().iter().any(|s| s.cs_refresh_always)
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
        unsafe { ins_compl_insert_bytes(ins_compl_leader(), -1) };
    }

    unsafe { expand_by_function(0, cpt_compl_pattern().data(), cb) };

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
    // The throwaway `copy_option_part` steps the entry into.
    let mut skipped = [0 as c_char; IOSIZE as usize];
    // Make the completion list linear (non-cyclic).
    ins_compl_make_linear();
    // Make a copy of 'cpt' in case the buffer gets wiped out.
    let cpt = unsafe { xstrdup(Buf::current().b_p_cpt) };
    unsafe { strip_caret_numbers_in_place(cpt) };

    cpt_sources().set_index(0);
    let mut p = cpt;
    while unsafe { *p } != 0 {
        p = unsafe { skip_cpt_delims(p) };
        if unsafe { *p } as c_int == NUL {
            break;
        }

        let idx = cpt_sources().index();
        if cpt_sources().row(idx).cs_refresh_always {
            let cb = unsafe { get_callback_if_cpt_func(p, idx) };
            if !cb.is_null() {
                unsafe { remove_old_matches() };
                let mut startcol = 0;
                let ret = unsafe {
                    get_userdefined_compl_info(Win::current().w_cursor.col, cb, &raw mut startcol)
                };
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
                    unsafe { get_cpt_func_completion_matches(cb) };
                }
            }
        }

        // Advance p.
        // SAFETY: `p` walks `'complete'` and `skipped` has `IOSIZE` bytes.
        unsafe { next_cpt_part(&raw mut p, skipped.as_mut_ptr(), IOSIZE as size_t) };
        if unsafe { may_advance_cpt_index(p) } {
            let _ = advance_cpt_sources_index_safe();
        }
    }
    cpt_sources().set_index(-1);

    unsafe { xfree(cpt.cast::<c_void>()) };
    // Make the list cyclic.
    compl_matches.set(ins_compl_make_cyclic());
}

/// C's `copy_option_part(&p, buf, len, ",")`: step `p` past one entry of
/// `'complete'`, copying what it stepped over into `buf`.
///
/// # Safety
/// `p` addresses a cursor into a NUL-terminated option string, and `buf` has
/// `len` writable bytes.
unsafe fn next_cpt_part(p: *mut *mut c_char, buf: *mut c_char, len: size_t) -> size_t {
    let comma = c",".as_ptr().cast_mut();
    // SAFETY: the caller's promise.
    unsafe { copy_option_part(p, buf, len, comma) }
}
