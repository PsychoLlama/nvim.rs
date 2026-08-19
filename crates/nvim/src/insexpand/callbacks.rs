//! `'completefunc'`, `'omnifunc'`, `'thesaurusfunc'` and the `'complete'` `F` flag.
//!
//! The `did_set_*` halves are the option callbacks that compile a funcname
//! into a `Callback`; [`expand_by_function`] is the call itself, which runs
//! the function twice (`findstart` then the matches) exactly as upstream
//! does.  The `cpt_sources_*` half tracks the per-`'complete'`-entry state
//! those functions need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{
    FAIL, IOSIZE, NUL, OK, OptionSetFlags, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
};

/// Step over the `,` and ` ` that separate two `'complete'` entries.
unsafe fn skip_cpt_delims(mut p: *mut c_char) -> *mut c_char {
    unsafe {
        while *p as c_int == ',' as c_int || *p as c_int == ' ' as c_int {
            p = p.offset(1);
        }
        p
    }
}

/// The number of entries in `'complete'` — every non-empty comma-separated
/// segment counts as one.
pub(crate) unsafe fn get_cpt_sources_count() -> c_int {
    unsafe {
        let mut dummy = [0 as c_char; LSIZE as usize];
        let mut count = 0;
        let mut p = (*curbuf.get()).b_p_cpt;
        while *p as c_int != NUL {
            p = skip_cpt_delims(p);
            if *p as c_int != NUL {
                // Advance p.
                copy_option_part(
                    &raw mut p,
                    dummy.as_mut_ptr(),
                    LSIZE as size_t,
                    c",".as_ptr().cast_mut(),
                );
                count += 1;
            }
        }
        count
    }
}

/// Copy a global callback function to a buffer-local callback.
pub(crate) unsafe fn copy_global_to_buflocal_cb(globcb: *mut Callback, bufcb: *mut Callback) {
    unsafe {
        callback_free(bufcb);
        if (*globcb).type_0 != kCallbackNone {
            callback_copy(bufcb, globcb);
        }
    }
}

/// Parse the `'completefunc'` value and set the callback function; the value
/// may be a function name, `function(<name>)`, `funcref(<name>)` or a lambda.
///
/// This is an `opt_did_set_cb` row in the generated option table.
pub unsafe fn did_set_completefunc(args: *mut optset_T) -> *const c_char {
    unsafe {
        let buf = (*args).os_buf as *mut buf_T;
        let retval = if (*args).os_flags.has(OptionSetFlags::LOCAL) {
            option_set_callback_func((*args).os_newval.string.data(), &raw mut (*buf).b_cfu_cb)
        } else {
            let retval = option_set_callback_func((*args).os_newval.string.data(), cfu_cb.ptr());
            if retval == OK && !(*args).os_flags.has(OptionSetFlags::GLOBAL) {
                set_buflocal_cfu_callback(buf);
            }
            retval
        };
        if retval == FAIL {
            &raw const e_invarg as *const c_char
        } else {
            ptr::null()
        }
    }
}

/// Copy the global `'completefunc'` callback into `buf`'s local one.
pub unsafe fn set_buflocal_cfu_callback(buf: *mut buf_T) {
    unsafe { copy_global_to_buflocal_cb(cfu_cb.ptr(), &raw mut (*buf).b_cfu_cb) }
}

/// Parse the `'omnifunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub unsafe fn did_set_omnifunc(args: *mut optset_T) -> *const c_char {
    unsafe {
        let buf = (*args).os_buf as *mut buf_T;
        let retval = if (*args).os_flags.has(OptionSetFlags::LOCAL) {
            option_set_callback_func((*args).os_newval.string.data(), &raw mut (*buf).b_ofu_cb)
        } else {
            let retval = option_set_callback_func((*args).os_newval.string.data(), ofu_cb.ptr());
            if retval == OK && !(*args).os_flags.has(OptionSetFlags::GLOBAL) {
                set_buflocal_ofu_callback(buf);
            }
            retval
        };
        if retval == FAIL {
            &raw const e_invarg as *const c_char
        } else {
            ptr::null()
        }
    }
}

/// Copy the global `'omnifunc'` callback into `buf`'s local one.
pub unsafe fn set_buflocal_ofu_callback(buf: *mut buf_T) {
    unsafe { copy_global_to_buflocal_cb(ofu_cb.ptr(), &raw mut (*buf).b_ofu_cb) }
}

