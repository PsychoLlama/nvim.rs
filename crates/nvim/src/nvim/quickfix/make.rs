//! `:make` and `:grep`, which run an external command.
//!
//! [`ex_make`] builds the command line from `'makeprg'`/`'grepprg'`, runs
//! it with its output redirected to a temporary file ([`get_mef_name`]) and
//! then reads that file as an error file. `:grep` with
//! `'grepprg'` set to `internal` is handled by `:vimgrep` instead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn grep_internal(mut cmdidx: cmdidx_T) -> ::core::ffi::c_int {
    unsafe {
        return ((cmdidx as ::core::ffi::c_int == CMD_grep as ::core::ffi::c_int
            || cmdidx as ::core::ffi::c_int == CMD_lgrep as ::core::ffi::c_int
            || cmdidx as ::core::ffi::c_int == CMD_grepadd as ::core::ffi::c_int
            || cmdidx as ::core::ffi::c_int == CMD_lgrepadd as ::core::ffi::c_int)
            && strcmp(
                b"internal\0".as_ptr() as *const ::core::ffi::c_char,
                if *(*curbuf.get()).b_p_gp as ::core::ffi::c_int == NUL {
                    p_gp.get()
                } else {
                    (*curbuf.get()).b_p_gp
                },
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn make_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        274 => {
            return b"make\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        248 => {
            return b"lmake\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        172 => {
            return b"grep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        239 => {
            return b"lgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        173 => {
            return b"grepadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        240 => {
            return b"lgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}

pub(crate) unsafe extern "C" fn make_get_fullcmd(
    mut makecmd: *const ::core::ffi::c_char,
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = strlen(p_shq.get())
            .wrapping_mul(2 as size_t)
            .wrapping_add(strlen(makecmd))
            .wrapping_add(1 as size_t);
        if *p_sp.get() as ::core::ffi::c_int != NUL {
            len = len.wrapping_add(
                strlen(p_sp.get())
                    .wrapping_add(strlen(fname))
                    .wrapping_add(3 as size_t),
            );
        }
        let cmd: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        snprintf(
            cmd,
            len,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            p_shq.get(),
            makecmd,
            p_shq.get(),
        );
        if *p_sp.get() as ::core::ffi::c_int != NUL {
            append_redir(cmd, len, p_sp.get(), fname);
        }
        if msg_col.get() == 0 as ::core::ffi::c_int {
            msg_didout.set(false_0 != 0);
        }
        msg_start();
        msg_puts(b":!\0".as_ptr() as *const ::core::ffi::c_char);
        msg_outtrans(cmd, 0 as ::core::ffi::c_int, false_0 != 0);
        return cmd;
    }
}

pub unsafe fn ex_make(mut eap: *mut exarg_T) {
    unsafe {
        let mut save_qfid: ::core::ffi::c_uint = 0;
        let mut enc: *mut ::core::ffi::c_char =
            if *(*curbuf.get()).b_p_menc as ::core::ffi::c_int != NUL {
                (*curbuf.get()).b_p_menc
            } else {
                p_menc.get()
            };
        if grep_internal((*eap).cmdidx) != 0 {
            ex_vimgrep(eap);
            return;
        }
        let au_name: *mut ::core::ffi::c_char = make_get_auname((*eap).cmdidx);
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
        if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
            wp = curwin.get();
        }
        autowrite_all();
        let mut fname: *mut ::core::ffi::c_char = get_mef_name();
        if fname.is_null() {
            return;
        }
        os_remove(fname);
        let cmd: *mut ::core::ffi::c_char = make_get_fullcmd((*eap).arg, fname);
        do_shell(cmd, 0 as ::core::ffi::c_int);
        incr_quickfix_busy();
        let mut errorformat: *mut ::core::ffi::c_char = if (*eap).cmdidx as ::core::ffi::c_int
            != CMD_make as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_lmake as ::core::ffi::c_int
        {
            if *(*curbuf.get()).b_p_gefm as ::core::ffi::c_int != NUL {
                (*curbuf.get()).b_p_gefm
            } else {
                p_gefm.get()
            }
        } else {
            p_efm.get()
        };
        let mut newlist: bool = (*eap).cmdidx as ::core::ffi::c_int
            != CMD_grepadd as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrepadd as ::core::ffi::c_int;
        let mut res: ::core::ffi::c_int = qf_init(
            wp,
            fname,
            errorformat,
            newlist as ::core::ffi::c_int,
            qf_cmdtitle(*(*eap).cmdlinep),
            enc,
        );
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4655 as ::core::ffi::c_uint,
                    b"void ex_make(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        '_cleanup: {
            if !wp.is_null() {
                qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                    && !(*wp).w_llist_ref.is_null()
                {
                    (*wp).w_llist_ref
                } else {
                    (*wp).w_llist
                };
                if qi.is_null() {
                    break '_cleanup;
                }
            }
            if res >= 0 as ::core::ffi::c_int {
                qf_list_changed(qf_get_curlist(qi));
            }
            save_qfid = (*qf_get_curlist(qi)).qf_id;
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
                && (*eap).forceit == 0
                && qflist_valid(wp, save_qfid) as ::core::ffi::c_int != 0
            {
                qf_jump_first(qi, save_qfid, false_0);
            }
        }
        decr_quickfix_busy();
        os_remove(fname);
        xfree(fname as *mut ::core::ffi::c_void);
        xfree(cmd as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn get_mef_name() -> *mut ::core::ffi::c_char {
    unsafe {
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        static start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
        static off: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if *p_mef.get() as ::core::ffi::c_int == NUL {
            name = vim_tempname();
            if name.is_null() {
                emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
            }
            return name;
        }
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = p_mef.get();
        while *p != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '#' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            return xstrdup(p_mef.get());
        }
        loop {
            if start.get() == -1 as ::core::ffi::c_int {
                start.set(os_get_pid() as ::core::ffi::c_int);
            } else {
                (*off.ptr()) += 19 as ::core::ffi::c_int;
            }
            name =
                xmalloc(strlen(p_mef.get()).wrapping_add(30 as size_t)) as *mut ::core::ffi::c_char;
            strcpy(name, p_mef.get());
            snprintf(
                name.offset(p.offset_from(p_mef.get()) as isize),
                strlen(name),
                b"%d%d\0".as_ptr() as *const ::core::ffi::c_char,
                start.get(),
                off.get(),
            );
            strcat(name, p.offset(2 as ::core::ffi::c_int as isize));
            let mut file_info: FileInfo = FileInfo {
                stat: uv_stat_t {
                    st_dev: 0,
                    st_mode: 0,
                    st_nlink: 0,
                    st_uid: 0,
                    st_gid: 0,
                    st_rdev: 0,
                    st_ino: 0,
                    st_size: 0,
                    st_blksize: 0,
                    st_blocks: 0,
                    st_flags: 0,
                    st_gen: 0,
                    st_atim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_mtim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_ctim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    st_birthtim: uv_timespec_t {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                },
            };
            let mut file_or_link_found: bool = os_fileinfo_link(name, &raw mut file_info);
            if !file_or_link_found {
                break;
            }
            xfree(name as *mut ::core::ffi::c_void);
        }
        return name;
    }
}
