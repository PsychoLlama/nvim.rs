//! Repr(C) mirror of `buf_T` from `buffer_defs.h`.
//!
//! This struct provides direct field access to `buf_T` from Rust, eliminating
//! the need for C accessor functions. Layout validated by `_Static_assert`
//! checks in `src/nvim/buf_struct_check.c`.
//!
//! # Safety
//! This struct MUST match the C `buf_T` layout exactly. All offsets are
//! validated at compile time via C static assertions.

#![allow(dead_code)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};

/// Neovim `linenr_T` (`int32_t`)
pub type LineNr = i32;
/// Neovim `colnr_T` (int/i32)
pub type ColNr = i32;
/// Neovim `handle_T` (int)
pub type HandleT = c_int;
/// Neovim `OptInt` (`int64_t`)
pub type OptInt = i64;
/// Neovim `varnumber_T` (`int64_t`)
pub type VarNumber = i64;

/// Mirror of C `pos_T` from `pos_defs.h` (12 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: LineNr,
    pub col: ColNr,
    pub coladd: ColNr,
}

/// Mirror of C `visualinfo_T` (32 bytes, offset 1424 in `buf_T`).
/// Layout: `vi_start` (`pos_T=12b`), `vi_end` (`pos_T=12b`), `vi_mode` (int=4b), `vi_curswant` (`colnr_T=4b`)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VisualInfoT {
    pub vi_start: PosT,
    pub vi_end: PosT,
    pub vi_mode: c_int,
    pub vi_curswant: ColNr,
}

/// Repr(C) mirror of C `buf_T` (`struct file_buffer`).
///
/// Fields are laid out exactly as in `buffer_defs.h`.
/// Complex nested types not accessed directly from Rust use
/// opaque `[u8; N]` padding to preserve correct offsets.
///
/// All field offsets are validated by `_Static_assert` in
/// `src/nvim/buf_struct_check.c`.
///
/// Field sizes measured from the actual C struct on `x86_64` Linux:
/// - "bool" options are actually C `int` (4 bytes) in this struct
/// - `OptInt` options are `int64_t` (8 bytes)
/// - Pointer fields are 8 bytes on `x86_64`
#[repr(C)]
pub struct BufStruct {
    // --- HEADER (offsets 0-119) ---

    // offset 0: handle_T handle  (b_fnum, int = 4 bytes)
    pub handle: HandleT,
    _pad0: [u8; 4], // to offset 8

    // offset 8: memline_T b_ml (112 bytes total).
    //   ml_line_count at sub-offset  0 (linenr_T = i32)
    //   ml_mfp        at sub-offset  8 (pointer = 8 bytes)
    //   ml_flags      at sub-offset 32 (int = 4 bytes)
    pub ml_line_count: LineNr, // abs 8
    _pad_ml0: [u8; 4],         // abs 12..15
    pub ml_mfp: *mut c_void,   // abs 16 (memfile_T*)
    _pad_ml1: [u8; 16],        // abs 24..39
    pub ml_flags: c_int,       // abs 40
    _pad_ml2: [u8; 76],        // abs 44..119  (total memline_T = 112, ends at 120)

    // --- BUFFER LIST (offsets 120-159) ---

    // offset 120: buf_T* b_next
    pub b_next: *mut c_void,

    // offset 128: buf_T* b_prev
    pub b_prev: *mut c_void,

    // offset 136: int b_nwindows
    pub b_nwindows: c_int,
    // offset 140: int b_flags
    pub b_flags: c_int,
    // offset 144: int b_locked
    pub b_locked: c_int,
    // offset 148: int b_locked_split
    pub b_locked_split: c_int,
    // offset 152: int b_ro_locked
    pub b_ro_locked: c_int,
    _pad1: [u8; 4], // abs 156..159

    // --- FILE NAMES (offsets 160-207) ---

    // offset 160: char* b_ffname
    pub b_ffname: *const c_char,
    // offset 168: char* b_sfname
    pub b_sfname: *const c_char,
    // offset 176: char* b_fname
    pub b_fname: *const c_char,

