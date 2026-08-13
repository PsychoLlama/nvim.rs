//! `do_window()` -- the CTRL-W commands.
//!
//! One `switch` over the letter that follows CTRL-W (or the argument of
//! `:wincmd`), with the count already parsed: split, close, only, exchange,
//! rotate, move to an edge, navigate in a direction, resize, jump to a tag or
//! file under the cursor, open the preview window, and the tab-page forms.
//! [`cmd_with_count`] builds the `:` command line the several letters that
//! delegate to an Ex command need.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::api::private::helpers::api_clear_error;
use crate::src::nvim::autocmd::{EVENT_TABNEWENTERED, apply_autocmds};
use crate::src::nvim::buffer::{
    bt_quickfix, buflist_findname_exp, buflist_findnr, buflist_getfile,
};
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::edit::beginline;
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::ex_getln::curbuf_locked;
use crate::src::nvim::file_search::grab_file_name;
use crate::src::nvim::getchar::{beep_flush, plain_vgetc, typebuf_maplen};
use crate::src::nvim::keycodes::{
    Ctrl__, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_I, Ctrl_J, Ctrl_K,
    Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T, Ctrl_V, Ctrl_W,
    Ctrl_X, Ctrl_Z, K_BS, K_DOWN, K_KENTER, K_LEFT, K_RIGHT, K_UP,
};
use crate::src::nvim::main::{
    Columns, KeyStuffed, KeyTyped, Rows, allow_keys, cmdmod, cmdwin_type, curbuf, curtab, curwin,
    e_buffer_nr_not_found, e_cmdwin, e_noalt, firstwin, g_do_tagpreview, langmap_mapchar, lastwin,
    no_mapping, p_langmap, p_lrm, p_pvh, postponed_split, prevwin, swb_flags, vgetc_busy,
};
use crate::src::nvim::mapping::langmap_adjust_mb;
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memory::{xfree, xmemdupz, xstrlcat, xstrlcpy};
use crate::src::nvim::message::{emsg, msg};
use crate::src::nvim::normal::{
    add_to_showcmd, check_text_or_curbuf_locked, do_nv_ident, find_ident_under_cursor,
    reset_VIsual_and_resel,
};
use crate::src::nvim::options::{kOptSwbFlagUseopen, kOptSwbFlagUsetab};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::quickfix::qf_view_result;
use crate::src::nvim::search::find_pattern_in_path;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    Error, FloatAnchor, VirtText, VirtTextChunk, WinConfig, colnr_T, exarg_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kFloatRelativeEditor, linenr_T, lpos_T, oparg_T, size_t,
    tabpage_T, win_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::winfloat::win_new_float;

