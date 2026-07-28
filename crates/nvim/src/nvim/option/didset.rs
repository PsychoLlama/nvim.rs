//! The callbacks the option table names for an option that just changed.
//!
//! They are `pub` only so the generated table can name them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_arabic(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if (*win).w_onebuf_opt.wo_arab != 0 {
        if p_tbidi.get() == 0 {
            if (*win).w_onebuf_opt.wo_rl == 0 {
                (*win).w_onebuf_opt.wo_rl = true_0;
                changed_window_setting(win);
            }
            if p_arshape.get() == 0 {
                p_arshape.set(true_0);
                redraw_all_later(UPD_NOT_VALID as c_int);
            }
        }
        if strcmp(p_enc.get(), b"utf-8\0".as_ptr() as *const c_char) != 0 as c_int {
            static w_arabic: GlobalCell<*mut c_char> = GlobalCell::new(
                b"W17: Arabic requires UTF-8, do ':set encoding=utf-8'\0".as_ptr() as *const c_char
                    as *mut c_char,
            );
            msg_source(HLF_W as c_int);
            msg(gettext(w_arabic.get()), HLF_W as c_int);
            set_vim_var_string(VV_WARNINGMSG, gettext(w_arabic.get()), -1 as ptrdiff_t);
        }
        p_deco.set(true_0);
        errmsg = set_option_value(
            kOptKeymap,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"arabic\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as c_int,
        );
    } else {
        if p_tbidi.get() == 0 {
            if (*win).w_onebuf_opt.wo_rl != 0 {
                (*win).w_onebuf_opt.wo_rl = false_0;
                changed_window_setting(win);
            }
        }
        (*(*win).w_buffer).b_p_iminsert = B_IMODE_NONE as OptInt;
        (*(*win).w_buffer).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
    }
    return errmsg;
}

