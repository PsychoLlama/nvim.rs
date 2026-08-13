//! Reading a file into a buffer -- `open_buffer()` and the scratch forms.
//!
//! [`open_buffer`] is what turns an empty `buf_T` into one with text: read the
//! file (or stdin), set `'filetype'` and run the `BufRead`/`BufNewFile`
//! autocommands, initialise undo and the swap file, and mark the buffer
//! loaded.  [`buf_open_scratch`] and [`read_buffer_into`] are the two forms
//! that skip the file entirely, and [`buf_contents_changed`] re-reads a file
//! into a hidden dummy buffer so it can be compared with what is in
//! memory.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, EVENT_BUFWINENTER, EVENT_STDINREADPOST,
    apply_autocmds, apply_autocmds_retval, aucmd_prepbuf, aucmd_restbuf, block_autocmds,
    unblock_autocmds,
};
use crate::src::nvim::change::{changed, save_file_ff, unchanged};
use crate::src::nvim::charset::buf_init_chartab;
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::fileio::{prep_exarg, readfile};
use crate::src::nvim::fold::foldUpdateAll;
use crate::src::nvim::help::get_local_additions;
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::main::{
    curbuf, curwin, firstbuf, getout, got_int, p_cpo, readonlymode, v_dying,
};
use crate::src::nvim::memfile::MfDirty;
use crate::src::nvim::memline::{ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_open};
use crate::src::nvim::memory::{xfree, xrealloc};
use crate::src::nvim::message::emsg;
use crate::src::nvim::option::{set_option_value_give_err, shortmess};
use crate::src::nvim::options::{kOptBufhidden, kOptBuftype, kOptSwapfile};
use crate::src::nvim::os::fs::os_getperm;
use crate::src::nvim::os::libc::{gettext, memcpy, strcmp};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    CMD_append, OptInt, OptVal, OptValData, String_0, StringBuilder, aco_save_T, buf_T, bufref_T,
    colnr_T, cstack_T, exarg_T, handle_T, int64_t, kFalse, linenr_T, size_t, win_T,
};
use crate::src::nvim::window::check_colorcolumn;

pub unsafe extern "C" fn calc_percentage(
    mut part: int64_t,
    mut whole: int64_t,
) -> ::core::ffi::c_int {
    return if part > 1000000 as int64_t {
        (part / (whole / 100 as int64_t)) as ::core::ffi::c_int
    } else {
        (part * 100 as int64_t / whole) as ::core::ffi::c_int
    };
}

pub unsafe extern "C" fn get_highest_fnum() -> ::core::ffi::c_int {
    return top_file_num.get() - 1 as ::core::ffi::c_int;
}

unsafe extern "C" fn read_buffer(
    mut read_stdin: bool,
    mut eap: *mut exarg_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = OK;
        let mut silent: bool = shortmess(SHM_FILEINFO as ::core::ffi::c_int);
        let mut line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
        retval = readfile(
            if read_stdin as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                (*curbuf.get()).b_ffname
            },
            if read_stdin as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                (*curbuf.get()).b_fname
            },
            line_count,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            eap,
            flags | READ_BUFFER as ::core::ffi::c_int,
            silent,
        );
        if retval == OK {
            loop {
                line_count -= 1;
                if line_count < 0 as linenr_T {
                    break;
                }
                ml_delete(1 as linenr_T);
            }
        } else {
            while (*curbuf.get()).b_ml.ml_line_count > line_count {
                ml_delete(line_count);
            }
        }
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        if read_stdin {
            if !readonlymode.get() && !buf_is_empty(curbuf.get()) {
                changed(curbuf.get());
            } else if retval != FAIL {
                unchanged(curbuf.get(), false_0 != 0, true_0 != 0);
            }
            apply_autocmds_retval(
                EVENT_STDINREADPOST,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
                &raw mut retval,
            );
        }
        return retval;
    }
}

pub unsafe extern "C" fn buf_ensure_loaded(mut buf: *mut buf_T) -> bool {
    unsafe {
        if !(*buf).b_ml.ml_mfp.is_null() {
            return true_0 != 0;
        }
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, buf);
        let mut status: ::core::ffi::c_int = open_buffer(
            false_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        );
        aucmd_restbuf(&raw mut aco);
        return status != FAIL;
    }
}

