/// Static assertions validating that Rust's BufStruct matches C's buf_T layout.
///
/// These compile-time checks ensure every field accessed directly from Rust
/// has the correct offset and the overall struct size matches.

#include <stddef.h>
#include "nvim/buffer_defs.h"

// Overall struct size
_Static_assert(sizeof(buf_T) == 12728,
               "buf_T size mismatch: update BufStruct in buf_struct.rs");

// Header fields
_Static_assert(offsetof(buf_T, handle) == 0, "handle offset mismatch");
_Static_assert(offsetof(buf_T, b_ml) == 8, "b_ml offset mismatch");
_Static_assert(offsetof(buf_T, b_next) == 120, "b_next offset mismatch");
_Static_assert(offsetof(buf_T, b_prev) == 128, "b_prev offset mismatch");

// State flags
_Static_assert(offsetof(buf_T, b_nwindows) == 136, "b_nwindows offset mismatch");
_Static_assert(offsetof(buf_T, b_flags) == 140, "b_flags offset mismatch");
_Static_assert(offsetof(buf_T, b_locked) == 144, "b_locked offset mismatch");
_Static_assert(offsetof(buf_T, b_locked_split) == 148, "b_locked_split offset mismatch");
_Static_assert(offsetof(buf_T, b_ro_locked) == 152, "b_ro_locked offset mismatch");

// File names
_Static_assert(offsetof(buf_T, b_ffname) == 160, "b_ffname offset mismatch");
_Static_assert(offsetof(buf_T, b_sfname) == 168, "b_sfname offset mismatch");
_Static_assert(offsetof(buf_T, b_fname) == 176, "b_fname offset mismatch");

// File ID
_Static_assert(offsetof(buf_T, file_id_valid) == 184, "file_id_valid offset mismatch");
_Static_assert(offsetof(buf_T, file_id) == 192, "file_id offset mismatch");

// Change tracking
_Static_assert(offsetof(buf_T, b_changed) == 208, "b_changed offset mismatch");
_Static_assert(offsetof(buf_T, b_changed_invalid) == 212, "b_changed_invalid offset mismatch");
_Static_assert(offsetof(buf_T, changedtick_di) == 216, "changedtick_di offset mismatch");
_Static_assert(offsetof(buf_T, b_last_changedtick) == 248, "b_last_changedtick offset mismatch");
_Static_assert(offsetof(buf_T, b_last_changedtick_i) == 256, "b_last_changedtick_i offset mismatch");
_Static_assert(offsetof(buf_T, b_last_changedtick_pum) == 264, "b_last_changedtick_pum offset mismatch");

// Save/mod state
_Static_assert(offsetof(buf_T, b_saving) == 272, "b_saving offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_set) == 273, "b_mod_set offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_top) == 276, "b_mod_top offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_bot) == 280, "b_mod_bot offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_xlines) == 284, "b_mod_xlines offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_tick_syn) == 312, "b_mod_tick_syn offset mismatch");
_Static_assert(offsetof(buf_T, b_mod_tick_decor) == 320, "b_mod_tick_decor offset mismatch");

// Time/size
_Static_assert(offsetof(buf_T, b_mtime) == 328, "b_mtime offset mismatch");
_Static_assert(offsetof(buf_T, b_mtime_ns) == 336, "b_mtime_ns offset mismatch");
_Static_assert(offsetof(buf_T, b_mtime_read) == 344, "b_mtime_read offset mismatch");
_Static_assert(offsetof(buf_T, b_mtime_read_ns) == 352, "b_mtime_read_ns offset mismatch");
_Static_assert(offsetof(buf_T, b_orig_size) == 360, "b_orig_size offset mismatch");
_Static_assert(offsetof(buf_T, b_orig_mode) == 368, "b_orig_mode offset mismatch");
_Static_assert(offsetof(buf_T, b_last_used) == 376, "b_last_used offset mismatch");

// Visual / marks region
_Static_assert(offsetof(buf_T, b_visual) == 1424, "b_visual offset mismatch");
_Static_assert(offsetof(buf_T, b_visual_mode_eval) == 1456, "b_visual_mode_eval offset mismatch");
_Static_assert(offsetof(buf_T, b_last_cursor) == 1464, "b_last_cursor offset mismatch");
_Static_assert(offsetof(buf_T, b_last_insert) == 1504, "b_last_insert offset mismatch");
_Static_assert(offsetof(buf_T, b_last_change) == 1544, "b_last_change offset mismatch");

