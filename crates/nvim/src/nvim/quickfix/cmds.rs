//! `:cfile`, `:cbuffer`, `:cexpr` and their variants.
//!
//! Each takes lines from somewhere other than a command — a file
//! ([`ex_cfile`]), a buffer range ([`ex_cbuffer`]) or the value of a
//! Vimscript expression ([`ex_cexpr`]) — parses them with `'errorformat'`
//! and either replaces or adds to a list. The `*_get_auname` helpers name
//! the `QuickFixCmdPre`/`QuickFixCmdPost` autocommand each one fires.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cfile_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        65 => {
            return b"cfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        68 => {
            return b"cgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        51 => {
            return b"caddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        233 => {
            return b"lfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        236 => {
            return b"lgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        218 => {
            return b"laddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}

pub unsafe fn ex_cfile(mut eap: *mut exarg_T) {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5343 as ::core::ffi::c_uint,
                    b"void ex_cfile(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut au_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        au_name = cfile_get_auname((*eap).cmdidx);
        if !au_name.is_null()
            && apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                au_name,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0
        {
            if aborting() {
                return;
            }
        }
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*eap).arg),
                    },
                },
                0 as ::core::ffi::c_int,
                0 as scid_T,
            );
        }
        let mut enc: *mut ::core::ffi::c_char =
            if *(*curbuf.get()).b_p_menc as ::core::ffi::c_int != NUL {
                (*curbuf.get()).b_p_menc
            } else {
                p_menc.get()
            };
        if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
            wp = curwin.get();
        }
        incr_quickfix_busy();
        let mut res: ::core::ffi::c_int = qf_init(
            wp,
            p_ef.get(),
            p_efm.get(),
            ((*eap).cmdidx as ::core::ffi::c_int != CMD_caddfile as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddfile as ::core::ffi::c_int)
                as ::core::ffi::c_int,
            qf_cmdtitle(*(*eap).cmdlinep),
            enc,
        );
        if !wp.is_null() {
            qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null()
            {
                (*wp).w_llist_ref
            } else {
                (*wp).w_llist
            };
            if qi.is_null() {
                decr_quickfix_busy();
                return;
            }
        }
        if res >= 0 as ::core::ffi::c_int {
            qf_list_changed(qf_get_curlist(qi));
        }
        let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
        if !au_name.is_null() {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        if res > 0 as ::core::ffi::c_int
            && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cfile as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfile as ::core::ffi::c_int)
            && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
        {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
    }
}

