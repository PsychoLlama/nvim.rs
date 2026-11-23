//! `Buf`'s field getters and setters: one per `Copy` field, each checked
//! against the buffer's slot as `slot_fields!` says.
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_uint};

use crate::buffer::BufFlags;
use crate::os::libc::{int16_t, int64_t, size_t, time_t, uint64_t};
use crate::slot;
use crate::types::*;
use crate::undo::node::{UndoLink, VisualInfo};
use crate::winlayer::Term;
use crate::winlayer::{Buf, BufId};

slot_fields! {
    pub Buf => Buffer {
        b_next, set_b_next: Option<BufId>;
        b_prev, set_b_prev: Option<BufId>;
        b_nwindows, set_b_nwindows: c_int;
        b_flags, set_b_flags: BufFlags;
        b_locked, set_b_locked: c_int;
        b_locked_split, set_b_locked_split: c_int;
        b_ro_locked, set_b_ro_locked: c_int;
        file_id_valid, set_file_id_valid: bool;
        b_changed, set_b_changed: c_int;
        b_changed_invalid, set_b_changed_invalid: bool;
        b_changedtick, set_b_changedtick: VarNumber;
        b_last_changedtick, set_b_last_changedtick: VarNumber;
        b_last_changedtick_i, set_b_last_changedtick_i: VarNumber;
        b_last_changedtick_pum, set_b_last_changedtick_pum: VarNumber;
        b_saving, set_b_saving: bool;
        b_mod_set, set_b_mod_set: bool;
        b_mod_top, set_b_mod_top: LineNr;
        b_mod_bot, set_b_mod_bot: LineNr;
        b_mod_xlines, set_b_mod_xlines: i32;
        b_mod_tick_syn, set_b_mod_tick_syn: DispTick;
        b_mod_tick_decor, set_b_mod_tick_decor: DispTick;
        b_mtime, set_b_mtime: int64_t;
        b_mtime_ns, set_b_mtime_ns: int64_t;
        b_mtime_read, set_b_mtime_read: int64_t;
        b_mtime_read_ns, set_b_mtime_read_ns: int64_t;
        b_orig_size, set_b_orig_size: uint64_t;
        b_orig_mode, set_b_orig_mode: c_int;
        b_last_used, set_b_last_used: time_t;
        b_visual: VisualInfo;
        b_visual_mode_eval, set_b_visual_mode_eval: c_int;
        b_changelistlen, set_b_changelistlen: c_int;
        b_new_change, set_b_new_change: bool;
        b_chartab, set_b_chartab: [uint64_t; 4];
        b_op_start, set_b_op_start: Pos;
        b_op_start_orig, set_b_op_start_orig: Pos;
        b_op_end, set_b_op_end: Pos;
        b_marks_read, set_b_marks_read: bool;
        b_modified_was_set, set_b_modified_was_set: bool;
        b_did_filetype, set_b_did_filetype: bool;
        b_keep_filetype, set_b_keep_filetype: bool;
        b_au_did_filetype, set_b_au_did_filetype: bool;
        b_u_oldhead, set_b_u_oldhead: UndoLink;
        b_u_newhead, set_b_u_newhead: UndoLink;
        b_u_curhead, set_b_u_curhead: UndoLink;
        b_u_numhead, set_b_u_numhead: c_int;
        b_u_synced, set_b_u_synced: bool;
        b_u_seq_last, set_b_u_seq_last: c_int;
        b_u_save_nr_last, set_b_u_save_nr_last: c_int;
        b_u_seq_cur, set_b_u_seq_cur: c_int;
        b_u_time_cur, set_b_u_time_cur: time_t;
        b_u_save_nr_cur, set_b_u_save_nr_cur: c_int;
        b_u_line_lnum, set_b_u_line_lnum: LineNr;
        b_u_line_colnr, set_b_u_line_colnr: ColNr;
        b_scanned, set_b_scanned: bool;
        b_p_iminsert, set_b_p_iminsert: OptInt;
        b_p_imsearch, set_b_p_imsearch: OptInt;
        b_kmap_state, set_b_kmap_state: int16_t;
        b_p_initialized, set_b_p_initialized: bool;
        b_p_ac, set_b_p_ac: c_int;
        b_p_ai, set_b_p_ai: c_int;
        b_p_ai_nopaste, set_b_p_ai_nopaste: c_int;
        b_bkc_flags, set_b_bkc_flags: c_uint;
        b_p_ci, set_b_p_ci: c_int;
        b_p_bin, set_b_p_bin: c_int;
        b_p_bomb, set_b_p_bomb: c_int;
        b_has_qf_entry, set_b_has_qf_entry: c_int;
        b_p_bl, set_b_p_bl: c_int;
        b_p_channel, set_b_p_channel: OptInt;
        b_p_cin, set_b_p_cin: c_int;
        b_cot_flags, set_b_cot_flags: c_uint;
        b_p_eof, set_b_p_eof: c_int;
        b_p_eol, set_b_p_eol: c_int;
        b_p_fixeol, set_b_p_fixeol: c_int;
        b_p_et, set_b_p_et: c_int;
        b_p_et_nobin, set_b_p_et_nobin: c_int;
        b_p_et_nopaste, set_b_p_et_nopaste: c_int;
        b_p_inf, set_b_p_inf: c_int;
        b_p_fs, set_b_p_fs: c_int;
        b_p_lisp, set_b_p_lisp: c_int;
        b_p_ml, set_b_p_ml: c_int;
        b_p_ml_nobin, set_b_p_ml_nobin: c_int;
        b_p_ma, set_b_p_ma: c_int;
        b_p_pi, set_b_p_pi: c_int;
        b_p_ro, set_b_p_ro: c_int;
        b_p_sw, set_b_p_sw: OptInt;
        b_p_scbk, set_b_p_scbk: OptInt;
        b_p_si, set_b_p_si: c_int;
        b_p_sts, set_b_p_sts: OptInt;
        b_p_sts_nopaste, set_b_p_sts_nopaste: OptInt;
        b_p_swf, set_b_p_swf: c_int;
        b_p_smc, set_b_p_smc: OptInt;
        b_p_ts, set_b_p_ts: OptInt;
        b_p_tw, set_b_p_tw: OptInt;
        b_p_tw_nobin, set_b_p_tw_nobin: OptInt;
        b_p_tw_nopaste, set_b_p_tw_nopaste: OptInt;
        b_p_wm, set_b_p_wm: OptInt;
        b_p_wm_nobin, set_b_p_wm_nobin: OptInt;
        b_p_wm_nopaste, set_b_p_wm_nopaste: OptInt;
        b_p_ar, set_b_p_ar: c_int;
        b_tc_flags, set_b_tc_flags: c_uint;
        b_p_ul, set_b_p_ul: OptInt;
        b_p_udf, set_b_p_udf: c_int;
        b_ind_level, set_b_ind_level: c_int;
        b_ind_open_imag, set_b_ind_open_imag: c_int;
        b_ind_no_brace, set_b_ind_no_brace: c_int;
        b_ind_first_open, set_b_ind_first_open: c_int;
        b_ind_open_extra, set_b_ind_open_extra: c_int;
        b_ind_close_extra, set_b_ind_close_extra: c_int;
        b_ind_open_left_imag, set_b_ind_open_left_imag: c_int;
        b_ind_jump_label, set_b_ind_jump_label: c_int;
        b_ind_case, set_b_ind_case: c_int;
        b_ind_case_code, set_b_ind_case_code: c_int;
        b_ind_case_break, set_b_ind_case_break: c_int;
        b_ind_param, set_b_ind_param: c_int;
        b_ind_func_type, set_b_ind_func_type: c_int;
        b_ind_comment, set_b_ind_comment: c_int;
        b_ind_in_comment, set_b_ind_in_comment: c_int;
        b_ind_in_comment2, set_b_ind_in_comment2: c_int;
        b_ind_cpp_baseclass, set_b_ind_cpp_baseclass: c_int;
        b_ind_continuation, set_b_ind_continuation: c_int;
        b_ind_unclosed, set_b_ind_unclosed: c_int;
        b_ind_unclosed2, set_b_ind_unclosed2: c_int;
        b_ind_unclosed_noignore, set_b_ind_unclosed_noignore: c_int;
        b_ind_unclosed_wrapped, set_b_ind_unclosed_wrapped: c_int;
        b_ind_unclosed_whiteok, set_b_ind_unclosed_whiteok: c_int;
        b_ind_matching_paren, set_b_ind_matching_paren: c_int;
        b_ind_paren_prev, set_b_ind_paren_prev: c_int;
        b_ind_maxparen, set_b_ind_maxparen: c_int;
        b_ind_maxcomment, set_b_ind_maxcomment: c_int;
        b_ind_scopedecl, set_b_ind_scopedecl: c_int;
        b_ind_scopedecl_code, set_b_ind_scopedecl_code: c_int;
        b_ind_java, set_b_ind_java: c_int;
        b_ind_js, set_b_ind_js: c_int;
        b_ind_keep_case_label, set_b_ind_keep_case_label: c_int;
        b_ind_hash_comment, set_b_ind_hash_comment: c_int;
        b_ind_cpp_namespace, set_b_ind_cpp_namespace: c_int;
        b_ind_if_for_while, set_b_ind_if_for_while: c_int;
        b_ind_cpp_extern_c, set_b_ind_cpp_extern_c: c_int;
        b_ind_pragma, set_b_ind_pragma: c_int;
        b_no_eol_lnum, set_b_no_eol_lnum: LineNr;
        b_start_eof, set_b_start_eof: c_int;
        b_start_eol, set_b_start_eol: c_int;
        b_start_ffc, set_b_start_ffc: c_int;
        b_bad_char, set_b_bad_char: c_int;
        b_start_bomb, set_b_start_bomb: c_int;
        b_may_swap, set_b_may_swap: bool;
        b_did_warn, set_b_did_warn: bool;
        b_help, set_b_help: bool;
        b_spell, set_b_spell: bool;
        b_prompt_append_new_line, set_b_prompt_append_new_line: bool;
        b_prompt_insert, set_b_prompt_insert: Key;
        terminal, set_terminal: Option<Term>;
        b_mapped_ctrl_c: c_int;
        b_prev_line_count, set_b_prev_line_count: c_int;
        update_need_codepoints, set_update_need_codepoints: bool;
        deleted_bytes, set_deleted_bytes: size_t;
        deleted_bytes2, set_deleted_bytes2: size_t;
        deleted_codepoints, set_deleted_codepoints: size_t;
        deleted_codeunits, set_deleted_codeunits: size_t;
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;

    /// A getter and a setter reach the field in place; under a `read()` the
    /// setter refuses and the getter answers; once freed, both refuse.
    #[test]
    fn a_field_is_read_and_written_in_place_and_checked() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        buffer.set_b_changed(1);
        assert_eq!(buffer.b_changed(), 1);
        let read = buffer.read();
        assert!(catch_unwind(AssertUnwindSafe(|| buffer.set_b_changed(0))).is_err());
        assert_eq!(buffer.b_changed(), 1);
        drop(read);
        buffer.set_b_changed(0);
        free_bare_buffer(buffer);
        assert!(catch_unwind(AssertUnwindSafe(|| buffer.b_changed())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| buffer.set_b_changed(1))).is_err());
    }
}
