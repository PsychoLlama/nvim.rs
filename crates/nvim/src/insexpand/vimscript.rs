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
use crate::eval::typval::{DictRef, NumBuf, dict_get_string_buf, list_items, list_iter};
use crate::guard::Allow;
use crate::keycodes::{Ctrl_E, Ctrl_N, Ctrl_Y, Key};
use crate::types::{
    FAIL, NUL, OK, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VarLock, kListLenMayKnow,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;

/// Fire `CompleteDone` with `v:event` describing how the completion ended:
/// `word` is the match accepted, if any.
pub(crate) fn do_autocmd_completedone(c: c_int, mode: c_int, word: Option<&CStr>) {
    let mut save_v_event = SAVE_V_EVENT_INIT;
    // SAFETY: `save_v_event` is this frame's, and lives until the restore
    // below hands the saved dict back.
    let v_event = unsafe { get_v_event(&raw mut save_v_event) };
    // SAFETY: `v_event` is the dict just built, and every value is a
    // NUL-terminated string.
    let add_str =
        |key: &str, val: &CStr| unsafe { (*v_event).add_str(key.as_bytes(), val.as_ptr()) };

    let mode_name = CTRL_X_MODE_NAMES[(mode & !CTRL_X_WANT_IDENT) as usize].unwrap_or(c"");
    let _ = add_str("complete_word", word.unwrap_or(c""));
    let _ = add_str("complete_type", mode_name);
    let reason = if c == Ctrl_Y || word.is_some() {
        c"accept"
    } else if c == Ctrl_E {
        c"cancel"
    } else {
        c"discard"
    };
    let _ = add_str("reason", reason);
    // SAFETY: as above.
    unsafe { (*v_event).set_keys_readonly() };

    ins_apply_autocmds(AutoEvent::CompleteDone);
    // SAFETY: the pair of `get_v_event`, with the same saved slot.
    unsafe { restore_v_event(v_event, &raw mut save_v_event) };
}

/// One match as a locked `v:completed_item` dict.
pub(crate) fn ins_compl_dict_alloc(m: MatchId) -> DictRef {
    // { word, abbr, menu, kind, info, user_data } — the same keys and the
    // same order `complete_info()` fills in, minus its "match" flag.
    let mut dict = tv_dict_alloc_lock(VarLock::Fixed);
    fill_complete_info_dict(&mut dict, m, false);
    dict
}

/// Add one match given as a Vimscript value: a string, or a dict with
/// `word`/`abbr`/`menu`/`kind`/`info` and the option keys.
///
/// `fast` uses `fast_breakcheck()` instead of `os_breakcheck()`. Answers
/// NOTDONE if the string is already in the list, OK if it was added, FAIL on
/// error.
pub(crate) fn ins_compl_add_tv(tv: &TypVal, dir: Direction, fast: bool) -> c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let word: *const c_char;
    let mut dup = false;
    let mut empty = false;
    let mut flags = if fast { CP_FAST } else { 0 };
    let mut extra = NO_EXTRA;
    let mut user_hl = NO_HL;
    let mut user_data = TYPVAL_T_INIT;

    if (*tv).v_type() == VAR_DICT && (*tv).dict_ref().is_some() {
        // The four extra strings are copied and owned by the match from
        // here on; the two highlight names and `word` are borrowed, so
        // each borrowing answer renders into a scratch of its own —
        // `word` outlives all of them.
        let d = (*tv).dict_ref();
        let borrowed = |key: &CStr, b: &mut NumBuf| b.dict_string(d, key.to_bytes());
        let get_nr = |key: &CStr| dict_get_number(d, key.to_bytes());
        let owned = |key: &[u8]| {
            let mut scratch = NumBuf::new();
            let text = dict_get_string_buf(d, key, &mut scratch);
            // SAFETY: a non-null answer is a NUL-terminated string.
            (!text.is_null()).then(|| XString::from_cstr(unsafe { cstr::at(text) }))
        };

        word = borrowed(c"word", &mut numbuf);
        extra[CPT_ABBR as usize] = owned(b"abbr");
        extra[CPT_MENU as usize] = owned(b"menu");
        extra[CPT_KIND as usize] = owned(b"kind");
        extra[CPT_INFO as usize] = owned(b"info");

        user_hl[0] = unsafe { get_user_highlight_attr(borrowed(c"abbr_hlgroup", &mut numbuf2)) };
        user_hl[1] = unsafe { get_user_highlight_attr(borrowed(c"kind_hlgroup", &mut numbuf2)) };

        let _ = dict_get_tv(d, b"user_data", &mut user_data);

        if get_nr(c"icase") != 0 {
            flags |= CP_ICASE;
        }
        dup = get_nr(c"dup") != 0;
        empty = get_nr(c"empty") != 0;
        if !borrowed(c"equal", &mut numbuf2).is_null() && get_nr(c"equal") != 0 {
            flags |= CP_EQUAL;
        }
    } else {
        word = numbuf.string_ptr_chk(tv);
    }

    if word.is_null() || (!empty && unsafe { *word } as c_int == NUL) {
        return FAIL;
    }

    // SAFETY: a non-null word is a NUL-terminated string, borrowed from the
    // value or rendered into `numbuf`, both of which outlive the add.
    let text = unsafe { cstr::bytes_at(word) };
    let score = FUZZY_SCORE_NONE;
    let data = Some(&mut user_data);
    // Anything but `OK` leaves the value with this frame -- `NOTDONE` (the
    // word was already in the list) included -- which drops it.
    ins_compl_add(text, None, extra, data, dir, flags, dup, user_hl, score)
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
    // By index: `ins_compl_add_tv` runs user code (an autocommand, a
    // `'completefunc'`), which may edit the very list it is reading.
    let mut at = 0;
    // SAFETY: the caller's promise: a live list.
    while at < list_items(unsafe { list.as_ref() }).len() {
        let tv = &raw const list_items(unsafe { list.as_ref() })[at].li_tv;
        if unsafe { ins_compl_add_tv(&*tv, dir, true) } == OK {
            // If dir was BACKWARD then honour it just once.
            dir = FORWARD;
        } else if did_emsg.get() != 0 {
            break;
        }
        at += 1;
    }
}