    // offset 184: int file_id_valid
    pub file_id_valid: c_int,
    _pad2: [u8; 4], // abs 188..191

    // offset 192: FileID file_id (16 bytes, opaque)
    pub file_id: [u8; 16],

    // --- CHANGE TRACKING (offsets 208-271) ---

    // offset 208: int b_changed
    pub b_changed: c_int,
    // offset 212: int b_changed_invalid (bool stored as int alignment)
    pub b_changed_invalid: c_int,

    // offset 216: ChangedtickDictItem changedtick_di (32 bytes, opaque)
    _changedtick_di: [u8; 32],

    // offset 248: varnumber_T b_last_changedtick (int64_t)
    pub b_last_changedtick: VarNumber,
    // offset 256: varnumber_T b_last_changedtick_i (int64_t)
    pub b_last_changedtick_i: VarNumber,
    // offset 264: varnumber_T b_last_changedtick_pum (int64_t)
    pub b_last_changedtick_pum: VarNumber,

    // --- SAVE/MOD STATE (offsets 272-327) ---

    // offset 272: bool b_saving (1 byte in C)
    pub b_saving: u8,
    // offset 273: bool b_mod_set (1 byte in C)
    pub b_mod_set: u8,
    _pad3: [u8; 2], // abs 274..275

    // offset 276: linenr_T b_mod_top
    pub b_mod_top: LineNr,
    // offset 280: linenr_T b_mod_bot
    pub b_mod_bot: LineNr,
    // offset 284: linenr_T b_mod_xlines
    pub b_mod_xlines: LineNr,

    // offset 288..311: b_wininfo (kvec_t(WinInfo*) = 24 bytes, opaque)
    _pad4a: [u8; 24],
    // offset 312: disptick_T b_mod_tick_syn (uint64_t)
    pub b_mod_tick_syn: u64,
    // offset 320: disptick_T b_mod_tick_decor (uint64_t)
    pub b_mod_tick_decor: u64,

    // --- TIME/SIZE (offsets 328-383) ---

    // offset 328: int64_t b_mtime
    pub b_mtime: i64,
    // offset 336: int64_t b_mtime_ns
    pub b_mtime_ns: i64,
    // offset 344: int64_t b_mtime_read
    pub b_mtime_read: i64,
    // offset 352: int64_t b_mtime_read_ns
    pub b_mtime_read_ns: i64,
    // offset 360: uint64_t b_orig_size
    pub b_orig_size: u64,
    // offset 368: int b_orig_mode
    pub b_orig_mode: c_int,
    _pad5: [u8; 4], // abs 372..375

    // offset 376: int64_t b_last_used (time_t on x86_64)
    pub b_last_used: i64,

    // --- NAMED MARKS (offsets 384-1423, opaque) ---
    // b_namedm: 1040 bytes
    _pad6: [u8; 1040],

    // --- VISUAL / MARKS (offsets 1424-1583) ---

    // offset 1424: visualinfo_T b_visual (32 bytes)
    pub b_visual: VisualInfoT,

    // offset 1456: int b_visual_mode_eval
    pub b_visual_mode_eval: c_int,
    _pad7: [u8; 4], // abs 1460..1463

    // offset 1464: fmark_T b_last_cursor (40 bytes, opaque)
    pub b_last_cursor: [u8; 40],
    // offset 1504: fmark_T b_last_insert (40 bytes, opaque)
    pub b_last_insert: [u8; 40],
    // offset 1544: fmark_T b_last_change (40 bytes, opaque)
    pub b_last_change: [u8; 40],

    // --- CHANGELIST (offsets 1584-5591) ---

    // offset 1584..5583: b_changelist (4000 bytes, opaque)
    _pad8: [u8; 4000],

    // offset 5584: int b_changelistlen
    pub b_changelistlen: c_int,
    // offset 5588: bool b_new_change (1 byte in C, stored in int-aligned slot)
    pub b_new_change: u8,
    _pad9: [u8; 3], // abs 5589..5591

