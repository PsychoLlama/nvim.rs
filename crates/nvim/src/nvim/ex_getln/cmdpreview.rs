//! `'inccommand'`: running the command being typed against a preview buffer.
//!
//! [`cmdpreview_may_show`] is the entry point — it saves everything the
//! command could change, executes it with `cmdpreview` set, shows the result
//! either in place or in a split preview window, and restores.  The
//! `cmdpreview_save_*` / `cmdpreview_restore_*` pairs are that save-restore.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn cmdpreview_get_bufnr() -> handle_T {
    return cmdpreview_bufnr.get();
}

pub unsafe extern "C" fn cmdpreview_get_ns() -> ::core::ffi::c_int {
    return cmdpreview_ns.get();
}

pub(crate) unsafe extern "C" fn cmdpreview_open_buf() -> *mut buf_T {
    unsafe {
        let mut cmdpreview_buf: *mut buf_T = if cmdpreview_bufnr.get() != 0 {
            buflist_findnr(cmdpreview_bufnr.get() as ::core::ffi::c_int)
        } else {
            ::core::ptr::null_mut::<buf_T>()
        };
        if cmdpreview_buf.is_null() {
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut bufnr: handle_T = nvim_create_buf(false_0 != 0, true_0 != 0, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return ::core::ptr::null_mut::<buf_T>();
            }
            cmdpreview_buf = buflist_findnr(bufnr as ::core::ffi::c_int);
        }
        if cmdpreview_buf == curbuf.get() {
            return ::core::ptr::null_mut::<buf_T>();
        }
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
        let mut retv: ::core::ffi::c_int = rename_buffer(b"[Preview]\0".as_ptr()
            as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char);
        aucmd_restbuf(&raw mut aco);
        if retv == FAIL {
            return ::core::ptr::null_mut::<buf_T>();
        }
        aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
        buf_clear();
        (*curbuf.get()).b_p_ma = true_0;
        (*curbuf.get()).b_p_ul = -1 as OptInt;
        (*curbuf.get()).b_p_tw = 0 as OptInt;
        aucmd_restbuf(&raw mut aco);
        cmdpreview_bufnr.set((*cmdpreview_buf).handle);
        return cmdpreview_buf;
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_open_win(mut cmdpreview_buf: *mut buf_T) -> *mut win_T {
    unsafe {
        let mut save_curwin: *mut win_T = curwin.get();
        if win_split(
            p_cwh.get() as ::core::ffi::c_int,
            WSP_BOT as ::core::ffi::c_int,
        ) == FAIL
        {
            return ::core::ptr::null_mut::<win_T>();
        }
        let mut preview_win: *mut win_T = curwin.get();
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut result: ::core::ffi::c_int = OK;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        result = do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            (*cmdpreview_buf).handle as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        try_leave(&raw mut tstate, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
            || result == FAIL
        {
            api_clear_error(&raw mut err);
            return ::core::ptr::null_mut::<win_T>();
        }
        (*curwin.get()).w_onebuf_opt.wo_cul = false_0;
        (*curwin.get()).w_onebuf_opt.wo_cuc = false_0;
        (*curwin.get()).w_onebuf_opt.wo_spell = false_0;
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
        win_enter(save_curwin, false_0 != 0);
        return preview_win;
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_close_win() {
    unsafe {
        let mut buf: *mut buf_T = if cmdpreview_bufnr.get() != 0 {
            buflist_findnr(cmdpreview_bufnr.get() as ::core::ffi::c_int)
        } else {
            ::core::ptr::null_mut::<buf_T>()
        };
        if !buf.is_null() {
            close_windows(buf, false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_save_undo(
    mut cp_undoinfo: *mut CpUndoInfo,
    mut buf: *mut buf_T,
) {
    unsafe {
        (*cp_undoinfo).save_b_u_synced = (*buf).b_u_synced;
        (*cp_undoinfo).save_b_u_oldhead = (*buf).b_u_oldhead;
        (*cp_undoinfo).save_b_u_newhead = (*buf).b_u_newhead;
        (*cp_undoinfo).save_b_u_curhead = (*buf).b_u_curhead;
        (*cp_undoinfo).save_b_u_numhead = (*buf).b_u_numhead;
        (*cp_undoinfo).save_b_u_seq_last = (*buf).b_u_seq_last;
        (*cp_undoinfo).save_b_u_save_nr_last = (*buf).b_u_save_nr_last;
        (*cp_undoinfo).save_b_u_seq_cur = (*buf).b_u_seq_cur;
        (*cp_undoinfo).save_b_u_time_cur = (*buf).b_u_time_cur;
        (*cp_undoinfo).save_b_u_save_nr_cur = (*buf).b_u_save_nr_cur;
        (*cp_undoinfo).save_b_u_line_ptr = (*buf).b_u_line_ptr;
        (*cp_undoinfo).save_b_u_line_lnum = (*buf).b_u_line_lnum;
        (*cp_undoinfo).save_b_u_line_colnr = (*buf).b_u_line_colnr;
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_restore_undo(
    mut cp_undoinfo: *const CpUndoInfo,
    mut buf: *mut buf_T,
) {
    unsafe {
        (*buf).b_u_oldhead = (*cp_undoinfo).save_b_u_oldhead;
        (*buf).b_u_newhead = (*cp_undoinfo).save_b_u_newhead;
        (*buf).b_u_curhead = (*cp_undoinfo).save_b_u_curhead;
        (*buf).b_u_numhead = (*cp_undoinfo).save_b_u_numhead;
        (*buf).b_u_seq_last = (*cp_undoinfo).save_b_u_seq_last;
        (*buf).b_u_save_nr_last = (*cp_undoinfo).save_b_u_save_nr_last;
        (*buf).b_u_seq_cur = (*cp_undoinfo).save_b_u_seq_cur;
        (*buf).b_u_time_cur = (*cp_undoinfo).save_b_u_time_cur;
        (*buf).b_u_save_nr_cur = (*cp_undoinfo).save_b_u_save_nr_cur;
        (*buf).b_u_line_ptr = (*cp_undoinfo).save_b_u_line_ptr;
        (*buf).b_u_line_lnum = (*cp_undoinfo).save_b_u_line_lnum;
        (*buf).b_u_line_colnr = (*cp_undoinfo).save_b_u_line_colnr;
        if (*buf).b_u_curhead.is_null() {
            (*buf).b_u_synced = (*cp_undoinfo).save_b_u_synced;
        }
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_prepare(mut cpinfo: *mut CpInfo) {
    unsafe {
        let mut saved_bufs: Set_ptr_t = SET_INIT;
        (*cpinfo).buf_info.capacity = 0 as size_t;
        (*cpinfo).buf_info.size = (*cpinfo).buf_info.capacity;
        (*cpinfo).buf_info.items = ::core::ptr::null_mut::<CpBufInfo>();
        (*cpinfo).win_info.capacity = 0 as size_t;
        (*cpinfo).win_info.size = (*cpinfo).win_info.capacity;
        (*cpinfo).win_info.items = ::core::ptr::null_mut::<CpWinInfo>();
        let mut win: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !win.is_null() {
            let mut buf: *mut buf_T = (*win).w_buffer;
            if (*buf).handle != cmdpreview_bufnr.get() {
                if !set_has_ptr_t(&raw mut saved_bufs, buf as ptr_t) {
                    let mut cp_bufinfo: CpBufInfo = CpBufInfo {
                        buf: ::core::ptr::null_mut::<buf_T>(),
                        save_b_p_ul: 0,
                        save_b_p_ma: 0,
                        save_b_changed: 0,
                        save_b_op_start: pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        },
                        save_b_op_end: pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        },
                        save_changedtick: 0,
                        undo_info: CpUndoInfo {
                            save_b_u_oldhead: ::core::ptr::null_mut::<u_header_T>(),
                            save_b_u_newhead: ::core::ptr::null_mut::<u_header_T>(),
                            save_b_u_curhead: ::core::ptr::null_mut::<u_header_T>(),
                            save_b_u_numhead: 0,
                            save_b_u_synced: false,
                            save_b_u_seq_last: 0,
                            save_b_u_save_nr_last: 0,
                            save_b_u_seq_cur: 0,
                            save_b_u_time_cur: 0,
                            save_b_u_save_nr_cur: 0,
                            save_b_u_line_ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            save_b_u_line_lnum: 0,
                            save_b_u_line_colnr: 0,
                        },
                    };
                    cp_bufinfo.buf = buf;
                    cp_bufinfo.save_b_p_ma = (*buf).b_p_ma;
                    cp_bufinfo.save_b_p_ul = (*buf).b_p_ul;
                    cp_bufinfo.save_b_changed = (*buf).b_changed;
                    cp_bufinfo.save_b_op_start = (*buf).b_op_start;
                    cp_bufinfo.save_b_op_end = (*buf).b_op_end;
                    cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
                    cmdpreview_save_undo(&raw mut cp_bufinfo.undo_info, buf);
                    if (*cpinfo).buf_info.size == (*cpinfo).buf_info.capacity {
                        (*cpinfo).buf_info.capacity = if (*cpinfo).buf_info.capacity != 0 {
                            (*cpinfo).buf_info.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        (*cpinfo).buf_info.items = xrealloc(
                            (*cpinfo).buf_info.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<CpBufInfo>()
                                .wrapping_mul((*cpinfo).buf_info.capacity),
                        ) as *mut CpBufInfo;
                    } else {
                    };
                    let c2rust_fresh15 = (*cpinfo).buf_info.size;
                    (*cpinfo).buf_info.size = (*cpinfo).buf_info.size.wrapping_add(1);
                    *(*cpinfo).buf_info.items.offset(c2rust_fresh15 as isize) = cp_bufinfo;
                    set_put_ptr_t(
                        &raw mut saved_bufs,
                        buf as ptr_t,
                        ::core::ptr::null_mut::<*mut ptr_t>(),
                    );
                    u_clearall(buf);
                    (*buf).b_p_ul = INT_MAX as OptInt;
                }
                let mut cp_wininfo: CpWinInfo = CpWinInfo {
                    win: ::core::ptr::null_mut::<win_T>(),
                    save_w_cursor: pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    },
                    save_viewstate: viewstate_T {
                        vs_curswant: 0,
                        vs_leftcol: 0,
                        vs_skipcol: 0,
                        vs_topline: 0,
                        vs_topfill: 0,
                        vs_botline: 0,
                        vs_empty_rows: 0,
                    },
                    save_w_p_cul: 0,
                    save_w_p_cuc: 0,
                };
                cp_wininfo.win = win;
                cp_wininfo.save_w_cursor = (*win).w_cursor;
                save_viewstate(win, &raw mut cp_wininfo.save_viewstate);
                cp_wininfo.save_w_p_cul = (*win).w_onebuf_opt.wo_cul;
                cp_wininfo.save_w_p_cuc = (*win).w_onebuf_opt.wo_cuc;
                if (*cpinfo).win_info.size == (*cpinfo).win_info.capacity {
                    (*cpinfo).win_info.capacity = if (*cpinfo).win_info.capacity != 0 {
                        (*cpinfo).win_info.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*cpinfo).win_info.items = xrealloc(
                        (*cpinfo).win_info.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<CpWinInfo>()
                            .wrapping_mul((*cpinfo).win_info.capacity),
                    ) as *mut CpWinInfo;
                } else {
                };
                let c2rust_fresh16 = (*cpinfo).win_info.size;
                (*cpinfo).win_info.size = (*cpinfo).win_info.size.wrapping_add(1);
                *(*cpinfo).win_info.items.offset(c2rust_fresh16 as isize) = cp_wininfo;
                (*win).w_onebuf_opt.wo_cul = false_0;
                (*win).w_onebuf_opt.wo_cuc = false_0;
            }
            win = (*win).w_next;
        }
        xfree(saved_bufs.keys as *mut ::core::ffi::c_void);
        xfree(saved_bufs.h.hash as *mut ::core::ffi::c_void);
        saved_bufs = SET_INIT;
        (*cpinfo).save_hls = p_hls.get() != 0;
        (*cpinfo).save_cmdmod = cmdmod.get();
        win_size_save(&raw mut (*cpinfo).save_view);
        save_search_patterns();
        p_hls.set(false_0);
        (*cmdmod.ptr()).cmod_split = 0 as ::core::ffi::c_int;
        (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
        (*cmdmod.ptr()).cmod_flags |= CMOD_NOSWAPFILE as ::core::ffi::c_int;
        u_sync(true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_restore_state(mut cpinfo: *mut CpInfo) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < (*cpinfo).buf_info.size {
            let mut cp_bufinfo: CpBufInfo = *(*cpinfo).buf_info.items.offset(i as isize);
            let mut buf: *mut buf_T = cp_bufinfo.buf;
            (*buf).b_changed = cp_bufinfo.save_b_changed;
            extmark_clear(
                buf,
                cmdpreview_ns.get() as uint32_t,
                0 as ::core::ffi::c_int,
                0 as colnr_T,
                MAXLNUM as ::core::ffi::c_int,
                MAXCOL as ::core::ffi::c_int,
            );
            if (*buf).b_u_seq_cur != cp_bufinfo.undo_info.save_b_u_seq_cur {
                let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut uhp: *mut u_header_T = if !(*buf).b_u_curhead.is_null() {
                    (*buf).b_u_curhead
                } else {
                    (*buf).b_u_newhead
                };
                while !uhp.is_null() {
                    uhp = (*uhp).uh_next.ptr;
                    count += 1;
                }
                let mut aco: aco_save_T = aco_save_T::default();
                aucmd_prepbuf(&raw mut aco, buf);
                if (*curbuf.get()).b_u_synced as ::core::ffi::c_int == false_0 {
                    u_sync(true_0 != 0);
                }
                if !u_undo_and_forget(count, false_0 != 0) {
                    abort();
                }
                aucmd_restbuf(&raw mut aco);
            }
            u_blockfree(buf);
            cmdpreview_restore_undo(&raw mut cp_bufinfo.undo_info, buf);
            (*buf).b_op_start = cp_bufinfo.save_b_op_start;
            (*buf).b_op_end = cp_bufinfo.save_b_op_end;
            if cp_bufinfo.save_changedtick != buf_get_changedtick(buf) {
                buf_set_changedtick(buf, cp_bufinfo.save_changedtick);
            }
            (*buf).b_p_ul = cp_bufinfo.save_b_p_ul;
            (*buf).b_p_ma = cp_bufinfo.save_b_p_ma;
            i = i.wrapping_add(1);
        }
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*cpinfo).win_info.size {
            let mut cp_wininfo: CpWinInfo = *(*cpinfo).win_info.items.offset(i_0 as isize);
            let mut win: *mut win_T = cp_wininfo.win;
            (*win).w_cursor = cp_wininfo.save_w_cursor;
            restore_viewstate(win, &raw mut cp_wininfo.save_viewstate);
            (*win).w_onebuf_opt.wo_cul = cp_wininfo.save_w_p_cul;
            (*win).w_onebuf_opt.wo_cuc = cp_wininfo.save_w_p_cuc;
            update_topline(win);
            i_0 = i_0.wrapping_add(1);
        }
        cmdmod.set((*cpinfo).save_cmdmod);
        p_hls.set((*cpinfo).save_hls as ::core::ffi::c_int);
        restore_search_patterns();
        win_size_restore(&raw mut (*cpinfo).save_view);
        ga_clear(&raw mut (*cpinfo).save_view);
        xfree((*cpinfo).win_info.items as *mut ::core::ffi::c_void);
        (*cpinfo).win_info.capacity = 0 as size_t;
        (*cpinfo).win_info.size = (*cpinfo).win_info.capacity;
        (*cpinfo).win_info.items = ::core::ptr::null_mut::<CpWinInfo>();
        xfree((*cpinfo).buf_info.items as *mut ::core::ffi::c_void);
        (*cpinfo).buf_info.capacity = 0 as size_t;
        (*cpinfo).buf_info.size = (*cpinfo).buf_info.capacity;
        (*cpinfo).buf_info.items = ::core::ptr::null_mut::<CpBufInfo>();
    }
}

pub(crate) unsafe extern "C" fn cmdpreview_may_show(mut _s: *mut CommandLineState) -> bool {
    unsafe {
        let mut cpinfo: CpInfo = CpInfo {
            win_info: C2Rust_Unnamed_50 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<CpWinInfo>(),
            },
            buf_info: C2Rust_Unnamed_49 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<CpBufInfo>(),
            },
            save_hls: false,
            save_cmdmod: cmdmod_T {
                cmod_flags: 0,
                cmod_split: 0,
                cmod_tab: 0,
                cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmod_filter_regmatch: regmatch_T {
                    regprog: ::core::ptr::null_mut::<regprog_T>(),
                    startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    rm_matchcol: 0,
                    rm_ic: false,
                },
                cmod_filter_force: false,
                cmod_verbose: 0,
                cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmod_did_sandbox: 0,
                cmod_verbose_save: 0,
                cmod_save_msg_silent: 0,
                cmod_save_msg_scroll: 0,
                cmod_did_esilent: 0,
            },
            save_view: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        };
        let mut icm_split: bool = false;
        let mut cmdpreview_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut cmdpreview_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut err: Error = Error {
            type_0: kErrorTypeException,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut cmdinfo: CmdParseInfo = CmdParseInfo {
            cmdmod: cmdmod_T {
                cmod_flags: 0,
                cmod_split: 0,
                cmod_tab: 0,
                cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmod_filter_regmatch: regmatch_T {
                    regprog: ::core::ptr::null_mut::<regprog_T>(),
                    startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    rm_matchcol: 0,
                    rm_ic: false,
                },
                cmod_filter_force: false,
                cmod_verbose: 0,
                cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmod_did_sandbox: 0,
                cmod_verbose_save: 0,
                cmod_save_msg_silent: 0,
                cmod_save_msg_scroll: 0,
                cmod_did_esilent: 0,
            },
            magic: C2Rust_Unnamed_21 {
                file: false,
                bar: false,
            },
        };
        let mut cmdpreview_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cmdline: *mut ::core::ffi::c_char = xstrdup((*ccline.ptr()).cmdbuff);
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        (*emsg_off.ptr()) += 1;
        if !parse_cmdline(
            &raw mut cmdline,
            &raw mut ea,
            &raw mut cmdinfo,
            &raw mut errormsg,
        ) {
            (*emsg_off.ptr()) -= 1;
        } else {
            (*emsg_off.ptr()) -= 1;
            if ea.argt & EX_PREVIEW as uint32_t == 0 {
                undo_cmdmod(&raw mut cmdinfo.cmdmod);
            } else {
                cursorcmd();
                cmdline_ui_flush();
                if ea.argt & EX_RANGE as uint32_t != 0 && ea.line1 > ea.line2 {
                    let mut lnum: linenr_T = ea.line1;
                    ea.line1 = ea.line2;
                    ea.line2 = lnum;
                }
                cpinfo = CpInfo {
                    win_info: C2Rust_Unnamed_50 {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<CpWinInfo>(),
                    },
                    buf_info: C2Rust_Unnamed_49 {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<CpBufInfo>(),
                    },
                    save_hls: false,
                    save_cmdmod: cmdmod_T {
                        cmod_flags: 0,
                        cmod_split: 0,
                        cmod_tab: 0,
                        cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        cmod_filter_regmatch: regmatch_T {
                            regprog: ::core::ptr::null_mut::<regprog_T>(),
                            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                            rm_matchcol: 0,
                            rm_ic: false,
                        },
                        cmod_filter_force: false,
                        cmod_verbose: 0,
                        cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        cmod_did_sandbox: 0,
                        cmod_verbose_save: 0,
                        cmod_save_msg_silent: 0,
                        cmod_save_msg_scroll: 0,
                        cmod_did_esilent: 0,
                    },
                    save_view: garray_T {
                        ga_len: 0,
                        ga_maxlen: 0,
                        ga_itemsize: 0,
                        ga_growsize: 0,
                        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    },
                };
                icm_split = *p_icm.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
                cmdpreview_buf = ::core::ptr::null_mut::<buf_T>();
                cmdpreview_win = ::core::ptr::null_mut::<win_T>();
                (*emsg_silent.ptr()) += 1;
                (*msg_silent.ptr()) += 1;
                block_autocmds();
                cmdpreview_prepare(&raw mut cpinfo);
                if icm_split as ::core::ffi::c_int != 0 && {
                    cmdpreview_buf = cmdpreview_open_buf();
                    cmdpreview_buf.is_null()
                } {
                    set_option_direct(
                        kOptInccommand,
                        OptVal {
                            type_0: kOptValTypeString,
                            data: OptValData {
                                string: String_0 {
                                    data: b"nosplit\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                        .wrapping_sub(1 as size_t),
                                },
                            },
                        },
                        0 as ::core::ffi::c_int,
                        SID_NONE,
                    );
                    icm_split = false_0 != 0;
                }
                if cmdpreview_ns.get() == 0 {
                    cmdpreview_ns.set(nvim_create_namespace(String_0 {
                        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0 as size_t,
                    }) as ::core::ffi::c_int);
                }
                cmdpreview.set(true_0 != 0);
                err = Error {
                    type_0: kErrorTypeNone,
                    msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                cmdpreview_type = execute_cmd(&raw mut ea, &raw mut cmdinfo, true);
                try_leave(&raw mut tstate, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    api_clear_error(&raw mut err);
                    cmdpreview_type = 0 as ::core::ffi::c_int;
                }
                if icm_split as ::core::ffi::c_int != 0
                    && cmdpreview_type == 2 as ::core::ffi::c_int
                    && {
                        cmdpreview_win = cmdpreview_open_win(cmdpreview_buf);
                        cmdpreview_win.is_null()
                    }
                {
                    cmdpreview_type = 1 as ::core::ffi::c_int;
                }
                if cmdpreview_type != 0 as ::core::ffi::c_int {
                    let mut save_rd: ::core::ffi::c_int = RedrawingDisabled.get();
                    RedrawingDisabled.set(0 as ::core::ffi::c_int);
                    update_screen();
                    RedrawingDisabled.set(save_rd);
                }
                if icm_split as ::core::ffi::c_int != 0
                    && cmdpreview_type == 2 as ::core::ffi::c_int
                    && !cmdpreview_win.is_null()
                {
                    cmdpreview_close_win();
                }
                cmdpreview_restore_state(&raw mut cpinfo);
                unblock_autocmds();
                (*msg_silent.ptr()) -= 1;
                (*emsg_silent.ptr()) -= 1;
                redrawcmdline();
            }
        }
        xfree(cmdline as *mut ::core::ffi::c_void);
        return cmdpreview_type != 0 as ::core::ffi::c_int;
    }
}