/// Add the matches a `'completefunc'`-style dict answers, and note its
/// optional `refresh` item.
///
/// # Safety
///
/// `dict` must point at a live dictionary, unaliased for the call.
pub(crate) unsafe fn ins_compl_add_dict(dict: *mut Dict) {
    // SAFETY: the caller's promise: a live dictionary.
    let find = |key: &[u8]| dict_find(unsafe { dict.as_ref() }, key);

    // Check for the optional "refresh" item.
    compl_opt_refresh_always.set(false);
    if let Some(di) = find(b"refresh")
        && di.di_tv.v_type() == VAR_STRING
    {
        let v = di.di_tv.string_or_null();
        // SAFETY: a non-null string of the item's own.
        if !v.is_null() && unsafe { cstr::eq_bytes(v, b"always") } {
            compl_opt_refresh_always.set(true);
        }
    }

    // Add completions from a "words" list.
    if let Some(di) = find(b"words")
        && di.di_tv.v_type() == VAR_LIST
    {
        // SAFETY: the list the item holds, live while the dictionary is.
        unsafe { ins_compl_add_list(di.di_tv.list_or_null()) };
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
    pub(crate) fn restore(self) {
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
    ins_compl_clear();
    ins_compl_free();
    compl_get_longest.set(compl_longest);

    compl_direction.set(FORWARD);
    if startcol > Win::current().w_cursor.col {
        startcol = Win::current().w_cursor.col;
    }
    compl_col.set(startcol);
    compl_lnum.set(Win::current().w_cursor.lnum);
    compl_length.set(Win::current().w_cursor.col - startcol);
    // compl_pattern doesn't need to be set.
    compl_orig_text().set(compl_text_from_line());
    unsafe { compl_orig_extmarks().save() };

    let mut flags = CP_ORIGINAL_TEXT;
    if p_ic() {
        flags |= CP_ICASE;
    }
    if ins_compl_add_orig_text(flags | CP_FAST).is_err() {
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

    may_trigger_modechanged();
    ui_flush();
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
        let startcol = tv_get_number_chk(&args[0]).unwrap_or(-1) as ColNr;
        if startcol > 0 {
            unsafe { set_completion(startcol - 1, args[1].list_or_null()) };
        }
    }
}

/// The `complete_add()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_add(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    (*result).write_number(ins_compl_add_tv(&args[0], kDirectionNotSet, false) as VarNumber);
}

/// The `complete_check()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_check(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _redraw = Allow::redraw();
    ins_compl_check_keys(0, true);
    result.write_number(ins_compl_interrupted() as VarNumber);
}