    // --- CHARTAB (offsets 5592-5623) ---

    // offset 5592: uint64_t b_chartab[4] (32 bytes)
    pub b_chartab: [u64; 4],

    // --- MAP/UCMD (offsets 5624-7703, opaque) ---
    _pad10: [u8; 2080],

    // --- OP MARKS (offsets 7704-7739) ---

    // offset 7704: pos_T b_op_start (12 bytes)
    pub b_op_start: PosT,
    // offset 7716: pos_T b_op_start_orig (12 bytes)
    pub b_op_start_orig: PosT,
    // offset 7728: pos_T b_op_end (12 bytes)
    pub b_op_end: PosT,

    // --- BOOL FLAGS (offsets 7740-7751) ---

    // offset 7740: bool b_marks_read (1 byte in C)
    pub b_marks_read: u8,
    // offset 7741: bool b_modified_was_set
    pub b_modified_was_set: u8,
    // offset 7742: bool b_did_filetype
    pub b_did_filetype: u8,
    // offset 7743: bool b_keep_filetype
    pub b_keep_filetype: u8,
    // offset 7744: bool b_au_did_filetype
    pub b_au_did_filetype: u8,
    _pad11: [u8; 7], // abs 7745..7751 (align to 7752 for pointer)

    // --- UNDO HEADERS (offsets 7752-7839) ---

    // offset 7752: uhp* b_u_oldhead (pointer)
    _b_u_oldhead: *mut c_void,
    // offset 7760: uhp* b_u_newhead (pointer)
    _b_u_newhead: *mut c_void,
    // offset 7768: uhp* b_u_curhead (pointer)
    _b_u_curhead: *mut c_void,
    // offset 7776: int b_u_numhead
    _b_u_numhead: c_int,

    // offset 7780: bool b_u_synced (1 byte in C)
    pub b_u_synced: u8,
    _pad12: [u8; 3], // abs 7781..7783

    // offset 7784: int b_u_seq_last
    pub b_u_seq_last: c_int,
    // offset 7788: int b_u_save_nr_last
    pub b_u_save_nr_last: c_int,
    // offset 7792: int b_u_seq_cur
    pub b_u_seq_cur: c_int,
    _pad13: [u8; 4], // abs 7796..7799

    // offset 7800..7839: b_u_time_cur(8), b_u_save_nr_cur(4), gap(4), b_u_line_ptr(8),
    //                    b_u_line_lnum(4), b_u_line_colnr(4), b_scanned(4), gap(4) = 40 bytes
    _pad14: [u8; 40],

    // --- INPUT MODE (offsets 7840-7887) ---

    // offset 7840: OptInt b_p_iminsert (int64_t)
    pub b_p_iminsert: OptInt,
    // offset 7848: OptInt b_p_imsearch (int64_t)
    pub b_p_imsearch: OptInt,

    // offset 7856: int16_t b_kmap_state
    pub b_kmap_state: i16,
    _pad15: [u8; 6], // abs 7858..7863

    // offset 7864..7887: b_kmap_ga garray_T (24 bytes, opaque)
    _pad16: [u8; 24],

    // --- OPTIONS INITIALIZED FLAG (offset 7888) ---

    // offset 7888: bool b_p_initialized (1 byte in C)
    pub b_p_initialized: u8,
    _pad17: [u8; 7], // abs 7889..7895

    // --- OPTION SCRIPT CONTEXT (offsets 7896-10079, opaque) ---
    // 10080 - 7896 = 2184 bytes
    _pad18: [u8; 2184],

    // ===== BUFFER OPTIONS REGION (offsets 10080-10927) =====
    //
    // All "bool" options here are stored as C `int` (4 bytes).
    // All pointer options are 8 bytes (x86_64).
    // All OptInt options are int64_t (8 bytes).
    //
    // Fields are in struct order; gaps are opaque padding.