/// Free an array of `'complete'` `F{func}` callbacks and null the pointer.
pub unsafe fn clear_cpt_callbacks(callbacks: *mut *mut Callback, count: c_int) {
    unsafe {
        if callbacks.is_null() || (*callbacks).is_null() {
            return;
        }
        for i in 0..count as isize {
            callback_free((*callbacks).offset(i));
        }
        xfree((*callbacks).cast::<c_void>());
        *callbacks = ptr::null_mut();
    }
}

/// Copy `cnt` `Callback`s from `src` to `*dest`, clearing what was there and
/// allocating the destination.
pub(crate) unsafe fn copy_cpt_callbacks(
    dest: *mut *mut Callback,
    dest_cnt: *mut c_int,
    src: *mut Callback,
    cnt: c_int,
) {
    unsafe {
        if cnt == 0 {
            return;
        }
        clear_cpt_callbacks(dest, *dest_cnt);
        *dest = xcalloc(cnt as size_t, size_of::<Callback>()).cast::<Callback>();
        *dest_cnt = cnt;
        for i in 0..cnt as isize {
            if (*src.offset(i)).type_0 != kCallbackNone {
                callback_copy((*dest).offset(i), src.offset(i));
            }
        }
    }
}

/// Copy the global `'complete'` `F{func}` callbacks into `buf`'s local array,
/// clearing any existing buffer-local callbacks first.
pub unsafe fn set_buflocal_cpt_callbacks(buf: *mut buf_T) {
    unsafe {
        if buf.is_null() || cpt_cb_count.get() == 0 {
            return;
        }
        copy_cpt_callbacks(
            &raw mut (*buf).b_p_cpt_cb,
            &raw mut (*buf).b_p_cpt_count,
            cpt_cb.get(),
            cpt_cb_count.get(),
        );
    }
}

/// Parse `'complete'` and (re)build the `F{func}` callbacks; entries other
/// than `F{func}` are counted but leave their slot empty.
pub unsafe fn set_cpt_callbacks(args: *mut optset_T) -> c_int {
    unsafe {
        let local = (*args).os_flags.has(OptionSetFlags::LOCAL);
        if curbuf.get().is_null() {
            return FAIL;
        }

        clear_cpt_callbacks(
            &raw mut (*curbuf.get()).b_p_cpt_cb,
            (*curbuf.get()).b_p_cpt_count,
        );
        (*curbuf.get()).b_p_cpt_count = 0;

        let count = get_cpt_sources_count();
        if count == 0 {
            return OK;
        }
        (*curbuf.get()).b_p_cpt_cb =
            xcalloc(count as size_t, size_of::<Callback>()).cast::<Callback>();
        (*curbuf.get()).b_p_cpt_count = count;

        let mut part = [0 as c_char; LSIZE as usize];
        let mut idx: isize = 0;
        let mut p = (*curbuf.get()).b_p_cpt;
        while *p as c_int != NUL {
            p = skip_cpt_delims(p);
            if *p as c_int != NUL {
                // Advance p.
                let slen = copy_option_part(
                    &raw mut p,
                    part.as_mut_ptr(),
                    LSIZE as size_t,
                    c",".as_ptr().cast_mut(),
                );
                if slen > 0 && part[0] as c_int == 'F' as c_int && part[1] as c_int != NUL {
                    // Drop the `^N` max-matches suffix.
                    let caret = vim_strchr(part.as_mut_ptr(), '^' as c_int);
                    if !caret.is_null() {
                        *caret = NUL as c_char;
                    }
                    let slot = (*curbuf.get()).b_p_cpt_cb.offset(idx);
                    if option_set_callback_func(part.as_mut_ptr().offset(1), slot) != OK {
                        (*slot).type_0 = kCallbackNone;
                    }
                }
                idx += 1;
            }
        }

        if !local {
            // ':set' was used instead of ':setlocal': cache the callback array.
            copy_cpt_callbacks(
                cpt_cb.ptr(),
                cpt_cb_count.ptr(),
                (*curbuf.get()).b_p_cpt_cb,
                (*curbuf.get()).b_p_cpt_count,
            );
        }
        OK
    }
}

/// Parse the `'thesaurusfunc'` value and set the callback function; an
/// `opt_did_set_cb` row in the generated option table.
pub unsafe fn did_set_thesaurusfunc(args: *mut optset_T) -> *const c_char {
    unsafe {
        let buf = (*args).os_buf as *mut buf_T;
        let retval = if (*args).os_flags.has(OptionSetFlags::LOCAL) {
            // Buffer-local option set.
            option_set_callback_func((*buf).b_p_tsrfu, &raw mut (*buf).b_tsrfu_cb)
        } else {
            // Global option set.
            let retval = option_set_callback_func(p_tsrfu.get(), tsrfu_cb.ptr());
            // When using :set, free the local callback.
            if !(*args).os_flags.has(OptionSetFlags::GLOBAL) {
                callback_free(&raw mut (*buf).b_tsrfu_cb);
            }
            retval
        };
        if retval == FAIL {
            &raw const e_invarg as *const c_char
        } else {
            ptr::null()
        }
    }
}

