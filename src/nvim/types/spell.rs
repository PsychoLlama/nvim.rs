// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type SpellAddType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fromto_T {
    pub ft_from: *mut ::core::ffi::c_char,
    pub ft_to: *mut ::core::ffi::c_char,
}
pub type idx_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct langp_T {
    pub lp_slang: *mut slang_T,
    pub lp_sallang: *mut slang_T,
    pub lp_replang: *mut slang_T,
    pub lp_region: ::core::ffi::c_int,
}
pub type salfirst_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct salitem_T {
    pub sm_lead: *mut ::core::ffi::c_char,
    pub sm_leadlen: ::core::ffi::c_int,
    pub sm_oneof: *mut ::core::ffi::c_char,
    pub sm_rules: *mut ::core::ffi::c_char,
    pub sm_to: *mut ::core::ffi::c_char,
    pub sm_lead_w: *mut ::core::ffi::c_int,
    pub sm_oneof_w: *mut ::core::ffi::c_int,
    pub sm_to_w: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct slang_S {
    pub sl_next: *mut slang_T,
    pub sl_name: *mut ::core::ffi::c_char,
    pub sl_fname: *mut ::core::ffi::c_char,
    pub sl_add: bool,
    pub sl_fbyts: *mut uint8_t,
    pub sl_fbyts_len: ::core::ffi::c_int,
    pub sl_fidxs: *mut idx_T,
    pub sl_kbyts: *mut uint8_t,
    pub sl_kidxs: *mut idx_T,
    pub sl_pbyts: *mut uint8_t,
    pub sl_pidxs: *mut idx_T,
    pub sl_info: *mut ::core::ffi::c_char,
    pub sl_regions: [::core::ffi::c_char; 17],
    pub sl_midword: *mut ::core::ffi::c_char,
    pub sl_wordcount: hashtab_T,
    pub sl_compmax: ::core::ffi::c_int,
    pub sl_compminlen: ::core::ffi::c_int,
    pub sl_compsylmax: ::core::ffi::c_int,
    pub sl_compoptions: ::core::ffi::c_int,
    pub sl_comppat: garray_T,
    pub sl_compprog: *mut regprog_T,
    pub sl_comprules: *mut uint8_t,
    pub sl_compstartflags: *mut uint8_t,
    pub sl_compallflags: *mut uint8_t,
    pub sl_nobreak: bool,
    pub sl_syllable: *mut ::core::ffi::c_char,
    pub sl_syl_items: garray_T,
    pub sl_prefixcnt: ::core::ffi::c_int,
    pub sl_prefprog: *mut *mut regprog_T,
    pub sl_rep: garray_T,
    pub sl_rep_first: [int16_t; 256],
    pub sl_sal: garray_T,
    pub sl_sal_first: [salfirst_T; 256],
    pub sl_followup: bool,
    pub sl_collapse: bool,
    pub sl_rem_accents: bool,
    pub sl_sofo: bool,
    pub sl_repsal: garray_T,
    pub sl_repsal_first: [int16_t; 256],
    pub sl_nosplitsugs: bool,
    pub sl_nocompoundsugs: bool,
    pub sl_sugtime: time_t,
    pub sl_sbyts: *mut uint8_t,
    pub sl_sbyts_len: ::core::ffi::c_int,
    pub sl_sidxs: *mut idx_T,
    pub sl_sugbuf: *mut buf_T,
    pub sl_sugloaded: bool,
    pub sl_has_map: bool,
    pub sl_map_hash: hashtab_T,
    pub sl_map_array: [::core::ffi::c_int; 256],
    pub sl_sounddone: hashtab_T,
}
pub type slang_T = slang_S;
pub type smt_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spelltab_T {
    pub st_isw: [bool; 256],
    pub st_isu: [bool; 256],
    pub st_fold: [uint8_t; 256],
    pub st_upper: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wordcount_T {
    pub wc_count: uint16_t,
    pub wc_word: [::core::ffi::c_char; 0],
}
