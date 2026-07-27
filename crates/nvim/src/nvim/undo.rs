use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::buffer::{bt_dontwrite, bt_prompt, buf_is_empty};
use crate::src::nvim::buffer_updates::{buf_updates_changedtick, buf_updates_unload};
use crate::src::nvim::change::{
    change_warning, changed, changed_bytes, changed_lines, file_ff_differs, unchanged,
};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_cursor_lnum, check_pos, coladvance, getviscol,
};
use crate::src::nvim::drawscreen::{redraw_later, redrawWinline};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::funcs::get_buf_arg;
use crate::src::nvim::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_alloc, tv_dict_alloc_ret, tv_get_string,
    tv_list_alloc, tv_list_append_dict,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_docmd::expr_map_locked;
use crate::src::nvim::ex_getln::{text_locked, text_locked_msg};
use crate::src::nvim::extmark::{extmark_apply_undo, extmark_splice_cols};
use crate::src::nvim::fileio::{get2c, get4c, get8ctime, read_eintr};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::garray::{ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    IObuff, KeyTyped, VIsual, VIsual_active, curbuf, curtab, curwin, e_modifiable, e_sandbox,
    e_textlock, fdo_flags, firstbuf, firstwin, global_busy, got_int, no_u_sync, p_cpo, p_fs,
    p_udir, p_ul, p_verbose, sandbox, textlock,
};
use crate::src::nvim::mark::{free_fmark, mark_adjust, setpcmark};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{
    ml_append_flags, ml_delete, ml_get, ml_get_buf, ml_replace, resolve_symlink,
};
use crate::src::nvim::memory::{
    time_to_bytes, xcalloc, xfree, xmalloc, xmallocz, xrealloc, xstrdup, xstrlcat,
};
use crate::src::nvim::message::{
    emsg, give_warning, iemsg, internal_error, messaging, msg, msg_end, msg_ext_set_kind,
    msg_putchar, msg_puts, msg_puts_hl, msg_start, semsg, smsg, smsg_keep, verb_msg, verbose_enter,
    verbose_leave,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::fs::{
    os_fchown, os_fileinfo, os_fopen, os_free_acl, os_fsync, os_get_acl, os_getperm, os_isdir,
    os_mkdir_recurse, os_open, os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::{
    abort, close, fclose, fdopen, fflush, fread, fwrite, getc, gettext, getuid, memcmp, memmove,
    memset, ngettext, strcmp, strftime, strlen, time,
};
use crate::src::nvim::os::time::{os_localtime_r, os_time, tm_zeroed};
use crate::src::nvim::path::{FullName_save, concat_fnames, path_tail, vim_ispathsep};
use crate::src::nvim::sha256::{SHA256_SUM_SIZE, Sha256};
use crate::src::nvim::spell::spell_check_window;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::{sort_strings, vim_snprintf, vim_snprintf_add, vim_strchr};
use crate::src::nvim::types::*;
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

/// Constants the transpiler copied in from the headers this module includes.
mod header {
    use super::{ExtmarkOp, ListLenSpecials, UndoObjectType, VarType, c_int, c_ulong};

    pub const NUL: c_int = 0;
    pub const OK: c_int = 1;
    pub const FAIL: c_int = 0;
    pub const EOF: c_int = -1;
    pub const SIZE_MAX: c_ulong = 18446744073709551615;
    pub const MAXLNUM: c_int = 2147483647;
    pub const MAXPATHL: c_int = 4096;
    pub const NMARKS: c_int = 26;

    /// `open` flags.
    pub const O_RDONLY: c_int = 0;
    pub const O_WRONLY: c_int = 0o1;
    pub const O_CREAT: c_int = 0o100;
    pub const O_EXCL: c_int = 0o200;
    pub const O_NOFOLLOW: c_int = 0o400000;

    /// `ml_append`/`ml_delete` flags.
    pub const ML_EMPTY: c_int = 0x1;

    /// `beginline` flags.
    pub const BL_SOL: c_int = 2;
    pub const BL_FIX: c_int = 4;

    /// The `u_header_T::uh_flags` bits.
    pub const UH_CHANGED: c_int = 1;
    pub const UH_EMPTYBUF: c_int = 2;
    pub const UH_RELOAD: c_int = 4;

    pub const UPD_NOT_VALID: c_int = 40;
    pub const HLF_T: c_int = 23;
    pub const kOptFdoFlagUndo: c_int = 512;
    pub const kListLenMayKnow: ListLenSpecials = -3;
    pub const kExtmarkNOOP: ExtmarkOp = 0;
    pub const kExtmarkUndo: ExtmarkOp = 1;
    pub const kExtmarkSplice: UndoObjectType = 0;
    pub const kExtmarkMove: UndoObjectType = 1;
    pub const VAR_UNKNOWN: VarType = 0;
    pub const VAR_STRING: VarType = 2;
}
use header::*;

mod apply;
mod eval;
mod file;
pub mod format;
mod read;
mod tree;
mod write;

use tree::*;

pub use apply::{u_redo, u_undo, u_undo_and_forget, undo_time};
pub use eval::{ex_undolist, f_undofile, f_undotree, u_force_get_undo_header};
pub use file::{u_compute_hash, u_get_undo_file_name};
pub use read::u_read_undo;
pub use tree::{u_blockfree, u_clearall, u_clearallandblockfree, u_clearline, u_undoline};
pub use write::u_write_undo;

/// The length of an undo file's buffer hash, in bytes: a SHA-256 digest.
pub const UNDO_HASH_SIZE: c_int = 32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufinfo_T {
    pub bi_buf: *mut buf_T,
    pub bi_fp: *mut FILE,
}
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0;
    (*a).col = 0;
    (*a).coladd = 0;
}
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const IOSIZE: c_int = 1024 + 1;
pub const CPO_UNDO: c_int = 'u' as c_int;
pub const NO_LOCAL_UNDOLEVEL: c_int = -123456;
static u_newcount: GlobalCell<c_int> = GlobalCell::new(0);
static u_oldcount: GlobalCell<c_int> = GlobalCell::new(0);
static undo_undoes: GlobalCell<bool> = GlobalCell::new(false);
static lastmark: GlobalCell<c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn u_save_cursor() -> c_int {
    let mut cur: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut top: linenr_T = if cur > 0 { cur - 1 } else { 0 };
    let mut bot: linenr_T = cur + 1;
    return u_save(top, bot);
}
pub unsafe extern "C" fn u_save(mut top: linenr_T, mut bot: linenr_T) -> c_int {
    return u_save_buf(curbuf.get(), top, bot);
}
pub unsafe extern "C" fn u_save_buf(
    mut buf: *mut buf_T,
    mut top: linenr_T,
    mut bot: linenr_T,
) -> c_int {
    if top >= bot || bot > (*buf).b_ml.ml_line_count + 1 {
        return FAIL;
    }
    if top + 2 == bot {
        u_saveline(buf, top + 1);
    }
    return u_savecommon(buf, top, bot, 0, false);
}
pub unsafe extern "C" fn u_savesub(mut lnum: linenr_T) -> c_int {
    return u_savecommon(curbuf.get(), lnum - 1, lnum + 1, lnum + 1, false);
}
pub unsafe extern "C" fn u_inssub(mut lnum: linenr_T) -> c_int {
    return u_savecommon(curbuf.get(), lnum - 1, lnum, lnum + 1, false);
}
pub unsafe extern "C" fn u_savedel(mut lnum: linenr_T, mut nlines: linenr_T) -> c_int {
    return u_savecommon(
        curbuf.get(),
        lnum - 1,
        lnum + nlines,
        if nlines == (*curbuf.get()).b_ml.ml_line_count {
            2
        } else {
            lnum
        },
        false,
    );
}
pub unsafe extern "C" fn undo_allowed(mut buf: *mut buf_T) -> bool {
    if (*buf).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
        return false;
    }
    if sandbox.get() != 0 {
        emsg(gettext(&raw const e_sandbox as *const c_char));
        return false;
    }
    if textlock.get() != 0 || expr_map_locked() {
        emsg(gettext(&raw const e_textlock as *const c_char));
        return false;
    }
    return true;
}
unsafe extern "C" fn get_undolevel(mut buf: *mut buf_T) -> OptInt {
    if (*buf).b_p_ul == NO_LOCAL_UNDOLEVEL as OptInt {
        return p_ul.get();
    }
    return (*buf).b_p_ul;
}
#[inline]
unsafe extern "C" fn zero_fmark_additional_data(mut fmarks: *mut fmark_T) {
    let mut i: size_t = 0;
    while i < NMARKS as size_t {
        let slot = &raw mut (*fmarks.add(i)).additional_data;
        xfree((*slot).cast());
        *slot = ptr::null_mut();
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn u_savecommon(
    mut buf: *mut buf_T,
    mut top: linenr_T,
    mut bot: linenr_T,
    mut newbot: linenr_T,
    mut reload: bool,
) -> c_int {
    if !reload {
        if !undo_allowed(buf) {
            return FAIL;
        }
        if buf == curbuf.get() {
            change_warning(buf, 0);
        }
        if bot > (*buf).b_ml.ml_line_count + 1 {
            emsg(gettext(c"E881: Line count changed unexpectedly".as_ptr()));
            return FAIL;
        }
    }
    let mut uep: *mut u_entry_T = ptr::null_mut();
    let mut prev_uep: *mut u_entry_T = ptr::null_mut();
    let mut size: linenr_T = bot - top - 1;
    if (*buf).b_u_synced {
        (*buf).b_new_change = true;
        let mut uhp: *mut u_header_T = ptr::null_mut();
        if get_undolevel(buf) >= 0 {
            uhp = xmalloc(size_of::<u_header_T>()) as *mut u_header_T;
            (*uhp).uh_extmark.capacity = 0;
            (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
            (*uhp).uh_extmark.items = ptr::null_mut();
        } else {
            uhp = ptr::null_mut();
        }
        let mut old_curhead: *mut u_header_T = (*buf).b_u_curhead;
        if !old_curhead.is_null() {
            (*buf).b_u_newhead = (*old_curhead).uh_next.ptr;
            (*buf).b_u_curhead = ptr::null_mut();
        }
        while (*buf).b_u_numhead as OptInt > get_undolevel(buf) && !(*buf).b_u_oldhead.is_null() {
            let mut uhfree: *mut u_header_T = (*buf).b_u_oldhead;
            if uhfree == old_curhead {
                u_freebranch(buf, uhfree, &raw mut old_curhead);
            } else if (*uhfree).uh_alt_next.ptr.is_null() {
                u_freeheader(buf, uhfree, &raw mut old_curhead);
            } else {
                while !(*uhfree).uh_alt_next.ptr.is_null() {
                    uhfree = (*uhfree).uh_alt_next.ptr;
                }
                u_freebranch(buf, uhfree, &raw mut old_curhead);
            }
        }
        if uhp.is_null() {
            if !old_curhead.is_null() {
                u_freebranch(buf, old_curhead, ptr::null_mut());
            }
            (*buf).b_u_synced = false;
            return OK;
        }
        (*uhp).uh_prev.ptr = ptr::null_mut();
        (*uhp).uh_next.ptr = (*buf).b_u_newhead;
        (*uhp).uh_alt_next.ptr = old_curhead;
        if !old_curhead.is_null() {
            (*uhp).uh_alt_prev.ptr = (*old_curhead).uh_alt_prev.ptr;
            if !(*uhp).uh_alt_prev.ptr.is_null() {
                (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = uhp;
            }
            (*old_curhead).uh_alt_prev.ptr = uhp;
            if (*buf).b_u_oldhead == old_curhead {
                (*buf).b_u_oldhead = uhp;
            }
        } else {
            (*uhp).uh_alt_prev.ptr = ptr::null_mut();
        }
        if !(*buf).b_u_newhead.is_null() {
            (*(*buf).b_u_newhead).uh_prev.ptr = uhp;
        }
        (*buf).b_u_seq_last += 1;
        (*uhp).uh_seq = (*buf).b_u_seq_last;
        (*buf).b_u_seq_cur = (*uhp).uh_seq;
        (*uhp).uh_time = time(ptr::null_mut());
        (*uhp).uh_save_nr = 0;
        (*buf).b_u_time_cur = (*uhp).uh_time + 1;
        (*uhp).uh_walk = 0;
        (*uhp).uh_entry = ptr::null_mut();
        (*uhp).uh_getbot_entry = ptr::null_mut();
        (*uhp).uh_cursor = (*curwin.get()).w_cursor;
        if virtual_active(curwin.get()) && (*curwin.get()).w_cursor.coladd > 0 {
            (*uhp).uh_cursor_vcol = getviscol() as colnr_T;
        } else {
            (*uhp).uh_cursor_vcol = -1;
        }
        (*uhp).uh_flags = (if (*buf).b_changed != 0 {
            UH_CHANGED as c_int
        } else {
            0
        }) + (if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
            UH_EMPTYBUF as c_int
        } else {
            0
        });
        zero_fmark_additional_data(&raw mut (*buf).b_namedm as *mut fmark_T);
        memmove(
            &raw mut (*uhp).uh_namedm as *mut fmark_T as *mut c_void,
            &raw mut (*buf).b_namedm as *mut fmark_T as *const c_void,
            size_of::<fmark_T>().wrapping_mul(NMARKS as size_t),
        );
        (*uhp).uh_visual = (*buf).b_visual;
        (*buf).b_u_newhead = uhp;
        if (*buf).b_u_oldhead.is_null() {
            (*buf).b_u_oldhead = uhp;
        }
        (*buf).b_u_numhead += 1;
    } else {
        if get_undolevel(buf) < 0 {
            return OK;
        }
        if size == 1 {
            uep = u_get_headentry(buf);
            prev_uep = ptr::null_mut();
            let mut i: c_int = 0;
            while i < 10 {
                if uep.is_null() {
                    break;
                }
                if (if (*(*buf).b_u_newhead).uh_getbot_entry != uep {
                    ((*uep).ue_top + (*uep).ue_size + 1
                        != (if (*uep).ue_bot == 0 {
                            (*buf).b_ml.ml_line_count + 1
                        } else {
                            (*uep).ue_bot
                        })) as c_int
                } else {
                    ((*uep).ue_lcount != (*buf).b_ml.ml_line_count) as c_int
                }) != 0
                    || (*uep).ue_size > 1
                        && top >= (*uep).ue_top
                        && top + 2 <= (*uep).ue_top + (*uep).ue_size + 1
                {
                    break;
                }
                if (*uep).ue_size == 1 && (*uep).ue_top == top {
                    if i > 0 {
                        u_getbot(buf);
                        (*buf).b_u_synced = false;
                        (*prev_uep).ue_next = (*uep).ue_next;
                        (*uep).ue_next = (*(*buf).b_u_newhead).uh_entry;
                        (*(*buf).b_u_newhead).uh_entry = uep;
                    }
                    if newbot != 0 {
                        (*uep).ue_bot = newbot;
                    } else if bot > (*buf).b_ml.ml_line_count {
                        (*uep).ue_bot = 0;
                    } else {
                        (*uep).ue_lcount = (*buf).b_ml.ml_line_count;
                        (*(*buf).b_u_newhead).uh_getbot_entry = uep;
                    }
                    return OK;
                }
                prev_uep = uep;
                uep = (*uep).ue_next;
                i += 1;
            }
        }
        u_getbot(buf);
    }
    uep = xmalloc(size_of::<u_entry_T>()) as *mut u_entry_T;
    memset(uep as *mut c_void, 0, size_of::<u_entry_T>());
    (*uep).ue_size = size;
    (*uep).ue_top = top;
    if newbot != 0 {
        (*uep).ue_bot = newbot;
    } else if bot > (*buf).b_ml.ml_line_count {
        (*uep).ue_bot = 0;
    } else {
        (*uep).ue_lcount = (*buf).b_ml.ml_line_count;
        (*(*buf).b_u_newhead).uh_getbot_entry = uep;
    }
    if size > 0 {
        (*uep).ue_array =
            xmalloc(size_of::<*mut c_char>().wrapping_mul(size as size_t)) as *mut *mut c_char;
        let mut lnum: linenr_T = 0;
        let mut i_0: c_int = 0;
        i_0 = 0;
        lnum = top + 1;
        while (i_0 as linenr_T) < size {
            fast_breakcheck();
            if got_int.get() {
                u_freeentry(uep, i_0);
                return FAIL;
            }
            let c2rust_fresh0 = lnum;
            lnum = lnum + 1;
            *(*uep).ue_array.offset(i_0 as isize) = u_save_line_buf(buf, c2rust_fresh0);
            i_0 += 1;
        }
    } else {
        (*uep).ue_array = ptr::null_mut();
    }
    (*uep).ue_next = (*(*buf).b_u_newhead).uh_entry;
    (*(*buf).b_u_newhead).uh_entry = uep;
    if reload {
        (*(*buf).b_u_newhead).uh_flags |= UH_RELOAD as c_int;
    }
    (*buf).b_u_synced = false;
    undo_undoes.set(false);
    return OK;
}
pub unsafe extern "C" fn undo_fmt_time(mut buf: *mut c_char, mut buflen: size_t, mut tt: time_t) {
    if time(ptr::null_mut()) - tt >= 100 {
        let mut curtime: tm = tm_zeroed();
        os_localtime_r(tt, &mut curtime);
        let mut n: size_t = 0;
        if time(ptr::null_mut()) - tt < (60 * 60 * 12) as time_t {
            n = strftime(buf, buflen, c"%H:%M:%S".as_ptr(), &raw mut curtime);
        } else {
            n = strftime(buf, buflen, c"%Y/%m/%d %H:%M:%S".as_ptr(), &raw mut curtime);
        }
        if n == 0 {
            *buf.offset(0) = NUL as c_char;
        }
    } else {
        let mut seconds: int64_t = time(ptr::null_mut()) as int64_t - tt as int64_t;
        vim_snprintf(
            buf,
            buflen,
            ngettext(
                c"%ld second ago".as_ptr(),
                c"%ld seconds ago".as_ptr(),
                seconds as uint32_t as c_ulong,
            ),
            seconds,
        );
    };
}
pub unsafe extern "C" fn u_sync(mut force: bool) {
    if (*curbuf.get()).b_u_synced || !force && no_u_sync.get() > 0 {
        return;
    }
    if get_undolevel(curbuf.get()) < 0 {
        (*curbuf.get()).b_u_synced = true;
    } else {
        u_getbot(curbuf.get());
        (*curbuf.get()).b_u_curhead = ptr::null_mut();
    };
}
pub unsafe extern "C" fn ex_undojoin(mut _eap: *mut exarg_T) {
    if (*curbuf.get()).b_u_newhead.is_null() {
        return;
    }
    if !(*curbuf.get()).b_u_curhead.is_null() {
        emsg(gettext(
            c"E790: undojoin is not allowed after undo".as_ptr(),
        ));
        return;
    }
    if !(*curbuf.get()).b_u_synced {
        return;
    }
    if get_undolevel(curbuf.get()) < 0 {
        return;
    }
    (*curbuf.get()).b_u_synced = false;
}
pub unsafe extern "C" fn u_unchanged(mut buf: *mut buf_T) {
    u_unch_branch((*buf).b_u_oldhead);
    (*buf).b_did_warn = false;
}
pub unsafe extern "C" fn u_find_first_changed() {
    let mut uhp: *mut u_header_T = (*curbuf.get()).b_u_newhead;
    if !(*curbuf.get()).b_u_curhead.is_null() || uhp.is_null() {
        return;
    }
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    if (*uep).ue_top != 0 || (*uep).ue_bot != 0 {
        return;
    }
    let mut lnum: linenr_T = 0;
    lnum = 1;
    while lnum < (*curbuf.get()).b_ml.ml_line_count && lnum <= (*uep).ue_size {
        if strcmp(
            ml_get_buf(curbuf.get(), lnum),
            *(*uep).ue_array.offset((lnum - 1) as isize),
        ) != 0
        {
            clearpos(&raw mut (*uhp).uh_cursor);
            (*uhp).uh_cursor.lnum = lnum;
            return;
        }
        lnum += 1;
    }
    if (*curbuf.get()).b_ml.ml_line_count != (*uep).ue_size {
        clearpos(&raw mut (*uhp).uh_cursor);
        (*uhp).uh_cursor.lnum = lnum;
    }
}
pub unsafe extern "C" fn u_update_save_nr(mut buf: *mut buf_T) {
    (*buf).b_u_save_nr_last += 1;
    (*buf).b_u_save_nr_cur = (*buf).b_u_save_nr_last;
    let mut uhp: *mut u_header_T = (*buf).b_u_curhead;
    if !uhp.is_null() {
        uhp = (*uhp).uh_next.ptr;
    } else {
        uhp = (*buf).b_u_newhead;
    }
    if !uhp.is_null() {
        (*uhp).uh_save_nr = (*buf).b_u_save_nr_last;
    }
}
pub unsafe extern "C" fn bufIsChanged(mut buf: *mut buf_T) -> bool {
    return if bt_prompt(buf) {
        (*buf).b_modified_was_set as c_int
    } else {
        (!bt_dontwrite(buf) && ((*buf).b_changed != 0 || file_ff_differs(buf, true))) as c_int
    } != 0;
}
pub unsafe extern "C" fn anyBufIsChanged() -> bool {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if bufIsChanged(buf) {
            return true;
        }
        buf = (*buf).b_next;
    }
    return false;
}
pub unsafe extern "C" fn curbufIsChanged() -> bool {
    return bufIsChanged(curbuf.get());
}
