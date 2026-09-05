//! Where an option's value actually lives.
//!
//! Upstream declares these in `option_vars.h` as one `EXTERN` per option, and
//! the transpiler parked the lot in `startup/mod.rs` beside `main()`. They are
//! the storage half of an option: [`crate::options::options`] is the table
//! row -- name, type, scopes, flags -- and its `var` field points at the cell
//! here. `crate::option` is what reads and writes them.
//!
//! Two spellings share the file. `p_<abbrev>` is an option's value: a number,
//! a boolean as `c_int`, or the allocated string the option owns. `<abbrev>_flags`
//! is the decoded form of a string option whose value is a comma-separated
//! set, kept beside the string so a hot path tests a bit instead of parsing
//! ('backspace', 'clipboard', 'display', ...); `crate::optionstr` fills it in
//! from the `did_set_*` callback.
//!
//! The window- and buffer-local options do not live here: a window or buffer
//! carries its own copy, and these are the global values.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::types::{BreakAt, OptInt, uint8_t};
use core::ffi::{c_char, c_int, c_uint};

pub(crate) static fenc_default: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());

pub(crate) static wim_flags: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub(crate) static p_ambw: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_acd: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ai: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_bin: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_bomb: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_bl: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_cin: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_channel: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_cink: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cinsd: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cinw: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cfu: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ofu: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tsrfu: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ci: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ar: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_aw: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_awa: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_bs: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bk: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_bkc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static bkc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_bdir: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bex: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static breakat_flags: GlobalCell<BreakAt> = GlobalCell::new(BreakAt::NONE);
pub(crate) static bo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_bsk: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_breakat: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bh: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_bt: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_busy: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_cmp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static cmp_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_enc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_deco: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ccv: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cino: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cedit: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cb: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static cb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_cwh: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ch: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_cms: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cpt: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cto: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_columns: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_confirm: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_cia: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static cia_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_cot: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static cot_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_ac: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_act: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_acl: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_pumborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_pb: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ph: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_pw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_pmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_com: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cpo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_debug: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_def: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_inc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dia: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dip: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dex: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dict: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dg: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_dy: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static dy_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_ead: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_emoji: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ea: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ep: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_eb: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ef: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_efm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_gefm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_gp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_eof: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_eol: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ei: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_et: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_exrc: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_fenc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fencs: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ff: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ffs: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fic: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ft: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fcs: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ffu: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fixeol: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_fcl: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fdls: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_fdo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static fdo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_fex: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_flp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_fs: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_gd: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_guicursor: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_guifont: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_guifontwide: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_hf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_hh: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_hlg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_hid: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_hl: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_hls: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_hi: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_arshape: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_icon: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_iconstring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ic: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_iminsert: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_imsearch: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_inf: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_inex: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_is: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_inde: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_indk: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_icm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_isf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_isi: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_isk: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_isp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_js: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_jop: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static jop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_keymap: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_kp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_km: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_langmap: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_lnr: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_lrm: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_lm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_lines: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_linespace: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_lisp: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_lop: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_lispwords: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ls: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_stal: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_lcs: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_lz: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_lpl: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_magic: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_menc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mef: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mps: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mat: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mco: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mfd: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mmd: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mmp: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mis: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_mopt: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_msc: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_msm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ml: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mle: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mls: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ma: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mod: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mouse: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mousem: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mousemev: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mousef: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mh: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_mousescroll: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_mousescroll_vert: GlobalCell<OptInt> = GlobalCell::new(3);
pub(crate) static p_mousescroll_hor: GlobalCell<OptInt> = GlobalCell::new(6);
pub(crate) static p_mouset: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_more: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_nf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_opfunc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_para: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_paste: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_pex: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_pm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_path: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_cdpath: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_pi: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_pyx: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_qe: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ro: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_rdb: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static rdb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_rdt: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_re: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_report: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_pvh: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_chi: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ari: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ri: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ru: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ruf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_pp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_qftf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_rtp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_scbk: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_sj: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_so: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_sbo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sections: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_secure: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sel: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_slm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ssop: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static ssop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_sh: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_shcf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sp: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_shq: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sxq: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sxe: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_srr: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_stmp: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_stl: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_wbr: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sr: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_shm: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sbr: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sc: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sloc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_sft: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sm: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_smd: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ss: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_siso: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_scs: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_si: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sta: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sts: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_sb: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sua: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_swf: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_smc: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tpm: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tal: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tpf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static tpf_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_tfu: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_spc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_spf: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_spl: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_spo: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static spo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_sps: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_spr: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sol: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_su: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_swb: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static swb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_spk: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_syn: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tcl: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static tcl_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_ts: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tbs: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_tc: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static tc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_tl: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tr: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_tags: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tgst: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_tbidi: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_tw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_to: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_timeout: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_tm: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_title: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_titlelen: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_titleold: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_titlestring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tsr: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_tgc: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ttimeout: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ttm: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_udir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_udf: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ul: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ur: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_uc: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ut: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_shada: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_shadafile: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_termsync: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_vsts: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_vts: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_vdir: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_vop: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static vop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_vb: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_ve: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static ve_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_verbose: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_warn: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_wop: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static wop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_window: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wak: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_wig: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_ww: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_wc: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wcm: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wic: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_wim: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_wmnu: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_winborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static p_wh: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wmh: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wiw: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wm: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ws: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_write: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_wa: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_wb: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_wd: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_cdh: GlobalCell<c_int> = GlobalCell::new(0);
