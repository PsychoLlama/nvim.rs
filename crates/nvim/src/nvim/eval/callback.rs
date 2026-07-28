//! The `Callback` value: building one from a typval, and calling it.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn callback_from_typval(
    callback: *mut Callback,
    arg: *const typval_T,
) -> bool {
    let mut r: c_int = OK;
    if (*arg).v_type as c_uint == VAR_PARTIAL as c_int as c_uint && !(*arg).vval.v_partial.is_null()
    {
        (*callback).data.partial = (*arg).vval.v_partial;
        (*(*callback).data.partial).pt_refcount += 1;
        (*callback).type_0 = kCallbackPartial;
    } else if (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint
        && !(*arg).vval.v_string.is_null()
        && ascii_isdigit(*(*arg).vval.v_string as c_int) as c_int != 0
    {
        r = FAIL;
    } else if (*arg).v_type as c_uint == VAR_FUNC as c_int as c_uint
        || (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint
    {
        let mut name: *mut c_char = (*arg).vval.v_string;
        if name.is_null() {
            r = FAIL;
        } else if *name as c_int == NUL {
            (*callback).type_0 = kCallbackNone;
            (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
        } else {
            (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
            if (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint {
                (*callback).data.funcref = get_scriptlocal_funcname(name);
            }
            if (*callback).data.funcref.is_null() {
                (*callback).data.funcref = xstrdup(name);
            }
            func_ref((*callback).data.funcref);
            (*callback).type_0 = kCallbackFuncref;
        }
    } else if nlua_is_table_from_lua(arg) {
        let mut name_0: *mut c_char = nlua_register_table_as_callable(arg);
        if !name_0.is_null() {
            (*callback).data.funcref = xstrdup(name_0);
            (*callback).type_0 = kCallbackFuncref;
        } else {
            r = FAIL;
        }
    } else if (*arg).v_type as c_uint == VAR_SPECIAL as c_int as c_uint
        || (*arg).v_type as c_uint == VAR_NUMBER as c_int as c_uint
            && (*arg).vval.v_number == 0 as varnumber_T
    {
        (*callback).type_0 = kCallbackNone;
        (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
    } else {
        r = FAIL;
    }
    if r == FAIL {
        emsg(gettext(
            b"E921: Invalid callback argument\0".as_ptr() as *const c_char
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}

pub unsafe extern "C" fn get_callback_depth() -> c_int {
    return callback_depth.get();
}

pub unsafe extern "C" fn callback_call(
    callback: *mut Callback,
    argcount_in: c_int,
    argvars_in: *mut typval_T,
    rettv: *mut typval_T,
) -> bool {
    if callback_depth.get() as OptInt > p_mfd.get() {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        return false_0 != 0;
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut name: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut args: Array = ARRAY_DICT_INIT;
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    let mut len: c_int = 0;
    match (*callback).type_0 as c_uint {
        1 => {
            name = (*callback).data.funcref;
            len = strlen(name) as c_int;
            if len >= 6 as c_int
                && memcmp(
                    name as *const c_void,
                    b"v:lua.\0".as_ptr() as *const c_char as *const c_void,
                    6 as size_t,
                ) == 0
            {
                name = name.offset(6 as c_int as isize);
                len = check_luafunc_name(name, false_0 != 0);
                if len == 0 as c_int {
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
                ::core::ptr::null::<c_char>(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            return rv.type_0 as c_uint == kObjectTypeBoolean as c_int as c_uint
                && rv.data.boolean as c_int == true_0;
        }
        0 => return false_0 != 0,
        _ => {}
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    (*callback_depth.ptr()) += 1;
    let mut ret: c_int = call_func(
        name,
        -1 as c_int,
        rettv,
        argcount_in,
        argvars_in,
        &raw mut funcexe,
    );
    (*callback_depth.ptr()) -= 1;
    return ret != 0;
}

pub unsafe extern "C" fn set_ref_in_callback(
    mut callback: *mut Callback,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    match (*callback).type_0 as c_uint {
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

pub(crate) unsafe extern "C" fn set_ref_in_callback_reader(
    mut reader: *mut CallbackReader,
    mut copyID: c_int,
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