    // offset 10080: int b_p_ac  (AutoComplete, stored as int not OptInt despite type alias)
    pub b_p_ac: c_int,
    // offset 10084: int b_p_ai  (AutoIndent)
    pub b_p_ai: c_int,
    // offset 10088: int b_p_ai_nopaste
    pub b_p_ai_nopaste: c_int,

    // offset 10092..10107: other bool options (16 bytes, opaque)
    _pad20: [u8; 16],
    // offset 10108: int b_p_ci  (CopyIndent)
    pub b_p_ci: c_int,

    // offset 10112: int b_p_bin  (BINary)
    pub b_p_bin: c_int,
    // offset 10116: int b_p_bomb
    pub b_p_bomb: c_int,

    // offset 10120: char* b_p_bh (BufHidden, pointer to string option)
    pub b_p_bh: *const c_char,
    // offset 10128: char* b_p_bt (BufType, pointer to string option)
    pub b_p_bt: *const c_char,

    // offset 10136: int64_t b_p_busy
    pub b_p_busy: OptInt,

    // offset 10144..10147: gap (4 bytes to align b_p_bl to int boundary)
    _pad23: [u8; 4],

    // offset 10148: int b_p_bl  (BufListed)
    pub b_p_bl: c_int,

    // offset 10152: int64_t b_p_channel
    pub b_p_channel: OptInt,

    // offset 10160: int b_p_cin  (CIndent)
    pub b_p_cin: c_int,
    // offset 10164..10199: gap (36 bytes, opaque)
    _pad25a: [u8; 36],
    // offset 10200: char* b_p_com  (Comments)
    pub b_p_com: *const c_char,
    // offset 10208..10351: remaining option fields (144 bytes, opaque)
    _pad25b: [u8; 144],

    // offset 10352: int b_p_eof
    pub b_p_eof: c_int,
    // offset 10356: int b_p_eol
    pub b_p_eol: c_int,
    // offset 10360: int b_p_fixeol
    pub b_p_fixeol: c_int,
    // offset 10364: int b_p_et  (ExpandTab)
    pub b_p_et: c_int,
    // offset 10368: int b_p_et_nobin
    pub b_p_et_nobin: c_int,
    // offset 10372: int b_p_et_nopaste
    pub b_p_et_nopaste: c_int,

    // offset 10376: char* b_p_fenc (FileENCoding)
    pub b_p_fenc: *const c_char,
    // offset 10384: char* b_p_ff  (FileFormat)
    pub b_p_ff: *const c_char,
    // offset 10392: char* b_p_ft  (FileType)
    pub b_p_ft: *const c_char,

    // offset 10400..10423: gap (24 bytes, opaque)
    _pad32: [u8; 24],

    // offset 10424: char* b_p_isk (IsKeyword)
    pub b_p_isk: *const c_char,

    // offset 10432..10447: gap (16 bytes, opaque)
    _pad33: [u8; 16],

    // offset 10448: char* b_p_inex (IncludeEXpr)
    pub b_p_inex: *const c_char,

    // offset 10456..10463: gap (8 bytes, opaque)
    _pad34: [u8; 8],

    // offset 10464: char* b_p_inde (INDentExpr)
    pub b_p_inde: *const c_char,

    // offset 10472..10487: gap (16 bytes, opaque)
    _pad35: [u8; 16],

    // offset 10488: char* b_p_fp  (FormatPrg)
    pub b_p_fp: *const c_char,
    // offset 10496: char* b_p_fex (FormatEXpr)
    pub b_p_fex: *const c_char,

    // offset 10504..10511: gap (8 bytes, opaque)
    _pad36: [u8; 8],

    // offset 10512: char* b_p_kp  (KeyProgram)
    pub b_p_kp: *const c_char,

    // offset 10520: int b_p_lisp  (LISP mode)
    pub b_p_lisp: c_int,

    // offset 10524..10543: gap (20 bytes, opaque)
    _pad37: [u8; 20],

    // offset 10544: char* b_p_mps  (MatchPairS, mutable for save/restore)
    pub b_p_mps: *mut c_char,

