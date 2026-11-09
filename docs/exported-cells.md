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

- `neovim::api::extmark::ns::NEXT_NAMESPACE_ID`

## `buffer`

- `neovim::buffer::top_file_num`

## `charset`

- `neovim::charset::g_chartab`

## `cmdexpand`

- `neovim::cmdexpand::compl_match_array`

## `cmdhist`

- `neovim::cmdhist::HISTORY`

## `diff`

- `neovim::diff::diff_flags`

## `drawscreen`

- `neovim::drawscreen::state::NEED_MAKETITLE`
- `neovim::drawscreen::state::REDRAW_TABLINE`

## `ex_docmd`

- `neovim::ex_docmd::ex_pressedreturn`

## `ex_getln`

- `neovim::ex_getln::CCLINE`
- `neovim::ex_getln::CMDPREVIEW_BUFNR`
- `neovim::ex_getln::CMDPREVIEW_NS`
- `neovim::ex_getln::LAST_PROMPT_ID`

## `getchar`

- `neovim::getchar::READBUF1`
- `neovim::getchar::READBUF2`
- `neovim::getchar::typeahead::TYPEBUF`

## `grid`

- `neovim::grid::DEFAULT_GRID`
- `neovim::grid::grid_assign_handle::LAST_GRID_HANDLE`
- `neovim::grid::line::BATCH`

## `guard`

- `neovim::guard::sandbox`

## `highlight`

- `neovim::highlight::ATTRS`

## `log`

- `neovim::log::DID_LOG_INIT`
- `neovim::log::DID_RECURSION_MSG`
- `neovim::log::g_min_log_level`
- `neovim::log::g_stats`

## `lua`

- `neovim::lua::executor::IN_FAST_CALLBACK`

## `message`

- `neovim::message::scrollback::SCROLLBACK`

## `option`

- `neovim::option::vars::BKC_FLAGS`
- `neovim::option::vars::BO_FLAGS`
- `neovim::option::vars::BREAKAT_FLAGS`
- `neovim::option::vars::CB_FLAGS`
- `neovim::option::vars::CMP_FLAGS`
- `neovim::option::vars::COT_FLAGS`
- `neovim::option::vars::DY_FLAGS`
- `neovim::option::vars::FDO_FLAGS`
- `neovim::option::vars::JOP_FLAGS`
- `neovim::option::vars::RDB_FLAGS`
- `neovim::option::vars::SSOP_FLAGS`
- `neovim::option::vars::SWB_FLAGS`
- `neovim::option::vars::TC_FLAGS`
- `neovim::option::vars::TCL_FLAGS`
- `neovim::option::vars::TPF_FLAGS`
- `neovim::option::vars::VE_FLAGS`
- `neovim::option::vars::VOP_FLAGS`
- `neovim::option::vars::WOP_FLAGS`

## `os`

- `neovim::os::input::blocking`
- `neovim::os::input::input_read_pos`
- `neovim::os::input::input_write_pos`
- `neovim::os::signal::REJECTING_DEADLY`

## `popupmenu`

- `neovim::popupmenu::PUM_EXTERNAL`
- `neovim::popupmenu::PUM_FIRST`
- `neovim::popupmenu::PUM_INVALID`
- `neovim::popupmenu::PUM_IS_VISIBLE`
- `neovim::popupmenu::PUM_SIZE`
- `neovim::popupmenu::state::PUM_WANT`

## `runtime`

- `neovim::runtime::state::current_sctx`

## `search`

- `neovim::search::pattern::last_idx`

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

- `neovim::state::WAS_SAFE`

## `tui`

- `neovim::TUI::cursor::CURSOR_STYLE_ENABLED`

## `ui`

- `neovim::ui::callbacks::UI_CB_EXT`
- `neovim::ui::CURSOR_COL`
- `neovim::ui::CURSOR_GRID_HANDLE`
- `neovim::ui::CURSOR_ROW`
- `neovim::ui::PENDING_CURSOR_UPDATE`
- `neovim::ui::PENDING_MODE_INFO_UPDATE`
- `neovim::ui::UI_EXT`

## `ui_compositor`

- `neovim::ui_compositor::COMPOSED_UIS`
- `neovim::ui_compositor::MSG_SEP_ROW`
- `neovim::ui_compositor::VALID_SCREEN`

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
