//! Rust components for Neovim
//!
//! This crate provides Rust implementations of Neovim functionality,
//! designed to be called from C code via FFI during the migration period.
//!
//! The crate re-exports all FFI functions from sub-crates so they are
//! available in the single `libnvim_rs.a` static library.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)] // FFI functions need unsafe but docs come later
#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::wildcard_imports)] // We want to re-export everything
#![allow(ambiguous_glob_reexports)] // nvim_api and nvim_unpacker define compatible FFI types

// Re-export all FFI functions from sub-crates
// This ensures they're included in the static library
pub use nvim_api::*;
pub use nvim_arabic::*;
pub use nvim_arglist::*;
pub use nvim_ascii::*;
pub use nvim_autocmd::*;
pub use nvim_buffer::*;
pub use nvim_buffer_updates::*;
pub use nvim_bufwrite::*;
pub use nvim_change::*;
pub use nvim_channel::*;
pub use nvim_charset::*;
pub use nvim_clipboard::*;
pub use nvim_cmdexpand::*;
pub use nvim_cmdhist::*;
pub use nvim_cmdline::*;
pub use nvim_collections::garray::*;
pub use nvim_collections::hashtab::*;
pub use nvim_collections::map::*;
pub use nvim_collections::queue::*;
pub use nvim_compositor::*;
pub use nvim_context::*;
pub use nvim_cursor::*;
pub use nvim_cursor_shape::*;
pub use nvim_debugger::*;
pub use nvim_decoration::*;
pub use nvim_decoration_provider::*;
pub use nvim_dict::*;
pub use nvim_diff::*;
pub use nvim_digraph::*;
pub use nvim_drawline::*;
pub use nvim_drawscreen::*;
pub use nvim_edit::*;
// Note: rs_hash_hash, rs_hash_hash_len are in nvim_memutil (not re-exported from hashtab)
pub use nvim_encoding::base64::*;
pub use nvim_encoding::sha256::*;
pub use nvim_eval::*;
pub use nvim_eval_codec::*;
pub use nvim_eval_exec::*;
pub use nvim_event::*;
pub use nvim_ex_cmds::display::{rs_do_ascii, rs_ex_z};
pub use nvim_ex_cmds::format::rs_ex_align;
pub use nvim_ex_cmds::lines::{rs_ex_change, rs_ex_copy};
pub use nvim_ex_cmds2::*;
pub use nvim_ex_docmd::*;
pub use nvim_ex_eval::*;
pub use nvim_extmark::*;
pub use nvim_fileio::*;
pub use nvim_filesearch::*;
pub use nvim_fold::*;
pub use nvim_funcall::*;
pub use nvim_fuzzy::*;
pub use nvim_getchar::*;
pub use nvim_grid::*;
pub use nvim_help::*;
pub use nvim_highlight::*;
pub use nvim_highlight_group::*;
pub use nvim_indent::*;
pub use nvim_indent_c::*;
pub use nvim_input::*;
pub use nvim_insexpand::*;
pub use nvim_keycodes::*;
pub use nvim_linematch::*;
pub use nvim_list::*;
pub use nvim_log::*;
pub use nvim_lua::*;
pub use nvim_main::*;
pub use nvim_mapping::*;
pub use nvim_mark::*;
pub use nvim_marktree::*;
pub use nvim_match::*;
pub use nvim_math::*;
pub use nvim_mbyte::*;
pub use nvim_memfile::*;
pub use nvim_memline::*;
pub use nvim_memutil::*;
pub use nvim_menu::*;
pub use nvim_message::*;
pub use nvim_mouse::*;
pub use nvim_move::*;
pub use nvim_msgpack::*;
pub use nvim_msgpack_rpc::*;
pub use nvim_normal::*;
pub use nvim_ops::*;
pub use nvim_option::*;
pub use nvim_optionstr::*;
pub use nvim_os::env::*;
pub use nvim_os::fs::*;
pub use nvim_os::time::*;
pub use nvim_path::*;
pub use nvim_plines::*;
pub use nvim_popupmenu::*;
pub use nvim_profile::*;
pub use nvim_quickfix::*;
pub use nvim_regexp::*;
pub use nvim_register::*;
pub use nvim_runtime::*;
pub use nvim_runtime::{
    rs_add_pack_start_dirs, rs_do_in_path_and_pp, rs_do_in_runtimepath, rs_ex_packadd,
    rs_ex_packloadall, rs_ex_runtime, rs_gen_expand_wildcards_and_cb, rs_get_runtime_cmd_flags,
    rs_load_pack_plugin, rs_load_plugins, rs_load_start_packages, rs_pack_has_entries,
    rs_set_context_in_runtime_cmd, rs_source_callback, rs_source_callback_vim_lua,
    rs_source_in_path_vim_lua, rs_source_runtime, rs_source_runtime_vim_lua,
};
pub use nvim_search::*;
pub use nvim_session::*;
pub use nvim_shada::*;
pub use nvim_sign::*;
pub use nvim_spell::*;
pub use nvim_state::*;
pub use nvim_statusline::*;
pub use nvim_strings::*;
pub use nvim_syntax::*;
pub use nvim_tag::*;
pub use nvim_terminal::*;
pub use nvim_testing::*;
pub use nvim_textformat::*;
pub use nvim_textobject::*;
pub use nvim_tui::*;
pub use nvim_typval::*;
pub use nvim_ugrid::*;
pub use nvim_ui::*;
pub use nvim_ui_client::*;
pub use nvim_undo::*;
pub use nvim_unpacker::*;
pub use nvim_usercmd::*;
pub use nvim_userfunc::*;
pub use nvim_utf8proc::*;
pub use nvim_vars::*;
pub use nvim_version::*;
pub use nvim_viewport::*;
pub use nvim_viml_parser::*;
pub use nvim_vterm::*;
pub use nvim_window::*;
pub use nvim_winfloat::*;

/// FFI-safe result type for operations that can fail
#[repr(C)]
pub struct NvimResult<T> {
    pub ok: bool,
    pub value: T,
}

/// Placeholder function to verify the build system works.
/// This will be removed once real functionality is added.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn nvim_rs_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(nvim_rs_version(), 1);
    }
}