// Changelist / chartab
_Static_assert(offsetof(buf_T, b_changelistlen) == 5584, "b_changelistlen offset mismatch");
_Static_assert(offsetof(buf_T, b_new_change) == 5588, "b_new_change offset mismatch");
_Static_assert(offsetof(buf_T, b_chartab) == 5592, "b_chartab offset mismatch");

// Op marks
_Static_assert(offsetof(buf_T, b_op_start) == 7704, "b_op_start offset mismatch");
_Static_assert(offsetof(buf_T, b_op_start_orig) == 7716, "b_op_start_orig offset mismatch");
_Static_assert(offsetof(buf_T, b_op_end) == 7728, "b_op_end offset mismatch");

// Bool flags
_Static_assert(offsetof(buf_T, b_marks_read) == 7740, "b_marks_read offset mismatch");
_Static_assert(offsetof(buf_T, b_modified_was_set) == 7741, "b_modified_was_set offset mismatch");
_Static_assert(offsetof(buf_T, b_did_filetype) == 7742, "b_did_filetype offset mismatch");
_Static_assert(offsetof(buf_T, b_keep_filetype) == 7743, "b_keep_filetype offset mismatch");
_Static_assert(offsetof(buf_T, b_au_did_filetype) == 7744, "b_au_did_filetype offset mismatch");

// Undo fields
_Static_assert(offsetof(buf_T, b_u_oldhead) == 7752, "b_u_oldhead offset mismatch");
_Static_assert(offsetof(buf_T, b_u_newhead) == 7760, "b_u_newhead offset mismatch");
_Static_assert(offsetof(buf_T, b_u_curhead) == 7768, "b_u_curhead offset mismatch");
_Static_assert(offsetof(buf_T, b_u_numhead) == 7776, "b_u_numhead offset mismatch");
_Static_assert(offsetof(buf_T, b_u_synced) == 7780, "b_u_synced offset mismatch");
_Static_assert(offsetof(buf_T, b_u_seq_last) == 7784, "b_u_seq_last offset mismatch");
_Static_assert(offsetof(buf_T, b_u_save_nr_last) == 7788, "b_u_save_nr_last offset mismatch");
_Static_assert(offsetof(buf_T, b_u_seq_cur) == 7792, "b_u_seq_cur offset mismatch");

// Scan flag
_Static_assert(offsetof(buf_T, b_scanned) == 7832, "b_scanned offset mismatch");

// Input mode / kmap
_Static_assert(offsetof(buf_T, b_p_iminsert) == 7840, "b_p_iminsert offset mismatch");
_Static_assert(offsetof(buf_T, b_p_imsearch) == 7848, "b_p_imsearch offset mismatch");
_Static_assert(offsetof(buf_T, b_kmap_state) == 7856, "b_kmap_state offset mismatch");
_Static_assert(offsetof(buf_T, b_p_initialized) == 7888, "b_p_initialized offset mismatch");