pub unsafe extern "C" fn do_window(
    mut nchar: ::core::ffi::c_int,
    mut Prenum: ::core::ffi::c_int,
    mut xchar: ::core::ffi::c_int,
) {
    unsafe {
        let mut config: WinConfig = WinConfig {
            window: 0,
            bufpos: lpos_T { lnum: 0, col: 0 },
            height: 0,
            width: 0,
            row: 0.,
            col: 0.,
            anchor: 0,
            relative: kFloatRelativeEditor,
            external: false,
            focusable: false,
            mouse: false,
            split: kWinSplitLeft,
            zindex: 0,
            style: kWinStyleUnused,
            border: false,
            shadow: false,
            border_chars: [[0; 32]; 8],
            border_hl_ids: [0; 8],
            border_attr: [0; 8],
            title: false,
            title_pos: kAlignLeft,
            title_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            title_width: 0,
            footer: false,
            footer_pos: kAlignLeft,
            footer_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            footer_width: 0,
            noautocmd: false,
            fixed: false,
            hide: false,
            _cmdline_offset: 0,
        };
        let mut err: Error = Error {
            type_0: kErrorTypeException,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut type_0: ::core::ffi::c_int = FIND_DEFINE as ::core::ffi::c_int;
        let mut cbuf: [::core::ffi::c_char; 40] = [0; 40];
        let mut Prenum1: ::core::ffi::c_int = if Prenum == 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            Prenum
        };
        's_1675: {
            '_newwindow: {
                '_wingotofile: {
                    'c_63358: {
                        match nchar {
                            83 | Ctrl_S | 115 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                if bt_quickfix(curbuf.get()) {
                                    break '_newwindow;
                                } else {
                                    win_split(Prenum, 0 as ::core::ffi::c_int);
                                    break 's_1675;
                                }
                            }
                            Ctrl_V | 118 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                if bt_quickfix(curbuf.get()) {
                                    break '_newwindow;
                                } else {
                                    win_split(Prenum, WSP_VERT as ::core::ffi::c_int);
                                    break 's_1675;
                                }
                            }
                            Ctrl_HAT | 94 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                if buflist_findnr(if Prenum == 0 as ::core::ffi::c_int {
                                    (*curwin.get()).w_alt_fnum
                                } else {
                                    Prenum
                                })
                                .is_null()
                                {
                                    if Prenum == 0 as ::core::ffi::c_int {
                                        emsg(gettext(
                                            &raw const e_noalt as *const ::core::ffi::c_char,
                                        ));
                                    } else {
                                        semsg_c!(
                                            gettext(
                                                &raw const e_buffer_nr_not_found
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            Prenum as int64_t,
                                        );
                                    }
                                    break 's_1675;
                                } else {
                                    if !curbuf_locked()
                                        && win_split(
                                            0 as ::core::ffi::c_int,
                                            0 as ::core::ffi::c_int,
                                        ) == OK
                                    {
                                        buflist_getfile(
                                            if Prenum == 0 as ::core::ffi::c_int {
                                                (*curwin.get()).w_alt_fnum
                                            } else {
                                                Prenum
                                            },
                                            0 as linenr_T,
                                            GETF_ALT as ::core::ffi::c_int,
                                            false_0,
                                        );
                                    }
                                    break 's_1675;
                                }
                            }
                            Ctrl_N | 110 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                break '_newwindow;
                            }
                            Ctrl_Q | 113 => {
                                reset_VIsual_and_resel();
                                cmd_with_count(
                                    c"quit".as_ptr() as *mut ::core::ffi::c_char,
                                    &raw mut cbuf as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                    Prenum as int64_t,
                                );
                                do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                                break 's_1675;
                            }
                            Ctrl_C | 99 => {
                                reset_VIsual_and_resel();
                                cmd_with_count(
                                    c"close".as_ptr() as *mut ::core::ffi::c_char,
                                    &raw mut cbuf as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                    Prenum as int64_t,
                                );
                                do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                                break 's_1675;
                            }
                            Ctrl_Z | 122 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                do_cmdline_cmd(c"pclose".as_ptr());
                                break 's_1675;
                            }
                            80 => {
                                let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
                                let mut wp2: *mut win_T = if curtab.get() == curtab.get() {
                                    firstwin.get()
                                } else {
                                    (*curtab.get()).tp_firstwin
                                };
                                while !wp2.is_null() {
                                    if (*wp2).w_onebuf_opt.wo_pvw != 0 {
                                        wp = wp2;
                                        break;
                                    } else {
                                        wp2 = (*wp2).w_next;
                                    }
                                }
                                if wp.is_null() {
                                    emsg(gettext(c"E441: There is no preview window".as_ptr()));
                                } else {
                                    win_goto(wp);
                                }
                                break 's_1675;
                            }
                            Ctrl_O | 111 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                cmd_with_count(
                                    c"only".as_ptr() as *mut ::core::ffi::c_char,
                                    &raw mut cbuf as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                    Prenum as int64_t,
                                );
                                do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                                break 's_1675;
                            }
                            Ctrl_W | 119 | 87 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                if firstwin.get() == lastwin.get()
                                    && Prenum != 1 as ::core::ffi::c_int
                                {
                                    beep_flush();
                                } else {
                                    let mut wp_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
                                    if Prenum != 0 {
                                        let mut last_focusable: *mut win_T = firstwin.get();
                                        wp_0 = firstwin.get();
                                        loop {
                                            Prenum -= 1;
                                            if Prenum <= 0 as ::core::ffi::c_int {
                                                break;
                                            }
                                            if !(*wp_0).w_floating
                                                || !(*wp_0).w_config.hide
                                                    && (*wp_0).w_config.focusable
                                                        as ::core::ffi::c_int
                                                        != 0
                                            {
                                                last_focusable = wp_0;
                                            }
                                            if (*wp_0).w_next.is_null() {
                                                break;
                                            }
                                            wp_0 = (*wp_0).w_next;
                                        }
                                        while !wp_0.is_null()
                                            && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                            && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                                || !(*wp_0).w_config.focusable)
                                        {
                                            wp_0 = (*wp_0).w_next;
                                        }
                                        if wp_0.is_null() {
                                            wp_0 = last_focusable;
                                        }
                                    } else if nchar == 'W' as ::core::ffi::c_int {
                                        wp_0 = (*curwin.get()).w_prev;
                                        if wp_0.is_null() {
                                            wp_0 = lastwin.get();
                                        }
                                        while !wp_0.is_null()
                                            && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                            && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                                || !(*wp_0).w_config.focusable)
                                        {
                                            wp_0 = (*wp_0).w_prev;
                                        }
                                    } else {
                                        wp_0 = (*curwin.get()).w_next;
                                        while !wp_0.is_null()
                                            && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                            && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                                || !(*wp_0).w_config.focusable)
                                        {
                                            wp_0 = (*wp_0).w_next;
                                        }
                                        if wp_0.is_null() {
                                            wp_0 = firstwin.get();
                                        }
                                    }
                                    win_goto(wp_0);
                                }
                                break 's_1675;
                            }
                            106 | K_DOWN | Ctrl_J => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                win_goto_ver(false_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            107 | K_UP | Ctrl_K => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                win_goto_ver(true_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            104 | K_LEFT | Ctrl_H | K_BS => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                win_goto_hor(true_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            108 | K_RIGHT | Ctrl_L => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                win_goto_hor(false_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            84 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
                                    msg(gettext(m_onlyone.get()), 0 as ::core::ffi::c_int);
                                } else {
                                    let mut oldtab: *mut tabpage_T = curtab.get();
                                    let mut wp_1: *mut win_T = curwin.get();
                                    if !win_new_tabpage(
                                        Prenum,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<*mut win_T>(),
                                    )
                                    .is_null()
                                        && valid_tabpage(oldtab) as ::core::ffi::c_int != 0
                                    {
                                        let mut newtab: *mut tabpage_T = curtab.get();
                                        goto_tabpage_tp(oldtab, true_0 != 0, true_0 != 0);
                                        if curwin.get() == wp_1 {
                                            win_close(curwin.get(), false_0 != 0, false_0 != 0);
                                        }
                                        if valid_tabpage(newtab) {
                                            goto_tabpage_tp(newtab, true_0 != 0, true_0 != 0);
                                            apply_autocmds(
                                                EVENT_TABNEWENTERED,
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                false_0 != 0,
                                                curbuf.get(),
                                            );
                                        }
                                    }
                                }
                                break 's_1675;
                            }
                            116 | Ctrl_T => {
                                win_goto(firstwin.get());
                                break 's_1675;
                            }
                            98 | Ctrl_B => {
                                win_goto(lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()));
                                break 's_1675;
                            }
                            112 | Ctrl_P => {
                                if !win_valid(prevwin.get())
                                    || (*prevwin.get()).w_config.hide as ::core::ffi::c_int != 0
                                    || !(*prevwin.get()).w_config.focusable
                                {
                                    beep_flush();
                                } else {
                                    win_goto(prevwin.get());
                                }
                                break 's_1675;
                            }
                            120 | Ctrl_X => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                win_exchange(Prenum);
                                break 's_1675;
                            }
                            Ctrl_R | 114 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                win_rotate(false_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            82 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                reset_VIsual_and_resel();
                                win_rotate(true_0 != 0, Prenum1);
                                break 's_1675;
                            }
                            75 | 74 | 72 | 76 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
                                    beep_flush();
                                } else {
                                    let dir: ::core::ffi::c_int =
                                        (if nchar == 'H' as ::core::ffi::c_int
                                            || nchar == 'L' as ::core::ffi::c_int
                                        {
                                            WSP_VERT as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }) | (if nchar == 'H' as ::core::ffi::c_int
                                            || nchar == 'K' as ::core::ffi::c_int
                                        {
                                            WSP_TOP as ::core::ffi::c_int
                                        } else {
                                            WSP_BOT as ::core::ffi::c_int
                                        });
                                    win_splitmove(curwin.get(), Prenum, dir);
                                }
                                break 's_1675;
                            }
                            61 => {
                                let mut mod_0: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_split
                                    & (WSP_VERT as ::core::ffi::c_int
                                        | WSP_HOR as ::core::ffi::c_int);
                                win_equal(
                                    ::core::ptr::null_mut::<win_T>(),
                                    false_0 != 0,
                                    if mod_0 == WSP_VERT as ::core::ffi::c_int {
                                        'v' as ::core::ffi::c_int
                                    } else if mod_0 == WSP_HOR as ::core::ffi::c_int {
                                        'h' as ::core::ffi::c_int
                                    } else {
                                        'b' as ::core::ffi::c_int
                                    },
                                );
                                break 's_1675;
                            }
                            43 => {
                                win_setheight((*curwin.get()).w_height + Prenum1);
                                break 's_1675;
                            }
                            45 => {
                                win_setheight((*curwin.get()).w_height - Prenum1);
                                break 's_1675;
                            }
                            Ctrl__ | 95 => {
                                win_setheight(if Prenum != 0 {
                                    Prenum
                                } else {
                                    Rows.get() - min_set_ch.get() as ::core::ffi::c_int
                                });
                                break 's_1675;
                            }
                            62 => {
                                win_setwidth((*curwin.get()).w_width + Prenum1);
                                break 's_1675;
                            }
                            60 => {
                                win_setwidth((*curwin.get()).w_width - Prenum1);
                                break 's_1675;
                            }
                            124 => {
                                win_setwidth(if Prenum != 0 as ::core::ffi::c_int {
                                    Prenum
                                } else {
                                    Columns.get()
                                });
                                break 's_1675;
                            }
                            125 => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                if Prenum != 0 {
                                    g_do_tagpreview.set(Prenum);
                                } else {
                                    g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
                                }
                                break 'c_63358;
                            }
                            93 | Ctrl_RSB => {
                                break 'c_63358;
                            }
                            102 | 70 | Ctrl_F => {
                                break '_wingotofile;
                            }
                            105 | Ctrl_I => {
                                type_0 = FIND_ANY as ::core::ffi::c_int;
                            }
                            100 | Ctrl_D => {}
                            K_KENTER | CAR => {
                                if bt_quickfix(curbuf.get()) {
                                    qf_view_result(true_0 != 0);
                                }
                                break 's_1675;
                            }
                            103 | Ctrl_G => {
                                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        &raw const e_cmdwin as *const ::core::ffi::c_char,
                                    ));
                                    return;
                                }
                                (*no_mapping.ptr()) += 1;
                                (*allow_keys.ptr()) += 1;
                                if xchar == NUL {
                                    xchar = plain_vgetc();
                                }
                                if *p_langmap.get() as ::core::ffi::c_int != 0
                                    && true
                                    && (p_lrm.get() != 0
                                        || (if vgetc_busy.get() != 0 {
                                            (typebuf_maplen() == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else {
                                            KeyTyped.get() as ::core::ffi::c_int
                                        }) != 0)
                                    && KeyStuffed.get() == 0
                                    && xchar >= 0 as ::core::ffi::c_int
                                {
                                    if xchar < 256 as ::core::ffi::c_int {
                                        xchar = (*langmap_mapchar.ptr())[xchar as usize]
                                            as ::core::ffi::c_int;
                                    } else {
                                        xchar = langmap_adjust_mb(xchar);
                                    }
                                }
                                (*no_mapping.ptr()) -= 1;
                                (*allow_keys.ptr()) -= 1;
                                add_to_showcmd(xchar);
                                match xchar {
                                    125 => {
                                        xchar = Ctrl_RSB;
                                        if Prenum != 0 {
                                            g_do_tagpreview.set(Prenum);
                                        } else {
                                            g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
                                        }
                                    }
                                    93 | Ctrl_RSB => {}
                                    102 | 70 => {
                                        (*cmdmod.ptr()).cmod_tab =
                                            tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                                        nchar = xchar;
                                        break '_wingotofile;
                                    }
                                    116 => {
                                        goto_tabpage(Prenum);
                                        break 's_1675;
                                    }
                                    84 => {
                                        goto_tabpage(-Prenum1);
                                        break 's_1675;
                                    }
                                    TAB => {
                                        if !goto_tabpage_lastused() {
                                            beep_flush();
                                        }
                                        break 's_1675;
                                    }
                                    101 => {
                                        if (*curwin.get()).w_floating as ::core::ffi::c_int != 0
                                            || !ui_has(kUIMultigrid)
                                        {
                                            beep_flush();
                                            break 's_1675;
                                        } else {
                                            config = WinConfig {
                                                window: 0,
                                                bufpos: lpos_T {
                                                    lnum: -1 as linenr_T,
                                                    col: 0 as colnr_T,
                                                },
                                                height: 0 as ::core::ffi::c_int,
                                                width: 0 as ::core::ffi::c_int,
                                                row: 0 as ::core::ffi::c_int
                                                    as ::core::ffi::c_double,
                                                col: 0 as ::core::ffi::c_int
                                                    as ::core::ffi::c_double,
                                                anchor: 0 as FloatAnchor,
                                                relative: kFloatRelativeEditor,
                                                external: false_0 != 0,
                                                focusable: true_0 != 0,
                                                mouse: true_0 != 0,
                                                split: kWinSplitLeft,
                                                zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                                                style: kWinStyleUnused,
                                                border: false,
                                                shadow: false,
                                                border_chars: [[0; 32]; 8],
                                                border_hl_ids: [0; 8],
                                                border_attr: [0; 8],
                                                title: false,
                                                title_pos: kAlignLeft,
                                                title_chunks: VirtText {
                                                    size: 0,
                                                    capacity: 0,
                                                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                                                },
                                                title_width: 0,
                                                footer: false,
                                                footer_pos: kAlignLeft,
                                                footer_chunks: VirtText {
                                                    size: 0,
                                                    capacity: 0,
                                                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                                                },
                                                footer_width: 0,
                                                noautocmd: false_0 != 0,
                                                fixed: false_0 != 0,
                                                hide: false_0 != 0,
                                                _cmdline_offset: INT_MAX,
                                            };
                                            config.width = (*curwin.get()).w_width;
                                            config.height = (*curwin.get()).w_height;
                                            config.external = true_0 != 0;
                                            err = Error {
                                                type_0: kErrorTypeNone,
                                                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            };
                                            if win_new_float(
                                                curwin.get(),
                                                false_0 != 0,
                                                config,
                                                &raw mut err,
                                            )
                                            .is_null()
                                            {
                                                emsg(err.msg);
                                                api_clear_error(&raw mut err);
                                                beep_flush();
                                            }
                                            break 's_1675;
                                        }
                                    }
                                    _ => {
                                        beep_flush();
                                        break 's_1675;
                                    }
                                }
                                if Prenum != 0 {
                                    postponed_split.set(Prenum);
                                } else {
                                    postponed_split.set(-1 as ::core::ffi::c_int);
                                }
                                do_nv_ident('g' as ::core::ffi::c_int, xchar);
                                postponed_split.set(0 as ::core::ffi::c_int);
                                break 's_1675;
                            }
                            _ => {
                                beep_flush();
                                break 's_1675;
                            }
                        }
                        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                            emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                            return;
                        }
                        let mut len: size_t = 0;
                        let mut ptr_0: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        len = find_ident_under_cursor(
                            &raw mut ptr_0,
                            FIND_IDENT as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                        if len == 0 as size_t {
                            break 's_1675;
                        } else {
                            ptr_0 = xmemdupz(ptr_0 as *const ::core::ffi::c_void, len)
                                as *mut ::core::ffi::c_char;
                            find_pattern_in_path(
                                ptr_0,
                                kDirectionNotSet,
                                len,
                                true_0 != 0,
                                Prenum == 0 as ::core::ffi::c_int,
                                type_0,
                                Prenum1,
                                ACTION_SPLIT as ::core::ffi::c_int,
                                1 as linenr_T,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                false_0 != 0,
                                false_0 != 0,
                            );
                            xfree(ptr_0 as *mut ::core::ffi::c_void);
                            (*curwin.get()).w_set_curswant = true_0;
                            break 's_1675;
                        }
                    }
                    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                        emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                        return;
                    }
                    if Prenum != 0 {
                        postponed_split.set(Prenum);
                    } else {
                        postponed_split.set(-1 as ::core::ffi::c_int);
                    }
                    if nchar != '}' as ::core::ffi::c_int {
                        g_do_tagpreview.set(0 as ::core::ffi::c_int);
                    }
                    do_nv_ident(Ctrl_RSB, NUL);
                    postponed_split.set(0 as ::core::ffi::c_int);
                    break 's_1675;
                }
                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                    return;
                }
                if check_text_or_curbuf_locked(::core::ptr::null_mut::<oparg_T>()) {
                    break 's_1675;
                } else {
                    let mut lnum: linenr_T = -1 as linenr_T;
                    let mut ptr: *mut ::core::ffi::c_char = grab_file_name(Prenum1, &raw mut lnum);
                    if !ptr.is_null() {
                        let mut oldtab_0: *mut tabpage_T = curtab.get();
                        let mut oldwin: *mut win_T = curwin.get();
                        setpcmark();
                        let mut wp_2: *mut win_T = ::core::ptr::null_mut::<win_T>();
                        if swb_flags.get()
                            & (kOptSwbFlagUseopen as ::core::ffi::c_int
                                | kOptSwbFlagUsetab as ::core::ffi::c_int)
                                as ::core::ffi::c_uint
                            != 0
                            && (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int
                        {
                            wp_2 = swbuf_goto_win_with_buf(buflist_findname_exp(ptr));
                        }
                        if wp_2.is_null()
                            && win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == OK
                        {
                            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                            if do_ecmd(
                                0 as ::core::ffi::c_int,
                                ptr,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<exarg_T>(),
                                ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                                ECMD_HIDE as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<win_T>(),
                            ) == FAIL
                            {
                                win_close(curwin.get(), false_0 != 0, false_0 != 0);
                                goto_tabpage_win(oldtab_0, oldwin);
                            } else {
                                wp_2 = curwin.get();
                            }
                        }
                        if !wp_2.is_null()
                            && nchar == 'F' as ::core::ffi::c_int
                            && lnum >= 0 as linenr_T
                        {
                            (*curwin.get()).w_cursor.lnum = lnum;
                            check_cursor_lnum(curwin.get());
                            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                        }
                        xfree(ptr as *mut ::core::ffi::c_void);
                    }
                    break 's_1675;
                }
            }
            if Prenum != 0 {
                vim_snprintf(
                    &raw mut cbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>().wrapping_sub(5 as size_t),
                    c"%ld".as_ptr(),
                    Prenum as int64_t,
                );
            } else {
                cbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            }
            if nchar == 'v' as ::core::ffi::c_int || nchar == Ctrl_V {
                xstrlcat(
                    &raw mut cbuf as *mut ::core::ffi::c_char,
                    c"v".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                );
            }
            xstrlcat(
                &raw mut cbuf as *mut ::core::ffi::c_char,
                c"new".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
            );
            do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
        };
    }
}

unsafe extern "C" fn cmd_with_count(
    mut cmd: *mut ::core::ffi::c_char,
    mut bufp: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut Prenum: int64_t,
) {
    unsafe {
        let mut len: size_t = xstrlcpy(bufp, cmd, bufsize);
        if Prenum > 0 as int64_t && len < bufsize {
            vim_snprintf(
                bufp.add(len),
                bufsize.wrapping_sub(len),
                c"%ld".as_ptr(),
                Prenum,
            );
        }
    }
}