pub unsafe extern "C" fn did_set_autochdir(mut _args: *mut optset_T) -> *const c_char {
    do_autochdir();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_binary(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    set_options_bin(
        (*args).os_oldval.boolean as c_int,
        (*buf).b_p_bin,
        (*args).os_flags,
    );
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_buflisted(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*args).os_oldval.boolean as c_int != (*buf).b_p_bl {
        apply_autocmds(
            (if (*buf).b_p_bl != 0 {
                EVENT_BUFADD as c_int
            } else {
                EVENT_BUFDELETE as c_int
            }) as event_T,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
            buf,
        );
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_cmdheight(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if p_ch.get() > (Rows.get() - min_rows(curtab.get()) + 1 as c_int) as OptInt {
        p_ch.set((Rows.get() - min_rows(curtab.get()) + 1 as c_int) as OptInt);
    }
    if (p_ch.get() != old_value
        || (tabline_height() + global_stl_height() + (*topframe.get()).fr_height) as OptInt
            != Rows.get() as OptInt - p_ch.get())
        && full_screen.get() as c_int != 0
    {
        command_height();
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_diff(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    diff_buf_adjust(win);
    if foldmethodIsDiff(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_eof_eol_fixeol_bomb(mut _args: *mut optset_T) -> *const c_char {
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_equalalways(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if p_ea.get() != 0 && (*args).os_oldval.boolean as u64 == 0 {
        win_equal(win, false_0 != 0, 0 as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldlevel(mut _args: *mut optset_T) -> *const c_char {
    newFoldLevel();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldminlines(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    foldUpdateAll(win);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldnestmax(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if foldmethodIsSyntax(win) as c_int != 0 || foldmethodIsIndent(win) as c_int != 0 {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_helpheight(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) {
        if (*curbuf.get()).b_help as c_int != 0 && ((*curwin.get()).w_height as OptInt) < p_hh.get()
        {
            win_setheight(p_hh.get() as c_int);
        }
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_hlsearch(mut _args: *mut optset_T) -> *const c_char {
    set_no_hlsearch(false_0 != 0);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_ignorecase(mut _args: *mut optset_T) -> *const c_char {
    if p_hls.get() != 0 {
        redraw_all_later(UPD_SOME_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_iminsert(mut _args: *mut optset_T) -> *const c_char {
    showmode();
    status_redraw_curbuf();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_langnoremap(mut _args: *mut optset_T) -> *const c_char {
    p_lrm.set((p_lnr.get() == 0) as c_int);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_langremap(mut _args: *mut optset_T) -> *const c_char {
    p_lnr.set((p_lrm.get() == 0) as c_int);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_laststatus(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if value == 3 as OptInt && old_value != 3 as OptInt {
        frame_new_height(
            topframe.get(),
            (*topframe.get()).fr_height - STATUS_HEIGHT as c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        win_comp_pos();
        clear_cmdline.set(true_0 != 0);
    }
    if old_value == 3 as OptInt && value != 3 as OptInt {
        frame_new_height(
            topframe.get(),
            (*topframe.get()).fr_height + STATUS_HEIGHT as c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        win_comp_pos();
    }
    status_redraw_curbuf();
    last_status(false_0 != 0);
    win_float_update_statusline();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_lines_or_columns(mut args: *mut optset_T) -> *const c_char {
    if p_lines.get() != Rows.get() as OptInt || p_columns.get() != Columns.get() as OptInt {
        if updating_screen.get() {
            let mut oldval: OptVal = OptVal {
                type_0: kOptValTypeNumber,
                data: (*args).os_oldval,
            };
            set_option_varp((*args).os_idx, (*args).os_varp, oldval, false_0 != 0);
        } else if full_screen.get() {
            screen_resize(p_columns.get() as c_int, p_lines.get() as c_int);
        } else {
            Rows.set(p_lines.get() as c_int);
            Columns.set(p_columns.get() as c_int);
            check_screensize();
            let mut new_row: c_int = (Rows.get() as OptInt
                - (if p_ch.get() > 1 as OptInt {
                    p_ch.get()
                } else {
                    1 as OptInt
                })) as c_int;
            if cmdline_row.get() > new_row && Rows.get() as OptInt > p_ch.get() {
                '_c2rust_label: {
                    if p_ch.get() >= 0 as OptInt && new_row <= 2147483647 as c_int {
                    } else {
                        __assert_fail(
                            b"p_ch >= 0 && new_row <= INT_MAX\0".as_ptr() as *const c_char,
                            b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                            2359 as c_uint,
                            b"const char *did_set_lines_or_columns(optset_T *)\0".as_ptr()
                                as *const c_char,
                        );
                    }
                };
                cmdline_row.set(new_row);
            }
        }
        if p_window.get() >= Rows.get() as OptInt || !option_was_set(kOptWindow) {
            p_window.set((Rows.get() - 1 as c_int) as OptInt);
        }
    }
    if p_sj.get() >= Rows.get() as OptInt && full_screen.get() as c_int != 0 {
        p_sj.set((Rows.get() / 2 as c_int) as OptInt);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_lisp(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    buf_init_chartab(buf, false);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_modifiable(mut _args: *mut optset_T) -> *const c_char {
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_modified(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*args).os_newval.boolean as u64 == 0 {
        save_file_ff(buf);
    }
    redraw_titles();
    (*buf).b_modified_was_set = (*args).os_newval.boolean as c_int != 0;
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_number_relativenumber(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if *(*win).w_onebuf_opt.wo_stc as c_int != NUL {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    check_signcolumn(::core::ptr::null_mut::<c_char>(), win);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_numberwidth(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_paste(mut _args: *mut optset_T) -> *const c_char {
    static old_p_paste: GlobalCell<c_int> = GlobalCell::new(false_0);
    static save_sm: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_sta: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_ru: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_ri: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    if p_paste.get() != 0 {
        if old_p_paste.get() == 0 {
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                (*buf).b_p_tw_nopaste = (*buf).b_p_tw;
                (*buf).b_p_wm_nopaste = (*buf).b_p_wm;
                (*buf).b_p_sts_nopaste = (*buf).b_p_sts;
                (*buf).b_p_ai_nopaste = (*buf).b_p_ai;
                (*buf).b_p_et_nopaste = (*buf).b_p_et;
                if !(*buf).b_p_vsts_nopaste.is_null() {
                    xfree((*buf).b_p_vsts_nopaste as *mut c_void);
                }
                (*buf).b_p_vsts_nopaste = if !(*buf).b_p_vsts.is_null()
                    && (*buf).b_p_vsts != empty_string_option.ptr() as *mut c_char
                {
                    xstrdup((*buf).b_p_vsts)
                } else {
                    ::core::ptr::null_mut::<c_char>()
                };
                buf = (*buf).b_next;
            }
            save_sm.set(p_sm.get());
            save_sta.set(p_sta.get());
            save_ru.set(p_ru.get());
            save_ri.set(p_ri.get());
            p_ai_nopaste.set(p_ai.get());
            p_et_nopaste.set(p_et.get());
            p_sts_nopaste.set(p_sts.get());
            p_tw_nopaste.set(p_tw.get());
            p_wm_nopaste.set(p_wm.get());
            if !(*p_vsts_nopaste.ptr()).is_null() {
                xfree(p_vsts_nopaste.get() as *mut c_void);
            }
            p_vsts_nopaste.set(
                if !(*p_vsts.ptr()).is_null()
                    && p_vsts.get() != empty_string_option.ptr() as *mut c_char
                {
                    xstrdup(p_vsts.get())
                } else {
                    ::core::ptr::null_mut::<c_char>()
                },
            );
        }
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            (*buf_0).b_p_tw = 0 as OptInt;
            (*buf_0).b_p_wm = 0 as OptInt;
            (*buf_0).b_p_sts = 0 as OptInt;
            (*buf_0).b_p_ai = 0 as c_int;
            (*buf_0).b_p_et = 0 as c_int;
            if !(*buf_0).b_p_vsts.is_null() {
                free_string_option((*buf_0).b_p_vsts);
            }
            (*buf_0).b_p_vsts = empty_string_option.ptr() as *mut c_char;
            let mut ptr_: *mut *mut c_void = &raw mut (*buf_0).b_p_vsts_array as *mut *mut c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            buf_0 = (*buf_0).b_next;
        }
        p_sm.set(0 as c_int);
        p_sta.set(0 as c_int);
        if p_ru.get() != 0 {
            status_redraw_all();
        }
        p_ru.set(0 as c_int);
        p_ri.set(0 as c_int);
        p_tw.set(0 as OptInt);
        p_wm.set(0 as OptInt);
        p_sts.set(0 as OptInt);
        p_ai.set(0 as c_int);
        p_et.set(0 as c_int);
        if !(*p_vsts.ptr()).is_null() {
            free_string_option(p_vsts.get());
        }
        p_vsts.set(empty_string_option.ptr() as *mut c_char);
    } else if old_p_paste.get() != 0 {
        let mut buf_1: *mut buf_T = firstbuf.get();
        while !buf_1.is_null() {
            (*buf_1).b_p_tw = (*buf_1).b_p_tw_nopaste;
            (*buf_1).b_p_wm = (*buf_1).b_p_wm_nopaste;
            (*buf_1).b_p_sts = (*buf_1).b_p_sts_nopaste;
            (*buf_1).b_p_ai = (*buf_1).b_p_ai_nopaste;
            (*buf_1).b_p_et = (*buf_1).b_p_et_nopaste;
            if !(*buf_1).b_p_vsts.is_null() {
                free_string_option((*buf_1).b_p_vsts);
            }
            (*buf_1).b_p_vsts = if !(*buf_1).b_p_vsts_nopaste.is_null() {
                xstrdup((*buf_1).b_p_vsts_nopaste)
            } else {
                empty_string_option.ptr() as *mut c_char
            };
            xfree((*buf_1).b_p_vsts_array as *mut c_void);
            if !(*buf_1).b_p_vsts.is_null()
                && (*buf_1).b_p_vsts != empty_string_option.ptr() as *mut c_char
            {
                tabstop_set((*buf_1).b_p_vsts, &raw mut (*buf_1).b_p_vsts_array);
            } else {
                (*buf_1).b_p_vsts_array = ::core::ptr::null_mut::<colnr_T>();
            }
            buf_1 = (*buf_1).b_next;
        }
        p_sm.set(save_sm.get());
        p_sta.set(save_sta.get());
        if p_ru.get() != save_ru.get() {
            status_redraw_all();
        }
        p_ru.set(save_ru.get());
        p_ri.set(save_ri.get());
        p_ai.set(p_ai_nopaste.get());
        p_et.set(p_et_nopaste.get());
        p_sts.set(p_sts_nopaste.get());
        p_tw.set(p_tw_nopaste.get());
        p_wm.set(p_wm_nopaste.get());
        if !(*p_vsts.ptr()).is_null() {
            free_string_option(p_vsts.get());
        }
        p_vsts.set(if !(*p_vsts_nopaste.ptr()).is_null() {
            xstrdup(p_vsts_nopaste.get())
        } else {
            empty_string_option.ptr() as *mut c_char
        });
    }
    old_p_paste.set(p_paste.get());
    didset_options_sctx(
        OPT_LOCAL as c_int | OPT_GLOBAL as c_int,
        p_paste_dep_opts.ptr() as *mut c_int,
    );
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_previewwindow(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_pvw == 0 {
        return ::core::ptr::null::<c_char>();
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_onebuf_opt.wo_pvw != 0 && wp != win {
            (*win).w_onebuf_opt.wo_pvw = false_0;
            return (e_preview_window_already_exists.ptr() as *const _) as *const c_char;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_pumblend(mut _args: *mut optset_T) -> *const c_char {
    hl_invalidate_blends();
    if pum_drawn() {
        pum_redraw();
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_readonly(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_ro == 0 && (*args).os_flags & OPT_LOCAL as c_int == 0 as c_int {
        readonlymode.set(false_0 != 0);
    }
    if (*buf).b_p_ro != 0 {
        (*buf).b_did_warn = false_0 != 0;
    }
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_scrollback(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if !(*buf).terminal.is_null() && value < old_value {
        on_scrollback_option_changed((*buf).terminal);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_scrollbind(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_scb == 0 {
        return ::core::ptr::null::<c_char>();
    }
    do_check_scrollbind(false_0 != 0);
    (*win).w_scbind_pos = get_vtopline(win);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_shiftwidth_tabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut pp: *mut OptInt = (*args).os_varp as *mut OptInt;
    if foldmethodIsIndent(win) {
        foldUpdateAll(win);
    }
    if pp == &raw mut (*buf).b_p_sw || (*buf).b_p_sw == 0 as OptInt {
        parse_cino(buf);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_showtabline(mut _args: *mut optset_T) -> *const c_char {
    win_new_screen_rows();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_smoothscroll(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_sms == 0 {
        (*win).w_skipcol = 0 as c_int as colnr_T;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_spell(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_spell != 0 {
        return parse_spelllang(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_swapfile(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_swf != 0 && p_uc.get() != 0 {
        ml_open_file(buf);
    } else {
        mf_close_file(buf, true_0 != 0);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_textwidth(mut _args: *mut optset_T) -> *const c_char {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            check_colorcolumn(::core::ptr::null_mut::<c_char>(), wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_title_icon(mut _args: *mut optset_T) -> *const c_char {
    did_set_title();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_titlelen(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if starting.get() != NO_SCREEN && old_value != p_titlelen.get() {
        need_maketitle.set(true_0 != 0);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_undofile(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_udf == 0 && p_udf.get() == 0 {
        return ::core::ptr::null::<c_char>();
    }
    let mut hash: [uint8_t; 32] = [0; 32];
    let mut bp: *mut buf_T = firstbuf.get();
    while !bp.is_null() {
        if (buf == bp
            || (*args).os_flags & OPT_GLOBAL as c_int != 0
            || (*args).os_flags == 0 as c_int)
            && !bufIsChanged(bp)
            && !(*bp).b_ml.ml_mfp.is_null()
        {
            u_compute_hash(bp, &raw mut hash as *mut uint8_t);
            u_read_undo(
                ::core::ptr::null_mut::<c_char>(),
                &raw mut hash as *mut uint8_t,
                (*bp).b_fname,
            );
        }
        bp = (*bp).b_next;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_global_undolevels(
    mut value: OptInt,
    mut old_value: OptInt,
) -> *const c_char {
    p_ul.set(old_value);
    u_sync(true_0 != 0);
    p_ul.set(value);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_buflocal_undolevels(
    mut buf: *mut buf_T,
    mut value: OptInt,
    mut old_value: OptInt,
) -> *const c_char {
    (*buf).b_p_ul = old_value;
    u_sync(true_0 != 0);
    (*buf).b_p_ul = value;
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_undolevels(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut pp: *mut OptInt = (*args).os_varp as *mut OptInt;
    if pp == p_ul.ptr() {
        did_set_global_undolevels((*args).os_newval.number, (*args).os_oldval.number);
    } else if pp == &raw mut (*buf).b_p_ul {
        did_set_buflocal_undolevels(buf, (*args).os_newval.number, (*args).os_oldval.number);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_updatecount(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if p_uc.get() != 0 && old_value == 0 {
        ml_open_files();
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_wildchar(mut args: *mut optset_T) -> *const c_char {
    let mut c: OptInt = *((*args).os_varp as *mut OptInt);
    if c == Ctrl_C as OptInt
        || c == '\n' as OptInt
        || c == '\r' as OptInt
        || c == K_KENTER as OptInt
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_winblend(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if value != old_value {
        (*win).w_onebuf_opt.wo_winbl = if (if (*win).w_onebuf_opt.wo_winbl < 100 as OptInt {
            (*win).w_onebuf_opt.wo_winbl
        } else {
            100 as OptInt
        }) > 0 as OptInt
        {
            if (*win).w_onebuf_opt.wo_winbl < 100 as OptInt {
                (*win).w_onebuf_opt.wo_winbl
            } else {
                100 as OptInt
            }
        } else {
            0 as OptInt
        };
        (*win).w_hl_needs_update = true_0;
        check_blending(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_window(mut _args: *mut optset_T) -> *const c_char {
    if p_window.get() < 1 as OptInt {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    } else if p_window.get() >= Rows.get() as OptInt {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_winheight(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) {
        if ((*curwin.get()).w_height as OptInt) < p_wh.get() {
            win_setheight(p_wh.get() as c_int);
        }
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_winwidth(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) && ((*curwin.get()).w_width as OptInt) < p_wiw.get() {
        win_setwidth(p_wiw.get() as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_wrap(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_wrap != 0 {
        (*win).w_leftcol = 0 as c_int as colnr_T;
    } else {
        (*win).w_skipcol = 0 as c_int as colnr_T;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_xhistory(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut is_p_chi: bool = (*args).os_varp as *mut OptInt == p_chi.ptr();
    let mut arg: *mut OptInt = if is_p_chi as c_int != 0 {
        p_chi.ptr()
    } else {
        (*args).os_varp as *mut OptInt
    };
    if is_p_chi {
        qf_resize_stack(*arg as c_int);
    } else {
        ll_resize_stack(win, *arg as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub(crate) unsafe extern "C" fn do_syntax_autocmd(mut buf: *mut buf_T, mut value_changed: bool) {
    static syn_recursive: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*syn_recursive.ptr()) += 1;
    (*buf).b_flags |= BF_SYN_SET;
    apply_autocmds(
        EVENT_SYNTAX,
        (*buf).b_p_syn,
        (*buf).b_fname,
        value_changed as c_int != 0 || syn_recursive.get() == 1 as c_int,
        buf,
    );
    (*syn_recursive.ptr()) -= 1;
}

pub(crate) unsafe extern "C" fn do_spelllang_source(mut win: *mut win_T) {
    let mut fname: [c_char; 200] = [0; 200];
    let mut q: *mut c_char = (*(*win).w_s).b_p_spl;
    if strncmp(q, b"cjk,\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
        q = q.offset(4 as c_int as isize);
    }
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    p = q;
    while *p as c_int != NUL {
        if !(*p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || ascii_isdigit(*p as c_int) as c_int != 0)
            && *p as c_int != '-' as c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if p > q {
        vim_snprintf(
            &raw mut fname as *mut c_char,
            ::core::mem::size_of::<[c_char; 200]>(),
            b"spell/%.*s.*\0".as_ptr() as *const c_char,
            p.offset_from(q) as c_int,
            q,
        );
        source_runtime_vim_lua(&raw mut fname as *mut c_char, DIP_ALL as c_int);
    }
}