pub(crate) unsafe extern "C" fn cbuffer_get_auname(
    mut cmdidx: cmdidx_T,
) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        55 => {
            return b"cbuffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        69 => {
            return b"cgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        49 => {
            return b"caddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        221 => {
            return b"lbuffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        237 => {
            return b"lgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        217 => {
            return b"laddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}

pub(crate) unsafe extern "C" fn cbuffer_process_args(
    mut eap: *mut exarg_T,
    mut bufp: *mut *mut buf_T,
    mut line1: *mut linenr_T,
    mut line2: *mut linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if *(*eap).arg as ::core::ffi::c_int == NUL {
            buf = curbuf.get();
        } else if *skipwhite(skipdigits((*eap).arg)) as ::core::ffi::c_int == NUL {
            buf = buflist_findnr(atoi((*eap).arg));
        }
        if buf.is_null() {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return FAIL;
        }
        if (*buf).b_ml.ml_mfp.is_null() {
            emsg(gettext(
                &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
            (*eap).line2 = (*buf).b_ml.ml_line_count;
        }
        if (*eap).line1 < 1 as linenr_T
            || (*eap).line1 > (*buf).b_ml.ml_line_count
            || (*eap).line2 < 1 as linenr_T
            || (*eap).line2 > (*buf).b_ml.ml_line_count
        {
            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            return FAIL;
        }
        *line1 = (*eap).line1;
        *line2 = (*eap).line2;
        *bufp = buf;
        return OK;
    }
}

pub unsafe fn ex_cbuffer(mut eap: *mut exarg_T) {
    unsafe {
        let mut au_name: *mut ::core::ffi::c_char = cbuffer_get_auname((*eap).cmdidx);
        if !au_name.is_null()
            && apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0
        {
            if aborting() {
                return;
            }
        }
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut line1: linenr_T = 0;
        let mut line2: linenr_T = 0;
        if cbuffer_process_args(eap, &raw mut buf, &raw mut line1, &raw mut line2) == FAIL {
            return;
        }
        let mut qf_title: *mut ::core::ffi::c_char = qf_cmdtitle(*(*eap).cmdlinep);
        if !(*buf).b_sfname.is_null() {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%s (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                qf_title,
                (*buf).b_sfname,
            );
            qf_title = IObuff.ptr() as *mut ::core::ffi::c_char;
        }
        incr_quickfix_busy();
        let mut res: ::core::ffi::c_int = qf_init_ext(
            qi,
            (*qi).qf_curlist,
            ::core::ptr::null::<::core::ffi::c_char>(),
            buf,
            ::core::ptr::null_mut::<typval_T>(),
            p_efm.get(),
            (*eap).cmdidx as ::core::ffi::c_int != CMD_caddbuffer as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddbuffer as ::core::ffi::c_int,
            (*eap).line1,
            (*eap).line2,
            qf_title,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        if qf_stack_empty(qi) {
            decr_quickfix_busy();
            return;
        }
        if res >= 0 as ::core::ffi::c_int {
            qf_list_changed(qf_get_curlist(qi));
        }
        let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
        if !au_name.is_null() {
            let curbuf_old: *const buf_T = curbuf.get();
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
            if curbuf.get() != curbuf_old as *mut buf_T {
                res = 0 as ::core::ffi::c_int;
            }
        }
        if res > 0 as ::core::ffi::c_int
            && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cbuffer as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbuffer as ::core::ffi::c_int)
            && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
        {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
    }
}

pub(crate) unsafe extern "C" fn cexpr_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        64 => {
            return b"cexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        70 => {
            return b"cgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        50 => {
            return b"caddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        232 => {
            return b"lexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        238 => {
            return b"lgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        216 => {
            return b"laddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}

pub(crate) unsafe extern "C" fn trigger_cexpr_autocmd(
    mut cmdidx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut au_name: *mut ::core::ffi::c_char = cexpr_get_auname(cmdidx as cmdidx_T);
        if !au_name.is_null()
            && apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0
        {
            if aborting() {
                return FAIL;
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn cexpr_core(
    mut eap: *const exarg_T,
    mut tv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*tv).vval.v_string.is_null()
            || (*tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut au_name: *mut ::core::ffi::c_char = cexpr_get_auname((*eap).cmdidx);
            incr_quickfix_busy();
            let mut res: ::core::ffi::c_int = qf_init_ext(
                qi,
                (*qi).qf_curlist,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<buf_T>(),
                tv,
                p_efm.get(),
                (*eap).cmdidx as ::core::ffi::c_int != CMD_caddexpr as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_laddexpr as ::core::ffi::c_int,
                0 as linenr_T,
                0 as linenr_T,
                qf_cmdtitle(*(*eap).cmdlinep),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
            if qf_stack_empty(qi) {
                decr_quickfix_busy();
                return FAIL;
            }
            if res >= 0 as ::core::ffi::c_int {
                qf_list_changed(qf_get_curlist(qi));
            }
            let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
            if !au_name.is_null() {
                apply_autocmds(
                    EVENT_QUICKFIXCMDPOST,
                    au_name,
                    (*curbuf.get()).b_fname,
                    true_0 != 0,
                    curbuf.get(),
                );
            }
            if res > 0 as ::core::ffi::c_int
                && ((*eap).cmdidx as ::core::ffi::c_int == CMD_cexpr as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_lexpr as ::core::ffi::c_int)
                && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
            {
                qf_jump_first(qi, save_qfid, (*eap).forceit);
            }
            decr_quickfix_busy();
            return OK;
        } else {
            emsg(gettext(
                b"E777: String or List expected\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return FAIL;
    }
}

pub unsafe fn ex_cexpr(mut eap: *mut exarg_T) {
    unsafe {
        if trigger_cexpr_autocmd((*eap).cmdidx as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut tv: *mut typval_T = eval_expr((*eap).arg, eap);
        if tv.is_null() {
            return;
        }
        cexpr_core(eap, tv);
        tv_free(tv);
    }
}
