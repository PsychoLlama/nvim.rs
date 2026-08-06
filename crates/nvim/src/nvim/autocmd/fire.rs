//! Firing: `apply_autocmds` and the walk it sets up.
//!
//! `apply_autocmds_group` is the whole event: it decides whether anything
//! matches, saves and swaps the editor state an autocommand is allowed to
//! see (`<afile>`, `<abuf>`, `v:event`, the search patterns, the redo
//! buffer), pushes an `AutoPatCmd` onto `active_apc_list` and runs the
//! matching commands through `do_cmdline`, then unwinds all of it.  The
//! three `apply_autocmds*` entry points above it differ only in what they
//! pass and what they return; `block_autocmds` is the editor-wide off
//! switch.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn apply_autocmds(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
) -> bool {
    unsafe {
        return apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL as ::core::ffi::c_int,
            buf,
            ::core::ptr::null_mut::<exarg_T>(),
            ::core::ptr::null_mut::<Object>(),
        );
    }
}

pub unsafe extern "C" fn apply_autocmds_exarg(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
) -> bool {
    unsafe {
        return apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL as ::core::ffi::c_int,
            buf,
            eap,
            ::core::ptr::null_mut::<Object>(),
        );
    }
}

pub unsafe extern "C" fn apply_autocmds_retval(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut retval: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        if should_abort(*retval) {
            return false_0 != 0;
        }
        let mut did_cmd: bool = apply_autocmds_group(
            event,
            fname,
            fname_io,
            force,
            AUGROUP_ALL as ::core::ffi::c_int,
            buf,
            ::core::ptr::null_mut::<exarg_T>(),
            ::core::ptr::null_mut::<Object>(),
        );
        if did_cmd as ::core::ffi::c_int != 0 && aborting() as ::core::ffi::c_int != 0 {
            *retval = FAIL;
        }
        return did_cmd;
    }
}

