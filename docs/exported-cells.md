# Exported cells

A static the compiler thinks another crate may name is _exported_ — `D`/`B`
in `nm`, where a local one is `d`/`b` — and every read of an exported static
goes through the GOT: one more load on every access, which nothing in the
source shows. A cell is exported when a body that names it is shippable
cross-crate: a `pub` generic or `#[inline]` item, a small call-free `pub` fn
(rustc inlines those across crates without being asked; intrinsic calls do
not count as calls), an `#[inline]` method of a trait impl for a `pub` type,
or the cell itself being `pub` and reachable.

`just state-exports` reads a release binary and fails on any
`GlobalCell`/`SharedCell` static that is exported and not listed here, and on
any row here whose cell is local now or gone. The list only shrinks: when a
module is finished its rows go, and a row never comes back. The state
records, the option table and the hot cells the script names are not
eligible for a row at all — those must be local.

The usual fix is `pub(crate)` on the leaf that names the cell (or on the
cell), a cell that was a constant in disguise becoming a `const`, or the
access moving into a body that is not a leaf. Check a `codegen-units = 1`
binary as well as the default one.

## `api`

- `neovim::api::extmark::ns::next_namespace_id`

## `autocmd`

- `neovim::autocmd::autocmds`

## `buffer`

- `neovim::buffer::top_file_num`

## `charset`

- `neovim::charset::g_chartab`

## `cmdexpand`

- `neovim::cmdexpand::compl_match_array`

## `cmdhist`

- `neovim::cmdhist::HISTORY`

## `decoration`

- `neovim::decoration::decor_state`

## `diff`

- `neovim::diff::diff_flags`

## `drawscreen`

- `neovim::drawscreen::state::need_maketitle`
- `neovim::drawscreen::state::redraw_tabline`

## `ex_cmds`

- `neovim::ex_cmds::subst::parse::old_sub`

## `ex_docmd`

- `neovim::ex_docmd::ex_pressedreturn`
- `neovim::ex_docmd::state::cmdmod`

## `ex_getln`

- `neovim::ex_getln::ccline`
- `neovim::ex_getln::cmdpreview_bufnr`
- `neovim::ex_getln::cmdpreview_ns`
- `neovim::ex_getln::last_prompt_id`

## `getchar`

- `neovim::getchar::READBUF1`
- `neovim::getchar::READBUF2`
- `neovim::getchar::typeahead::TYPEBUF`

## `grid`

- `neovim::grid::default_grid`
- `neovim::grid::grid_assign_handle::LAST_GRID_HANDLE`
- `neovim::grid::line::BATCH`

## `guard`

- `neovim::guard::sandbox`

## `highlight`

- `neovim::highlight::ATTRS`
- `neovim::highlight::URLS`

## `log`

- `neovim::log::DID_LOG_INIT`
- `neovim::log::DID_RECURSION_MSG`
- `neovim::log::g_min_log_level`
- `neovim::log::g_stats`

## `lua`

- `neovim::lua::executor::active_lstate`
- `neovim::lua::executor::global_lstate`
- `neovim::lua::executor::in_fast_callback`
- `neovim::lua::state::nlua_global_refs`

## `message`

- `neovim::message::scrollback::SCROLLBACK`

## `option`

- `neovim::option::vars::bkc_flags`
- `neovim::option::vars::bo_flags`
- `neovim::option::vars::breakat_flags`
- `neovim::option::vars::cb_flags`
- `neovim::option::vars::cmp_flags`
- `neovim::option::vars::cot_flags`
- `neovim::option::vars::dy_flags`
- `neovim::option::vars::fdo_flags`
- `neovim::option::vars::jop_flags`
- `neovim::option::vars::rdb_flags`
- `neovim::option::vars::ssop_flags`
- `neovim::option::vars::swb_flags`
- `neovim::option::vars::tc_flags`
- `neovim::option::vars::tcl_flags`
- `neovim::option::vars::tpf_flags`
- `neovim::option::vars::ve_flags`
- `neovim::option::vars::vop_flags`
- `neovim::option::vars::wop_flags`

## `os`

- `neovim::os::env::default_lib_dir`
- `neovim::os::env::default_vim_dir`
- `neovim::os::env::default_vimruntime_dir`
- `neovim::os::input::blocking`
- `neovim::os::input::input_read_pos`
- `neovim::os::input::input_write_pos`
- `neovim::os::signal::REJECTING_DEADLY`

## `popupmenu`

- `neovim::popupmenu::pum_external`
- `neovim::popupmenu::pum_first`
- `neovim::popupmenu::pum_invalid`
- `neovim::popupmenu::pum_is_visible`
- `neovim::popupmenu::pum_size`
- `neovim::popupmenu::state::pum_want`

## `runtime`

- `neovim::runtime::exestack`
- `neovim::runtime::runtime_search_path_valid`
- `neovim::runtime::script_items`
- `neovim::runtime::state::current_sctx`

## `search`

- `neovim::search::pattern::last_idx`
- `neovim::search::pattern::spats`

## `spell`

- `neovim::spell::did_set_spelltab`
- `neovim::spell::int_wordlist`
- `neovim::spell::spelltab`

## `startup`

- `neovim::startup::embedded_mode`
- `neovim::startup::eventloop::main_loop`
- `neovim::startup::ex_exitval`
- `neovim::startup::exiting`
- `neovim::startup::full_screen`
- `neovim::startup::headless_mode`
- `neovim::startup::nvim_testing`
- `neovim::startup::readonlymode`
- `neovim::startup::recoverymode`
- `neovim::startup::silent_mode`
- `neovim::startup::stderr_isatty`
- `neovim::startup::stdin_fd`
- `neovim::startup::stdin_isatty`
- `neovim::startup::stdout_isatty`
- `neovim::startup::ui_client_attached`
- `neovim::startup::ui_client_channel_id`
- `neovim::startup::ui_client_error_exit`
- `neovim::startup::ui_client_exit_status`
- `neovim::startup::ui_client_forward_stdin`
- `neovim::startup::used_stdin`
- `neovim::startup::v_dying`
- `neovim::startup::vim_ignored`

## `state`

- `neovim::state::was_safe`

## `tui`

- `neovim::tui::cursor::cursor_style_enabled`

## `ui`

- `neovim::ui::attached`
- `neovim::ui::callbacks::ui_cb_ext`
- `neovim::ui::cursor_col`
- `neovim::ui::cursor_grid_handle`
- `neovim::ui::cursor_row`
- `neovim::ui::pending_cursor_update`
- `neovim::ui::pending_mode_info_update`
- `neovim::ui::ui_ext`

## `ui_compositor`

- `neovim::ui_compositor::composed_uis`
- `neovim::ui_compositor::msg_sep_row`
- `neovim::ui_compositor::valid_screen`

## `window`

- `neovim::window::frame_locked`
- `neovim::window::last_win_id`

## `winlayer`

- `neovim::winlayer::graph::CURRENT_BUF`
- `neovim::winlayer::graph::CURRENT_TAB`
- `neovim::winlayer::graph::CURRENT_WIN`
- `neovim::winlayer::graph::cmdwin_type`
- `neovim::winlayer::graph::curbuf`
- `neovim::winlayer::graph::curtab`
- `neovim::winlayer::handles::BUFFERS`
- `neovim::winlayer::handles::FRAMES`
- `neovim::winlayer::handles::TABPAGES`
- `neovim::winlayer::handles::WINDOWS`
