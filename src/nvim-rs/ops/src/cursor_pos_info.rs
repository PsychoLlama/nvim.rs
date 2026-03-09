//! Cursor position info (`g CTRL-G` and `wordcount()`)
//!
//! Migrated from `cursor_pos_info()` in ops.c.

use std::ffi::{c_char, c_int, c_void};

/// Maximum column constant (matches C `MAXCOL = 0x7fffffff`)
const MAXCOL: c_int = 0x7fff_ffff;

/// Ctrl+V character code (matches C `Ctrl_V = 22`)
const CTRL_V: c_int = 22;

/// 'V' character (linewise visual)
const VISUAL_LINE: c_int = 0x56;

/// 'v' character (charwise visual)
const VISUAL_CHAR: c_int = 0x76;

/// Result from line_count_info call.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Default)]
struct CpiLineCountResult {
    byte_count: i64,
    word_count: i64,
    char_count: i64,
}

/// EOL format constant (matches C `EOL_DOS = 1`)
const EOL_DOS: c_int = 1;

extern "C" {
    // Buffer state (generic shims)
    fn nvim_curbuf_ml_empty() -> bool;
    fn nvim_curbuf_get_ml_line_count() -> c_int;
    fn nvim_curbuf_get_fileformat() -> c_int;

    // Visual state: individual shims replace nvim_cpi_get_visual_state
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_get_VIsual_lnum() -> c_int;
    fn nvim_get_VIsual_col() -> c_int;
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_p_sel_is_exclusive() -> bool;
    fn nvim_curwin_get_w_curswant() -> c_int;

    // Block visual
    fn nvim_cpi_setup_block_visual(
        min_lnum: c_int,
        min_col: c_int,
        max_lnum: c_int,
        max_col: c_int,
        out_start_vcol: *mut c_int,
        out_end_vcol: *mut c_int,
    );
    fn nvim_cpi_block_line_count(lnum: c_int, eol_size: c_int, out: *mut c_void);

    // Last line EOL adjustment (nvim_cpi_last_line_no_eol absorbed below)
    fn nvim_curbuf_get_b_p_eol() -> bool;
    fn nvim_curbuf_get_b_p_bin() -> c_int;
    fn nvim_curbuf_get_b_p_fixeol() -> bool;

    // Interrupt / breakcheck
    fn nvim_os_breakcheck();
    fn nvim_got_int() -> c_int;

    // Output / display
    fn nvim_msg_no_lines();
    fn nvim_cpi_format_visual_msg(
        line_count_selected: c_int,
        start_vcol: c_int,
        end_vcol: c_int,
        is_block_mode: c_int,
        curswant_is_max: c_int,
        word_count_cursor: i64,
        word_count: i64,
        char_count_cursor: i64,
        char_count: i64,
        byte_count_cursor: i64,
        byte_count: i64,
    );
    fn nvim_cpi_format_normal_msg(
        word_count_cursor: i64,
        word_count: i64,
        char_count_cursor: i64,
        char_count: i64,
        byte_count_cursor: i64,
        byte_count: i64,
    );
    fn nvim_cpi_append_bom_and_display(bom_count: i64);
    fn nvim_bomb_size() -> c_int;

    // dict operations (for nvim_cpi_populate_dict absorption)
    fn nvim_tag_tv_dict_add_nr(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        nr: i64,
    ) -> c_int;

    // Low-level buffer line access
    fn nvim_ml_get(lnum: c_int) -> *const c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

/// Inline replacement of `nvim_cpi_last_line_no_eol` (C function deleted).
/// Returns true if the last line has no trailing EOL byte.
///
/// # Safety
/// Reads from curbuf option fields via C shims.
unsafe fn last_line_no_eol() -> bool {
    !nvim_curbuf_get_b_p_eol() && (nvim_curbuf_get_b_p_bin() != 0 || !nvim_curbuf_get_b_p_fixeol())
}

/// Inline port of `nvim_cpi_populate_dict`.
/// Populates a wordcount dict_T with word/char/byte counts.
///
/// # Safety
/// `dict` must be a valid `dict_T *` or null.
#[allow(clippy::too_many_arguments)]
unsafe fn cpi_populate_dict(
    dict: *mut c_void,
    visual_active: c_int,
    word_count: i64,
    char_count: i64,
    byte_count: i64,
    bom_count: i64,
    word_count_cursor: i64,
    char_count_cursor: i64,
    byte_count_cursor: i64,
) {
    nvim_tag_tv_dict_add_nr(dict, c"words".as_ptr(), 5, word_count);
    nvim_tag_tv_dict_add_nr(dict, c"chars".as_ptr(), 5, char_count);
    nvim_tag_tv_dict_add_nr(dict, c"bytes".as_ptr(), 5, byte_count + bom_count);
    let bytes_key = if visual_active != 0 {
        c"visual_bytes"
    } else {
        c"cursor_bytes"
    };
    let chars_key = if visual_active != 0 {
        c"visual_chars"
    } else {
        c"cursor_chars"
    };
    let words_key = if visual_active != 0 {
        c"visual_words"
    } else {
        c"cursor_words"
    };
    nvim_tag_tv_dict_add_nr(dict, bytes_key.as_ptr(), 12, byte_count_cursor);
    nvim_tag_tv_dict_add_nr(dict, chars_key.as_ptr(), 12, char_count_cursor);
    nvim_tag_tv_dict_add_nr(dict, words_key.as_ptr(), 12, word_count_cursor);
}

/// `_Static_assert` constants are verified in the C accessor file.
#[inline]
fn lcr_void(lcr: &mut CpiLineCountResult) -> *mut c_void {
    std::ptr::from_mut(lcr).cast::<c_void>()
}

/// Rust port of `line_count_info()` from ops.c (C version deleted).
///
/// Count bytes, words, chars in a NUL-terminated C string up to `limit` bytes.
/// Returns byte count. `wc` and `cc` are incremented by word and char counts.
///
/// # Safety
/// `line` must point to a valid NUL-terminated C string.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn line_count_info(
    line: *const c_char,
    wc: &mut i64,
    cc: &mut i64,
    limit: i64,
    eol_size: c_int,
) -> i64 {
    // space (0x20) and tab (0x09) as c_char (i8)
    const SPACE: c_char = b' ' as i8;
    const TAB: c_char = b'\t' as i8;
    let mut i: i64 = 0;
    let mut words: i64 = 0;
    let mut chars: i64 = 0;
    let mut is_word = false;

    loop {
        if i >= limit {
            break;
        }
        let byte = unsafe { *line.add(i as usize) };
        if byte == 0 {
            break;
        }
        let is_space = byte == SPACE || byte == TAB;
        if is_word {
            if is_space {
                words += 1;
                is_word = false;
            }
        } else if !is_space {
            is_word = true;
        }
        chars += 1;
        let char_len = unsafe { utfc_ptr2len(line.add(i as usize)) };
        i += i64::from(char_len);
    }

    if is_word {
        words += 1;
    }
    *wc += words;

    // Add eol_size if end-of-line reached before limit
    if i < limit && unsafe { *line.add(i as usize) } == 0 {
        i += i64::from(eol_size);
        chars += i64::from(eol_size);
    }
    *cc += chars;
    i
}