/// Mark `copyID` references in an array of `F{func}` callbacks so they are not
/// garbage collected.
pub unsafe fn set_ref_in_cpt_callbacks(
    callbacks: *mut Callback,
    count: c_int,
    copyID: c_int,
) -> bool {
    unsafe {
        if callbacks.is_null() {
            return false;
        }
        let mut abort = false;
        for i in 0..count as isize {
            abort = abort
                || set_ref_in_callback(
                    callbacks.offset(i),
                    copyID,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
        }
        abort
    }
}

/// Mark the global `'completefunc'`, `'omnifunc'` and `'thesaurusfunc'`
/// callbacks with `copyID` so they are not garbage collected.
pub unsafe fn set_ref_in_insexpand_funcs(copyID: c_int) -> bool {
    unsafe {
        let mark =
            |cb: *mut Callback| set_ref_in_callback(cb, copyID, ptr::null_mut(), ptr::null_mut());
        let mut abort = mark(cfu_cb.ptr());
        abort = abort || mark(ofu_cb.ptr());
        abort = abort || mark(tsrfu_cb.ptr());
        abort = abort || set_ref_in_cpt_callbacks(cpt_cb.get(), cpt_cb_count.get(), copyID);
        abort
    }
}

/// The user-defined completion function name for completion `type_0`.
pub(crate) unsafe fn get_complete_funcname(type_0: c_int) -> *mut c_char {
    unsafe {
        match type_0 {
            CTRL_X_FUNCTION => (*curbuf.get()).b_p_cfu,
            CTRL_X_OMNI => (*curbuf.get()).b_p_ofu,
            CTRL_X_THESAURUS => {
                if *(*curbuf.get()).b_p_tsrfu as c_int == NUL {
                    p_tsrfu.get()
                } else {
                    (*curbuf.get()).b_p_tsrfu
                }
            }
            _ => c"".as_ptr().cast_mut(),
        }
    }
}

/// The callback to use for insert-mode completion of `type_0`.
pub(crate) unsafe fn get_insert_callback(type_0: c_int) -> *mut Callback {
    unsafe {
        if type_0 == CTRL_X_FUNCTION {
            return &raw mut (*curbuf.get()).b_cfu_cb;
        }
        if type_0 == CTRL_X_OMNI {
            return &raw mut (*curbuf.get()).b_ofu_cb;
        }
        // CTRL_X_THESAURUS
        if *(*curbuf.get()).b_p_tsrfu as c_int != NUL {
            &raw mut (*curbuf.get()).b_tsrfu_cb
        } else {
            tsrfu_cb.ptr()
        }
    }
}

/// Call `'completefunc'`, `'omnifunc'` or `'thesaurusfunc'` and add whatever
/// it answers to the match list.
///
/// `type_0` is one of `CTRL_X_OMNI`, `CTRL_X_FUNCTION` or `CTRL_X_THESAURUS`;
/// `cb` is set when a function in `'complete'` triggered this, null otherwise.
pub(crate) unsafe fn expand_by_function(type_0: c_int, base: *mut c_char, mut cb: *mut Callback) {
    unsafe {
        debug_assert!(!curbuf.get().is_null());

        let is_cpt_function = !cb.is_null();
        if !is_cpt_function {
            if *get_complete_funcname(type_0) as c_int == NUL {
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

        let mut matchlist: *mut list_T = ptr::null_mut();
        let mut matchdict: *mut dict_T = ptr::null_mut();
        let mut rettv = TYPVAL_T_INIT;
        let save_State = State.get();
        let pos = (*curwin.get()).w_cursor;

        // Lock the text to avoid weird things from happening.  Also disallow
        // switching to another window: it should not be needed and may end up
        // in Insert mode in another buffer.
        (*textlock.ptr()) += 1;
        if callback_call(cb, 2, args.as_mut_ptr(), &raw mut rettv) {
            match rettv.v_type {
                VAR_LIST => matchlist = rettv.vval.v_list,
                VAR_DICT => matchdict = rettv.vval.v_dict,
                // VAR_SPECIAL falls through to the default.
                // TODO(brammool): Give error message?
                _ => tv_clear(&raw mut rettv),
            }
        }
        (*textlock.ptr()) -= 1;

        (*curwin.get()).w_cursor = pos; // restore the cursor position
        check_cursor(curwin.get()); // make sure the position is valid, just in case
        validate_cursor(curwin.get());
        if !equalpos((*curwin.get()).w_cursor, pos) {
            emsg(gettext(E_COMPLDEL.as_ptr()));
        } else if !matchlist.is_null() {
            ins_compl_add_list(matchlist);
        } else if !matchdict.is_null() {
            ins_compl_add_dict(matchdict);
        }

        // Restore State, it might have been changed.
        State.set(save_State);
        if !matchdict.is_null() {
            tv_dict_unref(matchdict);
        }
        if !matchlist.is_null() {
            tv_list_unref(matchlist);
        }
    }
}

/// The attribute of the named highlight group, or `-1` for no name.
#[inline]
pub(crate) unsafe fn get_user_highlight_attr(hlname: *const c_char) -> c_int {
    unsafe {
        if !hlname.is_null() && *hlname as c_int != NUL {
            return syn_name2attr(hlname);
        }
        -1
    }
}

/// The callback `p` names if it refers to a user-defined function in
/// `'complete'`; `idx` indexes the callback array.
pub(crate) unsafe fn get_callback_if_cpt_func(mut p: *mut c_char, idx: c_int) -> *mut Callback {
    unsafe {
        if *p as c_int == 'o' as c_int {
            return &raw mut (*curbuf.get()).b_ofu_cb;
        }
        if *p as c_int == 'F' as c_int {
            p = p.offset(1);
            if *p as c_int != ',' as c_int && *p as c_int != NUL {
                // 'F{func}' case.
                let slot = (*curbuf.get()).b_p_cpt_cb.offset(idx as isize);
                return if (*slot).type_0 != kCallbackNone {
                    slot
                } else {
                    ptr::null_mut()
                };
            }
            return &raw mut (*curbuf.get()).b_cfu_cb; // 'cfu'
        }
        ptr::null_mut()
    }
}

/// Call the functions named in `'complete'` with `findstart=1` and record the
/// start column each answers.
pub(crate) unsafe fn prepare_cpt_compl_funcs() {
    unsafe {
        // Make a copy of 'cpt' in case the buffer gets wiped out.
        let cpt = xstrdup((*curbuf.get()).b_p_cpt);
        strip_caret_numbers_in_place(cpt);

        let mut idx: isize = 0;
        let mut p = cpt;
        while *p != 0 {
            p = skip_cpt_delims(p);
            if *p as c_int == NUL {
                break;
            }

            let source = (*cpt_sources_array.ptr()).offset(idx);
            let cb = get_callback_if_cpt_func(p, idx as c_int);
            if cb.is_null() {
                (*source).cs_startcol = -3;
            } else {
                let mut startcol = 0;
                if get_userdefined_compl_info((*curwin.get()).w_cursor.col, cb, &raw mut startcol)
                    == FAIL
                {
                    if startcol == -3 {
                        (*source).cs_refresh_always = false;
                    } else {
                        startcol = -2;
                    }
                } else if startcol < 0 || startcol > (*curwin.get()).w_cursor.col {
                    startcol = (*curwin.get()).w_cursor.col;
                }
                (*source).cs_startcol = startcol;
            }

            // Advance p.
            copy_option_part(
                &raw mut p,
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as size_t,
                c",".as_ptr().cast_mut(),
            );
            idx += 1;
        }
        xfree(cpt.cast::<c_void>());
    }
}

/// Advance `cpt_sources_index` by one, or report E684 and fail.
pub(crate) unsafe fn advance_cpt_sources_index_safe() -> c_int {
    unsafe {
        if cpt_sources_index.get() >= 0 && cpt_sources_index.get() < cpt_sources_count.get() - 1 {
            (*cpt_sources_index.ptr()) += 1;
            return OK;
        }
        semsg_c!(
            gettext(&raw const e_list_index_out_of_range_nr as *const c_char),
            cpt_sources_index.get(),
        );
        FAIL
    }
}

/// Reset the info associated with the completion sources.
pub(crate) unsafe fn cpt_sources_clear() {
    unsafe {
        xfree(cpt_sources_array.get().cast::<c_void>());
        cpt_sources_array.set(ptr::null_mut());
        cpt_sources_index.set(-1);
        cpt_sources_count.set(0);
    }
}

/// Build the per-`'complete'`-entry state: the source letter and its `^N`
/// max-matches limit.
pub(crate) unsafe fn setup_cpt_sources() {
    unsafe {
        cpt_sources_clear();

        let count = get_cpt_sources_count();
        if count == 0 {
            return;
        }
        cpt_sources_array
            .set(xcalloc(count as size_t, size_of::<cpt_source_T>()).cast::<cpt_source_T>());

        let mut part = [0 as c_char; LSIZE as usize];
        let mut idx: isize = 0;
        let mut p = (*curbuf.get()).b_p_cpt;
        while *p != 0 {
            p = skip_cpt_delims(p);
            if *p != 0 {
                // If not end of string, count this segment.
                let source = (*cpt_sources_array.ptr()).offset(idx);
                (*source).cs_flag = *p;
                part.fill(0);
                // Advance p.
                let slen = copy_option_part(
                    &raw mut p,
                    part.as_mut_ptr(),
                    LSIZE as size_t,
                    c",".as_ptr().cast_mut(),
                );
                if slen > 0 {
                    let caret = vim_strchr(part.as_mut_ptr(), '^' as c_int);
                    if !caret.is_null() {
                        (*source).cs_max_matches = atoi(caret.offset(1));
                    }
                }
                idx += 1;
            }
        }
        cpt_sources_count.set(count);
    }
}

/// Whether any completion source has `refresh` set to `always`.
pub(crate) unsafe fn is_cpt_func_refresh_always() -> bool {
    unsafe {
        (0..cpt_sources_count.get() as isize)
            .any(|i| (*(*cpt_sources_array.ptr()).offset(i)).cs_refresh_always)
    }
}

/// Collect matches through `cb` and record its `refresh:always` flag.
pub(crate) unsafe fn get_cpt_func_completion_matches(cb: *mut Callback) {
    unsafe {
        let cpt_src = (*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize);
        let startcol = (*cpt_src).cs_startcol;
        if startcol == -2 || startcol == -3 {
            return;
        }

        set_compl_globals(startcol, (*curwin.get()).w_cursor.col, true);

        // Insert the leader string (previously removed) before expansion.
        // This prevents flicker when `func` (e.g. an LSP client) is slow and
        // calls 'sleep', which triggers ui_flush().
        if !(*cpt_src).cs_refresh_always {
            ins_compl_insert_bytes(ins_compl_leader(), -1);
        }

        expand_by_function(0, (*cpt_compl_pattern.ptr()).data(), cb);

        if !(*cpt_src).cs_refresh_always {
            ins_compl_delete(false);
        }

        (*cpt_src).cs_refresh_always = compl_opt_refresh_always.get();
        compl_opt_refresh_always.set(false);
    }
}

/// Re-collect matches from the `'complete'` functions that set
/// `refresh:always`.
pub(crate) unsafe fn cpt_compl_refresh() {
    unsafe {
        // Make the completion list linear (non-cyclic).
        ins_compl_make_linear();
        // Make a copy of 'cpt' in case the buffer gets wiped out.
        let cpt = xstrdup((*curbuf.get()).b_p_cpt);
        strip_caret_numbers_in_place(cpt);

        cpt_sources_index.set(0);
        let mut p = cpt;
        while *p != 0 {
            p = skip_cpt_delims(p);
            if *p as c_int == NUL {
                break;
            }

            let source = (*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize);
            if (*source).cs_refresh_always {
                let cb = get_callback_if_cpt_func(p, cpt_sources_index.get());
                if !cb.is_null() {
                    remove_old_matches();
                    let mut startcol = 0;
                    let ret = get_userdefined_compl_info(
                        (*curwin.get()).w_cursor.col,
                        cb,
                        &raw mut startcol,
                    );
                    if ret == FAIL {
                        if startcol == -3 {
                            (*source).cs_refresh_always = false;
                        } else {
                            startcol = -2;
                        }
                    } else if startcol < 0 || startcol > (*curwin.get()).w_cursor.col {
                        startcol = (*curwin.get()).w_cursor.col;
                    }
                    (*source).cs_startcol = startcol;
                    if ret == OK {
                        compl_source_start_timer(cpt_sources_index.get());
                        get_cpt_func_completion_matches(cb);
                    }
                }
            }

            // Advance p.
            copy_option_part(
                &raw mut p,
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as size_t,
                c",".as_ptr().cast_mut(),
            );
            if may_advance_cpt_index(p) {
                advance_cpt_sources_index_safe();
            }
        }
        cpt_sources_index.set(-1);

        xfree(cpt.cast::<c_void>());
        // Make the list cyclic.
        compl_matches.set(ins_compl_make_cyclic());
    }
}
