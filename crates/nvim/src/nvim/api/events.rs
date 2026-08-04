use crate::src::nvim::api::private::helpers::api_typename;
use crate::src::nvim::api::private::validate::api_err_exp;
use crate::src::nvim::autocmd::do_termresponse_autocmd;
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::memory::strequal;
use crate::src::nvim::types::{
    Error, Integer, Object, String_0, VV_TERMRESPONSE, kObjectTypeString, ptrdiff_t, uint64_t,
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub unsafe extern "C" fn nvim_error_event(
    mut channel_id: uint64_t,
    mut _type_0: Integer,
    mut msg: String_0,
) {
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"nvim_error_event\0".as_ptr() as *const ::core::ffi::c_char,
        44 as ::core::ffi::c_int,
        true_0 != 0,
        b"async error on channel %ld: %s\0".as_ptr() as *const ::core::ffi::c_char,
        channel_id,
        if msg.size != 0 {
            msg.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
}
pub unsafe extern "C" fn nvim_ui_term_event(
    mut _channel_id: uint64_t,
    mut event: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    if strequal(
        b"termresponse\0".as_ptr() as *const ::core::ffi::c_char,
        event.data,
    ) {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"termresponse\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename(value.type_0),
            );
            return;
        }
        let termresponse: String_0 = value.data.string;
        set_vim_var_string(
            VV_TERMRESPONSE,
            termresponse.data,
            termresponse.size as ptrdiff_t,
        );
        do_termresponse_autocmd(termresponse);
    }
}
