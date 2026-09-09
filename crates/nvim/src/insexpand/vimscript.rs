//! The Vimscript face: `complete()`, `complete_info()`, `CompleteDone`.
//!
//! [`set_completion`] is `complete()`; [`ins_compl_add_tv`] turns one list
//! entry — a string or a dict with `word`/`abbr`/`menu`/`info`/`kind` — into
//! a match.  [`get_complete_info`] answers `complete_info()`, and
//! [`do_autocmd_completedone`] fires `CompleteDone` with
//! `v:completed_item`.
//!
//! Every `tv_dict_*` key here is an ordinary Rust `&str`: those functions copy
//! exactly the length they are given, so upstream's `S_LEN(key)` is
//! `key.as_ptr(), key.len()` and the transpile's `b"key\0"` plus
//! `size_of::<[c_char; N]>() - 1` goes away entirely.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::{NumBuf, tv_dict_get_string_alloc};
use crate::guard::Allow;
use crate::keycodes::{Ctrl_E, Ctrl_N, Ctrl_Y, Key};
use crate::types::{
    FAIL, NUL, OK, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VarLock, kListLenMayKnow,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;

/// Fire `CompleteDone` with `v:event` describing how the completion ended.
///
/// # Safety
///
/// `word` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn do_autocmd_completedone(c: c_int, mode: c_int, word: *mut c_char) {
    let mut save_v_event = SAVE_V_EVENT_INIT;
    let v_event = unsafe { get_v_event(&raw mut save_v_event) };
    let add_str = |key: &str, val: *const c_char| unsafe {
        tv_dict_add_str(v_event, key.as_ptr().cast(), key.len(), val)
    };

    let mode_str = match CTRL_X_MODE_NAMES[(mode & !CTRL_X_WANT_IDENT) as usize] {
        Some(name) => name.as_ptr(),
        None => c"".as_ptr(),
    };
    let _ = add_str(
        "complete_word",
        if word.is_null() { c"".as_ptr() } else { word },
    );
    let _ = add_str("complete_type", mode_str);
    let _ = add_str(
        "reason",
        if c == Ctrl_Y || !word.is_null() {
            c"accept".as_ptr()
        } else if c == Ctrl_E {
            c"cancel".as_ptr()
        } else {
            c"discard".as_ptr()
        },
    );
    unsafe { tv_dict_set_keys_readonly(v_event) };

    unsafe { ins_apply_autocmds(AutoEvent::CompleteDone) };
    unsafe { restore_v_event(v_event, &raw mut save_v_event) };
}

/// One match as a locked `v:completed_item` dict.
///
/// # Safety
///
/// `match_0` must point at a live `ComplItem`, unaliased for the call.
pub(crate) unsafe fn ins_compl_dict_alloc(match_0: *mut ComplItem) -> *mut Dict {
    // { word, abbr, menu, kind, info, user_data } — the same keys and the
    // same order `complete_info()` fills in, minus its "match" flag.
    let dict = unsafe { tv_dict_alloc_lock(VarLock::Fixed) };
    unsafe { fill_complete_info_dict(dict, match_0, false) };
    dict
}

