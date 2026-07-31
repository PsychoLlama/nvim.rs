use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_SWAPEXISTS, apply_autocmds, has_autocmd,
};
use crate::src::nvim::buffer::{buf_inc_changedtick, buf_spname, open_buffer, setfname};
use crate::src::nvim::change::{changed_internal, unchanged};
use crate::src::nvim::cursor::{check_cursor, coladvance};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::src::nvim::eval::typval::{
    tv_dict_add_nr, tv_dict_add_str_len, tv_list_append_allocated_string,
};
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::event::libuv::{uv_strerror, uv_uptime};
use crate::src::nvim::fileio::{
    buf_store_file_info, modname, read_eintr, readfile, vim_deltempdir, vim_rename, vim_tempname,
};
use crate::src::nvim::getchar::flush_buffers;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::main::{
    NameBuff, allbuf_lock, cmdline_row, cmdmod, curbuf, curwin, did_check_timestamps, firstbuf,
    getout, got_int, inhibit_delete_count, msg_ext_skip_flush, msg_row, msg_silent,
    need_check_timestamps, need_wait_return, no_lines_msg, no_wait_return, p_dir, p_shm, p_uc,
    p_verbose, recoverymode, swap_exists_action,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, mb_utflen, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memfile::{
    mf_close, mf_close_file, mf_find, mf_free, mf_free_fnames, mf_get, mf_need_trans, mf_new,
    mf_new_page_size, mf_open, mf_open_file, mf_put, mf_set_dirty, mf_set_fnames, mf_sync,
    mf_trans_del,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xrealloc, xstpcpy, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    do_dialog, emsg, iemsg, msg, msg_end, msg_ext_set_kind, msg_home_replace, msg_multiline,
    msg_outnum, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_reset_scroll, msg_start,
    semsg, set_keep_msg, siemsg, smsg, verb_msg,
};
use crate::src::nvim::option::{
    copy_option_part, get_fileformat, set_fileformat, set_option_value_give_err,
};
use crate::src::nvim::options::kOptFileencoding;
use crate::src::nvim::os::env::{
    expand_env, home_replace, home_replace_save, os_get_hostname, os_get_pid,
};
use crate::src::nvim::os::fs::{
    os_fileinfo, os_fileinfo_inode, os_fileinfo_link, os_fileinfo_size, os_isdir, os_mkdir_recurse,
    os_open, os_path_exists, os_remove, os_set_cloexec,
};
use crate::src::nvim::os::input::{line_breakcheck, os_char_avail};
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, close, gettext, lseek, memmove, readlink, strcasecmp, strcmp,
    strcpy, strlen, strncasecmp, strncmp,
};
use crate::src::nvim::os::proc::os_proc_running;
use crate::src::nvim::os::time::{os_ctime_r, os_time};
use crate::src::nvim::os::users::{os_get_uname, os_get_username};
use crate::src::nvim::path::{
    FreeWild, after_pathsep, concat_fnames, expand_wildcards, fix_fname, path_fnamecmp,
    path_full_compare, path_is_absolute, path_tail, same_directory, shorten_dir, vim_FullName,
    vim_ispathsep,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::spell::spell_delete_wordlist;
use crate::src::nvim::statusline::get_trans_bufname;
use crate::src::nvim::strings::{kv_do_printf, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    FileInfo, OptVal, OptValData, OptValType, String_0, StringBuilder, Timestamp, VimVarIndex,
    bhdr_T, blocknr_T, buf_T, chunksize_T, colnr_T, dict_T, exarg_T, file_comparison,
    flush_buffers_T, infoptr_T, int16_t, int64_t, linenr_T, list_T, memfile_T, mfdirty_T, off_T,
    pos_T, size_t, ssize_t, time_t, uint8_t, uint16_t, uint64_t, uv_uid_t, varnumber_T,
};
use crate::src::nvim::ui::{ui_flush, ui_has};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::version::Versions;

// The carve of the transpiled module; see each child's docs.
mod block0;
pub use self::block0::*;
mod swapname;
pub use self::swapname::*;
mod recover;
pub use self::recover::*;
mod tree;
pub(crate) use self::tree::*;
mod edit;
pub(crate) use self::edit::*;
mod offsets;
pub use self::offsets::*;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HLF_E: C2Rust_Unnamed_14 = 6;
pub const kOptValTypeString: OptValType = 2;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_16 = 8192;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const VIM_WARNING: C2Rust_Unnamed_18 = 2;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const READ_NEW: C2Rust_Unnamed_19 = 1;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MFS_ZERO: C2Rust_Unnamed_20 = 8;
pub const MFS_FLUSH: C2Rust_Unnamed_20 = 4;
pub const MFS_STOP: C2Rust_Unnamed_20 = 2;
pub const MFS_ALL: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MIN_SWAP_PAGE_SIZE: C2Rust_Unnamed_21 = 1048;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DataBlock {
    pub db_id: uint16_t,
    pub db_free: ::core::ffi::c_uint,
    pub db_txt_start: ::core::ffi::c_uint,
    pub db_txt_end: ::core::ffi::c_uint,
    pub db_line_count: ::core::ffi::c_long,
    pub db_index: [::core::ffi::c_uint; 0],
}
pub const DATA_ID: C2Rust_Unnamed_27 = 25697;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PointerEntry {
    pub pe_bnum: blocknr_T,
    pub pe_line_count: linenr_T,
    pub pe_old_lnum: linenr_T,
    pub pe_page_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PointerBlock {
    pub pb_id: uint16_t,
    pub pb_count: uint16_t,
    pub pb_count_max: uint16_t,
    pub pb_pointer: [PointerEntry; 0],
}
pub const PTR_ID: C2Rust_Unnamed_27 = 28788;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ZeroBlock {
    pub b0_id: [::core::ffi::c_char; 2],
    pub b0_version: [::core::ffi::c_char; 10],
    pub b0_page_size: [::core::ffi::c_char; 4],
    pub b0_mtime: [::core::ffi::c_char; 4],
    pub b0_ino: [::core::ffi::c_char; 4],
    pub b0_pid: [::core::ffi::c_char; 4],
    pub b0_uname: [::core::ffi::c_char; 40],
    pub b0_hname: [::core::ffi::c_char; 40],
    pub b0_fname: [::core::ffi::c_char; 900],
    pub b0_magic_long: ::core::ffi::c_long,
    pub b0_magic_int: ::core::ffi::c_int,
    pub b0_magic_short: int16_t,
    pub b0_magic_char: ::core::ffi::c_char,
}
pub const B0_HNAME_SIZE: C2Rust_Unnamed_28 = 40;
pub const B0_UNAME_SIZE: C2Rust_Unnamed_28 = 40;
pub const B0_FNAME_SIZE_ORG: C2Rust_Unnamed_28 = 900;
pub const B0_FNAME_SIZE_NOCRYPT: C2Rust_Unnamed_28 = 898;
pub const B0_FNAME_SIZE_CRYPT: C2Rust_Unnamed_28 = 890;
pub const B0_MAGIC_CHAR: C2Rust_Unnamed_29 = 85;
pub const B0_MAGIC_SHORT: C2Rust_Unnamed_29 = 269554195;
pub const B0_MAGIC_INT: C2Rust_Unnamed_29 = 539042339;
pub const B0_MAGIC_LONG: C2Rust_Unnamed_29 = 808530483;
pub const BLOCK0_ID1: C2Rust_Unnamed_27 = 48;
pub const BLOCK0_ID0: C2Rust_Unnamed_27 = 98;
pub const SEA_CHOICE_NONE: sea_choice_T = 0;
pub type sea_choice_T = ::core::ffi::c_uint;
pub const SEA_CHOICE_ABORT: sea_choice_T = 6;
pub const SEA_CHOICE_QUIT: sea_choice_T = 5;
pub const SEA_CHOICE_DELETE: sea_choice_T = 4;
pub const SEA_CHOICE_RECOVER: sea_choice_T = 3;
pub const SEA_CHOICE_EDIT: sea_choice_T = 2;
pub const SEA_CHOICE_READONLY: sea_choice_T = 1;
pub const SHM_ATTENTION: C2Rust_Unnamed_25 = 65;
pub type upd_block0_T = ::core::ffi::c_uint;
pub const UB_SAME_DIR: upd_block0_T = 1;
pub const UB_FNAME: upd_block0_T = 0;
/// `action` for [`ml_find_line`]: only release the locked block.
pub const ML_FLUSH: ::core::ffi::c_int = 2;
/// `action` for [`ml_find_line`]: a line is about to be deleted here.
pub const ML_DELETE: ::core::ffi::c_int = 17;
/// `action` for [`ml_find_line`]: a line is about to be inserted here.
pub const ML_INSERT: ::core::ffi::c_int = 18;
/// `action` for [`ml_find_line`]: just look the line up.
pub const ML_FIND: ::core::ffi::c_int = 19;
/// Lines below which two neighbouring `ml_chunksize` entries are merged, and
/// the size a split aims for.
pub const MLCS_MINL: ::core::ffi::c_int = 400;
/// Lines above which an `ml_chunksize` entry is split in two.
pub const MLCS_MAXL: ::core::ffi::c_int = 800;
/// `flags` for `ml_append_int`: carry the `DB_MARKED` bit onto the new line.
pub const ML_APPEND_MARK: ::core::ffi::c_int = 2;
/// `flags` for `ml_append_int`: this is a fresh file being read in, so the
/// block may be numbered negatively and need not keep its position.
pub const ML_APPEND_NEW: ::core::ffi::c_int = 1;
/// `flags` for `ml_delete_int`: say "--No lines in buffer--" if the buffer
/// ends up empty.
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_24 = 2;
pub const kEqualFiles: file_comparison = 1;
pub const EW_SILENT: C2Rust_Unnamed_26 = 32;
pub const EW_FILE: C2Rust_Unnamed_26 = 2;
pub const EW_KEEPALL: C2Rust_Unnamed_26 = 16;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BF_RECOVERED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ML_CHNK_ADDLINE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_CHNK_DELLINE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ML_CHNK_UPDLINE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ML_LINE_DIRTY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const ML_LOCKED_DIRTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const ML_LOCKED_POS: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const ML_ALLOCATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BH_DIRTY: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SEA_READONLY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DB_MARKED: ::core::ffi::c_uint = (1 as ::core::ffi::c_int as ::core::ffi::c_uint)
    << ::core::mem::size_of::<::core::ffi::c_uint>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const DB_INDEX_MASK: ::core::ffi::c_uint = !DB_MARKED;
pub const INDEX_SIZE: usize = ::core::mem::size_of::<::core::ffi::c_uint>();
pub const HEADER_SIZE: ::core::ffi::c_ulong = 24 as ::core::ffi::c_ulong;
pub const B0_DIRTY: ::core::ffi::c_int = 0x55 as ::core::ffi::c_int;
pub const B0_FF_MASK: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const B0_SAME_DIR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const B0_HAS_FENC: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const STACK_INCR: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
static lowest_marked: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static e_warning_pointer_block_corrupted: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E1364: Warning: Pointer block corrupted\0",
        )
    });
