use crate::api::private::helpers::{ERROR_INIT, Reported, api_typename};
use crate::api::private::validate::api_err_exp;
use crate::autocmd::do_termresponse_autocmd;
use crate::eval::vars::set_vim_var_string;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::memory::strequal;
use crate::types::{
    Error, Integer, Object, String_0, VV_TERMRESPONSE, kObjectTypeString, ptrdiff_t, uint64_t,
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe fn nvim_error_event(channel_id: uint64_t, _type_0: Integer, msg: String_0) {
    logmsg_c!(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        c"nvim_error_event".as_ptr(),
        44 as ::core::ffi::c_int,
        true,
        c"async error on channel %ld: %s".as_ptr(),
        channel_id,
        if msg.size != 0 {
            msg.data as *const ::core::ffi::c_char
        } else {
            c"".as_ptr()
        },
    );
}
pub unsafe fn nvim_ui_term_event(
    _channel_id: uint64_t,
    event: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    if strequal(c"termresponse".as_ptr(), event.data) {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                c"termresponse".as_ptr(),
                api_typename(kObjectTypeString),
                api_typename(value.type_0),
            );
            return ().reported(error);
        }
        let termresponse: String_0 = value.data.string;
        set_vim_var_string(
            VV_TERMRESPONSE,
            termresponse.data,
            termresponse.size as ptrdiff_t,
        );
        do_termresponse_autocmd(termresponse);
    }
    ().reported(error)
}