/// Add one match given as a Vimscript value: a string, or a dict with
/// `word`/`abbr`/`menu`/`kind`/`info` and the option keys.
///
/// `fast` uses `fast_breakcheck()` instead of `os_breakcheck()`. Answers
/// NOTDONE if the string is already in the list, OK if it was added, FAIL on
/// error.
///
/// # Safety
///
/// `tv` must point at an initialized typval, unaliased for the call.
pub(crate) unsafe fn ins_compl_add_tv(tv: *const TypVal, dir: Direction, fast: bool) -> c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let word: *const c_char;
    let mut dup = false;
    let mut empty = false;
    let mut flags = if fast { CP_FAST } else { 0 };
    let mut cptext: [*mut c_char; CPT_COUNT as usize] = [ptr::null_mut(); CPT_COUNT as usize];
    let mut user_hl: [c_int; 2] = [-1, -1];
    let mut user_data = TYPVAL_T_INIT;

    if unsafe { (*tv).v_type() } == VAR_DICT && !unsafe { (*tv).dict_or_null() }.is_null() {
        let d = unsafe { (*tv).dict_or_null() };
        // The four cptext strings are copied and owned by the match from
        // here on; the two highlight names and `word` are borrowed, so
        // each borrowing answer renders into a scratch of its own —
        // `word` outlives all of them.
        let borrowed = |key: &CStr, b: &mut NumBuf| unsafe { b.dict_string(d, key.as_ptr()) };
        let get_nr = |key: &CStr| unsafe { tv_dict_get_number(d, key.as_ptr()) };

        word = borrowed(c"word", &mut numbuf);
        cptext[CPT_ABBR as usize] = unsafe { tv_dict_get_string_alloc(d, c"abbr".as_ptr()) };
        cptext[CPT_MENU as usize] = unsafe { tv_dict_get_string_alloc(d, c"menu".as_ptr()) };
        cptext[CPT_KIND as usize] = unsafe { tv_dict_get_string_alloc(d, c"kind".as_ptr()) };
        cptext[CPT_INFO as usize] = unsafe { tv_dict_get_string_alloc(d, c"info".as_ptr()) };

        user_hl[0] = unsafe { get_user_highlight_attr(borrowed(c"abbr_hlgroup", &mut numbuf2)) };
        user_hl[1] = unsafe { get_user_highlight_attr(borrowed(c"kind_hlgroup", &mut numbuf2)) };

        let _ = unsafe { tv_dict_get_tv(d, c"user_data".as_ptr(), &raw mut user_data) };

        if get_nr(c"icase") != 0 {
            flags |= CP_ICASE;
        }
        dup = get_nr(c"dup") != 0;
        empty = get_nr(c"empty") != 0;
        if !borrowed(c"equal", &mut numbuf2).is_null() && get_nr(c"equal") != 0 {
            flags |= CP_EQUAL;
        }
    } else {
        word = unsafe { numbuf.string_chk(tv) };
    }

    if word.is_null() || (!empty && unsafe { *word } as c_int == NUL) {
        unsafe { free_cptext(cptext.as_ptr()) };
        unsafe { tv_clear(&raw mut user_data) };
        return FAIL;
    }

    let (text, cpt) = (word.cast_mut(), cptext.as_ptr());
    let (data, hl) = (&raw mut user_data, user_hl.as_ptr());
    let (none, score) = (ptr::null_mut(), FUZZY_SCORE_NONE);
    // SAFETY: `text` is NUL-terminated (`len < 0`), `cpt` is the four
    // allocated strings this call hands over, `data` and `hl` are this
    // frame's own locals, and there is no file name.
    let status =
        unsafe { ins_compl_add(text, -1, none, cpt, true, data, dir, flags, dup, hl, score) };
    // Anything but `OK` leaves the value with this frame -- `NOTDONE` (the
    // word was already in the list) included, which the transpile read as
    // success and leaked.
    if status != OK {
        unsafe { tv_clear(&raw mut user_data) };
    }
    status
}

/// Add every entry of `list` as a match.
///
/// # Safety
///
/// `list` must point at a live list, unaliased for the call.
pub(crate) unsafe fn ins_compl_add_list(list: *mut List) {
    let mut dir = compl_direction.get();
    if list.is_null() {
        return;
    }
    let mut li = unsafe { (*list).lv_first };
    while !li.is_null() {
        if unsafe { ins_compl_add_tv(&raw mut (*li).li_tv, dir, true) } == OK {
            // If dir was BACKWARD then honour it just once.
            dir = FORWARD;
        } else if did_emsg.get() != 0 {
            break;
        }
        li = unsafe { (*li).li_next };
    }
}

