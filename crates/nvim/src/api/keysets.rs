#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The `KeyDict_<name>` spelling is upstream's macro, and the wire keys it
// names are the API's own.
#![allow(non_snake_case)]

//! The keysets: the option-dict layouts the API takes by name.
//!
//! These structs are the source of truth `tools/apigen` generates the
//! keyset tables from (`crate::api::private::dispatch`), reading this file
//! by path, so the
//! shape here is load-bearing:
//!
//! - Declaration order fixes the table order, and a key's position in the
//!   table is a number the generated lookups answer with.
//! - **Every field is an `Option`**: `None` is the key the caller did not
//!   name, and that is the whole of "was it set?". A fresh keyset is
//!   [`Default::default`] -- all `None` -- which is why nothing here may be
//!   built by zeroing: `Option<Boolean>`'s all-zero image is
//!   `Some(false)`, the opposite answer.
//! - The field type inside the `Option` picks the `ObjectType` a value must
//!   arrive as; `Object` accepts any, and `HLGroupID` marks a
//!   highlight-group name the converter resolves to an id.
//! - A field whose name on the wire differs from its name here says so in a
//!   doc comment of the form ``Wire key: `name`.``
use crate::api::private::helpers::StringArray;
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufferHandle, Float, HLGroupID, Integer, LuaRef, Object,
    WindowHandle,
};

