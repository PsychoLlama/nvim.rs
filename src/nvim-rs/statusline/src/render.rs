//! Statusline rendering - main format string evaluation loop
//!
//! This module provides the main `build_stl_str` function that evaluates
//! statusline format strings and produces the rendered output.

use std::ffi::{c_char, c_int, CStr};

use nvim_window::WinHandle;

use crate::builder::StatuslineBuilder;
use crate::click::ClickTracker;
use crate::eval::calc_percentage;
use crate::format::{FormatParser, FormatSpec, StlFlag};
use crate::highlight::HighlightTracker;

// C callbacks for expression evaluation and other operations that need C-side support
extern "C" {
    fn xfree(ptr: *mut std::ffi::c_void);
    // eval_to_string: evaluates a VimL expression, returns allocated string
    fn eval_to_string(arg: *mut c_char, join_list: bool, use_simple_function: bool) -> *mut c_char;
    #[link_name = "xmemdupz"]
    fn render_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    // Buffer/Window accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> nvim_window::BufHandle;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;

    // Buffer accessors
    fn nvim_buf_get_b_fname(buf: nvim_window::BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: nvim_window::BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ro(buf: nvim_window::BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ft(buf: nvim_window::BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ma(buf: nvim_window::BufHandle) -> c_int;
    fn nvim_buf_get_b_changed(buf: nvim_window::BufHandle) -> bool;
    fn nvim_buf_get_help(buf: nvim_window::BufHandle) -> c_int;
    fn nvim_buf_get_fnum(buf: nvim_window::BufHandle) -> c_int;
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;

    // Highlight lookup
    fn nvim_syn_name2id(name: *const c_char) -> c_int;

    // Quickfix / keymap: direct underlying functions
    fn nvim_win_is_qf_win(wp: WinHandle) -> bool;
    fn nvim_win_get_llist_ref(wp: WinHandle) -> *mut std::ffi::c_void;
    fn nvim_stl_get_msg_loclist() -> *const c_char;
    fn nvim_stl_get_msg_qflist() -> *const c_char;
    fn get_keymap_str(wp: WinHandle, fmt: *const c_char, buf: *mut c_char, len: c_int) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    static showcmd_buf: [u8; 41];

    // Batch cursor info
    fn nvim_stl_get_win_cursor_info(wp: WinHandle) -> crate::stl_build::StlCursorInfo;

}

/// Rendering context for the statusline.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct RenderContext {
    /// Window handle
    pub wp: WinHandle,
    /// Maximum width for the output
    pub max_width: c_int,
    /// Fill character for separators
    pub fill_char: char,
    /// Whether to use StrWidth (multibyte) or simple byte counting
    pub use_strwidth: bool,
    /// Whether we're rendering for tabline
    pub is_tabline: bool,
    /// Whether we're rendering for statuscolumn
    pub is_statuscol: bool,
    /// Current group depth
    group_depth: c_int,
    /// Previous char was a flag item
    prevchar_isflag: bool,
    /// Previous char was an item
    prevchar_isitem: bool,
    /// Current highlight value
    cur_userhl: c_int,
}

impl RenderContext {
    /// Create a new render context.
    pub const fn new(wp: WinHandle, max_width: c_int, fill_char: char) -> Self {
        Self {
            wp,
            max_width,
            fill_char,
            use_strwidth: true,
            is_tabline: false,
            is_statuscol: false,
            group_depth: 0,
            prevchar_isflag: true,
            prevchar_isitem: false,
            cur_userhl: 0,
        }
    }
}

/// Result of building a statusline.
#[derive(Debug)]
pub struct BuildResult {
    /// The rendered output string
    pub output: Vec<u8>,
    /// Highlight records
    pub highlights: Vec<crate::highlight::StlHighlightRecord>,
    /// Click records (for tabline)
    pub clicks: Vec<(usize, crate::click::ClickType, c_int, Option<String>)>,
    /// Number of separator positions
    pub separator_count: usize,
    /// Truncation was applied
    pub truncated: bool,
}

/// Build statusline string from format.
///
/// This is the main rendering function that processes a format string
/// and produces the rendered statusline output.
#[allow(clippy::cast_sign_loss)]
pub fn build_stl_str(fmt: &str, ctx: &mut RenderContext) -> BuildResult {
    let mut builder = StatuslineBuilder::new(ctx.max_width.max(0) as usize);
    builder.set_fill_char(ctx.fill_char);

    let mut highlights = HighlightTracker::new();
    let mut clicks = ClickTracker::new();
    let mut parser = FormatParser::new(fmt);

    // Group stack for tracking nested groups
    let mut group_stack: Vec<GroupInfo> = Vec::new();

    // Process the format string
    while !parser.is_empty() {
        // First, copy any literal text
        let literal_start = parser.position();
        let literal_len = parser.skip_literal();
        if literal_len > 0 {
            let literal = &fmt[literal_start..literal_start + literal_len];
            builder.append_literal(literal);
            ctx.prevchar_isflag = false;
            ctx.prevchar_isitem = false;
        }

        // Now parse a format specifier
        if parser.is_empty() {
            break;
        }

        if let Some(spec) = parser.parse_spec() {
            process_spec(
                &spec,
                ctx,
                &mut builder,
                &mut highlights,
                &mut clicks,
                &mut group_stack,
            );
        }
    }

    // Close any unclosed groups
    while !group_stack.is_empty() {
        builder.end_group();
        group_stack.pop();
    }

    // Finalize with separator filling
    builder.finalize(ctx.max_width.max(0) as usize);

    BuildResult {
        output: builder.output().to_vec(),
        highlights: highlights.into_records(),
        clicks: clicks
            .iter()
            .map(|(s, t, n, f)| (s, t, n, f.map(String::from)))
            .collect(),
        separator_count: 0, // Would track from builder
        truncated: false,
    }
}

/// Info for tracking groups during rendering.
#[allow(dead_code)]
struct GroupInfo {
    /// Start position in output
    start_pos: usize,
    /// Minimum width for group
    minwid: c_int,
    /// Maximum width for group
    maxwid: c_int,
}

/// Process a single format specifier.
fn process_spec(
    spec: &FormatSpec,
    ctx: &mut RenderContext,
    builder: &mut StatuslineBuilder,
    highlights: &mut HighlightTracker,
    clicks: &mut ClickTracker,
    group_stack: &mut Vec<GroupInfo>,
) {
    match spec {
        FormatSpec::Percent => {
            builder.append_byte(b'%');
            ctx.prevchar_isflag = false;
            ctx.prevchar_isitem = false;
        }

        FormatSpec::Separator => {
            if ctx.group_depth == 0 {
                builder.add_separator();
            }
        }

        FormatSpec::TruncMark => {
            builder.add_truncation_marker();
        }

        FormatSpec::GroupStart { minwid, maxwid } => {
            let start_pos = builder.position();
            builder.start_group(*minwid, *maxwid);
            group_stack.push(GroupInfo {
                start_pos,
                minwid: *minwid,
                maxwid: *maxwid,
            });
            ctx.group_depth += 1;
        }

        FormatSpec::GroupEnd => {
            if ctx.group_depth > 0 {
                builder.end_group();
                group_stack.pop();
                ctx.group_depth -= 1;
            }
        }

        FormatSpec::UserHighlight(hl) => {
            let userhl = c_int::from(*hl);
            builder.set_highlight(userhl);
            highlights.set_user_hl(builder.position(), *hl);
            ctx.cur_userhl = userhl;
        }

        FormatSpec::NamedHighlight(name) => {
            // Look up the highlight group
            let syn_id = unsafe {
                let c_name = std::ffi::CString::new(name.as_str()).unwrap_or_default();
                nvim_syn_name2id(c_name.as_ptr())
            };
            if syn_id > 0 {
                builder.set_named_highlight(syn_id);
                highlights.set_named_hl(builder.position(), syn_id);
                ctx.cur_userhl = -syn_id;
            }
        }

        FormatSpec::TabPageNr(tabnr) => {
            builder.add_tab_page(*tabnr);
            match (*tabnr).cmp(&0) {
                std::cmp::Ordering::Greater => {
                    clicks.start_tab_switch(builder.position(), *tabnr);
                }
                std::cmp::Ordering::Less => {
                    clicks.start_tab_close(builder.position(), -*tabnr);
                }
                std::cmp::Ordering::Equal => {
                    clicks.end_region(builder.position());
                }
            }
        }

        FormatSpec::TabCloseNr(tabnr) => {
            builder.add_tab_page(-*tabnr);
            clicks.start_tab_close(builder.position(), *tabnr);
        }

        FormatSpec::ClickFunc(func) => {
            builder.add_click_func(func, 0);
            clicks.start_func_run(builder.position(), func, 0);
        }

        FormatSpec::Expression { expr, reevaluate } => {
            render_expression(expr, *reevaluate, ctx, builder);
        }

        FormatSpec::ExprEnd => {
            // End of expression block - nothing to do here
        }

        FormatSpec::Item {
            flag,
            minwid,
            maxwid,
            zeropad,
            left_align,
        } => {
            render_item(
                *flag,
                *minwid,
                *maxwid,
                *zeropad,
                *left_align,
                ctx,
                builder,
                highlights,
            );
        }
    }
}

/// Render an expression.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn render_expression(
    expr: &str,
    _reevaluate: bool,
    _ctx: &RenderContext,
    builder: &mut StatuslineBuilder,
) {
    unsafe {
        // Build a null-terminated copy of expr, evaluate it, free the copy.
        let expr_copy = render_xmemdupz(expr.as_ptr().cast(), expr.len());
        if expr_copy.is_null() {
            return;
        }
        let result_ptr = eval_to_string(expr_copy, true, false);
        xfree(expr_copy.cast());

        if !result_ptr.is_null() {
            let cstr = std::ffi::CStr::from_ptr(result_ptr);
            if let Ok(s) = cstr.to_str() {
                if !s.is_empty() {
                    builder.append_literal(s);
                }
            }
            xfree(result_ptr.cast());
        }
    }
}

/// Render a standard item.
#[allow(clippy::too_many_arguments)]
fn render_item(
    flag: StlFlag,
    minwid: c_int,
    maxwid: c_int,
    zeropad: bool,
    left_align: bool,
    ctx: &mut RenderContext,
    builder: &mut StatuslineBuilder,
    highlights: &mut HighlightTracker,
) {
    let mut tmp_buf = [0u8; 256];

    // Get the value for this item
    let (content, is_numeric, is_flag) = eval_item(flag, ctx, &mut tmp_buf);

    if content.is_empty() && is_flag {
        // Empty flag item - nothing to output
        ctx.prevchar_isflag = true;
        return;
    }

    // Check if we should skip leading character
    if !content.is_empty() {
        let first_char = content.as_bytes()[0];
        let should_skip = (first_char == b',' && !ctx.prevchar_isitem)
            || (first_char == b' ' && ctx.prevchar_isflag);
        if should_skip {
            let content = &content[1..];
            if content.is_empty() {
                ctx.prevchar_isflag = is_flag;
                return;
            }
            // Append the trimmed content
            append_with_width(
                content, minwid, maxwid, zeropad, left_align, is_numeric, ctx, builder,
            );
        } else {
            append_with_width(
                &content, minwid, maxwid, zeropad, left_align, is_numeric, ctx, builder,
            );
        }
    }

    // Handle statuscolumn highlights
    if flag == StlFlag::SignCol {
        highlights.add_statuscol_hl(builder.position(), ctx.cur_userhl, StlFlag::SignCol);
    } else if flag == StlFlag::FoldCol {
        highlights.add_statuscol_hl(builder.position(), ctx.cur_userhl, StlFlag::FoldCol);
    }

    ctx.prevchar_isflag = is_flag;
    ctx.prevchar_isitem = true;
}

/// Evaluate an item and return its string value.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn eval_item(flag: StlFlag, ctx: &RenderContext, buf: &mut [u8]) -> (String, bool, bool) {
    let wp = ctx.wp;

    unsafe {
        let buffer = nvim_win_get_buffer(wp);

        match flag {
            // Filename items
            StlFlag::FilePath => {
                let fname = get_filename(buffer, false);
                (fname, false, false)
            }
            StlFlag::FullPath => {
                let fname = get_filename(buffer, true);
                (fname, false, false)
            }
            StlFlag::Filename => {
                let fname = get_filename(buffer, false);
                let tail = fname.rsplit('/').next().unwrap_or(&fname);
                (tail.to_string(), false, false)
            }

            // Position items
            StlFlag::Line => {
                let lnum = nvim_win_get_cursor_lnum(wp);
                (format!("{lnum}"), true, false)
            }
            StlFlag::NumLines => {
                let count = nvim_win_buf_line_count(wp);
                (format!("{count}"), true, false)
            }
            StlFlag::Column => {
                let col = nvim_win_get_cursor_col(wp) + 1;
                (format!("{col}"), true, false)
            }
            StlFlag::VirtCol => {
                let vcol = nvim_win_get_virtcol(wp) + 1;
                (format!("{vcol}"), true, false)
            }
            StlFlag::VirtColAlt => {
                let col = nvim_win_get_cursor_col(wp) + 1;
                let vcol = nvim_win_get_virtcol(wp) + 1;
                if vcol == col {
                    (String::new(), true, true) // Empty like flag
                } else {
                    (format!("{vcol}"), true, false)
                }
            }

            // Percentage items
            StlFlag::Percentage => {
                let lnum = nvim_win_get_cursor_lnum(wp);
                let count = nvim_win_buf_line_count(wp);
                let perc = calc_percentage(lnum, count);
                (format!("{perc}"), true, false)
            }
            StlFlag::AltPercent => {
                let len = crate::rs_stl_get_rel_pos(buf.as_mut_ptr(), buf.len() as c_int, wp);
                if len > 0 {
                    let s = std::str::from_utf8_unchecked(&buf[..len as usize]);
                    (s.to_string(), false, false)
                } else {
                    ("Top".to_string(), false, false)
                }
            }

            // Buffer number
            StlFlag::BufNo => {
                let fnum = nvim_buf_get_fnum(buffer);
                (format!("{fnum}"), true, false)
            }

            // Offset items
            StlFlag::Offset => {
                let info = nvim_stl_get_win_cursor_info(wp);
                (format!("{}", info.byte_offset), true, false)
            }
            StlFlag::OffsetX => {
                let info = nvim_stl_get_win_cursor_info(wp);
                (format!("{:X}", info.byte_offset), true, false)
            }

            // Byte value items
            StlFlag::ByteVal => {
                let info = nvim_stl_get_win_cursor_info(wp);
                (format!("{}", info.byte_value), true, false)
            }
            StlFlag::ByteValX => {
                let info = nvim_stl_get_win_cursor_info(wp);
                (format!("{:X}", info.byte_value), true, false)
            }

            // Flag items
            StlFlag::RoFlag => {
                if nvim_buf_get_b_p_ro(buffer) != 0 {
                    ("[RO]".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }
            StlFlag::RoFlagAlt => {
                if nvim_buf_get_b_p_ro(buffer) != 0 {
                    (",RO".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }
            StlFlag::Modified => {
                let changed = nvim_buf_get_b_changed(buffer);
                let modifiable = nvim_buf_get_b_p_ma(buffer) != 0;
                let s = match (changed, modifiable) {
                    (true, _) => "[+]",
                    (false, false) => "[-]",
                    (false, true) => "",
                };
                (s.to_string(), false, true)
            }
            StlFlag::ModifiedAlt => {
                let changed = nvim_buf_get_b_changed(buffer);
                let modifiable = nvim_buf_get_b_p_ma(buffer) != 0;
                let s = match (changed, modifiable) {
                    (true, _) => ",+",
                    (false, false) => ",-",
                    (false, true) => "",
                };
                (s.to_string(), false, true)
            }
            StlFlag::HelpFlag => {
                if nvim_buf_get_help(buffer) != 0 {
                    ("[Help]".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }
            StlFlag::HelpFlagAlt => {
                if nvim_buf_get_help(buffer) != 0 {
                    (",HLP".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }
            StlFlag::PreviewFlag => {
                if nvim_win_get_pvw(wp) != 0 {
                    ("[Preview]".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }
            StlFlag::PreviewFlagAlt => {
                if nvim_win_get_pvw(wp) != 0 {
                    (",PRV".to_string(), false, true)
                } else {
                    (String::new(), false, true)
                }
            }

            // Filetype items
            StlFlag::Filetype => {
                let ft = get_filetype(buffer);
                if ft.is_empty() {
                    (String::new(), false, true)
                } else {
                    (format!("[{ft}]"), false, true)
                }
            }
            StlFlag::FiletypeAlt => {
                let ft = get_filetype(buffer);
                if ft.is_empty() {
                    (String::new(), false, true)
                } else {
                    (format!(",{}", ft.to_uppercase()), false, true)
                }
            }

            // Quickfix items
            StlFlag::Quickfix => {
                if nvim_win_is_qf_win(wp) {
                    let msg = if nvim_win_get_llist_ref(wp).is_null() {
                        nvim_stl_get_msg_qflist()
                    } else {
                        nvim_stl_get_msg_loclist()
                    };
                    if msg.is_null() {
                        (String::new(), false, false)
                    } else {
                        let msg_len = strlen(msg);
                        if msg_len > 0 {
                            let slice = std::slice::from_raw_parts(msg.cast::<u8>(), msg_len);
                            let s = std::str::from_utf8_unchecked(slice);
                            (s.to_string(), false, false)
                        } else {
                            (String::new(), false, false)
                        }
                    }
                } else {
                    (String::new(), false, false)
                }
            }

            // Keymap
            StlFlag::Keymap => {
                let fmt = c"<%s>".as_ptr();
                let len = get_keymap_str(wp, fmt, buf.as_mut_ptr().cast(), buf.len() as c_int);
                if len > 0 {
                    let s = std::str::from_utf8_unchecked(&buf[..len as usize]);
                    (s.to_string(), false, false)
                } else {
                    (String::new(), false, false)
                }
            }

            // Argument list
            StlFlag::ArgListStat => {
                let len = crate::rs_stl_append_arg_number(buf.as_mut_ptr(), buf.len(), wp);
                if len > 0 {
                    let s = std::str::from_utf8_unchecked(&buf[..len as usize]);
                    (s.to_string(), false, false)
                } else {
                    (String::new(), false, false)
                }
            }

            // Page number (printing) - always 0 (not applicable for screen display)
            StlFlag::PageNum => (String::from("0"), true, false),

            // Showcmd
            StlFlag::ShowCmd => {
                if showcmd_buf[0] == 0 {
                    (String::new(), false, false)
                } else {
                    let mut len = 0usize;
                    while len < showcmd_buf.len() && showcmd_buf[len] != 0 {
                        len += 1;
                    }
                    let s = std::str::from_utf8_unchecked(&showcmd_buf[..len]);
                    (s.to_string(), false, false)
                }
            }

            // Statuscolumn items - these get special handling
            StlFlag::FoldCol | StlFlag::SignCol => {
                // These need to be handled at a higher level with proper context
                (String::new(), false, false)
            }

            // Not directly rendered
            StlFlag::VimExpr
            | StlFlag::Separate
            | StlFlag::TruncMark
            | StlFlag::UserHl
            | StlFlag::Highlight
            | StlFlag::TabPageNr
            | StlFlag::TabCloseNr
            | StlFlag::ClickFunc => (String::new(), false, false),
        }
    }
}

/// Get filename from buffer.
unsafe fn get_filename(buf: nvim_window::BufHandle, full: bool) -> String {
    let ptr = if full {
        nvim_buf_get_b_ffname(buf)
    } else {
        nvim_buf_get_b_fname(buf)
    };

    if ptr.is_null() {
        return "[No Name]".to_string();
    }

    match CStr::from_ptr(ptr).to_str() {
        Ok(s) if !s.is_empty() => s.to_string(),
        _ => "[No Name]".to_string(),
    }
}

/// Get filetype from buffer.
unsafe fn get_filetype(buf: nvim_window::BufHandle) -> String {
    let ptr = nvim_buf_get_b_p_ft(buf);
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default()
}

/// Append content with width formatting.
#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn append_with_width(
    content: &str,
    minwid: c_int,
    maxwid: c_int,
    zeropad: bool,
    left_align: bool,
    is_numeric: bool,
    _ctx: &RenderContext,
    builder: &mut StatuslineBuilder,
) {
    let content_width = content.len() as c_int; // Simplified: use byte length

    // Apply truncation if needed
    let output = if maxwid > 0 && maxwid < 9999 && content_width > maxwid {
        // Truncate from beginning with '<' marker
        let excess = (content_width - maxwid + 1) as usize;
        if excess < content.len() {
            format!("<{}", &content[excess..])
        } else {
            "<".to_string()
        }
    } else {
        content.to_string()
    };

    let output_width = output.len() as c_int;

    // Apply padding if needed
    let abs_minwid = minwid.abs();
    if abs_minwid > output_width {
        let padding = (abs_minwid - output_width) as usize;
        let pad_char = if zeropad && is_numeric && !left_align {
            '0'
        } else {
            ' '
        };

        if left_align {
            builder.append_literal(&output);
            for _ in 0..padding {
                builder.append_byte(pad_char as u8);
            }
        } else {
            for _ in 0..padding {
                builder.append_byte(pad_char as u8);
            }
            builder.append_literal(&output);
        }
    } else {
        builder.append_literal(&output);
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Result structure for FFI.
#[repr(C)]
pub struct StlBuildResult {
    /// Number of bytes written to output buffer
    pub len: c_int,
    /// Number of highlight records
    pub hl_count: c_int,
    /// Number of click records
    pub click_count: c_int,
    /// Whether truncation was applied
    pub truncated: c_int,
}

/// C-compatible highlight record for FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StlHlRecordFfi {
    /// Start position in output buffer (pointer offset in C)
    pub start_offset: c_int,
    /// User highlight group (0=none, 1-9=User1-9, <0=syn_id)
    pub userhl: c_int,
    /// Item flag for statuscolumn (0 = none)
    pub item: c_int,
}

/// C-compatible click record for FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StlClickRecordFfi {
    /// Start position in output buffer
    pub start_offset: c_int,
    /// Click type (0=disabled, 1=tab switch, 2=tab close, 3=func run)
    pub click_type: c_int,
    /// Tab number for tab switch/close, or minwid for func run
    pub tabnr: c_int,
}

/// Extended result structure with room for highlight/click data.
#[repr(C)]
pub struct StlBuildResultExt {
    /// Basic result
    pub base: StlBuildResult,
    /// Width of output in screen cells
    pub width: c_int,
}

/// Build statusline string from format (FFI entry point).
///
/// # Safety
/// - `fmt` must be a valid C string.
/// - `out` must be a valid buffer of at least `out_len` bytes.
/// - `wp` must be a valid window handle.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_build_stl_str(
    fmt: *const c_char,
    out: *mut c_char,
    out_len: c_int,
    wp: WinHandle,
    max_width: c_int,
    fill_char: c_int,
) -> StlBuildResult {
    if fmt.is_null() || out.is_null() || out_len <= 0 {
        return StlBuildResult {
            len: 0,
            hl_count: 0,
            click_count: 0,
            truncated: 0,
        };
    }

    let Ok(fmt_str) = CStr::from_ptr(fmt).to_str() else {
        return StlBuildResult {
            len: 0,
            hl_count: 0,
            click_count: 0,
            truncated: 0,
        };
    };

    let fill = char::from_u32(fill_char as u32).unwrap_or(' ');
    let mut ctx = RenderContext::new(wp, max_width, fill);

    let result = build_stl_str(fmt_str, &mut ctx);

    // Copy output
    let copy_len = result.output.len().min((out_len - 1) as usize);
    std::ptr::copy_nonoverlapping(result.output.as_ptr(), out.cast(), copy_len);
    *out.add(copy_len) = 0; // NUL terminate

    StlBuildResult {
        len: copy_len as c_int,
        hl_count: result.highlights.len() as c_int,
        click_count: result.clicks.len() as c_int,
        truncated: c_int::from(result.truncated),
    }
}

/// Build statusline with highlight and click records.
///
/// This is the main FFI function for building a statusline with full
/// highlight and click record output. It replaces the C `build_stl_str_hl`
/// function.
///
/// # Safety
/// - `fmt` must be a valid C string.
/// - `out` must be a valid buffer of at least `out_len` bytes.
/// - `wp` must be a valid window handle.
/// - `hl_out` must be null or a valid buffer of at least `hl_max` records.
/// - `click_out` must be null or a valid buffer of at least `click_max` records.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_build_stl_str_hl(
    fmt: *const c_char,
    out: *mut c_char,
    out_len: c_int,
    wp: WinHandle,
    max_width: c_int,
    fill_char: c_int,
    hl_out: *mut StlHlRecordFfi,
    hl_max: c_int,
    click_out: *mut StlClickRecordFfi,
    click_max: c_int,
) -> StlBuildResultExt {
    let empty_result = StlBuildResultExt {
        base: StlBuildResult {
            len: 0,
            hl_count: 0,
            click_count: 0,
            truncated: 0,
        },
        width: 0,
    };

    if fmt.is_null() || out.is_null() || out_len <= 0 {
        return empty_result;
    }

    let Ok(fmt_str) = CStr::from_ptr(fmt).to_str() else {
        return empty_result;
    };

    let fill = char::from_u32(fill_char as u32).unwrap_or(' ');
    let mut ctx = RenderContext::new(wp, max_width, fill);

    let result = build_stl_str(fmt_str, &mut ctx);

    // Copy output string
    let copy_len = result.output.len().min((out_len - 1) as usize);
    std::ptr::copy_nonoverlapping(result.output.as_ptr(), out.cast(), copy_len);
    *out.add(copy_len) = 0; // NUL terminate

    // Copy highlight records
    let hl_count = if !hl_out.is_null() && hl_max > 0 {
        let max = (hl_max as usize).min(result.highlights.len());
        for (i, hl) in result.highlights.iter().take(max).enumerate() {
            *hl_out.add(i) = StlHlRecordFfi {
                start_offset: hl.start as c_int,
                userhl: hl.userhl,
                item: hl.item as c_int,
            };
        }
        // Add terminator if space
        if max < hl_max as usize {
            *hl_out.add(max) = StlHlRecordFfi {
                start_offset: -1,
                userhl: 0,
                item: 0,
            };
        }
        max as c_int
    } else {
        0
    };

    // Copy click records
    let click_count = if !click_out.is_null() && click_max > 0 {
        let max = (click_max as usize).min(result.clicks.len());
        for (i, (start, click_type, tabnr, _func)) in result.clicks.iter().take(max).enumerate() {
            let type_code = match *click_type {
                crate::click::ClickType::TabSwitch => 1,
                crate::click::ClickType::TabClose => 2,
                crate::click::ClickType::FuncRun => 3,
                crate::click::ClickType::Disabled => 0,
            };
            *click_out.add(i) = StlClickRecordFfi {
                start_offset: *start as c_int,
                click_type: type_code,
                tabnr: *tabnr,
            };
        }
        // Add terminator if space
        if max < click_max as usize {
            *click_out.add(max) = StlClickRecordFfi {
                start_offset: -1,
                click_type: 0,
                tabnr: 0,
            };
        }
        max as c_int
    } else {
        0
    };

    StlBuildResultExt {
        base: StlBuildResult {
            len: copy_len as c_int,
            hl_count,
            click_count,
            truncated: c_int::from(result.truncated),
        },
        width: copy_len as c_int, // TODO: calculate actual cell width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_new() {
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        assert_eq!(ctx.max_width, 80);
        assert_eq!(ctx.fill_char, ' ');
        assert!(!ctx.is_tabline);
        assert!(!ctx.is_statuscol);
    }

    #[test]
    fn test_build_result_defaults() {
        let result = BuildResult {
            output: Vec::new(),
            highlights: Vec::new(),
            clicks: Vec::new(),
            separator_count: 0,
            truncated: false,
        };
        assert!(result.output.is_empty());
        assert!(result.highlights.is_empty());
        assert!(!result.truncated);
    }

    #[test]
    fn test_append_with_width_simple() {
        let mut builder = StatuslineBuilder::new(80);
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        append_with_width("test", 0, 9999, false, false, false, &ctx, &mut builder);
        assert_eq!(builder.output_str(), Some("test"));
    }

    #[test]
    fn test_append_with_width_padding_right() {
        let mut builder = StatuslineBuilder::new(80);
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        append_with_width("hi", 5, 9999, false, false, false, &ctx, &mut builder);
        assert_eq!(builder.output_str(), Some("   hi"));
    }

    #[test]
    fn test_append_with_width_padding_left() {
        let mut builder = StatuslineBuilder::new(80);
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        append_with_width("hi", -5, 9999, false, true, false, &ctx, &mut builder);
        assert_eq!(builder.output_str(), Some("hi   "));
    }

    #[test]
    fn test_append_with_width_truncation() {
        let mut builder = StatuslineBuilder::new(80);
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        append_with_width("hello world", 0, 5, false, false, false, &ctx, &mut builder);
        let output = builder.output_str().unwrap();
        assert_eq!(output.len(), 5);
        assert!(output.starts_with('<'));
    }

    #[test]
    fn test_append_with_width_zeropad() {
        let mut builder = StatuslineBuilder::new(80);
        let ctx = RenderContext::new(WinHandle::null(), 80, ' ');
        append_with_width("42", 5, 9999, true, false, true, &ctx, &mut builder);
        assert_eq!(builder.output_str(), Some("00042"));
    }
}