/// Add the matches a `'completefunc'`-style dict answers, and note its
/// optional `refresh` item.
///
/// # Safety
///
/// `dict` must point at a live dictionary, unaliased for the call.
pub(crate) unsafe fn ins_compl_add_dict(dict: *mut Dict) {
    let find =
        |key: &str| unsafe { tv_dict_find(dict, key.as_ptr().cast(), key.len() as ptrdiff_t) };

    // Check for the optional "refresh" item.
    compl_opt_refresh_always.set(false);
    let di_refresh = find("refresh");
    if !di_refresh.is_null() && unsafe { (*di_refresh).di_tv.v_type() } == VAR_STRING {
        let v = unsafe { (*di_refresh).di_tv.string_or_null() };
        if !v.is_null() && unsafe { cstr::eq_bytes(v, b"always") } {
            compl_opt_refresh_always.set(true);
        }
    }

    // Add completions from a "words" list.
    let di_words = find("words");
    if !di_words.is_null() && unsafe { (*di_words).di_tv.v_type() } == VAR_LIST {
        unsafe { ins_compl_add_list((*di_words).di_tv.list_or_null()) };
    }
}

/// The extmarks that were sitting on `compl_orig_text`, kept so they can go
/// back when the completion is cancelled or the original text is completed.
///
/// The list is a `kvec` -- `extmark_splice_delete` pushes onto it through
/// `xrealloc`, so it stays C-shaped -- but the allocation is the
/// completion's own, and upstream frees it by hand at three sites with
/// `kv_destroy` written out.  `ComplOrigExtmarks` names the cell rather than
/// pointing into it, so it is `Copy`, needs no `unsafe` to make, and is the
/// single owner of the buffer: the address is produced only inside
/// [`save`](Self::save), and only for the length of that one call.
#[derive(Clone, Copy)]
pub(crate) struct ComplOrigExtmarks(());

/// The extmarks saved over the original text. See [`ComplOrigExtmarks`].
pub(crate) fn compl_orig_extmarks() -> ComplOrigExtmarks {
    ComplOrigExtmarks(())
}

impl ComplOrigExtmarks {
    /// Save the extmarks over the text `compl_col`/`compl_length` covers,
    /// invalidating them in the buffer.
    ///
    /// # Safety
    /// The cursor's line must be live, and `compl_col`/`compl_length` must
    /// describe a range inside it.
    pub(crate) unsafe fn save(self) {
        // The list is handed over by address because `kv_push` reallocates
        // it; a local stands in for the cell so nothing else can see it
        // half-grown, and `splice_delete` runs no editor code that could
        // look.
        let mut saved = COMPL_ORIG_EXTMARKS.get();
        // SAFETY: the caller's promise; `saved` is a live vector.
        let lnum = Win::current().w_cursor.lnum as c_int - 1;
        let buf = Buf::current();
        let (start, end) = (compl_col.get(), compl_col.get() + compl_length.get());
        let list = &raw mut saved;
        // SAFETY: the caller's promise -- `start .. end` is a range of the
        // cursor line -- and `list` is the local standing in for the cell.
        unsafe { extmark_splice_delete(buf, lnum, start, lnum, end, list, true, kExtmarkUndo) };
        COMPL_ORIG_EXTMARKS.set(saved);
    }

    /// Put the saved extmarks back, newest first.
    ///
    /// # Safety
    /// The buffer they were taken from must still be current.
    pub(crate) unsafe fn restore(self) {
        // The count is read once and the buffer once per step, exactly as
        // upstream's `for (i = kv_size(v); i > 0; i--) kv_A(v, i - 1)` did:
        // `extmark_apply_undo` re-enters the marktree, and nothing there
        // pushes onto this list, but nothing here assumes that either.
        for i in (0..COMPL_ORIG_EXTMARKS.get().size as isize).rev() {
            // SAFETY: `i` is within the list's own `size` undo objects, and
            // the buffer they name is still current (the caller's promise).
            unsafe { extmark_apply_undo(*COMPL_ORIG_EXTMARKS.get().items.offset(i), true) };
        }
    }