pub unsafe fn ml_open(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut b0p: *mut ZeroBlock = ::core::ptr::null_mut::<ZeroBlock>();
    let mut pp: *mut PointerBlock = ::core::ptr::null_mut::<PointerBlock>();
    let mut dp: *mut DataBlock = ::core::ptr::null_mut::<DataBlock>();
    (*buf).b_ml.ml_stack_size = 0 as ::core::ffi::c_int;
    (*buf).b_ml.ml_stack = ::core::ptr::null_mut::<infoptr_T>();
    (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
    (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
    (*buf).b_ml.ml_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*buf).b_ml.ml_line_offset = 0 as size_t;
    (*buf).b_ml.ml_chunksize = ::core::ptr::null_mut::<chunksize_T>();
    (*buf).b_ml.ml_usedchunks = 0 as ::core::ffi::c_int;
    if (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
        (*buf).b_p_swf = false_0;
    }
    if (*buf).terminal.is_null() && p_uc.get() != 0 && (*buf).b_p_swf != 0 {
        (*buf).b_may_swap = true_0 != 0;
    } else {
        (*buf).b_may_swap = false_0 != 0;
    }
    let mut mfp: *mut memfile_T = mf_open(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    if !mfp.is_null() {
        (*buf).b_ml.ml_mfp = mfp;
        (*buf).b_ml.ml_flags = ML_EMPTY;
        (*buf).b_ml.ml_line_count = 1 as ::core::ffi::c_int as linenr_T;
        hp = mf_new(mfp, false_0 != 0, 1 as ::core::ffi::c_uint);
        if (*hp).bh_bnum != 0 as blocknr_T {
            iemsg(gettext(
                b"E298: Didn't get block nr 0?\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            b0p = (*hp).bh_data as *mut ZeroBlock;
            (*b0p).b0_id[0 as ::core::ffi::c_int as usize] =
                BLOCK0_ID0 as ::core::ffi::c_int as ::core::ffi::c_char;
            (*b0p).b0_id[1 as ::core::ffi::c_int as usize] =
                BLOCK0_ID1 as ::core::ffi::c_int as ::core::ffi::c_char;
            (*b0p).b0_magic_long = B0_MAGIC_LONG as ::core::ffi::c_int as ::core::ffi::c_long;
            (*b0p).b0_magic_int = B0_MAGIC_INT as ::core::ffi::c_int;
            (*b0p).b0_magic_short = B0_MAGIC_SHORT as ::core::ffi::c_int as int16_t;
            (*b0p).b0_magic_char = B0_MAGIC_CHAR as ::core::ffi::c_int as ::core::ffi::c_char;
            xstrlcpy(
                xstpcpy(
                    &raw mut (*b0p).b0_version as *mut ::core::ffi::c_char,
                    b"VIM \0".as_ptr() as *const ::core::ffi::c_char,
                ),
                *(Versions.ptr() as *mut *mut ::core::ffi::c_char)
                    .offset(0 as ::core::ffi::c_int as isize),
                6 as size_t,
            );
            b0_store_number(
                (*mfp).mf_page_size as ::core::ffi::c_long,
                &mut (*b0p).b0_page_size,
            );
            if !(*buf).b_spell {
                (*b0p).b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as usize] = (if (*buf).b_changed != 0 {
                    B0_DIRTY
                } else {
                    0 as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                (*b0p).b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                    as usize] =
                    (get_fileformat(buf) + 1 as ::core::ffi::c_int) as ::core::ffi::c_char;
                set_b0_fname(b0p, buf);
                os_get_username(
                    &raw mut (*b0p).b0_uname as *mut ::core::ffi::c_char,
                    B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
                );
                (*b0p).b0_uname
                    [(B0_UNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
                    NUL as ::core::ffi::c_char;
                os_get_hostname(
                    &raw mut (*b0p).b0_hname as *mut ::core::ffi::c_char,
                    B0_HNAME_SIZE as ::core::ffi::c_int as size_t,
                );
                (*b0p).b0_hname
                    [(B0_HNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
                    NUL as ::core::ffi::c_char;
                b0_store_number(os_get_pid() as ::core::ffi::c_long, &mut (*b0p).b0_pid);
            }
            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
            if !(*buf).b_help && !(*buf).b_spell {
                mf_sync(mfp, 0 as ::core::ffi::c_int);
            }
            hp = ml_new_ptr(mfp);
            '_c2rust_label: {
                if !hp.is_null() {
                } else {
                    __assert_fail(
                        b"hp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        359 as ::core::ffi::c_uint,
                        b"int ml_open(buf_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if (*hp).bh_bnum != 1 as blocknr_T {
                iemsg(gettext(
                    b"E298: Didn't get block nr 1?\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                pp = (*hp).bh_data as *mut PointerBlock;
                (*pp).pb_count = 1 as uint16_t;
                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset(0 as ::core::ffi::c_int as isize))
                .pe_bnum = 2 as blocknr_T;
                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset(0 as ::core::ffi::c_int as isize))
                .pe_page_count = 1 as ::core::ffi::c_int;
                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset(0 as ::core::ffi::c_int as isize))
                .pe_old_lnum = 1 as ::core::ffi::c_int as linenr_T;
                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                    .offset(0 as ::core::ffi::c_int as isize))
                .pe_line_count = 1 as ::core::ffi::c_int as linenr_T;
                mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                hp = ml_new_data(mfp, false_0 != 0, 1 as int64_t);
                if (*hp).bh_bnum != 2 as blocknr_T {
                    iemsg(gettext(
                        b"E298: Didn't get block nr 2?\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                } else {
                    dp = (*hp).bh_data as *mut DataBlock;
                    (*dp).db_txt_start = (*dp).db_txt_start.wrapping_sub(1);
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset(0 as ::core::ffi::c_int as isize) = (*dp).db_txt_start;
                    (*dp).db_free = (*dp).db_free.wrapping_sub(
                        (1 as ::core::ffi::c_uint).wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                    );
                    (*dp).db_line_count = 1 as ::core::ffi::c_long;
                    *(dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize) =
                        NUL as ::core::ffi::c_char;
                    return OK;
                }
            }
        }
    }
    if !mfp.is_null() {
        if !hp.is_null() {
            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
        }
        mf_close(mfp, true_0 != 0);
    }
    (*buf).b_ml.ml_mfp = ::core::ptr::null_mut::<memfile_T>();
    return FAIL;
}
pub unsafe extern "C" fn ml_open_files() {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if (*buf).b_p_ro == 0 || (*buf).b_changed != 0 {
            ml_open_file(buf);
        }
        buf = (*buf).b_next;
    }
}
pub unsafe extern "C" fn ml_open_file(mut buf: *mut buf_T) {
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if mfp.is_null()
        || (*mfp).mf_fd >= 0 as ::core::ffi::c_int
        || (*buf).b_p_swf == 0
        || (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0
        || !(*buf).terminal.is_null()
    {
        return;
    }
    if (*buf).b_spell {
        let mut fname: *mut ::core::ffi::c_char = vim_tempname();
        if !fname.is_null() {
            mf_open_file(mfp, fname);
        }
        (*buf).b_may_swap = false_0 != 0;
        return;
    }
    let mut dirp: *mut ::core::ffi::c_char = p_dir.get();
    let mut found_existing_dir: bool = false_0 != 0;
    while *dirp as ::core::ffi::c_int != NUL {
        let mut fname_0: *mut ::core::ffi::c_char = findswapname(
            buf,
            &raw mut dirp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut found_existing_dir,
        );
        if dirp.is_null() {
            break;
        }
        if fname_0.is_null() {
            continue;
        }
        if mf_open_file(mfp, fname_0) != OK {
            continue;
        }
        (*mfp).mf_dirty = MF_DIRTY_YES_NOSYNC;
        ml_upd_block0(buf, UB_SAME_DIR);
        if mf_sync(mfp, MFS_ZERO as ::core::ffi::c_int) == OK {
            mf_set_dirty(mfp);
            break;
        } else {
            mf_close_file(buf, false_0 != 0);
        }
    }
    if *p_dir.get() as ::core::ffi::c_int != NUL && (*mfp).mf_fname.is_null() {
        need_wait_return.set(true_0 != 0);
        (*no_wait_return.ptr()) += 1;
        semsg(
            gettext(
                b"E303: Unable to open swap file for \"%s\", recovery impossible\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            if !buf_spname(buf).is_null() {
                buf_spname(buf)
            } else {
                (*buf).b_fname
            },
        );
        (*no_wait_return.ptr()) -= 1;
    }
    (*buf).b_may_swap = false_0 != 0;
}
pub unsafe extern "C" fn check_need_swap(mut newfile: bool) {
    let mut old_msg_silent: ::core::ffi::c_int = msg_silent.get();
    msg_silent.set(0 as ::core::ffi::c_int);
    if (*curbuf.get()).b_may_swap as ::core::ffi::c_int != 0
        && ((*curbuf.get()).b_p_ro == 0 || !newfile)
    {
        ml_open_file(curbuf.get());
    }
    msg_silent.set(old_msg_silent);
}
pub unsafe extern "C" fn ml_close(mut buf: *mut buf_T, mut del_file: ::core::ffi::c_int) {
    if (*buf).b_ml.ml_mfp.is_null() {
        return;
    }
    mf_close((*buf).b_ml.ml_mfp, del_file != 0);
    if (*buf).b_ml.ml_line_lnum != 0 as linenr_T
        && (*buf).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0
    {
        xfree((*buf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
    }
    xfree((*buf).b_ml.ml_stack as *mut ::core::ffi::c_void);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_ml.ml_chunksize as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    (*buf).b_ml.ml_mfp = ::core::ptr::null_mut::<memfile_T>();
    (*buf).b_flags &= !BF_RECOVERED;
}
pub unsafe extern "C" fn ml_close_all(mut del_file: bool) {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        ml_close(buf, del_file as ::core::ffi::c_int);
        buf = (*buf).b_next;
    }
    spell_delete_wordlist();
    vim_deltempdir();
}
pub unsafe extern "C" fn ml_close_notmod() {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !bufIsChanged(buf) {
            ml_close(buf, true_0);
        }
        buf = (*buf).b_next;
    }
}
static proc_running: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn ml_get(mut lnum: linenr_T) -> *mut ::core::ffi::c_char {
    return ml_get_buf_impl(curbuf.get(), lnum, false_0 != 0);
}
pub unsafe fn ml_get_buf(mut buf: *mut buf_T, mut lnum: linenr_T) -> *mut ::core::ffi::c_char {
    return ml_get_buf_impl(buf, lnum, false_0 != 0);
}
pub unsafe extern "C" fn ml_get_buf_mut(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
) -> *mut ::core::ffi::c_char {
    return ml_get_buf_impl(buf, lnum, true_0 != 0);
}
pub unsafe extern "C" fn ml_get_pos(mut pos: *const pos_T) -> *mut ::core::ffi::c_char {
    return ml_get_buf(curbuf.get(), (*pos).lnum).offset((*pos).col as isize);
}
pub unsafe extern "C" fn ml_get_len(mut lnum: linenr_T) -> colnr_T {
    return ml_get_buf_len(curbuf.get(), lnum);
}
pub unsafe extern "C" fn ml_get_pos_len(mut pos: *mut pos_T) -> colnr_T {
    return ml_get_buf_len(curbuf.get(), (*pos).lnum) - (*pos).col;
}
pub unsafe fn ml_get_buf_len(mut buf: *mut buf_T, mut lnum: linenr_T) -> colnr_T {
    let mut line: *const ::core::ffi::c_char = ml_get_buf(buf, lnum);
    if *line as ::core::ffi::c_int == NUL {
        return 0 as colnr_T;
    }
    '_c2rust_label: {
        if (*buf).b_ml.ml_line_textlen > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"buf->b_ml.ml_line_textlen > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1899 as ::core::ffi::c_uint,
                b"colnr_T ml_get_buf_len(buf_T *, linenr_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return (*buf).b_ml.ml_line_textlen - 1 as colnr_T;
}
pub unsafe extern "C" fn gchar_pos(mut pos: *mut pos_T) -> ::core::ffi::c_int {
    if (*pos).col == MAXCOL as ::core::ffi::c_int || (*pos).col > ml_get_len((*pos).lnum) {
        return NUL;
    }
    return utf_ptr2char(ml_get_pos(pos));
}
pub unsafe extern "C" fn ml_line_alloced() -> ::core::ffi::c_int {
    return (*curbuf.get()).b_ml.ml_flags & ML_LINE_DIRTY;
}
unsafe extern "C" fn ml_append_flush(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if lnum > (*buf).b_ml.ml_line_count {
        return FAIL;
    }
    if (*buf).b_ml.ml_line_lnum != 0 as linenr_T {
        ml_flush_line(buf, false_0 != 0);
    }
    return ml_append_int(buf, lnum, line, len, flags);
}
pub unsafe extern "C" fn ml_append(
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
    mut newfile: bool,
) -> ::core::ffi::c_int {
    return ml_append_flags(
        lnum,
        line,
        len,
        if newfile as ::core::ffi::c_int != 0 {
            ML_APPEND_NEW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn ml_append_flags(
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*curbuf.get()).b_ml.ml_mfp.is_null()
        && open_buffer(
            false_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        ) == FAIL
    {
        return FAIL;
    }
    return ml_append_flush(curbuf.get(), lnum, line, len, flags);
}
pub unsafe fn ml_append_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
    mut newfile: bool,
) -> ::core::ffi::c_int {
    if (*buf).b_ml.ml_mfp.is_null() {
        return FAIL;
    }
    return ml_append_flush(
        buf,
        lnum,
        line,
        len,
        if newfile as ::core::ffi::c_int != 0 {
            ML_APPEND_NEW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn ml_add_deleted_len(mut ptr: *mut ::core::ffi::c_char, mut len: ssize_t) {
    ml_add_deleted_len_buf(curbuf.get(), ptr, len);
}
pub unsafe extern "C" fn ml_add_deleted_len_buf(
    mut buf: *mut buf_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut len: ssize_t,
) {
    if inhibit_delete_count.get() != 0 {
        return;
    }
    let mut maxlen: ssize_t = strlen(ptr) as ssize_t;
    if len == -1 as ssize_t || len > maxlen {
        len = maxlen;
    }
    (*buf).deleted_bytes = (*buf)
        .deleted_bytes
        .wrapping_add((len as size_t).wrapping_add(1 as size_t));
    (*buf).deleted_bytes2 = (*buf)
        .deleted_bytes2
        .wrapping_add((len as size_t).wrapping_add(1 as size_t));
    if (*buf).update_need_codepoints {
        mb_utflen(
            ptr,
            len as size_t,
            &raw mut (*buf).deleted_codepoints,
            &raw mut (*buf).deleted_codeunits,
        );
        (*buf).deleted_codepoints = (*buf).deleted_codepoints.wrapping_add(1);
        (*buf).deleted_codeunits = (*buf).deleted_codeunits.wrapping_add(1);
    }
}
pub unsafe extern "C" fn ml_replace(
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut copy: bool,
) -> ::core::ffi::c_int {
    return ml_replace_buf(curbuf.get(), lnum, line, copy, false_0 != 0);
}
pub unsafe extern "C" fn ml_replace_len(
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut copy: bool,
) -> ::core::ffi::c_int {
    return ml_replace_buf_len(curbuf.get(), lnum, line, len, copy, false_0 != 0);
}
pub unsafe fn ml_replace_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut copy: bool,
    mut noalloc: bool,
) -> ::core::ffi::c_int {
    let mut len: size_t = if !line.is_null() {
        strlen(line)
    } else {
        -1 as ::core::ffi::c_int as size_t
    };
    return ml_replace_buf_len(buf, lnum, line, len, copy, noalloc);
}
pub unsafe extern "C" fn ml_replace_buf_len(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line_arg: *mut ::core::ffi::c_char,
    mut len_arg: size_t,
    mut copy: bool,
    mut noalloc: bool,
) -> ::core::ffi::c_int {
    let mut line: *mut ::core::ffi::c_char = line_arg;
    if line.is_null() {
        return FAIL;
    }
    if (*buf).b_ml.ml_mfp.is_null()
        && open_buffer(
            false_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        ) == FAIL
    {
        return FAIL;
    }
    if copy {
        '_c2rust_label: {
            if !noalloc {
            } else {
                __assert_fail(
                    b"!noalloc\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2583 as ::core::ffi::c_uint,
                    b"int ml_replace_buf_len(buf_T *, linenr_T, char *, size_t, _Bool, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        line = xmemdupz(line as *const ::core::ffi::c_void, len_arg) as *mut ::core::ffi::c_char;
    }
    if (*buf).b_ml.ml_line_lnum != lnum {
        ml_flush_line(buf, false_0 != 0);
    }
    if (*buf).update_callbacks.size != 0 {
        ml_add_deleted_len_buf(buf, ml_get_buf(buf, lnum), -1 as ssize_t);
    }
    if (*buf).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
        xfree((*buf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
    }
    (*buf).b_ml.ml_line_ptr = line;
    (*buf).b_ml.ml_line_textlen =
        (len_arg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
    (*buf).b_ml.ml_line_lnum = lnum;
    (*buf).b_ml.ml_flags = ((*buf).b_ml.ml_flags | ML_LINE_DIRTY) & !ML_EMPTY;
    if noalloc {
        ml_flush_line(buf, true_0 != 0);
    }
    return OK;
}
pub unsafe fn ml_delete_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut message: bool,
) -> ::core::ffi::c_int {
    ml_flush_line(buf, false_0 != 0);
    return ml_delete_int(
        buf,
        lnum,
        if message as ::core::ffi::c_int != 0 {
            ML_DEL_MESSAGE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn ml_delete(mut lnum: linenr_T) -> ::core::ffi::c_int {
    return ml_delete_flags(lnum, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn ml_delete_flags(
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    ml_flush_line(curbuf.get(), false_0 != 0);
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
        return FAIL;
    }
    return ml_delete_int(curbuf.get(), lnum, flags);
}
pub unsafe extern "C" fn ml_setmarked(mut lnum: linenr_T) {
    if lnum < 1 as linenr_T
        || lnum > (*curbuf.get()).b_ml.ml_line_count
        || (*curbuf.get()).b_ml.ml_mfp.is_null()
    {
        return;
    }
    if lowest_marked.get() == 0 as linenr_T || lowest_marked.get() > lnum {
        lowest_marked.set(lnum);
    }
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    hp = ml_find_line(curbuf.get(), lnum, ML_FIND as ::core::ffi::c_int);
    if hp.is_null() {
        return;
    }
    let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
        .offset((lnum - (*curbuf.get()).b_ml.ml_locked_low) as isize) |= DB_MARKED;
    (*curbuf.get()).b_ml.ml_flags |= ML_LOCKED_DIRTY;
}
pub unsafe extern "C" fn ml_firstmarked() -> linenr_T {
    if (*curbuf.get()).b_ml.ml_mfp.is_null() {
        return 0 as linenr_T;
    }
    let mut lnum: linenr_T = lowest_marked.get();
    while lnum <= (*curbuf.get()).b_ml.ml_line_count {
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        hp = ml_find_line(curbuf.get(), lnum, ML_FIND as ::core::ffi::c_int);
        if hp.is_null() {
            return 0 as linenr_T;
        }
        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
        let mut i: ::core::ffi::c_int =
            lnum as ::core::ffi::c_int - (*curbuf.get()).b_ml.ml_locked_low as ::core::ffi::c_int;
        while lnum <= (*curbuf.get()).b_ml.ml_locked_high {
            if *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i as isize) & DB_MARKED
                != 0
            {
                *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i as isize) &=
                    DB_INDEX_MASK;
                (*curbuf.get()).b_ml.ml_flags |= ML_LOCKED_DIRTY;
                lowest_marked.set(lnum + 1 as linenr_T);
                return lnum;
            }
            i += 1;
            lnum += 1;
        }
    }
    return 0 as linenr_T;
}
pub unsafe extern "C" fn ml_clearmarked() {
    if (*curbuf.get()).b_ml.ml_mfp.is_null() {
        return;
    }
    let mut lnum: linenr_T = lowest_marked.get();
    while lnum <= (*curbuf.get()).b_ml.ml_line_count {
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        hp = ml_find_line(curbuf.get(), lnum, ML_FIND as ::core::ffi::c_int);
        if hp.is_null() {
            return;
        }
        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
        let mut i: ::core::ffi::c_int =
            lnum as ::core::ffi::c_int - (*curbuf.get()).b_ml.ml_locked_low as ::core::ffi::c_int;
        while lnum <= (*curbuf.get()).b_ml.ml_locked_high {
            if *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i as isize) & DB_MARKED
                != 0
            {
                *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i as isize) &=
                    DB_INDEX_MASK;
                (*curbuf.get()).b_ml.ml_flags |= ML_LOCKED_DIRTY;
            }
            i += 1;
            lnum += 1;
        }
    }
    lowest_marked.set(0 as ::core::ffi::c_int as linenr_T);
}
pub unsafe extern "C" fn ml_flush_deleted_bytes(
    mut buf: *mut buf_T,
    mut codepoints: *mut size_t,
    mut codeunits: *mut size_t,
) -> size_t {
    let mut ret: size_t = (*buf).deleted_bytes;
    *codepoints = (*buf).deleted_codepoints;
    *codeunits = (*buf).deleted_codeunits;
    (*buf).deleted_bytes = 0 as size_t;
    (*buf).deleted_codepoints = 0 as size_t;
    (*buf).deleted_codeunits = 0 as size_t;
    return ret;
}
pub unsafe extern "C" fn inc(mut lp: *mut pos_T) -> ::core::ffi::c_int {
    if (*lp).col != MAXCOL as ::core::ffi::c_int {
        let p: *const ::core::ffi::c_char = ml_get_pos(lp);
        if *p as ::core::ffi::c_int != NUL {
            let l: ::core::ffi::c_int = utfc_ptr2len(p);
            (*lp).col += l;
            return if *p.offset(l as isize) as ::core::ffi::c_int != NUL {
                0 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            };
        }
    }
    if (*lp).lnum != (*curbuf.get()).b_ml.ml_line_count {
        (*lp).col = 0 as ::core::ffi::c_int as colnr_T;
        (*lp).lnum += 1;
        (*lp).coladd = 0 as ::core::ffi::c_int as colnr_T;
        return 1 as ::core::ffi::c_int;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn incl(mut lp: *mut pos_T) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    r = inc(lp);
    if r >= 1 as ::core::ffi::c_int && (*lp).col != 0 {
        r = inc(lp);
    }
    return r;
}
pub unsafe extern "C" fn dec(mut lp: *mut pos_T) -> ::core::ffi::c_int {
    (*lp).coladd = 0 as ::core::ffi::c_int as colnr_T;
    if (*lp).col == MAXCOL as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = ml_get((*lp).lnum);
        (*lp).col = ml_get_len((*lp).lnum);
        (*lp).col -= utf_head_off(p, p.offset((*lp).col as isize));
        return 0 as ::core::ffi::c_int;
    }
    if (*lp).col > 0 as ::core::ffi::c_int {
        (*lp).col -= 1;
        let mut p_0: *mut ::core::ffi::c_char = ml_get((*lp).lnum);
        (*lp).col -= utf_head_off(p_0, p_0.offset((*lp).col as isize));
        return 0 as ::core::ffi::c_int;
    }
    if (*lp).lnum > 1 as linenr_T {
        (*lp).lnum -= 1;
        let mut p_1: *mut ::core::ffi::c_char = ml_get((*lp).lnum);
        (*lp).col = ml_get_len((*lp).lnum);
        (*lp).col -= utf_head_off(p_1, p_1.offset((*lp).col as isize));
        return 1 as ::core::ffi::c_int;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn decl(mut lp: *mut pos_T) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    r = dec(lp);
    if r == 1 as ::core::ffi::c_int && (*lp).col != 0 {
        r = dec(lp);
    }
    return r;
}
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
