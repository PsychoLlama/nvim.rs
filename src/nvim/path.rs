use crate::src::nvim::charset::{backslash_halve, backslash_halve_save, rem_backslash, skipwhite};
use crate::src::nvim::cmdexpand::globpath;
use crate::src::nvim::eval_1::eval_to_string;
use crate::src::nvim::ex_docmd::eval_vars;
use crate::src::nvim::fileio::{file_pat_to_reg_pat, match_file_list};
use crate::src::nvim::garray::{
    ga_clear_strings, ga_concat_strings, ga_grow, ga_init, ga_remove_duplicate_strings,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    curbuf, emsg_off, emsg_silent, got_int, p_cdpath, p_fic, p_path, p_su, p_wig, NameBuff,
};
use crate::src::nvim::mbyte::{
    mb_isalpha, mb_strcmp_ic, mb_strnicmp, mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::env::{expand_env, expand_env_save_opt, os_getenv, vim_env_iter};
use crate::src::nvim::os::fs::{
    os_can_exe, os_closedir, os_dirname, os_file_is_readable, os_fileid, os_fileid_equal,
    os_fileinfo, os_fileinfo_id_equal, os_fileinfo_link, os_isdir, os_path_exists, os_realpath,
    os_scandir, os_scandir_next,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, memcpy, memmove, qsort, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp,
    strrchr,
};
use crate::src::nvim::os::shell::{get_cmd_output, os_expand_wildcards};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr};
pub use crate::src::nvim::types::{
    __compar_fn_t, __gid_t, __mode_t, __off_t, __pthread_internal_list, __pthread_list_t,
    __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, __uid_t, alist_T, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T,
    gid_t, handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mode_t, mtnode_inner_s, mtnode_s, off_t, partial_S, partial_T, pos_T, pos_save_T,
    proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T,
    size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T,
    synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uid_t, uint16_t, uint32_t, uint64_t,
    uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv__work, uv_async_cb,
    uv_async_s, uv_async_s_u as C2Rust_Unnamed_17, uv_async_t, uv_buf_t, uv_close_cb, uv_dirent_s,
    uv_dirent_t, uv_dirent_type_t, uv_file, uv_fs_cb, uv_fs_s, uv_fs_t, uv_fs_type, uv_gid_t,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_12, uv_handle_t, uv_handle_type, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_16, uv_loop_s_timer_heap as C2Rust_Unnamed_15,
    uv_loop_t, uv_mutex_t, uv_req_type, uv_rwlock_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed_13, uv_signal_s_u as C2Rust_Unnamed_14, uv_signal_t,
    uv_stat_t, uv_timespec_t, uv_uid_t, varnumber_T, virt_line, visualinfo_T, win_T, window_S,
    wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos, BoolVarValue,
    BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_4,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1, Directory, ExtmarkUndoObject,
    FileComparison, FileID, FileInfo, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal, Timestamp, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, QUEUE,
};
extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_DIRENT_BLOCK: uv_dirent_type_t = 7;
pub const UV_DIRENT_CHAR: uv_dirent_type_t = 6;
pub const UV_DIRENT_SOCKET: uv_dirent_type_t = 5;
pub const UV_DIRENT_FIFO: uv_dirent_type_t = 4;
pub const UV_DIRENT_LINK: uv_dirent_type_t = 3;
pub const UV_DIRENT_DIR: uv_dirent_type_t = 2;
pub const UV_DIRENT_FILE: uv_dirent_type_t = 1;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
pub const UV_FS_LUTIME: uv_fs_type = 36;
pub const UV_FS_MKSTEMP: uv_fs_type = 35;
pub const UV_FS_STATFS: uv_fs_type = 34;
pub const UV_FS_CLOSEDIR: uv_fs_type = 33;
pub const UV_FS_READDIR: uv_fs_type = 32;
pub const UV_FS_OPENDIR: uv_fs_type = 31;
pub const UV_FS_LCHOWN: uv_fs_type = 30;
pub const UV_FS_COPYFILE: uv_fs_type = 29;
pub const UV_FS_REALPATH: uv_fs_type = 28;
pub const UV_FS_FCHOWN: uv_fs_type = 27;
pub const UV_FS_CHOWN: uv_fs_type = 26;
pub const UV_FS_READLINK: uv_fs_type = 25;
pub const UV_FS_SYMLINK: uv_fs_type = 24;
pub const UV_FS_LINK: uv_fs_type = 23;
pub const UV_FS_SCANDIR: uv_fs_type = 22;
pub const UV_FS_RENAME: uv_fs_type = 21;
pub const UV_FS_MKDTEMP: uv_fs_type = 20;
pub const UV_FS_MKDIR: uv_fs_type = 19;
pub const UV_FS_RMDIR: uv_fs_type = 18;
pub const UV_FS_UNLINK: uv_fs_type = 17;
pub const UV_FS_FDATASYNC: uv_fs_type = 16;
pub const UV_FS_FSYNC: uv_fs_type = 15;
pub const UV_FS_FCHMOD: uv_fs_type = 14;
pub const UV_FS_CHMOD: uv_fs_type = 13;
pub const UV_FS_ACCESS: uv_fs_type = 12;
pub const UV_FS_FUTIME: uv_fs_type = 11;
pub const UV_FS_UTIME: uv_fs_type = 10;
pub const UV_FS_FTRUNCATE: uv_fs_type = 9;
pub const UV_FS_FSTAT: uv_fs_type = 8;
pub const UV_FS_LSTAT: uv_fs_type = 7;
pub const UV_FS_STAT: uv_fs_type = 6;
pub const UV_FS_SENDFILE: uv_fs_type = 5;
pub const UV_FS_WRITE: uv_fs_type = 4;
pub const UV_FS_READ: uv_fs_type = 3;
pub const UV_FS_CLOSE: uv_fs_type = 2;
pub const UV_FS_OPEN: uv_fs_type = 1;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const UV_FS_UNKNOWN: uv_fs_type = -1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_18 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_18 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_18 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_18 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_18 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_18 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_18 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_18 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_18 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_18 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_18 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_18 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_18 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_18 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_18 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_18 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_19 = 64;
pub const kShellOptWrite: C2Rust_Unnamed_19 = 32;
pub const kShellOptRead: C2Rust_Unnamed_19 = 16;
pub const kShellOptSilent: C2Rust_Unnamed_19 = 8;
pub const kShellOptDoOut: C2Rust_Unnamed_19 = 4;
pub const kShellOptExpand: C2Rust_Unnamed_19 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_20 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_20 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_20 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_20 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_20 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_20 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_20 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_20 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_20 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_20 = 512;
pub const EW_ICASE: C2Rust_Unnamed_20 = 256;
pub const EW_PATH: C2Rust_Unnamed_20 = 128;
pub const EW_EXEC: C2Rust_Unnamed_20 = 64;
pub const EW_SILENT: C2Rust_Unnamed_20 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_20 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_20 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_20 = 4;
pub const EW_FILE: C2Rust_Unnamed_20 = 2;
pub const EW_DIR: C2Rust_Unnamed_20 = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub const URL_BACKSLASH: C2Rust_Unnamed_21 = 2;
pub const URL_SLASH: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn path_full_compare(
    s1: *mut ::core::ffi::c_char,
    s2: *mut ::core::ffi::c_char,
    checkname: bool,
    expandenv: bool,
) -> FileComparison {
    let mut expanded1: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut full1: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut full2: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut file_id_1: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_2: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if expandenv {
        expand_env(s1, &raw mut expanded1 as *mut ::core::ffi::c_char, MAXPATHL);
    } else {
        xstrlcpy(
            &raw mut expanded1 as *mut ::core::ffi::c_char,
            s1,
            MAXPATHL as size_t,
        );
    }
    let mut id_ok_1: bool = os_fileid(
        &raw mut expanded1 as *mut ::core::ffi::c_char,
        &raw mut file_id_1,
    );
    let mut id_ok_2: bool = os_fileid(s2, &raw mut file_id_2);
    if !id_ok_1 && !id_ok_2 {
        if checkname {
            vim_FullName(
                &raw mut expanded1 as *mut ::core::ffi::c_char,
                &raw mut full1 as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
            vim_FullName(
                s2,
                &raw mut full2 as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
            if path_fnamecmp(
                &raw mut full1 as *mut ::core::ffi::c_char,
                &raw mut full2 as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                return kEqualFileNames;
            }
        }
        return kBothFilesMissing;
    }
    if !id_ok_1 || !id_ok_2 {
        return kOneFileMissing;
    }
    if os_fileid_equal(&raw mut file_id_1, &raw mut file_id_2) {
        return kEqualFiles;
    }
    return kDifferentFiles;
}
#[no_mangle]
pub unsafe extern "C" fn path_tail(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if fname.is_null() {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut tail: *const ::core::ffi::c_char = get_past_head(fname);
    let mut p: *const ::core::ffi::c_char = tail;
    while *p as ::core::ffi::c_int != NUL {
        if vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
            tail = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return tail as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn path_tail_with_sep(
    mut fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut past_head: *mut ::core::ffi::c_char = get_past_head(fname);
    let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
    while tail > past_head && after_pathsep(fname, tail) != 0 {
        tail = tail.offset(-1);
    }
    return tail;
}
#[no_mangle]
pub unsafe extern "C" fn invocation_path_tail(
    mut invocation: *const ::core::ffi::c_char,
    mut len: *mut size_t,
) -> *const ::core::ffi::c_char {
    let mut tail: *const ::core::ffi::c_char = get_past_head(invocation);
    let mut p: *const ::core::ffi::c_char = tail;
    while *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
        let mut was_sep: bool = vim_ispathsep_nocolon(*p as ::core::ffi::c_int);
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        if was_sep {
            tail = p;
        }
    }
    if !len.is_null() {
        *len = p.offset_from(tail) as size_t;
    }
    return tail;
}
#[no_mangle]
pub unsafe extern "C" fn path_next_component(
    mut fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *fname as ::core::ffi::c_int != NUL && !vim_ispathsep(*fname as ::core::ffi::c_int) {
        fname = fname.offset(utfc_ptr2len(fname as *mut ::core::ffi::c_char) as isize);
    }
    if *fname as ::core::ffi::c_int != NUL {
        fname = fname.offset(1);
    }
    return fname;
}
pub unsafe extern "C" fn path_head_length() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn is_path_head(mut path: *const ::core::ffi::c_char) -> bool {
    return vim_ispathsep(*path as ::core::ffi::c_int);
}
pub unsafe extern "C" fn get_past_head(
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut retval: *const ::core::ffi::c_char = path;
    while vim_ispathsep(*retval as ::core::ffi::c_int) {
        retval = retval.offset(1);
    }
    return retval as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn vim_ispathsep(mut c: ::core::ffi::c_int) -> bool {
    return c == '/' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn vim_ispathsep_nocolon(mut c: ::core::ffi::c_int) -> bool {
    return vim_ispathsep(c);
}
pub unsafe extern "C" fn vim_ispathlistsep(mut c: ::core::ffi::c_int) -> bool {
    return c == ':' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn shorten_dir_len(
    mut str: *mut ::core::ffi::c_char,
    mut trim_len: ::core::ffi::c_int,
) {
    let mut tail: *mut ::core::ffi::c_char = path_tail(str);
    let mut d: *mut ::core::ffi::c_char = str;
    let mut skip: bool = false_0 != 0;
    let mut dirchunk_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = str;
    loop {
        if s >= tail {
            let c2rust_fresh0 = d;
            d = d.offset(1);
            *c2rust_fresh0 = *s;
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
        } else if vim_ispathsep(*s as ::core::ffi::c_int) {
            let c2rust_fresh1 = d;
            d = d.offset(1);
            *c2rust_fresh1 = *s;
            skip = false_0 != 0;
            dirchunk_len = 0 as ::core::ffi::c_int;
        } else if !skip {
            let c2rust_fresh2 = d;
            d = d.offset(1);
            *c2rust_fresh2 = *s;
            if *s as ::core::ffi::c_int != '~' as ::core::ffi::c_int
                && *s as ::core::ffi::c_int != '.' as ::core::ffi::c_int
            {
                dirchunk_len += 1;
                if dirchunk_len >= trim_len {
                    skip = true_0 != 0;
                }
            }
            let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
            loop {
                l -= 1;
                if l <= 0 as ::core::ffi::c_int {
                    break;
                }
                s = s.offset(1);
                let c2rust_fresh3 = d;
                d = d.offset(1);
                *c2rust_fresh3 = *s;
            }
        }
        s = s.offset(1);
    }
}
pub unsafe extern "C" fn shorten_dir(mut str: *mut ::core::ffi::c_char) {
    shorten_dir_len(str, 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn dir_of_file_exists(mut fname: *mut ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
    if p == fname {
        return true_0 != 0;
    }
    let mut c: ::core::ffi::c_char = *p;
    *p = NUL as ::core::ffi::c_char;
    let mut retval: bool = os_isdir(fname);
    *p = c;
    return retval;
}
pub unsafe extern "C" fn path_fnamecmp(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return mb_strcmp_ic(p_fic.get() != 0, fname1, fname2);
}
pub unsafe extern "C" fn path_fnamencmp(
    fname1: *const ::core::ffi::c_char,
    fname2: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if p_fic.get() != 0 {
        return mb_strnicmp(fname1, fname2, len);
    }
    return strncmp(fname1, fname2, len);
}
#[inline]
unsafe extern "C" fn do_concat_fnames(
    mut fname1: *mut ::core::ffi::c_char,
    len1: size_t,
    mut fname2: *const ::core::ffi::c_char,
    len2: size_t,
    sep: bool,
) -> *mut ::core::ffi::c_char {
    if sep as ::core::ffi::c_int != 0
        && *fname1 as ::core::ffi::c_int != 0
        && after_pathsep(fname1, fname1.offset(len1 as isize)) == 0
    {
        *fname1.offset(len1 as isize) = PATHSEP as ::core::ffi::c_char;
        memmove(
            fname1
                .offset(len1 as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            fname2 as *const ::core::ffi::c_void,
            len2.wrapping_add(1 as size_t),
        );
    } else {
        memmove(
            fname1.offset(len1 as isize) as *mut ::core::ffi::c_void,
            fname2 as *const ::core::ffi::c_void,
            len2.wrapping_add(1 as size_t),
        );
    }
    return fname1;
}
pub unsafe extern "C" fn concat_fnames(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    let len1: size_t = strlen(fname1);
    let len2: size_t = strlen(fname2);
    let mut dest: *mut ::core::ffi::c_char =
        xmalloc(len1.wrapping_add(len2).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
    memmove(
        dest as *mut ::core::ffi::c_void,
        fname1 as *const ::core::ffi::c_void,
        len1.wrapping_add(1 as size_t),
    );
    return do_concat_fnames(dest, len1, fname2, len2, sep);
}
pub unsafe extern "C" fn concat_fnames_realloc(
    mut fname1: *mut ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    let len1: size_t = strlen(fname1);
    let len2: size_t = strlen(fname2);
    return do_concat_fnames(
        xrealloc(
            fname1 as *mut ::core::ffi::c_void,
            len1.wrapping_add(len2).wrapping_add(3 as size_t),
        ) as *mut ::core::ffi::c_char,
        len1,
        fname2,
        len2,
        sep,
    );
}
pub unsafe extern "C" fn add_pathsep(mut p: *mut ::core::ffi::c_char) -> bool {
    let len: size_t = strlen(p);
    if *p as ::core::ffi::c_int != NUL && after_pathsep(p, p.offset(len as isize)) == 0 {
        let pathsep_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
        if len > (MAXPATHL as size_t).wrapping_sub(pathsep_len) {
            return false_0 != 0;
        }
        memcpy(
            p.offset(len as isize) as *mut ::core::ffi::c_void,
            PATHSEPSTR.as_ptr() as *const ::core::ffi::c_void,
            pathsep_len,
        );
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn FullName_save(
    mut fname: *const ::core::ffi::c_char,
    mut force: bool,
) -> *mut ::core::ffi::c_char {
    if fname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if vim_FullName(fname, buf, MAXPATHL as size_t, force) == FAIL {
        xfree(buf as *mut ::core::ffi::c_void);
        return xstrdup(fname);
    }
    return buf;
}
pub unsafe extern "C" fn save_abs_path(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if !path_is_absolute(name) {
        return FullName_save(name, true_0 != 0);
    }
    return xstrdup(name);
}
pub unsafe extern "C" fn path_has_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    while *p != 0 {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else {
            let mut wildcards: *const ::core::ffi::c_char =
                b"*?[{`'$\0".as_ptr() as *const ::core::ffi::c_char;
            if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null()
                || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '~' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                return true_0 != 0;
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn pstrcmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return pathcmp(
        *(a as *mut *mut ::core::ffi::c_char),
        *(b as *mut *mut ::core::ffi::c_char),
        -1 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn path_has_exp_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    while *p as ::core::ffi::c_int != NUL {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else {
            let mut wildcards: *const ::core::ffi::c_char =
                b"*?[{\0".as_ptr() as *const ::core::ffi::c_char;
            if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null() {
                return true_0 != 0;
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> size_t {
    return do_path_expand(gap, path, 0 as size_t, flags, false_0 != 0);
}
unsafe extern "C" fn scandir_next_with_dots(mut dir: *mut Directory) -> *const ::core::ffi::c_char {
    static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if dir.is_null() {
        count.set(0 as ::core::ffi::c_int);
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    (*count.ptr()) += 1 as ::core::ffi::c_int;
    if count.get() == 1 as ::core::ffi::c_int || count.get() == 2 as ::core::ffi::c_int {
        return if count.get() == 1 as ::core::ffi::c_int {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"..\0".as_ptr() as *const ::core::ffi::c_char
        };
    }
    return os_scandir_next(dir);
}
unsafe extern "C" fn do_path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut wildoff: size_t,
    mut flags: ::core::ffi::c_int,
    mut didstar: bool,
) -> size_t {
    let mut start_len: ::core::ffi::c_int = (*gap).ga_len;
    let mut starstar: bool = false_0 != 0;
    static stardepth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if stardepth.get() > 0 as ::core::ffi::c_int && flags & EW_NOBREAK as ::core::ffi::c_int == 0 {
        os_breakcheck();
        if got_int.get() {
            return 0 as size_t;
        }
    }
    let buflen: size_t = strlen(path).wrapping_add(MAXPATHL as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = buf;
    let mut s: *mut ::core::ffi::c_char = buf;
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_end: *const ::core::ffi::c_char = path;
    while *path_end as ::core::ffi::c_int != NUL {
        if path_end >= path.offset(wildoff as isize)
            && rem_backslash(path_end) as ::core::ffi::c_int != 0
        {
            let c2rust_fresh5 = path_end;
            path_end = path_end.offset(1);
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = *c2rust_fresh5;
        } else if vim_ispathsep_nocolon(*path_end as ::core::ffi::c_int) {
            if !e.is_null() {
                break;
            }
            s = p.offset(1 as ::core::ffi::c_int as isize);
        } else if path_end >= path.offset(wildoff as isize)
            && (!vim_strchr(
                b"*?[{~$\0".as_ptr() as *const ::core::ffi::c_char,
                *path_end as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
                || p_fic.get() == 0
                    && flags & EW_ICASE as ::core::ffi::c_int != 0
                    && mb_isalpha(utf_ptr2char(path_end)) as ::core::ffi::c_int != 0)
        {
            e = p;
        }
        let mut charlen: ::core::ffi::c_int = utfc_ptr2len(path_end);
        memcpy(
            p as *mut ::core::ffi::c_void,
            path_end as *const ::core::ffi::c_void,
            charlen as size_t,
        );
        p = p.offset(charlen as isize);
        path_end = path_end.offset(charlen as isize);
    }
    e = p;
    *e = NUL as ::core::ffi::c_char;
    p = buf.offset(wildoff as isize);
    while p < s {
        if rem_backslash(p) {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            e = e.offset(-1);
            s = s.offset(-1);
        }
        p = p.offset(1);
    }
    p = s;
    while p < e {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
        {
            starstar = true_0 != 0;
        }
        p = p.offset(1);
    }
    let mut starts_with_dot: ::core::ffi::c_int =
        (*s as ::core::ffi::c_int == '.' as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
        s,
        e,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0,
    );
    if pat.is_null() {
        xfree(buf as *mut ::core::ffi::c_void);
        return 0 as size_t;
    }
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = flags & EW_ICASE as ::core::ffi::c_int != 0 || p_fic.get() != 0;
    if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
        (*emsg_silent.ptr()) += 1;
    }
    let mut nobreak: bool = flags & EW_NOBREAK as ::core::ffi::c_int != 0;
    regmatch.regprog = vim_regcomp(
        pat,
        RE_MAGIC
            | (if nobreak as ::core::ffi::c_int != 0 {
                RE_NOBREAK
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
        (*emsg_silent.ptr()) -= 1;
    }
    xfree(pat as *mut ::core::ffi::c_void);
    if regmatch.regprog.is_null()
        && flags & EW_NOTWILD as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        xfree(buf as *mut ::core::ffi::c_void);
        return 0 as size_t;
    }
    let mut len: size_t = s.offset_from(buf) as size_t;
    if !didstar
        && stardepth.get() < 100 as ::core::ffi::c_int
        && starstar as ::core::ffi::c_int != 0
        && e.offset_from(s) == 2 as isize
        && *path_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
    {
        vim_snprintf(
            s,
            buflen.wrapping_sub(len),
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            path_end.offset(1 as ::core::ffi::c_int as isize),
        );
        (*stardepth.ptr()) += 1;
        do_path_expand(gap, buf, len, flags, true_0 != 0);
        (*stardepth.ptr()) -= 1;
    }
    *s = NUL as ::core::ffi::c_char;
    let mut dir: Directory = Directory {
        request: uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
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
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        },
        ent: uv_dirent_t {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            type_0: UV_DIRENT_UNKNOWN,
        },
    };
    let mut dirpath: *mut ::core::ffi::c_char = (if *buf as ::core::ffi::c_int == NUL {
        b".\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        buf as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    if os_file_is_readable(dirpath) as ::core::ffi::c_int != 0
        && os_scandir(&raw mut dir, dirpath) as ::core::ffi::c_int != 0
    {
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        scandir_next_with_dots(::core::ptr::null_mut::<Directory>());
        while !got_int.get() && {
            name = scandir_next_with_dots(&raw mut dir);
            !name.is_null()
        } {
            len = s.offset_from(buf) as size_t;
            if !((*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '.' as ::core::ffi::c_int
                || starts_with_dot != 0
                || flags & EW_DODOT as ::core::ffi::c_int != 0
                    && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && (*name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '.' as ::core::ffi::c_int
                        || *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL))
                && (!regmatch.regprog.is_null()
                    && vim_regexec(&raw mut regmatch, name, 0 as colnr_T) as ::core::ffi::c_int
                        != 0
                    || flags & EW_NOTWILD as ::core::ffi::c_int != 0
                        && path_fnamencmp(
                            path.offset(len as isize),
                            name,
                            e.offset_from(s) as size_t,
                        ) == 0 as ::core::ffi::c_int))
            {
                continue;
            }
            len = len.wrapping_add(vim_snprintf(
                s,
                buflen.wrapping_sub(len),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            ) as size_t);
            if len.wrapping_add(1 as size_t) >= buflen {
                continue;
            }
            if starstar as ::core::ffi::c_int != 0 && stardepth.get() < 100 as ::core::ffi::c_int {
                vim_snprintf(
                    buf.offset(len as isize),
                    buflen.wrapping_sub(len),
                    b"/**%s\0".as_ptr() as *const ::core::ffi::c_char,
                    path_end,
                );
                (*stardepth.ptr()) += 1;
                do_path_expand(gap, buf, len.wrapping_add(1 as size_t), flags, true_0 != 0);
                (*stardepth.ptr()) -= 1;
            }
            vim_snprintf(
                buf.offset(len as isize),
                buflen.wrapping_sub(len),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                path_end,
            );
            if path_has_exp_wildcard(path_end) {
                if stardepth.get() < 100 as ::core::ffi::c_int {
                    (*stardepth.ptr()) += 1;
                    do_path_expand(gap, buf, len.wrapping_add(1 as size_t), flags, false_0 != 0);
                    (*stardepth.ptr()) -= 1;
                }
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
                if *path_end as ::core::ffi::c_int != NUL {
                    backslash_halve(
                        buf.offset(len as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                    );
                }
                if if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
                    os_fileinfo_link(buf, &raw mut file_info) as ::core::ffi::c_int
                } else {
                    os_path_exists(buf) as ::core::ffi::c_int
                } != 0
                {
                    addfile(gap, buf, flags);
                }
            }
        }
        os_closedir(&raw mut dir);
    }
    xfree(buf as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    let mut matches: size_t = ((*gap).ga_len - start_len) as size_t;
    if matches > 0 as size_t && !got_int.get() {
        qsort(
            ((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(start_len as isize)
                as *mut ::core::ffi::c_void,
            matches,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
            Some(
                pstrcmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    return matches;
}
unsafe extern "C" fn find_previous_pathsep(
    mut path: *mut ::core::ffi::c_char,
    mut psep: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *psep > path && vim_ispathsep(**psep as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        *psep = (*psep).offset(-1);
    }
    while *psep > path {
        if vim_ispathsep(**psep as ::core::ffi::c_int) {
            return OK;
        }
        *psep = (*psep).offset(
            -((utf_head_off(path, (*psep).offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return FAIL;
}
unsafe extern "C" fn is_unique(
    mut maybe_unique: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut i: ::core::ffi::c_int,
) -> bool {
    let mut candidate_len: size_t = strlen(maybe_unique);
    let mut other_paths: *mut *mut ::core::ffi::c_char =
        (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < (*gap).ga_len {
        if j != i {
            let mut other_path_len: size_t = strlen(*other_paths.offset(j as isize));
            if other_path_len >= candidate_len {
                let mut rival: *mut ::core::ffi::c_char = (*other_paths.offset(j as isize))
                    .offset(other_path_len as isize)
                    .offset(-(candidate_len as isize));
                if path_fnamecmp(maybe_unique, rival) == 0 as ::core::ffi::c_int
                    && (rival == *other_paths.offset(j as isize)
                        || vim_ispathsep(*rival.offset(-(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0)
                {
                    return false_0 != 0;
                }
            }
        }
        j += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn expand_path_option(
    mut curdir: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) {
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut curdirlen: size_t = 0 as size_t;
    while *path_option as ::core::ffi::c_int != NUL {
        let mut buflen: size_t = copy_option_part(
            &raw mut path_option,
            buf,
            MAXPATHL as size_t,
            b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if !vim_strchr(buf, '`' as ::core::ffi::c_int).is_null() {
            continue;
        }
        if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || vim_ispathsep(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            if (*curbuf.get()).b_ffname.is_null() {
                continue;
            }
            let mut p: *mut ::core::ffi::c_char = path_tail((*curbuf.get()).b_ffname);
            let mut plen: size_t = p.offset_from((*curbuf.get()).b_ffname) as size_t;
            if plen.wrapping_add(strlen(buf)) >= MAXPATHL as size_t {
                continue;
            }
            if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                *buf.offset(plen as isize) = NUL as ::core::ffi::c_char;
            } else {
                memmove(
                    buf.offset(plen as isize) as *mut ::core::ffi::c_void,
                    buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    buflen.wrapping_sub(2 as size_t).wrapping_add(1 as size_t),
                );
            }
            memmove(
                buf as *mut ::core::ffi::c_void,
                (*curbuf.get()).b_ffname as *const ::core::ffi::c_void,
                plen,
            );
            buflen = simplify_filename(buf);
        } else if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            strcpy(buf, curdir);
            if curdirlen == 0 as size_t {
                curdirlen = strlen(curdir);
            }
            buflen = curdirlen;
        } else {
            if path_with_url(buf) != 0 {
                continue;
            }
            if !path_is_absolute(buf) {
                if curdirlen == 0 as size_t {
                    curdirlen = strlen(curdir);
                }
                if curdirlen.wrapping_add(buflen).wrapping_add(3 as size_t) > MAXPATHL as size_t {
                    continue;
                }
                memmove(
                    buf.offset(curdirlen as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    buflen.wrapping_add(1 as size_t),
                );
                strcpy(buf, curdir);
                *buf.offset(curdirlen as isize) = PATHSEP as ::core::ffi::c_char;
                buflen = simplify_filename(buf);
            }
        }
        ga_grow(gap, 1 as ::core::ffi::c_int);
        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) =
            xmemdupz(buf as *const ::core::ffi::c_void, buflen) as *mut ::core::ffi::c_char;
        (*gap).ga_len += 1;
    }
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_path_cutoff(
    mut fname: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) -> *mut ::core::ffi::c_char {
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut path_part: *mut *mut ::core::ffi::c_char =
        (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut cutoff: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *fname.offset(j as isize) as ::core::ffi::c_int
            == *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
            && *fname.offset(j as isize) as ::core::ffi::c_int != NUL
            && *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL
        {
            j += 1;
        }
        if j > maxlen {
            maxlen = j;
            cutoff = fname.offset(j as isize);
        }
        i += 1;
    }
    if !cutoff.is_null() {
        while vim_ispathsep(*cutoff as ::core::ffi::c_int) {
            cutoff = cutoff.offset(utfc_ptr2len(cutoff) as isize);
        }
    }
    return cutoff;
}
unsafe extern "C" fn uniquefy_paths(
    mut gap: *mut garray_T,
    mut pattern: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
) {
    let mut fnames: *mut *mut ::core::ffi::c_char = (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut sort_again: bool = false_0 != 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut path_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut in_curdir: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut short_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ga_remove_duplicate_strings(gap);
    ga_init(
        &raw mut path_ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    let mut len: size_t = strlen(pattern);
    let mut file_pattern: *mut ::core::ffi::c_char =
        xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
    *file_pattern.offset(0 as ::core::ffi::c_int as isize) = '*' as ::core::ffi::c_char;
    *file_pattern.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    strcpy(
        file_pattern.offset(1 as ::core::ffi::c_int as isize),
        pattern,
    );
    let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
        file_pattern,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0,
    );
    xfree(file_pattern as *mut ::core::ffi::c_void);
    if pat.is_null() {
        return;
    }
    regmatch.rm_ic = true_0 != 0;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    xfree(pat as *mut ::core::ffi::c_void);
    if regmatch.regprog.is_null() {
        return;
    }
    let mut curdir: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(curdir, MAXPATHL as size_t);
    expand_path_option(curdir, path_option, &raw mut path_ga);
    in_curdir = xcalloc(
        (*gap).ga_len as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len && !got_int.get() {
        let mut path: *mut ::core::ffi::c_char = *fnames.offset(i as isize);
        let mut dir_end: *const ::core::ffi::c_char = gettail_dir(path);
        len = strlen(path);
        let mut is_in_curdir: bool =
            path_fnamencmp(curdir, path, dir_end.offset_from(path) as size_t)
                == 0 as ::core::ffi::c_int
                && *curdir.offset(dir_end.offset_from(path) as isize) as ::core::ffi::c_int == NUL;
        if is_in_curdir {
            *in_curdir.offset(i as isize) =
                xmemdupz(path as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        }
        let mut path_cutoff: *mut ::core::ffi::c_char = get_path_cutoff(path, &raw mut path_ga);
        if *pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            && *pattern.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
            && vim_ispathsep_nocolon(
                *pattern.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
            && !path_cutoff.is_null()
            && vim_regexec(&raw mut regmatch, path_cutoff, 0 as colnr_T) as ::core::ffi::c_int != 0
            && is_unique(path_cutoff, gap, i) as ::core::ffi::c_int != 0
        {
            sort_again = true_0 != 0;
            memmove(
                path as *mut ::core::ffi::c_void,
                path_cutoff as *const ::core::ffi::c_void,
                strlen(path_cutoff).wrapping_add(1 as size_t),
            );
        } else {
            let mut pathsep_p: *mut ::core::ffi::c_char = path
                .offset(len as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            while find_previous_pathsep(path, &raw mut pathsep_p) != 0 {
                if !(vim_regexec(
                    &raw mut regmatch,
                    pathsep_p.offset(1 as ::core::ffi::c_int as isize),
                    0 as colnr_T,
                ) as ::core::ffi::c_int
                    != 0
                    && is_unique(pathsep_p.offset(1 as ::core::ffi::c_int as isize), gap, i)
                        as ::core::ffi::c_int
                        != 0
                    && !path_cutoff.is_null()
                    && pathsep_p.offset(1 as ::core::ffi::c_int as isize) >= path_cutoff)
                {
                    continue;
                }
                sort_again = true_0 != 0;
                memmove(
                    path as *mut ::core::ffi::c_void,
                    pathsep_p.offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (path
                        .offset(len as isize)
                        .offset_from(pathsep_p.offset(1 as ::core::ffi::c_int as isize))
                        as size_t)
                        .wrapping_add(1 as size_t),
                );
                break;
            }
        }
        if path_is_absolute(path) {
            short_name = path_shorten_fname(path, curdir);
            if !short_name.is_null() && short_name > path.offset(1 as ::core::ffi::c_int as isize) {
                vim_snprintf(
                    path,
                    MAXPATHL as size_t,
                    b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    short_name,
                );
            }
        }
        os_breakcheck();
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*gap).ga_len && !got_int.get() {
        let mut path_0: *mut ::core::ffi::c_char = *in_curdir.offset(i_0 as isize);
        if !path_0.is_null() {
            short_name = path_shorten_fname(path_0, curdir);
            if short_name.is_null() {
                short_name = path_0;
            }
            if is_unique(short_name, gap, i_0) {
                strcpy(*fnames.offset(i_0 as isize), short_name);
            } else {
                let mut rel_pathsize: size_t = (1 as size_t)
                    .wrapping_add(
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    )
                    .wrapping_add(strlen(short_name))
                    .wrapping_add(1 as size_t);
                let mut rel_path: *mut ::core::ffi::c_char =
                    xmalloc(rel_pathsize) as *mut ::core::ffi::c_char;
                vim_snprintf(
                    rel_path,
                    rel_pathsize,
                    b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    short_name,
                );
                xfree(*fnames.offset(i_0 as isize) as *mut ::core::ffi::c_void);
                *fnames.offset(i_0 as isize) = rel_path;
                sort_again = true_0 != 0;
                os_breakcheck();
            }
        }
        i_0 += 1;
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < (*gap).ga_len {
        xfree(*in_curdir.offset(i_1 as isize) as *mut ::core::ffi::c_void);
        i_1 += 1;
    }
    xfree(in_curdir as *mut ::core::ffi::c_void);
    ga_clear_strings(&raw mut path_ga);
    vim_regfree(regmatch.regprog);
    if sort_again {
        ga_remove_duplicate_strings(gap);
    }
}
pub unsafe extern "C" fn gettail_dir(
    fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut dir_end: *const ::core::ffi::c_char = fname;
    let mut next_dir_end: *const ::core::ffi::c_char = fname;
    let mut look_for_sep: bool = true_0 != 0;
    let mut p: *const ::core::ffi::c_char = fname;
    while *p as ::core::ffi::c_int != NUL {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            if look_for_sep {
                next_dir_end = p;
                look_for_sep = false_0 != 0;
            }
        } else {
            if !look_for_sep {
                dir_end = next_dir_end;
            }
            look_for_sep = true_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return dir_end;
}
unsafe extern "C" fn expand_in_path(
    gap: *mut garray_T,
    pattern: *mut ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut path_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut path_option: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        };
    let curdir: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(curdir, MAXPATHL as size_t);
    ga_init(
        &raw mut path_ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if flags & EW_CDPATH as ::core::ffi::c_int != 0 {
        expand_path_option(curdir, p_cdpath.get(), &raw mut path_ga);
    } else {
        expand_path_option(curdir, path_option, &raw mut path_ga);
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    if path_ga.ga_len <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let paths: *mut ::core::ffi::c_char = ga_concat_strings(
        &raw mut path_ga,
        b",\0".as_ptr() as *const ::core::ffi::c_char,
    );
    ga_clear_strings(&raw mut path_ga);
    let mut glob_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & EW_ICASE as ::core::ffi::c_int != 0 {
        glob_flags |= WILD_ICASE as ::core::ffi::c_int;
    }
    if flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
        glob_flags |= WILD_ADD_SLASH as ::core::ffi::c_int;
    }
    globpath(
        paths,
        pattern,
        gap,
        glob_flags,
        flags & EW_CDPATH as ::core::ffi::c_int != 0,
    );
    xfree(paths as *mut ::core::ffi::c_void);
    return (*gap).ga_len;
}
unsafe extern "C" fn has_env_var(mut p: *mut ::core::ffi::c_char) -> bool {
    while *p != 0 {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else if !vim_strchr(
            b"$\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return true_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn has_special_wildchar(
    mut p: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    while *p != 0 {
        if *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            break;
        }
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\r' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\n' as ::core::ffi::c_int
        {
            p = p.offset(1);
        } else if !vim_strchr(
            SPECIAL_WILDCHAR.as_ptr(),
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                && flags & EW_NOTFOUND as ::core::ffi::c_int == 0)
            {
                if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    && vim_strchr(p, '}' as ::core::ffi::c_int).is_null())
                {
                    if !((*p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
                        && vim_strchr(p, *p as uint8_t as ::core::ffi::c_int).is_null())
                    {
                        return true_0 != 0;
                    }
                }
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn gen_expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut add_pat: ::core::ffi::c_int = 0;
    let mut did_expand_in_path: bool = false_0 != 0;
    let mut path_option: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        };
    if recursive.get() {
        return os_expand_wildcards(num_pat, pat, num_file, file, flags);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_pat {
        if has_special_wildchar(*pat.offset(i as isize), flags) as ::core::ffi::c_int != 0
            && !(vim_backtick(*pat.offset(i as isize)) as ::core::ffi::c_int != 0
                && *(*pat.offset(i as isize)).offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int)
        {
            return os_expand_wildcards(num_pat, pat, num_file, file, flags);
        }
        i += 1;
    }
    recursive.set(true_0 != 0);
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
    );
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < num_pat && !got_int.get() {
        add_pat = -1 as ::core::ffi::c_int;
        p = *pat.offset(i_0 as isize);
        if vim_backtick(p) {
            add_pat = expand_backtick(&raw mut ga, p, flags);
            if add_pat == -1 as ::core::ffi::c_int {
                recursive.set(false_0 != 0);
                ga_clear_strings(&raw mut ga);
                *num_file = 0 as ::core::ffi::c_int;
                *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                return FAIL;
            }
        } else {
            if has_env_var(p) as ::core::ffi::c_int != 0
                && flags & EW_NOTENV as ::core::ffi::c_int == 0
                || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
            {
                p = expand_env_save_opt(p, true_0 != 0);
                if p.is_null() {
                    p = *pat.offset(i_0 as isize);
                } else if has_env_var(p) as ::core::ffi::c_int != 0
                    || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                {
                    xfree(p as *mut ::core::ffi::c_void);
                    ga_clear_strings(&raw mut ga);
                    i_0 = os_expand_wildcards(
                        num_pat,
                        pat,
                        num_file,
                        file,
                        flags | EW_KEEPDOLLAR as ::core::ffi::c_int,
                    );
                    recursive.set(false_0 != 0);
                    return i_0;
                }
            }
            if path_has_exp_wildcard(p) as ::core::ffi::c_int != 0
                || flags & EW_ICASE as ::core::ffi::c_int != 0
            {
                if flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int) != 0
                    && !path_is_absolute(p)
                    && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && (vim_ispathsep(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                                && vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0))
                {
                    recursive.set(false_0 != 0);
                    add_pat = expand_in_path(&raw mut ga, p, flags);
                    recursive.set(true_0 != 0);
                    did_expand_in_path = true_0 != 0;
                } else {
                    recursive.set(false_0 != 0);
                    let mut tmp_add_pat: size_t = path_expand(&raw mut ga, p, flags);
                    recursive.set(true_0 != 0);
                    '_c2rust_label: {
                        if tmp_add_pat <= 2147483647 as ::core::ffi::c_int as size_t {
                        } else {
                            __assert_fail(
                                b"tmp_add_pat <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1375 as ::core::ffi::c_uint,
                                b"int gen_expand_wildcards(int, char **, int *, char ***, int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    add_pat = tmp_add_pat as ::core::ffi::c_int;
                }
            }
        }
        if add_pat == -1 as ::core::ffi::c_int
            || add_pat == 0 as ::core::ffi::c_int && flags & EW_NOTFOUND as ::core::ffi::c_int != 0
        {
            let mut t: *mut ::core::ffi::c_char = backslash_halve_save(p);
            if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
                addfile(
                    &raw mut ga,
                    t,
                    flags | EW_DIR as ::core::ffi::c_int | EW_FILE as ::core::ffi::c_int,
                );
            } else {
                addfile(&raw mut ga, t, flags);
            }
            if t != p {
                xfree(t as *mut ::core::ffi::c_void);
            }
        }
        if did_expand_in_path as ::core::ffi::c_int != 0
            && !(ga.ga_len <= 0 as ::core::ffi::c_int)
            && flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int) != 0
        {
            recursive.set(false_0 != 0);
            uniquefy_paths(&raw mut ga, p, path_option);
            recursive.set(true_0 != 0);
        }
        if p != *pat.offset(i_0 as isize) {
            xfree(p as *mut ::core::ffi::c_void);
        }
        i_0 += 1;
    }
    *num_file = ga.ga_len;
    *file = (if !ga.ga_data.is_null() {
        ga.ga_data
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    recursive.set(false_0 != 0);
    return if flags & EW_EMPTYOK as ::core::ffi::c_int != 0 || !ga.ga_data.is_null() {
        OK
    } else {
        FAIL
    };
}
pub unsafe extern "C" fn FreeWild(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    if count <= 0 as ::core::ffi::c_int || files.is_null() {
        return;
    }
    loop {
        let c2rust_fresh7 = count;
        count = count - 1;
        if c2rust_fresh7 == 0 {
            break;
        }
        xfree(*files.offset(count as isize) as *mut ::core::ffi::c_void);
    }
    xfree(files as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn vim_backtick(mut p: *mut ::core::ffi::c_char) -> bool {
    return *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *p
            .offset(strlen(p) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int;
}
unsafe extern "C" fn expand_backtick(
    mut gap: *mut garray_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmd: *mut ::core::ffi::c_char = xmemdupz(
        pat.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        strlen(pat).wrapping_sub(2 as size_t),
    ) as *mut ::core::ffi::c_char;
    if *cmd as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
        buffer = eval_to_string(
            cmd.offset(1 as ::core::ffi::c_int as isize),
            true_0 != 0,
            false_0 != 0,
        );
    } else {
        buffer = get_cmd_output(
            cmd,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            if flags & EW_SILENT as ::core::ffi::c_int != 0 {
                kShellOptSilent as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    xfree(cmd as *mut ::core::ffi::c_void);
    if buffer.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    cmd = buffer;
    while *cmd as ::core::ffi::c_int != NUL {
        cmd = skipwhite(cmd);
        p = cmd;
        while *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if p > cmd {
            let mut i: ::core::ffi::c_char = *p;
            *p = NUL as ::core::ffi::c_char;
            addfile(gap, cmd, flags);
            *p = i;
            cnt += 1;
        }
        cmd = p;
        while *cmd as ::core::ffi::c_int != NUL
            && (*cmd as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int)
        {
            cmd = cmd.offset(1);
        }
    }
    xfree(buffer as *mut ::core::ffi::c_void);
    return cnt;
}
pub unsafe extern "C" fn addfile(
    mut gap: *mut garray_T,
    mut f: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut isdir: bool = false;
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
    if flags & EW_NOTFOUND as ::core::ffi::c_int == 0
        && (if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
            !os_fileinfo_link(f, &raw mut file_info) as ::core::ffi::c_int
        } else {
            !os_path_exists(f) as ::core::ffi::c_int
        }) != 0
    {
        return;
    }
    isdir = os_isdir(f);
    if isdir as ::core::ffi::c_int != 0 && flags & EW_DIR as ::core::ffi::c_int == 0
        || !isdir && flags & EW_FILE as ::core::ffi::c_int == 0
    {
        return;
    }
    if !isdir
        && flags & EW_EXEC as ::core::ffi::c_int != 0
        && !os_can_exe(
            f,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            flags & EW_SHELLCMD as ::core::ffi::c_int == 0,
        )
    {
        return;
    }
    let mut p: *mut ::core::ffi::c_char = xmalloc(
        strlen(f)
            .wrapping_add(1 as size_t)
            .wrapping_add(isdir as size_t),
    ) as *mut ::core::ffi::c_char;
    strcpy(p, f);
    if isdir as ::core::ffi::c_int != 0 && flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
        add_pathsep(p);
    }
    ga_grow(gap, 1 as ::core::ffi::c_int);
    *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) = p;
    (*gap).ga_len += 1;
}
pub unsafe extern "C" fn simplify_filename(mut filename: *mut ::core::ffi::c_char) -> size_t {
    let mut components: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut stripping_disabled: bool = false_0 != 0;
    let mut relative: bool = true_0 != 0;
    let mut p: *mut ::core::ffi::c_char = filename;
    if vim_ispathsep(*p as ::core::ffi::c_int) {
        relative = false_0 != 0;
        loop {
            p = p.offset(1);
            if !vim_ispathsep(*p as ::core::ffi::c_int) {
                break;
            }
        }
    }
    let mut start: *mut ::core::ffi::c_char = p;
    let mut p_end: *mut ::core::ffi::c_char = p.offset(strlen(p) as isize);
    if start > filename.offset(2 as ::core::ffi::c_int as isize) {
        memmove(
            filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
        );
        p_end = p_end.offset(
            -(p.offset_from(filename.offset(1 as ::core::ffi::c_int as isize)) as size_t as isize),
        );
        p = filename.offset(1 as ::core::ffi::c_int as isize);
        start = p;
    }
    loop {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                (p_end.offset_from(p.offset(1 as ::core::ffi::c_int as isize)) as size_t)
                    .wrapping_add(1 as size_t),
            );
            p_end = p_end.offset(-1);
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            if p == start && relative as ::core::ffi::c_int != 0 {
                p = p.offset(
                    (1 as ::core::ffi::c_int
                        + (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL)
                            as ::core::ffi::c_int) as isize,
                );
            } else {
                let mut tail: *mut ::core::ffi::c_char = p.offset(1 as ::core::ffi::c_int as isize);
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    while vim_ispathsep(*tail as ::core::ffi::c_int) {
                        tail = tail.offset(utfc_ptr2len(tail) as isize);
                    }
                } else if p > start {
                    p = p.offset(-1);
                }
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail.offset_from(p) as size_t as isize));
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            let mut tail_0: *mut ::core::ffi::c_char = p.offset(2 as ::core::ffi::c_int as isize);
            while vim_ispathsep(*tail_0 as ::core::ffi::c_int) {
                tail_0 = tail_0.offset(utfc_ptr2len(tail_0) as isize);
            }
            if components > 0 as ::core::ffi::c_int {
                let mut do_strip: bool = false_0 != 0;
                if !stripping_disabled {
                    let mut saved_char: ::core::ffi::c_char =
                        *p.offset(-1 as ::core::ffi::c_int as isize);
                    *p.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
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
                    if !os_fileinfo_link(filename, &raw mut file_info) {
                        do_strip = true_0 != 0;
                    }
                    *p.offset(-1 as ::core::ffi::c_int as isize) = saved_char;
                    p = p.offset(-1);
                    while p > start && after_pathsep(start, p) == 0 {
                        p = p.offset(
                            -((utf_head_off(start, p.offset(-(1 as ::core::ffi::c_int as isize)))
                                + 1 as ::core::ffi::c_int) as isize),
                        );
                    }
                    if !do_strip {
                        saved_char = *tail_0;
                        *tail_0 = NUL as ::core::ffi::c_char;
                        if os_fileinfo(filename, &raw mut file_info) {
                            do_strip = true_0 != 0;
                        } else {
                            stripping_disabled = true_0 != 0;
                        }
                        *tail_0 = saved_char;
                        if do_strip {
                            let mut new_file_info: FileInfo = FileInfo {
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
                            if p == start && relative as ::core::ffi::c_int != 0 {
                                os_fileinfo(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char,
                                    &raw mut new_file_info,
                                );
                            } else {
                                saved_char = *p;
                                *p = NUL as ::core::ffi::c_char;
                                os_fileinfo(filename, &raw mut new_file_info);
                                *p = saved_char;
                            }
                            if !os_fileinfo_id_equal(&raw mut file_info, &raw mut new_file_info) {
                                do_strip = false_0 != 0;
                            }
                        }
                    }
                }
                if !do_strip {
                    p = tail_0;
                    components = 0 as ::core::ffi::c_int;
                } else {
                    if p == start
                        && relative as ::core::ffi::c_int != 0
                        && *tail_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                    {
                        let c2rust_fresh4 = p;
                        p = p.offset(1);
                        *c2rust_fresh4 = '.' as ::core::ffi::c_char;
                        *p = NUL as ::core::ffi::c_char;
                    } else {
                        if p > start
                            && *tail_0.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                        {
                            p = p.offset(-1);
                        }
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            tail_0 as *const ::core::ffi::c_void,
                            (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                        );
                        p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
                    }
                    components -= 1;
                }
            } else if p == start && !relative {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail_0 as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
            } else {
                if p == start.offset(2 as ::core::ffi::c_int as isize)
                    && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                {
                    memmove(
                        p.offset(-(2 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
                    );
                    p_end = p_end.offset(-(2 as ::core::ffi::c_int as isize));
                    tail_0 = tail_0.offset(-(2 as ::core::ffi::c_int as isize));
                }
                p = tail_0;
            }
        } else {
            components += 1;
            p = path_next_component(p) as *mut ::core::ffi::c_char;
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
    }
    return p_end.offset_from(filename) as size_t;
}
pub unsafe extern "C" fn path_has_drive_letter(
    mut p: *const ::core::ffi::c_char,
    mut path_len: size_t,
) -> bool {
    return path_len >= 2 as size_t
        && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int)
        && (path_len == 2 as size_t
            || (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0);
}
pub unsafe extern "C" fn path_is_url(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if strncmp(
        p,
        b":/\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return URL_SLASH as ::core::ffi::c_int;
    } else if strncmp(
        p,
        b":\\\\\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return URL_BACKSLASH as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_with_url(
    mut fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*fname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *fname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *fname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *fname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
    {
        return 0 as ::core::ffi::c_int;
    }
    if path_has_drive_letter(fname, strlen(fname)) {
        return 0 as ::core::ffi::c_int;
    }
    p = fname.offset(1 as ::core::ffi::c_int as isize);
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '+' as ::core::ffi::c_int
        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return path_is_url(p);
}
#[no_mangle]
pub unsafe extern "C" fn path_with_extension(
    mut path: *const ::core::ffi::c_char,
    mut extension: *const ::core::ffi::c_char,
) -> bool {
    let mut last_dot: *const ::core::ffi::c_char = strrchr(path, '.' as ::core::ffi::c_int);
    if last_dot.is_null() {
        return false_0 != 0;
    }
    return mb_strcmp_ic(
        p_fic.get() != 0,
        last_dot.offset(1 as ::core::ffi::c_int as isize),
        extension,
    ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn vim_isAbsName(mut name: *const ::core::ffi::c_char) -> bool {
    return path_with_url(name) != 0 as ::core::ffi::c_int
        || path_is_absolute(name) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_FullName(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: bool,
) -> ::core::ffi::c_int {
    *buf = NUL as ::core::ffi::c_char;
    if fname.is_null() {
        return FAIL;
    }
    if strlen(fname) > len.wrapping_sub(1 as size_t) {
        xstrlcpy(buf, fname, len);
        return FAIL;
    }
    if path_with_url(fname) != 0 {
        xstrlcpy(buf, fname, len);
        return OK;
    }
    let mut rv: ::core::ffi::c_int = path_to_absolute(fname, buf, len, force as ::core::ffi::c_int);
    if rv == FAIL {
        xstrlcpy(buf, fname, len);
    }
    return rv;
}
pub unsafe extern "C" fn fix_fname(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return FullName_save(fname, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn path_fix_case(mut name: *mut ::core::ffi::c_char) {
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
    if !os_fileinfo_link(name, &raw mut file_info) {
        return;
    }
    let mut slash: *mut ::core::ffi::c_char = strrchr(name, '/' as ::core::ffi::c_int);
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir: Directory = Directory {
        request: uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
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
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        },
        ent: uv_dirent_t {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            type_0: UV_DIRENT_UNKNOWN,
        },
    };
    let mut ok: bool = false;
    if slash.is_null() {
        ok = os_scandir(&raw mut dir, b".\0".as_ptr() as *const ::core::ffi::c_char);
        tail = name;
    } else {
        *slash = NUL as ::core::ffi::c_char;
        ok = os_scandir(&raw mut dir, name);
        *slash = '/' as ::core::ffi::c_char;
        tail = slash.offset(1 as ::core::ffi::c_int as isize);
    }
    if !ok {
        return;
    }
    let mut taillen: size_t = strlen(tail);
    let mut entry: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        entry = os_scandir_next(&raw mut dir);
        if entry.is_null() {
            break;
        }
        if !(strcasecmp(tail, entry as *mut ::core::ffi::c_char) == 0 as ::core::ffi::c_int
            && taillen == strlen(entry))
        {
            continue;
        }
        let mut newname: [::core::ffi::c_char; 4097] = [0; 4097];
        xstrlcpy(
            &raw mut newname as *mut ::core::ffi::c_char,
            name,
            (MAXPATHL + 1 as ::core::ffi::c_int) as size_t,
        );
        xstrlcpy(
            (&raw mut newname as *mut ::core::ffi::c_char).offset(tail.offset_from(name) as isize),
            entry,
            (MAXPATHL as isize - tail.offset_from(name) + 1 as isize) as size_t,
        );
        let mut file_info_new: FileInfo = FileInfo {
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
        if !(os_fileinfo_link(
            &raw mut newname as *mut ::core::ffi::c_char,
            &raw mut file_info_new,
        ) as ::core::ffi::c_int
            != 0
            && os_fileinfo_id_equal(&raw mut file_info, &raw mut file_info_new)
                as ::core::ffi::c_int
                != 0)
        {
            continue;
        }
        strcpy(tail, entry as *mut ::core::ffi::c_char);
        break;
    }
    os_closedir(&raw mut dir);
}
pub unsafe extern "C" fn after_pathsep(
    mut b: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return (p > b
        && vim_ispathsep(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
        && utf_head_off(b, p.offset(-(1 as ::core::ffi::c_int as isize)))
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn same_directory(
    mut f1: *mut ::core::ffi::c_char,
    mut f2: *mut ::core::ffi::c_char,
) -> bool {
    let mut ffname: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut t1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if f1.is_null() || f2.is_null() {
        return false_0 != 0;
    }
    vim_FullName(
        f1,
        &raw mut ffname as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        false_0 != 0,
    );
    t1 = path_tail_with_sep(&raw mut ffname as *mut ::core::ffi::c_char);
    t2 = path_tail_with_sep(f2);
    return t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) == t2.offset_from(f2)
        && pathcmp(
            &raw mut ffname as *mut ::core::ffi::c_char,
            f2,
            t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn pathcmp(
    mut p: *const ::core::ffi::c_char,
    mut q: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    i = 0 as ::core::ffi::c_int;
    j = 0 as ::core::ffi::c_int;
    while maxlen < 0 as ::core::ffi::c_int || i < maxlen && j < maxlen {
        let mut c1: ::core::ffi::c_int = utf_ptr2char(p.offset(i as isize));
        let mut c2: ::core::ffi::c_int = utf_ptr2char(q.offset(j as isize));
        if c1 == NUL {
            if c2 == NUL {
                return 0 as ::core::ffi::c_int;
            }
            s = q;
            i = j;
            break;
        } else if c2 == NUL {
            s = p;
            break;
        } else {
            if if p_fic.get() != 0 {
                (mb_toupper(c1) != mb_toupper(c2)) as ::core::ffi::c_int
            } else {
                (c1 != c2) as ::core::ffi::c_int
            } != 0
            {
                if vim_ispathsep(c1) {
                    return -1 as ::core::ffi::c_int;
                }
                if vim_ispathsep(c2) {
                    return 1 as ::core::ffi::c_int;
                }
                return if p_fic.get() != 0 {
                    mb_toupper(c1) - mb_toupper(c2)
                } else {
                    c1 - c2
                };
            }
            i += utfc_ptr2len(p.offset(i as isize));
            j += utfc_ptr2len(q.offset(j as isize));
        }
    }
    if s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut c1_0: ::core::ffi::c_int = utf_ptr2char(s.offset(i as isize));
    let mut c2_0: ::core::ffi::c_int = utf_ptr2char(
        s.offset(i as isize)
            .offset(utfc_ptr2len(s.offset(i as isize)) as isize),
    );
    if c2_0 == NUL
        && i > 0 as ::core::ffi::c_int
        && after_pathsep(s, s.offset(i as isize)) == 0
        && c1_0 == '/' as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if s == q {
        return -1 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_try_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut dirname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = full_path;
    if os_dirname(dirname, MAXPATHL as size_t) == OK {
        p = path_shorten_fname(full_path, dirname);
        if p.is_null() || *p as ::core::ffi::c_int == NUL {
            p = full_path;
        }
    }
    xfree(dirname as *mut ::core::ffi::c_void);
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn path_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
    mut dir_name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if full_path.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    '_c2rust_label: {
        if !dir_name.is_null() {
        } else {
            __assert_fail(
                b"dir_name != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2108 as ::core::ffi::c_uint,
                b"char *path_shorten_fname(char *, char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: size_t = strlen(dir_name);
    if path_fnamencmp(dir_name, full_path, len) != 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if len == path_head_length() as size_t && is_path_head(dir_name) as ::core::ffi::c_int != 0 {
        return full_path.offset(len as isize);
    }
    let mut p: *mut ::core::ffi::c_char = full_path.offset(len as isize);
    if !vim_ispathsep(*p as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    loop {
        p = p.offset(1);
        if !vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
            break;
        }
    }
    return p;
}
pub unsafe extern "C" fn expand_wildcards_eval(
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut eval_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut exp_pat: *mut ::core::ffi::c_char = *pat;
    let mut ignored_msg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut usedlen: size_t = 0;
    let is_cur_alt_file: bool = *exp_pat as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *exp_pat as ::core::ffi::c_int == '#' as ::core::ffi::c_int;
    let mut star_follows: bool = false_0 != 0;
    if is_cur_alt_file as ::core::ffi::c_int != 0
        || *exp_pat as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        (*emsg_off.ptr()) += 1;
        eval_pat = eval_vars(
            exp_pat,
            exp_pat,
            &raw mut usedlen,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut ignored_msg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        );
        (*emsg_off.ptr()) -= 1;
        if !eval_pat.is_null() {
            star_follows = strcmp(
                exp_pat.offset(usedlen as isize),
                b"*\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int;
            exp_pat = concat_str(eval_pat, exp_pat.offset(usedlen as isize));
        }
    }
    if !exp_pat.is_null() {
        ret = expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut exp_pat,
            num_file,
            file,
            flags,
        );
    }
    if !eval_pat.is_null() {
        if *num_file == 0 as ::core::ffi::c_int
            && is_cur_alt_file as ::core::ffi::c_int != 0
            && star_follows as ::core::ffi::c_int != 0
        {
            *file = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                as *mut *mut ::core::ffi::c_char;
            **file = eval_pat;
            eval_pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
            *num_file = 1 as ::core::ffi::c_int;
            ret = OK;
        }
        xfree(exp_pat as *mut ::core::ffi::c_void);
        xfree(eval_pat as *mut ::core::ffi::c_void);
    }
    return ret;
}
pub unsafe extern "C" fn expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_files: *mut ::core::ffi::c_int,
    mut files: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int =
        gen_expand_wildcards(num_pat, pat, num_files, files, flags);
    if flags & EW_KEEPALL as ::core::ffi::c_int != 0 || retval == FAIL {
        return retval;
    }
    if *p_wig.get() != 0 {
        '_c2rust_label: {
            if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
            } else {
                __assert_fail(
                    b"*num_files == 0 || *files != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2221 as ::core::ffi::c_uint,
                    b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < *num_files {
            let mut ffname: *mut ::core::ffi::c_char =
                FullName_save(*(*files).offset(i as isize), false_0 != 0);
            '_c2rust_label_0: {
                if !(*(*files).offset(i as isize)).is_null() {
                } else {
                    __assert_fail(
                        b"(*files)[i] != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2224 as ::core::ffi::c_uint,
                        b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_1: {
                if !ffname.is_null() {
                } else {
                    __assert_fail(
                        b"ffname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2225 as ::core::ffi::c_uint,
                        b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if match_file_list(p_wig.get(), *(*files).offset(i as isize), ffname) {
                xfree(*(*files).offset(i as isize) as *mut ::core::ffi::c_void);
                let mut j: ::core::ffi::c_int = i;
                while (j + 1 as ::core::ffi::c_int) < *num_files {
                    *(*files).offset(j as isize) =
                        *(*files).offset((j + 1 as ::core::ffi::c_int) as isize);
                    j += 1;
                }
                *num_files -= 1;
                i -= 1;
            }
            xfree(ffname as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    '_c2rust_label_2: {
        if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
        } else {
            __assert_fail(
                b"*num_files == 0 || *files != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2241 as ::core::ffi::c_uint,
                b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if *num_files > 1 as ::core::ffi::c_int && !got_int.get() {
        let mut non_suf_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < *num_files {
            if !match_suffix(*(*files).offset(i_0 as isize)) {
                let mut p: *mut ::core::ffi::c_char = *(*files).offset(i_0 as isize);
                let mut j_0: ::core::ffi::c_int = i_0;
                while j_0 > non_suf_match {
                    *(*files).offset(j_0 as isize) =
                        *(*files).offset((j_0 - 1 as ::core::ffi::c_int) as isize);
                    j_0 -= 1;
                }
                let c2rust_fresh8 = non_suf_match;
                non_suf_match = non_suf_match + 1;
                let c2rust_lvalue_ptr = &raw mut *(*files).offset(c2rust_fresh8 as isize);
                *c2rust_lvalue_ptr = p;
            }
            i_0 += 1;
        }
    }
    if *num_files == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void = files as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return FAIL;
    }
    return retval;
}
pub unsafe extern "C" fn match_suffix(mut fname: *mut ::core::ffi::c_char) -> bool {
    let mut suf_buf: [::core::ffi::c_char; 30] = [0; 30];
    let mut fnamelen: size_t = strlen(fname);
    let mut setsuflen: size_t = 0 as size_t;
    let mut setsuf: *mut ::core::ffi::c_char = p_su.get();
    while *setsuf != 0 {
        setsuflen = copy_option_part(
            &raw mut setsuf,
            &raw mut suf_buf as *mut ::core::ffi::c_char,
            MAXSUFLEN as size_t,
            b".,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if setsuflen == 0 as size_t {
            let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
            if !vim_strchr(tail, '.' as ::core::ffi::c_int).is_null() {
                continue;
            }
            setsuflen = 1 as size_t;
            break;
        } else {
            if fnamelen >= setsuflen
                && path_fnamencmp(
                    &raw mut suf_buf as *mut ::core::ffi::c_char,
                    fname
                        .offset(fnamelen as isize)
                        .offset(-(setsuflen as isize)),
                    setsuflen,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            setsuflen = 0 as size_t;
        }
    }
    return setsuflen != 0 as size_t;
}
pub const MAXSUFLEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn path_full_dir_name(
    mut directory: *mut ::core::ffi::c_char,
    mut buffer: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if strlen(directory) == 0 as size_t {
        return os_dirname(buffer, len);
    }
    if !os_realpath(directory, buffer, len).is_null() {
        return OK;
    }
    if path_is_absolute(directory) {
        return FAIL;
    }
    let mut old_dir: [::core::ffi::c_char; 4096] = [0; 4096];
    if os_dirname(
        &raw mut old_dir as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == FAIL
    {
        return FAIL;
    }
    xstrlcpy(buffer, &raw mut old_dir as *mut ::core::ffi::c_char, len);
    if append_path(buffer, directory, len) == FAIL {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn append_path(
    mut path: *mut ::core::ffi::c_char,
    mut to_append: *const ::core::ffi::c_char,
    mut max_len: size_t,
) -> ::core::ffi::c_int {
    let mut current_length: size_t = strlen(path);
    let mut to_append_length: size_t = strlen(to_append);
    if to_append_length == 0 as size_t
        || strcmp(to_append, b".\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    if current_length > 0 as size_t
        && !vim_ispathsep_nocolon(
            *path.offset(current_length.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        )
    {
        if current_length
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            )
            .wrapping_add(1 as size_t)
            > max_len
        {
            return FAIL;
        }
        xstrlcpy(
            path.offset(current_length as isize),
            PATHSEPSTR.as_ptr(),
            max_len.wrapping_sub(current_length),
        );
        current_length = (current_length as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    if current_length
        .wrapping_add(to_append_length)
        .wrapping_add(1 as size_t)
        > max_len
    {
        return FAIL;
    }
    xstrlcpy(
        path.offset(current_length as isize),
        to_append,
        max_len.wrapping_sub(current_length),
    );
    return OK;
}
unsafe extern "C" fn path_to_absolute(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    *buf = NUL as ::core::ffi::c_char;
    let mut relative_directory: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    let mut end_of_path: *const ::core::ffi::c_char = fname;
    if force != 0 || !path_is_absolute(fname) {
        p = strrchr(fname, '/' as ::core::ffi::c_int);
        if p.is_null()
            && strcmp(fname, b"..\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            p = fname.offset(2 as ::core::ffi::c_int as isize);
        }
        if !p.is_null() {
            if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && strcmp(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    b"..\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(3 as ::core::ffi::c_int as isize);
            }
            '_c2rust_label: {
                if p >= fname {
                } else {
                    __assert_fail(
                        b"p >= fname\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2403 as ::core::ffi::c_uint,
                        b"int path_to_absolute(const char *, char *, size_t, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                relative_directory as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                (p.offset_from(fname) + 1 as isize) as size_t,
            );
            *relative_directory.offset((p.offset_from(fname) + 1 as isize) as isize) =
                NUL as ::core::ffi::c_char;
            end_of_path = if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                p.offset(1 as ::core::ffi::c_int as isize)
            } else {
                p
            };
        } else {
            *relative_directory.offset(0 as ::core::ffi::c_int as isize) =
                NUL as ::core::ffi::c_char;
        }
        if FAIL == path_full_dir_name(relative_directory, buf, len) {
            xfree(relative_directory as *mut ::core::ffi::c_void);
            return FAIL;
        }
    }
    xfree(relative_directory as *mut ::core::ffi::c_void);
    return append_path(buf, end_of_path, len);
}
#[no_mangle]
pub unsafe extern "C" fn path_is_absolute(mut fname: *const ::core::ffi::c_char) -> bool {
    return *fname as ::core::ffi::c_int == '/' as ::core::ffi::c_int
        || *fname as ::core::ffi::c_int == '~' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_guess_exepath(
    mut argv0: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
) {
    let mut path: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    if path.is_null() || path_is_absolute(argv0) as ::core::ffi::c_int != 0 {
        xstrlcpy(buf, argv0, bufsize);
    } else if *argv0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        || !strchr(argv0, PATHSEP).is_null()
    {
        if os_dirname(buf, MAXPATHL as size_t) != OK {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        xstrlcat(buf, PATHSEPSTR.as_ptr(), bufsize);
        xstrlcat(buf, argv0, bufsize);
    } else {
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ENV_SEPCHAR as ::core::ffi::c_char,
                path,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            if dir_len.wrapping_add(1 as size_t)
                <= ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
            {
                xmemcpyz(
                    NameBuff.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    dir as *const ::core::ffi::c_void,
                    dir_len,
                );
                xstrlcat(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                xstrlcat(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    argv0,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                if os_can_exe(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                ) {
                    xstrlcpy(buf, NameBuff.ptr() as *mut ::core::ffi::c_char, bufsize);
                    return;
                }
            }
            if iter.is_null() {
                break;
            }
        }
        xstrlcpy(buf, argv0, bufsize);
    }
    xfree(path as *mut ::core::ffi::c_void);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPECIAL_WILDCHAR: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"`'{\0") };
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_NOBREAK: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