pub unsafe extern "C" fn apply_autocmds_group(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) -> bool {
    unsafe {
        let mut win_ignore: bool = false;
        let mut save_autocmd_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut save_autocmd_fname_full: bool = false;
        let mut save_autocmd_bufnr: ::core::ffi::c_int = 0;
        let mut save_autocmd_match: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut save_autocmd_busy: ::core::ffi::c_int = 0;
        let mut save_autocmd_nested: ::core::ffi::c_int = 0;
        let mut save_changed: bool = false;
        let mut old_curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut afile_orig: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut save_current_sctx: sctx_T = sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        };
        let mut funccal_entry: funccal_entry_T = funccal_entry_T {
            top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            next: ::core::ptr::null_mut::<funccal_entry_T>(),
        };
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut patcmd: AutoPatCmd = AutoPatCmd {
            lastpat: ::core::ptr::null_mut::<AutoPat>(),
            auidx: 0,
            ausize: 0,
            afile_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tail: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            group: 0,
            event: EVENT_BUFADD,
            script_ctx: sctx_T {
                sc_sid: 0,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            },
            arg_bufnr: 0,
            data: ::core::ptr::null_mut::<Object>(),
            next: ::core::ptr::null_mut::<AutoPatCmd>(),
        };
        let mut sfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut retval: bool = false_0 != 0;
        static nesting: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut save_cmdarg: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        static filechangeshell_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut wait_time: proftime_T = 0;
        let mut did_save_redobuff: bool = false_0 != 0;
        let mut save_redo: save_redo_T = save_redo_T {
            sr_redobuff: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            sr_old_redobuff: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
        };
        let save_KeyTyped: bool = KeyTyped.get();
        if !(event as ::core::ffi::c_uint
            == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size == 0 as size_t
            || is_autocmd_blocked() as ::core::ffi::c_int != 0)
        {
            if !(autocmd_busy.get() as ::core::ffi::c_int != 0
                && !(force as ::core::ffi::c_int != 0
                    || autocmd_nested.get() as ::core::ffi::c_int != 0))
            {
                if !aborting() {
                    if !(filechangeshell_busy.get() as ::core::ffi::c_int != 0
                        && (event as ::core::ffi::c_uint
                            == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int as ::core::ffi::c_uint
                            || event as ::core::ffi::c_uint
                                == EVENT_FILECHANGEDSHELLPOST as ::core::ffi::c_int
                                    as ::core::ffi::c_uint))
                    {
                        if !event_ignored(event, p_ei.get()) {
                            win_ignore = false_0 != 0;
                            if buf == curbuf.get()
                                && (*event_names.ptr())[event as usize].event
                                    <= 0 as ::core::ffi::c_int
                            {
                                win_ignore =
                                    event_ignored(event, (*curwin.get()).w_onebuf_opt.wo_eiw);
                            } else if !buf.is_null()
                                && (*event_names.ptr())[event as usize].event
                                    <= 0 as ::core::ffi::c_int
                                && (*buf).b_nwindows > 0 as ::core::ffi::c_int
                            {
                                win_ignore = true_0 != 0;
                                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                                while !tp.is_null() {
                                    let mut wp: *mut win_T = if tp == curtab.get() {
                                        firstwin.get()
                                    } else {
                                        (*tp).tp_firstwin
                                    };
                                    while !wp.is_null() {
                                        if (*wp).w_buffer == buf
                                            && !event_ignored(event, (*wp).w_onebuf_opt.wo_eiw)
                                        {
                                            win_ignore = false_0 != 0;
                                            break;
                                        } else {
                                            wp = (*wp).w_next;
                                        }
                                    }
                                    tp = (*tp).tp_next as *mut tabpage_T;
                                }
                            }
                            if !win_ignore {
                                if nesting.get() == 10 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        (e_autocommand_nesting_too_deep.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ));
                                } else if !(autocmd_no_enter.get() != 0
                                    && (event as ::core::ffi::c_uint
                                        == EVENT_WINENTER as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_BUFENTER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint)
                                    || autocmd_no_leave.get() != 0
                                        && (event as ::core::ffi::c_uint
                                            == EVENT_WINLEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_BUFLEAVE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint))
                                {
                                    save_autocmd_fname = autocmd_fname.get();
                                    save_autocmd_fname_full = autocmd_fname_full.get();
                                    save_autocmd_bufnr = autocmd_bufnr.get();
                                    save_autocmd_match = autocmd_match.get();
                                    save_autocmd_busy = autocmd_busy.get() as ::core::ffi::c_int;
                                    save_autocmd_nested =
                                        autocmd_nested.get() as ::core::ffi::c_int;
                                    save_changed = (*curbuf.get()).b_changed != 0;
                                    old_curbuf = curbuf.get();
                                    if fname_io.is_null() {
                                        if event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEME as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_OPTIONSET as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_MODECHANGED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_MARKSET as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            autocmd_fname
                                            .set(::core::ptr::null_mut::<::core::ffi::c_char>());
                                        } else if !fname.is_null()
                                            && ends_excmd(*fname as ::core::ffi::c_int) == 0
                                        {
                                            autocmd_fname.set(fname);
                                        } else if !buf.is_null() {
                                            autocmd_fname.set((*buf).b_ffname);
                                        } else {
                                            autocmd_fname
                                            .set(::core::ptr::null_mut::<::core::ffi::c_char>());
                                        }
                                    } else {
                                        autocmd_fname.set(fname_io);
                                    }
                                    afile_orig = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    if !(*autocmd_fname.ptr()).is_null() {
                                        afile_orig = xstrdup(autocmd_fname.get());
                                        autocmd_fname.set(xstrnsave(
                                            autocmd_fname.get(),
                                            MAXPATHL as size_t,
                                        ));
                                    }
                                    autocmd_fname_full.set(false_0 != 0);
                                    autocmd_bufnr.set(if buf.is_null() {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        (*buf).handle as ::core::ffi::c_int
                                    });
                                    if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
                                        if buf.is_null() {
                                            fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        } else if event as ::core::ffi::c_uint
                                            == EVENT_SYNTAX as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            fname = (*buf).b_p_syn;
                                        } else if event as ::core::ffi::c_uint
                                            == EVENT_FILETYPE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            fname = (*buf).b_p_ft;
                                        } else {
                                            if !(*buf).b_sfname.is_null() {
                                                sfname = xstrdup((*buf).b_sfname);
                                            }
                                            fname = (*buf).b_ffname;
                                        }
                                        if fname.is_null() {
                                            fname = b"\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char;
                                        }
                                        fname = xstrdup(fname);
                                    } else {
                                        sfname = xstrdup(fname);
                                        if event as ::core::ffi::c_uint
                                            == EVENT_CMDLINECHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDLINEENTER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDLINELEAVEPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDLINELEAVE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDUNDEFINED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CURSORMOVEDC as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDWINENTER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_CMDWINLEAVE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_COLORSCHEME as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_DIRCHANGED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_FILETYPE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_FUNCUNDEFINED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_MARKSET as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_MENUPOPUP as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_MODECHANGED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_OPTIONSET as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_PROGRESS as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_QUICKFIXCMDPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_QUICKFIXCMDPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_REMOTEREPLY as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_SIGNAL as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_SPELLFILEMISSING as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_SYNTAX as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_TABCLOSED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_USER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_WINCLOSED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_WINRESIZED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_WINSCROLLED as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            fname = xstrdup(fname);
                                            autocmd_fname_full.set(true_0 != 0);
                                        } else {
                                            fname = FullName_save(fname, false_0 != 0);
                                        }
                                    }
                                    if fname.is_null() {
                                        xfree(sfname as *mut ::core::ffi::c_void);
                                        retval = false_0 != 0;
                                    } else {
                                        autocmd_match.set(fname);
                                        (*RedrawingDisabled.ptr()) += 1;
                                        estack_push(
                                            ETYPE_AUCMD,
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            0 as linenr_T,
                                        );
                                        save_current_sctx = current_sctx.get();
                                        if do_profiling.get() == PROF_YES {
                                            wait_time = prof_child_enter();
                                        }
                                        funccal_entry = funccal_entry_T {
                                            top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(
                                            ),
                                            next: ::core::ptr::null_mut::<funccal_entry_T>(),
                                        };
                                        save_funccal(&raw mut funccal_entry);
                                        if !autocmd_busy.get() {
                                            save_search_patterns();
                                            if !ins_compl_active() {
                                                saveRedobuff(&raw mut save_redo);
                                                did_save_redobuff = true_0 != 0;
                                            }
                                            (*curbuf.get()).b_did_filetype =
                                                (*curbuf.get()).b_keep_filetype;
                                        }
                                        autocmd_busy.set(true_0 != 0);
                                        filechangeshell_busy.set(
                                            event as ::core::ffi::c_uint
                                                == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint,
                                        );
                                        (*nesting.ptr()) += 1;
                                        if event as ::core::ffi::c_uint
                                            == EVENT_FILETYPE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*curbuf.get()).b_did_filetype = true_0 != 0;
                                        }
                                        tail = path_tail(fname);
                                        patcmd = AutoPatCmd_S {
                                            lastpat: ::core::ptr::null_mut::<AutoPat>(),
                                            auidx: 0 as size_t,
                                            ausize: (*autocmds.ptr())
                                                [event as ::core::ffi::c_int as usize]
                                                .size,
                                            afile_orig: afile_orig,
                                            fname: fname,
                                            sfname: sfname,
                                            tail: tail,
                                            group: group,
                                            event: event,
                                            script_ctx: sctx_T {
                                                sc_sid: 0,
                                                sc_seq: 0,
                                                sc_lnum: 0,
                                                sc_chan: 0,
                                            },
                                            arg_bufnr: autocmd_bufnr.get(),
                                            data: ::core::ptr::null_mut::<Object>(),
                                            next: ::core::ptr::null_mut::<AutoPatCmd>(),
                                        };
                                        aucmd_next(&raw mut patcmd);
                                        if !patcmd.lastpat.is_null() {
                                            patcmd.next = active_apc_list.get();
                                            active_apc_list.set(&raw mut patcmd);
                                            patcmd.data = data;
                                            let mut save_cmdbang: varnumber_T =
                                                get_vim_var_nr(VV_CMDBANG);
                                            if !eap.is_null() {
                                                save_cmdarg = set_cmdarg(
                                                    eap,
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                );
                                                set_vim_var_nr(
                                                    VV_CMDBANG,
                                                    (*eap).forceit as varnumber_T,
                                                );
                                            } else {
                                                save_cmdarg =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            }
                                            retval = true_0 != 0;
                                            if nesting.get() == 1 as ::core::ffi::c_int {
                                                check_lnums(true_0 != 0);
                                            } else {
                                                check_lnums_nested(true_0 != 0);
                                            }
                                            let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
                                            let save_ex_pressedreturn: bool = get_pressedreturn();
                                            do_cmdline(
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            Some(
                                                getnextac
                                                    as unsafe extern "C" fn(
                                                        ::core::ffi::c_int,
                                                        *mut ::core::ffi::c_void,
                                                        ::core::ffi::c_int,
                                                        bool,
                                                    ) -> *mut ::core::ffi::c_char,
                                            ),
                                            &raw mut patcmd as *mut ::core::ffi::c_void,
                                            DOCMD_NOWAIT as ::core::ffi::c_int
                                                | DOCMD_VERBOSE as ::core::ffi::c_int
                                                | DOCMD_REPEAT as ::core::ffi::c_int,
                                        );
                                            (*did_emsg.ptr()) += save_did_emsg;
                                            set_pressedreturn(save_ex_pressedreturn);
                                            if nesting.get() == 1 as ::core::ffi::c_int {
                                                reset_lnums();
                                            }
                                            if !eap.is_null() {
                                                set_cmdarg(
                                                    ::core::ptr::null_mut::<exarg_T>(),
                                                    save_cmdarg,
                                                );
                                                set_vim_var_nr(VV_CMDBANG, save_cmdbang);
                                            }
                                            if active_apc_list.get() == &raw mut patcmd {
                                                active_apc_list.set(patcmd.next);
                                            }
                                        }
                                        (*RedrawingDisabled.ptr()) -= 1;
                                        autocmd_busy.set(save_autocmd_busy != 0);
                                        filechangeshell_busy.set(false_0 != 0);
                                        autocmd_nested.set(save_autocmd_nested != 0);
                                        xfree(
                                            (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                                ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            ))
                                            .es_name
                                                as *mut ::core::ffi::c_void,
                                        );
                                        estack_pop();
                                        xfree(afile_orig as *mut ::core::ffi::c_void);
                                        xfree(autocmd_fname.get() as *mut ::core::ffi::c_void);
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
                                        xfree(fname as *mut ::core::ffi::c_void);
                                        xfree(sfname as *mut ::core::ffi::c_void);
                                        (*nesting.ptr()) -= 1;
                                        if !autocmd_busy.get() {
                                            restore_search_patterns();
                                            if did_save_redobuff {
                                                restoreRedobuff(&raw mut save_redo);
                                            }
                                            (*curbuf.get()).b_did_filetype = false_0 != 0;
                                            while !(*au_pending_free_buf.ptr()).is_null() {
                                                let mut b: *mut buf_T =
                                                    (*au_pending_free_buf.get()).b_next;
                                                xfree(au_pending_free_buf.get()
                                                    as *mut ::core::ffi::c_void);
                                                au_pending_free_buf.set(b);
                                            }
                                            while !(*au_pending_free_win.ptr()).is_null() {
                                                let mut w: *mut win_T =
                                                    (*au_pending_free_win.get()).w_next;
                                                xfree(au_pending_free_win.get()
                                                    as *mut ::core::ffi::c_void);
                                                au_pending_free_win.set(w);
                                            }
                                        }
                                        if curbuf.get() == old_curbuf
                                            && (event as ::core::ffi::c_uint
                                                == EVENT_BUFREADPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || event as ::core::ffi::c_uint
                                                    == EVENT_BUFWRITEPOST as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                || event as ::core::ffi::c_uint
                                                    == EVENT_FILEAPPENDPOST as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                || event as ::core::ffi::c_uint
                                                    == EVENT_VIMLEAVE as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                || event as ::core::ffi::c_uint
                                                    == EVENT_VIMLEAVEPRE as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint)
                                        {
                                            if (*curbuf.get()).b_changed
                                                != save_changed as ::core::ffi::c_int
                                            {
                                                need_maketitle.set(true_0 != 0);
                                            }
                                            (*curbuf.get()).b_changed =
                                                save_changed as ::core::ffi::c_int;
                                        }
                                        au_cleanup();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if event as ::core::ffi::c_uint
            == EVENT_BUFWIPEOUT as ::core::ffi::c_int as ::core::ffi::c_uint
            && !buf.is_null()
        {
            aubuflocal_remove(buf);
        }
        if retval as ::core::ffi::c_int == OK
            && event as ::core::ffi::c_uint
                == EVENT_FILETYPE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*curbuf.get()).b_au_did_filetype = true_0 != 0;
        }
        return retval;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_autocmds() {
    unsafe {
        if !is_autocmd_blocked() {
            termresponse_changed.set(false_0 != 0);
        }
        (*autocmd_blocked.ptr()) += 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unblock_autocmds() {
    unsafe {
        (*autocmd_blocked.ptr()) -= 1;
        if !is_autocmd_blocked()
            && termresponse_changed.get() as ::core::ffi::c_int != 0
            && has_event(EVENT_TERMRESPONSE) as ::core::ffi::c_int != 0
        {
            let sequence: String_0 = cstr_to_string(get_vim_var_str(VV_TERMRESPONSE));
            do_termresponse_autocmd(sequence);
            api_free_string(sequence);
        }
    }
}

pub unsafe extern "C" fn is_autocmd_blocked() -> bool {
    return autocmd_blocked.get() != 0 as ::core::ffi::c_int;
}
