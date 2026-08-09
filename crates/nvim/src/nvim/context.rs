use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_string, api_set_error, arena_dict, copy_array,
    copy_object, cstr_as_string, string_to_array,
};
use crate::src::nvim::api::vimscript::exec_impl;
use crate::src::nvim::eval::encode::encode_vim_list_to_buf;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::eval::userfunc::func_tbl_get;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::src::nvim::option::{get_option_value, optval_free, set_option_value};
use crate::src::nvim::options::kOptShada;
use crate::src::nvim::os::libc::{snprintf, strlen, strncmp};
use crate::src::nvim::shada::{
    shada_encode_buflist, shada_encode_gvars, shada_encode_jumps, shada_encode_regs,
    shada_read_string,
};
use crate::src::nvim::types::{
    Arena, Array, Context, Dict, Error, KeyDict_exec_opts, KeyValuePair, Object, OptVal,
    OptValData, OptValType, String_0, VAR_LIST, VAR_UNKNOWN, VAR_UNLOCKED, hashitem_T, hashtab_T,
    kErrorTypeException, kErrorTypeNone, kObjectTypeArray, kObjectTypeString, key_value_pair,
    object, object_data as C2Rust_Unnamed_0, size_t, typval_T, typval_vval_union, uint8_t,
    uint64_t,
};
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContextVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Context,
}
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const kCtxFuncs: C2Rust_Unnamed_1 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_1 = 16;
pub const kCtxGVars: C2Rust_Unnamed_1 = 8;
pub const kCtxBufs: C2Rust_Unnamed_1 = 4;
pub const kCtxJumps: C2Rust_Unnamed_1 = 2;
pub const kCtxRegs: C2Rust_Unnamed_1 = 1;
pub const OPT_GLOBAL: C2Rust_Unnamed_2 = 1;
pub const kShaDaForceit: C2Rust_Unnamed_3 = 4;
pub const kShaDaWantInfo: C2Rust_Unnamed_3 = 1;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_2 = 128;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_2 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_2 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_2 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_2 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_2 = 2;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub static kCtxAll: GlobalCell<::core::ffi::c_int> = GlobalCell::new(
    kCtxRegs as ::core::ffi::c_int
        | kCtxJumps as ::core::ffi::c_int
        | kCtxBufs as ::core::ffi::c_int
        | kCtxGVars as ::core::ffi::c_int
        | kCtxSFuncs as ::core::ffi::c_int
        | kCtxFuncs as ::core::ffi::c_int,
);
static ctx_stack: GlobalCell<ContextVec> = GlobalCell::new(ContextVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Context>(),
});
pub unsafe extern "C" fn ctx_size() -> size_t {
    return (*ctx_stack.ptr()).size;
}
pub unsafe extern "C" fn ctx_get(mut index: size_t) -> *mut Context {
    if index < (*ctx_stack.ptr()).size {
        return (*ctx_stack.ptr()).items.add(
            (*ctx_stack.ptr())
                .size
                .wrapping_sub(index)
                .wrapping_sub(1 as size_t),
        );
    }
    return ::core::ptr::null_mut::<Context>();
}
pub unsafe extern "C" fn ctx_free(mut ctx: *mut Context) {
    api_free_string((*ctx).regs);
    api_free_string((*ctx).jumps);
    api_free_string((*ctx).bufs);
    api_free_string((*ctx).gvars);
    api_free_array((*ctx).funcs);
}
pub unsafe extern "C" fn ctx_save(mut ctx: *mut Context, flags: ::core::ffi::c_int) {
    if ctx.is_null() {
        if (*ctx_stack.ptr()).size == (*ctx_stack.ptr()).capacity {
            (*ctx_stack.ptr()).capacity = if (*ctx_stack.ptr()).capacity != 0 {
                (*ctx_stack.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*ctx_stack.ptr()).items = xrealloc(
                (*ctx_stack.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Context>().wrapping_mul((*ctx_stack.ptr()).capacity),
            ) as *mut Context;
        } else {
        };
        let c2rust_fresh0 = (*ctx_stack.ptr()).size;
        (*ctx_stack.ptr()).size = (*ctx_stack.ptr()).size.wrapping_add(1);
        *(*ctx_stack.ptr()).items.add(c2rust_fresh0) = Context {
            regs: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            jumps: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            bufs: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            gvars: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            funcs: Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        ctx = (*ctx_stack.ptr()).items.add(
            (*ctx_stack.ptr())
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t),
        );
    }
    if flags & kCtxRegs as ::core::ffi::c_int != 0 {
        (*ctx).regs = shada_encode_regs();
    }
    if flags & kCtxJumps as ::core::ffi::c_int != 0 {
        (*ctx).jumps = shada_encode_jumps();
    }
    if flags & kCtxBufs as ::core::ffi::c_int != 0 {
        (*ctx).bufs = shada_encode_buflist();
    }
    if flags & kCtxGVars as ::core::ffi::c_int != 0 {
        (*ctx).gvars = shada_encode_gvars();
    }
    if flags & kCtxFuncs as ::core::ffi::c_int != 0 {
        ctx_save_funcs(ctx, false_0 != 0);
    } else if flags & kCtxSFuncs as ::core::ffi::c_int != 0 {
        ctx_save_funcs(ctx, true_0 != 0);
    }
}
pub unsafe extern "C" fn ctx_restore(mut ctx: *mut Context, flags: ::core::ffi::c_int) -> bool {
    let mut free_ctx: bool = false_0 != 0;
    if ctx.is_null() {
        if (*ctx_stack.ptr()).size == 0 as size_t {
            return false_0 != 0;
        }
        (*ctx_stack.ptr()).size = (*ctx_stack.ptr()).size.wrapping_sub(1);
        ctx = (*ctx_stack.ptr()).items.add((*ctx_stack.ptr()).size);
        free_ctx = true_0 != 0;
    }
    let mut op_shada: OptVal = get_option_value(kOptShada, OPT_GLOBAL as ::core::ffi::c_int);
    set_option_value(
        kOptShada,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: c"!,'100,%".as_ptr() as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_GLOBAL as ::core::ffi::c_int,
    );
    if flags & kCtxRegs as ::core::ffi::c_int != 0 {
        ctx_restore_regs(ctx);
    }
    if flags & kCtxJumps as ::core::ffi::c_int != 0 {
        ctx_restore_jumps(ctx);
    }
    if flags & kCtxBufs as ::core::ffi::c_int != 0 {
        ctx_restore_bufs(ctx);
    }
    if flags & kCtxGVars as ::core::ffi::c_int != 0 {
        ctx_restore_gvars(ctx);
    }
    if flags & kCtxFuncs as ::core::ffi::c_int != 0 {
        ctx_restore_funcs(ctx);
    }
    if free_ctx {
        ctx_free(ctx);
    }
    set_option_value(kOptShada, op_shada, OPT_GLOBAL as ::core::ffi::c_int);
    optval_free(op_shada);
    return true_0 != 0;
}
#[inline]
unsafe extern "C" fn ctx_restore_regs(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).regs,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_jumps(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).jumps,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_bufs(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).bufs,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_gvars(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).gvars,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_save_funcs(mut ctx: *mut Context, mut scriptonly: bool) {
    (*ctx).funcs = ARRAY_DICT_INIT;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let hiht_: *mut hashtab_T = func_tbl_get();
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hitodo_ = hitodo_.wrapping_sub(1);
            let name: *const ::core::ffi::c_char = (*hi).hi_key;
            let mut islambda: bool =
                strncmp(name, c"<lambda>".as_ptr(), 8 as size_t) == 0 as ::core::ffi::c_int;
            let mut isscript: bool = *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int;
            if !islambda && (!scriptonly || isscript as ::core::ffi::c_int != 0) {
                let mut cmd_len: size_t =
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_add(strlen(name));
                let mut cmd: *mut ::core::ffi::c_char =
                    xmalloc(cmd_len) as *mut ::core::ffi::c_char;
                snprintf(cmd, cmd_len, c"func! %s".as_ptr(), name);
                let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: true };
                let mut func_body: String_0 = exec_impl(
                    (1 as ::core::ffi::c_int as uint64_t)
                        << ::core::mem::size_of::<uint64_t>()
                            .wrapping_mul(8 as usize)
                            .wrapping_sub(1 as usize),
                    cstr_as_string(cmd),
                    &raw mut opts,
                    &raw mut err,
                );
                xfree(cmd as *mut ::core::ffi::c_void);
                if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    if (*ctx).funcs.size == (*ctx).funcs.capacity {
                        (*ctx).funcs.capacity = if (*ctx).funcs.capacity != 0 {
                            (*ctx).funcs.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        (*ctx).funcs.items = xrealloc(
                            (*ctx).funcs.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul((*ctx).funcs.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh1 = (*ctx).funcs.size;
                    (*ctx).funcs.size = (*ctx).funcs.size.wrapping_add(1);
                    *(*ctx).funcs.items.add(c2rust_fresh1) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_0 { string: func_body },
                    };
                }
                api_clear_error(&raw mut err);
            }
        }
        hi = hi.offset(1);
    }
}
#[inline]
unsafe extern "C" fn ctx_restore_funcs(mut ctx: *mut Context) {
    let mut i: size_t = 0 as size_t;
    while i < (*ctx).funcs.size {
        do_cmdline_cmd((*(*ctx).funcs.items.add(i)).data.string.data);
        i = i.wrapping_add(1);
    }
}
#[inline]
unsafe extern "C" fn array_to_string(mut array: Array, mut err: *mut Error) -> String_0 {
    let mut sbuf: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    let mut list_tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 { array: array },
        },
        &raw mut list_tv,
        err,
    );
    debug_assert!(
        list_tv.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint,
        "list_tv.v_type == VAR_LIST"
    );
    if !encode_vim_list_to_buf(list_tv.vval.v_list, &raw mut sbuf.size, &raw mut sbuf.data) {
        api_set_error(
            err,
            kErrorTypeException,
            c"%s".as_ptr(),
            c"E474: Failed to convert list to msgpack string buffer".as_ptr(),
        );
    }
    tv_clear(&raw mut list_tv);
    return sbuf;
}
pub unsafe extern "C" fn ctx_to_dict(mut ctx: *mut Context, mut arena: *mut Arena) -> Dict {
    debug_assert!(!ctx.is_null(), "ctx != NULL");
    let mut rv: Dict = arena_dict(arena, 5 as size_t);
    let c2rust_fresh2 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.add(c2rust_fresh2) = key_value_pair {
        key: cstr_as_string(c"regs".as_ptr()),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).regs, false, arena),
            },
        },
    };
    let c2rust_fresh3 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.add(c2rust_fresh3) = key_value_pair {
        key: cstr_as_string(c"jumps".as_ptr()),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).jumps, false, arena),
            },
        },
    };
    let c2rust_fresh4 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.add(c2rust_fresh4) = key_value_pair {
        key: cstr_as_string(c"bufs".as_ptr()),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).bufs, false, arena),
            },
        },
    };
    let c2rust_fresh5 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.add(c2rust_fresh5) = key_value_pair {
        key: cstr_as_string(c"gvars".as_ptr()),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).gvars, false, arena),
            },
        },
    };
    let c2rust_fresh6 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.add(c2rust_fresh6) = key_value_pair {
        key: cstr_as_string(c"funcs".as_ptr()),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: copy_array((*ctx).funcs, arena),
            },
        },
    };
    return rv;
}
pub unsafe extern "C" fn ctx_from_dict(
    mut dict: Dict,
    mut ctx: *mut Context,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    debug_assert!(!ctx.is_null(), "ctx != NULL");
    let mut types: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < dict.size
        && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        let mut item: KeyValuePair = *dict.items.add(i);
        if item.value.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if strequal(item.key.data, c"regs".as_ptr()) {
                types |= kCtxRegs as ::core::ffi::c_int;
                (*ctx).regs = array_to_string(item.value.data.array, err);
            } else if strequal(item.key.data, c"jumps".as_ptr()) {
                types |= kCtxJumps as ::core::ffi::c_int;
                (*ctx).jumps = array_to_string(item.value.data.array, err);
            } else if strequal(item.key.data, c"bufs".as_ptr()) {
                types |= kCtxBufs as ::core::ffi::c_int;
                (*ctx).bufs = array_to_string(item.value.data.array, err);
            } else if strequal(item.key.data, c"gvars".as_ptr()) {
                types |= kCtxGVars as ::core::ffi::c_int;
                (*ctx).gvars = array_to_string(item.value.data.array, err);
            } else if strequal(item.key.data, c"funcs".as_ptr()) {
                types |= kCtxFuncs as ::core::ffi::c_int;
                (*ctx).funcs = copy_object(item.value, ::core::ptr::null_mut::<Arena>())
                    .data
                    .array;
            }
        }
        i = i.wrapping_add(1);
    }
    return types;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
