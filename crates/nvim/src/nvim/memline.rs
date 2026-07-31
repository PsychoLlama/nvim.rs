use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_SWAPEXISTS, apply_autocmds, has_autocmd,
};
use crate::src::nvim::buffer::{buf_inc_changedtick, buf_spname, open_buffer, setfname};
use crate::src::nvim::change::{changed_internal, unchanged};
use crate::src::nvim::cursor::{check_cursor, coladvance};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::src::nvim::eval::typval::{
    tv_dict_add_nr, tv_dict_add_str, tv_dict_add_str_len, tv_list_append_allocated_string,
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
    __off_t, FileInfo, OptInt, OptVal, OptValData, OptValType, String_0, StringBuilder, Timestamp,
    VimVarIndex, bhdr_T, blocknr_T, buf_T, chunksize_T, colnr_T, dict_T, exarg_T, file_comparison,
    flush_buffers_T, infoptr_T, int16_t, int32_t, int64_t, linenr_T, list_T, memfile_T, mfdirty_T,
    off_T, pos_T, ptrdiff_t, size_t, ssize_t, time_t, uint8_t, uint16_t, uint64_t, uv_stat_t,
    uv_timespec_t, uv_uid_t, varnumber_T,
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
pub const MLCS_MINL: C2Rust_Unnamed_31 = 400;
pub const ML_FIND: C2Rust_Unnamed_30 = 19;
pub const ML_INSERT: C2Rust_Unnamed_30 = 18;
pub const ML_DELETE: C2Rust_Unnamed_30 = 17;
pub const ML_FLUSH: C2Rust_Unnamed_30 = 2;
pub const MLCS_MAXL: C2Rust_Unnamed_31 = 800;
pub const ML_APPEND_MARK: C2Rust_Unnamed_23 = 2;
pub const ML_APPEND_NEW: C2Rust_Unnamed_23 = 1;
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_22 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_24 = 2;
pub const kEqualFiles: file_comparison = 1;
pub const EW_SILENT: C2Rust_Unnamed_26 = 32;
pub const EW_FILE: C2Rust_Unnamed_26 = 2;
pub const EW_KEEPALL: C2Rust_Unnamed_26 = 16;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
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
static e_ml_get_invalid_lnum_nr: GlobalCell<[::core::ffi::c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E315: ml_get: Invalid lnum: %ld\0",
    )
});
static e_ml_get_cannot_find_line_nr_in_buffer_nr_str: GlobalCell<[::core::ffi::c_char; 50]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
            *b"E316: ml_get: Cannot find line %ldin buffer %d %s\0",
        )
    });
static e_pointer_block_id_wrong: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E317: Pointer block id wrong\0",
    )
});
static e_pointer_block_id_wrong_two: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E317: Pointer block id wrong 2\0",
        )
    });
static e_pointer_block_id_wrong_three: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E317: Pointer block id wrong 3\0",
        )
    });
static e_pointer_block_id_wrong_four: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E317: Pointer block id wrong 4\0",
        )
    });
static e_line_number_out_of_range_nr_past_the_end: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E322: Line number out of range: %ld past the end\0",
        )
    });