/// C-callable export of `line_count_info` for use by `nvim_cpi_block_line_count` in ops.c.
///
/// This replaces the static C `line_count_info` which has been deleted from ops.c.
///
/// # Safety
/// `line` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn nvim_rs_line_count_info(
    line: *const c_char,
    wc: &mut i64,
    cc: &mut i64,
    limit: i64,
    eol_size: c_int,
) -> i64 {
    line_count_info(line, wc, cc, limit, eol_size)
}

/// Count bytes/words/chars on a buffer line (Rust replacement for nvim_cpi_line_count_info).
///
/// # Safety
/// Calls `nvim_ml_get` to get a valid line pointer.
unsafe fn cpi_line_count_info(
    lnum: c_int,
    col_limit: c_int,
    eol_size: c_int,
) -> CpiLineCountResult {
    let line = nvim_ml_get(lnum);
    let mut wc: i64 = 0;
    let mut cc: i64 = 0;
    let bc = line_count_info(line, &mut wc, &mut cc, i64::from(col_limit), eol_size);
    CpiLineCountResult {
        byte_count: bc,
        word_count: wc,
        char_count: cc,
    }
}

/// Count bytes/words/chars on a buffer line starting at start_col (Rust replacement for nvim_cpi_line_count_info_at).
///
/// # Safety
/// Calls `nvim_ml_get` to get a valid line pointer.
#[allow(clippy::cast_sign_loss)]
unsafe fn cpi_line_count_info_at(
    lnum: c_int,
    start_col: c_int,
    len: c_int,
    eol_size: c_int,
) -> CpiLineCountResult {
    let line = nvim_ml_get(lnum).add(start_col as usize);
    let mut wc: i64 = 0;
    let mut cc: i64 = 0;
    let bc = line_count_info(line, &mut wc, &mut cc, i64::from(len), eol_size);
    CpiLineCountResult {
        byte_count: bc,
        word_count: wc,
        char_count: cc,
    }
}

