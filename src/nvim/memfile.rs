use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    ExtmarkUndoObject, FileID, FileInfo, FloatAnchor, FloatRelative, GridView, Intersection, Loop,
    LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MultiQueue, OptInt, Proc, ProcType,
    RStream, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Stream,
    Terminal, Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __off_t, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, alist_T, bhdr_T,
    blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S,
    disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    loop_0_children as C2Rust_Unnamed_20, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue, off_T,
    off_t, partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t,
    stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_22, stream_write_cb,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue,
    uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_17, uv_async_t, uv_buf_t,
    uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_12, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_23, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_16, uv_loop_s_timer_heap as C2Rust_Unnamed_15,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_25, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_13, uv_signal_s_u as C2Rust_Unnamed_14,
    uv_signal_t, uv_stat_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_21, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_24, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_18, uv_timer_s_u as C2Rust_Unnamed_19, uv_timer_t,
    uv_timespec_t, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn lseek(__fd: ::core::ffi::c_int, __offset: __off_t, __whence: ::core::ffi::c_int) -> __off_t;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mh_get_int64_t(set: *mut Set_int64_t, key: int64_t) -> uint32_t;
    fn map_del_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut int64_t,
    ) -> int64_t;
    fn map_put_ref_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
        new_item: *mut bool,
    ) -> *mut int64_t;
    fn map_ref_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
    ) -> *mut int64_t;
    fn map_del_int64_t_ptr_t(
        map: *mut Map_int64_t_ptr_t,
        key: int64_t,
        key_alloc: *mut int64_t,
    ) -> ptr_t;
    fn map_put_ref_int64_t_ptr_t(
        map: *mut Map_int64_t_ptr_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_swapclose: [::core::ffi::c_char; 0];
    fn read_eintr(
        fd: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_void,
        bufsize: size_t,
    ) -> ssize_t;
    fn write_eintr(
        fd: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_void,
        bufsize: size_t,
    ) -> ssize_t;
    static firstbuf: GlobalCell<*mut buf_T>;
    static got_int: GlobalCell<bool>;
    static did_swapwrite_msg: GlobalCell<bool>;
    static main_loop: SharedCell<Loop>;
    fn ml_open_file(buf: *mut buf_T);
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn os_open(
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_fsync(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_fileinfo_link(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_fd(file_descriptor: ::core::ffi::c_int, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_blocksize(file_info: *const FileInfo) -> uint64_t;
    fn os_char_avail() -> bool;
    fn os_breakcheck();
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MFS_ZERO: C2Rust_Unnamed_26 = 8;
pub const MFS_FLUSH: C2Rust_Unnamed_26 = 4;
pub const MFS_STOP: C2Rust_Unnamed_26 = 2;
pub const MFS_ALL: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAX_SWAP_PAGE_SIZE: C2Rust_Unnamed_27 = 50000;
pub const MIN_SWAP_PAGE_SIZE: C2Rust_Unnamed_27 = 1048;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const S_IRUSR: ::core::ffi::c_int = __S_IREAD;
pub const S_IWUSR: ::core::ffi::c_int = __S_IWRITE;
pub const S_IREAD: ::core::ffi::c_int = S_IRUSR;
pub const S_IWRITE: ::core::ffi::c_int = S_IWUSR;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_int64_t = Set_int64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<int64_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_int64_t_int64_t(
    mut map: *mut Map_int64_t_int64_t,
    mut key: int64_t,
    mut value: int64_t,
) {
    let mut val: *mut int64_t = map_put_ref_int64_t_int64_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut int64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
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
#[inline]
unsafe extern "C" fn map_put_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut int64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const BH_DIRTY: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const BH_LOCKED: ::core::ffi::c_uint = 2 as ::core::ffi::c_uint;
pub const MEMFILE_PAGE_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
static e_block_was_not_locked: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E293: Block was not locked\0")
});
#[no_mangle]
pub unsafe extern "C" fn mf_open(
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut memfile_T {
    let mut mfp: *mut memfile_T = xmalloc(::core::mem::size_of::<memfile_T>()) as *mut memfile_T;
    if fname.is_null() {
        (*mfp).mf_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*mfp).mf_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*mfp).mf_fd = -1 as ::core::ffi::c_int;
    } else if !mf_do_open(mfp, fname, flags) {
        xfree(mfp as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<memfile_T>();
    }
    (*mfp).mf_free_first = ::core::ptr::null_mut::<bhdr_T>();
    (*mfp).mf_dirty = MF_DIRTY_NO;
    (*mfp).mf_hash = Map_int64_t_ptr_t {
        set: SET_INIT,
        values: ::core::ptr::null_mut::<ptr_t>(),
    };
    (*mfp).mf_trans = Map_int64_t_int64_t {
        set: SET_INIT,
        values: ::core::ptr::null_mut::<int64_t>(),
    };
    (*mfp).mf_page_size = MEMFILE_PAGE_SIZE as ::core::ffi::c_uint;
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
    if (*mfp).mf_fd >= 0 as ::core::ffi::c_int
        && os_fileinfo_fd((*mfp).mf_fd, &raw mut file_info) as ::core::ffi::c_int != 0
    {
        let mut blocksize: uint64_t = os_fileinfo_blocksize(&raw mut file_info);
        if blocksize >= MIN_SWAP_PAGE_SIZE as ::core::ffi::c_int as uint64_t
            && blocksize <= MAX_SWAP_PAGE_SIZE as ::core::ffi::c_int as uint64_t
        {
            (*mfp).mf_page_size = blocksize as ::core::ffi::c_uint;
        }
    }
    let mut size: off_T = 0;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int || flags & (O_TRUNC | O_EXCL) != 0 || {
        size = lseek((*mfp).mf_fd, 0 as __off_t, SEEK_END) as off_T;
        size <= 0 as off_T
    } {
        (*mfp).mf_blocknr_max = 0 as blocknr_T;
    } else {
        '_c2rust_label: {
            if ::core::mem::size_of::<off_T>() <= ::core::mem::size_of::<blocknr_T>()
                && (*mfp).mf_page_size > 0 as ::core::ffi::c_uint
                && (*mfp).mf_page_size.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_long
                    <= 9223372036854775807 as off_T - size
            {
            } else {
                __assert_fail(
                    b"sizeof(off_T) <= sizeof(blocknr_T) && mfp->mf_page_size > 0 && mfp->mf_page_size - 1 <= INT64_MAX - size\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memfile.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    133 as ::core::ffi::c_uint,
                    b"memfile_T *mf_open(char *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*mfp).mf_blocknr_max = (size as blocknr_T + (*mfp).mf_page_size as blocknr_T
            - 1 as blocknr_T)
            / (*mfp).mf_page_size as blocknr_T;
    }
    (*mfp).mf_blocknr_min = -1 as blocknr_T;
    (*mfp).mf_neg_count = 0 as blocknr_T;
    (*mfp).mf_infile_count = (*mfp).mf_blocknr_max;
    return mfp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_open_file(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if mf_do_open(mfp, fname, O_RDWR | O_CREAT | O_EXCL) {
        (*mfp).mf_dirty = MF_DIRTY_YES;
        return OK;
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn mf_close(mut mfp: *mut memfile_T, mut del_file: bool) {
    if mfp.is_null() {
        return;
    }
    if (*mfp).mf_fd >= 0 as ::core::ffi::c_int && close((*mfp).mf_fd) < 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_swapclose as *const ::core::ffi::c_char,
        ));
    }
    if del_file as ::core::ffi::c_int != 0 && !(*mfp).mf_fname.is_null() {
        os_remove((*mfp).mf_fname);
    }
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        mf_free_bhdr(hp);
        __i = __i.wrapping_add(1);
    }
    while !(*mfp).mf_free_first.is_null() {
        xfree(mf_rem_free(mfp) as *mut ::core::ffi::c_void);
    }
    xfree((*mfp).mf_hash.set.keys as *mut ::core::ffi::c_void);
    xfree((*mfp).mf_hash.set.h.hash as *mut ::core::ffi::c_void);
    (*mfp).mf_hash.set = SET_INIT;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_hash.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    xfree((*mfp).mf_trans.set.keys as *mut ::core::ffi::c_void);
    xfree((*mfp).mf_trans.set.h.hash as *mut ::core::ffi::c_void);
    (*mfp).mf_trans.set = SET_INIT;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_trans.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    mf_free_fnames(mfp);
    xfree(mfp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn mf_close_file(mut buf: *mut buf_T, mut getlines: bool) {
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if mfp.is_null() || (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        return;
    }
    if getlines {
        let mut lnum: linenr_T = 1 as linenr_T;
        while lnum <= (*buf).b_ml.ml_line_count {
            ml_get_buf(buf, lnum);
            lnum += 1;
        }
    }
    if close((*mfp).mf_fd) < 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_swapclose as *const ::core::ffi::c_char,
        ));
    }
    (*mfp).mf_fd = -1 as ::core::ffi::c_int;
    if !(*mfp).mf_fname.is_null() {
        os_remove((*mfp).mf_fname);
        mf_free_fnames(mfp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mf_new_page_size(
    mut mfp: *mut memfile_T,
    mut new_size: ::core::ffi::c_uint,
) {
    (*mfp).mf_page_size = new_size;
}
#[no_mangle]
pub unsafe extern "C" fn mf_new(
    mut mfp: *mut memfile_T,
    mut negative: bool,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut freep: *mut bhdr_T = (*mfp).mf_free_first;
    if !negative && !freep.is_null() && (*freep).bh_page_count >= page_count {
        if (*freep).bh_page_count > page_count {
            hp = mf_alloc_bhdr(mfp, page_count);
            (*hp).bh_bnum = (*freep).bh_bnum;
            (*freep).bh_bnum += page_count as blocknr_T;
            (*freep).bh_page_count = (*freep).bh_page_count.wrapping_sub(page_count);
        } else {
            let mut p: *mut ::core::ffi::c_void =
                xmalloc(((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t));
            hp = mf_rem_free(mfp);
            (*hp).bh_data = p;
        }
    } else {
        hp = mf_alloc_bhdr(mfp, page_count);
        if negative {
            let c2rust_fresh0 = (*mfp).mf_blocknr_min;
            (*mfp).mf_blocknr_min = (*mfp).mf_blocknr_min - 1;
            (*hp).bh_bnum = c2rust_fresh0;
            (*mfp).mf_neg_count += 1;
        } else {
            (*hp).bh_bnum = (*mfp).mf_blocknr_max;
            (*mfp).mf_blocknr_max += page_count as blocknr_T;
        }
    }
    (*hp).bh_flags = BH_LOCKED | BH_DIRTY;
    (*mfp).mf_dirty = MF_DIRTY_YES;
    (*hp).bh_page_count = page_count;
    map_put_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        hp as ptr_t,
    );
    memset(
        (*hp).bh_data,
        0 as ::core::ffi::c_int,
        ((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t),
    );
    return hp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_get(
    mut mfp: *mut memfile_T,
    mut nr: blocknr_T,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    if nr >= (*mfp).mf_blocknr_max || nr <= (*mfp).mf_blocknr_min {
        return ::core::ptr::null_mut::<bhdr_T>();
    }
    let mut hp: *mut bhdr_T =
        map_get_int64_t_ptr_t(&raw mut (*mfp).mf_hash, nr as int64_t) as *mut bhdr_T;
    if hp.is_null() {
        if nr < 0 as blocknr_T || nr >= (*mfp).mf_infile_count {
            return ::core::ptr::null_mut::<bhdr_T>();
        }
        if page_count > 0 as ::core::ffi::c_uint {
            hp = mf_alloc_bhdr(mfp, page_count);
        }
        if hp.is_null() {
            return ::core::ptr::null_mut::<bhdr_T>();
        }
        (*hp).bh_bnum = nr;
        (*hp).bh_flags = 0 as ::core::ffi::c_uint;
        (*hp).bh_page_count = page_count;
        if mf_read(mfp, hp) == FAIL {
            mf_free_bhdr(hp);
            return ::core::ptr::null_mut::<bhdr_T>();
        }
    } else {
        map_del_int64_t_ptr_t(
            &raw mut (*mfp).mf_hash,
            (*hp).bh_bnum as int64_t,
            ::core::ptr::null_mut::<int64_t>(),
        );
    }
    (*hp).bh_flags |= BH_LOCKED;
    map_put_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        hp as ptr_t,
    );
    return hp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_put(
    mut mfp: *mut memfile_T,
    mut hp: *mut bhdr_T,
    mut dirty: bool,
    mut infile: bool,
) {
    let mut flags: ::core::ffi::c_uint = (*hp).bh_flags;
    if flags & BH_LOCKED == 0 as ::core::ffi::c_uint {
        iemsg(gettext(
            (e_block_was_not_locked.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
    }
    flags &= !BH_LOCKED;
    if dirty {
        flags |= BH_DIRTY;
        if (*mfp).mf_dirty as ::core::ffi::c_uint
            != MF_DIRTY_YES_NOSYNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*mfp).mf_dirty = MF_DIRTY_YES;
        }
    }
    (*hp).bh_flags = flags;
    if infile {
        mf_trans_add(mfp, hp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mf_free(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) {
    xfree((*hp).bh_data);
    map_del_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    if (*hp).bh_bnum < 0 as blocknr_T {
        xfree(hp as *mut ::core::ffi::c_void);
        (*mfp).mf_neg_count -= 1;
    } else {
        mf_ins_free(mfp, hp);
    };
}
#[no_mangle]
pub unsafe extern "C" fn mf_sync(
    mut mfp: *mut memfile_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut got_int_save: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        (*mfp).mf_dirty = MF_DIRTY_NO;
        return FAIL;
    }
    got_int.set(false_0 != 0);
    let mut status: ::core::ffi::c_int = OK;
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        if (flags & MFS_ALL as ::core::ffi::c_int != 0 || (*hp).bh_bnum >= 0 as blocknr_T)
            && (*hp).bh_flags & 1 as ::core::ffi::c_uint != 0
            && (status == 1 as ::core::ffi::c_int
                || (*hp).bh_bnum >= 0 as blocknr_T && (*hp).bh_bnum < (*mfp).mf_infile_count)
        {
            if !(flags & MFS_ZERO as ::core::ffi::c_int != 0 && (*hp).bh_bnum != 0 as blocknr_T) {
                if mf_write(mfp, hp) == 0 as ::core::ffi::c_int {
                    if status == 0 as ::core::ffi::c_int {
                        break;
                    }
                    status = 0 as ::core::ffi::c_int;
                }
                if flags & MFS_STOP as ::core::ffi::c_int != 0 {
                    if os_char_avail() {
                        break;
                    }
                } else if (*main_loop.ptr()).recursive == 0 {
                    os_breakcheck();
                }
                if got_int.get() {
                    break;
                }
            }
        }
        __i = __i.wrapping_add(1);
    }
    if hp.is_null() || status == FAIL {
        (*mfp).mf_dirty = MF_DIRTY_NO;
    }
    if flags & MFS_FLUSH as ::core::ffi::c_int != 0 {
        if os_fsync((*mfp).mf_fd) != 0 {
            status = FAIL;
        }
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | got_int_save != 0);
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mf_set_dirty(mut mfp: *mut memfile_T) {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        if (*hp).bh_bnum > 0 as blocknr_T {
            (*hp).bh_flags |= 1 as ::core::ffi::c_uint;
        }
        __i = __i.wrapping_add(1);
    }
    (*mfp).mf_dirty = MF_DIRTY_YES;
}
#[no_mangle]
pub unsafe extern "C" fn mf_release_all() -> bool {
    let mut retval: bool = false_0 != 0;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if !mfp.is_null() {
            if (*mfp).mf_fd < 0 as ::core::ffi::c_int
                && (*buf).b_may_swap as ::core::ffi::c_int != 0
            {
                ml_open_file(buf);
            }
            if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*mfp).mf_hash.set.h.size as ::core::ffi::c_int {
                    let mut hp: *mut bhdr_T =
                        *(*mfp).mf_hash.values.offset(i as isize) as *mut bhdr_T;
                    if (*hp).bh_flags & BH_LOCKED == 0
                        && ((*hp).bh_flags & BH_DIRTY == 0 || mf_write(mfp, hp) != FAIL)
                    {
                        map_del_int64_t_ptr_t(
                            &raw mut (*mfp).mf_hash,
                            (*hp).bh_bnum as int64_t,
                            ::core::ptr::null_mut::<int64_t>(),
                        );
                        mf_free_bhdr(hp);
                        retval = true_0 != 0;
                    } else {
                        i += 1;
                    }
                }
            }
        }
        buf = (*buf).b_next;
    }
    return retval;
}
unsafe extern "C" fn mf_alloc_bhdr(
    mut mfp: *mut memfile_T,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = xmalloc(::core::mem::size_of::<bhdr_T>()) as *mut bhdr_T;
    (*hp).bh_data = xmalloc(((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t));
    (*hp).bh_page_count = page_count;
    return hp;
}
unsafe extern "C" fn mf_free_bhdr(mut hp: *mut bhdr_T) {
    xfree((*hp).bh_data);
    xfree(hp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn mf_ins_free(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) {
    (*hp).bh_data = (*mfp).mf_free_first as *mut ::core::ffi::c_void;
    (*mfp).mf_free_first = hp;
}
unsafe extern "C" fn mf_rem_free(mut mfp: *mut memfile_T) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = (*mfp).mf_free_first;
    (*mfp).mf_free_first = (*hp).bh_data as *mut bhdr_T;
    return hp;
}
unsafe extern "C" fn mf_read(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) -> ::core::ffi::c_int {
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    let mut page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
    let mut offset: off_T = (page_size as blocknr_T * (*hp).bh_bnum) as off_T;
    if lseek((*mfp).mf_fd, offset as __off_t, SEEK_SET) != offset {
        semsg(
            b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(b"E294: Seek error in swap file read\0".as_ptr() as *const ::core::ffi::c_char),
            strerror(*__errno_location()),
        );
        return FAIL;
    }
    '_c2rust_label: {
        if (*hp).bh_page_count
            <= (2147483647 as ::core::ffi::c_int as ::core::ffi::c_uint)
                .wrapping_mul(2 as ::core::ffi::c_uint)
                .wrapping_add(1 as ::core::ffi::c_uint)
                .wrapping_div(page_size)
        {
        } else {
            __assert_fail(
                b"hp->bh_page_count <= UINT_MAX / page_size\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/memfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                545 as ::core::ffi::c_uint,
                b"int mf_read(memfile_T *, bhdr_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut size: ::core::ffi::c_uint = page_size.wrapping_mul((*hp).bh_page_count);
    if read_eintr((*mfp).mf_fd, (*hp).bh_data, size as size_t) as ::core::ffi::c_uint != size {
        semsg(
            b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(b"E295: Read error in swap file\0".as_ptr() as *const ::core::ffi::c_char),
            strerror(*__errno_location()),
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn mf_write(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) -> ::core::ffi::c_int {
    let mut hp2: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut page_count: ::core::ffi::c_uint = 0;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int && !(*mfp).mf_reopen {
        return FAIL;
    }
    if (*hp).bh_bnum < 0 as blocknr_T {
        if mf_trans_add(mfp, hp) == FAIL {
            return FAIL;
        }
    }
    let mut page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
    loop {
        let mut nr: blocknr_T = (*hp).bh_bnum;
        if nr > (*mfp).mf_infile_count {
            nr = (*mfp).mf_infile_count;
            hp2 = map_get_int64_t_ptr_t(&raw mut (*mfp).mf_hash, nr as int64_t) as *mut bhdr_T;
        } else {
            hp2 = hp;
        }
        let mut offset: off_T = (page_size as blocknr_T * nr) as off_T;
        if hp2.is_null() {
            page_count = 1 as ::core::ffi::c_uint;
        } else {
            page_count = (*hp2).bh_page_count;
        }
        let mut size: ::core::ffi::c_uint = page_size.wrapping_mul(page_count);
        let mut attempt: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while attempt <= 2 as ::core::ffi::c_int {
            if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                if lseek((*mfp).mf_fd, offset as __off_t, SEEK_SET) != offset {
                    semsg(
                        b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"E296: Seek error in swap file write\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        strerror(*__errno_location()),
                    );
                    return FAIL;
                }
                let mut data: *mut ::core::ffi::c_void = if hp2.is_null() {
                    (*hp).bh_data
                } else {
                    (*hp2).bh_data
                };
                if write_eintr((*mfp).mf_fd, data, size as size_t) as ::core::ffi::c_uint == size {
                    break;
                }
            }
            if attempt == 1 as ::core::ffi::c_int {
                if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                    close((*mfp).mf_fd);
                }
                (*mfp).mf_fd = os_open((*mfp).mf_fname, (*mfp).mf_flags, S_IREAD | S_IWRITE);
                (*mfp).mf_reopen = (*mfp).mf_fd < 0 as ::core::ffi::c_int;
            }
            if attempt == 2 as ::core::ffi::c_int || (*mfp).mf_fd < 0 as ::core::ffi::c_int {
                if !did_swapwrite_msg.get() {
                    emsg(gettext(
                        b"E297: Write error in swap file\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                did_swapwrite_msg.set(true_0 != 0);
                return FAIL;
            }
            attempt += 1;
        }
        did_swapwrite_msg.set(false_0 != 0);
        if !hp2.is_null() {
            (*hp2).bh_flags &= !BH_DIRTY;
        }
        if nr + page_count as blocknr_T > (*mfp).mf_infile_count {
            (*mfp).mf_infile_count = nr + page_count as blocknr_T;
        }
        if nr == (*hp).bh_bnum {
            break;
        }
    }
    return OK;
}
unsafe extern "C" fn mf_trans_add(
    mut mfp: *mut memfile_T,
    mut hp: *mut bhdr_T,
) -> ::core::ffi::c_int {
    if (*hp).bh_bnum >= 0 as blocknr_T {
        return OK;
    }
    let mut new_bnum: blocknr_T = 0;
    let mut freep: *mut bhdr_T = (*mfp).mf_free_first;
    let mut page_count: ::core::ffi::c_uint = (*hp).bh_page_count;
    if !freep.is_null() && (*freep).bh_page_count >= page_count {
        new_bnum = (*freep).bh_bnum;
        if (*freep).bh_page_count > page_count {
            (*freep).bh_bnum += page_count as blocknr_T;
            (*freep).bh_page_count = (*freep).bh_page_count.wrapping_sub(page_count);
        } else {
            freep = mf_rem_free(mfp);
            xfree(freep as *mut ::core::ffi::c_void);
        }
    } else {
        new_bnum = (*mfp).mf_blocknr_max;
        (*mfp).mf_blocknr_max += page_count as blocknr_T;
    }
    let mut old_bnum: blocknr_T = (*hp).bh_bnum;
    map_del_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    (*hp).bh_bnum = new_bnum;
    map_put_int64_t_ptr_t(&raw mut (*mfp).mf_hash, new_bnum as int64_t, hp as ptr_t);
    map_put_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_bnum as int64_t,
        new_bnum as int64_t,
    );
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn mf_trans_del(mut mfp: *mut memfile_T, mut old_nr: blocknr_T) -> blocknr_T {
    let mut num: *mut blocknr_T = map_ref_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_nr as int64_t,
        ::core::ptr::null_mut::<*mut int64_t>(),
    ) as *mut blocknr_T;
    if num.is_null() {
        return old_nr;
    }
    (*mfp).mf_neg_count -= 1;
    let mut new_bnum: blocknr_T = *num;
    map_del_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_nr as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    return new_bnum;
}
#[no_mangle]
pub unsafe extern "C" fn mf_free_fnames(mut mfp: *mut memfile_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_fname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_ffname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
}
#[no_mangle]
pub unsafe extern "C" fn mf_set_fnames(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
) {
    (*mfp).mf_fname = fname;
    (*mfp).mf_ffname = FullName_save((*mfp).mf_fname, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn mf_fullname(mut mfp: *mut memfile_T) {
    if mfp.is_null() || (*mfp).mf_fname.is_null() || (*mfp).mf_ffname.is_null() {
        return;
    }
    xfree((*mfp).mf_fname as *mut ::core::ffi::c_void);
    (*mfp).mf_fname = (*mfp).mf_ffname;
    (*mfp).mf_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn mf_need_trans(mut mfp: *mut memfile_T) -> bool {
    return !(*mfp).mf_fname.is_null() && (*mfp).mf_neg_count > 0 as blocknr_T;
}
unsafe extern "C" fn mf_do_open(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    mf_set_fnames(mfp, fname);
    '_c2rust_label: {
        if !(*mfp).mf_fname.is_null() {
        } else {
            __assert_fail(
                b"mfp->mf_fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                763 as ::core::ffi::c_uint,
                b"_Bool mf_do_open(memfile_T *, char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
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
    if flags & O_CREAT != 0
        && os_fileinfo_link((*mfp).mf_fname, &raw mut file_info) as ::core::ffi::c_int != 0
    {
        (*mfp).mf_fd = -1 as ::core::ffi::c_int;
        emsg(gettext(
            b"E300: Swap file already exists (symlink attack?)\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    } else {
        flags |= O_NOFOLLOW;
        (*mfp).mf_flags = flags;
        (*mfp).mf_fd = os_open((*mfp).mf_fname, flags, S_IREAD | S_IWRITE);
    }
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        mf_free_fnames(mfp);
        return false_0 != 0;
    }
    os_set_cloexec((*mfp).mf_fd);
    return true_0 != 0;
}
pub const __S_IREAD: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const __S_IWRITE: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
