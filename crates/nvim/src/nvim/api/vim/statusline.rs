//! `nvim_eval_statusline()`: rendering a statusline expression.
//!
//! The longest function in the module, because a statusline is evaluated
//! against a *window* with a fill character, a maximum width and an
//! optional statuscolumn line number, and because the `highlights` option
//! makes it report every group boundary in the result as well as the text.
//! `nvim__complete_set` shares the window plumbing.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, dict_put, has_key};

pub unsafe extern "C" fn nvim_eval_statusline(
    mut str: String_0,
    mut opts: *mut KeyDict_eval_statusline,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut result: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut maxwidth: ::core::ffi::c_int = 0;
        let mut fillchar: schar_T = 0 as schar_T;
        let mut statuscol_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if str.size < 2 as size_t
            || memcmp(
                str.data as *const ::core::ffi::c_void,
                c"%!".as_ptr() as *const ::core::ffi::c_void,
                2 as size_t,
            ) != 0 as ::core::ffi::c_int
        {
            let errmsg: *const ::core::ffi::c_char = check_stl_option(str.data);
            if !errmsg.is_null() {
                api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), errmsg);
                return result;
            }
        }
        let mut window: Window = (*opts).winid;
        if has_key(
            (*opts).is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__fillchar,
        ) {
            if !(*(*opts).fillchar.data as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                && utfc_ptr2len((*opts).fillchar.data) as size_t == (*opts).fillchar.size)
            {
                api_err_exp(
                    err,
                    c"fillchar".as_ptr(),
                    c"single character".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return result;
            }
            let mut c: ::core::ffi::c_int = 0;
            fillchar = utfc_ptr2schar((*opts).fillchar.data, &raw mut c);
        }
        let mut use_bools: ::core::ffi::c_int =
            (*opts).use_winbar as ::core::ffi::c_int + (*opts).use_tabline as ::core::ffi::c_int;
        let mut wp: *mut win_T = if (*opts).use_tabline as ::core::ffi::c_int != 0 {
            curwin.get()
        } else {
            find_window_by_handle(window, err)
        };
        if wp.is_null() {
            api_set_error(
                err,
                kErrorTypeException,
                c"unknown winid %d".as_ptr(),
                window,
            );
            return result;
        }
        if has_key(
            (*opts).is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum,
        ) {
            statuscol_lnum = (*opts).use_statuscol_lnum as ::core::ffi::c_int;
            if !(statuscol_lnum > 0 as ::core::ffi::c_int
                && statuscol_lnum as linenr_T <= (*(*wp).w_buffer).b_ml.ml_line_count)
            {
                api_err_invalid(
                    err,
                    c"use_statuscol_lnum".as_ptr(),
                    c"out of range".as_ptr(),
                    0 as int64_t,
                    false,
                );
                return result;
            }
            use_bools += 1;
        }
        if !(use_bools <= 1 as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Can only use one of 'use_winbar', 'use_tabline' and 'use_statuscol_lnum'"
                    .as_ptr(),
            );
            return result;
        }
        let mut stc_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut scl_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut statuscol: statuscol_T = statuscol_T {
            width: 0 as ::core::ffi::c_int,
            lnum: 0,
            sign_cul_id: 0,
            draw: false,
            hlrec: ::core::ptr::null_mut::<stl_hlrec_t>(),
            foldinfo: foldinfo_T {
                fi_lnum: 0,
                fi_level: 0,
                fi_low_level: 0,
                fi_lines: 0,
            },
            fold_vcol: [0; 9],
            sattrs: ::core::ptr::null_mut::<SignTextAttrs>(),
        };
        let mut sattrs: [SignTextAttrs; 9] = [
            SignTextAttrs {
                text: [0 as schar_T, 0],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
            SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            },
        ];
        if statuscol_lnum != 0 {
            let mut line_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut cul_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut num_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut lnum: linenr_T = statuscol_lnum as linenr_T;
            let mut cursorline_fi: foldinfo_T = foldinfo_T {
                fi_lnum: 0 as linenr_T,
                fi_level: 0,
                fi_low_level: 0,
                fi_lines: 0,
            };
            decor_redraw_signs(
                wp,
                (*wp).w_buffer,
                lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                &raw mut sattrs as *mut SignTextAttrs,
                &raw mut line_id,
                &raw mut cul_id,
                &raw mut num_id,
            );
            statuscol.sattrs = &raw mut sattrs as *mut SignTextAttrs;
            statuscol.foldinfo = fold_info(wp, lnum);
            win_update_cursorline(wp, &raw mut cursorline_fi);
            statuscol.sign_cul_id =
                if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
                    cul_id
                } else {
                    0 as ::core::ffi::c_int
                };
            scl_hl_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
                HLF_CLS
            } else {
                HLF_SC
            };
            if num_id != 0 {
                stc_hl_id = num_id;
            } else if use_cursor_line_highlight(wp, lnum) {
                stc_hl_id = HLF_CLN;
            } else if (*wp).w_onebuf_opt.wo_rnu != 0 {
                stc_hl_id = if lnum < (*wp).w_cursor.lnum {
                    HLF_LNA
                } else {
                    HLF_LNB
                };
            } else {
                stc_hl_id = HLF_N;
            }
            set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
            set_vim_var_nr(
                VV_RELNUM,
                labs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_long) as varnumber_T,
            );
            set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
        } else if fillchar == 0 as schar_T && !(*opts).use_tabline {
            if (*opts).use_winbar {
                fillchar = (*wp).w_p_fcs_chars.wbr;
            } else {
                let mut group: hlf_T = HLF_NONE;
                fillchar = fillchar_status(&raw mut group, wp);
            }
        }
        if has_key(
            (*opts).is_set__eval_statusline_,
            KEYSET_OPTIDX_eval_statusline__maxwidth,
        ) {
            maxwidth = (*opts).maxwidth as ::core::ffi::c_int;
        } else {
            maxwidth = if statuscol_lnum != 0 {
                win_col_off(wp)
            } else if (*opts).use_tabline as ::core::ffi::c_int != 0
                || !(*opts).use_winbar && global_stl_height() > 0 as ::core::ffi::c_int
            {
                Columns.get()
            } else {
                (*wp).w_width
            };
        }
        result = arena_dict(arena, 3 as size_t);
        let mut buf: *mut ::core::ffi::c_char =
            arena_alloc(arena, MAXPATHL as size_t, false) as *mut ::core::ffi::c_char;
        let mut hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
        let mut hltab_len: size_t = 0 as size_t;
        let mut p_crb_save: ::core::ffi::c_int = (*wp).w_onebuf_opt.wo_crb;
        (*wp).w_onebuf_opt.wo_crb = false_0;
        let mut width: ::core::ffi::c_int = build_stl_str_hl(
            wp,
            buf,
            MAXPATHL as size_t,
            str.data,
            kOptInvalid,
            0 as ::core::ffi::c_int,
            fillchar,
            maxwidth,
            if (*opts).highlights as ::core::ffi::c_int != 0 {
                &raw mut hltab
            } else {
                ::core::ptr::null_mut::<*mut stl_hlrec_t>()
            },
            &raw mut hltab_len,
            ::core::ptr::null_mut::<*mut StlClickRecord>(),
            if statuscol_lnum != 0 {
                &raw mut statuscol
            } else {
                ::core::ptr::null_mut::<statuscol_T>()
            },
        );
        dict_put(&mut result, c"width", Object::integer(width as Integer));
        (*wp).w_onebuf_opt.wo_crb = p_crb_save;
        if (*opts).highlights {
            let mut hl_values: Array = arena_array(arena, hltab_len.wrapping_add(1 as size_t));
            let mut user_group: [::core::ffi::c_char; 15] = [0; 15];
            let mut dfltname: *const ::core::ffi::c_char = get_default_stl_hl(
                if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                    ::core::ptr::null_mut::<win_T>()
                } else {
                    wp
                },
                (*opts).use_winbar,
                stc_hl_id,
            );
            if (*hltab).start.is_null() || (*hltab).start.offset_from(buf) != 0 {
                let mut hl_info: Dict = arena_dict(arena, 3 as size_t);
                dict_put(&mut hl_info, c"start", Object::integer(0 as Integer));
                dict_put(
                    &mut hl_info,
                    c"group",
                    Object::string(cstr_as_string(dfltname)),
                );
                let mut groups: Array = arena_array(arena, 1 as size_t);
                array_add(&mut groups, Object::string(cstr_as_string(dfltname)));
                dict_put(&mut hl_info, c"groups", Object::array(groups));
                array_add(&mut hl_values, Object::dict(hl_info));
            }
            let mut sp: *mut stl_hlrec_t = hltab;
            while !(*sp).start.is_null() {
                let mut grpname: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                if (*sp).userhl == 0 as ::core::ffi::c_int {
                    grpname = get_default_stl_hl(
                        if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                            ::core::ptr::null_mut::<win_T>()
                        } else {
                            wp
                        },
                        (*opts).use_winbar,
                        stc_hl_id,
                    );
                } else if (*sp).userhl < 0 as ::core::ffi::c_int {
                    grpname = syn_id2name(-(*sp).userhl);
                } else {
                    snprintf(
                        &raw mut user_group as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 15]>(),
                        c"User%d".as_ptr(),
                        (*sp).userhl,
                    );
                    grpname = arena_strdup(arena, &raw mut user_group as *mut ::core::ffi::c_char);
                }
                let mut combine: *const ::core::ffi::c_char = if (*sp).item as ::core::ffi::c_uint
                    == STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    syn_id2name(scl_hl_id) as *const ::core::ffi::c_char
                } else if (*sp).item as ::core::ffi::c_uint
                    == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    grpname
                } else {
                    dfltname
                };
                let mut hl_info_0: Dict = arena_dict(arena, 3 as size_t);
                dict_put(
                    &mut hl_info_0,
                    c"start",
                    Object::integer((*sp).start.offset_from(buf) as i64),
                );
                dict_put(
                    &mut hl_info_0,
                    c"group",
                    Object::string(cstr_as_string(grpname)),
                );
                let mut groups_0: Array = arena_array(
                    arena,
                    (1 as ::core::ffi::c_int + (combine != grpname) as ::core::ffi::c_int)
                        as size_t,
                );
                if combine != grpname {
                    array_add(&mut groups_0, Object::string(cstr_as_string(combine)));
                }
                array_add(&mut groups_0, Object::string(cstr_as_string(grpname)));
                dict_put(&mut hl_info_0, c"groups", Object::array(groups_0));
                array_add(&mut hl_values, Object::dict(hl_info_0));
                sp = sp.offset(1);
            }
            dict_put(&mut result, c"highlights", Object::array(hl_values));
        }
        dict_put(&mut result, c"str", Object::string(cstr_as_string(buf)));
        return result;
    }
}

pub unsafe extern "C" fn nvim__complete_set(
    mut index: Integer,
    mut opts: *mut KeyDict_complete_set,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut rv: Dict = arena_dict(arena, 2 as size_t);
        if get_cot_flags() & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
        {
            api_set_error(
                err,
                kErrorTypeException,
                c"completeopt option does not include popup".as_ptr(),
            );
            return rv;
        }
        if has_key(
            (*opts).is_set__complete_set_,
            KEYSET_OPTIDX_complete_set__info,
        ) {
            let mut wp: *mut win_T = pum_set_info(index as ::core::ffi::c_int, (*opts).info.data);
            if !wp.is_null() {
                dict_put(&mut rv, c"winid", Object::window((*wp).handle));
                dict_put(&mut rv, c"bufnr", Object::buffer((*(*wp).w_buffer).handle));
            }
        }
        return rv;
    }
}
