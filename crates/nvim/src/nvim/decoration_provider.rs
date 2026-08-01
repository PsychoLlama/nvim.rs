use crate::src::nvim::api::extmark::describe_ns;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_object, api_object_to_bool,
};
use crate::src::nvim::decoration::decor_check_to_be_deleted;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::hl_check_ns;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_call_ref};
use crate::src::nvim::main::{decor_state, display_tick, ns_hl_active, textlock};
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::message::msg_schedule_semsg_multiline;
use crate::src::nvim::r#move::validate_botline_win;
use crate::src::nvim::os::libc::__assert_fail;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::{
    Arena, Array, DecorProvider, DecorProvider_state as C2Rust_Unnamed_13, Error, Integer, LuaRef,
    LuaRetMode, MarkTree, NS, Object, buf_T, int64_t, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeBuffer, kObjectTypeInteger, kObjectTypeNil, kObjectTypeWindow, linenr_T, object,
    object_data as C2Rust_Unnamed_12, size_t, uint8_t, win_T,
};
pub const kDecorProviderDisabled: C2Rust_Unnamed_13 = 4;
pub const kDecorProviderRedrawDisabled: C2Rust_Unnamed_13 = 3;
pub const kDecorProviderWinDisabled: C2Rust_Unnamed_13 = 2;
pub const kDecorProviderActive: C2Rust_Unnamed_13 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorProvider,
}
pub const kRetMulti: LuaRetMode = 3;
pub const kRetNilBool: LuaRetMode = 1;
pub const CB_MAX_ERROR: C2Rust_Unnamed_20 = 3;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
static decor_providers: GlobalCell<C2Rust_Unnamed_19> = GlobalCell::new(C2Rust_Unnamed_19 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<DecorProvider>(),
});
unsafe extern "C" fn decor_provider_error(
    mut provider: *mut DecorProvider,
    mut name: *const ::core::ffi::c_char,
    mut msg: *const ::core::ffi::c_char,
) {
    let mut ns: *const ::core::ffi::c_char = describe_ns(
        (*provider).ns_id,
        b"(UNKNOWN PLUGIN)\0".as_ptr() as *const ::core::ffi::c_char,
    );
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"decor_provider_error\0".as_ptr() as *const ::core::ffi::c_char,
        29 as ::core::ffi::c_int,
        true_0 != 0,
        b"Error in decoration provider \"%s\" (ns=%s):\n%s\0".as_ptr()
            as *const ::core::ffi::c_char,
        name,
        ns,
        msg,
    );
    msg_schedule_semsg_multiline(
        b"Decoration provider \"%s\" (ns=%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
        name,
        ns,
        msg,
    );
}
unsafe extern "C" fn decor_provider_invoke(
    mut provider_idx: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut ref_0: LuaRef,
    mut args: Array,
    mut default_true: bool,
    mut res: *mut Array,
) -> bool {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    (*textlock.ptr()) += 1;
    let mut ret: Object = nlua_call_ref(
        ref_0,
        name,
        args,
        (if !res.is_null() {
            kRetMulti as ::core::ffi::c_int
        } else {
            kRetNilBool as ::core::ffi::c_int
        }) as LuaRetMode,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    (*textlock.ptr()) -= 1;
    let mut provider: *mut DecorProvider =
        (*decor_providers.ptr()).items.offset(provider_idx as isize);
    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        (*provider).error_count = 0 as uint8_t;
        if !res.is_null() {
            '_c2rust_label: {
                if ret.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"ret.type == kObjectTypeArray\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/decoration_provider.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        50 as ::core::ffi::c_uint,
                        b"_Bool decor_provider_invoke(int, const char *, LuaRef, Array, _Bool, Array *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            *res = ret.data.array;
            return true_0 != 0;
        } else if api_object_to_bool(
            ret,
            b"provider %s retval\0".as_ptr() as *const ::core::ffi::c_char,
            default_true,
            &raw mut err,
        ) {
            return true_0 != 0;
        }
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
        && ((*provider).error_count as ::core::ffi::c_int) < CB_MAX_ERROR as ::core::ffi::c_int
    {
        decor_provider_error(provider, name, err.msg);
        (*provider).error_count = (*provider).error_count.wrapping_add(1);
        if (*provider).error_count as ::core::ffi::c_int >= CB_MAX_ERROR as ::core::ffi::c_int {
            (*provider).state = kDecorProviderDisabled;
        }
    }
    api_clear_error(&raw mut err);
    api_free_object(ret);
    return false_0 != 0;
}
pub unsafe extern "C" fn decor_providers_invoke_spell(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
) {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).spell_nav != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 6] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 6];
            args.capacity = 6 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh0 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh1 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh1 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh2 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: start_row as Integer,
                },
            };
            let c2rust_fresh3 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: start_col as Integer,
                },
            };
            let c2rust_fresh4 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh4 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: end_row as Integer,
                },
            };
            let c2rust_fresh5 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: end_col as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"spell\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).spell_nav,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn decor_providers_invoke_conceal_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
) -> bool {
    let mut keys: size_t = (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys;
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).conceal_line != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 4] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 4];
            args.capacity = 4 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh6 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh7 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh8 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: row as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"conceal_line\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).conceal_line,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
    return (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys > keys;
}
pub unsafe extern "C" fn decor_providers_start() {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_start != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh9 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick.get() as ::core::ffi::c_int as Integer,
                },
            };
            let mut active: bool = decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"start\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_start,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
            (*(*decor_providers.ptr()).items.offset(i as isize)).state =
                (if active as ::core::ffi::c_int != 0 {
                    kDecorProviderActive as ::core::ffi::c_int
                } else {
                    kDecorProviderRedrawDisabled as ::core::ffi::c_int
                }) as C2Rust_Unnamed_13;
        } else if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*(*decor_providers.ptr()).items.offset(i as isize)).state = kDecorProviderActive;
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn decor_providers_invoke_win(mut wp: *mut win_T) {
    '_c2rust_label: {
        if (*decor_state.ptr()).current_end == 0 as ::core::ffi::c_int
            && (*decor_state.ptr()).future_begin
                == (*decor_state.ptr()).ranges_i.size as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"decor_state.current_end == 0 && decor_state.future_begin == (int)kv_size(decor_state.ranges_i)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration_provider.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                139 as ::core::ffi::c_uint,
                b"void decor_providers_invoke_win(win_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*decor_providers.ptr()).size > 0 as size_t {
        validate_botline_win(wp);
    }
    let mut botline: linenr_T = if (*wp).w_botline < (*(*wp).w_buffer).b_ml.ml_line_count {
        (*wp).w_botline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderWinDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*p).state = kDecorProviderActive;
        }
        (*p).win_skip_row = 0 as ::core::ffi::c_int;
        (*p).win_skip_col = 0 as ::core::ffi::c_int;
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_win != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 4] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 4];
            args.capacity = 4 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh10 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh10 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh11 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh11 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh12 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh12 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: ((*wp).w_topline - 1 as linenr_T) as Integer,
                },
            };
            let c2rust_fresh13 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh13 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (botline - 1 as linenr_T) as Integer,
                },
            };
            if !decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"win\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_win,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            ) {
                (*(*decor_providers.ptr()).items.offset(i as isize)).state =
                    kDecorProviderWinDisabled;
            }
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn decor_providers_invoke_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
) {
    (*decor_state.ptr()).running_decor_provider = true_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_line != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 3] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 3];
            args.capacity = 3 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh14 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh14 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh15 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh15 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh16 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh16 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: row as Integer,
                },
            };
            if !decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"line\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_line,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            ) {
                (*(*decor_providers.ptr()).items.offset(i as isize)).state =
                    kDecorProviderWinDisabled;
            }
            hl_check_ns();
        }
        i = i.wrapping_add(1);
    }
    (*decor_state.ptr()).running_decor_provider = false_0 != 0;
}
pub unsafe extern "C" fn decor_providers_invoke_range(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
) {
    (*decor_state.ptr()).running_decor_provider = true_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_range != LUA_NOREF
        {
            if !((*p).win_skip_row > end_row
                || (*p).win_skip_row == end_row && (*p).win_skip_col >= end_col)
            {
                let mut args: Array = ARRAY_DICT_INIT;
                let mut args__items: [Object; 6] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_12 { boolean: false },
                }; 6];
                args.capacity = 6 as size_t;
                args.items = &raw mut args__items as *mut Object;
                let c2rust_fresh17 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh17 as isize) = object {
                    type_0: kObjectTypeWindow,
                    data: C2Rust_Unnamed_12 {
                        integer: (*wp).handle as Integer,
                    },
                };
                let c2rust_fresh18 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh18 as isize) = object {
                    type_0: kObjectTypeBuffer,
                    data: C2Rust_Unnamed_12 {
                        integer: (*(*wp).w_buffer).handle as Integer,
                    },
                };
                let c2rust_fresh19 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh19 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: start_row as Integer,
                    },
                };
                let c2rust_fresh20 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh20 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: start_col as Integer,
                    },
                };
                let c2rust_fresh21 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh21 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: end_row as Integer,
                    },
                };
                let c2rust_fresh22 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh22 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: end_col as Integer,
                    },
                };
                let mut res: Array = ARRAY_DICT_INIT;
                let mut status: bool = decor_provider_invoke(
                    i as ::core::ffi::c_int,
                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                    (*p).redraw_range,
                    args,
                    true_0 != 0,
                    &raw mut res,
                );
                p = (*decor_providers.ptr()).items.offset(i as isize);
                if !status {
                    (*p).state = kDecorProviderWinDisabled;
                } else if res.size >= 1 as size_t {
                    let mut first: Object = *res.items.offset(0 as ::core::ffi::c_int as isize);
                    if first.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if first.data.boolean as ::core::ffi::c_int == false_0 {
                            (*p).state = kDecorProviderWinDisabled;
                        }
                    } else if first.type_0 as ::core::ffi::c_uint
                        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut row: Integer = first.data.integer;
                        let mut col: Integer = 0 as Integer;
                        if res.size >= 2 as size_t {
                            let mut second: Object =
                                *res.items.offset(1 as ::core::ffi::c_int as isize);
                            if second.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                col = second.data.integer;
                            }
                        }
                        (*p).win_skip_row = row as ::core::ffi::c_int;
                        (*p).win_skip_col = col as ::core::ffi::c_int;
                    }
                }
                api_free_array(res);
                hl_check_ns();
            }
        }
        i = i.wrapping_add(1);
    }
    (*decor_state.ptr()).running_decor_provider = false_0 != 0;
}
pub unsafe extern "C" fn decor_providers_invoke_buf(mut buf: *mut buf_T) {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_buf != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh23 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh23 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh24 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh24 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick.get() as int64_t,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_buf,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn decor_providers_invoke_end() {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_end != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 1];
            args.capacity = 1 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh25 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh25 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick.get() as ::core::ffi::c_int as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"end\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_end,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
    decor_check_to_be_deleted();
}
pub unsafe extern "C" fn decor_provider_invalidate_hl() {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_providers.ptr()).size {
        (*(*decor_providers.ptr()).items.offset(i as isize)).hl_cached = false_0 != 0;
        i = i.wrapping_add(1);
    }
    if ns_hl_active.get() != 0 {
        ns_hl_active.set(-1 as ::core::ffi::c_int as NS);
        hl_check_ns();
    }
}
pub unsafe extern "C" fn get_decor_provider(mut ns_id: NS, mut force: bool) -> *mut DecorProvider {
    '_c2rust_label: {
        if ns_id > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"ns_id > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration_provider.rs\0".as_ptr() as *const ::core::ffi::c_char,
                305 as ::core::ffi::c_uint,
                b"DecorProvider *get_decor_provider(NS, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: size_t = (*decor_providers.ptr()).size;
    let mut i: size_t = 0 as size_t;
    while i < len {
        let mut p: *mut DecorProvider = (*decor_providers.ptr()).items.offset(i as isize);
        if (*p).ns_id == ns_id {
            return p;
        }
        i = i.wrapping_add(1);
    }
    if !force {
        return ::core::ptr::null_mut::<DecorProvider>();
    }
    if (*decor_providers.ptr()).capacity <= len {
        (*decor_providers.ptr()).size = len.wrapping_add(1 as size_t);
        (*decor_providers.ptr()).capacity = (*decor_providers.ptr()).size;
        (*decor_providers.ptr()).capacity = (*decor_providers.ptr()).capacity.wrapping_sub(1);
        (*decor_providers.ptr()).capacity |=
            (*decor_providers.ptr()).capacity >> 1 as ::core::ffi::c_int;
        (*decor_providers.ptr()).capacity |=
            (*decor_providers.ptr()).capacity >> 2 as ::core::ffi::c_int;
        (*decor_providers.ptr()).capacity |=
            (*decor_providers.ptr()).capacity >> 4 as ::core::ffi::c_int;
        (*decor_providers.ptr()).capacity |=
            (*decor_providers.ptr()).capacity >> 8 as ::core::ffi::c_int;
        (*decor_providers.ptr()).capacity |=
            (*decor_providers.ptr()).capacity >> 16 as ::core::ffi::c_int;
        (*decor_providers.ptr()).capacity = (*decor_providers.ptr()).capacity.wrapping_add(1);
        (*decor_providers.ptr()).capacity = (*decor_providers.ptr()).capacity;
        (*decor_providers.ptr()).items = xrealloc(
            (*decor_providers.ptr()).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<DecorProvider>().wrapping_mul((*decor_providers.ptr()).capacity),
        ) as *mut DecorProvider;
    } else {
        if (*decor_providers.ptr()).size <= len {
            (*decor_providers.ptr()).size = len.wrapping_add(1 as size_t);
        } else {
        };
    };
    let mut item: *mut DecorProvider = (*decor_providers.ptr()).items.offset(len as isize);
    *item = DecorProvider {
        ns_id: ns_id,
        state: kDecorProviderDisabled,
        win_skip_row: 0 as ::core::ffi::c_int,
        win_skip_col: 0 as ::core::ffi::c_int,
        redraw_start: LUA_NOREF,
        redraw_buf: LUA_NOREF,
        redraw_win: LUA_NOREF,
        redraw_line: LUA_NOREF,
        redraw_range: LUA_NOREF,
        redraw_end: LUA_NOREF,
        hl_def: LUA_NOREF,
        spell_nav: LUA_NOREF,
        conceal_line: -1 as LuaRef,
        hl_valid: false_0,
        hl_cached: false_0 != 0,
        error_count: 0 as uint8_t,
    };
    return item;
}
pub unsafe extern "C" fn decor_provider_clear(mut p: *mut DecorProvider) {
    if p.is_null() {
        return;
    }
    if (*p).redraw_start != LUA_NOREF {
        api_free_luaref((*p).redraw_start);
        (*p).redraw_start = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_buf != LUA_NOREF {
        api_free_luaref((*p).redraw_buf);
        (*p).redraw_buf = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_win != LUA_NOREF {
        api_free_luaref((*p).redraw_win);
        (*p).redraw_win = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_line != LUA_NOREF {
        api_free_luaref((*p).redraw_line);
        (*p).redraw_line = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_range != LUA_NOREF {
        api_free_luaref((*p).redraw_range);
        (*p).redraw_range = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_end != LUA_NOREF {
        api_free_luaref((*p).redraw_end);
        (*p).redraw_end = LUA_NOREF as LuaRef;
    }
    if (*p).spell_nav != LUA_NOREF {
        api_free_luaref((*p).spell_nav);
        (*p).spell_nav = LUA_NOREF as LuaRef;
    }
    if (*p).conceal_line != LUA_NOREF {
        api_free_luaref((*p).conceal_line);
        (*p).conceal_line = LUA_NOREF as LuaRef;
    }
    (*p).state = kDecorProviderDisabled;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
