use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
    fn feof(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn time(__timer: *mut time_t) -> time_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_arglist_exp(
        str: *mut ::core::ffi::c_char,
        fcountp: *mut ::core::ffi::c_int,
        fnamesp: *mut *mut *mut ::core::ffi::c_char,
        wig: bool,
    ) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_findname_exp(fname: *mut ::core::ffi::c_char) -> *mut buf_T;
    static p_enc: GlobalCell<*mut ::core::ffi::c_char>;
    static p_msm: GlobalCell<*mut ::core::ffi::c_char>;
    static p_verbose: GlobalCell<OptInt>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn has_non_ascii(s: *const ::core::ffi::c_char) -> bool;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    static e_exists: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_isadir2: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    static e_write: [::core::ffi::c_char; 0];
    static e_bufloaded: [::core::ffi::c_char; 0];
    static e_notset: [::core::ffi::c_char; 0];
    fn vim_fgets(buf: *mut ::core::ffi::c_char, size: ::core::ffi::c_int, fp: *mut FILE) -> bool;
    fn get2c(fd: *mut FILE) -> ::core::ffi::c_int;
    fn get3c(fd: *mut FILE) -> ::core::ffi::c_int;
    fn get4c(fd: *mut FILE) -> ::core::ffi::c_int;
    fn get8ctime(fd: *mut FILE) -> time_t;
    fn read_string(fd: *mut FILE, cnt: size_t) -> *mut ::core::ffi::c_char;
    fn put_bytes(fd: *mut FILE, number: uintmax_t, len: size_t) -> bool;
    fn put_time(fd: *mut FILE, time_: time_t) -> ::core::ffi::c_int;
    fn buf_reload(buf: *mut buf_T, orig_mode: ::core::ffi::c_int, reload_options: bool);
    fn vim_tempname() -> *mut ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static msg_didout: GlobalCell<bool>;
    static curwin: GlobalCell<*mut win_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static IObuff: GlobalCell<[::core::ffi::c_char; 1025]>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static got_int: GlobalCell<bool>;
    static hash_removed: ::core::ffi::c_char;
    fn hash_init(ht: *mut hashtab_T);
    fn hash_clear(ht: *mut hashtab_T);
    fn hash_clear_all(ht: *mut hashtab_T, off: ::core::ffi::c_uint);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_lookup(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        hash: hash_T,
    ) -> *mut hashitem_T;
    fn hash_add(ht: *mut hashtab_T, key: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn hash_add_item(
        ht: *mut hashtab_T,
        hi: *mut hashitem_T,
        key: *mut ::core::ffi::c_char,
        hash: hash_T,
    );
    fn hash_hash(key: *const ::core::ffi::c_char) -> hash_T;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_ptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_cptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_toupper(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utf_valid_string(s: *const ::core::ffi::c_char, end: *const ::core::ffi::c_char) -> bool;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn enc_canonize(enc: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn convert_setup(
        vcp: *mut vimconv_T,
        from: *mut ::core::ffi::c_char,
        to: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn string_convert(
        vcp: *const vimconv_T,
        ptr: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_start();
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_outtrans_long(longstr: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int);
    fn msg_clr_eos();
    fn verbose_enter();
    fn verbose_leave();
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_mkdir(path: *const ::core::ffi::c_char, mode: int32_t) -> ::core::ffi::c_int;
    fn os_mkdir_recurse(
        dir: *const ::core::ffi::c_char,
        mode: int32_t,
        failed_dir: *mut *mut ::core::ffi::c_char,
        created: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn line_breakcheck();
    fn fast_breakcheck();
    fn veryfast_breakcheck();
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn get_xdg_home(idx: XDGVarType) -> *mut ::core::ffi::c_char;
    fn os_time() -> Timestamp;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_prog(
        prog: *mut *mut regprog_T,
        ignore_case: bool,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn dir_of_file_exists(fname: *mut ::core::ffi::c_char) -> bool;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn estack_push(
        type_0: etype_T,
        name: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut estack_T;
    fn estack_pop();
    static first_lang: GlobalCell<*mut slang_T>;
    static int_wordlist: GlobalCell<*mut ::core::ffi::c_char>;
    static spelltab: GlobalCell<spelltab_T>;
    static did_set_spelltab: GlobalCell<bool>;
    static e_format: GlobalCell<*mut ::core::ffi::c_char>;
    fn spell_enc() -> *mut ::core::ffi::c_char;
    fn slang_alloc(lang: *mut ::core::ffi::c_char) -> *mut slang_T;
    fn slang_free(lp: *mut slang_T);
    fn slang_clear(lp: *mut slang_T);
    fn slang_clear_sug(lp: *mut slang_T);
    fn count_common_word(
        lp: *mut slang_T,
        word: *mut ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        count: uint8_t,
    );
    fn byte_in_str(str: *mut uint8_t, n: ::core::ffi::c_int) -> bool;
    fn init_syl_tab(slang: *mut slang_T) -> ::core::ffi::c_int;
    fn parse_spelllang(wp: *mut win_T) -> *mut ::core::ffi::c_char;
    fn captype(
        word: *const ::core::ffi::c_char,
        end: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn open_spellbuf() -> *mut buf_T;
    fn close_spellbuf(buf: *mut buf_T);
    fn clear_spell_chartab(sp: *mut spelltab_T);
    fn init_spell_chartab();
    fn spell_casefold(
        wp: *const win_T,
        str: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_char,
        buflen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn onecap_copy(word: *const ::core::ffi::c_char, wcopy: *mut ::core::ffi::c_char, upper: bool);
    fn spell_soundfold(
        slang: *mut slang_T,
        inword: *mut ::core::ffi::c_char,
        folded: bool,
        res: *mut ::core::ffi::c_char,
    );
    fn ui_flush();
    fn bufIsChanged(buf: *mut buf_T) -> bool;
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type uintmax_t = ::libc::uintmax_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
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
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
pub type linenr_T = int32_t;
pub type colnr_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
    pub b_wininfo: C2Rust_Unnamed_11,
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
    pub b_signcols: C2Rust_Unnamed_3,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_1,
    pub update_callbacks: C2Rust_Unnamed_0,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
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
pub struct C2Rust_Unnamed_1 {
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
    pub data: C2Rust_Unnamed_2,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
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
pub struct C2Rust_Unnamed_3 {
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
pub type disptick_T = uint64_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_4,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_5,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub fc_fixvar: [C2Rust_Unnamed_6; 12],
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
pub struct C2Rust_Unnamed_6 {
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
    pub uh_next: C2Rust_Unnamed_10,
    pub uh_prev: C2Rust_Unnamed_9,
    pub uh_alt_next: C2Rust_Unnamed_8,
    pub uh_alt_prev: C2Rust_Unnamed_7,
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
pub union C2Rust_Unnamed_7 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
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
pub struct C2Rust_Unnamed_11 {
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
    pub type_0: C2Rust_Unnamed_12,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_13,
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
pub type except_T = vim_exception;
pub type CMD_index = ::core::ffi::c_int;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub type cmdidx_T = CMD_index;
pub type cmd_addr_T = ::core::ffi::c_uint;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exarg {
    pub arg: *mut ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub arglens: *mut size_t,
    pub argc: size_t,
    pub nextcmd: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub cmdlinep: *mut *mut ::core::ffi::c_char,
    pub cmdline_tofree: *mut ::core::ffi::c_char,
    pub cmdidx: cmdidx_T,
    pub argt: uint32_t,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: cmd_addr_T,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: linenr_T,
    pub append: ::core::ffi::c_int,
    pub usefilter: ::core::ffi::c_int,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: ::core::ffi::c_int,
    pub mkdir_p: ::core::ffi::c_int,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    pub errmsg: *mut ::core::ffi::c_char,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut cstack_T,
}
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
pub type exarg_T = exarg;
pub type Boolean = bool;
pub type Integer = int64_t;
pub type Float = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
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
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
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
pub type OptIndex = ::core::ffi::c_int;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_15 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_15 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_15 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_15 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_15 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_15 = 20;
pub const UPD_VALID: C2Rust_Unnamed_15 = 10;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type auto_event = ::core::ffi::c_uint;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
pub type event_T = auto_event;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub type AutoPatCmd = AutoPatCmd_S;
pub type etype_T = ::core::ffi::c_uint;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_16 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_17 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_17 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_17 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_17 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_17 = 1;
pub const CONV_NONE: C2Rust_Unnamed_17 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_18 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_18 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_18 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_18 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_18 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_18 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_18 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_18 = 1;
pub type XDGVarType = ::core::ffi::c_int;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const MAXWLEN: C2Rust_Unnamed_19 = 254;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAXREGIONS: C2Rust_Unnamed_20 = 8;
pub type idx_T = ::core::ffi::c_int;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const WF_CAPMASK: C2Rust_Unnamed_21 = 198;
pub const WF_KEEPCAP: C2Rust_Unnamed_21 = 128;
pub const WF_FIXCAP: C2Rust_Unnamed_21 = 64;
pub const WF_AFX: C2Rust_Unnamed_21 = 32;
pub const WF_BANNED: C2Rust_Unnamed_21 = 16;
pub const WF_RARE: C2Rust_Unnamed_21 = 8;
pub const WF_ALLCAP: C2Rust_Unnamed_21 = 4;
pub const WF_ONECAP: C2Rust_Unnamed_21 = 2;
pub const WF_REGION: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const WF_NOCOMPAFT: C2Rust_Unnamed_22 = 8192;
pub const WF_NOCOMPBEF: C2Rust_Unnamed_22 = 4096;
pub const WF_COMPROOT: C2Rust_Unnamed_22 = 2048;
pub const WF_NOSUGGEST: C2Rust_Unnamed_22 = 1024;
pub const WF_NEEDCOMP: C2Rust_Unnamed_22 = 512;
pub const WF_HAS_AFF: C2Rust_Unnamed_22 = 256;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const WFP_COMPFORBID: C2Rust_Unnamed_23 = 16;
pub const WFP_COMPPERMIT: C2Rust_Unnamed_23 = 8;
pub const WFP_UP: C2Rust_Unnamed_23 = 4;
pub const WFP_NC: C2Rust_Unnamed_23 = 2;
pub const WFP_RARE: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const COMP_CHECKTRIPLE: C2Rust_Unnamed_24 = 8;
pub const COMP_CHECKCASE: C2Rust_Unnamed_24 = 4;
pub const COMP_CHECKREP: C2Rust_Unnamed_24 = 2;
pub const COMP_CHECKDUP: C2Rust_Unnamed_24 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fromto_T {
    pub ft_from: *mut ::core::ffi::c_char,
    pub ft_to: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct salitem_T {
    pub sm_lead: *mut ::core::ffi::c_char,
    pub sm_leadlen: ::core::ffi::c_int,
    pub sm_oneof: *mut ::core::ffi::c_char,
    pub sm_rules: *mut ::core::ffi::c_char,
    pub sm_to: *mut ::core::ffi::c_char,
    pub sm_lead_w: *mut ::core::ffi::c_int,
    pub sm_oneof_w: *mut ::core::ffi::c_int,
    pub sm_to_w: *mut ::core::ffi::c_int,
}
pub type salfirst_T = ::core::ffi::c_int;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub const SP_OTHERERROR: C2Rust_Unnamed_25 = -3;
pub const SP_FORMERROR: C2Rust_Unnamed_25 = -2;
pub const SP_TRUNCERROR: C2Rust_Unnamed_25 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct slang_S {
    pub sl_next: *mut slang_T,
    pub sl_name: *mut ::core::ffi::c_char,
    pub sl_fname: *mut ::core::ffi::c_char,
    pub sl_add: bool,
    pub sl_fbyts: *mut uint8_t,
    pub sl_fbyts_len: ::core::ffi::c_int,
    pub sl_fidxs: *mut idx_T,
    pub sl_kbyts: *mut uint8_t,
    pub sl_kidxs: *mut idx_T,
    pub sl_pbyts: *mut uint8_t,
    pub sl_pidxs: *mut idx_T,
    pub sl_info: *mut ::core::ffi::c_char,
    pub sl_regions: [::core::ffi::c_char; 17],
    pub sl_midword: *mut ::core::ffi::c_char,
    pub sl_wordcount: hashtab_T,
    pub sl_compmax: ::core::ffi::c_int,
    pub sl_compminlen: ::core::ffi::c_int,
    pub sl_compsylmax: ::core::ffi::c_int,
    pub sl_compoptions: ::core::ffi::c_int,
    pub sl_comppat: garray_T,
    pub sl_compprog: *mut regprog_T,
    pub sl_comprules: *mut uint8_t,
    pub sl_compstartflags: *mut uint8_t,
    pub sl_compallflags: *mut uint8_t,
    pub sl_nobreak: bool,
    pub sl_syllable: *mut ::core::ffi::c_char,
    pub sl_syl_items: garray_T,
    pub sl_prefixcnt: ::core::ffi::c_int,
    pub sl_prefprog: *mut *mut regprog_T,
    pub sl_rep: garray_T,
    pub sl_rep_first: [int16_t; 256],
    pub sl_sal: garray_T,
    pub sl_sal_first: [salfirst_T; 256],
    pub sl_followup: bool,
    pub sl_collapse: bool,
    pub sl_rem_accents: bool,
    pub sl_sofo: bool,
    pub sl_repsal: garray_T,
    pub sl_repsal_first: [int16_t; 256],
    pub sl_nosplitsugs: bool,
    pub sl_nocompoundsugs: bool,
    pub sl_sugtime: time_t,
    pub sl_sbyts: *mut uint8_t,
    pub sl_sbyts_len: ::core::ffi::c_int,
    pub sl_sidxs: *mut idx_T,
    pub sl_sugbuf: *mut buf_T,
    pub sl_sugloaded: bool,
    pub sl_has_map: bool,
    pub sl_map_hash: hashtab_T,
    pub sl_map_array: [::core::ffi::c_int; 256],
    pub sl_sounddone: hashtab_T,
}
pub type slang_T = slang_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct langp_T {
    pub lp_slang: *mut slang_T,
    pub lp_sallang: *mut slang_T,
    pub lp_replang: *mut slang_T,
    pub lp_region: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spelltab_T {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [uint8_t; 256],
    pub st_upper: [uint8_t; 256],
}
pub type SpellAddType = ::core::ffi::c_uint;
pub const SPELL_ADD_RARE: SpellAddType = 2;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const BY_FLAGS2: C2Rust_Unnamed_28 = 3;
pub const BY_FLAGS: C2Rust_Unnamed_28 = 2;
pub const BY_INDEX: C2Rust_Unnamed_28 = 1;
pub const BY_NOFLAGS: C2Rust_Unnamed_28 = 0;
pub const BY_SPECIAL: C2Rust_Unnamed_28 = 3;
pub const SN_SYLLABLE: C2Rust_Unnamed_30 = 9;
pub const SN_NOBREAK: C2Rust_Unnamed_30 = 10;
pub const SN_COMPOUND: C2Rust_Unnamed_30 = 8;
pub const SN_NOCOMPOUNDSUGS: C2Rust_Unnamed_30 = 16;
pub const SN_NOSPLITSUGS: C2Rust_Unnamed_30 = 14;
pub const SN_SUGFILE: C2Rust_Unnamed_30 = 11;
pub const SN_WORDS: C2Rust_Unnamed_30 = 13;
pub const SN_MAP: C2Rust_Unnamed_30 = 7;
pub const SN_SOFO: C2Rust_Unnamed_30 = 6;
pub const SAL_REM_ACCENTS: C2Rust_Unnamed_29 = 4;
pub const SAL_COLLAPSE: C2Rust_Unnamed_29 = 2;
pub const SAL_F0LLOWUP: C2Rust_Unnamed_29 = 1;
pub const SN_SAL: C2Rust_Unnamed_30 = 5;
pub const SN_REPSAL: C2Rust_Unnamed_30 = 12;
pub const SN_REP: C2Rust_Unnamed_30 = 4;
pub const SN_PREFCOND: C2Rust_Unnamed_30 = 3;
pub const SN_MIDWORD: C2Rust_Unnamed_30 = 2;
pub const CF_UPPER: C2Rust_Unnamed_31 = 2;
pub const CF_WORD: C2Rust_Unnamed_31 = 1;
pub const SN_CHARFLAGS: C2Rust_Unnamed_30 = 1;
pub const SN_REGION: C2Rust_Unnamed_30 = 0;
pub const SN_INFO: C2Rust_Unnamed_30 = 15;
pub const SN_END: C2Rust_Unnamed_30 = 255;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spellinfo_T {
    pub si_foldroot: *mut wordnode_T,
    pub si_foldwcount: ::core::ffi::c_int,
    pub si_keeproot: *mut wordnode_T,
    pub si_keepwcount: ::core::ffi::c_int,
    pub si_prefroot: *mut wordnode_T,
    pub si_sugtree: ::core::ffi::c_int,
    pub si_blocks: *mut sblock_T,
    pub si_blocks_cnt: ::core::ffi::c_int,
    pub si_did_emsg: ::core::ffi::c_int,
    pub si_compress_cnt: ::core::ffi::c_int,
    pub si_first_free: *mut wordnode_T,
    pub si_free_count: ::core::ffi::c_int,
    pub si_spellbuf: *mut buf_T,
    pub si_ascii: ::core::ffi::c_int,
    pub si_add: ::core::ffi::c_int,
    pub si_clear_chartab: ::core::ffi::c_int,
    pub si_region: ::core::ffi::c_int,
    pub si_conv: vimconv_T,
    pub si_memtot: ::core::ffi::c_int,
    pub si_verbose: ::core::ffi::c_int,
    pub si_msg_count: ::core::ffi::c_int,
    pub si_info: *mut ::core::ffi::c_char,
    pub si_region_count: ::core::ffi::c_int,
    pub si_region_name: [::core::ffi::c_char; 17],
    pub si_rep: garray_T,
    pub si_repsal: garray_T,
    pub si_sal: garray_T,
    pub si_sofofr: *mut ::core::ffi::c_char,
    pub si_sofoto: *mut ::core::ffi::c_char,
    pub si_nosugfile: ::core::ffi::c_int,
    pub si_nosplitsugs: ::core::ffi::c_int,
    pub si_nocompoundsugs: ::core::ffi::c_int,
    pub si_followup: ::core::ffi::c_int,
    pub si_collapse: ::core::ffi::c_int,
    pub si_commonwords: hashtab_T,
    pub si_sugtime: time_t,
    pub si_rem_accents: ::core::ffi::c_int,
    pub si_map: garray_T,
    pub si_midword: *mut ::core::ffi::c_char,
    pub si_compmax: ::core::ffi::c_int,
    pub si_compminlen: ::core::ffi::c_int,
    pub si_compsylmax: ::core::ffi::c_int,
    pub si_compoptions: ::core::ffi::c_int,
    pub si_comppat: garray_T,
    pub si_compflags: *mut ::core::ffi::c_char,
    pub si_nobreak: ::core::ffi::c_char,
    pub si_syllable: *mut ::core::ffi::c_char,
    pub si_prefcond: garray_T,
    pub si_newprefID: ::core::ffi::c_int,
    pub si_newcompID: ::core::ffi::c_int,
}
pub type wordnode_T = wordnode_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wordnode_S {
    pub wn_u1: C2Rust_Unnamed_27,
    pub wn_u2: C2Rust_Unnamed_26,
    pub wn_child: *mut wordnode_T,
    pub wn_sibling: *mut wordnode_T,
    pub wn_refs: ::core::ffi::c_int,
    pub wn_byte: uint8_t,
    pub wn_affixID: uint8_t,
    pub wn_flags: uint16_t,
    pub wn_region: int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_26 {
    pub next: *mut wordnode_T,
    pub wnode: *mut wordnode_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_27 {
    pub hashkey: [uint8_t; 6],
    pub index: ::core::ffi::c_int,
}
pub type sblock_T = sblock_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sblock_S {
    pub sb_used: ::core::ffi::c_int,
    pub sb_next: *mut sblock_T,
    pub sb_data: [::core::ffi::c_char; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct afffile_T {
    pub af_enc: *mut ::core::ffi::c_char,
    pub af_flagtype: ::core::ffi::c_int,
    pub af_rare: ::core::ffi::c_uint,
    pub af_keepcase: ::core::ffi::c_uint,
    pub af_bad: ::core::ffi::c_uint,
    pub af_needaffix: ::core::ffi::c_uint,
    pub af_circumfix: ::core::ffi::c_uint,
    pub af_needcomp: ::core::ffi::c_uint,
    pub af_comproot: ::core::ffi::c_uint,
    pub af_compforbid: ::core::ffi::c_uint,
    pub af_comppermit: ::core::ffi::c_uint,
    pub af_nosuggest: ::core::ffi::c_uint,
    pub af_pfxpostpone: ::core::ffi::c_int,
    pub af_ignoreextra: bool,
    pub af_pref: hashtab_T,
    pub af_suff: hashtab_T,
    pub af_comp: hashtab_T,
}
pub type affentry_T = affentry_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct affentry_S {
    pub ae_next: *mut affentry_T,
    pub ae_chop: *mut ::core::ffi::c_char,
    pub ae_add: *mut ::core::ffi::c_char,
    pub ae_flags: *mut ::core::ffi::c_char,
    pub ae_cond: *mut ::core::ffi::c_char,
    pub ae_prog: *mut regprog_T,
    pub ae_compforbid: ::core::ffi::c_char,
    pub ae_comppermit: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct affheader_T {
    pub ah_key: [::core::ffi::c_char; 17],
    pub ah_flag: ::core::ffi::c_uint,
    pub ah_newID: ::core::ffi::c_int,
    pub ah_combine: ::core::ffi::c_int,
    pub ah_follows: ::core::ffi::c_int,
    pub ah_first: *mut affentry_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct compitem_T {
    pub ci_key: [::core::ffi::c_char; 17],
    pub ci_flag: ::core::ffi::c_uint,
    pub ci_newID: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const SPL_FNAME_ADD: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b".add.\0") };
pub const SPL_FNAME_ASCII: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".ascii.\0") };
pub const VIMSUGMAGIC: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"VIMsug\0") };
pub const VIMSUGMAGICL: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VIMSUGVERSION: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ZERO_FLAG: ::core::ffi::c_int = 65009 as ::core::ffi::c_int;
pub const VIMSPELLMAGIC: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"VIMspell\0") };
pub const VIMSPELLMAGICL: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize);
pub const VIMSPELLVERSION: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const SNF_REQUIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const COMPOUND_MAX_LEN: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
static e_spell_trunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E758: Truncated spell file\0".as_ptr() as *const ::core::ffi::c_char);
static e_error_while_reading_sug_file_str: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E782: Error while reading .sug file: %s\0",
        )
    });
static e_duplicate_char_in_map_entry: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E783: Duplicate char in MAP entry\0",
        )
    });
static e_illegal_character_in_word: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E1280: Illegal character in word\0".as_ptr() as *const ::core::ffi::c_char);
static e_afftrailing: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Trailing text in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char);
static e_affname: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"Affix name too long in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
);
static msg_compressing: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Compressing word tree...\0".as_ptr() as *const ::core::ffi::c_char);
pub const MAXLINELEN: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const AFT_CHAR: ::core::ffi::c_int = 0;
pub const AFT_LONG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const AFT_CAPLONG: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AFT_NUM: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const AH_KEY_LEN: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const SBLOCKSIZE: ::core::ffi::c_int = 16000 as ::core::ffi::c_int;
pub const WN_MASK: ::core::ffi::c_int = 0xffff as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn spell_check_magic_string(fd: *mut FILE) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 8] = [0; 8];
    let n__SPRB: size_t =
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t);
    let fd__SPRB: *mut FILE = fd;
    let buf__SPRB: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    let read_bytes__SPRB: size_t = fread(
        buf__SPRB as *mut ::core::ffi::c_void,
        1 as size_t,
        n__SPRB,
        fd__SPRB,
    ) as size_t;
    if read_bytes__SPRB != n__SPRB {
        return if feof(fd__SPRB) != 0 {
            SP_TRUNCERROR as ::core::ffi::c_int
        } else {
            SP_OTHERERROR as ::core::ffi::c_int
        };
    }
    if memcmp(
        &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        VIMSPELLMAGIC.as_ptr() as *const ::core::ffi::c_void,
        VIMSPELLMAGICL,
    ) != 0 as ::core::ffi::c_int
    {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn spell_load_file(
    mut fname: *mut ::core::ffi::c_char,
    mut lang: *mut ::core::ffi::c_char,
    mut old_lp: *mut slang_T,
    mut silent: bool,
) -> *mut slang_T {
    let mut len: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut scms_ret: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lp: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
    let mut res: ::core::ffi::c_int = 0;
    let mut did_estack_push: bool = false_0 != 0;
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    '_endOK: {
        '_endFAIL: {
            if fd.is_null() {
                if !silent {
                    semsg(
                        gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                        fname,
                    );
                } else if p_verbose.get() > 2 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        &raw const e_notopen as *const ::core::ffi::c_char,
                        fname,
                    );
                    verbose_leave();
                }
            } else {
                if p_verbose.get() > 2 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Reading spell file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        fname,
                    );
                    verbose_leave();
                }
                if old_lp.is_null() {
                    lp = slang_alloc(lang);
                    (*lp).sl_fname = xstrdup(fname);
                    (*lp).sl_add = !strstr(path_tail(fname), SPL_FNAME_ADD.as_ptr()).is_null();
                } else {
                    lp = old_lp;
                }
                estack_push(ETYPE_SPELL, fname, 0 as linenr_T);
                did_estack_push = true_0 != 0;
                scms_ret = spell_check_magic_string(fd);
                match scms_ret {
                    -2 | -1 => {
                        semsg(
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            gettext(b"E757: This does not look like a spell file\0".as_ptr()
                                as *const ::core::ffi::c_char),
                        );
                    }
                    -3 => {
                        semsg(
                            gettext(b"E5042: Failed to read spell file %s: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            strerror(ferror(fd)),
                        );
                    }
                    0 | _ => {
                        c = getc(fd);
                        if c < VIMSPELLVERSION {
                            emsg(gettext(
                                b"E771: Old spell file, needs to be updated\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        } else if c > VIMSPELLVERSION {
                            emsg(gettext(
                                b"E772: Spell file is for newer version of Vim\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        } else {
                            '_truncerr: loop {
                                n = getc(fd);
                                '_someerror: {
                                    if n == SN_END as ::core::ffi::c_int {
                                        res = spell_read_tree(
                                            fd,
                                            &raw mut (*lp).sl_fbyts,
                                            &raw mut (*lp).sl_fbyts_len,
                                            &raw mut (*lp).sl_fidxs,
                                            false_0 != 0,
                                            0 as ::core::ffi::c_int,
                                        );
                                        if res == 0 as ::core::ffi::c_int {
                                            res = spell_read_tree(
                                                fd,
                                                &raw mut (*lp).sl_kbyts,
                                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                                &raw mut (*lp).sl_kidxs,
                                                false_0 != 0,
                                                0 as ::core::ffi::c_int,
                                            );
                                            if res == 0 as ::core::ffi::c_int {
                                                res = spell_read_tree(
                                                    fd,
                                                    &raw mut (*lp).sl_pbyts,
                                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                                    &raw mut (*lp).sl_pidxs,
                                                    true_0 != 0,
                                                    (*lp).sl_prefixcnt,
                                                );
                                                if res == 0 as ::core::ffi::c_int {
                                                    if old_lp.is_null() && !lang.is_null() {
                                                        (*lp).sl_next = first_lang.get();
                                                        first_lang.set(lp);
                                                    }
                                                    break '_endOK;
                                                }
                                            }
                                        }
                                    } else {
                                        c = getc(fd);
                                        len = get4c(fd);
                                        if len < 0 as ::core::ffi::c_int {
                                            break '_truncerr;
                                        }
                                        res = 0 as ::core::ffi::c_int;
                                        match n {
                                            15 => {
                                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                    &raw mut (*lp).sl_info
                                                        as *mut *mut ::core::ffi::c_void;
                                                xfree(*ptr_);
                                                *ptr_ = NULL;
                                                *ptr_;
                                                (*lp).sl_info = read_string(fd, len as size_t);
                                                if (*lp).sl_info.is_null() {
                                                    break '_endFAIL;
                                                }
                                            }
                                            0 => {
                                                res = read_region_section(fd, lp, len);
                                            }
                                            1 => {
                                                res = read_charflags_section(fd);
                                            }
                                            2 => {
                                                (*lp).sl_midword = read_string(fd, len as size_t);
                                                if (*lp).sl_midword.is_null() {
                                                    break '_endFAIL;
                                                }
                                            }
                                            3 => {
                                                res = read_prefcond_section(fd, lp);
                                            }
                                            4 => {
                                                res = read_rep_section(
                                                    fd,
                                                    &raw mut (*lp).sl_rep,
                                                    &raw mut (*lp).sl_rep_first as *mut int16_t,
                                                );
                                            }
                                            12 => {
                                                res = read_rep_section(
                                                    fd,
                                                    &raw mut (*lp).sl_repsal,
                                                    &raw mut (*lp).sl_repsal_first as *mut int16_t,
                                                );
                                            }
                                            5 => {
                                                res = read_sal_section(fd, lp);
                                            }
                                            6 => {
                                                res = read_sofo_section(fd, lp);
                                            }
                                            7 => {
                                                p = read_string(fd, len as size_t);
                                                if p.is_null() {
                                                    break '_endFAIL;
                                                }
                                                set_map_str(lp, p);
                                                xfree(p as *mut ::core::ffi::c_void);
                                            }
                                            13 => {
                                                res = read_words_section(fd, lp, len);
                                            }
                                            11 => {
                                                (*lp).sl_sugtime = get8ctime(fd);
                                            }
                                            14 => {
                                                (*lp).sl_nosplitsugs = true_0 != 0;
                                            }
                                            16 => {
                                                (*lp).sl_nocompoundsugs = true_0 != 0;
                                            }
                                            8 => {
                                                res = read_compound(fd, lp, len);
                                            }
                                            10 => {
                                                (*lp).sl_nobreak = true_0 != 0;
                                            }
                                            9 => {
                                                (*lp).sl_syllable = read_string(fd, len as size_t);
                                                if (*lp).sl_syllable.is_null() {
                                                    break '_endFAIL;
                                                }
                                                if init_syl_tab(lp) != OK {
                                                    break '_endFAIL;
                                                }
                                            }
                                            _ => {
                                                if c & SNF_REQUIRED != 0 {
                                                    emsg(gettext(
                                                        b"E770: Unsupported section in spell file\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ));
                                                    break '_endFAIL;
                                                } else {
                                                    loop {
                                                        len -= 1;
                                                        if len < 0 as ::core::ffi::c_int {
                                                            break '_someerror;
                                                        }
                                                        if getc(fd) < 0 as ::core::ffi::c_int {
                                                            break '_truncerr;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if res == SP_FORMERROR as ::core::ffi::c_int {
                                    emsg(gettext(e_format.get()));
                                    break '_endFAIL;
                                } else {
                                    if res == SP_TRUNCERROR as ::core::ffi::c_int {
                                        break;
                                    }
                                    if res == SP_OTHERERROR as ::core::ffi::c_int {
                                        break '_endFAIL;
                                    }
                                }
                            }
                            emsg(gettext(e_spell_trunc.get()));
                        }
                    }
                }
            }
        }
        if !lang.is_null() {
            *lang = NUL as ::core::ffi::c_char;
        }
        if !lp.is_null() && old_lp.is_null() {
            slang_free(lp);
        }
        lp = ::core::ptr::null_mut::<slang_T>();
    }
    if !fd.is_null() {
        fclose(fd);
    }
    if did_estack_push {
        estack_pop();
    }
    return lp;
}
unsafe extern "C" fn tree_count_words(
    mut byts: *const uint8_t,
    mut byts_len: ::core::ffi::c_int,
    mut idxs: *mut idx_T,
) {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut wordcount: [::core::ffi::c_int; 254] = [0; 254];
    arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
    curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
    wordcount[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while depth >= 0 as ::core::ffi::c_int && !got_int.get() {
        if curi[depth as usize]
            > *byts.offset(arridx[depth as usize] as isize) as ::core::ffi::c_int
        {
            *idxs.offset(arridx[depth as usize] as isize) = wordcount[depth as usize] as idx_T;
            if depth > 0 as ::core::ffi::c_int {
                wordcount[(depth - 1 as ::core::ffi::c_int) as usize] += wordcount[depth as usize];
            }
            depth -= 1;
            fast_breakcheck();
        } else {
            let mut n: idx_T = arridx[depth as usize] + curi[depth as usize] as idx_T;
            curi[depth as usize] += 1;
            let mut c: ::core::ffi::c_int = *byts.offset(n as isize) as ::core::ffi::c_int;
            if c == 0 as ::core::ffi::c_int {
                wordcount[depth as usize] += 1;
                while (n as ::core::ffi::c_int + 1 as ::core::ffi::c_int) < byts_len
                    && *byts.offset((n as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    n += 1;
                    curi[depth as usize] += 1;
                }
            } else {
                depth += 1;
                arridx[depth as usize] = *idxs.offset(n as isize);
                curi[depth as usize] = 1 as ::core::ffi::c_int;
                wordcount[depth as usize] = 0 as ::core::ffi::c_int;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn suggest_load_files() {
    let mut c: ::core::ffi::c_int = 0;
    let mut timestamp: time_t = 0;
    let mut wcount: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 254] = [0; 254];
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut slang: *mut slang_T = (*lp).lp_slang;
        if (*slang).sl_sugtime != 0 as time_t && !(*slang).sl_sugloaded {
            (*slang).sl_sugloaded = true_0 != 0;
            let mut dotp: *mut ::core::ffi::c_char =
                strrchr((*slang).sl_fname, '.' as ::core::ffi::c_int);
            if !(dotp.is_null()
                || path_fnamecmp(dotp, b".spl\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int)
            {
                strcpy(
                    dotp,
                    b".sug\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                let mut fd: *mut FILE = os_fopen(
                    (*slang).sl_fname,
                    b"r\0".as_ptr() as *const ::core::ffi::c_char,
                );
                '_nextone: {
                    if !fd.is_null() {
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < VIMSUGMAGICL {
                            buf[i as usize] = getc(fd) as ::core::ffi::c_char;
                            i += 1;
                        }
                        if strncmp(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            VIMSUGMAGIC.as_ptr(),
                            VIMSUGMAGICL as size_t,
                        ) != 0 as ::core::ffi::c_int
                        {
                            semsg(
                                gettext(
                                    b"E778: This does not look like a .sug file: %s\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                (*slang).sl_fname,
                            );
                        } else {
                            c = getc(fd);
                            if c < VIMSUGVERSION {
                                semsg(
                                    gettext(
                                        b"E779: Old .sug file, needs to be updated: %s\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ),
                                    (*slang).sl_fname,
                                );
                            } else if c > VIMSUGVERSION {
                                semsg(
                                    gettext(
                                        b"E780: .sug file is for newer version of Vim: %s\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ),
                                    (*slang).sl_fname,
                                );
                            } else {
                                timestamp = get8ctime(fd);
                                if timestamp != (*slang).sl_sugtime {
                                    semsg(
                                        gettext(
                                            b"E781: .sug file doesn't match .spl file: %s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        (*slang).sl_fname,
                                    );
                                } else {
                                    '_someerror: {
                                        if spell_read_tree(
                                            fd,
                                            &raw mut (*slang).sl_sbyts,
                                            &raw mut (*slang).sl_sbyts_len,
                                            &raw mut (*slang).sl_sidxs,
                                            false_0 != 0,
                                            0 as ::core::ffi::c_int,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            (*slang).sl_sugbuf = open_spellbuf();
                                            wcount = get4c(fd);
                                            if wcount >= 0 as ::core::ffi::c_int {
                                                ga_init(
                                                    &raw mut ga,
                                                    1 as ::core::ffi::c_int,
                                                    100 as ::core::ffi::c_int,
                                                );
                                                let mut wordnr: ::core::ffi::c_int =
                                                    0 as ::core::ffi::c_int;
                                                while wordnr < wcount {
                                                    ga.ga_len = 0 as ::core::ffi::c_int;
                                                    loop {
                                                        c = getc(fd);
                                                        if c < 0 as ::core::ffi::c_int {
                                                            break '_someerror;
                                                        }
                                                        ga_grow(
                                                            &raw mut ga,
                                                            1 as ::core::ffi::c_int,
                                                        );
                                                        *(ga.ga_data as *mut uint8_t)
                                                            .offset(ga.ga_len as isize) =
                                                            c as uint8_t;
                                                        ga.ga_len += 1;
                                                        if c == NUL {
                                                            break;
                                                        }
                                                    }
                                                    if ml_append_buf(
                                                        (*slang).sl_sugbuf,
                                                        wordnr as linenr_T,
                                                        ga.ga_data as *mut ::core::ffi::c_char,
                                                        ga.ga_len as colnr_T,
                                                        true_0 != 0,
                                                    ) == FAIL
                                                    {
                                                        break '_someerror;
                                                    }
                                                    wordnr += 1;
                                                }
                                                ga_clear(&raw mut ga);
                                                tree_count_words(
                                                    (*slang).sl_fbyts,
                                                    (*slang).sl_fbyts_len,
                                                    (*slang).sl_fidxs,
                                                );
                                                tree_count_words(
                                                    (*slang).sl_sbyts,
                                                    (*slang).sl_sbyts_len,
                                                    (*slang).sl_sidxs,
                                                );
                                                break '_nextone;
                                            }
                                        }
                                    }
                                    semsg(
                                        gettext(
                                            (e_error_while_reading_sug_file_str.ptr() as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        (*slang).sl_fname,
                                    );
                                    slang_clear_sug(slang);
                                }
                            }
                        }
                    }
                }
                if !fd.is_null() {
                    fclose(fd);
                }
                strcpy(
                    dotp,
                    b".spl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
        }
        lpi += 1;
    }
}
unsafe extern "C" fn read_cnt_string(
    mut fd: *mut FILE,
    mut cnt_bytes: ::core::ffi::c_int,
    mut cntp: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cnt_bytes {
        let c: ::core::ffi::c_int = getc(fd);
        if c == EOF {
            *cntp = SP_TRUNCERROR as ::core::ffi::c_int;
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        cnt = ((cnt as ::core::ffi::c_uint) << 8 as ::core::ffi::c_int)
            .wrapping_add(c as ::core::ffi::c_uint) as ::core::ffi::c_int;
        i += 1;
    }
    *cntp = cnt;
    if cnt == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut str: *mut ::core::ffi::c_char = read_string(fd, cnt as size_t);
    if str.is_null() {
        *cntp = SP_OTHERERROR as ::core::ffi::c_int;
    }
    return str;
}
unsafe extern "C" fn read_region_section(
    mut fd: *mut FILE,
    mut lp: *mut slang_T,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if len > MAXREGIONS as ::core::ffi::c_int * 2 as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    let n__SPRNB: size_t = len as size_t;
    let fd__SPRNB: *mut FILE = fd;
    let buf__SPRNB: *mut ::core::ffi::c_char =
        &raw mut (*lp).sl_regions as *mut ::core::ffi::c_char;
    let n__SPRB: size_t = n__SPRNB;
    let fd__SPRB: *mut FILE = fd__SPRNB;
    let buf__SPRB: *mut ::core::ffi::c_char = buf__SPRNB;
    let read_bytes__SPRB: size_t = fread(
        buf__SPRB as *mut ::core::ffi::c_void,
        1 as size_t,
        n__SPRB,
        fd__SPRB,
    ) as size_t;
    if read_bytes__SPRB != n__SPRB {
        return if feof(fd__SPRB) != 0 {
            SP_TRUNCERROR as ::core::ffi::c_int
        } else {
            SP_OTHERERROR as ::core::ffi::c_int
        };
    }
    if !memchr(buf__SPRNB as *const ::core::ffi::c_void, NUL, n__SPRNB).is_null() {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    (*lp).sl_regions[len as usize] = NUL as ::core::ffi::c_char;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_charflags_section(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let mut flagslen: ::core::ffi::c_int = 0;
    let mut follen: ::core::ffi::c_int = 0;
    let mut flags: *mut ::core::ffi::c_char =
        read_cnt_string(fd, 1 as ::core::ffi::c_int, &raw mut flagslen);
    if flagslen < 0 as ::core::ffi::c_int {
        return flagslen;
    }
    let mut fol: *mut ::core::ffi::c_char =
        read_cnt_string(fd, 2 as ::core::ffi::c_int, &raw mut follen);
    if follen < 0 as ::core::ffi::c_int {
        xfree(flags as *mut ::core::ffi::c_void);
        return follen;
    }
    if !flags.is_null() && !fol.is_null() {
        set_spell_charflags(flags, flagslen, fol);
    }
    xfree(flags as *mut ::core::ffi::c_void);
    xfree(fol as *mut ::core::ffi::c_void);
    if flags.is_null() as ::core::ffi::c_int != fol.is_null() as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_prefcond_section(
    mut fd: *mut FILE,
    mut lp: *mut slang_T,
) -> ::core::ffi::c_int {
    let cnt: ::core::ffi::c_int = get2c(fd);
    if cnt <= 0 as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    (*lp).sl_prefprog =
        xcalloc(cnt as size_t, ::core::mem::size_of::<*mut regprog_T>()) as *mut *mut regprog_T;
    (*lp).sl_prefixcnt = cnt;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cnt {
        let n: ::core::ffi::c_int = getc(fd);
        if n < 0 as ::core::ffi::c_int || n >= MAXWLEN as ::core::ffi::c_int {
            return SP_FORMERROR as ::core::ffi::c_int;
        }
        if n > 0 as ::core::ffi::c_int {
            let mut buf: [::core::ffi::c_char; 255] = [0; 255];
            buf[0 as ::core::ffi::c_int as usize] = '^' as ::core::ffi::c_char;
            let n__SPRNB: size_t = n as size_t;
            let fd__SPRNB: *mut FILE = fd;
            let buf__SPRNB: *mut ::core::ffi::c_char =
                (&raw mut buf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize);
            let n__SPRB: size_t = n__SPRNB;
            let fd__SPRB: *mut FILE = fd__SPRNB;
            let buf__SPRB: *mut ::core::ffi::c_char = buf__SPRNB;
            let read_bytes__SPRB: size_t = fread(
                buf__SPRB as *mut ::core::ffi::c_void,
                1 as size_t,
                n__SPRB,
                fd__SPRB,
            ) as size_t;
            if read_bytes__SPRB != n__SPRB {
                return if feof(fd__SPRB) != 0 {
                    SP_TRUNCERROR as ::core::ffi::c_int
                } else {
                    SP_OTHERERROR as ::core::ffi::c_int
                };
            }
            if !memchr(buf__SPRNB as *const ::core::ffi::c_void, NUL, n__SPRNB).is_null() {
                return SP_FORMERROR as ::core::ffi::c_int;
            }
            buf[(n + 1 as ::core::ffi::c_int) as usize] = NUL as ::core::ffi::c_char;
            *(*lp).sl_prefprog.offset(i as isize) = vim_regcomp(
                &raw mut buf as *mut ::core::ffi::c_char,
                RE_MAGIC | RE_STRING,
            );
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_rep_section(
    mut fd: *mut FILE,
    mut gap: *mut garray_T,
    mut first: *mut int16_t,
) -> ::core::ffi::c_int {
    let mut ftp: *mut fromto_T = ::core::ptr::null_mut::<fromto_T>();
    let mut cnt: ::core::ffi::c_int = get2c(fd);
    if cnt < 0 as ::core::ffi::c_int {
        return SP_TRUNCERROR as ::core::ffi::c_int;
    }
    ga_grow(gap, cnt);
    while (*gap).ga_len < cnt {
        let mut c: ::core::ffi::c_int = 0;
        ftp = ((*gap).ga_data as *mut fromto_T).offset((*gap).ga_len as isize);
        (*ftp).ft_from = read_cnt_string(fd, 1 as ::core::ffi::c_int, &raw mut c);
        if c < 0 as ::core::ffi::c_int {
            return c;
        }
        if c == 0 as ::core::ffi::c_int {
            return SP_FORMERROR as ::core::ffi::c_int;
        }
        (*ftp).ft_to = read_cnt_string(fd, 1 as ::core::ffi::c_int, &raw mut c);
        if c <= 0 as ::core::ffi::c_int {
            xfree((*ftp).ft_from as *mut ::core::ffi::c_void);
            if c < 0 as ::core::ffi::c_int {
                return c;
            }
            return SP_FORMERROR as ::core::ffi::c_int;
        }
        (*gap).ga_len += 1;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        *first.offset(i as isize) = -1 as int16_t;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*gap).ga_len {
        ftp = ((*gap).ga_data as *mut fromto_T).offset(i_0 as isize);
        if *first.offset(*(*ftp).ft_from as uint8_t as isize) as ::core::ffi::c_int
            == -1 as ::core::ffi::c_int
        {
            *first.offset(*(*ftp).ft_from as uint8_t as isize) = i_0 as int16_t;
        }
        i_0 += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_sal_section(
    mut fd: *mut FILE,
    mut slang: *mut slang_T,
) -> ::core::ffi::c_int {
    (*slang).sl_sofo = false_0 != 0;
    let flags: ::core::ffi::c_int = getc(fd);
    if flags & SAL_F0LLOWUP as ::core::ffi::c_int != 0 {
        (*slang).sl_followup = true_0 != 0;
    }
    if flags & SAL_COLLAPSE as ::core::ffi::c_int != 0 {
        (*slang).sl_collapse = true_0 != 0;
    }
    if flags & SAL_REM_ACCENTS as ::core::ffi::c_int != 0 {
        (*slang).sl_rem_accents = true_0 != 0;
    }
    let mut cnt: ::core::ffi::c_int = get2c(fd);
    if cnt < 0 as ::core::ffi::c_int {
        return SP_TRUNCERROR as ::core::ffi::c_int;
    }
    let mut gap: *mut garray_T = &raw mut (*slang).sl_sal;
    ga_init(
        gap,
        ::core::mem::size_of::<salitem_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    ga_grow(gap, cnt + 1 as ::core::ffi::c_int);
    while (*gap).ga_len < cnt {
        let mut c: ::core::ffi::c_int = NUL;
        let mut smp: *mut salitem_T =
            ((*gap).ga_data as *mut salitem_T).offset((*gap).ga_len as isize);
        let mut ccnt: ::core::ffi::c_int = getc(fd);
        if ccnt < 0 as ::core::ffi::c_int {
            return SP_TRUNCERROR as ::core::ffi::c_int;
        }
        let mut p: *mut ::core::ffi::c_char =
            xmalloc((ccnt as size_t).wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
        (*smp).sm_lead = p;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ccnt {
            c = getc(fd);
            if !vim_strchr(
                b"0123456789(-<^$\0".as_ptr() as *const ::core::ffi::c_char,
                c,
            )
            .is_null()
            {
                break;
            }
            let c2rust_fresh19 = p;
            p = p.offset(1);
            *c2rust_fresh19 = c as uint8_t as ::core::ffi::c_char;
            i += 1;
        }
        (*smp).sm_leadlen = p.offset_from((*smp).sm_lead) as ::core::ffi::c_int;
        let c2rust_fresh20 = p;
        p = p.offset(1);
        *c2rust_fresh20 = NUL as ::core::ffi::c_char;
        if c == '(' as ::core::ffi::c_int {
            (*smp).sm_oneof = p;
            i += 1;
            while i < ccnt {
                c = getc(fd);
                if c == ')' as ::core::ffi::c_int {
                    break;
                }
                let c2rust_fresh21 = p;
                p = p.offset(1);
                *c2rust_fresh21 = c as uint8_t as ::core::ffi::c_char;
                i += 1;
            }
            let c2rust_fresh22 = p;
            p = p.offset(1);
            *c2rust_fresh22 = NUL as ::core::ffi::c_char;
            i += 1;
            if i < ccnt {
                c = getc(fd);
            }
        } else {
            (*smp).sm_oneof = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*smp).sm_rules = p;
        if i < ccnt {
            let c2rust_fresh23 = p;
            p = p.offset(1);
            *c2rust_fresh23 = c as uint8_t as ::core::ffi::c_char;
        }
        i += 1;
        if i < ccnt {
            let n__SPRNB: size_t = (ccnt - i) as size_t;
            let fd__SPRNB: *mut FILE = fd;
            let buf__SPRNB: *mut ::core::ffi::c_char = p;
            let n__SPRB: size_t = n__SPRNB;
            let fd__SPRB: *mut FILE = fd__SPRNB;
            let buf__SPRB: *mut ::core::ffi::c_char = buf__SPRNB;
            let read_bytes__SPRB: size_t = fread(
                buf__SPRB as *mut ::core::ffi::c_void,
                1 as size_t,
                n__SPRB,
                fd__SPRB,
            ) as size_t;
            if read_bytes__SPRB != n__SPRB {
                xfree((*smp).sm_lead as *mut ::core::ffi::c_void);
                return if feof(fd__SPRB) != 0 {
                    SP_TRUNCERROR as ::core::ffi::c_int
                } else {
                    SP_OTHERERROR as ::core::ffi::c_int
                };
            }
            if !memchr(buf__SPRNB as *const ::core::ffi::c_void, NUL, n__SPRNB).is_null() {
                xfree((*smp).sm_lead as *mut ::core::ffi::c_void);
                return SP_FORMERROR as ::core::ffi::c_int;
            }
            p = p.offset((ccnt - i) as isize);
        }
        let c2rust_fresh24 = p;
        p = p.offset(1);
        *c2rust_fresh24 = NUL as ::core::ffi::c_char;
        (*smp).sm_to = read_cnt_string(fd, 1 as ::core::ffi::c_int, &raw mut ccnt);
        if ccnt < 0 as ::core::ffi::c_int {
            xfree((*smp).sm_lead as *mut ::core::ffi::c_void);
            return ccnt;
        }
        (*smp).sm_lead_w = mb_str2wide((*smp).sm_lead);
        (*smp).sm_leadlen = mb_charlen((*smp).sm_lead);
        if (*smp).sm_oneof.is_null() {
            (*smp).sm_oneof_w = ::core::ptr::null_mut::<::core::ffi::c_int>();
        } else {
            (*smp).sm_oneof_w = mb_str2wide((*smp).sm_oneof);
        }
        if (*smp).sm_to.is_null() {
            (*smp).sm_to_w = ::core::ptr::null_mut::<::core::ffi::c_int>();
        } else {
            (*smp).sm_to_w = mb_str2wide((*smp).sm_to);
        }
        (*gap).ga_len += 1;
    }
    if !((*gap).ga_len <= 0 as ::core::ffi::c_int) {
        let mut smp_0: *mut salitem_T =
            ((*gap).ga_data as *mut salitem_T).offset((*gap).ga_len as isize);
        let mut p_0: *mut ::core::ffi::c_char = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
        *p_0.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        (*smp_0).sm_lead = p_0;
        (*smp_0).sm_lead_w = mb_str2wide((*smp_0).sm_lead);
        (*smp_0).sm_leadlen = 0 as ::core::ffi::c_int;
        (*smp_0).sm_oneof = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*smp_0).sm_oneof_w = ::core::ptr::null_mut::<::core::ffi::c_int>();
        (*smp_0).sm_rules = p_0;
        (*smp_0).sm_to = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*smp_0).sm_to_w = ::core::ptr::null_mut::<::core::ffi::c_int>();
        (*gap).ga_len += 1;
    }
    set_sal_first(slang);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_words_section(
    mut fd: *mut FILE,
    mut lp: *mut slang_T,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    let mut word: [uint8_t; 254] = [0; 254];
    while done < len {
        i = 0 as ::core::ffi::c_int;
        loop {
            let mut c: ::core::ffi::c_int = getc(fd);
            if c == EOF {
                return SP_TRUNCERROR as ::core::ffi::c_int;
            }
            word[i as usize] = c as uint8_t;
            if word[i as usize] as ::core::ffi::c_int == NUL {
                break;
            }
            if i == MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
                return SP_FORMERROR as ::core::ffi::c_int;
            }
            i += 1;
        }
        count_common_word(
            lp,
            &raw mut word as *mut uint8_t as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            10 as uint8_t,
        );
        done += i + 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_sofo_section(
    mut fd: *mut FILE,
    mut slang: *mut slang_T,
) -> ::core::ffi::c_int {
    let mut cnt: ::core::ffi::c_int = 0;
    let mut res: ::core::ffi::c_int = 0;
    (*slang).sl_sofo = true_0 != 0;
    let mut from: *mut ::core::ffi::c_char =
        read_cnt_string(fd, 2 as ::core::ffi::c_int, &raw mut cnt);
    if cnt < 0 as ::core::ffi::c_int {
        return cnt;
    }
    let mut to: *mut ::core::ffi::c_char =
        read_cnt_string(fd, 2 as ::core::ffi::c_int, &raw mut cnt);
    if cnt < 0 as ::core::ffi::c_int {
        xfree(from as *mut ::core::ffi::c_void);
        return cnt;
    }
    if !from.is_null() && !to.is_null() {
        res = set_sofo(slang, from, to);
    } else if !from.is_null() || !to.is_null() {
        res = SP_FORMERROR as ::core::ffi::c_int;
    } else {
        res = 0 as ::core::ffi::c_int;
    }
    xfree(from as *mut ::core::ffi::c_void);
    xfree(to as *mut ::core::ffi::c_void);
    return res;
}
unsafe extern "C" fn read_compound(
    mut fd: *mut FILE,
    mut slang: *mut slang_T,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut todo: ::core::ffi::c_int = len;
    let mut cnt: ::core::ffi::c_int = 0;
    if todo < 2 as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    todo -= 1;
    let mut c: ::core::ffi::c_int = getc(fd);
    if c < 2 as ::core::ffi::c_int {
        c = MAXWLEN as ::core::ffi::c_int;
    }
    (*slang).sl_compmax = c;
    todo -= 1;
    c = getc(fd);
    if c < 1 as ::core::ffi::c_int {
        c = 0 as ::core::ffi::c_int;
    }
    (*slang).sl_compminlen = c;
    todo -= 1;
    c = getc(fd);
    if c < 1 as ::core::ffi::c_int {
        c = MAXWLEN as ::core::ffi::c_int;
    }
    (*slang).sl_compsylmax = c;
    c = getc(fd);
    if c != 0 as ::core::ffi::c_int {
        ungetc(c, fd);
    } else {
        todo -= 1;
        c = getc(fd);
        todo -= 1;
        (*slang).sl_compoptions = c;
        let mut gap: *mut garray_T = &raw mut (*slang).sl_comppat;
        c = get2c(fd);
        if c < 0 as ::core::ffi::c_int {
            return SP_TRUNCERROR as ::core::ffi::c_int;
        }
        todo -= 2 as ::core::ffi::c_int;
        ga_init(
            gap,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            c,
        );
        ga_grow(gap, c);
        loop {
            c -= 1;
            if c < 0 as ::core::ffi::c_int {
                break;
            }
            let c2rust_fresh2 = (*gap).ga_len;
            (*gap).ga_len = (*gap).ga_len + 1;
            let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                .offset(c2rust_fresh2 as isize);
            *c2rust_lvalue_ptr = read_cnt_string(fd, 1 as ::core::ffi::c_int, &raw mut cnt);
            if cnt < 0 as ::core::ffi::c_int {
                return cnt;
            }
            todo -= cnt + 1 as ::core::ffi::c_int;
        }
    }
    if todo < 0 as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    if todo as size_t > COMPOUND_MAX_LEN as size_t {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    let mut patsize: size_t = (todo as size_t)
        .wrapping_mul(2 as size_t)
        .wrapping_add(7 as size_t);
    patsize = patsize.wrapping_add((todo as size_t).wrapping_mul(2 as size_t));
    let mut flagsize: size_t = (todo as size_t).wrapping_add(1 as size_t);
    let mut pat: *mut ::core::ffi::c_char = xmalloc(patsize) as *mut ::core::ffi::c_char;
    let mut cp: *mut uint8_t = xmalloc(flagsize) as *mut uint8_t;
    (*slang).sl_compstartflags = cp;
    *cp = NUL as uint8_t;
    let mut ap: *mut uint8_t = xmalloc(flagsize) as *mut uint8_t;
    (*slang).sl_compallflags = ap;
    *ap = NUL as uint8_t;
    let mut crp: *mut uint8_t = xmalloc(flagsize) as *mut uint8_t;
    (*slang).sl_comprules = crp;
    let mut pp: *mut ::core::ffi::c_char = pat;
    let c2rust_fresh3 = pp;
    pp = pp.offset(1);
    *c2rust_fresh3 = '^' as ::core::ffi::c_char;
    let c2rust_fresh4 = pp;
    pp = pp.offset(1);
    *c2rust_fresh4 = '\\' as ::core::ffi::c_char;
    let c2rust_fresh5 = pp;
    pp = pp.offset(1);
    *c2rust_fresh5 = '(' as ::core::ffi::c_char;
    let mut atstart: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        let c2rust_fresh6 = todo;
        todo = todo - 1;
        if c2rust_fresh6 <= 0 as ::core::ffi::c_int {
            break;
        }
        c = getc(fd);
        if c == EOF {
            xfree(pat as *mut ::core::ffi::c_void);
            return SP_TRUNCERROR as ::core::ffi::c_int;
        }
        if vim_strchr(b"?*+[]/\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
            && !byte_in_str((*slang).sl_compallflags, c)
        {
            let c2rust_fresh7 = ap;
            ap = ap.offset(1);
            *c2rust_fresh7 = c as uint8_t;
            *ap = NUL as uint8_t;
        }
        if atstart != 0 as ::core::ffi::c_int {
            if c == '[' as ::core::ffi::c_int {
                atstart = 2 as ::core::ffi::c_int;
            } else if c == ']' as ::core::ffi::c_int {
                atstart = 0 as ::core::ffi::c_int;
            } else {
                if !byte_in_str((*slang).sl_compstartflags, c) {
                    let c2rust_fresh8 = cp;
                    cp = cp.offset(1);
                    *c2rust_fresh8 = c as uint8_t;
                    *cp = NUL as uint8_t;
                }
                if atstart == 1 as ::core::ffi::c_int {
                    atstart = 0 as ::core::ffi::c_int;
                }
            }
        }
        if !crp.is_null() {
            if c == '?' as ::core::ffi::c_int
                || c == '+' as ::core::ffi::c_int
                || c == '*' as ::core::ffi::c_int
            {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*slang).sl_comprules as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
                crp = ::core::ptr::null_mut::<uint8_t>();
            } else {
                let c2rust_fresh9 = crp;
                crp = crp.offset(1);
                *c2rust_fresh9 = c as uint8_t;
            }
        }
        if c == '/' as ::core::ffi::c_int {
            let c2rust_fresh10 = pp;
            pp = pp.offset(1);
            *c2rust_fresh10 = '\\' as ::core::ffi::c_char;
            let c2rust_fresh11 = pp;
            pp = pp.offset(1);
            *c2rust_fresh11 = '|' as ::core::ffi::c_char;
            atstart = 1 as ::core::ffi::c_int;
        } else {
            if c == '?' as ::core::ffi::c_int
                || c == '+' as ::core::ffi::c_int
                || c == '~' as ::core::ffi::c_int
            {
                let c2rust_fresh12 = pp;
                pp = pp.offset(1);
                *c2rust_fresh12 = '\\' as ::core::ffi::c_char;
            }
            pp = pp.offset(utf_char2bytes(c, pp) as isize);
        }
    }
    let c2rust_fresh13 = pp;
    pp = pp.offset(1);
    *c2rust_fresh13 = '\\' as ::core::ffi::c_char;
    let c2rust_fresh14 = pp;
    pp = pp.offset(1);
    *c2rust_fresh14 = ')' as ::core::ffi::c_char;
    let c2rust_fresh15 = pp;
    pp = pp.offset(1);
    *c2rust_fresh15 = '$' as ::core::ffi::c_char;
    *pp = NUL as ::core::ffi::c_char;
    if !crp.is_null() {
        *crp = NUL as uint8_t;
    }
    (*slang).sl_compprog = vim_regcomp(pat, RE_MAGIC + RE_STRING + RE_STRICT);
    xfree(pat as *mut ::core::ffi::c_void);
    if (*slang).sl_compprog.is_null() {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn set_sofo(
    mut lp: *mut slang_T,
    mut from: *const ::core::ffi::c_char,
    mut to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut gap: *mut garray_T = &raw mut (*lp).sl_sal;
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_int>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ga_grow(gap, 256 as ::core::ffi::c_int);
    memset(
        (*gap).ga_data,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<*mut ::core::ffi::c_int>().wrapping_mul(256 as size_t),
    );
    (*gap).ga_len = 256 as ::core::ffi::c_int;
    p = from;
    s = to;
    while *p as ::core::ffi::c_int != NUL && *s as ::core::ffi::c_int != NUL {
        let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
        s = s.offset(utf_ptr2len(s) as isize);
        if c >= 256 as ::core::ffi::c_int {
            (*lp).sl_sal_first[(c & 0xff as ::core::ffi::c_int) as usize] += 1;
        }
    }
    if *p as ::core::ffi::c_int != NUL || *s as ::core::ffi::c_int != NUL {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        if (*lp).sl_sal_first[i as usize] > 0 as ::core::ffi::c_int {
            p = xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(
                ((*lp).sl_sal_first[i as usize] as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as size_t,
            )) as *const ::core::ffi::c_char;
            *((*gap).ga_data as *mut *mut ::core::ffi::c_int).offset(i as isize) =
                p as *mut ::core::ffi::c_int;
            *(p as *mut ::core::ffi::c_int) = 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    memset(
        &raw mut (*lp).sl_sal_first as *mut salfirst_T as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<salfirst_T>().wrapping_mul(256 as size_t),
    );
    p = from;
    s = to;
    while *p as ::core::ffi::c_int != NUL && *s as ::core::ffi::c_int != NUL {
        let c_0: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
        let i_0: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if c_0 >= 256 as ::core::ffi::c_int {
            let mut inp: *mut ::core::ffi::c_int = *((*gap).ga_data
                as *mut *mut ::core::ffi::c_int)
                .offset((c_0 & 0xff as ::core::ffi::c_int) as isize);
            while *inp != 0 as ::core::ffi::c_int {
                inp = inp.offset(1);
            }
            let c2rust_fresh16 = inp;
            inp = inp.offset(1);
            *c2rust_fresh16 = c_0;
            let c2rust_fresh17 = inp;
            inp = inp.offset(1);
            *c2rust_fresh17 = i_0;
            let c2rust_fresh18 = inp;
            inp = inp.offset(1);
            *c2rust_fresh18 = NUL;
        } else {
            (*lp).sl_sal_first[c_0 as usize] = i_0 as salfirst_T;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn set_sal_first(mut lp: *mut slang_T) {
    let mut gap: *mut garray_T = &raw mut (*lp).sl_sal;
    let mut sfirst: *mut salfirst_T = &raw mut (*lp).sl_sal_first as *mut salfirst_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        *sfirst.offset(i as isize) = -1 as ::core::ffi::c_int as salfirst_T;
        i += 1;
    }
    let mut smp: *mut salitem_T = (*gap).ga_data as *mut salitem_T;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*gap).ga_len {
        let mut c: ::core::ffi::c_int =
            *(*smp.offset(i_0 as isize)).sm_lead_w & 0xff as ::core::ffi::c_int;
        if *sfirst.offset(c as isize) == -1 as ::core::ffi::c_int {
            *sfirst.offset(c as isize) = i_0 as salfirst_T;
            while (i_0 + 1 as ::core::ffi::c_int) < (*gap).ga_len
                && *(*smp.offset((i_0 + 1 as ::core::ffi::c_int) as isize)).sm_lead_w
                    & 0xff as ::core::ffi::c_int
                    == c
            {
                i_0 += 1;
            }
            let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i_0 + n < (*gap).ga_len {
                if *(*smp.offset((i_0 + n) as isize)).sm_lead_w & 0xff as ::core::ffi::c_int == c {
                    let mut tsal: salitem_T = salitem_T {
                        sm_lead: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        sm_leadlen: 0,
                        sm_oneof: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        sm_rules: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        sm_to: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        sm_lead_w: ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        sm_oneof_w: ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        sm_to_w: ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    };
                    i_0 += 1;
                    n -= 1;
                    tsal = *smp.offset((i_0 + n) as isize);
                    memmove(
                        smp.offset(i_0 as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        smp.offset(i_0 as isize) as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<salitem_T>().wrapping_mul(n as size_t),
                    );
                    *smp.offset(i_0 as isize) = tsal;
                }
                n += 1;
            }
        }
        i_0 += 1;
    }
}
unsafe extern "C" fn mb_str2wide(mut s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut res: *mut ::core::ffi::c_int = xmalloc(
        (mb_charlen(s) as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
    ) as *mut ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = s;
    while *p as ::core::ffi::c_int != NUL {
        let c2rust_fresh25 = i;
        i = i + 1;
        *res.offset(c2rust_fresh25 as isize) = mb_ptr2char_adv(&raw mut p);
    }
    *res.offset(i as isize) = NUL;
    return res;
}
unsafe extern "C" fn spell_read_tree(
    mut fd: *mut FILE,
    mut bytsp: *mut *mut uint8_t,
    mut bytsp_len: *mut ::core::ffi::c_int,
    mut idxsp: *mut *mut idx_T,
    mut prefixtree: bool,
    mut prefixcnt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = get4c(fd);
    if len < 0 as ::core::ffi::c_int {
        return SP_TRUNCERROR as ::core::ffi::c_int;
    }
    if len as size_t
        > (SIZE_MAX as usize).wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
    {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    if len <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut bp: *mut uint8_t = xcalloc(1 as size_t, len as size_t) as *mut uint8_t;
    *bytsp = bp;
    if !bytsp_len.is_null() {
        *bytsp_len = len;
    }
    let mut ip: *mut idx_T = xcalloc(len as size_t, ::core::mem::size_of::<idx_T>()) as *mut idx_T;
    *idxsp = ip;
    let mut idx: ::core::ffi::c_int = read_tree_node(
        fd,
        bp,
        ip,
        len,
        0 as idx_T,
        prefixtree,
        prefixcnt,
        0 as ::core::ffi::c_int,
    );
    if idx < 0 as ::core::ffi::c_int {
        return idx;
    }
    if idx != len {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_tree_node(
    mut fd: *mut FILE,
    mut byts: *mut uint8_t,
    mut idxs: *mut idx_T,
    mut maxidx: ::core::ffi::c_int,
    mut startidx: idx_T,
    mut prefixtree: bool,
    mut maxprefcondnr: ::core::ffi::c_int,
    mut depth: ::core::ffi::c_int,
) -> idx_T {
    let mut idx: idx_T = startidx;
    if depth > MAXWLEN as ::core::ffi::c_int {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = getc(fd);
    if len <= 0 as ::core::ffi::c_int {
        return SP_TRUNCERROR as ::core::ffi::c_int;
    }
    if startidx as ::core::ffi::c_int + len >= maxidx {
        return SP_FORMERROR as ::core::ffi::c_int;
    }
    let c2rust_fresh0 = idx;
    idx = idx + 1;
    *byts.offset(c2rust_fresh0 as isize) = len as uint8_t;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= len {
        let mut c: ::core::ffi::c_int = getc(fd);
        if c < 0 as ::core::ffi::c_int {
            return SP_TRUNCERROR as ::core::ffi::c_int;
        }
        if c <= BY_SPECIAL as ::core::ffi::c_int {
            if c == BY_NOFLAGS as ::core::ffi::c_int && !prefixtree {
                *idxs.offset(idx as isize) = 0 as ::core::ffi::c_int as idx_T;
            } else if c != BY_INDEX as ::core::ffi::c_int {
                if prefixtree {
                    if c == BY_FLAGS as ::core::ffi::c_int {
                        c = getc(fd) << 24 as ::core::ffi::c_int;
                    } else {
                        c = 0 as ::core::ffi::c_int;
                    }
                    c |= getc(fd);
                    let mut n: ::core::ffi::c_int = get2c(fd);
                    if n >= maxprefcondnr {
                        return SP_FORMERROR as ::core::ffi::c_int;
                    }
                    c |= n << 8 as ::core::ffi::c_int;
                } else {
                    let mut c2: ::core::ffi::c_int = c;
                    c = getc(fd);
                    if c2 == BY_FLAGS2 as ::core::ffi::c_int {
                        c = (getc(fd) << 8 as ::core::ffi::c_int) + c;
                    }
                    if c & WF_REGION as ::core::ffi::c_int != 0 {
                        c = (getc(fd) << 16 as ::core::ffi::c_int) + c;
                    }
                    if c & WF_AFX as ::core::ffi::c_int != 0 {
                        c = (getc(fd) << 24 as ::core::ffi::c_int) + c;
                    }
                }
                *idxs.offset(idx as isize) = c as idx_T;
                c = 0 as ::core::ffi::c_int;
            } else {
                let mut n_0: ::core::ffi::c_int = get3c(fd);
                if n_0 < 0 as ::core::ffi::c_int || n_0 >= maxidx {
                    return SP_FORMERROR as ::core::ffi::c_int;
                }
                *idxs.offset(idx as isize) = (n_0 + SHARED_MASK) as idx_T;
                c = getc(fd);
            }
        }
        let c2rust_fresh1 = idx;
        idx = idx + 1;
        *byts.offset(c2rust_fresh1 as isize) = c as uint8_t;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 <= len {
        if *byts.offset((startidx as ::core::ffi::c_int + i_0) as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *idxs.offset((startidx as ::core::ffi::c_int + i_0) as isize) as ::core::ffi::c_int
                & SHARED_MASK
                != 0
            {
                *idxs.offset((startidx as ::core::ffi::c_int + i_0) as isize) &= !SHARED_MASK;
            } else {
                *idxs.offset((startidx as ::core::ffi::c_int + i_0) as isize) = idx;
                idx = read_tree_node(
                    fd,
                    byts,
                    idxs,
                    maxidx,
                    idx,
                    prefixtree,
                    maxprefcondnr,
                    depth + 1 as ::core::ffi::c_int,
                );
                if idx < 0 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        i_0 += 1;
    }
    return idx;
}
pub const SHARED_MASK: ::core::ffi::c_int = 0x8000000 as ::core::ffi::c_int;
unsafe extern "C" fn spell_reload_one(mut fname: *mut ::core::ffi::c_char, mut added_word: bool) {
    let mut didit: bool = false_0 != 0;
    let mut slang: *mut slang_T = first_lang.get();
    while !slang.is_null() {
        if path_full_compare(fname, (*slang).sl_fname, false_0 != 0, true_0 != 0)
            as ::core::ffi::c_uint
            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            slang_clear(slang);
            if spell_load_file(
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                slang,
                false_0 != 0,
            )
            .is_null()
            {
                slang_clear(slang);
            }
            redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
            didit = true_0 != 0;
        }
        slang = (*slang).sl_next;
    }
    if added_word as ::core::ffi::c_int != 0 && !didit {
        parse_spelllang(curwin.get());
    }
}
pub const PFX_FLAGS: ::core::ffi::c_int = -256 as ::core::ffi::c_int;
pub const CONDIT_COMB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONDIT_CFIX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CONDIT_SUF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONDIT_AFF: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
static compress_start: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(30000 as ::core::ffi::c_int);
static compress_inc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(100 as ::core::ffi::c_int);
static compress_added: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(500000 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn spell_check_msm() -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = p_msm.get();
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut start: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) * 10 as ::core::ffi::c_int
            / (SBLOCKSIZE / 102 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
        return FAIL;
    }
    p = p.offset(1);
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut incr: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) * 102 as ::core::ffi::c_int
            / (SBLOCKSIZE / 10 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
        return FAIL;
    }
    p = p.offset(1);
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut added: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int)
            * 1024 as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int != NUL {
        return FAIL;
    }
    if start == 0 as ::core::ffi::c_int
        || incr == 0 as ::core::ffi::c_int
        || added == 0 as ::core::ffi::c_int
        || incr > start
    {
        return FAIL;
    }
    compress_start.set(start);
    compress_inc.set(incr);
    compress_added.set(added);
    return OK;
}
unsafe extern "C" fn spell_read_aff(
    mut spin: *mut spellinfo_T,
    mut fname: *mut ::core::ffi::c_char,
) -> *mut afffile_T {
    let mut rline: [::core::ffi::c_char; 500] = [0; 500];
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut items: [*mut ::core::ffi::c_char; 30] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 30];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_aff: *mut affheader_T = ::core::ptr::null_mut::<affheader_T>();
    let mut did_postpone_prefix: bool = false_0 != 0;
    let mut aff_todo: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut low: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fol: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut upp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found_map: bool = false_0 != 0;
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let mut compminlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compsylmax: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compoptions: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compmax: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compflags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut midword: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syllable: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sofofrom: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sofoto: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return ::core::ptr::null_mut::<afffile_T>();
    }
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"Reading affix file %s...\0".as_ptr() as *const ::core::ffi::c_char),
        fname,
    );
    spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    let mut do_rep: bool = (*spin).si_rep.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_repsal: bool = (*spin).si_repsal.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_sal: bool = (*spin).si_sal.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_mapline: bool = (*spin).si_map.ga_len <= 0 as ::core::ffi::c_int;
    let mut aff: *mut afffile_T =
        getroom(spin, ::core::mem::size_of::<afffile_T>(), true_0 != 0) as *mut afffile_T;
    hash_init(&raw mut (*aff).af_pref);
    hash_init(&raw mut (*aff).af_suff);
    hash_init(&raw mut (*aff).af_comp);
    while !vim_fgets(&raw mut rline as *mut ::core::ffi::c_char, MAXLINELEN, fd) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if *(&raw mut rline as *mut ::core::ffi::c_char) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
        {
            continue;
        }
        xfree(pc as *mut ::core::ffi::c_void);
        if (*spin).si_conv.vc_type != CONV_NONE as ::core::ffi::c_int {
            pc = string_convert(
                &raw mut (*spin).si_conv,
                &raw mut rline as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<size_t>(),
            );
            if pc.is_null() {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion failure for word in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    &raw mut rline as *mut ::core::ffi::c_char,
                );
                continue;
            } else {
                line = pc;
            }
        } else {
            pc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            line = &raw mut rline as *mut ::core::ffi::c_char;
        }
        let mut itemcnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        p = line;
        loop {
            while *p as ::core::ffi::c_int != NUL
                && *p as uint8_t as ::core::ffi::c_int <= ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if itemcnt == MAXITEMCNT {
                break;
            }
            let c2rust_fresh33 = itemcnt;
            itemcnt = itemcnt + 1;
            let c2rust_lvalue_ptr = &raw mut items[c2rust_fresh33 as usize];
            *c2rust_lvalue_ptr = p as *mut ::core::ffi::c_char;
            if itemcnt == 2 as ::core::ffi::c_int
                && spell_info_item(
                    items[0 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
            {
                while *p as uint8_t as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == TAB
                {
                    p = p.offset(1);
                }
            } else {
                while *p as uint8_t as ::core::ffi::c_int > ' ' as ::core::ffi::c_int {
                    p = p.offset(1);
                }
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh34 = p;
            p = p.offset(1);
            *c2rust_fresh34 = NUL as ::core::ffi::c_char;
        }
        if itemcnt <= 0 as ::core::ffi::c_int {
            continue;
        }
        if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"SET\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && (*aff).af_enc.is_null()
        {
            (*aff).af_enc =
                enc_canonize(items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char);
            if (*spin).si_ascii == 0
                && convert_setup(&raw mut (*spin).si_conv, (*aff).af_enc, p_enc.get()) == FAIL
            {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion in %s not supported: from %s to %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    (*aff).af_enc,
                    p_enc.get(),
                );
            }
            (*spin).si_conv.vc_fail = true_0 != 0;
        } else if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"FLAG\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && (*aff).af_flagtype == AFT_CHAR
        {
            if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"long\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_LONG;
            } else if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"num\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_NUM;
            } else if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"caplong\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_CAPLONG;
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Invalid value for FLAG in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    items[1 as ::core::ffi::c_int as usize],
                );
            }
            if (*aff).af_rare != 0 as ::core::ffi::c_uint
                || (*aff).af_keepcase != 0 as ::core::ffi::c_uint
                || (*aff).af_bad != 0 as ::core::ffi::c_uint
                || (*aff).af_needaffix != 0 as ::core::ffi::c_uint
                || (*aff).af_circumfix != 0 as ::core::ffi::c_uint
                || (*aff).af_needcomp != 0 as ::core::ffi::c_uint
                || (*aff).af_comproot != 0 as ::core::ffi::c_uint
                || (*aff).af_nosuggest != 0 as ::core::ffi::c_uint
                || !compflags.is_null()
                || (*aff).af_suff.ht_used > 0 as size_t
                || (*aff).af_pref.ht_used > 0 as size_t
            {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"FLAG after using flags in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    items[1 as ::core::ffi::c_int as usize],
                );
            }
        } else if spell_info_item(
            items[0 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
            && itemcnt > 1 as ::core::ffi::c_int
        {
            p = getroom(
                spin,
                (if (*spin).si_info.is_null() {
                    0 as size_t
                } else {
                    strlen((*spin).si_info)
                })
                .wrapping_add(strlen(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ))
                .wrapping_add(strlen(
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ))
                .wrapping_add(3 as size_t),
                false_0 != 0,
            ) as *mut ::core::ffi::c_char;
            if !(*spin).si_info.is_null() {
                strcpy(p, (*spin).si_info);
                strcat(p, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
            strcat(
                p,
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
            );
            strcat(p, b" \0".as_ptr() as *const ::core::ffi::c_char);
            strcat(
                p,
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
            );
            (*spin).si_info = p;
        } else if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"MIDWORD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && midword.is_null()
        {
            midword = getroom_save(
                spin,
                items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
            );
        } else {
            if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"TRY\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                continue;
            }
            if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"RAR\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"RARE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_rare == 0 as ::core::ffi::c_uint
            {
                (*aff).af_rare = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"KEP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"KEEPCASE\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_keepcase == 0 as ::core::ffi::c_uint
            {
                (*aff).af_keepcase = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"BAD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"FORBIDDENWORD\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_bad == 0 as ::core::ffi::c_uint
            {
                (*aff).af_bad = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NEEDAFFIX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_needaffix == 0 as ::core::ffi::c_uint
            {
                (*aff).af_needaffix = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CIRCUMFIX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_circumfix == 0 as ::core::ffi::c_uint
            {
                (*aff).af_circumfix = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSUGGEST\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_nosuggest == 0 as ::core::ffi::c_uint
            {
                (*aff).af_nosuggest = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NEEDCOMPOUND\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"ONLYINCOMPOUND\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_needcomp == 0 as ::core::ffi::c_uint
            {
                (*aff).af_needcomp = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDROOT\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_comproot == 0 as ::core::ffi::c_uint
            {
                (*aff).af_comproot = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDFORBIDFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_compforbid == 0 as ::core::ffi::c_uint
            {
                (*aff).af_compforbid = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
                if (*aff).af_pref.ht_used > 0 as size_t {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Defining COMPOUNDFORBIDFLAG after PFX item may give wrong results in %s line %d\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDPERMITFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_comppermit == 0 as ::core::ffi::c_uint
            {
                (*aff).af_comppermit = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
                if (*aff).af_pref.ht_used > 0 as size_t {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Defining COMPOUNDPERMITFLAG after PFX item may give wrong results in %s line %d\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compflags.is_null()
            {
                p = getroom(
                    spin,
                    strlen(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                        .wrapping_add(2 as size_t),
                    false_0 != 0,
                ) as *mut ::core::ffi::c_char;
                strcpy(p, items[1 as ::core::ffi::c_int as usize]);
                strcat(p, b"+\0".as_ptr() as *const ::core::ffi::c_char);
                compflags = p;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDRULES\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDRULES value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDRULE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if !compflags.is_null()
                    || *skipdigits(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int
                        != NUL
                {
                    let mut l: ::core::ffi::c_int = strlen(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    if !compflags.is_null() {
                        l += strlen(compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    }
                    p = getroom(spin, l as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
                    if !compflags.is_null() {
                        strcpy(p, compflags);
                        strcat(p, b"/\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    strcat(
                        p,
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    );
                    compflags = p;
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDWORDMAX\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compmax == 0 as ::core::ffi::c_int
            {
                compmax =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compmax == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDWORDMAX value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDMIN\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compminlen == 0 as ::core::ffi::c_int
            {
                compminlen =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compminlen == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDMIN value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDSYLMAX\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compsylmax == 0 as ::core::ffi::c_int
            {
                compsylmax =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compsylmax == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDSYLMAX value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDDUP\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKDUP as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDREP\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKREP as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDCASE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKCASE as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDTRIPLE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKTRIPLE as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDPATTERN\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Wrong CHECKCOMPOUNDPATTERN value in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDPATTERN\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int,
            ) {
                let mut gap: *mut garray_T = &raw mut (*spin).si_comppat;
                let mut i: ::core::ffi::c_int = 0;
                i = 0 as ::core::ffi::c_int;
                while i < (*gap).ga_len - 1 as ::core::ffi::c_int {
                    if strcmp(
                        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                        && strcmp(
                            *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                                .offset((i + 1 as ::core::ffi::c_int) as isize),
                            items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    i += 2 as ::core::ffi::c_int;
                }
                if i >= (*gap).ga_len {
                    ga_grow(gap, 2 as ::core::ffi::c_int);
                    let c2rust_fresh35 = (*gap).ga_len;
                    (*gap).ga_len = (*gap).ga_len + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *((*gap).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh35 as isize);
                    *c2rust_lvalue_ptr_0 = getroom_save(
                        spin,
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                    let c2rust_fresh36 = (*gap).ga_len;
                    (*gap).ga_len = (*gap).ga_len + 1;
                    let c2rust_lvalue_ptr_1 = &raw mut *((*gap).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh36 as isize);
                    *c2rust_lvalue_ptr_1 = getroom_save(
                        spin,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SYLLABLE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && syllable.is_null()
            {
                syllable = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOBREAK\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nobreak = true_0 as ::core::ffi::c_char;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSPLITSUGS\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nosplitsugs = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOCOMPOUNDSUGS\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nocompoundsugs = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSUGFILE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nosugfile = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"PFXPOSTPONE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*aff).af_pfxpostpone = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"IGNOREEXTRA\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*aff).af_ignoreextra = true_0 != 0;
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"PFX\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"SFX\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && aff_todo == 0 as ::core::ffi::c_int
                && itemcnt >= 4 as ::core::ffi::c_int
            {
                let mut lasti: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
                let mut key: [::core::ffi::c_char; 17] = [0; 17];
                if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    == 'P' as ::core::ffi::c_int
                {
                    tp = &raw mut (*aff).af_pref;
                } else {
                    tp = &raw mut (*aff).af_suff;
                }
                xstrlcpy(
                    &raw mut key as *mut ::core::ffi::c_char,
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    AH_KEY_LEN as size_t,
                );
                hi = hash_find(tp, &raw mut key as *mut ::core::ffi::c_char);
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    cur_aff = (*hi).hi_key as *mut affheader_T;
                    if (*cur_aff).ah_combine
                        != (*items[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                            == 'Y' as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Different combining flag in continued affix block in %s line %d: %s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                    if (*cur_aff).ah_follows == 0 {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Duplicate affix in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                } else {
                    cur_aff = getroom(spin, ::core::mem::size_of::<affheader_T>(), true_0 != 0)
                        as *mut affheader_T;
                    (*cur_aff).ah_flag = affitem2flag(
                        (*aff).af_flagtype,
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        fname,
                        lnum,
                    );
                    if (*cur_aff).ah_flag == 0 as ::core::ffi::c_uint
                        || strlen(
                            items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        ) >= AH_KEY_LEN as size_t
                    {
                        break;
                    }
                    if (*cur_aff).ah_flag == (*aff).af_bad
                        || (*cur_aff).ah_flag == (*aff).af_rare
                        || (*cur_aff).ah_flag == (*aff).af_keepcase
                        || (*cur_aff).ah_flag == (*aff).af_needaffix
                        || (*cur_aff).ah_flag == (*aff).af_circumfix
                        || (*cur_aff).ah_flag == (*aff).af_nosuggest
                        || (*cur_aff).ah_flag == (*aff).af_needcomp
                        || (*cur_aff).ah_flag == (*aff).af_comproot
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in %s line %d: %s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                    strcpy(
                        &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                    hash_add(tp, &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char);
                    (*cur_aff).ah_combine = (*items[2 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        == 'Y' as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                }
                if itemcnt > lasti
                    && strcmp(
                        items[lasti as usize] as *const ::core::ffi::c_char,
                        b"S\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    lasti += 1;
                    (*cur_aff).ah_follows = true_0;
                } else {
                    (*cur_aff).ah_follows = false_0;
                }
                if itemcnt > lasti
                    && !(*aff).af_ignoreextra
                    && *items[lasti as usize] as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[lasti as usize],
                    );
                }
                if strcmp(
                    items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"Y\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                    && strcmp(
                        items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"N\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Expected Y or N in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[2 as ::core::ffi::c_int as usize],
                    );
                }
                if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    == 'P' as ::core::ffi::c_int
                    && (*aff).af_pfxpostpone != 0
                {
                    if (*cur_aff).ah_newID == 0 as ::core::ffi::c_int {
                        check_renumber(spin);
                        (*spin).si_newprefID += 1;
                        (*cur_aff).ah_newID = (*spin).si_newprefID;
                        did_postpone_prefix = false_0 != 0;
                    } else {
                        did_postpone_prefix = true_0 != 0;
                    }
                }
                aff_todo =
                    atoi(items[3 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"PFX\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"SFX\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && aff_todo > 0 as ::core::ffi::c_int
                && strcmp(
                    &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char,
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                && itemcnt >= 5 as ::core::ffi::c_int
            {
                let mut aff_entry: *mut affentry_T = ::core::ptr::null_mut::<affentry_T>();
                let mut lasti_0: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
                if itemcnt > lasti_0
                    && *items[lasti_0 as usize] as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                    && (strcmp(
                        items[lasti_0 as usize] as *const ::core::ffi::c_char,
                        b"-\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                        || itemcnt != lasti_0 + 1 as ::core::ffi::c_int)
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[lasti_0 as usize],
                    );
                }
                aff_todo -= 1;
                aff_entry = getroom(spin, ::core::mem::size_of::<affentry_T>(), true_0 != 0)
                    as *mut affentry_T;
                if strcmp(
                    items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    (*aff_entry).ae_chop = getroom_save(
                        spin,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
                if strcmp(
                    items[3 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    (*aff_entry).ae_add = getroom_save(
                        spin,
                        items[3 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                    (*aff_entry).ae_flags =
                        vim_strchr((*aff_entry).ae_add, '/' as ::core::ffi::c_int);
                    if !(*aff_entry).ae_flags.is_null() {
                        let c2rust_fresh37 = (*aff_entry).ae_flags;
                        (*aff_entry).ae_flags = (*aff_entry).ae_flags.offset(1);
                        *c2rust_fresh37 = NUL as ::core::ffi::c_char;
                        aff_process_flags(aff, aff_entry);
                    }
                }
                if (*spin).si_ascii == 0
                    || !(has_non_ascii((*aff_entry).ae_chop) as ::core::ffi::c_int != 0
                        || has_non_ascii((*aff_entry).ae_add) as ::core::ffi::c_int != 0)
                {
                    (*aff_entry).ae_next = (*cur_aff).ah_first;
                    (*cur_aff).ah_first = aff_entry;
                    if strcmp(
                        items[4 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b".\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                    {
                        let mut buf: [::core::ffi::c_char; 500] = [0; 500];
                        (*aff_entry).ae_cond = getroom_save(
                            spin,
                            items[4 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        );
                        snprintf(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 500]>(),
                            if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                == 'P' as ::core::ffi::c_int
                            {
                                b"^%s\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"%s$\0".as_ptr() as *const ::core::ffi::c_char
                            },
                            items[4 as ::core::ffi::c_int as usize],
                        );
                        (*aff_entry).ae_prog = vim_regcomp(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            RE_MAGIC + RE_STRING + RE_STRICT,
                        );
                        if (*aff_entry).ae_prog.is_null() {
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Broken condition in %s line %d: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                                lnum,
                                items[4 as ::core::ffi::c_int as usize],
                            );
                        }
                    }
                    if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == 'P' as ::core::ffi::c_int
                        && (*aff).af_pfxpostpone != 0
                        && (*aff_entry).ae_flags.is_null()
                    {
                        let mut upper: bool = false_0 != 0;
                        if !(*aff_entry).ae_chop.is_null()
                            && !(*aff_entry).ae_add.is_null()
                            && *(*aff_entry)
                                .ae_chop
                                .offset(utfc_ptr2len((*aff_entry).ae_chop) as isize)
                                as ::core::ffi::c_int
                                == NUL
                        {
                            let mut c: ::core::ffi::c_int = utf_ptr2char((*aff_entry).ae_chop);
                            let mut c_up: ::core::ffi::c_int = if c >= 128 as ::core::ffi::c_int {
                                mb_toupper(c)
                            } else {
                                (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
                            };
                            if c_up != c
                                && ((*aff_entry).ae_cond.is_null()
                                    || utf_ptr2char((*aff_entry).ae_cond) == c)
                            {
                                p = (*aff_entry)
                                    .ae_add
                                    .offset(strlen((*aff_entry).ae_add) as isize);
                                p = p.offset(
                                    -((utf_head_off(
                                        (*aff_entry).ae_add,
                                        p.offset(-(1 as ::core::ffi::c_int as isize)),
                                    ) + 1 as ::core::ffi::c_int)
                                        as isize),
                                );
                                if utf_ptr2char(p) == c_up {
                                    upper = true_0 != 0;
                                    (*aff_entry).ae_chop =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    *p = NUL as ::core::ffi::c_char;
                                    if !(*aff_entry).ae_cond.is_null() {
                                        let mut buf_0: [::core::ffi::c_char; 500] = [0; 500];
                                        onecap_copy(
                                            items[4 as ::core::ffi::c_int as usize]
                                                as *const ::core::ffi::c_char,
                                            &raw mut buf_0 as *mut ::core::ffi::c_char,
                                            true_0 != 0,
                                        );
                                        (*aff_entry).ae_cond = getroom_save(
                                            spin,
                                            &raw mut buf_0 as *mut ::core::ffi::c_char,
                                        );
                                        if !(*aff_entry).ae_cond.is_null() {
                                            snprintf(
                                                &raw mut buf_0 as *mut ::core::ffi::c_char,
                                                MAXLINELEN as size_t,
                                                b"^%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                (*aff_entry).ae_cond,
                                            );
                                            vim_regfree((*aff_entry).ae_prog);
                                            (*aff_entry).ae_prog = vim_regcomp(
                                                &raw mut buf_0 as *mut ::core::ffi::c_char,
                                                RE_MAGIC + RE_STRING,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        if (*aff_entry).ae_chop.is_null() {
                            let mut idx: ::core::ffi::c_int = 0;
                            idx = (*spin).si_prefcond.ga_len - 1 as ::core::ffi::c_int;
                            while idx >= 0 as ::core::ffi::c_int {
                                p = *((*spin).si_prefcond.ga_data as *mut *mut ::core::ffi::c_char)
                                    .offset(idx as isize);
                                if str_equal(p, (*aff_entry).ae_cond) {
                                    break;
                                }
                                idx -= 1;
                            }
                            if idx < 0 as ::core::ffi::c_int {
                                idx = (*spin).si_prefcond.ga_len;
                                let mut pp: *mut *mut ::core::ffi::c_char = ga_append_via_ptr(
                                    &raw mut (*spin).si_prefcond,
                                    ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
                                )
                                    as *mut *mut ::core::ffi::c_char;
                                *pp = if (*aff_entry).ae_cond.is_null() {
                                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                                } else {
                                    getroom_save(spin, (*aff_entry).ae_cond)
                                };
                            }
                            if (*aff_entry).ae_add.is_null() {
                                p = b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                p = (*aff_entry).ae_add;
                            }
                            let mut n: ::core::ffi::c_int = PFX_FLAGS;
                            if (*cur_aff).ah_combine == 0 {
                                n |= WFP_NC as ::core::ffi::c_int;
                            }
                            if upper {
                                n |= WFP_UP as ::core::ffi::c_int;
                            }
                            if (*aff_entry).ae_comppermit != 0 {
                                n |= WFP_COMPPERMIT as ::core::ffi::c_int;
                            }
                            if (*aff_entry).ae_compforbid != 0 {
                                n |= WFP_COMPFORBID as ::core::ffi::c_int;
                            }
                            tree_add_word(
                                spin,
                                p,
                                (*spin).si_prefroot,
                                n,
                                idx,
                                (*cur_aff).ah_newID,
                            );
                            did_postpone_prefix = true_0 != 0;
                        }
                        if aff_todo == 0 as ::core::ffi::c_int && !did_postpone_prefix {
                            (*spin).si_newprefID -= 1;
                            (*cur_aff).ah_newID = 0 as ::core::ffi::c_int;
                        }
                    }
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"FOL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && fol.is_null()
            {
                fol =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"LOW\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && low.is_null()
            {
                low =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"UPP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && upp.is_null()
            {
                upp =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"REP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"REPSAL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                if *(*__ctype_b_loc()).offset(
                    *items[1 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int
                        as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Expected REP(SAL) count in %s line %d\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                    );
                }
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"REP\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"REPSAL\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && itemcnt >= 3 as ::core::ffi::c_int
            {
                if itemcnt > 3 as ::core::ffi::c_int
                    && *items[3 as ::core::ffi::c_int as usize]
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        != '#' as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[3 as ::core::ffi::c_int as usize],
                    );
                }
                if if *items[0 as ::core::ffi::c_int as usize]
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == 'S' as ::core::ffi::c_int
                {
                    do_repsal as ::core::ffi::c_int
                } else {
                    do_rep as ::core::ffi::c_int
                } != 0
                {
                    p = items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                            *p = ' ' as ::core::ffi::c_char;
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    p = items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                            *p = ' ' as ::core::ffi::c_char;
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    add_fromto(
                        spin,
                        if *items[0 as ::core::ffi::c_int as usize]
                            .offset(3 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'S' as ::core::ffi::c_int
                        {
                            &raw mut (*spin).si_repsal
                        } else {
                            &raw mut (*spin).si_rep
                        },
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"MAP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if !found_map {
                    found_map = true_0 != 0;
                    if *(*__ctype_b_loc()).offset(
                        *items[1 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int
                            as isize,
                    ) as ::core::ffi::c_int
                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Expected MAP count in %s line %d\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                        );
                    }
                } else if do_mapline {
                    p = items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        let mut c_0: ::core::ffi::c_int =
                            mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char);
                        if !((*spin).si_map.ga_len <= 0 as ::core::ffi::c_int)
                            && !vim_strchr(
                                (*spin).si_map.ga_data as *const ::core::ffi::c_char,
                                c_0,
                            )
                            .is_null()
                            || !vim_strchr(p, c_0).is_null()
                        {
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Duplicate character in MAP in %s line %d\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                                lnum,
                            );
                        }
                    }
                    ga_concat(
                        &raw mut (*spin).si_map,
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    );
                    ga_append(&raw mut (*spin).si_map, '/' as uint8_t);
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SAL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int,
            ) {
                if do_sal {
                    if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"followup\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_followup = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"collapse_result\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_collapse = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"remove_accents\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_rem_accents = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else {
                        add_fromto(
                            spin,
                            &raw mut (*spin).si_sal,
                            items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                            (if strcmp(
                                items[2 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_char,
                                b"_\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                items[2 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_char
                            }) as *mut ::core::ffi::c_char,
                        );
                    }
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SOFOFROM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && sofofrom.is_null()
            {
                sofofrom = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SOFOTO\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && sofoto.is_null()
            {
                sofoto = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"COMMON\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while i_0 < itemcnt {
                    if (*hash_find(
                        &raw mut (*spin).si_commonwords,
                        items[i_0 as usize] as *const ::core::ffi::c_char,
                    ))
                    .hi_key
                    .is_null()
                        || (*hash_find(
                            &raw mut (*spin).si_commonwords,
                            items[i_0 as usize] as *const ::core::ffi::c_char,
                        ))
                        .hi_key
                            == &raw const hash_removed as *mut ::core::ffi::c_char
                    {
                        p = xstrdup(items[i_0 as usize] as *const ::core::ffi::c_char);
                        hash_add(&raw mut (*spin).si_commonwords, p);
                    }
                    i_0 += 1;
                }
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(
                        b"Unrecognized or duplicate item in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    fname,
                    lnum,
                    items[0 as ::core::ffi::c_int as usize],
                );
            }
        }
    }
    if !fol.is_null() || !low.is_null() || !upp.is_null() {
        if (*spin).si_clear_chartab != 0 {
            init_spell_chartab();
            (*spin).si_clear_chartab = false_0;
        }
        xfree(fol as *mut ::core::ffi::c_void);
        xfree(low as *mut ::core::ffi::c_void);
        xfree(upp as *mut ::core::ffi::c_void);
    }
    if compmax != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compmax,
            compmax,
            b"COMPOUNDWORDMAX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compmax = compmax;
    }
    if compminlen != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compminlen,
            compminlen,
            b"COMPOUNDMIN\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compminlen = compminlen;
    }
    if compsylmax != 0 as ::core::ffi::c_int {
        if syllable.is_null() {
            smsg(
                0 as ::core::ffi::c_int,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                gettext(b"COMPOUNDSYLMAX used without SYLLABLE\0".as_ptr()
                    as *const ::core::ffi::c_char),
            );
        }
        aff_check_number(
            (*spin).si_compsylmax,
            compsylmax,
            b"COMPOUNDSYLMAX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compsylmax = compsylmax;
    }
    if compoptions != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compoptions,
            compoptions,
            b"COMPOUND options\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
        (*spin).si_compoptions |= compoptions;
    }
    if !compflags.is_null() {
        process_compflags(spin, aff, compflags);
    }
    if (*spin).si_newcompID < (*spin).si_newprefID {
        if (*spin).si_newcompID == 127 as ::core::ffi::c_int
            || (*spin).si_newcompID == 255 as ::core::ffi::c_int
        {
            msg(
                gettext(b"Too many postponed prefixes\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else if (*spin).si_newprefID == 0 as ::core::ffi::c_int
            || (*spin).si_newprefID == 127 as ::core::ffi::c_int
        {
            msg(
                gettext(b"Too many compound flags\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                gettext(
                    b"Too many postponed prefixes and/or compound flags\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                0 as ::core::ffi::c_int,
            );
        }
    }
    if !syllable.is_null() {
        aff_check_string(
            (*spin).si_syllable,
            syllable,
            b"SYLLABLE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_syllable = syllable;
    }
    if !sofofrom.is_null() || !sofoto.is_null() {
        if sofofrom.is_null() || sofoto.is_null() {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Missing SOFO%s line in %s\0".as_ptr() as *const ::core::ffi::c_char),
                if sofofrom.is_null() {
                    b"FROM\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"TO\0".as_ptr() as *const ::core::ffi::c_char
                },
                fname,
            );
        } else if !((*spin).si_sal.ga_len <= 0 as ::core::ffi::c_int) {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Both SAL and SOFO lines in %s\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
            );
        } else {
            aff_check_string(
                (*spin).si_sofofr,
                sofofrom,
                b"SOFOFROM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            aff_check_string(
                (*spin).si_sofoto,
                sofoto,
                b"SOFOTO\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            (*spin).si_sofofr = sofofrom;
            (*spin).si_sofoto = sofoto;
        }
    }
    if !midword.is_null() {
        aff_check_string(
            (*spin).si_midword,
            midword,
            b"MIDWORD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_midword = midword;
    }
    xfree(pc as *mut ::core::ffi::c_void);
    fclose(fd);
    return aff;
}
pub const MAXITEMCNT: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
unsafe extern "C" fn is_aff_rule(
    mut items: *mut *mut ::core::ffi::c_char,
    mut itemcnt: ::core::ffi::c_int,
    mut rulename: *mut ::core::ffi::c_char,
    mut mincount: ::core::ffi::c_int,
) -> bool {
    return strcmp(*items.offset(0 as ::core::ffi::c_int as isize), rulename)
        == 0 as ::core::ffi::c_int
        && (itemcnt == mincount
            || itemcnt > mincount
                && *(*items.offset(mincount as isize)).offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int);
}
unsafe extern "C" fn aff_process_flags(mut affile: *mut afffile_T, mut entry: *mut affentry_T) {
    if !(*entry).ae_flags.is_null()
        && ((*affile).af_compforbid != 0 as ::core::ffi::c_uint
            || (*affile).af_comppermit != 0 as ::core::ffi::c_uint)
    {
        let mut p: *mut ::core::ffi::c_char = (*entry).ae_flags;
        while *p as ::core::ffi::c_int != NUL {
            let mut prevp: *mut ::core::ffi::c_char = p;
            let mut flag: ::core::ffi::c_uint = get_affitem((*affile).af_flagtype, &raw mut p);
            if flag == (*affile).af_comppermit || flag == (*affile).af_compforbid {
                memmove(
                    prevp as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
                p = prevp;
                if flag == (*affile).af_comppermit {
                    (*entry).ae_comppermit = true_0 as ::core::ffi::c_char;
                } else {
                    (*entry).ae_compforbid = true_0 as ::core::ffi::c_char;
                }
            }
            if (*affile).af_flagtype == AFT_NUM
                && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
        if *(*entry).ae_flags as ::core::ffi::c_int == NUL {
            (*entry).ae_flags = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
}
unsafe extern "C" fn spell_info_item(mut s: *mut ::core::ffi::c_char) -> bool {
    return strcmp(s, b"NAME\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"HOME\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"VERSION\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcmp(s, b"AUTHOR\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcmp(s, b"EMAIL\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"COPYRIGHT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn affitem2flag(
    mut flagtype: ::core::ffi::c_int,
    mut item: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
    mut lnum: ::core::ffi::c_int,
) -> ::core::ffi::c_uint {
    let mut p: *mut ::core::ffi::c_char = item;
    let mut res: ::core::ffi::c_uint = get_affitem(flagtype, &raw mut p);
    if res == 0 as ::core::ffi::c_uint {
        if flagtype == AFT_NUM {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Flag is not a number in %s line %d: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                fname,
                lnum,
                item,
            );
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Illegal flag in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
                lnum,
                item,
            );
        }
    }
    if *p as ::core::ffi::c_int != NUL {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(e_affname.get()),
            fname,
            lnum,
            item,
        );
        return 0 as ::core::ffi::c_uint;
    }
    return res;
}
unsafe extern "C" fn get_affitem(
    mut flagtype: ::core::ffi::c_int,
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_uint {
    let mut res: ::core::ffi::c_int = 0;
    if flagtype == AFT_NUM {
        if !ascii_isdigit(**pp as ::core::ffi::c_int) {
            *pp = (*pp).offset(1);
            return 0 as ::core::ffi::c_uint;
        }
        res = getdigits_int(pp, true_0 != 0, 0 as ::core::ffi::c_int);
        if res == 0 as ::core::ffi::c_int {
            res = ZERO_FLAG;
        }
    } else {
        res = mb_ptr2char_adv(pp as *mut *const ::core::ffi::c_char);
        if flagtype == AFT_LONG
            || flagtype == AFT_CAPLONG
                && res >= 'A' as ::core::ffi::c_int
                && res <= 'Z' as ::core::ffi::c_int
        {
            if **pp as ::core::ffi::c_int == NUL {
                return 0 as ::core::ffi::c_uint;
            }
            res = mb_ptr2char_adv(pp as *mut *const ::core::ffi::c_char)
                + (res << 16 as ::core::ffi::c_int);
        }
    }
    return res as ::core::ffi::c_uint;
}
unsafe extern "C" fn process_compflags(
    mut spin: *mut spellinfo_T,
    mut aff: *mut afffile_T,
    mut compflags: *mut ::core::ffi::c_char,
) {
    let mut ci: *mut compitem_T = ::core::ptr::null_mut::<compitem_T>();
    let mut id: ::core::ffi::c_int = 0;
    let mut key: [::core::ffi::c_char; 17] = [0; 17];
    let mut len: ::core::ffi::c_int =
        strlen(compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if !(*spin).si_compflags.is_null() {
        len += strlen((*spin).si_compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char =
        getroom(spin, len as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    if !(*spin).si_compflags.is_null() {
        strcpy(p, (*spin).si_compflags);
        strcat(p, b"/\0".as_ptr() as *const ::core::ffi::c_char);
    }
    (*spin).si_compflags = p;
    let mut tp: *mut uint8_t = (p as *mut uint8_t).offset(strlen(p) as isize);
    p = compflags;
    while *p as ::core::ffi::c_int != NUL {
        if !vim_strchr(
            b"/?*+[]\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            let c2rust_fresh38 = p;
            p = p.offset(1);
            let c2rust_fresh39 = tp;
            tp = tp.offset(1);
            *c2rust_fresh39 = *c2rust_fresh38 as uint8_t;
        } else {
            let mut prevp: *mut ::core::ffi::c_char = p;
            let mut flag: ::core::ffi::c_uint = get_affitem((*aff).af_flagtype, &raw mut p);
            if flag != 0 as ::core::ffi::c_uint {
                xmemcpyz(
                    &raw mut key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    prevp as *const ::core::ffi::c_void,
                    p.offset_from(prevp) as size_t,
                );
                let mut hi: *mut hashitem_T = hash_find(
                    &raw mut (*aff).af_comp,
                    &raw mut key as *mut ::core::ffi::c_char,
                );
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    id = (*((*hi).hi_key as *mut compitem_T)).ci_newID;
                } else {
                    ci = getroom(spin, ::core::mem::size_of::<compitem_T>(), true_0 != 0)
                        as *mut compitem_T;
                    strcpy(
                        &raw mut (*ci).ci_key as *mut ::core::ffi::c_char,
                        &raw mut key as *mut ::core::ffi::c_char,
                    );
                    (*ci).ci_flag = flag;
                    loop {
                        check_renumber(spin);
                        let c2rust_fresh40 = (*spin).si_newcompID;
                        (*spin).si_newcompID = (*spin).si_newcompID - 1;
                        id = c2rust_fresh40;
                        if vim_strchr(b"/?*+[]\\-^\0".as_ptr() as *const ::core::ffi::c_char, id)
                            .is_null()
                        {
                            break;
                        }
                    }
                    (*ci).ci_newID = id;
                    hash_add(
                        &raw mut (*aff).af_comp,
                        &raw mut (*ci).ci_key as *mut ::core::ffi::c_char,
                    );
                }
                let c2rust_fresh41 = tp;
                tp = tp.offset(1);
                *c2rust_fresh41 = id as uint8_t;
            }
            if (*aff).af_flagtype == AFT_NUM
                && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
    }
    *tp = NUL as uint8_t;
}
unsafe extern "C" fn check_renumber(mut spin: *mut spellinfo_T) {
    if (*spin).si_newprefID == (*spin).si_newcompID
        && (*spin).si_newcompID < 128 as ::core::ffi::c_int
    {
        (*spin).si_newprefID = 127 as ::core::ffi::c_int;
        (*spin).si_newcompID = 255 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn flag_in_afflist(
    mut flagtype: ::core::ffi::c_int,
    mut afflist: *mut ::core::ffi::c_char,
    mut flag: ::core::ffi::c_uint,
) -> bool {
    match flagtype {
        AFT_CHAR => return !vim_strchr(afflist, flag as ::core::ffi::c_int).is_null(),
        AFT_CAPLONG | AFT_LONG => {
            let mut p: *mut ::core::ffi::c_char = afflist;
            while *p as ::core::ffi::c_int != NUL {
                let mut n: ::core::ffi::c_uint =
                    mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char)
                        as ::core::ffi::c_uint;
                if (flagtype == AFT_LONG
                    || n >= 'A' as ::core::ffi::c_uint && n <= 'Z' as ::core::ffi::c_uint)
                    && *p as ::core::ffi::c_int != NUL
                {
                    n = (mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char)
                        as ::core::ffi::c_uint)
                        .wrapping_add(n << 16 as ::core::ffi::c_int);
                }
                if n == flag {
                    return true_0 != 0;
                }
            }
        }
        AFT_NUM => {
            let mut p_0: *mut ::core::ffi::c_char = afflist;
            while *p_0 as ::core::ffi::c_int != NUL {
                let mut digits: ::core::ffi::c_int =
                    getdigits_int(&raw mut p_0, true_0 != 0, 0 as ::core::ffi::c_int);
                '_c2rust_label: {
                    if digits >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"digits >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2954 as ::core::ffi::c_uint,
                            b"_Bool flag_in_afflist(int, char *, unsigned int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut n_0: ::core::ffi::c_uint = digits as ::core::ffi::c_uint;
                if n_0 == 0 as ::core::ffi::c_uint {
                    n_0 = ZERO_FLAG as ::core::ffi::c_uint;
                }
                if n_0 == flag {
                    return true_0 != 0;
                }
                if *p_0 as ::core::ffi::c_int != NUL {
                    p_0 = p_0.offset(1);
                }
            }
        }
        _ => {}
    }
    return false_0 != 0;
}
unsafe extern "C" fn aff_check_number(
    mut spinval: ::core::ffi::c_int,
    mut affval: ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
) {
    if spinval != 0 as ::core::ffi::c_int && spinval != affval {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"%s value differs from what is used in another .aff file\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
    }
}
unsafe extern "C" fn aff_check_string(
    mut spinval: *mut ::core::ffi::c_char,
    mut affval: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
) {
    if !spinval.is_null() && strcmp(spinval, affval) != 0 as ::core::ffi::c_int {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"%s value differs from what is used in another .aff file\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
    }
}
unsafe extern "C" fn str_equal(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> bool {
    if s1.is_null() || s2.is_null() {
        return s1 == s2;
    }
    return strcmp(s1, s2) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn add_fromto(
    mut spin: *mut spellinfo_T,
    mut gap: *mut garray_T,
    mut from: *mut ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
) {
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut ftp: *mut fromto_T =
        ga_append_via_ptr(gap, ::core::mem::size_of::<fromto_T>()) as *mut fromto_T;
    spell_casefold(
        curwin.get(),
        from,
        strlen(from) as ::core::ffi::c_int,
        &raw mut word as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*ftp).ft_from = getroom_save(spin, &raw mut word as *mut ::core::ffi::c_char);
    spell_casefold(
        curwin.get(),
        to,
        strlen(to) as ::core::ffi::c_int,
        &raw mut word as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*ftp).ft_to = getroom_save(spin, &raw mut word as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn sal_to_bool(mut s: *mut ::core::ffi::c_char) -> bool {
    return strcmp(s, b"1\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"true\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn spell_free_aff(mut aff: *mut afffile_T) {
    xfree((*aff).af_enc as *mut ::core::ffi::c_void);
    let mut ht: *mut hashtab_T = &raw mut (*aff).af_pref;
    loop {
        let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
        let mut hi: *mut hashitem_T = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                let mut ah: *mut affheader_T = (*hi).hi_key as *mut affheader_T;
                let mut ae: *mut affentry_T = (*ah).ah_first;
                while !ae.is_null() {
                    vim_regfree((*ae).ae_prog);
                    ae = (*ae).ae_next;
                }
            }
            hi = hi.offset(1);
        }
        if ht == &raw mut (*aff).af_suff {
            break;
        }
        ht = &raw mut (*aff).af_suff;
    }
    hash_clear(&raw mut (*aff).af_pref);
    hash_clear(&raw mut (*aff).af_suff);
    hash_clear(&raw mut (*aff).af_comp);
}
unsafe extern "C" fn spell_read_dic(
    mut spin: *mut spellinfo_T,
    mut fname: *mut ::core::ffi::c_char,
    mut affile: *mut afffile_T,
) -> ::core::ffi::c_int {
    let mut ht: hashtab_T = hashtab_T {
        ht_mask: 0,
        ht_used: 0,
        ht_filled: 0,
        ht_changed: 0,
        ht_locked: 0,
        ht_array: ::core::ptr::null_mut::<hashitem_T>(),
        ht_smallarray: [hashitem_T {
            hi_hash: 0,
            hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        }; 16],
    };
    let mut line: [::core::ffi::c_char; 500] = [0; 500];
    let mut store_afflist: [::core::ffi::c_char; 254] = [0; 254];
    let mut pc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut w: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut non_ascii: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retval: ::core::ffi::c_int = OK;
    let mut message: [::core::ffi::c_char; 754] = [0; 754];
    let mut duplicate: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last_msg_time: Timestamp = 0 as Timestamp;
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return FAIL;
    }
    hash_init(&raw mut ht);
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"Reading dictionary file %s...\0".as_ptr() as *const ::core::ffi::c_char),
        fname,
    );
    spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    (*spin).si_msg_count = 999999 as ::core::ffi::c_int;
    if vim_fgets(&raw mut line as *mut ::core::ffi::c_char, MAXLINELEN, fd) as ::core::ffi::c_int
        != 0
        || !ascii_isdigit(
            *skipwhite(&raw mut line as *mut ::core::ffi::c_char) as ::core::ffi::c_int
        )
    {
        semsg(
            gettext(b"E760: No word count in %s\0".as_ptr() as *const ::core::ffi::c_char),
            fname,
        );
    }
    while !vim_fgets(&raw mut line as *mut ::core::ffi::c_char, MAXLINELEN, fd) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if line[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == '#' as ::core::ffi::c_int
            || line[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int
        {
            continue;
        }
        let mut l: ::core::ffi::c_int =
            strlen(&raw mut line as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        while l > 0 as ::core::ffi::c_int
            && line[(l - 1 as ::core::ffi::c_int) as usize] as uint8_t as ::core::ffi::c_int
                <= ' ' as ::core::ffi::c_int
        {
            l -= 1;
        }
        if l == 0 as ::core::ffi::c_int {
            continue;
        }
        line[l as usize] = NUL as ::core::ffi::c_char;
        if (*spin).si_conv.vc_type != CONV_NONE as ::core::ffi::c_int {
            pc = string_convert(
                &raw mut (*spin).si_conv,
                &raw mut line as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<size_t>(),
            );
            if pc.is_null() {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion failure for word in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    &raw mut line as *mut ::core::ffi::c_char,
                );
                continue;
            } else {
                w = pc;
            }
        } else {
            pc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            w = &raw mut line as *mut ::core::ffi::c_char;
        }
        let mut afflist: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = w;
        while *p as ::core::ffi::c_int != NUL {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int)
            {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            } else if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                *p = NUL as ::core::ffi::c_char;
                afflist = p.offset(1 as ::core::ffi::c_int as isize);
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if (*spin).si_ascii != 0 && has_non_ascii(w) as ::core::ffi::c_int != 0 {
            non_ascii += 1;
            xfree(pc as *mut ::core::ffi::c_void);
        } else {
            if (*spin).si_verbose != 0 && (*spin).si_msg_count > 10000 as ::core::ffi::c_int {
                (*spin).si_msg_count = 0 as ::core::ffi::c_int;
                if os_time() > last_msg_time {
                    last_msg_time = os_time();
                    vim_snprintf(
                        &raw mut message as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 754]>(),
                        gettext(b"line %6d, word %6d - %s\0".as_ptr() as *const ::core::ffi::c_char),
                        lnum,
                        (*spin).si_foldwcount + (*spin).si_keepwcount,
                        w,
                    );
                    msg_start();
                    msg_outtrans_long(
                        &raw mut message as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                    );
                    msg_clr_eos();
                    msg_didout.set(false_0 != 0);
                    msg_col.set(0 as ::core::ffi::c_int);
                    ui_flush();
                }
            }
            let mut dw: *mut ::core::ffi::c_char = getroom_save(spin, w);
            if dw.is_null() {
                retval = FAIL;
                xfree(pc as *mut ::core::ffi::c_void);
                break;
            } else {
                let mut hash: hash_T = hash_hash(dw);
                let mut hi: *mut hashitem_T = hash_lookup(&raw mut ht, dw, strlen(dw), hash);
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    if p_verbose.get() > 0 as OptInt {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Duplicate word in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            dw,
                        );
                    } else if duplicate == 0 as ::core::ffi::c_int {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"First duplicate word in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            dw,
                        );
                    }
                    duplicate += 1;
                } else {
                    hash_add_item(&raw mut ht, hi, dw, hash);
                }
                let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                store_afflist[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                let mut pfxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut need_affix: bool = false_0 != 0;
                if !afflist.is_null() {
                    flags |= get_affix_flags(affile, afflist);
                    if (*affile).af_needaffix != 0 as ::core::ffi::c_uint
                        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_needaffix)
                            as ::core::ffi::c_int
                            != 0
                    {
                        need_affix = true_0 != 0;
                    }
                    if (*affile).af_pfxpostpone != 0 {
                        pfxlen = get_pfxlist(
                            affile,
                            afflist,
                            &raw mut store_afflist as *mut ::core::ffi::c_char,
                        );
                    }
                    if !(*spin).si_compflags.is_null() {
                        get_compflags(
                            affile,
                            afflist,
                            (&raw mut store_afflist as *mut ::core::ffi::c_char)
                                .offset(pfxlen as isize),
                        );
                    }
                }
                if store_word(
                    spin,
                    dw,
                    flags,
                    (*spin).si_region,
                    &raw mut store_afflist as *mut ::core::ffi::c_char,
                    need_affix,
                ) == FAIL
                {
                    retval = FAIL;
                }
                if !afflist.is_null() {
                    if store_aff_word(
                        spin,
                        dw,
                        afflist,
                        affile,
                        &raw mut (*affile).af_suff,
                        &raw mut (*affile).af_pref,
                        CONDIT_SUF,
                        flags,
                        &raw mut store_afflist as *mut ::core::ffi::c_char,
                        pfxlen,
                    ) == FAIL
                    {
                        retval = FAIL;
                    }
                    if store_aff_word(
                        spin,
                        dw,
                        afflist,
                        affile,
                        &raw mut (*affile).af_pref,
                        ::core::ptr::null_mut::<hashtab_T>(),
                        CONDIT_SUF,
                        flags,
                        &raw mut store_afflist as *mut ::core::ffi::c_char,
                        pfxlen,
                    ) == FAIL
                    {
                        retval = FAIL;
                    }
                }
                xfree(pc as *mut ::core::ffi::c_void);
            }
        }
    }
    if duplicate > 0 as ::core::ffi::c_int {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"%d duplicate word(s) in %s\0".as_ptr() as *const ::core::ffi::c_char),
            duplicate,
            fname,
        );
    }
    if (*spin).si_ascii != 0 && non_ascii > 0 as ::core::ffi::c_int {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"Ignored %d word(s) with non-ASCII characters in %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            non_ascii,
            fname,
        );
    }
    hash_clear(&raw mut ht);
    fclose(fd);
    return retval;
}
unsafe extern "C" fn get_affix_flags(
    mut affile: *mut afffile_T,
    mut afflist: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*affile).af_keepcase != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_keepcase)
            as ::core::ffi::c_int
            != 0
    {
        flags |= WF_KEEPCAP as ::core::ffi::c_int | WF_FIXCAP as ::core::ffi::c_int;
    }
    if (*affile).af_rare != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_rare) as ::core::ffi::c_int
            != 0
    {
        flags |= WF_RARE as ::core::ffi::c_int;
    }
    if (*affile).af_bad != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_bad) as ::core::ffi::c_int
            != 0
    {
        flags |= WF_BANNED as ::core::ffi::c_int;
    }
    if (*affile).af_needcomp != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_needcomp)
            as ::core::ffi::c_int
            != 0
    {
        flags |= WF_NEEDCOMP as ::core::ffi::c_int;
    }
    if (*affile).af_comproot != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_comproot)
            as ::core::ffi::c_int
            != 0
    {
        flags |= WF_COMPROOT as ::core::ffi::c_int;
    }
    if (*affile).af_nosuggest != 0 as ::core::ffi::c_uint
        && flag_in_afflist((*affile).af_flagtype, afflist, (*affile).af_nosuggest)
            as ::core::ffi::c_int
            != 0
    {
        flags |= WF_NOSUGGEST as ::core::ffi::c_int;
    }
    return flags;
}
unsafe extern "C" fn get_pfxlist(
    mut affile: *mut afffile_T,
    mut afflist: *mut ::core::ffi::c_char,
    mut store_afflist: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut key: [::core::ffi::c_char; 17] = [0; 17];
    let mut p: *mut ::core::ffi::c_char = afflist;
    while *p as ::core::ffi::c_int != NUL {
        let mut prevp: *mut ::core::ffi::c_char = p;
        if get_affitem((*affile).af_flagtype, &raw mut p) != 0 as ::core::ffi::c_uint {
            xmemcpyz(
                &raw mut key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                prevp as *const ::core::ffi::c_void,
                p.offset_from(prevp) as size_t,
            );
            let mut hi: *mut hashitem_T = hash_find(
                &raw mut (*affile).af_pref,
                &raw mut key as *mut ::core::ffi::c_char,
            );
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                let mut id: ::core::ffi::c_int = (*((*hi).hi_key as *mut affheader_T)).ah_newID;
                if id != 0 as ::core::ffi::c_int {
                    let c2rust_fresh32 = cnt;
                    cnt = cnt + 1;
                    *store_afflist.offset(c2rust_fresh32 as isize) =
                        id as uint8_t as ::core::ffi::c_char;
                }
            }
        }
        if (*affile).af_flagtype == AFT_NUM && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
    }
    *store_afflist.offset(cnt as isize) = NUL as ::core::ffi::c_char;
    return cnt;
}
unsafe extern "C" fn get_compflags(
    mut affile: *mut afffile_T,
    mut afflist: *mut ::core::ffi::c_char,
    mut store_afflist: *mut ::core::ffi::c_char,
) {
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut key: [::core::ffi::c_char; 17] = [0; 17];
    let mut p: *mut ::core::ffi::c_char = afflist;
    while *p as ::core::ffi::c_int != NUL {
        let mut prevp: *mut ::core::ffi::c_char = p;
        if get_affitem((*affile).af_flagtype, &raw mut p) != 0 as ::core::ffi::c_uint {
            xmemcpyz(
                &raw mut key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                prevp as *const ::core::ffi::c_void,
                p.offset_from(prevp) as size_t,
            );
            let mut hi: *mut hashitem_T = hash_find(
                &raw mut (*affile).af_comp,
                &raw mut key as *mut ::core::ffi::c_char,
            );
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                let c2rust_fresh31 = cnt;
                cnt = cnt + 1;
                *store_afflist.offset(c2rust_fresh31 as isize) =
                    (*((*hi).hi_key as *mut compitem_T)).ci_newID as uint8_t as ::core::ffi::c_char;
            }
        }
        if (*affile).af_flagtype == AFT_NUM && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
    }
    *store_afflist.offset(cnt as isize) = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn store_aff_word(
    mut spin: *mut spellinfo_T,
    mut word: *mut ::core::ffi::c_char,
    mut afflist: *mut ::core::ffi::c_char,
    mut affile: *mut afffile_T,
    mut ht: *mut hashtab_T,
    mut xht: *mut hashtab_T,
    mut condit: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut pfxlist: *mut ::core::ffi::c_char,
    mut pfxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ae: *mut affentry_T = ::core::ptr::null_mut::<affentry_T>();
    let mut newword: [::core::ffi::c_char; 254] = [0; 254];
    let mut retval: ::core::ffi::c_int = OK;
    let mut j: ::core::ffi::c_int = 0;
    let mut store_afflist: [::core::ffi::c_char; 254] = [0; 254];
    let mut pfx_pfxlist: [::core::ffi::c_char; 254] = [0; 254];
    let mut wordlen: size_t = strlen(word);
    let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int && retval == OK {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            let mut ah: *mut affheader_T = (*hi).hi_key as *mut affheader_T;
            if (condit & CONDIT_COMB == 0 as ::core::ffi::c_int || (*ah).ah_combine != 0)
                && flag_in_afflist((*affile).af_flagtype, afflist, (*ah).ah_flag)
                    as ::core::ffi::c_int
                    != 0
            {
                ae = (*ah).ah_first;
                while !ae.is_null() {
                    if (!xht.is_null()
                        || (*affile).af_pfxpostpone == 0
                        || !(*ae).ae_chop.is_null()
                        || !(*ae).ae_flags.is_null())
                        && ((*ae).ae_chop.is_null() || strlen((*ae).ae_chop) < wordlen)
                        && ((*ae).ae_prog.is_null()
                            || vim_regexec_prog(
                                &raw mut (*ae).ae_prog,
                                false_0 != 0,
                                word,
                                0 as colnr_T,
                            ) as ::core::ffi::c_int
                                != 0)
                        && (condit & CONDIT_CFIX == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                            == (condit & CONDIT_AFF == 0 as ::core::ffi::c_int
                                || (*ae).ae_flags.is_null()
                                || !flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_circumfix,
                                )) as ::core::ffi::c_int
                    {
                        if xht.is_null() {
                            if (*ae).ae_add.is_null() {
                                *(&raw mut newword as *mut ::core::ffi::c_char) =
                                    NUL as ::core::ffi::c_char;
                            } else {
                                xstrlcpy(
                                    &raw mut newword as *mut ::core::ffi::c_char,
                                    (*ae).ae_add,
                                    MAXWLEN as ::core::ffi::c_int as size_t,
                                );
                            }
                            let mut p: *mut ::core::ffi::c_char = word;
                            if !(*ae).ae_chop.is_null() {
                                let mut i: ::core::ffi::c_int = mb_charlen((*ae).ae_chop);
                                while i > 0 as ::core::ffi::c_int {
                                    p = p.offset(utfc_ptr2len(p) as isize);
                                    i -= 1;
                                }
                            }
                            xstrlcat(
                                &raw mut newword as *mut ::core::ffi::c_char,
                                p,
                                MAXWLEN as ::core::ffi::c_int as size_t,
                            );
                        } else {
                            xstrlcpy(
                                &raw mut newword as *mut ::core::ffi::c_char,
                                word,
                                MAXWLEN as ::core::ffi::c_int as size_t,
                            );
                            if !(*ae).ae_chop.is_null() {
                                let mut p_0: *mut ::core::ffi::c_char = (&raw mut newword
                                    as *mut ::core::ffi::c_char)
                                    .offset(strlen(&raw mut newword as *mut ::core::ffi::c_char)
                                        as isize);
                                let mut i_0: ::core::ffi::c_int = mb_charlen((*ae).ae_chop);
                                while i_0 > 0 as ::core::ffi::c_int {
                                    p_0 = p_0.offset(
                                        -((utf_head_off(
                                            &raw mut newword as *mut ::core::ffi::c_char,
                                            p_0.offset(-(1 as ::core::ffi::c_int as isize)),
                                        ) + 1 as ::core::ffi::c_int)
                                            as isize),
                                    );
                                    i_0 -= 1;
                                }
                                *p_0 = NUL as ::core::ffi::c_char;
                            }
                            if !(*ae).ae_add.is_null() {
                                xstrlcat(
                                    &raw mut newword as *mut ::core::ffi::c_char,
                                    (*ae).ae_add,
                                    MAXWLEN as ::core::ffi::c_int as size_t,
                                );
                            }
                        }
                        let mut use_flags: ::core::ffi::c_int = flags;
                        let mut use_pfxlist: *mut ::core::ffi::c_char = pfxlist;
                        let mut use_pfxlen: ::core::ffi::c_int = pfxlen;
                        let mut need_affix: bool = false_0 != 0;
                        let mut use_condit: ::core::ffi::c_int = condit | CONDIT_COMB | CONDIT_AFF;
                        if !(*ae).ae_flags.is_null() {
                            use_flags |= get_affix_flags(affile, (*ae).ae_flags);
                            if (*affile).af_needaffix != 0 as ::core::ffi::c_uint
                                && flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_needaffix,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                need_affix = true_0 != 0;
                            }
                            if (*affile).af_circumfix != 0 as ::core::ffi::c_uint
                                && flag_in_afflist(
                                    (*affile).af_flagtype,
                                    (*ae).ae_flags,
                                    (*affile).af_circumfix,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                use_condit |= CONDIT_CFIX;
                                if condit & CONDIT_CFIX == 0 as ::core::ffi::c_int {
                                    need_affix = true_0 != 0;
                                }
                            }
                            if (*affile).af_pfxpostpone != 0 || !(*spin).si_compflags.is_null() {
                                if (*affile).af_pfxpostpone != 0 {
                                    use_pfxlen = get_pfxlist(
                                        affile,
                                        (*ae).ae_flags,
                                        &raw mut store_afflist as *mut ::core::ffi::c_char,
                                    );
                                } else {
                                    use_pfxlen = 0 as ::core::ffi::c_int;
                                }
                                use_pfxlist = &raw mut store_afflist as *mut ::core::ffi::c_char;
                                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i_1 < pfxlen {
                                    j = 0 as ::core::ffi::c_int;
                                    while j < use_pfxlen {
                                        if *pfxlist.offset(i_1 as isize) as ::core::ffi::c_int
                                            == *use_pfxlist.offset(j as isize) as ::core::ffi::c_int
                                        {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if j == use_pfxlen {
                                        let c2rust_fresh29 = use_pfxlen;
                                        use_pfxlen = use_pfxlen + 1;
                                        *use_pfxlist.offset(c2rust_fresh29 as isize) =
                                            *pfxlist.offset(i_1 as isize);
                                    }
                                    i_1 += 1;
                                }
                                if !(*spin).si_compflags.is_null() {
                                    get_compflags(
                                        affile,
                                        (*ae).ae_flags,
                                        use_pfxlist.offset(use_pfxlen as isize),
                                    );
                                } else {
                                    *use_pfxlist.offset(use_pfxlen as isize) =
                                        NUL as ::core::ffi::c_char;
                                }
                                let mut i_2: ::core::ffi::c_int = pfxlen;
                                while *pfxlist.offset(i_2 as isize) as ::core::ffi::c_int != NUL {
                                    j = use_pfxlen;
                                    while *use_pfxlist.offset(j as isize) as ::core::ffi::c_int
                                        != NUL
                                    {
                                        if *pfxlist.offset(i_2 as isize) as ::core::ffi::c_int
                                            == *use_pfxlist.offset(j as isize) as ::core::ffi::c_int
                                        {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if *use_pfxlist.offset(j as isize) as ::core::ffi::c_int == NUL
                                    {
                                        let c2rust_fresh30 = j;
                                        j = j + 1;
                                        *use_pfxlist.offset(c2rust_fresh30 as isize) =
                                            *pfxlist.offset(i_2 as isize);
                                        *use_pfxlist.offset(j as isize) =
                                            NUL as ::core::ffi::c_char;
                                    }
                                    i_2 += 1;
                                }
                            }
                        }
                        if !use_pfxlist.is_null() && (*ae).ae_compforbid as ::core::ffi::c_int != 0
                        {
                            xmemcpyz(
                                &raw mut pfx_pfxlist as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                use_pfxlist as *const ::core::ffi::c_void,
                                use_pfxlen as size_t,
                            );
                            use_pfxlist = &raw mut pfx_pfxlist as *mut ::core::ffi::c_char;
                        }
                        if !(*spin).si_prefroot.is_null()
                            && !(*(*spin).si_prefroot).wn_sibling.is_null()
                        {
                            use_flags |= WF_HAS_AFF as ::core::ffi::c_int;
                            if (*ah).ah_combine == 0 && !use_pfxlist.is_null() {
                                use_pfxlist = use_pfxlist.offset(use_pfxlen as isize);
                            }
                        }
                        if !(*spin).si_compflags.is_null() && (*ae).ae_comppermit == 0 {
                            if !xht.is_null() {
                                use_flags |= WF_NOCOMPAFT as ::core::ffi::c_int;
                            } else {
                                use_flags |= WF_NOCOMPBEF as ::core::ffi::c_int;
                            }
                        }
                        if store_word(
                            spin,
                            &raw mut newword as *mut ::core::ffi::c_char,
                            use_flags,
                            (*spin).si_region,
                            use_pfxlist,
                            need_affix,
                        ) == FAIL
                        {
                            retval = FAIL;
                        }
                        if condit & CONDIT_SUF != 0 && !(*ae).ae_flags.is_null() {
                            if store_aff_word(
                                spin,
                                &raw mut newword as *mut ::core::ffi::c_char,
                                (*ae).ae_flags,
                                affile,
                                &raw mut (*affile).af_suff,
                                xht,
                                use_condit
                                    & (if xht.is_null() {
                                        !(0 as ::core::ffi::c_int)
                                    } else {
                                        !CONDIT_SUF
                                    }),
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            ) == FAIL
                            {
                                retval = FAIL;
                            }
                        }
                        if !xht.is_null() && (*ah).ah_combine != 0 {
                            if store_aff_word(
                                spin,
                                &raw mut newword as *mut ::core::ffi::c_char,
                                afflist,
                                affile,
                                xht,
                                ::core::ptr::null_mut::<hashtab_T>(),
                                use_condit,
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            ) == FAIL
                                || !(*ae).ae_flags.is_null()
                                    && store_aff_word(
                                        spin,
                                        &raw mut newword as *mut ::core::ffi::c_char,
                                        (*ae).ae_flags,
                                        affile,
                                        xht,
                                        ::core::ptr::null_mut::<hashtab_T>(),
                                        use_condit,
                                        use_flags,
                                        use_pfxlist,
                                        pfxlen,
                                    ) == FAIL
                            {
                                retval = FAIL;
                            }
                        }
                    }
                    ae = (*ae).ae_next;
                }
            }
        }
        hi = hi.offset(1);
    }
    return retval;
}
unsafe extern "C" fn spell_read_wordfile(
    mut spin: *mut spellinfo_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut rline: [::core::ffi::c_char; 500] = [0; 500];
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = OK;
    let mut did_word: bool = false_0 != 0;
    let mut non_ascii: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return FAIL;
    }
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"Reading word file %s...\0".as_ptr() as *const ::core::ffi::c_char),
        fname,
    );
    spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    while !vim_fgets(&raw mut rline as *mut ::core::ffi::c_char, MAXLINELEN, fd) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if *(&raw mut rline as *mut ::core::ffi::c_char) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
        {
            continue;
        }
        let mut l: ::core::ffi::c_int =
            strlen(&raw mut rline as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        while l > 0 as ::core::ffi::c_int
            && rline[(l - 1 as ::core::ffi::c_int) as usize] as uint8_t as ::core::ffi::c_int
                <= ' ' as ::core::ffi::c_int
        {
            l -= 1;
        }
        if l == 0 as ::core::ffi::c_int {
            continue;
        }
        rline[l as usize] = NUL as ::core::ffi::c_char;
        xfree(pc as *mut ::core::ffi::c_void);
        if (*spin).si_conv.vc_type != CONV_NONE as ::core::ffi::c_int {
            pc = string_convert(
                &raw mut (*spin).si_conv,
                &raw mut rline as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<size_t>(),
            );
            if pc.is_null() {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion failure for word in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    &raw mut rline as *mut ::core::ffi::c_char,
                );
                continue;
            } else {
                line = pc;
            }
        } else {
            pc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            line = &raw mut rline as *mut ::core::ffi::c_char;
        }
        if *line as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
            line = line.offset(1);
            if strncmp(
                line,
                b"encoding=\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                if (*spin).si_conv.vc_type != CONV_NONE as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Duplicate /encoding= line ignored in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                        line.offset(-(1 as ::core::ffi::c_int as isize)),
                    );
                } else if did_word {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"/encoding= line after word ignored in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                        line.offset(-(1 as ::core::ffi::c_int as isize)),
                    );
                } else {
                    line = line.offset(9 as ::core::ffi::c_int as isize);
                    let mut enc: *mut ::core::ffi::c_char = enc_canonize(line);
                    if (*spin).si_ascii == 0
                        && convert_setup(&raw mut (*spin).si_conv, enc, p_enc.get()) == FAIL
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Conversion in %s not supported: from %s to %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            line,
                            p_enc.get(),
                        );
                    }
                    xfree(enc as *mut ::core::ffi::c_void);
                    (*spin).si_conv.vc_fail = true_0 != 0;
                }
            } else if strncmp(
                line,
                b"regions=\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                if (*spin).si_region_count > 1 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Duplicate /regions= line ignored in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                        line,
                    );
                } else {
                    line = line.offset(8 as ::core::ffi::c_int as isize);
                    if strlen(line)
                        > (MAXREGIONS as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as size_t
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Too many regions in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            line,
                        );
                    } else {
                        (*spin).si_region_count =
                            strlen(line) as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
                        strcpy(
                            &raw mut (*spin).si_region_name as *mut ::core::ffi::c_char,
                            line,
                        );
                        (*spin).si_region = ((1 as ::core::ffi::c_int) << (*spin).si_region_count)
                            - 1 as ::core::ffi::c_int;
                    }
                }
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"/ line ignored in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    line.offset(-(1 as ::core::ffi::c_int as isize)),
                );
            }
        } else {
            let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut regionmask: ::core::ffi::c_int = (*spin).si_region;
            let mut p: *mut ::core::ffi::c_char = vim_strchr(line, '/' as ::core::ffi::c_int);
            if !p.is_null() {
                let c2rust_fresh28 = p;
                p = p.offset(1);
                *c2rust_fresh28 = NUL as ::core::ffi::c_char;
                while *p as ::core::ffi::c_int != NUL {
                    if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                        flags |= WF_KEEPCAP as ::core::ffi::c_int | WF_FIXCAP as ::core::ffi::c_int;
                    } else if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        flags |= WF_BANNED as ::core::ffi::c_int;
                    } else if *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                        flags |= WF_RARE as ::core::ffi::c_int;
                    } else if ascii_isdigit(*p as uint8_t as ::core::ffi::c_int) {
                        if flags & WF_REGION as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            regionmask = 0 as ::core::ffi::c_int;
                        }
                        flags |= WF_REGION as ::core::ffi::c_int;
                        l = *p as uint8_t as ::core::ffi::c_int - '0' as ::core::ffi::c_int;
                        if l == 0 as ::core::ffi::c_int || l > (*spin).si_region_count {
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Invalid region nr in %s line %d: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                                lnum,
                                p,
                            );
                            break;
                        } else {
                            regionmask |= (1 as ::core::ffi::c_int) << l - 1 as ::core::ffi::c_int;
                        }
                    } else {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Unrecognized flags in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            p,
                        );
                        break;
                    }
                    p = p.offset(1);
                }
            }
            if (*spin).si_ascii != 0 && has_non_ascii(line) as ::core::ffi::c_int != 0 {
                non_ascii += 1;
            } else if store_word(
                spin,
                line,
                flags,
                regionmask,
                ::core::ptr::null::<::core::ffi::c_char>(),
                false_0 != 0,
            ) == FAIL
            {
                retval = FAIL;
                break;
            } else {
                did_word = true_0 != 0;
            }
        }
    }
    xfree(pc as *mut ::core::ffi::c_void);
    fclose(fd);
    if (*spin).si_ascii != 0 && non_ascii > 0 as ::core::ffi::c_int {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            gettext(b"Ignored %d words with non-ASCII characters\0".as_ptr()
                as *const ::core::ffi::c_char),
            non_ascii,
        );
        spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    }
    return retval;
}
unsafe extern "C" fn getroom(
    mut spin: *mut spellinfo_T,
    mut len: size_t,
    mut align: bool,
) -> *mut ::core::ffi::c_void {
    let mut bl: *mut sblock_T = (*spin).si_blocks;
    '_c2rust_label: {
        if len <= 16000 as size_t {
        } else {
            __assert_fail(
                b"len <= SBLOCKSIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3771 as ::core::ffi::c_uint,
                b"void *getroom(spellinfo_T *, size_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if align as ::core::ffi::c_int != 0 && !bl.is_null() {
        (*bl).sb_used = (((*bl).sb_used as size_t)
            .wrapping_add(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_sub(1 as size_t)
            & !::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_sub(1 as size_t))
            as ::core::ffi::c_int;
    }
    if bl.is_null() || ((*bl).sb_used as size_t).wrapping_add(len) > SBLOCKSIZE as size_t {
        bl = xcalloc(
            1 as size_t,
            (16 as size_t)
                .wrapping_add(SBLOCKSIZE as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut sblock_T;
        (*bl).sb_next = (*spin).si_blocks;
        (*spin).si_blocks = bl;
        (*bl).sb_used = 0 as ::core::ffi::c_int;
        (*spin).si_blocks_cnt += 1;
    }
    let mut p: *mut ::core::ffi::c_char =
        (&raw mut (*bl).sb_data as *mut ::core::ffi::c_char).offset((*bl).sb_used as isize);
    (*bl).sb_used += len as ::core::ffi::c_int;
    return p as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn getroom_save(
    mut spin: *mut spellinfo_T,
    mut s: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let s_size: size_t = strlen(s).wrapping_add(1 as size_t);
    return memcpy(
        getroom(spin, s_size, false_0 != 0),
        s as *const ::core::ffi::c_void,
        s_size,
    ) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn free_blocks(mut bl: *mut sblock_T) {
    while !bl.is_null() {
        let mut next: *mut sblock_T = (*bl).sb_next;
        xfree(bl as *mut ::core::ffi::c_void);
        bl = next;
    }
}
unsafe extern "C" fn wordtree_alloc(mut spin: *mut spellinfo_T) -> *mut wordnode_T {
    return getroom(spin, ::core::mem::size_of::<wordnode_T>(), true_0 != 0) as *mut wordnode_T;
}
unsafe extern "C" fn valid_spell_word(
    mut word: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> bool {
    if !utf_valid_string(word, end) {
        return false_0 != 0;
    }
    let mut p: *const ::core::ffi::c_char = word;
    while *p as ::core::ffi::c_int != NUL && p < end {
        if (*p as uint8_t as ::core::ffi::c_int) < ' ' as ::core::ffi::c_int
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return false_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return true_0 != 0;
}
unsafe extern "C" fn store_word(
    mut spin: *mut spellinfo_T,
    mut word: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut region: ::core::ffi::c_int,
    mut pfxlist: *const ::core::ffi::c_char,
    mut need_affix: bool,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = strlen(word) as ::core::ffi::c_int;
    let mut ct: ::core::ffi::c_int = captype(word, word.offset(len as isize));
    let mut foldword: [::core::ffi::c_char; 254] = [0; 254];
    let mut res: ::core::ffi::c_int = OK;
    if !valid_spell_word(word, word.offset(len as isize)) {
        return FAIL;
    }
    spell_casefold(
        curwin.get(),
        word,
        len,
        &raw mut foldword as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    let mut p: *const ::core::ffi::c_char = pfxlist;
    while res == OK {
        if !need_affix || !p.is_null() && *p as ::core::ffi::c_int != NUL {
            res = tree_add_word(
                spin,
                &raw mut foldword as *mut ::core::ffi::c_char,
                (*spin).si_foldroot,
                ct | flags,
                region,
                if p.is_null() {
                    0 as ::core::ffi::c_int
                } else {
                    *p as ::core::ffi::c_int
                },
            );
        }
        if p.is_null() || *p as ::core::ffi::c_int == NUL {
            break;
        }
        p = p.offset(1);
    }
    (*spin).si_foldwcount += 1;
    if res == OK
        && (ct == WF_KEEPCAP as ::core::ffi::c_int || flags & WF_KEEPCAP as ::core::ffi::c_int != 0)
    {
        let mut p_0: *const ::core::ffi::c_char = pfxlist;
        while res == OK {
            if !need_affix || !p_0.is_null() && *p_0 as ::core::ffi::c_int != NUL {
                res = tree_add_word(
                    spin,
                    word,
                    (*spin).si_keeproot,
                    flags,
                    region,
                    if p_0.is_null() {
                        0 as ::core::ffi::c_int
                    } else {
                        *p_0 as ::core::ffi::c_int
                    },
                );
            }
            if p_0.is_null() || *p_0 as ::core::ffi::c_int == NUL {
                break;
            }
            p_0 = p_0.offset(1);
        }
        (*spin).si_keepwcount += 1;
    }
    return res;
}
unsafe extern "C" fn tree_add_word(
    mut spin: *mut spellinfo_T,
    mut word: *const ::core::ffi::c_char,
    mut root: *mut wordnode_T,
    mut flags: ::core::ffi::c_int,
    mut region: ::core::ffi::c_int,
    mut affixID: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut node: *mut wordnode_T = root;
    let mut prev: *mut *mut wordnode_T = ::core::ptr::null_mut::<*mut wordnode_T>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if !node.is_null() && (*node).wn_refs > 1 as ::core::ffi::c_int {
            (*node).wn_refs -= 1;
            let mut copyprev: *mut *mut wordnode_T = prev;
            let mut copyp: *mut wordnode_T = node;
            while !copyp.is_null() {
                let mut np: *mut wordnode_T = get_wordnode(spin);
                if np.is_null() {
                    return FAIL;
                }
                (*np).wn_child = (*copyp).wn_child;
                if !(*np).wn_child.is_null() {
                    (*(*np).wn_child).wn_refs += 1;
                }
                (*np).wn_byte = (*copyp).wn_byte;
                if (*np).wn_byte as ::core::ffi::c_int == NUL {
                    (*np).wn_flags = (*copyp).wn_flags;
                    (*np).wn_region = (*copyp).wn_region;
                    (*np).wn_affixID = (*copyp).wn_affixID;
                }
                (*np).wn_refs = 1 as ::core::ffi::c_int;
                if !copyprev.is_null() {
                    *copyprev = np;
                }
                copyprev = &raw mut (*np).wn_sibling;
                if copyp == node {
                    node = np;
                }
                copyp = (*copyp).wn_sibling;
            }
        }
        while !node.is_null()
            && (((*node).wn_byte as ::core::ffi::c_int)
                < *word.offset(i as isize) as uint8_t as ::core::ffi::c_int
                || (*node).wn_byte as ::core::ffi::c_int == NUL
                    && (if flags < 0 as ::core::ffi::c_int {
                        (((*node).wn_affixID as ::core::ffi::c_uint)
                            < affixID as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    } else {
                        (((*node).wn_flags as ::core::ffi::c_uint)
                            < (flags & WN_MASK) as ::core::ffi::c_uint
                            || (*node).wn_flags as ::core::ffi::c_int == flags & WN_MASK
                                && (if (*spin).si_sugtree != 0 {
                                    (((*node).wn_region as ::core::ffi::c_int
                                        & 0xffff as ::core::ffi::c_int)
                                        < region)
                                        as ::core::ffi::c_int
                                } else {
                                    (((*node).wn_affixID as ::core::ffi::c_uint)
                                        < affixID as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                }) != 0) as ::core::ffi::c_int
                    }) != 0)
        {
            prev = &raw mut (*node).wn_sibling;
            node = *prev;
        }
        if node.is_null()
            || (*node).wn_byte as ::core::ffi::c_int
                != *word.offset(i as isize) as uint8_t as ::core::ffi::c_int
            || *word.offset(i as isize) as ::core::ffi::c_int == NUL
                && (flags < 0 as ::core::ffi::c_int
                    || (*spin).si_sugtree != 0
                    || (*node).wn_flags as ::core::ffi::c_int != flags & WN_MASK
                    || (*node).wn_affixID as ::core::ffi::c_int != affixID)
        {
            let mut np_0: *mut wordnode_T = get_wordnode(spin);
            if np_0.is_null() {
                return FAIL;
            }
            (*np_0).wn_byte = *word.offset(i as isize) as uint8_t;
            if node.is_null() {
                (*np_0).wn_refs = 1 as ::core::ffi::c_int;
            } else {
                (*np_0).wn_refs = (*node).wn_refs;
                (*node).wn_refs = 1 as ::core::ffi::c_int;
            }
            if !prev.is_null() {
                *prev = np_0;
            }
            (*np_0).wn_sibling = node;
            node = np_0;
        }
        if *word.offset(i as isize) as ::core::ffi::c_int == NUL {
            (*node).wn_flags = flags as uint16_t;
            (*node).wn_region = ((*node).wn_region as ::core::ffi::c_int
                | region as int16_t as ::core::ffi::c_int)
                as int16_t;
            (*node).wn_affixID = affixID as uint8_t;
            break;
        } else {
            prev = &raw mut (*node).wn_child;
            node = *prev;
            i += 1;
        }
    }
    (*spin).si_msg_count += 1;
    if (*spin).si_compress_cnt > 1 as ::core::ffi::c_int {
        (*spin).si_compress_cnt -= 1;
        if (*spin).si_compress_cnt == 1 as ::core::ffi::c_int {
            (*spin).si_blocks_cnt += compress_inc.get();
        }
    }
    if if (*spin).si_compress_cnt == 1 as ::core::ffi::c_int {
        ((*spin).si_free_count < MAXWLEN as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        ((*spin).si_blocks_cnt >= compress_start.get()) as ::core::ffi::c_int
    } != 0
    {
        (*spin).si_blocks_cnt -= compress_inc.get();
        (*spin).si_compress_cnt = compress_added.get();
        if (*spin).si_verbose != 0 {
            msg_start();
            msg_puts(gettext(msg_compressing.get()));
            msg_clr_eos();
            msg_didout.set(false_0 != 0);
            msg_col.set(0 as ::core::ffi::c_int);
            ui_flush();
        }
        wordtree_compress(
            spin,
            (*spin).si_foldroot,
            b"case-folded\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if affixID >= 0 as ::core::ffi::c_int {
            wordtree_compress(
                spin,
                (*spin).si_keeproot,
                b"keep-case\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    return OK;
}
unsafe extern "C" fn get_wordnode(mut spin: *mut spellinfo_T) -> *mut wordnode_T {
    let mut n: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    if (*spin).si_first_free.is_null() {
        n = getroom(spin, ::core::mem::size_of::<wordnode_T>(), true_0 != 0) as *mut wordnode_T;
    } else {
        n = (*spin).si_first_free;
        (*spin).si_first_free = (*n).wn_child;
        memset(
            n as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<wordnode_T>(),
        );
        (*spin).si_free_count -= 1;
    }
    return n;
}
unsafe extern "C" fn deref_wordnode(
    mut spin: *mut spellinfo_T,
    mut node: *mut wordnode_T,
) -> ::core::ffi::c_int {
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*node).wn_refs -= 1;
    if (*node).wn_refs == 0 as ::core::ffi::c_int {
        let mut np: *mut wordnode_T = node;
        while !np.is_null() {
            if !(*np).wn_child.is_null() {
                cnt += deref_wordnode(spin, (*np).wn_child);
            }
            free_wordnode(spin, np);
            cnt += 1;
            np = (*np).wn_sibling;
        }
        cnt += 1;
    }
    return cnt;
}
unsafe extern "C" fn free_wordnode(mut spin: *mut spellinfo_T, mut n: *mut wordnode_T) {
    (*n).wn_child = (*spin).si_first_free;
    (*spin).si_first_free = n;
    (*spin).si_free_count += 1;
}
unsafe extern "C" fn wordtree_compress(
    mut spin: *mut spellinfo_T,
    mut root: *mut wordnode_T,
    mut name: *const ::core::ffi::c_char,
) {
    let mut ht: hashtab_T = hashtab_T {
        ht_mask: 0,
        ht_used: 0,
        ht_filled: 0,
        ht_changed: 0,
        ht_locked: 0,
        ht_array: ::core::ptr::null_mut::<hashitem_T>(),
        ht_smallarray: [hashitem_T {
            hi_hash: 0,
            hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        }; 16],
    };
    let mut tot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut perc: ::core::ffi::c_long = 0;
    if (*root).wn_sibling.is_null() {
        return;
    }
    hash_init(&raw mut ht);
    let n: ::core::ffi::c_int = node_compress(spin, (*root).wn_sibling, &raw mut ht, &raw mut tot);
    if (*spin).si_verbose != 0 || p_verbose.get() > 2 as OptInt {
        if tot > 1000000 as ::core::ffi::c_int {
            perc = ((tot - n) / (tot / 100 as ::core::ffi::c_int)) as ::core::ffi::c_long;
        } else if tot == 0 as ::core::ffi::c_int {
            perc = 0 as ::core::ffi::c_long;
        } else {
            perc = ((tot - n) * 100 as ::core::ffi::c_int / tot) as ::core::ffi::c_long;
        }
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            gettext(
                b"Compressed %s: %d of %d nodes; %d (%ld%%) remaining\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
            n,
            tot,
            tot - n,
            perc,
        );
        spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    }
    hash_clear(&raw mut ht);
}
unsafe extern "C" fn node_compress(
    mut spin: *mut spellinfo_T,
    mut node: *mut wordnode_T,
    mut ht: *mut hashtab_T,
    mut tot: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut tp: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    let mut child: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_uint = 0;
    let mut compressed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut np: *mut wordnode_T = node;
    while !np.is_null() && !got_int.get() {
        len += 1;
        child = (*np).wn_child;
        if !child.is_null() {
            compressed += node_compress(spin, child, ht, tot);
            let mut hash: hash_T = hash_hash(
                &raw mut (*child).wn_u1.hashkey as *mut uint8_t as *mut ::core::ffi::c_char,
            );
            let mut hi: *mut hashitem_T = hash_lookup(
                ht,
                &raw mut (*child).wn_u1.hashkey as *mut uint8_t as *const ::core::ffi::c_char,
                strlen(&raw mut (*child).wn_u1.hashkey as *mut uint8_t as *mut ::core::ffi::c_char),
                hash,
            );
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                tp = (*hi).hi_key as *mut wordnode_T;
                while !tp.is_null() {
                    if node_equal(child, tp) {
                        (*tp).wn_refs += 1;
                        compressed += deref_wordnode(spin, child);
                        (*np).wn_child = tp;
                        break;
                    } else {
                        tp = (*tp).wn_u2.next;
                    }
                }
                if tp.is_null() {
                    tp = (*hi).hi_key as *mut wordnode_T;
                    (*child).wn_u2.next = (*tp).wn_u2.next;
                    (*tp).wn_u2.next = child;
                }
            } else {
                hash_add_item(
                    ht,
                    hi,
                    &raw mut (*child).wn_u1.hashkey as *mut uint8_t as *mut ::core::ffi::c_char,
                    hash,
                );
            }
        }
        np = (*np).wn_sibling;
    }
    *tot += len + 1 as ::core::ffi::c_int;
    (*node).wn_u1.hashkey[0 as ::core::ffi::c_int as usize] = len as uint8_t;
    let mut nr: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut np_0: *mut wordnode_T = node;
    while !np_0.is_null() {
        if (*np_0).wn_byte as ::core::ffi::c_int == NUL {
            n = ((*np_0).wn_flags as ::core::ffi::c_int
                + (((*np_0).wn_region as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                + (((*np_0).wn_affixID as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
                as ::core::ffi::c_uint;
        } else {
            n = ((*np_0).wn_byte as uintptr_t).wrapping_add(
                ((*np_0).wn_child.expose_addr() as uintptr_t) << 8 as ::core::ffi::c_int,
            ) as ::core::ffi::c_uint;
        }
        nr = nr.wrapping_mul(101 as ::core::ffi::c_uint).wrapping_add(n);
        np_0 = (*np_0).wn_sibling;
    }
    n = nr & 0xff as ::core::ffi::c_uint;
    (*node).wn_u1.hashkey[1 as ::core::ffi::c_int as usize] = (if n == 0 as ::core::ffi::c_uint {
        1 as ::core::ffi::c_int
    } else {
        n as uint8_t as ::core::ffi::c_int
    }) as uint8_t;
    n = nr >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint;
    (*node).wn_u1.hashkey[2 as ::core::ffi::c_int as usize] = (if n == 0 as ::core::ffi::c_uint {
        1 as ::core::ffi::c_int
    } else {
        n as uint8_t as ::core::ffi::c_int
    }) as uint8_t;
    n = nr >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint;
    (*node).wn_u1.hashkey[3 as ::core::ffi::c_int as usize] = (if n == 0 as ::core::ffi::c_uint {
        1 as ::core::ffi::c_int
    } else {
        n as uint8_t as ::core::ffi::c_int
    }) as uint8_t;
    n = nr >> 24 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint;
    (*node).wn_u1.hashkey[4 as ::core::ffi::c_int as usize] = (if n == 0 as ::core::ffi::c_uint {
        1 as ::core::ffi::c_int
    } else {
        n as uint8_t as ::core::ffi::c_int
    }) as uint8_t;
    (*node).wn_u1.hashkey[5 as ::core::ffi::c_int as usize] = NUL as uint8_t;
    veryfast_breakcheck();
    return compressed;
}
unsafe extern "C" fn node_equal(mut n1: *mut wordnode_T, mut n2: *mut wordnode_T) -> bool {
    let mut p1: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    let mut p2: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    p1 = n1;
    p2 = n2;
    while !p1.is_null() && !p2.is_null() {
        if (*p1).wn_byte as ::core::ffi::c_int != (*p2).wn_byte as ::core::ffi::c_int
            || (if (*p1).wn_byte as ::core::ffi::c_int == NUL {
                ((*p1).wn_flags as ::core::ffi::c_int != (*p2).wn_flags as ::core::ffi::c_int
                    || (*p1).wn_region as ::core::ffi::c_int
                        != (*p2).wn_region as ::core::ffi::c_int
                    || (*p1).wn_affixID as ::core::ffi::c_int
                        != (*p2).wn_affixID as ::core::ffi::c_int)
                    as ::core::ffi::c_int
            } else {
                ((*p1).wn_child != (*p2).wn_child) as ::core::ffi::c_int
            }) != 0
        {
            break;
        }
        p1 = (*p1).wn_sibling;
        p2 = (*p2).wn_sibling;
    }
    return p1.is_null() && p2.is_null();
}
unsafe extern "C" fn rep_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut fromto_T = s1 as *mut fromto_T;
    let mut p2: *mut fromto_T = s2 as *mut fromto_T;
    return strcmp((*p1).ft_from, (*p2).ft_from);
}
unsafe extern "C" fn write_vim_spell(
    mut spin: *mut spellinfo_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    let mut regionmask: ::core::ffi::c_int = 0;
    let mut fd: *mut FILE = os_fopen(fname, b"w\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return FAIL;
    }
    let mut fwv: size_t = fwrite(
        VIMSPELLMAGIC.as_ptr() as *const ::core::ffi::c_void,
        VIMSPELLMAGICL,
        1 as size_t,
        fd,
    ) as size_t;
    if fwv == 1 as size_t {
        putc(VIMSPELLVERSION, fd);
        if !(*spin).si_info.is_null() {
            putc(SN_INFO as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut i: size_t = strlen((*spin).si_info);
            put_bytes(fd, i as uintmax_t, 4 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_info as *const ::core::ffi::c_void,
                    i,
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        if (*spin).si_region_count > 1 as ::core::ffi::c_int {
            putc(SN_REGION as ::core::ffi::c_int, fd);
            putc(SNF_REQUIRED, fd);
            let mut l: size_t = ((*spin).si_region_count as size_t).wrapping_mul(2 as size_t);
            put_bytes(fd, l as uintmax_t, 4 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    &raw mut (*spin).si_region_name as *mut ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    l,
                    1 as size_t,
                    fd,
                )) as size_t;
            regionmask =
                ((1 as ::core::ffi::c_int) << (*spin).si_region_count) - 1 as ::core::ffi::c_int;
        } else {
            regionmask = 0 as ::core::ffi::c_int;
        }
        if (*spin).si_ascii == 0 && (*spin).si_add == 0 {
            let mut folchars: [::core::ffi::c_char; 1024] = [0; 1024];
            putc(SN_CHARFLAGS as ::core::ffi::c_int, fd);
            putc(SNF_REQUIRED, fd);
            let mut l_0: size_t = 0 as size_t;
            let mut i_0: size_t = 128 as size_t;
            while i_0 < 256 as size_t {
                l_0 = l_0.wrapping_add(utf_char2bytes(
                    (*spelltab.ptr()).st_fold[i_0 as usize] as ::core::ffi::c_int,
                    (&raw mut folchars as *mut ::core::ffi::c_char).offset(l_0 as isize),
                ) as size_t);
                i_0 = i_0.wrapping_add(1);
            }
            put_bytes(
                fd,
                ((1 as ::core::ffi::c_int + 128 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)
                    as uintmax_t)
                    .wrapping_add(l_0 as uintmax_t),
                4 as size_t,
            );
            fputc(128 as ::core::ffi::c_int, fd);
            let mut i_1: size_t = 128 as size_t;
            while i_1 < 256 as size_t {
                let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if (*spelltab.ptr()).st_isw[i_1 as usize] {
                    flags |= CF_WORD as ::core::ffi::c_int;
                }
                if (*spelltab.ptr()).st_isu[i_1 as usize] {
                    flags |= CF_UPPER as ::core::ffi::c_int;
                }
                fputc(flags, fd);
                i_1 = i_1.wrapping_add(1);
            }
            put_bytes(fd, l_0 as uintmax_t, 2 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    &raw mut folchars as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    l_0,
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        if !(*spin).si_midword.is_null() {
            putc(SN_MIDWORD as ::core::ffi::c_int, fd);
            putc(SNF_REQUIRED, fd);
            let mut i_2: size_t = strlen((*spin).si_midword);
            put_bytes(fd, i_2 as uintmax_t, 4 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_midword as *const ::core::ffi::c_void,
                    i_2,
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        if !((*spin).si_prefcond.ga_len <= 0 as ::core::ffi::c_int) {
            putc(SN_PREFCOND as ::core::ffi::c_int, fd);
            putc(SNF_REQUIRED, fd);
            let mut l_1: size_t = write_spell_prefcond(
                ::core::ptr::null_mut::<FILE>(),
                &raw mut (*spin).si_prefcond,
                &raw mut fwv,
            ) as size_t;
            put_bytes(fd, l_1 as uintmax_t, 4 as size_t);
            write_spell_prefcond(fd, &raw mut (*spin).si_prefcond, &raw mut fwv);
        }
        let mut round: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
        while round <= 3 as ::core::ffi::c_uint {
            let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
            's_229: {
                if round == 1 as ::core::ffi::c_uint {
                    gap = &raw mut (*spin).si_rep;
                } else if round == 2 as ::core::ffi::c_uint {
                    if !(*spin).si_sofofr.is_null() && !(*spin).si_sofoto.is_null() {
                        break 's_229;
                    } else {
                        gap = &raw mut (*spin).si_sal;
                    }
                } else {
                    gap = &raw mut (*spin).si_repsal;
                }
                if (*gap).ga_len > 0 as ::core::ffi::c_int {
                    if round != 2 as ::core::ffi::c_uint {
                        qsort(
                            (*gap).ga_data,
                            (*gap).ga_len as size_t,
                            ::core::mem::size_of::<fromto_T>(),
                            Some(
                                rep_compare
                                    as unsafe extern "C" fn(
                                        *const ::core::ffi::c_void,
                                        *const ::core::ffi::c_void,
                                    )
                                        -> ::core::ffi::c_int,
                            ),
                        );
                    }
                    let mut sect_id: ::core::ffi::c_int = if round == 1 as ::core::ffi::c_uint {
                        SN_REP as ::core::ffi::c_int
                    } else if round == 2 as ::core::ffi::c_uint {
                        SN_SAL as ::core::ffi::c_int
                    } else {
                        SN_REPSAL as ::core::ffi::c_int
                    };
                    putc(sect_id, fd);
                    putc(0 as ::core::ffi::c_int, fd);
                    let mut l_2: size_t = 2 as size_t;
                    '_c2rust_label: {
                        if (*gap).ga_len >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"gap->ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                4420 as ::core::ffi::c_uint,
                                b"int write_vim_spell(spellinfo_T *, char *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let mut i_3: size_t = 0 as size_t;
                    while i_3 < (*gap).ga_len as size_t {
                        let mut ftp: *mut fromto_T =
                            ((*gap).ga_data as *mut fromto_T).offset(i_3 as isize);
                        l_2 = l_2.wrapping_add((1 as size_t).wrapping_add(strlen((*ftp).ft_from)));
                        l_2 = l_2.wrapping_add((1 as size_t).wrapping_add(strlen((*ftp).ft_to)));
                        i_3 = i_3.wrapping_add(1);
                    }
                    if round == 2 as ::core::ffi::c_uint {
                        l_2 = l_2.wrapping_add(1);
                    }
                    put_bytes(fd, l_2 as uintmax_t, 4 as size_t);
                    if round == 2 as ::core::ffi::c_uint {
                        let mut i_4: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if (*spin).si_followup != 0 {
                            i_4 |= SAL_F0LLOWUP as ::core::ffi::c_int;
                        }
                        if (*spin).si_collapse != 0 {
                            i_4 |= SAL_COLLAPSE as ::core::ffi::c_int;
                        }
                        if (*spin).si_rem_accents != 0 {
                            i_4 |= SAL_REM_ACCENTS as ::core::ffi::c_int;
                        }
                        putc(i_4, fd);
                    }
                    put_bytes(fd, (*gap).ga_len as uintmax_t, 2 as size_t);
                    let mut i_5: size_t = 0 as size_t;
                    while i_5 < (*gap).ga_len as size_t {
                        let mut ftp_0: *mut fromto_T =
                            ((*gap).ga_data as *mut fromto_T).offset(i_5 as isize);
                        let mut rr: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
                        while rr <= 2 as ::core::ffi::c_uint {
                            let mut p: *mut ::core::ffi::c_char = if rr == 1 as ::core::ffi::c_uint
                            {
                                (*ftp_0).ft_from
                            } else {
                                (*ftp_0).ft_to
                            };
                            l_2 = strlen(p);
                            '_c2rust_label_0: {
                                if l_2 < 2147483647 as ::core::ffi::c_int as size_t {
                                } else {
                                    __assert_fail(
                                        b"l < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/spellfile.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        4453 as ::core::ffi::c_uint,
                                        b"int write_vim_spell(spellinfo_T *, char *)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            putc(l_2 as ::core::ffi::c_int, fd);
                            if l_2 > 0 as size_t {
                                fwv = (fwv as ::core::ffi::c_ulong
                                    & fwrite(p as *const ::core::ffi::c_void, l_2, 1 as size_t, fd))
                                    as size_t;
                            }
                            rr = rr.wrapping_add(1);
                        }
                        i_5 = i_5.wrapping_add(1);
                    }
                }
            }
            round = round.wrapping_add(1);
        }
        if !(*spin).si_sofofr.is_null() && !(*spin).si_sofoto.is_null() {
            putc(SN_SOFO as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut l_3: size_t = strlen((*spin).si_sofofr);
            put_bytes(
                fd,
                (l_3 as uintmax_t)
                    .wrapping_add(strlen((*spin).si_sofoto) as uintmax_t)
                    .wrapping_add(4 as uintmax_t),
                4 as size_t,
            );
            put_bytes(fd, l_3 as uintmax_t, 2 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_sofofr as *const ::core::ffi::c_void,
                    l_3,
                    1 as size_t,
                    fd,
                )) as size_t;
            l_3 = strlen((*spin).si_sofoto);
            put_bytes(fd, l_3 as uintmax_t, 2 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_sofoto as *const ::core::ffi::c_void,
                    l_3,
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        if (*spin).si_commonwords.ht_used > 0 as size_t {
            putc(SN_WORDS as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut round_0: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
            while round_0 <= 2 as ::core::ffi::c_uint {
                let mut todo: size_t = 0;
                let mut len: size_t = 0 as size_t;
                let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
                todo = (*spin).si_commonwords.ht_used;
                hi = (*spin).si_commonwords.ht_array;
                while todo > 0 as size_t {
                    if !((*hi).hi_key.is_null()
                        || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                    {
                        let mut l_4: size_t = strlen((*hi).hi_key).wrapping_add(1 as size_t);
                        len = len.wrapping_add(l_4);
                        if round_0 == 2 as ::core::ffi::c_uint {
                            fwv = (fwv as ::core::ffi::c_ulong
                                & fwrite(
                                    (*hi).hi_key as *const ::core::ffi::c_void,
                                    l_4,
                                    1 as size_t,
                                    fd,
                                )) as size_t;
                        }
                        todo = todo.wrapping_sub(1);
                    }
                    hi = hi.offset(1);
                }
                if round_0 == 1 as ::core::ffi::c_uint {
                    put_bytes(fd, len as uintmax_t, 4 as size_t);
                }
                round_0 = round_0.wrapping_add(1);
            }
        }
        if !((*spin).si_map.ga_len <= 0 as ::core::ffi::c_int) {
            putc(SN_MAP as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut l_5: size_t = (*spin).si_map.ga_len as size_t;
            put_bytes(fd, l_5 as uintmax_t, 4 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite((*spin).si_map.ga_data, l_5, 1 as size_t, fd)) as size_t;
        }
        if (*spin).si_nosugfile == 0
            && (!((*spin).si_sal.ga_len <= 0 as ::core::ffi::c_int)
                || !(*spin).si_sofofr.is_null() && !(*spin).si_sofoto.is_null())
        {
            putc(SN_SUGFILE as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            put_bytes(fd, 8 as uintmax_t, 4 as size_t);
            (*spin).si_sugtime = time(::core::ptr::null_mut::<time_t>());
            put_time(fd, (*spin).si_sugtime);
        }
        if (*spin).si_nosplitsugs != 0 {
            putc(SN_NOSPLITSUGS as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            put_bytes(fd, 0 as uintmax_t, 4 as size_t);
        }
        if (*spin).si_nocompoundsugs != 0 {
            putc(SN_NOCOMPOUNDSUGS as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            put_bytes(fd, 0 as uintmax_t, 4 as size_t);
        }
        if !(*spin).si_compflags.is_null() {
            putc(SN_COMPOUND as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut l_6: size_t = strlen((*spin).si_compflags);
            '_c2rust_label_1: {
                if (*spin).si_comppat.ga_len >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"spin->si_comppat.ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4562 as ::core::ffi::c_uint,
                        b"int write_vim_spell(spellinfo_T *, char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut i_6: size_t = 0 as size_t;
            while i_6 < (*spin).si_comppat.ga_len as size_t {
                l_6 = l_6.wrapping_add(
                    strlen(
                        *((*spin).si_comppat.ga_data as *mut *mut ::core::ffi::c_char)
                            .offset(i_6 as isize),
                    )
                    .wrapping_add(1 as size_t),
                );
                i_6 = i_6.wrapping_add(1);
            }
            put_bytes(
                fd,
                (l_6 as uintmax_t).wrapping_add(7 as uintmax_t),
                4 as size_t,
            );
            putc((*spin).si_compmax, fd);
            putc((*spin).si_compminlen, fd);
            putc((*spin).si_compsylmax, fd);
            putc(0 as ::core::ffi::c_int, fd);
            putc((*spin).si_compoptions, fd);
            put_bytes(fd, (*spin).si_comppat.ga_len as uintmax_t, 2 as size_t);
            let mut i_7: size_t = 0 as size_t;
            while i_7 < (*spin).si_comppat.ga_len as size_t {
                let mut p_0: *mut ::core::ffi::c_char = *((*spin).si_comppat.ga_data
                    as *mut *mut ::core::ffi::c_char)
                    .offset(i_7 as isize);
                '_c2rust_label_2: {
                    if strlen(p_0) < 2147483647 as ::core::ffi::c_int as size_t {
                    } else {
                        __assert_fail(
                            b"strlen(p) < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            4576 as ::core::ffi::c_uint,
                            b"int write_vim_spell(spellinfo_T *, char *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                putc(strlen(p_0) as ::core::ffi::c_int, fd);
                fwv = (fwv as ::core::ffi::c_ulong
                    & fwrite(
                        p_0 as *const ::core::ffi::c_void,
                        strlen(p_0),
                        1 as size_t,
                        fd,
                    )) as size_t;
                i_7 = i_7.wrapping_add(1);
            }
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_compflags as *const ::core::ffi::c_void,
                    strlen((*spin).si_compflags),
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        if (*spin).si_nobreak != 0 {
            putc(SN_NOBREAK as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            put_bytes(fd, 0 as uintmax_t, 4 as size_t);
        }
        if !(*spin).si_syllable.is_null() {
            putc(SN_SYLLABLE as ::core::ffi::c_int, fd);
            putc(0 as ::core::ffi::c_int, fd);
            let mut l_7: size_t = strlen((*spin).si_syllable);
            put_bytes(fd, l_7 as uintmax_t, 4 as size_t);
            fwv = (fwv as ::core::ffi::c_ulong
                & fwrite(
                    (*spin).si_syllable as *const ::core::ffi::c_void,
                    l_7,
                    1 as size_t,
                    fd,
                )) as size_t;
        }
        putc(SN_END as ::core::ffi::c_int, fd);
        (*spin).si_memtot = 0 as ::core::ffi::c_int;
        let mut round_1: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
        while round_1 <= 3 as ::core::ffi::c_uint {
            let mut tree: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
            if round_1 == 1 as ::core::ffi::c_uint {
                tree = (*(*spin).si_foldroot).wn_sibling;
            } else if round_1 == 2 as ::core::ffi::c_uint {
                tree = (*(*spin).si_keeproot).wn_sibling;
            } else {
                tree = (*(*spin).si_prefroot).wn_sibling;
            }
            clear_node(tree);
            let mut nodecount: size_t = put_node(
                ::core::ptr::null_mut::<FILE>(),
                tree,
                0 as ::core::ffi::c_int,
                regionmask,
                round_1 == 3 as ::core::ffi::c_uint,
            ) as size_t;
            put_bytes(fd, nodecount as uintmax_t, 4 as size_t);
            '_c2rust_label_3: {
                if nodecount.wrapping_add(
                    nodecount.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
                ) < 2147483647 as ::core::ffi::c_int as size_t
                {
                } else {
                    __assert_fail(
                        b"nodecount + nodecount * sizeof(int) < INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4630 as ::core::ffi::c_uint,
                        b"int write_vim_spell(spellinfo_T *, char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*spin).si_memtot += nodecount
                .wrapping_add(nodecount.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()))
                as ::core::ffi::c_int;
            put_node(
                fd,
                tree,
                0 as ::core::ffi::c_int,
                regionmask,
                round_1 == 3 as ::core::ffi::c_uint,
            );
            round_1 = round_1.wrapping_add(1);
        }
        if putc(0 as ::core::ffi::c_int, fd) == EOF {
            retval = FAIL;
        }
    }
    if fclose(fd) == EOF {
        retval = FAIL;
    }
    if fwv != 1 as size_t {
        retval = FAIL;
    }
    if retval == FAIL {
        emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
    }
    return retval;
}
unsafe extern "C" fn clear_node(mut node: *mut wordnode_T) {
    if !node.is_null() {
        let mut np: *mut wordnode_T = node;
        while !np.is_null() {
            (*np).wn_u1.index = 0 as ::core::ffi::c_int;
            (*np).wn_u2.wnode = ::core::ptr::null_mut::<wordnode_T>();
            if (*np).wn_byte as ::core::ffi::c_int != NUL {
                clear_node((*np).wn_child);
            }
            np = (*np).wn_sibling;
        }
    }
}
unsafe extern "C" fn put_node(
    mut fd: *mut FILE,
    mut node: *mut wordnode_T,
    mut idx: ::core::ffi::c_int,
    mut regionmask: ::core::ffi::c_int,
    mut prefixtree: bool,
) -> ::core::ffi::c_int {
    if node.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*node).wn_u1.index = idx;
    let mut siblingcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut np: *mut wordnode_T = node;
    while !np.is_null() {
        siblingcount += 1;
        np = (*np).wn_sibling;
    }
    if !fd.is_null() {
        putc(siblingcount, fd);
    }
    let mut np_0: *mut wordnode_T = node;
    while !np_0.is_null() {
        if (*np_0).wn_byte as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            if !fd.is_null() {
                if prefixtree {
                    if (*np_0).wn_flags as ::core::ffi::c_int
                        == PFX_FLAGS as uint16_t as ::core::ffi::c_int
                    {
                        putc(BY_NOFLAGS as ::core::ffi::c_int, fd);
                    } else {
                        putc(BY_FLAGS as ::core::ffi::c_int, fd);
                        putc((*np_0).wn_flags as ::core::ffi::c_int, fd);
                    }
                    putc((*np_0).wn_affixID as ::core::ffi::c_int, fd);
                    put_bytes(fd, (*np_0).wn_region as uintmax_t, 2 as size_t);
                } else {
                    let mut flags: ::core::ffi::c_int = (*np_0).wn_flags as ::core::ffi::c_int;
                    if regionmask != 0 as ::core::ffi::c_int
                        && (*np_0).wn_region as ::core::ffi::c_int != regionmask
                    {
                        flags |= WF_REGION as ::core::ffi::c_int;
                    }
                    if (*np_0).wn_affixID as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        flags |= WF_AFX as ::core::ffi::c_int;
                    }
                    if flags == 0 as ::core::ffi::c_int {
                        putc(BY_NOFLAGS as ::core::ffi::c_int, fd);
                    } else {
                        if (*np_0).wn_flags as ::core::ffi::c_int >= 0x100 as ::core::ffi::c_int {
                            putc(BY_FLAGS2 as ::core::ffi::c_int, fd);
                            putc(flags, fd);
                            putc(
                                (flags as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_int,
                                fd,
                            );
                        } else {
                            putc(BY_FLAGS as ::core::ffi::c_int, fd);
                            putc(flags, fd);
                        }
                        if flags & WF_REGION as ::core::ffi::c_int != 0 {
                            putc((*np_0).wn_region as ::core::ffi::c_int, fd);
                        }
                        if flags & WF_AFX as ::core::ffi::c_int != 0 {
                            putc((*np_0).wn_affixID as ::core::ffi::c_int, fd);
                        }
                    }
                }
            }
        } else {
            if (*(*np_0).wn_child).wn_u1.index != 0 as ::core::ffi::c_int
                && (*(*np_0).wn_child).wn_u2.wnode != node
            {
                if !fd.is_null() {
                    putc(BY_INDEX as ::core::ffi::c_int, fd);
                    put_bytes(
                        fd,
                        (*(*np_0).wn_child).wn_u1.index as uintmax_t,
                        3 as size_t,
                    );
                }
            } else if (*(*np_0).wn_child).wn_u2.wnode.is_null() {
                (*(*np_0).wn_child).wn_u2.wnode = node;
            }
            if !fd.is_null() {
                if putc((*np_0).wn_byte as ::core::ffi::c_int, fd) == EOF {
                    emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        np_0 = (*np_0).wn_sibling;
    }
    let mut newindex: ::core::ffi::c_int = idx + siblingcount + 1 as ::core::ffi::c_int;
    let mut np_1: *mut wordnode_T = node;
    while !np_1.is_null() {
        if (*np_1).wn_byte as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && (*(*np_1).wn_child).wn_u2.wnode == node
        {
            newindex = put_node(fd, (*np_1).wn_child, newindex, regionmask, prefixtree);
        }
        np_1 = (*np_1).wn_sibling;
    }
    return newindex;
}
#[no_mangle]
pub unsafe extern "C" fn ex_mkspell(mut eap: *mut exarg_T) {
    let mut fcount: ::core::ffi::c_int = 0;
    let mut fnames: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut ascii: bool = false_0 != 0;
    if strncmp(
        arg,
        b"-ascii\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        ascii = true_0 != 0;
        arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
    }
    if get_arglist_exp(arg, &raw mut fcount, &raw mut fnames, false_0 != 0) != OK {
        return;
    }
    mkspell(fcount, fnames, ascii, (*eap).forceit != 0, false_0 != 0);
    FreeWild(fcount, fnames);
}
unsafe extern "C" fn spell_make_sugfile(
    mut spin: *mut spellinfo_T,
    mut wfname: *mut ::core::ffi::c_char,
) {
    let mut len: ::core::ffi::c_int = 0;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slang: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
    let mut free_slang: bool = false_0 != 0;
    slang = first_lang.get();
    while !slang.is_null() {
        if path_full_compare(wfname, (*slang).sl_fname, false_0 != 0, true_0 != 0)
            as ::core::ffi::c_uint
            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        slang = (*slang).sl_next;
    }
    if slang.is_null() {
        spell_message(
            spin,
            gettext(b"Reading back spell file...\0".as_ptr() as *const ::core::ffi::c_char),
        );
        slang = spell_load_file(
            wfname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<slang_T>(),
            false_0 != 0,
        );
        if slang.is_null() {
            return;
        }
        free_slang = true_0 != 0;
    }
    (*spin).si_blocks = ::core::ptr::null_mut::<sblock_T>();
    (*spin).si_blocks_cnt = 0 as ::core::ffi::c_int;
    (*spin).si_compress_cnt = 0 as ::core::ffi::c_int;
    (*spin).si_free_count = 0 as ::core::ffi::c_int;
    (*spin).si_first_free = ::core::ptr::null_mut::<wordnode_T>();
    (*spin).si_foldwcount = 0 as ::core::ffi::c_int;
    spell_message(
        spin,
        gettext(b"Performing soundfolding...\0".as_ptr() as *const ::core::ffi::c_char),
    );
    if sug_filltree(spin, slang) != FAIL {
        if sug_maketable(spin) != FAIL {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Number of words after soundfolding: %ld\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*(*spin).si_spellbuf).b_ml.ml_line_count as int64_t,
            );
            spell_message(spin, gettext(msg_compressing.get()));
            wordtree_compress(
                spin,
                (*spin).si_foldroot,
                b"case-folded\0".as_ptr() as *const ::core::ffi::c_char,
            );
            fname = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            xstrlcpy(fname, wfname, MAXPATHL as size_t);
            len = strlen(fname) as ::core::ffi::c_int;
            *fname.offset((len - 2 as ::core::ffi::c_int) as isize) = 'u' as ::core::ffi::c_char;
            *fname.offset((len - 1 as ::core::ffi::c_int) as isize) = 'g' as ::core::ffi::c_char;
            sug_write(spin, fname);
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    if free_slang {
        slang_free(slang);
    }
    free_blocks((*spin).si_blocks);
    close_spellbuf((*spin).si_spellbuf);
}
unsafe extern "C" fn sug_filltree(
    mut spin: *mut spellinfo_T,
    mut slang: *mut slang_T,
) -> ::core::ffi::c_int {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut tword: [::core::ffi::c_char; 254] = [0; 254];
    let mut tsalword: [::core::ffi::c_char; 254] = [0; 254];
    let mut words_done: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut wordcount: [::core::ffi::c_int; 254] = [0; 254];
    (*spin).si_foldroot = wordtree_alloc(spin);
    (*spin).si_sugtree = true_0;
    let mut byts: *mut uint8_t = (*slang).sl_fbyts;
    let mut idxs: *mut idx_T = (*slang).sl_fidxs;
    if byts.is_null() || idxs.is_null() {
        return FAIL;
    }
    arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
    curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
    wordcount[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while depth >= 0 as ::core::ffi::c_int && !got_int.get() {
        if curi[depth as usize]
            > *byts.offset(arridx[depth as usize] as isize) as ::core::ffi::c_int
        {
            *idxs.offset(arridx[depth as usize] as isize) = wordcount[depth as usize] as idx_T;
            if depth > 0 as ::core::ffi::c_int {
                wordcount[(depth - 1 as ::core::ffi::c_int) as usize] += wordcount[depth as usize];
            }
            depth -= 1;
            line_breakcheck();
        } else {
            let mut n: idx_T = arridx[depth as usize] + curi[depth as usize] as idx_T;
            curi[depth as usize] += 1;
            let mut c: ::core::ffi::c_int = *byts.offset(n as isize) as ::core::ffi::c_int;
            if c == 0 as ::core::ffi::c_int {
                tword[depth as usize] = NUL as ::core::ffi::c_char;
                spell_soundfold(
                    slang,
                    &raw mut tword as *mut ::core::ffi::c_char,
                    true_0 != 0,
                    &raw mut tsalword as *mut ::core::ffi::c_char,
                );
                if tree_add_word(
                    spin,
                    &raw mut tsalword as *mut ::core::ffi::c_char,
                    (*spin).si_foldroot,
                    (words_done >> 16 as ::core::ffi::c_int) as ::core::ffi::c_int,
                    (words_done & 0xffff as ::core::ffi::c_uint) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) == FAIL
                {
                    return FAIL;
                }
                words_done = words_done.wrapping_add(1);
                wordcount[depth as usize] += 1;
                (*spin).si_blocks_cnt = 0 as ::core::ffi::c_int;
                while (n as ::core::ffi::c_int + 1 as ::core::ffi::c_int) < (*slang).sl_fbyts_len
                    && *byts.offset((n as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    n += 1;
                    curi[depth as usize] += 1;
                }
            } else {
                let c2rust_fresh27 = depth;
                depth = depth + 1;
                tword[c2rust_fresh27 as usize] = c as uint8_t as ::core::ffi::c_char;
                arridx[depth as usize] = *idxs.offset(n as isize);
                curi[depth as usize] = 1 as ::core::ffi::c_int;
                wordcount[depth as usize] = 0 as ::core::ffi::c_int;
            }
        }
    }
    smsg(
        0 as ::core::ffi::c_int,
        gettext(b"Total number of words: %d\0".as_ptr() as *const ::core::ffi::c_char),
        words_done,
    );
    return OK;
}
unsafe extern "C" fn sug_maketable(mut spin: *mut spellinfo_T) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut res: ::core::ffi::c_int = OK;
    (*spin).si_spellbuf = open_spellbuf();
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    if sug_filltable(
        spin,
        (*(*spin).si_foldroot).wn_sibling,
        0 as ::core::ffi::c_int,
        &raw mut ga,
    ) == -1 as ::core::ffi::c_int
    {
        res = FAIL;
    }
    ga_clear(&raw mut ga);
    return res;
}
unsafe extern "C" fn sug_filltable(
    mut spin: *mut spellinfo_T,
    mut node: *mut wordnode_T,
    mut startwordnr: ::core::ffi::c_int,
    mut gap: *mut garray_T,
) -> ::core::ffi::c_int {
    let mut wordnr: ::core::ffi::c_int = startwordnr;
    let mut p: *mut wordnode_T = node;
    while !p.is_null() {
        if (*p).wn_byte as ::core::ffi::c_int == NUL {
            (*gap).ga_len = 0 as ::core::ffi::c_int;
            let mut prev_nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut np: *mut wordnode_T = p;
            while !np.is_null() && (*np).wn_byte as ::core::ffi::c_int == NUL {
                ga_grow(gap, 10 as ::core::ffi::c_int);
                let mut nr: ::core::ffi::c_int = (((*np).wn_flags as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int)
                    + ((*np).wn_region as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int);
                nr -= prev_nr;
                prev_nr += nr;
                (*gap).ga_len += offset2bytes(
                    nr,
                    ((*gap).ga_data as *mut ::core::ffi::c_char).offset((*gap).ga_len as isize),
                );
                np = (*np).wn_sibling;
            }
            let c2rust_fresh26 = (*gap).ga_len;
            (*gap).ga_len = (*gap).ga_len + 1;
            *((*gap).ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh26 as isize) =
                NUL as ::core::ffi::c_char;
            if ml_append_buf(
                (*spin).si_spellbuf,
                wordnr as linenr_T,
                (*gap).ga_data as *mut ::core::ffi::c_char,
                (*gap).ga_len as colnr_T,
                true_0 != 0,
            ) == FAIL
            {
                return -1 as ::core::ffi::c_int;
            }
            wordnr += 1;
            while !(*p).wn_sibling.is_null()
                && (*(*p).wn_sibling).wn_byte as ::core::ffi::c_int == NUL
            {
                (*p).wn_sibling = (*(*p).wn_sibling).wn_sibling;
            }
            (*p).wn_flags = 0 as uint16_t;
            (*p).wn_region = 0 as int16_t;
        } else {
            wordnr = sug_filltable(spin, (*p).wn_child, wordnr, gap);
            if wordnr == -1 as ::core::ffi::c_int {
                return -1 as ::core::ffi::c_int;
            }
        }
        p = (*p).wn_sibling;
    }
    return wordnr;
}
unsafe extern "C" fn offset2bytes(
    mut nr: ::core::ffi::c_int,
    mut buf_in: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut uint8_t = buf_in as *mut uint8_t;
    let mut b1: ::core::ffi::c_int = nr % 255 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    let mut rem: ::core::ffi::c_int = nr / 255 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = rem % 255 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    rem = rem / 255 as ::core::ffi::c_int;
    let mut b3: ::core::ffi::c_int = rem % 255 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    let mut b4: ::core::ffi::c_int = rem / 255 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if b4 > 1 as ::core::ffi::c_int || b3 > 0x1f as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) =
            (0xe0 as ::core::ffi::c_int + b4) as uint8_t;
        *buf.offset(1 as ::core::ffi::c_int as isize) = b3 as uint8_t;
        *buf.offset(2 as ::core::ffi::c_int as isize) = b2 as uint8_t;
        *buf.offset(3 as ::core::ffi::c_int as isize) = b1 as uint8_t;
        return 4 as ::core::ffi::c_int;
    }
    if b3 > 1 as ::core::ffi::c_int || b2 > 0x3f as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) =
            (0xc0 as ::core::ffi::c_int + b3) as uint8_t;
        *buf.offset(1 as ::core::ffi::c_int as isize) = b2 as uint8_t;
        *buf.offset(2 as ::core::ffi::c_int as isize) = b1 as uint8_t;
        return 3 as ::core::ffi::c_int;
    }
    if b2 > 1 as ::core::ffi::c_int || b1 > 0x7f as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) =
            (0x80 as ::core::ffi::c_int + b2) as uint8_t;
        *buf.offset(1 as ::core::ffi::c_int as isize) = b1 as uint8_t;
        return 2 as ::core::ffi::c_int;
    }
    *buf.offset(0 as ::core::ffi::c_int as isize) = b1 as uint8_t;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sug_write(mut spin: *mut spellinfo_T, mut fname: *mut ::core::ffi::c_char) {
    let mut tree: *mut wordnode_T = ::core::ptr::null_mut::<wordnode_T>();
    let mut nodecount: size_t = 0;
    let mut wcount: linenr_T = 0;
    let mut fd: *mut FILE = os_fopen(fname, b"w\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"Writing suggestion file %s...\0".as_ptr() as *const ::core::ffi::c_char),
        fname,
    );
    spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    '_theend: {
        if fwrite(
            VIMSUGMAGIC.as_ptr() as *const ::core::ffi::c_void,
            VIMSUGMAGICL as size_t,
            1 as size_t,
            fd,
        ) != 1 as ::core::ffi::c_ulong
        {
            emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
        } else {
            putc(VIMSUGVERSION, fd);
            put_time(fd, (*spin).si_sugtime);
            (*spin).si_memtot = 0 as ::core::ffi::c_int;
            tree = (*(*spin).si_foldroot).wn_sibling;
            clear_node(tree);
            nodecount = put_node(
                ::core::ptr::null_mut::<FILE>(),
                tree,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            ) as size_t;
            put_bytes(fd, nodecount as uintmax_t, 4 as size_t);
            '_c2rust_label: {
                if nodecount.wrapping_add(
                    nodecount.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
                ) < 2147483647 as ::core::ffi::c_int as size_t
                {
                } else {
                    __assert_fail(
                        b"nodecount + nodecount * sizeof(int) < INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5135 as ::core::ffi::c_uint,
                        b"void sug_write(spellinfo_T *, char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*spin).si_memtot += nodecount
                .wrapping_add(nodecount.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()))
                as ::core::ffi::c_int;
            put_node(
                fd,
                tree,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
            wcount = (*(*spin).si_spellbuf).b_ml.ml_line_count;
            '_c2rust_label_0: {
                if wcount >= 0 as linenr_T {
                } else {
                    __assert_fail(
                        b"wcount >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5143 as ::core::ffi::c_uint,
                        b"void sug_write(spellinfo_T *, char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            put_bytes(fd, wcount as uintmax_t, 4 as size_t);
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= wcount {
                let mut line: *mut ::core::ffi::c_char = ml_get_buf((*spin).si_spellbuf, lnum);
                let mut len: ::core::ffi::c_int =
                    ml_get_buf_len((*spin).si_spellbuf, lnum) + 1 as ::core::ffi::c_int;
                if fwrite(
                    line as *const ::core::ffi::c_void,
                    len as size_t,
                    1 as size_t,
                    fd,
                ) == 0 as ::core::ffi::c_ulong
                {
                    emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
                    break '_theend;
                } else {
                    (*spin).si_memtot += len;
                    lnum += 1;
                }
            }
            if putc(0 as ::core::ffi::c_int, fd) == EOF {
                emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
            }
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"Estimated runtime memory use: %d bytes\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*spin).si_memtot,
            );
            spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
        }
    }
    fclose(fd);
}
unsafe extern "C" fn mkspell(
    mut fcount: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut ascii: bool,
    mut over_write: bool,
    mut added_word: bool,
) {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut afile: [*mut afffile_T; 8] = [::core::ptr::null_mut::<afffile_T>(); 8];
    let mut error: bool = false_0 != 0;
    let mut spin: spellinfo_T = spellinfo_T {
        si_foldroot: ::core::ptr::null_mut::<wordnode_T>(),
        si_foldwcount: 0,
        si_keeproot: ::core::ptr::null_mut::<wordnode_T>(),
        si_keepwcount: 0,
        si_prefroot: ::core::ptr::null_mut::<wordnode_T>(),
        si_sugtree: 0,
        si_blocks: ::core::ptr::null_mut::<sblock_T>(),
        si_blocks_cnt: 0,
        si_did_emsg: 0,
        si_compress_cnt: 0,
        si_first_free: ::core::ptr::null_mut::<wordnode_T>(),
        si_free_count: 0,
        si_spellbuf: ::core::ptr::null_mut::<buf_T>(),
        si_ascii: 0,
        si_add: 0,
        si_clear_chartab: 0,
        si_region: 0,
        si_conv: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
        si_memtot: 0,
        si_verbose: 0,
        si_msg_count: 0,
        si_info: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_region_count: 0,
        si_region_name: [0; 17],
        si_rep: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_repsal: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_sal: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_sofofr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_sofoto: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_nosugfile: 0,
        si_nosplitsugs: 0,
        si_nocompoundsugs: 0,
        si_followup: 0,
        si_collapse: 0,
        si_commonwords: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
        si_sugtime: 0,
        si_rem_accents: 0,
        si_map: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_midword: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_compmax: 0,
        si_compminlen: 0,
        si_compsylmax: 0,
        si_compoptions: 0,
        si_comppat: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_compflags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_nobreak: 0,
        si_syllable: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_prefcond: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_newprefID: 0,
        si_newcompID: 0,
    };
    memset(
        &raw mut spin as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<spellinfo_T>(),
    );
    spin.si_verbose = !added_word as ::core::ffi::c_int;
    spin.si_ascii = ascii as ::core::ffi::c_int;
    spin.si_followup = true_0;
    spin.si_rem_accents = true_0;
    ga_init(
        &raw mut spin.si_rep,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_repsal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_sal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_map,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_comppat,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_prefcond,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        50 as ::core::ffi::c_int,
    );
    hash_init(&raw mut spin.si_commonwords);
    spin.si_newcompID = 127 as ::core::ffi::c_int;
    let mut innames: *mut *mut ::core::ffi::c_char = fnames.offset(
        (if fcount == 1 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as isize,
    );
    let mut incount: ::core::ffi::c_int = fcount - 1 as ::core::ffi::c_int;
    let mut wfname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if fcount >= 1 as ::core::ffi::c_int {
        let mut len: ::core::ffi::c_int =
            strlen(*fnames.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if fcount == 1 as ::core::ffi::c_int
            && len > 4 as ::core::ffi::c_int
            && strcmp(
                (*fnames.offset(0 as ::core::ffi::c_int as isize))
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".add\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            incount = 1 as ::core::ffi::c_int;
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                b"%s.spl\0".as_ptr() as *const ::core::ffi::c_char,
                *fnames.offset(0 as ::core::ffi::c_int as isize),
            );
        } else if fcount == 1 as ::core::ffi::c_int {
            incount = 1 as ::core::ffi::c_int;
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                SPL_FNAME_TMPL.as_ptr(),
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                if spin.si_ascii != 0 {
                    b"ascii\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    spell_enc() as *const ::core::ffi::c_char
                },
            );
        } else if len > 4 as ::core::ffi::c_int
            && strcmp(
                (*fnames.offset(0 as ::core::ffi::c_int as isize))
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".spl\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xstrlcpy(
                wfname,
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                MAXPATHL as size_t,
            );
        } else {
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                SPL_FNAME_TMPL.as_ptr(),
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                if spin.si_ascii != 0 {
                    b"ascii\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    spell_enc() as *const ::core::ffi::c_char
                },
            );
        }
        if !strstr(path_tail(wfname), SPL_FNAME_ASCII.as_ptr()).is_null() {
            spin.si_ascii = true_0;
        }
        if !strstr(path_tail(wfname), SPL_FNAME_ADD.as_ptr()).is_null() {
            spin.si_add = true_0;
        }
    }
    '_theend: {
        if incount <= 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else if !vim_strchr(path_tail(wfname), '_' as ::core::ffi::c_int).is_null() {
            emsg(gettext(
                b"E751: Output file name must not have region name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        } else if incount > MAXREGIONS as ::core::ffi::c_int {
            semsg(
                gettext(b"E754: Only up to %d regions supported\0".as_ptr()
                    as *const ::core::ffi::c_char),
                MAXREGIONS as ::core::ffi::c_int,
            );
        } else if !over_write && os_path_exists(wfname) as ::core::ffi::c_int != 0 {
            emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
        } else if os_isdir(wfname) {
            semsg(
                gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                wfname,
            );
        } else {
            fname = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < incount {
                afile[i as usize] = ::core::ptr::null_mut::<afffile_T>();
                if incount > 1 as ::core::ffi::c_int {
                    let mut len_0: ::core::ffi::c_int =
                        strlen(*innames.offset(i as isize)) as ::core::ffi::c_int;
                    if strlen(path_tail(*innames.offset(i as isize))) < 5 as size_t
                        || *(*innames.offset(i as isize))
                            .offset((len_0 - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            != '_' as ::core::ffi::c_int
                    {
                        semsg(
                            gettext(b"E755: Invalid region in %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            *innames.offset(i as isize),
                        );
                        break '_theend;
                    } else {
                        spin.si_region_name[(i * 2 as ::core::ffi::c_int) as usize] =
                            (if (*(*innames.offset(i as isize))
                                .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                < 'A' as ::core::ffi::c_int
                                || *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    > 'Z' as ::core::ffi::c_int
                            {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            }) as uint8_t as ::core::ffi::c_char;
                        spin.si_region_name
                            [(i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
                            (if (*(*innames.offset(i as isize))
                                .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                < 'A' as ::core::ffi::c_int
                                || *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    > 'Z' as ::core::ffi::c_int
                            {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            }) as uint8_t as ::core::ffi::c_char;
                    }
                }
                i += 1;
            }
            spin.si_region_count = incount;
            spin.si_foldroot = wordtree_alloc(&raw mut spin);
            spin.si_keeproot = wordtree_alloc(&raw mut spin);
            spin.si_prefroot = wordtree_alloc(&raw mut spin);
            if spin.si_add == 0 {
                spin.si_clear_chartab = true_0;
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < incount && !error {
                spin.si_conv.vc_type = CONV_NONE as ::core::ffi::c_int;
                spin.si_region = (1 as ::core::ffi::c_int) << i_0;
                vim_snprintf(
                    fname,
                    MAXPATHL as size_t,
                    b"%s.aff\0".as_ptr() as *const ::core::ffi::c_char,
                    *innames.offset(i_0 as isize),
                );
                if os_path_exists(fname) {
                    afile[i_0 as usize] = spell_read_aff(&raw mut spin, fname) as *mut afffile_T;
                    if afile[i_0 as usize].is_null() {
                        error = true_0 != 0;
                    } else {
                        vim_snprintf(
                            fname,
                            MAXPATHL as size_t,
                            b"%s.dic\0".as_ptr() as *const ::core::ffi::c_char,
                            *innames.offset(i_0 as isize),
                        );
                        if spell_read_dic(
                            &raw mut spin,
                            fname,
                            afile[i_0 as usize] as *mut afffile_T,
                        ) == FAIL
                        {
                            error = true_0 != 0;
                        }
                    }
                } else if spell_read_wordfile(&raw mut spin, *innames.offset(i_0 as isize)) == FAIL
                {
                    error = true_0 != 0;
                }
                convert_setup(
                    &raw mut spin.si_conv,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                i_0 += 1;
            }
            if !spin.si_compflags.is_null() && spin.si_nobreak as ::core::ffi::c_int != 0 {
                msg(
                    gettext(
                        b"Warning: both compounding and NOBREAK specified\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    0 as ::core::ffi::c_int,
                );
            }
            if !error && !got_int.get() {
                spell_message(&raw mut spin, gettext(msg_compressing.get()));
                wordtree_compress(
                    &raw mut spin,
                    spin.si_foldroot,
                    b"case-folded\0".as_ptr() as *const ::core::ffi::c_char,
                );
                wordtree_compress(
                    &raw mut spin,
                    spin.si_keeproot,
                    b"keep-case\0".as_ptr() as *const ::core::ffi::c_char,
                );
                wordtree_compress(
                    &raw mut spin,
                    spin.si_prefroot,
                    b"prefixes\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if !error && !got_int.get() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Writing spell file %s...\0".as_ptr() as *const ::core::ffi::c_char),
                    wfname,
                );
                spell_message(&raw mut spin, IObuff.ptr() as *mut ::core::ffi::c_char);
                error = write_vim_spell(&raw mut spin, wfname) == FAIL;
                spell_message(
                    &raw mut spin,
                    gettext(b"Done!\0".as_ptr() as *const ::core::ffi::c_char),
                );
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Estimated runtime memory use: %d bytes\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    spin.si_memtot,
                );
                spell_message(&raw mut spin, IObuff.ptr() as *mut ::core::ffi::c_char);
                if !error {
                    spell_reload_one(wfname, added_word);
                }
            }
            ga_clear(&raw mut spin.si_rep);
            ga_clear(&raw mut spin.si_repsal);
            ga_clear(&raw mut spin.si_sal);
            ga_clear(&raw mut spin.si_map);
            ga_clear(&raw mut spin.si_comppat);
            ga_clear(&raw mut spin.si_prefcond);
            hash_clear_all(&raw mut spin.si_commonwords, 0 as ::core::ffi::c_uint);
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < incount {
                if !afile[i_1 as usize].is_null() {
                    spell_free_aff(afile[i_1 as usize] as *mut afffile_T);
                }
                i_1 += 1;
            }
            free_blocks(spin.si_blocks);
            if spin.si_sugtime != 0 as time_t && !error && !got_int.get() {
                spell_make_sugfile(&raw mut spin, wfname);
            }
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(wfname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn spell_message(
    mut spin: *const spellinfo_T,
    mut str: *mut ::core::ffi::c_char,
) {
    if (*spin).si_verbose != 0 || p_verbose.get() > 2 as OptInt {
        if (*spin).si_verbose == 0 {
            verbose_enter();
        }
        msg(str, 0 as ::core::ffi::c_int);
        ui_flush();
        if (*spin).si_verbose == 0 {
            verbose_leave();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_spell(mut eap: *mut exarg_T) {
    spell_add_word(
        (*eap).arg,
        strlen((*eap).arg) as ::core::ffi::c_int,
        (if (*eap).cmdidx as ::core::ffi::c_int == CMD_spellwrong as ::core::ffi::c_int {
            SPELL_ADD_BAD as ::core::ffi::c_int
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_spellrare as ::core::ffi::c_int {
            SPELL_ADD_RARE as ::core::ffi::c_int
        } else {
            SPELL_ADD_GOOD as ::core::ffi::c_int
        }) as SpellAddType,
        if (*eap).forceit != 0 {
            0 as ::core::ffi::c_int
        } else {
            (*eap).line2 as ::core::ffi::c_int
        },
        (*eap).cmdidx as ::core::ffi::c_int == CMD_spellundo as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn spell_add_word(
    mut word: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut what: SpellAddType,
    mut idx: ::core::ffi::c_int,
    mut undo: bool,
) {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut new_spf: bool = false_0 != 0;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fnamebuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: [::core::ffi::c_char; 508] = [0; 508];
    let mut spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !valid_spell_word(word, word.offset(len as isize)) {
        emsg(gettext(e_illegal_character_in_word.get()));
        return;
    }
    if idx == 0 as ::core::ffi::c_int {
        if (*int_wordlist.ptr()).is_null() {
            int_wordlist.set(vim_tempname());
            if (*int_wordlist.ptr()).is_null() {
                return;
            }
        }
        fname = int_wordlist.get();
    } else {
        let mut i: ::core::ffi::c_int = 0;
        if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
            init_spellfile();
            new_spf = true_0 != 0;
        }
        if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
            semsg(
                gettext(&raw const e_notset as *const ::core::ffi::c_char),
                b"spellfile\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        fnamebuf = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        spf = (*(*curwin.get()).w_s).b_p_spf;
        i = 1 as ::core::ffi::c_int;
        while *spf as ::core::ffi::c_int != NUL {
            copy_option_part(
                &raw mut spf,
                fnamebuf,
                MAXPATHL as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if i == idx {
                break;
            }
            if *spf as ::core::ffi::c_int == NUL {
                semsg(
                    gettext(b"E765: 'spellfile' does not have %d entries\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    idx,
                );
                xfree(fnamebuf as *mut ::core::ffi::c_void);
                return;
            }
            i += 1;
        }
        buf = buflist_findname_exp(fnamebuf);
        if !buf.is_null() && (*buf).b_ml.ml_mfp.is_null() {
            buf = ::core::ptr::null_mut::<buf_T>();
        }
        if !buf.is_null() && bufIsChanged(buf) as ::core::ffi::c_int != 0 {
            emsg(gettext(
                &raw const e_bufloaded as *const ::core::ffi::c_char,
            ));
            xfree(fnamebuf as *mut ::core::ffi::c_void);
            return;
        }
        fname = fnamebuf;
    }
    if what as ::core::ffi::c_uint == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
        || undo as ::core::ffi::c_int != 0
    {
        let mut fpos_next: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fpos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        fd = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
        if !fd.is_null() {
            while !vim_fgets(
                &raw mut line as *mut ::core::ffi::c_char,
                MAXWLEN as ::core::ffi::c_int * 2 as ::core::ffi::c_int,
                fd,
            ) {
                fpos = fpos_next;
                fpos_next = ftell(fd) as ::core::ffi::c_int;
                if fpos_next < 0 as ::core::ffi::c_int {
                    break;
                }
                if !(strncmp(
                    word,
                    &raw mut line as *mut ::core::ffi::c_char,
                    len as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (line[len as usize] as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                        || (line[len as usize] as uint8_t as ::core::ffi::c_int)
                            < ' ' as ::core::ffi::c_int))
                {
                    continue;
                }
                fclose(fd);
                fd = os_fopen(fname, b"r+\0".as_ptr() as *const ::core::ffi::c_char);
                if fd.is_null() {
                    break;
                }
                if fseek(fd, fpos as ::core::ffi::c_long, SEEK_SET) == 0 as ::core::ffi::c_int {
                    fputc('#' as ::core::ffi::c_int, fd);
                    if undo {
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            fname,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Word '%.*s' removed from %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            len,
                            word,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                        );
                    }
                }
                if fseek(fd, fpos_next as ::core::ffi::c_long, SEEK_SET) == 0 as ::core::ffi::c_int
                {
                    continue;
                }
                semsg(
                    b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"Seek error in spellfile\0".as_ptr() as *const ::core::ffi::c_char),
                    strerror(*__errno_location()),
                );
                break;
            }
            if !fd.is_null() {
                fclose(fd);
            }
        }
    }
    if !undo {
        fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
        if fd.is_null() && new_spf as ::core::ffi::c_int != 0 {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !dir_of_file_exists(fname) && {
                p = path_tail_with_sep(fname);
                p != fname
            } {
                let mut c: ::core::ffi::c_char = *p;
                *p = NUL as ::core::ffi::c_char;
                os_mkdir(fname, 0o755 as int32_t);
                *p = c;
                fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        if fd.is_null() {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                fname,
            );
        } else {
            if what as ::core::ffi::c_uint
                == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    fd,
                    b"%.*s/!\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            } else if what as ::core::ffi::c_uint
                == SPELL_ADD_RARE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    fd,
                    b"%.*s/?\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            } else {
                fprintf(
                    fd,
                    b"%.*s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            }
            fclose(fd);
            home_replace(
                ::core::ptr::null::<buf_T>(),
                fname,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Word '%.*s' added to %s\0".as_ptr() as *const ::core::ffi::c_char),
                len,
                word,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
    }
    if !fd.is_null() {
        mkspell(
            1 as ::core::ffi::c_int,
            &raw mut fname,
            false_0 != 0,
            true_0 != 0,
            true_0 != 0,
        );
        if !buf.is_null() {
            buf_reload(buf, (*buf).b_orig_mode, false_0 != 0);
        }
        redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    }
    xfree(fnamebuf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn init_spellfile() {
    let mut lend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut aspath: bool = false_0 != 0;
    let mut lstart: *mut ::core::ffi::c_char = (*curbuf.get()).b_s.b_p_spl;
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL
        || (*(*curwin.get()).w_s).b_langp.ga_len <= 0 as ::core::ffi::c_int
    {
        return;
    }
    lend = (*(*curwin.get()).w_s).b_p_spl;
    while *lend as ::core::ffi::c_int != NUL
        && vim_strchr(
            b",._\0".as_ptr() as *const ::core::ffi::c_char,
            *lend as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        if vim_ispathsep(*lend as ::core::ffi::c_int) {
            aspath = true_0 != 0;
            lstart = lend.offset(1 as ::core::ffi::c_int as isize);
        }
        lend = lend.offset(1);
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut buf_len: size_t = MAXPATHL as size_t;
    if !aspath {
        let mut xdg_path: *mut ::core::ffi::c_char = get_xdg_home(kXDGDataHome);
        xstrlcpy(buf, xdg_path, buf_len);
        xfree(xdg_path as *mut ::core::ffi::c_void);
        xstrlcat(
            buf,
            b"/site/spell\0".as_ptr() as *const ::core::ffi::c_char,
            buf_len,
        );
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        if os_mkdir_recurse(
            buf,
            0o755 as int32_t,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ) != 0 as ::core::ffi::c_int
        {
            xfree(buf as *mut ::core::ffi::c_void);
            xfree(failed_dir as *mut ::core::ffi::c_void);
            return;
        }
    } else {
        if lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t >= buf_len {
            xfree(buf as *mut ::core::ffi::c_void);
            return;
        }
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            (*curbuf.get()).b_s.b_p_spl as *const ::core::ffi::c_void,
            lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t,
        );
    }
    vim_snprintf(
        buf.offset(strlen(buf) as isize),
        buf_len.wrapping_sub(strlen(buf)),
        b"/%.*s\0".as_ptr() as *const ::core::ffi::c_char,
        lend.offset_from(lstart) as ::core::ffi::c_int,
        lstart,
    );
    let mut fname: *mut ::core::ffi::c_char = (*(*((*(*curwin.get()).w_s).b_langp.ga_data
        as *mut langp_T)
        .offset(0 as ::core::ffi::c_int as isize))
    .lp_slang)
        .sl_fname;
    let mut enc_suffix: *const ::core::ffi::c_char = if !fname.is_null()
        && !strstr(
            path_tail(fname),
            b".ascii.\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
    {
        b"ascii\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        spell_enc() as *const ::core::ffi::c_char
    };
    vim_snprintf(
        buf.offset(strlen(buf) as isize),
        buf_len.wrapping_sub(strlen(buf)),
        b".%s.add\0".as_ptr() as *const ::core::ffi::c_char,
        enc_suffix,
    );
    set_option_value_give_err(
        kOptSpellfile,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(buf),
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn set_spell_charflags(
    mut flags_in: *const ::core::ffi::c_char,
    mut cnt: ::core::ffi::c_int,
    mut fol: *const ::core::ffi::c_char,
) {
    let mut flags: *const uint8_t = flags_in as *mut uint8_t;
    let mut new_st: spelltab_T = spelltab_T {
        st_isw: [false; 256],
        st_isu: [false; 256],
        st_fold: [0; 256],
        st_upper: [0; 256],
    };
    let mut p: *const ::core::ffi::c_char = fol;
    clear_spell_chartab(&raw mut new_st);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 128 as ::core::ffi::c_int {
        if i < cnt {
            new_st.st_isw[(i + 128 as ::core::ffi::c_int) as usize] =
                *flags.offset(i as isize) as ::core::ffi::c_int & CF_WORD as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
            new_st.st_isu[(i + 128 as ::core::ffi::c_int) as usize] =
                *flags.offset(i as isize) as ::core::ffi::c_int & CF_UPPER as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
        }
        if *p as ::core::ffi::c_int != NUL {
            let mut c: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p);
            new_st.st_fold[(i + 128 as ::core::ffi::c_int) as usize] = c as uint8_t;
            if i + 128 as ::core::ffi::c_int != c
                && new_st.st_isu[(i + 128 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                    != 0
                && c < 256 as ::core::ffi::c_int
            {
                new_st.st_upper[c as usize] = (i + 128 as ::core::ffi::c_int) as uint8_t;
            }
        }
        i += 1;
    }
    set_spell_finish(&raw mut new_st);
}
unsafe extern "C" fn set_spell_finish(mut new_st: *mut spelltab_T) -> ::core::ffi::c_int {
    if did_set_spelltab.get() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            if (*spelltab.ptr()).st_isw[i as usize] as ::core::ffi::c_int
                != (*new_st).st_isw[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_isu[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_isu[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_fold[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_fold[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_upper[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_upper[i as usize] as ::core::ffi::c_int
            {
                emsg(gettext(
                    b"E763: Word characters differ between spell files\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            i += 1;
        }
    } else {
        spelltab.set(*new_st);
        did_set_spelltab.set(true_0 != 0);
    }
    return OK;
}
unsafe extern "C" fn write_spell_prefcond(
    mut fd: *mut FILE,
    mut gap: *mut garray_T,
    mut fwv: *mut size_t,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if (*gap).ga_len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"gap->ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                5698 as ::core::ffi::c_uint,
                b"int write_spell_prefcond(FILE *, garray_T *, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !fd.is_null() {
        put_bytes(fd, (*gap).ga_len as uintmax_t, 2 as size_t);
    }
    let mut totlen: size_t = (2 as size_t).wrapping_add((*gap).ga_len as size_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut p: *mut ::core::ffi::c_char =
            *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        if !p.is_null() {
            let mut len: size_t = strlen(p);
            if !fd.is_null() {
                '_c2rust_label_0: {
                    if len <= 2147483647 as ::core::ffi::c_int as size_t {
                    } else {
                        __assert_fail(
                            b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            5710 as ::core::ffi::c_uint,
                            b"int write_spell_prefcond(FILE *, garray_T *, size_t *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                fputc(len as ::core::ffi::c_int, fd);
                *fwv = (*fwv as ::core::ffi::c_ulong
                    & fwrite(p as *const ::core::ffi::c_void, len, 1 as size_t, fd))
                    as size_t;
            }
            totlen = totlen.wrapping_add(len);
        } else if !fd.is_null() {
            fputc(0 as ::core::ffi::c_int, fd);
        }
        i += 1;
    }
    '_c2rust_label_1: {
        if totlen <= 2147483647 as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"totlen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                5720 as ::core::ffi::c_uint,
                b"int write_spell_prefcond(FILE *, garray_T *, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return totlen as ::core::ffi::c_int;
}
unsafe extern "C" fn set_map_str(mut lp: *mut slang_T, mut map: *const ::core::ffi::c_char) {
    let mut headc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *map as ::core::ffi::c_int == NUL {
        (*lp).sl_has_map = false_0 != 0;
        return;
    }
    (*lp).sl_has_map = true_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        (*lp).sl_map_array[i as usize] = 0 as ::core::ffi::c_int;
        i += 1;
    }
    hash_init(&raw mut (*lp).sl_map_hash);
    let mut p: *const ::core::ffi::c_char = map;
    while *p as ::core::ffi::c_int != NUL {
        let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
        if c == '/' as ::core::ffi::c_int {
            headc = 0 as ::core::ffi::c_int;
        } else {
            if headc == 0 as ::core::ffi::c_int {
                headc = c;
            }
            if c >= 256 as ::core::ffi::c_int {
                let mut cl: ::core::ffi::c_int = utf_char2len(c);
                let mut headcl: ::core::ffi::c_int = utf_char2len(headc);
                let mut hash: hash_T = 0;
                let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
                let mut b: *mut ::core::ffi::c_char =
                    xmalloc(((cl + headcl) as size_t).wrapping_add(2 as size_t))
                        as *mut ::core::ffi::c_char;
                utf_char2bytes(c, b);
                *b.offset(cl as isize) = NUL as ::core::ffi::c_char;
                utf_char2bytes(
                    headc,
                    b.offset(cl as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                );
                *b.offset((cl + 1 as ::core::ffi::c_int + headcl) as isize) =
                    NUL as ::core::ffi::c_char;
                hash = hash_hash(b);
                hi = hash_lookup(&raw mut (*lp).sl_map_hash, b, strlen(b), hash);
                if (*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
                {
                    hash_add_item(&raw mut (*lp).sl_map_hash, hi, b, hash);
                } else {
                    emsg(gettext(
                        (e_duplicate_char_in_map_entry.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    xfree(b as *mut ::core::ffi::c_void);
                }
            } else {
                (*lp).sl_map_array[c as usize] = headc;
            }
        }
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_STRICT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