/// Accumulated word/char/byte counts.
struct Counts {
    byte_count: i64,
    char_count: i64,
    word_count: i64,
    byte_count_cursor: i64,
    char_count_cursor: i64,
    word_count_cursor: i64,
}

/// Parameters for the counting loop.
struct CountParams {
    eol_size: c_int,
    line_count: c_int,
    visual_active: bool,
    visual_mode: c_int,
    min_lnum: c_int,
    min_col: c_int,
    max_lnum: c_int,
    max_col: c_int,
    cursor_lnum: c_int,
    cursor_col: c_int,
}

/// Count a visual-mode selected region on a single line.
///
/// # Safety
/// Calls C accessor functions.
#[allow(clippy::similar_names)]
unsafe fn count_visual_line(p: &CountParams, lnum: c_int) -> CpiLineCountResult {
    let mut lcr = CpiLineCountResult::default();
    match p.visual_mode {
        v if v == CTRL_V => {
            nvim_cpi_block_line_count(lnum, p.eol_size, lcr_void(&mut lcr));
        }
        VISUAL_LINE => {
            lcr = cpi_line_count_info(lnum, MAXCOL, p.eol_size);
        }
        VISUAL_CHAR => {
            let col_start = if lnum == p.min_lnum { p.min_col } else { 0 };
            let col_end = if lnum == p.max_lnum {
                p.max_col - col_start + 1
            } else {
                MAXCOL
            };
            lcr = cpi_line_count_info_at(lnum, col_start, col_end, p.eol_size);
        }
        _ => {}
    }
    lcr
}

/// Run the main counting loop over all buffer lines.
///
/// # Safety
/// Calls C accessor functions that access global buffer state.
unsafe fn count_lines(p: &CountParams) -> Option<Counts> {
    let mut c = Counts {
        byte_count: 0,
        char_count: 0,
        word_count: 0,
        byte_count_cursor: 0,
        char_count_cursor: 0,
        word_count_cursor: 0,
    };
    let mut last_check: i64 = 100_000;

    for lnum in 1..=p.line_count {
        if c.byte_count > last_check {
            nvim_os_breakcheck();
            if nvim_got_int() != 0 {
                return None;
            }
            last_check = c.byte_count + 100_000;
        }

        if p.visual_active && lnum >= p.min_lnum && lnum <= p.max_lnum {
            let lcr = count_visual_line(p, lnum);
            c.byte_count_cursor += lcr.byte_count;
            c.word_count_cursor += lcr.word_count;
            c.char_count_cursor += lcr.char_count;

            if lnum == p.line_count && last_line_no_eol() {
                c.byte_count_cursor -= i64::from(p.eol_size);
            }
        } else if !p.visual_active && lnum == p.cursor_lnum {
            c.word_count_cursor += c.word_count;
            c.char_count_cursor += c.char_count;
            let lcr = cpi_line_count_info(lnum, p.cursor_col + 1, p.eol_size);
            c.byte_count_cursor = c.byte_count + lcr.byte_count;
            c.word_count_cursor += lcr.word_count;
            c.char_count_cursor += lcr.char_count;
        }

        let lcr = cpi_line_count_info(lnum, MAXCOL, p.eol_size);
        c.byte_count += lcr.byte_count;
        c.word_count += lcr.word_count;
        c.char_count += lcr.char_count;
    }

    if last_line_no_eol() {
        c.byte_count -= i64::from(p.eol_size);
    }

    Some(c)
}

