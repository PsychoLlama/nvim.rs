//! Firing: `apply_autocmds` and the walk it sets up.
//!
//! [`apply_autocmds_group`] is the whole event: it decides whether anything
//! matches, saves and swaps the editor state an autocommand is allowed to
//! see (`<afile>`, `<abuf>`, `v:event`, the search patterns, the redo
//! buffer), pushes an `AutoPatCmd` onto `active_apc_list` and runs the
//! matching commands through `do_cmdline`, then unwinds all of it.  The
//! three `apply_autocmds*` entry points above it differ only in what they
//! pass and what they return; [`block_autocmds`] is the editor-wide off
//! switch.
//!
//! **`patcmd` is a stack local whose address escapes.**  It is linked onto
//! the global `active_apc_list` for the duration of `do_cmdline`, because a
//! handler can wipe out a buffer (`aubuflocal_remove` walks that list and
//! clears the matching `arg_bufnr`) or fire nested events that push their
//! own.  It cannot become owned data, cannot move, and the unlink is
//! guarded by `active_apc_list == &patcmd` because a nested walk may
//! already have taken it off.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::DoCmdOpts;
use crate::getchar::KeyBuffer;
use crate::guard::Suppress;
use crate::types::{FAIL, MAXPATHL, OK};

/// A zeroed `sctx_T`; `getnextac` fills `patcmd.script_ctx` in from the
/// autocommand it is about to run.
const SCTX_INIT: sctx_T = sctx_T {
    sc_sid: 0,
    sc_seq: 0,
    sc_lnum: 0,
    sc_chan: 0,
};

/// An empty `save_redo_T`; `save_redobuff` fills it in.
const SAVE_REDO_INIT: save_redo_T = save_redo_T {
    sr_redobuff: KeyBuffer::EMPTY,
    sr_old_redobuff: KeyBuffer::EMPTY,
};

/// The events whose `<afile>` is not a file name, so nothing is set.
fn afile_is_not_a_name(event: event_T) -> bool {
    matches!(
        event,
        EVENT_COLORSCHEME
            | EVENT_COLORSCHEMEPRE
            | EVENT_OPTIONSET
            | EVENT_MODECHANGED
            | EVENT_MARKSET
    )
}

/// The events whose file-name argument is a name of something else -- a
/// command line, a filetype, a signal -- and must not be expanded to a
/// full path.
fn afile_is_not_expanded(event: event_T) -> bool {
    matches!(
        event,
        EVENT_CMDLINECHANGED
            | EVENT_CMDLINEENTER
            | EVENT_CMDLINELEAVEPRE
            | EVENT_CMDLINELEAVE
            | EVENT_CMDUNDEFINED
            | EVENT_CURSORMOVEDC
            | EVENT_CMDWINENTER
            | EVENT_CMDWINLEAVE
            | EVENT_COLORSCHEME
            | EVENT_COLORSCHEMEPRE
            | EVENT_DIRCHANGED
            | EVENT_DIRCHANGEDPRE
            | EVENT_FILETYPE
            | EVENT_FUNCUNDEFINED
            | EVENT_MARKSET
            | EVENT_MENUPOPUP
            | EVENT_MODECHANGED
            | EVENT_OPTIONSET
            | EVENT_PROGRESS
            | EVENT_QUICKFIXCMDPOST
            | EVENT_QUICKFIXCMDPRE
            | EVENT_REMOTEREPLY
            | EVENT_SIGNAL
            | EVENT_SPELLFILEMISSING
            | EVENT_SYNTAX
            | EVENT_TABCLOSED
            | EVENT_USER
            | EVENT_WINCLOSED
            | EVENT_WINRESIZED
            | EVENT_WINSCROLLED
    )
}

/// The events that do not set or reset the `Changed` flag themselves, so
/// it is put back afterwards.
fn keeps_changed_flag(event: event_T) -> bool {
    matches!(
        event,
        EVENT_BUFREADPOST
            | EVENT_BUFWRITEPOST
            | EVENT_FILEAPPENDPOST
            | EVENT_VIMLEAVE
            | EVENT_VIMLEAVEPRE
    )
}

