//! The Vimscript builtins that work over a whole container.
//!
//! Carved by what the builtin does to it:
//!
//! | child | what |
//! | --- | --- |
//! | [`filtermap`] | `filter()`, `map()`, `mapnew()`, `foreach()` -- and, one level down, the four per-container walks |
//! | [`count`] | `count()` and `add()` |
//! | [`extend`] | `extend()`, `extendnew()`, `insert()` |
//!
//! What stays here is `remove()` and `reverse()` -- the two that only take
//! something out of a container or turn it around, with no expression and no
//! second container involved -- plus the `filtermap_T` constants and the two
//! statics the children share.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_blob_set, tv_blob_set_ret, tv_list_locked, tv_list_set_ret,
};
use crate::src::nvim::eval::typval::{
    tv_blob_remove, tv_check_for_string_or_list_or_blob_arg, tv_dict_remove, tv_list_remove,
    tv_list_reverse, value_check_lock,
};
use crate::src::nvim::main::{c_bytes, e_listdictblobarg};
use crate::src::nvim::strings::reverse_text;
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_LIST, VAR_STRING, blob_T, list_T, size_t, typval_T,
    uint8_t,
};

// The carve of the transpiled module; see each child's docs.
mod count;
mod extend;
mod filtermap;

pub use self::count::*;
pub use self::extend::*;
pub use self::filtermap::*;

pub type filtermap_T = ::core::ffi::c_uint;
pub const FILTERMAP_FOREACH: filtermap_T = 3;
pub const FILTERMAP_MAPNEW: filtermap_T = 2;
pub const FILTERMAP_MAP: filtermap_T = 1;
pub const FILTERMAP_FILTER: filtermap_T = 0;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static e_argument_of_str_must_be_list_string_or_dictionary: [::core::ffi::c_char; 58] =
    c_bytes(b"E706: Argument of %s must be a List, String or Dictionary\0");
static e_argument_of_str_must_be_list_string_dictionary_or_blob: [::core::ffi::c_char; 65] =
    c_bytes(b"E1250: Argument of %s must be a List, String, Dictionary or Blob\0");
pub unsafe extern "C" fn f_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let arg_errmsg: *const ::core::ffi::c_char = c"remove() argument".as_ptr();
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
            semsg_c!(
                &raw const e_listdictblobarg as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                c"remove()".as_ptr(),
            );
        };
    }
}
pub unsafe extern "C" fn f_reverse(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
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
                c"reverse() argument".as_ptr(),
                TV_TRANSLATE as size_t,
            ) {
                tv_list_reverse(l);
                tv_list_set_ret(rettv, l);
            }
        }
    }
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
