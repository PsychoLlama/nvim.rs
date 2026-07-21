use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn abort() -> !;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn getuid() -> __uid_t;
    fn getgid() -> __gid_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xmemdup(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t;
    fn mh_put_cstr_t(set: *mut Set_cstr_t, key: cstr_t, new: *mut MHPutStatus) -> uint32_t;
    fn mh_get_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> uint32_t;
    fn mh_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, new: *mut MHPutStatus) -> uint32_t;
    fn map_del_cstr_t_ptr_t(
        map: *mut Map_cstr_t_ptr_t,
        key: cstr_t,
        key_alloc: *mut cstr_t,
    ) -> ptr_t;
    fn map_ref_cstr_t_ptr_t(
        map: *mut Map_cstr_t_ptr_t,
        key: cstr_t,
        key_alloc: *mut *mut cstr_t,
    ) -> *mut ptr_t;
    fn map_put_ref_cstr_t_ptr_t(
        map: *mut Map_cstr_t_ptr_t,
        key: cstr_t,
        key_alloc: *mut *mut cstr_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_free_string(value: String_0);
    fn api_free_dict(value: Dict);
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buflist_setfpos(
        buf: *mut buf_T,
        win: *mut win_T,
        lnum: linenr_T,
        col: colnr_T,
        copy_options: bool,
    );
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn bt_terminal(buf: *const buf_T) -> bool;
    fn KeyDict__shada_search_pat_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict__shada_mark_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict__shada_register_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict__shada_buflist_item_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn clr_history(histype: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn hist_iter(
        iter: *const ::core::ffi::c_void,
        history_type: uint8_t,
        zero: bool,
        hist: *mut histentry_T,
    ) -> *const ::core::ffi::c_void;
    fn hist_get_array(
        history_type: uint8_t,
        new_hisidx: *mut *mut ::core::ffi::c_int,
        new_hisnum: *mut *mut ::core::ffi::c_int,
    ) -> *mut histentry_T;
    fn decode_string(
        s: *const ::core::ffi::c_char,
        len: size_t,
        force_blob: bool,
        s_allocated: bool,
    ) -> typval_T;
    fn unpack_typval(
        data: *mut *const ::core::ffi::c_char,
        size: *mut size_t,
        ret: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn get_copyID() -> ::core::ffi::c_int;
    fn set_ref_in_ht(
        ht: *mut hashtab_T,
        copyID: ::core::ffi::c_int,
        list_stack: *mut *mut list_stack_T,
    ) -> bool;
    fn set_ref_in_list_items(
        l: *mut list_T,
        copyID: ::core::ffi::c_int,
        ht_stack: *mut *mut ht_stack_T,
    ) -> bool;
    fn var_flavour(varname: *mut ::core::ffi::c_char) -> var_flavour_T;
    fn var_set_global(name: *const ::core::ffi::c_char, vartv: typval_T);
    fn encode_vim_to_msgpack(
        packer: *mut PackerBuffer,
        tv: *mut typval_T,
        objname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static hash_removed: ::core::ffi::c_char;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn siemsg(s: *const ::core::ffi::c_char, ...);
    fn verbose_enter();
    fn verbose_leave();
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_allocated_string(l: *mut list_T, str: *mut ::core::ffi::c_char);
    fn tv_clear(tv: *mut typval_T);
    fn tv_copy(from: *const typval_T, to: *mut typval_T);
    fn sub_get_replacement(ret_sub: *mut SubReplacementString);
    fn sub_set_replacement(sub: SubReplacementString);
    fn set_no_hlsearch(flag: bool);
    fn get_globvar_ht() -> *mut hashtab_T;
    fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T;
    fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T);
    fn modname(
        fname: *const ::core::ffi::c_char,
        ext: *const ::core::ffi::c_char,
        prepend_dot: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_rename(
        from: *const ::core::ffi::c_char,
        to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static firstbuf: GlobalCell<*mut buf_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static no_hlsearch: GlobalCell<bool>;
    fn os_time() -> Timestamp;
    fn free_fmark(fm: fmark_T);
    fn free_xfmark(fm: xfmark_T);
    fn setpcmark();
    fn mark_get(
        buf: *mut buf_T,
        win: *mut win_T,
        fmp: *mut fmark_T,
        flag: MarkGet,
        name: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn cleanup_jumplist(wp: *mut win_T, loadfiles: bool);
    fn mark_jumplist_iter(
        iter: *const ::core::ffi::c_void,
        win: *const win_T,
        fm: *mut xfmark_T,
    ) -> *const ::core::ffi::c_void;
    fn mark_global_iter(
        iter: *const ::core::ffi::c_void,
        name: *mut ::core::ffi::c_char,
        fm: *mut xfmark_T,
    ) -> *const ::core::ffi::c_void;
    fn mark_buffer_iter(
        iter: *const ::core::ffi::c_void,
        buf: *const buf_T,
        name: *mut ::core::ffi::c_char,
        fm: *mut fmark_T,
    ) -> *const ::core::ffi::c_void;
    fn mark_set_global(name: ::core::ffi::c_char, fm: xfmark_T, update: bool) -> bool;
    fn mark_set_local(
        name: ::core::ffi::c_char,
        buf: *mut buf_T,
        fm: fmark_T,
        update: bool,
    ) -> bool;
    fn set_last_cursor(win: *mut win_T);
    static namedfm: GlobalCell<[xfmark_T; 36]>;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn mpack_check_buffer(packer: *mut PackerBuffer);
    fn mpack_uint64(ptr: *mut *mut ::core::ffi::c_char, i: uint64_t);
    fn mpack_integer(ptr: *mut *mut ::core::ffi::c_char, i: Integer);
    fn mpack_str(str: String_0, packer: *mut PackerBuffer);
    fn mpack_bin(str: String_0, packer: *mut PackerBuffer);
    fn mpack_raw(data: *const ::core::ffi::c_char, len: size_t, packer: *mut PackerBuffer);
    fn packer_string_buffer() -> PackerBuffer;
    fn packer_take_string(buffer: *mut PackerBuffer) -> String_0;
    fn unpack_string(data: *mut *const ::core::ffi::c_char, size: *mut size_t) -> String_0;
    fn unpack_array(data: *mut *const ::core::ffi::c_char, size: *mut size_t) -> ssize_t;
    fn unpack_integer(
        data: *mut *const ::core::ffi::c_char,
        size: *mut size_t,
        res: *mut Integer,
    ) -> bool;
    fn unpack_skip(data: *mut *const ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    fn push_additional_data(
        ad: *mut AdditionalDataBuilder,
        data: *const ::core::ffi::c_char,
        size: size_t,
    );
    fn unpack_keydict(
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        ad: *mut AdditionalDataBuilder,
        data: *mut *const ::core::ffi::c_char,
        size: *mut size_t,
        error: *mut *mut ::core::ffi::c_char,
    ) -> bool;
    fn magic_isset() -> bool;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    static p_enc: GlobalCell<*mut ::core::ffi::c_char>;
    static p_fs: GlobalCell<::core::ffi::c_int>;
    static p_hi: GlobalCell<OptInt>;
    static p_shada: GlobalCell<*mut ::core::ffi::c_char>;
    static p_shadafile: GlobalCell<*mut ::core::ffi::c_char>;
    static p_verbose: GlobalCell<OptInt>;
    fn file_open(
        ret_fp: *mut FileDescriptor,
        fname: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn file_open_buffer(ret_fp: *mut FileDescriptor, data: *mut ::core::ffi::c_char, len: size_t);
    fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> ::core::ffi::c_int;
    fn file_flush(fp: *mut FileDescriptor) -> ::core::ffi::c_int;
    fn file_read(
        fp: *mut FileDescriptor,
        ret_buf: *mut ::core::ffi::c_char,
        size: size_t,
    ) -> ptrdiff_t;
    fn file_try_read_buffered(fp: *mut FileDescriptor, size: size_t) -> *mut ::core::ffi::c_char;
    fn file_skip(fp: *mut FileDescriptor, size: size_t) -> ptrdiff_t;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_getperm(name: *const ::core::ffi::c_char) -> int32_t;
    fn os_fchown(fd: ::core::ffi::c_int, owner: uv_uid_t, group: uv_gid_t) -> ::core::ffi::c_int;
    fn os_mkdir_recurse(
        dir: *const ::core::ffi::c_char,
        mode: int32_t,
        failed_dir: *mut *mut ::core::ffi::c_char,
        created: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_fileinfo(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_get_pid() -> int64_t;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn home_replace_save(
        buf: *mut buf_T,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn stdpaths_user_state_subpath(
        fname: *const ::core::ffi::c_char,
        trailing_pathseps: size_t,
        escape_commas: bool,
    ) -> *mut ::core::ffi::c_char;
    fn regtilde(
        source: *mut ::core::ffi::c_char,
        magic: ::core::ffi::c_int,
        preview: bool,
    ) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn concat_fnames_realloc(
        fname1: *mut ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        sep: bool,
    ) -> *mut ::core::ffi::c_char;
    fn path_try_shorten_fname(full_path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn op_global_reg_iter(
        iter: *const ::core::ffi::c_void,
        name: *mut ::core::ffi::c_char,
        reg: *mut yankreg_T,
        is_unnamed: *mut bool,
    ) -> *const ::core::ffi::c_void;
    fn op_reg_set(name: ::core::ffi::c_char, reg: yankreg_T, is_unnamed: bool) -> bool;
    fn op_reg_get(name: ::core::ffi::c_char) -> *const yankreg_T;
    fn get_search_pattern(pat: *mut SearchPattern);
    fn get_substitute_pattern(pat: *mut SearchPattern);
    fn set_search_pattern(pat: SearchPattern);
    fn set_substitute_pattern(pat: SearchPattern);
    fn set_last_used_pattern(is_substitute_pattern: bool);
    fn search_was_last_used() -> bool;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static longVersion: GlobalCell<*mut ::core::ffi::c_char>;
}
pub type __uint64_t = u64;
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintmax_t = ::libc::uintmax_t;
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed = -8;
pub const UV_EUNATCH: C2Rust_Unnamed = -49;
pub const UV_ENODATA: C2Rust_Unnamed = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed = -94;
pub const UV_EILSEQ: C2Rust_Unnamed = -84;
pub const UV_EFTYPE: C2Rust_Unnamed = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed = -112;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_ENXIO: C2Rust_Unnamed = -6;
pub const UV_EOF: C2Rust_Unnamed = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_EXDEV: C2Rust_Unnamed = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed = -110;
pub const UV_ESRCH: C2Rust_Unnamed = -3;
pub const UV_ESPIPE: C2Rust_Unnamed = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed = -108;
pub const UV_EROFS: C2Rust_Unnamed = -30;
pub const UV_ERANGE: C2Rust_Unnamed = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed = -93;
pub const UV_EPROTO: C2Rust_Unnamed = -71;
pub const UV_EPIPE: C2Rust_Unnamed = -32;
pub const UV_EPERM: C2Rust_Unnamed = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed = -107;
pub const UV_ENOSYS: C2Rust_Unnamed = -38;
pub const UV_ENOSPC: C2Rust_Unnamed = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed = -92;
pub const UV_ENONET: C2Rust_Unnamed = -64;
pub const UV_ENOMEM: C2Rust_Unnamed = -12;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENODEV: C2Rust_Unnamed = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ENFILE: C2Rust_Unnamed = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed = -90;
pub const UV_EMFILE: C2Rust_Unnamed = -24;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EISCONN: C2Rust_Unnamed = -106;
pub const UV_EIO: C2Rust_Unnamed = -5;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EINTR: C2Rust_Unnamed = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed = -113;
pub const UV_EFBIG: C2Rust_Unnamed = -27;
pub const UV_EFAULT: C2Rust_Unnamed = -14;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed = -103;
pub const UV_ECHARSET: C2Rust_Unnamed = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed = -125;
pub const UV_EBUSY: C2Rust_Unnamed = -16;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const UV_EALREADY: C2Rust_Unnamed = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed = -98;
pub const UV_EACCES: C2Rust_Unnamed = -13;
pub const UV_E2BIG: C2Rust_Unnamed = -7;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec_t {
    pub tv_sec: ::core::ffi::c_long,
    pub tv_nsec: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed_0 = 2;
pub const MPACK_EOF: C2Rust_Unnamed_0 = 1;
pub const MPACK_OK: C2Rust_Unnamed_0 = 0;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
    pub boolean: Boolean,
    pub integer: Integer,
    pub floating: Float,
    pub string: String_0,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
}
pub type KeyValuePair = key_value_pair;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Float = ::core::ffi::c_double;
pub type Integer = int64_t;
pub type Boolean = bool;
pub type ObjectType = ::core::ffi::c_uint;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub type proftime_T = uint64_t;
pub type OptInt = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct file_buffer {
    pub handle: handle_T,
    pub b_ml: memline_T,
    pub b_next: *mut buf_T,
    pub b_prev: *mut buf_T,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: ::core::ffi::c_int,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: varnumber_T,
    pub b_last_changedtick_i: varnumber_T,
    pub b_last_changedtick_pum: varnumber_T,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: linenr_T,
    pub b_mod_bot: linenr_T,
    pub b_mod_xlines: linenr_T,
    pub b_wininfo: C2Rust_Unnamed_13,
    pub b_mod_tick_syn: disptick_T,
    pub b_mod_tick_decor: disptick_T,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [fmark_T; 26],
    pub b_visual: visualinfo_T,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: fmark_T,
    pub b_last_insert: fmark_T,
    pub b_last_change: fmark_T,
    pub b_changelist: [fmark_T; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut mapblock_T; 256],
    pub b_first_abbr: *mut mapblock_T,
    pub b_ucmds: garray_T,
    pub b_op_start: pos_T,
    pub b_op_start_orig: pos_T,
    pub b_op_end: pos_T,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    pub b_u_oldhead: *mut u_header_T,
    pub b_u_newhead: *mut u_header_T,
    pub b_u_curhead: *mut u_header_T,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: linenr_T,
    pub b_u_line_colnr: colnr_T,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub b_kmap_ga: garray_T,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [sctx_T; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut colnr_T,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut colnr_T,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: linenr_T,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut dict_T,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: fmark_T,
    pub b_s: synblock_T,
    pub b_signcols: C2Rust_Unnamed_5,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_3,
    pub update_callbacks: C2Rust_Unnamed_2,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint32_t_uint32_t {
    pub set: Set_uint32_t,
    pub values: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint32_t {
    pub h: MapHash,
    pub keys: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MapHash {
    pub n_buckets: uint32_t,
    pub size: uint32_t,
    pub n_occupied: uint32_t,
    pub upper_bound: uint32_t,
    pub n_keys: uint32_t,
    pub keys_capacity: uint32_t,
    pub hash: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTree {
    pub root: *mut MTNode,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub n_nodes: size_t,
    pub id2node: [Map_uint64_t_ptr_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_ptr_t {
    pub set: Set_uint64_t,
    pub values: *mut ptr_t,
}
pub type ptr_t = *mut ::core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
pub type MTNode = mtnode_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_s {
    pub n: int32_t,
    pub level: int16_t,
    pub p_idx: int16_t,
    pub intersect: Intersection,
    pub parent: *mut MTNode,
    pub key: [MTKey; 19],
    pub s: [mtnode_inner_s; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_inner_s {
    pub i_ptr: [*mut MTNode; 20],
    pub i_meta: [[uint32_t; 5]; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: uint16_t,
    pub decor_data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorInlineData {
    pub hl: DecorHighlightInline,
    pub ext: DecorExt,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: C2Rust_Unnamed_4,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub virt_text: VirtText,
    pub virt_lines: VirtLines,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtTextChunk {
    pub text: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type DecorPriority = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Intersection {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
    pub init_array: [uint64_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_5 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synblock_T {
    pub b_keywtab: hashtab_T,
    pub b_keywtab_ic: hashtab_T,
    pub b_syn_error: bool,
    pub b_syn_slow: bool,
    pub b_syn_ic: ::core::ffi::c_int,
    pub b_syn_foldlevel: ::core::ffi::c_int,
    pub b_syn_spell: ::core::ffi::c_int,
    pub b_syn_patterns: garray_T,
    pub b_syn_clusters: garray_T,
    pub b_spell_cluster_id: ::core::ffi::c_int,
    pub b_nospell_cluster_id: ::core::ffi::c_int,
    pub b_syn_containedin: ::core::ffi::c_int,
    pub b_syn_sync_flags: ::core::ffi::c_int,
    pub b_syn_sync_id: int16_t,
    pub b_syn_sync_minlines: linenr_T,
    pub b_syn_sync_maxlines: linenr_T,
    pub b_syn_sync_linebreaks: linenr_T,
    pub b_syn_linecont_pat: *mut ::core::ffi::c_char,
    pub b_syn_linecont_prog: *mut regprog_T,
    pub b_syn_linecont_time: syn_time_T,
    pub b_syn_linecont_ic: ::core::ffi::c_int,
    pub b_syn_topgrp: ::core::ffi::c_int,
    pub b_syn_conceal: ::core::ffi::c_int,
    pub b_syn_folditems: ::core::ffi::c_int,
    pub b_sst_array: *mut synstate_T,
    pub b_sst_len: ::core::ffi::c_int,
    pub b_sst_first: *mut synstate_T,
    pub b_sst_firstfree: *mut synstate_T,
    pub b_sst_freecount: ::core::ffi::c_int,
    pub b_sst_check_lnum: linenr_T,
    pub b_sst_lasttick: disptick_T,
    pub b_langp: garray_T,
    pub b_spell_ismw: [bool; 256],
    pub b_spell_ismw_mb: *mut ::core::ffi::c_char,
    pub b_p_spc: *mut ::core::ffi::c_char,
    pub b_cap_prog: *mut regprog_T,
    pub b_p_spf: *mut ::core::ffi::c_char,
    pub b_p_spl: *mut ::core::ffi::c_char,
    pub b_p_spo: *mut ::core::ffi::c_char,
    pub b_p_spo_flags: ::core::ffi::c_uint,
    pub b_cjk: ::core::ffi::c_int,
    pub b_syn_chartab: [uint8_t; 32],
    pub b_syn_isk: *mut ::core::ffi::c_char,
}
pub type regprog_T = regprog;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type disptick_T = uint64_t;
pub type linenr_T = int32_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_6,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub sst_stack: [bufstate_T; 7],
    pub sst_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufstate_T {
    pub bs_idx: ::core::ffi::c_int,
    pub bs_flags: ::core::ffi::c_int,
    pub bs_seqnr: ::core::ffi::c_int,
    pub bs_cchar: ::core::ffi::c_int,
    pub bs_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg_extmatch_T {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_time_T {
    pub total: proftime_T,
    pub slowest: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmark_T {
    pub mark: pos_T,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: fmarkv_T,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmarkv_T {
    pub topline_offset: linenr_T,
    pub skipcol: colnr_T,
}
pub type colnr_T = ::core::ffi::c_int;
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_7,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
}
pub type partial_T = partial_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct partial_S {
    pub pt_refcount: ::core::ffi::c_int,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
pub type QUEUE = queue;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLockStatus,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: ::core::ffi::c_int,
    pub bv_lock: VarLockStatus,
}
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: ::core::ffi::c_int,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLockStatus,
    pub lua_table_ref: LuaRef,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type SpecialVarValue = ::core::ffi::c_uint;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type BoolVarValue = ::core::ffi::c_uint;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type varnumber_T = int64_t;
pub type VarType = ::core::ffi::c_uint;
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
pub type ufunc_T = ufunc_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: ::core::ffi::c_int,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [C2Rust_Unnamed_8; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: ::core::ffi::c_int,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScopeDictDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_8 {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
pub type scid_T = ::core::ffi::c_int;
pub type u_header_T = u_header;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_header {
    pub uh_next: C2Rust_Unnamed_12,
    pub uh_prev: C2Rust_Unnamed_11,
    pub uh_alt_next: C2Rust_Unnamed_10,
    pub uh_alt_prev: C2Rust_Unnamed_9,
    pub uh_seq: ::core::ffi::c_int,
    pub uh_walk: ::core::ffi::c_int,
    pub uh_entry: *mut u_entry_T,
    pub uh_getbot_entry: *mut u_entry_T,
    pub uh_cursor: pos_T,
    pub uh_cursor_vcol: colnr_T,
    pub uh_flags: ::core::ffi::c_int,
    pub uh_namedm: [fmark_T; 26],
    pub uh_extmark: extmark_undo_vec_t,
    pub uh_visual: visualinfo_T,
    pub uh_time: time_t,
    pub uh_save_nr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visualinfo_T {
    pub vi_start: pos_T,
    pub vi_end: pos_T,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extmark_undo_vec_t {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExtmarkUndoObject,
}
pub type ExtmarkUndoObject = undo_object;
pub type u_entry_T = u_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_entry {
    pub ue_next: *mut u_entry_T,
    pub ue_top: linenr_T,
    pub ue_bot: linenr_T,
    pub ue_lcount: linenr_T,
    pub ue_array: *mut *mut ::core::ffi::c_char,
    pub ue_size: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
pub type mapblock_T = mapblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mapblock {
    pub m_next: *mut mapblock_T,
    pub m_alt: *mut mapblock_T,
    pub m_keys: *mut ::core::ffi::c_char,
    pub m_str: *mut ::core::ffi::c_char,
    pub m_orig_str: *mut ::core::ffi::c_char,
    pub m_luaref: LuaRef,
    pub m_keylen: ::core::ffi::c_int,
    pub m_mode: ::core::ffi::c_int,
    pub m_simplified: ::core::ffi::c_int,
    pub m_noremap: ::core::ffi::c_int,
    pub m_silent: ::core::ffi::c_char,
    pub m_nowait: ::core::ffi::c_char,
    pub m_expr: ::core::ffi::c_char,
    pub m_script_ctx: sctx_T,
    pub m_desc: *mut ::core::ffi::c_char,
    pub m_replace_keycodes: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub type WinInfo = wininfo_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wininfo_S {
    pub wi_win: *mut win_T,
    pub wi_mark: fmark_T,
    pub wi_optset: bool,
    pub wi_opt: winopt_T,
    pub wi_fold_manual: bool,
    pub wi_folds: garray_T,
    pub wi_changelistidx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winopt_T {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: *mut ::core::ffi::c_char,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: *mut ::core::ffi::c_char,
    pub wo_eiw: *mut ::core::ffi::c_char,
    pub wo_fdc_save: *mut ::core::ffi::c_char,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: *mut ::core::ffi::c_char,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: *mut ::core::ffi::c_char,
    pub wo_fdm_save: *mut ::core::ffi::c_char,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: *mut ::core::ffi::c_char,
    pub wo_fdt: *mut ::core::ffi::c_char,
    pub wo_fmr: *mut ::core::ffi::c_char,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: *mut ::core::ffi::c_char,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: *mut ::core::ffi::c_char,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: *mut ::core::ffi::c_char,
    pub wo_cc: *mut ::core::ffi::c_char,
    pub wo_sbr: *mut ::core::ffi::c_char,
    pub wo_stc: *mut ::core::ffi::c_char,
    pub wo_stl: *mut ::core::ffi::c_char,
    pub wo_wbr: *mut ::core::ffi::c_char,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: *mut ::core::ffi::c_char,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: *mut ::core::ffi::c_char,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: *mut ::core::ffi::c_char,
    pub wo_lcs: *mut ::core::ffi::c_char,
    pub wo_fcs: *mut ::core::ffi::c_char,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [sctx_T; 51],
}
pub type win_T = window_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_S {
    pub handle: handle_T,
    pub w_buffer: *mut buf_T,
    pub w_s: *mut synblock_T,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    pub w_ns_set: Set_uint32_t,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    pub w_hl_needs_update: ::core::ffi::c_int,
    pub w_prev: *mut win_T,
    pub w_next: *mut win_T,
    pub w_locked: bool,
    pub w_frame: *mut frame_T,
    pub w_cursor: pos_T,
    pub w_curswant: colnr_T,
    pub w_set_curswant: ::core::ffi::c_int,
    pub w_cursorline: linenr_T,
    pub w_last_cursorline: linenr_T,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: linenr_T,
    pub w_old_cursor_fcol: colnr_T,
    pub w_old_cursor_lcol: colnr_T,
    pub w_old_visual_lnum: linenr_T,
    pub w_old_visual_col: colnr_T,
    pub w_old_curswant: colnr_T,
    pub w_last_cursor_lnum_rnu: linenr_T,
    pub w_p_lcs_chars: lcs_chars_T,
    pub w_p_fcs_chars: fcs_chars_T,
    pub w_topline: linenr_T,
    pub w_topline_was_set: ::core::ffi::c_char,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: colnr_T,
    pub w_skipcol: colnr_T,
    pub w_last_topline: linenr_T,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: colnr_T,
    pub w_last_skipcol: colnr_T,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: pos_save_T,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: ::core::ffi::c_int,
    pub w_valid_cursor: pos_T,
    pub w_valid_leftcol: colnr_T,
    pub w_valid_skipcol: colnr_T,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: linenr_T,
    pub w_viewport_last_botline: linenr_T,
    pub w_viewport_last_topfill: linenr_T,
    pub w_viewport_last_skipcol: linenr_T,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: colnr_T,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: linenr_T,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut wline_T,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: garray_T,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: linenr_T,
    pub w_redraw_bot: linenr_T,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: disptick_T,
    pub w_stl_cursor: pos_T,
    pub w_stl_virtcol: colnr_T,
    pub w_stl_topline: linenr_T,
    pub w_stl_line_count: linenr_T,
    pub w_stl_topfill: ::core::ffi::c_int,
    pub w_stl_empty: ::core::ffi::c_char,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: pos_T,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut alist_T,
    pub w_arg_idx: ::core::ffi::c_int,
    pub w_arg_idx_invalid: ::core::ffi::c_int,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: winopt_T,
    pub w_allbuf_opt: winopt_T,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut dict_T,
    pub w_pcmark: pos_T,
    pub w_prev_pcmark: pos_T,
    pub w_jumplist: [xfmark_T; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut matchitem_T,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [taggy_T; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: linenr_T,
    pub w_statuscol_line_count: linenr_T,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut qf_info_T,
    pub w_llist_ref: *mut qf_info_T,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickDefinition {
    pub type_0: C2Rust_Unnamed_14,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_14 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_14 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_14 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_14 = 0;
pub type qf_info_T = qf_info_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinConfig {
    pub window: Window,
    pub bufpos: lpos_T,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub row: ::core::ffi::c_double,
    pub col: ::core::ffi::c_double,
    pub anchor: FloatAnchor,
    pub relative: FloatRelative,
    pub external: bool,
    pub focusable: bool,
    pub mouse: bool,
    pub split: WinSplit,
    pub zindex: ::core::ffi::c_int,
    pub style: WinStyle,
    pub border: bool,
    pub shadow: bool,
    pub border_chars: [[::core::ffi::c_char; 32]; 8],
    pub border_hl_ids: [::core::ffi::c_int; 8],
    pub border_attr: [::core::ffi::c_int; 8],
    pub title: bool,
    pub title_pos: AlignTextPos,
    pub title_chunks: VirtText,
    pub title_width: ::core::ffi::c_int,
    pub footer: bool,
    pub footer_pos: AlignTextPos,
    pub footer_chunks: VirtText,
    pub footer_width: ::core::ffi::c_int,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub _cmdline_offset: ::core::ffi::c_int,
}
pub type AlignTextPos = ::core::ffi::c_uint;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub type WinStyle = ::core::ffi::c_uint;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub type WinSplit = ::core::ffi::c_uint;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub type FloatRelative = ::core::ffi::c_uint;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type FloatAnchor = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenGrid {
    pub handle: handle_T,
    pub chars: *mut schar_T,
    pub attrs: *mut sattr_T,
    pub vcols: *mut colnr_T,
    pub line_offset: *mut size_t,
    pub dirty_col: *mut ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: ::core::ffi::c_int,
    pub comp_row: ::core::ffi::c_int,
    pub comp_col: ::core::ffi::c_int,
    pub comp_width: ::core::ffi::c_int,
    pub comp_height: ::core::ffi::c_int,
    pub comp_index: size_t,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridView {
    pub target: *mut ScreenGrid,
    pub row_offset: ::core::ffi::c_int,
    pub col_offset: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct taggy_T {
    pub tagname: *mut ::core::ffi::c_char,
    pub fmark: fmark_T,
    pub cur_match: ::core::ffi::c_int,
    pub cur_fnum: ::core::ffi::c_int,
    pub user_data: *mut ::core::ffi::c_char,
}
pub type matchitem_T = matchitem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchitem {
    pub mit_next: *mut matchitem_T,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: regmmatch_T,
    pub mit_pos_array: *mut llpos_T,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: linenr_T,
    pub mit_botlnum: linenr_T,
    pub mit_hl: match_T,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_T {
    pub rm: regmmatch_T,
    pub buf: *mut buf_T,
    pub lnum: linenr_T,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: linenr_T,
    pub startcol: colnr_T,
    pub endcol: colnr_T,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: proftime_T,
}
pub type buf_T = file_buffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmmatch_T {
    pub regprog: *mut regprog_T,
    pub startpos: [lpos_T; 10],
    pub endpos: [lpos_T; 10],
    pub rmm_matchcol: colnr_T,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct llpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct xfmark_T {
    pub fmark: fmark_T,
    pub fname: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wline_T {
    pub wl_lnum: linenr_T,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: linenr_T,
    pub wl_lastlnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_save_T {
    pub w_topline_save: ::core::ffi::c_int,
    pub w_topline_corr: ::core::ffi::c_int,
    pub w_cursor_save: pos_T,
    pub w_cursor_corr: pos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fcs_chars_T {
    pub stl: schar_T,
    pub stlnc: schar_T,
    pub wbr: schar_T,
    pub horiz: schar_T,
    pub horizup: schar_T,
    pub horizdown: schar_T,
    pub vert: schar_T,
    pub vertleft: schar_T,
    pub vertright: schar_T,
    pub verthoriz: schar_T,
    pub fold: schar_T,
    pub foldopen: schar_T,
    pub foldclosed: schar_T,
    pub foldsep: schar_T,
    pub foldinner: schar_T,
    pub diff: schar_T,
    pub msgsep: schar_T,
    pub eob: schar_T,
    pub lastline: schar_T,
    pub trunc: schar_T,
    pub truncrl: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lcs_chars_T {
    pub eol: schar_T,
    pub ext: schar_T,
    pub prec: schar_T,
    pub nbsp: schar_T,
    pub space: schar_T,
    pub tab1: schar_T,
    pub tab2: schar_T,
    pub tab3: schar_T,
    pub leadtab1: schar_T,
    pub leadtab2: schar_T,
    pub leadtab3: schar_T,
    pub lead: schar_T,
    pub trail: schar_T,
    pub multispace: *mut schar_T,
    pub leadmultispace: *mut schar_T,
    pub conceal: schar_T,
}
pub type frame_T = frame_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct frame_S {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut frame_T,
    pub fr_next: *mut frame_T,
    pub fr_prev: *mut frame_T,
    pub fr_child: *mut frame_T,
    pub fr_win: *mut win_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memline_T {
    pub ml_line_count: linenr_T,
    pub ml_mfp: *mut memfile_T,
    pub ml_stack: *mut infoptr_T,
    pub ml_stack_top: ::core::ffi::c_int,
    pub ml_stack_size: ::core::ffi::c_int,
    pub ml_flags: ::core::ffi::c_int,
    pub ml_line_textlen: colnr_T,
    pub ml_line_lnum: linenr_T,
    pub ml_line_ptr: *mut ::core::ffi::c_char,
    pub ml_line_offset: size_t,
    pub ml_line_offset_ff: ::core::ffi::c_int,
    pub ml_locked: *mut bhdr_T,
    pub ml_locked_low: linenr_T,
    pub ml_locked_high: linenr_T,
    pub ml_locked_lineadd: ::core::ffi::c_int,
    pub ml_chunksize: *mut chunksize_T,
    pub ml_numchunks: ::core::ffi::c_int,
    pub ml_usedchunks: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chunksize_T {
    pub mlcs_numlines: ::core::ffi::c_int,
    pub mlcs_totalsize: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bhdr_T {
    pub bh_bnum: blocknr_T,
    pub bh_data: *mut ::core::ffi::c_void,
    pub bh_page_count: ::core::ffi::c_uint,
    pub bh_flags: ::core::ffi::c_uint,
}
pub type blocknr_T = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct infoptr_T {
    pub ip_bnum: blocknr_T,
    pub ip_low: linenr_T,
    pub ip_high: linenr_T,
    pub ip_index: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memfile_T {
    pub mf_fname: *mut ::core::ffi::c_char,
    pub mf_ffname: *mut ::core::ffi::c_char,
    pub mf_fd: ::core::ffi::c_int,
    pub mf_flags: ::core::ffi::c_int,
    pub mf_reopen: bool,
    pub mf_free_first: *mut bhdr_T,
    pub mf_hash: Map_int64_t_ptr_t,
    pub mf_trans: Map_int64_t_int64_t,
    pub mf_blocknr_max: blocknr_T,
    pub mf_blocknr_min: blocknr_T,
    pub mf_neg_count: blocknr_T,
    pub mf_infile_count: blocknr_T,
    pub mf_page_size: ::core::ffi::c_uint,
    pub mf_dirty: mfdirty_T,
}
pub type mfdirty_T = ::core::ffi::c_uint;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_int64_t {
    pub set: Set_int64_t,
    pub values: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int64_t {
    pub h: MapHash,
    pub keys: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_ptr_t {
    pub set: Set_int64_t,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut String_0,
}
pub type OptionalKeys = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type FieldHashfn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict__shada_search_pat {
    pub is_set___shada_search_pat_: OptionalKeys,
    pub magic: Boolean,
    pub smartcase: Boolean,
    pub has_line_offset: Boolean,
    pub place_cursor_at_end: Boolean,
    pub is_last_used: Boolean,
    pub is_substitute_pattern: Boolean,
    pub highlighted: Boolean,
    pub search_backward: Boolean,
    pub offset: Integer,
    pub pat: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict__shada_mark {
    pub is_set___shada_mark_: OptionalKeys,
    pub n: Integer,
    pub l: Integer,
    pub c: Integer,
    pub f: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict__shada_register {
    pub is_set___shada_register_: OptionalKeys,
    pub rc: StringArray,
    pub ru: Boolean,
    pub rt: Integer,
    pub n: Integer,
    pub rw: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict__shada_buflist_item {
    pub is_set___shada_buflist_item_: OptionalKeys,
    pub l: Integer,
    pub c: Integer,
    pub f: String_0,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_15 = 2147483647;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const VAR_TYPE_BLOB: C2Rust_Unnamed_16 = 10;
pub const VAR_TYPE_SPECIAL: C2Rust_Unnamed_16 = 7;
pub const VAR_TYPE_BOOL: C2Rust_Unnamed_16 = 6;
pub const VAR_TYPE_FLOAT: C2Rust_Unnamed_16 = 5;
pub const VAR_TYPE_DICT: C2Rust_Unnamed_16 = 4;
pub const VAR_TYPE_LIST: C2Rust_Unnamed_16 = 3;
pub const VAR_TYPE_FUNC: C2Rust_Unnamed_16 = 2;
pub const VAR_TYPE_STRING: C2Rust_Unnamed_16 = 1;
pub const VAR_TYPE_NUMBER: C2Rust_Unnamed_16 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ht_stack_S {
    pub ht: *mut hashtab_T,
    pub prev: *mut ht_stack_S,
}
pub type ht_stack_T = ht_stack_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct list_stack_S {
    pub list: *mut list_T,
    pub prev: *mut list_stack_S,
}
pub type list_stack_T = list_stack_S;
pub type cstr_t = *const ::core::ffi::c_char;
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_ptr_t {
    pub h: MapHash,
    pub keys: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_cstr_t_ptr_t {
    pub set: Set_cstr_t,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
pub type MarkGet = ::core::ffi::c_uint;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffblock_S {
    pub df_next: *mut diff_T,
    pub df_lnum: [linenr_T; 8],
    pub df_count: [linenr_T; 8],
    pub is_linematched: bool,
    pub has_changes: bool,
    pub df_changes: garray_T,
}
pub type diff_T = diffblock_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tabpage_S {
    pub handle: handle_T,
    pub tp_next: *mut tabpage_T,
    pub tp_topframe: *mut frame_T,
    pub tp_curwin: *mut win_T,
    pub tp_prevwin: *mut win_T,
    pub tp_firstwin: *mut win_T,
    pub tp_lastwin: *mut win_T,
    pub tp_old_Rows_avail: int64_t,
    pub tp_old_Columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut diff_T,
    pub tp_diffbuf: [*mut buf_T; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut frame_T; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut dict_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
pub type tabpage_T = tabpage_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubReplacementString {
    pub sub: *mut ::core::ffi::c_char,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const HIST_DEBUG: C2Rust_Unnamed_17 = 4;
pub const HIST_INPUT: C2Rust_Unnamed_17 = 3;
pub const HIST_EXPR: C2Rust_Unnamed_17 = 2;
pub const HIST_SEARCH: C2Rust_Unnamed_17 = 1;
pub const HIST_CMD: C2Rust_Unnamed_17 = 0;
pub const HIST_INVALID: C2Rust_Unnamed_17 = -1;
pub const HIST_DEFAULT: C2Rust_Unnamed_17 = -2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const HIST_COUNT: C2Rust_Unnamed_18 = 5;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct histentry_T {
    pub hisnum: ::core::ffi::c_int,
    pub hisstr: *mut ::core::ffi::c_char,
    pub hisstrlen: size_t,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type VimVarIndex = ::core::ffi::c_uint;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct packer_buffer_t {
    pub startptr: *mut ::core::ffi::c_char,
    pub ptr: *mut ::core::ffi::c_char,
    pub endptr: *mut ::core::ffi::c_char,
    pub anydata: *mut ::core::ffi::c_void,
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer) -> ()>;
pub type PackerBuffer = packer_buffer_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
pub type var_flavour_T = ::core::ffi::c_uint;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalDataBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kFileMkDir: C2Rust_Unnamed_19 = 256;
pub const kFileNonBlocking: C2Rust_Unnamed_19 = 128;
pub const kFileAppend: C2Rust_Unnamed_19 = 64;
pub const kFileTruncate: C2Rust_Unnamed_19 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_19 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_19 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_19 = 4;
pub const kFileCreate: C2Rust_Unnamed_19 = 2;
pub const kFileReadOnly: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const NUM_REGISTERS: C2Rust_Unnamed_20 = 39;
pub const PLUS_REGISTER: C2Rust_Unnamed_20 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_20 = 37;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_20 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_20 = 36;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchOffset {
    pub dir: ::core::ffi::c_char,
    pub line: bool,
    pub end: bool,
    pub off: int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchPattern {
    pub pat: *mut ::core::ffi::c_char,
    pub patlen: size_t,
    pub magic: bool,
    pub no_scs: bool,
    pub timestamp: Timestamp,
    pub off: SearchOffset,
    pub additional_data: *mut AdditionalData,
}
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kShaDaMissingError: C2Rust_Unnamed_21 = 16;
pub const kShaDaGetOldfiles: C2Rust_Unnamed_21 = 8;
pub const kShaDaForceit: C2Rust_Unnamed_21 = 4;
pub const kShaDaWantMarks: C2Rust_Unnamed_21 = 2;
pub const kShaDaWantInfo: C2Rust_Unnamed_21 = 1;
pub const kSDWriteReadNotShada: ShaDaWriteResult = 1;
pub type ShaDaWriteResult = ::core::ffi::c_uint;
pub const kSDWriteIgnError: ShaDaWriteResult = 3;
pub const kSDWriteFailed: ShaDaWriteResult = 2;
pub const kSDWriteSuccessful: ShaDaWriteResult = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WriteMergerState {
    pub hms: [HistoryMergerState; 5],
    pub global_marks: [ShadaEntry; 26],
    pub numbered_marks: [ShadaEntry; 10],
    pub registers: [ShadaEntry; 37],
    pub jumps: [ShadaEntry; 100],
    pub jumps_size: size_t,
    pub search_pattern: ShadaEntry,
    pub sub_search_pattern: ShadaEntry,
    pub replacement: ShadaEntry,
    pub dumped_variables: Set_cstr_t,
    pub file_marks: Map_cstr_t_ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShadaEntry {
    pub type_0: ShadaEntryType,
    pub can_free_entry: bool,
    pub timestamp: Timestamp,
    pub data: C2Rust_Unnamed_22,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub header: Dict,
    pub filemark: shada_filemark,
    pub search_pattern: KeyDict__shada_search_pat,
    pub history_item: history_item,
    pub reg: reg,
    pub global_var: global_var,
    pub unknown_item: C2Rust_Unnamed_23,
    pub sub_string: sub_string,
    pub buffer_list: buffer_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffer_list {
    pub size: size_t,
    pub buffers: *mut buffer_list_buffer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffer_list_buffer {
    pub pos: pos_T,
    pub fname: *mut ::core::ffi::c_char,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sub_string {
    pub sub: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub type_0: uint64_t,
    pub contents: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct global_var {
    pub name: *mut ::core::ffi::c_char,
    pub value: typval_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg {
    pub name: ::core::ffi::c_char,
    pub type_0: MotionType,
    pub contents: *mut String_0,
    pub is_unnamed: bool,
    pub contents_size: size_t,
    pub width: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct history_item {
    pub histtype: uint8_t,
    pub string: *mut ::core::ffi::c_char,
    pub sep: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct shada_filemark {
    pub name: ::core::ffi::c_char,
    pub mark: pos_T,
    pub fname: *mut ::core::ffi::c_char,
}
pub type ShadaEntryType = ::core::ffi::c_int;
pub const kSDItemChange: ShadaEntryType = 11;
pub const kSDItemLocalMark: ShadaEntryType = 10;
pub const kSDItemBufferList: ShadaEntryType = 9;
pub const kSDItemJump: ShadaEntryType = 8;
pub const kSDItemGlobalMark: ShadaEntryType = 7;
pub const kSDItemVariable: ShadaEntryType = 6;
pub const kSDItemRegister: ShadaEntryType = 5;
pub const kSDItemHistoryEntry: ShadaEntryType = 4;
pub const kSDItemSubString: ShadaEntryType = 3;
pub const kSDItemSearchPattern: ShadaEntryType = 2;
pub const kSDItemHeader: ShadaEntryType = 1;
pub const kSDItemMissing: ShadaEntryType = 0;
pub const kSDItemUnknown: ShadaEntryType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HistoryMergerState {
    pub hmll: HMLList,
    pub do_merge: bool,
    pub reading: bool,
    pub iter: *const ::core::ffi::c_void,
    pub last_hist_entry: ShadaEntry,
    pub history_type: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HMLList {
    pub entries: *mut HMLListEntry,
    pub first: *mut HMLListEntry,
    pub last: *mut HMLListEntry,
    pub free_entry: *mut HMLListEntry,
    pub last_free_entry: *mut HMLListEntry,
    pub size: size_t,
    pub num_entries: size_t,
    pub contained_entries: Map_cstr_t_ptr_t,
}
pub type HMLListEntry = hm_llist_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hm_llist_entry {
    pub data: ShadaEntry,
    pub next: *mut hm_llist_entry,
    pub prev: *mut hm_llist_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileMarks {
    pub marks: [ShadaEntry; 29],
    pub changes: [ShadaEntry; 100],
    pub changes_size: size_t,
    pub additional_marks: *mut ShadaEntry,
    pub additional_marks_size: size_t,
    pub greatest_timestamp: Timestamp,
}
pub const kSDReadChanges: SRNIFlags = 2048;
pub const kSDReadLocalMarks: SRNIFlags = 1024;
pub const kSDReadGlobalMarks: SRNIFlags = 128;
pub const kSDReadVariables: SRNIFlags = 64;
pub const kSDReadRegisters: SRNIFlags = 32;
pub const kSDReadHistory: SRNIFlags = 16;
pub const kSDReadUnknown: SRNIFlags = 4096;
pub const kSDReadUndisableableData: SRNIFlags = 268;
pub const kSDReadStatusMalformed: ShaDaReadResult = 4;
pub const kSDReadStatusReadError: ShaDaReadResult = 2;
pub const kSDReadStatusNotShaDa: ShaDaReadResult = 3;
pub const kSDReadStatusFinished: ShaDaReadResult = 1;
pub const kSDReadStatusSuccess: ShaDaReadResult = 0;
pub type ShaDaReadResult = ::core::ffi::c_uint;
pub type SearchPatternGetter = Option<unsafe extern "C" fn(*mut SearchPattern) -> ()>;
pub const kSDReadBufferList: SRNIFlags = 512;
pub type SRNIFlags = ::core::ffi::c_uint;
pub const kSDReadHeader: SRNIFlags = 2;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const PTRDIFF_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn __bswap_64(mut __bsx: __uint64_t) -> __uint64_t {
    return ((__bsx as ::core::ffi::c_ulonglong & 0xff00000000000000 as ::core::ffi::c_ulonglong)
        >> 56 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000000000 as ::core::ffi::c_ulonglong)
            >> 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000000000 as ::core::ffi::c_ulonglong)
            >> 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00000000 as ::core::ffi::c_ulonglong)
            >> 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000 as ::core::ffi::c_ulonglong)
            << 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000 as ::core::ffi::c_ulonglong)
            << 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00 as ::core::ffi::c_ulonglong)
            << 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff as ::core::ffi::c_ulonglong)
            << 56 as ::core::ffi::c_int) as __uint64_t;
}
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const ROOT_UID: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: AdditionalDataBuilder = AdditionalDataBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_put_cstr_t(
    mut set: *mut Set_cstr_t,
    mut key: cstr_t,
    mut key_alloc: *mut *mut cstr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const KEYSET_OPTIDX__shada_search_pat__sp: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__c: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__f: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__l: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__rt: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__ru: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__rw: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__c: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__f: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__l: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MPACK_ITEM_SIZE: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn file_eof(fp: *const FileDescriptor) -> bool {
    return (*fp).eof as ::core::ffi::c_int != 0 && (*fp).read_pos == (*fp).write_pos;
}
#[inline(always)]
unsafe extern "C" fn file_fd(fp: *const FileDescriptor) -> ::core::ffi::c_int {
    return (*fp).fd;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn mark_global_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return if name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        name as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
    } else if ascii_isdigit(name as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        NMARKS + (name as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn mark_local_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return if name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        name as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
    } else if name as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        NMARKS
    } else if name as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
        NMARKS + 1 as ::core::ffi::c_int
    } else if name as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        NMARKS + 2 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh0 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh0 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh1 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh1 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh2 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh2 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh3 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh3 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh4 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh4 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh5 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh5 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_uint(mut buf: *mut *mut ::core::ffi::c_char, mut val: uint32_t) {
    if val > 0xffff as uint32_t {
        let c2rust_fresh6 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh6 = 0xce as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, val);
    } else if val > 0xff as uint32_t {
        let c2rust_fresh7 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh7 = 0xcd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, val);
    } else if val > 0x7f as uint32_t {
        let c2rust_fresh8 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh8 = 0xcc as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh9 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh9 = val as ::core::ffi::c_char;
    } else {
        let c2rust_fresh10 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh10 = val as ::core::ffi::c_char;
    };
}
#[inline]
unsafe extern "C" fn mpack_bool(mut buf: *mut *mut ::core::ffi::c_char, mut val: bool) {
    let c2rust_fresh11 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh11 = (0xc2 as ::core::ffi::c_int
        | (if val as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh12 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh12 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh13 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh13 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh14 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh14 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
#[inline]
unsafe extern "C" fn mpack_map(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh15 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh15 = (0x80 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh16 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh16 = 0xde as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh17 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh17 = 0xdf as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
#[inline]
unsafe extern "C" fn mpack_remaining(mut packer: *mut PackerBuffer) -> size_t {
    return (*packer).endptr.offset_from((*packer).ptr) as size_t;
}
#[inline]
unsafe extern "C" fn file_space(mut fp: *mut FileDescriptor) -> size_t {
    return (*fp)
        .buffer
        .offset(ARENA_BLOCK_SIZE as isize)
        .offset_from((*fp).write_pos) as size_t;
}
#[inline]
unsafe extern "C" fn op_reg_index(regname: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if ascii_isdigit(regname) {
        return regname - '0' as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname == '-' as ::core::ffi::c_int {
        return DELETION_REGISTER as ::core::ffi::c_int;
    } else if regname == '*' as ::core::ffi::c_int {
        return STAR_REGISTER as ::core::ffi::c_int;
    } else if regname == '+' as ::core::ffi::c_int {
        return PLUS_REGISTER as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}
pub const DEFAULT_POS: pos_T = pos_T {
    lnum: 1 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
};
static default_pos: GlobalCell<pos_T> = GlobalCell::new(DEFAULT_POS);
static sd_default_values: GlobalCell<[ShadaEntry; 12]> = GlobalCell::new([
    ShadaEntry {
        type_0: kSDItemMissing,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemHeader,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0 as size_t,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemSearchPattern,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            search_pattern: KeyDict__shada_search_pat {
                is_set___shada_search_pat_: 0,
                magic: true,
                smartcase: false,
                has_line_offset: false,
                place_cursor_at_end: false,
                is_last_used: true,
                is_substitute_pattern: false,
                highlighted: false,
                search_backward: false,
                offset: 0 as Integer,
                pat: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                },
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemSubString,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            sub_string: sub_string {
                sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemHistoryEntry,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            history_item: history_item {
                histtype: HIST_CMD as ::core::ffi::c_int as uint8_t,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                sep: '\0' as ::core::ffi::c_char,
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemRegister,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            reg: reg {
                name: '\0' as ::core::ffi::c_char,
                type_0: kMTCharWise,
                contents: ::core::ptr::null_mut::<String_0>(),
                is_unnamed: false,
                contents_size: 0 as size_t,
                width: 0 as size_t,
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemVariable,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            global_var: global_var {
                name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                value: typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    },
                },
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemGlobalMark,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '"' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemJump,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '\0' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemBufferList,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            buffer_list: buffer_list {
                size: 0 as size_t,
                buffers: ::core::ptr::null_mut::<buffer_list_buffer>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemLocalMark,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '"' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemChange,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '\0' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
]);
#[inline]
unsafe extern "C" fn hmll_init(hmll: *mut HMLList, size: size_t) {
    *hmll = HMLList {
        entries: xcalloc(size, ::core::mem::size_of::<HMLListEntry>()) as *mut HMLListEntry,
        first: ::core::ptr::null_mut::<HMLListEntry>(),
        last: ::core::ptr::null_mut::<HMLListEntry>(),
        free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
        last_free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
        size: size,
        num_entries: 0 as size_t,
        contained_entries: MAP_INIT,
    };
    (*hmll).last_free_entry = (*hmll).entries;
}
#[inline]
unsafe extern "C" fn hmll_remove(hmll: *mut HMLList, hmll_entry: *mut HMLListEntry) {
    if hmll_entry
        == (*hmll)
            .last_free_entry
            .offset(-(1 as ::core::ffi::c_int as isize))
    {
        (*hmll).last_free_entry = (*hmll).last_free_entry.offset(-1);
    } else {
        '_c2rust_label: {
            if (*hmll).free_entry.is_null() {
            } else {
                __assert_fail(
                    b"hmll->free_entry == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    449 as ::core::ffi::c_uint,
                    b"void hmll_remove(HMLList *const, HMLListEntry *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*hmll).free_entry = hmll_entry;
    }
    let mut val: ptr_t = map_del_cstr_t_ptr_t(
        &raw mut (*hmll).contained_entries,
        (*hmll_entry).data.data.history_item.string as cstr_t,
        ::core::ptr::null_mut::<cstr_t>(),
    );
    '_c2rust_label_0: {
        if !val.is_null() {
        } else {
            __assert_fail(
                b"val\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                454 as ::core::ffi::c_uint,
                b"void hmll_remove(HMLList *const, HMLListEntry *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*hmll_entry).next.is_null() {
        (*hmll).last = (*hmll_entry).prev as *mut HMLListEntry;
    } else {
        (*(*hmll_entry).next).prev = (*hmll_entry).prev;
    }
    if (*hmll_entry).prev.is_null() {
        (*hmll).first = (*hmll_entry).next as *mut HMLListEntry;
    } else {
        (*(*hmll_entry).prev).next = (*hmll_entry).next;
    }
    (*hmll).num_entries = (*hmll).num_entries.wrapping_sub(1);
    shada_free_shada_entry(&raw mut (*hmll_entry).data);
}
#[inline]
unsafe extern "C" fn hmll_insert(
    hmll: *mut HMLList,
    mut hmll_entry: *mut HMLListEntry,
    data: ShadaEntry,
) {
    if (*hmll).num_entries == (*hmll).size {
        if hmll_entry == (*hmll).first {
            hmll_entry = ::core::ptr::null_mut::<HMLListEntry>();
        }
        '_c2rust_label: {
            if !(*hmll).first.is_null() {
            } else {
                __assert_fail(
                    b"hmll->first != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    484 as ::core::ffi::c_uint,
                    b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        hmll_remove(hmll, (*hmll).first);
    }
    let mut target_entry: *mut HMLListEntry = ::core::ptr::null_mut::<HMLListEntry>();
    if (*hmll).free_entry.is_null() {
        '_c2rust_label_0: {
            if (*hmll).last_free_entry.offset_from((*hmll).entries) as size_t == (*hmll).num_entries
            {
            } else {
                __assert_fail(
                    b"(size_t)(hmll->last_free_entry - hmll->entries) == hmll->num_entries\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    490 as ::core::ffi::c_uint,
                    b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let c2rust_fresh20 = (*hmll).last_free_entry;
        (*hmll).last_free_entry = (*hmll).last_free_entry.offset(1);
        target_entry = c2rust_fresh20;
    } else {
        '_c2rust_label_1: {
            if ((*hmll).last_free_entry.offset_from((*hmll).entries) as size_t)
                .wrapping_sub(1 as size_t)
                == (*hmll).num_entries
            {
            } else {
                __assert_fail(
                    b"(size_t)(hmll->last_free_entry - hmll->entries) - 1 == hmll->num_entries\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    494 as ::core::ffi::c_uint,
                    b"void hmll_insert(HMLList *const, HMLListEntry *, const ShadaEntry)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        target_entry = (*hmll).free_entry;
        (*hmll).free_entry = ::core::ptr::null_mut::<HMLListEntry>();
    }
    (*target_entry).data = data;
    let mut new_item: bool = false_0 != 0;
    let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
        &raw mut (*hmll).contained_entries,
        data.data.history_item.string as cstr_t,
        ::core::ptr::null_mut::<*mut cstr_t>(),
        &raw mut new_item,
    );
    if new_item {
        *val = target_entry as ptr_t;
    }
    (*hmll).num_entries = (*hmll).num_entries.wrapping_add(1);
    (*target_entry).prev = hmll_entry as *mut hm_llist_entry;
    if hmll_entry.is_null() {
        (*target_entry).next = (*hmll).first as *mut hm_llist_entry;
        (*hmll).first = target_entry;
    } else {
        (*target_entry).next = (*hmll_entry).next;
        (*hmll_entry).next = target_entry as *mut hm_llist_entry;
    }
    if (*target_entry).next.is_null() {
        (*hmll).last = target_entry;
    } else {
        (*(*target_entry).next).prev = target_entry as *mut hm_llist_entry;
    };
}
#[inline]
unsafe extern "C" fn hmll_dealloc(hmll: *mut HMLList) {
    xfree((*hmll).contained_entries.set.keys as *mut ::core::ffi::c_void);
    xfree((*hmll).contained_entries.set.h.hash as *mut ::core::ffi::c_void);
    (*hmll).contained_entries.set = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*hmll).contained_entries.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree((*hmll).entries as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn sd_reader_skip(
    sd_reader: *mut FileDescriptor,
    offset: size_t,
) -> ShaDaReadResult {
    let skip_bytes: ptrdiff_t = file_skip(sd_reader, offset);
    if skip_bytes < 0 as ptrdiff_t {
        semsg(
            gettext(
                b"E886: System error while skipping in ShaDa file: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_strerror(skip_bytes as ::core::ffi::c_int),
        );
        return kSDReadStatusReadError;
    } else if skip_bytes != offset as ptrdiff_t {
        '_c2rust_label: {
            if skip_bytes < offset as ptrdiff_t {
            } else {
                __assert_fail(
                    b"skip_bytes < (ptrdiff_t)offset\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    548 as ::core::ffi::c_uint,
                    b"ShaDaReadResult sd_reader_skip(FileDescriptor *const, const size_t)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if file_eof(sd_reader) {
            semsg(
                gettext(
                    b"E576: Reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                offset as uint64_t,
            );
        } else {
            semsg(
                gettext(
                    b"E886: System error while skipping in ShaDa file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                gettext(b"too few bytes read\0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
        return kSDReadStatusNotShaDa;
    }
    return kSDReadStatusSuccess;
}
unsafe extern "C" fn close_file(mut cookie: *mut FileDescriptor) {
    let error: ::core::ffi::c_int = file_close(cookie, p_fs.get() != 0);
    if error != 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E886: System error while closing ShaDa file: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_strerror(error),
        );
    }
}
unsafe extern "C" fn shada_read_file(
    file: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let fname: *mut ::core::ffi::c_char = shada_filename(file);
    if fname.is_null() {
        return FAIL;
    }
    let mut sd_reader: FileDescriptor = FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    };
    let mut of_ret: ::core::ffi::c_int = file_open(
        &raw mut sd_reader,
        fname,
        kFileReadOnly as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if p_verbose.get() > 1 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Reading ShaDa file \"%s\"%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char),
            fname,
            if flags & kShaDaWantInfo as ::core::ffi::c_int != 0 {
                gettext(b" info\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if flags & kShaDaWantMarks as ::core::ffi::c_int != 0 {
                gettext(b" marks\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if flags & kShaDaGetOldfiles as ::core::ffi::c_int != 0 {
                gettext(b" oldfiles\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if of_ret != 0 as ::core::ffi::c_int {
                gettext(b" FAILED\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        verbose_leave();
    }
    if of_ret != 0 as ::core::ffi::c_int {
        if of_ret != UV_ENOENT as ::core::ffi::c_int
            || flags & kShaDaMissingError as ::core::ffi::c_int != 0
        {
            semsg(
                gettext(
                    b"E886: System error while opening ShaDa file %s for reading: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                fname,
                uv_strerror(of_ret),
            );
        }
        xfree(fname as *mut ::core::ffi::c_void);
        return FAIL;
    }
    xfree(fname as *mut ::core::ffi::c_void);
    shada_read(&raw mut sd_reader, flags);
    close_file(&raw mut sd_reader);
    return OK;
}
unsafe extern "C" fn shada_hist_iter(
    iter: *const ::core::ffi::c_void,
    history_type: uint8_t,
    zero: bool,
    hist: *mut ShadaEntry,
) -> *const ::core::ffi::c_void {
    let mut hist_he: histentry_T = histentry_T {
        hisnum: 0,
        hisstr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        hisstrlen: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let ret: *const ::core::ffi::c_void = hist_iter(iter, history_type, zero, &raw mut hist_he);
    if hist_he.hisstr.is_null() {
        *hist = ShadaEntry {
            type_0: kSDItemMissing,
            can_free_entry: false,
            timestamp: 0,
            data: C2Rust_Unnamed_22 {
                header: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
    } else {
        *hist = ShadaEntry {
            type_0: kSDItemHistoryEntry,
            can_free_entry: zero,
            timestamp: hist_he.timestamp,
            data: C2Rust_Unnamed_22 {
                history_item: history_item {
                    histtype: history_type,
                    string: hist_he.hisstr,
                    sep: (if history_type as ::core::ffi::c_int == HIST_SEARCH as ::core::ffi::c_int
                    {
                        *hist_he
                            .hisstr
                            .offset(hist_he.hisstrlen.wrapping_add(1 as size_t) as isize)
                            as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as ::core::ffi::c_char,
                },
            },
            additional_data: hist_he.additional_data,
        };
    }
    return ret;
}
unsafe extern "C" fn hms_insert(hms_p: *mut HistoryMergerState, entry: ShadaEntry, do_iter: bool) {
    if do_iter {
        while (*hms_p).last_hist_entry.type_0 as ::core::ffi::c_int
            != kSDItemMissing as ::core::ffi::c_int
            && (*hms_p).last_hist_entry.timestamp < entry.timestamp
        {
            hms_insert(hms_p, (*hms_p).last_hist_entry, false_0 != 0);
            if (*hms_p).iter.is_null() {
                (*hms_p).last_hist_entry.type_0 = kSDItemMissing;
                break;
            } else {
                (*hms_p).iter = shada_hist_iter(
                    (*hms_p).iter,
                    (*hms_p).history_type,
                    (*hms_p).reading,
                    &raw mut (*hms_p).last_hist_entry,
                );
            }
        }
    }
    let hmll: *mut HMLList = &raw mut (*hms_p).hmll;
    let mut key_alloc: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
    let mut val: *mut ptr_t = map_ref_cstr_t_ptr_t(
        &raw mut (*hms_p).hmll.contained_entries,
        entry.data.history_item.string as cstr_t,
        &raw mut key_alloc,
    );
    if !val.is_null() {
        let existing_entry: *mut HMLListEntry = *val as *mut HMLListEntry;
        if entry.timestamp > (*existing_entry).data.timestamp {
            hmll_remove(hmll, existing_entry);
        } else if !do_iter && entry.timestamp == (*existing_entry).data.timestamp {
            shada_free_shada_entry(&raw mut (*existing_entry).data);
            (*existing_entry).data = entry;
            *key_alloc = entry.data.history_item.string as cstr_t;
            return;
        } else {
            return;
        }
    }
    let mut insert_after: *mut HMLListEntry = ::core::ptr::null_mut::<HMLListEntry>();
    insert_after = (*hmll).last;
    while !insert_after.is_null() {
        if (*insert_after).data.timestamp <= entry.timestamp {
            break;
        }
        insert_after = (*insert_after).prev as *mut HMLListEntry;
    }
    hmll_insert(hmll, insert_after, entry);
}
#[inline]
unsafe extern "C" fn hms_init(
    hms_p: *mut HistoryMergerState,
    history_type: uint8_t,
    num_elements: size_t,
    do_merge: bool,
    reading: bool,
) {
    hmll_init(&raw mut (*hms_p).hmll, num_elements);
    (*hms_p).do_merge = do_merge;
    (*hms_p).reading = reading;
    (*hms_p).iter = shada_hist_iter(
        ::core::ptr::null::<::core::ffi::c_void>(),
        history_type,
        (*hms_p).reading,
        &raw mut (*hms_p).last_hist_entry,
    );
    (*hms_p).history_type = history_type;
}
#[inline]
unsafe extern "C" fn hms_insert_whole_neovim_history(hms_p: *mut HistoryMergerState) {
    while (*hms_p).last_hist_entry.type_0 as ::core::ffi::c_int
        != kSDItemMissing as ::core::ffi::c_int
    {
        hms_insert(hms_p, (*hms_p).last_hist_entry, false_0 != 0);
        if (*hms_p).iter.is_null() {
            break;
        }
        (*hms_p).iter = shada_hist_iter(
            (*hms_p).iter,
            (*hms_p).history_type,
            (*hms_p).reading,
            &raw mut (*hms_p).last_hist_entry,
        );
    }
}
#[inline]
unsafe extern "C" fn hms_to_he_array(
    hms_p: *const HistoryMergerState,
    hist_array: *mut histentry_T,
    new_hisidx: *mut ::core::ffi::c_int,
    new_hisnum: *mut ::core::ffi::c_int,
) {
    let mut hist: *mut histentry_T = hist_array;
    let mut cur_entry: *mut HMLListEntry = (*hms_p).hmll.first as *mut HMLListEntry;
    while !cur_entry.is_null() {
        (*hist).timestamp = (*cur_entry).data.timestamp;
        (*hist).hisnum =
            hist.offset_from(hist_array) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        (*hist).hisstr = (*cur_entry).data.data.history_item.string;
        (*hist).hisstrlen = strlen((*cur_entry).data.data.history_item.string);
        (*hist).additional_data = (*cur_entry).data.additional_data;
        hist = hist.offset(1);
        cur_entry = (*cur_entry).next as *mut HMLListEntry;
    }
    *new_hisnum = hist.offset_from(hist_array) as ::core::ffi::c_int;
    *new_hisidx = *new_hisnum - 1 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn hms_dealloc(hms_p: *mut HistoryMergerState) {
    hmll_dealloc(&raw mut (*hms_p).hmll);
}
unsafe extern "C" fn var_shada_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut flavour: var_flavour_T,
) -> *const ::core::ffi::c_void {
    let mut hi: *const hashitem_T = ::core::ptr::null::<hashitem_T>();
    let mut globvarht: *mut hashtab_T = get_globvar_ht();
    let mut hifirst: *const hashitem_T = (*globvarht).ht_array;
    let hinum: size_t = (*globvarht).ht_mask.wrapping_add(1 as size_t);
    *name = ::core::ptr::null::<::core::ffi::c_char>();
    if iter.is_null() {
        hi = (*globvarht).ht_array;
        while (hi.offset_from(hifirst) as size_t) < hinum
            && ((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
                || var_flavour((*hi).hi_key) as ::core::ffi::c_uint
                    & flavour as ::core::ffi::c_uint
                    == 0)
        {
            hi = hi.offset(1);
        }
        if hi.offset_from(hifirst) as size_t == hinum {
            return ::core::ptr::null::<::core::ffi::c_void>();
        }
    } else {
        hi = iter as *const hashitem_T;
    }
    *name = &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
        as *mut dictitem_T))
        .di_key as *mut ::core::ffi::c_char;
    tv_copy(
        &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
            as *mut dictitem_T))
            .di_tv,
        rettv,
    );
    loop {
        hi = hi.offset(1);
        if (hi.offset_from(hifirst) as size_t) >= hinum {
            break;
        }
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            && var_flavour((*hi).hi_key) as ::core::ffi::c_uint & flavour as ::core::ffi::c_uint
                != 0
        {
            return hi as *const ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_void>();
}
unsafe extern "C" fn find_buffer(
    fname_bufs: *mut Map_cstr_t_ptr_t,
    fname: *const ::core::ffi::c_char,
) -> *mut buf_T {
    let mut key_alloc: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
    let mut new_item: bool = false_0 != 0;
    let mut ref_0: *mut *mut buf_T = map_put_ref_cstr_t_ptr_t(
        fname_bufs,
        fname as cstr_t,
        &raw mut key_alloc,
        &raw mut new_item,
    ) as *mut *mut buf_T;
    if new_item {
        *key_alloc = xstrdup(fname) as cstr_t;
    } else {
        return *ref_0;
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).b_ffname.is_null() {
            if path_fnamecmp(fname, (*buf).b_ffname) == 0 as ::core::ffi::c_int {
                *ref_0 = buf;
                return buf;
            }
        }
        buf = (*buf).b_next;
    }
    *ref_0 = ::core::ptr::null_mut::<buf_T>();
    return ::core::ptr::null_mut::<buf_T>();
}
#[inline]
unsafe extern "C" fn marks_equal(a: pos_T, b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col;
}
unsafe extern "C" fn marklist_insert(
    mut jumps_arr: *mut ::core::ffi::c_void,
    mut jump_size: size_t,
    mut jl_len: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut jumps: *mut ::core::ffi::c_char = jumps_arr as *mut ::core::ffi::c_char;
    if i > 0 as ::core::ffi::c_int {
        if jl_len == JUMPLISTSIZE {
            i -= 1;
            if i > 0 as ::core::ffi::c_int {
                memmove(
                    jumps as *mut ::core::ffi::c_void,
                    jumps.offset(jump_size as isize) as *const ::core::ffi::c_void,
                    jump_size.wrapping_mul(i as size_t),
                );
            }
        } else if i != jl_len {
            memmove(
                jumps.offset(
                    ((i + 1 as ::core::ffi::c_int) as size_t).wrapping_mul(jump_size) as isize,
                ) as *mut ::core::ffi::c_void,
                jumps.offset((i as size_t).wrapping_mul(jump_size) as isize)
                    as *const ::core::ffi::c_void,
                jump_size.wrapping_mul((jl_len - i) as size_t),
            );
        }
    } else if i == 0 as ::core::ffi::c_int {
        if jl_len == JUMPLISTSIZE {
            return -1 as ::core::ffi::c_int;
        } else if jl_len > 0 as ::core::ffi::c_int {
            memmove(
                jumps.offset(jump_size as isize) as *mut ::core::ffi::c_void,
                jumps as *const ::core::ffi::c_void,
                jump_size.wrapping_mul(jl_len as size_t),
            );
        }
    }
    return i;
}
unsafe extern "C" fn shada_read(sd_reader: *mut FileDescriptor, flags: ::core::ffi::c_int) {
    let mut oldfiles_list: *mut list_T = get_vim_var_list(VV_OLDFILES);
    let force: bool = flags & kShaDaForceit as ::core::ffi::c_int != 0;
    let get_old_files: bool = flags
        & (kShaDaGetOldfiles as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int)
        != 0
        && (force as ::core::ffi::c_int != 0
            || tv_list_len(oldfiles_list) == 0 as ::core::ffi::c_int);
    let want_marks: bool = flags & kShaDaWantMarks as ::core::ffi::c_int != 0;
    let srni_flags: ::core::ffi::c_uint = ((if flags & kShaDaWantInfo as ::core::ffi::c_int != 0 {
        kSDReadUndisableableData as ::core::ffi::c_int
            | kSDReadRegisters as ::core::ffi::c_int
            | kSDReadGlobalMarks as ::core::ffi::c_int
            | (if p_hi.get() != 0 {
                kSDReadHistory as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if !find_shada_parameter('!' as ::core::ffi::c_int).is_null() {
                kSDReadVariables as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if !find_shada_parameter('%' as ::core::ffi::c_int).is_null()
                && (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int
            {
                kSDReadBufferList as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
    } else {
        0 as ::core::ffi::c_int
    }) | (if want_marks as ::core::ffi::c_int != 0
        && get_shada_parameter('\'' as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
    {
        kSDReadLocalMarks as ::core::ffi::c_int | kSDReadChanges as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) | (if get_old_files as ::core::ffi::c_int != 0 {
        kSDReadLocalMarks as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    })) as ::core::ffi::c_uint;
    if srni_flags == 0 as ::core::ffi::c_uint {
        return;
    }
    let mut hms: [HistoryMergerState; 5] = [HistoryMergerState {
        hmll: HMLList {
            entries: ::core::ptr::null_mut::<HMLListEntry>(),
            first: ::core::ptr::null_mut::<HMLListEntry>(),
            last: ::core::ptr::null_mut::<HMLListEntry>(),
            free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
            last_free_entry: ::core::ptr::null_mut::<HMLListEntry>(),
            size: 0,
            num_entries: 0,
            contained_entries: Map_cstr_t_ptr_t {
                set: Set_cstr_t {
                    h: MapHash {
                        n_buckets: 0,
                        size: 0,
                        n_occupied: 0,
                        upper_bound: 0,
                        n_keys: 0,
                        keys_capacity: 0,
                        hash: ::core::ptr::null_mut::<uint32_t>(),
                    },
                    keys: ::core::ptr::null_mut::<cstr_t>(),
                },
                values: ::core::ptr::null_mut::<ptr_t>(),
            },
        },
        do_merge: false,
        reading: false,
        iter: ::core::ptr::null::<::core::ffi::c_void>(),
        last_hist_entry: ShadaEntry {
            type_0: kSDItemMissing,
            can_free_entry: false,
            timestamp: 0,
            data: C2Rust_Unnamed_22 {
                header: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        history_type: 0,
    }; 5];
    if srni_flags & kSDReadHistory as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < HIST_COUNT as ::core::ffi::c_int {
            hms_init(
                (&raw mut hms as *mut HistoryMergerState).offset(i as isize),
                i as uint8_t,
                p_hi.get() as size_t,
                true_0 != 0,
                true_0 != 0,
            );
            i += 1;
        }
    }
    let mut cur_entry: ShadaEntry = ShadaEntry {
        type_0: kSDItemMissing,
        can_free_entry: false,
        timestamp: 0,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut cl_bufs: Set_ptr_t = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    let mut fname_bufs: Map_cstr_t_ptr_t = MAP_INIT;
    let mut oldfiles_set: Set_cstr_t = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
    if get_old_files as ::core::ffi::c_int != 0
        && (oldfiles_list.is_null() || force as ::core::ffi::c_int != 0)
    {
        oldfiles_list = tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        set_vim_var_list(VV_OLDFILES, oldfiles_list);
    }
    let mut srni_ret: ShaDaReadResult = kSDReadStatusSuccess;
    loop {
        srni_ret = shada_read_next_item(sd_reader, &raw mut cur_entry, srni_flags, 0 as size_t);
        if srni_ret as ::core::ffi::c_uint
            == kSDReadStatusFinished as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        match srni_ret as ::core::ffi::c_uint {
            1 => {
                abort();
            }
            3 | 2 => {
                break;
            }
            4 => {}
            0 | _ => {
                let mut spat: SearchPattern = SearchPattern {
                    pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    patlen: 0,
                    magic: false,
                    no_scs: false,
                    timestamp: 0,
                    off: SearchOffset {
                        dir: 0,
                        line: false,
                        end: false,
                        off: 0,
                    },
                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                };
                's_732: {
                    match cur_entry.type_0 as ::core::ffi::c_int {
                        0 => {
                            abort();
                        }
                        1 => {
                            shada_free_shada_entry(&raw mut cur_entry);
                        }
                        2 => {
                            if !force {
                                let mut pat: SearchPattern = SearchPattern {
                                    pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    patlen: 0,
                                    magic: false,
                                    no_scs: false,
                                    timestamp: 0,
                                    off: SearchOffset {
                                        dir: 0,
                                        line: false,
                                        end: false,
                                        off: 0,
                                    },
                                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                };
                                if cur_entry.data.search_pattern.is_substitute_pattern {
                                    get_substitute_pattern(&raw mut pat);
                                } else {
                                    get_search_pattern(&raw mut pat);
                                }
                                if !pat.pat.is_null() && pat.timestamp >= cur_entry.timestamp {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                    break 's_732;
                                }
                            }
                            spat = SearchPattern {
                                pat: cur_entry.data.search_pattern.pat.data,
                                patlen: cur_entry.data.search_pattern.pat.size,
                                magic: cur_entry.data.search_pattern.magic as bool,
                                no_scs: !cur_entry.data.search_pattern.smartcase,
                                timestamp: cur_entry.timestamp,
                                off: SearchOffset {
                                    dir: (if cur_entry.data.search_pattern.search_backward
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        '?' as ::core::ffi::c_int
                                    } else {
                                        '/' as ::core::ffi::c_int
                                    })
                                        as ::core::ffi::c_char,
                                    line: cur_entry.data.search_pattern.has_line_offset as bool,
                                    end: cur_entry.data.search_pattern.place_cursor_at_end as bool,
                                    off: cur_entry.data.search_pattern.offset as int64_t,
                                },
                                additional_data: cur_entry.additional_data,
                            };
                            if cur_entry.data.search_pattern.is_substitute_pattern {
                                set_substitute_pattern(spat);
                            } else {
                                set_search_pattern(spat);
                            }
                            if cur_entry.data.search_pattern.is_last_used {
                                set_last_used_pattern(
                                    cur_entry.data.search_pattern.is_substitute_pattern as bool,
                                );
                                set_no_hlsearch(!cur_entry.data.search_pattern.highlighted);
                            }
                        }
                        3 => {
                            if !force {
                                let mut sub: SubReplacementString = SubReplacementString {
                                    sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    timestamp: 0,
                                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                };
                                sub_get_replacement(&raw mut sub);
                                if !sub.sub.is_null() && sub.timestamp >= cur_entry.timestamp {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                    break 's_732;
                                }
                            }
                            sub_set_replacement(SubReplacementString {
                                sub: cur_entry.data.sub_string.sub,
                                timestamp: cur_entry.timestamp,
                                additional_data: cur_entry.additional_data,
                            });
                            regtilde(
                                cur_entry.data.sub_string.sub,
                                magic_isset() as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                        }
                        4 => {
                            if cur_entry.data.history_item.histtype as ::core::ffi::c_int
                                >= HIST_COUNT as ::core::ffi::c_int
                            {
                                shada_free_shada_entry(&raw mut cur_entry);
                            } else {
                                hms_insert(
                                    (&raw mut hms as *mut HistoryMergerState).offset(
                                        cur_entry.data.history_item.histtype as ::core::ffi::c_int
                                            as isize,
                                    ),
                                    cur_entry,
                                    true_0 != 0,
                                );
                            }
                        }
                        5 => {
                            if cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                != kMTCharWise as ::core::ffi::c_int
                                && cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                    != kMTLineWise as ::core::ffi::c_int
                                && cur_entry.data.reg.type_0 as ::core::ffi::c_int
                                    != kMTBlockWise as ::core::ffi::c_int
                            {
                                shada_free_shada_entry(&raw mut cur_entry);
                            } else {
                                if !force {
                                    let reg: *const yankreg_T = op_reg_get(cur_entry.data.reg.name);
                                    if reg.is_null() || (*reg).timestamp >= cur_entry.timestamp {
                                        shada_free_shada_entry(&raw mut cur_entry);
                                        break 's_732;
                                    }
                                }
                                if !op_reg_set(
                                    cur_entry.data.reg.name,
                                    yankreg_T {
                                        y_array: cur_entry.data.reg.contents,
                                        y_size: cur_entry.data.reg.contents_size,
                                        y_type: cur_entry.data.reg.type_0,
                                        y_width: cur_entry.data.reg.width as colnr_T,
                                        timestamp: cur_entry.timestamp,
                                        additional_data: cur_entry.additional_data,
                                    },
                                    cur_entry.data.reg.is_unnamed,
                                ) {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                }
                            }
                        }
                        6 => {
                            var_set_global(
                                cur_entry.data.global_var.name,
                                cur_entry.data.global_var.value,
                            );
                            cur_entry.data.global_var.value.v_type = VAR_UNKNOWN;
                            shada_free_shada_entry(&raw mut cur_entry);
                        }
                        8 | 7 => {
                            let mut buf: *mut buf_T =
                                find_buffer(&raw mut fname_bufs, cur_entry.data.filemark.fname);
                            if !buf.is_null() {
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut cur_entry.data.filemark.fname
                                        as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL_0;
                                *ptr_;
                            }
                            let mut fm: xfmark_T = xfmark_T {
                                fmark: fmark_T {
                                    mark: cur_entry.data.filemark.mark,
                                    fnum: if buf.is_null() {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        (*buf).handle as ::core::ffi::c_int
                                    },
                                    timestamp: cur_entry.timestamp,
                                    view: fmarkv_T {
                                        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                                        skipcol: 0 as colnr_T,
                                    },
                                    additional_data: cur_entry.additional_data,
                                },
                                fname: if buf.is_null() {
                                    cur_entry.data.filemark.fname
                                } else {
                                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                                },
                            };
                            if cur_entry.type_0 as ::core::ffi::c_int
                                == kSDItemGlobalMark as ::core::ffi::c_int
                            {
                                if !mark_set_global(cur_entry.data.filemark.name, fm, !force) {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                }
                            } else {
                                let mut i_0: ::core::ffi::c_int = 0;
                                i_0 = (*curwin.get()).w_jumplistlen;
                                while i_0 > 0 as ::core::ffi::c_int {
                                    let jl_entry: xfmark_T = (*curwin.get()).w_jumplist
                                        [(i_0 - 1 as ::core::ffi::c_int) as usize];
                                    if jl_entry.fmark.timestamp <= cur_entry.timestamp {
                                        if marks_equal(
                                            jl_entry.fmark.mark,
                                            cur_entry.data.filemark.mark,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                            && (if buf.is_null() {
                                                (!jl_entry.fname.is_null()
                                                    && strcmp(fm.fname, jl_entry.fname)
                                                        == 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            } else {
                                                (fm.fmark.fnum == jl_entry.fmark.fnum)
                                                    as ::core::ffi::c_int
                                            }) != 0
                                        {
                                            i_0 = -1 as ::core::ffi::c_int;
                                        }
                                        break;
                                    } else {
                                        i_0 -= 1;
                                    }
                                }
                                if i_0 > 0 as ::core::ffi::c_int
                                    && (*curwin.get()).w_jumplistlen == JUMPLISTSIZE
                                {
                                    free_xfmark(
                                        (*curwin.get()).w_jumplist
                                            [0 as ::core::ffi::c_int as usize],
                                    );
                                }
                                i_0 = marklist_insert(
                                    &raw mut (*curwin.get()).w_jumplist as *mut xfmark_T
                                        as *mut ::core::ffi::c_void,
                                    ::core::mem::size_of::<xfmark_T>(),
                                    (*curwin.get()).w_jumplistlen,
                                    i_0,
                                );
                                if i_0 != -1 as ::core::ffi::c_int {
                                    (*curwin.get()).w_jumplist[i_0 as usize] = fm;
                                    if (*curwin.get()).w_jumplistlen < JUMPLISTSIZE {
                                        (*curwin.get()).w_jumplistlen += 1;
                                    }
                                    if (*curwin.get()).w_jumplistidx >= i_0
                                        && (*curwin.get()).w_jumplistidx + 1 as ::core::ffi::c_int
                                            <= (*curwin.get()).w_jumplistlen
                                    {
                                        (*curwin.get()).w_jumplistidx += 1;
                                    }
                                } else {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                }
                            }
                        }
                        9 => {
                            let mut i_1: size_t = 0 as size_t;
                            while i_1 < cur_entry.data.buffer_list.size {
                                let sfname: *mut ::core::ffi::c_char = path_try_shorten_fname(
                                    (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                        .fname,
                                );
                                let buf_0: *mut buf_T = buflist_new(
                                    (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                        .fname,
                                    sfname,
                                    0 as linenr_T,
                                    BLN_LISTED as ::core::ffi::c_int,
                                );
                                if !buf_0.is_null() {
                                    let mut view: fmarkv_T = fmarkv_T {
                                        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                                        skipcol: 0 as colnr_T,
                                    };
                                    let fmarkp___: *mut fmark_T = &raw mut (*buf_0).b_last_cursor;
                                    free_fmark(*fmarkp___);
                                    let fmarkp__: *mut fmark_T = fmarkp___;
                                    (*fmarkp__).mark =
                                        (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                            .pos;
                                    (*fmarkp__).fnum = 0 as ::core::ffi::c_int;
                                    (*fmarkp__).timestamp = os_time();
                                    (*fmarkp__).view = view;
                                    (*fmarkp__).additional_data =
                                        ::core::ptr::null_mut::<AdditionalData>();
                                    buflist_setfpos(
                                        buf_0,
                                        curwin.get(),
                                        (*buf_0).b_last_cursor.mark.lnum,
                                        (*buf_0).b_last_cursor.mark.col,
                                        false_0 != 0,
                                    );
                                    xfree((*buf_0).additional_data as *mut ::core::ffi::c_void);
                                    (*buf_0).additional_data =
                                        (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                            .additional_data;
                                    (*cur_entry.data.buffer_list.buffers.offset(i_1 as isize))
                                        .additional_data =
                                        ::core::ptr::null_mut::<AdditionalData>();
                                }
                                i_1 = i_1.wrapping_add(1);
                            }
                            shada_free_shada_entry(&raw mut cur_entry);
                        }
                        11 | 10 => {
                            if get_old_files as ::core::ffi::c_int != 0
                                && !set_has_cstr_t(
                                    &raw mut oldfiles_set,
                                    cur_entry.data.filemark.fname as cstr_t,
                                )
                            {
                                let mut fname: *mut ::core::ffi::c_char =
                                    cur_entry.data.filemark.fname;
                                if want_marks {
                                    fname = xstrdup(fname);
                                }
                                set_put_cstr_t(
                                    &raw mut oldfiles_set,
                                    fname as cstr_t,
                                    ::core::ptr::null_mut::<*mut cstr_t>(),
                                );
                                tv_list_append_allocated_string(oldfiles_list, fname);
                                if !want_marks {
                                    cur_entry.data.filemark.fname =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                }
                            }
                            if !want_marks {
                                shada_free_shada_entry(&raw mut cur_entry);
                            } else {
                                let mut buf_1: *mut buf_T =
                                    find_buffer(&raw mut fname_bufs, cur_entry.data.filemark.fname);
                                if buf_1.is_null() {
                                    shada_free_shada_entry(&raw mut cur_entry);
                                } else {
                                    let fm_0: fmark_T = fmark_T {
                                        mark: cur_entry.data.filemark.mark,
                                        fnum: (*buf_1).handle as ::core::ffi::c_int,
                                        timestamp: cur_entry.timestamp,
                                        view: fmarkv_T {
                                            topline_offset: MAXLNUM as ::core::ffi::c_int
                                                as linenr_T,
                                            skipcol: 0 as colnr_T,
                                        },
                                        additional_data: cur_entry.additional_data,
                                    };
                                    if cur_entry.type_0 as ::core::ffi::c_int
                                        == kSDItemLocalMark as ::core::ffi::c_int
                                    {
                                        if !mark_set_local(
                                            cur_entry.data.filemark.name,
                                            buf_1,
                                            fm_0,
                                            !force,
                                        ) {
                                            shada_free_shada_entry(&raw mut cur_entry);
                                            break 's_732;
                                        }
                                    } else {
                                        set_put_ptr_t(
                                            &raw mut cl_bufs,
                                            buf_1 as ptr_t,
                                            ::core::ptr::null_mut::<*mut ptr_t>(),
                                        );
                                        let mut i_2: ::core::ffi::c_int = 0;
                                        i_2 = (*buf_1).b_changelistlen;
                                        while i_2 > 0 as ::core::ffi::c_int {
                                            let jl_entry_0: fmark_T = (*buf_1).b_changelist
                                                [(i_2 - 1 as ::core::ffi::c_int) as usize];
                                            if jl_entry_0.timestamp <= cur_entry.timestamp {
                                                if marks_equal(
                                                    jl_entry_0.mark,
                                                    cur_entry.data.filemark.mark,
                                                ) {
                                                    i_2 = -1 as ::core::ffi::c_int;
                                                }
                                                break;
                                            } else {
                                                i_2 -= 1;
                                            }
                                        }
                                        if i_2 > 0 as ::core::ffi::c_int
                                            && (*buf_1).b_changelistlen == JUMPLISTSIZE
                                        {
                                            free_fmark(
                                                (*buf_1).b_changelist
                                                    [0 as ::core::ffi::c_int as usize],
                                            );
                                        }
                                        i_2 = marklist_insert(
                                            &raw mut (*buf_1).b_changelist as *mut fmark_T
                                                as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<fmark_T>(),
                                            (*buf_1).b_changelistlen,
                                            i_2,
                                        );
                                        if i_2 != -1 as ::core::ffi::c_int {
                                            (*buf_1).b_changelist[i_2 as usize] = fm_0;
                                            if (*buf_1).b_changelistlen < JUMPLISTSIZE {
                                                (*buf_1).b_changelistlen += 1;
                                            }
                                        } else {
                                            xfree(fm_0.additional_data as *mut ::core::ffi::c_void);
                                        }
                                    }
                                    xfree(
                                        cur_entry.data.filemark.fname as *mut ::core::ffi::c_void,
                                    );
                                }
                            }
                        }
                        -1 | _ => {}
                    }
                }
            }
        }
    }
    if srni_flags & kSDReadHistory as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_3 < HIST_COUNT as ::core::ffi::c_int {
            hms_insert_whole_neovim_history(
                (&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize),
            );
            clr_history(i_3);
            let mut new_hisidx: *mut ::core::ffi::c_int =
                ::core::ptr::null_mut::<::core::ffi::c_int>();
            let mut new_hisnum: *mut ::core::ffi::c_int =
                ::core::ptr::null_mut::<::core::ffi::c_int>();
            let mut hist: *mut histentry_T =
                hist_get_array(i_3 as uint8_t, &raw mut new_hisidx, &raw mut new_hisnum);
            if !hist.is_null() {
                hms_to_he_array(
                    (&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize),
                    hist,
                    new_hisidx,
                    new_hisnum,
                );
            }
            hms_dealloc((&raw mut hms as *mut HistoryMergerState).offset(i_3 as isize));
            i_3 += 1;
        }
    }
    if cl_bufs.h.n_occupied != 0 {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if set_has_ptr_t(&raw mut cl_bufs, (*wp).w_buffer as ptr_t) {
                    (*wp).w_changelistidx = (*(*wp).w_buffer).b_changelistlen;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    xfree(cl_bufs.keys as *mut ::core::ffi::c_void);
    xfree(cl_bufs.h.hash as *mut ::core::ffi::c_void);
    cl_bufs = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < fname_bufs.set.h.n_keys {
        key = *fname_bufs.set.keys.offset(__i as isize) as *const ::core::ffi::c_char;
        xfree(key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
        __i = __i.wrapping_add(1);
    }
    xfree(fname_bufs.set.keys as *mut ::core::ffi::c_void);
    xfree(fname_bufs.set.h.hash as *mut ::core::ffi::c_void);
    fname_bufs.set = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut fname_bufs.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
    xfree(oldfiles_set.keys as *mut ::core::ffi::c_void);
    xfree(oldfiles_set.h.hash as *mut ::core::ffi::c_void);
    oldfiles_set = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
}
static default_shada_file: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
unsafe extern "C" fn shada_get_default_file() -> *const ::core::ffi::c_char {
    if (*default_shada_file.ptr()).is_null() {
        let mut shada_dir: *mut ::core::ffi::c_char = stdpaths_user_state_subpath(
            b"shada\0".as_ptr() as *const ::core::ffi::c_char,
            0 as size_t,
            false_0 != 0,
        );
        default_shada_file.set(concat_fnames_realloc(
            shada_dir,
            b"main.shada\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ));
    }
    return default_shada_file.get();
}
unsafe extern "C" fn shada_filename(
    mut file: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if file.is_null() || *file as ::core::ffi::c_int == NUL {
        if !(*p_shadafile.ptr()).is_null() && *p_shadafile.get() as ::core::ffi::c_int != NUL {
            if !strequal(
                p_shadafile.get(),
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                file = p_shadafile.get();
            } else {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else {
            file = find_shada_parameter('n' as ::core::ffi::c_int);
            if file.is_null() || *file as ::core::ffi::c_int == NUL {
                file = shada_get_default_file();
            }
            let mut len: size_t = expand_env(
                file as *mut ::core::ffi::c_char,
                (NameBuff.ptr() as *mut ::core::ffi::c_char)
                    .offset(0 as ::core::ffi::c_int as isize),
                MAXPATHL,
            );
            file = (NameBuff.ptr() as *mut ::core::ffi::c_char)
                .offset(0 as ::core::ffi::c_int as isize);
            return xmemdupz(file as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        }
    }
    return xstrdup(file);
}
pub const SHADA_MPACK_FREE_SPACE: ::core::ffi::c_int = 4 as ::core::ffi::c_int * MPACK_ITEM_SIZE;
unsafe extern "C" fn shada_check_buffer(mut packer: *mut PackerBuffer) {
    if mpack_remaining(packer) < SHADA_MPACK_FREE_SPACE as size_t {
        (*packer).packer_flush.expect("non-null function pointer")(packer);
    }
}
unsafe extern "C" fn additional_data_len(mut src: *mut AdditionalData) -> uint32_t {
    return if !src.is_null() {
        (*src).nitems
    } else {
        0 as uint32_t
    };
}
unsafe extern "C" fn dump_additional_data(
    mut src: *mut AdditionalData,
    mut sbuf: *mut PackerBuffer,
) {
    if !src.is_null() {
        mpack_raw(
            &raw mut (*src).data as *mut ::core::ffi::c_char,
            (*src).nbytes as size_t,
            sbuf,
        );
    }
}
unsafe extern "C" fn shada_pack_entry(
    packer: *mut PackerBuffer,
    mut entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    let mut packed: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut ret: ShaDaWriteResult = kSDWriteFailed;
    let mut sbuf: PackerBuffer = packer_string_buffer();
    shada_check_buffer(&raw mut sbuf);
    '_shada_pack_entry_error: {
        match entry.type_0 as ::core::ffi::c_int {
            0 => {
                abort();
            }
            -1 => {
                mpack_raw(
                    entry.data.unknown_item.contents,
                    entry.data.unknown_item.size,
                    &raw mut sbuf,
                );
            }
            4 => {
                let is_hist_search: bool = entry.data.history_item.histtype as ::core::ffi::c_int
                    == HIST_SEARCH as ::core::ffi::c_int;
                let mut arr_size: uint32_t = (2 as uint32_t)
                    .wrapping_add(is_hist_search as uint32_t)
                    .wrapping_add(additional_data_len(entry.additional_data));
                mpack_array(&raw mut sbuf.ptr, arr_size);
                mpack_uint(
                    &raw mut sbuf.ptr,
                    entry.data.history_item.histtype as uint32_t,
                );
                mpack_bin(
                    cstr_as_string(entry.data.history_item.string),
                    &raw mut sbuf,
                );
                if is_hist_search {
                    mpack_uint(
                        &raw mut sbuf.ptr,
                        entry.data.history_item.sep as uint8_t as uint32_t,
                    );
                }
                dump_additional_data(entry.additional_data, &raw mut sbuf);
            }
            6 => {
                let mut is_blob: bool = entry.data.global_var.value.v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint;
                let mut arr_size_0: uint32_t = ((2 as ::core::ffi::c_int
                    + (if is_blob as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })) as uint32_t)
                    .wrapping_add(additional_data_len(entry.additional_data));
                mpack_array(&raw mut sbuf.ptr, arr_size_0);
                let varname: String_0 = cstr_as_string(entry.data.global_var.name);
                mpack_bin(varname, &raw mut sbuf);
                let mut vardesc: [::core::ffi::c_char; 256] = ::core::mem::transmute::<
                    [u8; 256],
                    [::core::ffi::c_char; 256],
                >(
                    *b"variable g:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                );
                memcpy(
                    (&raw mut vardesc as *mut ::core::ffi::c_char).offset(
                        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize)
                            as isize,
                    ) as *mut ::core::ffi::c_void,
                    varname.data as *const ::core::ffi::c_void,
                    varname.size.wrapping_add(1 as size_t),
                );
                if encode_vim_to_msgpack(
                    &raw mut sbuf,
                    &raw mut entry.data.global_var.value,
                    &raw mut vardesc as *mut ::core::ffi::c_char,
                ) == FAIL
                {
                    ret = kSDWriteIgnError;
                    semsg(
                        gettext(b"E574: Failed to write variable %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        entry.data.global_var.name,
                    );
                    break '_shada_pack_entry_error;
                } else {
                    if is_blob {
                        mpack_check_buffer(&raw mut sbuf);
                        mpack_integer(
                            &raw mut sbuf.ptr,
                            VAR_TYPE_BLOB as ::core::ffi::c_int as Integer,
                        );
                    }
                    dump_additional_data(entry.additional_data, &raw mut sbuf);
                }
            }
            3 => {
                let mut arr_size_1: uint32_t =
                    (1 as uint32_t).wrapping_add(additional_data_len(entry.additional_data));
                mpack_array(&raw mut sbuf.ptr, arr_size_1);
                mpack_bin(cstr_as_string(entry.data.sub_string.sub), &raw mut sbuf);
                dump_additional_data(entry.additional_data, &raw mut sbuf);
            }
            2 => {
                let mut entry_map_size: uint32_t = (1 as uint32_t)
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .magic as ::core::ffi::c_int
                            == entry.data.search_pattern.magic as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .is_last_used as ::core::ffi::c_int
                            == entry.data.search_pattern.is_last_used as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .smartcase as ::core::ffi::c_int
                            == entry.data.search_pattern.smartcase as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .has_line_offset as ::core::ffi::c_int
                            == entry.data.search_pattern.has_line_offset as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .place_cursor_at_end as ::core::ffi::c_int
                            == entry.data.search_pattern.place_cursor_at_end as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .is_substitute_pattern as ::core::ffi::c_int
                            == entry.data.search_pattern.is_substitute_pattern
                                as ::core::ffi::c_int) as ::core::ffi::c_int
                            as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .highlighted as ::core::ffi::c_int
                            == entry.data.search_pattern.highlighted as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .offset
                            == entry.data.search_pattern.offset)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .search_backward as ::core::ffi::c_int
                            == entry.data.search_pattern.search_backward as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(additional_data_len(entry.additional_data));
                mpack_map(&raw mut sbuf.ptr, entry_map_size);
                mpack_str(
                    String_0 {
                        data: b"sp\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    },
                    &raw mut sbuf,
                );
                mpack_bin(entry.data.search_pattern.pat, &raw mut sbuf);
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .magic as ::core::ffi::c_int
                    == entry.data.search_pattern.magic as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"sm\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .magic,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .is_last_used as ::core::ffi::c_int
                    == entry.data.search_pattern.is_last_used as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"su\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .is_last_used,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .smartcase as ::core::ffi::c_int
                    == entry.data.search_pattern.smartcase as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"sc\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .smartcase,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .has_line_offset as ::core::ffi::c_int
                    == entry.data.search_pattern.has_line_offset as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"sl\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .has_line_offset,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .place_cursor_at_end as ::core::ffi::c_int
                    == entry.data.search_pattern.place_cursor_at_end as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"se\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .place_cursor_at_end,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .is_substitute_pattern as ::core::ffi::c_int
                    == entry.data.search_pattern.is_substitute_pattern as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"ss\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .is_substitute_pattern,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .highlighted as ::core::ffi::c_int
                    == entry.data.search_pattern.highlighted as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"sh\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .highlighted,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .search_backward as ::core::ffi::c_int
                    == entry.data.search_pattern.search_backward as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"sb\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(
                        &raw mut sbuf.ptr,
                        !(*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .search_pattern
                            .search_backward,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .search_pattern
                    .offset
                    == entry.data.search_pattern.offset)
                {
                    mpack_str(
                        String_0 {
                            data: b"so\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_integer(&raw mut sbuf.ptr, entry.data.search_pattern.offset);
                }
                dump_additional_data(entry.additional_data, &raw mut sbuf);
            }
            11 | 7 | 10 | 8 => {
                let mut entry_map_size_0: size_t = (1 as uint32_t)
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .filemark
                            .mark
                            .lnum
                            == entry.data.filemark.mark.lnum)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .filemark
                            .mark
                            .col
                            == entry.data.filemark.mark.col)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .filemark
                            .name as ::core::ffi::c_int
                            == entry.data.filemark.name as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(additional_data_len(entry.additional_data))
                    as size_t;
                mpack_map(&raw mut sbuf.ptr, entry_map_size_0 as uint32_t);
                mpack_str(
                    String_0 {
                        data: b"f\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    },
                    &raw mut sbuf,
                );
                mpack_bin(cstr_as_string(entry.data.filemark.fname), &raw mut sbuf);
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .filemark
                    .mark
                    .lnum
                    == entry.data.filemark.mark.lnum)
                {
                    mpack_str(
                        String_0 {
                            data: b"l\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_integer(&raw mut sbuf.ptr, entry.data.filemark.mark.lnum as Integer);
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .filemark
                    .mark
                    .col
                    == entry.data.filemark.mark.col)
                {
                    mpack_str(
                        String_0 {
                            data: b"c\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_integer(&raw mut sbuf.ptr, entry.data.filemark.mark.col as Integer);
                }
                '_c2rust_label: {
                    if (if entry.type_0 as ::core::ffi::c_int == kSDItemJump as ::core::ffi::c_int
                        || entry.type_0 as ::core::ffi::c_int == kSDItemChange as ::core::ffi::c_int
                    {
                        ((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .filemark
                            .name as ::core::ffi::c_int
                            == entry.data.filemark.name as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    }) != 0
                    {
                    } else {
                        __assert_fail(
                            b"entry.type == kSDItemJump || entry.type == kSDItemChange ? CHECK_DEFAULT(entry, filemark.name) : true\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/shada.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1471 as ::core::ffi::c_uint,
                            b"ShaDaWriteResult shada_pack_entry(PackerBuffer *const, ShadaEntry, const size_t)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .filemark
                    .name as ::core::ffi::c_int
                    == entry.data.filemark.name as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"n\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_uint(
                        &raw mut sbuf.ptr,
                        entry.data.filemark.name as uint8_t as uint32_t,
                    );
                }
                dump_additional_data(entry.additional_data, &raw mut sbuf);
            }
            5 => {
                let mut entry_map_size_1: uint32_t = (2 as uint32_t)
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .reg
                            .type_0 as ::core::ffi::c_int
                            == entry.data.reg.type_0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .reg
                            .width
                            == entry.data.reg.width) as ::core::ffi::c_int
                            as uint32_t,
                    )
                    .wrapping_add(
                        !((*sd_default_values.ptr())[entry.type_0 as usize]
                            .data
                            .reg
                            .is_unnamed as ::core::ffi::c_int
                            == entry.data.reg.is_unnamed as ::core::ffi::c_int)
                            as ::core::ffi::c_int as uint32_t,
                    )
                    .wrapping_add(additional_data_len(entry.additional_data));
                mpack_map(&raw mut sbuf.ptr, entry_map_size_1);
                mpack_str(
                    String_0 {
                        data: b"rc\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    },
                    &raw mut sbuf,
                );
                mpack_array(&raw mut sbuf.ptr, entry.data.reg.contents_size as uint32_t);
                let mut i: size_t = 0 as size_t;
                while i < entry.data.reg.contents_size {
                    mpack_bin(*entry.data.reg.contents.offset(i as isize), &raw mut sbuf);
                    i = i.wrapping_add(1);
                }
                mpack_str(
                    String_0 {
                        data: b"n\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    },
                    &raw mut sbuf,
                );
                mpack_uint(
                    &raw mut sbuf.ptr,
                    entry.data.reg.name as uint8_t as uint32_t,
                );
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .reg
                    .type_0 as ::core::ffi::c_int
                    == entry.data.reg.type_0 as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"rt\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_uint(
                        &raw mut sbuf.ptr,
                        entry.data.reg.type_0 as uint8_t as uint32_t,
                    );
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .reg
                    .width
                    == entry.data.reg.width)
                {
                    mpack_str(
                        String_0 {
                            data: b"rw\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_uint64(&raw mut sbuf.ptr, entry.data.reg.width as uint64_t);
                }
                if !((*sd_default_values.ptr())[entry.type_0 as usize]
                    .data
                    .reg
                    .is_unnamed as ::core::ffi::c_int
                    == entry.data.reg.is_unnamed as ::core::ffi::c_int)
                {
                    mpack_str(
                        String_0 {
                            data: b"ru\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bool(&raw mut sbuf.ptr, entry.data.reg.is_unnamed);
                }
                dump_additional_data(entry.additional_data, &raw mut sbuf);
            }
            9 => {
                mpack_array(&raw mut sbuf.ptr, entry.data.buffer_list.size as uint32_t);
                let mut i_0: size_t = 0 as size_t;
                while i_0 < entry.data.buffer_list.size {
                    let mut entry_map_size_2: size_t = (1 as size_t)
                        .wrapping_add(
                            ((*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                .pos
                                .lnum
                                != (*default_pos.ptr()).lnum)
                                as ::core::ffi::c_int as size_t,
                        )
                        .wrapping_add(
                            ((*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                .pos
                                .col
                                != (*default_pos.ptr()).col)
                                as ::core::ffi::c_int as size_t,
                        )
                        .wrapping_add(additional_data_len(
                            (*entry.data.buffer_list.buffers.offset(i_0 as isize)).additional_data,
                        ) as size_t);
                    mpack_map(&raw mut sbuf.ptr, entry_map_size_2 as uint32_t);
                    mpack_str(
                        String_0 {
                            data: b"f\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                        &raw mut sbuf,
                    );
                    mpack_bin(
                        cstr_as_string(
                            (*entry.data.buffer_list.buffers.offset(i_0 as isize)).fname,
                        ),
                        &raw mut sbuf,
                    );
                    if (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                        .pos
                        .lnum
                        != 1 as linenr_T
                    {
                        mpack_str(
                            String_0 {
                                data: b"l\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &raw mut sbuf,
                        );
                        mpack_uint64(
                            &raw mut sbuf.ptr,
                            (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                .pos
                                .lnum as uint64_t,
                        );
                    }
                    if (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                        .pos
                        .col
                        != 0 as ::core::ffi::c_int
                    {
                        mpack_str(
                            String_0 {
                                data: b"c\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            &raw mut sbuf,
                        );
                        mpack_uint64(
                            &raw mut sbuf.ptr,
                            (*entry.data.buffer_list.buffers.offset(i_0 as isize))
                                .pos
                                .col as uint64_t,
                        );
                    }
                    dump_additional_data(
                        (*entry.data.buffer_list.buffers.offset(i_0 as isize)).additional_data,
                        &raw mut sbuf,
                    );
                    i_0 = i_0.wrapping_add(1);
                }
            }
            1 => {
                mpack_map(&raw mut sbuf.ptr, entry.data.header.size as uint32_t);
                let mut i_1: size_t = 0 as size_t;
                while i_1 < entry.data.header.size {
                    mpack_str(
                        (*entry.data.header.items.offset(i_1 as isize)).key,
                        &raw mut sbuf,
                    );
                    let obj: Object = (*entry.data.header.items.offset(i_1 as isize)).value;
                    match obj.type_0 as ::core::ffi::c_uint {
                        4 => {
                            mpack_bin(obj.data.string, &raw mut sbuf);
                        }
                        2 => {
                            mpack_integer(&raw mut sbuf.ptr, obj.data.integer);
                        }
                        _ => {
                            abort();
                        }
                    }
                    i_1 = i_1.wrapping_add(1);
                }
            }
            _ => {}
        }
        packed = packer_take_string(&raw mut sbuf);
        if max_kbyte == 0 || packed.size <= max_kbyte.wrapping_mul(1024 as size_t) {
            shada_check_buffer(packer);
            if entry.type_0 as ::core::ffi::c_int == kSDItemUnknown as ::core::ffi::c_int {
                mpack_uint64(&raw mut (*packer).ptr, entry.data.unknown_item.type_0);
            } else {
                mpack_uint64(&raw mut (*packer).ptr, entry.type_0 as uint64_t);
            }
            mpack_uint64(&raw mut (*packer).ptr, entry.timestamp);
            if packed.size > 0 as size_t {
                mpack_uint64(&raw mut (*packer).ptr, packed.size as uint64_t);
                mpack_raw(packed.data, packed.size, packer);
            }
            if (*packer).anyint != 0 as int64_t {
                break '_shada_pack_entry_error;
            }
        }
        ret = kSDWriteSuccessful;
    }
    xfree(sbuf.startptr as *mut ::core::ffi::c_void);
    return ret;
}
#[inline(always)]
unsafe extern "C" fn shada_pack_pfreed_entry(
    packer: *mut PackerBuffer,
    mut entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    let mut ret: ShaDaWriteResult = shada_pack_entry(packer, entry, max_kbyte);
    shada_free_shada_entry(&raw mut entry);
    return ret;
}
unsafe extern "C" fn compare_file_marks(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let a_fms: *const *const FileMarks = a as *const *const FileMarks;
    let b_fms: *const *const FileMarks = b as *const *const FileMarks;
    return if (**a_fms).greatest_timestamp == (**b_fms).greatest_timestamp {
        0 as ::core::ffi::c_int
    } else if (**a_fms).greatest_timestamp > (**b_fms).greatest_timestamp {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn shada_check_status(
    mut initial_fpos: uintmax_t,
    mut status: ::core::ffi::c_int,
    mut remaining: size_t,
) -> ShaDaReadResult {
    match status {
        0 => {
            if remaining != 0 {
                semsg(
                    gettext(
                        b"E576: Failed to parse ShaDa file: extra bytes in msgpack string at position %lu\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    initial_fpos as uint64_t,
                );
                return kSDReadStatusNotShaDa;
            }
            return kSDReadStatusSuccess;
        }
        1 => {
            semsg(
                gettext(
                    b"E576: Failed to parse ShaDa file: incomplete msgpack string at position %lu\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                initial_fpos as uint64_t,
            );
            return kSDReadStatusNotShaDa;
        }
        _ => {
            semsg(
                gettext(
                    b"E576: Failed to parse ShaDa file due to a msgpack parser error at position %lu\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                initial_fpos as uint64_t,
            );
            return kSDReadStatusNotShaDa;
        }
    };
}
#[inline]
unsafe extern "C" fn shada_read_when_writing(
    sd_reader: *mut FileDescriptor,
    srni_flags: ::core::ffi::c_uint,
    max_kbyte: size_t,
    wms: *mut WriteMergerState,
    packer: *mut PackerBuffer,
) -> ShaDaWriteResult {
    let mut ret: ShaDaWriteResult = kSDWriteSuccessful;
    let mut entry: ShadaEntry = ShadaEntry {
        type_0: kSDItemMissing,
        can_free_entry: false,
        timestamp: 0,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut srni_ret: ShaDaReadResult = kSDReadStatusSuccess;
    loop {
        srni_ret = shada_read_next_item(sd_reader, &raw mut entry, srni_flags, max_kbyte);
        if srni_ret as ::core::ffi::c_uint
            == kSDReadStatusFinished as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        match srni_ret as ::core::ffi::c_uint {
            1 => {
                abort();
            }
            3 => {
                ret = kSDWriteReadNotShada;
            }
            2 => {}
            4 => {
                continue;
            }
            0 | _ => {
                's_781: {
                    match entry.type_0 as ::core::ffi::c_int {
                        1 | 9 => {
                            abort();
                        }
                        -1 => {
                            ret = shada_pack_entry(packer, entry, 0 as size_t);
                            shada_free_shada_entry(&raw mut entry);
                        }
                        2 => {
                            let wms_entry: *mut ShadaEntry =
                                if entry.data.search_pattern.is_substitute_pattern
                                    as ::core::ffi::c_int
                                    != 0
                                {
                                    &raw mut (*wms).sub_search_pattern
                                } else {
                                    &raw mut (*wms).search_pattern
                                };
                            's_94: {
                                if (*wms_entry).type_0 as ::core::ffi::c_int
                                    != kSDItemMissing as ::core::ffi::c_int
                                {
                                    if (*wms_entry).timestamp >= entry.timestamp {
                                        shada_free_shada_entry(&raw mut entry);
                                        break 's_94;
                                    } else {
                                        shada_free_shada_entry(wms_entry);
                                    }
                                }
                                *wms_entry = entry;
                            }
                        }
                        3 => {
                            let wms_entry_0: *mut ShadaEntry = &raw mut (*wms).replacement;
                            's_132: {
                                if (*wms_entry_0).type_0 as ::core::ffi::c_int
                                    != kSDItemMissing as ::core::ffi::c_int
                                {
                                    if (*wms_entry_0).timestamp >= entry.timestamp {
                                        shada_free_shada_entry(&raw mut entry);
                                        break 's_132;
                                    } else {
                                        shada_free_shada_entry(wms_entry_0);
                                    }
                                }
                                *wms_entry_0 = entry;
                            }
                        }
                        4 => {
                            if entry.data.history_item.histtype as ::core::ffi::c_int
                                >= HIST_COUNT as ::core::ffi::c_int
                            {
                                ret = shada_pack_entry(packer, entry, 0 as size_t);
                                shada_free_shada_entry(&raw mut entry);
                            } else if (*wms).hms[entry.data.history_item.histtype as usize]
                                .hmll
                                .size
                                != 0 as size_t
                            {
                                hms_insert(
                                    (&raw mut (*wms).hms as *mut HistoryMergerState)
                                        .offset(entry.data.history_item.histtype as isize),
                                    entry,
                                    true_0 != 0,
                                );
                            } else {
                                shada_free_shada_entry(&raw mut entry);
                            }
                        }
                        5 => {
                            let idx: ::core::ffi::c_int =
                                op_reg_index(entry.data.reg.name as ::core::ffi::c_int);
                            if idx < 0 as ::core::ffi::c_int {
                                ret = shada_pack_entry(packer, entry, 0 as size_t);
                                shada_free_shada_entry(&raw mut entry);
                            } else {
                                let wms_entry_1: *mut ShadaEntry = (&raw mut (*wms).registers
                                    as *mut ShadaEntry)
                                    .offset(idx as isize);
                                's_223: {
                                    if (*wms_entry_1).type_0 as ::core::ffi::c_int
                                        != kSDItemMissing as ::core::ffi::c_int
                                    {
                                        if (*wms_entry_1).timestamp >= entry.timestamp {
                                            shada_free_shada_entry(&raw mut entry);
                                            break 's_223;
                                        } else {
                                            shada_free_shada_entry(wms_entry_1);
                                        }
                                    }
                                    *wms_entry_1 = entry;
                                }
                            }
                        }
                        6 => {
                            if !set_has_cstr_t(
                                &raw mut (*wms).dumped_variables,
                                entry.data.global_var.name as cstr_t,
                            ) {
                                ret = shada_pack_entry(packer, entry, 0 as size_t);
                            }
                            shada_free_shada_entry(&raw mut entry);
                        }
                        7 => {
                            if ascii_isdigit(entry.data.filemark.name as ::core::ffi::c_int) {
                                let mut processed_mark: bool = false_0 != 0;
                                let mut i: size_t = ::core::mem::size_of::<[ShadaEntry; 10]>()
                                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[ShadaEntry; 10]>()
                                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    );
                                while i > 0 as size_t {
                                    let mut wms_entry_2: ShadaEntry =
                                        (*wms).numbered_marks[i.wrapping_sub(1 as size_t) as usize];
                                    if wms_entry_2.type_0 as ::core::ffi::c_int
                                        == kSDItemGlobalMark as ::core::ffi::c_int
                                    {
                                        if wms_entry_2.timestamp == entry.timestamp
                                            && (wms_entry_2.additional_data.is_null()
                                                && entry.additional_data.is_null())
                                            && marks_equal(
                                                wms_entry_2.data.filemark.mark,
                                                entry.data.filemark.mark,
                                            )
                                                as ::core::ffi::c_int
                                                != 0
                                            && strcmp(
                                                wms_entry_2.data.filemark.fname,
                                                entry.data.filemark.fname,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            shada_free_shada_entry(&raw mut entry);
                                            processed_mark = true_0 != 0;
                                            break;
                                        } else if wms_entry_2.timestamp >= entry.timestamp {
                                            processed_mark = true_0 != 0;
                                            if i < ::core::mem::size_of::<[ShadaEntry; 10]>()
                                                .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[ShadaEntry; 10]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ShadaEntry,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                )
                                            {
                                                replace_numbered_mark(wms, i, entry);
                                            } else {
                                                shada_free_shada_entry(&raw mut entry);
                                            }
                                            break;
                                        }
                                    }
                                    i = i.wrapping_sub(1);
                                }
                                if !processed_mark {
                                    replace_numbered_mark(wms, 0 as size_t, entry);
                                }
                            } else {
                                let idx_0: ::core::ffi::c_int =
                                    mark_global_index(entry.data.filemark.name);
                                if idx_0 < 0 as ::core::ffi::c_int {
                                    ret = shada_pack_entry(packer, entry, 0 as size_t);
                                    shada_free_shada_entry(&raw mut entry);
                                } else {
                                    let mut mark: *mut ShadaEntry =
                                        if idx_0 < 26 as ::core::ffi::c_int {
                                            (&raw mut (*wms).global_marks as *mut ShadaEntry)
                                                .offset(idx_0 as isize)
                                        } else {
                                            (&raw mut (*wms).numbered_marks as *mut ShadaEntry)
                                                .offset((idx_0 - 26 as ::core::ffi::c_int) as isize)
                                        };
                                    if (*mark).type_0 as ::core::ffi::c_int
                                        == kSDItemMissing as ::core::ffi::c_int
                                    {
                                        if (*namedfm.ptr())[idx_0 as usize].fmark.timestamp
                                            >= entry.timestamp
                                        {
                                            shada_free_shada_entry(&raw mut entry);
                                            break 's_781;
                                        }
                                    }
                                    let wms_entry_3: *mut ShadaEntry = mark;
                                    's_401: {
                                        if (*wms_entry_3).type_0 as ::core::ffi::c_int
                                            != kSDItemMissing as ::core::ffi::c_int
                                        {
                                            if (*wms_entry_3).timestamp >= entry.timestamp {
                                                shada_free_shada_entry(&raw mut entry);
                                                break 's_401;
                                            } else {
                                                shada_free_shada_entry(wms_entry_3);
                                            }
                                        }
                                        *wms_entry_3 = entry;
                                    }
                                }
                            }
                        }
                        11 | 10 => {
                            if shada_removable(entry.data.filemark.fname) {
                                shada_free_shada_entry(&raw mut entry);
                            } else {
                                let fname: *const ::core::ffi::c_char = entry.data.filemark.fname;
                                let mut key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
                                let mut new_item: bool = false_0 != 0;
                                let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
                                    &raw mut (*wms).file_marks,
                                    fname as cstr_t,
                                    &raw mut key,
                                    &raw mut new_item,
                                );
                                if new_item {
                                    *key = xstrdup(fname) as cstr_t;
                                }
                                if (*val).is_null() {
                                    *val = xcalloc(1 as size_t, ::core::mem::size_of::<FileMarks>())
                                        as ptr_t;
                                }
                                let filemarks: *mut FileMarks = *val as *mut FileMarks;
                                if entry.timestamp > (*filemarks).greatest_timestamp {
                                    (*filemarks).greatest_timestamp = entry.timestamp;
                                }
                                if entry.type_0 as ::core::ffi::c_int
                                    == kSDItemLocalMark as ::core::ffi::c_int
                                {
                                    let idx_1: ::core::ffi::c_int =
                                        mark_local_index(entry.data.filemark.name);
                                    if idx_1 < 0 as ::core::ffi::c_int {
                                        (*filemarks).additional_marks_size =
                                            (*filemarks).additional_marks_size.wrapping_add(1);
                                        (*filemarks).additional_marks = xrealloc(
                                            (*filemarks).additional_marks
                                                as *mut ::core::ffi::c_void,
                                            (*filemarks)
                                                .additional_marks_size
                                                .wrapping_mul(::core::mem::size_of::<ShadaEntry>()),
                                        )
                                            as *mut ShadaEntry;
                                        *(*filemarks).additional_marks.offset(
                                            (*filemarks)
                                                .additional_marks_size
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        ) = entry;
                                    } else {
                                        let wms_entry_4: *mut ShadaEntry =
                                            (&raw mut (*filemarks).marks as *mut ShadaEntry)
                                                .offset(idx_1 as isize);
                                        let mut set_wms: bool = true_0 != 0;
                                        if (*wms_entry_4).type_0 as ::core::ffi::c_int
                                            != kSDItemMissing as ::core::ffi::c_int
                                        {
                                            if (*wms_entry_4).timestamp >= entry.timestamp {
                                                shada_free_shada_entry(&raw mut entry);
                                                break 's_781;
                                            } else if (*wms_entry_4).can_free_entry {
                                                if *key
                                                    == (*wms_entry_4).data.filemark.fname as cstr_t
                                                {
                                                    *key = entry.data.filemark.fname as cstr_t;
                                                }
                                                shada_free_shada_entry(wms_entry_4);
                                            }
                                        } else {
                                            let mut buf: *mut buf_T = firstbuf.get();
                                            while !buf.is_null() {
                                                if !(*buf).b_ffname.is_null()
                                                    && path_fnamecmp(
                                                        entry.data.filemark.fname,
                                                        (*buf).b_ffname,
                                                    ) == 0 as ::core::ffi::c_int
                                                {
                                                    let mut fm: fmark_T = fmark_T {
                                                        mark: pos_T {
                                                            lnum: 0,
                                                            col: 0,
                                                            coladd: 0,
                                                        },
                                                        fnum: 0,
                                                        timestamp: 0,
                                                        view: fmarkv_T {
                                                            topline_offset: 0,
                                                            skipcol: 0,
                                                        },
                                                        additional_data: ::core::ptr::null_mut::<
                                                            AdditionalData,
                                                        >(
                                                        ),
                                                    };
                                                    mark_get(
                                                        buf,
                                                        curwin.get(),
                                                        &raw mut fm,
                                                        kMarkBufLocal,
                                                        entry.data.filemark.name
                                                            as ::core::ffi::c_int,
                                                    );
                                                    if fm.timestamp >= entry.timestamp {
                                                        set_wms = false_0 != 0;
                                                        shada_free_shada_entry(&raw mut entry);
                                                        break;
                                                    }
                                                }
                                                buf = (*buf).b_next;
                                            }
                                        }
                                        if set_wms {
                                            *wms_entry_4 = entry;
                                        }
                                    }
                                } else {
                                    let mut i_0: ::core::ffi::c_int = 0;
                                    i_0 = (*filemarks).changes_size as ::core::ffi::c_int;
                                    while i_0 > 0 as ::core::ffi::c_int {
                                        let jl_entry: ShadaEntry = (*filemarks).changes
                                            [(i_0 - 1 as ::core::ffi::c_int) as usize];
                                        if jl_entry.timestamp <= entry.timestamp {
                                            if marks_equal(
                                                jl_entry.data.filemark.mark,
                                                entry.data.filemark.mark,
                                            ) {
                                                i_0 = -1 as ::core::ffi::c_int;
                                            }
                                            break;
                                        } else {
                                            i_0 -= 1;
                                        }
                                    }
                                    if i_0 > 0 as ::core::ffi::c_int
                                        && (*filemarks).changes_size == JUMPLISTSIZE as size_t
                                    {
                                        shada_free_shada_entry(
                                            (&raw mut (*filemarks).changes as *mut ShadaEntry)
                                                .offset(0 as ::core::ffi::c_int as isize),
                                        );
                                    }
                                    i_0 = marklist_insert(
                                        &raw mut (*filemarks).changes as *mut ShadaEntry
                                            as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<ShadaEntry>(),
                                        (*filemarks).changes_size as ::core::ffi::c_int,
                                        i_0,
                                    );
                                    if i_0 != -1 as ::core::ffi::c_int {
                                        (*filemarks).changes[i_0 as usize] = entry;
                                        if (*filemarks).changes_size < JUMPLISTSIZE as size_t {
                                            (*filemarks).changes_size =
                                                (*filemarks).changes_size.wrapping_add(1);
                                        }
                                    } else {
                                        shada_free_shada_entry(&raw mut entry);
                                    }
                                }
                            }
                        }
                        8 => {
                            let mut i_1: ::core::ffi::c_int = 0;
                            i_1 = (*wms).jumps_size as ::core::ffi::c_int;
                            while i_1 > 0 as ::core::ffi::c_int {
                                let jl_entry_0: ShadaEntry =
                                    (*wms).jumps[(i_1 - 1 as ::core::ffi::c_int) as usize];
                                if jl_entry_0.timestamp <= entry.timestamp {
                                    if marks_equal(
                                        jl_entry_0.data.filemark.mark,
                                        entry.data.filemark.mark,
                                    ) as ::core::ffi::c_int
                                        != 0
                                        && strcmp(
                                            jl_entry_0.data.filemark.fname,
                                            entry.data.filemark.fname,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        i_1 = -1 as ::core::ffi::c_int;
                                    }
                                    break;
                                } else {
                                    i_1 -= 1;
                                }
                            }
                            if i_1 > 0 as ::core::ffi::c_int
                                && (*wms).jumps_size == JUMPLISTSIZE as size_t
                            {
                                shada_free_shada_entry(
                                    (&raw mut (*wms).jumps as *mut ShadaEntry)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                );
                            }
                            i_1 = marklist_insert(
                                &raw mut (*wms).jumps as *mut ShadaEntry
                                    as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<ShadaEntry>(),
                                (*wms).jumps_size as ::core::ffi::c_int,
                                i_1,
                            );
                            if i_1 != -1 as ::core::ffi::c_int {
                                (*wms).jumps[i_1 as usize] = entry;
                                if (*wms).jumps_size < JUMPLISTSIZE as size_t {
                                    (*wms).jumps_size = (*wms).jumps_size.wrapping_add(1);
                                }
                            } else {
                                shada_free_shada_entry(&raw mut entry);
                            }
                        }
                        0 | _ => {}
                    }
                }
                continue;
            }
        }
        return ret;
    }
    return ret;
}
#[inline(always)]
unsafe extern "C" fn ignore_buf(buf: *const buf_T, removable_bufs: *mut Set_ptr_t) -> bool {
    return buf.is_null()
        || (*buf).b_ffname.is_null()
        || (*buf).b_p_bl == 0 && (*buf).b_p_initialized as ::core::ffi::c_int != 0
        || bt_quickfix(buf) as ::core::ffi::c_int != 0
        || bt_terminal(buf) as ::core::ffi::c_int != 0
        || set_has_ptr_t(removable_bufs, buf as ptr_t) as ::core::ffi::c_int != 0;
}
#[inline(always)]
unsafe extern "C" fn shada_get_buflist(removable_bufs: *mut Set_ptr_t) -> ShadaEntry {
    let mut max_bufs: ::core::ffi::c_int = get_shada_parameter('%' as ::core::ffi::c_int);
    let mut buf_count: size_t = 0 as size_t;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !ignore_buf(buf, removable_bufs)
            && (*buf).b_p_bl != 0
            && (max_bufs < 0 as ::core::ffi::c_int || buf_count < max_bufs as size_t)
        {
            buf_count = buf_count.wrapping_add(1);
        }
        buf = (*buf).b_next;
    }
    let mut buflist_entry: ShadaEntry = ShadaEntry {
        type_0: kSDItemBufferList,
        can_free_entry: false,
        timestamp: os_time(),
        data: C2Rust_Unnamed_22 {
            buffer_list: buffer_list {
                size: buf_count,
                buffers: xmalloc(
                    buf_count.wrapping_mul(::core::mem::size_of::<buffer_list_buffer>()),
                ) as *mut buffer_list_buffer,
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut i: size_t = 0 as size_t;
    let mut buf_0: *mut buf_T = firstbuf.get();
    while !buf_0.is_null() {
        if !(ignore_buf(buf_0, removable_bufs) as ::core::ffi::c_int != 0 || (*buf_0).b_p_bl == 0) {
            if i >= buf_count {
                break;
            }
            *buflist_entry.data.buffer_list.buffers.offset(i as isize) = buffer_list_buffer {
                pos: (*buf_0).b_last_cursor.mark,
                fname: (*buf_0).b_ffname,
                additional_data: (*buf_0).additional_data,
            }
                as buffer_list_buffer;
            i = i.wrapping_add(1);
        }
        buf_0 = (*buf_0).b_next;
    }
    return buflist_entry;
}
#[inline(always)]
unsafe extern "C" fn add_search_pattern(
    ret_pse: *mut ShadaEntry,
    get_pattern: SearchPatternGetter,
    is_substitute_pattern: bool,
    search_last_used: bool,
    search_highlighted: bool,
) {
    let defaults: ShadaEntry =
        (*sd_default_values.ptr())[kSDItemSearchPattern as ::core::ffi::c_int as usize];
    let mut pat: SearchPattern = SearchPattern {
        pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        patlen: 0,
        magic: false,
        no_scs: false,
        timestamp: 0,
        off: SearchOffset {
            dir: 0,
            line: false,
            end: false,
            off: 0,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    get_pattern.expect("non-null function pointer")(&raw mut pat);
    if !pat.pat.is_null() {
        *ret_pse = ShadaEntry {
            type_0: kSDItemSearchPattern,
            can_free_entry: false_0 != 0,
            timestamp: pat.timestamp,
            data: C2Rust_Unnamed_22 {
                search_pattern: KeyDict__shada_search_pat {
                    is_set___shada_search_pat_: 0,
                    magic: pat.magic as Boolean,
                    smartcase: !pat.no_scs,
                    has_line_offset: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                        defaults.data.search_pattern.has_line_offset as ::core::ffi::c_int
                    } else {
                        pat.off.line as ::core::ffi::c_int
                    } != 0,
                    place_cursor_at_end: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                        defaults.data.search_pattern.place_cursor_at_end as ::core::ffi::c_int
                    } else {
                        pat.off.end as ::core::ffi::c_int
                    } != 0,
                    is_last_used: is_substitute_pattern as ::core::ffi::c_int
                        ^ search_last_used as ::core::ffi::c_int
                        != 0,
                    is_substitute_pattern: is_substitute_pattern as Boolean,
                    highlighted: is_substitute_pattern as ::core::ffi::c_int
                        ^ search_last_used as ::core::ffi::c_int
                        != 0
                        && search_highlighted as ::core::ffi::c_int != 0,
                    search_backward: !is_substitute_pattern
                        && pat.off.dir as ::core::ffi::c_int == '?' as ::core::ffi::c_int,
                    offset: if is_substitute_pattern as ::core::ffi::c_int != 0 {
                        defaults.data.search_pattern.offset
                    } else {
                        pat.off.off as Integer
                    },
                    pat: cstr_as_string(pat.pat),
                },
            },
            additional_data: pat.additional_data,
        };
    }
}
#[inline(always)]
unsafe extern "C" fn shada_initialize_registers(
    wms: *mut WriteMergerState,
    mut max_reg_lines: ::core::ffi::c_int,
) {
    let mut reg_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let limit_reg_lines: bool = max_reg_lines >= 0 as ::core::ffi::c_int;
    loop {
        let mut reg: yankreg_T = yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        let mut name: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        let mut is_unnamed: bool = false_0 != 0;
        reg_iter = op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
        if name as ::core::ffi::c_int == NUL {
            break;
        }
        if !(limit_reg_lines as ::core::ffi::c_int != 0 && reg.y_size > max_reg_lines as size_t) {
            (*wms).registers[op_reg_index(name as ::core::ffi::c_int) as usize] = ShadaEntry {
                type_0: kSDItemRegister,
                can_free_entry: false_0 != 0,
                timestamp: reg.timestamp,
                data: C2Rust_Unnamed_22 {
                    reg: reg {
                        name: name,
                        type_0: reg.y_type,
                        contents: reg.y_array,
                        is_unnamed: is_unnamed,
                        contents_size: reg.y_size,
                        width: (if reg.y_type as ::core::ffi::c_int
                            == kMTBlockWise as ::core::ffi::c_int
                        {
                            reg.y_width as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as size_t,
                    },
                },
                additional_data: reg.additional_data,
            };
        }
        if reg_iter.is_null() {
            break;
        }
    }
}
#[inline(always)]
unsafe extern "C" fn replace_numbered_mark(
    wms: *mut WriteMergerState,
    idx: size_t,
    entry: ShadaEntry,
) {
    shada_free_shada_entry(
        (&raw mut (*wms).numbered_marks as *mut ShadaEntry).offset(
            ::core::mem::size_of::<[ShadaEntry; 10]>()
                .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                .wrapping_div(
                    (::core::mem::size_of::<[ShadaEntry; 10]>()
                        .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                        == 0) as ::core::ffi::c_int as usize,
                )
                .wrapping_sub(1 as usize) as isize,
        ),
    );
    let mut i: size_t = idx;
    while i < ::core::mem::size_of::<[ShadaEntry; 10]>()
        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
        .wrapping_div(
            (::core::mem::size_of::<[ShadaEntry; 10]>()
                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize)
    {
        if (*wms).numbered_marks[i as usize].type_0 as ::core::ffi::c_int
            == kSDItemGlobalMark as ::core::ffi::c_int
        {
            (*wms).numbered_marks[i as usize].data.filemark.name =
                ('0' as ::core::ffi::c_int + i as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_char;
        }
        i = i.wrapping_add(1);
    }
    memmove(
        (&raw mut (*wms).numbered_marks as *mut ShadaEntry)
            .offset(idx as isize)
            .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*wms).numbered_marks as *mut ShadaEntry).offset(idx as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<ShadaEntry>().wrapping_mul(
            ::core::mem::size_of::<[ShadaEntry; 10]>()
                .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                .wrapping_div(
                    (::core::mem::size_of::<[ShadaEntry; 10]>()
                        .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
                .wrapping_sub(1 as size_t)
                .wrapping_sub(idx),
        ),
    );
    (*wms).numbered_marks[idx as usize] = entry;
    (*wms).numbered_marks[idx as usize].data.filemark.name =
        ('0' as ::core::ffi::c_int + idx as ::core::ffi::c_int) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn find_removable_bufs(mut removable_bufs: *mut Set_ptr_t) {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).b_ffname.is_null() && shada_removable((*buf).b_ffname) as ::core::ffi::c_int != 0
        {
            set_put_ptr_t(
                removable_bufs,
                buf as ptr_t,
                ::core::ptr::null_mut::<*mut ptr_t>(),
            );
        }
        buf = (*buf).b_next;
    }
}
unsafe extern "C" fn hist_type2char(type_0: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match type_0 {
        0 => return ':' as ::core::ffi::c_int,
        1 => return '/' as ::core::ffi::c_int,
        2 => return '=' as ::core::ffi::c_int,
        3 => return '@' as ::core::ffi::c_int,
        4 => return '>' as ::core::ffi::c_int,
        _ => {
            abort();
        }
    };
}
unsafe extern "C" fn packer_buffer_for_file(mut file: *mut FileDescriptor) -> PackerBuffer {
    if file_space(file) < SHADA_MPACK_FREE_SPACE as size_t {
        file_flush(file);
    }
    return packer_buffer_t {
        startptr: (*file).buffer,
        ptr: (*file).write_pos,
        endptr: (*file).buffer.offset(ARENA_BLOCK_SIZE as isize),
        anydata: file as *mut ::core::ffi::c_void,
        anyint: 0 as int64_t,
        packer_flush: Some(flush_file_buffer as unsafe extern "C" fn(*mut PackerBuffer) -> ()),
    };
}
unsafe extern "C" fn flush_file_buffer(mut buffer: *mut PackerBuffer) {
    let mut fd: *mut FileDescriptor = (*buffer).anydata as *mut FileDescriptor;
    (*fd).write_pos = (*buffer).ptr;
    (*buffer).anyint = file_flush(fd) as int64_t;
    (*buffer).ptr = (*fd).write_pos;
}
unsafe extern "C" fn shada_write(
    sd_writer: *mut FileDescriptor,
    sd_reader: *mut FileDescriptor,
) -> ShaDaWriteResult {
    let mut file_markss_size: size_t = 0;
    let mut all_file_markss: *mut *mut FileMarks = ::core::ptr::null_mut::<*mut FileMarks>();
    let mut cur_file_marks: *mut *mut FileMarks = ::core::ptr::null_mut::<*mut FileMarks>();
    let mut val_0: ptr_t = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut file_markss_to_dump: size_t = 0;
    let mut ret: ShaDaWriteResult = kSDWriteSuccessful;
    let mut max_kbyte_i: ::core::ffi::c_int = get_shada_parameter('s' as ::core::ffi::c_int);
    if max_kbyte_i < 0 as ::core::ffi::c_int {
        max_kbyte_i = 10 as ::core::ffi::c_int;
    }
    if max_kbyte_i == 0 as ::core::ffi::c_int {
        return ret;
    }
    let wms: *mut WriteMergerState =
        xcalloc(1 as size_t, ::core::mem::size_of::<WriteMergerState>()) as *mut WriteMergerState;
    let mut dump_one_history: [bool; 5] = [false; 5];
    let dump_global_vars: bool = !find_shada_parameter('!' as ::core::ffi::c_int).is_null();
    let mut max_reg_lines: ::core::ffi::c_int = get_shada_parameter('<' as ::core::ffi::c_int);
    if max_reg_lines < 0 as ::core::ffi::c_int {
        max_reg_lines = get_shada_parameter('"' as ::core::ffi::c_int);
    }
    let dump_registers: bool = max_reg_lines != 0 as ::core::ffi::c_int;
    let mut removable_bufs: Set_ptr_t = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    let max_kbyte: size_t = max_kbyte_i as size_t;
    let num_marked_files: size_t = get_shada_parameter('\'' as ::core::ffi::c_int) as size_t;
    let dump_global_marks: bool =
        get_shada_parameter('f' as ::core::ffi::c_int) != 0 as ::core::ffi::c_int;
    let mut dump_history: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < HIST_COUNT as ::core::ffi::c_int {
        let mut num_saved: ::core::ffi::c_int = get_shada_parameter(hist_type2char(i));
        if num_saved == -1 as ::core::ffi::c_int {
            num_saved = p_hi.get() as ::core::ffi::c_int;
        }
        if num_saved > 0 as ::core::ffi::c_int {
            dump_history = true_0 != 0;
            dump_one_history[i as usize] = true_0 != 0;
            hms_init(
                (&raw mut (*wms).hms as *mut HistoryMergerState).offset(i as isize),
                i as uint8_t,
                num_saved as size_t,
                !sd_reader.is_null(),
                false_0 != 0,
            );
        } else {
            dump_one_history[i as usize] = false_0 != 0;
        }
        i += 1;
    }
    let srni_flags: ::core::ffi::c_uint = (kSDReadUndisableableData as ::core::ffi::c_int
        | kSDReadUnknown as ::core::ffi::c_int
        | (if dump_history as ::core::ffi::c_int != 0 {
            kSDReadHistory as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })
        | (if dump_registers as ::core::ffi::c_int != 0 {
            kSDReadRegisters as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })
        | (if dump_global_vars as ::core::ffi::c_int != 0 {
            kSDReadVariables as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })
        | (if dump_global_marks as ::core::ffi::c_int != 0 {
            kSDReadGlobalMarks as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })
        | (if num_marked_files != 0 {
            kSDReadLocalMarks as ::core::ffi::c_int | kSDReadChanges as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_uint;
    let mut packer: PackerBuffer = packer_buffer_for_file(sd_writer);
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            set_last_cursor(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    find_removable_bufs(&raw mut removable_bufs);
    '_shada_write_exit: {
        let mut c2rust_lvalue: [KeyValuePair; 5] = [
            key_value_pair {
                key: String_0 {
                    data: b"generator\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                        .wrapping_sub(1 as size_t),
                },
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_1 {
                        string: String_0 {
                            data: b"nvim\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
            },
            key_value_pair {
                key: String_0 {
                    data: b"version\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                        .wrapping_sub(1 as size_t),
                },
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_1 {
                        string: cstr_as_string(longVersion.get()),
                    },
                },
            },
            key_value_pair {
                key: String_0 {
                    data: b"max_kbyte\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                        .wrapping_sub(1 as size_t),
                },
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_1 {
                        integer: max_kbyte as Integer,
                    },
                },
            },
            key_value_pair {
                key: String_0 {
                    data: b"pid\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                        .wrapping_sub(1 as size_t),
                },
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_1 {
                        integer: os_get_pid(),
                    },
                },
            },
            key_value_pair {
                key: String_0 {
                    data: b"encoding\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as size_t),
                },
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_1 {
                        string: cstr_as_string(p_enc.get()),
                    },
                },
            },
        ];
        if shada_pack_entry(
            &raw mut packer,
            ShadaEntry {
                type_0: kSDItemHeader,
                can_free_entry: false,
                timestamp: os_time(),
                data: C2Rust_Unnamed_22 {
                    header: Dict {
                        size: 5 as size_t,
                        capacity: 5 as size_t,
                        items: &raw mut c2rust_lvalue as *mut KeyValuePair,
                    },
                },
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            },
            0 as size_t,
        ) as ::core::ffi::c_uint
            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ret = kSDWriteFailed;
        } else {
            if !find_shada_parameter('%' as ::core::ffi::c_int).is_null() {
                let mut buflist_entry: ShadaEntry = shada_get_buflist(&raw mut removable_bufs);
                if shada_pack_entry(&raw mut packer, buflist_entry, 0 as size_t)
                    as ::core::ffi::c_uint
                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
                    ret = kSDWriteFailed;
                    break '_shada_write_exit;
                } else {
                    xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
                }
            }
            's_310: {
                if dump_global_vars {
                    let mut var_iter: *const ::core::ffi::c_void =
                        ::core::ptr::null::<::core::ffi::c_void>();
                    let cur_timestamp: Timestamp = os_time();
                    loop {
                        let mut vartv: typval_T = typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        };
                        let mut name: *const ::core::ffi::c_char =
                            ::core::ptr::null::<::core::ffi::c_char>();
                        var_iter = var_shada_iter(
                            var_iter,
                            &raw mut name,
                            &raw mut vartv,
                            VAR_FLAVOUR_SHADA,
                        );
                        if name.is_null() {
                            break 's_310;
                        }
                        's_190: {
                            match vartv.v_type as ::core::ffi::c_uint {
                                3 | 9 => {
                                    tv_clear(&raw mut vartv);
                                    break 's_190;
                                }
                                5 => {
                                    let mut di: *mut dict_T = vartv.vval.v_dict;
                                    let mut copyID: ::core::ffi::c_int = get_copyID();
                                    if !set_ref_in_ht(
                                        &raw mut (*di).dv_hashtab,
                                        copyID,
                                        ::core::ptr::null_mut::<*mut list_stack_T>(),
                                    ) && copyID == (*di).dv_copyID
                                    {
                                        tv_clear(&raw mut vartv);
                                        break 's_190;
                                    }
                                }
                                4 => {
                                    let mut l: *mut list_T = vartv.vval.v_list;
                                    let mut copyID_0: ::core::ffi::c_int = get_copyID();
                                    if !set_ref_in_list_items(
                                        l,
                                        copyID_0,
                                        ::core::ptr::null_mut::<*mut ht_stack_T>(),
                                    ) && copyID_0 == (*l).lv_copyID
                                    {
                                        tv_clear(&raw mut vartv);
                                        break 's_190;
                                    }
                                }
                                _ => {}
                            }
                            let mut tgttv: typval_T = typval_T {
                                v_type: VAR_UNKNOWN,
                                v_lock: VAR_UNLOCKED,
                                vval: typval_vval_union { v_number: 0 },
                            };
                            tv_copy(&raw mut vartv, &raw mut tgttv);
                            let mut spe_ret: ShaDaWriteResult = kSDWriteSuccessful;
                            spe_ret = shada_pack_entry(
                                &raw mut packer,
                                ShadaEntry {
                                    type_0: kSDItemVariable,
                                    can_free_entry: false,
                                    timestamp: cur_timestamp,
                                    data: C2Rust_Unnamed_22 {
                                        global_var: global_var {
                                            name: name as *mut ::core::ffi::c_char,
                                            value: tgttv,
                                        },
                                    },
                                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                                },
                                max_kbyte,
                            );
                            if spe_ret as ::core::ffi::c_uint
                                == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                tv_clear(&raw mut vartv);
                                tv_clear(&raw mut tgttv);
                                ret = kSDWriteFailed;
                                break '_shada_write_exit;
                            } else {
                                tv_clear(&raw mut vartv);
                                tv_clear(&raw mut tgttv);
                                if spe_ret as ::core::ffi::c_uint
                                    == kSDWriteSuccessful as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    set_put_cstr_t(
                                        &raw mut (*wms).dumped_variables,
                                        name as cstr_t,
                                        ::core::ptr::null_mut::<*mut cstr_t>(),
                                    );
                                }
                            }
                        }
                        if var_iter.is_null() {
                            break 's_310;
                        }
                    }
                }
            }
            if num_marked_files > 0 as size_t {
                (*wms).jumps_size = shada_init_jumps(
                    &raw mut (*wms).jumps as *mut ShadaEntry,
                    &raw mut removable_bufs,
                );
            }
            if dump_one_history[HIST_SEARCH as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                let search_highlighted: bool = !(no_hlsearch.get() as ::core::ffi::c_int != 0
                    || !find_shada_parameter('h' as ::core::ffi::c_int).is_null());
                let search_last_used: bool = search_was_last_used();
                add_search_pattern(
                    &raw mut (*wms).search_pattern,
                    Some(get_search_pattern as unsafe extern "C" fn(*mut SearchPattern) -> ()),
                    false_0 != 0,
                    search_last_used,
                    search_highlighted,
                );
                add_search_pattern(
                    &raw mut (*wms).sub_search_pattern,
                    Some(get_substitute_pattern as unsafe extern "C" fn(*mut SearchPattern) -> ()),
                    true_0 != 0,
                    search_last_used,
                    search_highlighted,
                );
                let mut sub: SubReplacementString = SubReplacementString {
                    sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    timestamp: 0,
                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                };
                sub_get_replacement(&raw mut sub);
                if !sub.sub.is_null() {
                    (*wms).replacement = ShadaEntry {
                        type_0: kSDItemSubString,
                        can_free_entry: false_0 != 0,
                        timestamp: sub.timestamp,
                        data: C2Rust_Unnamed_22 {
                            sub_string: sub_string { sub: sub.sub },
                        },
                        additional_data: sub.additional_data,
                    };
                }
            }
            if dump_global_marks {
                let mut global_mark_iter: *const ::core::ffi::c_void =
                    ::core::ptr::null::<::core::ffi::c_void>();
                let mut digit_mark_idx: size_t = 0 as size_t;
                loop {
                    let mut name_0: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                    let mut fm: xfmark_T = xfmark_T {
                        fmark: fmark_T {
                            mark: pos_T {
                                lnum: 0,
                                col: 0,
                                coladd: 0,
                            },
                            fnum: 0,
                            timestamp: 0,
                            view: fmarkv_T {
                                topline_offset: 0,
                                skipcol: 0,
                            },
                            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                        },
                        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
                    global_mark_iter =
                        mark_global_iter(global_mark_iter, &raw mut name_0, &raw mut fm);
                    if name_0 as ::core::ffi::c_int == NUL {
                        break;
                    }
                    let mut fname: *const ::core::ffi::c_char =
                        ::core::ptr::null::<::core::ffi::c_char>();
                    's_367: {
                        if fm.fmark.fnum == 0 as ::core::ffi::c_int {
                            '_c2rust_label: {
                                if !fm.fname.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm.fname != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/shada.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2441 as ::core::ffi::c_uint,
                                        b"ShaDaWriteResult shada_write(FileDescriptor *const, FileDescriptor *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            if shada_removable(fm.fname) {
                                break 's_367;
                            } else {
                                fname = fm.fname;
                            }
                        } else {
                            let buf: *const buf_T = buflist_findnr(fm.fmark.fnum);
                            if buf.is_null()
                                || (*buf).b_ffname.is_null()
                                || set_has_ptr_t(&raw mut removable_bufs, buf as ptr_t)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                break 's_367;
                            } else {
                                fname = (*buf).b_ffname;
                            }
                        }
                        let entry: ShadaEntry = ShadaEntry {
                            type_0: kSDItemGlobalMark,
                            can_free_entry: false_0 != 0,
                            timestamp: fm.fmark.timestamp,
                            data: C2Rust_Unnamed_22 {
                                filemark: shada_filemark {
                                    name: name_0,
                                    mark: fm.fmark.mark,
                                    fname: fname as *mut ::core::ffi::c_char,
                                },
                            },
                            additional_data: fm.fmark.additional_data,
                        };
                        if ascii_isdigit(name_0 as ::core::ffi::c_int) {
                            let c2rust_fresh18 = digit_mark_idx;
                            digit_mark_idx = digit_mark_idx.wrapping_add(1);
                            replace_numbered_mark(wms, c2rust_fresh18, entry);
                        } else {
                            (*wms).global_marks[mark_global_index(name_0) as usize] = entry;
                        }
                    }
                    if global_mark_iter.is_null() {
                        break;
                    }
                }
            }
            if dump_registers {
                shada_initialize_registers(wms, max_reg_lines);
            }
            if num_marked_files > 0 as size_t {
                let mut buf_0: *mut buf_T = firstbuf.get();
                while !buf_0.is_null() {
                    if !ignore_buf(buf_0, &raw mut removable_bufs) {
                        let mut local_marks_iter: *const ::core::ffi::c_void =
                            ::core::ptr::null::<::core::ffi::c_void>();
                        let fname_0: *const ::core::ffi::c_char = (*buf_0).b_ffname;
                        let mut map_key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
                        let mut new_item: bool = false_0 != 0;
                        let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
                            &raw mut (*wms).file_marks,
                            fname_0 as cstr_t,
                            &raw mut map_key,
                            &raw mut new_item,
                        );
                        if new_item {
                            *map_key = xstrdup(fname_0) as cstr_t;
                        }
                        if (*val).is_null() {
                            *val =
                                xcalloc(1 as size_t, ::core::mem::size_of::<FileMarks>()) as ptr_t;
                        }
                        let filemarks: *mut FileMarks = *val as *mut FileMarks;
                        loop {
                            let mut fm_0: fmark_T = fmark_T {
                                mark: pos_T {
                                    lnum: 0,
                                    col: 0,
                                    coladd: 0,
                                },
                                fnum: 0,
                                timestamp: 0,
                                view: fmarkv_T {
                                    topline_offset: 0,
                                    skipcol: 0,
                                },
                                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                            };
                            let mut name_1: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            local_marks_iter = mark_buffer_iter(
                                local_marks_iter,
                                buf_0,
                                &raw mut name_1,
                                &raw mut fm_0,
                            );
                            if name_1 as ::core::ffi::c_int == NUL {
                                break;
                            }
                            (*filemarks).marks[mark_local_index(name_1) as usize] = ShadaEntry {
                                type_0: kSDItemLocalMark,
                                can_free_entry: false_0 != 0,
                                timestamp: fm_0.timestamp,
                                data: C2Rust_Unnamed_22 {
                                    filemark: shada_filemark {
                                        name: name_1,
                                        mark: fm_0.mark,
                                        fname: fname_0 as *mut ::core::ffi::c_char,
                                    },
                                },
                                additional_data: fm_0.additional_data,
                            };
                            if fm_0.timestamp > (*filemarks).greatest_timestamp {
                                (*filemarks).greatest_timestamp = fm_0.timestamp;
                            }
                            if local_marks_iter.is_null() {
                                break;
                            }
                        }
                        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i_0 < (*buf_0).b_changelistlen {
                            let fm_1: fmark_T = (*buf_0).b_changelist[i_0 as usize];
                            (*filemarks).changes[i_0 as usize] = ShadaEntry {
                                type_0: kSDItemChange,
                                can_free_entry: false_0 != 0,
                                timestamp: fm_1.timestamp,
                                data: C2Rust_Unnamed_22 {
                                    filemark: shada_filemark {
                                        name: 0,
                                        mark: fm_1.mark,
                                        fname: fname_0 as *mut ::core::ffi::c_char,
                                    },
                                },
                                additional_data: fm_1.additional_data,
                            };
                            if fm_1.timestamp > (*filemarks).greatest_timestamp {
                                (*filemarks).greatest_timestamp = fm_1.timestamp;
                            }
                            i_0 += 1;
                        }
                        (*filemarks).changes_size = (*buf_0).b_changelistlen as size_t;
                    }
                    buf_0 = (*buf_0).b_next;
                }
            }
            if !sd_reader.is_null() {
                let srww_ret: ShaDaWriteResult =
                    shada_read_when_writing(sd_reader, srni_flags, max_kbyte, wms, &raw mut packer);
                if srww_ret as ::core::ffi::c_uint
                    != kSDWriteSuccessful as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = srww_ret;
                }
            }
            if dump_global_marks as ::core::ffi::c_int != 0
                && !ignore_buf(curbuf.get(), &raw mut removable_bufs)
                && (*curwin.get()).w_cursor.lnum != 0 as linenr_T
            {
                replace_numbered_mark(
                    wms,
                    0 as size_t,
                    ShadaEntry {
                        type_0: kSDItemGlobalMark,
                        can_free_entry: false_0 != 0,
                        timestamp: os_time(),
                        data: C2Rust_Unnamed_22 {
                            filemark: shada_filemark {
                                name: '0' as ::core::ffi::c_char,
                                mark: (*curwin.get()).w_cursor,
                                fname: (*curbuf.get()).b_ffname,
                            },
                        },
                        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                    },
                );
            }
            let mut i_: size_t = 0 as size_t;
            while i_
                < ::core::mem::size_of::<[ShadaEntry; 26]>()
                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ShadaEntry; 26]>()
                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                            == 0) as ::core::ffi::c_int as usize,
                    )
            {
                if (*wms).global_marks[i_ as usize].type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).global_marks[i_ as usize],
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                i_ = i_.wrapping_add(1);
            }
            let mut i__0: size_t = 0 as size_t;
            while i__0
                < ::core::mem::size_of::<[ShadaEntry; 10]>()
                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ShadaEntry; 10]>()
                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                            == 0) as ::core::ffi::c_int as usize,
                    )
            {
                if (*wms).numbered_marks[i__0 as usize].type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).numbered_marks[i__0 as usize],
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                i__0 = i__0.wrapping_add(1);
            }
            let mut i__1: size_t = 0 as size_t;
            while i__1
                < ::core::mem::size_of::<[ShadaEntry; 37]>()
                    .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ShadaEntry; 37]>()
                            .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                            == 0) as ::core::ffi::c_int as usize,
                    )
            {
                if (*wms).registers[i__1 as usize].type_0 as ::core::ffi::c_int
                    != kSDItemMissing as ::core::ffi::c_int
                {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (*wms).registers[i__1 as usize],
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    }
                }
                i__1 = i__1.wrapping_add(1);
            }
            let mut i_1: size_t = 0 as size_t;
            while i_1 < (*wms).jumps_size {
                if shada_pack_pfreed_entry(&raw mut packer, (*wms).jumps[i_1 as usize], max_kbyte)
                    as ::core::ffi::c_uint
                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = kSDWriteFailed;
                    break '_shada_write_exit;
                } else {
                    i_1 = i_1.wrapping_add(1);
                }
            }
            if (*wms).search_pattern.type_0 as ::core::ffi::c_int
                != kSDItemMissing as ::core::ffi::c_int
            {
                if shada_pack_pfreed_entry(&raw mut packer, (*wms).search_pattern, max_kbyte)
                    as ::core::ffi::c_uint
                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = kSDWriteFailed;
                    break '_shada_write_exit;
                }
            }
            if (*wms).sub_search_pattern.type_0 as ::core::ffi::c_int
                != kSDItemMissing as ::core::ffi::c_int
            {
                if shada_pack_pfreed_entry(&raw mut packer, (*wms).sub_search_pattern, max_kbyte)
                    as ::core::ffi::c_uint
                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = kSDWriteFailed;
                    break '_shada_write_exit;
                }
            }
            if (*wms).replacement.type_0 as ::core::ffi::c_int
                != kSDItemMissing as ::core::ffi::c_int
            {
                if shada_pack_pfreed_entry(&raw mut packer, (*wms).replacement, max_kbyte)
                    as ::core::ffi::c_uint
                    == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = kSDWriteFailed;
                    break '_shada_write_exit;
                }
            }
            file_markss_size = (*wms).file_marks.set.h.size as size_t;
            all_file_markss =
                xmalloc(file_markss_size.wrapping_mul(::core::mem::size_of::<*mut FileMarks>()))
                    as *mut *mut FileMarks;
            cur_file_marks = all_file_markss;
            val_0 = ::core::ptr::null_mut::<::core::ffi::c_void>();
            let mut __i: uint32_t = 0;
            __i = 0 as uint32_t;
            while __i < (*wms).file_marks.set.h.n_keys {
                val_0 = *(*wms).file_marks.values.offset(__i as isize);
                let c2rust_fresh19 = cur_file_marks;
                cur_file_marks = cur_file_marks.offset(1);
                let c2rust_lvalue_ptr = &raw mut *c2rust_fresh19;
                *c2rust_lvalue_ptr = val_0 as *mut FileMarks;
                __i = __i.wrapping_add(1);
            }
            qsort(
                all_file_markss as *mut ::core::ffi::c_void,
                file_markss_size,
                ::core::mem::size_of::<*mut FileMarks>(),
                Some(
                    compare_file_marks
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            file_markss_to_dump = if num_marked_files < file_markss_size {
                num_marked_files
            } else {
                file_markss_size
            };
            let mut i_2: size_t = 0 as size_t;
            while i_2 < file_markss_to_dump {
                let mut i__2: size_t = 0 as size_t;
                while i__2
                    < ::core::mem::size_of::<[ShadaEntry; 29]>()
                        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
                        .wrapping_div(
                            (::core::mem::size_of::<[ShadaEntry; 29]>()
                                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                                == 0) as ::core::ffi::c_int as usize,
                        )
                {
                    if (**all_file_markss.offset(i_2 as isize)).marks[i__2 as usize].type_0
                        as ::core::ffi::c_int
                        != kSDItemMissing as ::core::ffi::c_int
                    {
                        if shada_pack_pfreed_entry(
                            &raw mut packer,
                            (**all_file_markss.offset(i_2 as isize)).marks[i__2 as usize],
                            max_kbyte,
                        ) as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            ret = kSDWriteFailed;
                            break '_shada_write_exit;
                        }
                    }
                    i__2 = i__2.wrapping_add(1);
                }
                let mut j: size_t = 0 as size_t;
                while j < (**all_file_markss.offset(i_2 as isize)).changes_size {
                    if shada_pack_pfreed_entry(
                        &raw mut packer,
                        (**all_file_markss.offset(i_2 as isize)).changes[j as usize],
                        max_kbyte,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    } else {
                        j = j.wrapping_add(1);
                    }
                }
                let mut j_0: size_t = 0 as size_t;
                while j_0 < (**all_file_markss.offset(i_2 as isize)).additional_marks_size {
                    if shada_pack_entry(
                        &raw mut packer,
                        *(**all_file_markss.offset(i_2 as isize))
                            .additional_marks
                            .offset(j_0 as isize),
                        0 as size_t,
                    ) as ::core::ffi::c_uint
                        == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        shada_free_shada_entry(
                            (**all_file_markss.offset(i_2 as isize))
                                .additional_marks
                                .offset(j_0 as isize),
                        );
                        ret = kSDWriteFailed;
                        break '_shada_write_exit;
                    } else {
                        shada_free_shada_entry(
                            (**all_file_markss.offset(i_2 as isize))
                                .additional_marks
                                .offset(j_0 as isize),
                        );
                        j_0 = j_0.wrapping_add(1);
                    }
                }
                xfree(
                    (**all_file_markss.offset(i_2 as isize)).additional_marks
                        as *mut ::core::ffi::c_void,
                );
                i_2 = i_2.wrapping_add(1);
            }
            xfree(all_file_markss as *mut ::core::ffi::c_void);
            if dump_history {
                let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                loop {
                    if i_3 >= HIST_COUNT as ::core::ffi::c_int {
                        break '_shada_write_exit;
                    }
                    if dump_one_history[i_3 as usize] {
                        hms_insert_whole_neovim_history(
                            (&raw mut (*wms).hms as *mut HistoryMergerState).offset(i_3 as isize),
                        );
                        let mut cur_entry: *mut HMLListEntry =
                            (*wms).hms[i_3 as usize].hmll.first as *mut HMLListEntry;
                        while !cur_entry.is_null() {
                            if shada_pack_pfreed_entry(
                                &raw mut packer,
                                (*cur_entry).data,
                                max_kbyte,
                            ) as ::core::ffi::c_uint
                                == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                ret = kSDWriteFailed;
                                break;
                            } else {
                                cur_entry = (*cur_entry).next as *mut HMLListEntry;
                            }
                        }
                        if ret as ::core::ffi::c_uint
                            == kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break '_shada_write_exit;
                        }
                    }
                    i_3 += 1;
                }
            }
        }
    }
    let mut i_4: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_4 < HIST_COUNT as ::core::ffi::c_int {
        if dump_one_history[i_4 as usize] {
            hms_dealloc((&raw mut (*wms).hms as *mut HistoryMergerState).offset(i_4 as isize));
        }
        i_4 += 1;
    }
    let mut stored_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut __i_0: uint32_t = 0;
    __i_0 = 0 as uint32_t;
    while __i_0 < (*wms).file_marks.set.h.n_keys {
        stored_key =
            *(*wms).file_marks.set.keys.offset(__i_0 as isize) as *const ::core::ffi::c_char;
        val_0 = *(*wms).file_marks.values.offset(__i_0 as isize);
        xfree(stored_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
        xfree(val_0 as *mut ::core::ffi::c_void);
        __i_0 = __i_0.wrapping_add(1);
    }
    xfree((*wms).file_marks.set.keys as *mut ::core::ffi::c_void);
    xfree((*wms).file_marks.set.h.hash as *mut ::core::ffi::c_void);
    (*wms).file_marks.set = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*wms).file_marks.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree(removable_bufs.keys as *mut ::core::ffi::c_void);
    xfree(removable_bufs.h.hash as *mut ::core::ffi::c_void);
    removable_bufs = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    packer.packer_flush.expect("non-null function pointer")(&raw mut packer);
    xfree((*wms).dumped_variables.keys as *mut ::core::ffi::c_void);
    xfree((*wms).dumped_variables.h.hash as *mut ::core::ffi::c_void);
    (*wms).dumped_variables = Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    };
    xfree(wms as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn shada_write_file(
    file: *const ::core::ffi::c_char,
    mut nomerge: bool,
) -> ::core::ffi::c_int {
    let fname: *mut ::core::ffi::c_char = shada_filename(file);
    if fname.is_null() {
        return FAIL;
    }
    let mut tempname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sd_writer: FileDescriptor = FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    };
    let mut sd_reader: FileDescriptor = FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    };
    let mut did_open_writer: bool = false_0 != 0;
    let mut did_open_reader: bool = false_0 != 0;
    's_240: {
        's_163: {
            's_154: {
                if !nomerge {
                    let mut error: ::core::ffi::c_int = 0;
                    error = file_open(
                        &raw mut sd_reader,
                        fname,
                        kFileReadOnly as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    if error != 0 as ::core::ffi::c_int {
                        if error != UV_ENOENT as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    b"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                ),
                                fname,
                                uv_strerror(error),
                            );
                        }
                        nomerge = true_0 != 0;
                        break 's_163;
                    } else {
                        did_open_reader = true_0 != 0;
                        tempname = modname(
                            fname,
                            b".tmp.a\0".as_ptr() as *const ::core::ffi::c_char,
                            false_0 != 0,
                        );
                        if tempname.is_null() {
                            nomerge = true_0 != 0;
                            break 's_163;
                        } else {
                            let mut perm: ::core::ffi::c_int =
                                os_getperm(fname) as ::core::ffi::c_int;
                            perm = if perm >= 0 as ::core::ffi::c_int {
                                perm & 0o777 as ::core::ffi::c_int | 0o600 as ::core::ffi::c_int
                            } else {
                                0o600 as ::core::ffi::c_int
                            };
                            loop {
                                error = file_open(
                                    &raw mut sd_writer,
                                    tempname,
                                    kFileCreateOnly as ::core::ffi::c_int
                                        | kFileNoSymlink as ::core::ffi::c_int,
                                    perm,
                                );
                                if error != 0 {
                                    if error == UV_EEXIST as ::core::ffi::c_int
                                        || error == UV_ELOOP as ::core::ffi::c_int
                                    {
                                        let wp: *mut ::core::ffi::c_char = tempname
                                            .offset(strlen(tempname) as isize)
                                            .offset(-(1 as ::core::ffi::c_int as isize));
                                        if *wp as ::core::ffi::c_int == 'z' as ::core::ffi::c_int {
                                            semsg(
                                                gettext(
                                                    b"E138: All %s.tmp.X files exist, cannot write ShaDa file!\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                fname,
                                            );
                                            xfree(fname as *mut ::core::ffi::c_void);
                                            xfree(tempname as *mut ::core::ffi::c_void);
                                            if did_open_reader {
                                                close_file(&raw mut sd_reader);
                                            }
                                            return FAIL;
                                        }
                                        *wp += 1;
                                    } else {
                                        semsg(
                                            gettext(
                                                b"E886: System error while opening temporary ShaDa file %s for writing: %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            tempname,
                                            uv_strerror(error),
                                        );
                                        break 's_154;
                                    }
                                } else {
                                    did_open_writer = true_0 != 0;
                                    break 's_154;
                                }
                            }
                        }
                    }
                }
            }
            if !nomerge {
                break 's_240;
            }
        }
        let tail: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
        if tail != fname {
            let tail_save: ::core::ffi::c_char = *tail;
            *tail = NUL as ::core::ffi::c_char;
            if !os_isdir(fname) {
                let mut ret: ::core::ffi::c_int = 0;
                let mut failed_dir: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                ret = os_mkdir_recurse(
                    fname,
                    0o700 as int32_t,
                    &raw mut failed_dir,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                );
                if ret != 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            b"E886: Failed to create directory %s for writing ShaDa file: %s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        failed_dir,
                        uv_strerror(ret),
                    );
                    xfree(fname as *mut ::core::ffi::c_void);
                    xfree(failed_dir as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            *tail = tail_save;
        }
        let mut error_0: ::core::ffi::c_int = file_open(
            &raw mut sd_writer,
            fname,
            kFileCreate as ::core::ffi::c_int | kFileTruncate as ::core::ffi::c_int,
            0o600 as ::core::ffi::c_int,
        );
        if error_0 != 0 {
            semsg(
                gettext(
                    b"E886: System error while opening ShaDa file %s for writing: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                fname,
                uv_strerror(error_0),
            );
        } else {
            did_open_writer = true_0 != 0;
        }
    }
    if !did_open_writer {
        xfree(fname as *mut ::core::ffi::c_void);
        xfree(tempname as *mut ::core::ffi::c_void);
        if did_open_reader {
            close_file(&raw mut sd_reader);
        }
        return FAIL;
    }
    if p_verbose.get() > 1 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Writing ShaDa file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            fname,
        );
        verbose_leave();
    }
    let sw_ret: ShaDaWriteResult = shada_write(
        &raw mut sd_writer,
        if nomerge as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<FileDescriptor>()
        } else {
            &raw mut sd_reader
        },
    );
    '_c2rust_label: {
        if sw_ret as ::core::ffi::c_uint
            != kSDWriteIgnError as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"sw_ret != kSDWriteIgnError\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2802 as ::core::ffi::c_uint,
                b"int shada_write_file(const char *const, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !nomerge {
        if did_open_reader {
            close_file(&raw mut sd_reader);
        }
        let mut did_remove: bool = false_0 != 0;
        's_417: {
            '_shada_write_file_did_not_remove: {
                if sw_ret as ::core::ffi::c_uint
                    == kSDWriteSuccessful as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut old_info: FileInfo = FileInfo {
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
                    if !os_fileinfo(fname, &raw mut old_info)
                        || old_info.stat.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t
                        || getuid() != ROOT_UID as __uid_t
                            && (if old_info.stat.st_uid == getuid() as uint64_t {
                                old_info.stat.st_mode & 0o200 as uint64_t
                            } else {
                                if old_info.stat.st_gid == getgid() as uint64_t {
                                    old_info.stat.st_mode & 0o20 as uint64_t
                                } else {
                                    old_info.stat.st_mode & 0o2 as uint64_t
                                }
                            }) == 0
                    {
                        semsg(
                            gettext(b"E137: ShaDa file is not writable: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                        );
                        break '_shada_write_file_did_not_remove;
                    } else {
                        if getuid() == ROOT_UID as __uid_t {
                            if old_info.stat.st_uid != ROOT_UID as uint64_t
                                || old_info.stat.st_gid != getgid() as uint64_t
                            {
                                let old_uid: uv_uid_t = old_info.stat.st_uid as uv_uid_t;
                                let old_gid: uv_gid_t = old_info.stat.st_gid as uv_gid_t;
                                let fchown_ret: ::core::ffi::c_int =
                                    os_fchown(file_fd(&raw mut sd_writer), old_uid, old_gid);
                                if fchown_ret != 0 as ::core::ffi::c_int {
                                    semsg(
                                        gettext(
                                            b"E136: Failed setting uid and gid for file %s: %s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        tempname,
                                        uv_strerror(fchown_ret),
                                    );
                                    break '_shada_write_file_did_not_remove;
                                }
                            }
                        }
                        if vim_rename(tempname, fname) == -1 as ::core::ffi::c_int {
                            semsg(
                                gettext(b"E136: Can't rename ShaDa file from %s to %s!\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                tempname,
                                fname,
                            );
                        } else {
                            did_remove = true_0 != 0;
                            os_remove(tempname);
                        }
                    }
                } else if sw_ret as ::core::ffi::c_uint
                    == kSDWriteReadNotShada as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(
                            b"E136: Did not rename %s because %s does not look like a ShaDa file\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        tempname,
                        fname,
                    );
                } else {
                    semsg(
                        gettext(
                            b"E136: Did not rename %s to %s because there were errors during writing it\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        tempname,
                        fname,
                    );
                }
                if did_remove {
                    break 's_417;
                }
            }
            semsg(
                gettext(
                    b"E136: Do not forget to remove %s or rename it manually to %s.\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                tempname,
                fname,
            );
        }
        xfree(tempname as *mut ::core::ffi::c_void);
    }
    close_file(&raw mut sd_writer);
    xfree(fname as *mut ::core::ffi::c_void);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn shada_read_marks() -> ::core::ffi::c_int {
    return shada_read_file(
        ::core::ptr::null::<::core::ffi::c_char>(),
        kShaDaWantMarks as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn shada_read_everything(
    fname: *const ::core::ffi::c_char,
    forceit: bool,
    missing_ok: bool,
) -> ::core::ffi::c_int {
    return shada_read_file(
        fname,
        kShaDaWantInfo as ::core::ffi::c_int
            | kShaDaWantMarks as ::core::ffi::c_int
            | kShaDaGetOldfiles as ::core::ffi::c_int
            | (if forceit as ::core::ffi::c_int != 0 {
                kShaDaForceit as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if missing_ok as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                kShaDaMissingError as ::core::ffi::c_int
            }),
    );
}
unsafe extern "C" fn shada_free_shada_entry(entry: *mut ShadaEntry) {
    if entry.is_null() || !(*entry).can_free_entry {
        return;
    }
    match (*entry).type_0 as ::core::ffi::c_int {
        -1 => {
            xfree((*entry).data.unknown_item.contents as *mut ::core::ffi::c_void);
        }
        1 => {
            api_free_dict((*entry).data.header);
        }
        11 | 8 | 7 | 10 => {
            xfree((*entry).data.filemark.fname as *mut ::core::ffi::c_void);
        }
        2 => {
            api_free_string((*entry).data.search_pattern.pat);
        }
        5 => {
            let mut i: size_t = 0 as size_t;
            while i < (*entry).data.reg.contents_size {
                api_free_string(*(*entry).data.reg.contents.offset(i as isize));
                i = i.wrapping_add(1);
            }
            xfree((*entry).data.reg.contents as *mut ::core::ffi::c_void);
        }
        4 => {
            xfree((*entry).data.history_item.string as *mut ::core::ffi::c_void);
        }
        6 => {
            xfree((*entry).data.global_var.name as *mut ::core::ffi::c_void);
            tv_clear(&raw mut (*entry).data.global_var.value);
        }
        3 => {
            xfree((*entry).data.sub_string.sub as *mut ::core::ffi::c_void);
        }
        9 => {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*entry).data.buffer_list.size {
                xfree(
                    (*(*entry).data.buffer_list.buffers.offset(i_0 as isize)).fname
                        as *mut ::core::ffi::c_void,
                );
                xfree(
                    (*(*entry).data.buffer_list.buffers.offset(i_0 as isize)).additional_data
                        as *mut ::core::ffi::c_void,
                );
                i_0 = i_0.wrapping_add(1);
            }
            xfree((*entry).data.buffer_list.buffers as *mut ::core::ffi::c_void);
        }
        0 | _ => {}
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*entry).additional_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
}
unsafe extern "C" fn fread_len(
    sd_reader: *mut FileDescriptor,
    buffer: *mut ::core::ffi::c_char,
    length: size_t,
) -> ShaDaReadResult {
    let read_bytes: ptrdiff_t = file_read(sd_reader, buffer, length);
    if read_bytes < 0 as ptrdiff_t {
        semsg(
            gettext(
                b"E886: System error while reading ShaDa file: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_strerror(read_bytes as ::core::ffi::c_int),
        );
        return kSDReadStatusReadError;
    }
    if read_bytes != length as ptrdiff_t {
        semsg(
            gettext(
                b"E576: Error while reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ),
            length as uint64_t,
        );
        return kSDReadStatusNotShaDa;
    }
    return kSDReadStatusSuccess;
}
unsafe extern "C" fn msgpack_read_uint64(
    sd_reader: *mut FileDescriptor,
    mut allow_eof: bool,
    result: *mut uint64_t,
) -> ShaDaReadResult {
    let fpos: uintmax_t = (*sd_reader).bytes_read as uintmax_t;
    let mut ret: uint8_t = 0;
    let mut read_bytes: ptrdiff_t = file_read(
        sd_reader,
        &raw mut ret as *mut ::core::ffi::c_char,
        1 as size_t,
    );
    if read_bytes < 0 as ptrdiff_t {
        semsg(
            gettext(
                b"E886: System error while reading integer from ShaDa file: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_strerror(read_bytes as ::core::ffi::c_int),
        );
        return kSDReadStatusReadError;
    } else if read_bytes == 0 as ptrdiff_t {
        if allow_eof as ::core::ffi::c_int != 0 && file_eof(sd_reader) as ::core::ffi::c_int != 0 {
            return kSDReadStatusFinished;
        }
        semsg(
            gettext(
                b"E576: Error while reading ShaDa file: expected positive integer at position %lu, but got nothing\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ),
            fpos as uint64_t,
        );
        return kSDReadStatusNotShaDa;
    }
    let mut first_char: ::core::ffi::c_int = ret as ::core::ffi::c_int;
    if !first_char & 0x80 as ::core::ffi::c_int != 0 {
        *result = first_char as uint8_t as uint64_t;
    } else {
        let mut length: size_t = 0 as size_t;
        match first_char {
            204 => {
                length = 1 as size_t;
            }
            205 => {
                length = 2 as size_t;
            }
            206 => {
                length = 4 as size_t;
            }
            207 => {
                length = 8 as size_t;
            }
            _ => {
                semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: expected positive integer at position %lu\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    fpos as uint64_t,
                );
                return kSDReadStatusNotShaDa;
            }
        }
        let mut buf: uint64_t = 0 as uint64_t;
        let mut buf_u8: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
        let mut fl_ret: ShaDaReadResult = kSDReadStatusSuccess;
        fl_ret = fread_len(
            sd_reader,
            buf_u8
                .offset(::core::mem::size_of::<uint64_t>().wrapping_sub(length as usize) as isize),
            length,
        );
        if fl_ret as ::core::ffi::c_uint
            != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return fl_ret;
        }
        *result = __bswap_64(buf as __uint64_t) as uint64_t;
    }
    return kSDReadStatusSuccess;
}
unsafe extern "C" fn shada_read_next_item(
    sd_reader: *mut FileDescriptor,
    entry: *mut ShadaEntry,
    flags: ::core::ffi::c_uint,
    max_kbyte: size_t,
) -> ShaDaReadResult {
    let mut verify_but_ignore: bool = false;
    let mut type_u64: uint64_t = 0;
    let mut timestamp_u64: uint64_t = 0;
    let mut length_u64: uint64_t = 0;
    let mut initial_fpos: uint64_t = 0;
    let mut ad: AdditionalDataBuilder = AdditionalDataBuilder {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut read_additional_array_elements: uint32_t = 0;
    let mut error_alloc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mru_ret: ShaDaReadResult = kSDReadStatusSuccess;
    let mut length: size_t = 0;
    let mut parse_pos: uint64_t = 0;
    let mut buf_allocated: bool = false;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut read_ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut read_size: size_t = 0;
    let mut ret: ShaDaReadResult = kSDReadStatusMalformed;
    '_shada_read_next_item_end: {
        '_shada_read_next_item_error: loop {
            memset(
                entry as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<ShadaEntry>(),
            );
            if file_eof(sd_reader) {
                return kSDReadStatusFinished;
            }
            verify_but_ignore = false_0 != 0;
            type_u64 = kSDItemMissing as ::core::ffi::c_int as uint64_t;
            timestamp_u64 = 0;
            length_u64 = 0;
            initial_fpos = (*sd_reader).bytes_read;
            ad = KV_INITIAL_VALUE;
            read_additional_array_elements = 0 as uint32_t;
            error_alloc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            mru_ret = kSDReadStatusSuccess;
            mru_ret = msgpack_read_uint64(sd_reader, true_0 != 0, &raw mut type_u64);
            if mru_ret as ::core::ffi::c_uint
                != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    mru_ret = msgpack_read_uint64(sd_reader, false_0 != 0, &raw mut timestamp_u64);
                    mru_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                }
                || {
                    mru_ret = msgpack_read_uint64(sd_reader, false_0 != 0, &raw mut length_u64);
                    mru_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                }
            {
                return mru_ret;
            }
            if length_u64 > PTRDIFF_MAX as uint64_t {
                semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: there is an item at position %lu that is stated to be too long\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    initial_fpos,
                );
                return kSDReadStatusNotShaDa;
            }
            length = length_u64 as size_t;
            (*entry).timestamp = timestamp_u64;
            (*entry).can_free_entry = true_0 != 0;
            if type_u64 == 0 as uint64_t {
                semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: there is an item at position %lu that must not be there: Missing items are for internal uses only\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    initial_fpos,
                );
                return kSDReadStatusNotShaDa;
            }
            if (if type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t {
                (flags & kSDReadUnknown as ::core::ffi::c_int as ::core::ffi::c_uint == 0)
                    as ::core::ffi::c_int
            } else {
                (((1 as ::core::ffi::c_int) << type_u64) as ::core::ffi::c_uint & flags == 0)
                    as ::core::ffi::c_int
            }) != 0
                || max_kbyte != 0 && length > max_kbyte.wrapping_mul(1024 as size_t)
            {
                if initial_fpos == 0 as uint64_t
                    && (type_u64 == '\n' as uint64_t
                        || type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t)
                {
                    verify_but_ignore = true_0 != 0;
                } else {
                    let srs_ret: ShaDaReadResult = sd_reader_skip(sd_reader, length);
                    if srs_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return srs_ret;
                    }
                    continue;
                }
            }
            parse_pos = (*sd_reader).bytes_read;
            buf_allocated = false_0 != 0;
            buf = file_try_read_buffered(sd_reader, length);
            if buf.is_null() {
                buf_allocated = true_0 != 0;
                buf = xmalloc(length) as *mut ::core::ffi::c_char;
                let fl_ret: ShaDaReadResult = fread_len(sd_reader, buf, length);
                if fl_ret as ::core::ffi::c_uint
                    != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = fl_ret;
                    break;
                }
            }
            read_ptr = buf;
            read_size = length;
            if verify_but_ignore {
                let mut status: ::core::ffi::c_int =
                    unpack_skip(&raw mut read_ptr, &raw mut read_size);
                let mut spm_ret: ShaDaReadResult =
                    shada_check_status(parse_pos as uintmax_t, status, read_size);
                if buf_allocated {
                    xfree(buf as *mut ::core::ffi::c_void);
                }
                if spm_ret as ::core::ffi::c_uint
                    != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return spm_ret;
                }
            } else {
                if type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t {
                    (*entry).type_0 = kSDItemUnknown;
                    (*entry).data.unknown_item.size = length;
                    (*entry).data.unknown_item.type_0 = type_u64;
                    if initial_fpos == 0 as uint64_t {
                        let mut status_0: ::core::ffi::c_int =
                            unpack_skip(&raw mut read_ptr, &raw mut read_size);
                        let mut spm_ret_0: ShaDaReadResult =
                            shada_check_status(parse_pos as uintmax_t, status_0, read_size);
                        if spm_ret_0 as ::core::ffi::c_uint
                            != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if buf_allocated {
                                xfree(buf as *mut ::core::ffi::c_void);
                            }
                            (*entry).type_0 = kSDItemMissing;
                            return spm_ret_0;
                        }
                    }
                    (*entry).data.unknown_item.contents =
                        (if buf_allocated as ::core::ffi::c_int != 0 {
                            buf as *mut ::core::ffi::c_void
                        } else {
                            xmemdup(buf as *const ::core::ffi::c_void, length)
                        }) as *mut ::core::ffi::c_char;
                    return kSDReadStatusSuccess;
                }
                (*entry).data = (*sd_default_values.ptr())[type_u64 as usize].data;
                's_900: {
                    match type_u64 as ShadaEntryType as ::core::ffi::c_int {
                        2 => {
                            let mut it: *mut KeyDict__shada_search_pat =
                                &raw mut (*entry).data.search_pattern;
                            if !unpack_keydict(
                                it as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_search_pat_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: search pattern entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                (*it).pat = NULL_STRING;
                                break '_shada_read_next_item_error;
                            } else if !((*it).is_set___shada_search_pat_
                                as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX__shada_search_pat__sp
                                != 0 as ::core::ffi::c_ulonglong)
                            {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: search pattern entry at position %lu has no pattern\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                (*entry).data.search_pattern.pat = copy_string(
                                    (*entry).data.search_pattern.pat,
                                    ::core::ptr::null_mut::<Arena>(),
                                );
                            }
                        }
                        11 | 8 | 7 | 10 => {
                            let mut it_0: KeyDict__shada_mark = KeyDict__shada_mark {
                                is_set___shada_mark_: 0 as OptionalKeys,
                                n: 0,
                                l: 0,
                                c: 0,
                                f: String_0 {
                                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    size: 0,
                                },
                            };
                            if !unpack_keydict(
                                &raw mut it_0 as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_mark_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: mark entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__n
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if type_u64 == kSDItemJump as ::core::ffi::c_int as uint64_t
                                        || type_u64
                                            == kSDItemChange as ::core::ffi::c_int as uint64_t
                                    {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: mark entry at position %lu has n key which is only valid for local and global mark entries\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else {
                                        (*entry).data.filemark.name = it_0.n as ::core::ffi::c_char;
                                    }
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__l
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.mark.lnum = it_0.l as linenr_T;
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__c
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.mark.col = it_0.c as colnr_T;
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__f
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.fname = xmemdupz(
                                        it_0.f.data as *const ::core::ffi::c_void,
                                        it_0.f.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                                if (*entry).data.filemark.fname.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu is missing file name\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else if (*entry).data.filemark.mark.lnum <= 0 as linenr_T {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu has invalid line number\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else if (*entry).data.filemark.mark.col < 0 as ::core::ffi::c_int
                                {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu has invalid column number\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                }
                            }
                        }
                        5 => {
                            let mut it_1: KeyDict__shada_register = KeyDict__shada_register {
                                is_set___shada_register_: 0 as OptionalKeys,
                                rc: StringArray {
                                    size: 0,
                                    capacity: 0,
                                    items: ::core::ptr::null_mut::<String_0>(),
                                },
                                ru: false,
                                rt: 0,
                                n: 0,
                                rw: 0,
                            };
                            if !unpack_keydict(
                                &raw mut it_1 as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_register_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: register entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                xfree(it_1.rc.items as *mut ::core::ffi::c_void);
                                it_1.rc.capacity = 0 as size_t;
                                it_1.rc.size = it_1.rc.capacity;
                                it_1.rc.items = ::core::ptr::null_mut::<String_0>();
                                break '_shada_read_next_item_error;
                            } else if it_1.rc.size == 0 as size_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: register entry at position %lu has rc key with missing or empty array\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                (*entry).data.reg.contents_size = it_1.rc.size;
                                (*entry).data.reg.contents = xmalloc(
                                    it_1.rc
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<String_0>()),
                                )
                                    as *mut String_0;
                                let mut j: size_t = 0 as size_t;
                                while j < it_1.rc.size {
                                    *(*entry).data.reg.contents.offset(j as isize) = copy_string(
                                        *it_1.rc.items.offset(j as isize),
                                        ::core::ptr::null_mut::<Arena>(),
                                    );
                                    j = j.wrapping_add(1);
                                }
                                xfree(it_1.rc.items as *mut ::core::ffi::c_void);
                                it_1.rc.capacity = 0 as size_t;
                                it_1.rc.size = it_1.rc.capacity;
                                it_1.rc.items = ::core::ptr::null_mut::<String_0>();
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__ru
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.is_unnamed = it_1.ru;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__rt
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.type_0 = it_1.rt as uint8_t as MotionType;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__n
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.name = it_1.n as ::core::ffi::c_char;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__rw
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.width = it_1.rw as size_t;
                                }
                            }
                        }
                        4 => {
                            let mut len: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len < 2 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: history entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut hist_type: Integer = 0;
                                if !unpack_integer(
                                    &raw mut read_ptr,
                                    &raw mut read_size,
                                    &raw mut hist_type,
                                ) {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: history entry at position %lu has wrong history type type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    let item: String_0 =
                                        unpack_string(&raw mut read_ptr, &raw mut read_size);
                                    if item.data.is_null() {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: history entry at position %lu has wrong history string type\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else if !memchr(
                                        item.data as *const ::core::ffi::c_void,
                                        0 as ::core::ffi::c_int,
                                        item.size,
                                    )
                                    .is_null()
                                    {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: history entry at position %lu contains string with zero byte inside\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else {
                                        (*entry).data.history_item.histtype = hist_type as uint8_t;
                                        let is_hist_search: bool =
                                            (*entry).data.history_item.histtype
                                                as ::core::ffi::c_int
                                                == HIST_SEARCH as ::core::ffi::c_int;
                                        if is_hist_search {
                                            if len < 3 as ssize_t {
                                                semsg(
                                                    gettext(
                                                        b"E575: Error while reading ShaDa file: search history entry at position %lu does not have separator character\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    initial_fpos,
                                                );
                                                break '_shada_read_next_item_error;
                                            } else {
                                                let mut sep_type: Integer = 0;
                                                if !unpack_integer(
                                                    &raw mut read_ptr,
                                                    &raw mut read_size,
                                                    &raw mut sep_type,
                                                ) {
                                                    semsg(
                                                        gettext(
                                                            b"E575: Error while reading ShaDa file: search history entry at position %lu has wrong history separator type\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        initial_fpos,
                                                    );
                                                    break '_shada_read_next_item_error;
                                                } else {
                                                    (*entry).data.history_item.sep =
                                                        sep_type as ::core::ffi::c_char;
                                                }
                                            }
                                        }
                                        let mut strsize: size_t = item
                                            .size
                                            .wrapping_add(1 as size_t)
                                            .wrapping_add(1 as size_t);
                                        (*entry).data.history_item.string =
                                            xmalloc(strsize) as *mut ::core::ffi::c_char;
                                        memcpy(
                                            (*entry).data.history_item.string
                                                as *mut ::core::ffi::c_void,
                                            item.data as *const ::core::ffi::c_void,
                                            item.size,
                                        );
                                        *(*entry)
                                            .data
                                            .history_item
                                            .string
                                            .offset(strsize.wrapping_sub(2 as size_t) as isize) =
                                            0 as ::core::ffi::c_char;
                                        *(*entry)
                                            .data
                                            .history_item
                                            .string
                                            .offset(strsize.wrapping_sub(1 as size_t) as isize) =
                                            (*entry).data.history_item.sep;
                                        read_additional_array_elements = (len
                                            - (2 as ::core::ffi::c_int
                                                + is_hist_search as ::core::ffi::c_int)
                                                as ssize_t)
                                            as uint32_t;
                                    }
                                }
                            }
                        }
                        6 => {
                            let mut len_0: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_0 < 2 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: variable entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut name: String_0 =
                                    unpack_string(&raw mut read_ptr, &raw mut read_size);
                                if name.data.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable name type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    (*entry).data.global_var.name = xmemdupz(
                                        name.data as *const ::core::ffi::c_void,
                                        name.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    let mut binval: String_0 =
                                        unpack_string(&raw mut read_ptr, &raw mut read_size);
                                    let mut is_blob: bool = false_0 != 0;
                                    if !binval.data.is_null() {
                                        if len_0 > 2 as ssize_t {
                                            let mut type_0: Integer = 0;
                                            if !unpack_integer(
                                                &raw mut read_ptr,
                                                &raw mut read_size,
                                                &raw mut type_0,
                                            ) || type_0
                                                != VAR_TYPE_BLOB as ::core::ffi::c_int as Integer
                                            {
                                                semsg(
                                                    gettext(
                                                        b"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable type\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    initial_fpos,
                                                );
                                                break '_shada_read_next_item_error;
                                            } else {
                                                is_blob = true_0 != 0;
                                            }
                                        }
                                        (*entry).data.global_var.value = decode_string(
                                            binval.data,
                                            binval.size,
                                            is_blob,
                                            false_0 != 0,
                                        );
                                    } else {
                                        let mut status_1: ::core::ffi::c_int = unpack_typval(
                                            &raw mut read_ptr,
                                            &raw mut read_size,
                                            &raw mut (*entry).data.global_var.value,
                                        );
                                        if status_1 != MPACK_OK as ::core::ffi::c_int {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: variable entry at position %lu has value that cannot be converted to the Vimscript value\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        }
                                    }
                                    read_additional_array_elements = (len_0
                                        - 2 as ssize_t
                                        - (if is_blob as ::core::ffi::c_int != 0 {
                                            1 as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }) as ssize_t)
                                        as uint32_t;
                                }
                            }
                        }
                        3 => {
                            let mut len_1: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_1 < 1 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: sub string entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut sub: String_0 =
                                    unpack_string(&raw mut read_ptr, &raw mut read_size);
                                if sub.data.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: sub string entry at position %lu has wrong sub string type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    (*entry).data.sub_string.sub =
                                        xmemdupz(sub.data as *const ::core::ffi::c_void, sub.size)
                                            as *mut ::core::ffi::c_char;
                                    read_additional_array_elements =
                                        (len_1 - 1 as ssize_t) as uint32_t;
                                }
                            }
                        }
                        9 => {
                            let mut len_2: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_2 < 0 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: buffer list entry at position %lu is not an array\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else if len_2 != 0 as ssize_t {
                                (*entry).data.buffer_list.buffers = xcalloc(
                                    len_2 as size_t,
                                    ::core::mem::size_of::<buffer_list_buffer>(),
                                )
                                    as *mut buffer_list_buffer;
                                let mut i: size_t = 0 as size_t;
                                loop {
                                    if i >= len_2 as size_t {
                                        break 's_900;
                                    }
                                    (*entry).data.buffer_list.size =
                                        (*entry).data.buffer_list.size.wrapping_add(1);
                                    let mut it_2: KeyDict__shada_buflist_item =
                                        KeyDict__shada_buflist_item {
                                            is_set___shada_buflist_item_: 0 as OptionalKeys,
                                            l: 0,
                                            c: 0,
                                            f: String_0 {
                                                data: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                size: 0,
                                            },
                                        };
                                    let mut it_ad: AdditionalDataBuilder = KV_INITIAL_VALUE;
                                    if !unpack_keydict(
                                        &raw mut it_2 as *mut ::core::ffi::c_void,
                                        Some(
                                            KeyDict__shada_buflist_item_get_field
                                                as unsafe extern "C" fn(
                                                    *const ::core::ffi::c_char,
                                                    size_t,
                                                )
                                                    -> *mut KeySetLink,
                                        ),
                                        &raw mut it_ad,
                                        &raw mut read_ptr,
                                        &raw mut read_size,
                                        &raw mut error_alloc,
                                    ) {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                            error_alloc,
                                        );
                                        xfree(it_ad.items as *mut ::core::ffi::c_void);
                                        it_ad.capacity = 0 as size_t;
                                        it_ad.size = it_ad.capacity;
                                        it_ad.items =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        break '_shada_read_next_item_error;
                                    } else {
                                        let mut e: *mut buffer_list_buffer =
                                            (*entry).data.buffer_list.buffers.offset(i as isize)
                                                as *mut buffer_list_buffer;
                                        (*e).additional_data = it_ad.items as *mut AdditionalData;
                                        (*e).pos = default_pos.get();
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__l
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).pos.lnum = it_2.l as linenr_T;
                                        }
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__c
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).pos.col = it_2.c as colnr_T;
                                        }
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__f
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).fname = xmemdupz(
                                                it_2.f.data as *const ::core::ffi::c_void,
                                                it_2.f.size,
                                            )
                                                as *mut ::core::ffi::c_char;
                                        }
                                        if (*e).pos.lnum <= 0 as linenr_T {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid line number\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else if (*e).pos.col < 0 as ::core::ffi::c_int {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid column number\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else if (*e).fname.is_null() {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that does not have a file name\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else {
                                            i = i.wrapping_add(1);
                                        }
                                    }
                                }
                            }
                        }
                        0 | -1 => {
                            abort();
                        }
                        1 | _ => {}
                    }
                }
                let mut i_0: uint32_t = 0 as uint32_t;
                while i_0 < read_additional_array_elements {
                    let mut item_start: *const ::core::ffi::c_char = read_ptr;
                    let mut status_2: ::core::ffi::c_int =
                        unpack_skip(&raw mut read_ptr, &raw mut read_size);
                    if status_2 != 0 {
                        break '_shada_read_next_item_error;
                    }
                    push_additional_data(
                        &raw mut ad,
                        item_start,
                        read_ptr.offset_from(item_start) as size_t,
                    );
                    i_0 = i_0.wrapping_add(1);
                }
                if read_size != 0 {
                    semsg(
                        gettext(
                            b"E575: Error while reading ShaDa file: item entry at position %lu additional bytes\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        initial_fpos,
                    );
                    break;
                } else {
                    (*entry).type_0 = type_u64 as ShadaEntryType;
                    (*entry).additional_data = ad.items as *mut AdditionalData;
                    ret = kSDReadStatusSuccess;
                    break '_shada_read_next_item_end;
                }
            }
        }
        (*entry).type_0 = type_u64 as ShadaEntryType;
        shada_free_shada_entry(entry);
        (*entry).type_0 = kSDItemMissing;
        xfree(error_alloc as *mut ::core::ffi::c_void);
        xfree(ad.items as *mut ::core::ffi::c_void);
        ad.capacity = 0 as size_t;
        ad.size = ad.capacity;
        ad.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if buf_allocated {
        xfree(buf as *mut ::core::ffi::c_void);
    }
    return ret;
}
unsafe extern "C" fn shada_removable(mut name: *const ::core::ffi::c_char) -> bool {
    let mut part: [::core::ffi::c_char; 4097] = [0; 4097];
    let mut retval: bool = false_0 != 0;
    let mut new_name: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
    let mut p: *mut ::core::ffi::c_char = p_shada.get();
    while *p != 0 {
        copy_option_part(
            &raw mut p,
            &raw mut part as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4097]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 4097]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            b", \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if part[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 'r' as ::core::ffi::c_int
        {
            continue;
        }
        home_replace(
            ::core::ptr::null::<buf_T>(),
            (&raw mut part as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        let mut n: size_t = strlen(NameBuff.ptr() as *mut ::core::ffi::c_char);
        if mb_strnicmp(NameBuff.ptr() as *mut ::core::ffi::c_char, new_name, n)
            != 0 as ::core::ffi::c_int
        {
            continue;
        }
        retval = true_0 != 0;
        break;
    }
    xfree(new_name as *mut ::core::ffi::c_void);
    return retval;
}
#[inline]
unsafe extern "C" fn shada_init_jumps(
    mut jumps: *mut ShadaEntry,
    removable_bufs: *mut Set_ptr_t,
) -> size_t {
    let mut jumps_size: size_t = 0 as size_t;
    let mut jump_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    setpcmark();
    cleanup_jumplist(curwin.get(), false_0 != 0);
    loop {
        let mut fm: xfmark_T = xfmark_T {
            fmark: fmark_T {
                mark: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                fnum: 0,
                timestamp: 0,
                view: fmarkv_T {
                    topline_offset: 0,
                    skipcol: 0,
                },
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            },
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        jump_iter = mark_jumplist_iter(jump_iter, curwin.get(), &raw mut fm);
        if fm.fmark.mark.lnum == 0 as linenr_T {
            siemsg(
                b"ShaDa: mark lnum zero (ji:%p, js:%p, len:%i)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                jump_iter as *mut ::core::ffi::c_void,
                (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (*curwin.get()).w_jumplistlen,
            );
        } else {
            let buf: *const buf_T = if fm.fmark.fnum == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<buf_T>()
            } else {
                buflist_findnr(fm.fmark.fnum)
            };
            if if !buf.is_null() {
                ignore_buf(buf, removable_bufs) as ::core::ffi::c_int
            } else {
                (fm.fmark.fnum != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } == 0
            {
                let fname: *const ::core::ffi::c_char = if fm.fmark.fnum == 0 as ::core::ffi::c_int
                {
                    if fm.fname.is_null() {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    } else {
                        fm.fname
                    }
                } else if !buf.is_null() {
                    (*buf).b_ffname
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                };
                if !fname.is_null() {
                    let c2rust_fresh21 = jumps_size;
                    jumps_size = jumps_size.wrapping_add(1);
                    *jumps.offset(c2rust_fresh21 as isize) = ShadaEntry {
                        type_0: kSDItemJump,
                        can_free_entry: false_0 != 0,
                        timestamp: fm.fmark.timestamp,
                        data: C2Rust_Unnamed_22 {
                            filemark: shada_filemark {
                                name: NUL as ::core::ffi::c_char,
                                mark: fm.fmark.mark,
                                fname: fname as *mut ::core::ffi::c_char,
                            },
                        },
                        additional_data: fm.fmark.additional_data,
                    };
                }
            }
        }
        if jump_iter.is_null() {
            break;
        }
    }
    return jumps_size;
}
#[no_mangle]
pub unsafe extern "C" fn shada_encode_regs() -> String_0 {
    let wms: *mut WriteMergerState =
        xcalloc(1 as size_t, ::core::mem::size_of::<WriteMergerState>()) as *mut WriteMergerState;
    shada_initialize_registers(wms, -1 as ::core::ffi::c_int);
    let mut packer: PackerBuffer = packer_string_buffer();
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[ShadaEntry; 37]>()
        .wrapping_div(::core::mem::size_of::<ShadaEntry>())
        .wrapping_div(
            (::core::mem::size_of::<[ShadaEntry; 37]>()
                .wrapping_rem(::core::mem::size_of::<ShadaEntry>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        if (*wms).registers[i as usize].type_0 as ::core::ffi::c_int
            == kSDItemRegister as ::core::ffi::c_int
        {
            if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                == shada_pack_pfreed_entry(
                    &raw mut packer,
                    (*wms).registers[i as usize],
                    0 as size_t,
                ) as ::core::ffi::c_uint
            {
                abort();
            }
        }
        i = i.wrapping_add(1);
    }
    xfree(wms as *mut ::core::ffi::c_void);
    return packer_take_string(&raw mut packer);
}
#[no_mangle]
pub unsafe extern "C" fn shada_encode_jumps() -> String_0 {
    let mut removable_bufs: Set_ptr_t = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    find_removable_bufs(&raw mut removable_bufs);
    let mut jumps: [ShadaEntry; 100] = [ShadaEntry {
        type_0: kSDItemMissing,
        can_free_entry: false,
        timestamp: 0,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    }; 100];
    let mut jumps_size: size_t =
        shada_init_jumps(&raw mut jumps as *mut ShadaEntry, &raw mut removable_bufs);
    let mut packer: PackerBuffer = packer_string_buffer();
    let mut i: size_t = 0 as size_t;
    while i < jumps_size {
        if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
            == shada_pack_pfreed_entry(&raw mut packer, jumps[i as usize], 0 as size_t)
                as ::core::ffi::c_uint
        {
            abort();
        }
        i = i.wrapping_add(1);
    }
    return packer_take_string(&raw mut packer);
}
#[no_mangle]
pub unsafe extern "C" fn shada_encode_buflist() -> String_0 {
    let mut removable_bufs: Set_ptr_t = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    find_removable_bufs(&raw mut removable_bufs);
    let mut buflist_entry: ShadaEntry = shada_get_buflist(&raw mut removable_bufs);
    let mut packer: PackerBuffer = packer_string_buffer();
    if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
        == shada_pack_entry(&raw mut packer, buflist_entry, 0 as size_t) as ::core::ffi::c_uint
    {
        abort();
    }
    xfree(buflist_entry.data.buffer_list.buffers as *mut ::core::ffi::c_void);
    return packer_take_string(&raw mut packer);
}
#[no_mangle]
pub unsafe extern "C" fn shada_encode_gvars() -> String_0 {
    let mut packer: PackerBuffer = packer_string_buffer();
    let mut var_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let cur_timestamp: Timestamp = os_time();
    loop {
        let mut vartv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        var_iter = var_shada_iter(
            var_iter,
            &raw mut name,
            &raw mut vartv,
            (VAR_FLAVOUR_DEFAULT as ::core::ffi::c_int
                | VAR_FLAVOUR_SESSION as ::core::ffi::c_int
                | VAR_FLAVOUR_SHADA as ::core::ffi::c_int) as var_flavour_T,
        );
        if name.is_null() {
            break;
        }
        if vartv.v_type as ::core::ffi::c_uint
            != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            && vartv.v_type as ::core::ffi::c_uint
                != VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tgttv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv_copy(&raw mut vartv, &raw mut tgttv);
            let mut r: ShaDaWriteResult = shada_pack_entry(
                &raw mut packer,
                ShadaEntry {
                    type_0: kSDItemVariable,
                    can_free_entry: false,
                    timestamp: cur_timestamp,
                    data: C2Rust_Unnamed_22 {
                        global_var: global_var {
                            name: name as *mut ::core::ffi::c_char,
                            value: tgttv,
                        },
                    },
                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                },
                0 as size_t,
            );
            if kSDWriteFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                == r as ::core::ffi::c_uint
            {
                abort();
            }
            tv_clear(&raw mut tgttv);
        }
        tv_clear(&raw mut vartv);
        if var_iter.is_null() {
            break;
        }
    }
    return packer_take_string(&raw mut packer);
}
#[no_mangle]
pub unsafe extern "C" fn shada_read_string(mut string: String_0, flags: ::core::ffi::c_int) {
    if string.size == 0 as size_t {
        return;
    }
    let mut sd_reader: FileDescriptor = FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    };
    file_open_buffer(&raw mut sd_reader, string.data, string.size);
    shada_read(&raw mut sd_reader, flags);
    close_file(&raw mut sd_reader);
}
#[no_mangle]
pub unsafe extern "C" fn get_shada_parameter(mut type_0: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = find_shada_parameter(type_0);
    if !p.is_null() && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        return atoi(p);
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn find_shada_parameter(
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = p_shada.get();
    while *p != 0 {
        if *p as ::core::ffi::c_int == type_0 {
            return p.offset(1 as ::core::ffi::c_int as isize);
        }
        if *p as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            break;
        } else {
            p = vim_strchr(p, ',' as ::core::ffi::c_int);
            if p.is_null() {
                break;
            }
            p = p.offset(1);
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn check_marks_read() {
    if !(*curbuf.get()).b_marks_read
        && get_shada_parameter('\'' as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
        && !(*curbuf.get()).b_ffname.is_null()
    {
        shada_read_marks();
    }
    (*curbuf.get()).b_marks_read = true_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