/// Visual display parameters for output.
struct VisualDisplayParams {
    visual_active: bool,
    visual_mode: c_int,
    curswant: c_int,
    line_count_selected: c_int,
    blk_start_vcol: c_int,
    blk_end_vcol: c_int,
}

/// Display or store the counted results.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn output_results(dict: *mut c_void, vp: &VisualDisplayParams, c: &Counts) {
    let bom_count = i64::from(nvim_bomb_size());

    if dict.is_null() {
        if vp.visual_active {
            nvim_cpi_format_visual_msg(
                vp.line_count_selected,
                vp.blk_start_vcol,
                vp.blk_end_vcol,
                c_int::from(vp.visual_mode == CTRL_V),
                c_int::from(vp.curswant == MAXCOL),
                c.word_count_cursor,
                c.word_count,
                c.char_count_cursor,
                c.char_count,
                c.byte_count_cursor,
                c.byte_count,
            );
        } else {
            nvim_cpi_format_normal_msg(
                c.word_count_cursor,
                c.word_count,
                c.char_count_cursor,
                c.char_count,
                c.byte_count_cursor,
                c.byte_count,
            );
        }
        nvim_cpi_append_bom_and_display(bom_count);
    } else {
        cpi_populate_dict(
            dict,
            c_int::from(vp.visual_active),
            c.word_count,
            c.char_count,
            c.byte_count,
            bom_count,
            c.word_count_cursor,
            c.char_count_cursor,
            c.byte_count_cursor,
        );
    }
}

