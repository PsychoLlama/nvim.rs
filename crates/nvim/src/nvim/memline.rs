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
use crate::src::nvim::map::mh_get_int64_t;
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
    __assert_fail, __errno_location, close, gettext, lseek, memchr, memmove, readlink, strcasecmp,
    strcmp, strcpy, strlen, strncasecmp, strncmp,
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
    __off_t, FileInfo, Map_int64_t_ptr_t, OptInt, OptVal, OptValData, OptValType, String_0,
    StringBuilder, Timestamp, VimVarIndex, bhdr_T, blocknr_T, buf_T, chunksize_T, colnr_T, dict_T,
    exarg_T, file_comparison, flush_buffers_T, infoptr_T, int16_t, int32_t, int64_t, linenr_T,
    list_T, memfile_T, mfdirty_T, off_T, pos_T, ptr_t, ptrdiff_t, size_t, ssize_t, time_t, uint8_t,
    uint16_t, uint32_t, uint64_t, uv_stat_t, uv_timespec_t, uv_uid_t, varnumber_T,
};
use crate::src::nvim::ui::{ui_flush, ui_has};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::version::Versions;
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
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
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
            long_to_char(
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
                long_to_char(
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
pub unsafe extern "C" fn ml_setname(mut buf: *mut buf_T) {
    let mut success: bool = false_0 != 0;
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        if p_uc.get() != 0 as OptInt
            && (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            ml_open_file(buf);
        }
        return;
    }
    let mut dirp: *mut ::core::ffi::c_char = p_dir.get();
    let mut found_existing_dir: bool = false_0 != 0;
    while *dirp as ::core::ffi::c_int != NUL {
        let mut fname: *mut ::core::ffi::c_char = findswapname(
            buf,
            &raw mut dirp,
            (*mfp).mf_fname,
            &raw mut found_existing_dir,
        );
        if dirp.is_null() {
            break;
        }
        if fname.is_null() {
            continue;
        }
        if path_fnamecmp(fname, (*mfp).mf_fname) == 0 as ::core::ffi::c_int {
            xfree(fname as *mut ::core::ffi::c_void);
            success = true_0 != 0;
            break;
        } else {
            if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                close((*mfp).mf_fd);
                (*mfp).mf_fd = -1 as ::core::ffi::c_int;
            }
            if vim_rename((*mfp).mf_fname, fname) == 0 as ::core::ffi::c_int {
                success = true_0 != 0;
                mf_free_fnames(mfp);
                mf_set_fnames(mfp, fname);
                ml_upd_block0(buf, UB_SAME_DIR);
                break;
            } else {
                xfree(fname as *mut ::core::ffi::c_void);
            }
        }
    }
    if (*mfp).mf_fd == -1 as ::core::ffi::c_int {
        (*mfp).mf_fd = os_open((*mfp).mf_fname, O_RDWR, 0 as ::core::ffi::c_int);
        if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E301: Oops, lost the swap file!!!\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        os_set_cloexec((*mfp).mf_fd);
    }
    if !success {
        emsg(gettext(
            b"E302: Could not rename swap file\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
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
pub unsafe extern "C" fn ml_timestamp(mut buf: *mut buf_T) {
    ml_upd_block0(buf, UB_FNAME);
}
unsafe extern "C" fn ml_check_b0_id(mut b0p: *mut ZeroBlock) -> bool {
    return (*b0p).b0_id[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        == BLOCK0_ID0 as ::core::ffi::c_int
        && (*b0p).b0_id[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == BLOCK0_ID1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ml_check_b0_strings(mut b0p: *mut ZeroBlock) -> bool {
    return !memchr(
        &raw mut (*b0p).b0_version as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        NUL,
        10 as size_t,
    )
    .is_null()
        && !memchr(
            &raw mut (*b0p).b0_uname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            NUL,
            B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
        )
        .is_null()
        && !memchr(
            &raw mut (*b0p).b0_hname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            NUL,
            B0_HNAME_SIZE as ::core::ffi::c_int as size_t,
        )
        .is_null()
        && !memchr(
            &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            NUL,
            B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
        )
        .is_null();
}
unsafe extern "C" fn ml_upd_block0(mut buf: *mut buf_T, mut what: upd_block0_T) {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if mfp.is_null() || {
        hp = mf_get(mfp, 0 as blocknr_T, 1 as ::core::ffi::c_uint);
        hp.is_null()
    } {
        return;
    }
    let mut b0p: *mut ZeroBlock = (*hp).bh_data as *mut ZeroBlock;
    if ml_check_b0_id(b0p) as ::core::ffi::c_int == FAIL {
        iemsg(gettext(
            b"E304: ml_upd_block0(): Didn't get block 0??\0".as_ptr() as *const ::core::ffi::c_char,
        ));
    } else if what as ::core::ffi::c_uint == UB_FNAME as ::core::ffi::c_int as ::core::ffi::c_uint {
        set_b0_fname(b0p, buf);
    } else {
        set_b0_dir_flag(b0p, buf);
    }
    mf_put(mfp, hp, true_0 != 0, false_0 != 0);
}
unsafe extern "C" fn set_b0_fname(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    if (*buf).b_ffname.is_null() {
        (*b0p).b0_fname[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    } else {
        let mut uname: [::core::ffi::c_char; 40] = [0; 40];
        home_replace(
            ::core::ptr::null::<buf_T>(),
            (*buf).b_ffname,
            &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char,
            B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
            true_0 != 0,
        );
        if (*b0p).b0_fname[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == '~' as ::core::ffi::c_int
        {
            let mut retval: ::core::ffi::c_int = os_get_username(
                &raw mut uname as *mut ::core::ffi::c_char,
                B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
            );
            let mut ulen: size_t = strlen(&raw mut uname as *mut ::core::ffi::c_char);
            let mut flen: size_t = strlen(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char);
            if retval == FAIL
                || ulen.wrapping_add(flen)
                    > (B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as size_t
            {
                xstrlcpy(
                    &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char,
                    (*buf).b_ffname,
                    B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
                );
            } else {
                memmove(
                    (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                        .offset(ulen as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    flen,
                );
                memmove(
                    (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    &raw mut uname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ulen,
                );
            }
        }
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
        if os_fileinfo((*buf).b_ffname, &raw mut file_info) {
            long_to_char(
                file_info.stat.st_mtim.tv_sec,
                &raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char,
            );
            long_to_char(
                os_fileinfo_inode(&raw mut file_info) as ::core::ffi::c_long,
                &raw mut (*b0p).b0_ino as *mut ::core::ffi::c_char,
            );
            buf_store_file_info(buf, &raw mut file_info);
            (*buf).b_mtime_read = (*buf).b_mtime;
            (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
        } else {
            long_to_char(
                0 as ::core::ffi::c_long,
                &raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char,
            );
            long_to_char(
                0 as ::core::ffi::c_long,
                &raw mut (*b0p).b0_ino as *mut ::core::ffi::c_char,
            );
            (*buf).b_mtime = 0 as int64_t;
            (*buf).b_mtime_ns = 0 as int64_t;
            (*buf).b_mtime_read = 0 as int64_t;
            (*buf).b_mtime_read_ns = 0 as int64_t;
            (*buf).b_orig_size = 0 as uint64_t;
            (*buf).b_orig_mode = 0 as ::core::ffi::c_int;
        }
    }
    add_b0_fenc(b0p, curbuf.get());
}
unsafe extern "C" fn set_b0_dir_flag(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    if same_directory((*(*buf).b_ml.ml_mfp).mf_fname, (*buf).b_ffname) {
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] = ((*b0p)
            .b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            | B0_SAME_DIR)
            as ::core::ffi::c_char;
    } else {
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] = ((*b0p)
            .b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            & !B0_SAME_DIR)
            as ::core::ffi::c_char;
    };
}
unsafe extern "C" fn add_b0_fenc(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    let size: ::core::ffi::c_int = B0_FNAME_SIZE_NOCRYPT as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = strlen((*buf).b_p_fenc) as ::core::ffi::c_int;
    if strlen(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
        + n
        + 1 as ::core::ffi::c_int
        > size
    {
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] = ((*b0p)
            .b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            & !B0_HAS_FENC)
            as ::core::ffi::c_char;
    } else {
        memmove(
            (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                .offset(size as isize)
                .offset(-(n as isize)) as *mut ::core::ffi::c_void,
            (*buf).b_p_fenc as *const ::core::ffi::c_void,
            n as size_t,
        );
        *(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
            .offset(size as isize)
            .offset(-(n as isize))
            .offset(-(1 as ::core::ffi::c_int as isize)) = NUL as ::core::ffi::c_char;
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] = ((*b0p)
            .b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            | B0_HAS_FENC)
            as ::core::ffi::c_char;
    };
}
unsafe extern "C" fn swapfile_proc_running(
    mut b0p: *const ZeroBlock,
    mut swap_fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut st: FileInfo = FileInfo {
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
    let mut uptime: ::core::ffi::c_double = 0.;
    if os_fileinfo(swap_fname, &raw mut st) as ::core::ffi::c_int != 0
        && uv_uptime(&raw mut uptime) == 0 as ::core::ffi::c_int
        && (st.stat.st_mtim.tv_sec as Timestamp) < os_time().wrapping_sub(uptime as Timestamp)
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut pid: ::core::ffi::c_int =
        char_to_long(&raw const (*b0p).b0_pid as *const ::core::ffi::c_char) as ::core::ffi::c_int;
    return if os_proc_running(pid) as ::core::ffi::c_int != 0 {
        pid
    } else {
        0 as ::core::ffi::c_int
    };
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
                    } else if b0_magic_wrong(b0p) != 0 {
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
                            != char_to_long(
                                &raw mut (*b0p).b0_page_size as *mut ::core::ffi::c_char,
                            ) as ::core::ffi::c_uint
                        {
                            let mut previous_page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
                            mf_new_page_size(
                                mfp,
                                char_to_long(
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
                        mtime = char_to_long(&raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char)
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
                                msg_outnum(char_to_long(
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
pub unsafe extern "C" fn recover_names(
    mut fname: *mut ::core::ffi::c_char,
    mut do_list: bool,
    mut ret_list: *mut list_T,
    mut nr: ::core::ffi::c_int,
    mut fname_out: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut num_names: ::core::ffi::c_int = 0;
    let mut names: [*mut ::core::ffi::c_char; 6] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 6];
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut file_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut fname_res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname_buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if !fname.is_null() {
        fname_res = if resolve_symlink(fname, &raw mut fname_buf as *mut ::core::ffi::c_char) == OK
        {
            &raw mut fname_buf as *mut ::core::ffi::c_char
        } else {
            fname
        };
    }
    msg_ext_skip_flush.set(true_0 != 0);
    if do_list {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        msg(
            gettext(b"Swap files found:\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    let mut dir_name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    dir_name.data =
        xmalloc(strlen(p_dir.get()).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut dirp: *mut ::core::ffi::c_char = p_dir.get();
    while *dirp != 0 {
        dir_name.size = copy_option_part(
            &raw mut dirp,
            dir_name.data,
            31000 as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if *dir_name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *dir_name.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            if fname.is_null() {
                names[0 as ::core::ffi::c_int as usize] = xmemdupz(
                    b"*.sw?\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                names[1 as ::core::ffi::c_int as usize] = xmemdupz(
                    b".*.sw?\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                names[2 as ::core::ffi::c_int as usize] = xmemdupz(
                    b".sw?\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                num_names = 3 as ::core::ffi::c_int;
            } else {
                num_names = recov_file_names(
                    &raw mut names as *mut *mut ::core::ffi::c_char,
                    fname_res,
                    true_0 != 0,
                );
            }
        } else if fname.is_null() {
            names[0 as ::core::ffi::c_int as usize] = concat_fnames(
                dir_name.data,
                b"*.sw?\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            names[1 as ::core::ffi::c_int as usize] = concat_fnames(
                dir_name.data,
                b".*.sw?\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            names[2 as ::core::ffi::c_int as usize] = concat_fnames(
                dir_name.data,
                b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            num_names = 3 as ::core::ffi::c_int;
        } else {
            p = dir_name.data.offset(dir_name.size as isize);
            if after_pathsep(dir_name.data, p) != 0
                && dir_name.size > 1 as size_t
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            {
                tail = make_percent_swname(dir_name.data, p, fname_res);
            } else {
                tail = path_tail(fname_res);
                tail = concat_fnames(dir_name.data, tail, true_0 != 0);
            }
            num_names = recov_file_names(
                &raw mut names as *mut *mut ::core::ffi::c_char,
                tail,
                false_0 != 0,
            );
            xfree(tail as *mut ::core::ffi::c_void);
        }
        let mut num_files: ::core::ffi::c_int = 0;
        if num_names == 0 as ::core::ffi::c_int {
            num_files = 0 as ::core::ffi::c_int;
        } else if expand_wildcards(
            num_names,
            &raw mut names as *mut *mut ::core::ffi::c_char,
            &raw mut num_files,
            &raw mut files,
            EW_KEEPALL as ::core::ffi::c_int
                | EW_FILE as ::core::ffi::c_int
                | EW_SILENT as ::core::ffi::c_int,
        ) == FAIL
        {
            num_files = 0 as ::core::ffi::c_int;
        }
        if *dirp as ::core::ffi::c_int == NUL
            && file_count + num_files == 0 as ::core::ffi::c_int
            && !fname.is_null()
        {
            let mut swapname: *mut ::core::ffi::c_char = modname(
                fname_res,
                b".swp\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
            if !swapname.is_null() {
                if os_path_exists(swapname) {
                    files = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                        as *mut *mut ::core::ffi::c_char;
                    *files.offset(0 as ::core::ffi::c_int as isize) = swapname;
                    swapname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    num_files = 1 as ::core::ffi::c_int;
                }
                xfree(swapname as *mut ::core::ffi::c_void);
            }
        }
        if !(*curbuf.get()).b_ml.ml_mfp.is_null()
            && {
                p = (*(*curbuf.get()).b_ml.ml_mfp).mf_fname;
                !p.is_null()
            }
            && ret_list.is_null()
        {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num_files {
                if path_full_compare(p, *files.offset(i as isize), true_0 != 0, false_0 != 0)
                    as ::core::ffi::c_uint
                    & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                    num_files -= 1;
                    if num_files == 0 as ::core::ffi::c_int {
                        xfree(files as *mut ::core::ffi::c_void);
                    } else {
                        while i < num_files {
                            *files.offset(i as isize) =
                                *files.offset((i + 1 as ::core::ffi::c_int) as isize);
                            i += 1;
                        }
                    }
                }
                i += 1;
            }
        }
        if nr > 0 as ::core::ffi::c_int {
            file_count += num_files;
            if nr <= file_count {
                *fname_out = xstrdup(
                    *files.offset((nr - 1 as ::core::ffi::c_int + num_files - file_count) as isize),
                );
                dirp = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        } else if do_list {
            if *dir_name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *dir_name.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
            {
                if fname.is_null() {
                    msg_puts(gettext(
                        b"   In current directory:\n\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                } else {
                    msg_puts(gettext(
                        b"   Using specified name:\n\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
            } else {
                msg_puts(gettext(
                    b"   In directory \0".as_ptr() as *const ::core::ffi::c_char
                ));
                msg_home_replace(dir_name.data);
                msg_puts(b":\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
            if num_files != 0 {
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < num_files {
                    file_count += 1;
                    msg_outnum(file_count);
                    msg_puts(b".    \0".as_ptr() as *const ::core::ffi::c_char);
                    msg_puts(path_tail(*files.offset(i_0 as isize)));
                    msg_putchar('\n' as ::core::ffi::c_int);
                    let mut msg_0: StringBuilder = KV_INITIAL_VALUE;
                    msg_0.capacity =
                        (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
                    msg_0.items = xrealloc(
                        msg_0.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(msg_0.capacity),
                    ) as *mut ::core::ffi::c_char;
                    swapfile_info(*files.offset(i_0 as isize), &raw mut msg_0);
                    let mut need_clear: bool = false_0 != 0;
                    msg_multiline(
                        String_0 {
                            data: msg_0.items,
                            size: msg_0.size,
                        },
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        false_0 != 0,
                        &raw mut need_clear,
                    );
                    xfree(msg_0.items as *mut ::core::ffi::c_void);
                    msg_0.capacity = 0 as size_t;
                    msg_0.size = msg_0.capacity;
                    msg_0.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    i_0 += 1;
                }
            } else {
                msg_puts(gettext(
                    b"      -- none --\n\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            ui_flush();
        } else if !ret_list.is_null() {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < num_files {
                let mut name: *mut ::core::ffi::c_char =
                    concat_fnames(dir_name.data, *files.offset(i_1 as isize), true_0 != 0);
                tv_list_append_allocated_string(ret_list, name);
                i_1 += 1;
            }
        } else {
            file_count += num_files;
        }
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < num_names {
            xfree(names[i_2 as usize] as *mut ::core::ffi::c_void);
            i_2 += 1;
        }
        if num_files > 0 as ::core::ffi::c_int {
            FreeWild(num_files, files);
        }
    }
    msg_ext_skip_flush.set(false_0 != 0);
    xfree(dir_name.data as *mut ::core::ffi::c_void);
    return file_count;
}
pub unsafe extern "C" fn make_percent_swname(
    mut dir: *mut ::core::ffi::c_char,
    mut dir_end: *mut ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut d: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut f: *mut ::core::ffi::c_char = fix_fname(if !name.is_null() {
        name
    } else {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    });
    if f.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut s: *mut ::core::ffi::c_char = xstrdup(f);
    d = s;
    while *d as ::core::ffi::c_int != NUL {
        if vim_ispathsep(*d as ::core::ffi::c_int) {
            *d = '%' as ::core::ffi::c_char;
        }
        d = d.offset(utfc_ptr2len(d) as isize);
    }
    *dir_end.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    d = concat_fnames(dir, s, true_0 != 0);
    xfree(s as *mut ::core::ffi::c_void);
    xfree(f as *mut ::core::ffi::c_void);
    return d;
}
static proc_running: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn swapfile_dict(mut fname: *const ::core::ffi::c_char, mut d: *mut dict_T) {
    let mut fd: ::core::ffi::c_int = 0;
    let mut b0: ZeroBlock = ZeroBlock {
        b0_id: [0; 2],
        b0_version: [0; 10],
        b0_page_size: [0; 4],
        b0_mtime: [0; 4],
        b0_ino: [0; 4],
        b0_pid: [0; 4],
        b0_uname: [0; 40],
        b0_hname: [0; 40],
        b0_fname: [0; 900],
        b0_magic_long: 0,
        b0_magic_int: 0,
        b0_magic_short: 0,
        b0_magic_char: 0,
    };
    fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
    if fd >= 0 as ::core::ffi::c_int {
        if read_eintr(
            fd,
            &raw mut b0 as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<ZeroBlock>(),
        ) as usize
            == ::core::mem::size_of::<ZeroBlock>()
        {
            if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL {
                tv_dict_add_str(
                    d,
                    b"error\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    b"Not a swap file\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else if b0_magic_wrong(&raw mut b0) != 0 {
                tv_dict_add_str(
                    d,
                    b"error\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    b"Magic number mismatch\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                tv_dict_add_str_len(
                    d,
                    b"version\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    &raw mut b0.b0_version as *mut ::core::ffi::c_char,
                    10 as ::core::ffi::c_int,
                );
                tv_dict_add_str_len(
                    d,
                    b"user\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                    &raw mut b0.b0_uname as *mut ::core::ffi::c_char,
                    B0_UNAME_SIZE as ::core::ffi::c_int,
                );
                tv_dict_add_str_len(
                    d,
                    b"host\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                    &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
                    B0_HNAME_SIZE as ::core::ffi::c_int,
                );
                tv_dict_add_str_len(
                    d,
                    b"fname\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                    B0_FNAME_SIZE_ORG as ::core::ffi::c_int,
                );
                tv_dict_add_nr(
                    d,
                    b"pid\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    swapfile_proc_running(&raw mut b0, fname) as varnumber_T,
                );
                tv_dict_add_nr(
                    d,
                    b"mtime\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    char_to_long(&raw mut b0.b0_mtime as *mut ::core::ffi::c_char) as varnumber_T,
                );
                tv_dict_add_nr(
                    d,
                    b"dirty\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    (if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
                        != 0
                    {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_nr(
                    d,
                    b"inode\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    char_to_long(&raw mut b0.b0_ino as *mut ::core::ffi::c_char) as varnumber_T,
                );
            }
        } else {
            tv_dict_add_str(
                d,
                b"error\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                b"Cannot read file\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        close(fd);
    } else {
        tv_dict_add_str(
            d,
            b"error\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            b"Cannot open file\0".as_ptr() as *const ::core::ffi::c_char,
        );
    };
}
unsafe extern "C" fn swapfile_info(
    mut fname: *mut ::core::ffi::c_char,
    mut msg_0: *mut StringBuilder,
) -> time_t {
    '_c2rust_label: {
        if !fname.is_null() {
        } else {
            __assert_fail(
                b"fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1545 as ::core::ffi::c_uint,
                b"time_t swapfile_info(char *, StringBuilder *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut b0: ZeroBlock = ZeroBlock {
        b0_id: [0; 2],
        b0_version: [0; 10],
        b0_page_size: [0; 4],
        b0_mtime: [0; 4],
        b0_ino: [0; 4],
        b0_pid: [0; 4],
        b0_uname: [0; 40],
        b0_hname: [0; 40],
        b0_fname: [0; 900],
        b0_magic_long: 0,
        b0_magic_int: 0,
        b0_magic_short: 0,
        b0_magic_char: 0,
    };
    let mut x: time_t = 0 as ::core::ffi::c_int as time_t;
    let mut uname: [::core::ffi::c_char; 40] = [0; 40];
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
    if os_fileinfo(fname, &raw mut file_info) {
        if os_get_uname(
            file_info.stat.st_uid as uv_uid_t,
            &raw mut uname as *mut ::core::ffi::c_char,
            B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
        ) == OK
        {
            kv_do_printf(
                msg_0,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                gettext(b"          owned by: \0".as_ptr() as *const ::core::ffi::c_char),
                &raw mut uname as *mut ::core::ffi::c_char,
            );
            kv_do_printf(
                msg_0,
                gettext(b"   dated: \0".as_ptr() as *const ::core::ffi::c_char),
            );
        } else {
            kv_do_printf(
                msg_0,
                gettext(b"             dated: \0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
        x = file_info.stat.st_mtim.tv_sec as time_t;
        let mut ctime_buf: [::core::ffi::c_char; 100] = [0; 100];
        kv_do_printf(
            msg_0,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            os_ctime_r(x, &mut ctime_buf, true),
        );
    }
    let mut fd: ::core::ffi::c_int = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
    if fd >= 0 as ::core::ffi::c_int {
        if read_eintr(
            fd,
            &raw mut b0 as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<ZeroBlock>(),
        ) as usize
            == ::core::mem::size_of::<ZeroBlock>()
        {
            if strncmp(
                &raw mut b0.b0_version as *mut ::core::ffi::c_char,
                b"VIM 3.0\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                kv_do_printf(
                    msg_0,
                    gettext(
                        b"         [from Vim version 3.0]\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                );
            } else if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL {
                kv_do_printf(
                    msg_0,
                    gettext(b"         [does not look like a Nvim swap file]\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
            } else if !ml_check_b0_strings(&raw mut b0) {
                kv_do_printf(
                    msg_0,
                    gettext(
                        b"         [garbled strings (not nul terminated)]\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                );
            } else {
                kv_do_printf(
                    msg_0,
                    gettext(b"         file name: \0".as_ptr() as *const ::core::ffi::c_char),
                );
                if b0.b0_fname[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL {
                    kv_do_printf(
                        msg_0,
                        gettext(b"[No Name]\0".as_ptr() as *const ::core::ffi::c_char),
                    );
                } else {
                    kv_do_printf(
                        msg_0,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                    );
                }
                kv_do_printf(
                    msg_0,
                    gettext(b"\n          modified: \0".as_ptr() as *const ::core::ffi::c_char),
                );
                kv_do_printf(
                    msg_0,
                    if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
                        != 0
                    {
                        gettext(b"YES\0".as_ptr() as *const ::core::ffi::c_char)
                    } else {
                        gettext(b"no\0".as_ptr() as *const ::core::ffi::c_char)
                    },
                );
                if *(&raw mut b0.b0_uname as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL
                {
                    kv_do_printf(
                        msg_0,
                        gettext(b"\n         user name: \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    kv_do_printf(
                        msg_0,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut b0.b0_uname as *mut ::core::ffi::c_char,
                    );
                }
                if *(&raw mut b0.b0_hname as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL
                {
                    if *(&raw mut b0.b0_uname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                        != NUL
                    {
                        kv_do_printf(
                            msg_0,
                            gettext(b"   host name: \0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    } else {
                        kv_do_printf(
                            msg_0,
                            gettext(
                                b"\n         host name: \0".as_ptr() as *const ::core::ffi::c_char
                            ),
                        );
                    }
                    kv_do_printf(
                        msg_0,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
                    );
                }
                if char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_long
                {
                    kv_do_printf(
                        msg_0,
                        gettext(b"\n        process ID: \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    kv_do_printf(
                        msg_0,
                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                        char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int,
                    );
                    proc_running.set(swapfile_proc_running(&raw mut b0, fname));
                    if proc_running.get() != 0 {
                        kv_do_printf(
                            msg_0,
                            gettext(b" (STILL RUNNING)\0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    }
                }
                if b0_magic_wrong(&raw mut b0) != 0 {
                    kv_do_printf(
                        msg_0,
                        gettext(b"\n         [not usable on this computer]\0".as_ptr()
                            as *const ::core::ffi::c_char),
                    );
                }
            }
        } else {
            kv_do_printf(
                msg_0,
                gettext(b"         [cannot be read]\0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
        close(fd);
    } else {
        kv_do_printf(
            msg_0,
            gettext(b"         [cannot be opened]\0".as_ptr() as *const ::core::ffi::c_char),
        );
    }
    kv_do_printf(msg_0, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    return x;
}
unsafe extern "C" fn swapfile_unchanged(mut fname: *mut ::core::ffi::c_char) -> bool {
    let mut b0: ZeroBlock = ZeroBlock {
        b0_id: [0; 2],
        b0_version: [0; 10],
        b0_page_size: [0; 4],
        b0_mtime: [0; 4],
        b0_ino: [0; 4],
        b0_pid: [0; 4],
        b0_uname: [0; 40],
        b0_hname: [0; 40],
        b0_fname: [0; 900],
        b0_magic_long: 0,
        b0_magic_int: 0,
        b0_magic_short: 0,
        b0_magic_char: 0,
    };
    if !os_path_exists(fname) {
        return false_0 != 0;
    }
    let mut fd: ::core::ffi::c_int = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
    if fd < 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if read_eintr(
        fd,
        &raw mut b0 as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<ZeroBlock>(),
    ) as usize
        != ::core::mem::size_of::<ZeroBlock>()
    {
        close(fd);
        return false_0 != 0;
    }
    let mut ret: bool = true_0 != 0;
    if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL || b0_magic_wrong(&raw mut b0) != 0
    {
        ret = false_0 != 0;
    }
    if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
        != 0
    {
        ret = false_0 != 0;
    }
    if *(&raw mut b0.b0_hname as *mut ::core::ffi::c_char) as ::core::ffi::c_int == NUL {
        ret = false_0 != 0;
    } else {
        let mut hostname: [::core::ffi::c_char; 40] = [0; 40];
        os_get_hostname(
            &raw mut hostname as *mut ::core::ffi::c_char,
            B0_HNAME_SIZE as ::core::ffi::c_int as size_t,
        );
        hostname[(B0_HNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
            NUL as ::core::ffi::c_char;
        b0.b0_hname[(B0_HNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
            NUL as ::core::ffi::c_char;
        if strcasecmp(
            &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
            &raw mut hostname as *mut ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
        {
            ret = false_0 != 0;
        }
    }
    if char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char) == 0 as ::core::ffi::c_long
        || swapfile_proc_running(&raw mut b0, fname) != 0
    {
        ret = false_0 != 0;
    }
    close(fd);
    return ret;
}
unsafe extern "C" fn recov_file_names(
    mut names: *mut *mut ::core::ffi::c_char,
    mut path: *mut ::core::ffi::c_char,
    mut prepend_dot: bool,
) -> ::core::ffi::c_int {
    let mut num_names: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if prepend_dot {
        *names.offset(num_names as isize) = modname(
            path,
            b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        if (*names.offset(num_names as isize)).is_null() {
            return num_names;
        }
        num_names += 1;
    }
    *names.offset(num_names as isize) = concat_fnames(
        path,
        b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    if num_names >= 1 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char =
            *names.offset((num_names - 1 as ::core::ffi::c_int) as isize);
        let mut i: ::core::ffi::c_int =
            strlen(*names.offset((num_names - 1 as ::core::ffi::c_int) as isize))
                as ::core::ffi::c_int
                - strlen(*names.offset(num_names as isize)) as ::core::ffi::c_int;
        if i > 0 as ::core::ffi::c_int {
            p = p.offset(i as isize);
        }
        if strcmp(p, *names.offset(num_names as isize)) != 0 as ::core::ffi::c_int {
            num_names += 1;
        } else {
            xfree(*names.offset(num_names as isize) as *mut ::core::ffi::c_void);
        }
    } else {
        num_names += 1;
    }
    return num_names;
}
pub unsafe extern "C" fn ml_sync_all(
    mut check_file: ::core::ffi::c_int,
    mut check_char: ::core::ffi::c_int,
    mut do_fsync: bool,
) {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !((*buf).b_ml.ml_mfp.is_null() || (*(*buf).b_ml.ml_mfp).mf_fname.is_null()) {
            ml_flush_line(buf, false_0 != 0);
            ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
            if bufIsChanged(buf) as ::core::ffi::c_int != 0
                && check_file != 0
                && mf_need_trans((*buf).b_ml.ml_mfp) as ::core::ffi::c_int != 0
                && !(*buf).b_ffname.is_null()
            {
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
                if !os_fileinfo((*buf).b_ffname, &raw mut file_info)
                    || file_info.stat.st_mtim.tv_sec as int64_t != (*buf).b_mtime_read
                    || file_info.stat.st_mtim.tv_nsec as int64_t != (*buf).b_mtime_read_ns
                    || os_fileinfo_size(&raw mut file_info) != (*buf).b_orig_size
                {
                    ml_preserve(buf, false_0 != 0, do_fsync);
                    did_check_timestamps.set(false_0 != 0);
                    need_check_timestamps.set(true_0 != 0);
                }
            }
            if (*(*buf).b_ml.ml_mfp).mf_dirty as ::core::ffi::c_uint
                == MF_DIRTY_YES as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                mf_sync(
                    (*buf).b_ml.ml_mfp,
                    (if check_char != 0 {
                        MFS_STOP as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if do_fsync as ::core::ffi::c_int != 0
                        && bufIsChanged(buf) as ::core::ffi::c_int != 0
                    {
                        MFS_FLUSH as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                );
                if check_char != 0 && os_char_avail() as ::core::ffi::c_int != 0 {
                    break;
                }
            }
        }
        buf = (*buf).b_next;
    }
}
pub unsafe extern "C" fn ml_preserve(mut buf: *mut buf_T, mut message: bool, mut do_fsync: bool) {
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    let mut got_int_save: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    if mfp.is_null() || (*mfp).mf_fname.is_null() {
        if message {
            emsg(gettext(
                b"E313: Cannot preserve, there is no swap file\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
        return;
    }
    got_int.set(false_0 != 0);
    ml_flush_line(buf, false_0 != 0);
    ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
    let mut status: ::core::ffi::c_int = mf_sync(
        mfp,
        MFS_ALL as ::core::ffi::c_int
            | (if do_fsync as ::core::ffi::c_int != 0 {
                MFS_FLUSH as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
    '_theend: {
        if mf_need_trans(mfp) as ::core::ffi::c_int != 0 && !got_int.get() {
            let mut lnum: linenr_T = 1 as linenr_T;
            while mf_need_trans(mfp) as ::core::ffi::c_int != 0 && lnum <= (*buf).b_ml.ml_line_count
            {
                let mut hp: *mut bhdr_T = ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
                if hp.is_null() {
                    status = FAIL;
                    break '_theend;
                } else {
                    lnum = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
                }
            }
            ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
            if mf_sync(
                mfp,
                MFS_ALL as ::core::ffi::c_int
                    | (if do_fsync as ::core::ffi::c_int != 0 {
                        MFS_FLUSH as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
            ) == FAIL
            {
                status = FAIL;
            }
            (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
        }
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | got_int_save != 0);
    if message {
        if status == OK {
            msg(
                gettext(b"File preserved\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else {
            emsg(gettext(
                b"E314: Preserve failed\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}
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
unsafe extern "C" fn ml_get_buf_impl(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut will_change: bool,
) -> *mut ::core::ffi::c_char {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static questions: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new([0; 4]);
    if (*buf).b_ml.ml_mfp.is_null() {
        (*buf).b_ml.ml_line_textlen = 1 as ::core::ffi::c_int as colnr_T;
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    '_errorret: {
        if lnum > (*buf).b_ml.ml_line_count {
            if recursive.get() == 0 as ::core::ffi::c_int {
                (*recursive.ptr()) += 1;
                siemsg(
                    gettext(
                        (e_ml_get_invalid_lnum_nr.ptr() as *const _) as *const ::core::ffi::c_char,
                    ),
                    lnum as int64_t,
                );
                (*recursive.ptr()) -= 1;
            }
            ml_flush_line(buf, false_0 != 0);
        } else {
            lnum = if lnum > 1 as linenr_T {
                lnum
            } else {
                1 as linenr_T
            };
            if (*buf).b_ml.ml_line_lnum != lnum {
                ml_flush_line(buf, false_0 != 0);
                let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                hp = ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
                if hp.is_null() {
                    if recursive.get() == 0 as ::core::ffi::c_int {
                        (*recursive.ptr()) += 1;
                        get_trans_bufname(buf);
                        shorten_dir(NameBuff.ptr() as *mut ::core::ffi::c_char);
                        siemsg(
                            gettext(
                                (e_ml_get_cannot_find_line_nr_in_buffer_nr_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            lnum as int64_t,
                            (*buf).handle,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                        );
                        (*recursive.ptr()) -= 1;
                    }
                    break '_errorret;
                } else {
                    let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
                    let mut idx: ::core::ffi::c_int = lnum as ::core::ffi::c_int
                        - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                    let mut start: ::core::ffi::c_uint =
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                            & DB_INDEX_MASK;
                    let mut end: ::core::ffi::c_uint = if idx == 0 as ::core::ffi::c_int {
                        (*dp).db_txt_end
                    } else {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((idx - 1 as ::core::ffi::c_int) as isize)
                            & DB_INDEX_MASK
                    };
                    (*buf).b_ml.ml_line_ptr =
                        (dp as *mut ::core::ffi::c_char).offset(start as isize);
                    (*buf).b_ml.ml_line_textlen = end.wrapping_sub(start) as colnr_T;
                    (*buf).b_ml.ml_line_lnum = lnum;
                    (*buf).b_ml.ml_flags &= !(ML_LINE_DIRTY | ML_ALLOCATED);
                }
            }
            if will_change {
                (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
                ml_add_deleted_len_buf(buf, (*buf).b_ml.ml_line_ptr, -1 as ssize_t);
            }
            return (*buf).b_ml.ml_line_ptr;
        }
    }
    strcpy(
        questions.ptr() as *mut ::core::ffi::c_char,
        b"???\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    (*buf).b_ml.ml_line_textlen = 4 as ::core::ffi::c_int as colnr_T;
    (*buf).b_ml.ml_line_lnum = lnum;
    return questions.ptr() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn ml_line_alloced() -> ::core::ffi::c_int {
    return (*curbuf.get()).b_ml.ml_flags & ML_LINE_DIRTY;
}
unsafe extern "C" fn ml_append_int(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line_arg: *mut ::core::ffi::c_char,
    mut len_arg: colnr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut db_idx: ::core::ffi::c_int = 0;
    let mut line_count: ::core::ffi::c_int = 0;
    let mut dp: *mut DataBlock = ::core::ptr::null_mut::<DataBlock>();
    let mut line: *mut ::core::ffi::c_char = line_arg;
    let mut len: colnr_T = len_arg;
    if lnum > (*buf).b_ml.ml_line_count || (*buf).b_ml.ml_mfp.is_null() {
        return FAIL;
    }
    if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
        lowest_marked.set(lnum + 1 as linenr_T);
    }
    if len == 0 as ::core::ffi::c_int {
        len = (strlen(line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
    }
    let mut space_needed: int64_t = len as int64_t + INDEX_SIZE as int64_t;
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    let mut page_size: int64_t = (*mfp).mf_page_size as int64_t;
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    '_theend: {
        hp = ml_find_line(
            buf,
            if lnum == 0 as linenr_T {
                1 as linenr_T
            } else {
                lnum
            },
            ML_INSERT as ::core::ffi::c_int,
        );
        if !hp.is_null() {
            (*buf).b_ml.ml_flags &= !ML_EMPTY;
            db_idx = 0;
            if lnum == 0 as linenr_T {
                db_idx = -1 as ::core::ffi::c_int;
            } else {
                db_idx = (lnum - (*buf).b_ml.ml_locked_low) as ::core::ffi::c_int;
            }
            line_count = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
                - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
            dp = (*hp).bh_data as *mut DataBlock;
            if ((*dp).db_free as int64_t) < space_needed
                && db_idx == line_count - 1 as ::core::ffi::c_int
                && lnum < (*buf).b_ml.ml_line_count
            {
                (*buf).b_ml.ml_locked_lineadd -= 1;
                (*buf).b_ml.ml_locked_high -= 1;
                hp = ml_find_line(buf, lnum + 1 as linenr_T, ML_INSERT as ::core::ffi::c_int);
                if hp.is_null() {
                    break '_theend;
                } else {
                    db_idx = -1 as ::core::ffi::c_int;
                    line_count = ((*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low)
                        as ::core::ffi::c_int;
                    dp = (*hp).bh_data as *mut DataBlock;
                }
            }
            if (*buf).b_prev_line_count == 0 as ::core::ffi::c_int {
                (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count as ::core::ffi::c_int;
            }
            (*buf).b_ml.ml_line_count += 1;
            if (*dp).db_free as int64_t >= space_needed {
                (*dp).db_txt_start = (*dp).db_txt_start.wrapping_sub(len as ::core::ffi::c_uint);
                (*dp).db_free = (*dp)
                    .db_free
                    .wrapping_sub(space_needed as ::core::ffi::c_uint);
                (*dp).db_line_count += 1;
                if line_count > db_idx + 1 as ::core::ffi::c_int {
                    let mut offset: ::core::ffi::c_int = if db_idx < 0 as ::core::ffi::c_int {
                        (*dp).db_txt_end as ::core::ffi::c_int
                    } else {
                        (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(db_idx as isize)
                            & DB_INDEX_MASK) as ::core::ffi::c_int
                    };
                    memmove(
                        (dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize)
                            as *mut ::core::ffi::c_void,
                        (dp as *mut ::core::ffi::c_char)
                            .offset((*dp).db_txt_start as isize)
                            .offset(len as isize)
                            as *const ::core::ffi::c_void,
                        (offset as size_t).wrapping_sub(
                            ((*dp).db_txt_start as size_t).wrapping_add(len as size_t),
                        ),
                    );
                    let mut i: ::core::ffi::c_int = line_count - 1 as ::core::ffi::c_int;
                    while i > db_idx {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((i + 1 as ::core::ffi::c_int) as isize) =
                            (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset(i as isize))
                            .wrapping_sub(len as ::core::ffi::c_uint);
                        i -= 1;
                    }
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset((db_idx + 1 as ::core::ffi::c_int) as isize) =
                        (offset as colnr_T - len) as ::core::ffi::c_uint;
                } else {
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset((db_idx + 1 as ::core::ffi::c_int) as isize) = (*dp).db_txt_start;
                }
                memmove(
                    (dp as *mut ::core::ffi::c_char).offset(
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((db_idx + 1 as ::core::ffi::c_int) as isize)
                            as isize,
                    ) as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    len as size_t,
                );
                if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset((db_idx + 1 as ::core::ffi::c_int) as isize) |= DB_MARKED;
                }
                (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
                if flags & ML_APPEND_NEW as ::core::ffi::c_int == 0 {
                    (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
                }
            } else {
                let mut line_count_left: ::core::ffi::c_int = 0;
                let mut line_count_right: ::core::ffi::c_int = 0;
                let mut page_count_left: ::core::ffi::c_int = 0;
                let mut page_count_right: ::core::ffi::c_int = 0;
                let mut hp_left: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                let mut hp_right: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                let mut hp_new: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                let mut lines_moved: ::core::ffi::c_int = 0;
                let mut data_moved: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut total_moved: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut stack_idx: ::core::ffi::c_int = 0;
                let mut in_left: bool = false;
                let mut lnum_left: linenr_T = 0;
                let mut lnum_right: linenr_T = 0;
                let mut pp_new: *mut PointerBlock = ::core::ptr::null_mut::<PointerBlock>();
                if db_idx < 0 as ::core::ffi::c_int {
                    lines_moved = 0 as ::core::ffi::c_int;
                    in_left = true_0 != 0;
                } else {
                    lines_moved = line_count - db_idx - 1 as ::core::ffi::c_int;
                    if lines_moved == 0 as ::core::ffi::c_int {
                        in_left = false_0 != 0;
                    } else {
                        data_moved = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(db_idx as isize)
                            & DB_INDEX_MASK)
                            .wrapping_sub((*dp).db_txt_start)
                            as ::core::ffi::c_int;
                        total_moved = data_moved + lines_moved * INDEX_SIZE as ::core::ffi::c_int;
                        if (*dp).db_free as int64_t + total_moved as int64_t >= space_needed {
                            in_left = true_0 != 0;
                            space_needed = total_moved as int64_t;
                        } else {
                            in_left = false_0 != 0;
                            space_needed += total_moved as int64_t;
                        }
                    }
                }
                let mut page_count: int64_t =
                    (space_needed + HEADER_SIZE as int64_t + page_size - 1 as int64_t) / page_size;
                hp_new = ml_new_data(
                    mfp,
                    flags & ML_APPEND_NEW as ::core::ffi::c_int != 0,
                    page_count,
                );
                if db_idx < 0 as ::core::ffi::c_int {
                    hp_left = hp_new;
                    hp_right = hp;
                    line_count_left = 0 as ::core::ffi::c_int;
                    line_count_right = line_count;
                } else {
                    hp_left = hp;
                    hp_right = hp_new;
                    line_count_left = line_count;
                    line_count_right = 0 as ::core::ffi::c_int;
                }
                let mut dp_right: *mut DataBlock = (*hp_right).bh_data as *mut DataBlock;
                let mut dp_left: *mut DataBlock = (*hp_left).bh_data as *mut DataBlock;
                let mut bnum_left: blocknr_T = (*hp_left).bh_bnum;
                let mut bnum_right: blocknr_T = (*hp_right).bh_bnum;
                page_count_left = (*hp_left).bh_page_count as ::core::ffi::c_int;
                page_count_right = (*hp_right).bh_page_count as ::core::ffi::c_int;
                if !in_left {
                    (*dp_right).db_txt_start = (*dp_right)
                        .db_txt_start
                        .wrapping_sub(len as ::core::ffi::c_uint);
                    (*dp_right).db_free = (*dp_right).db_free.wrapping_sub(
                        (len as ::core::ffi::c_uint)
                            .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                    );
                    *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                        .offset(0 as ::core::ffi::c_int as isize) = (*dp_right).db_txt_start;
                    if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                        *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                            .offset(0 as ::core::ffi::c_int as isize) |= DB_MARKED;
                    }
                    memmove(
                        (dp_right as *mut ::core::ffi::c_char)
                            .offset((*dp_right).db_txt_start as isize)
                            as *mut ::core::ffi::c_void,
                        line as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                    line_count_right += 1;
                }
                if lines_moved != 0 {
                    (*dp_right).db_txt_start = (*dp_right)
                        .db_txt_start
                        .wrapping_sub(data_moved as ::core::ffi::c_uint);
                    (*dp_right).db_free = (*dp_right)
                        .db_free
                        .wrapping_sub(total_moved as ::core::ffi::c_uint);
                    memmove(
                        (dp_right as *mut ::core::ffi::c_char)
                            .offset((*dp_right).db_txt_start as isize)
                            as *mut ::core::ffi::c_void,
                        (dp_left as *mut ::core::ffi::c_char)
                            .offset((*dp_left).db_txt_start as isize)
                            as *const ::core::ffi::c_void,
                        data_moved as size_t,
                    );
                    let mut offset_0: ::core::ffi::c_int = (*dp_right)
                        .db_txt_start
                        .wrapping_sub((*dp_left).db_txt_start)
                        as ::core::ffi::c_int;
                    (*dp_left).db_txt_start = (*dp_left)
                        .db_txt_start
                        .wrapping_add(data_moved as ::core::ffi::c_uint);
                    (*dp_left).db_free = (*dp_left)
                        .db_free
                        .wrapping_add(total_moved as ::core::ffi::c_uint);
                    let mut to: ::core::ffi::c_int = line_count_right;
                    let mut from: ::core::ffi::c_int = db_idx + 1 as ::core::ffi::c_int;
                    while from < line_count_left {
                        *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                            .offset(to as isize) = (*(&raw mut (*dp).db_index
                            as *mut ::core::ffi::c_uint)
                            .offset(from as isize))
                        .wrapping_add(offset_0 as ::core::ffi::c_uint);
                        from += 1;
                        to += 1;
                    }
                    line_count_right += lines_moved;
                    line_count_left -= lines_moved;
                }
                if in_left {
                    (*dp_left).db_txt_start = (*dp_left)
                        .db_txt_start
                        .wrapping_sub(len as ::core::ffi::c_uint);
                    (*dp_left).db_free = (*dp_left).db_free.wrapping_sub(
                        (len as ::core::ffi::c_uint)
                            .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                    );
                    *(&raw mut (*dp_left).db_index as *mut ::core::ffi::c_uint)
                        .offset(line_count_left as isize) = (*dp_left).db_txt_start;
                    if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                        *(&raw mut (*dp_left).db_index as *mut ::core::ffi::c_uint)
                            .offset(line_count_left as isize) |= DB_MARKED;
                    }
                    memmove(
                        (dp_left as *mut ::core::ffi::c_char)
                            .offset((*dp_left).db_txt_start as isize)
                            as *mut ::core::ffi::c_void,
                        line as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                    line_count_left += 1;
                }
                if db_idx < 0 as ::core::ffi::c_int {
                    lnum_left = lnum + 1 as linenr_T;
                    lnum_right = 0 as ::core::ffi::c_int as linenr_T;
                } else {
                    lnum_left = 0 as ::core::ffi::c_int as linenr_T;
                    if in_left {
                        lnum_right = lnum + 2 as linenr_T;
                    } else {
                        lnum_right = lnum + 1 as linenr_T;
                    }
                }
                (*dp_left).db_line_count = line_count_left as ::core::ffi::c_long;
                (*dp_right).db_line_count = line_count_right as ::core::ffi::c_long;
                if lines_moved != 0 || in_left as ::core::ffi::c_int != 0 {
                    (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
                }
                if flags & ML_APPEND_NEW as ::core::ffi::c_int == 0
                    && db_idx >= 0 as ::core::ffi::c_int
                    && in_left as ::core::ffi::c_int != 0
                {
                    (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
                }
                mf_put(mfp, hp_new, true_0 != 0, false_0 != 0);
                let mut lineadd: ::core::ffi::c_int = (*buf).b_ml.ml_locked_lineadd;
                (*buf).b_ml.ml_locked_lineadd = 0 as ::core::ffi::c_int;
                ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
                stack_idx = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
                while stack_idx >= 0 as ::core::ffi::c_int {
                    let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(stack_idx as isize);
                    let mut pb_idx: ::core::ffi::c_int = (*ip).ip_index;
                    hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
                    if hp.is_null() {
                        break '_theend;
                    }
                    let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
                    if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                        iemsg(gettext(
                            (e_pointer_block_id_wrong_three.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                        break '_theend;
                    } else if ((*pp).pb_count as ::core::ffi::c_int)
                        < (*pp).pb_count_max as ::core::ffi::c_int
                    {
                        if (pb_idx + 1 as ::core::ffi::c_int) < (*pp).pb_count as ::core::ffi::c_int
                        {
                            memmove(
                                (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 2 as ::core::ffi::c_int) as isize)
                                    as *mut ::core::ffi::c_void,
                                (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize)
                                    as *const ::core::ffi::c_void,
                                (((*pp).pb_count as ::core::ffi::c_int
                                    - pb_idx
                                    - 1 as ::core::ffi::c_int)
                                    as size_t)
                                    .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                            );
                        }
                        (*pp).pb_count = (*pp).pb_count.wrapping_add(1);
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_line_count = line_count_left as linenr_T;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_bnum = bnum_left;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_page_count = page_count_left;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                        .pe_line_count = line_count_right as linenr_T;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                        .pe_bnum = bnum_right;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                        .pe_page_count = page_count_right;
                        if lnum_left != 0 as linenr_T {
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_old_lnum = lnum_left;
                        }
                        if lnum_right != 0 as linenr_T {
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_old_lnum = lnum_right;
                        }
                        mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                        (*buf).b_ml.ml_stack_top = stack_idx + 1 as ::core::ffi::c_int;
                        if lineadd != 0 {
                            (*buf).b_ml.ml_stack_top -= 1;
                            ml_lineadd(buf, lineadd);
                            (*(*buf)
                                .b_ml
                                .ml_stack
                                .offset((*buf).b_ml.ml_stack_top as isize))
                            .ip_high = ((*(*buf)
                                .b_ml
                                .ml_stack
                                .offset((*buf).b_ml.ml_stack_top as isize))
                            .ip_high as ::core::ffi::c_int
                                + lineadd) as linenr_T;
                            (*buf).b_ml.ml_stack_top += 1;
                        }
                        break;
                    } else {
                        loop {
                            hp_new = ml_new_ptr(mfp);
                            if hp_new.is_null() {
                                break '_theend;
                            }
                            pp_new = (*hp_new).bh_data as *mut PointerBlock;
                            if (*hp).bh_bnum != 1 as blocknr_T {
                                break;
                            }
                            memmove(
                                pp_new as *mut ::core::ffi::c_void,
                                pp as *const ::core::ffi::c_void,
                                page_size as size_t,
                            );
                            (*pp).pb_count = 1 as uint16_t;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_bnum = (*hp_new).bh_bnum;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_line_count = (*buf).b_ml.ml_line_count;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_old_lnum = 1 as ::core::ffi::c_int as linenr_T;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_page_count = 1 as ::core::ffi::c_int;
                            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                            hp = hp_new;
                            pp = pp_new;
                            (*ip).ip_index = 0 as ::core::ffi::c_int;
                            stack_idx += 1;
                        }
                        total_moved =
                            (*pp).pb_count as ::core::ffi::c_int - pb_idx - 1 as ::core::ffi::c_int;
                        if total_moved != 0 {
                            memmove(
                                (&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize)
                                    as *const ::core::ffi::c_void,
                                (total_moved as size_t)
                                    .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                            );
                            (*pp_new).pb_count = total_moved as uint16_t;
                            (*pp).pb_count = ((*pp).pb_count as ::core::ffi::c_int
                                - (total_moved - 1 as ::core::ffi::c_int))
                                as uint16_t;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_bnum = bnum_right;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_line_count = line_count_right as linenr_T;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_page_count = page_count_right;
                            if lnum_right != 0 {
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                .pe_old_lnum = lnum_right;
                            }
                        } else {
                            (*pp_new).pb_count = 1 as uint16_t;
                            (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_bnum = bnum_right;
                            (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_line_count = line_count_right as linenr_T;
                            (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_page_count = page_count_right;
                            (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                .offset(0 as ::core::ffi::c_int as isize))
                            .pe_old_lnum = lnum_right;
                        }
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_bnum = bnum_left;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_line_count = line_count_left as linenr_T;
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(pb_idx as isize))
                        .pe_page_count = page_count_left;
                        if lnum_left != 0 {
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_old_lnum = lnum_left;
                        }
                        lnum_left = 0 as ::core::ffi::c_int as linenr_T;
                        lnum_right = 0 as ::core::ffi::c_int as linenr_T;
                        line_count_right = 0 as ::core::ffi::c_int;
                        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i_0 < (*pp_new).pb_count as ::core::ffi::c_int {
                            line_count_right += (*(&raw mut (*pp_new).pb_pointer
                                as *mut PointerEntry)
                                .offset(i_0 as isize))
                            .pe_line_count
                                as ::core::ffi::c_int;
                            i_0 += 1;
                        }
                        line_count_left = 0 as ::core::ffi::c_int;
                        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i_1 < (*pp).pb_count as ::core::ffi::c_int {
                            line_count_left += (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(i_1 as isize))
                            .pe_line_count
                                as ::core::ffi::c_int;
                            i_1 += 1;
                        }
                        bnum_left = (*hp).bh_bnum;
                        bnum_right = (*hp_new).bh_bnum;
                        page_count_left = 1 as ::core::ffi::c_int;
                        page_count_right = 1 as ::core::ffi::c_int;
                        mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                        mf_put(mfp, hp_new, true_0 != 0, false_0 != 0);
                        stack_idx -= 1;
                    }
                }
                if stack_idx < 0 as ::core::ffi::c_int {
                    iemsg(gettext(
                        b"E318: Updated too many blocks?\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
                }
            }
            ml_updatechunk(
                buf,
                lnum + 1 as linenr_T,
                len as ::core::ffi::c_int,
                ML_CHNK_ADDLINE,
            );
            ret = OK;
        }
    }
    return ret;
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
unsafe extern "C" fn ml_delete_int(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
        (*lowest_marked.ptr()) -= 1;
    }
    if (*buf).b_ml.ml_line_count == 1 as linenr_T {
        if flags & ML_DEL_MESSAGE as ::core::ffi::c_int != 0 {
            set_keep_msg(
                gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        }
        let mut i: ::core::ffi::c_int = ml_replace_buf(
            buf,
            1 as linenr_T,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            true_0 != 0,
            false_0 != 0,
        );
        (*buf).b_ml.ml_flags |= ML_EMPTY;
        return i;
    }
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if mfp.is_null() {
        return FAIL;
    }
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    hp = ml_find_line(buf, lnum, ML_DELETE as ::core::ffi::c_int);
    if hp.is_null() {
        return FAIL;
    }
    let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
    let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
        - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int =
        lnum as ::core::ffi::c_int - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
    if (*buf).b_prev_line_count == 0 as ::core::ffi::c_int {
        (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count as ::core::ffi::c_int;
    }
    (*buf).b_ml.ml_line_count -= 1;
    let mut line_start: ::core::ffi::c_int =
        (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
            & DB_INDEX_MASK) as ::core::ffi::c_int;
    let mut line_size: ::core::ffi::c_int = 0;
    if idx == 0 as ::core::ffi::c_int {
        line_size = (*dp)
            .db_txt_end
            .wrapping_sub(line_start as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    } else {
        line_size = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
            .offset((idx - 1 as ::core::ffi::c_int) as isize)
            & DB_INDEX_MASK)
            .wrapping_sub(line_start as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if line_size >= 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"line_size >= 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2687 as ::core::ffi::c_uint,
                b"int ml_delete_int(buf_T *, linenr_T, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    ml_add_deleted_len_buf(
        buf,
        (dp as *mut ::core::ffi::c_char).offset(line_start as isize),
        (line_size - 1 as ::core::ffi::c_int) as ssize_t,
    );
    let mut ret: ::core::ffi::c_int = FAIL;
    '_theend: {
        's_274: {
            if count == 1 as ::core::ffi::c_int {
                mf_free(mfp, hp);
                (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
                let mut stack_idx: ::core::ffi::c_int =
                    (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
                loop {
                    if stack_idx < 0 as ::core::ffi::c_int {
                        break 's_274;
                    }
                    (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
                    let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(stack_idx as isize);
                    idx = (*ip).ip_index;
                    hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
                    if hp.is_null() {
                        break '_theend;
                    }
                    let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
                    if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                        iemsg(gettext(
                            (e_pointer_block_id_wrong_four.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                        break '_theend;
                    } else {
                        (*pp).pb_count = (*pp).pb_count.wrapping_sub(1);
                        count = (*pp).pb_count as ::core::ffi::c_int;
                        if count == 0 as ::core::ffi::c_int {
                            mf_free(mfp, hp);
                            stack_idx -= 1;
                        } else {
                            if count != idx {
                                memmove(
                                    (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset((idx + 1 as ::core::ffi::c_int) as isize)
                                        as *const ::core::ffi::c_void,
                                    ((count - idx) as size_t)
                                        .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                                );
                            }
                            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                            (*buf).b_ml.ml_stack_top = stack_idx;
                            if (*buf).b_ml.ml_locked_lineadd != 0 as ::core::ffi::c_int {
                                ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
                                (*(*buf)
                                    .b_ml
                                    .ml_stack
                                    .offset((*buf).b_ml.ml_stack_top as isize))
                                .ip_high = ((*(*buf)
                                    .b_ml
                                    .ml_stack
                                    .offset((*buf).b_ml.ml_stack_top as isize))
                                .ip_high
                                    as ::core::ffi::c_int
                                    + (*buf).b_ml.ml_locked_lineadd)
                                    as linenr_T;
                            }
                            (*buf).b_ml.ml_stack_top += 1;
                            break 's_274;
                        }
                    }
                }
            } else {
                let mut text_start: ::core::ffi::c_int = (*dp).db_txt_start as ::core::ffi::c_int;
                memmove(
                    (dp as *mut ::core::ffi::c_char)
                        .offset(text_start as isize)
                        .offset(line_size as isize) as *mut ::core::ffi::c_void,
                    (dp as *mut ::core::ffi::c_char).offset(text_start as isize)
                        as *const ::core::ffi::c_void,
                    (line_start - text_start) as size_t,
                );
                let mut i_0: ::core::ffi::c_int = idx;
                while i_0 < count - 1 as ::core::ffi::c_int {
                    *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i_0 as isize) =
                        (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((i_0 + 1 as ::core::ffi::c_int) as isize))
                        .wrapping_add(line_size as ::core::ffi::c_uint);
                    i_0 += 1;
                }
                (*dp).db_free = (*dp).db_free.wrapping_add(
                    (line_size as ::core::ffi::c_uint)
                        .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                );
                (*dp).db_txt_start = (*dp)
                    .db_txt_start
                    .wrapping_add(line_size as ::core::ffi::c_uint);
                (*dp).db_line_count -= 1;
                (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
            }
        }
        ml_updatechunk(buf, lnum, line_size, ML_CHNK_DELLINE);
        ret = OK;
    }
    return ret;
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
unsafe extern "C" fn ml_flush_line(mut buf: *mut buf_T, mut noalloc: bool) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if (*buf).b_ml.ml_line_lnum == 0 as linenr_T || (*buf).b_ml.ml_mfp.is_null() {
        return;
    }
    if (*buf).b_ml.ml_flags & ML_LINE_DIRTY != 0 {
        if entered.get() {
            return;
        }
        entered.set(true_0 != 0);
        (*buf).flush_count += 1;
        let mut lnum: linenr_T = (*buf).b_ml.ml_line_lnum;
        let mut new_line: *mut ::core::ffi::c_char = (*buf).b_ml.ml_line_ptr;
        let mut hp: *mut bhdr_T = ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
        if hp.is_null() {
            siemsg(
                gettext(b"E320: Cannot find line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                lnum as int64_t,
            );
        } else {
            let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
            let mut idx: ::core::ffi::c_int =
                lnum as ::core::ffi::c_int - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
            let mut start: ::core::ffi::c_int =
                (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                    & DB_INDEX_MASK) as ::core::ffi::c_int;
            let mut old_line: *mut ::core::ffi::c_char =
                (dp as *mut ::core::ffi::c_char).offset(start as isize);
            let mut old_len: ::core::ffi::c_int = 0;
            if idx == 0 as ::core::ffi::c_int {
                old_len = (*dp).db_txt_end as ::core::ffi::c_int - start;
            } else {
                old_len = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize)
                    & DB_INDEX_MASK) as ::core::ffi::c_int
                    - start;
            }
            let mut new_len: colnr_T = (*buf).b_ml.ml_line_textlen;
            let mut extra: ::core::ffi::c_int = new_len as ::core::ffi::c_int - old_len;
            if (*dp).db_free as ::core::ffi::c_int >= extra {
                let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high
                    as ::core::ffi::c_int
                    - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                if extra != 0 as ::core::ffi::c_int && idx < count - 1 as ::core::ffi::c_int {
                    memmove(
                        (dp as *mut ::core::ffi::c_char)
                            .offset((*dp).db_txt_start as isize)
                            .offset(-(extra as isize))
                            as *mut ::core::ffi::c_void,
                        (dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize)
                            as *const ::core::ffi::c_void,
                        (start - (*dp).db_txt_start as ::core::ffi::c_int) as size_t,
                    );
                    let mut i: ::core::ffi::c_int = idx + 1 as ::core::ffi::c_int;
                    while i < count {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(i as isize) =
                            (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset(i as isize))
                            .wrapping_sub(extra as ::core::ffi::c_uint);
                        i += 1;
                    }
                }
                *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize) =
                    (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize))
                        .wrapping_sub(extra as ::core::ffi::c_uint);
                (*dp).db_free = (*dp).db_free.wrapping_sub(extra as ::core::ffi::c_uint);
                (*dp).db_txt_start = (*dp)
                    .db_txt_start
                    .wrapping_sub(extra as ::core::ffi::c_uint);
                memmove(
                    old_line.offset(-(extra as isize)) as *mut ::core::ffi::c_void,
                    new_line as *const ::core::ffi::c_void,
                    new_len as size_t,
                );
                (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
                if extra != 0 as ::core::ffi::c_int {
                    ml_updatechunk(buf, lnum, extra, ML_CHNK_UPDLINE);
                }
            } else {
                ml_append_int(
                    buf,
                    lnum,
                    new_line,
                    new_len,
                    if *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                        & DB_MARKED
                        != 0
                    {
                        ML_APPEND_MARK as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                ml_delete_int(buf, lnum, 0 as ::core::ffi::c_int);
            }
        }
        if !noalloc {
            xfree(new_line as *mut ::core::ffi::c_void);
        }
        entered.set(false_0 != 0);
    } else if (*buf).b_ml.ml_flags & ML_ALLOCATED != 0 {
        '_c2rust_label: {
            if !noalloc {
            } else {
                __assert_fail(
                    b"!noalloc\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2969 as ::core::ffi::c_uint,
                    b"void ml_flush_line(buf_T *, _Bool)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        xfree((*buf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
    }
    (*buf).b_ml.ml_flags &= !(ML_LINE_DIRTY | ML_ALLOCATED);
    (*buf).b_ml.ml_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*buf).b_ml.ml_line_offset = 0 as size_t;
}
unsafe extern "C" fn ml_new_data(
    mut mfp: *mut memfile_T,
    mut negative: bool,
    mut page_count: int64_t,
) -> *mut bhdr_T {
    '_c2rust_label: {
        if page_count >= 0 as int64_t {
        } else {
            __assert_fail(
                b"page_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2981 as ::core::ffi::c_uint,
                b"bhdr_T *ml_new_data(memfile_T *, _Bool, int64_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut hp: *mut bhdr_T = mf_new(mfp, negative, page_count as ::core::ffi::c_uint);
    let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
    (*dp).db_id = DATA_ID as ::core::ffi::c_int as uint16_t;
    (*dp).db_txt_end = (page_count as ::core::ffi::c_uint).wrapping_mul((*mfp).mf_page_size);
    (*dp).db_txt_start = (*dp).db_txt_end;
    (*dp).db_free = (*dp)
        .db_txt_start
        .wrapping_sub(HEADER_SIZE as ::core::ffi::c_uint);
    (*dp).db_line_count = 0 as ::core::ffi::c_long;
    return hp;
}
unsafe extern "C" fn ml_new_ptr(mut mfp: *mut memfile_T) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = mf_new(mfp, false_0 != 0, 1 as ::core::ffi::c_uint);
    let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
    (*pp).pb_id = PTR_ID as ::core::ffi::c_int as uint16_t;
    (*pp).pb_count = 0 as uint16_t;
    (*pp).pb_count_max = ((*mfp).mf_page_size as usize)
        .wrapping_sub(8 as usize)
        .wrapping_div(::core::mem::size_of::<PointerEntry>()) as uint16_t;
    return hp;
}
unsafe extern "C" fn ml_find_line(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut action: ::core::ffi::c_int,
) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut top: ::core::ffi::c_int = 0;
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if !(*buf).b_ml.ml_locked.is_null() {
        if action & 0x10 as ::core::ffi::c_int != 0
            && (*buf).b_ml.ml_locked_low <= lnum
            && (*buf).b_ml.ml_locked_high >= lnum
        {
            if action == ML_INSERT as ::core::ffi::c_int {
                (*buf).b_ml.ml_locked_lineadd += 1;
                (*buf).b_ml.ml_locked_high += 1;
            } else if action == ML_DELETE as ::core::ffi::c_int {
                (*buf).b_ml.ml_locked_lineadd -= 1;
                (*buf).b_ml.ml_locked_high -= 1;
            }
            return (*buf).b_ml.ml_locked;
        }
        mf_put(
            mfp,
            (*buf).b_ml.ml_locked,
            (*buf).b_ml.ml_flags & ML_LOCKED_DIRTY != 0,
            (*buf).b_ml.ml_flags & ML_LOCKED_POS != 0,
        );
        (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
        if (*buf).b_ml.ml_locked_lineadd != 0 as ::core::ffi::c_int {
            ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
        }
    }
    if action == ML_FLUSH as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<bhdr_T>();
    }
    let mut bnum: blocknr_T = 1 as blocknr_T;
    let mut bnum2: blocknr_T = 0;
    let mut page_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut low: linenr_T = 1 as linenr_T;
    let mut high: linenr_T = (*buf).b_ml.ml_line_count;
    if action == ML_FIND as ::core::ffi::c_int {
        top = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
        while top >= 0 as ::core::ffi::c_int {
            let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(top as isize);
            if (*ip).ip_low <= lnum && (*ip).ip_high >= lnum {
                bnum = (*ip).ip_bnum;
                low = (*ip).ip_low;
                high = (*ip).ip_high;
                (*buf).b_ml.ml_stack_top = top;
                break;
            } else {
                top -= 1;
            }
        }
        if top < 0 as ::core::ffi::c_int {
            (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
        }
    } else {
        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
    }
    '_error_noblock: {
        loop {
            hp = mf_get(mfp, bnum, page_count as ::core::ffi::c_uint);
            if hp.is_null() {
                break '_error_noblock;
            }
            if action == ML_INSERT as ::core::ffi::c_int {
                high += 1;
            } else if action == ML_DELETE as ::core::ffi::c_int {
                high -= 1;
            }
            let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
            if (*dp).db_id as ::core::ffi::c_int == DATA_ID as ::core::ffi::c_int {
                (*buf).b_ml.ml_locked = hp;
                (*buf).b_ml.ml_locked_low = low;
                (*buf).b_ml.ml_locked_high = high;
                (*buf).b_ml.ml_locked_lineadd = 0 as ::core::ffi::c_int;
                (*buf).b_ml.ml_flags &= !(ML_LOCKED_DIRTY | ML_LOCKED_POS);
                return hp;
            }
            let mut pp: *mut PointerBlock = dp as *mut PointerBlock;
            if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                iemsg(gettext(
                    (e_pointer_block_id_wrong.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                break;
            } else {
                top = ml_add_stack(buf);
                let mut ip_0: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(top as isize);
                (*ip_0).ip_bnum = bnum;
                (*ip_0).ip_low = low;
                (*ip_0).ip_high = high;
                (*ip_0).ip_index = -1 as ::core::ffi::c_int;
                let mut dirty: bool = false_0 != 0;
                let mut idx: ::core::ffi::c_int = 0;
                idx = 0 as ::core::ffi::c_int;
                while idx < (*pp).pb_count as ::core::ffi::c_int {
                    let mut t: linenr_T = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                        .offset(idx as isize))
                    .pe_line_count;
                    low += t;
                    if low > lnum {
                        (*ip_0).ip_index = idx;
                        bnum = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(idx as isize))
                        .pe_bnum;
                        page_count = (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                            .offset(idx as isize))
                        .pe_page_count;
                        high = low - 1 as linenr_T;
                        low -= t;
                        if bnum < 0 as blocknr_T {
                            bnum2 = mf_trans_del(mfp, bnum);
                            if bnum != bnum2 {
                                bnum = bnum2;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(idx as isize))
                                .pe_bnum = bnum;
                                dirty = true_0 != 0;
                            }
                        }
                        break;
                    } else {
                        idx += 1;
                    }
                }
                if idx >= (*pp).pb_count as ::core::ffi::c_int {
                    if lnum > (*buf).b_ml.ml_line_count {
                        siemsg(
                            gettext(
                                (e_line_number_out_of_range_nr_past_the_end.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            lnum as int64_t - (*buf).b_ml.ml_line_count as int64_t,
                        );
                    } else {
                        siemsg(
                            gettext(
                                (e_line_count_wrong_in_block_nr.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            bnum,
                        );
                    }
                    break;
                } else {
                    if action == ML_DELETE as ::core::ffi::c_int {
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry).offset(idx as isize))
                            .pe_line_count -= 1;
                        dirty = true_0 != 0;
                    } else if action == ML_INSERT as ::core::ffi::c_int {
                        (*(&raw mut (*pp).pb_pointer as *mut PointerEntry).offset(idx as isize))
                            .pe_line_count += 1;
                        dirty = true_0 != 0;
                    }
                    mf_put(mfp, hp, dirty, false_0 != 0);
                }
            }
        }
        mf_put(mfp, hp, false_0 != 0, false_0 != 0);
    }
    if action == ML_DELETE as ::core::ffi::c_int {
        ml_lineadd(buf, 1 as ::core::ffi::c_int);
    } else if action == ML_INSERT as ::core::ffi::c_int {
        ml_lineadd(buf, -1 as ::core::ffi::c_int);
    }
    (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
    return ::core::ptr::null_mut::<bhdr_T>();
}
unsafe extern "C" fn ml_add_stack(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut top: ::core::ffi::c_int = (*buf).b_ml.ml_stack_top;
    if top == (*buf).b_ml.ml_stack_size {
        (*buf).b_ml.ml_stack_size += STACK_INCR;
        let mut new_size: size_t =
            ::core::mem::size_of::<infoptr_T>().wrapping_mul((*buf).b_ml.ml_stack_size as size_t);
        (*buf).b_ml.ml_stack =
            xrealloc((*buf).b_ml.ml_stack as *mut ::core::ffi::c_void, new_size) as *mut infoptr_T;
    }
    (*buf).b_ml.ml_stack_top += 1;
    return top;
}
unsafe extern "C" fn ml_lineadd(mut buf: *mut buf_T, mut count: ::core::ffi::c_int) {
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    let mut idx: ::core::ffi::c_int = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
    while idx >= 0 as ::core::ffi::c_int {
        let mut ip: *mut infoptr_T = (*buf).b_ml.ml_stack.offset(idx as isize);
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
        if hp.is_null() {
            break;
        }
        let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
        if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
            iemsg(gettext(
                (e_pointer_block_id_wrong_two.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            break;
        } else {
            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry).offset((*ip).ip_index as isize))
                .pe_line_count = ((*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                .offset((*ip).ip_index as isize))
            .pe_line_count as ::core::ffi::c_int
                + count) as linenr_T;
            (*ip).ip_high = ((*ip).ip_high as ::core::ffi::c_int + count) as linenr_T;
            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
            idx -= 1;
        }
    }
}
pub unsafe extern "C" fn resolve_symlink(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tmp: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if fname.is_null() {
        return FAIL;
    }
    xstrlcpy(
        &raw mut tmp as *mut ::core::ffi::c_char,
        fname,
        MAXPATHL as size_t,
    );
    loop {
        depth += 1;
        if depth == 100 as ::core::ffi::c_int {
            semsg(
                gettext(b"E773: Symlink loop for \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
            );
            return FAIL;
        }
        let mut ret: ::core::ffi::c_int = readlink(
            &raw mut tmp as *mut ::core::ffi::c_char,
            buf,
            (MAXPATHL - 1 as ::core::ffi::c_int) as size_t,
        ) as ::core::ffi::c_int;
        if ret <= 0 as ::core::ffi::c_int {
            if *__errno_location() == EINVAL || *__errno_location() == ENOENT {
                if depth == 1 as ::core::ffi::c_int {
                    return FAIL;
                }
                break;
            } else {
                return FAIL;
            }
        } else {
            *buf.offset(ret as isize) = NUL as ::core::ffi::c_char;
            if path_is_absolute(buf) {
                strcpy(&raw mut tmp as *mut ::core::ffi::c_char, buf);
            } else {
                let mut tail: *mut ::core::ffi::c_char =
                    path_tail(&raw mut tmp as *mut ::core::ffi::c_char);
                if strlen(tail).wrapping_add(strlen(buf)) >= MAXPATHL as size_t {
                    return FAIL;
                }
                strcpy(tail, buf);
            }
        }
    }
    return vim_FullName(
        &raw mut tmp as *mut ::core::ffi::c_char,
        buf,
        MAXPATHL as size_t,
        true_0 != 0,
    );
}
pub unsafe extern "C" fn makeswapname(
    mut fname: *mut ::core::ffi::c_char,
    mut _ffname: *mut ::core::ffi::c_char,
    mut _buf: *mut buf_T,
    mut dir_name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut fname_res: *mut ::core::ffi::c_char = fname;
    let mut fname_buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if resolve_symlink(fname, &raw mut fname_buf as *mut ::core::ffi::c_char) == OK {
        fname_res = &raw mut fname_buf as *mut ::core::ffi::c_char;
    }
    let mut len: ::core::ffi::c_int = strlen(dir_name) as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = dir_name.offset(len as isize);
    if after_pathsep(dir_name, s) != 0
        && len > 1 as ::core::ffi::c_int
        && *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *s.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        let mut r: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        s = make_percent_swname(dir_name, s, fname_res);
        if !s.is_null() {
            r = modname(
                s,
                b".swp\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            xfree(s as *mut ::core::ffi::c_void);
        }
        return r;
    }
    let mut r_0: *mut ::core::ffi::c_char = modname(
        fname_res,
        b".swp\0".as_ptr() as *const ::core::ffi::c_char,
        *dir_name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *dir_name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL,
    );
    if r_0.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    s = get_file_in_dir(r_0, dir_name);
    xfree(r_0 as *mut ::core::ffi::c_void);
    return s;
}
pub unsafe extern "C" fn get_file_in_dir(
    mut fname: *mut ::core::ffi::c_char,
    mut dname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
    if *dname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && *dname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        retval = xstrdup(fname);
    } else if *dname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && vim_ispathsep(*dname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        if tail == fname {
            retval = concat_fnames(
                dname.offset(2 as ::core::ffi::c_int as isize),
                tail,
                true_0 != 0,
            );
        } else {
            let mut save_char: ::core::ffi::c_char = *tail;
            *tail = NUL as ::core::ffi::c_char;
            let mut t: *mut ::core::ffi::c_char = concat_fnames(
                fname,
                dname.offset(2 as ::core::ffi::c_int as isize),
                true_0 != 0,
            );
            *tail = save_char;
            retval = concat_fnames(t, tail, true_0 != 0);
            xfree(t as *mut ::core::ffi::c_void);
        }
    } else {
        retval = concat_fnames(dname, tail, true_0 != 0);
    }
    return retval;
}
unsafe extern "C" fn attention_message(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fhname: *mut ::core::ffi::c_char,
    mut msg_0: *mut StringBuilder,
) {
    '_c2rust_label: {
        if !(*buf).b_fname.is_null() {
        } else {
            __assert_fail(
                b"buf->b_fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3379 as ::core::ffi::c_uint,
                b"void attention_message(buf_T *, char *, char *, StringBuilder *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    emsg(gettext(
        b"E325: ATTENTION\0".as_ptr() as *const ::core::ffi::c_char
    ));
    kv_do_printf(
        msg_0,
        gettext(b"Found a swap file by the name \"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    kv_do_printf(
        msg_0,
        b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
        fhname,
    );
    let swap_mtime: time_t = swapfile_info(fname, msg_0);
    kv_do_printf(
        msg_0,
        gettext(b"While opening file \"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    kv_do_printf(
        msg_0,
        b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*buf).b_fname,
    );
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
    if !os_fileinfo((*buf).b_fname, &raw mut file_info) {
        kv_do_printf(
            msg_0,
            gettext(b"      CANNOT BE FOUND\0".as_ptr() as *const ::core::ffi::c_char),
        );
    } else {
        kv_do_printf(
            msg_0,
            gettext(b"             dated: \0".as_ptr() as *const ::core::ffi::c_char),
        );
        let mut x: time_t = file_info.stat.st_mtim.tv_sec as time_t;
        let mut ctime_buf: [::core::ffi::c_char; 50] = [0; 50];
        kv_do_printf(
            msg_0,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            os_ctime_r(x, &mut ctime_buf, true),
        );
        if swap_mtime != 0 as time_t && x > swap_mtime {
            kv_do_printf(
                msg_0,
                gettext(b"      NEWER than swap file!\n\0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
    }
    kv_do_printf(
        msg_0,
        gettext(
            b"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n\0"
                .as_ptr() as *const ::core::ffi::c_char,
        ),
    );
    kv_do_printf(
        msg_0,
        gettext(b"(2) An edit session for this file crashed.\n\0".as_ptr()
            as *const ::core::ffi::c_char),
    );
    kv_do_printf(
        msg_0,
        gettext(
            b"    If this is the case, use \":recover\" or \"nvim -r \0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
    );
    kv_do_printf(
        msg_0,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        (*buf).b_fname,
    );
    kv_do_printf(
        msg_0,
        gettext(
            b"\"\n    to recover the changes (see \":help recovery\").\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
    );
    kv_do_printf(
        msg_0,
        gettext(
            b"    If you did this already, delete the swap file \"\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
    );
    kv_do_printf(msg_0, b"%s\0".as_ptr() as *const ::core::ffi::c_char, fname);
    kv_do_printf(
        msg_0,
        gettext(b"\"\n    to avoid this message.\n\0".as_ptr() as *const ::core::ffi::c_char),
    );
}
unsafe extern "C" fn do_swapexists(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
) -> sea_choice_T {
    set_vim_var_string(VV_SWAPNAME, fname, -1 as ptrdiff_t);
    set_vim_var_string(
        VV_SWAPCHOICE,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    (*allbuf_lock.ptr()) += 1;
    apply_autocmds(
        EVENT_SWAPEXISTS,
        (*buf).b_fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    (*allbuf_lock.ptr()) -= 1;
    set_vim_var_string(
        VV_SWAPNAME,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    match *get_vim_var_str(VV_SWAPCHOICE) as ::core::ffi::c_int {
        111 => return SEA_CHOICE_READONLY,
        101 => return SEA_CHOICE_EDIT,
        114 => return SEA_CHOICE_RECOVER,
        100 => return SEA_CHOICE_DELETE,
        113 => return SEA_CHOICE_QUIT,
        97 => return SEA_CHOICE_ABORT,
        _ => {}
    }
    return SEA_CHOICE_NONE;
}
unsafe extern "C" fn findswapname(
    mut buf: *mut buf_T,
    mut dirp: *mut *mut ::core::ffi::c_char,
    mut old_fname: *mut ::core::ffi::c_char,
    mut found_existing_dir: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut buf_fname: *mut ::core::ffi::c_char = (*buf).b_fname;
    let dir_len: size_t = strlen(*dirp).wrapping_add(1 as size_t);
    let mut dir_name: *mut ::core::ffi::c_char = xmalloc(dir_len) as *mut ::core::ffi::c_char;
    copy_option_part(
        dirp,
        dir_name,
        dir_len,
        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    let mut fname: *mut ::core::ffi::c_char =
        makeswapname(buf_fname, (*buf).b_ffname, buf, dir_name);
    loop {
        let mut n: size_t = 0;
        if fname.is_null() {
            break;
        }
        n = strlen(fname);
        if n == 0 as size_t {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut fname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            break;
        } else {
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
            let mut file_or_link_found: bool = os_fileinfo_link(fname, &raw mut file_info);
            if !file_or_link_found {
                break;
            }
            if !old_fname.is_null() && path_fnamecmp(fname, old_fname) == 0 as ::core::ffi::c_int {
                break;
            }
            if *fname.offset(n.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                == 'w' as ::core::ffi::c_int
                && *fname.offset(n.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int
            {
                if !recoverymode.get()
                    && !buf_fname.is_null()
                    && !(*buf).b_help
                    && (*buf).b_flags & BF_DUMMY == 0
                {
                    let mut fd: ::core::ffi::c_int = 0;
                    let mut b0: ZeroBlock = ZeroBlock {
                        b0_id: [0; 2],
                        b0_version: [0; 10],
                        b0_page_size: [0; 4],
                        b0_mtime: [0; 4],
                        b0_ino: [0; 4],
                        b0_pid: [0; 4],
                        b0_uname: [0; 40],
                        b0_hname: [0; 40],
                        b0_fname: [0; 900],
                        b0_magic_long: 0,
                        b0_magic_int: 0,
                        b0_magic_short: 0,
                        b0_magic_char: 0,
                    };
                    let mut differ: bool = false_0 != 0;
                    fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
                    if fd >= 0 as ::core::ffi::c_int {
                        if read_eintr(
                            fd,
                            &raw mut b0 as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<ZeroBlock>(),
                        ) as usize
                            == ::core::mem::size_of::<ZeroBlock>()
                        {
                            proc_running.set(swapfile_proc_running(&raw mut b0, fname));
                            if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                                - 2 as ::core::ffi::c_int)
                                as usize] as ::core::ffi::c_int
                                & B0_SAME_DIR
                                != 0
                            {
                                if path_fnamecmp(
                                    path_tail((*buf).b_ffname),
                                    path_tail(&raw mut b0.b0_fname as *mut ::core::ffi::c_char),
                                ) != 0 as ::core::ffi::c_int
                                    || !same_directory(fname, (*buf).b_ffname)
                                {
                                    expand_env(
                                        &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        MAXPATHL,
                                    );
                                    if fnamecmp_ino(
                                        (*buf).b_ffname,
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        char_to_long(
                                            &raw mut b0.b0_ino as *mut ::core::ffi::c_char,
                                        ),
                                    ) {
                                        differ = true_0 != 0;
                                    }
                                }
                            } else {
                                expand_env(
                                    &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                                    MAXPATHL,
                                );
                                if fnamecmp_ino(
                                    (*buf).b_ffname,
                                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                                    char_to_long(&raw mut b0.b0_ino as *mut ::core::ffi::c_char),
                                ) {
                                    differ = true_0 != 0;
                                }
                            }
                        }
                        close(fd);
                    }
                    if !differ
                        && (*curbuf.get()).b_flags & BF_RECOVERED == 0
                        && vim_strchr(p_shm.get(), SHM_ATTENTION as ::core::ffi::c_int).is_null()
                    {
                        let mut choice: sea_choice_T = SEA_CHOICE_NONE;
                        if os_path_exists((*buf).b_fname) as ::core::ffi::c_int != 0
                            && swapfile_unchanged(fname) as ::core::ffi::c_int != 0
                        {
                            choice = SEA_CHOICE_DELETE;
                            if p_verbose.get() > 0 as OptInt {
                                verb_msg(gettext(
                                    b"Found a swap file that is not useful, deleting it\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                        }
                        if choice as ::core::ffi::c_uint
                            == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            && swap_exists_action.get() != SEA_NONE
                            && has_autocmd(EVENT_SWAPEXISTS, buf_fname, buf) as ::core::ffi::c_int
                                != 0
                        {
                            choice = do_swapexists(buf, fname);
                        }
                        if choice as ::core::ffi::c_uint
                            == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            && swap_exists_action.get() == SEA_READONLY
                        {
                            choice = SEA_CHOICE_READONLY;
                        }
                        proc_running.set(0 as ::core::ffi::c_int);
                        if choice as ::core::ffi::c_uint
                            == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            (*no_wait_return.ptr()) += 1;
                            let mut msg_0: StringBuilder = KV_INITIAL_VALUE;
                            msg_0.capacity =
                                (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
                            msg_0.items = xrealloc(
                                msg_0.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<::core::ffi::c_char>()
                                    .wrapping_mul(msg_0.capacity),
                            ) as *mut ::core::ffi::c_char;
                            let mut fhname: *mut ::core::ffi::c_char =
                                home_replace_save(::core::ptr::null_mut::<buf_T>(), fname);
                            attention_message(buf, fname, fhname, &raw mut msg_0);
                            got_int.set(false_0 != 0);
                            flush_buffers(FLUSH_TYPEAHEAD);
                            if swap_exists_action.get() != SEA_NONE {
                                kv_do_printf(
                                    &raw mut msg_0,
                                    gettext(
                                        b"Swap file \"\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                                kv_do_printf(
                                    &raw mut msg_0,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    fhname,
                                );
                                kv_do_printf(
                                    &raw mut msg_0,
                                    gettext(b"\" already exists!\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                );
                                let mut run_but: *mut ::core::ffi::c_char = gettext(
                                    b"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                let mut but: *mut ::core::ffi::c_char = gettext(
                                    b"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                                choice = do_dialog(
                                    VIM_WARNING as ::core::ffi::c_int,
                                    gettext(
                                        b"VIM - ATTENTION\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                    msg_0.items,
                                    if proc_running.get() != 0 {
                                        run_but
                                    } else {
                                        but
                                    },
                                    1 as ::core::ffi::c_int,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    false_0,
                                ) as sea_choice_T;
                                choice = (choice as ::core::ffi::c_uint).wrapping_add(
                                    (proc_running.get() != 0
                                        && choice as ::core::ffi::c_uint
                                            >= 4 as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                        as ::core::ffi::c_uint,
                                ) as sea_choice_T;
                                msg_reset_scroll();
                            } else {
                                let mut need_clear: bool = false_0 != 0;
                                msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
                                msg_multiline(
                                    String_0 {
                                        data: msg_0.items,
                                        size: msg_0.size,
                                    },
                                    0 as ::core::ffi::c_int,
                                    false_0 != 0,
                                    false_0 != 0,
                                    &raw mut need_clear,
                                );
                            }
                            (*no_wait_return.ptr()) -= 1;
                            xfree(msg_0.items as *mut ::core::ffi::c_void);
                            msg_0.capacity = 0 as size_t;
                            msg_0.size = msg_0.capacity;
                            msg_0.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            xfree(fhname as *mut ::core::ffi::c_void);
                        }
                        match choice as ::core::ffi::c_uint {
                            1 => {
                                (*buf).b_p_ro = true_0;
                            }
                            3 => {
                                swap_exists_action.set(SEA_RECOVER);
                            }
                            4 => {
                                os_remove(fname);
                            }
                            5 => {
                                swap_exists_action.set(SEA_QUIT);
                            }
                            6 => {
                                swap_exists_action.set(SEA_QUIT);
                                got_int.set(true_0 != 0);
                            }
                            0 => {
                                msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                                if msg_silent.get() == 0 as ::core::ffi::c_int {
                                    need_wait_return.set(true_0 != 0);
                                }
                            }
                            2 | _ => {}
                        }
                        if choice as ::core::ffi::c_uint
                            != SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            && !os_path_exists(fname)
                        {
                            break;
                        }
                    }
                }
            }
            if *fname.offset(n.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == 'a' as ::core::ffi::c_int
            {
                if *fname.offset(n.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    == 'a' as ::core::ffi::c_int
                {
                    emsg(gettext(
                        b"E326: Too many swap files found\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                        &raw mut fname as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL_0;
                    let _ = *ptr__0;
                    break;
                } else {
                    *fname.offset(n.wrapping_sub(2 as size_t) as isize) -= 1;
                    *fname.offset(n.wrapping_sub(1 as size_t) as isize) =
                        ('z' as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                }
            }
            *fname.offset(n.wrapping_sub(1 as size_t) as isize) -= 1;
        }
    }
    if os_isdir(dir_name) {
        *found_existing_dir = true_0 != 0;
    } else if !*found_existing_dir && **dirp as ::core::ffi::c_int == NUL {
        let mut ret: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        ret = os_mkdir_recurse(
            dir_name,
            0o755 as int32_t,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if ret != 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"E303: Unable to create directory \"%s\" for swap file, recovery impossible: %s\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                failed_dir,
                uv_strerror(ret),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
    }
    xfree(dir_name as *mut ::core::ffi::c_void);
    return fname;
}
unsafe extern "C" fn b0_magic_wrong(mut b0p: *mut ZeroBlock) -> ::core::ffi::c_int {
    return ((*b0p).b0_magic_long != B0_MAGIC_LONG as ::core::ffi::c_int as ::core::ffi::c_long
        || (*b0p).b0_magic_int != B0_MAGIC_INT as ::core::ffi::c_int
        || (*b0p).b0_magic_short as ::core::ffi::c_int
            != B0_MAGIC_SHORT as ::core::ffi::c_int as int16_t as ::core::ffi::c_int
        || (*b0p).b0_magic_char as ::core::ffi::c_int != B0_MAGIC_CHAR as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn fnamecmp_ino(
    mut fname_c: *mut ::core::ffi::c_char,
    mut fname_s: *mut ::core::ffi::c_char,
    mut ino_block0: ::core::ffi::c_long,
) -> bool {
    let mut ino_c: uint64_t = 0 as uint64_t;
    let mut ino_s: uint64_t = 0;
    let mut buf_c: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut buf_s: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut retval_c: ::core::ffi::c_int = 0;
    let mut retval_s: ::core::ffi::c_int = 0;
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
    if os_fileinfo(fname_c, &raw mut file_info) {
        ino_c = os_fileinfo_inode(&raw mut file_info);
    }
    if os_fileinfo(fname_s, &raw mut file_info) {
        ino_s = os_fileinfo_inode(&raw mut file_info);
    } else {
        ino_s = ino_block0 as uint64_t;
    }
    if ino_c != 0 && ino_s != 0 {
        return ino_c != ino_s;
    }
    retval_c = vim_FullName(
        fname_c,
        &raw mut buf_c as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        true_0 != 0,
    );
    retval_s = vim_FullName(
        fname_s,
        &raw mut buf_s as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        true_0 != 0,
    );
    if retval_c == OK && retval_s == OK {
        return strcmp(
            &raw mut buf_c as *mut ::core::ffi::c_char,
            &raw mut buf_s as *mut ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int;
    }
    if ino_s == 0 as uint64_t && ino_c == 0 as uint64_t && retval_c == FAIL && retval_s == FAIL {
        return strcmp(fname_c, fname_s) != 0 as ::core::ffi::c_int;
    }
    return true_0 != 0;
}
unsafe extern "C" fn long_to_char(mut n: ::core::ffi::c_long, mut s_in: *mut ::core::ffi::c_char) {
    let mut s: *mut uint8_t = s_in as *mut uint8_t;
    *s.offset(0 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
    n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
    *s.offset(1 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
    n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
    *s.offset(2 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
    n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
    *s.offset(3 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
}
unsafe extern "C" fn char_to_long(mut s_in: *const ::core::ffi::c_char) -> ::core::ffi::c_long {
    let mut s: *const uint8_t = s_in as *mut uint8_t;
    let mut retval: ::core::ffi::c_long =
        *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
    retval <<= 8 as ::core::ffi::c_int;
    retval |= *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
    retval <<= 8 as ::core::ffi::c_int;
    retval |= *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
    retval <<= 8 as ::core::ffi::c_int;
    retval |= *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
    return retval;
}
pub unsafe extern "C" fn ml_setflags(mut buf: *mut buf_T) {
    if (*buf).b_ml.ml_mfp.is_null() {
        return;
    }
    let mut hp: *mut bhdr_T = mf_find((*buf).b_ml.ml_mfp, 0 as blocknr_T);
    if !hp.is_null() {
        let mut b0p: *mut ZeroBlock = (*hp).bh_data as *mut ZeroBlock;
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
            (if (*buf).b_changed != 0 {
                B0_DIRTY
            } else {
                0 as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
        (*b0p).b0_fname
            [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] = ((*b0p)
            .b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            & !B0_FF_MASK
            | (get_fileformat(buf) + 1 as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        add_b0_fenc(b0p, buf);
        (*hp).bh_flags |= BH_DIRTY;
        mf_sync((*buf).b_ml.ml_mfp, MFS_ZERO as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn ml_updatechunk(
    mut buf: *mut buf_T,
    mut line: linenr_T,
    mut len: ::core::ffi::c_int,
    mut updtype: ::core::ffi::c_int,
) {
    static ml_upd_lastbuf: GlobalCell<*mut buf_T> =
        GlobalCell::new(::core::ptr::null_mut::<buf_T>());
    static ml_upd_lastline: GlobalCell<linenr_T> = GlobalCell::new(0);
    static ml_upd_lastcurline: GlobalCell<linenr_T> = GlobalCell::new(0);
    static ml_upd_lastcurix: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    let mut curline: linenr_T = ml_upd_lastcurline.get();
    let mut curix: ::core::ffi::c_int = ml_upd_lastcurix.get();
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    if (*buf).b_ml.ml_usedchunks == -1 as ::core::ffi::c_int || len == 0 as ::core::ffi::c_int {
        return;
    }
    if (*buf).b_ml.ml_chunksize.is_null() {
        (*buf).b_ml.ml_chunksize =
            xmalloc(::core::mem::size_of::<chunksize_T>().wrapping_mul(100 as size_t))
                as *mut chunksize_T;
        (*buf).b_ml.ml_numchunks = 100 as ::core::ffi::c_int;
        (*buf).b_ml.ml_usedchunks = 1 as ::core::ffi::c_int;
        (*(*buf)
            .b_ml
            .ml_chunksize
            .offset(0 as ::core::ffi::c_int as isize))
        .mlcs_numlines = 1 as ::core::ffi::c_int;
        (*(*buf)
            .b_ml
            .ml_chunksize
            .offset(0 as ::core::ffi::c_int as isize))
        .mlcs_totalsize = 1 as ::core::ffi::c_int;
    }
    if updtype == ML_CHNK_UPDLINE && (*buf).b_ml.ml_line_count == 1 as linenr_T {
        (*buf).b_ml.ml_usedchunks = 1 as ::core::ffi::c_int;
        (*(*buf)
            .b_ml
            .ml_chunksize
            .offset(0 as ::core::ffi::c_int as isize))
        .mlcs_numlines = 1 as ::core::ffi::c_int;
        (*(*buf)
            .b_ml
            .ml_chunksize
            .offset(0 as ::core::ffi::c_int as isize))
        .mlcs_totalsize = (*buf).b_ml.ml_line_textlen as ::core::ffi::c_int;
        return;
    }
    if buf != ml_upd_lastbuf.get()
        || line != ml_upd_lastline.get() + 1 as linenr_T
        || updtype != ML_CHNK_ADDLINE
    {
        curline = 1 as ::core::ffi::c_int as linenr_T;
        curix = 0 as ::core::ffi::c_int;
        while curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
            && line
                >= curline
                    + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines as linenr_T
        {
            curline = (curline as ::core::ffi::c_int
                + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
                as linenr_T;
            curix += 1;
        }
    } else if curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
        && line
            >= curline
                + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines as linenr_T
    {
        curline = (curline as ::core::ffi::c_int
            + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
            as linenr_T;
        curix += 1;
    }
    let mut curchnk: *mut chunksize_T = (*buf).b_ml.ml_chunksize.offset(curix as isize);
    if updtype == ML_CHNK_DELLINE {
        len = -len;
    }
    (*curchnk).mlcs_totalsize += len;
    if updtype == ML_CHNK_ADDLINE {
        let mut rest: ::core::ffi::c_int = 0;
        let mut dp: *mut DataBlock = ::core::ptr::null_mut::<DataBlock>();
        (*curchnk).mlcs_numlines += 1;
        if (*buf).b_ml.ml_usedchunks + 1 as ::core::ffi::c_int >= (*buf).b_ml.ml_numchunks {
            (*buf).b_ml.ml_numchunks =
                (*buf).b_ml.ml_numchunks * 3 as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
            (*buf).b_ml.ml_chunksize = xrealloc(
                (*buf).b_ml.ml_chunksize as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<chunksize_T>()
                    .wrapping_mul((*buf).b_ml.ml_numchunks as size_t),
            ) as *mut chunksize_T;
        }
        if (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
            >= MLCS_MAXL as ::core::ffi::c_int
        {
            let mut end_idx: ::core::ffi::c_int = 0;
            let mut text_end: ::core::ffi::c_int = 0;
            memmove(
                (*buf)
                    .b_ml
                    .ml_chunksize
                    .offset(curix as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (*buf).b_ml.ml_chunksize.offset(curix as isize) as *const ::core::ffi::c_void,
                (((*buf).b_ml.ml_usedchunks - curix) as size_t)
                    .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
            );
            let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut linecnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while curline < (*buf).b_ml.ml_line_count && linecnt < MLCS_MINL as ::core::ffi::c_int {
                hp = ml_find_line(buf, curline, ML_FIND as ::core::ffi::c_int);
                if hp.is_null() {
                    (*buf).b_ml.ml_usedchunks = -1 as ::core::ffi::c_int;
                    return;
                }
                dp = (*hp).bh_data as *mut DataBlock;
                let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high
                    as ::core::ffi::c_int
                    - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                let mut idx: ::core::ffi::c_int =
                    curline as ::core::ffi::c_int - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                curline = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
                rest = count - idx;
                if linecnt + rest > MLCS_MINL as ::core::ffi::c_int {
                    end_idx =
                        idx + MLCS_MINL as ::core::ffi::c_int - linecnt - 1 as ::core::ffi::c_int;
                    linecnt = MLCS_MINL as ::core::ffi::c_int;
                } else {
                    end_idx = count - 1 as ::core::ffi::c_int;
                    linecnt += rest;
                }
                if idx == 0 as ::core::ffi::c_int {
                    text_end = (*dp).db_txt_end as ::core::ffi::c_int;
                } else {
                    text_end = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset((idx - 1 as ::core::ffi::c_int) as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int;
                }
                size += text_end
                    - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset(end_idx as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int;
            }
            (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines = linecnt;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset((curix + 1 as ::core::ffi::c_int) as isize))
            .mlcs_numlines -= linecnt;
            (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize = size;
            (*(*buf)
                .b_ml
                .ml_chunksize
                .offset((curix + 1 as ::core::ffi::c_int) as isize))
            .mlcs_totalsize -= size;
            (*buf).b_ml.ml_usedchunks += 1;
            ml_upd_lastbuf.set(::core::ptr::null_mut::<buf_T>());
            return;
        } else if (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines
            >= MLCS_MINL as ::core::ffi::c_int
            && curix == (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
            && (*buf).b_ml.ml_line_count - line <= 1 as linenr_T
        {
            curchnk = (*buf)
                .b_ml
                .ml_chunksize
                .offset(curix as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            (*buf).b_ml.ml_usedchunks += 1;
            if line == (*buf).b_ml.ml_line_count {
                (*curchnk).mlcs_numlines = 0 as ::core::ffi::c_int;
                (*curchnk).mlcs_totalsize = 0 as ::core::ffi::c_int;
            } else {
                hp = ml_find_line(
                    buf,
                    (*buf).b_ml.ml_line_count,
                    ML_FIND as ::core::ffi::c_int,
                );
                if hp.is_null() {
                    (*buf).b_ml.ml_usedchunks = -1 as ::core::ffi::c_int;
                    return;
                }
                dp = (*hp).bh_data as *mut DataBlock;
                if (*dp).db_line_count == 1 as ::core::ffi::c_long {
                    rest = (*dp).db_txt_end.wrapping_sub((*dp).db_txt_start) as ::core::ffi::c_int;
                } else {
                    rest = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                        .offset(((*dp).db_line_count - 2 as ::core::ffi::c_long) as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int
                        - (*dp).db_txt_start as ::core::ffi::c_int;
                }
                (*curchnk).mlcs_totalsize = rest;
                (*curchnk).mlcs_numlines = 1 as ::core::ffi::c_int;
                (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_totalsize -= rest;
                (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines -=
                    1 as ::core::ffi::c_int;
            }
        }
    } else if updtype == ML_CHNK_DELLINE {
        (*curchnk).mlcs_numlines -= 1;
        ml_upd_lastbuf.set(::core::ptr::null_mut::<buf_T>());
        if curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
            && (*curchnk).mlcs_numlines
                + (*curchnk.offset(1 as ::core::ffi::c_int as isize)).mlcs_numlines
                <= MLCS_MINL as ::core::ffi::c_int
        {
            curix += 1;
            curchnk = (*buf).b_ml.ml_chunksize.offset(curix as isize);
        } else if curix == 0 as ::core::ffi::c_int
            && (*curchnk).mlcs_numlines <= 0 as ::core::ffi::c_int
        {
            (*buf).b_ml.ml_usedchunks -= 1;
            memmove(
                (*buf).b_ml.ml_chunksize as *mut ::core::ffi::c_void,
                (*buf)
                    .b_ml
                    .ml_chunksize
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                ((*buf).b_ml.ml_usedchunks as size_t)
                    .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
            );
            return;
        } else if curix == 0 as ::core::ffi::c_int
            || (*curchnk).mlcs_numlines > 10 as ::core::ffi::c_int
                && (*curchnk).mlcs_numlines
                    + (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines
                    > MLCS_MINL as ::core::ffi::c_int
        {
            return;
        }
        (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_numlines +=
            (*curchnk).mlcs_numlines;
        (*curchnk.offset(-1 as ::core::ffi::c_int as isize)).mlcs_totalsize +=
            (*curchnk).mlcs_totalsize;
        (*buf).b_ml.ml_usedchunks -= 1;
        if curix < (*buf).b_ml.ml_usedchunks {
            memmove(
                (*buf).b_ml.ml_chunksize.offset(curix as isize) as *mut ::core::ffi::c_void,
                (*buf)
                    .b_ml
                    .ml_chunksize
                    .offset(curix as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                (((*buf).b_ml.ml_usedchunks - curix) as size_t)
                    .wrapping_mul(::core::mem::size_of::<chunksize_T>()),
            );
        }
        return;
    }
    ml_upd_lastbuf.set(buf);
    ml_upd_lastline.set(line);
    ml_upd_lastcurline.set(curline);
    ml_upd_lastcurix.set(curix);
}
pub unsafe extern "C" fn ml_find_line_or_offset(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut offp: *mut ::core::ffi::c_int,
    mut no_ff: bool,
) -> ::core::ffi::c_int {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut text_end: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_int = 0;
    let mut ffdos: ::core::ffi::c_int =
        (!no_ff && get_fileformat(buf) == EOL_DOS) as ::core::ffi::c_int;
    let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut can_cache: bool =
        lnum != 0 as linenr_T && ffdos == 0 && (*buf).b_ml.ml_line_lnum == lnum;
    if lnum == 0 as linenr_T || (*buf).b_ml.ml_line_lnum < lnum || !no_ff {
        ml_flush_line(curbuf.get(), false_0 != 0);
    } else if can_cache as ::core::ffi::c_int != 0 && (*buf).b_ml.ml_line_offset > 0 as size_t {
        return (*buf).b_ml.ml_line_offset as ::core::ffi::c_int;
    }
    if (*buf).b_ml.ml_usedchunks == -1 as ::core::ffi::c_int
        || (*buf).b_ml.ml_chunksize.is_null()
        || lnum < 0 as linenr_T
    {
        if no_ff as ::core::ffi::c_int != 0
            && !(*buf).b_ml.ml_mfp.is_null()
            && (lnum == 1 as linenr_T || lnum == 2 as linenr_T)
        {
            return lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
        return -1 as ::core::ffi::c_int;
    }
    if offp.is_null() {
        offset = 0 as ::core::ffi::c_int;
    } else {
        offset = *offp;
    }
    if lnum == 0 as linenr_T && offset <= 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut curline: linenr_T = 1 as linenr_T;
    let mut curix: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while curix < (*buf).b_ml.ml_usedchunks - 1 as ::core::ffi::c_int
        && (lnum != 0 as linenr_T
            && lnum
                >= curline
                    + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines as linenr_T
            || offset != 0 as ::core::ffi::c_int
                && offset
                    > size
                        + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize
                        + ffdos * (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
    {
        curline = (curline as ::core::ffi::c_int
            + (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines)
            as linenr_T;
        size += (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_totalsize;
        if offset != 0 && ffdos != 0 {
            size += (*(*buf).b_ml.ml_chunksize.offset(curix as isize)).mlcs_numlines;
        }
        curix += 1;
    }
    while lnum != 0 as linenr_T && curline < lnum
        || offset != 0 as ::core::ffi::c_int && size < offset
    {
        if curline > (*buf).b_ml.ml_line_count || {
            hp = ml_find_line(buf, curline, ML_FIND as ::core::ffi::c_int);
            hp.is_null()
        } {
            return -1 as ::core::ffi::c_int;
        }
        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
        let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
            - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0;
        idx = (curline - (*buf).b_ml.ml_locked_low) as ::core::ffi::c_int;
        let mut start_idx: ::core::ffi::c_int = idx;
        if idx == 0 as ::core::ffi::c_int {
            text_end = (*dp).db_txt_end as ::core::ffi::c_int;
        } else {
            text_end = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                .offset((idx - 1 as ::core::ffi::c_int) as isize)
                & DB_INDEX_MASK) as ::core::ffi::c_int;
        }
        if lnum != 0 as linenr_T {
            if curline + (count as linenr_T - idx as linenr_T) >= lnum {
                idx += (lnum - curline - 1 as linenr_T) as ::core::ffi::c_int;
            } else {
                idx = count - 1 as ::core::ffi::c_int;
            }
        } else {
            extra = 0 as ::core::ffi::c_int;
            while offset
                >= size + text_end
                    - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                        & DB_INDEX_MASK) as ::core::ffi::c_int
                    + ffdos
            {
                if ffdos != 0 {
                    size += 1;
                }
                if idx == count - 1 as ::core::ffi::c_int {
                    extra = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    idx += 1;
                }
            }
        }
        let mut len: ::core::ffi::c_int = text_end
            - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                & DB_INDEX_MASK) as ::core::ffi::c_int;
        size += len;
        if offset != 0 as ::core::ffi::c_int && size >= offset {
            if size + ffdos == offset {
                *offp = 0 as ::core::ffi::c_int;
            } else if idx == start_idx {
                *offp = offset - size + len;
            } else {
                *offp = offset - size + len
                    - (text_end
                        - (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((idx - 1 as ::core::ffi::c_int) as isize)
                            & DB_INDEX_MASK) as ::core::ffi::c_int);
            }
            curline = (curline as ::core::ffi::c_int + (idx - start_idx + extra)) as linenr_T;
            if curline > (*buf).b_ml.ml_line_count {
                return -1 as ::core::ffi::c_int;
            }
            return curline as ::core::ffi::c_int;
        }
        curline = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
    }
    if lnum != 0 as linenr_T {
        if ffdos != 0 {
            size += (lnum - 1 as linenr_T) as ::core::ffi::c_int;
        }
        if ((*buf).b_p_fixeol == 0 || (*buf).b_p_bin != 0)
            && (*buf).b_p_eol == 0
            && lnum > (*buf).b_ml.ml_line_count
        {
            size -= ffdos + 1 as ::core::ffi::c_int;
        }
    }
    if can_cache as ::core::ffi::c_int != 0 && size > 0 as ::core::ffi::c_int {
        (*buf).b_ml.ml_line_offset = size as size_t;
    }
    return size;
}
pub unsafe extern "C" fn goto_byte(mut cnt: ::core::ffi::c_int) {
    let mut boff: ::core::ffi::c_int = cnt;
    ml_flush_line(curbuf.get(), false_0 != 0);
    setpcmark();
    if boff != 0 {
        boff -= 1;
    }
    let mut lnum: linenr_T =
        ml_find_line_or_offset(curbuf.get(), 0 as linenr_T, &raw mut boff, false_0 != 0)
            as linenr_T;
    if lnum < 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    } else {
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = boff;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_set_curswant = true_0;
    }
    check_cursor(curwin.get());
    mb_adjust_cursor();
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