#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict__shada_buflist_item {
    pub l: Option<Integer>,
    pub c: Option<Integer>,
    pub f: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict__shada_mark {
    pub n: Option<Integer>,
    pub l: Option<Integer>,
    pub c: Option<Integer>,
    pub f: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict__shada_register {
    pub rc: Option<StringArray>,
    pub ru: Option<Boolean>,
    pub rt: Option<Integer>,
    pub n: Option<Integer>,
    pub rw: Option<Integer>,
}
/// ShaDa entries spell their keys as two-letter codes, so every field here
/// names the one it travels under.
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict__shada_search_pat {
    /// Wire key: `sm`.
    pub magic: Option<Boolean>,
    /// Wire key: `sc`.
    pub smartcase: Option<Boolean>,
    /// Wire key: `sl`.
    pub has_line_offset: Option<Boolean>,
    /// Wire key: `se`.
    pub place_cursor_at_end: Option<Boolean>,
    /// Wire key: `su`.
    pub is_last_used: Option<Boolean>,
    /// Wire key: `ss`.
    pub is_substitute_pattern: Option<Boolean>,
    /// Wire key: `sh`.
    pub highlighted: Option<Boolean>,
    /// Wire key: `sb`.
    pub search_backward: Option<Boolean>,
    /// Wire key: `so`.
    pub offset: Option<Integer>,
    /// Wire key: `sp`.
    pub pat: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_buf_attach {
    pub on_lines: Option<LuaRef>,
    pub on_bytes: Option<LuaRef>,
    pub on_changedtick: Option<LuaRef>,
    pub on_detach: Option<LuaRef>,
    pub on_reload: Option<LuaRef>,
    pub utf_sizes: Option<Boolean>,
    pub preview: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_buf_delete {
    pub force: Option<Boolean>,
    pub unload: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_clear_autocmds {
    pub buffer: Option<BufferHandle>,
    pub buf: Option<BufferHandle>,
    pub event: Option<Object>,
    pub group: Option<Object>,
    pub pattern: Option<Object>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_cmd {
    pub cmd: Option<ApiString>,
    pub range: Option<Array>,
    pub count: Option<Integer>,
    pub reg: Option<ApiString>,
    pub bang: Option<Boolean>,
    pub args: Option<Array>,
    pub magic: Option<ApiDict>,
    pub mods: Option<ApiDict>,
    pub nargs: Option<Object>,
    pub addr: Option<ApiString>,
    pub nextcmd: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_cmd_magic {
    pub file: Option<Boolean>,
    pub bar: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_cmd_mods {
    pub silent: Option<Boolean>,
    pub emsg_silent: Option<Boolean>,
    pub unsilent: Option<Boolean>,
    pub filter: Option<ApiDict>,
    pub sandbox: Option<Boolean>,
    pub noautocmd: Option<Boolean>,
    pub browse: Option<Boolean>,
    pub confirm: Option<Boolean>,
    pub hide: Option<Boolean>,
    pub horizontal: Option<Boolean>,
    pub keepalt: Option<Boolean>,
    pub keepjumps: Option<Boolean>,
    pub keepmarks: Option<Boolean>,
    pub keeppatterns: Option<Boolean>,
    pub lockmarks: Option<Boolean>,
    pub noswapfile: Option<Boolean>,
    pub tab: Option<Integer>,
    pub verbose: Option<Integer>,
    pub vertical: Option<Boolean>,
    pub split: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_cmd_mods_filter {
    pub pattern: Option<ApiString>,
    pub force: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_cmd_opts {
    pub output: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_complete_set {
    pub info: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_context {
    pub types: Option<Array>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_create_augroup {
    pub clear: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_create_autocmd {
    pub buffer: Option<BufferHandle>,
    pub buf: Option<BufferHandle>,
    pub callback: Option<Object>,
    pub command: Option<ApiString>,
    pub desc: Option<ApiString>,
    pub group: Option<Object>,
    pub nested: Option<Boolean>,
    pub once: Option<Boolean>,
    pub pattern: Option<Object>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_echo_opts {
    pub err: Option<Boolean>,
    pub verbose: Option<Boolean>,
    pub _truncate: Option<Boolean>,
    pub kind: Option<ApiString>,
    pub id: Option<Object>,
    pub title: Option<ApiString>,
    pub status: Option<ApiString>,
    pub percent: Option<Integer>,
    pub source: Option<ApiString>,
    pub data: Option<ApiDict>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_empty {}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_eval_statusline {
    pub winid: Option<WindowHandle>,
    pub maxwidth: Option<Integer>,
    pub fillchar: Option<ApiString>,
    pub highlights: Option<Boolean>,
    pub use_winbar: Option<Boolean>,
    pub use_tabline: Option<Boolean>,
    pub use_statuscol_lnum: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_exec_autocmds {
    pub buffer: Option<BufferHandle>,
    pub buf: Option<BufferHandle>,
    pub group: Option<Object>,
    pub modeline: Option<Boolean>,
    pub pattern: Option<Object>,
    pub data: Option<Object>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_exec_opts {
    pub output: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_autocmds {
    pub event: Option<Object>,
    pub group: Option<Object>,
    pub pattern: Option<Object>,
    pub buffer: Option<Object>,
    pub buf: Option<Object>,
    pub id: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_commands {
    pub builtin: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_extmark {
    pub details: Option<Boolean>,
    pub hl_name: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_extmarks {
    pub limit: Option<Integer>,
    pub details: Option<Boolean>,
    pub hl_name: Option<Boolean>,
    pub overlap: Option<Boolean>,
    /// Wire key: `type`.
    pub type_0: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_highlight {
    pub id: Option<Integer>,
    pub name: Option<ApiString>,
    pub link: Option<Boolean>,
    pub create: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_get_ns {
    pub winid: Option<WindowHandle>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub altfont: Option<Boolean>,
    pub blink: Option<Boolean>,
    pub bold: Option<Boolean>,
    pub conceal: Option<Boolean>,
    pub dim: Option<Boolean>,
    pub italic: Option<Boolean>,
    pub nocombine: Option<Boolean>,
    pub overline: Option<Boolean>,
    pub reverse: Option<Boolean>,
    pub standout: Option<Boolean>,
    pub strikethrough: Option<Boolean>,
    pub undercurl: Option<Boolean>,
    pub underdashed: Option<Boolean>,
    pub underdotted: Option<Boolean>,
    pub underdouble: Option<Boolean>,
    pub underline: Option<Boolean>,
    /// Wire key: `default`.
    pub default_: Option<Boolean>,
    pub cterm: Option<ApiDict>,
    pub foreground: Option<Object>,
    pub fg: Option<Object>,
    pub background: Option<Object>,
    pub bg: Option<Object>,
    pub ctermfg: Option<Object>,
    pub ctermbg: Option<Object>,
    pub special: Option<Object>,
    pub sp: Option<Object>,
    pub link: Option<HLGroupID>,
    pub link_global: Option<HLGroupID>,
    pub fallback: Option<Boolean>,
    pub blend: Option<Integer>,
    pub fg_indexed: Option<Boolean>,
    pub bg_indexed: Option<Boolean>,
    pub force: Option<Boolean>,
    pub update: Option<Boolean>,
    pub url: Option<ApiString>,
}
/// The `cterm` sub-dict of a highlight definition. An unset attribute reads
/// as false, like every other attribute here that the caller left out.
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_highlight_cterm {
    pub bold: Option<Boolean>,
    pub standout: Option<Boolean>,
    pub strikethrough: Option<Boolean>,
    pub underline: Option<Boolean>,
    pub undercurl: Option<Boolean>,
    pub underdouble: Option<Boolean>,
    pub underdotted: Option<Boolean>,
    pub underdashed: Option<Boolean>,
    pub italic: Option<Boolean>,
    pub reverse: Option<Boolean>,
    pub altfont: Option<Boolean>,
    pub dim: Option<Boolean>,
    pub blink: Option<Boolean>,
    pub conceal: Option<Boolean>,
    pub overline: Option<Boolean>,
    pub nocombine: Option<Boolean>,
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_keymap {
    pub noremap: Option<Boolean>,
    pub nowait: Option<Boolean>,
    pub silent: Option<Boolean>,
    pub script: Option<Boolean>,
    pub expr: Option<Boolean>,
    pub unique: Option<Boolean>,
    pub callback: Option<LuaRef>,
    pub desc: Option<ApiString>,
    pub replace_keycodes: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_ns_opts {
    pub wins: Option<Array>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_open_term {
    pub on_input: Option<LuaRef>,
    pub force_crlf: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_option {
    pub scope: Option<ApiString>,
    pub win: Option<WindowHandle>,
    pub buf: Option<BufferHandle>,
    pub filetype: Option<ApiString>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_redraw {
    pub flush: Option<Boolean>,
    pub cursor: Option<Boolean>,
    pub valid: Option<Boolean>,
    pub statuscolumn: Option<Boolean>,
    pub statusline: Option<Boolean>,
    pub tabline: Option<Boolean>,
    pub winbar: Option<Boolean>,
    pub range: Option<Array>,
    pub win: Option<WindowHandle>,
    pub buf: Option<BufferHandle>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_runtime {
    pub is_lua: Option<Boolean>,
    pub do_source: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_set_decoration_provider {
    pub on_start: Option<LuaRef>,
    pub on_buf: Option<LuaRef>,
    pub on_win: Option<LuaRef>,
    pub on_line: Option<LuaRef>,
    pub on_range: Option<LuaRef>,
    pub on_end: Option<LuaRef>,
    pub _on_hl_def: Option<LuaRef>,
    pub _on_spell_nav: Option<LuaRef>,
    pub _on_conceal_line: Option<LuaRef>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_set_extmark {
    pub id: Option<Integer>,
    pub end_line: Option<Integer>,
    pub end_row: Option<Integer>,
    pub end_col: Option<Integer>,
    pub hl_group: Option<Object>,
    pub virt_text: Option<Array>,
    pub virt_text_pos: Option<ApiString>,
    pub virt_text_win_col: Option<Integer>,
    pub virt_text_hide: Option<Boolean>,
    pub virt_text_repeat_linebreak: Option<Boolean>,
    pub hl_eol: Option<Boolean>,
    pub hl_mode: Option<ApiString>,
    pub invalidate: Option<Boolean>,
    pub ephemeral: Option<Boolean>,
    pub priority: Option<Integer>,
    pub right_gravity: Option<Boolean>,
    pub end_right_gravity: Option<Boolean>,
    pub virt_lines: Option<Array>,
    pub virt_lines_above: Option<Boolean>,
    pub virt_lines_leftcol: Option<Boolean>,
    pub virt_lines_overflow: Option<ApiString>,
    pub strict: Option<Boolean>,
    pub sign_text: Option<ApiString>,
    pub sign_hl_group: Option<HLGroupID>,
    pub number_hl_group: Option<HLGroupID>,
    pub line_hl_group: Option<HLGroupID>,
    pub cursorline_hl_group: Option<HLGroupID>,
    pub conceal: Option<ApiString>,
    pub conceal_lines: Option<ApiString>,
    pub spell: Option<Boolean>,
    pub ui_watched: Option<Boolean>,
    pub undo_restore: Option<Boolean>,
    pub url: Option<ApiString>,
    pub scoped: Option<Boolean>,
    pub _subpriority: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_tabpage_config {
    pub after: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_user_command {
    pub addr: Option<Object>,
    pub bang: Option<Boolean>,
    pub bar: Option<Boolean>,
    pub complete: Option<Object>,
    pub count: Option<Object>,
    pub desc: Option<Object>,
    pub force: Option<Boolean>,
    pub keepscript: Option<Boolean>,
    pub nargs: Option<Object>,
    pub preview: Option<Object>,
    pub range: Option<Object>,
    /// Wire key: `register`.
    pub register_: Option<Boolean>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_win_config {
    pub external: Option<Boolean>,
    pub fixed: Option<Boolean>,
    pub focusable: Option<Boolean>,
    pub footer: Option<Object>,
    pub footer_pos: Option<ApiString>,
    pub hide: Option<Boolean>,
    pub height: Option<Integer>,
    pub mouse: Option<Boolean>,
    pub relative: Option<ApiString>,
    pub row: Option<Float>,
    pub style: Option<ApiString>,
    pub noautocmd: Option<Boolean>,
    pub vertical: Option<Boolean>,
    pub win: Option<WindowHandle>,
    pub width: Option<Integer>,
    pub zindex: Option<Integer>,
    pub anchor: Option<ApiString>,
    pub border: Option<Object>,
    pub bufpos: Option<Array>,
    pub col: Option<Float>,
    pub split: Option<ApiString>,
    pub title: Option<Object>,
    pub title_pos: Option<ApiString>,
    pub _cmdline_offset: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_win_text_height {
    pub start_row: Option<Integer>,
    pub end_row: Option<Integer>,
    pub start_vcol: Option<Integer>,
    pub end_vcol: Option<Integer>,
    pub max_height: Option<Integer>,
}
#[derive(Default, Debug)]
#[repr(C)]
pub struct KeyDict_xdl_diff {
    pub on_hunk: Option<LuaRef>,
    pub result_type: Option<ApiString>,
    pub algorithm: Option<ApiString>,
    pub ctxlen: Option<Integer>,
    pub interhunkctxlen: Option<Integer>,
    pub linematch: Option<Object>,
    pub ignore_whitespace: Option<Boolean>,
    pub ignore_whitespace_change: Option<Boolean>,
    pub ignore_whitespace_change_at_eol: Option<Boolean>,
    pub ignore_cr_at_eol: Option<Boolean>,
    pub ignore_blank_lines: Option<Boolean>,
    pub indent_heuristic: Option<Boolean>,
}