// Buffer options region
_Static_assert(offsetof(buf_T, b_p_ac) == 10080, "b_p_ac offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ai) == 10084, "b_p_ai offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ai_nopaste) == 10088, "b_p_ai_nopaste offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ci) == 10108, "b_p_ci offset mismatch");
_Static_assert(offsetof(buf_T, b_p_bin) == 10112, "b_p_bin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_bomb) == 10116, "b_p_bomb offset mismatch");
_Static_assert(offsetof(buf_T, b_p_bh) == 10120, "b_p_bh offset mismatch");
_Static_assert(offsetof(buf_T, b_p_bt) == 10128, "b_p_bt offset mismatch");
_Static_assert(offsetof(buf_T, b_p_busy) == 10136, "b_p_busy offset mismatch");
_Static_assert(offsetof(buf_T, b_p_bl) == 10148, "b_p_bl offset mismatch");
_Static_assert(offsetof(buf_T, b_p_channel) == 10152, "b_p_channel offset mismatch");
_Static_assert(offsetof(buf_T, b_p_cin) == 10160, "b_p_cin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_com) == 10200, "b_p_com offset mismatch");
_Static_assert(offsetof(buf_T, b_p_eof) == 10352, "b_p_eof offset mismatch");
_Static_assert(offsetof(buf_T, b_p_eol) == 10356, "b_p_eol offset mismatch");
_Static_assert(offsetof(buf_T, b_p_fixeol) == 10360, "b_p_fixeol offset mismatch");
_Static_assert(offsetof(buf_T, b_p_et) == 10364, "b_p_et offset mismatch");
_Static_assert(offsetof(buf_T, b_p_et_nobin) == 10368, "b_p_et_nobin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_et_nopaste) == 10372, "b_p_et_nopaste offset mismatch");
_Static_assert(offsetof(buf_T, b_p_fenc) == 10376, "b_p_fenc offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ff) == 10384, "b_p_ff offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ft) == 10392, "b_p_ft offset mismatch");
_Static_assert(offsetof(buf_T, b_p_inf) == 10416, "b_p_inf offset mismatch");
_Static_assert(offsetof(buf_T, b_p_isk) == 10424, "b_p_isk offset mismatch");
_Static_assert(offsetof(buf_T, b_p_inex) == 10448, "b_p_inex offset mismatch");
_Static_assert(offsetof(buf_T, b_p_inde) == 10464, "b_p_inde offset mismatch");
_Static_assert(offsetof(buf_T, b_p_fp) == 10488, "b_p_fp offset mismatch");
_Static_assert(offsetof(buf_T, b_p_fex) == 10496, "b_p_fex offset mismatch");
_Static_assert(offsetof(buf_T, b_p_kp) == 10512, "b_p_kp offset mismatch");
_Static_assert(offsetof(buf_T, b_p_lisp) == 10520, "b_p_lisp offset mismatch");
_Static_assert(offsetof(buf_T, b_p_mps) == 10544, "b_p_mps offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ml) == 10552, "b_p_ml offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ml_nobin) == 10556, "b_p_ml_nobin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ma) == 10560, "b_p_ma offset mismatch");
_Static_assert(offsetof(buf_T, b_p_nf) == 10568, "b_p_nf offset mismatch");
_Static_assert(offsetof(buf_T, b_p_pi) == 10576, "b_p_pi offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ro) == 10592, "b_p_ro offset mismatch");
_Static_assert(offsetof(buf_T, b_p_sw) == 10600, "b_p_sw offset mismatch");
_Static_assert(offsetof(buf_T, b_p_scbk) == 10608, "b_p_scbk offset mismatch");
_Static_assert(offsetof(buf_T, b_p_sts) == 10624, "b_p_sts offset mismatch");
_Static_assert(offsetof(buf_T, b_p_sts_nopaste) == 10632, "b_p_sts_nopaste offset mismatch");
_Static_assert(offsetof(buf_T, b_p_syn) == 10664, "b_p_syn offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ts) == 10672, "b_p_ts offset mismatch");
_Static_assert(offsetof(buf_T, b_p_tw) == 10680, "b_p_tw offset mismatch");
_Static_assert(offsetof(buf_T, b_p_tw_nobin) == 10688, "b_p_tw_nobin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_tw_nopaste) == 10696, "b_p_tw_nopaste offset mismatch");
_Static_assert(offsetof(buf_T, b_p_wm_nobin) == 10712, "b_p_wm_nobin offset mismatch");
_Static_assert(offsetof(buf_T, b_p_wm_nopaste) == 10720, "b_p_wm_nopaste offset mismatch");
_Static_assert(offsetof(buf_T, b_p_vsts_array) == 10736, "b_p_vsts_array offset mismatch");
_Static_assert(offsetof(buf_T, b_p_vts_array) == 10760, "b_p_vts_array offset mismatch");
_Static_assert(offsetof(buf_T, b_p_efm) == 10800, "b_p_efm offset mismatch");
_Static_assert(offsetof(buf_T, b_p_path) == 10816, "b_p_path offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ar) == 10824, "b_p_ar offset mismatch");
_Static_assert(offsetof(buf_T, b_p_ul) == 10904, "b_p_ul offset mismatch");
_Static_assert(offsetof(buf_T, b_p_udf) == 10912, "b_p_udf offset mismatch");

// Tail fields
_Static_assert(offsetof(buf_T, b_ind_level) == 10928, "b_ind_level offset mismatch");
_Static_assert(offsetof(buf_T, b_no_eol_lnum) == 11076, "b_no_eol_lnum offset mismatch");
_Static_assert(offsetof(buf_T, b_start_eof) == 11080, "b_start_eof offset mismatch");
_Static_assert(offsetof(buf_T, b_start_eol) == 11084, "b_start_eol offset mismatch");
_Static_assert(offsetof(buf_T, b_start_ffc) == 11088, "b_start_ffc offset mismatch");
_Static_assert(offsetof(buf_T, b_start_fenc) == 11096, "b_start_fenc offset mismatch");
_Static_assert(offsetof(buf_T, b_start_bomb) == 11108, "b_start_bomb offset mismatch");
_Static_assert(offsetof(buf_T, b_bufvar) == 11112, "b_bufvar offset mismatch");
_Static_assert(offsetof(buf_T, b_vars) == 11136, "b_vars offset mismatch");
_Static_assert(offsetof(buf_T, b_may_swap) == 11144, "b_may_swap offset mismatch");
_Static_assert(offsetof(buf_T, b_did_warn) == 11145, "b_did_warn offset mismatch");
_Static_assert(offsetof(buf_T, b_help) == 11146, "b_help offset mismatch");
_Static_assert(offsetof(buf_T, b_spell) == 11147, "b_spell offset mismatch");
_Static_assert(offsetof(buf_T, b_s) == 11240, "b_s offset mismatch");
_Static_assert(offsetof(buf_T, terminal) == 12456, "terminal offset mismatch");
_Static_assert(offsetof(buf_T, additional_data) == 12464, "additional_data offset mismatch");
_Static_assert(offsetof(buf_T, b_mapped_ctrl_c) == 12472, "b_mapped_ctrl_c offset mismatch");
_Static_assert(offsetof(buf_T, b_marktree) == 12480, "b_marktree offset mismatch");
_Static_assert(offsetof(buf_T, b_prev_line_count) == 12624, "b_prev_line_count offset mismatch");
_Static_assert(offsetof(buf_T, update_channels) == 12632, "update_channels offset mismatch");
_Static_assert(offsetof(buf_T, update_callbacks) == 12656, "update_callbacks offset mismatch");
_Static_assert(offsetof(buf_T, deleted_bytes) == 12688, "deleted_bytes offset mismatch");
_Static_assert(offsetof(buf_T, deleted_bytes2) == 12696, "deleted_bytes2 offset mismatch");
_Static_assert(offsetof(buf_T, deleted_codepoints) == 12704, "deleted_codepoints offset mismatch");
_Static_assert(offsetof(buf_T, deleted_codeunits) == 12712, "deleted_codeunits offset mismatch");
_Static_assert(offsetof(buf_T, flush_count) == 12720, "flush_count offset mismatch");

// memline_T subfield offsets (for ml_line_count/ml_mfp/ml_flags within b_ml)
_Static_assert(offsetof(memline_T, ml_line_count) == 0,
               "memline_T.ml_line_count offset mismatch");
_Static_assert(offsetof(memline_T, ml_mfp) == 8,
               "memline_T.ml_mfp offset mismatch");
_Static_assert(offsetof(memline_T, ml_flags) == 32,
               "memline_T.ml_flags offset mismatch");
_Static_assert(sizeof(memline_T) == 112, "memline_T size mismatch");

// visualinfo_T layout checks
_Static_assert(sizeof(visualinfo_T) == 32, "visualinfo_T size mismatch");
_Static_assert(offsetof(visualinfo_T, vi_start) == 0, "visualinfo_T.vi_start offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_end) == 12, "visualinfo_T.vi_end offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_mode) == 24, "visualinfo_T.vi_mode offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_curswant) == 28,
               "visualinfo_T.vi_curswant offset mismatch");

// FileID size check
_Static_assert(sizeof(FileID) == 16, "FileID size mismatch");