pub unsafe extern "C" fn open_buffer(
    mut read_stdin: bool,
    mut eap: *mut exarg_T,
    mut flags_arg: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut flags: ::core::ffi::c_int = flags_arg;
        let mut retval: ::core::ffi::c_int = OK;
        let mut old_curbuf: bufref_T = bufref_T::default();
        let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
        let mut read_fifo: bool = false_0 != 0;
        let mut silent: bool = shortmess(SHM_FILEINFO as ::core::ffi::c_int);
        if readonlymode.get() as ::core::ffi::c_int != 0
            && !(*curbuf.get()).b_ffname.is_null()
            && (*curbuf.get()).b_flags & BF_NEVERLOADED != 0
        {
            (*curbuf.get()).b_p_ro = true_0;
        }
        if ml_open(curbuf.get()) == FAIL {
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                curbuf.get(),
                0 as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
            curbuf.set(::core::ptr::null_mut::<buf_T>());
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                if !(*buf).b_ml.ml_mfp.is_null() {
                    curbuf.set(buf);
                    break;
                } else {
                    buf = (*buf).b_next;
                }
            }
            if (*curbuf.ptr()).is_null() {
                emsg(gettext(
                    c"E82: Cannot allocate any buffer, exiting...".as_ptr(),
                ));
                v_dying.set(2 as ::core::ffi::c_int);
                getout(2 as ::core::ffi::c_int);
            }
            emsg(gettext(
                c"E83: Cannot allocate buffer, using other one...".as_ptr(),
            ));
            enter_buffer(curbuf.get());
            if old_tw != (*curbuf.get()).b_p_tw {
                check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
            }
            return FAIL;
        }
        if !(*curbuf.get()).b_ml.ml_mfp.is_null() {
            (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty = MfDirty::YesNoSync;
        }
        set_bufref(&raw mut old_curbuf, curbuf.get());
        (*curbuf.get()).b_modified_was_set = false_0 != 0;
        (*curwin.get()).w_valid = 0 as ::core::ffi::c_int;
        if bt_nofileread(curbuf.get()) {
            flags |= READ_NOFILE as ::core::ffi::c_int;
        }
        if !(*curbuf.get()).b_ffname.is_null() {
            let mut save_bin: ::core::ffi::c_int = (*curbuf.get()).b_p_bin;
            let mut perm: ::core::ffi::c_int =
                os_getperm((*curbuf.get()).b_ffname) as ::core::ffi::c_int;
            if perm >= 0 as ::core::ffi::c_int
                && (false
                    || perm & __S_IFMT == 0o10000 as ::core::ffi::c_int
                    || perm & __S_IFMT == 0o140000 as ::core::ffi::c_int)
            {
                read_fifo = true_0 != 0;
            }
            if read_fifo {
                (*curbuf.get()).b_p_bin = true_0;
            }
            retval = readfile(
                (*curbuf.get()).b_ffname,
                (*curbuf.get()).b_fname,
                0 as linenr_T,
                0 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                eap,
                flags
                    | READ_NEW as ::core::ffi::c_int
                    | (if read_fifo as ::core::ffi::c_int != 0 {
                        READ_FIFO as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                silent,
            );
            if read_fifo {
                (*curbuf.get()).b_p_bin = save_bin;
                if retval == OK {
                    retval = read_buffer(false_0 != 0, eap, flags);
                }
            }
            if bt_help(curbuf.get()) {
                get_local_additions();
            }
        } else if read_stdin {
            let mut save_bin_0: ::core::ffi::c_int = (*curbuf.get()).b_p_bin;
            (*curbuf.get()).b_p_bin = true_0;
            retval = readfile(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as linenr_T,
                0 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                ::core::ptr::null_mut::<exarg_T>(),
                flags | READ_NEW as ::core::ffi::c_int + READ_STDIN as ::core::ffi::c_int,
                silent,
            );
            (*curbuf.get()).b_p_bin = save_bin_0;
            if retval == OK {
                retval = read_buffer(true_0 != 0, eap, flags);
            }
        }
        if !(*curbuf.get()).b_ml.ml_mfp.is_null()
            && (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty == MfDirty::YesNoSync
        {
            (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty = MfDirty::Yes;
        }
        if (*curbuf.get()).b_flags & BF_NEVERLOADED != 0 {
            buf_init_chartab(curbuf.get(), false);
            parse_cino(curbuf.get());
        }
        if got_int.get() as ::core::ffi::c_int != 0
            && !vim_strchr(p_cpo.get(), CPO_INTMOD).is_null()
            || (*curbuf.get()).b_modified_was_set as ::core::ffi::c_int != 0
            || aborting() as ::core::ffi::c_int != 0
                && !vim_strchr(p_cpo.get(), CPO_INTMOD).is_null()
        {
            changed(curbuf.get());
        } else if retval != FAIL && !read_stdin && !read_fifo {
            unchanged(curbuf.get(), false_0 != 0, true_0 != 0);
        }
        save_file_ff(curbuf.get());
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
        (*curbuf.get()).b_last_changedtick_pum = buf_get_changedtick(curbuf.get());
        if aborting() {
            (*curbuf.get()).b_flags |= BF_READERR;
        }
        foldUpdateAll(curwin.get());
        if (*curwin.get()).w_valid & VALID_TOPLINE == 0 {
            (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
            (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
        }
        apply_autocmds_retval(
            EVENT_BUFENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
            &raw mut retval,
        );
        if retval == FAIL {
            return retval;
        }
        if bufref_valid(&raw mut old_curbuf) as ::core::ffi::c_int != 0
            && !(*old_curbuf.br_buf).b_ml.ml_mfp.is_null()
        {
            let mut aco: aco_save_T = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, old_curbuf.br_buf);
            do_modelines(0 as ::core::ffi::c_int);
            (*curbuf.get()).b_flags &= !(BF_CHECK_RO | BF_NEVERLOADED);
            if flags & READ_NOWINENTER as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                apply_autocmds_retval(
                    EVENT_BUFWINENTER,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                    &raw mut retval,
                );
            }
            aucmd_restbuf(&raw mut aco);
        }
        return retval;
    }
}

pub unsafe extern "C" fn buf_contents_changed(mut buf: *mut buf_T) -> bool {
    unsafe {
        let mut differ: bool = true_0 != 0;
        let mut newbuf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            BLN_DUMMY as ::core::ffi::c_int,
        );
        if newbuf.is_null() {
            return true_0 != 0;
        }
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
        prep_exarg(&raw mut ea, buf);
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, newbuf);
        block_autocmds();
        if ml_open(curbuf.get()) == OK
            && readfile(
                (*buf).b_ffname,
                (*buf).b_fname,
                0 as linenr_T,
                0 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                &raw mut ea,
                READ_NEW as ::core::ffi::c_int | READ_DUMMY as ::core::ffi::c_int,
                false_0 != 0,
            ) == OK
        {
            if (*buf).b_ml.ml_line_count == (*curbuf.get()).b_ml.ml_line_count {
                differ = false_0 != 0;
                let mut lnum: linenr_T = 1 as linenr_T;
                while lnum <= (*curbuf.get()).b_ml.ml_line_count {
                    if strcmp(ml_get_buf(buf, lnum), ml_get(lnum)) != 0 as ::core::ffi::c_int {
                        differ = true_0 != 0;
                        break;
                    } else {
                        lnum += 1;
                    }
                }
            }
        }
        xfree(ea.cmd as *mut ::core::ffi::c_void);
        aucmd_restbuf(&raw mut aco);
        if curbuf.get() != newbuf {
            wipe_buffer(newbuf, false_0 != 0);
        }
        unblock_autocmds();
        return differ;
    }
}

pub unsafe extern "C" fn buf_open_scratch(
    mut bufnr: handle_T,
    mut bufname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if do_ecmd(
            bufnr,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_ONE as ::core::ffi::c_int as linenr_T,
            ECMD_HIDE as ::core::ffi::c_int,
            ::core::ptr::null_mut::<win_T>(),
        ) == FAIL
        {
            return FAIL;
        }
        if !bufname.is_null() {
            apply_autocmds(
                EVENT_BUFFILEPRE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            setfname(
                curbuf.get(),
                bufname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                true_0 != 0,
            );
            apply_autocmds(
                EVENT_BUFFILEPOST,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        set_option_value_give_err(
            kOptBufhidden,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: c"hide".as_ptr() as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        set_option_value_give_err(
            kOptBuftype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: c"nofile".as_ptr() as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        set_option_value_give_err(
            kOptSwapfile,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        return OK;
    }
}

pub unsafe extern "C" fn read_buffer_into(
    mut buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    mut sb: *mut StringBuilder,
) {
    unsafe {
        debug_assert!(!buf.is_null(), "buf");
        debug_assert!(!sb.is_null(), "sb");
        if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
            return;
        }
        let mut written: size_t = 0 as size_t;
        let mut len: size_t = 0 as size_t;
        let mut lnum: linenr_T = start;
        let mut lp: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
        let mut lplen: size_t = ml_get_buf_len(buf, lnum) as size_t;
        loop {
            if lplen == 0 as size_t {
                len = 0 as size_t;
            } else if *lp.add(written) as ::core::ffi::c_int == NL {
                len = 1 as size_t;
                if (*sb).size == (*sb).capacity {
                    (*sb).capacity = if (*sb).capacity != 0 {
                        (*sb).capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*sb).items = xrealloc(
                        (*sb).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*sb).capacity),
                    ) as *mut ::core::ffi::c_char;
                } else {
                };
                let c2rust_fresh7 = (*sb).size;
                (*sb).size = (*sb).size.wrapping_add(1);
                *(*sb).items.add(c2rust_fresh7) = '\0' as ::core::ffi::c_char;
            } else {
                let mut s: *mut ::core::ffi::c_char = vim_strchr(lp.add(written), NL);
                len = if s.is_null() {
                    lplen.wrapping_sub(written)
                } else {
                    s.offset_from(lp.add(written)) as size_t
                };
                if len > 0 as size_t {
                    if (*sb).capacity < (*sb).size.wrapping_add(len) {
                        (*sb).capacity = (*sb).size.wrapping_add(len);
                        (*sb).capacity = (*sb).capacity.wrapping_sub(1);
                        (*sb).capacity |= (*sb).capacity >> 1 as ::core::ffi::c_int;
                        (*sb).capacity |= (*sb).capacity >> 2 as ::core::ffi::c_int;
                        (*sb).capacity |= (*sb).capacity >> 4 as ::core::ffi::c_int;
                        (*sb).capacity |= (*sb).capacity >> 8 as ::core::ffi::c_int;
                        (*sb).capacity |= (*sb).capacity >> 16 as ::core::ffi::c_int;
                        (*sb).capacity = (*sb).capacity.wrapping_add(1);
                        (*sb).items = xrealloc(
                            (*sb).items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul((*sb).capacity),
                        ) as *mut ::core::ffi::c_char;
                    }
                    assert!(!(*sb).items.is_null(), "(*sb).items");
                    memcpy(
                        (*sb).items.add((*sb).size) as *mut ::core::ffi::c_void,
                        lp.add(written) as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(len),
                    );
                    (*sb).size = (*sb).size.wrapping_add(len);
                }
            }
            if len == lplen.wrapping_sub(written) {
                if lnum != end
                    || (*buf).b_p_bin == 0 && (*buf).b_p_fixeol != 0
                    || lnum != (*buf).b_no_eol_lnum
                        && (lnum != (*buf).b_ml.ml_line_count || (*buf).b_p_eol != 0)
                {
                    if (*sb).size == (*sb).capacity {
                        (*sb).capacity = if (*sb).capacity != 0 {
                            (*sb).capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        (*sb).items = xrealloc(
                            (*sb).items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul((*sb).capacity),
                        ) as *mut ::core::ffi::c_char;
                    } else {
                    };
                    let c2rust_fresh8 = (*sb).size;
                    (*sb).size = (*sb).size.wrapping_add(1);
                    *(*sb).items.add(c2rust_fresh8) = '\n' as ::core::ffi::c_char;
                }
                lnum += 1;
                if lnum > end {
                    break;
                }
                lp = ml_get_buf(buf, lnum);
                lplen = ml_get_buf_len(buf, lnum) as size_t;
                written = 0 as size_t;
            } else if len > 0 as size_t {
                written = written.wrapping_add(len);
            }
        }
    }
}