    /// C's `kv_destroy(compl_orig_extmarks)`.
    pub(crate) fn clear(self) {
        let saved = COMPL_ORIG_EXTMARKS.replace(EXTMARK_UNDO_VEC_INIT);
        // SAFETY: the buffer is this owner's own, and `xfree` takes null.
        unsafe { xfree(saved.items.cast::<c_void>()) };
    }
}

/// Start the completion `complete()` describes: `startcol` is where the
/// matched text starts (1 is the first column) and `list` holds the matches.
///
/// # Safety
///
/// `list` must point at a live list, unaliased for the call.
pub(crate) unsafe fn set_completion(mut startcol: ColNr, list: *mut List) {
    let cur_cot_flags = get_cot_flags();
    let compl_longest = cur_cot_flags & kOptCotFlagLongest as c_uint != 0;
    let compl_no_insert = cur_cot_flags & kOptCotFlagNoinsert as c_uint != 0;
    let compl_no_select = cur_cot_flags & kOptCotFlagNoselect as c_uint != 0;

    // If already doing completions stop it.
    if ctrl_x_mode_not_default() {
        ins_compl_prep(' ' as c_int);
    }
    unsafe { ins_compl_clear() };
    unsafe { ins_compl_free() };
    compl_get_longest.set(compl_longest);

    compl_direction.set(FORWARD);
    if startcol > Win::current().w_cursor.col {
        startcol = Win::current().w_cursor.col;
    }
    compl_col.set(startcol);
    compl_lnum.set(Win::current().w_cursor.lnum);
    compl_length.set(Win::current().w_cursor.col - startcol);
    // compl_pattern doesn't need to be set.
    // SAFETY: `compl_col`/`compl_length` were just set to a range of the
    // cursor line.
    let orig = unsafe { compl_text_from_line(get_cursor_line_ptr()) };
    compl_orig_text().set(orig);
    unsafe { compl_orig_extmarks().save() };

    let mut flags = CP_ORIGINAL_TEXT;
    if p_ic.get() != 0 {
        flags |= CP_ICASE;
    }
    // SAFETY: `compl_orig_text` is the text being completed.
    if unsafe { ins_compl_add_orig_text(flags | CP_FAST) }.is_err() {
        return;
    }

    ctrl_x_mode.set(CTRL_X_EVAL);

    unsafe { ins_compl_add_list(list) };
    compl_matches.set(ins_compl_make_cyclic());
    compl_started.set(true);
    compl_used_match.set(true);
    compl_cont_status.set(0);
    let save_w_wrow = Win::current().w_wrow;
    let save_w_leftcol = Win::current().w_leftcol;

    compl_curr_match.set(compl_first_match.get());
    let no_select = compl_no_select || compl_longest;
    if compl_no_insert || no_select {
        let _ = ins_complete(Key::Down.code(), false);
        if no_select {
            let _ = ins_complete(Key::Up.code(), false);
        }
    } else {
        let _ = ins_complete(Ctrl_N, false);
    }
    compl_enter_selects.set(compl_no_insert);

    // Lazily show the popup menu, unless we got interrupted.
    if !compl_interrupted.get() {
        show_pum(save_w_wrow, save_w_leftcol);
    }

    unsafe { may_trigger_modechanged() };
    unsafe { ui_flush() };
}

/// The `complete()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if State.get() & MODE_INSERT == 0 {
        emsg(gettext(c"E785: complete() can only be used in Insert mode"));
        return;
    }

    // Check for undo allowed here, because if something was already
    // inserted the line was already saved for undo and this check isn't
    // done.
    if !undo_allowed(Buf::current()) {
        return;
    }

    if args[1].v_type() != VAR_LIST {
        emsg(gettext(e_invarg));
    } else {
        let startcol = unsafe { tv_get_number_chk(&args[0], ptr::null_mut()) } as ColNr;
        if startcol > 0 {
            unsafe { set_completion(startcol - 1, args[1].list_or_null()) };
        }
    }
}

/// The `complete_add()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_add(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    unsafe {
        (*result).write_number(ins_compl_add_tv(&args[0], kDirectionNotSet, false) as VarNumber)
    };
}

