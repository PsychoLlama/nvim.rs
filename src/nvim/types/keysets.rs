// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_buf_attach {
    pub is_set__buf_attach_: OptionalKeys,
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: Boolean,
    pub preview: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_buf_delete {
    pub is_set__buf_delete_: OptionalKeys,
    pub force: Boolean,
    pub unload: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_clear_autocmds {
    pub is_set__clear_autocmds_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub event: Object,
    pub group: Object,
    pub pattern: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd {
    pub is_set__cmd_: OptionalKeys,
    pub cmd: String_0,
    pub range: Array,
    pub count: Integer,
    pub reg: String_0,
    pub bang: Boolean,
    pub args: Array,
    pub magic: Dict,
    pub mods: Dict,
    pub nargs: Object,
    pub addr: String_0,
    pub nextcmd: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_opts {
    pub output: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_complete_set {
    pub is_set__complete_set_: OptionalKeys,
    pub info: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_context {
    pub is_set__context_: OptionalKeys,
    pub types: Array,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_create_augroup {
    pub is_set__create_augroup_: OptionalKeys,
    pub clear: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_create_autocmd {
    pub is_set__create_autocmd_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub callback: Object,
    pub command: String_0,
    pub desc: String_0,
    pub group: Object,
    pub nested: Boolean,
    pub once: Boolean,
    pub pattern: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_echo_opts {
    pub is_set__echo_opts_: OptionalKeys,
    pub err: Boolean,
    pub verbose: Boolean,
    pub _truncate: Boolean,
    pub kind: String_0,
    pub id: Object,
    pub title: String_0,
    pub status: String_0,
    pub percent: Integer,
    pub source: String_0,
    pub data: Dict,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_empty {
    pub is_set__empty_: OptionalKeys,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_eval_statusline {
    pub is_set__eval_statusline_: OptionalKeys,
    pub winid: Window,
    pub maxwidth: Integer,
    pub fillchar: String_0,
    pub highlights: Boolean,
    pub use_winbar: Boolean,
    pub use_tabline: Boolean,
    pub use_statuscol_lnum: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_exec_autocmds {
    pub is_set__exec_autocmds_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub group: Object,
    pub modeline: Boolean,
    pub pattern: Object,
    pub data: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_exec_opts {
    pub output: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_autocmds {
    pub is_set__get_autocmds_: OptionalKeys,
    pub event: Object,
    pub group: Object,
    pub pattern: Object,
    pub buffer: Object,
    pub buf: Object,
    pub id: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_commands {
    pub builtin: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmark {
    pub is_set__get_extmark_: OptionalKeys,
    pub details: Boolean,
    pub hl_name: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmarks {
    pub is_set__get_extmarks_: OptionalKeys,
    pub limit: Integer,
    pub details: Boolean,
    pub hl_name: Boolean,
    pub overlap: Boolean,
    pub type_0: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_highlight {
    pub is_set__get_highlight_: OptionalKeys,
    pub id: Integer,
    pub name: String_0,
    pub link: Boolean,
    pub create: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_ns {
    pub is_set__get_ns_: OptionalKeys,
    pub winid: Window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub is_set__highlight_: OptionalKeys,
    pub altfont: Boolean,
    pub blink: Boolean,
    pub bold: Boolean,
    pub conceal: Boolean,
    pub dim: Boolean,
    pub italic: Boolean,
    pub nocombine: Boolean,
    pub overline: Boolean,
    pub reverse: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub undercurl: Boolean,
    pub underdashed: Boolean,
    pub underdotted: Boolean,
    pub underdouble: Boolean,
    pub underline: Boolean,
    pub default_: Boolean,
    pub cterm: Dict,
    pub foreground: Object,
    pub fg: Object,
    pub background: Object,
    pub bg: Object,
    pub ctermfg: Object,
    pub ctermbg: Object,
    pub special: Object,
    pub sp: Object,
    pub link: HLGroupID,
    pub link_global: HLGroupID,
    pub fallback: Boolean,
    pub blend: Integer,
    pub fg_indexed: Boolean,
    pub bg_indexed: Boolean,
    pub force: Boolean,
    pub update: Boolean,
    pub url: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_keymap {
    pub is_set__keymap_: OptionalKeys,
    pub noremap: Boolean,
    pub nowait: Boolean,
    pub silent: Boolean,
    pub script: Boolean,
    pub expr: Boolean,
    pub unique: Boolean,
    pub callback: LuaRef,
    pub desc: String_0,
    pub replace_keycodes: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_ns_opts {
    pub is_set__ns_opts_: OptionalKeys,
    pub wins: Array,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_open_term {
    pub is_set__open_term_: OptionalKeys,
    pub on_input: LuaRef,
    pub force_crlf: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_option {
    pub is_set__option_: OptionalKeys,
    pub scope: String_0,
    pub win: Window,
    pub buf: Buffer,
    pub filetype: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_redraw {
    pub is_set__redraw_: OptionalKeys,
    pub flush: Boolean,
    pub cursor: Boolean,
    pub valid: Boolean,
    pub statuscolumn: Boolean,
    pub statusline: Boolean,
    pub tabline: Boolean,
    pub winbar: Boolean,
    pub range: Array,
    pub win: Window,
    pub buf: Buffer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_runtime {
    pub is_lua: Boolean,
    pub do_source: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_decoration_provider {
    pub is_set__set_decoration_provider_: OptionalKeys,
    pub on_start: LuaRef,
    pub on_buf: LuaRef,
    pub on_win: LuaRef,
    pub on_line: LuaRef,
    pub on_range: LuaRef,
    pub on_end: LuaRef,
    pub _on_hl_def: LuaRef,
    pub _on_spell_nav: LuaRef,
    pub _on_conceal_line: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_extmark {
    pub is_set__set_extmark_: OptionalKeys,
    pub id: Integer,
    pub end_line: Integer,
    pub end_row: Integer,
    pub end_col: Integer,
    pub hl_group: Object,
    pub virt_text: Array,
    pub virt_text_pos: String_0,
    pub virt_text_win_col: Integer,
    pub virt_text_hide: Boolean,
    pub virt_text_repeat_linebreak: Boolean,
    pub hl_eol: Boolean,
    pub hl_mode: String_0,
    pub invalidate: Boolean,
    pub ephemeral: Boolean,
    pub priority: Integer,
    pub right_gravity: Boolean,
    pub end_right_gravity: Boolean,
    pub virt_lines: Array,
    pub virt_lines_above: Boolean,
    pub virt_lines_leftcol: Boolean,
    pub virt_lines_overflow: String_0,
    pub strict: Boolean,
    pub sign_text: String_0,
    pub sign_hl_group: HLGroupID,
    pub number_hl_group: HLGroupID,
    pub line_hl_group: HLGroupID,
    pub cursorline_hl_group: HLGroupID,
    pub conceal: String_0,
    pub conceal_lines: String_0,
    pub spell: Boolean,
    pub ui_watched: Boolean,
    pub undo_restore: Boolean,
    pub url: String_0,
    pub scoped: Boolean,
    pub _subpriority: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_tabpage_config {
    pub is_set__tabpage_config_: OptionalKeys,
    pub after: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_user_command {
    pub is_set__user_command_: OptionalKeys,
    pub addr: Object,
    pub bang: Boolean,
    pub bar: Boolean,
    pub complete: Object,
    pub count: Object,
    pub desc: Object,
    pub force: Boolean,
    pub keepscript: Boolean,
    pub nargs: Object,
    pub preview: Object,
    pub range: Object,
    pub register_: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_win_config {
    pub is_set__win_config_: OptionalKeys,
    pub external: Boolean,
    pub fixed: Boolean,
    pub focusable: Boolean,
    pub footer: Object,
    pub footer_pos: String_0,
    pub hide: Boolean,
    pub height: Integer,
    pub mouse: Boolean,
    pub relative: String_0,
    pub row: Float,
    pub style: String_0,
    pub noautocmd: Boolean,
    pub vertical: Boolean,
    pub win: Window,
    pub width: Integer,
    pub zindex: Integer,
    pub anchor: String_0,
    pub border: Object,
    pub bufpos: Array,
    pub col: Float,
    pub split: String_0,
    pub title: Object,
    pub title_pos: String_0,
    pub _cmdline_offset: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_win_text_height {
    pub is_set__win_text_height_: OptionalKeys,
    pub start_row: Integer,
    pub end_row: Integer,
    pub start_vcol: Integer,
    pub end_vcol: Integer,
    pub max_height: Integer,
}