/// Give info about the position of the cursor (for "g CTRL-G").
/// In Visual mode, give info about the selected region.
///
/// When `dict` is not NULL, store the info there instead of displaying it.
///
/// # Safety
///
/// - `dict` may be null (display mode) or a valid `dict_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "cursor_pos_info")]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_cursor_pos_info(dict: *mut c_void) {
    // Check for empty buffer
    if nvim_curbuf_ml_empty() {
        if dict.is_null() {
            nvim_msg_no_lines();
        } else {
            cpi_populate_dict(dict, 0, 0, 0, 0, 0, 0, 0, 0);
        }
        return;
    }

    let eol_size: c_int = if nvim_curbuf_get_fileformat() == EOL_DOS {
        2
    } else {
        1
    };
    let line_count = nvim_curbuf_get_ml_line_count();

    // Get visual state via individual shims (nvim_cpi_get_visual_state absorbed)
    let visual_active_int = nvim_get_VIsual_active();
    let visual_mode = nvim_get_VIsual_mode();
    let visual_lnum = nvim_get_VIsual_lnum();
    let visual_col = nvim_get_VIsual_col();
    let cursor_lnum = nvim_get_cursor_lnum();
    let cursor_col = nvim_get_cursor_col();
    let sel_exclusive = c_int::from(nvim_p_sel_is_exclusive());
    let curswant = nvim_curwin_get_w_curswant();

    let visual_active = visual_active_int != 0;

    // Set up visual mode positions
    let (min_lnum, min_col, max_lnum, max_col, line_count_selected) = if visual_active {
        let (min_l, min_c, max_l, max_c) = if visual_lnum < cursor_lnum
            || (visual_lnum == cursor_lnum && visual_col <= cursor_col)
        {
            (visual_lnum, visual_col, cursor_lnum, cursor_col)
        } else {
            (cursor_lnum, cursor_col, visual_lnum, visual_col)
        };

        let max_c = if sel_exclusive != 0 && max_c > 0 {
            max_c - 1
        } else {
            max_c
        };

        (min_l, min_c, max_l, max_c, max_l - min_l + 1)
    } else {
        (0, 0, 0, 0, 0)
    };

    // Set up block visual (get vcols)
    let mut blk_start_vcol: c_int = 0;
    let mut blk_end_vcol: c_int = 0;
    if visual_active && visual_mode == CTRL_V {
        nvim_cpi_setup_block_visual(
            min_lnum,
            min_col,
            max_lnum,
            max_col,
            &raw mut blk_start_vcol,
            &raw mut blk_end_vcol,
        );
        if curswant == MAXCOL {
            blk_end_vcol = MAXCOL;
        }
        // Swap if needed
        if blk_end_vcol < blk_start_vcol {
            blk_end_vcol += blk_start_vcol;
            blk_start_vcol = blk_end_vcol - blk_start_vcol;
            blk_end_vcol -= blk_start_vcol;
        }
    }

    // Main counting loop
    let params = CountParams {
        eol_size,
        line_count,
        visual_active,
        visual_mode,
        min_lnum,
        min_col,
        max_lnum,
        max_col,
        cursor_lnum,
        cursor_col,
    };
    let Some(c) = count_lines(&params) else {
        return; // interrupted
    };

    let vp = VisualDisplayParams {
        visual_active,
        visual_mode,
        curswant,
        line_count_selected,
        blk_start_vcol,
        blk_end_vcol,
    };
    output_results(dict, &vp, &c);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAXCOL, 0x7fff_ffff);
        assert_eq!(CTRL_V, 22);
        assert_eq!(VISUAL_LINE, 0x56);
        assert_eq!(VISUAL_CHAR, 0x76);
    }

    #[test]
    fn test_line_count_result_default() {
        let lcr = CpiLineCountResult::default();
        assert_eq!(lcr.byte_count, 0);
        assert_eq!(lcr.word_count, 0);
        assert_eq!(lcr.char_count, 0);
    }

    #[test]
    fn test_position_ordering_logic() {
        // When visual < cursor
        let (vis_l, vis_c) = (1, 5);
        let (cur_l, cur_c) = (3, 10);
        let (min_l, _min_c, max_l, _max_c) = if vis_l < cur_l || (vis_l == cur_l && vis_c <= cur_c)
        {
            (vis_l, vis_c, cur_l, cur_c)
        } else {
            (cur_l, cur_c, vis_l, vis_c)
        };
        assert_eq!(min_l, 1);
        assert_eq!(max_l, 3);

        // When cursor < visual
        let (vis_l, vis_c) = (5, 10);
        let (cur_l, cur_c) = (2, 3);
        let (min_l, _min_c, max_l, _max_c) = if vis_l < cur_l || (vis_l == cur_l && vis_c <= cur_c)
        {
            (vis_l, vis_c, cur_l, cur_c)
        } else {
            (cur_l, cur_c, vis_l, vis_c)
        };
        assert_eq!(min_l, 2);
        assert_eq!(max_l, 5);

        // Same line, visual before cursor
        let (vis_l, vis_c) = (3, 2);
        let (cur_l, cur_c) = (3, 8);
        let (min_l, min_c, max_l, max_c) = if vis_l < cur_l || (vis_l == cur_l && vis_c <= cur_c) {
            (vis_l, vis_c, cur_l, cur_c)
        } else {
            (cur_l, cur_c, vis_l, vis_c)
        };
        assert_eq!(min_l, 3);
        assert_eq!(min_c, 2);
        assert_eq!(max_l, 3);
        assert_eq!(max_c, 8);
    }

    #[test]
    fn test_vcol_swap_logic() {
        let mut sv: c_int = 10;
        let mut ev: c_int = 5;
        if ev < sv {
            ev += sv;
            sv = ev - sv;
            ev -= sv;
        }
        assert_eq!(sv, 5);
        assert_eq!(ev, 10);

        // No swap needed
        let mut sv: c_int = 3;
        let mut ev: c_int = 8;
        if ev < sv {
            ev += sv;
            sv = ev - sv;
            ev -= sv;
        }
        assert_eq!(sv, 3);
        assert_eq!(ev, 8);
    }

    #[test]
    fn test_exclusive_selection_adjust() {
        let max_col = 5;
        let sel_exclusive = true;
        let adjusted = if sel_exclusive && max_col > 0 {
            max_col - 1
        } else {
            max_col
        };
        assert_eq!(adjusted, 4);

        let max_col = 0;
        let adjusted = if sel_exclusive && max_col > 0 {
            max_col - 1
        } else {
            max_col
        };
        assert_eq!(adjusted, 0);
    }
}