/// The `complete_check()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_check(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _redraw = Allow::redraw();
    ins_compl_check_keys(0, true);
    result.write_number(ins_compl_interrupted() as VarNumber);
}

/// Fill `di` with one match, as `complete_info()` reports it.
///
/// # Safety
///
/// `di` must point at a live dictionary, unaliased for the call. `match_0`
/// must point at a live `ComplItem`, unaliased for the call.
pub(crate) unsafe fn fill_complete_info_dict(
    di: *mut Dict,
    match_0: *mut ComplItem,
    add_match: bool,
) {
    let add_str = |key: &str, val: *const c_char| unsafe {
        tv_dict_add_str(di, key.as_ptr().cast(), key.len(), val)
    };

    let _ = add_str("word", unsafe { (*match_0).cp_str }.data());
    let _ = add_str("abbr", unsafe { (*match_0).cp_text[CPT_ABBR as usize] });
    let _ = add_str("menu", unsafe { (*match_0).cp_text[CPT_MENU as usize] });
    let _ = add_str("kind", unsafe { (*match_0).cp_text[CPT_KIND as usize] });
    let _ = add_str("info", unsafe { (*match_0).cp_text[CPT_INFO as usize] });
    if add_match {
        // SAFETY: `match_0` is a live match.
        let in_array = unsafe { (*match_0).cp_in_match_array } as BoolVarValue;
        let (key, klen) = ("match".as_ptr().cast(), "match".len());
        // SAFETY: `di` is the dict being built and `key` a static name.
        let _ = unsafe { tv_dict_add_bool(di, key, klen, in_array) };
    }
    if unsafe { (*match_0).cp_user_data.v_type() } == VAR_UNKNOWN {
        // Add an empty string for backwards compatibility.
        let _ = add_str("user_data", c"".as_ptr());
    } else {
        let (key, klen) = ("user_data".as_ptr().cast(), "user_data".len());
        // SAFETY: `di` is the dict being built, and the value is the address
        // of one of the live match's fields, taken from its raw pointer.
        let _ = unsafe { tv_dict_add_tv(di, key, klen, &raw mut (*match_0).cp_user_data) };
    }
}

