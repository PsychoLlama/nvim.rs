// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type MessagePackType = ::core::ffi::c_uint;
pub type VimVarIndex = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
pub type exprtype_T = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
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
pub type var_flavour_T = ::core::ffi::c_uint;