/// Fire `event`, in every group.
pub unsafe fn apply_autocmds(
    event: event_T,
    fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    force: bool,
    buf: *mut buf_T,
) -> bool {
    unsafe {
        apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL,
            buf,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        )
    }
}

/// [`apply_autocmds`], passing an `exarg_T` on so `v:cmdarg` and
/// `v:cmdbang` are set for the handlers.
pub unsafe fn apply_autocmds_exarg(
    event: event_T,
    fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    force: bool,
    buf: *mut buf_T,
    eap: *mut exarg_T,
) -> bool {
    unsafe {
        apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL,
            buf,
            eap,
            ::core::ptr::null_mut(),
        )
    }
}

/// [`apply_autocmds`] threaded through a caller's `OK`/`FAIL`: it does
/// nothing once that says to abort, and turns it to `FAIL` if a handler
/// aborted.
pub unsafe fn apply_autocmds_retval(
    event: event_T,
    fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    force: bool,
    buf: *mut buf_T,
    retval: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        if should_abort(*retval) {
            return false;
        }
        let did_cmd = apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL,
            buf,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        );
        if did_cmd && aborting() {
            *retval = FAIL;
        }
        did_cmd
    }
}

/// Fire `event` for `buf`, running every autocommand in `group` whose
/// pattern matches.
///
/// Answers whether any autocommand was executed.  The body is one labeled
/// block, which is upstream's `goto BYPASS_AU`: every reason not to fire
/// leaves it, and the two things that happen either way -- wiping a
/// buffer's own autocommands, and remembering that `FileType` ran -- are
/// after it.
pub unsafe fn apply_autocmds_group(
    event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    force: bool,
    group: ::core::ffi::c_int,
    buf: *mut buf_T,
    eap: *mut exarg_T,
    data: *mut Object,
) -> bool {
    unsafe {
        static nesting: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static filechangeshell_busy: GlobalCell<bool> = GlobalCell::new(false);

        let mut sfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
        let mut retval = false;
        let mut did_save_redobuff = false;
        let mut save_redo = SAVE_REDO_INIT;
        let save_KeyTyped = KeyTyped.get();

        'bypass: {
            // Nothing to fire, or firing is off.
            if event == NUM_EVENTS || (*au_event_vec(event)).size == 0 || is_autocmd_blocked() {
                break 'bypass;
            }
            // While autocommands are running, a new one only fires when it
            // was asked for explicitly (`++nested`).
            if autocmd_busy.get() && !(force || autocmd_nested.get()) {
                break 'bypass;
            }
            // An error, interrupt or uncaught exception is pending.
            if aborting() {
                break 'bypass;
            }
            // FileChangedShell never nests: it would loop forever.
            if filechangeshell_busy.get()
                && (event == EVENT_FILECHANGEDSHELL || event == EVENT_FILECHANGEDSHELLPOST)
            {
                break 'bypass;
            }
            if event_ignored(event, p_ei.get()) {
                break 'bypass;
            }

            // 'eventignorewin' is per window, so the question is whether
            // *every* window showing the buffer ignores the event.  Only
            // window-local events (`event <= 0`) can be listed there.
            let mut win_ignore = false;
            if buf == curbuf.get() && event_row(event).event <= 0 {
                win_ignore = event_ignored(event, (*curwin.get()).w_onebuf_opt.wo_eiw);
            } else if !buf.is_null() && event_row(event).event <= 0 && (*buf).b_nwindows > 0 {
                win_ignore = true;
                let mut tp = first_tabpage.get();
                while !tp.is_null() {
                    let mut wp = if tp == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !wp.is_null() {
                        if (*wp).w_buffer == buf && !event_ignored(event, (*wp).w_onebuf_opt.wo_eiw)
                        {
                            win_ignore = false;
                            break;
                        }
                        wp = (*wp).w_next;
                    }
                    tp = (*tp).tp_next;
                }
            }
            if win_ignore {
                break 'bypass;
            }

            // Nesting is allowed but bounded: it is easy to write an
            // endless loop.
            if nesting.get() == 10 {
                emsg(gettext(E_AUTOCOMMAND_NESTING_TOO_DEEP.as_ptr()));
                break 'bypass;
            }
            // `:all` and `:ball` turn these off while they shuffle windows.
            if (autocmd_no_enter.get() != 0 && (event == EVENT_WINENTER || event == EVENT_BUFENTER))
                || (autocmd_no_leave.get() != 0
                    && (event == EVENT_WINLEAVE || event == EVENT_BUFLEAVE))
            {
                break 'bypass;
            }

            // Save the autocmd_* variables and what we know about the
            // current buffer.
            let save_autocmd_fname = autocmd_fname.get();
            let save_autocmd_fname_full = autocmd_fname_full.get();
            let save_autocmd_bufnr = autocmd_bufnr.get();
            let save_autocmd_match = autocmd_match.get();
            let save_autocmd_busy = autocmd_busy.get();
            let save_autocmd_nested = autocmd_nested.get();
            let save_changed = (*curbuf.get()).b_changed != 0;
            let old_curbuf = curbuf.get();

            // `<afile>`.  A copy, so renaming a buffer or changing
            // directory cannot invalidate it.
            autocmd_fname.set(if !fname_io.is_null() {
                fname_io
            } else if afile_is_not_a_name(event) {
                ::core::ptr::null_mut()
            } else if !fname.is_null() && ends_excmd(*fname as ::core::ffi::c_int) == 0 {
                fname
            } else if !buf.is_null() {
                (*buf).b_ffname
            } else {
                ::core::ptr::null_mut()
            });
            // The unexpanded `<afile>`, kept for the API's `file` field.
            let mut afile_orig: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
            if !autocmd_fname.get().is_null() {
                afile_orig = xstrdup(autocmd_fname.get());
                // MAXPATHL, because `eval_vars` resolves the full path in
                // place later.
                autocmd_fname.set(xstrnsave(autocmd_fname.get(), MAXPATHL as size_t));
            }
            autocmd_fname_full.set(false);

            // `<abuf>`.
            autocmd_bufnr.set(if buf.is_null() { 0 } else { (*buf).handle });

            // The name to match the patterns against.  Always a full path,
            // in case a pattern has `allow_dirs` set.
            if fname.is_null() || *fname == 0 {
                if buf.is_null() {
                    fname = ::core::ptr::null_mut();
                } else if event == EVENT_SYNTAX {
                    fname = (*buf).b_p_syn;
                } else if event == EVENT_FILETYPE {
                    fname = (*buf).b_p_ft;
                } else {
                    if !(*buf).b_sfname.is_null() {
                        sfname = xstrdup((*buf).b_sfname);
                    }
                    fname = (*buf).b_ffname;
                }
                if fname.is_null() {
                    fname = c"".as_ptr().cast_mut();
                }
                // A copy, so it can be changed.
                fname = xstrdup(fname);
            } else {
                sfname = xstrdup(fname);
                if afile_is_not_expanded(event) {
                    fname = xstrdup(fname);
                    // Don't expand it later either.
                    autocmd_fname_full.set(true);
                } else {
                    fname = full_name_save(fname, false);
                }
            }
            if fname.is_null() {
                // Out of memory.
                xfree(sfname.cast::<::core::ffi::c_void>());
                retval = false;
                break 'bypass;
            }

            // `<amatch>`.
            autocmd_match.set(fname);

            // Don't redraw while running autocommands.
            let redraw_off = Suppress::redraw();

            // `es_name` and `es_lnum` are filled in by `aucmd_next`.
            estack_push(ETYPE_AUCMD, ::core::ptr::null_mut(), 0);

            let save_current_sctx = current_sctx.get();

            let mut wait_time: proftime_T = 0;
            if do_profiling.get() == PROF_YES {
                // Doesn't count for the caller itself.
                wait_time = prof_child_enter();
            }

            // Don't use the caller's function-local variables.
            let mut funccal_entry = funccal_entry_T {
                top_funccal: ::core::ptr::null_mut(),
                next: ::core::ptr::null_mut(),
            };
            save_funccal(&raw mut funccal_entry);

            // Only the outermost firing saves the search patterns and the
            // redo buffer.
            if !autocmd_busy.get() {
                save_search_patterns();
                if !ins_compl_active() {
                    save_redobuff(&raw mut save_redo);
                    did_save_redobuff = true;
                }
                (*curbuf.get()).b_did_filetype = (*curbuf.get()).b_keep_filetype;
            }

            // Some commands need to know autocommands are running.
            autocmd_busy.set(true);
            filechangeshell_busy.set(event == EVENT_FILECHANGEDSHELL);
            // See the matching decrement below.
            *nesting.ptr() += 1;

            // Remembered for `did_filetype()`.
            if event == EVENT_FILETYPE {
                (*curbuf.get()).b_did_filetype = true;
            }

            let tail = path_tail(fname);

            // The walk's cursor.  This is a *stack local* whose address is
            // published on `active_apc_list` below; see the module docs.
            let mut patcmd = AutoPatCmd_S {
                // `aucmd_next` sets `lastpat` back to null when there is
                // nothing left to run.
                lastpat: ::core::ptr::null_mut(),
                auidx: 0,
                // The size is snapshotted so that patterns added by a
                // handler cannot extend this walk into an endless loop.
                ausize: (*au_event_vec(event)).size,
                afile_orig,
                fname,
                sfname,
                tail,
                group,
                event,
                script_ctx: SCTX_INIT,
                arg_bufnr: autocmd_bufnr.get(),
                data: ::core::ptr::null_mut(),
                next: ::core::ptr::null_mut(),
            };
            aucmd_next(&raw mut patcmd);

            // Something matched: run the autocommands.
            if !patcmd.lastpat.is_null() {
                patcmd.next = active_apc_list.get();
                active_apc_list.set(&raw mut patcmd);
                patcmd.data = data;

                // `v:cmdarg`/`v:cmdbang`, only when a pattern matched.
                let save_cmdbang = get_vim_var_nr(Vv::Cmdbang);
                let save_cmdarg = if eap.is_null() {
                    ::core::ptr::null_mut()
                } else {
                    let saved = set_cmdarg(eap, ::core::ptr::null_mut());
                    set_vim_var_nr(Vv::Cmdbang, (*eap).forceit as varnumber_T);
                    saved
                };
                retval = true;

                // Make the cursor and topline valid.  The outermost firing
                // saves them for `reset_lnums`; a nested one only corrects.
                if nesting.get() == 1 {
                    check_lnums(true);
                } else {
                    check_lnums_nested(true);
                }

                let save_did_emsg = did_emsg.get();
                let save_ex_pressedreturn = get_pressedreturn();

                // `getnextac` is what iterates: it is pulled once per
                // matching autocommand.
                do_cmdline(
                    ::core::ptr::null_mut(),
                    Some(getnextac),
                    (&raw mut patcmd).cast::<::core::ffi::c_void>(),
                    DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT,
                );

                *did_emsg.ptr() += save_did_emsg;
                set_pressedreturn(save_ex_pressedreturn);

                if nesting.get() == 1 {
                    // Restore the cursor and topline unless they changed.
                    reset_lnums();
                }

                if !eap.is_null() {
                    set_cmdarg(::core::ptr::null_mut(), save_cmdarg);
                    set_vim_var_nr(Vv::Cmdbang, save_cmdbang);
                }
                // Unlink -- guarded, because a nested walk may already
                // have taken this node off the list.
                if active_apc_list.get() == &raw mut patcmd {
                    active_apc_list.set(patcmd.next);
                }
            }

            drop(redraw_off);
            autocmd_busy.set(save_autocmd_busy);
            filechangeshell_busy.set(false);
            autocmd_nested.set(save_autocmd_nested);
            // `SOURCING_NAME`: `aucmd_next` left the last one here.
            xfree(
                crate::runtime::innermost_frame()
                    .es_name
                    .cast::<::core::ffi::c_void>(),
            );
            estack_pop();
            xfree(afile_orig.cast::<::core::ffi::c_void>());
            xfree(autocmd_fname.get().cast::<::core::ffi::c_void>());
            autocmd_fname.set(save_autocmd_fname);
            autocmd_fname_full.set(save_autocmd_fname_full);
            autocmd_bufnr.set(save_autocmd_bufnr);
            autocmd_match.set(save_autocmd_match);
            current_sctx.set(save_current_sctx);
            restore_funccal();
            if do_profiling.get() == PROF_YES {
                prof_child_exit(wait_time);
            }
            KeyTyped.set(save_KeyTyped);
            xfree(fname.cast::<::core::ffi::c_void>());
            xfree(sfname.cast::<::core::ffi::c_void>());
            // See the matching increment above.
            *nesting.ptr() -= 1;

            // The outermost firing puts the search patterns and the redo
            // buffer back, and frees what the handlers deferred.
            if !autocmd_busy.get() {
                restore_search_patterns();
                if did_save_redobuff {
                    restore_redobuff(&raw mut save_redo);
                }
                (*curbuf.get()).b_did_filetype = false;
                while !au_pending_free_buf.get().is_null() {
                    let b = (*au_pending_free_buf.get()).b_next;
                    xfree(au_pending_free_buf.get().cast::<::core::ffi::c_void>());
                    au_pending_free_buf.set(b);
                }
                while !au_pending_free_win.get().is_null() {
                    let w = (*au_pending_free_win.get()).w_next;
                    xfree(au_pending_free_win.get().cast::<::core::ffi::c_void>());
                    au_pending_free_win.set(w);
                }
            }

            // Only if we are still in the same buffer.
            if curbuf.get() == old_curbuf && keeps_changed_flag(event) {
                if (*curbuf.get()).b_changed != save_changed as ::core::ffi::c_int {
                    need_maketitle.set(true);
                }
                (*curbuf.get()).b_changed = save_changed as ::core::ffi::c_int;
            }

            // The patterns and commands marked deleted can really go now.
            au_cleanup();
        }

        // Wiping a buffer takes its buffer-local autocommands with it,
        // whether or not anything fired.
        if event == EVENT_BUFWIPEOUT && !buf.is_null() {
            aubuflocal_remove(buf);
        }
        if retval as ::core::ffi::c_int == OK && event == EVENT_FILETYPE {
            (*curbuf.get()).b_au_did_filetype = true;
        }

        retval
    }
}

/// Turn autocommands off editor-wide, nestably.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_autocmds() {
    unsafe {
        // Remember that we may need to fire `TermResponse` later.
        if !is_autocmd_blocked() {
            termresponse_changed.set(false);
        }
        *autocmd_blocked.ptr() += 1;
    }
}

/// Undo one [`block_autocmds`], firing the `TermResponse` that arrived
/// while they were off.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unblock_autocmds() {
    unsafe {
        *autocmd_blocked.ptr() -= 1;
        if !is_autocmd_blocked() && termresponse_changed.get() && has_event(EVENT_TERMRESPONSE) {
            let sequence = cstr_to_string(get_vim_var_str(Vv::Termresponse));
            do_termresponse_autocmd(sequence);
            api_free_string(sequence);
        }
    }
}

/// Whether [`block_autocmds`] is in effect.
pub fn is_autocmd_blocked() -> bool {
    autocmd_blocked.get() != 0
}