/// Fill `di` with match `m`, as `complete_info()` reports it.
pub(crate) fn fill_complete_info_dict(di: &mut Dict, m: MatchId, add_match: bool) {
    m.with(|item| {
        let null = ptr::null();
        let extra = |which: c_int| {
            item.extra[which as usize]
                .as_ref()
                .map_or(null, |s| s.as_ptr())
        };
        // SAFETY: each value is null or a NUL-terminated string of the match's.
        let add_str = |di: &mut Dict, key: &str, val: *const c_char| unsafe {
            let _ = di.add_str(key.as_bytes(), val);
        };
        add_str(di, "word", item.text.as_ptr());
        add_str(di, "abbr", extra(CPT_ABBR));
        add_str(di, "menu", extra(CPT_MENU));
        add_str(di, "kind", extra(CPT_KIND));
        add_str(di, "info", extra(CPT_INFO));
        if add_match {
            let _ = di.add_bool(b"match", item.in_match_array as BoolVarValue);
        }
        if item.user_data.v_type() == VAR_UNKNOWN {
            // Add an empty string for backwards compatibility.
            add_str(di, "user_data", c"".as_ptr());
        } else {
            let _ = di.add_tv(b"user_data", &item.user_data);
        }
    });
}

/// Fill `retdict` with whatever of `complete_info()` `what_list` asked for.
///
/// # Safety
///
/// `what_list` must point at a live list, unaliased for the call. `retdict`
/// must point at a live dictionary, unaliased for the call.
pub(crate) unsafe fn get_complete_info(what_list: *mut List, retdict: *mut Dict) {
    let mut numbuf = NumBuf::new();
    let add_nr = |key: &str, val: VarNumber| unsafe { (*retdict).add_number(key.as_bytes(), val) };

    let mut what_flag;
    if what_list.is_null() {
        what_flag = CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
    } else {
        what_flag = 0;
        for item in list_iter(unsafe { what_list.as_ref() }) {
            // `tv_get_string` answers "" rather than NULL for anything it
            // cannot render, so this is never a null pointer.
            let what = unsafe { CStr::from_ptr(numbuf.string_ptr(&item.li_tv)) };
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
        }
    }

    let mut ret = Ok(());
    if what_flag & CI_WHAT_MODE != 0 {
        let (key, klen) = ("mode".as_ptr().cast(), "mode".len());
        // SAFETY: `retdict` is the dict being built and `ins_compl_mode`
        // answers a NUL-terminated static name.
        ret = unsafe { (*retdict).add_str(cstr::slice_at(key, klen), ins_compl_mode()) };
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
        ret = unsafe { (*retdict).add_str_len(cstr::slice_at(key, klen), text, len.max(0)) };
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
        let held = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        li = held.as_ptr();
        let key = if has_matches && !has_items {
            "matches"
        } else {
            "items"
        };
        ret = unsafe { (*retdict).add_list(key.as_bytes(), Some(held)) };
    }
    if ret.is_ok()
        && what_flag & CI_WHAT_SELECTED != 0
        && curr_match().is_some_and(|curr| curr.with(|m| m.number) == -1)
    {
        ins_compl_update_sequence_numbers();
    }
    if ret.is_ok() {
        let mut list_idx = 0;
        for match_0 in matches_from(first_match()) {
            if match_0.is_original() {
                continue;
            }
            let in_array = match_0.in_match_array();
            if has_items || (has_matches && in_array) {
                let mut di_held = tv_dict_alloc();
                fill_complete_info_dict(&mut di_held, match_0, has_matches && has_items);
                // SAFETY: the list just made, which `retdict` holds.
                unsafe { (*li).push_dict(Some(di_held)) };
            }
            if curr_match()
                .is_some_and(|curr| curr.with(|m| m.number) == match_0.with(|m| m.number))
            {
                selected_idx = list_idx;
            }
            if !has_matches || in_array {
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
        let mut di_held = tv_dict_alloc();
        let curr = curr_match().expect("a selected item is the current match");
        fill_complete_info_dict(&mut di_held, curr, false);
        let (key, klen) = ("completed".as_ptr().cast(), "completed".len());
        let _ = unsafe { (*retdict).add_dict(cstr::slice_at(key, klen), Some(di_held)) };
    }
}

/// The `complete_info()` function; a `VimLFunc` row in the builtin table.
pub fn f_complete_info(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(result);

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