/// Fill `retdict` with whatever of `complete_info()` `what_list` asked for.
///
/// # Safety
///
/// `what_list` must point at a live list, unaliased for the call. `retdict`
/// must point at a live dictionary, unaliased for the call.
pub(crate) unsafe fn get_complete_info(what_list: *mut List, retdict: *mut Dict) {
    let mut numbuf = NumBuf::new();
    let add_nr = |key: &str, val: VarNumber| unsafe {
        tv_dict_add_nr(retdict, key.as_ptr().cast(), key.len(), val)
    };

    let mut what_flag;
    if what_list.is_null() {
        what_flag = CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
    } else {
        what_flag = 0;
        let mut item = unsafe { tv_list_first(what_list) };
        while !item.is_null() {
            // `tv_get_string` answers "" rather than NULL for anything it
            // cannot render, so this is never a null pointer.
            let what = unsafe { CStr::from_ptr(numbuf.string(&raw mut (*item).li_tv)) };
            what_flag |= match what.to_bytes() {
                b"mode" => CI_WHAT_MODE,
                b"pum_visible" => CI_WHAT_PUM_VISIBLE,
                b"items" => CI_WHAT_ITEMS,
                b"selected" => CI_WHAT_SELECTED,
                b"completed" => CI_WHAT_COMPLETED,
                b"preinserted_text" => CI_WHAT_PREINSERTED_TEXT,
                b"matches" => CI_WHAT_MATCHES,
                _ => 0,
            };
            item = unsafe { (*item).li_next };
        }
    }

    let mut ret = Ok(());
    if what_flag & CI_WHAT_MODE != 0 {
        let (key, klen) = ("mode".as_ptr().cast(), "mode".len());
        // SAFETY: `retdict` is the dict being built and `ins_compl_mode`
        // answers a NUL-terminated static name.
        ret = unsafe { tv_dict_add_str(retdict, key, klen, ins_compl_mode()) };
    }

    if ret.is_ok() && what_flag & CI_WHAT_PUM_VISIBLE != 0 {
        ret = add_nr("pum_visible", pum_visible() as VarNumber);
    }

    if ret.is_ok() && what_flag & CI_WHAT_PREINSERTED_TEXT != 0 {
        let line = get_cursor_line_ptr();
        let len = compl_ins_end_col.get() - Win::current().w_cursor.col;
        let text = if len > 0 {
            // SAFETY: the cursor column is inside the cursor line.
            unsafe { line.offset(Win::current().w_cursor.col as isize) }
        } else {
            c"".as_ptr()
        };
        let (key, klen) = ("preinserted_text".as_ptr().cast(), "preinserted_text".len());
        // SAFETY: `text` is readable for `len` bytes.
        ret = unsafe { tv_dict_add_str_len(retdict, key, klen, text, len.max(0)) };
    }

    if ret.is_err()
        || what_flag & (CI_WHAT_ITEMS | CI_WHAT_SELECTED | CI_WHAT_MATCHES | CI_WHAT_COMPLETED) == 0
    {
        return;
    }

    let mut li: *mut List = ptr::null_mut();
    let mut selected_idx = -1;
    let has_items = what_flag & CI_WHAT_ITEMS != 0;
    let has_matches = what_flag & CI_WHAT_MATCHES != 0;
    let has_completed = what_flag & CI_WHAT_COMPLETED != 0;
    if has_items || has_matches {
        li = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        let key = if has_matches && !has_items {
            "matches"
        } else {
            "items"
        };
        ret = unsafe { tv_dict_add_list(retdict, key.as_ptr().cast(), key.len(), li) };
    }
    if ret.is_ok()
        && what_flag & CI_WHAT_SELECTED != 0
        && !compl_curr_match.get().is_null()
        && unsafe { (*compl_curr_match.get()).cp_number } == -1
    {
        ins_compl_update_sequence_numbers();
    }
    if ret.is_ok() {
        let mut list_idx = 0;
        for match_0 in matches_from(first_match()) {
            if match_0.is_original() {
                continue;
            }
            if has_items || (has_matches && match_0.cp_in_match_array) {
                // SAFETY: a fresh dict, taken over by the list, and
                // `match_0` is a live match.
                unsafe {
                    let di = tv_dict_alloc();
                    tv_list_append_dict(li, di);
                    fill_complete_info_dict(di, match_0.raw(), has_matches && has_items);
                }
            }
            if curr_match().is_some_and(|curr| curr.cp_number == match_0.cp_number) {
                selected_idx = list_idx;
            }
            if !has_matches || match_0.cp_in_match_array {
                list_idx += 1;
            }
        }
    }
    if ret.is_ok() && what_flag & CI_WHAT_SELECTED != 0 {
        ret = add_nr("selected", selected_idx as VarNumber);
        if let Some(wp) = win_float_find_preview() {
            let _ = add_nr("preview_winid", wp.handle as VarNumber);
            let _ = add_nr("preview_bufnr", wp.buffer().handle as VarNumber);
        }
    }
    if ret.is_ok() && selected_idx != -1 && has_completed {
        let di = unsafe { tv_dict_alloc() };
        unsafe { fill_complete_info_dict(di, compl_curr_match.get(), false) };
        let (key, klen) = ("completed".as_ptr().cast(), "completed".len());
        let _ = unsafe { tv_dict_add_dict(retdict, key, klen, di) };
    }
}

/// The `complete_info()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_info(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    unsafe { tv_dict_alloc_ret(result) };

    let mut what_list: *mut List = ptr::null_mut();
    if !args.is_empty() {
        if !args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
            emsg(gettext(e_listreq));
            return;
        }
        what_list = args[0].list_or_null();
    }
    unsafe { get_complete_info(what_list, (*result).dict_or_null()) };
}
