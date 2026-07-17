extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    pub type Unpacker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn toupper(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn abort() -> !;
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
    fn strcpy(
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
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn strchrsub(str: *mut ::core::ffi::c_char, c: ::core::ffi::c_char, x: ::core::ffi::c_char);
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn strnequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char, n: size_t) -> bool;
    fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t;
    fn map_del_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut uint64_t,
    ) -> ptr_t;
    fn map_put_ref_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut *mut uint64_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    static mut autocmd_fname: *mut ::core::ffi::c_char;
    static mut autocmd_fname_full: bool;
    static mut autocmd_bufnr: ::core::ffi::c_int;
    static mut autocmd_match: *mut ::core::ffi::c_char;
    static mut aucmd_win_vec: C2Rust_Unnamed_35;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn appended_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    static mut channels: Map_uint64_t_ptr_t;
    fn callback_reader_free(reader: *mut CallbackReader);
    static mut empty_string_option: [::core::ffi::c_char; 0];
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_lpl: ::core::ffi::c_int;
    static mut p_mfd: OptInt;
    static mut p_verbose: OptInt;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_str2nr(
        start: *const ::core::ffi::c_char,
        prep: *mut ::core::ffi::c_int,
        len: *mut ::core::ffi::c_int,
        what: ::core::ffi::c_int,
        nptr: *mut varnumber_T,
        unptr: *mut uvarnumber_T,
        maxlen: ::core::ffi::c_int,
        strict: bool,
        overflow: *mut bool,
    );
    fn hex2nr(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static e_command_too_recursive: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_invexpr2: [::core::ffi::c_char; 0];
    static e_invchan: [::core::ffi::c_char; 0];
    static e_invchanjob: [::core::ffi::c_char; 0];
    static e_letwrong: [::core::ffi::c_char; 0];
    static e_illvar: [::core::ffi::c_char; 0];
    static e_cannot_mod: [::core::ffi::c_char; 0];
    static e_invalblob: [::core::ffi::c_char; 0];
    static e_dictkey: [::core::ffi::c_char; 0];
    static e_dictkey_len: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_missingparen: [::core::ffi::c_char; 0];
    static e_nobufnr: [::core::ffi::c_char; 0];
    static e_using_float_as_string: [::core::ffi::c_char; 0];
    static e_not_callable_type_str: [::core::ffi::c_char; 0];
    static e_fast_api_disabled: [::core::ffi::c_char; 0];
    static e_invalid_value_for_blob_nr: [::core::ffi::c_char; 0];
    static e_stray_closing_curly_str: [::core::ffi::c_char; 0];
    static line_msg: [::core::ffi::c_char; 0];
    static mut EVALARG_EVALUATE: evalarg_T;
    fn encode_list_write(
        data: *mut ::core::ffi::c_void,
        buf: *const ::core::ffi::c_char,
        len: size_t,
    );
    fn encode_tv2string(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn encode_tv2echo(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    static mut gc_first_dict: *mut dict_T;
    static mut gc_first_list: *mut list_T;
    static mut hash_removed: ::core::ffi::c_char;
    fn eexe_mod_op(
        tv1: *mut typval_T,
        tv2: *const typval_T,
        op: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn hash_init(ht: *mut hashtab_T);
    fn verb_msg(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn msg_multiline(
        str: String_0,
        hl_id: ::core::ffi::c_int,
        check_int: bool,
        hist: bool,
        need_clear: *mut bool,
    );
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg_multiline(
        s: *const ::core::ffi::c_char,
        kind: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        multiline: bool,
    ) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_ext_set_append(append: bool);
    fn msg_start();
    fn msg_outnum(n: ::core::ffi::c_int);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_puts_len(
        str: *const ::core::ffi::c_char,
        len: ptrdiff_t,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    );
    fn msg_sb_eol();
    fn msg_clr_eos();
    fn msg_end() -> bool;
    fn verbose_enter();
    fn verbose_leave();
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    static mut msg_ext_skip_verbose: bool;
    static tv_empty_string: *const ::core::ffi::c_char;
    static mut tv_in_free_unref_items: bool;
    fn tv_list_watch_add(l: *mut list_T, lw: *mut listwatch_T);
    fn tv_list_watch_remove(l: *mut list_T, lwrem: *mut listwatch_T);
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_free_contents(l: *mut list_T);
    fn tv_list_free_list(l: *mut list_T);
    fn tv_list_free(l: *mut list_T);
    fn tv_list_unref(l: *mut list_T);
    fn tv_list_append_owned_tv(l: *mut list_T, tv: typval_T) -> *mut typval_T;
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_copy(
        conv: *const vimconv_T,
        orig: *mut list_T,
        deep: bool,
        copyID: ::core::ffi::c_int,
    ) -> *mut list_T;
    fn tv_list_check_range_index_one(
        l: *mut list_T,
        n1: *mut ::core::ffi::c_int,
        quiet: bool,
    ) -> *mut listitem_T;
    fn tv_list_check_range_index_two(
        l: *mut list_T,
        n1: *mut ::core::ffi::c_int,
        li1: *const listitem_T,
        n2: *mut ::core::ffi::c_int,
        quiet: bool,
    ) -> ::core::ffi::c_int;
    fn tv_list_assign_range(
        dest: *mut list_T,
        src: *mut list_T,
        idx1_arg: ::core::ffi::c_int,
        idx2: ::core::ffi::c_int,
        empty_idx2: bool,
        op: *const ::core::ffi::c_char,
        varname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_list_concat(l1: *mut list_T, l2: *mut list_T, tv: *mut typval_T) -> ::core::ffi::c_int;
    fn tv_list_slice_or_index(
        list: *mut list_T,
        range: bool,
        n1_arg: varnumber_T,
        n2_arg: varnumber_T,
        exclusive: bool,
        rettv: *mut typval_T,
        verbose: bool,
    ) -> ::core::ffi::c_int;
    fn tv_list_join(
        gap: *mut garray_T,
        l: *mut list_T,
        sep: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_list_equal(l1: *mut list_T, l2: *mut list_T, ic: bool) -> bool;
    fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T;
    fn tv_list_find_nr(l: *mut list_T, n: ::core::ffi::c_int, ret_error: *mut bool) -> varnumber_T;
    fn callback_free(callback: *mut Callback);
    fn callback_put(cb: *mut Callback, tv: *mut typval_T);
    fn tv_dict_watcher_notify(
        dict: *mut dict_T,
        key: *const ::core::ffi::c_char,
        newtv: *mut typval_T,
        oldtv: *mut typval_T,
    );
    fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T;
    fn tv_dict_item_free(item: *mut dictitem_T);
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_free_contents(d: *mut dict_T);
    fn tv_dict_free_dict(d: *mut dict_T);
    fn tv_dict_free(d: *mut dict_T);
    fn tv_dict_unref(d: *mut dict_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
    fn tv_dict_get_callback(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: ptrdiff_t,
        result: *mut Callback,
    ) -> bool;
    fn tv_dict_wrong_func_name(
        d: *mut dict_T,
        tv: *mut typval_T,
        name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_equal(d1: *mut dict_T, d2: *mut dict_T, ic: bool) -> bool;
    fn tv_dict_copy(
        conv: *const vimconv_T,
        orig: *mut dict_T,
        deep: bool,
        copyID: ::core::ffi::c_int,
    ) -> *mut dict_T;
    fn tv_blob_alloc() -> *mut blob_T;
    fn tv_blob_unref(b: *mut blob_T);
    fn tv_blob_equal(b1: *const blob_T, b2: *const blob_T) -> bool;
    fn tv_blob_slice_or_index(
        blob: *const blob_T,
        is_range: bool,
        n1: varnumber_T,
        n2: varnumber_T,
        exclusive: bool,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn tv_blob_check_index(
        bloblen: ::core::ffi::c_int,
        n1: varnumber_T,
        quiet: bool,
    ) -> ::core::ffi::c_int;
    fn tv_blob_check_range(
        bloblen: ::core::ffi::c_int,
        n1: varnumber_T,
        n2: varnumber_T,
        quiet: bool,
    ) -> ::core::ffi::c_int;
    fn tv_blob_set_range(
        dest: *mut blob_T,
        n1: varnumber_T,
        n2: varnumber_T,
        src: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn tv_blob_set_append(blob: *mut blob_T, idx: ::core::ffi::c_int, byte: uint8_t);
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T;
    fn tv_blob_copy(from: *mut blob_T, to: *mut typval_T);
    fn tv_clear(tv: *mut typval_T);
    fn tv_copy(from: *const typval_T, to: *mut typval_T);
    fn tv_check_lock(
        tv: *const typval_T,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn value_check_lock(
        lock: VarLockStatus,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn tv_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool;
    fn tv_check_num(tv: *const typval_T) -> bool;
    fn tv_check_str(tv: *const typval_T) -> bool;
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_get_float(tv: *const typval_T) -> float_T;
    fn tv_get_string_buf_chk(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string_buf(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv2bool(tv: *const typval_T) -> bool;
    fn func_init();
    fn get_lambda_tv(
        arg: *mut *mut ::core::ffi::c_char,
        rettv: *mut typval_T,
        evalarg: *mut evalarg_T,
    ) -> ::core::ffi::c_int;
    fn deref_func_name(
        name: *const ::core::ffi::c_char,
        lenp: *mut ::core::ffi::c_int,
        partialp: *mut *mut partial_T,
        no_autoload: bool,
        found_var: *mut bool,
    ) -> *mut ::core::ffi::c_char;
    fn get_func_tv(
        name: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        rettv: *mut typval_T,
        arg: *mut *mut ::core::ffi::c_char,
        evalarg: *mut evalarg_T,
        funcexe: *mut funcexe_T,
    ) -> ::core::ffi::c_int;
    fn find_func(name: *const ::core::ffi::c_char) -> *mut ufunc_T;
    fn save_funccal(entry: *mut funccal_entry_T);
    fn restore_funccal();
    fn get_current_funccal() -> *mut funccall_T;
    fn call_func(
        funcname: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        rettv: *mut typval_T,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        funcexe: *mut funcexe_T,
    ) -> ::core::ffi::c_int;
    fn call_simple_luafunc(
        funcname: *const ::core::ffi::c_char,
        len: size_t,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn call_simple_func(
        funcname: *const ::core::ffi::c_char,
        len: size_t,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn get_scriptlocal_funcname(funcname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn eval_fname_script(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn func_unref(name: *mut ::core::ffi::c_char);
    fn func_ptr_unref(fp: *mut ufunc_T);
    fn func_ref(name: *mut ::core::ffi::c_char);
    fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T);
    fn free_unref_funccal(copyID: ::core::ffi::c_int, testing: ::core::ffi::c_int) -> bool;
    fn get_funccal_args_ht() -> *mut hashtab_T;
    fn set_ref_in_previous_funccal(copyID: ::core::ffi::c_int) -> bool;
    fn set_ref_in_call_stack(copyID: ::core::ffi::c_int) -> bool;
    fn set_ref_in_functions(copyID: ::core::ffi::c_int) -> bool;
    fn set_ref_in_func_args(copyID: ::core::ffi::c_int) -> bool;
    fn set_ref_in_func(
        name: *mut ::core::ffi::c_char,
        fp_in: *mut ufunc_T,
        copyID: ::core::ffi::c_int,
    ) -> bool;
    fn evalvars_init();
    fn garbage_collect_globvars(copyID: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn garbage_collect_vimvars(copyID: ::core::ffi::c_int) -> bool;
    fn garbage_collect_scriptvars(copyID: ::core::ffi::c_int) -> bool;
    fn eval_one_expr_in_str(
        p: *mut ::core::ffi::c_char,
        gap: *mut garray_T,
        evaluate: bool,
    ) -> *mut ::core::ffi::c_char;
    fn ex_let_vars(
        arg_start: *mut ::core::ffi::c_char,
        tv: *mut typval_T,
        copy: ::core::ffi::c_int,
        semicolon: ::core::ffi::c_int,
        var_count: ::core::ffi::c_int,
        is_const: ::core::ffi::c_int,
        op: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn skip_var_list(
        arg: *const ::core::ffi::c_char,
        var_count: *mut ::core::ffi::c_int,
        semicolon: *mut ::core::ffi::c_int,
        silent: bool,
    ) -> *const ::core::ffi::c_char;
    fn get_vimvar_dict() -> *mut dict_T;
    fn get_vim_var_dict(idx: VimVarIndex) -> *mut dict_T;
    fn get_vim_var_partial(idx: VimVarIndex) -> *mut partial_T;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T);
    fn eval_variable(
        name: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        rettv: *mut typval_T,
        dip: *mut *mut dictitem_T,
        verbose: bool,
        no_autoload: bool,
    ) -> ::core::ffi::c_int;
    fn check_vars(name: *const ::core::ffi::c_char, len: size_t);
    fn find_var(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        htp: *mut *mut hashtab_T,
        no_autoload: ::core::ffi::c_int,
    ) -> *mut dictitem_T;
    fn set_var(name: *const ::core::ffi::c_char, name_len: size_t, tv: *mut typval_T, copy: bool);
    fn set_var_const(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        tv: *mut typval_T,
        copy: bool,
        is_const: bool,
    );
    fn var_check_ro(
        flags: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn var_check_lock(
        flags: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn var_wrong_func_name(name: *const ::core::ffi::c_char, new_var: bool) -> bool;
    fn valid_varname(varname: *const ::core::ffi::c_char) -> bool;
    fn optval_as_tv(value: OptVal, numbool: bool) -> typval_T;
    fn time_watcher_init(
        loop_0: *mut Loop,
        watcher: *mut TimeWatcher,
        data: *mut ::core::ffi::c_void,
    );
    fn time_watcher_start(
        watcher: *mut TimeWatcher,
        cb: time_cb,
        timeout: uint64_t,
        repeat: uint64_t,
    );
    fn time_watcher_stop(watcher: *mut TimeWatcher);
    fn time_watcher_close(watcher: *mut TimeWatcher, cb: time_cb);
    fn check_secure() -> bool;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_has_expr_args(cmdidx: cmdidx_T) -> bool;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn check_nextcmd(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_ref_in_findfunc(copyID: ::core::ffi::c_int) -> bool;
    fn get_pressedreturn() -> bool;
    fn set_pressedreturn(val: bool);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn aborting() -> bool;
    fn discard_current_exception();
    static mut msg_didout: bool;
    static mut emsg_off: ::core::ffi::c_int;
    static mut need_clr_eos: bool;
    static mut emsg_skip: ::core::ffi::c_int;
    static mut emsg_severe: bool;
    static mut did_emsg: ::core::ffi::c_int;
    static mut called_emsg: ::core::ffi::c_int;
    static mut do_profiling: ::core::ffi::c_int;
    static mut did_throw: bool;
    static mut force_abort: bool;
    static mut may_garbage_collect: bool;
    static mut want_garbage_collect: bool;
    static mut garbage_collect_at_exit: bool;
    static mut current_sctx: sctx_T;
    static mut provider_caller_scope: caller_scope;
    static mut provider_call_nesting: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut textlock: ::core::ffi::c_int;
    static mut sandbox: ::core::ffi::c_int;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut got_int: bool;
    fn syn_name2id(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn set_ref_in_cpt_callbacks(
        callbacks: *mut Callback,
        count: ::core::ffi::c_int,
        copyID: ::core::ffi::c_int,
    ) -> bool;
    fn set_ref_in_insexpand_funcs(copyID: ::core::ffi::c_int) -> bool;
    fn trans_special(
        srcp: *mut *const ::core::ffi::c_char,
        src_len: size_t,
        dst: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        escape_ks: bool,
        did_simplify: *mut bool,
    ) -> ::core::ffi::c_uint;
    fn find_special_key(
        srcp: *mut *const ::core::ffi::c_char,
        src_len: size_t,
        modp: *mut ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        did_simplify: *mut bool,
    ) -> ::core::ffi::c_int;
    static mut main_loop: Loop;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_is_deferred_safe() -> bool;
    fn nlua_is_table_from_lua(arg: *const typval_T) -> bool;
    fn nlua_register_table_as_callable(arg: *const typval_T) -> *mut ::core::ffi::c_char;
    fn mark_get(
        buf: *mut buf_T,
        win: *mut win_T,
        fmp: *mut fmark_T,
        flag: MarkGet,
        name: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn mark_global_iter(
        iter: *const ::core::ffi::c_void,
        name: *mut ::core::ffi::c_char,
        fm: *mut xfmark_T,
    ) -> *const ::core::ffi::c_void;
    fn vim_to_object(obj: *mut typval_T, arena: *mut Arena, reuse_strdata: bool) -> Object;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_copy_char(fp: *mut *const ::core::ffi::c_char, tp: *mut *mut ::core::ffi::c_char);
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn string_convert(
        vcp: *const vimconv_T,
        ptr: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn mb_strcmp_ic(
        ic: bool,
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn check_cursor_moved(wp: *mut win_T);
    fn validate_botline_win(wp: *mut win_T);
    fn set_ref_in_opfunc(copyID: ::core::ffi::c_int) -> bool;
    fn find_option_end(
        arg: *const ::core::ffi::c_char,
        opt_idxp: *mut OptIndex,
    ) -> *const ::core::ffi::c_char;
    fn was_set_insecurely(
        wp: *mut win_T,
        opt_idx: OptIndex,
        opt_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn is_tty_option(name: *const ::core::ffi::c_char) -> bool;
    fn get_tty_option(name: *const ::core::ffi::c_char) -> OptVal;
    fn is_option_hidden(opt_idx: OptIndex) -> bool;
    fn get_option_value(opt_idx: OptIndex, opt_flags: ::core::ffi::c_int) -> OptVal;
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn free_string_option(p: *mut ::core::ffi::c_char);
    fn os_can_exe(
        name: *const ::core::ffi::c_char,
        abspath: *mut *mut ::core::ffi::c_char,
        use_path: bool,
    ) -> bool;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn shell_build_argv(
        cmd: *const ::core::ffi::c_char,
        extra_args: *const ::core::ffi::c_char,
    ) -> *mut *mut ::core::ffi::c_char;
    fn shell_free_argv(argv: *mut *mut ::core::ffi::c_char);
    fn shell_argv_to_str(argv: *mut *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_system(
        argv: *mut *mut ::core::ffi::c_char,
        input: *const ::core::ffi::c_char,
        len: size_t,
        output: *mut *mut ::core::ffi::c_char,
        nread: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn prof_child_enter(tm: *mut proftime_T);
    fn prof_child_exit(tm: *mut proftime_T);
    fn set_ref_in_quickfix(copyID: ::core::ffi::c_int) -> bool;
    fn op_global_reg_iter(
        iter: *const ::core::ffi::c_void,
        name: *mut ::core::ffi::c_char,
        reg: *mut yankreg_T,
        is_unnamed: *mut bool,
    ) -> *const ::core::ffi::c_void;
    fn get_reg_contents(
        regname: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_void;
    fn vim_regsub(
        rmp: *mut regmatch_T,
        source: *mut ::core::ffi::c_char,
        expr: *mut typval_T,
        dest: *mut ::core::ffi::c_char,
        destlen: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
        -> bool;
    fn script_is_lua(sid: scid_T) -> bool;
    fn get_scriptname(script_ctx: sctx_T, should_free: *mut bool) -> *mut ::core::ffi::c_char;
    fn sourcing_a_script(eap: *mut exarg_T) -> ::core::ffi::c_int;
    fn script_autoload(name: *const ::core::ffi::c_char, name_len: size_t, reload: bool) -> bool;
    static mut exestack: garray_T;
    fn set_ref_in_tagfunc(copyID: ::core::ffi::c_int) -> bool;
    fn u_clearallandblockfree(buf: *mut buf_T);
    fn ui_has(ext: UIExtension) -> bool;
    fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue;
    fn multiqueue_free(self_0: *mut MultiQueue);
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
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
pub type size_t = usize;
pub type ssize_t = isize;
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
pub type ptrdiff_t = isize;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_5,
    pub internal_fields: *mut ::core::ffi::c_void,
    pub stop_flag: ::core::ffi::c_uint,
    pub flags: ::core::ffi::c_ulong,
    pub backend_fd: ::core::ffi::c_int,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub watchers: *mut *mut uv__io_t,
    pub nwatchers: ::core::ffi::c_uint,
    pub nfds: ::core::ffi::c_uint,
    pub wq: uv__queue,
    pub wq_mutex: uv_mutex_t,
    pub wq_async: uv_async_t,
    pub cloexec_lock: uv_rwlock_t,
    pub closing_handles: *mut uv_handle_t,
    pub process_handles: uv__queue,
    pub prepare_handles: uv__queue,
    pub check_handles: uv__queue,
    pub idle_handles: uv__queue,
    pub async_handles: uv__queue,
    pub async_unused: Option<unsafe extern "C" fn() -> ()>,
    pub async_io_watcher: uv__io_t,
    pub async_wfd: ::core::ffi::c_int,
    pub timer_heap: C2Rust_Unnamed_3,
    pub timer_counter: uint64_t,
    pub time: uint64_t,
    pub signal_pipefd: [::core::ffi::c_int; 2],
    pub signal_io_watcher: uv__io_t,
    pub child_watcher: uv_signal_t,
    pub emfile_fd: ::core::ffi::c_int,
    pub inotify_read_watcher: uv__io_t,
    pub inotify_watchers: *mut ::core::ffi::c_void,
    pub inotify_fd: ::core::ffi::c_int,
}
pub type uv__io_t = uv__io_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__io_s {
    pub cb: uv__io_cb,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub pevents: ::core::ffi::c_uint,
    pub events: ::core::ffi::c_uint,
    pub fd: ::core::ffi::c_int,
}
pub type uv__io_cb =
    Option<unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> ()>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_2,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_0,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_1,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_close_cb = Option<unsafe extern "C" fn(*mut uv_handle_t) -> ()>;
pub type uv_handle_type = ::core::ffi::c_uint;
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
pub type uv_loop_t = uv_loop_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_4,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_mutex_t = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type uv_file = ::core::ffi::c_int;
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
pub type uv_req_type = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_6,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
pub type uv_connection_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> ()>;
pub type uv_stream_t = uv_stream_s;
pub type uv_shutdown_t = uv_shutdown_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_shutdown_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub handle: *mut uv_stream_t,
    pub cb: uv_shutdown_cb,
}
pub type uv_shutdown_cb =
    Option<unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> ()>;
pub type uv_connect_t = uv_connect_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_connect_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_connect_cb,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
}
pub type uv_connect_cb = Option<unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()>;
pub type uv_read_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()>;
pub type uv_alloc_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_7,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_8,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_10,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_9,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_11,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_12,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub exit_cb: uv_exit_cb,
    pub pid: ::core::ffi::c_int,
    pub queue: uv__queue,
    pub status: ::core::ffi::c_int,
}
pub type uv_exit_cb =
    Option<unsafe extern "C" fn(*mut uv_process_t, int64_t, ::core::ffi::c_int) -> ()>;
pub type uv_process_t = uv_process_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_stdio_flags = ::core::ffi::c_uint;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stdio_container_s {
    pub flags: uv_stdio_flags,
    pub data: C2Rust_Unnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
    pub stream: *mut uv_stream_t,
    pub fd: ::core::ffi::c_int,
}
pub type uv_stdio_container_t = uv_stdio_container_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_options_s {
    pub exit_cb: uv_exit_cb,
    pub file: *const ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub env: *mut *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_uint,
    pub stdio_count: ::core::ffi::c_int,
    pub stdio: *mut uv_stdio_container_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
}
pub type uv_process_options_t = uv_process_options_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type ApiDispatchWrapper =
    Option<unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
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
    pub b_wininfo: C2Rust_Unnamed_26,
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
    pub b_signcols: C2Rust_Unnamed_18,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_16,
    pub update_callbacks: C2Rust_Unnamed_15,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
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
pub struct C2Rust_Unnamed_16 {
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
    pub data: C2Rust_Unnamed_17,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
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
pub struct C2Rust_Unnamed_18 {
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
    pub sst_union: C2Rust_Unnamed_19,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
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
    pub data: C2Rust_Unnamed_20,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
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
    pub fc_fixvar: [C2Rust_Unnamed_21; 12],
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
pub struct C2Rust_Unnamed_21 {
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
    pub uh_next: C2Rust_Unnamed_25,
    pub uh_prev: C2Rust_Unnamed_24,
    pub uh_alt_next: C2Rust_Unnamed_23,
    pub uh_alt_prev: C2Rust_Unnamed_22,
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
pub union C2Rust_Unnamed_22 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
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
pub struct C2Rust_Unnamed_26 {
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
    pub type_0: C2Rust_Unnamed_27,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_27 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_27 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_27 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_27 = 0;
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
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_28,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
pub type Proc = proc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut dict_T,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: *mut MultiQueue,
}
pub type MultiQueue = multiqueue;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_29,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub events: *mut MultiQueue,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_29 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type uvarnumber_T = uint64_t;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DictWatcher {
    pub callback: Callback,
    pub key_pattern: *mut ::core::ffi::c_char,
    pub key_pattern_len: size_t,
    pub node: QUEUE,
    pub busy: bool,
    pub needs_free: bool,
}
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
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_30 = 76;
pub const HLF_PRE: C2Rust_Unnamed_30 = 75;
pub const HLF_OK: C2Rust_Unnamed_30 = 74;
pub const HLF_SO: C2Rust_Unnamed_30 = 73;
pub const HLF_SE: C2Rust_Unnamed_30 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_30 = 71;
pub const HLF_TS: C2Rust_Unnamed_30 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_30 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_30 = 68;
pub const HLF_CU: C2Rust_Unnamed_30 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_30 = 66;
pub const HLF_WBR: C2Rust_Unnamed_30 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_30 = 64;
pub const HLF_MSG: C2Rust_Unnamed_30 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_30 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_30 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_30 = 60;
pub const HLF_0: C2Rust_Unnamed_30 = 59;
pub const HLF_QFL: C2Rust_Unnamed_30 = 58;
pub const HLF_MC: C2Rust_Unnamed_30 = 57;
pub const HLF_CUL: C2Rust_Unnamed_30 = 56;
pub const HLF_CUC: C2Rust_Unnamed_30 = 55;
pub const HLF_TPF: C2Rust_Unnamed_30 = 54;
pub const HLF_TPS: C2Rust_Unnamed_30 = 53;
pub const HLF_TP: C2Rust_Unnamed_30 = 52;
pub const HLF_PBR: C2Rust_Unnamed_30 = 51;
pub const HLF_PST: C2Rust_Unnamed_30 = 50;
pub const HLF_PSB: C2Rust_Unnamed_30 = 49;
pub const HLF_PSX: C2Rust_Unnamed_30 = 48;
pub const HLF_PNX: C2Rust_Unnamed_30 = 47;
pub const HLF_PSK: C2Rust_Unnamed_30 = 46;
pub const HLF_PNK: C2Rust_Unnamed_30 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_30 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_30 = 43;
pub const HLF_PSI: C2Rust_Unnamed_30 = 42;
pub const HLF_PNI: C2Rust_Unnamed_30 = 41;
pub const HLF_SPL: C2Rust_Unnamed_30 = 40;
pub const HLF_SPR: C2Rust_Unnamed_30 = 39;
pub const HLF_SPC: C2Rust_Unnamed_30 = 38;
pub const HLF_SPB: C2Rust_Unnamed_30 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_30 = 36;
pub const HLF_SC: C2Rust_Unnamed_30 = 35;
pub const HLF_TXA: C2Rust_Unnamed_30 = 34;
pub const HLF_TXD: C2Rust_Unnamed_30 = 33;
pub const HLF_DED: C2Rust_Unnamed_30 = 32;
pub const HLF_CHD: C2Rust_Unnamed_30 = 31;
pub const HLF_ADD: C2Rust_Unnamed_30 = 30;
pub const HLF_FC: C2Rust_Unnamed_30 = 29;
pub const HLF_FL: C2Rust_Unnamed_30 = 28;
pub const HLF_WM: C2Rust_Unnamed_30 = 27;
pub const HLF_W: C2Rust_Unnamed_30 = 26;
pub const HLF_VNC: C2Rust_Unnamed_30 = 25;
pub const HLF_V: C2Rust_Unnamed_30 = 24;
pub const HLF_T: C2Rust_Unnamed_30 = 23;
pub const HLF_VSP: C2Rust_Unnamed_30 = 22;
pub const HLF_C: C2Rust_Unnamed_30 = 21;
pub const HLF_SNC: C2Rust_Unnamed_30 = 20;
pub const HLF_S: C2Rust_Unnamed_30 = 19;
pub const HLF_R: C2Rust_Unnamed_30 = 18;
pub const HLF_CLF: C2Rust_Unnamed_30 = 17;
pub const HLF_CLS: C2Rust_Unnamed_30 = 16;
pub const HLF_CLN: C2Rust_Unnamed_30 = 15;
pub const HLF_LNB: C2Rust_Unnamed_30 = 14;
pub const HLF_LNA: C2Rust_Unnamed_30 = 13;
pub const HLF_N: C2Rust_Unnamed_30 = 12;
pub const HLF_CM: C2Rust_Unnamed_30 = 11;
pub const HLF_M: C2Rust_Unnamed_30 = 10;
pub const HLF_LC: C2Rust_Unnamed_30 = 9;
pub const HLF_L: C2Rust_Unnamed_30 = 8;
pub const HLF_I: C2Rust_Unnamed_30 = 7;
pub const HLF_E: C2Rust_Unnamed_30 = 6;
pub const HLF_D: C2Rust_Unnamed_30 = 5;
pub const HLF_AT: C2Rust_Unnamed_30 = 4;
pub const HLF_TERM: C2Rust_Unnamed_30 = 3;
pub const HLF_EOB: C2Rust_Unnamed_30 = 2;
pub const HLF_8: C2Rust_Unnamed_30 = 1;
pub const HLF_NONE: C2Rust_Unnamed_30 = 0;
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type C2Rust_Unnamed_31 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_31 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_31 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_31 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_31 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_31 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_31 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_31 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_31 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_31 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_31 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_31 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_31 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_31 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_31 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_31 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_31 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_31 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_31 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_31 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_31 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_31 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_31 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_31 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_31 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_31 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_31 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_31 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_31 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_31 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_31 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_31 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_31 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_31 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_31 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_31 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_31 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_31 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_31 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_31 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_31 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_31 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_31 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_31 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_31 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_31 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_31 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_31 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_31 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_31 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_31 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_31 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_31 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_31 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_31 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_31 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_31 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_31 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_31 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_31 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_31 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_31 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_31 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_31 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_31 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_31 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_31 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_31 = -2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const REGSUB_BACKSLASH: C2Rust_Unnamed_32 = 4;
pub const REGSUB_MAGIC: C2Rust_Unnamed_32 = 2;
pub const REGSUB_COPY: C2Rust_Unnamed_32 = 1;
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
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub const kWinOptWrap: C2Rust_Unnamed_33 = 50;
pub const kWinOptWinhighlight: C2Rust_Unnamed_33 = 49;
pub const kWinOptWinfixwidth: C2Rust_Unnamed_33 = 48;
pub const kWinOptWinfixheight: C2Rust_Unnamed_33 = 47;
pub const kWinOptWinfixbuf: C2Rust_Unnamed_33 = 46;
pub const kWinOptWinblend: C2Rust_Unnamed_33 = 45;
pub const kWinOptWinbar: C2Rust_Unnamed_33 = 44;
pub const kWinOptVirtualedit: C2Rust_Unnamed_33 = 43;
pub const kWinOptStatusline: C2Rust_Unnamed_33 = 42;
pub const kWinOptStatuscolumn: C2Rust_Unnamed_33 = 41;
pub const kWinOptSpell: C2Rust_Unnamed_33 = 40;
pub const kWinOptSmoothscroll: C2Rust_Unnamed_33 = 39;
pub const kWinOptSigncolumn: C2Rust_Unnamed_33 = 38;
pub const kWinOptSidescrolloff: C2Rust_Unnamed_33 = 37;
pub const kWinOptShowbreak: C2Rust_Unnamed_33 = 36;
pub const kWinOptScrolloff: C2Rust_Unnamed_33 = 35;
pub const kWinOptScrollbind: C2Rust_Unnamed_33 = 34;
pub const kWinOptScroll: C2Rust_Unnamed_33 = 33;
pub const kWinOptRightleftcmd: C2Rust_Unnamed_33 = 32;
pub const kWinOptRightleft: C2Rust_Unnamed_33 = 31;
pub const kWinOptRelativenumber: C2Rust_Unnamed_33 = 30;
pub const kWinOptPreviewwindow: C2Rust_Unnamed_33 = 29;
pub const kWinOptNumberwidth: C2Rust_Unnamed_33 = 28;
pub const kWinOptNumber: C2Rust_Unnamed_33 = 27;
pub const kWinOptListchars: C2Rust_Unnamed_33 = 26;
pub const kWinOptList: C2Rust_Unnamed_33 = 25;
pub const kWinOptLinebreak: C2Rust_Unnamed_33 = 24;
pub const kWinOptLhistory: C2Rust_Unnamed_33 = 23;
pub const kWinOptFoldtext: C2Rust_Unnamed_33 = 22;
pub const kWinOptFoldnestmax: C2Rust_Unnamed_33 = 21;
pub const kWinOptFoldminlines: C2Rust_Unnamed_33 = 20;
pub const kWinOptFoldmethod: C2Rust_Unnamed_33 = 19;
pub const kWinOptFoldmarker: C2Rust_Unnamed_33 = 18;
pub const kWinOptFoldlevel: C2Rust_Unnamed_33 = 17;
pub const kWinOptFoldignore: C2Rust_Unnamed_33 = 16;
pub const kWinOptFoldexpr: C2Rust_Unnamed_33 = 15;
pub const kWinOptFoldenable: C2Rust_Unnamed_33 = 14;
pub const kWinOptFoldcolumn: C2Rust_Unnamed_33 = 13;
pub const kWinOptFillchars: C2Rust_Unnamed_33 = 12;
pub const kWinOptEventignorewin: C2Rust_Unnamed_33 = 11;
pub const kWinOptDiff: C2Rust_Unnamed_33 = 10;
pub const kWinOptCursorlineopt: C2Rust_Unnamed_33 = 9;
pub const kWinOptCursorline: C2Rust_Unnamed_33 = 8;
pub const kWinOptCursorcolumn: C2Rust_Unnamed_33 = 7;
pub const kWinOptCursorbind: C2Rust_Unnamed_33 = 6;
pub const kWinOptConceallevel: C2Rust_Unnamed_33 = 5;
pub const kWinOptConcealcursor: C2Rust_Unnamed_33 = 4;
pub const kWinOptColorcolumn: C2Rust_Unnamed_33 = 3;
pub const kWinOptBreakindentopt: C2Rust_Unnamed_33 = 2;
pub const kWinOptBreakindent: C2Rust_Unnamed_33 = 1;
pub const kWinOptArabic: C2Rust_Unnamed_33 = 0;
pub const kWinOptInvalid: C2Rust_Unnamed_33 = -1;
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
    pub cs_pend: C2Rust_Unnamed_34,
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
pub union C2Rust_Unnamed_34 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct time_watcher {
    pub uv: uv_timer_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: time_cb,
    pub close_cb: time_cb,
    pub events: *mut MultiQueue,
    pub blockable: bool,
}
pub type time_cb = Option<unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> ()>;
pub type TimeWatcher = time_watcher;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aucmdwin_T {
    pub auc_win: *mut win_T,
    pub auc_win_used: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_35 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallbackReader {
    pub cb: Callback,
    pub self_0: *mut dict_T,
    pub buffer: garray_T,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    pub type_0: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
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
pub type UIExtension = ::core::ffi::c_uint;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_background: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_37,
    pub is_rpc: bool,
    pub detach: bool,
    pub rpc: RpcState,
    pub term: *mut Terminal,
    pub on_data: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub exit_status: ::core::ffi::c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_36,
    pub info: Dict,
    pub client_type: ClientType,
}
pub type ClientType = ::core::ffi::c_int;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_36 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ChannelCallFrame,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
    pub result_mem: ArenaMem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_37 {
    pub proc: Proc,
    pub uv: LibuvProc,
    pub pty: PtyProc,
    pub socket: RStream,
    pub stdio: StdioPair,
    pub err: StderrState,
    pub internal: InternalState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PtyProc {
    pub proc: Proc,
    pub width: uint16_t,
    pub height: uint16_t,
    pub winsize: winsize,
    pub tty_fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_38 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_38 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_38 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_38 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_38 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_38 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_38 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_38 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_38 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
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
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_39 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_39 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_39 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_39 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_39 = 1;
pub const CONV_NONE: C2Rust_Unnamed_39 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lval_T {
    pub ll_name: *const ::core::ffi::c_char,
    pub ll_name_len: size_t,
    pub ll_exp_name: *mut ::core::ffi::c_char,
    pub ll_tv: *mut typval_T,
    pub ll_li: *mut listitem_T,
    pub ll_list: *mut list_T,
    pub ll_range: bool,
    pub ll_empty2: bool,
    pub ll_n1: ::core::ffi::c_int,
    pub ll_n2: ::core::ffi::c_int,
    pub ll_dict: *mut dict_T,
    pub ll_di: *mut dictitem_T,
    pub ll_newkey: *mut ::core::ffi::c_char,
    pub ll_blob: *mut blob_T,
}
pub type var_flavour_T = ::core::ffi::c_uint;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const GLV_READ_ONLY: C2Rust_Unnamed_40 = 16;
pub const GLV_NO_AUTOLOAD: C2Rust_Unnamed_40 = 4;
pub const GLV_QUIET: C2Rust_Unnamed_40 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timer_T {
    pub tw: TimeWatcher,
    pub timer_id: ::core::ffi::c_int,
    pub repeat_count: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub emsg_count: ::core::ffi::c_int,
    pub timeout: int64_t,
    pub stopped: bool,
    pub paused: bool,
    pub callback: Callback,
}
pub type exprtype_T = ::core::ffi::c_uint;
pub const EXPR_ISNOT: exprtype_T = 10;
pub const EXPR_IS: exprtype_T = 9;
pub const EXPR_NOMATCH: exprtype_T = 8;
pub const EXPR_MATCH: exprtype_T = 7;
pub const EXPR_SEQUAL: exprtype_T = 6;
pub const EXPR_SMALLER: exprtype_T = 5;
pub const EXPR_GEQUAL: exprtype_T = 4;
pub const EXPR_GREATER: exprtype_T = 3;
pub const EXPR_NEQUAL: exprtype_T = 2;
pub const EXPR_EQUAL: exprtype_T = 1;
pub const EXPR_UNKNOWN: exprtype_T = 0;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_41 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funcexe_T {
    pub fe_argv_func: ArgvFunc,
    pub fe_firstline: linenr_T,
    pub fe_lastline: linenr_T,
    pub fe_doesrange: *mut bool,
    pub fe_evaluate: bool,
    pub fe_partial: *mut partial_T,
    pub fe_selfdict: *mut dict_T,
    pub fe_basetv: *mut typval_T,
    pub fe_found_var: bool,
}
pub type ArgvFunc = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut typval_T,
        ::core::ffi::c_int,
        *mut ufunc_T,
    ) -> ::core::ffi::c_int,
>;
pub const KE_SNR: key_extra = 82;
pub const kGRegExprSrc: GRegFlags = 2;
pub const FSK_IN_STRING: C2Rust_Unnamed_44 = 4;
pub const FSK_KEYCODE: C2Rust_Unnamed_44 = 1;
pub const FSK_SIMPLIFY: C2Rust_Unnamed_44 = 8;
pub const OPT_LOCAL: C2Rust_Unnamed_45 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_45 = 1;
pub type funccal_entry_T = funccal_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccal_entry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut funccal_entry_T,
}
pub const GLV_STOP: glv_status_T = 2;
pub type glv_status_T = ::core::ffi::c_uint;
pub const GLV_OK: glv_status_T = 1;
pub const GLV_FAIL: glv_status_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct forinfo_T {
    pub fi_semicolon: ::core::ffi::c_int,
    pub fi_varcount: ::core::ffi::c_int,
    pub fi_lw: listwatch_T,
    pub fi_list: *mut list_T,
    pub fi_bi: ::core::ffi::c_int,
    pub fi_blob: *mut blob_T,
    pub fi_string: *mut ::core::ffi::c_char,
    pub fi_byte_idx: ::core::ffi::c_int,
}
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
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_43 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_43 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct caller_scope {
    pub script_ctx: sctx_T,
    pub es_entry: estack_T,
    pub autocmd_fname: *mut ::core::ffi::c_char,
    pub autocmd_match: *mut ::core::ffi::c_char,
    pub autocmd_fname_full: bool,
    pub autocmd_bufnr: ::core::ffi::c_int,
    pub funccalp: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_42,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_42 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
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
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_43 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_43 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_43 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_43 = 4;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const FSK_KEEP_X_KEY: C2Rust_Unnamed_44 = 2;
pub type GRegFlags = ::core::ffi::c_uint;
pub const kGRegList: GRegFlags = 4;
pub const kGRegNoExpr: GRegFlags = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_45 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_45 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_45 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_45 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_45 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_45 = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_1: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint64_t = Set_uint64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint64_t>(),
};
pub const MAP_INIT: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
#[inline(always)]
unsafe extern "C" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = '\u{8}' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const FF: ::core::ffi::c_int = '\u{c}' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0 || c == NUL;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(&raw mut channels, id) as *mut Channel;
}
pub const COPYID_INC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const COPYID_MASK: ::core::ffi::c_int = !(0x1 as ::core::ffi::c_int);
pub const FNE_INCL_BR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const DICT_MAXNEST: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
static mut e_missbrac: *const ::core::ffi::c_char =
    b"E111: Missing ']'\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_list_end: *const ::core::ffi::c_char =
    b"E697: Missing end of List ']': %s\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_cannot_slice_dictionary: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E719: Cannot slice a Dictionary\0",
    )
};
static mut e_cannot_index_special_variable: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E909: Cannot index a special variable\0",
    )
};
static mut e_nowhitespace: *const ::core::ffi::c_char =
    b"E274: No white space allowed before parenthesis\0".as_ptr() as *const ::core::ffi::c_char;
static mut e_cannot_index_a_funcref: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E695: Cannot index a Funcref\0",
    )
};
static mut e_variable_nested_too_deep_for_making_copy: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"E698: Variable nested too deep for making a copy\0",
    )
};
static mut e_string_list_or_blob_required: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E1098: String, List or Blob required\0",
    )
};
static mut e_expression_too_recursive_str: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E1169: Expression too recursive: %s\0",
    )
};
static mut e_dot_can_only_be_used_on_dictionary_str: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E1203: Dot can only be used on a dictionary: %s\0",
    )
};
static mut e_empty_function_name: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E1192: Empty function name\0")
};
static mut e_cannot_use_partial_here: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E1265: Cannot use a partial here\0",
    )
};
static mut namespace_char: *mut ::core::ffi::c_char =
    b"abglstvw\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut eval_lavars_used: *mut bool = ::core::ptr::null_mut::<bool>();
static mut echo_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut last_timer_id: uint64_t = 1 as uint64_t;
static mut timers: Map_uint64_t_ptr_t = MAP_INIT;
#[no_mangle]
pub unsafe extern "C" fn get_v_event(mut sve: *mut save_v_event_T) -> *mut dict_T {
    let mut v_event: *mut dict_T = get_vim_var_dict(VV_EVENT);
    if (*v_event).dv_hashtab.ht_used > 0 as size_t {
        (*sve).sve_did_save = true_0 != 0;
        (*sve).sve_hashtab = (*v_event).dv_hashtab;
        hash_init(&raw mut (*v_event).dv_hashtab);
    } else {
        (*sve).sve_did_save = false_0 != 0;
    }
    return v_event;
}
#[no_mangle]
pub unsafe extern "C" fn restore_v_event(mut v_event: *mut dict_T, mut sve: *mut save_v_event_T) {
    tv_dict_free_contents(v_event);
    if (*sve).sve_did_save {
        (*v_event).dv_hashtab = (*sve).sve_hashtab;
    } else {
        hash_init(&raw mut (*v_event).dv_hashtab);
    };
}
#[no_mangle]
pub unsafe extern "C" fn num_divide(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    let mut result: varnumber_T = 0;
    if n2 == 0 as varnumber_T {
        if n1 == 0 as varnumber_T {
            result = VARNUMBER_MIN as varnumber_T;
        } else if n1 < 0 as varnumber_T {
            result = -VARNUMBER_MAX as varnumber_T;
        } else {
            result = VARNUMBER_MAX as varnumber_T;
        }
    } else if n1 == VARNUMBER_MIN as varnumber_T && n2 == -1 as varnumber_T {
        result = VARNUMBER_MAX as varnumber_T;
    } else {
        result = n1 / n2;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn num_modulus(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    return if n2 == 0 as varnumber_T {
        0 as varnumber_T
    } else {
        n1 % n2
    };
}
#[no_mangle]
pub unsafe extern "C" fn eval_init() {
    evalvars_init();
    func_init();
}
#[no_mangle]
pub unsafe extern "C" fn fill_evalarg_from_eap(
    mut evalarg: *mut evalarg_T,
    mut eap: *mut exarg_T,
    mut skip: bool,
) {
    *evalarg = evalarg_T {
        eval_flags: if skip as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            EVAL_EVALUATE as ::core::ffi::c_int
        },
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if eap.is_null() {
        return;
    }
    if sourcing_a_script(eap) != 0 {
        (*evalarg).eval_getline = (*eap).ea_getline;
        (*evalarg).eval_cookie = (*eap).cookie;
    }
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_bool(
    mut arg: *mut ::core::ffi::c_char,
    mut error: *mut bool,
    mut eap: *mut exarg_T,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: bool = false_0 != 0;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        emsg_skip += 1;
    }
    let mut r: ::core::ffi::c_int = if use_simple_function as ::core::ffi::c_int != 0 {
        eval0_simple_funccal(arg, &raw mut tv, eap, &raw mut evalarg)
    } else {
        eval0(arg, &raw mut tv, eap, &raw mut evalarg)
    };
    if r == FAIL {
        *error = true_0 != 0;
    } else {
        *error = false_0 != 0;
        if !skip {
            retval = tv_get_number_chk(&raw mut tv, error) != 0 as varnumber_T;
            tv_clear(&raw mut tv);
        }
    }
    if skip {
        emsg_skip -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}
unsafe extern "C" fn eval1_emsg(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
) -> ::core::ffi::c_int {
    let start: *const ::core::ffi::c_char = *arg;
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let called_emsg_before: ::core::ffi::c_int = called_emsg;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let ret: ::core::ffi::c_int = eval1(arg, rettv, &raw mut evalarg);
    if ret == FAIL {
        if !aborting() && did_emsg == did_emsg_before && called_emsg == called_emsg_before {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                start,
            );
        }
    }
    clear_evalarg(&raw mut evalarg, eap);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn eval_expr_valid_arg(tv: *const typval_T) -> bool {
    return (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*tv).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || !(*tv).vval.v_string.is_null()
                && *(*tv).vval.v_string as ::core::ffi::c_int != NUL);
}
unsafe extern "C" fn eval_expr_partial(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let partial: *mut partial_T = (*expr).vval.v_partial;
    if partial.is_null() {
        return FAIL;
    }
    let s: *const ::core::ffi::c_char = partial_name(partial);
    if s.is_null() || *s as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    if call_func(
        s,
        -1 as ::core::ffi::c_int,
        rettv,
        argc,
        argv,
        &raw mut funcexe,
    ) == FAIL
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn eval_expr_func(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let s: *const ::core::ffi::c_char = if (*expr).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*expr).vval.v_string as *const ::core::ffi::c_char
    } else {
        tv_get_string_buf_chk(expr, &raw mut buf as *mut ::core::ffi::c_char)
    };
    if s.is_null() || *s as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    if call_func(
        s,
        -1 as ::core::ffi::c_int,
        rettv,
        argc,
        argv,
        &raw mut funcexe,
    ) == FAIL
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn eval_expr_string(
    mut expr: *const typval_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut s: *mut ::core::ffi::c_char =
        tv_get_string_buf_chk(expr, &raw mut buf as *mut ::core::ffi::c_char)
            as *mut ::core::ffi::c_char;
    if s.is_null() {
        return FAIL;
    }
    s = skipwhite(s);
    if eval1_emsg(&raw mut s, rettv, ::core::ptr::null_mut::<exarg_T>()) == FAIL {
        return FAIL;
    }
    if *skipwhite(s) as ::core::ffi::c_int != NUL {
        tv_clear(rettv);
        semsg(
            gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
            s,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn eval_expr_typval(
    mut expr: *const typval_T,
    mut want_func: bool,
    mut argv: *mut typval_T,
    mut argc: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    if (*expr).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return eval_expr_partial(expr, argv, argc, rettv);
    }
    if (*expr).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || want_func as ::core::ffi::c_int != 0
    {
        return eval_expr_func(expr, argv, argc, rettv);
    }
    return eval_expr_string(expr, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn eval_expr_to_bool(
    mut expr: *const typval_T,
    mut error: *mut bool,
) -> bool {
    let mut argv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv,
        0 as ::core::ffi::c_int,
        &raw mut rettv,
    ) == FAIL
    {
        *error = true_0 != 0;
        return false_0 != 0;
    }
    let res: bool = tv_get_number_chk(&raw mut rettv, error) != 0 as varnumber_T;
    tv_clear(&raw mut rettv);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_string_skip(
    mut arg: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    skip: bool,
) -> *mut ::core::ffi::c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        emsg_skip += 1;
    }
    if eval0(arg, &raw mut tv, eap, &raw mut evalarg) == FAIL || skip as ::core::ffi::c_int != 0 {
        retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        retval = xstrdup(tv_get_string(&raw mut tv));
        tv_clear(&raw mut tv);
    }
    if skip {
        emsg_skip -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn skip_expr(
    mut pp: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let save_flags: ::core::ffi::c_int = if evalarg.is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*evalarg).eval_flags
    };
    if !evalarg.is_null() {
        (*evalarg).eval_flags &= !(EVAL_EVALUATE as ::core::ffi::c_int);
    }
    *pp = skipwhite(*pp);
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut res: ::core::ffi::c_int =
        eval1(pp, &raw mut rettv, ::core::ptr::null_mut::<evalarg_T>());
    if !evalarg.is_null() {
        (*evalarg).eval_flags = save_flags;
    }
    return res;
}
unsafe extern "C" fn typval2string(
    mut tv: *mut typval_T,
    mut join_list: bool,
) -> *mut ::core::ffi::c_char {
    if join_list as ::core::ffi::c_int != 0
        && (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        if !(*tv).vval.v_list.is_null() {
            tv_list_join(
                &raw mut ga,
                (*tv).vval.v_list,
                b"\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if tv_list_len((*tv).vval.v_list) > 0 as ::core::ffi::c_int {
                ga_append(&raw mut ga, NL as uint8_t);
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut ::core::ffi::c_char;
    } else if (*tv).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return encode_tv2string(tv, ::core::ptr::null_mut::<size_t>());
    }
    return xstrdup(tv_get_string(tv));
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_string_eap(
    mut arg: *mut ::core::ffi::c_char,
    join_list: bool,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut ::core::ffi::c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: ::core::ffi::c_int = if use_simple_function as ::core::ffi::c_int != 0 {
        eval0_simple_funccal(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    } else {
        eval0(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    };
    if r == FAIL {
        retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        retval = typval2string(&raw mut tv, join_list);
        tv_clear(&raw mut tv);
    }
    clear_evalarg(&raw mut evalarg, ::core::ptr::null_mut::<exarg_T>());
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_string(
    mut arg: *mut ::core::ffi::c_char,
    join_list: bool,
    use_simple_function: bool,
) -> *mut ::core::ffi::c_char {
    return eval_to_string_eap(
        arg,
        join_list,
        ::core::ptr::null_mut::<exarg_T>(),
        use_simple_function,
    );
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_string_safe(
    mut arg: *mut ::core::ffi::c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;
    retval = eval_to_string(arg, false_0 != 0, use_simple_function);
    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    restore_funccal();
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn eval_to_number(
    mut expr: *mut ::core::ffi::c_char,
    use_simple_function: bool,
) -> varnumber_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    let mut p: *mut ::core::ffi::c_char = skipwhite(expr);
    let mut r: ::core::ffi::c_int = NOTDONE;
    emsg_off += 1;
    if use_simple_function {
        r = may_call_simple_func(expr, &raw mut rettv);
    }
    if r == NOTDONE {
        r = eval1(&raw mut p, &raw mut rettv, &raw mut EVALARG_EVALUATE);
    }
    if r == FAIL {
        retval = -1 as varnumber_T;
    } else {
        retval = tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
        tv_clear(&raw mut rettv);
    }
    emsg_off -= 1;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn eval_expr(
    mut arg: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) -> *mut typval_T {
    return eval_expr_ext(arg, eap, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn eval_expr_ext(
    mut arg: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut typval_T {
    let mut tv: *mut typval_T = xmalloc(::core::mem::size_of::<typval_T>()) as *mut typval_T;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: ::core::ffi::c_int = NOTDONE;
    if use_simple_function {
        r = eval0_simple_funccal(arg, tv, eap, &raw mut evalarg);
    }
    if r == NOTDONE {
        r = eval0(arg, tv, eap, &raw mut evalarg);
    }
    if r == FAIL {
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut tv as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return tv;
}
#[no_mangle]
pub unsafe extern "C" fn call_vim_function(
    mut func: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut typval_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut ret: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = strlen(func) as ::core::ffi::c_int;
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    '_fail: {
        if len >= 6 as ::core::ffi::c_int
            && memcmp(
                func as *const ::core::ffi::c_void,
                b"v:lua.\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                6 as size_t,
            ) == 0
        {
            func = func.offset(6 as ::core::ffi::c_int as isize);
            len = check_luafunc_name(func, false_0 != 0);
            if len == 0 as ::core::ffi::c_int {
                ret = FAIL;
                break '_fail;
            } else {
                pt = get_vim_var_partial(VV_LUA);
            }
        }
        (*rettv).v_type = VAR_UNKNOWN;
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = pt;
        ret = call_func(func, len, rettv, argc, argv, &raw mut funcexe);
    }
    if ret == FAIL {
        tv_clear(rettv);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn call_func_retstr(
    func: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut typval_T,
) -> *mut ::core::ffi::c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    let retval: *mut ::core::ffi::c_char = xstrdup(tv_get_string(&raw mut rettv));
    tv_clear(&raw mut rettv);
    return retval as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn call_func_retlist(
    mut func: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut typval_T,
) -> *mut ::core::ffi::c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    if rettv.v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(&raw mut rettv);
        return NULL_0;
    }
    return rettv.vval.v_list as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn eval_foldexpr(
    mut wp: *mut win_T,
    mut cp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let saved_sctx: sctx_T = current_sctx;
    let use_sandbox: bool =
        was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL as ::core::ffi::c_int) != 0;
    let mut arg: *mut ::core::ffi::c_char = skipwhite((*wp).w_onebuf_opt.wo_fde);
    current_sctx = (*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as ::core::ffi::c_int as usize];
    emsg_off += 1;
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;
    *cp = NUL;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut EVALARG_EVALUATE,
    ) == FAIL
    {
        retval = 0 as varnumber_T;
    } else {
        if tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            retval = tv.vval.v_number;
        } else if tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || tv.vval.v_string.is_null()
        {
            retval = 0 as varnumber_T;
        } else {
            let mut s: *mut ::core::ffi::c_char = tv.vval.v_string;
            if *s as ::core::ffi::c_int != NUL
                && !ascii_isdigit(*s as ::core::ffi::c_int)
                && *s as ::core::ffi::c_int != '-' as ::core::ffi::c_int
            {
                let c2rust_fresh10 = s;
                s = s.offset(1);
                *cp = *c2rust_fresh10 as uint8_t as ::core::ffi::c_int;
            }
            retval = atol(s) as varnumber_T;
        }
        tv_clear(&raw mut tv);
    }
    emsg_off -= 1;
    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    clear_evalarg(
        &raw mut EVALARG_EVALUATE,
        ::core::ptr::null_mut::<exarg_T>(),
    );
    current_sctx = saved_sctx;
    return retval as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn eval_foldtext(mut wp: *mut win_T) -> Object {
    let use_sandbox: bool =
        was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL as ::core::ffi::c_int) != 0;
    let mut arg: *mut ::core::ffi::c_char = (*wp).w_onebuf_opt.wo_fdt;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    };
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut EVALARG_EVALUATE,
    ) == FAIL
    {
        retval = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                },
            },
        };
    } else {
        if tv.v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            retval = vim_to_object(&raw mut tv, ::core::ptr::null_mut::<Arena>(), false_0 != 0);
        } else {
            retval = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_to_string(tv_get_string(&raw mut tv)),
                },
            };
        }
        tv_clear(&raw mut tv);
    }
    clear_evalarg(
        &raw mut EVALARG_EVALUATE,
        ::core::ptr::null_mut::<exarg_T>(),
    );
    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    restore_funccal();
    return retval;
}
unsafe extern "C" fn to_name_end(
    mut arg: *const ::core::ffi::c_char,
    mut use_namespace: bool,
) -> *const ::core::ffi::c_char {
    if !eval_isnamec1(*arg as ::core::ffi::c_int) {
        return arg;
    }
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = arg.offset(1 as ::core::ffi::c_int as isize);
    while *p as ::core::ffi::c_int != NUL
        && eval_isnamec(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && (p != arg.offset(1 as ::core::ffi::c_int as isize)
                || !use_namespace
                || vim_strchr(
                    b"bgstvw\0".as_ptr() as *const ::core::ffi::c_char,
                    *arg as ::core::ffi::c_int,
                )
                .is_null())
        {
            break;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return p;
}
unsafe extern "C" fn get_lval_dict_item(
    mut lp: *mut lval_T,
    mut name: *mut ::core::ffi::c_char,
    mut key: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut key_end: *mut *mut ::core::ffi::c_char,
    mut var1: *mut typval_T,
    mut flags: ::core::ffi::c_int,
    mut unlet: bool,
    mut rettv: *mut typval_T,
) -> glv_status_T {
    let mut quiet: bool = flags & GLV_QUIET as ::core::ffi::c_int != 0;
    let mut p: *mut ::core::ffi::c_char = *key_end;
    if len == -1 as ::core::ffi::c_int {
        key = tv_get_string(var1) as *mut ::core::ffi::c_char;
    }
    (*lp).ll_list = ::core::ptr::null_mut::<list_T>();
    if (*(*lp).ll_tv).vval.v_dict.is_null() {
        (*(*lp).ll_tv).vval.v_dict = tv_dict_alloc();
        (*(*(*lp).ll_tv).vval.v_dict).dv_refcount += 1;
    }
    (*lp).ll_dict = (*(*lp).ll_tv).vval.v_dict;
    (*lp).ll_di = tv_dict_find((*lp).ll_dict, key, len as ptrdiff_t);
    if !rettv.is_null()
        && (*(*lp).ll_dict).dv_scope as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint
    {
        let mut prevval: ::core::ffi::c_char = 0;
        if len != -1 as ::core::ffi::c_int {
            prevval = *key.offset(len as isize);
            *key.offset(len as isize) = NUL as ::core::ffi::c_char;
        } else {
            prevval = 0 as ::core::ffi::c_char;
        }
        let mut wrong: bool = (*(*lp).ll_dict).dv_scope as ::core::ffi::c_uint
            == VAR_DEF_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_is_func(*rettv) as ::core::ffi::c_int != 0
            && var_wrong_func_name(key, (*lp).ll_di.is_null()) as ::core::ffi::c_int != 0
            || !valid_varname(key);
        if len != -1 as ::core::ffi::c_int {
            *key.offset(len as isize) = prevval;
        }
        if wrong {
            return GLV_FAIL;
        }
    }
    if !(*lp).ll_di.is_null()
        && tv_is_luafunc(&raw mut (*(*lp).ll_di).di_tv) as ::core::ffi::c_int != 0
        && len == -1 as ::core::ffi::c_int
        && rettv.is_null()
    {
        semsg(
            &raw const e_illvar as *const ::core::ffi::c_char,
            b"v:['lua']\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return GLV_FAIL;
    }
    if (*lp).ll_di.is_null() {
        if (*lp).ll_dict == get_vimvar_dict()
            || &raw mut (*(*lp).ll_dict).dv_hashtab == get_funccal_args_ht()
        {
            semsg(
                gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                name,
            );
            return GLV_FAIL;
        }
        if *p as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            || unlet as ::core::ffi::c_int != 0
        {
            if !quiet {
                semsg(
                    gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                    key,
                );
            }
            return GLV_FAIL;
        }
        if len == -1 as ::core::ffi::c_int {
            (*lp).ll_newkey = xstrdup(key);
        } else {
            (*lp).ll_newkey = xmemdupz(key as *const ::core::ffi::c_void, len as size_t)
                as *mut ::core::ffi::c_char;
        }
        *key_end = p;
        return GLV_STOP;
    } else if flags & GLV_READ_ONLY as ::core::ffi::c_int == 0
        && (var_check_ro(
            (*(*lp).ll_di).di_flags as ::core::ffi::c_int,
            name,
            p.offset_from(name) as size_t,
        ) as ::core::ffi::c_int
            != 0
            || var_check_lock(
                (*(*lp).ll_di).di_flags as ::core::ffi::c_int,
                name,
                p.offset_from(name) as size_t,
            ) as ::core::ffi::c_int
                != 0)
    {
        return GLV_FAIL;
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_di).di_tv;
    return GLV_OK;
}
unsafe extern "C" fn get_lval_blob(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    let bloblen: ::core::ffi::c_int = tv_blob_len((*(*lp).ll_tv).vval.v_blob);
    if empty1 {
        (*lp).ll_n1 = 0 as ::core::ffi::c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as ::core::ffi::c_int;
    }
    if tv_blob_check_index(bloblen, (*lp).ll_n1 as varnumber_T, quiet) == FAIL {
        return FAIL;
    }
    if (*lp).ll_range as ::core::ffi::c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as ::core::ffi::c_int;
        if tv_blob_check_range(
            bloblen,
            (*lp).ll_n1 as varnumber_T,
            (*lp).ll_n2 as varnumber_T,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_blob = (*(*lp).ll_tv).vval.v_blob;
    (*lp).ll_tv = ::core::ptr::null_mut::<typval_T>();
    return OK;
}
unsafe extern "C" fn get_lval_list(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut _flags: ::core::ffi::c_int,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    if empty1 {
        (*lp).ll_n1 = 0 as ::core::ffi::c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as ::core::ffi::c_int;
    }
    (*lp).ll_dict = ::core::ptr::null_mut::<dict_T>();
    (*lp).ll_list = (*(*lp).ll_tv).vval.v_list;
    (*lp).ll_li = tv_list_check_range_index_one((*lp).ll_list, &raw mut (*lp).ll_n1, quiet);
    if (*lp).ll_li.is_null() {
        return FAIL;
    }
    if (*lp).ll_range as ::core::ffi::c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as ::core::ffi::c_int;
        if tv_list_check_range_index_two(
            (*lp).ll_list,
            &raw mut (*lp).ll_n1,
            (*lp).ll_li,
            &raw mut (*lp).ll_n2,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_li).li_tv;
    return OK;
}
unsafe extern "C" fn get_lval_subscript(
    mut lp: *mut lval_T,
    mut p: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut _ht: *mut hashtab_T,
    mut _v: *mut dictitem_T,
    mut unlet: bool,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut quiet: bool = flags & GLV_QUIET as ::core::ffi::c_int != 0;
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var1.v_type = VAR_UNKNOWN;
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var2.v_type = VAR_UNKNOWN;
    let mut empty1: bool = false_0 != 0;
    let mut rc: ::core::ffi::c_int = FAIL;
    '_done: {
        while *p as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '=' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '.' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                    != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !quiet {
                    semsg(
                        gettext(
                            &raw const e_dot_can_only_be_used_on_dictionary_str
                                as *const ::core::ffi::c_char,
                        ),
                        name,
                    );
                }
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                    != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                    != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !quiet {
                    emsg(gettext(
                        b"E689: Can only index a List, Dictionary or Blob\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                }
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*lp).ll_tv).vval.v_list.is_null()
            {
                tv_list_alloc_ret(
                    (*lp).ll_tv,
                    kListLenUnknown as ::core::ffi::c_int as ptrdiff_t,
                );
            } else if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*lp).ll_tv).vval.v_blob.is_null()
            {
                tv_blob_alloc_ret((*lp).ll_tv);
            }
            if (*lp).ll_range {
                if !quiet {
                    emsg(gettext(
                        b"E708: [:] must come last\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                break '_done;
            } else {
                let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut key: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    key = p.offset(1 as ::core::ffi::c_int as isize);
                    len = 0 as ::core::ffi::c_int;
                    while *key.offset(len as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *key.offset(len as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *key.offset(len as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *key.offset(len as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(*key.offset(len as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                        || *key.offset(len as isize) as ::core::ffi::c_int
                            == '_' as ::core::ffi::c_int
                    {
                        len += 1;
                    }
                    if len == 0 as ::core::ffi::c_int {
                        if !quiet {
                            emsg(gettext(b"E713: Cannot use empty key after .\0".as_ptr()
                                as *const ::core::ffi::c_char));
                        }
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    p = key.offset(len as isize);
                } else {
                    p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                    if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                        empty1 = true_0 != 0;
                    } else {
                        empty1 = false_0 != 0;
                        if eval1(&raw mut p, &raw mut var1, &raw mut EVALARG_EVALUATE) == FAIL {
                            break '_done;
                        }
                        if !tv_check_str(&raw mut var1) {
                            break '_done;
                        }
                        p = skipwhite(p);
                    }
                    if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                        if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if !quiet {
                                emsg(gettext(
                                    &raw const e_cannot_slice_dictionary
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_done;
                        } else if !rettv.is_null()
                            && !((*rettv).v_type as ::core::ffi::c_uint
                                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !(*rettv).vval.v_list.is_null())
                            && !((*rettv).v_type as ::core::ffi::c_uint
                                == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !(*rettv).vval.v_blob.is_null())
                        {
                            if !quiet {
                                emsg(gettext(
                                    b"E709: [:] requires a List or Blob value\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_done;
                        } else {
                            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                            if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                                (*lp).ll_empty2 = true_0 != 0;
                            } else {
                                (*lp).ll_empty2 = false_0 != 0;
                                if eval1(&raw mut p, &raw mut var2, &raw mut EVALARG_EVALUATE)
                                    == FAIL
                                {
                                    break '_done;
                                }
                                if !tv_check_str(&raw mut var2) {
                                    break '_done;
                                }
                            }
                            (*lp).ll_range = true_0 != 0;
                        }
                    } else {
                        (*lp).ll_range = false_0 != 0;
                    }
                    if *p as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
                        if !quiet {
                            emsg(gettext(e_missbrac));
                        }
                        break '_done;
                    } else {
                        p = p.offset(1);
                    }
                }
                if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut glv_status: glv_status_T = get_lval_dict_item(
                        lp,
                        name,
                        key,
                        len,
                        &raw mut p,
                        &raw mut var1,
                        flags,
                        unlet,
                        rettv,
                    );
                    if glv_status as ::core::ffi::c_uint
                        == GLV_FAIL as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break '_done;
                    }
                    if glv_status as ::core::ffi::c_uint
                        == GLV_STOP as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                } else if (*(*lp).ll_tv).v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if get_lval_blob(lp, &raw mut var1, &raw mut var2, empty1, quiet) == FAIL {
                        break '_done;
                    } else {
                        break;
                    }
                } else if get_lval_list(lp, &raw mut var1, &raw mut var2, empty1, flags, quiet)
                    == FAIL
                {
                    break '_done;
                }
                tv_clear(&raw mut var1);
                tv_clear(&raw mut var2);
                var1.v_type = VAR_UNKNOWN;
                var2.v_type = VAR_UNKNOWN;
            }
        }
        rc = OK;
    }
    tv_clear(&raw mut var1);
    tv_clear(&raw mut var2);
    return if rc == OK {
        p
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_lval(
    name: *mut ::core::ffi::c_char,
    rettv: *mut typval_T,
    lp: *mut lval_T,
    unlet: bool,
    skip: bool,
    flags: ::core::ffi::c_int,
    fne_flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut quiet: ::core::ffi::c_int = flags & GLV_QUIET as ::core::ffi::c_int;
    memset(
        lp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<lval_T>(),
    );
    if skip {
        (*lp).ll_name = name;
        return find_name_end(
            name,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            FNE_INCL_BR | fne_flags,
        ) as *mut ::core::ffi::c_char;
    }
    let mut expr_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expr_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = find_name_end(
        name,
        &raw mut expr_start as *mut *const ::core::ffi::c_char,
        &raw mut expr_end as *mut *const ::core::ffi::c_char,
        fne_flags,
    ) as *mut ::core::ffi::c_char;
    if !expr_start.is_null() {
        if unlet as ::core::ffi::c_int != 0
            && !ascii_iswhite(*p as ::core::ffi::c_int)
            && ends_excmd(*p as ::core::ffi::c_int) == 0
            && *p as ::core::ffi::c_int != '[' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                p,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*lp).ll_exp_name = make_expanded_name(name, expr_start, expr_end, p);
        (*lp).ll_name = (*lp).ll_exp_name;
        if (*lp).ll_exp_name.is_null() {
            if !aborting() && quiet == 0 {
                emsg_severe = true_0 != 0;
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    name,
                );
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*lp).ll_name_len = 0 as size_t;
        } else {
            (*lp).ll_name_len = strlen((*lp).ll_name);
        }
    } else {
        (*lp).ll_name = name;
        (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    }
    if *p as ::core::ffi::c_int != '[' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        || (*lp).ll_name.is_null()
    {
        return p;
    }
    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut v: *mut dictitem_T = find_var(
        (*lp).ll_name,
        (*lp).ll_name_len,
        if flags & GLV_READ_ONLY as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<*mut hashtab_T>()
        } else {
            &raw mut ht
        },
        flags & GLV_NO_AUTOLOAD as ::core::ffi::c_int,
    );
    if v.is_null() && quiet == 0 {
        semsg(
            gettext(b"E121: Undefined variable: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            (*lp).ll_name_len as ::core::ffi::c_int,
            (*lp).ll_name,
        );
    }
    if v.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*lp).ll_tv = &raw mut (*v).di_tv;
    if tv_is_luafunc((*lp).ll_tv) {
        return p;
    }
    p = get_lval_subscript(lp, p, name, rettv, ht, v, unlet, flags);
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn clear_lval(mut lp: *mut lval_T) {
    xfree((*lp).ll_exp_name as *mut ::core::ffi::c_void);
    xfree((*lp).ll_newkey as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn set_var_lval(
    mut lp: *mut lval_T,
    mut endp: *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut copy: bool,
    is_const: bool,
    mut op: *const ::core::ffi::c_char,
) {
    let mut cc: ::core::ffi::c_int = 0;
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*lp).ll_tv.is_null() {
        cc = *endp as uint8_t as ::core::ffi::c_int;
        *endp = NUL as ::core::ffi::c_char;
        if !(*lp).ll_blob.is_null() {
            if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                semsg(
                    gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
                    op,
                );
                return;
            }
            if value_check_lock(
                (*(*lp).ll_blob).bv_lock,
                (*lp).ll_name,
                TV_CSTRING as size_t,
            ) {
                return;
            }
            if (*lp).ll_range as ::core::ffi::c_int != 0
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*lp).ll_empty2 {
                    (*lp).ll_n2 = tv_blob_len((*lp).ll_blob) - 1 as ::core::ffi::c_int;
                }
                if tv_blob_set_range(
                    (*lp).ll_blob,
                    (*lp).ll_n1 as varnumber_T,
                    (*lp).ll_n2 as varnumber_T,
                    rettv,
                ) == FAIL
                {
                    return;
                }
            } else {
                let mut error: bool = false_0 != 0;
                let val: varnumber_T = tv_get_number_chk(rettv, &raw mut error);
                if !error {
                    if val < 0 as varnumber_T || val > 255 as varnumber_T {
                        semsg(
                            gettext(
                                &raw const e_invalid_value_for_blob_nr
                                    as *const ::core::ffi::c_char,
                            ),
                            val,
                        );
                    } else {
                        tv_blob_set_append((*lp).ll_blob, (*lp).ll_n1, val as uint8_t);
                    }
                }
            }
        } else if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if is_const {
                emsg(gettext(
                    &raw const e_cannot_mod as *const ::core::ffi::c_char,
                ));
                *endp = cc as ::core::ffi::c_char;
                return;
            }
            di = ::core::ptr::null_mut::<dictitem_T>();
            if eval_variable(
                (*lp).ll_name,
                (*lp).ll_name_len as ::core::ffi::c_int,
                &raw mut tv,
                &raw mut di,
                true_0 != 0,
                false_0 != 0,
            ) == OK
            {
                if (di.is_null()
                    || !var_check_ro(
                        (*di).di_flags as ::core::ffi::c_int,
                        (*lp).ll_name,
                        TV_CSTRING as size_t,
                    ) && !tv_check_lock(
                        &raw mut (*di).di_tv,
                        (*lp).ll_name,
                        TV_CSTRING as size_t,
                    ))
                    && eexe_mod_op(&raw mut tv, rettv, op) == OK
                {
                    set_var((*lp).ll_name, (*lp).ll_name_len, &raw mut tv, false_0 != 0);
                }
                tv_clear(&raw mut tv);
            }
        } else {
            set_var_const((*lp).ll_name, (*lp).ll_name_len, rettv, copy, is_const);
        }
        *endp = cc as ::core::ffi::c_char;
    } else if !value_check_lock(
        (if (*lp).ll_newkey.is_null() {
            (*(*lp).ll_tv).v_lock as ::core::ffi::c_uint
        } else {
            (*(*(*lp).ll_tv).vval.v_dict).dv_lock as ::core::ffi::c_uint
        }) as VarLockStatus,
        (*lp).ll_name,
        TV_CSTRING as size_t,
    ) {
        if (*lp).ll_range {
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a range\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            tv_list_assign_range(
                (*lp).ll_list,
                (*rettv).vval.v_list,
                (*lp).ll_n1,
                (*lp).ll_n2,
                (*lp).ll_empty2,
                op,
                (*lp).ll_name,
            );
        } else {
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut dict: *mut dict_T = (*lp).ll_dict;
            let mut watched: bool = tv_dict_is_watched(dict);
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a list or dict\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            '_notify: {
                if !(*lp).ll_newkey.is_null() {
                    if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                        semsg(
                            gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                            (*lp).ll_newkey,
                        );
                        return;
                    }
                    if tv_dict_wrong_func_name((*(*lp).ll_tv).vval.v_dict, rettv, (*lp).ll_newkey)
                        != 0
                    {
                        return;
                    }
                    di = tv_dict_item_alloc((*lp).ll_newkey);
                    if tv_dict_add((*(*lp).ll_tv).vval.v_dict, di) == FAIL {
                        xfree(di as *mut ::core::ffi::c_void);
                        return;
                    }
                    (*lp).ll_tv = &raw mut (*di).di_tv;
                } else {
                    if watched {
                        tv_copy((*lp).ll_tv, &raw mut oldtv);
                    }
                    if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                        eexe_mod_op((*lp).ll_tv, rettv, op);
                        break '_notify;
                    } else {
                        tv_clear((*lp).ll_tv);
                    }
                }
                if copy {
                    tv_copy(rettv, (*lp).ll_tv);
                } else {
                    *(*lp).ll_tv = *rettv;
                    (*(*lp).ll_tv).v_lock = VAR_UNLOCKED;
                    tv_init(rettv);
                }
            }
            if watched {
                if oldtv.v_type as ::core::ffi::c_uint
                    == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    '_c2rust_label: {
                        if !(*lp).ll_newkey.is_null() {
                        } else {
                            __assert_fail(
                                b"lp->ll_newkey != NULL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                1418 as ::core::ffi::c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        (*lp).ll_newkey,
                        (*lp).ll_tv,
                        ::core::ptr::null_mut::<typval_T>(),
                    );
                } else {
                    let mut di_: *mut dictitem_T = (*lp).ll_di;
                    '_c2rust_label_0: {
                        if !(&raw mut (*di_).di_key as *mut ::core::ffi::c_char).is_null() {
                        } else {
                            __assert_fail(
                                b"di_->di_key != NULL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                1422 as ::core::ffi::c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        &raw mut (*di_).di_key as *mut ::core::ffi::c_char,
                        (*lp).ll_tv,
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn eval_for_line(
    mut arg: *const ::core::ffi::c_char,
    mut errp: *mut bool,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> *mut ::core::ffi::c_void {
    let mut fi: *mut forinfo_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<forinfo_T>()) as *mut forinfo_T;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let skip: bool = (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int == 0;
    *errp = true_0 != 0;
    let mut expr: *const ::core::ffi::c_char = skip_var_list(
        arg,
        &raw mut (*fi).fi_varcount,
        &raw mut (*fi).fi_semicolon,
        false_0 != 0,
    );
    if expr.is_null() {
        return fi as *mut ::core::ffi::c_void;
    }
    expr = skipwhite(expr);
    if *expr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 'i' as ::core::ffi::c_int
        || *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 'n' as ::core::ffi::c_int
        || !(*expr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || ascii_iswhite(*expr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0)
    {
        emsg(gettext(
            b"E690: Missing \"in\" after :for\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return fi as *mut ::core::ffi::c_void;
    }
    if skip {
        emsg_skip += 1;
    }
    expr = skipwhite(expr.offset(2 as ::core::ffi::c_int as isize));
    if eval0(expr as *mut ::core::ffi::c_char, &raw mut tv, eap, evalarg) == OK {
        *errp = false_0 != 0;
        if !skip {
            if tv.v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                l = tv.vval.v_list;
                if l.is_null() {
                    tv_clear(&raw mut tv);
                } else {
                    (*fi).fi_list = l;
                    tv_list_watch_add(l, &raw mut (*fi).fi_lw);
                    (*fi).fi_lw.lw_item = tv_list_first(l);
                }
            } else if tv.v_type as ::core::ffi::c_uint
                == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*fi).fi_bi = 0 as ::core::ffi::c_int;
                if !tv.vval.v_blob.is_null() {
                    let mut btv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    tv_blob_copy(tv.vval.v_blob, &raw mut btv);
                    (*fi).fi_blob = btv.vval.v_blob;
                }
                tv_clear(&raw mut tv);
            } else if tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*fi).fi_byte_idx = 0 as ::core::ffi::c_int;
                (*fi).fi_string = tv.vval.v_string;
                tv.vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                if (*fi).fi_string.is_null() {
                    (*fi).fi_string = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
                }
            } else {
                emsg(gettext(
                    &raw const e_string_list_or_blob_required as *const ::core::ffi::c_char,
                ));
                tv_clear(&raw mut tv);
            }
        }
    }
    if skip {
        emsg_skip -= 1;
    }
    return fi as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn next_for_item(
    mut fi_void: *mut ::core::ffi::c_void,
    mut arg: *mut ::core::ffi::c_char,
) -> bool {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if !(*fi).fi_blob.is_null() {
        if (*fi).fi_bi >= tv_blob_len((*fi).fi_blob) {
            return false_0 != 0;
        }
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_NUMBER;
        tv.v_lock = VAR_FIXED;
        tv.vval.v_number = tv_blob_get((*fi).fi_blob, (*fi).fi_bi) as varnumber_T;
        (*fi).fi_bi += 1;
        return ex_let_vars(
            arg,
            &raw mut tv,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) == OK;
    }
    if !(*fi).fi_string.is_null() {
        let len: ::core::ffi::c_int =
            utfc_ptr2len((*fi).fi_string.offset((*fi).fi_byte_idx as isize));
        if len == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut tv_0: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_0.v_type = VAR_STRING;
        tv_0.v_lock = VAR_FIXED;
        tv_0.vval.v_string = xmemdupz(
            (*fi).fi_string.offset((*fi).fi_byte_idx as isize) as *const ::core::ffi::c_void,
            len as size_t,
        ) as *mut ::core::ffi::c_char;
        (*fi).fi_byte_idx += len;
        let result: ::core::ffi::c_int = (ex_let_vars(
            arg,
            &raw mut tv_0,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) == OK) as ::core::ffi::c_int;
        xfree(tv_0.vval.v_string as *mut ::core::ffi::c_void);
        return result != 0;
    }
    let mut item: *mut listitem_T = (*fi).fi_lw.lw_item;
    if item.is_null() {
        return false_0 != 0;
    }
    (*fi).fi_lw.lw_item = (*item).li_next;
    return ex_let_vars(
        arg,
        &raw mut (*item).li_tv,
        true_0,
        (*fi).fi_semicolon,
        (*fi).fi_varcount,
        false_0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) == OK;
}
#[no_mangle]
pub unsafe extern "C" fn free_for_info(mut fi_void: *mut ::core::ffi::c_void) {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if fi.is_null() {
        return;
    }
    if !(*fi).fi_list.is_null() {
        tv_list_watch_remove((*fi).fi_list, &raw mut (*fi).fi_lw);
        tv_list_unref((*fi).fi_list);
    } else if !(*fi).fi_blob.is_null() {
        tv_blob_unref((*fi).fi_blob);
    } else {
        xfree((*fi).fi_string as *mut ::core::ffi::c_void);
    }
    xfree(fi as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn set_context_for_expression(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
    mut cmdidx: cmdidx_T,
) {
    let mut got_eq: bool = false_0 != 0;
    if cmdidx as ::core::ffi::c_int == CMD_let as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_const as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_USER_VARS as ::core::ffi::c_int;
        if strpbrk(
            arg,
            b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            let mut p: *mut ::core::ffi::c_char = arg.offset(strlen(arg) as isize);
            while p >= arg {
                (*xp).xp_pattern = p;
                p = p.offset(
                    -((utf_head_off(arg, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
                if ascii_iswhite(*p as ::core::ffi::c_int) {
                    break;
                }
            }
            return;
        }
    } else {
        (*xp).xp_context = if cmdidx as ::core::ffi::c_int == CMD_call as ::core::ffi::c_int {
            EXPAND_FUNCTIONS as ::core::ffi::c_int
        } else {
            EXPAND_EXPRESSION as ::core::ffi::c_int
        };
    }
    loop {
        (*xp).xp_pattern = strpbrk(
            arg,
            b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*xp).xp_pattern.is_null() {
            break;
        }
        let mut c: ::core::ffi::c_int = *(*xp).xp_pattern as uint8_t as ::core::ffi::c_int;
        if c == '&' as ::core::ffi::c_int {
            c = *(*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int;
            if c == '&' as ::core::ffi::c_int {
                (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                (*xp).xp_context = if cmdidx as ::core::ffi::c_int != CMD_let as ::core::ffi::c_int
                    || got_eq as ::core::ffi::c_int != 0
                {
                    EXPAND_EXPRESSION as ::core::ffi::c_int
                } else {
                    EXPAND_NOTHING as ::core::ffi::c_int
                };
            } else if c != ' ' as ::core::ffi::c_int {
                (*xp).xp_context = EXPAND_SETTINGS as ::core::ffi::c_int;
                if (c == 'l' as ::core::ffi::c_int || c == 'g' as ::core::ffi::c_int)
                    && *(*xp).xp_pattern.offset(2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                {
                    (*xp).xp_pattern = (*xp).xp_pattern.offset(2 as ::core::ffi::c_int as isize);
                }
            }
        } else if c == '$' as ::core::ffi::c_int {
            (*xp).xp_context = EXPAND_ENV_VARS as ::core::ffi::c_int;
        } else if c == '=' as ::core::ffi::c_int {
            got_eq = true_0 != 0;
            (*xp).xp_context = EXPAND_EXPRESSION as ::core::ffi::c_int;
        } else {
            if c == '#' as ::core::ffi::c_int
                && (*xp).xp_context == EXPAND_EXPRESSION as ::core::ffi::c_int
            {
                break;
            }
            if (c == '<' as ::core::ffi::c_int || c == '#' as ::core::ffi::c_int)
                && (*xp).xp_context == EXPAND_FUNCTIONS as ::core::ffi::c_int
                && vim_strchr((*xp).xp_pattern, '(' as ::core::ffi::c_int).is_null()
            {
                break;
            }
            if cmdidx as ::core::ffi::c_int != CMD_let as ::core::ffi::c_int
                || got_eq as ::core::ffi::c_int != 0
            {
                if c == '"' as ::core::ffi::c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as ::core::ffi::c_int;
                        if !(c != NUL && c != '"' as ::core::ffi::c_int) {
                            break;
                        }
                        if c == '\\' as ::core::ffi::c_int
                            && *(*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                != NUL
                        {
                            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                } else if c == '\'' as ::core::ffi::c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as ::core::ffi::c_int;
                        if !(c != NUL && c != '\'' as ::core::ffi::c_int) {
                            break;
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                } else if c == '|' as ::core::ffi::c_int {
                    if *(*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '|' as ::core::ffi::c_int
                    {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        (*xp).xp_context = EXPAND_EXPRESSION as ::core::ffi::c_int;
                    } else {
                        (*xp).xp_context = EXPAND_COMMANDS as ::core::ffi::c_int;
                    }
                } else {
                    (*xp).xp_context = EXPAND_EXPRESSION as ::core::ffi::c_int;
                }
            } else {
                (*xp).xp_context = EXPAND_EXPRESSION as ::core::ffi::c_int;
            }
        }
        arg = (*xp).xp_pattern;
        if *arg as ::core::ffi::c_int != NUL {
            loop {
                arg = arg.offset(1);
                c = *arg as uint8_t as ::core::ffi::c_int;
                if !(c != NUL
                    && (c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int))
                {
                    break;
                }
            }
        }
    }
    if cmd_has_expr_args(cmdidx) as ::core::ffi::c_int != 0
        && (*xp).xp_context == EXPAND_EXPRESSION as ::core::ffi::c_int
    {
        loop {
            let n: *mut ::core::ffi::c_char = skiptowhite(arg);
            if n == arg
                || ascii_iswhite_or_nul(*skipwhite(n) as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0
            {
                break;
            }
            arg = skipwhite(n);
        }
    }
    (*xp).xp_pattern = arg;
}
#[no_mangle]
pub unsafe extern "C" fn pattern_match(
    mut pat: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut ic: bool,
) -> ::core::ffi::c_int {
    let mut matches: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo;
    p_cpo = &raw mut empty_string_option as *mut ::core::ffi::c_char;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = ic;
        matches = vim_regexec_nl(&raw mut regmatch, text, 0 as colnr_T) as ::core::ffi::c_int;
        vim_regfree(regmatch.regprog);
    }
    p_cpo = save_cpo;
    return matches;
}
unsafe extern "C" fn eval_func(
    arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    name: *mut ::core::ffi::c_char,
    name_len: ::core::ffi::c_int,
    rettv: *mut typval_T,
    flags: ::core::ffi::c_int,
    basetv: *mut typval_T,
) -> ::core::ffi::c_int {
    let evaluate: bool = flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut s: *mut ::core::ffi::c_char = name;
    let mut len: ::core::ffi::c_int = name_len;
    let mut found_var: bool = false_0 != 0;
    if !evaluate {
        check_vars(s, len as size_t);
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    s = deref_func_name(
        s,
        &raw mut len,
        &raw mut partial,
        !evaluate,
        &raw mut found_var,
    );
    s = xmemdupz(s as *const ::core::ffi::c_void, len as size_t) as *mut ::core::ffi::c_char;
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin).w_cursor.lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = partial;
    funcexe.fe_basetv = basetv;
    funcexe.fe_found_var = found_var;
    let mut ret: ::core::ffi::c_int = get_func_tv(s, len, rettv, arg, evalarg, &raw mut funcexe);
    xfree(s as *mut ::core::ffi::c_void);
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && !evaluate
        && **arg as ::core::ffi::c_int == '(' as ::core::ffi::c_int
    {
        (*rettv).vval.v_string = tv_empty_string as *mut ::core::ffi::c_char;
        (*rettv).v_type = VAR_FUNC;
    }
    if evaluate as ::core::ffi::c_int != 0 && aborting() as ::core::ffi::c_int != 0 {
        if ret == OK {
            tv_clear(rettv);
        }
        ret = FAIL;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn clear_evalarg(mut evalarg: *mut evalarg_T, mut eap: *mut exarg_T) {
    if evalarg.is_null() {
        return;
    }
    if !(*evalarg).eval_tofree.is_null() {
        if !eap.is_null() {
            xfree((*eap).cmdline_tofree as *mut ::core::ffi::c_void);
            (*eap).cmdline_tofree = *(*eap).cmdlinep;
            *(*eap).cmdlinep = (*evalarg).eval_tofree;
        } else {
            xfree((*evalarg).eval_tofree as *mut ::core::ffi::c_void);
        }
        (*evalarg).eval_tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn eval0(
    mut arg: *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let called_emsg_before: ::core::ffi::c_int = called_emsg;
    let mut end_error: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = skipwhite(arg);
    let mut ret: ::core::ffi::c_int = eval1(&raw mut p, rettv, evalarg);
    if ret != FAIL {
        end_error = ends_excmd(*p as ::core::ffi::c_int) == 0;
    }
    if ret == FAIL || end_error as ::core::ffi::c_int != 0 {
        if ret != FAIL {
            tv_clear(rettv);
        }
        if !aborting() && did_emsg == did_emsg_before && called_emsg == called_emsg_before {
            if end_error {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    p,
                );
            } else {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    arg,
                );
            }
        }
        if !eap.is_null() && !p.is_null() {
            let mut nextcmd: *mut ::core::ffi::c_char = check_nextcmd(p);
            if !nextcmd.is_null() && *nextcmd as ::core::ffi::c_int != '|' as ::core::ffi::c_int {
                (*eap).nextcmd = nextcmd;
            }
        }
        return FAIL;
    }
    if !eap.is_null() {
        (*eap).nextcmd = check_nextcmd(p);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn may_call_simple_func(
    mut arg: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut parens: *const ::core::ffi::c_char =
        strstr(arg, b"()\0".as_ptr() as *const ::core::ffi::c_char);
    let mut r: ::core::ffi::c_int = NOTDONE;
    if !parens.is_null()
        && *skipwhite(parens.offset(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int == NUL
    {
        if strnequal(
            arg,
            b"v:lua.\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) {
            let mut p: *const ::core::ffi::c_char = arg.offset(6 as ::core::ffi::c_int as isize);
            if p != parens && skip_luafunc_name(p) == parens {
                r = call_simple_luafunc(p, parens.offset_from(p) as size_t, rettv);
            }
        } else {
            let mut p_0: *const ::core::ffi::c_char = if strncmp(
                arg,
                b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                skipdigits(arg.offset(5 as ::core::ffi::c_int as isize))
                    as *const ::core::ffi::c_char
            } else {
                arg
            };
            if to_name_end(p_0, true_0 != 0) == parens {
                r = call_simple_func(arg, parens.offset_from(arg) as size_t, rettv);
            }
        }
    }
    return r;
}
unsafe extern "C" fn eval0_simple_funccal(
    mut arg: *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = may_call_simple_func(arg, rettv);
    if r == NOTDONE {
        r = eval0(arg, rettv, eap, evalarg);
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn eval1(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    memset(
        rettv as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<typval_T>(),
    );
    if eval2(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut ::core::ffi::c_char = *arg;
    if *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
        let op_falsy: bool = *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '?' as ::core::ffi::c_int;
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as ::core::ffi::c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: ::core::ffi::c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if op_falsy {
                result = tv2bool(rettv);
            } else if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            if error as ::core::ffi::c_int != 0 || !op_falsy || !result {
                tv_clear(rettv);
            }
            if error {
                return FAIL;
            }
        }
        if op_falsy {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        (*evalarg_used).eval_flags = if if op_falsy as ::core::ffi::c_int != 0 {
            !result as ::core::ffi::c_int
        } else {
            result as ::core::ffi::c_int
        } != 0
        {
            orig_flags
        } else {
            orig_flags & !(EVAL_EVALUATE as ::core::ffi::c_int)
        };
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
            (*evalarg_used).eval_flags = orig_flags;
            return FAIL;
        }
        if !op_falsy || !result {
            *rettv = var2;
        }
        if !op_falsy {
            p = *arg;
            if *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
                emsg(gettext(
                    b"E109: Missing ':' after '?'\0".as_ptr() as *const ::core::ffi::c_char
                ));
                if evaluate as ::core::ffi::c_int != 0 && result as ::core::ffi::c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as ::core::ffi::c_int)
            };
            if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
                if evaluate as ::core::ffi::c_int != 0 && result as ::core::ffi::c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            if evaluate as ::core::ffi::c_int != 0 && !result {
                *rettv = var2;
            }
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval2(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    if eval3(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut ::core::ffi::c_char = *arg;
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '|' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '|' as ::core::ffi::c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as ::core::ffi::c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: ::core::ffi::c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '|' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int
        {
            *arg = skipwhite((*arg).offset(2 as ::core::ffi::c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as ::core::ffi::c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval3(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as ::core::ffi::c_int != 0 && !result {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) != 0 as varnumber_T {
                    result = true_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval3(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    if eval4(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut ::core::ffi::c_char = *arg;
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '&' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '&' as ::core::ffi::c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as ::core::ffi::c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: ::core::ffi::c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
        let mut result: bool = true_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) == 0 as varnumber_T {
                result = false_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '&' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '&' as ::core::ffi::c_int
        {
            *arg = skipwhite((*arg).offset(2 as ::core::ffi::c_int as isize));
            (*evalarg_used).eval_flags = if result as ::core::ffi::c_int != 0 {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as ::core::ffi::c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval4(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as ::core::ffi::c_int != 0 && result as ::core::ffi::c_int != 0 {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) == 0 as varnumber_T {
                    result = false_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval4(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut type_0: exprtype_T = EXPR_UNKNOWN;
    let mut len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    if eval5(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut ::core::ffi::c_char = *arg;
    match *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
        61 => {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
            {
                type_0 = EXPR_EQUAL;
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '~' as ::core::ffi::c_int
            {
                type_0 = EXPR_MATCH;
            }
        }
        33 => {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
            {
                type_0 = EXPR_NEQUAL;
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '~' as ::core::ffi::c_int
            {
                type_0 = EXPR_NOMATCH;
            }
        }
        62 => {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '=' as ::core::ffi::c_int
            {
                type_0 = EXPR_GREATER;
                len = 1 as ::core::ffi::c_int;
            } else {
                type_0 = EXPR_GEQUAL;
            }
        }
        60 => {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '=' as ::core::ffi::c_int
            {
                type_0 = EXPR_SMALLER;
                len = 1 as ::core::ffi::c_int;
            } else {
                type_0 = EXPR_SEQUAL;
            }
        }
        105 => {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
            {
                if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'n' as ::core::ffi::c_int
                    && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'o' as ::core::ffi::c_int
                    && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 't' as ::core::ffi::c_int
                {
                    len = 5 as ::core::ffi::c_int;
                }
                if *(*__ctype_b_loc())
                    .offset(*p.offset(len as isize) as uint8_t as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                    && *p.offset(len as isize) as ::core::ffi::c_int != '_' as ::core::ffi::c_int
                {
                    type_0 = (if len == 2 as ::core::ffi::c_int {
                        EXPR_IS as ::core::ffi::c_int
                    } else {
                        EXPR_ISNOT as ::core::ffi::c_int
                    }) as exprtype_T;
                }
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint != EXPR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint {
        let mut ic: bool = false;
        if *p.offset(len as isize) as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
            ic = true_0 != 0;
            len += 1;
        } else if *p.offset(len as isize) as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            ic = false_0 != 0;
            len += 1;
        } else {
            ic = p_ic != 0;
        }
        *arg = skipwhite(p.offset(len as isize));
        if eval5(arg, &raw mut var2, evalarg) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0 {
            let ret: ::core::ffi::c_int = typval_compare(rettv, &raw mut var2, type_0, ic);
            tv_clear(&raw mut var2);
            return ret;
        }
    }
    return OK;
}
unsafe extern "C" fn eval_addblob(mut tv1: *mut typval_T, mut tv2: *mut typval_T) {
    let b1: *const blob_T = (*tv1).vval.v_blob;
    let b2: *const blob_T = (*tv2).vval.v_blob;
    let b: *mut blob_T = tv_blob_alloc();
    let mut len1: int64_t = tv_blob_len(b1) as int64_t;
    let mut len2: int64_t = tv_blob_len(b2) as int64_t;
    let mut totallen: int64_t = len1 + len2;
    if totallen >= 0 as int64_t && totallen <= INT_MAX as int64_t {
        ga_grow(&raw mut (*b).bv_ga, totallen as ::core::ffi::c_int);
        if len1 > 0 as int64_t {
            memmove(
                (*b).bv_ga.ga_data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (*b1).bv_ga.ga_data,
                len1 as size_t,
            );
        }
        if len2 > 0 as int64_t {
            memmove(
                ((*b).bv_ga.ga_data as *mut ::core::ffi::c_char).offset(len1 as isize)
                    as *mut ::core::ffi::c_void,
                (*b2).bv_ga.ga_data,
                len2 as size_t,
            );
        }
        (*b).bv_ga.ga_len = totallen as ::core::ffi::c_int;
    }
    tv_clear(tv1);
    tv_blob_set_ret(tv1, b);
}
unsafe extern "C" fn eval_addlist(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut var3: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if tv_list_concat((*tv1).vval.v_list, (*tv2).vval.v_list, &raw mut var3) == FAIL {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    tv_clear(tv1);
    *tv1 = var3;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn grow_string_tv(
    mut tv1: *mut typval_T,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv1).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv1).vval.v_string.is_null()
    {
        return FAIL;
    }
    let mut len1: size_t = strlen((*tv1).vval.v_string);
    let mut len2: size_t = strlen(s2);
    let mut p: *mut ::core::ffi::c_char = xrealloc(
        (*tv1).vval.v_string as *mut ::core::ffi::c_void,
        len1.wrapping_add(len2).wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    memmove(
        p.offset(len1 as isize) as *mut ::core::ffi::c_void,
        s2 as *const ::core::ffi::c_void,
        len2.wrapping_add(1 as size_t),
    );
    (*tv1).vval.v_string = p;
    return OK;
}
unsafe extern "C" fn eval_concat_str(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let s1: *const ::core::ffi::c_char =
        tv_get_string_buf(tv1, &raw mut buf1 as *mut ::core::ffi::c_char);
    let s2: *const ::core::ffi::c_char =
        tv_get_string_buf_chk(tv2, &raw mut buf2 as *mut ::core::ffi::c_char);
    if s2.is_null() {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    if grow_string_tv(tv1, s2) == OK {
        return OK;
    }
    let mut p: *mut ::core::ffi::c_char = concat_str(s1, s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = p;
    return OK;
}
unsafe extern "C" fn eval_addsub_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut error: bool = false_0 != 0;
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut f1: float_T = 0 as ::core::ffi::c_int as float_T;
    let mut f2: float_T = 0 as ::core::ffi::c_int as float_T;
    if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        f1 = (*tv1).vval.v_float;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv2).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            f1 = n1 as float_T;
        }
    }
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv1).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            f2 = n2 as float_T;
        }
    }
    tv_clear(tv1);
    if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv2).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if op == '+' as ::core::ffi::c_int {
            f1 = f1 + f2;
        } else {
            f1 = f1 - f2;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '+' as ::core::ffi::c_int {
            n1 = n1 + n2;
        } else {
            n1 = n1 - n2;
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}
unsafe extern "C" fn eval5(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    if eval6(arg, rettv, evalarg, false_0 != 0) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: ::core::ffi::c_int = **arg as uint8_t as ::core::ffi::c_int;
        let mut concat: bool = op == '.' as ::core::ffi::c_int;
        if op != '+' as ::core::ffi::c_int && op != '-' as ::core::ffi::c_int && !concat {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
        } != 0;
        if (op != '+' as ::core::ffi::c_int
            || (*rettv).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*rettv).v_type as ::core::ffi::c_uint
                    != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint)
            && (op == '.' as ::core::ffi::c_int
                || (*rettv).v_type as ::core::ffi::c_uint
                    != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint)
            && evaluate as ::core::ffi::c_int != 0
        {
            if op == '.' as ::core::ffi::c_int && !tv_check_str(rettv)
                || op != '.' as ::core::ffi::c_int && !tv_check_num(rettv)
            {
                tv_clear(rettv);
                return FAIL;
            }
        }
        if op == '.' as ::core::ffi::c_int
            && *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
        {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval6(arg, &raw mut var2, evalarg, op == '.' as ::core::ffi::c_int) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if evaluate {
            if op == '.' as ::core::ffi::c_int {
                if eval_concat_str(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if op == '+' as ::core::ffi::c_int
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
                && var2.v_type as ::core::ffi::c_uint
                    == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                eval_addblob(rettv, &raw mut var2);
            } else if op == '+' as ::core::ffi::c_int
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                && var2.v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if eval_addlist(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if eval_addsub_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
            tv_clear(&raw mut var2);
        }
    }
    return OK;
}
unsafe extern "C" fn eval_multdiv_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut use_float: bool = false_0 != 0;
    let mut f1: float_T = 0 as ::core::ffi::c_int as float_T;
    let mut f2: float_T = 0 as ::core::ffi::c_int as float_T;
    let mut error: bool = false_0 != 0;
    if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        f1 = (*tv1).vval.v_float;
        use_float = true_0 != 0;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
    }
    tv_clear(tv1);
    if error {
        tv_clear(tv2);
        return FAIL;
    }
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !use_float {
            f1 = n1 as float_T;
            use_float = true_0 != 0;
        }
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        tv_clear(tv2);
        if error {
            return FAIL;
        }
        if use_float {
            f2 = n2 as float_T;
        }
    }
    if use_float {
        if op == '*' as ::core::ffi::c_int {
            f1 = f1 * f2;
        } else if op == '/' as ::core::ffi::c_int {
            f1 = if f2 == 0 as ::core::ffi::c_int as float_T {
                if f1 == 0 as ::core::ffi::c_int as float_T {
                    ::core::f32::NAN as float_T
                } else if f1 > 0 as ::core::ffi::c_int as float_T {
                    ::core::f32::INFINITY as float_T
                } else {
                    -::core::f32::INFINITY as float_T
                }
            } else {
                f1 / f2
            };
        } else {
            emsg(gettext(
                b"E804: Cannot use '%' with Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '*' as ::core::ffi::c_int {
            n1 = n1 * n2;
        } else if op == '/' as ::core::ffi::c_int {
            n1 = num_divide(n1, n2);
        } else {
            n1 = num_modulus(n1, n2);
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}
unsafe extern "C" fn eval6(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> ::core::ffi::c_int {
    if eval7(arg, rettv, evalarg, want_string) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: ::core::ffi::c_int = **arg as uint8_t as ::core::ffi::c_int;
        if op != '*' as ::core::ffi::c_int
            && op != '/' as ::core::ffi::c_int
            && op != '%' as ::core::ffi::c_int
        {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
        } != 0;
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval7(arg, &raw mut var2, evalarg, false_0 != 0) == FAIL {
            return FAIL;
        }
        if evaluate {
            if eval_multdiv_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
        }
    }
    return OK;
}
unsafe extern "C" fn eval7(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut ret: ::core::ffi::c_int = OK;
    static mut recurse: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut start_leader: *const ::core::ffi::c_char = *arg;
    while **arg as ::core::ffi::c_int == '!' as ::core::ffi::c_int
        || **arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || **arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    }
    let mut end_leader: *const ::core::ffi::c_char = *arg;
    if recurse == 1000 as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_expression_too_recursive_str as *const ::core::ffi::c_char),
            *arg,
        );
        return FAIL;
    }
    recurse += 1;
    match **arg as ::core::ffi::c_int {
        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
            ret = eval_number(arg, rettv, evaluate, want_string);
            if ret == OK && evaluate as ::core::ffi::c_int != 0 && end_leader > start_leader {
                ret = eval7_leader(rettv, true_0 != 0, start_leader, &raw mut end_leader);
            }
        }
        34 => {
            ret = eval_string(arg, rettv, evaluate, false_0 != 0);
        }
        39 => {
            ret = eval_lit_string(arg, rettv, evaluate, false_0 != 0);
        }
        91 => {
            ret = eval_list(arg, rettv, evalarg);
        }
        35 => {
            ret = eval_lit_dict(arg, rettv, evalarg);
        }
        123 => {
            ret = get_lambda_tv(arg, rettv, evalarg);
            if ret == NOTDONE {
                ret = eval_dict(arg, rettv, evalarg, false_0 != 0);
            }
        }
        38 => {
            ret = eval_option(arg as *mut *const ::core::ffi::c_char, rettv, evaluate);
        }
        36 => {
            if *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int
                || *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\'' as ::core::ffi::c_int
            {
                ret = eval_interp_string(arg, rettv, evaluate);
            } else {
                ret = eval_env_var(arg, rettv, evaluate as ::core::ffi::c_int);
            }
        }
        64 => {
            *arg = (*arg).offset(1);
            if evaluate {
                (*rettv).v_type = VAR_STRING;
                (*rettv).vval.v_string = get_reg_contents(
                    **arg as ::core::ffi::c_int,
                    kGRegExprSrc as ::core::ffi::c_int,
                ) as *mut ::core::ffi::c_char;
            }
            if **arg as ::core::ffi::c_int != NUL {
                *arg = (*arg).offset(1);
            }
        }
        40 => {
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            ret = eval1(arg, rettv, evalarg);
            if **arg as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
                *arg = (*arg).offset(1);
            } else if ret == OK {
                emsg(gettext(
                    b"E110: Missing ')'\0".as_ptr() as *const ::core::ffi::c_char
                ));
                tv_clear(rettv);
                ret = FAIL;
            }
        }
        _ => {
            ret = NOTDONE;
        }
    }
    if ret == NOTDONE {
        let mut s: *mut ::core::ffi::c_char = *arg;
        let mut alias: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = get_name_len(
            arg as *mut *const ::core::ffi::c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            s = alias;
        }
        if len <= 0 as ::core::ffi::c_int {
            ret = FAIL;
        } else {
            let flags: ::core::ffi::c_int = if evalarg.is_null() {
                0 as ::core::ffi::c_int
            } else {
                (*evalarg).eval_flags
            };
            if *skipwhite(*arg) as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                *arg = skipwhite(*arg);
                ret = eval_func(
                    arg,
                    evalarg,
                    s,
                    len,
                    rettv,
                    flags,
                    ::core::ptr::null_mut::<typval_T>(),
                );
            } else if evaluate {
                ret = eval_variable(
                    s,
                    len,
                    rettv,
                    ::core::ptr::null_mut::<*mut dictitem_T>(),
                    true_0 != 0,
                    false_0 != 0,
                );
            } else {
                check_vars(s, len as size_t);
                if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !evaluate
                    && strnequal(
                        s,
                        b"v:lua.\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = OK;
            }
        }
        xfree(alias as *mut ::core::ffi::c_void);
    }
    *arg = skipwhite(*arg);
    if ret == OK {
        ret = handle_subscript(
            arg as *mut *const ::core::ffi::c_char,
            rettv,
            evalarg,
            true_0 != 0,
        );
    }
    if ret == OK && evaluate as ::core::ffi::c_int != 0 && end_leader > start_leader {
        ret = eval7_leader(rettv, false_0 != 0, start_leader, &raw mut end_leader);
    }
    recurse -= 1;
    return ret;
}
unsafe extern "C" fn eval7_leader(
    rettv: *mut typval_T,
    numeric_only: bool,
    start_leader: *const ::core::ffi::c_char,
    end_leaderp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut end_leader: *const ::core::ffi::c_char = *end_leaderp;
    let mut ret: ::core::ffi::c_int = OK;
    let mut error: bool = false_0 != 0;
    let mut val: varnumber_T = 0 as varnumber_T;
    let mut f: float_T = 0.0f64;
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        f = (*rettv).vval.v_float;
    } else {
        val = tv_get_number_chk(rettv, &raw mut error);
    }
    if error {
        tv_clear(rettv);
        ret = FAIL;
    } else {
        while end_leader > start_leader {
            end_leader = end_leader.offset(-1);
            if *end_leader as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                if numeric_only {
                    end_leader = end_leader.offset(1);
                    break;
                } else if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*rettv).v_type = VAR_BOOL;
                    val = (if f == 0.0f64 {
                        kBoolVarTrue as ::core::ffi::c_int
                    } else {
                        kBoolVarFalse as ::core::ffi::c_int
                    }) as varnumber_T;
                } else {
                    val = (val == 0) as ::core::ffi::c_int as varnumber_T;
                }
            } else if *end_leader as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    f = -f;
                } else {
                    val = -val;
                }
            }
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_clear(rettv);
            (*rettv).vval.v_float = f;
        } else {
            tv_clear(rettv);
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = val;
        }
    }
    *end_leaderp = end_leader;
    return ret;
}
unsafe extern "C" fn call_func_rettv(
    arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    rettv: *mut typval_T,
    evaluate: bool,
    selfdict: *mut dict_T,
    basetv: *mut typval_T,
    lua_funcname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut functv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut is_lua: bool = false_0 != 0;
    let mut ret: ::core::ffi::c_int = 0;
    '_theend: {
        if evaluate {
            functv = *rettv;
            (*rettv).v_type = VAR_UNKNOWN;
            if functv.v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pt = functv.vval.v_partial;
                is_lua = is_luafunc(pt);
                funcname = if is_lua as ::core::ffi::c_int != 0 {
                    lua_funcname
                } else {
                    partial_name(pt) as *const ::core::ffi::c_char
                };
            } else {
                funcname = functv.vval.v_string;
                if funcname.is_null() || *funcname as ::core::ffi::c_int == NUL {
                    emsg(gettext(
                        &raw const e_empty_function_name as *const ::core::ffi::c_char,
                    ));
                    ret = FAIL;
                    break '_theend;
                }
            }
        } else {
            funcname = b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin).w_cursor.lnum;
        funcexe.fe_evaluate = evaluate;
        funcexe.fe_partial = pt;
        funcexe.fe_selfdict = selfdict;
        funcexe.fe_basetv = basetv;
        ret = get_func_tv(
            funcname,
            if is_lua as ::core::ffi::c_int != 0 {
                (*arg).offset_from(funcname) as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            },
            rettv,
            arg,
            evalarg,
            &raw mut funcexe,
        );
    }
    if evaluate {
        tv_clear(&raw mut functv);
    }
    return ret;
}
unsafe extern "C" fn eval_lambda(
    arg: *mut *mut ::core::ffi::c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    *arg = (*arg).offset(2 as ::core::ffi::c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut ret: ::core::ffi::c_int = get_lambda_tv(arg, rettv, evalarg);
    if ret != OK {
        return FAIL;
    } else if **arg as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        if verbose {
            if *skipwhite(*arg) as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                emsg(gettext(e_nowhitespace));
            } else {
                semsg(
                    gettext(&raw const e_missingparen as *const ::core::ffi::c_char),
                    b"lambda\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        tv_clear(rettv);
        ret = FAIL;
    } else {
        ret = call_func_rettv(
            arg,
            evalarg,
            rettv,
            evaluate,
            ::core::ptr::null_mut::<dict_T>(),
            &raw mut base,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    return ret;
}
unsafe extern "C" fn eval_method(
    arg: *mut *mut ::core::ffi::c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    *arg = (*arg).offset(2 as ::core::ffi::c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut len: ::core::ffi::c_int = 0;
    let mut name: *mut ::core::ffi::c_char = *arg;
    let mut lua_funcname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut alias: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strnequal(
        name,
        b"v:lua.\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) {
        lua_funcname = name.offset(6 as ::core::ffi::c_int as isize);
        *arg = skip_luafunc_name(lua_funcname) as *mut ::core::ffi::c_char;
        *arg = skipwhite(*arg);
        len = (*arg).offset_from(lua_funcname) as ::core::ffi::c_int;
    } else {
        len = get_name_len(
            arg as *mut *const ::core::ffi::c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            name = alias;
        }
    }
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = OK;
    if len <= 0 as ::core::ffi::c_int {
        if verbose {
            if lua_funcname.is_null() {
                emsg(gettext(
                    b"E260: Missing name after ->\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    name,
                );
            }
        }
        ret = FAIL;
    } else {
        *arg = skipwhite(*arg);
        let mut paren: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if **arg as ::core::ffi::c_int != '(' as ::core::ffi::c_int
            && lua_funcname.is_null()
            && alias.is_null()
            && {
                paren = vim_strchr(*arg, '(' as ::core::ffi::c_int);
                !paren.is_null()
            }
        {
            *arg = name;
            *paren = NUL as ::core::ffi::c_char;
            let mut ref_0: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            ref_0.v_type = VAR_UNKNOWN;
            if eval7(arg, &raw mut ref_0, evalarg, false_0 != 0) == FAIL {
                *arg = name.offset(len as isize);
                ret = FAIL;
            } else if *skipwhite(*arg) as ::core::ffi::c_int != NUL {
                if verbose {
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        *arg,
                    );
                }
                ret = FAIL;
            } else if ref_0.v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                && !ref_0.vval.v_string.is_null()
            {
                name = ref_0.vval.v_string;
                ref_0.vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                tofree = name;
                len = strlen(name) as ::core::ffi::c_int;
            } else if ref_0.v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && !ref_0.vval.v_partial.is_null()
            {
                if (*ref_0.vval.v_partial).pt_argc > 0 as ::core::ffi::c_int
                    || !(*ref_0.vval.v_partial).pt_dict.is_null()
                {
                    if verbose {
                        emsg(gettext(
                            &raw const e_cannot_use_partial_here as *const ::core::ffi::c_char,
                        ));
                    }
                    ret = FAIL;
                } else {
                    name = xstrdup(partial_name(ref_0.vval.v_partial));
                    tofree = name;
                    if name.is_null() {
                        ret = FAIL;
                        name = *arg;
                    } else {
                        len = strlen(name) as ::core::ffi::c_int;
                    }
                }
            } else {
                if verbose {
                    semsg(
                        gettext(&raw const e_not_callable_type_str as *const ::core::ffi::c_char),
                        name,
                    );
                }
                ret = FAIL;
            }
            tv_clear(&raw mut ref_0);
            *paren = '(' as ::core::ffi::c_char;
        }
        if ret == OK {
            if **arg as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                if verbose {
                    semsg(
                        gettext(&raw const e_missingparen as *const ::core::ffi::c_char),
                        name,
                    );
                }
                ret = FAIL;
            } else if ascii_iswhite(
                *(*arg).offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) {
                if verbose {
                    emsg(gettext(e_nowhitespace));
                }
                ret = FAIL;
            } else if !lua_funcname.is_null() {
                if evaluate {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = call_func_rettv(
                    arg,
                    evalarg,
                    rettv,
                    evaluate,
                    ::core::ptr::null_mut::<dict_T>(),
                    &raw mut base,
                    lua_funcname,
                );
            } else {
                ret = eval_func(
                    arg,
                    evalarg,
                    name,
                    len,
                    rettv,
                    if evaluate as ::core::ffi::c_int != 0 {
                        EVAL_EVALUATE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    &raw mut base,
                );
            }
        }
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    if !alias.is_null() {
        xfree(alias as *mut ::core::ffi::c_void);
    }
    return ret;
}
unsafe extern "C" fn eval_index(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut empty1: bool = false_0 != 0;
    let mut empty2: bool = false_0 != 0;
    let mut range: bool = false_0 != 0;
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut keylen: ptrdiff_t = -1 as ptrdiff_t;
    if check_can_index(rettv, evaluate, verbose) == FAIL {
        return FAIL;
    }
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if **arg as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        key = (*arg).offset(1 as ::core::ffi::c_int as isize);
        keylen = 0 as ptrdiff_t;
        while eval_isdictc(*key.offset(keylen as isize) as ::core::ffi::c_int) {
            keylen += 1;
        }
        if keylen == 0 as ptrdiff_t {
            return FAIL;
        }
        *arg = skipwhite(key.offset(keylen as isize));
    } else {
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        if **arg as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            empty1 = true_0 != 0;
        } else if eval1(arg, &raw mut var1, evalarg) == FAIL {
            return FAIL;
        } else if evaluate as ::core::ffi::c_int != 0 && !tv_check_str(&raw mut var1) {
            tv_clear(&raw mut var1);
            return FAIL;
        }
        if **arg as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            range = true_0 != 0;
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            if **arg as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                empty2 = true_0 != 0;
            } else if eval1(arg, &raw mut var2, evalarg) == FAIL {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                return FAIL;
            } else if evaluate as ::core::ffi::c_int != 0 && !tv_check_str(&raw mut var2) {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                tv_clear(&raw mut var2);
                return FAIL;
            }
        }
        if **arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
            if verbose {
                emsg(gettext(e_missbrac));
            }
            tv_clear(&raw mut var1);
            if range {
                tv_clear(&raw mut var2);
            }
            return FAIL;
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    }
    if evaluate {
        let mut res: ::core::ffi::c_int = eval_index_inner(
            rettv,
            range,
            if empty1 as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var1
            },
            if empty2 as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var2
            },
            false_0 != 0,
            key,
            keylen,
            verbose,
        );
        if !empty1 {
            tv_clear(&raw mut var1);
        }
        if range {
            tv_clear(&raw mut var2);
        }
        return res;
    }
    return OK;
}
unsafe extern "C" fn check_can_index(
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    match (*rettv).v_type as ::core::ffi::c_uint {
        3 | 9 => {
            if verbose {
                emsg(gettext(
                    &raw const e_cannot_index_a_funcref as *const ::core::ffi::c_char,
                ));
            }
            return FAIL;
        }
        6 => {
            if verbose {
                emsg(gettext(
                    &raw const e_using_float_as_string as *const ::core::ffi::c_char,
                ));
            }
            return FAIL;
        }
        7 | 8 => {
            if verbose {
                emsg(gettext(
                    &raw const e_cannot_index_special_variable as *const ::core::ffi::c_char,
                ));
            }
            return FAIL;
        }
        0 => {
            if evaluate {
                emsg(gettext(
                    &raw const e_cannot_index_special_variable as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        }
        2 | 1 | 4 | 5 | 10 | _ => {}
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn f_slice(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_can_index(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        true_0 != 0,
        false_0 != 0,
    ) != OK
    {
        return;
    }
    tv_copy(argvars, rettv);
    eval_index_inner(
        rettv,
        true_0 != 0,
        argvars.offset(1 as ::core::ffi::c_int as isize),
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ::core::ptr::null_mut::<typval_T>()
        } else {
            argvars.offset(2 as ::core::ffi::c_int as isize)
        },
        true_0 != 0,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
        false_0 != 0,
    );
}
unsafe extern "C" fn eval_index_inner(
    mut rettv: *mut typval_T,
    mut is_range: bool,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut exclusive: bool,
    mut key: *const ::core::ffi::c_char,
    mut keylen: ptrdiff_t,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    let mut n1: varnumber_T = 0 as varnumber_T;
    let mut n2: varnumber_T = 0 as varnumber_T;
    if !var1.is_null()
        && (*rettv).v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        n1 = tv_get_number(var1);
    }
    if is_range {
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if verbose {
                emsg(gettext(
                    &raw const e_cannot_slice_dictionary as *const ::core::ffi::c_char,
                ));
            }
            return FAIL;
        }
        if !var2.is_null() {
            n2 = tv_get_number(var2);
        } else {
            n2 = VARNUMBER_MAX as varnumber_T;
        }
    }
    match (*rettv).v_type as ::core::ffi::c_uint {
        1 | 2 => {
            let s: *const ::core::ffi::c_char = tv_get_string(rettv);
            let mut v: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut len: ::core::ffi::c_int = strlen(s) as ::core::ffi::c_int;
            if exclusive {
                if is_range {
                    v = string_slice(s, n1, n2, exclusive);
                } else {
                    v = char_from_string(s, n1);
                }
            } else if is_range {
                if n1 < 0 as varnumber_T {
                    n1 = len as varnumber_T + n1;
                    if n1 < 0 as varnumber_T {
                        n1 = 0 as varnumber_T;
                    }
                }
                if n2 < 0 as varnumber_T {
                    n2 = len as varnumber_T + n2;
                } else if n2 >= len as varnumber_T {
                    n2 = len as varnumber_T;
                }
                if n1 >= len as varnumber_T || n2 < 0 as varnumber_T || n1 > n2 {
                    v = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    v = xmemdupz(
                        s.offset(n1 as isize) as *const ::core::ffi::c_void,
                        (n2 as size_t)
                            .wrapping_sub(n1 as size_t)
                            .wrapping_add(1 as size_t),
                    ) as *mut ::core::ffi::c_char;
                }
            } else if n1 >= len as varnumber_T || n1 < 0 as varnumber_T {
                v = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                v = xmemdupz(
                    s.offset(n1 as isize) as *const ::core::ffi::c_void,
                    1 as size_t,
                ) as *mut ::core::ffi::c_char;
            }
            tv_clear(rettv);
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = v;
        }
        10 => {
            tv_blob_slice_or_index((*rettv).vval.v_blob, is_range, n1, n2, exclusive, rettv);
        }
        4 => {
            if var1.is_null() {
                n1 = 0 as varnumber_T;
            }
            if var2.is_null() {
                n2 = VARNUMBER_MAX as varnumber_T;
            }
            if tv_list_slice_or_index(
                (*rettv).vval.v_list,
                is_range,
                n1,
                n2,
                exclusive,
                rettv,
                verbose,
            ) == FAIL
            {
                return FAIL;
            }
        }
        5 => {
            if key.is_null() {
                key = tv_get_string_chk(var1);
                if key.is_null() {
                    return FAIL;
                }
            }
            let item: *mut dictitem_T = tv_dict_find((*rettv).vval.v_dict, key, keylen);
            if item.is_null() && verbose as ::core::ffi::c_int != 0 {
                if keylen > 0 as ptrdiff_t {
                    semsg(
                        gettext(&raw const e_dictkey_len as *const ::core::ffi::c_char),
                        keylen,
                        key,
                    );
                } else {
                    semsg(
                        gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                        key,
                    );
                }
            }
            if item.is_null() || tv_is_luafunc(&raw mut (*item).di_tv) as ::core::ffi::c_int != 0 {
                return FAIL;
            }
            let mut tmp: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv_copy(&raw mut (*item).di_tv, &raw mut tmp);
            tv_clear(rettv);
            *rettv = tmp;
        }
        7 | 8 | 3 | 6 | 9 | 0 | _ => {}
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn eval_option(
    arg: *mut *const ::core::ffi::c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> ::core::ffi::c_int {
    let working: bool = **arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int;
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: ::core::ffi::c_int = 0;
    let option_end: *mut ::core::ffi::c_char =
        find_option_var_end(arg, &raw mut opt_idx, &raw mut opt_flags) as *mut ::core::ffi::c_char;
    if option_end.is_null() {
        if !rettv.is_null() {
            semsg(
                gettext(b"E112: Option name missing: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *arg,
            );
        }
        return FAIL;
    }
    if !evaluate {
        *arg = option_end;
        return OK;
    }
    let mut c: ::core::ffi::c_char = *option_end;
    *option_end = NUL as ::core::ffi::c_char;
    let mut ret: ::core::ffi::c_int = OK;
    let mut is_tty_opt: bool = is_tty_option(*arg);
    if opt_idx as ::core::ffi::c_int == kOptInvalid as ::core::ffi::c_int && !is_tty_opt {
        if !rettv.is_null() {
            semsg(
                gettext(b"E113: Unknown option: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *arg,
            );
        }
        ret = FAIL;
    } else if !rettv.is_null() {
        let mut value: OptVal = if is_tty_opt as ::core::ffi::c_int != 0 {
            get_tty_option(*arg)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        '_c2rust_label: {
            if value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"value.type != kOptValTypeNil\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    3409 as ::core::ffi::c_uint,
                    b"int eval_option(const char **const, typval_T *const, const _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        *rettv = optval_as_tv(value, true_0 != 0);
    } else if working as ::core::ffi::c_int != 0
        && !is_tty_opt
        && is_option_hidden(opt_idx) as ::core::ffi::c_int != 0
    {
        ret = FAIL;
    }
    *option_end = c;
    *arg = option_end;
    return ret;
}
unsafe extern "C" fn eval_number(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut want_string: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char =
        skipdigits((*arg).offset(1 as ::core::ffi::c_int as isize));
    let mut get_float: bool = false_0 != 0;
    if !want_string
        && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
        && ascii_isdigit(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        get_float = true_0 != 0;
        p = skipdigits(p.offset(2 as ::core::ffi::c_int as isize));
        if *p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
        {
            p = p.offset(1);
            if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if !ascii_isdigit(*p as ::core::ffi::c_int) {
                get_float = false_0 != 0;
            } else {
                p = skipdigits(p.offset(1 as ::core::ffi::c_int as isize));
            }
        }
        if *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        {
            get_float = false_0 != 0;
        }
    }
    if get_float {
        let mut f: float_T = 0.;
        *arg = (*arg).offset(string2float(*arg, &raw mut f) as isize);
        if evaluate {
            (*rettv).v_type = VAR_FLOAT;
            (*rettv).vval.v_float = f;
        }
    } else if **arg as ::core::ffi::c_int == '0' as ::core::ffi::c_int
        && (*(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'z' as ::core::ffi::c_int
            || *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'Z' as ::core::ffi::c_int)
    {
        let mut blob: *mut blob_T = ::core::ptr::null_mut::<blob_T>();
        if evaluate {
            blob = tv_blob_alloc();
        }
        let mut bp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        bp = (*arg).offset(2 as ::core::ffi::c_int as isize);
        while ascii_isxdigit(*bp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int) {
            if !ascii_isxdigit(*bp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int) {
                if !blob.is_null() {
                    emsg(gettext(
                        b"E973: Blob literal should have an even number of hex characters\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ));
                    ga_clear(&raw mut (*blob).bv_ga);
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut blob as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    *ptr_;
                }
                return FAIL;
            }
            if !blob.is_null() {
                ga_append(
                    &raw mut (*blob).bv_ga,
                    ((hex2nr(*bp as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
                        + hex2nr(*bp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int))
                        as uint8_t,
                );
            }
            if *bp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && ascii_isxdigit(*bp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                bp = bp.offset(1);
            }
            bp = bp.offset(2 as ::core::ffi::c_int as isize);
        }
        if !blob.is_null() {
            tv_blob_set_ret(rettv, blob);
        }
        *arg = bp;
    } else {
        let mut len: ::core::ffi::c_int = 0;
        let mut n: varnumber_T = 0;
        vim_str2nr(
            *arg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut len,
            STR2NR_ALL as ::core::ffi::c_int,
            &raw mut n,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as ::core::ffi::c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if len == 0 as ::core::ffi::c_int {
            if evaluate {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    *arg,
                );
            }
            return FAIL;
        }
        *arg = (*arg).offset(len as isize);
        if evaluate {
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = n;
        }
    }
    return OK;
}
unsafe extern "C" fn eval_string(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let arg_end: *const ::core::ffi::c_char = (*arg).offset(strlen(*arg) as isize);
    let mut extra: ::core::ffi::c_uint = (if interpolate as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as ::core::ffi::c_uint;
    let off: ::core::ffi::c_int = if interpolate as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    p = (*arg).offset(off as isize);
    while *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
            if *p as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut flags: ::core::ffi::c_int =
                    FSK_KEYCODE as ::core::ffi::c_int | FSK_IN_STRING as ::core::ffi::c_int;
                extra = extra.wrapping_add(5 as ::core::ffi::c_uint);
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '*' as ::core::ffi::c_int
                {
                    flags |= FSK_SIMPLIFY as ::core::ffi::c_int;
                }
                if find_special_key(
                    &raw mut p as *mut *const ::core::ffi::c_char,
                    arg_end.offset_from(p) as size_t,
                    &raw mut modifiers,
                    flags,
                    ::core::ptr::null_mut::<bool>(),
                ) != 0 as ::core::ffi::c_int
                {
                    p = p.offset(-1);
                }
            }
        } else if interpolate as ::core::ffi::c_int != 0
            && (*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int)
        {
            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '{' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
            if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '}' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '}' as ::core::ffi::c_int
            {
                semsg(
                    gettext(&raw const e_stray_closing_curly_str as *const ::core::ffi::c_char),
                    *arg,
                );
                return FAIL;
            }
            extra = extra.wrapping_sub(1);
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
        && !(interpolate as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int)
    {
        semsg(
            gettext(b"E114: Missing quote: %s\0".as_ptr() as *const ::core::ffi::c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    (*rettv).v_type = VAR_STRING;
    let len: ::core::ffi::c_int = (p.offset_from(*arg) + extra as isize) as ::core::ffi::c_int;
    (*rettv).vval.v_string = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
    let mut end: *mut ::core::ffi::c_char = (*rettv).vval.v_string;
    p = (*arg).offset(off as isize);
    while *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            's_424: {
                p = p.offset(1);
                match *p as ::core::ffi::c_int {
                    98 => {
                        let c2rust_fresh0 = end;
                        end = end.offset(1);
                        *c2rust_fresh0 = BS as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    101 => {
                        let c2rust_fresh1 = end;
                        end = end.offset(1);
                        *c2rust_fresh1 = ESC as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    102 => {
                        let c2rust_fresh2 = end;
                        end = end.offset(1);
                        *c2rust_fresh2 = FF as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    110 => {
                        let c2rust_fresh3 = end;
                        end = end.offset(1);
                        *c2rust_fresh3 = NL as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    114 => {
                        let c2rust_fresh4 = end;
                        end = end.offset(1);
                        *c2rust_fresh4 = CAR as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    116 => {
                        let c2rust_fresh5 = end;
                        end = end.offset(1);
                        *c2rust_fresh5 = TAB as ::core::ffi::c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    88 | 120 | 117 | 85 => {
                        if ascii_isxdigit(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) {
                            let mut n: ::core::ffi::c_int = 0;
                            let mut nr: ::core::ffi::c_int = 0;
                            let mut c: ::core::ffi::c_int =
                                toupper(*p as uint8_t as ::core::ffi::c_int);
                            if c == 'X' as ::core::ffi::c_int {
                                n = 2 as ::core::ffi::c_int;
                            } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                                n = 4 as ::core::ffi::c_int;
                            } else {
                                n = 8 as ::core::ffi::c_int;
                            }
                            nr = 0 as ::core::ffi::c_int;
                            loop {
                                n -= 1;
                                if !(n >= 0 as ::core::ffi::c_int
                                    && ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                                {
                                    break;
                                }
                                p = p.offset(1);
                                nr = (nr << 4 as ::core::ffi::c_int)
                                    + hex2nr(*p as ::core::ffi::c_int);
                            }
                            p = p.offset(1);
                            if c != 'X' as ::core::ffi::c_int {
                                end = end.offset(utf_char2bytes(nr, end) as isize);
                            } else {
                                let c2rust_fresh6 = end;
                                end = end.offset(1);
                                *c2rust_fresh6 = nr as ::core::ffi::c_char;
                            }
                        }
                        break 's_424;
                    }
                    48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                        let c2rust_fresh7 = p;
                        p = p.offset(1);
                        *end = (*c2rust_fresh7 as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        if *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                        {
                            let c2rust_fresh8 = p;
                            p = p.offset(1);
                            *end = (((*end as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
                                + *c2rust_fresh8 as ::core::ffi::c_int
                                - '0' as ::core::ffi::c_int)
                                as ::core::ffi::c_char;
                            if *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                            {
                                let c2rust_fresh9 = p;
                                p = p.offset(1);
                                *end = (((*end as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
                                    + *c2rust_fresh9 as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int)
                                    as ::core::ffi::c_char;
                            }
                        }
                        end = end.offset(1);
                        break 's_424;
                    }
                    60 => {
                        let mut flags_0: ::core::ffi::c_int =
                            FSK_KEYCODE as ::core::ffi::c_int | FSK_IN_STRING as ::core::ffi::c_int;
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '*' as ::core::ffi::c_int
                        {
                            flags_0 |= FSK_SIMPLIFY as ::core::ffi::c_int;
                        }
                        extra = trans_special(
                            &raw mut p as *mut *const ::core::ffi::c_char,
                            arg_end.offset_from(p) as size_t,
                            end,
                            flags_0,
                            false_0 != 0,
                            ::core::ptr::null_mut::<bool>(),
                        );
                        if extra != 0 as ::core::ffi::c_uint {
                            end = end.offset(extra as isize);
                            if end >= (*rettv).vval.v_string.offset(len as isize) {
                                iemsg(b"eval_string() used more space than allocated\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            }
                            break 's_424;
                        }
                    }
                    _ => {}
                }
                mb_copy_char(&raw mut p as *mut *const ::core::ffi::c_char, &raw mut end);
            }
        } else {
            if interpolate as ::core::ffi::c_int != 0
                && (*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int)
            {
                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '{' as ::core::ffi::c_int
                {
                    break;
                }
                p = p.offset(1);
            }
            mb_copy_char(&raw mut p as *mut *const ::core::ffi::c_char, &raw mut end);
        }
    }
    *end = NUL as ::core::ffi::c_char;
    if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int && !interpolate {
        p = p.offset(1);
    }
    *arg = p;
    return OK;
}
unsafe extern "C" fn eval_lit_string(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut reduce: ::core::ffi::c_int = if interpolate as ::core::ffi::c_int != 0 {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let off: ::core::ffi::c_int = if interpolate as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    p = (*arg).offset(off as isize);
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\'' as ::core::ffi::c_int
            {
                break;
            }
            reduce += 1;
            p = p.offset(1);
        } else if interpolate {
            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '{' as ::core::ffi::c_int
                {
                    break;
                }
                p = p.offset(1);
                reduce += 1;
            } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                p = p.offset(1);
                if *p as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_stray_closing_curly_str as *const ::core::ffi::c_char),
                        *arg,
                    );
                    return FAIL;
                }
                reduce += 1;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
        && !(interpolate as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int)
    {
        semsg(
            gettext(b"E115: Missing quote: %s\0".as_ptr() as *const ::core::ffi::c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    let mut str: *mut ::core::ffi::c_char =
        xmalloc((p.offset_from(*arg) - reduce as isize) as size_t) as *mut ::core::ffi::c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = str;
    p = (*arg).offset(off as isize);
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\'' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
        } else if interpolate as ::core::ffi::c_int != 0
            && (*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int)
        {
            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '{' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
        }
        mb_copy_char(&raw mut p as *mut *const ::core::ffi::c_char, &raw mut str);
    }
    *str = NUL as ::core::ffi::c_char;
    *arg = p.offset(off as isize);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn eval_interp_string(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    *arg = (*arg).offset(1);
    let quote: ::core::ffi::c_int = **arg as uint8_t as ::core::ffi::c_int;
    *arg = (*arg).offset(1);
    loop {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if quote == '"' as ::core::ffi::c_int {
            ret = eval_string(arg, &raw mut tv, evaluate, true_0 != 0);
        } else {
            ret = eval_lit_string(arg, &raw mut tv, evaluate, true_0 != 0);
        }
        if ret == FAIL {
            break;
        }
        if evaluate {
            ga_concat(&raw mut ga, tv.vval.v_string);
            tv_clear(&raw mut tv);
        }
        if **arg as ::core::ffi::c_int != '{' as ::core::ffi::c_int {
            *arg = (*arg).offset(1);
            break;
        } else {
            let mut p: *mut ::core::ffi::c_char = eval_one_expr_in_str(*arg, &raw mut ga, evaluate);
            if p.is_null() {
                ret = FAIL;
                break;
            } else {
                *arg = p;
            }
        }
    }
    (*rettv).v_type = VAR_STRING;
    if ret != FAIL && evaluate as ::core::ffi::c_int != 0 {
        ga_append(&raw mut ga, NUL as uint8_t);
    }
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn partial_name(mut pt: *mut partial_T) -> *mut ::core::ffi::c_char {
    if !pt.is_null() {
        if !(*pt).pt_name.is_null() {
            return (*pt).pt_name;
        }
        if !(*pt).pt_func.is_null() {
            return &raw mut (*(*pt).pt_func).uf_name as *mut ::core::ffi::c_char;
        }
    }
    return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn partial_free(mut pt: *mut partial_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*pt).pt_argc {
        tv_clear((*pt).pt_argv.offset(i as isize));
        i += 1;
    }
    xfree((*pt).pt_argv as *mut ::core::ffi::c_void);
    tv_dict_unref((*pt).pt_dict);
    if !(*pt).pt_name.is_null() {
        func_unref((*pt).pt_name);
        xfree((*pt).pt_name as *mut ::core::ffi::c_void);
    } else {
        func_ptr_unref((*pt).pt_func);
    }
    xfree(pt as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn partial_unref(mut pt: *mut partial_T) {
    if pt.is_null() {
        return;
    }
    (*pt).pt_refcount -= 1;
    if (*pt).pt_refcount <= 0 as ::core::ffi::c_int {
        partial_free(pt);
    }
}
unsafe extern "C" fn eval_list(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
    } != 0;
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if evaluate {
        l = tv_list_alloc(kListLenShouldKnow as ::core::ffi::c_int as ptrdiff_t);
    }
    *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    '_failret: {
        while **arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int
            && **arg as ::core::ffi::c_int != NUL
        {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval1(arg, &raw mut tv, evalarg) == FAIL {
                break '_failret;
            }
            if evaluate {
                tv.v_lock = VAR_UNLOCKED;
                tv_list_append_owned_tv(l, tv);
            }
            let mut had_comma: bool = **arg as ::core::ffi::c_int == ',' as ::core::ffi::c_int;
            if had_comma {
                *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            }
            if **arg as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                break;
            }
            if had_comma {
                continue;
            }
            semsg(
                gettext(b"E696: Missing comma in List: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *arg,
            );
            break '_failret;
        }
        if **arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
            semsg(gettext(e_list_end), *arg);
        } else {
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            if evaluate {
                tv_list_set_ret(rettv, l);
            }
            return OK;
        }
    }
    if evaluate {
        tv_list_free(l);
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn func_equal(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut ic: bool,
) -> bool {
    let mut s1: *mut ::core::ffi::c_char = if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*tv1).vval.v_string
    } else {
        partial_name((*tv1).vval.v_partial)
    };
    if !s1.is_null() && *s1 as ::core::ffi::c_int == NUL {
        s1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut s2: *mut ::core::ffi::c_char = if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*tv2).vval.v_string
    } else {
        partial_name((*tv2).vval.v_partial)
    };
    if !s2.is_null() && *s2 as ::core::ffi::c_int == NUL {
        s2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if s1.is_null() || s2.is_null() {
        if s1 != s2 {
            return false_0 != 0;
        }
    } else if strcmp(s1, s2) != 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut d1: *mut dict_T = if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv1).vval.v_partial).pt_dict
    };
    let mut d2: *mut dict_T = if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv2).vval.v_partial).pt_dict
    };
    if d1.is_null() || d2.is_null() {
        if d1 != d2 {
            return false_0 != 0;
        }
    } else if !tv_dict_equal(d1, d2, ic) {
        return false_0 != 0;
    }
    let mut a1: ::core::ffi::c_int = if (*tv1).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        0 as ::core::ffi::c_int
    } else {
        (*(*tv1).vval.v_partial).pt_argc
    };
    let mut a2: ::core::ffi::c_int = if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        0 as ::core::ffi::c_int
    } else {
        (*(*tv2).vval.v_partial).pt_argc
    };
    if a1 != a2 {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < a1 {
        if !tv_equal(
            (*(*tv1).vval.v_partial).pt_argv.offset(i as isize),
            (*(*tv2).vval.v_partial).pt_argv.offset(i as isize),
            ic,
        ) {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_copyID() -> ::core::ffi::c_int {
    static mut current_copyID: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    current_copyID += COPYID_INC;
    return current_copyID;
}
#[no_mangle]
pub unsafe extern "C" fn garbage_collect(mut testing: bool) -> bool {
    let mut abort_0: bool = false_0 != 0;
    if !testing {
        want_garbage_collect = false_0 != 0;
        may_garbage_collect = false_0 != 0;
        garbage_collect_at_exit = false_0 != 0;
    }
    if exestack.ga_maxlen - exestack.ga_len > 500 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = exestack.ga_len / 2 as ::core::ffi::c_int;
        if n < exestack.ga_growsize {
            n = exestack.ga_growsize;
        }
        if exestack.ga_len + n < exestack.ga_maxlen {
            let mut new_len: size_t =
                (exestack.ga_itemsize as size_t).wrapping_mul((exestack.ga_len + n) as size_t);
            let mut pp: *mut ::core::ffi::c_char =
                xrealloc(exestack.ga_data, new_len) as *mut ::core::ffi::c_char;
            exestack.ga_maxlen = exestack.ga_len + n;
            exestack.ga_data = pp as *mut ::core::ffi::c_void;
        }
    }
    let copyID: ::core::ffi::c_int = get_copyID();
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_previous_funccal(copyID) as ::core::ffi::c_int != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || garbage_collect_scriptvars(copyID) as ::core::ffi::c_int != 0;
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_item(
                &raw mut (*buf).b_bufvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_callback,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_interrupt,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_cfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ofu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tsrfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ffu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        if !abort_0 && !(*buf).b_p_cpt_cb.is_null() {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_cpt_callbacks((*buf).b_p_cpt_cb, (*buf).b_p_cpt_count, copyID)
                    as ::core::ffi::c_int
                    != 0;
        }
        buf = (*buf).b_next;
    }
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_insexpand_funcs(copyID) as ::core::ffi::c_int != 0;
    abort_0 =
        abort_0 as ::core::ffi::c_int != 0 || set_ref_in_opfunc(copyID) as ::core::ffi::c_int != 0;
    abort_0 =
        abort_0 as ::core::ffi::c_int != 0 || set_ref_in_tagfunc(copyID) as ::core::ffi::c_int != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_findfunc(copyID) as ::core::ffi::c_int != 0;
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_item(
                    &raw mut (*wp).w_winvar.di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0;
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < aucmd_win_vec.size as ::core::ffi::c_int {
        if !(*aucmd_win_vec.items.offset(i as isize)).auc_win.is_null() {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_item(
                    &raw mut (*(*aucmd_win_vec.items.offset(i as isize)).auc_win)
                        .w_winvar
                        .di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0;
        }
        i += 1;
    }
    let mut reg_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
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
        if reg_iter.is_null() {
            break;
        }
    }
    let mut mark_iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
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
        let mut name_0: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        mark_iter = mark_global_iter(mark_iter, &raw mut name_0, &raw mut fm);
        if mark_iter.is_null() {
            break;
        }
    }
    let mut tp_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp_0.is_null() {
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_item(
                &raw mut (*tp_0).tp_winvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    abort_0 = abort_0 as ::core::ffi::c_int != 0 || garbage_collect_globvars(copyID) != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_call_stack(copyID) as ::core::ffi::c_int != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_functions(copyID) as ::core::ffi::c_int != 0;
    let mut data: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < channels.set.h.n_keys {
        data = *channels.values.offset(__i as isize) as *mut Channel;
        set_ref_in_callback_reader(
            &raw mut (*data).on_data,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback_reader(
            &raw mut (*data).on_stderr,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback(
            &raw mut (*data).on_exit,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i = __i.wrapping_add(1);
    }
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i_0: uint32_t = 0;
    __i_0 = 0 as uint32_t;
    while __i_0 < timers.set.h.n_keys {
        timer = *timers.values.offset(__i_0 as isize) as *mut timer_T;
        set_ref_in_callback(
            &raw mut (*timer).callback,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i_0 = __i_0.wrapping_add(1);
    }
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_func_args(copyID) as ::core::ffi::c_int != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || garbage_collect_vimvars(copyID) as ::core::ffi::c_int != 0;
    abort_0 = abort_0 as ::core::ffi::c_int != 0
        || set_ref_in_quickfix(copyID) as ::core::ffi::c_int != 0;
    let mut did_free: bool = false_0 != 0;
    if !abort_0 {
        did_free = free_unref_items(copyID) != 0;
        did_free = free_unref_funccal(copyID, testing as ::core::ffi::c_int) as ::core::ffi::c_int
            != 0
            || did_free as ::core::ffi::c_int != 0;
    } else if p_verbose > 0 as OptInt {
        verb_msg(gettext(
            b"Not enough memory to set references, garbage collection aborted!\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    }
    return did_free;
}
unsafe extern "C" fn free_unref_items(mut copyID: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut did_free: bool = false_0 != 0;
    tv_in_free_unref_items = true_0 != 0;
    let mut dd: *mut dict_T = gc_first_dict;
    while !dd.is_null() {
        if (*dd).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_contents(dd);
            did_free = true_0 != 0;
        }
        dd = (*dd).dv_used_next;
    }
    let mut ll: *mut list_T = gc_first_list;
    while !ll.is_null() {
        if tv_list_copyid(ll) & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll) {
            tv_list_free_contents(ll);
            did_free = true_0 != 0;
        }
        ll = (*ll).lv_used_next;
    }
    let mut dd_next: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut dd_0: *mut dict_T = gc_first_dict;
    while !dd_0.is_null() {
        dd_next = (*dd_0).dv_used_next;
        if (*dd_0).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_dict(dd_0);
        }
        dd_0 = dd_next;
    }
    let mut ll_next: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut ll_0: *mut list_T = gc_first_list;
    while !ll_0.is_null() {
        ll_next = (*ll_0).lv_used_next;
        if (*ll_0).lv_copyID & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll_0) {
            tv_list_free_list(ll_0);
        }
        ll_0 = ll_next;
    }
    tv_in_free_unref_items = false_0 != 0;
    return did_free as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_ht(
    mut ht: *mut hashtab_T,
    mut copyID: ::core::ffi::c_int,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut ht_stack: *mut ht_stack_T = ::core::ptr::null_mut::<ht_stack_T>();
    let mut cur_ht: *mut hashtab_T = ht;
    loop {
        if !abort_0 {
            let hiht_: *mut hashtab_T = cur_ht;
            let mut hitodo_: size_t = (*hiht_).ht_used;
            let mut hi: *mut hashitem_T = (*hiht_).ht_array;
            while hitodo_ != 0 {
                if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
                    hitodo_ = hitodo_.wrapping_sub(1);
                    abort_0 = abort_0 as ::core::ffi::c_int != 0
                        || set_ref_in_item(
                            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                            copyID,
                            &raw mut ht_stack,
                            list_stack,
                        ) as ::core::ffi::c_int
                            != 0;
                }
                hi = hi.offset(1);
            }
        }
        if ht_stack.is_null() {
            break;
        }
        cur_ht = (*ht_stack).ht;
        let mut tempitem: *mut ht_stack_T = ht_stack;
        ht_stack = (*ht_stack).prev as *mut ht_stack_T;
        xfree(tempitem as *mut ::core::ffi::c_void);
    }
    return abort_0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_list_items(
    mut l: *mut list_T,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut list_stack: *mut list_stack_T = ::core::ptr::null_mut::<list_stack_T>();
    let mut cur_l: *mut list_T = l;
    loop {
        let l_: *mut list_T = cur_l;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if abort_0 {
                    break;
                }
                abort_0 =
                    set_ref_in_item(&raw mut (*li).li_tv, copyID, ht_stack, &raw mut list_stack);
                li = (*li).li_next;
            }
        }
        if list_stack.is_null() {
            break;
        }
        cur_l = (*list_stack).list;
        let mut tempitem: *mut list_stack_T = list_stack;
        list_stack = (*list_stack).prev as *mut list_stack_T;
        xfree(tempitem as *mut ::core::ffi::c_void);
    }
    return abort_0;
}
unsafe extern "C" fn set_ref_in_item_dict(
    mut dd: *mut dict_T,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if dd.is_null() || (*dd).dv_copyID == copyID {
        return false_0 != 0;
    }
    (*dd).dv_copyID = copyID;
    if ht_stack.is_null() {
        return set_ref_in_ht(&raw mut (*dd).dv_hashtab, copyID, list_stack);
    }
    let newitem: *mut ht_stack_T = xmalloc(::core::mem::size_of::<ht_stack_T>()) as *mut ht_stack_T;
    (*newitem).ht = &raw mut (*dd).dv_hashtab;
    (*newitem).prev = *ht_stack as *mut ht_stack_S;
    *ht_stack = newitem;
    let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    let mut watcher: *mut DictWatcher = ::core::ptr::null_mut::<DictWatcher>();
    w = (*dd).watchers.next as *mut QUEUE;
    while w != &raw mut (*dd).watchers {
        let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
        watcher = tv_dict_watcher_node_data(w);
        set_ref_in_callback(&raw mut (*watcher).callback, copyID, ht_stack, list_stack);
        w = next;
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_item_list(
    mut ll: *mut list_T,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if ll.is_null() || (*ll).lv_copyID == copyID {
        return false_0 != 0;
    }
    (*ll).lv_copyID = copyID;
    if list_stack.is_null() {
        return set_ref_in_list_items(ll, copyID, ht_stack);
    }
    let newitem: *mut list_stack_T =
        xmalloc(::core::mem::size_of::<list_stack_T>()) as *mut list_stack_T;
    (*newitem).list = ll;
    (*newitem).prev = *list_stack as *mut list_stack_S;
    *list_stack = newitem;
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_item_partial(
    mut pt: *mut partial_T,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if pt.is_null() || (*pt).pt_copyID == copyID {
        return false_0 != 0;
    }
    (*pt).pt_copyID = copyID;
    let mut abort_0: bool = set_ref_in_func((*pt).pt_name, (*pt).pt_func, copyID);
    if !(*pt).pt_dict.is_null() {
        let mut dtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        dtv.v_type = VAR_DICT;
        dtv.vval.v_dict = (*pt).pt_dict;
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_item(&raw mut dtv, copyID, ht_stack, list_stack) as ::core::ffi::c_int
                != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*pt).pt_argc {
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_item(
                (*pt).pt_argv.offset(i as isize),
                copyID,
                ht_stack,
                list_stack,
            ) as ::core::ffi::c_int
                != 0;
        i += 1;
    }
    return abort_0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_item(
    mut tv: *mut typval_T,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    match (*tv).v_type as ::core::ffi::c_uint {
        5 => return set_ref_in_item_dict((*tv).vval.v_dict, copyID, ht_stack, list_stack),
        4 => return set_ref_in_item_list((*tv).vval.v_list, copyID, ht_stack, list_stack),
        3 => {
            abort_0 = set_ref_in_func(
                (*tv).vval.v_string,
                ::core::ptr::null_mut::<ufunc_T>(),
                copyID,
            );
        }
        9 => {
            return set_ref_in_item_partial((*tv).vval.v_partial, copyID, ht_stack, list_stack);
        }
        0 | 7 | 8 | 6 | 1 | 2 | 10 | _ => {}
    }
    return abort_0;
}
unsafe extern "C" fn get_literal_key(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut tv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(**arg as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && **arg as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || **arg as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && **arg as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(**arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        && **arg as ::core::ffi::c_int != '_' as ::core::ffi::c_int
        && **arg as ::core::ffi::c_int != '-' as ::core::ffi::c_int
    {
        return FAIL;
    }
    p = *arg;
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    (*tv).v_type = VAR_STRING;
    (*tv).vval.v_string = xmemdupz(
        *arg as *const ::core::ffi::c_void,
        p.offset_from(*arg) as size_t,
    ) as *mut ::core::ffi::c_char;
    *arg = skipwhite(p);
    return OK;
}
unsafe extern "C" fn eval_dict(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut literal: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
    } != 0;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curly_expr: *mut ::core::ffi::c_char =
        skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if *curly_expr as ::core::ffi::c_int != '}' as ::core::ffi::c_int
        && !literal
        && eval1(
            &raw mut curly_expr,
            &raw mut tv,
            ::core::ptr::null_mut::<evalarg_T>(),
        ) == OK
        && *skipwhite(curly_expr) as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        return NOTDONE;
    }
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if evaluate {
        d = tv_dict_alloc();
    }
    let mut tvkey: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tvkey.v_type = VAR_UNKNOWN;
    tv.v_type = VAR_UNKNOWN;
    *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    '_failret: {
        while **arg as ::core::ffi::c_int != '}' as ::core::ffi::c_int
            && **arg as ::core::ffi::c_int != NUL
        {
            if (if literal as ::core::ffi::c_int != 0 {
                get_literal_key(arg, &raw mut tvkey)
            } else {
                eval1(arg, &raw mut tvkey, evalarg)
            }) == FAIL
            {
                break '_failret;
            }
            if **arg as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
                semsg(
                    gettext(b"E720: Missing colon in Dictionary: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    *arg,
                );
                tv_clear(&raw mut tvkey);
                break '_failret;
            } else {
                if evaluate {
                    key = tv_get_string_buf_chk(
                        &raw mut tvkey,
                        &raw mut buf as *mut ::core::ffi::c_char,
                    ) as *mut ::core::ffi::c_char;
                    if key.is_null() {
                        tv_clear(&raw mut tvkey);
                        break '_failret;
                    }
                }
                *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
                if eval1(arg, &raw mut tv, evalarg) == FAIL {
                    tv_clear(&raw mut tvkey);
                    break '_failret;
                } else {
                    if evaluate {
                        let mut item: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
                        if !item.is_null() {
                            semsg(
                                gettext(b"E721: Duplicate key in Dictionary: \"%s\"\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                key,
                            );
                            tv_clear(&raw mut tvkey);
                            tv_clear(&raw mut tv);
                            break '_failret;
                        } else {
                            item = tv_dict_item_alloc(key);
                            (*item).di_tv = tv;
                            (*item).di_tv.v_lock = VAR_UNLOCKED;
                            if tv_dict_add(d, item) == FAIL {
                                tv_dict_item_free(item);
                            }
                        }
                    }
                    tv_clear(&raw mut tvkey);
                    let mut had_comma: bool =
                        **arg as ::core::ffi::c_int == ',' as ::core::ffi::c_int;
                    if had_comma {
                        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
                    }
                    if **arg as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        break;
                    }
                    if had_comma {
                        continue;
                    }
                    semsg(
                        gettext(b"E722: Missing comma in Dictionary: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *arg,
                    );
                    break '_failret;
                }
            }
        }
        if **arg as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
            semsg(
                gettext(b"E723: Missing end of Dictionary '}': %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                *arg,
            );
        } else {
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            if evaluate {
                tv_dict_set_ret(rettv, d);
            }
            return OK;
        }
    }
    if !d.is_null() {
        tv_dict_free(d);
    }
    return FAIL;
}
unsafe extern "C" fn eval_lit_dict(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = OK;
    if *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '{' as ::core::ffi::c_int
    {
        *arg = (*arg).offset(1);
        ret = eval_dict(arg, rettv, evalarg, true_0 != 0);
    } else {
        ret = NOTDONE;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn string2float(
    text: *const ::core::ffi::c_char,
    ret_value: *mut float_T,
) -> size_t {
    if strncasecmp(
        text as *mut ::core::ffi::c_char,
        b"inf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        3 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *ret_value = ::core::f32::INFINITY as float_T;
        return 3 as size_t;
    }
    if strncasecmp(
        text as *mut ::core::ffi::c_char,
        b"-inf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        4 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *ret_value = -::core::f32::INFINITY as float_T;
        return 4 as size_t;
    }
    if strncasecmp(
        text as *mut ::core::ffi::c_char,
        b"nan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        3 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *ret_value = ::core::f32::NAN as float_T;
        return 3 as size_t;
    }
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *ret_value = strtod(text, &raw mut s) as float_T;
    return s.offset_from(text) as size_t;
}
unsafe extern "C" fn eval_env_var(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evaluate: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    *arg = (*arg).offset(1);
    let mut name: *mut ::core::ffi::c_char = *arg;
    let mut len: ::core::ffi::c_int = get_env_len(arg as *mut *const ::core::ffi::c_char);
    if evaluate != 0 {
        if len == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        let mut cc: ::core::ffi::c_int = *name.offset(len as isize) as ::core::ffi::c_int;
        *name.offset(len as isize) = NUL as ::core::ffi::c_char;
        let mut string: *mut ::core::ffi::c_char = vim_getenv(name);
        if string.is_null() || *string as ::core::ffi::c_int == NUL {
            xfree(string as *mut ::core::ffi::c_void);
            string = expand_env_save(name.offset(-(1 as ::core::ffi::c_int as isize)));
            if !string.is_null() && *string as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut string as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                *ptr_;
            }
        }
        *name.offset(len as isize) = cc as ::core::ffi::c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = string;
        (*rettv).v_lock = VAR_UNLOCKED;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_to_argv(
    mut cmd_tv: *mut typval_T,
    mut cmd: *mut *const ::core::ffi::c_char,
    mut executable: *mut bool,
) -> *mut *mut ::core::ffi::c_char {
    if (*cmd_tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut cmd_str: *const ::core::ffi::c_char = tv_get_string(cmd_tv);
        if !cmd.is_null() {
            *cmd = cmd_str;
        }
        return shell_build_argv(cmd_str, ::core::ptr::null::<::core::ffi::c_char>());
    }
    if (*cmd_tv).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected String or List\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    let mut argl: *mut list_T = (*cmd_tv).vval.v_list;
    let mut argc: ::core::ffi::c_int = tv_list_len(argl);
    if argc == 0 {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    let mut arg0: *const ::core::ffi::c_char = tv_get_string_chk(
        &raw mut (*(tv_list_first as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(argl))
            .li_tv,
    );
    let mut exe_resolved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if arg0.is_null() || !os_can_exe(arg0, &raw mut exe_resolved, true_0 != 0) {
        if !arg0.is_null() && !executable.is_null() {
            let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                b"'%s' is not executable\0".as_ptr() as *const ::core::ffi::c_char,
                arg0,
            );
            semsg(
                gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            *executable = false_0 != 0;
        }
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    if !cmd.is_null() {
        *cmd = exe_resolved;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv: *mut *mut ::core::ffi::c_char = xcalloc(
        (argc as size_t).wrapping_add(1 as size_t),
        ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let l_: *const list_T = argl;
    if !l_.is_null() {
        let mut arg: *const listitem_T = (*l_).lv_first;
        while !arg.is_null() {
            let mut a: *const ::core::ffi::c_char = tv_get_string_chk(&raw const (*arg).li_tv);
            if a.is_null() {
                shell_free_argv(argv);
                xfree(exe_resolved as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
            }
            let c2rust_fresh11 = i;
            i = i + 1;
            let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh11 as isize);
            *c2rust_lvalue_ptr = xstrdup(a);
            arg = (*arg).li_next;
        }
    }
    xfree(*argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void);
    *argv.offset(0 as ::core::ffi::c_int as isize) = exe_resolved;
    return argv;
}
unsafe extern "C" fn string_to_list(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    keepempty: bool,
) -> *mut list_T {
    if !keepempty && *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == NL
    {
        len = len.wrapping_sub(1);
    }
    let list: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    encode_list_write(list as *mut ::core::ffi::c_void, str, len);
    return list;
}
unsafe extern "C" fn get_system_output_as_rettv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut retlist: bool,
) {
    let mut wait_time: proftime_T = 0;
    let mut profiling: bool = do_profiling == PROF_YES;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if check_secure() {
        return;
    }
    let mut input_len: ptrdiff_t = 0;
    let mut input: *mut ::core::ffi::c_char = save_tv_as_string(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut input_len,
        false_0 != 0,
        false_0 != 0,
    );
    if input_len < 0 as ptrdiff_t {
        '_c2rust_label: {
            if input.is_null() {
            } else {
                __assert_fail(
                    b"input == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    4731 as ::core::ffi::c_uint,
                    b"void get_system_output_as_rettv(typval_T *, typval_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    let mut executable: bool = true_0 != 0;
    let mut argv: *mut *mut ::core::ffi::c_char = tv_to_argv(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        &raw mut executable,
    );
    if argv.is_null() {
        if !executable {
            set_vim_var_nr(VV_SHELL_ERROR, -1 as varnumber_T);
        }
        xfree(input as *mut ::core::ffi::c_void);
        return;
    }
    if p_verbose > 3 as OptInt {
        let mut cmdstr: *mut ::core::ffi::c_char = shell_argv_to_str(argv);
        verbose_enter_scroll();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Executing command: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            cmdstr,
        );
        msg_puts(b"\n\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        xfree(cmdstr as *mut ::core::ffi::c_void);
    }
    if profiling {
        prof_child_enter(&raw mut wait_time);
    }
    let mut nread: size_t = 0 as size_t;
    let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_int = os_system(
        argv,
        input,
        input_len as size_t,
        &raw mut res,
        &raw mut nread,
    );
    if profiling {
        prof_child_exit(&raw mut wait_time);
    }
    xfree(input as *mut ::core::ffi::c_void);
    set_vim_var_nr(VV_SHELL_ERROR, status as varnumber_T);
    if res.is_null() {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        } else {
            (*rettv).vval.v_string = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
        }
        return;
    }
    if retlist {
        let mut keepempty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            keepempty = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int;
        }
        (*rettv).vval.v_list = string_to_list(res, nread, keepempty != 0);
        tv_list_ref((*rettv).vval.v_list);
        (*rettv).v_type = VAR_LIST;
        xfree(res as *mut ::core::ffi::c_void);
    } else {
        memchrsub(
            res as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            1 as ::core::ffi::c_char,
            nread,
        );
        (*rettv).vval.v_string = res;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_system(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_systemlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn callback_from_typval(
    callback: *mut Callback,
    arg: *const typval_T,
) -> bool {
    let mut r: ::core::ffi::c_int = OK;
    if (*arg).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*arg).vval.v_partial.is_null()
    {
        (*callback).data.partial = (*arg).vval.v_partial;
        (*(*callback).data.partial).pt_refcount += 1;
        (*callback).type_0 = kCallbackPartial;
    } else if (*arg).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*arg).vval.v_string.is_null()
        && ascii_isdigit(*(*arg).vval.v_string as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        r = FAIL;
    } else if (*arg).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*arg).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut name: *mut ::core::ffi::c_char = (*arg).vval.v_string;
        if name.is_null() {
            r = FAIL;
        } else if *name as ::core::ffi::c_int == NUL {
            (*callback).type_0 = kCallbackNone;
            (*callback).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            (*callback).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if (*arg).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*callback).data.funcref = get_scriptlocal_funcname(name);
            }
            if (*callback).data.funcref.is_null() {
                (*callback).data.funcref = xstrdup(name);
            }
            func_ref((*callback).data.funcref);
            (*callback).type_0 = kCallbackFuncref;
        }
    } else if nlua_is_table_from_lua(arg) {
        let mut name_0: *mut ::core::ffi::c_char = nlua_register_table_as_callable(arg);
        if !name_0.is_null() {
            (*callback).data.funcref = xstrdup(name_0);
            (*callback).type_0 = kCallbackFuncref;
        } else {
            r = FAIL;
        }
    } else if (*arg).v_type as ::core::ffi::c_uint
        == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*arg).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*arg).vval.v_number == 0 as varnumber_T
    {
        (*callback).type_0 = kCallbackNone;
        (*callback).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        r = FAIL;
    }
    if r == FAIL {
        emsg(gettext(
            b"E921: Invalid callback argument\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}
static mut callback_depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn get_callback_depth() -> ::core::ffi::c_int {
    return callback_depth;
}
#[no_mangle]
pub unsafe extern "C" fn callback_call(
    callback: *mut Callback,
    argcount_in: ::core::ffi::c_int,
    argvars_in: *mut typval_T,
    rettv: *mut typval_T,
) -> bool {
    if callback_depth as OptInt > p_mfd {
        emsg(gettext(
            &raw const e_command_too_recursive as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut args: Array = ARRAY_DICT_INIT;
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    };
    let mut len: ::core::ffi::c_int = 0;
    match (*callback).type_0 as ::core::ffi::c_uint {
        1 => {
            name = (*callback).data.funcref;
            len = strlen(name) as ::core::ffi::c_int;
            if len >= 6 as ::core::ffi::c_int
                && memcmp(
                    name as *const ::core::ffi::c_void,
                    b"v:lua.\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    6 as size_t,
                ) == 0
            {
                name = name.offset(6 as ::core::ffi::c_int as isize);
                len = check_luafunc_name(name, false_0 != 0);
                if len == 0 as ::core::ffi::c_int {
                    return false_0 != 0;
                }
                partial = get_vim_var_partial(VV_LUA);
            } else {
                partial = ::core::ptr::null_mut::<partial_T>();
            }
        }
        2 => {
            partial = (*callback).data.partial;
            name = partial_name(partial);
        }
        3 => {
            rv = nlua_call_ref(
                (*callback).data.luaref,
                ::core::ptr::null::<::core::ffi::c_char>(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            return rv.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && rv.data.boolean as ::core::ffi::c_int == true_0;
        }
        0 => return false_0 != 0,
        _ => {}
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    callback_depth += 1;
    let mut ret: ::core::ffi::c_int = call_func(
        name,
        -1 as ::core::ffi::c_int,
        rettv,
        argcount_in,
        argvars_in,
        &raw mut funcexe,
    );
    callback_depth -= 1;
    return ret != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_callback(
    mut callback: *mut Callback,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    match (*callback).type_0 as ::core::ffi::c_uint {
        2 => {
            tv.v_type = VAR_PARTIAL;
            tv.vval.v_partial = (*callback).data.partial;
            return set_ref_in_item(&raw mut tv, copyID, ht_stack, list_stack);
        }
        3 => {
            abort();
        }
        1 | 0 | _ => {}
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_callback_reader(
    mut reader: *mut CallbackReader,
    mut copyID: ::core::ffi::c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if set_ref_in_callback(&raw mut (*reader).cb, copyID, ht_stack, list_stack) {
        return true_0 != 0;
    }
    if !(*reader).self_0.is_null() {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_DICT;
        tv.vval.v_dict = (*reader).self_0;
        return set_ref_in_item(&raw mut tv, copyID, ht_stack, list_stack);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn find_timer_by_nr(mut xx: varnumber_T) -> *mut timer_T {
    return map_get_uint64_t_ptr_t(&raw mut timers, xx as uint64_t) as *mut timer_T;
}
#[no_mangle]
pub unsafe extern "C" fn add_timer_info(mut rettv: *mut typval_T, mut timer: *mut timer_T) {
    let mut list: *mut list_T = (*rettv).vval.v_list;
    let mut dict: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(list, dict);
    tv_dict_add_nr(
        dict,
        b"id\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        (*timer).timer_id as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"time\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*timer).timeout as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"paused\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*timer).paused as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"repeat\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (if (*timer).repeat_count < 0 as ::core::ffi::c_int {
            -1 as ::core::ffi::c_int
        } else {
            (*timer).repeat_count
        }) as varnumber_T,
    );
    let mut di: *mut dictitem_T =
        tv_dict_item_alloc(b"callback\0".as_ptr() as *const ::core::ffi::c_char);
    if tv_dict_add(dict, di) == FAIL {
        xfree(di as *mut ::core::ffi::c_void);
        return;
    }
    callback_put(&raw mut (*timer).callback, &raw mut (*di).di_tv);
}
#[no_mangle]
pub unsafe extern "C" fn add_timer_info_all(mut rettv: *mut typval_T) {
    tv_list_alloc_ret(rettv, timers.set.h.size as ptrdiff_t);
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < timers.set.h.n_keys {
        timer = *timers.values.offset(__i as isize) as *mut timer_T;
        if !(*timer).stopped || (*timer).refcount > 1 as ::core::ffi::c_int {
            add_timer_info(rettv, timer);
        }
        __i = __i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn timer_due_cb(
    mut _tw: *mut TimeWatcher,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg;
    let called_emsg_before: ::core::ffi::c_int = called_emsg;
    let save_ex_pressedreturn: bool = get_pressedreturn();
    if (*timer).stopped as ::core::ffi::c_int != 0 || (*timer).paused as ::core::ffi::c_int != 0 {
        return;
    }
    (*timer).refcount += 1;
    if (*timer).repeat_count >= 0 as ::core::ffi::c_int && {
        (*timer).repeat_count -= 1;
        (*timer).repeat_count == 0 as ::core::ffi::c_int
    } {
        timer_stop(timer);
    }
    let mut argv: [typval_T; 2] = [
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    argv[0 as ::core::ffi::c_int as usize].vval.v_number = (*timer).timer_id as varnumber_T;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(
        &raw mut (*timer).callback,
        1 as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    if called_emsg > called_emsg_before && did_emsg != 0 {
        (*timer).emsg_count += 1;
        if did_throw {
            discard_current_exception();
        }
    }
    did_emsg = save_did_emsg;
    set_pressedreturn(save_ex_pressedreturn);
    if (*timer).emsg_count >= 3 as ::core::ffi::c_int {
        timer_stop(timer);
    }
    tv_clear(&raw mut rettv);
    if !(*timer).stopped && (*timer).timeout == 0 as int64_t {
        time_watcher_start(
            &raw mut (*timer).tw,
            Some(
                timer_due_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
            0 as uint64_t,
            0 as uint64_t,
        );
    }
    timer_decref(timer);
}
#[no_mangle]
pub unsafe extern "C" fn timer_start(
    timeout: int64_t,
    repeat_count: ::core::ffi::c_int,
    callback: *const Callback,
) -> uint64_t {
    let mut timer: *mut timer_T = xmalloc(::core::mem::size_of::<timer_T>()) as *mut timer_T;
    (*timer).refcount = 1 as ::core::ffi::c_int;
    (*timer).stopped = false_0 != 0;
    (*timer).paused = false_0 != 0;
    (*timer).emsg_count = 0 as ::core::ffi::c_int;
    (*timer).repeat_count = repeat_count;
    (*timer).timeout = timeout;
    let c2rust_fresh17 = last_timer_id;
    last_timer_id = last_timer_id.wrapping_add(1);
    (*timer).timer_id = c2rust_fresh17 as ::core::ffi::c_int;
    (*timer).callback = *callback;
    time_watcher_init(
        &raw mut main_loop,
        &raw mut (*timer).tw,
        timer as *mut ::core::ffi::c_void,
    );
    (*timer).tw.events = multiqueue_new_child(main_loop.events);
    (*timer).tw.blockable = true_0 != 0;
    time_watcher_start(
        &raw mut (*timer).tw,
        Some(
            timer_due_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
        timeout as uint64_t,
        timeout as uint64_t,
    );
    map_put_uint64_t_ptr_t(
        &raw mut timers,
        (*timer).timer_id as uint64_t,
        timer as ptr_t,
    );
    return (*timer).timer_id as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn timer_stop(mut timer: *mut timer_T) {
    if (*timer).stopped {
        return;
    }
    (*timer).stopped = true_0 != 0;
    time_watcher_stop(&raw mut (*timer).tw);
    time_watcher_close(
        &raw mut (*timer).tw,
        Some(
            timer_close_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
}
unsafe extern "C" fn timer_close_cb(mut _tw: *mut TimeWatcher, mut data: *mut ::core::ffi::c_void) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    multiqueue_free((*timer).tw.events);
    callback_free(&raw mut (*timer).callback);
    map_del_uint64_t_ptr_t(
        &raw mut timers,
        (*timer).timer_id as uint64_t,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    timer_decref(timer);
}
unsafe extern "C" fn timer_decref(mut timer: *mut timer_T) {
    (*timer).refcount -= 1;
    if (*timer).refcount == 0 as ::core::ffi::c_int {
        xfree(timer as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn timer_stop_all() {
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < timers.set.h.n_keys {
        timer = *timers.values.offset(__i as isize) as *mut timer_T;
        timer_stop(timer);
        __i = __i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn timer_teardown() {
    timer_stop_all();
}
#[no_mangle]
pub unsafe extern "C" fn save_tv_as_string(
    mut tv: *mut typval_T,
    len: *mut ptrdiff_t,
    mut endnl: bool,
    mut crlf: bool,
) -> *mut ::core::ffi::c_char {
    *len = 0 as ptrdiff_t;
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*tv).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*tv).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut ret: *const ::core::ffi::c_char = tv_get_string_chk(tv);
        if !ret.is_null() {
            *len = strlen(ret) as ptrdiff_t;
            return xmemdupz(ret as *const ::core::ffi::c_void, *len as size_t)
                as *mut ::core::ffi::c_char;
        } else {
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut buf: *mut buf_T = buflist_findnr((*tv).vval.v_number as ::core::ffi::c_int);
        if !buf.is_null() {
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= (*buf).b_ml.ml_line_count {
                let mut p: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
                while *p as ::core::ffi::c_int != NUL {
                    *len += 1 as ptrdiff_t;
                    p = p.offset(1);
                }
                *len += 1 as ptrdiff_t;
                lnum += 1;
            }
        } else {
            semsg(
                gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                (*tv).vval.v_number,
            );
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if *len == 0 as ptrdiff_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut ret_0: *mut ::core::ffi::c_char =
            xmalloc((*len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut end: *mut ::core::ffi::c_char = ret_0;
        let mut lnum_0: linenr_T = 1 as linenr_T;
        while lnum_0 <= (*buf).b_ml.ml_line_count {
            let mut p_0: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum_0);
            while *p_0 as ::core::ffi::c_int != NUL {
                let c2rust_fresh12 = end;
                end = end.offset(1);
                *c2rust_fresh12 = (if *p_0 as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                    NUL
                } else {
                    *p_0 as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                p_0 = p_0.offset(1);
            }
            let c2rust_fresh13 = end;
            end = end.offset(1);
            *c2rust_fresh13 = '\n' as ::core::ffi::c_char;
            lnum_0 += 1;
        }
        *end = NUL as ::core::ffi::c_char;
        *len = end.offset_from(ret_0) as ptrdiff_t;
        return ret_0;
    }
    '_c2rust_label: {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_LIST\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                5197 as ::core::ffi::c_uint,
                b"char *save_tv_as_string(typval_T *, ptrdiff_t *const, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut list: *mut list_T = (*tv).vval.v_list;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            *len += strlen(tv_get_string(&raw const (*li).li_tv)) as ptrdiff_t
                + (if crlf as ::core::ffi::c_int != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as ptrdiff_t;
            li = (*li).li_next;
        }
    }
    if *len == 0 as ptrdiff_t {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut ret_1: *mut ::core::ffi::c_char = xmalloc((*len as size_t).wrapping_add(
        (if endnl as ::core::ffi::c_int != 0 {
            if crlf as ::core::ffi::c_int != 0 {
                2 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }
        } else {
            0 as ::core::ffi::c_int
        }) as size_t,
    )) as *mut ::core::ffi::c_char;
    let mut end_0: *mut ::core::ffi::c_char = ret_1;
    let l__0: *const list_T = list;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut s: *const ::core::ffi::c_char = tv_get_string(&raw const (*li_0).li_tv);
            while *s as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                let c2rust_fresh14 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh14 = (if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                    '\0' as ::core::ffi::c_int
                } else {
                    *s as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                s = s.offset(1);
            }
            if endnl as ::core::ffi::c_int != 0 || !(*li_0).li_next.is_null() {
                if crlf {
                    let c2rust_fresh15 = end_0;
                    end_0 = end_0.offset(1);
                    *c2rust_fresh15 = '\r' as ::core::ffi::c_char;
                }
                let c2rust_fresh16 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh16 = '\n' as ::core::ffi::c_char;
            }
            li_0 = (*li_0).li_next;
        }
    }
    *end_0 = NUL as ::core::ffi::c_char;
    *len = end_0.offset_from(ret_1) as ptrdiff_t;
    return ret_1;
}
#[no_mangle]
pub unsafe extern "C" fn buf_byteidx_to_charidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut byteidx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
    if *str as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    let mut t: *mut ::core::ffi::c_char = str;
    let mut count: ::core::ffi::c_int = 0;
    count = 0 as ::core::ffi::c_int;
    while *t as ::core::ffi::c_int != NUL && t <= str.offset(byteidx as isize) {
        t = t.offset(utfc_ptr2len(t) as isize);
        count += 1;
    }
    if *t as ::core::ffi::c_int == NUL
        && byteidx != 0 as ::core::ffi::c_int
        && t == str.offset(byteidx as isize)
    {
        count += 1;
    }
    return count - 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn buf_charidx_to_byteidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut charidx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
    let mut t: *mut ::core::ffi::c_char = str;
    while *t as ::core::ffi::c_int != NUL && {
        charidx -= 1;
        charidx > 0 as ::core::ffi::c_int
    } {
        t = t.offset(utfc_ptr2len(t) as isize);
    }
    return t.offset_from(str) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn var2fpos(
    tv: *const typval_T,
    dollar_lnum: bool,
    ret_fnum: *mut ::core::ffi::c_int,
    charcol: bool,
    mut wp: *mut win_T,
) -> *mut pos_T {
    static mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut bp: *mut buf_T = (*wp).w_buffer;
    if (*tv).v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let mut l: *mut list_T = (*tv).vval.v_list;
        if l.is_null() {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos.lnum = tv_list_find_nr(l, 0 as ::core::ffi::c_int, &raw mut error) as linenr_T;
        if error as ::core::ffi::c_int != 0
            || pos.lnum <= 0 as linenr_T
            || pos.lnum > (*bp).b_ml.ml_line_count
        {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos.col = tv_list_find_nr(l, 1 as ::core::ffi::c_int, &raw mut error) as colnr_T;
        if error {
            return ::core::ptr::null_mut::<pos_T>();
        }
        let mut len: ::core::ffi::c_int = 0;
        if charcol {
            len = mb_charlen(ml_get_buf(bp, pos.lnum));
        } else {
            len = ml_get_buf_len(bp, pos.lnum) as ::core::ffi::c_int;
        }
        let mut li: *mut listitem_T = tv_list_find(l, 1 as ::core::ffi::c_int);
        if !li.is_null()
            && (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*li).li_tv.vval.v_string.is_null()
            && strcmp(
                (*li).li_tv.vval.v_string,
                b"$\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            pos.col = (len + 1 as ::core::ffi::c_int) as colnr_T;
        }
        if pos.col == 0 as ::core::ffi::c_int || pos.col > len + 1 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos.col -= 1;
        pos.coladd = tv_list_find_nr(l, 2 as ::core::ffi::c_int, &raw mut error) as colnr_T;
        if error {
            pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
        return &raw mut pos;
    }
    let name: *const ::core::ffi::c_char = tv_get_string_chk(tv);
    if name.is_null() {
        return ::core::ptr::null_mut::<pos_T>();
    }
    pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
    {
        pos = (*wp).w_cursor;
    } else if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'v' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if VIsual_active as ::core::ffi::c_int != 0 && wp == curwin {
            pos = VIsual;
        } else {
            pos = (*wp).w_cursor;
        }
    } else if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\'' as ::core::ffi::c_int
    {
        let mut mname: ::core::ffi::c_int =
            *name.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
        let fm: *const fmark_T =
            mark_get(bp, wp, ::core::ptr::null_mut::<fmark_T>(), kMarkAll, mname);
        if fm.is_null() || (*fm).mark.lnum <= 0 as linenr_T {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos = (*fm).mark;
        *ret_fnum = if mname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && mname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || ascii_isdigit(mname) as ::core::ffi::c_int != 0
        {
            (*fm).fnum
        } else {
            *ret_fnum
        };
    }
    if pos.lnum != 0 as linenr_T {
        if charcol {
            pos.col =
                buf_byteidx_to_charidx(bp, pos.lnum, pos.col as ::core::ffi::c_int) as colnr_T;
        }
        return &raw mut pos;
    }
    pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'w' as ::core::ffi::c_int
        && dollar_lnum as ::core::ffi::c_int != 0
    {
        check_cursor_moved(wp);
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
        if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '0' as ::core::ffi::c_int
        {
            update_topline(wp);
            pos.lnum = if (*wp).w_topline > 0 as linenr_T {
                (*wp).w_topline
            } else {
                1 as linenr_T
            };
            return &raw mut pos;
        } else if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '$' as ::core::ffi::c_int
        {
            validate_botline_win(wp);
            pos.lnum = if (*wp).w_botline > 0 as linenr_T {
                (*wp).w_botline - 1 as linenr_T
            } else {
                0 as linenr_T
            };
            return &raw mut pos;
        }
    } else if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '$' as ::core::ffi::c_int
    {
        if dollar_lnum {
            pos.lnum = (*bp).b_ml.ml_line_count;
            pos.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            pos.lnum = (*wp).w_cursor.lnum;
            if charcol {
                pos.col = mb_charlen(ml_get_buf(bp, (*wp).w_cursor.lnum));
            } else {
                pos.col = ml_get_buf_len(bp, (*wp).w_cursor.lnum);
            }
        }
        return &raw mut pos;
    }
    return ::core::ptr::null_mut::<pos_T>();
}
#[no_mangle]
pub unsafe extern "C" fn list2fpos(
    mut arg: *mut typval_T,
    mut posp: *mut pos_T,
    mut fnump: *mut ::core::ffi::c_int,
    mut curswantp: *mut colnr_T,
    mut charcol: bool,
) -> ::core::ffi::c_int {
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if (*arg).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || {
            l = (*arg).vval.v_list;
            l.is_null()
        }
        || tv_list_len(l)
            < (if fnump.is_null() {
                2 as ::core::ffi::c_int
            } else {
                3 as ::core::ffi::c_int
            })
        || tv_list_len(l)
            > (if fnump.is_null() {
                4 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            })
    {
        return FAIL;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    if !fnump.is_null() {
        let c2rust_fresh18 = i;
        i = i + 1;
        n = tv_list_find_nr(l, c2rust_fresh18, ::core::ptr::null_mut::<bool>())
            as ::core::ffi::c_int;
        if n < 0 as ::core::ffi::c_int {
            return FAIL;
        }
        if n == 0 as ::core::ffi::c_int {
            n = (*curbuf).handle as ::core::ffi::c_int;
        }
        *fnump = n;
    }
    let c2rust_fresh19 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh19, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if n < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    (*posp).lnum = n as linenr_T;
    let c2rust_fresh20 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh20, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if n < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    if charcol {
        let mut buf: *mut buf_T = buflist_findnr(if fnump.is_null() {
            (*curbuf).handle as ::core::ffi::c_int
        } else {
            *fnump
        });
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return FAIL;
        }
        n = buf_charidx_to_byteidx(
            buf,
            if (*posp).lnum == 0 as linenr_T {
                (*curwin).w_cursor.lnum
            } else {
                (*posp).lnum
            },
            n,
        ) + 1 as ::core::ffi::c_int;
    }
    (*posp).col = n as colnr_T;
    n = tv_list_find_nr(l, i, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if n < 0 as ::core::ffi::c_int {
        (*posp).coladd = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        (*posp).coladd = n as colnr_T;
    }
    if !curswantp.is_null() {
        *curswantp = tv_list_find_nr(
            l,
            i + 1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
        ) as colnr_T;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_env_len(
    mut arg: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = *arg;
    while vim_isIDc(*p as uint8_t as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = p.offset_from(*arg) as ::core::ffi::c_int;
    *arg = p;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn get_id_len(arg: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = *arg;
    while eval_isnamec(*p as ::core::ffi::c_int) {
        if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            len = p.offset_from(*arg) as ::core::ffi::c_int;
            if len > 1 as ::core::ffi::c_int
                || len == 1 as ::core::ffi::c_int
                    && vim_strchr(namespace_char, **arg as uint8_t as ::core::ffi::c_int).is_null()
            {
                break;
            }
        }
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as ::core::ffi::c_int;
    }
    len = p.offset_from(*arg) as ::core::ffi::c_int;
    *arg = skipwhite(p);
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn get_name_len(
    arg: *mut *const ::core::ffi::c_char,
    mut alias: *mut *mut ::core::ffi::c_char,
    mut evaluate: bool,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    *alias = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *(*arg).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == K_SPECIAL as ::core::ffi::c_char as ::core::ffi::c_int
        && *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == KS_EXTRA as ::core::ffi::c_char as ::core::ffi::c_int
        && *(*arg).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char as ::core::ffi::c_int
    {
        *arg = (*arg).offset(3 as ::core::ffi::c_int as isize);
        return get_id_len(arg) + 3 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = eval_fname_script(*arg);
    if len > 0 as ::core::ffi::c_int {
        *arg = (*arg).offset(len as isize);
    }
    let mut expr_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expr_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = find_name_end(
        *arg,
        &raw mut expr_start as *mut *const ::core::ffi::c_char,
        &raw mut expr_end as *mut *const ::core::ffi::c_char,
        if len > 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            FNE_CHECK_START
        },
    );
    if !expr_start.is_null() {
        if !evaluate {
            len += p.offset_from(*arg) as ::core::ffi::c_int;
            *arg = skipwhite(p);
            return len;
        }
        let mut temp_string: *mut ::core::ffi::c_char = make_expanded_name(
            (*arg).offset(-(len as isize)),
            expr_start,
            expr_end,
            p as *mut ::core::ffi::c_char,
        );
        if temp_string.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        *alias = temp_string;
        *arg = skipwhite(p);
        return strlen(temp_string) as ::core::ffi::c_int;
    }
    len += get_id_len(arg);
    if len == 0 as ::core::ffi::c_int
        && verbose as ::core::ffi::c_int != 0
        && **arg as ::core::ffi::c_int != NUL
    {
        semsg(
            gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
            *arg,
        );
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn find_name_end(
    mut arg: *const ::core::ffi::c_char,
    mut expr_start: *mut *const ::core::ffi::c_char,
    mut expr_end: *mut *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    if !expr_start.is_null() {
        *expr_start = ::core::ptr::null::<::core::ffi::c_char>();
        *expr_end = ::core::ptr::null::<::core::ffi::c_char>();
    }
    if flags & FNE_CHECK_START != 0
        && !eval_isnamec1(*arg as ::core::ffi::c_int)
        && *arg as ::core::ffi::c_int != '{' as ::core::ffi::c_int
    {
        return arg;
    }
    let mut mb_nest: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut br_nest: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = arg;
    while *p as ::core::ffi::c_int != NUL
        && (eval_isnamec(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
            || flags & FNE_INCL_BR != 0
                && (*p as ::core::ffi::c_int == '[' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                        && eval_isdictc(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0)
            || mb_nest != 0 as ::core::ffi::c_int
            || br_nest != 0 as ::core::ffi::c_int)
    {
        if *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
            p = p.offset(1 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != NUL
                && *p as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
            {
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            p = p.offset(1 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != NUL
                && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
            {
                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                }
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        } else if br_nest == 0 as ::core::ffi::c_int
            && mb_nest == 0 as ::core::ffi::c_int
            && *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        {
            len = p.offset_from(arg) as ::core::ffi::c_int;
            if len > 1 as ::core::ffi::c_int
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '}' as ::core::ffi::c_int
                || len == 1 as ::core::ffi::c_int
                    && vim_strchr(namespace_char, *arg as uint8_t as ::core::ffi::c_int).is_null()
            {
                break;
            }
        }
        if mb_nest == 0 as ::core::ffi::c_int {
            if *p as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                br_nest += 1;
            } else if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                br_nest -= 1;
            }
        }
        if br_nest == 0 as ::core::ffi::c_int {
            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                mb_nest += 1;
                if !expr_start.is_null() && (*expr_start).is_null() {
                    *expr_start = p;
                }
            } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                mb_nest -= 1;
                if !expr_start.is_null()
                    && mb_nest == 0 as ::core::ffi::c_int
                    && (*expr_end).is_null()
                {
                    *expr_end = p;
                }
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return p;
}
unsafe extern "C" fn make_expanded_name(
    mut in_start: *const ::core::ffi::c_char,
    mut expr_start: *mut ::core::ffi::c_char,
    mut expr_end: *mut ::core::ffi::c_char,
    mut in_end: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if expr_end.is_null() || in_end.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *expr_start = NUL as ::core::ffi::c_char;
    *expr_end = NUL as ::core::ffi::c_char;
    let mut c1: ::core::ffi::c_char = *in_end;
    *in_end = NUL as ::core::ffi::c_char;
    let mut temp_result: *mut ::core::ffi::c_char = eval_to_string(
        expr_start.offset(1 as ::core::ffi::c_int as isize),
        false_0 != 0,
        false_0 != 0,
    );
    if !temp_result.is_null() {
        let mut retvalsize: size_t = (expr_start.offset_from(in_start) as size_t)
            .wrapping_add(strlen(temp_result))
            .wrapping_add(in_end.offset_from(expr_end) as size_t)
            .wrapping_add(1 as size_t);
        retval = xmalloc(retvalsize) as *mut ::core::ffi::c_char;
        vim_snprintf(
            retval,
            retvalsize,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            in_start,
            temp_result,
            expr_end.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    xfree(temp_result as *mut ::core::ffi::c_void);
    *in_end = c1;
    *expr_start = '{' as ::core::ffi::c_char;
    *expr_end = '}' as ::core::ffi::c_char;
    if !retval.is_null() {
        temp_result = find_name_end(
            retval,
            &raw mut expr_start as *mut *const ::core::ffi::c_char,
            &raw mut expr_end as *mut *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_char;
        if !expr_start.is_null() {
            temp_result = make_expanded_name(retval, expr_start, expr_end, temp_result);
            xfree(retval as *mut ::core::ffi::c_void);
            retval = temp_result;
        }
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn eval_isnamec(mut c: ::core::ffi::c_int) -> bool {
    return c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || c == '_' as ::core::ffi::c_int
        || c == ':' as ::core::ffi::c_int
        || c == AUTOLOAD_CHAR;
}
#[no_mangle]
pub unsafe extern "C" fn eval_isnamec1(mut c: ::core::ffi::c_int) -> bool {
    return c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || c == '_' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn eval_isdictc(mut c: ::core::ffi::c_int) -> bool {
    return c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || c == '_' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn set_argv_var(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
) {
    let mut l: *mut list_T = tv_list_alloc(argc as ptrdiff_t);
    tv_list_set_lock(l, VAR_FIXED);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < argc {
        tv_list_append_string(
            l,
            *argv.offset(i as isize) as *const ::core::ffi::c_char,
            -1 as ssize_t,
        );
        (*tv_list_last(l)).li_tv.v_lock = VAR_FIXED;
        i += 1;
    }
    set_vim_var_list(VV_ARGV, l);
}
#[no_mangle]
pub unsafe extern "C" fn is_luafunc(mut partial: *mut partial_T) -> bool {
    return partial == get_vim_var_partial(VV_LUA);
}
unsafe extern "C" fn tv_is_luafunc(mut tv: *mut typval_T) -> bool {
    return (*tv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && is_luafunc((*tv).vval.v_partial) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn skip_luafunc_name(
    mut p: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn check_luafunc_name(
    str: *const ::core::ffi::c_char,
    paren: bool,
) -> ::core::ffi::c_int {
    let p: *const ::core::ffi::c_char = skip_luafunc_name(str);
    if *p as ::core::ffi::c_int
        != (if paren as ::core::ffi::c_int != 0 {
            '(' as ::core::ffi::c_int
        } else {
            NUL
        })
    {
        return 0 as ::core::ffi::c_int;
    }
    return p.offset_from(str) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn char_from_string(
    mut str: *const ::core::ffi::c_char,
    mut index: varnumber_T,
) -> *mut ::core::ffi::c_char {
    let mut nchar: varnumber_T = index;
    if str.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut slen: size_t = strlen(str);
    if index < 0 as varnumber_T {
        let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut nbyte: size_t = 0 as size_t;
        while nbyte < slen {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            clen += 1;
        }
        nchar = clen as varnumber_T + index;
        if nchar < 0 as varnumber_T {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    let mut nbyte_0: size_t = 0 as size_t;
    while nchar > 0 as varnumber_T && nbyte_0 < slen {
        nbyte_0 = nbyte_0.wrapping_add(utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t);
        nchar -= 1;
    }
    if nbyte_0 >= slen {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xmemdupz(
        str.offset(nbyte_0 as isize) as *const ::core::ffi::c_void,
        utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t,
    ) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn char_idx2byte(
    mut str: *const ::core::ffi::c_char,
    mut str_len: size_t,
    mut idx: varnumber_T,
) -> ssize_t {
    let mut nchar: varnumber_T = idx;
    let mut nbyte: size_t = 0 as size_t;
    if nchar >= 0 as varnumber_T {
        while nchar > 0 as varnumber_T && nbyte < str_len {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 as varnumber_T && nbyte > 0 as size_t {
            nbyte = nbyte.wrapping_sub(1);
            nbyte = nbyte.wrapping_sub(utf_head_off(str, str.offset(nbyte as isize)) as size_t);
            nchar += 1;
        }
        if nchar < 0 as varnumber_T {
            return -1 as ssize_t;
        }
    }
    return nbyte as ssize_t;
}
#[no_mangle]
pub unsafe extern "C" fn string_slice(
    mut str: *const ::core::ffi::c_char,
    mut first: varnumber_T,
    mut last: varnumber_T,
    mut exclusive: bool,
) -> *mut ::core::ffi::c_char {
    if str.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut slen: size_t = strlen(str);
    let mut start_byte: ssize_t = char_idx2byte(str, slen, first);
    if start_byte < 0 as ssize_t {
        start_byte = 0 as ssize_t;
    }
    let mut end_byte: ssize_t = 0;
    if last == -1 as varnumber_T && !exclusive || last == VARNUMBER_MAX as varnumber_T {
        end_byte = slen as ssize_t;
    } else {
        end_byte = char_idx2byte(str, slen, last);
        if !exclusive && end_byte >= 0 as ssize_t && end_byte < slen as ssize_t {
            end_byte += utfc_ptr2len(str.offset(end_byte as isize)) as ssize_t;
        }
    }
    if start_byte >= slen as ssize_t || end_byte <= start_byte {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xmemdupz(
        str.offset(start_byte as isize) as *const ::core::ffi::c_void,
        (end_byte - start_byte) as size_t,
    ) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn handle_subscript(
    arg: *mut *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut ret: ::core::ffi::c_int = OK;
    let mut selfdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut lua_funcname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if tv_is_luafunc(rettv) {
        if !evaluate {
            tv_clear(rettv);
        }
        if **arg as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
            tv_clear(rettv);
            ret = FAIL;
        } else {
            *arg = (*arg).offset(1);
            lua_funcname = *arg;
            let len: ::core::ffi::c_int = check_luafunc_name(*arg, true_0 != 0);
            if len == 0 as ::core::ffi::c_int {
                tv_clear(rettv);
                ret = FAIL;
            }
            *arg = (*arg).offset(len as isize);
        }
    }
    while ret == OK
        && ((**arg as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            || **arg as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            || **arg as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                && (!evaluate || tv_is_func(*rettv) as ::core::ffi::c_int != 0))
            && !ascii_iswhite(
                *(*arg).offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            )
            || **arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                && *(*arg).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '>' as ::core::ffi::c_int)
    {
        if **arg as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
            ret = call_func_rettv(
                arg as *mut *mut ::core::ffi::c_char,
                evalarg,
                rettv,
                evaluate,
                selfdict,
                ::core::ptr::null_mut::<typval_T>(),
                lua_funcname,
            );
            if aborting() {
                if ret == OK {
                    tv_clear(rettv);
                }
                ret = FAIL;
            }
            tv_dict_unref(selfdict);
            selfdict = ::core::ptr::null_mut::<dict_T>();
        } else if **arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            if *(*arg).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '{' as ::core::ffi::c_int
            {
                ret = eval_lambda(
                    arg as *mut *mut ::core::ffi::c_char,
                    rettv,
                    evalarg,
                    verbose,
                );
            } else {
                ret = eval_method(
                    arg as *mut *mut ::core::ffi::c_char,
                    rettv,
                    evalarg,
                    verbose,
                );
            }
        } else {
            tv_dict_unref(selfdict);
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                selfdict = (*rettv).vval.v_dict;
                if !selfdict.is_null() {
                    (*selfdict).dv_refcount += 1;
                }
            } else {
                selfdict = ::core::ptr::null_mut::<dict_T>();
            }
            if eval_index(
                arg as *mut *mut ::core::ffi::c_char,
                rettv,
                evalarg,
                verbose,
            ) == FAIL
            {
                tv_clear(rettv);
                ret = FAIL;
            }
        }
    }
    if !selfdict.is_null() && tv_is_func(*rettv) as ::core::ffi::c_int != 0 {
        set_selfdict(rettv, selfdict);
    }
    tv_dict_unref(selfdict);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T) {
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*(*rettv).vval.v_partial).pt_auto
        && !(*(*rettv).vval.v_partial).pt_dict.is_null()
    {
        return;
    }
    make_partial(selfdict, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn var_item_copy(
    conv: *const vimconv_T,
    from: *mut typval_T,
    to: *mut typval_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static mut recurse: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int = OK;
    if recurse >= DICT_MAXNEST {
        emsg(gettext(
            &raw const e_variable_nested_too_deep_for_making_copy as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    recurse += 1;
    match (*from).v_type as ::core::ffi::c_uint {
        1 | 6 | 3 | 9 | 7 | 8 => {
            tv_copy(from, to);
        }
        2 => {
            if conv.is_null()
                || (*conv).vc_type == CONV_NONE as ::core::ffi::c_int
                || (*from).vval.v_string.is_null()
            {
                tv_copy(from, to);
            } else {
                (*to).v_type = VAR_STRING;
                (*to).v_lock = VAR_UNLOCKED;
                (*to).vval.v_string = string_convert(
                    conv as *mut vimconv_T,
                    (*from).vval.v_string,
                    ::core::ptr::null_mut::<size_t>(),
                );
                if (*to).vval.v_string.is_null() {
                    (*to).vval.v_string = xstrdup((*from).vval.v_string);
                }
            }
        }
        4 => {
            (*to).v_type = VAR_LIST;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_list.is_null() {
                (*to).vval.v_list = ::core::ptr::null_mut::<list_T>();
            } else if copyID != 0 as ::core::ffi::c_int
                && tv_list_copyid((*from).vval.v_list) == copyID
            {
                (*to).vval.v_list = tv_list_latest_copy((*from).vval.v_list);
                tv_list_ref((*to).vval.v_list);
            } else {
                (*to).vval.v_list = tv_list_copy(conv, (*from).vval.v_list, deep, copyID);
            }
            if (*to).vval.v_list.is_null() && !(*from).vval.v_list.is_null() {
                ret = FAIL;
            }
        }
        10 => {
            tv_blob_copy((*from).vval.v_blob, to);
        }
        5 => {
            (*to).v_type = VAR_DICT;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_dict.is_null() {
                (*to).vval.v_dict = ::core::ptr::null_mut::<dict_T>();
            } else if copyID != 0 as ::core::ffi::c_int
                && (*(*from).vval.v_dict).dv_copyID == copyID
            {
                (*to).vval.v_dict = (*(*from).vval.v_dict).dv_copydict;
                (*(*to).vval.v_dict).dv_refcount += 1;
            } else {
                (*to).vval.v_dict = tv_dict_copy(conv, (*from).vval.v_dict, deep, copyID);
            }
            if (*to).vval.v_dict.is_null() && !(*from).vval.v_dict.is_null() {
                ret = FAIL;
            }
        }
        0 => {
            internal_error(b"var_item_copy(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char);
            ret = FAIL;
        }
        _ => {}
    }
    recurse -= 1;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ex_echo(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut atstart: bool = true_0 != 0;
    let mut need_clear: bool = true_0 != 0;
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let called_emsg_before: ::core::ffi::c_int = called_emsg;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if (*eap).skip != 0 {
        emsg_skip += 1;
    }
    while *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        && !got_int
    {
        need_clr_eos = true_0 != 0;
        let mut p: *mut ::core::ffi::c_char = arg;
        if eval1(&raw mut arg, &raw mut rettv, &raw mut evalarg) == FAIL {
            if !aborting() && did_emsg == did_emsg_before && called_emsg == called_emsg_before {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    p,
                );
            }
            need_clr_eos = false_0 != 0;
            break;
        } else {
            need_clr_eos = false_0 != 0;
            if (*eap).skip == 0 {
                if atstart {
                    atstart = false_0 != 0;
                    msg_ext_set_append(
                        (*eap).cmdidx as ::core::ffi::c_int == CMD_echon as ::core::ffi::c_int,
                    );
                    msg_ext_set_kind(b"echo\0".as_ptr() as *const ::core::ffi::c_char);
                    if (*eap).cmdidx as ::core::ffi::c_int == CMD_echo as ::core::ffi::c_int {
                        if !msg_didout {
                            msg_sb_eol();
                        }
                        msg_start();
                    }
                } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_echo as ::core::ffi::c_int {
                    msg_puts_hl(
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        echo_hl_id,
                        false_0 != 0,
                    );
                }
                let mut tofree: *mut ::core::ffi::c_char =
                    encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>());
                msg_multiline(
                    cstr_as_string(tofree),
                    echo_hl_id,
                    true_0 != 0,
                    false_0 != 0,
                    &raw mut need_clear,
                );
                xfree(tofree as *mut ::core::ffi::c_void);
            }
            tv_clear(&raw mut rettv);
            arg = skipwhite(arg);
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
    clear_evalarg(&raw mut evalarg, eap);
    msg_ext_set_append(false_0 != 0);
    if (*eap).skip != 0 {
        emsg_skip -= 1;
    } else {
        if ui_has(kUIMessages) as ::core::ffi::c_int != 0
            && (*(*eap).arg as ::core::ffi::c_int == NUL
                || *(*eap).arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                || *(*eap).arg as ::core::ffi::c_int == '\n' as ::core::ffi::c_int)
        {
            msg_puts_len(
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ptrdiff_t,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else if need_clear {
            msg_clr_eos();
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_echo as ::core::ffi::c_int {
            msg_end();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_echohl(mut eap: *mut exarg_T) {
    echo_hl_id = syn_name2id((*eap).arg);
}
#[no_mangle]
pub unsafe extern "C" fn get_echo_hl_id() -> ::core::ffi::c_int {
    return echo_hl_id;
}
#[no_mangle]
pub unsafe extern "C" fn ex_execute(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut ret: ::core::ffi::c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    if (*eap).skip != 0 {
        emsg_skip += 1;
    }
    while *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        ret = eval1_emsg(&raw mut arg, &raw mut rettv, eap);
        if ret == FAIL {
            break;
        }
        if (*eap).skip == 0 {
            let argstr: *const ::core::ffi::c_char =
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_execute as ::core::ffi::c_int {
                    tv_get_string(&raw mut rettv)
                } else {
                    (if rettv.v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                    } else {
                        encode_tv2string(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                    }) as *const ::core::ffi::c_char
                };
            let len: size_t = strlen(argstr);
            ga_grow(
                &raw mut ga,
                len as ::core::ffi::c_int + 2 as ::core::ffi::c_int,
            );
            if !(ga.ga_len <= 0 as ::core::ffi::c_int) {
                let c2rust_fresh21 = ga.ga_len;
                ga.ga_len = ga.ga_len + 1;
                *(ga.ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh21 as isize) =
                    ' ' as ::core::ffi::c_char;
            }
            memcpy(
                (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize)
                    as *mut ::core::ffi::c_void,
                argstr as *const ::core::ffi::c_void,
                len.wrapping_add(1 as size_t),
            );
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_execute as ::core::ffi::c_int {
                xfree(argstr as *mut ::core::ffi::c_void);
            }
            ga.ga_len += len as ::core::ffi::c_int;
        }
        tv_clear(&raw mut rettv);
        arg = skipwhite(arg);
    }
    if ret != FAIL && !ga.ga_data.is_null() {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_echomsg as ::core::ffi::c_int {
            msg_ext_set_kind(b"echomsg\0".as_ptr() as *const ::core::ffi::c_char);
            msg(ga.ga_data as *const ::core::ffi::c_char, echo_hl_id);
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_echoerr as ::core::ffi::c_int {
            let mut save_did_emsg: ::core::ffi::c_int = did_emsg;
            emsg_multiline(
                ga.ga_data as *const ::core::ffi::c_char,
                b"echoerr\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_E as ::core::ffi::c_int,
                true_0 != 0,
            );
            if !force_abort {
                did_emsg = save_did_emsg;
            }
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_execute as ::core::ffi::c_int {
            do_cmdline(
                ga.ga_data as *mut ::core::ffi::c_char,
                (*eap).ea_getline,
                (*eap).cookie,
                DOCMD_NOWAIT as ::core::ffi::c_int | DOCMD_VERBOSE as ::core::ffi::c_int,
            );
        }
    }
    ga_clear(&raw mut ga);
    if (*eap).skip != 0 {
        emsg_skip -= 1;
    }
    (*eap).nextcmd = check_nextcmd(arg);
}
#[no_mangle]
pub unsafe extern "C" fn find_option_var_end(
    arg: *mut *const ::core::ffi::c_char,
    opt_idxp: *mut OptIndex,
    opt_flags: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = *arg;
    p = p.offset(1);
    if *p as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        *opt_flags = OPT_GLOBAL as ::core::ffi::c_int;
        p = p.offset(2 as ::core::ffi::c_int as isize);
    } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        *opt_flags = OPT_LOCAL as ::core::ffi::c_int;
        p = p.offset(2 as ::core::ffi::c_int as isize);
    } else {
        *opt_flags = 0 as ::core::ffi::c_int;
    }
    let mut end: *const ::core::ffi::c_char = find_option_end(p, opt_idxp);
    *arg = if end.is_null() { *arg } else { p };
    return end;
}
#[no_mangle]
pub unsafe extern "C" fn var_flavour(mut varname: *mut ::core::ffi::c_char) -> var_flavour_T {
    let mut p: *mut ::core::ffi::c_char = varname;
    if *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        loop {
            p = p.offset(1);
            if *p == 0 {
                break;
            }
            if *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            {
                return VAR_FLAVOUR_SESSION;
            }
        }
        return VAR_FLAVOUR_SHADA;
    }
    return VAR_FLAVOUR_DEFAULT;
}
#[no_mangle]
pub unsafe extern "C" fn var_set_global(name: *const ::core::ffi::c_char, mut vartv: typval_T) {
    let mut funccall_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccall_entry);
    set_var(name, strlen(name), &raw mut vartv, false_0 != 0);
    restore_funccal();
}
#[no_mangle]
pub unsafe extern "C" fn last_set_msg(mut script_ctx: sctx_T) {
    if script_ctx.sc_sid == 0 as ::core::ffi::c_int {
        return;
    }
    let mut should_free: bool = false;
    let mut p: *mut ::core::ffi::c_char = get_scriptname(script_ctx, &raw mut should_free);
    msg_ext_skip_verbose = true_0 != 0;
    verbose_enter();
    msg_puts(gettext(
        b"\n\tLast set from \0".as_ptr() as *const ::core::ffi::c_char
    ));
    msg_puts(p);
    if script_ctx.sc_lnum > 0 as linenr_T {
        msg_puts(gettext(&raw const line_msg as *const ::core::ffi::c_char));
        msg_outnum(script_ctx.sc_lnum as ::core::ffi::c_int);
    } else if script_is_lua(script_ctx.sc_sid) {
        msg_puts(gettext(
            b" (run Nvim with -V1 for more details)\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    if should_free {
        xfree(p as *mut ::core::ffi::c_void);
    }
    verbose_leave();
}
#[no_mangle]
pub unsafe extern "C" fn do_string_sub(
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut pat: *mut ::core::ffi::c_char,
    mut sub: *mut ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut flags: *const ::core::ffi::c_char,
    mut ret_len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo;
    p_cpo = &raw mut empty_string_option as *mut ::core::ffi::c_char;
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        200 as ::core::ffi::c_int,
    );
    regmatch.rm_ic = p_ic != 0;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        let mut tail: *mut ::core::ffi::c_char = str;
        let mut end: *mut ::core::ffi::c_char = str.offset(len as isize);
        let mut do_all: bool = *flags.offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 'g' as ::core::ffi::c_int;
        let mut sublen: ::core::ffi::c_int = 0;
        let mut zero_width: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        while vim_regexec_nl(&raw mut regmatch, str, tail.offset_from(str) as colnr_T) {
            if regmatch.startp[0 as ::core::ffi::c_int as usize]
                == regmatch.endp[0 as ::core::ffi::c_int as usize]
            {
                if zero_width == regmatch.startp[0 as ::core::ffi::c_int as usize] {
                    let mut i: ::core::ffi::c_int = utfc_ptr2len(tail);
                    memmove(
                        (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize)
                            as *mut ::core::ffi::c_void,
                        tail as *const ::core::ffi::c_void,
                        i as size_t,
                    );
                    ga.ga_len += i;
                    tail = tail.offset(i as isize);
                    continue;
                } else {
                    zero_width = regmatch.startp[0 as ::core::ffi::c_int as usize];
                }
            }
            sublen = vim_regsub(
                &raw mut regmatch,
                sub,
                expr,
                tail,
                0 as ::core::ffi::c_int,
                REGSUB_MAGIC as ::core::ffi::c_int,
            );
            if sublen <= 0 as ::core::ffi::c_int {
                ga_clear(&raw mut ga);
                break;
            } else {
                ga_grow(
                    &raw mut ga,
                    (end.offset_from(tail) + sublen as isize
                        - regmatch.endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(regmatch.startp[0 as ::core::ffi::c_int as usize]))
                        as ::core::ffi::c_int,
                );
                let mut i_0: ::core::ffi::c_int = regmatch.startp[0 as ::core::ffi::c_int as usize]
                    .offset_from(tail)
                    as ::core::ffi::c_int;
                memmove(
                    (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize)
                        as *mut ::core::ffi::c_void,
                    tail as *const ::core::ffi::c_void,
                    i_0 as size_t,
                );
                vim_regsub(
                    &raw mut regmatch,
                    sub,
                    expr,
                    (ga.ga_data as *mut ::core::ffi::c_char)
                        .offset(ga.ga_len as isize)
                        .offset(i_0 as isize),
                    sublen,
                    REGSUB_COPY as ::core::ffi::c_int | REGSUB_MAGIC as ::core::ffi::c_int,
                );
                ga.ga_len += i_0 + sublen - 1 as ::core::ffi::c_int;
                tail = regmatch.endp[0 as ::core::ffi::c_int as usize];
                if *tail as ::core::ffi::c_int == NUL {
                    break;
                }
                if !do_all {
                    break;
                }
            }
        }
        if !ga.ga_data.is_null() {
            strcpy(
                (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize),
                tail,
            );
            ga.ga_len += end.offset_from(tail) as ::core::ffi::c_int;
        }
        vim_regfree(regmatch.regprog);
    }
    if !ga.ga_data.is_null() {
        str = ga.ga_data as *mut ::core::ffi::c_char;
        len = ga.ga_len as size_t;
    }
    let mut ret: *mut ::core::ffi::c_char = xstrnsave(str, len);
    ga_clear(&raw mut ga);
    if p_cpo == &raw mut empty_string_option as *mut ::core::ffi::c_char {
        p_cpo = save_cpo;
    } else {
        if *p_cpo as ::core::ffi::c_int == NUL {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(save_cpo),
                    },
                },
                0 as ::core::ffi::c_int,
            );
        }
        free_string_option(save_cpo);
    }
    if !ret_len.is_null() {
        *ret_len = len;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn common_job_callbacks(
    mut vopts: *mut dict_T,
    mut on_stdout: *mut CallbackReader,
    mut on_stderr: *mut CallbackReader,
    mut on_exit: *mut Callback,
) -> bool {
    if tv_dict_get_callback(
        vopts,
        b"on_stdout\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
        &raw mut (*on_stdout).cb,
    ) as ::core::ffi::c_int
        != 0
        && tv_dict_get_callback(
            vopts,
            b"on_stderr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut (*on_stderr).cb,
        ) as ::core::ffi::c_int
            != 0
        && tv_dict_get_callback(
            vopts,
            b"on_exit\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            on_exit,
        ) as ::core::ffi::c_int
            != 0
    {
        (*on_stdout).buffered = tv_dict_get_number(
            vopts,
            b"stdout_buffered\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        (*on_stderr).buffered = tv_dict_get_number(
            vopts,
            b"stderr_buffered\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        if (*on_stdout).buffered as ::core::ffi::c_int != 0
            && (*on_stdout).cb.type_0 as ::core::ffi::c_uint
                == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*on_stdout).self_0 = vopts;
        }
        if (*on_stderr).buffered as ::core::ffi::c_int != 0
            && (*on_stderr).cb.type_0 as ::core::ffi::c_uint
                == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*on_stderr).self_0 = vopts;
        }
        (*vopts).dv_refcount += 1;
        return true_0 != 0;
    }
    callback_reader_free(on_stdout);
    callback_reader_free(on_stderr);
    callback_free(on_exit);
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn find_job(mut id: uint64_t, mut show_error: bool) -> *mut Channel {
    let mut data: *mut Channel = find_channel(id);
    if data.is_null()
        || (*data).streamtype as ::core::ffi::c_uint
            != kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
        || proc_is_stopped(&raw mut (*data).stream.proc) as ::core::ffi::c_int != 0
    {
        if show_error {
            if !data.is_null()
                && (*data).streamtype as ::core::ffi::c_uint
                    != kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    &raw const e_invchanjob as *const ::core::ffi::c_char,
                ));
            } else {
                emsg(gettext(&raw const e_invchan as *const ::core::ffi::c_char));
            }
        }
        return ::core::ptr::null_mut::<Channel>();
    }
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn script_host_eval(
    mut name: *mut ::core::ffi::c_char,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut args: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_string(
        args,
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
        -1 as ssize_t,
    );
    *rettv = eval_call_provider(
        name,
        b"eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn eval_call_provider(
    mut provider: *mut ::core::ffi::c_char,
    mut method: *mut ::core::ffi::c_char,
    mut arguments: *mut list_T,
    mut discard: bool,
) -> typval_T {
    if !eval_has_provider(provider, false_0 != 0) {
        semsg(
            b"E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"\0".as_ptr()
                as *const ::core::ffi::c_char,
            provider,
        );
        return typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: 0 as varnumber_T,
            },
        };
    }
    let mut func: [::core::ffi::c_char; 256] = [0; 256];
    let mut name_len: ::core::ffi::c_int = snprintf(
        &raw mut func as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
        b"provider#%s#Call\0".as_ptr() as *const ::core::ffi::c_char,
        provider,
    );
    let mut saved_provider_caller_scope: caller_scope = provider_caller_scope as caller_scope;
    provider_caller_scope = caller_scope {
        script_ctx: current_sctx,
        es_entry: *(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize),
        autocmd_fname: autocmd_fname,
        autocmd_match: autocmd_match,
        autocmd_fname_full: autocmd_fname_full,
        autocmd_bufnr: autocmd_bufnr,
        funccalp: get_current_funccal() as *mut ::core::ffi::c_void,
    } as caller_scope;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    provider_call_nesting += 1;
    let mut argvars: [typval_T; 3] = [
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: method },
        },
        typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: arguments },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv_list_ref(arguments);
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    call_func(
        &raw mut func as *mut ::core::ffi::c_char,
        name_len,
        &raw mut rettv,
        2 as ::core::ffi::c_int,
        &raw mut argvars as *mut typval_T,
        &raw mut funcexe,
    );
    tv_list_unref(arguments);
    restore_funccal();
    provider_caller_scope = saved_provider_caller_scope as caller_scope;
    provider_call_nesting -= 1;
    '_c2rust_label: {
        if provider_call_nesting >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"provider_call_nesting >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                6585 as ::core::ffi::c_uint,
                b"typval_T eval_call_provider(char *, char *, list_T *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if discard {
        tv_clear(&raw mut rettv);
    }
    return rettv;
}
#[no_mangle]
pub unsafe extern "C" fn eval_has_provider(
    mut feat: *const ::core::ffi::c_char,
    mut throw_if_fast: bool,
) -> bool {
    if !strequal(feat, b"clipboard\0".as_ptr() as *const ::core::ffi::c_char)
        && !strequal(feat, b"python3\0".as_ptr() as *const ::core::ffi::c_char)
        && !strequal(
            feat,
            b"python3_compiled\0".as_ptr() as *const ::core::ffi::c_char,
        )
        && !strequal(
            feat,
            b"python3_dynamic\0".as_ptr() as *const ::core::ffi::c_char,
        )
        && !strequal(feat, b"perl\0".as_ptr() as *const ::core::ffi::c_char)
        && !strequal(feat, b"ruby\0".as_ptr() as *const ::core::ffi::c_char)
        && !strequal(feat, b"node\0".as_ptr() as *const ::core::ffi::c_char)
    {
        return false_0 != 0;
    }
    if throw_if_fast as ::core::ffi::c_int != 0 && !nlua_is_deferred_safe() {
        semsg(
            &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
            b"Vimscript function\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    let mut name: [::core::ffi::c_char; 32] = [0; 32];
    snprintf(
        &raw mut name as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        feat,
    );
    strchrsub(
        &raw mut name as *mut ::core::ffi::c_char,
        '_' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    );
    let mut buf: [::core::ffi::c_char; 256] = [0; 256];
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut len: ::core::ffi::c_int = snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
        b"g:loaded_%s_provider\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut name as *mut ::core::ffi::c_char,
    );
    if eval_variable(
        &raw mut buf as *mut ::core::ffi::c_char,
        len,
        &raw mut tv,
        ::core::ptr::null_mut::<*mut dictitem_T>(),
        false_0 != 0,
        true_0 != 0,
    ) == FAIL
    {
        len = snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"provider#%s#bogus\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut name as *mut ::core::ffi::c_char,
        );
        script_autoload(
            &raw mut buf as *mut ::core::ffi::c_char,
            len as size_t,
            false_0 != 0,
        );
        len = snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"g:loaded_%s_provider\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut name as *mut ::core::ffi::c_char,
        );
        if eval_variable(
            &raw mut buf as *mut ::core::ffi::c_char,
            len,
            &raw mut tv,
            ::core::ptr::null_mut::<*mut dictitem_T>(),
            false_0 != 0,
            true_0 != 0,
        ) == FAIL
        {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                b"provider#%s#Call\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut name as *mut ::core::ffi::c_char,
            );
            if !find_func(&raw mut buf as *mut ::core::ffi::c_char).is_null() && p_lpl != 0 {
                semsg(
                    b"provider: %s: missing required variable g:loaded_%s_provider\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    &raw mut name as *mut ::core::ffi::c_char,
                    &raw mut name as *mut ::core::ffi::c_char,
                );
            }
            return false_0 != 0;
        }
    }
    let mut ok: bool = if tv.v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (2 as varnumber_T == tv.vval.v_number) as ::core::ffi::c_int
    } else {
        false_0
    } != 0;
    if ok {
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"provider#%s#Call\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut name as *mut ::core::ffi::c_char,
        );
        if find_func(&raw mut buf as *mut ::core::ffi::c_char).is_null() {
            semsg(
                b"provider: %s: g:loaded_%s_provider=2 but %s is not defined\0".as_ptr()
                    as *const ::core::ffi::c_char,
                &raw mut name as *mut ::core::ffi::c_char,
                &raw mut name as *mut ::core::ffi::c_char,
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            ok = false_0 != 0;
        }
    }
    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn eval_fmt_source_name_line(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
) {
    if !(*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name
    .is_null()
    {
        snprintf(
            buf,
            bufsize,
            b"%s:%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
    } else {
        snprintf(buf, bufsize, b"?\0".as_ptr() as *const ::core::ffi::c_char);
    };
}
#[no_mangle]
pub unsafe extern "C" fn prompt_get_input(mut buf: *mut buf_T) -> *mut ::core::ffi::c_char {
    if !bt_prompt(buf) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut lnum_start: linenr_T = (*buf).b_prompt_start.mark.lnum;
    let mut lnum_last: linenr_T = (*buf).b_ml.ml_line_count;
    let mut text: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum_start);
    if strlen(text) as ::core::ffi::c_int >= (*buf).b_prompt_start.mark.col {
        text = text.offset((*buf).b_prompt_start.mark.col as isize);
    }
    let mut full_text: *mut ::core::ffi::c_char = xstrdup(text);
    let mut i: linenr_T = lnum_start + 1 as linenr_T;
    while i <= lnum_last {
        let mut half_text: *mut ::core::ffi::c_char =
            concat_str(full_text, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        xfree(full_text as *mut ::core::ffi::c_void);
        full_text = concat_str(half_text, ml_get_buf(buf, i));
        xfree(half_text as *mut ::core::ffi::c_void);
        i += 1;
    }
    return full_text;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_invoke_callback() {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 2] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 2];
    let mut lnum: linenr_T = (*curbuf).b_ml.ml_line_count;
    let mut user_input: *mut ::core::ffi::c_char = prompt_get_input(curbuf);
    if user_input.is_null() {
        return;
    }
    ml_append(
        lnum,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        0 as colnr_T,
        false_0 != 0,
    );
    appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
    (*curwin).w_cursor.lnum = lnum + 1 as linenr_T;
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curbuf).b_prompt_start.mark.lnum = lnum + 1 as linenr_T;
    if (*curbuf).b_prompt_callback.type_0 as ::core::ffi::c_uint
        == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xfree(user_input as *mut ::core::ffi::c_void);
    } else {
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[0 as ::core::ffi::c_int as usize].vval.v_string = user_input;
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        callback_call(
            &raw mut (*curbuf).b_prompt_callback,
            1 as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        tv_clear(&raw mut rettv);
    }
    u_clearallandblockfree(curbuf);
    (*curbuf).b_prompt_start.mark.lnum = (*curbuf).b_ml.ml_line_count;
    (*curbuf).b_prompt_append_new_line = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn invoke_prompt_interrupt() -> bool {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 1] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 1];
    if (*curbuf).b_prompt_interrupt.type_0 as ::core::ffi::c_uint
        == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    got_int = false_0 != 0;
    let mut ret: ::core::ffi::c_int = callback_call(
        &raw mut (*curbuf).b_prompt_interrupt,
        0 as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    ) as ::core::ffi::c_int;
    tv_clear(&raw mut rettv);
    return ret != FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn typval_compare(
    mut typ1: *mut typval_T,
    mut typ2: *mut typval_T,
    mut type_0: exprtype_T,
    mut ic: bool,
) -> ::core::ffi::c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let type_is: bool = type_0 as ::core::ffi::c_uint
        == EXPR_IS as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint;
    if type_is as ::core::ffi::c_int != 0
        && (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint
    {
        n1 = (type_0 as ::core::ffi::c_uint
            == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int as varnumber_T;
    } else if (*typ1).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*typ2).v_type as ::core::ffi::c_uint
            == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as ::core::ffi::c_uint == (*typ2).v_type as ::core::ffi::c_uint
                && (*typ1).vval.v_blob == (*typ2).vval.v_blob)
                as ::core::ffi::c_int as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        } else if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                != EXPR_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && type_0 as ::core::ffi::c_uint
                    != EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint {
                emsg(gettext(b"E977: Can only compare Blob with Blob\0".as_ptr()
                    as *const ::core::ffi::c_char));
            } else {
                emsg(gettext(
                    &raw const e_invalblob as *const ::core::ffi::c_char,
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_blob_equal((*typ1).vval.v_blob, (*typ2).vval.v_blob) as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*typ2).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as ::core::ffi::c_uint == (*typ2).v_type as ::core::ffi::c_uint
                && (*typ1).vval.v_list == (*typ2).vval.v_list)
                as ::core::ffi::c_int as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        } else if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                != EXPR_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && type_0 as ::core::ffi::c_uint
                    != EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint {
                emsg(gettext(b"E691: Can only compare List with List\0".as_ptr()
                    as *const ::core::ffi::c_char));
            } else {
                emsg(gettext(
                    b"E692: Invalid operation for List\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_list_equal((*typ1).vval.v_list, (*typ2).vval.v_list, ic) as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*typ2).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as ::core::ffi::c_uint == (*typ2).v_type as ::core::ffi::c_uint
                && (*typ1).vval.v_dict == (*typ2).vval.v_dict)
                as ::core::ffi::c_int as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        } else if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                != EXPR_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && type_0 as ::core::ffi::c_uint
                    != EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*typ1).v_type as ::core::ffi::c_uint != (*typ2).v_type as ::core::ffi::c_uint {
                emsg(gettext(
                    b"E735: Can only compare Dictionary with Dictionary\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            } else {
                emsg(gettext(
                    b"E736: Invalid operation for Dictionary\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_dict_equal((*typ1).vval.v_dict, (*typ2).vval.v_dict, ic) as varnumber_T;
            if type_0 as ::core::ffi::c_uint
                == EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
            }
        }
    } else if tv_is_func(*typ1) as ::core::ffi::c_int != 0
        || tv_is_func(*typ2) as ::core::ffi::c_int != 0
    {
        if type_0 as ::core::ffi::c_uint != EXPR_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint != EXPR_IS as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                b"E694: Invalid operation for Funcrefs\0".as_ptr() as *const ::core::ffi::c_char
            ));
            tv_clear(typ1);
            return FAIL;
        }
        if (*typ1).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*typ1).vval.v_partial.is_null()
            || (*typ2).v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*typ2).vval.v_partial.is_null()
        {
            n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as ::core::ffi::c_int
                as varnumber_T;
        } else if type_is {
            if (*typ1).v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*typ2).v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
            } else if (*typ1).v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*typ2).v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as ::core::ffi::c_int
                    as varnumber_T;
            } else {
                n1 = false_0 as varnumber_T;
            }
        } else {
            n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
        }
        if type_0 as ::core::ffi::c_uint == EXPR_NEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                == EXPR_ISNOT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
        }
    } else if ((*typ1).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*typ2).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint)
        && type_0 as ::core::ffi::c_uint != EXPR_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != EXPR_NOMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let f1: float_T = tv_get_float(typ1);
        let f2: float_T = tv_get_float(typ2);
        n1 = false_0 as varnumber_T;
        match type_0 as ::core::ffi::c_uint {
            9 | 1 => {
                n1 = (f1 == f2) as ::core::ffi::c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (f1 != f2) as ::core::ffi::c_int as varnumber_T;
            }
            3 => {
                n1 = (f1 > f2) as ::core::ffi::c_int as varnumber_T;
            }
            4 => {
                n1 = (f1 >= f2) as ::core::ffi::c_int as varnumber_T;
            }
            5 => {
                n1 = (f1 < f2) as ::core::ffi::c_int as varnumber_T;
            }
            6 => {
                n1 = (f1 <= f2) as ::core::ffi::c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else if ((*typ1).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*typ2).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint)
        && type_0 as ::core::ffi::c_uint != EXPR_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != EXPR_NOMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        n1 = tv_get_number(typ1);
        n2 = tv_get_number(typ2);
        match type_0 as ::core::ffi::c_uint {
            9 | 1 => {
                n1 = (n1 == n2) as ::core::ffi::c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (n1 != n2) as ::core::ffi::c_int as varnumber_T;
            }
            3 => {
                n1 = (n1 > n2) as ::core::ffi::c_int as varnumber_T;
            }
            4 => {
                n1 = (n1 >= n2) as ::core::ffi::c_int as varnumber_T;
            }
            5 => {
                n1 = (n1 < n2) as ::core::ffi::c_int as varnumber_T;
            }
            6 => {
                n1 = (n1 <= n2) as ::core::ffi::c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else {
        let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
        let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
        let s1: *const ::core::ffi::c_char =
            tv_get_string_buf(typ1, &raw mut buf1 as *mut ::core::ffi::c_char);
        let s2: *const ::core::ffi::c_char =
            tv_get_string_buf(typ2, &raw mut buf2 as *mut ::core::ffi::c_char);
        let mut i: ::core::ffi::c_int = 0;
        if type_0 as ::core::ffi::c_uint != EXPR_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != EXPR_NOMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            i = mb_strcmp_ic(ic, s1, s2);
        } else {
            i = 0 as ::core::ffi::c_int;
        }
        n1 = false_0 as varnumber_T;
        match type_0 as ::core::ffi::c_uint {
            9 | 1 => {
                n1 = (i == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (i != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            3 => {
                n1 = (i > 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            4 => {
                n1 = (i >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            5 => {
                n1 = (i < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            6 => {
                n1 = (i <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T;
            }
            7 | 8 => {
                n1 = pattern_match(s2, s1, ic) as varnumber_T;
                if type_0 as ::core::ffi::c_uint
                    == EXPR_NOMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    n1 = (n1 == 0) as ::core::ffi::c_int as varnumber_T;
                }
            }
            0 | _ => {}
        }
    }
    tv_clear(typ1);
    (*typ1).v_type = VAR_NUMBER;
    (*typ1).vval.v_number = n1;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn typval_tostring(
    mut arg: *mut typval_T,
    mut quotes: bool,
) -> *mut ::core::ffi::c_char {
    if arg.is_null() {
        return xstrdup(b"(does not exist)\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if !quotes
        && (*arg).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(if (*arg).vval.v_string.is_null() {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            (*arg).vval.v_string as *const ::core::ffi::c_char
        });
    }
    return encode_tv2string(arg, ::core::ptr::null_mut::<size_t>());
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline(always)]
unsafe extern "C" fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    (*tv).v_type = VAR_LIST;
    (*tv).vval.v_list = l;
    tv_list_ref(l);
}
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval.h\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    return (*l).lv_copyID;
}
#[inline]
unsafe extern "C" fn tv_list_latest_copy(l: *const list_T) -> *mut list_T {
    return (*l).lv_copylist;
}
#[inline]
unsafe extern "C" fn tv_list_has_watchers(l: *const list_T) -> bool {
    return !l.is_null() && !(*l).lv_watch.is_null();
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
#[inline(always)]
unsafe extern "C" fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    (*tv).v_type = VAR_DICT;
    (*tv).vval.v_dict = d;
    if !d.is_null() {
        (*d).dv_refcount += 1;
    }
}
#[inline]
unsafe extern "C" fn tv_dict_is_watched(d: *const dict_T) -> bool {
    return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_set_ret(tv: *mut typval_T, b: *mut blob_T) {
    (*tv).v_type = VAR_BLOB;
    (*tv).vval.v_blob = b;
    if !b.is_null() {
        (*b).bv_refcount += 1;
    }
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_get(b: *const blob_T, mut idx: ::core::ffi::c_int) -> uint8_t {
    return *((*b).bv_ga.ga_data as *mut uint8_t).offset(idx as isize);
}
#[inline]
unsafe extern "C" fn tv_init(tv: *mut typval_T) {
    if !tv.is_null() {
        memset(
            tv as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<typval_T>(),
        );
    }
}
#[inline(always)]
unsafe extern "C" fn tv_dict_watcher_node_data(mut q: *mut QUEUE) -> *mut DictWatcher {
    return (q as *mut ::core::ffi::c_char).offset(-(32 as ::core::ffi::c_ulong as isize))
        as *mut DictWatcher;
}
#[inline(always)]
unsafe extern "C" fn tv_is_func(tv: typval_T) -> bool {
    return tv.v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv.v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
#[inline]
unsafe extern "C" fn proc_is_stopped(mut proc: *mut Proc) -> bool {
    let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
    return exited as ::core::ffi::c_int != 0 || (*proc).stopped_time != 0 as uint64_t;
}
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