    // offset 10552: int b_p_ml    (Modeline)
    pub b_p_ml: c_int,
    // offset 10556: int b_p_ml_nobin
    pub b_p_ml_nobin: c_int,
    // offset 10560: int b_p_ma    (Modifiable)
    pub b_p_ma: c_int,

    // offset 10564..10567: gap (4 bytes, opaque)
    _pad39: [u8; 4],

    // offset 10568: char* b_p_nf  (NumberFormats)
    pub b_p_nf: *const c_char,

    // offset 10576: int b_p_pi  (PreserveIndent)
    pub b_p_pi: c_int,
    // offset 10580..10591: gap (12 bytes, opaque)
    _pad41: [u8; 12],

    // offset 10592: int b_p_ro    (ReadOnly)
    pub b_p_ro: c_int,

    // offset 10596..10599: gap (4 bytes, opaque)
    _pad42: [u8; 4],

    // offset 10600: OptInt b_p_sw  (ShiftWidth, int64_t)
    pub b_p_sw: OptInt,

    // offset 10608: OptInt b_p_scbk (scrollback, int64_t)
    pub b_p_scbk: OptInt,
    // offset 10616..10623: b_p_si (int) + gap (4 bytes, opaque)
    _pad43: [u8; 8],

    // offset 10624: OptInt b_p_sts  (SofttabStop, int64_t)
    pub b_p_sts: OptInt,
    // offset 10632: OptInt b_p_sts_nopaste (int64_t)
    pub b_p_sts_nopaste: OptInt,

    // offset 10640..10663: gap (24 bytes, opaque)
    _pad44: [u8; 24],

    // offset 10664: char* b_p_syn  (SYNtax)
    pub b_p_syn: *const c_char,

    // offset 10672: OptInt b_p_ts  (TabStop, int64_t)
    pub b_p_ts: OptInt,
    // offset 10680: OptInt b_p_tw  (TextWidth, int64_t)
    pub b_p_tw: OptInt,
    // offset 10688: OptInt b_p_tw_nobin (int64_t)
    pub b_p_tw_nobin: OptInt,
    // offset 10696: OptInt b_p_tw_nopaste (int64_t)
    pub b_p_tw_nopaste: OptInt,

    // offset 10704..10711: gap (8 bytes, opaque)
    _pad45: [u8; 8],

    // offset 10712: OptInt b_p_wm_nobin (int64_t)
    pub b_p_wm_nobin: OptInt,
    // offset 10720: OptInt b_p_wm_nopaste (int64_t)
    pub b_p_wm_nopaste: OptInt,

    // offset 10728..10735: gap (8 bytes, opaque)
    _pad46: [u8; 8],

    // offset 10736: int* b_p_vsts_array (pointer)
    pub b_p_vsts_array: *mut c_int,

    // offset 10744..10759: gap (16 bytes, opaque)
    _pad47: [u8; 16],

    // offset 10760: int* b_p_vts_array (pointer)
    pub b_p_vts_array: *mut c_int,

    // offset 10768..10799: gap (32 bytes, opaque)
    _pad48: [u8; 32],

    // offset 10800: char* b_p_efm  (ErrorFormatMsg)
    pub b_p_efm: *const c_char,

    // offset 10808..10815: gap (8 bytes, opaque)
    _pad49: [u8; 8],

    // offset 10816: char* b_p_path
    pub b_p_path: *const c_char,

    // offset 10824: int b_p_ar    (AutoRead, int not int64_t)
    pub b_p_ar: c_int,

    // offset 10828..10903: gap (76 bytes, opaque)
    _pad50: [u8; 76],

    // offset 10904: OptInt b_p_ul  (UndoLevels, int64_t)
    pub b_p_ul: OptInt,

    // offset 10912: int b_p_udf   (UnDo File, int not bool)
    pub b_p_udf: c_int,

    // offset 10916..10927: gap (12 bytes, opaque)
    _pad51: [u8; 12],

    // --- INDENT FIELDS (offsets 10928-11075) ---