static e_line_count_wrong_in_block_nr: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E323: Line count wrong in block %ld\0",
        )
    });
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
                &raw mut (*b0p).b0_page_size as *mut ::core::ffi::c_char,
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
                b0_store_number(
                    os_get_pid() as ::core::ffi::c_long,
                    &raw mut (*b0p).b0_pid as *mut ::core::ffi::c_char,
                );
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
pub unsafe extern "C" fn ml_recover(mut checkext: bool) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hl_id: ::core::ffi::c_int = 0;
    let mut b0p: *mut ZeroBlock = ::core::ptr::null_mut::<ZeroBlock>();
    let mut org_file_info: FileInfo = FileInfo {
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
    let mut swp_file_info: FileInfo = FileInfo {
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
    let mut mtime: ::core::ffi::c_int = 0;
    let mut b0_ff: ::core::ffi::c_int = 0;
    let mut bnum: blocknr_T = 0;
    let mut page_count: ::core::ffi::c_uint = 0;
    let mut lnum: linenr_T = 0;
    let mut line_count: linenr_T = 0;
    let mut idx: ::core::ffi::c_int = 0;
    let mut error: ::core::ffi::c_int = 0;
    let mut cannot_open: bool = false;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut mfp: *mut memfile_T = ::core::ptr::null_mut::<memfile_T>();
    let mut fname_used: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut b0_fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ip: *mut infoptr_T = ::core::ptr::null_mut::<infoptr_T>();
    let mut directly: bool = false;
    let mut serious_error: bool = true_0 != 0;
    let mut orig_file_status: ::core::ffi::c_int = NOTDONE;
    recoverymode.set(true_0 != 0);
    let mut called_from_main: ::core::ffi::c_int =
        (*curbuf.get()).b_ml.ml_mfp.is_null() as ::core::ffi::c_int;
    let mut fname: *mut ::core::ffi::c_char = (*curbuf.get()).b_fname;
    if fname.is_null() {
        fname = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut len: ::core::ffi::c_int = strlen(fname) as ::core::ffi::c_int;
    '_theend: {
        if checkext as ::core::ffi::c_int != 0
            && len >= 4 as ::core::ffi::c_int
            && strncasecmp(
                fname
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
            && !vim_strchr(
                b"abcdefghijklmnopqrstuvw\0".as_ptr() as *const ::core::ffi::c_char,
                if (*fname.offset((len - 2 as ::core::ffi::c_int) as isize) as uint8_t
                    as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *fname.offset((len - 2 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *fname.offset((len - 2 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int
                } else {
                    *fname.offset((len - 2 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                },
            )
            .is_null()
            && (*fname.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *fname.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *fname.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *fname.offset((len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
        {
            directly = true_0 != 0;
            fname_used = xstrdup(fname);
        } else {
            directly = false_0 != 0;
            len = recover_names(
                fname,
                false_0 != 0,
                ::core::ptr::null_mut::<list_T>(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            if len == 0 as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E305: No swap file found for %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    fname,
                );
                break '_theend;
            } else {
                let mut i: ::core::ffi::c_int = 0;
                if len == 1 as ::core::ffi::c_int {
                    i = 1 as ::core::ffi::c_int;
                } else {
                    recover_names(
                        fname,
                        true_0 != 0,
                        ::core::ptr::null_mut::<list_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    );
                    if !ui_has(kUIMessages) {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    i = prompt_for_input(
                        gettext(b"Enter number of swap file to use (0 to quit): \0".as_ptr()
                            as *const ::core::ffi::c_char),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if i < 1 as ::core::ffi::c_int || i > len {
                        break '_theend;
                    }
                }
                recover_names(
                    fname,
                    false_0 != 0,
                    ::core::ptr::null_mut::<list_T>(),
                    i,
                    &raw mut fname_used,
                );
            }
        }
        if !fname_used.is_null() {
            if called_from_main != 0 && ml_open(curbuf.get()) == FAIL {
                getout(1 as ::core::ffi::c_int);
            }
            buf = xmalloc(::core::mem::size_of::<buf_T>()) as *mut buf_T;
            (*buf).b_ml.ml_stack_size = 0 as ::core::ffi::c_int;
            (*buf).b_ml.ml_stack = ::core::ptr::null_mut::<infoptr_T>();
            (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
            (*buf).b_ml.ml_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
            (*buf).b_ml.ml_line_offset = 0 as size_t;
            (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
            (*buf).b_ml.ml_flags = 0 as ::core::ffi::c_int;
            p = xstrdup(fname_used);
            mfp = mf_open(fname_used, O_RDONLY);
            fname_used = p;
            if mfp.is_null() || (*mfp).mf_fd < 0 as ::core::ffi::c_int {
                semsg(
                    gettext(b"E306: Cannot open %s\0".as_ptr() as *const ::core::ffi::c_char),
                    fname_used,
                );
            } else {
                (*buf).b_ml.ml_mfp = mfp;
                (*mfp).mf_page_size =
                    MIN_SWAP_PAGE_SIZE as ::core::ffi::c_int as ::core::ffi::c_uint;
                hl_id = HLF_E as ::core::ffi::c_int;
                msg_ext_set_kind(b"emsg\0".as_ptr() as *const ::core::ffi::c_char);
                hp = mf_get(mfp, 0 as blocknr_T, 1 as ::core::ffi::c_uint);
                if hp.is_null() {
                    msg_start();
                    msg_puts_hl(
                        gettext(b"Unable to read block 0 from \0".as_ptr()
                            as *const ::core::ffi::c_char),
                        hl_id,
                        true_0 != 0,
                    );
                    msg_outtrans((*mfp).mf_fname, hl_id, true_0 != 0);
                    msg_puts_hl(
                        gettext(
                            b"\nMaybe no changes were made or Nvim did not update the swap file.\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        hl_id,
                        true_0 != 0,
                    );
                    msg_end();
                } else {
                    b0p = (*hp).bh_data as *mut ZeroBlock;
                    if strncmp(
                        &raw mut (*b0p).b0_version as *mut ::core::ffi::c_char,
                        b"VIM 3.0\0".as_ptr() as *const ::core::ffi::c_char,
                        7 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        msg_start();
                        msg_outtrans((*mfp).mf_fname, 0 as ::core::ffi::c_int, true_0 != 0);
                        msg_puts_hl(
                            gettext(b" cannot be used with this version of Nvim.\n\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            0 as ::core::ffi::c_int,
                            true_0 != 0,
                        );
                        msg_puts_hl(
                            gettext(
                                b"Use Vim version 3.0.\n\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            0 as ::core::ffi::c_int,
                            true_0 != 0,
                        );
                        msg_end();
                    } else if ml_check_b0_id(b0p) as ::core::ffi::c_int == FAIL {
                        semsg(
                            gettext(b"E307: %s does not look like a Nvim swap file\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*mfp).mf_fname,
                        );
                    } else if b0_magic_wrong(b0p) {
                        msg_start();
                        msg_outtrans((*mfp).mf_fname, hl_id, true_0 != 0);
                        msg_puts_hl(
                            gettext(b" cannot be used on this computer.\n\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            hl_id,
                            true_0 != 0,
                        );
                        msg_puts_hl(
                            gettext(b"The file was created on \0".as_ptr()
                                as *const ::core::ffi::c_char),
                            hl_id,
                            true_0 != 0,
                        );
                        (*b0p).b0_fname[0 as ::core::ffi::c_int as usize] =
                            NUL as ::core::ffi::c_char;
                        msg_puts_hl(
                            &raw mut (*b0p).b0_hname as *mut ::core::ffi::c_char,
                            hl_id,
                            true_0 != 0,
                        );
                        msg_puts_hl(
                            gettext(b",\nor the file has been damaged.\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            hl_id,
                            true_0 != 0,
                        );
                        msg_end();
                    } else {
                        if (*mfp).mf_page_size
                            != b0_read_number(
                                &raw mut (*b0p).b0_page_size as *mut ::core::ffi::c_char,
                            ) as ::core::ffi::c_uint
                        {
                            let mut previous_page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
                            mf_new_page_size(
                                mfp,
                                b0_read_number(
                                    &raw mut (*b0p).b0_page_size as *mut ::core::ffi::c_char,
                                ) as ::core::ffi::c_uint,
                            );
                            if (*mfp).mf_page_size < previous_page_size {
                                msg_start();
                                msg_outtrans((*mfp).mf_fname, hl_id, true_0 != 0);
                                msg_puts_hl(
                                    gettext(
                                        b" has been damaged (page size is smaller than minimum value).\n\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    hl_id,
                                    true_0 != 0,
                                );
                                msg_end();
                                break '_theend;
                            } else {
                                let mut size: off_T = lseek((*mfp).mf_fd, 0 as __off_t, SEEK_END);
                                (*mfp).mf_blocknr_max = (if size <= 0 as off_T {
                                    0 as off_T
                                } else {
                                    size / (*mfp).mf_page_size as off_T
                                })
                                    as blocknr_T;
                                (*mfp).mf_infile_count = (*mfp).mf_blocknr_max;
                                p = xmalloc((*mfp).mf_page_size as size_t)
                                    as *mut ::core::ffi::c_char;
                                memmove(
                                    p as *mut ::core::ffi::c_void,
                                    (*hp).bh_data,
                                    previous_page_size as size_t,
                                );
                                xfree((*hp).bh_data);
                                (*hp).bh_data = p as *mut ::core::ffi::c_void;
                                b0p = (*hp).bh_data as *mut ZeroBlock;
                            }
                        }
                        if directly {
                            expand_env(
                                &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char,
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                                MAXPATHL,
                            );
                            if setfname(
                                curbuf.get(),
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                true_0 != 0,
                            ) == FAIL
                            {
                                break '_theend;
                            }
                        }
                        msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
                        msg_ext_skip_flush.set(true_0 != 0);
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            (*mfp).mf_fname,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Using swap file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                        );
                        if !buf_spname(curbuf.get()).is_null() {
                            xstrlcpy(
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                                buf_spname(curbuf.get()),
                                MAXPATHL as size_t,
                            );
                        } else {
                            home_replace(
                                ::core::ptr::null::<buf_T>(),
                                (*curbuf.get()).b_ffname,
                                NameBuff.ptr() as *mut ::core::ffi::c_char,
                                MAXPATHL as size_t,
                                true_0 != 0,
                            );
                        }
                        msg_putchar('\n' as ::core::ffi::c_int);
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Original file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                        );
                        msg_putchar('\n' as ::core::ffi::c_int);
                        msg_ext_skip_flush.set(false_0 != 0);
                        org_file_info = FileInfo {
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
                        swp_file_info = FileInfo {
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
                        mtime = b0_read_number(&raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int;
                        if !(*curbuf.get()).b_ffname.is_null()
                            && os_fileinfo((*curbuf.get()).b_ffname, &raw mut org_file_info)
                                as ::core::ffi::c_int
                                != 0
                            && (os_fileinfo((*mfp).mf_fname, &raw mut swp_file_info)
                                as ::core::ffi::c_int
                                != 0
                                && org_file_info.stat.st_mtim.tv_sec
                                    > swp_file_info.stat.st_mtim.tv_sec
                                || org_file_info.stat.st_mtim.tv_sec
                                    != mtime as ::core::ffi::c_long)
                        {
                            emsg(gettext(
                                b"E308: Warning: Original file may have been changed\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        ui_flush();
                        b0_ff = (*b0p).b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                            - 2 as ::core::ffi::c_int)
                            as usize] as ::core::ffi::c_int
                            & B0_FF_MASK;
                        if (*b0p).b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                            - 2 as ::core::ffi::c_int)
                            as usize] as ::core::ffi::c_int
                            & B0_HAS_FENC
                            != 0
                        {
                            let mut fnsize: ::core::ffi::c_int =
                                B0_FNAME_SIZE_NOCRYPT as ::core::ffi::c_int;
                            p = (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                                .offset(fnsize as isize);
                            while p > &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char
                                && *p.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != NUL
                            {
                                p = p.offset(-1);
                            }
                            b0_fenc = xstrnsave(
                                p,
                                (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                                    .offset(fnsize as isize)
                                    .offset_from(p) as size_t,
                            );
                        }
                        mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                        hp = ::core::ptr::null_mut::<bhdr_T>();
                        while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 {
                            ml_delete(1 as linenr_T);
                        }
                        if !(*curbuf.get()).b_ffname.is_null() {
                            orig_file_status = readfile(
                                (*curbuf.get()).b_ffname,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                0 as linenr_T,
                                0 as linenr_T,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                ::core::ptr::null_mut::<exarg_T>(),
                                READ_NEW as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                        }
                        if b0_ff != 0 as ::core::ffi::c_int {
                            set_fileformat(
                                b0_ff - 1 as ::core::ffi::c_int,
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                        }
                        if !b0_fenc.is_null() {
                            set_option_value_give_err(
                                kOptFileencoding,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: cstr_as_string(b0_fenc),
                                    },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                            xfree(b0_fenc as *mut ::core::ffi::c_void);
                        }
                        unchanged(curbuf.get(), true_0 != 0, true_0 != 0);
                        bnum = 1 as blocknr_T;
                        page_count = 1 as ::core::ffi::c_uint;
                        lnum = 0 as linenr_T;
                        line_count = 0 as linenr_T;
                        idx = 0 as ::core::ffi::c_int;
                        error = 0 as ::core::ffi::c_int;
                        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
                        (*buf).b_ml.ml_stack = ::core::ptr::null_mut::<infoptr_T>();
                        (*buf).b_ml.ml_stack_size = 0 as ::core::ffi::c_int;
                        cannot_open = (*curbuf.get()).b_ffname.is_null();
                        serious_error = false_0 != 0;
                        's_977: while !got_int.get() {
                            if !hp.is_null() {
                                mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                            }
                            's_533: {
                                hp = mf_get(mfp, bnum, page_count);
                                if hp.is_null() {
                                    if bnum == 1 as blocknr_T {
                                        semsg(
                                            gettext(
                                                b"E309: Unable to read block 1 from %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            (*mfp).mf_fname,
                                        );
                                        break '_theend;
                                    } else {
                                        error += 1;
                                        let c2rust_fresh0 = lnum;
                                        lnum = lnum + 1;
                                        ml_append(
                                            c2rust_fresh0,
                                            gettext(b"???MANY LINES MISSING\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            0 as colnr_T,
                                            true_0 != 0,
                                        );
                                    }
                                } else {
                                    let mut pp: *mut PointerBlock =
                                        (*hp).bh_data as *mut PointerBlock;
                                    if (*pp).pb_id as ::core::ffi::c_int
                                        == PTR_ID as ::core::ffi::c_int
                                    {
                                        let mut ptr_block_error: bool = false_0 != 0;
                                        if (*pp).pb_count_max as ::core::ffi::c_int
                                            != ((*mfp).mf_page_size as usize)
                                                .wrapping_sub(8 as usize)
                                                .wrapping_div(::core::mem::size_of::<PointerEntry>())
                                                as uint16_t as ::core::ffi::c_int
                                        {
                                            ptr_block_error = true_0 != 0;
                                            (*pp).pb_count_max = ((*mfp).mf_page_size as usize)
                                                .wrapping_sub(8 as usize)
                                                .wrapping_div(::core::mem::size_of::<PointerEntry>())
                                                as uint16_t;
                                        }
                                        if (*pp).pb_count as ::core::ffi::c_int
                                            > (*pp).pb_count_max as ::core::ffi::c_int
                                        {
                                            ptr_block_error = true_0 != 0;
                                            (*pp).pb_count = (*pp).pb_count_max;
                                        }
                                        if ptr_block_error {
                                            emsg(gettext(
                                                (e_warning_pointer_block_corrupted.ptr()
                                                    as *const _)
                                                    as *const ::core::ffi::c_char,
                                            ));
                                        }
                                        if idx == 0 as ::core::ffi::c_int
                                            && line_count != 0 as linenr_T
                                        {
                                            let mut i_0: ::core::ffi::c_int =
                                                0 as ::core::ffi::c_int;
                                            while i_0 < (*pp).pb_count as ::core::ffi::c_int {
                                                line_count -= (*(&raw mut (*pp).pb_pointer
                                                    as *mut PointerEntry)
                                                    .offset(i_0 as isize))
                                                .pe_line_count;
                                                i_0 += 1;
                                            }
                                            if line_count != 0 as linenr_T {
                                                error += 1;
                                                let c2rust_fresh1 = lnum;
                                                lnum = lnum + 1;
                                                ml_append(
                                                    c2rust_fresh1,
                                                    gettext(b"???LINE COUNT WRONG\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    0 as colnr_T,
                                                    true_0 != 0,
                                                );
                                            }
                                        }
                                        if (*pp).pb_count as ::core::ffi::c_int
                                            == 0 as ::core::ffi::c_int
                                        {
                                            let c2rust_fresh2 = lnum;
                                            lnum = lnum + 1;
                                            ml_append(
                                                c2rust_fresh2,
                                                gettext(b"???EMPTY BLOCK\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                0 as colnr_T,
                                                true_0 != 0,
                                            );
                                            error += 1;
                                        } else if idx < (*pp).pb_count as ::core::ffi::c_int {
                                            if (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                                .offset(idx as isize))
                                            .pe_bnum
                                                < 0 as blocknr_T
                                            {
                                                if !cannot_open {
                                                    line_count = (*(&raw mut (*pp).pb_pointer
                                                        as *mut PointerEntry)
                                                        .offset(idx as isize))
                                                    .pe_line_count;
                                                    let mut pe_old_lnum: linenr_T =
                                                        (*(&raw mut (*pp).pb_pointer
                                                            as *mut PointerEntry)
                                                            .offset(idx as isize))
                                                        .pe_old_lnum;
                                                    if line_count <= 0 as linenr_T
                                                        || pe_old_lnum < 1 as linenr_T
                                                        || readfile(
                                                            (*curbuf.get()).b_ffname,
                                                            ::core::ptr::null_mut::<
                                                                ::core::ffi::c_char,
                                                            >(
                                                            ),
                                                            lnum,
                                                            pe_old_lnum - 1 as linenr_T,
                                                            line_count,
                                                            ::core::ptr::null_mut::<exarg_T>(),
                                                            0 as ::core::ffi::c_int,
                                                            false_0 != 0,
                                                        ) != OK
                                                    {
                                                        cannot_open = true_0 != 0;
                                                    } else {
                                                        lnum += line_count;
                                                    }
                                                }
                                                if cannot_open {
                                                    error += 1;
                                                    let c2rust_fresh3 = lnum;
                                                    lnum = lnum + 1;
                                                    ml_append(
                                                        c2rust_fresh3,
                                                        gettext(b"???LINES MISSING\0".as_ptr()
                                                            as *const ::core::ffi::c_char),
                                                        0 as colnr_T,
                                                        true_0 != 0,
                                                    );
                                                }
                                                idx += 1;
                                                break 's_533;
                                            } else {
                                                let mut top: ::core::ffi::c_int = ml_add_stack(buf);
                                                ip = (*buf).b_ml.ml_stack.offset(top as isize);
                                                (*ip).ip_bnum = bnum;
                                                (*ip).ip_index = idx;
                                                bnum = (*(&raw mut (*pp).pb_pointer
                                                    as *mut PointerEntry)
                                                    .offset(idx as isize))
                                                .pe_bnum;
                                                line_count = (*(&raw mut (*pp).pb_pointer
                                                    as *mut PointerEntry)
                                                    .offset(idx as isize))
                                                .pe_line_count;
                                                page_count = (*(&raw mut (*pp).pb_pointer
                                                    as *mut PointerEntry)
                                                    .offset(idx as isize))
                                                .pe_page_count
                                                    as ::core::ffi::c_uint;
                                                if page_count < 1 as ::core::ffi::c_uint
                                                    || bnum + page_count as blocknr_T
                                                        > (*mfp).mf_blocknr_max + 1 as blocknr_T
                                                {
                                                    error += 1;
                                                    let c2rust_fresh4 = lnum;
                                                    lnum = lnum + 1;
                                                    ml_append(
                                                        c2rust_fresh4,
                                                        gettext(
                                                            b"???ILLEGAL BLOCK NUMBER\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        0 as ::core::ffi::c_int,
                                                        true_0 != 0,
                                                    );
                                                    idx = (*ip).ip_index + 1 as ::core::ffi::c_int;
                                                    bnum = (*ip).ip_bnum;
                                                    page_count = 1 as ::core::ffi::c_uint;
                                                    (*buf).b_ml.ml_stack_top -= 1;
                                                    break 's_533;
                                                } else {
                                                    idx = 0 as ::core::ffi::c_int;
                                                    break 's_533;
                                                }
                                            }
                                        }
                                    } else {
                                        let mut dp: *mut DataBlock =
                                            (*hp).bh_data as *mut DataBlock;
                                        if (*dp).db_id as ::core::ffi::c_int
                                            != DATA_ID as ::core::ffi::c_int
                                        {
                                            if bnum == 1 as blocknr_T {
                                                semsg(
                                                    gettext(
                                                        b"E310: Block 1 ID wrong (%s not a .swp file?)\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    (*mfp).mf_fname,
                                                );
                                                break '_theend;
                                            } else {
                                                error += 1;
                                                let c2rust_fresh5 = lnum;
                                                lnum = lnum + 1;
                                                ml_append(
                                                    c2rust_fresh5,
                                                    gettext(b"???BLOCK MISSING\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    0 as colnr_T,
                                                    true_0 != 0,
                                                );
                                            }
                                        } else {
                                            let mut has_error: bool = false_0 != 0;
                                            if page_count.wrapping_mul((*mfp).mf_page_size)
                                                != (*dp).db_txt_end
                                            {
                                                let c2rust_fresh6 = lnum;
                                                lnum = lnum + 1;
                                                ml_append(
                                                    c2rust_fresh6,
                                                    gettext(
                                                        b"??? from here until ???END lines may be messed up\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    0 as colnr_T,
                                                    true_0 != 0,
                                                );
                                                error += 1;
                                                has_error = true_0 != 0;
                                                (*dp).db_txt_end =
                                                    page_count.wrapping_mul((*mfp).mf_page_size);
                                            }
                                            *(dp as *mut ::core::ffi::c_char)
                                                .offset((*dp).db_txt_end as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize)) =
                                                NUL as ::core::ffi::c_char;
                                            if line_count as ::core::ffi::c_long
                                                != (*dp).db_line_count
                                            {
                                                let c2rust_fresh7 = lnum;
                                                lnum = lnum + 1;
                                                ml_append(
                                                    c2rust_fresh7,
                                                    gettext(
                                                        b"??? from here until ???END lines may have been inserted/deleted\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    0 as colnr_T,
                                                    true_0 != 0,
                                                );
                                                error += 1;
                                                has_error = true_0 != 0;
                                            }
                                            let mut did_questions: bool = false_0 != 0;
                                            let mut i_1: ::core::ffi::c_int =
                                                0 as ::core::ffi::c_int;
                                            while (i_1 as ::core::ffi::c_long) < (*dp).db_line_count
                                            {
                                                if (&raw mut (*dp).db_index
                                                    as *mut ::core::ffi::c_uint)
                                                    .offset(i_1 as isize)
                                                    as *mut ::core::ffi::c_char
                                                    >= (dp as *mut ::core::ffi::c_char)
                                                        .offset((*dp).db_txt_start as isize)
                                                {
                                                    error += 1;
                                                    let c2rust_fresh8 = lnum;
                                                    lnum = lnum + 1;
                                                    ml_append(
                                                        c2rust_fresh8,
                                                        gettext(
                                                            b"??? lines may be missing\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        0 as colnr_T,
                                                        true_0 != 0,
                                                    );
                                                    break;
                                                } else {
                                                    let mut txt_start: ::core::ffi::c_int =
                                                        (*(&raw mut (*dp).db_index
                                                            as *mut ::core::ffi::c_uint)
                                                            .offset(i_1 as isize)
                                                            & DB_INDEX_MASK)
                                                            as ::core::ffi::c_int;
                                                    's_868: {
                                                        if txt_start
                                                            <= HEADER_SIZE as ::core::ffi::c_int
                                                            || txt_start
                                                                >= (*dp).db_txt_end
                                                                    as ::core::ffi::c_int
                                                        {
                                                            error += 1;
                                                            if did_questions {
                                                                break 's_868;
                                                            } else {
                                                                did_questions = true_0 != 0;
                                                                p = b"???\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                                    as *mut ::core::ffi::c_char;
                                                            }
                                                        } else {
                                                            did_questions = false_0 != 0;
                                                            p = (dp as *mut ::core::ffi::c_char)
                                                                .offset(txt_start as isize);
                                                        }
                                                        let c2rust_fresh9 = lnum;
                                                        lnum = lnum + 1;
                                                        ml_append(
                                                            c2rust_fresh9,
                                                            p,
                                                            0 as colnr_T,
                                                            true_0 != 0,
                                                        );
                                                    }
                                                    i_1 += 1;
                                                }
                                            }
                                            if has_error {
                                                let c2rust_fresh10 = lnum;
                                                lnum = lnum + 1;
                                                ml_append(
                                                    c2rust_fresh10,
                                                    gettext(b"???END\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    0 as colnr_T,
                                                    true_0 != 0,
                                                );
                                            }
                                        }
                                    }
                                }
                                if (*buf).b_ml.ml_stack_top == 0 as ::core::ffi::c_int {
                                    break 's_977;
                                }
                                (*buf).b_ml.ml_stack_top -= 1;
                                ip = (*buf)
                                    .b_ml
                                    .ml_stack
                                    .offset((*buf).b_ml.ml_stack_top as isize);
                                bnum = (*ip).ip_bnum;
                                idx = (*ip).ip_index + 1 as ::core::ffi::c_int;
                                page_count = 1 as ::core::ffi::c_uint;
                            }
                            line_breakcheck();
                        }
                        if orig_file_status != OK
                            || (*curbuf.get()).b_ml.ml_line_count
                                != lnum * 2 as linenr_T + 1 as linenr_T
                        {
                            if !((*curbuf.get()).b_ml.ml_line_count == 2 as linenr_T
                                && *ml_get(1 as linenr_T) as ::core::ffi::c_int == NUL)
                            {
                                changed_internal(curbuf.get());
                                buf_inc_changedtick(curbuf.get());
                            }
                        } else {
                            idx = 1 as ::core::ffi::c_int;
                            while idx as linenr_T <= lnum {
                                p = xstrnsave(
                                    ml_get(idx as linenr_T),
                                    ml_get_len(idx as linenr_T) as size_t,
                                );
                                let mut i_2: ::core::ffi::c_int =
                                    strcmp(p, ml_get(idx as linenr_T + lnum));
                                xfree(p as *mut ::core::ffi::c_void);
                                if i_2 != 0 as ::core::ffi::c_int {
                                    changed_internal(curbuf.get());
                                    buf_inc_changedtick(curbuf.get());
                                    break;
                                } else {
                                    idx += 1;
                                }
                            }
                        }
                        while (*curbuf.get()).b_ml.ml_line_count > lnum
                            && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0
                        {
                            ml_delete((*curbuf.get()).b_ml.ml_line_count);
                        }
                        (*curbuf.get()).b_flags |= BF_RECOVERED;
                        check_cursor(curwin.get());
                        msg_ext_skip_flush.set(!got_int.get());
                        recoverymode.set(false_0 != 0);
                        if got_int.get() {
                            emsg(gettext(b"E311: Recovery Interrupted\0".as_ptr()
                                as *const ::core::ffi::c_char));
                        } else if error != 0 {
                            (*no_wait_return.ptr()) += 1;
                            msg_ext_set_kind(b"emsg\0".as_ptr() as *const ::core::ffi::c_char);
                            msg(
                                b">>>>>>>>>>>>>\n\0".as_ptr() as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            );
                            emsg(
                                gettext(
                                    b"E312: Errors detected while recovering; look for lines starting with ???\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                ),
                            );
                            (*no_wait_return.ptr()) -= 1;
                            msg_putchar('\n' as ::core::ffi::c_int);
                            msg(
                                gettext(b"See \":help E312\" for more information.\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                0 as ::core::ffi::c_int,
                            );
                            msg(
                                b"\n>>>>>>>>>>>>>\0".as_ptr() as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            );
                        } else {
                            msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
                            if (*curbuf.get()).b_changed != 0 {
                                msg(
                                    gettext(
                                        b"Recovery completed. You should check if everything is OK.\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    0 as ::core::ffi::c_int,
                                );
                                msg_puts(
                                    gettext(
                                        b"\n(You might want to write out this file under another name\n\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                );
                                msg_puts(gettext(
                                    b"and run diff with the original file to check for changes)\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            } else {
                                msg(
                                    gettext(
                                        b"Recovery completed. Buffer contents equals file contents.\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    0 as ::core::ffi::c_int,
                                );
                            }
                            msg_puts(gettext(
                                b"\nYou may want to delete the .swp file now.\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                            if swapfile_proc_running(b0p, fname_used) != 0 {
                                msg_puts(gettext(b"\nNote: process STILL RUNNING: \0".as_ptr()
                                    as *const ::core::ffi::c_char));
                                msg_outnum(b0_read_number(
                                    &raw mut (*b0p).b0_pid as *mut ::core::ffi::c_char,
                                ) as ::core::ffi::c_int);
                            }
                            if !ui_has(kUIMessages) {
                                msg_puts(b"\n\n\0".as_ptr() as *const ::core::ffi::c_char);
                            }
                            cmdline_row.set(msg_row.get());
                        }
                        redraw_curbuf_later(UPD_NOT_VALID);
                    }
                }
            }
        }
    }
    msg_ext_skip_flush.set(false_0 != 0);
    xfree(fname_used as *mut ::core::ffi::c_void);
    recoverymode.set(false_0 != 0);
    if !mfp.is_null() {
        if !hp.is_null() {
            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
        }
        mf_close(mfp, false_0 != 0);
    }
    if !buf.is_null() {
        xfree((*buf).b_ml.ml_stack as *mut ::core::ffi::c_void);
        xfree(buf as *mut ::core::ffi::c_void);
    }
    if serious_error as ::core::ffi::c_int != 0 && called_from_main != 0 {
        ml_close(curbuf.get(), true_0);
    } else {
        apply_autocmds(
            EVENT_BUFREADPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*curbuf.get()).b_fname,
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*curbuf.get()).b_fname,
            false_0 != 0,
            curbuf.get(),
        );
    };
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
