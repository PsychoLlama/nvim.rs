use crate::src::nvim::eval::typval::{
    tv_blob_copy, tv_blob_remove, tv_check_for_string_or_list_or_blob_arg, tv_clear, tv_copy,
    tv_dict_add_tv, tv_dict_alloc_ret, tv_dict_copy, tv_dict_extend, tv_dict_item_remove,
    tv_dict_remove, tv_dict_unref, tv_equal, tv_get_number_chk, tv_get_string, tv_get_string_chk,
    tv_list_alloc_ret, tv_list_append_owned_tv, tv_list_append_tv, tv_list_copy, tv_list_extend,
    tv_list_find, tv_list_insert_tv, tv_list_item_remove, tv_list_remove, tv_list_reverse,
    tv_list_unref, value_check_lock,
};
use crate::src::nvim::eval::vars::{
    get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr, set_vim_var_string,
    set_vim_var_type, var_check_fixed, var_check_ro,
};
use crate::src::nvim::eval_1::{eval_expr_typval, get_copyID};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::garray::{ga_append, ga_concat, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    did_emsg, e_invalblob, e_invarg, e_invarg2, e_list_index_out_of_range_nr, e_listblobarg,
    e_listblobreq, e_listdictarg, e_listdictblobarg, e_string_required,
};
use crate::src::nvim::mbyte::{mb_strnicmp, utfc_ptr2len};
use crate::src::nvim::memory::xmemdupz;
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{__assert_fail, memmove, strcmp, strlen, strstr};
use crate::src::nvim::strings::reverse_text;
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictitem_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed_0, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    iconv_t, int32_t, int64_t, key_value_pair, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, object, object_data as C2Rust_Unnamed, partial_S, partial_T,
    proftime_T, ptrdiff_t, queue, scid_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_S,
    ufunc_T, uint64_t, uint8_t, varnumber_T, vimconv_T, ApiDispatchWrapper, Arena, Array,
    BoolVarValue, Boolean, Dict, Error, ErrorType, EvalFuncData, Float, Integer, KeyValuePair,
    ListLenSpecials, LuaRef, MsgpackRpcRequestHandler, Object, ObjectType, ScopeDictDictItem,
    ScopeType, SpecialVarValue, String_0, VarLockStatus, VarType, VimVarIndex, QUEUE,
};
extern "C" {
    fn hash_lock(ht: *mut hashtab_T);
    fn hash_unlock(ht: *mut hashtab_T);
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
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
pub type filtermap_T = ::core::ffi::c_uint;
pub const FILTERMAP_FOREACH: filtermap_T = 3;
pub const FILTERMAP_MAPNEW: filtermap_T = 2;
pub const FILTERMAP_MAP: filtermap_T = 1;
pub const FILTERMAP_FILTER: filtermap_T = 0;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static e_argument_of_str_must_be_list_string_or_dictionary: GlobalCell<[::core::ffi::c_char; 58]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
            *b"E706: Argument of %s must be a List, String or Dictionary\0",
        )
    });