    // offset 10928: int b_ind_level
    pub b_ind_level: c_int,

    // offset 10932..11075: remaining indent fields (144 bytes, opaque)
    _pad52: [u8; 144],

    // --- TAIL SECTION (offsets 11076-12727) ---

    // offset 11076: linenr_T b_no_eol_lnum
    pub b_no_eol_lnum: LineNr,
    // offset 11080: int b_start_eof
    pub b_start_eof: c_int,
    // offset 11084: int b_start_eol
    pub b_start_eol: c_int,

    // offset 11088: int b_start_ffc  (first char of 'ff' when edit started)
    pub b_start_ffc: c_int,
    // offset 11092..11095: alignment gap (4 bytes, opaque)
    _pad53a: [u8; 4],
    // offset 11096: char* b_start_fenc  ('fileencoding' when edit started or NULL)
    pub b_start_fenc: *const c_char,
    // offset 11104..11107: gap (4 bytes, opaque)
    _pad53b: [u8; 4],

    // offset 11108: int b_start_bomb
    pub b_start_bomb: c_int,

    // offset 11112..11135: b_bufvar ScopeDictDictItem (24 bytes, opaque)
    _pad54: [u8; 24],

    // offset 11136: dict_T* b_vars (pointer)
    pub b_vars: *mut c_void,

    // offset 11144: bool b_may_swap (1 byte in C)
    pub b_may_swap: u8,
    // offset 11145: bool b_did_warn (1 byte in C)
    pub b_did_warn: u8,
    // offset 11146: bool b_help (1 byte in C)
    pub b_help: u8,
    // offset 11147: bool b_spell (1 byte in C)
    pub b_spell: u8,

    // offset 11148..11239: b_prompt_text, b_prompt_start, other fields (92 bytes, opaque)
    _pad55: [u8; 92],

    // offset 11240..12407: synblock_T b_s (1168 bytes, opaque)
    _pad56: [u8; 1168],

    // offset 12408..12455: b_signcols (48 bytes, opaque)
    _pad57: [u8; 48],

    // offset 12456: Terminal* terminal (pointer)
    pub terminal: *mut c_void,
    // offset 12464: dict_T* additional_data (pointer)
    pub additional_data: *mut c_void,

    // offset 12472: int b_mapped_ctrl_c
    pub b_mapped_ctrl_c: c_int,
    _pad58: [u8; 4], // abs 12476..12479

    // offset 12480..12575: MarkTree b_marktree[kMTCount] (96 bytes, opaque)
    _pad59: [u8; 96],

    // offset 12576..12623: other fields (48 bytes, opaque)
    _pad60: [u8; 48],

    // offset 12624: int b_prev_line_count
    pub b_prev_line_count: c_int,
    _pad61: [u8; 4], // abs 12628..12631

    // offset 12632..12655: update_channels (kvec_t(uint64_t) = 24 bytes, opaque)
    _pad62a: [u8; 24],
    // offset 12656..12679: update_callbacks (kvec_t(BufUpdateCallbacks) = 24 bytes, opaque)
    _pad62b: [u8; 24],
    // offset 12680..12687: update_need_codepoints (bool=1) + 7 bytes alignment padding
    _pad62c: [u8; 8],

    // offset 12688: size_t deleted_bytes
    pub deleted_bytes: usize,
    // offset 12696: size_t deleted_bytes2
    pub deleted_bytes2: usize,
    // offset 12704: size_t deleted_codepoints
    pub deleted_codepoints: usize,
    // offset 12712: size_t deleted_codeunits
    pub deleted_codeunits: usize,

    // offset 12720: int flush_count
    pub flush_count: c_int,
    _pad_end: [u8; 4], // abs 12724..12727  (total = 12728)
}

// Verify overall size at compile time.
const _: () = assert!(std::mem::size_of::<BufStruct>() == 12728);

impl BufStruct {
    /// Get the `b_ml.ml_line_count` value (number of lines in buffer).
    #[inline]
    #[must_use]
    pub fn ml_line_count(&self) -> LineNr {
        self.ml_line_count
    }

