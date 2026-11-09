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

- `neovim::buffer::TOP_FILE_NUM`

## `charset`

- `neovim::charset::G_CHARTAB`

## `cmdexpand`

- `neovim::cmdexpand::CMDLINE_PUM`

## `cmdhist`

- `neovim::cmdhist::HISTORY`

## `diff`

- `neovim::diff::DIFF_FLAGS`

## `drawscreen`

- `neovim::drawscreen::state::NEED_MAKETITLE`
- `neovim::drawscreen::state::REDRAW_TABLINE`

## `ex_docmd`

- `neovim::ex_docmd::EX_PRESSEDRETURN`

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

- `neovim::guard::SANDBOX`

## `highlight`

- `neovim::highlight::ATTRS`

## `log`

- `neovim::log::DID_LOG_INIT`
- `neovim::log::DID_RECURSION_MSG`
- `neovim::log::G_MIN_LOG_LEVEL`
- `neovim::log::G_STATS`

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

- `neovim::os::input::BLOCKING`
- `neovim::os::input::INPUT_READ_POS`
- `neovim::os::input::INPUT_WRITE_POS`
- `neovim::os::signal::REJECTING_DEADLY`

## `popupmenu`

- `neovim::popupmenu::PUM_EXTERNAL`
- `neovim::popupmenu::PUM_FIRST`
- `neovim::popupmenu::PUM_INVALID`
- `neovim::popupmenu::PUM_IS_VISIBLE`
- `neovim::popupmenu::PUM_SIZE`
- `neovim::popupmenu::state::PUM_WANT`

## `runtime`

- `neovim::runtime::state::CURRENT_SCTX`

## `search`

- `neovim::search::pattern::LAST_IDX`

## `spell`

- `neovim::spell::DID_SET_SPELLTAB`
- `neovim::spell::INT_WORDLIST`
- `neovim::spell::SPELLTAB`

## `startup`

- `neovim::startup::EMBEDDED_MODE`
- `neovim::startup::eventloop::MAIN_LOOP`
- `neovim::startup::EX_EXITVAL`
- `neovim::startup::EXITING`
- `neovim::startup::FULL_SCREEN`
- `neovim::startup::HEADLESS_MODE`
- `neovim::startup::NVIM_TESTING`
- `neovim::startup::READONLYMODE`
- `neovim::startup::RECOVERYMODE`
- `neovim::startup::SILENT_MODE`
- `neovim::startup::STDERR_ISATTY`
- `neovim::startup::STDIN_FD`
- `neovim::startup::STDIN_ISATTY`
- `neovim::startup::STDOUT_ISATTY`
- `neovim::startup::UI_CLIENT_ATTACHED`
- `neovim::startup::UI_CLIENT_CHANNEL_ID`
- `neovim::startup::UI_CLIENT_ERROR_EXIT`
- `neovim::startup::UI_CLIENT_EXIT_STATUS`
- `neovim::startup::UI_CLIENT_FORWARD_STDIN`
- `neovim::startup::USED_STDIN`
- `neovim::startup::V_DYING`
- `neovim::startup::VIM_IGNORED`

## `state`

- `neovim::state::WAS_SAFE`

## `tui`

- `neovim::tui::cursor::CURSOR_STYLE_ENABLED`

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

- `neovim::window::FRAME_LOCKED`
- `neovim::window::LAST_WIN_ID`

## `winlayer`

- `neovim::winlayer::graph::CURRENT_BUF`
- `neovim::winlayer::graph::CURRENT_TAB`
- `neovim::winlayer::graph::CURRENT_WIN`
- `neovim::winlayer::graph::CMDWIN_TYPE`
- `neovim::winlayer::graph::CURBUF`
- `neovim::winlayer::graph::CURTAB`
- `neovim::winlayer::handles::BUFFERS`
- `neovim::winlayer::handles::FRAMES`
- `neovim::winlayer::handles::TABPAGES`
- `neovim::winlayer::handles::WINDOWS`
