use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, arena_array, arena_dict, arena_string, cstr_as_string,
    cstr_to_string, try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::api_err_exp;
use crate::src::nvim::eval::typval::{tv_clear, tv_dict_find};
use crate::src::nvim::eval::userfunc::call_func;
use crate::src::nvim::eval::{clear_evalarg, eval0};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::garray::{ga_clear, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::main::{
    EVALARG_EVALUATE, capture_ga, current_sctx, curwin, did_emsg, did_throw, force_abort, msg_col,
    msg_silent, redir_off, suppress_errthrow,
};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, memmove, strlen};
use crate::src::nvim::runtime::do_source_str;
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, Boolean, CMD_index, Dict, Error, ExprAST, ExprASTNode,
    ExprASTNodeType, ExprAssignmentType, ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope,
    ExprParserFlags, Integer, KeyDict_exec_opts, KeyValuePair, Object, ParserHighlight,
    ParserHighlightChunk, ParserLine, ParserPosition, ParserState, SpecialVarValue, String_0,
    TryState, VAR_DICT, VAR_FUNC, VAR_PARTIAL, VAR_UNKNOWN, VAR_UNLOCKED, dict_T, dictitem_T,
    exarg_T, except_T, funcexe_T, garray_T, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, key_value_pair,
    linenr_T, msglist_T, object, object_data as C2Rust_Unnamed, partial_T, ptrdiff_t, sctx_T,
    size_t, typval_T, typval_vval_union, uint64_t, uvarnumber_T,
};
use crate::src::nvim::viml::parser::expressions::{
    ccs_tab, east_node_type_tab, eltkn_cmp_type_tab, expr_asgn_type_tab, viml_pexpr_free_ast,
    viml_pexpr_parse,
};
use crate::src::nvim::viml::parser::parser::{
    parser_simple_get_line, viml_parser_destroy, viml_parser_init,
};
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_13 = 20;
pub const CMD_snext: CMD_index = 414;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub const kExprAsgnConcat: ExprAssignmentType = 3;
pub const kExprAsgnSubtract: ExprAssignmentType = 2;
pub const kExprAsgnAdd: ExprAssignmentType = 1;
pub const kExprAsgnPlain: ExprAssignmentType = 0;
pub const kExprOptScopeLocal: ExprOptScope = 108;
pub const kExprOptScopeGlobal: ExprOptScope = 103;
pub const kExprOptScopeUnspecified: ExprOptScope = 0;
pub const kCCStrategyIgnoreCase: ExprCaseCompareStrategy = 63;
pub const kCCStrategyMatchCase: ExprCaseCompareStrategy = 35;
pub const kCCStrategyUseOption: ExprCaseCompareStrategy = 0;
pub const kExprCmpIdentical: ExprComparisonType = 4;
pub const kExprCmpGreaterOrEqual: ExprComparisonType = 3;
pub const kExprCmpGreater: ExprComparisonType = 2;
pub const kExprCmpMatches: ExprComparisonType = 1;
pub const kExprCmpEqual: ExprComparisonType = 0;
pub const kExprNodeAssignment: ExprASTNodeType = 38;
pub const kExprNodeEnvironment: ExprASTNodeType = 37;
pub const kExprNodeOption: ExprASTNodeType = 36;
pub const kExprNodeMod: ExprASTNodeType = 35;
pub const kExprNodeDivision: ExprASTNodeType = 34;
pub const kExprNodeMultiplication: ExprASTNodeType = 33;
pub const kExprNodeNot: ExprASTNodeType = 32;
pub const kExprNodeBinaryMinus: ExprASTNodeType = 31;
pub const kExprNodeUnaryMinus: ExprASTNodeType = 30;
pub const kExprNodeAnd: ExprASTNodeType = 29;
pub const kExprNodeOr: ExprASTNodeType = 28;
pub const kExprNodeDoubleQuotedString: ExprASTNodeType = 27;
pub const kExprNodeSingleQuotedString: ExprASTNodeType = 26;
pub const kExprNodeFloat: ExprASTNodeType = 25;
pub const kExprNodeInteger: ExprASTNodeType = 24;
pub const kExprNodeConcatOrSubscript: ExprASTNodeType = 23;
pub const kExprNodeConcat: ExprASTNodeType = 22;
pub const kExprNodeComparison: ExprASTNodeType = 21;
pub const kExprNodeArrow: ExprASTNodeType = 20;
pub const kExprNodeColon: ExprASTNodeType = 19;
pub const kExprNodeComma: ExprASTNodeType = 18;
pub const kExprNodeCurlyBracesIdentifier: ExprASTNodeType = 17;
pub const kExprNodeDictLiteral: ExprASTNodeType = 16;
pub const kExprNodeLambda: ExprASTNodeType = 15;
pub const kExprNodeUnknownFigure: ExprASTNodeType = 14;
pub const kExprNodeComplexIdentifier: ExprASTNodeType = 13;
pub const kExprNodePlainKey: ExprASTNodeType = 12;
pub const kExprNodePlainIdentifier: ExprASTNodeType = 11;
pub const kExprNodeCall: ExprASTNodeType = 10;
pub const kExprNodeNested: ExprASTNodeType = 9;
pub const kExprNodeBinaryPlus: ExprASTNodeType = 8;
pub const kExprNodeUnaryPlus: ExprASTNodeType = 7;
pub const kExprNodeListLiteral: ExprASTNodeType = 6;
pub const kExprNodeSubscript: ExprASTNodeType = 5;
pub const kExprNodeRegister: ExprASTNodeType = 4;
pub const kExprNodeTernaryValue: ExprASTNodeType = 3;
pub const kExprNodeTernary: ExprASTNodeType = 2;
pub const kExprNodeOpMissing: ExprASTNodeType = 1;
pub const kExprNodeMissing: ExprASTNodeType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTConvStackItem {
    pub node_p: *mut *mut ExprASTNode,
    pub ret_node_p: *mut Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTConvStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExprASTConvStackItem,
    pub init_array: [ExprASTConvStackItem; 16],
}
pub const kExprFlagsParseLet: ExprParserFlags = 4;
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const kExprFlagsMulti: ExprParserFlags = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe extern "C" fn nvim_exec2(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = ARRAY_DICT_INIT;
    let mut output: String_0 = exec_impl(channel_id, src, opts, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return result;
    }
    if (*opts).output {
        if result.size == result.capacity {
            result.capacity = if result.capacity != 0 {
                result.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            result.items = xrealloc(
                result.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<KeyValuePair>().wrapping_mul(result.capacity),
            ) as *mut KeyValuePair;
        } else {
        };
        let c2rust_fresh0 = result.size;
        result.size = result.size.wrapping_add(1);
        *result.items.offset(c2rust_fresh0 as isize) = key_value_pair {
            key: cstr_to_string(b"output\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: output },
            },
        };
    }
    return result;
}
pub unsafe extern "C" fn exec_impl(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> String_0 {
    let save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let save_redir_off: bool = redir_off.get();
    let save_capture_ga: *mut garray_T = capture_ga.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if (*opts).output {
        ga_init(
            &raw mut capture_local,
            1 as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        capture_ga.set(&raw mut capture_local);
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    if (*opts).output {
        (*msg_silent.ptr()) += 1;
        redir_off.set(false);
        msg_col.set(0 as ::core::ffi::c_int);
    }
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    do_source_str(
        src.data,
        b"nvim_exec2()\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    if (*opts).output {
        capture_ga.set(save_capture_ga);
        msg_silent.set(save_msg_silent);
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    current_sctx.set(save_current_sctx);
    try_leave(&raw mut tstate, err);
    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        if (*opts).output as ::core::ffi::c_int != 0
            && capture_local.ga_len > 1 as ::core::ffi::c_int
        {
            let mut s: String_0 = String_0 {
                data: capture_local.ga_data as *mut ::core::ffi::c_char,
                size: capture_local.ga_len as size_t,
            };
            if *s.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\n' as ::core::ffi::c_int
            {
                memmove(
                    s.data as *mut ::core::ffi::c_void,
                    s.data.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    s.size.wrapping_sub(1 as size_t),
                );
                *s.data.offset(s.size.wrapping_sub(1 as size_t) as isize) =
                    NUL as ::core::ffi::c_char;
                s.size = s.size.wrapping_sub(1 as size_t);
            }
            return s;
        }
    }
    if (*opts).output {
        ga_clear(&raw mut capture_local);
    }
    return STRING_INIT;
}
pub unsafe extern "C" fn nvim_command(mut cmd: String_0, mut err: *mut Error) {
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    do_cmdline_cmd(cmd.data);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_eval(
    mut expr: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
        did_throw.set(false_0 != 0);
        did_emsg.set(false_0);
    }
    (*recursive.ptr()) += 1;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut ok: ::core::ffi::c_int = 0;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    ok = eval0(
        expr.data,
        &raw mut rettv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    );
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    try_leave(&raw mut tstate, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        if ok == FAIL {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to evaluate expression: '%.*s'\0".as_ptr() as *const ::core::ffi::c_char,
                256 as ::core::ffi::c_int,
                expr.data,
            );
        } else {
            rv = vim_to_object(&raw mut rettv, arena, false_0 != 0);
        }
    }
    tv_clear(&raw mut rettv);
    (*recursive.ptr()) -= 1;
    return rv;
}
unsafe extern "C" fn _call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut self_0: *mut dict_T,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if args.size > MAX_FUNC_ARGS as ::core::ffi::c_int as size_t {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Function called with too many arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    let mut vim_args: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        object_to_vim(
            *args.items.offset(i as isize),
            (&raw mut vim_args as *mut typval_T).offset(i as isize),
            err,
        );
        i = i.wrapping_add(1);
    }
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
        did_throw.set(false_0 != 0);
        did_emsg.set(false_0);
    }
    (*recursive.ptr()) += 1;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_selfdict = self_0;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    call_func(
        fn_0.data,
        fn_0.size as ::core::ffi::c_int,
        &raw mut rettv,
        args.size as ::core::ffi::c_int,
        &raw mut vim_args as *mut typval_T,
        &raw mut funcexe,
    );
    try_leave(&raw mut tstate, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        rv = vim_to_object(&raw mut rettv, arena, false_0 != 0);
    }
    tv_clear(&raw mut rettv);
    (*recursive.ptr()) -= 1;
    while i > 0 as size_t {
        i = i.wrapping_sub(1);
        tv_clear((&raw mut vim_args as *mut typval_T).offset(i as isize));
    }
    return rv;
}
pub unsafe extern "C" fn nvim_call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return _call_function(fn_0, args, ::core::ptr::null_mut::<dict_T>(), arena, err);
}
pub unsafe extern "C" fn nvim_call_dict_function(
    mut dict: Object,
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut mustfree: bool = false_0 != 0;
    match dict.type_0 as ::core::ffi::c_uint {
        4 => {
            let mut eval_ret: ::core::ffi::c_int = 0;
            let mut tstate: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate);
            eval_ret = eval0(
                dict.data.string.data,
                &raw mut rettv,
                ::core::ptr::null_mut::<exarg_T>(),
                EVALARG_EVALUATE.ptr(),
            );
            clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
            try_leave(&raw mut tstate, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return rv;
            }
            if eval_ret != OK {
                abort();
            }
            mustfree = true_0 != 0;
        }
        6 => {
            object_to_vim(dict, &raw mut rettv, err);
        }
        _ => {
            if true {
                api_err_exp(
                    err,
                    b"dict argument\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Dict\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return rv;
            }
        }
    }
    let mut self_dict: *mut dict_T = rettv.vval.v_dict;
    '_end: {
        if rettv.v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            || self_dict.is_null()
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"dict not found\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if !fn_0.data.is_null()
                && fn_0.size > 0 as size_t
                && dict.type_0 as ::core::ffi::c_uint
                    != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let di: *mut dictitem_T =
                    tv_dict_find(self_dict, fn_0.data, fn_0.size as ptrdiff_t);
                if di.is_null() {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        fn_0.data,
                    );
                    break '_end;
                } else if (*di).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"partial function not supported\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_end;
                } else if !((*di).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Not a function: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        fn_0.data,
                    );
                    break '_end;
                } else {
                    fn_0 = String_0 {
                        data: (*di).di_tv.vval.v_string,
                        size: strlen((*di).di_tv.vval.v_string),
                    };
                }
            }
            if !(!fn_0.data.is_null() && fn_0.size >= 1 as size_t) {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid function name: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"(empty)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                rv = _call_function(fn_0, args, self_dict, arena, err);
            }
        }
    }
    if mustfree {
        tv_clear(&raw mut rettv);
    }
    return rv;
}
pub unsafe extern "C" fn nvim_parse_expression(
    mut expr: String_0,
    mut flags: String_0,
    mut hl: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut pflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < flags.size {
        match *flags.data.offset(i as isize) as ::core::ffi::c_int {
            109 => {
                pflags |= kExprFlagsMulti as ::core::ffi::c_int;
            }
            69 => {
                pflags |= kExprFlagsDisallowEOC as ::core::ffi::c_int;
            }
            108 => {
                pflags |= kExprFlagsParseLet as ::core::ffi::c_int;
            }
            NUL => {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid flag: '\\0' (%u)\0".as_ptr() as *const ::core::ffi::c_char,
                    *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                );
                return ARRAY_DICT_INIT;
            }
            _ => {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid flag: '%c' (%u)\0".as_ptr() as *const ::core::ffi::c_char,
                    *flags.data.offset(i as isize) as ::core::ffi::c_int,
                    *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                );
                return ARRAY_DICT_INIT;
            }
        }
        i = i.wrapping_add(1);
    }
    let mut parser_lines: [ParserLine; 2] = [
        ParserLine {
            data: expr.data,
            size: expr.size,
            allocated: false_0 != 0,
        },
        ParserLine {
            data: ::core::ptr::null::<::core::ffi::c_char>(),
            size: 0 as size_t,
            allocated: false_0 != 0,
        },
    ];
    let mut plines_p: *mut ParserLine = &raw mut parser_lines as *mut ParserLine;
    let mut colors: ParserHighlight = ParserHighlight {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ParserHighlightChunk>(),
        init_array: [ParserHighlightChunk {
            start: ParserPosition { line: 0, col: 0 },
            end_col: 0,
            group: ::core::ptr::null::<::core::ffi::c_char>(),
        }; 16],
    };
    colors.capacity = colors.init_array.len();
    colors.items = colors.init_array.as_mut_ptr();
    let colors_p: *mut ParserHighlight = if hl as ::core::ffi::c_int != 0 {
        &raw mut colors
    } else {
        ::core::ptr::null_mut::<ParserHighlight>()
    };
    let mut pstate: ParserState = ::core::mem::zeroed();
    viml_parser_init(
        &raw mut pstate,
        Some(parser_simple_get_line),
        &raw mut plines_p as *mut ::core::ffi::c_void,
        colors_p,
    );
    let mut east: ExprAST = viml_pexpr_parse(&raw mut pstate, pflags);
    let ret_size: size_t = (2 as size_t)
        .wrapping_add(!east.err.msg.is_null() as ::core::ffi::c_int as size_t)
        .wrapping_add(hl as size_t)
        .wrapping_add(0 as size_t);
    let mut ret: Dict = arena_dict(arena, ret_size);
    let c2rust_fresh1 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh1 as isize) = key_value_pair {
        key: cstr_as_string(b"len\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if pstate.pos.line == 1 as size_t {
                    parser_lines[0 as ::core::ffi::c_int as usize].size
                } else {
                    pstate.pos.col
                }) as Integer,
            },
        },
    };
    if !east.err.msg.is_null() {
        let mut err_dict: Dict = arena_dict(arena, 2 as size_t);
        let c2rust_fresh2 = err_dict.size;
        err_dict.size = err_dict.size.wrapping_add(1);
        *err_dict.items.offset(c2rust_fresh2 as isize) = key_value_pair {
            key: cstr_as_string(b"message\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(arena, cstr_as_string(east.err.msg)),
                },
            },
        };
        let c2rust_fresh3 = err_dict.size;
        err_dict.size = err_dict.size.wrapping_add(1);
        *err_dict.items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"arg\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        arena,
                        String_0 {
                            data: east.err.arg as *mut ::core::ffi::c_char,
                            size: east.err.arg_len as size_t,
                        },
                    ),
                },
            },
        };
        let c2rust_fresh4 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"error\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: err_dict },
            },
        };
    }
    if hl {
        let mut hl_arr: Array = arena_array(arena, colors.size);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < colors.size {
            let chunk: ParserHighlightChunk = *colors.items.offset(i_0 as isize);
            let mut chunk_arr: Array = arena_array(arena, 4 as size_t);
            let c2rust_fresh5 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.start.line as Integer,
                },
            };
            let c2rust_fresh6 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.start.col as Integer,
                },
            };
            let c2rust_fresh7 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.end_col as Integer,
                },
            };
            let c2rust_fresh8 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(chunk.group),
                },
            };
            let c2rust_fresh9 = hl_arr.size;
            hl_arr.size = hl_arr.size.wrapping_add(1);
            *hl_arr.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: chunk_arr },
            };
            i_0 = i_0.wrapping_add(1);
        }
        let c2rust_fresh10 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"highlight\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_arr },
            },
        };
    }
    if colors.items != &raw mut colors.init_array as *mut ParserHighlightChunk {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut colors.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    let mut ast_conv_stack: ExprASTConvStack = ExprASTConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ExprASTConvStackItem>(),
        init_array: [ExprASTConvStackItem {
            node_p: ::core::ptr::null_mut::<*mut ExprASTNode>(),
            ret_node_p: ::core::ptr::null_mut::<Object>(),
        }; 16],
    };
    ast_conv_stack.capacity = ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
        .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    ast_conv_stack.size = 0 as size_t;
    ast_conv_stack.items = &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem;
    let mut ast: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if ast_conv_stack.size == ast_conv_stack.capacity {
        ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            ast_conv_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        ast_conv_stack.items = (if ast_conv_stack.capacity
            == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if ast_conv_stack.items
                == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
            {
                ast_conv_stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        as *mut ::core::ffi::c_void,
                    ast_conv_stack.items as *mut ::core::ffi::c_void,
                    ast_conv_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            }
        } else {
            if ast_conv_stack.items
                == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
            {
                memcpy(
                    xmalloc(
                        ast_conv_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                    ),
                    ast_conv_stack.items as *const ::core::ffi::c_void,
                    ast_conv_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            } else {
                xrealloc(
                    ast_conv_stack.items as *mut ::core::ffi::c_void,
                    ast_conv_stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            }
        }) as *mut ExprASTConvStackItem;
    } else {
    };
    let c2rust_fresh11 = ast_conv_stack.size;
    ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
    *ast_conv_stack.items.offset(c2rust_fresh11 as isize) = ExprASTConvStackItem {
        node_p: &raw mut east.root,
        ret_node_p: &raw mut ast,
    };
    while ast_conv_stack.size != 0 {
        let mut cur_item: ExprASTConvStackItem = *ast_conv_stack.items.offset(
            ast_conv_stack
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
        let node: *mut ExprASTNode = *cur_item.node_p;
        if node.is_null() {
            '_c2rust_label: {
                if ast_conv_stack.size == 1 as size_t {
                } else {
                    __assert_fail(
                        b"kv_size(ast_conv_stack) == 1\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/vimscript.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        511 as ::core::ffi::c_uint,
                        b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
        } else {
            if (*cur_item.ret_node_p).type_0 as ::core::ffi::c_uint
                == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut items_size: size_t = (3 as ::core::ffi::c_int
                    + !(*node).children.is_null() as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainIdentifier as ::core::ffi::c_int
                                as ::core::ffi::c_uint) as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainIdentifier as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainKey as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeEnvironment as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeRegister as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + 3 as ::core::ffi::c_int
                        * ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeComparison as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeFloat as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeDoubleQuotedString as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeSingleQuotedString as ::core::ffi::c_int
                                as ::core::ffi::c_uint) as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeAssignment as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + 0 as ::core::ffi::c_int)
                    as size_t;
                let mut ret_node: Dict = arena_dict(arena, items_size);
                *cur_item.ret_node_p = object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed { dict: ret_node },
                };
            }
            let mut ret_node_0: *mut Dict = &raw mut (*cur_item.ret_node_p).data.dict;
            if !(*node).children.is_null() {
                let num_children: size_t = (1 as ::core::ffi::c_int
                    + !(*(*node).children).next.is_null() as ::core::ffi::c_int)
                    as size_t;
                let mut children_array: Array = arena_array(arena, num_children);
                let mut i_1: size_t = 0 as size_t;
                while i_1 < num_children {
                    let c2rust_fresh12 = children_array.size;
                    children_array.size = children_array.size.wrapping_add(1);
                    *children_array.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                    i_1 = i_1.wrapping_add(1);
                }
                let c2rust_fresh13 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh13 as isize) = key_value_pair {
                    key: cstr_as_string(b"children\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed {
                            array: children_array,
                        },
                    },
                };
                if ast_conv_stack.size == ast_conv_stack.capacity {
                    ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    ast_conv_stack.items = (if ast_conv_stack.capacity
                        == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            ast_conv_stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                                    as *mut ::core::ffi::c_void,
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    } else {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            memcpy(
                                xmalloc(
                                    ast_conv_stack
                                        .capacity
                                        .wrapping_mul(
                                            ::core::mem::size_of::<ExprASTConvStackItem>(),
                                        ),
                                ),
                                ast_conv_stack.items as *const ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        } else {
                            xrealloc(
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    }) as *mut ExprASTConvStackItem;
                } else {
                };
                let c2rust_fresh14 = ast_conv_stack.size;
                ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                *ast_conv_stack.items.offset(c2rust_fresh14 as isize) = ExprASTConvStackItem {
                    node_p: &raw mut (*node).children,
                    ret_node_p: children_array
                        .items
                        .offset(0 as ::core::ffi::c_int as isize),
                };
            } else if !(*node).next.is_null() {
                if ast_conv_stack.size == ast_conv_stack.capacity {
                    ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    ast_conv_stack.items = (if ast_conv_stack.capacity
                        == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            ast_conv_stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                                    as *mut ::core::ffi::c_void,
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    } else {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            memcpy(
                                xmalloc(
                                    ast_conv_stack
                                        .capacity
                                        .wrapping_mul(
                                            ::core::mem::size_of::<ExprASTConvStackItem>(),
                                        ),
                                ),
                                ast_conv_stack.items as *const ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        } else {
                            xrealloc(
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    }) as *mut ExprASTConvStackItem;
                } else {
                };
                let c2rust_fresh15 = ast_conv_stack.size;
                ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                *ast_conv_stack.items.offset(c2rust_fresh15 as isize) = ExprASTConvStackItem {
                    node_p: &raw mut (*node).next,
                    ret_node_p: cur_item.ret_node_p.offset(1 as ::core::ffi::c_int as isize),
                };
            } else {
                ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
                let c2rust_fresh16 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                    key: cstr_as_string(b"type\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(
                                *(&raw const east_node_type_tab
                                    as *const *const ::core::ffi::c_char)
                                    .offset((*node).type_0 as isize),
                            ),
                        },
                    },
                };
                let mut start_array: Array = arena_array(arena, 2 as size_t);
                let c2rust_fresh17 = start_array.size;
                start_array.size = start_array.size.wrapping_add(1);
                *start_array.items.offset(c2rust_fresh17 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*node).start.line as Integer,
                    },
                };
                let c2rust_fresh18 = start_array.size;
                start_array.size = start_array.size.wrapping_add(1);
                *start_array.items.offset(c2rust_fresh18 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*node).start.col as Integer,
                    },
                };
                let c2rust_fresh19 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh19 as isize) = key_value_pair {
                    key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: start_array },
                    },
                };
                let c2rust_fresh20 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                    key: cstr_as_string(b"len\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*node).len as Integer,
                        },
                    },
                };
                match (*node).type_0 as ::core::ffi::c_uint {
                    27 | 26 => {
                        let mut str: Object = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: arena_string(
                                    arena,
                                    String_0 {
                                        data: (*node).data.str.value,
                                        size: (*node).data.str.size,
                                    },
                                ),
                            },
                        };
                        let c2rust_fresh21 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh21 as isize) = key_value_pair {
                            key: cstr_as_string(b"svalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: str,
                        };
                        xfree((*node).data.str.value as *mut ::core::ffi::c_void);
                    }
                    36 => {
                        let c2rust_fresh22 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh22 as isize) = key_value_pair {
                            key: cstr_as_string(b"scope\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.opt.scope as Integer,
                                },
                            },
                        };
                        let c2rust_fresh23 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh23 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.opt.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.opt.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    11 => {
                        let c2rust_fresh24 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh24 as isize) = key_value_pair {
                            key: cstr_as_string(b"scope\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.var.scope as Integer,
                                },
                            },
                        };
                        let c2rust_fresh25 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh25 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.var.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.var.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    12 => {
                        let c2rust_fresh26 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh26 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.var.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.var.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    37 => {
                        let c2rust_fresh27 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.env.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.env.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    4 => {
                        let c2rust_fresh28 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.reg.name as Integer,
                                },
                            },
                        };
                    }
                    21 => {
                        let c2rust_fresh29 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"cmp_type\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: cstr_as_string(
                                        *(&raw const eltkn_cmp_type_tab
                                            as *const *const ::core::ffi::c_char)
                                            .offset((*node).data.cmp.type_0 as isize),
                                    ),
                                },
                            },
                        };
                        let c2rust_fresh30 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh30 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"ccs_strategy\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: cstr_as_string(
                                        *(&raw const ccs_tab as *const *const ::core::ffi::c_char)
                                            .offset((*node).data.cmp.ccs as isize),
                                    ),
                                },
                            },
                        };
                        let c2rust_fresh31 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh31 as isize) = key_value_pair {
                            key: cstr_as_string(b"invert\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeBoolean,
                                data: C2Rust_Unnamed {
                                    boolean: (*node).data.cmp.inv,
                                },
                            },
                        };
                    }
                    25 => {
                        let c2rust_fresh32 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh32 as isize) = key_value_pair {
                            key: cstr_as_string(b"fvalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeFloat,
                                data: C2Rust_Unnamed {
                                    floating: (*node).data.flt.value,
                                },
                            },
                        };
                    }
                    24 => {
                        let c2rust_fresh33 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh33 as isize) = key_value_pair {
                            key: cstr_as_string(b"ivalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: if (*node).data.num.value
                                        > 9223372036854775807 as uvarnumber_T
                                    {
                                        9223372036854775807 as Integer
                                    } else {
                                        (*node).data.num.value as Integer
                                    },
                                },
                            },
                        };
                    }
                    38 => {
                        let asgn_type: ExprAssignmentType = (*node).data.ass.type_0;
                        let mut str_0: String_0 = if asgn_type as ::core::ffi::c_uint
                            == kExprAsgnPlain as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            STRING_INIT
                        } else {
                            cstr_as_string(
                                *(&raw const expr_asgn_type_tab
                                    as *const *const ::core::ffi::c_char)
                                    .offset(asgn_type as isize),
                            )
                        };
                        let c2rust_fresh34 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh34 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"augmentation\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed { string: str_0 },
                            },
                        };
                    }
                    0 | 1 | 2 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 14 | 15 | 16 | 17 | 18 | 19
                    | 20 | 22 | 23 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 35 | _ => {}
                }
                '_c2rust_label_0: {
                    if (*cur_item.ret_node_p).data.dict.size
                        == (*cur_item.ret_node_p).data.dict.capacity
                    {
                    } else {
                        __assert_fail(
                            b"cur_item.ret_node_p->data.dict.size == cur_item.ret_node_p->data.dict.capacity\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/vimscript.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            640 as ::core::ffi::c_uint,
                            b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                xfree(*cur_item.node_p as *mut ::core::ffi::c_void);
                *cur_item.node_p = ::core::ptr::null_mut::<ExprASTNode>();
            }
        }
    }
    if ast_conv_stack.items != &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut ast_conv_stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
    let c2rust_fresh35 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh35 as isize) = key_value_pair {
        key: cstr_as_string(b"ast\0".as_ptr() as *const ::core::ffi::c_char),
        value: ast,
    };
    '_c2rust_label_1: {
        if ret.size == ret.capacity {
        } else {
            __assert_fail(
                b"ret.size == ret.capacity\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/vimscript.rs\0".as_ptr() as *const ::core::ffi::c_char,
                649 as ::core::ffi::c_uint,
                b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    viml_pexpr_free_ast(east);
    viml_parser_destroy(&mut pstate);
    return ret;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