    /// Check if `b_ml.ml_flags` has `ML_EMPTY` bit set (bit 0 = 0x01).
    #[inline]
    #[must_use]
    pub fn ml_is_empty(&self) -> bool {
        // ML_EMPTY = 0x01
        (self.ml_flags & 0x01) != 0
    }

    /// Check if `b_ml.ml_mfp` is null (no memfile/swap).
    #[inline]
    #[must_use]
    pub fn ml_mfp_is_null(&self) -> bool {
        self.ml_mfp.is_null()
    }

    /// Get the first character of `b_p_bt` (buftype option).
    ///
    /// Returns 0 if the pointer is null.
    ///
    /// # Safety
    /// `b_p_bt` must be a valid C string pointer.
    #[inline]
    #[must_use]
    pub unsafe fn buftype_char0(&self) -> u8 {
        if self.b_p_bt.is_null() {
            0
        } else {
            *self.b_p_bt as u8
        }
    }

    /// Get the third character (index 2) of `b_p_bt` (buftype option).
    ///
    /// Used to distinguish "nofile" (n,o,f) from "nowrite" (n,o,w).
    ///
    /// # Safety
    /// `b_p_bt` must be a valid C string pointer of at least 3 chars.
    #[inline]
    #[must_use]
    pub unsafe fn buftype_char2(&self) -> u8 {
        if self.b_p_bt.is_null() {
            0
        } else {
            *self.b_p_bt.add(2) as u8
        }
    }

    /// Get the first character of `b_p_bh` (bufhidden option).
    ///
    /// # Safety
    /// `b_p_bh` must be a valid C string pointer.
    #[inline]
    #[must_use]
    pub unsafe fn bufhidden_char0(&self) -> u8 {
        if self.b_p_bh.is_null() {
            0
        } else {
            *self.b_p_bh as u8
        }
    }

    /// Get the first character of `b_p_ff` (fileformat option).
    ///
    /// # Safety
    /// `b_p_ff` must be a valid C string pointer.
    #[inline]
    #[must_use]
    pub unsafe fn fileformat_char0(&self) -> u8 {
        if self.b_p_ff.is_null() {
            0
        } else {
            *self.b_p_ff as u8
        }
    }

    /// Copy the `file_id` bytes into `out`.
    ///
    /// # Safety
    /// `out` must point to at least 16 bytes of writable memory.
    #[inline]
    pub unsafe fn copy_file_id(&self, out: *mut u8) {
        std::ptr::copy_nonoverlapping(self.file_id.as_ptr(), out, 16);
    }

    /// Set the `file_id` and `file_id_valid` fields.
    ///
    /// # Safety
    /// If `valid` is true, `file_id` must point to at least 16 bytes of readable memory.
    #[inline]
    pub unsafe fn set_file_id_data(&mut self, file_id: *const u8, valid: bool) {
        if valid {
            std::ptr::copy_nonoverlapping(file_id, self.file_id.as_mut_ptr(), 16);
        }
        self.file_id_valid = c_int::from(valid);
    }
}

/// Get an immutable reference to `BufStruct` from a `BufHandle`.
///
/// # Safety
/// The handle must be a valid non-null `buf_T*` with a lifetime at least as
/// long as `'a`.
#[inline]
#[must_use]
pub const unsafe fn buf_ref<'a>(bp: crate::BufHandle) -> &'a BufStruct {
    &*(bp.as_ptr().cast::<BufStruct>())
}

/// Get a mutable reference to `BufStruct` from a `BufHandle`.
///
/// # Safety
/// The handle must be a valid non-null `buf_T*` with a lifetime at least as
/// long as `'a`, and the caller must guarantee exclusive access.
#[inline]
#[must_use]
pub unsafe fn buf_mut<'a>(bp: crate::BufHandle) -> &'a mut BufStruct {
    &mut *(bp.as_ptr().cast::<BufStruct>())
}
