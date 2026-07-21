// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type MPConvPartialStage = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal {
    pub type_0: MPConvStackValType,
    pub tv: *mut typval_T,
    pub saved_copyID: ::core::ffi::c_int,
    pub data: MPConvStackVal_data,
}
pub type MPConvStackValType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union MPConvStackVal_data {
    pub d: MPConvStackVal_data_d,
    pub l: MPConvStackVal_data_l,
    pub p: MPConvStackVal_data_p,
    pub a: MPConvStackVal_data_a,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal_data_a {
    pub arg: *mut typval_T,
    pub argv: *mut typval_T,
    pub todo: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal_data_d {
    pub dict: *mut dict_T,
    pub dictp: *mut *mut dict_T,
    pub hi: *mut hashitem_T,
    pub todo: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal_data_l {
    pub list: *mut list_T,
    pub li: *mut listitem_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal_data_p {
    pub stage: MPConvPartialStage,
    pub pt: *mut partial_T,
}