static e_argument_of_str_must_be_list_string_dictionary_or_blob: GlobalCell<
    [::core::ffi::c_char; 65],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E1250: Argument of %s must be a List, String, Dictionary or Blob\0",
    )
});
unsafe extern "C" fn filter_map_one(
    mut tv: *mut typval_T,
    mut expr: *mut typval_T,
    filtermap: filtermap_T,
    mut newtv: *mut typval_T,
    mut remp: *mut bool,
) -> ::core::ffi::c_int {
    let mut argv: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    let mut retval: ::core::ffi::c_int = FAIL;
    tv_copy(tv, get_vim_var_tv(VV_VAL));
    (*newtv).v_type = VAR_UNKNOWN;
    '_theend: {
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*expr).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            do_cmdline_cmd((*expr).vval.v_string);
            if did_emsg.get() == 0 {
                retval = OK;
            }
        } else {
            argv[0 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_KEY);
            argv[1 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_VAL);
            if eval_expr_typval(
                expr,
                false_0 != 0,
                &raw mut argv as *mut typval_T,
                2 as ::core::ffi::c_int,
                newtv,
            ) != FAIL
            {
                if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut error: bool = false_0 != 0;
                    *remp = tv_get_number_chk(newtv, &raw mut error) == 0 as varnumber_T;
                    tv_clear(newtv);
                    if error {
                        break '_theend;
                    }
                } else if filtermap as ::core::ffi::c_uint
                    == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_clear(newtv);
                }
                retval = OK;
            }
        }
    }
    tv_clear(get_vim_var_tv(VV_VAL));
    return retval;
}
unsafe extern "C" fn filter_map_dict(
    mut d: *mut dict_T,
    mut filtermap: filtermap_T,
    mut _func_name: *const ::core::ffi::c_char,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).v_type = VAR_DICT;
        (*rettv).vval.v_dict = ::core::ptr::null_mut::<dict_T>();
    }
    if d.is_null()
        || filtermap as ::core::ffi::c_uint
            == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
            && value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
    {
        return;
    }
    let mut d_ret: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_dict_alloc_ret(rettv);
        d_ret = (*rettv).vval.v_dict;
    }
    let prev_lock: VarLockStatus = (*d).dv_lock;
    if (*d).dv_lock as ::core::ffi::c_uint
        == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*d).dv_lock = VAR_LOCKED;
    }
    hash_lock(&raw mut (*d).dv_hashtab);
    let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
    while dihi_todo_ != 0 {
        if !((*dihi_).hi_key.is_null()
            || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            dihi_todo_ = dihi_todo_.wrapping_sub(1);
            let di: *mut dictitem_T = (*dihi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                && (value_check_lock(
                    (*di).di_tv.v_lock,
                    arg_errmsg,
                    18446744073709551615 as size_t,
                ) as ::core::ffi::c_int
                    != 0
                    || var_check_ro(
                        (*di).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        18446744073709551615 as size_t,
                    ) as ::core::ffi::c_int
                        != 0)
            {
                break;
            }
            set_vim_var_string(
                VV_KEY,
                &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            let mut newtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut rem: bool = false;
            let mut r: ::core::ffi::c_int = filter_map_one(
                &raw mut (*di).di_tv,
                expr,
                filtermap,
                &raw mut newtv,
                &raw mut rem,
            );
            tv_clear(get_vim_var_tv(VV_KEY));
            if r == 0 as ::core::ffi::c_int || did_emsg.get() != 0 {
                tv_clear(&raw mut newtv);
                break;
            } else if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_clear(&raw mut (*di).di_tv);
                newtv.v_lock = VAR_UNLOCKED;
                (*di).di_tv = newtv;
            } else if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                r = tv_dict_add_tv(
                    d_ret,
                    &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                    strlen(&raw mut (*di).di_key as *mut ::core::ffi::c_char),
                    &raw mut newtv,
                );
                tv_clear(&raw mut newtv);
                if r == 0 as ::core::ffi::c_int {
                    break;
                }
            } else if filtermap as ::core::ffi::c_uint
                == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                && rem as ::core::ffi::c_int != 0
            {
                if var_check_fixed(
                    (*di).di_flags as ::core::ffi::c_int,
                    arg_errmsg,
                    18446744073709551615 as size_t,
                ) as ::core::ffi::c_int
                    != 0
                    || var_check_ro(
                        (*di).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        18446744073709551615 as size_t,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    break;
                }
                tv_dict_item_remove(d, di);
            }
        }
        dihi_ = dihi_.offset(1);
    }
    hash_unlock(&raw mut (*d).dv_hashtab);
    (*d).dv_lock = prev_lock;
}
unsafe extern "C" fn filter_map_blob(
    mut blob_arg: *mut blob_T,
    mut filtermap: filtermap_T,
    mut expr: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
) {
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).v_type = VAR_BLOB;
        (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
    }
    let mut b: *mut blob_T = blob_arg;
    if b.is_null()
        || filtermap as ::core::ffi::c_uint
            == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
            && value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
    {
        return;
    }
    let mut b_ret: *mut blob_T = b;
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_blob_copy(b, rettv);
        b_ret = (*rettv).vval.v_blob;
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    let prev_lock: VarLockStatus = (*b).bv_lock;
    if (*b).bv_lock as ::core::ffi::c_uint == 0 as ::core::ffi::c_uint {
        (*b).bv_lock = VAR_LOCKED;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*b).bv_ga.ga_len {
        let val: varnumber_T = tv_blob_get(b, i) as varnumber_T;
        let mut tv: typval_T = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: val },
        };
        set_vim_var_nr(VV_KEY, idx as varnumber_T);
        let mut newtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rem: bool = false;
        if filter_map_one(&raw mut tv, expr, filtermap, &raw mut newtv, &raw mut rem) == FAIL
            || did_emsg.get() != 0
        {
            break;
        }
        if filtermap as ::core::ffi::c_uint
            != FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if newtv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && newtv.v_type as ::core::ffi::c_uint
                    != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_clear(&raw mut newtv);
                emsg(
                    &raw const e_invalblob as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
                break;
            } else if filtermap as ::core::ffi::c_uint
                != FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if newtv.vval.v_number != val {
                    tv_blob_set(b_ret, i, newtv.vval.v_number as uint8_t);
                }
            } else if rem {
                let p: *mut ::core::ffi::c_char =
                    (*blob_arg).bv_ga.ga_data as *mut ::core::ffi::c_char;
                memmove(
                    p.offset(i as isize) as *mut ::core::ffi::c_void,
                    p.offset(i as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    ((*b).bv_ga.ga_len - i - 1 as ::core::ffi::c_int) as size_t,
                );
                (*b).bv_ga.ga_len -= 1;
                i -= 1;
            }
        }
        idx += 1;
        i += 1;
    }
    (*b).bv_lock = prev_lock;
}
unsafe extern "C" fn filter_map_string(
    mut str: *const ::core::ffi::c_char,
    mut filtermap: filtermap_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    set_vim_var_type(VV_KEY, VAR_NUMBER);
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
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        len = utfc_ptr2len(p);
        let mut tv: typval_T = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                    as *mut ::core::ffi::c_char,
            },
        };
        set_vim_var_nr(VV_KEY, idx as varnumber_T);
        let mut newtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rem: bool = false;
        if filter_map_one(&raw mut tv, expr, filtermap, &raw mut newtv, &raw mut rem) == FAIL
            || did_emsg.get() != 0
        {
            tv_clear(&raw mut newtv);
            tv_clear(&raw mut tv);
            break;
        } else {
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                || filtermap as ::core::ffi::c_uint
                    == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if newtv.v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_clear(&raw mut newtv);
                    tv_clear(&raw mut tv);
                    emsg(
                        &raw const e_string_required as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    break;
                } else {
                    ga_concat(&raw mut ga, newtv.vval.v_string);
                }
            } else if filtermap as ::core::ffi::c_uint
                == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
                || !rem
            {
                ga_concat(&raw mut ga, tv.vval.v_string);
            }
            tv_clear(&raw mut newtv);
            tv_clear(&raw mut tv);
            idx += 1;
            p = p.offset(len as isize);
        }
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn filter_map_list(
    mut l: *mut list_T,
    mut filtermap: filtermap_T,
    mut _func_name: *const ::core::ffi::c_char,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).v_type = VAR_LIST;
        (*rettv).vval.v_list = ::core::ptr::null_mut::<list_T>();
    }
    if l.is_null()
        || filtermap as ::core::ffi::c_uint
            == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
            && value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
    {
        return;
    }
    let mut l_ret: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        l_ret = (*rettv).vval.v_list;
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    let prev_lock: VarLockStatus = tv_list_locked(l);
    if tv_list_locked(l) as ::core::ffi::c_uint
        == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_set_lock(l, VAR_LOCKED);
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut li: *mut listitem_T = tv_list_first(l);
    while !li.is_null() {
        if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
            && value_check_lock((*li).li_tv.v_lock, arg_errmsg, TV_TRANSLATE as size_t)
                as ::core::ffi::c_int
                != 0
        {
            break;
        }
        set_vim_var_nr(VV_KEY, idx as varnumber_T);
        let mut newtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rem: bool = false;
        if filter_map_one(
            &raw mut (*li).li_tv,
            expr,
            filtermap,
            &raw mut newtv,
            &raw mut rem,
        ) == FAIL
        {
            break;
        }
        if did_emsg.get() != 0 {
            tv_clear(&raw mut newtv);
            break;
        } else {
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_clear(&raw mut (*li).li_tv);
                newtv.v_lock = VAR_UNLOCKED;
                (*li).li_tv = newtv;
            } else if filtermap as ::core::ffi::c_uint
                == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_list_append_owned_tv(l_ret, newtv);
            }
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                && rem as ::core::ffi::c_int != 0
            {
                li = tv_list_item_remove(l, li);
            } else {
                li = (*li).li_next;
            }
            idx += 1;
        }
    }
    tv_list_set_lock(l, prev_lock);
}
unsafe extern "C" fn filter_map(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut filtermap: filtermap_T,
) {
    let func_name: *const ::core::ffi::c_char = if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"map()\0".as_ptr() as *const ::core::ffi::c_char
    } else if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"mapnew()\0".as_ptr() as *const ::core::ffi::c_char
    } else if filtermap as ::core::ffi::c_uint
        == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"filter()\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"foreach()\0".as_ptr() as *const ::core::ffi::c_char
    };
    let arg_errmsg: *const ::core::ffi::c_char = if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"map() argument\0".as_ptr() as *const ::core::ffi::c_char
    } else if filtermap as ::core::ffi::c_uint
        == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"mapnew() argument\0".as_ptr() as *const ::core::ffi::c_char
    } else if filtermap as ::core::ffi::c_uint
        == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"filter() argument\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"foreach() argument\0".as_ptr() as *const ::core::ffi::c_char
    };
    if filtermap as ::core::ffi::c_uint
        != FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            (e_argument_of_str_must_be_list_string_dictionary_or_blob.ptr() as *const _)
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            func_name,
        );
        return;
    }
    let mut expr: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
    if (*expr).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut save_key: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    prepare_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        filter_map_dict(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            filtermap,
            func_name,
            arg_errmsg,
            expr,
            rettv,
        );
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        filter_map_blob(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob,
            filtermap,
            expr,
            arg_errmsg,
            rettv,
        );
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        filter_map_string(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            filtermap,
            expr,
            rettv,
        );
    } else {
        '_c2rust_label: {
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"argvars[0].v_type == VAR_LIST\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/list.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    393 as ::core::ffi::c_uint,
                    b"void filter_map(typval_T *, typval_T *, filtermap_T)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        filter_map_list(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            filtermap,
            func_name,
            arg_errmsg,
            expr,
            rettv,
        );
    }
    restore_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    (*did_emsg.ptr()) |= save_did_emsg;
}
pub unsafe extern "C" fn f_filter(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    filter_map(argvars, rettv, FILTERMAP_FILTER);
}
pub unsafe extern "C" fn f_map(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    filter_map(argvars, rettv, FILTERMAP_MAP);
}
pub unsafe extern "C" fn f_mapnew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    filter_map(argvars, rettv, FILTERMAP_MAPNEW);
}
pub unsafe extern "C" fn f_foreach(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    filter_map(argvars, rettv, FILTERMAP_FOREACH);
}
pub unsafe extern "C" fn f_add(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !value_check_lock(
            tv_list_locked(l),
            b"add() argument\0".as_ptr() as *const ::core::ffi::c_char,
            TV_TRANSLATE as size_t,
        ) {
            tv_list_append_tv(l, argvars.offset(1 as ::core::ffi::c_int as isize));
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        if !b.is_null()
            && !value_check_lock(
                (*b).bv_lock,
                b"add() argument\0".as_ptr() as *const ::core::ffi::c_char,
                TV_TRANSLATE as size_t,
            )
        {
            let mut error: bool = false_0 != 0;
            let n: varnumber_T = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if !error {
                ga_append(&raw mut (*b).bv_ga, n as uint8_t);
                tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
            }
        }
    } else {
        emsg(&raw const e_listblobreq as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
    };
}
unsafe extern "C" fn count_string(
    mut haystack: *const ::core::ffi::c_char,
    mut needle: *const ::core::ffi::c_char,
    mut ic: bool,
) -> varnumber_T {
    let mut n: varnumber_T = 0 as varnumber_T;
    let mut p: *const ::core::ffi::c_char = haystack;
    if p.is_null() || needle.is_null() || *needle as ::core::ffi::c_int == NUL {
        return 0 as varnumber_T;
    }
    let mut needlelen: size_t = strlen(needle);
    if ic {
        while *p as ::core::ffi::c_int != NUL {
            if mb_strnicmp(p, needle, needlelen) == 0 as ::core::ffi::c_int {
                n += 1;
                p = p.offset(needlelen as isize);
            } else {
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
        }
    } else {
        let mut next: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        loop {
            next = strstr(p, needle);
            if next.is_null() {
                break;
            }
            n += 1;
            p = next.offset(needlelen as isize);
        }
    }
    return n;
}
unsafe extern "C" fn count_list(
    mut l: *mut list_T,
    mut needle: *mut typval_T,
    mut idx: int64_t,
    mut ic: bool,
) -> varnumber_T {
    if tv_list_len(l) == 0 as ::core::ffi::c_int {
        return 0 as varnumber_T;
    }
    let mut li: *mut listitem_T = tv_list_find(l, idx as ::core::ffi::c_int);
    if li.is_null() {
        semsg(
            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            idx,
        );
        return 0 as varnumber_T;
    }
    let mut n: varnumber_T = 0 as varnumber_T;
    while !li.is_null() {
        if tv_equal(&raw mut (*li).li_tv, needle, ic) {
            n += 1;
        }
        li = (*li).li_next;
    }
    return n;
}
unsafe extern "C" fn count_dict(
    mut d: *mut dict_T,
    mut needle: *mut typval_T,
    mut ic: bool,
) -> varnumber_T {
    if d.is_null() {
        return 0 as varnumber_T;
    }
    let mut n: varnumber_T = 0 as varnumber_T;
    let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
    while dihi_todo_ != 0 {
        if !((*dihi_).hi_key.is_null()
            || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            dihi_todo_ = dihi_todo_.wrapping_sub(1);
            let di: *mut dictitem_T = (*dihi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            if tv_equal(&raw mut (*di).di_tv, needle, ic) {
                n += 1;
            }
        }
        dihi_ = dihi_.offset(1);
    }
    return n;
}
pub unsafe extern "C" fn f_count(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: varnumber_T = 0 as varnumber_T;
    let mut ic: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ic = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
    }
    if !error
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        n = count_string(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string,
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize)),
            ic != 0,
        );
    } else if !error
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut idx: int64_t = 0 as int64_t;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            idx = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
        }
        if !error {
            n = count_list(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                argvars.offset(1 as ::core::ffi::c_int as isize),
                idx,
                ic != 0,
            );
        }
    } else if !error
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(&raw const e_invarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
            } else {
                n = count_dict(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_dict,
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    ic != 0,
                );
            }
        }
    } else if !error {
        semsg(
            (e_argument_of_str_must_be_list_string_or_dictionary.ptr() as *const _)
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"count()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    (*rettv).vval.v_number = n;
}
unsafe extern "C" fn extend_dict(
    mut argvars: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut is_new: bool,
    mut rettv: *mut typval_T,
) {
    let mut d1: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d1.is_null() {
        let locked: bool = value_check_lock(VAR_FIXED, arg_errmsg, TV_TRANSLATE as size_t);
        '_c2rust_label: {
            if locked as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"locked == true\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/list.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    584 as ::core::ffi::c_uint,
                    b"void extend_dict(typval_T *, const char *, _Bool, typval_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    let d2: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d2.is_null() {
        tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        return;
    }
    if !is_new
        && value_check_lock((*d1).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) as ::core::ffi::c_int
            != 0
    {
        return;
    }
    if is_new {
        d1 = tv_dict_copy(
            ::core::ptr::null::<vimconv_T>(),
            d1,
            false_0 != 0,
            get_copyID(),
        );
        if d1.is_null() {
            return;
        }
    }
    let mut action: *const ::core::ffi::c_char = b"force\0".as_ptr() as *const ::core::ffi::c_char;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let av: [*const ::core::ffi::c_char; 3] = [
            b"keep\0".as_ptr() as *const ::core::ffi::c_char,
            b"force\0".as_ptr() as *const ::core::ffi::c_char,
            b"error\0".as_ptr() as *const ::core::ffi::c_char,
        ];
        action = tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
        if action.is_null() {
            if is_new {
                tv_dict_unref(d1);
            }
            return;
        }
        let mut i: size_t = 0;
        i = 0 as size_t;
        while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 3]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 3]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if strcmp(action, av[i as usize]) == 0 as ::core::ffi::c_int {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == 3 as size_t {
            if is_new {
                tv_dict_unref(d1);
            }
            semsg(
                &raw const e_invarg2 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                action,
            );
            return;
        }
    }
    tv_dict_extend(d1, d2, action);
    if is_new {
        *rettv = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: d1 },
        };
    } else {
        tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
    };
}
unsafe extern "C" fn extend_list(
    mut argvars: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
    mut is_new: bool,
    mut rettv: *mut typval_T,
) {
    let mut error: bool = false_0 != 0;
    let mut l1: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let l2: *mut list_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !is_new
        && value_check_lock(tv_list_locked(l1), arg_errmsg, TV_TRANSLATE as size_t)
            as ::core::ffi::c_int
            != 0
    {
        return;
    }
    if is_new {
        l1 = tv_list_copy(
            ::core::ptr::null::<vimconv_T>(),
            l1,
            false_0 != 0,
            get_copyID(),
        );
        if l1.is_null() {
            return;
        }
    }
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    's_92: {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut before: ::core::ffi::c_int = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if !error {
                if before == tv_list_len(l1) {
                    item = ::core::ptr::null_mut::<listitem_T>();
                    break 's_92;
                } else {
                    item = tv_list_find(l1, before);
                    if item.is_null() {
                        semsg(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            before as int64_t,
                        );
                    } else {
                        break 's_92;
                    }
                }
            }
            if is_new {
                tv_list_unref(l1);
            }
            return;
        } else {
            item = ::core::ptr::null_mut::<listitem_T>();
        }
    }
    tv_list_extend(l1, l2, item);
    if is_new {
        *rettv = typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: l1 },
        };
    } else {
        tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
    };
}
unsafe extern "C" fn extend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *mut ::core::ffi::c_char,
    mut is_new: bool,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        extend_list(argvars, arg_errmsg, is_new, rettv);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        extend_dict(argvars, arg_errmsg, is_new, rettv);
    } else {
        semsg(
            &raw const e_listdictarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            if is_new as ::core::ffi::c_int != 0 {
                b"extendnew()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"extend()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
    };
}
pub unsafe extern "C" fn f_extend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut errmsg: *mut ::core::ffi::c_char =
        b"extend() argument\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    extend(argvars, rettv, errmsg, false_0 != 0);
}
pub unsafe extern "C" fn f_extendnew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut errmsg: *mut ::core::ffi::c_char = b"extendnew() argument\0".as_ptr()
        as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char;
    extend(argvars, rettv, errmsg, true_0 != 0);
}
pub unsafe extern "C" fn f_insert(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        if b.is_null()
            || value_check_lock(
                (*b).bv_lock,
                b"insert() argument\0".as_ptr() as *const ::core::ffi::c_char,
                TV_TRANSLATE as size_t,
            ) as ::core::ffi::c_int
                != 0
        {
            return;
        }
        let mut before: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let len: ::core::ffi::c_int = tv_blob_len(b);
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            before = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return;
            }
            if before < 0 as ::core::ffi::c_int || before > len {
                semsg(
                    &raw const e_invarg2 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    tv_get_string(argvars.offset(2 as ::core::ffi::c_int as isize)),
                );
                return;
            }
        }
        let val: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return;
        }
        if val < 0 as ::core::ffi::c_int || val > 255 as ::core::ffi::c_int {
            semsg(
                &raw const e_invarg2 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
            );
            return;
        }
        ga_grow(&raw mut (*b).bv_ga, 1 as ::core::ffi::c_int);
        let p: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
        memmove(
            p.offset(before as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(before as isize) as *const ::core::ffi::c_void,
            (len - before) as size_t,
        );
        *p.offset(before as isize) = val as uint8_t;
        (*b).bv_ga.ga_len += 1;
        tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            &raw const e_listblobarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"insert()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if value_check_lock(
            tv_list_locked(l),
            b"insert() argument\0".as_ptr() as *const ::core::ffi::c_char,
            TV_TRANSLATE as size_t,
        ) {
            return;
        }
        let mut before_0: int64_t = 0 as int64_t;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            before_0 = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as int64_t;
        }
        if error {
            return;
        }
        let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        if before_0 != tv_list_len(l) as int64_t {
            item = tv_list_find(l, before_0 as ::core::ffi::c_int);
            if item.is_null() {
                semsg(
                    &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    before_0,
                );
                l = ::core::ptr::null_mut::<list_T>();
            }
        }
        if !l.is_null() {
            tv_list_insert_tv(l, argvars.offset(1 as ::core::ffi::c_int as isize), item);
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        }
    };
}
pub unsafe extern "C" fn f_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let arg_errmsg: *const ::core::ffi::c_char =
        b"remove() argument\0".as_ptr() as *const ::core::ffi::c_char;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_dict_remove(argvars, rettv, arg_errmsg);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_blob_remove(argvars, rettv, arg_errmsg);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_remove(argvars, rettv, arg_errmsg);
    } else {
        semsg(
            &raw const e_listdictblobarg as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"remove()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    };
}
pub unsafe extern "C" fn f_reverse(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_string_or_list_or_blob_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        let len: ::core::ffi::c_int = tv_blob_len(b);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len / 2 as ::core::ffi::c_int {
            let tmp: uint8_t = tv_blob_get(b, i);
            tv_blob_set(b, i, tv_blob_get(b, len - i - 1 as ::core::ffi::c_int));
            tv_blob_set(b, len - i - 1 as ::core::ffi::c_int, tmp);
            i += 1;
        }
        tv_blob_set_ret(rettv, b);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).v_type = VAR_STRING;
        if !(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
        {
            (*rettv).vval.v_string = reverse_text(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_string,
            );
        } else {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !value_check_lock(
            tv_list_locked(l),
            b"reverse() argument\0".as_ptr() as *const ::core::ffi::c_char,
            TV_TRANSLATE as size_t,
        ) {
            tv_list_reverse(l);
            tv_list_set_ret(rettv, l);
        }
    }
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
unsafe extern "C" fn tv_list_locked(l: *const list_T) -> VarLockStatus {
    if l.is_null() {
        return VAR_FIXED;
    }
    return (*l).lv_lock;
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
                    b"src/nvim/eval/list.rs\0".as_ptr() as *const ::core::ffi::c_char,
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
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
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
#[inline(always)]
unsafe extern "C" fn tv_blob_set(blob: *mut blob_T, mut idx: ::core::ffi::c_int, mut c: uint8_t) {
    *((*blob).bv_ga.ga_data as *mut uint8_t).offset(idx as isize) = c;
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
