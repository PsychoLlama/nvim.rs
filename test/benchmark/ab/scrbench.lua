-- Timing canary for the screen pipeline (B12-2).
--
--   nvim --headless -c 'set columns=200 lines=60' -c 'luafile scrbench.lua' -c 'qa!'
--
-- Run it through scrbench.sh, which does the interleaved A/B the drift rule
-- requires. Prints "phase<TAB>ms" lines prefixed with SCRBENCH so the shell
-- can pick them out of whatever else lands on stdout.
--
-- Why `--headless -c` and not `--headless -l`: in `-l` script mode
-- `full_screen` is false, `did_set_lines_or_columns` takes its
-- never-resize branch, and the grid stays 80x24 while the window layout is
-- computed for the new 'columns' -- which aborts on a grid assertion (see
-- the B12-2 docket entry). `-c` mode has full_screen set, so 'columns'
-- really resizes the grid and each redraw does ~6x the work.
--
-- Phase map (which code each one is the canary for):
--   ctl       nothing            -- noise floor (mbyte, untouched by B12)
--   wrap      grid + drawline    -- full-clear redraw of long wrapped lines
--   delta     grid_put_linebuf   -- redraw after a one-cell change: the
--                                   changed-cells comparison loop, which is
--                                   what interactive editing actually runs
--   relnum    drawline + grid    -- 'relativenumber' moves rewrite every
--                                   line's number column on every motion
--   winsplit  grid_clear/lines   -- six-window layout, separators, many
--                                   grid_line_start/flush pairs per redraw
--   scroll    grid_{ins,del}_lines -- <C-e>/<C-y> over a wrapped buffer
--   mbyte     grid_line_puts     -- the UTF-8 compositing loop, the schar
--                                   cache and double-width handling
--   lbr       charsize_regular   -- 'linebreak'+'breakindent'+'showbreak',
--                                   the plines slow path
--   vcol      getvcol/charsize   -- virtcol() over tab-heavy lines, fast path
--   strdw     linetabsize_col    -- strdisplaywidth(), no window involved
--   height    plines_win_nofill  -- nvim_win_text_height over the buffer
--   scrpos    plines_win_col     -- screenpos() per line
--   hlsearch  update_search_hl   -- one 'hlsearch' pattern, per-cell
--   matches   update_search_hl   -- four match-list entries on top of it
--   pum       pum_redraw         -- insert-mode completion, one pum repaint
--                                   per <C-n> over a scrolling 60-item list
--   syntax    syn_current_attr   -- hand-defined keyword/region/matchgroup
--                                   items over a 200-line buffer, resyncing
--                                   on every redraw (B12-8)
--   stl       build_stl_str_hl   -- the statusline item loop: a %{}/%(..%)/%<
--                                   heavy 'statusline', 'rulerformat',
--                                   'winbar' and 'tabline' rebuilt by
--                                   :redrawstatus! (B19-1)
--   winchurn  win_split_ins /    -- split, exchange, rotate, :resize, wincmd =
--             win_close /           and close, back to one window every round,
--             win_equal_rec         under 'equalalways'
--   bufchurn  do_buffer_ext /    -- the whole listed cycle by :bnext, plus one
--             open_buffer /         buffer created, entered and wiped per
--             close_buffer          round, with BufEnter/BufLeave watching
--   termchurn terminal_receive / -- a fixed escape-sequence stream fed into an
--             vterm_input_write /   nvim_open_term terminal, then the refresh
--             refresh_terminal      forced synchronously (B21-2)
--
-- The last three phases drive COMMANDS, not redraws, and a command that quietly
-- does nothing is invisible in a timing number. Each of them therefore runs
-- its round once before the measurement with the effect read back between the
-- stages (`winlayout()` moved, the current buffer cycled, the autocmds
-- fired), and raises rather than reporting a number if it did not. The first
-- draft of `winchurn` was caught by exactly that: `wincmd x` followed by
-- `wincmd r` in the same two-window row is the identity.
--
-- Measured self-A/B noise (same binary twice, min of 5, ~10 s a run):
-- TOTAL within 1%, every phase within +/-3.3% EXCEPT `scroll` at 5.3% --
-- that phase resolves nothing under about 6%.
--
-- Re-measured at B19-1 with 17 phases and the `scroll` fix below
-- (5e23ad6128, CGU-1, 8 rounds, sides swapped, ~40 s a pair):
-- TOTAL -0.3% / +0.1%, worst phase `mbyte` at -4.2 / -0.2%, `stl`
-- +0.5 / +0.6%, and **`scroll` -0.7 / +1.1%** -- its recorded 5.3% was noise
-- on a 17 ms no-op; now that it really scrolls it reads 207 ms and is one of
-- the quieter phases.
--
-- Re-measured with 19 phases at 46c4bf3a3a (CGU-1, 12 rounds, sides swapped)
-- against a DEAD-CODE PADDING build -- the same source plus one never-called
-- `#[inline(never)]` fn. That, not a byte copy, is the honest floor: two
-- functionally identical binaries move more than most real changes do.
-- TOTAL -0.5 / +0.7%; `winchurn` +1.2 / +2.4% and `bufchurn` +2.0 / +0.9%,
-- so neither new phase resolves anything under ~2.5%. The loudest phases are
-- `winsplit` (-2.5 / +7.4), `wrap` (-6.4 / +4.9), `scroll` (-5.2 / +3.1) and
-- `ctl` itself (+3.9 / -4.3) -- read TOTAL and the sign pattern, not one
-- phase.
--
-- `ctl` is `strwidth()` (strings/eval.rs + mbyte/cells.rs) and reaches none
-- of the window/buffer families, so it stays honest here. `opbench`'s `ctl`
-- is `bufnr('%')`, which goes through `tv_get_buf` into buffer.rs, and is
-- therefore NOT a floor for a batch that owns buffer.rs.

local NLINE = 400 -- lines in the wrapped fixture

local function ms(f, rounds)
  local t0 = vim.uv.hrtime()
  for _ = 1, rounds do
    f()
  end
  return (vim.uv.hrtime() - t0) / 1e6
end

local out = {}
local function phase(name, rounds, f)
  out[#out + 1] = string.format('SCRBENCH\t%s\t%.1f', name, ms(f, rounds))
end

-- Deterministic corpus. Long lines so every buffer line wraps several times
-- at 200 columns, tabs so the tabstop arithmetic runs, and a few very long
-- ones so the wrap loop has a tail.
local wrapped = {}
for i = 1, NLINE do
  wrapped[i] = string.format(
    '%s\tid=%04d\t%s',
    string.rep(string.format('word%03d ', i), 30),
    i,
    string.rep('tail ', 12)
  )
end

-- Multibyte: CJK double-width, combining marks and a couple of glyphs long
-- enough to miss the 4-byte inline schar and land in the glyph cache.
local mb = {}
for i = 1, NLINE do
  mb[i] = string.format(
    '%s %s %s %d',
    string.rep('\228\184\173\230\150\135', 12), -- 中文, double width
    string.rep('e\204\129', 20), -- e + combining acute
    string.rep('\240\159\142\137\239\184\143', 6), -- emoji + VS16, long schar
    i
  )
end

local function fill(lines)
  vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
end

vim.o.laststatus = 2
vim.o.ruler = false
vim.o.showcmd = false
vim.o.lazyredraw = false

-- Noise floor: a Vimscript call of comparable cost that reaches neither
-- plines nor grid. Distrust any phase whose swing is not larger than this.
phase('ctl', 200000, function()
  local _ = vim.fn.strwidth('word001 word002 word003 word004 word005')
end)

fill(wrapped)
vim.o.wrap = true
vim.o.number = true
vim.o.list = false

-- Full-clear redraw: every cell of every window line is written, so this is
-- grid_put_linebuf's "everything is dirty" path plus the whole draw stack.
phase('wrap', 120, function()
  vim.cmd('redraw!')
end)

-- Delta redraw: one buffer cell changes, so grid_put_linebuf compares a full
-- screen of cells and writes almost none of them.
local tick = 0
phase('delta', 400, function()
  tick = tick + 1
  vim.api.nvim_buf_set_text(0, 0, 0, 0, 1, { tick % 2 == 0 and 'W' or 'w' })
  vim.cmd('redraw')
end)

vim.o.relativenumber = true
vim.api.nvim_win_set_cursor(0, { 1, 0 })

-- 'relativenumber': every motion rewrites the number column of every visible
-- line, which is a grid_line_puts per row on top of the usual draw.
phase('relnum', 300, function()
  local l = vim.api.nvim_win_get_cursor(0)[1]
  vim.api.nvim_win_set_cursor(0, { l % 100 + 1, 0 })
  vim.cmd('redraw')
end)
vim.o.relativenumber = false

-- Six windows: separators, six independent line batches per redraw, and six
-- window grids' worth of grid_line_start/flush.
vim.cmd('vsplit | vsplit | split | wincmd l | split')
phase('winsplit', 100, function()
  vim.cmd('redraw!')
end)
vim.cmd('only')
fill(wrapped)

-- Scrolling moves whole rows: grid_ins_lines / grid_del_lines shift
-- line_offset[] and the UI gets a scroll event instead of a repaint.
-- B19-1 CORRECTION: this phase used `3\21`, whose comment said 3<C-e>. Lua's
-- `\21` is DECIMAL 21 = CTRL-U, not CTRL-E (5), and at the top of the buffer
-- both 3<C-u> and 3<C-y> fail -- `line('w0')` measured 1 before and after, so
-- the phase drove two rejected normal commands and two redraws of an
-- unchanged screen and reached neither grid_ins_lines nor grid_del_lines.
-- It gated nothing. `\5` really scrolls (w0 1 -> 4 -> 1 per round).
phase('scroll', 2000, function()
  vim.cmd('normal! 3\5') -- 3<C-e>
  vim.cmd('redraw')
  vim.cmd('normal! 3\25') -- 3<C-y>
  vim.cmd('redraw')
end)

-- Multibyte: grid_line_puts decodes each character, interns anything over
-- four bytes in the glyph cache, and doubles up wide cells.
fill(mb)
phase('mbyte', 120, function()
  vim.cmd('redraw!')
end)

-- The plines slow path: 'linebreak', 'breakindent' and 'showbreak' all force
-- charsize_regular instead of the inlined fast one.
fill(wrapped)
vim.o.linebreak = true
vim.o.breakindent = true
vim.o.showbreak = '>> '
phase('lbr', 100, function()
  vim.cmd('redraw!')
end)
vim.o.linebreak = false
vim.o.breakindent = false
vim.o.showbreak = ''

-- virtcol() walks the line through charsize_fast_impl and the tabstop
-- padding, with no screen work at all.
vim.o.tabstop = 7
phase('vcol', 20000, function()
  local _ = vim.fn.virtcol({ 200, 900 })
end)

-- strdisplaywidth() is linetabsize_col on a plain string.
local long = wrapped[7]
phase('strdw', 20000, function()
  local _ = vim.fn.strdisplaywidth(long)
end)

-- nvim_win_text_height over the whole buffer: plines_win_nofill for every
-- line, i.e. linesize_fast end to end.
phase('height', 300, function()
  local _ = vim.api.nvim_win_text_height(0, {})
end)

-- screenpos() reaches plines_win_col, a different driver over the same
-- charsize functions.
phase('scrpos', 40000, function()
  local _ = vim.fn.screenpos(0, 200, 400)
end)

-- B12-6: 'hlsearch' and the match list. `update_search_hl` runs once per
-- CELL of every highlighted line, and nothing above reaches it -- none of the
-- other phases has a search pattern or a match. `hlsearch` is the one-pattern
-- case, `matches` adds four match-list entries with a negative-priority one
-- so the priority walk has something to order.
fill(wrapped)
vim.o.wrap = true
vim.o.number = false
vim.o.hlsearch = true
vim.fn.setreg('/', 'word0')
vim.api.nvim_win_set_cursor(0, { 1, 0 })

phase('hlsearch', 120, function()
  vim.cmd('redraw!')
end)

vim.fn.matchadd('Search', 'id=', 10)
vim.fn.matchadd('ErrorMsg', 'tail', 20)
vim.fn.matchadd('DiffAdd', 'word1', -1)
vim.fn.matchaddpos('Todo', { { 1, 1, 8 }, { 2, 4, 8 }, { 3, 1 } }, 30)

phase('matches', 120, function()
  vim.cmd('redraw!')
end)

-- B12-7: the popup menu. `pum_redraw` writes every cell of every pum row on
-- every selection change and no phase above reaches it (the `wrap`/`delta`
-- redraws have no menu). Insert-mode completion is the driver: one
-- <C-x><C-u> builds the match list, and each following <C-n> re-runs
-- pum_display -> pum_set_selected -> pum_redraw over a 60-item list that is
-- taller than 'pumheight' (so the scrollbar draws) and carries `kind` and
-- `menu` text (so all three columns draw). 'completeopt' deliberately omits
-- preview/popup: those open a window per selection, which is not this path.
-- Measured: with 'pumheight' forced to 1 -- which makes pum_display bail out
-- before pum_redraw -- the phase runs 59% faster, so that is roughly the
-- share of it this canary is actually watching.
local pum_items = {}
for i = 1, 60 do
  pum_items[i] = {
    word = string.format('completion_word_%03d', i),
    kind = (i % 3 == 0) and 'f' or 'v',
    menu = string.format('[src %02d]', i % 7),
  }
end
_G.__scrbench_compl = function(findstart, _)
  return findstart == 1 and 0 or pum_items
end
vim.o.hlsearch = false
vim.fn.clearmatches()
vim.o.completefunc = 'v:lua.__scrbench_compl'
vim.o.completeopt = 'menu,menuone,noselect'
vim.api.nvim_buf_set_lines(0, 0, -1, false, { '' })
local pum_keys = vim.api.nvim_replace_termcodes(
  'i<C-x><C-u>' .. string.rep('<C-n>', 50) .. '<C-e><Esc>',
  true,
  false,
  true
)
phase('pum', 12, function()
  vim.api.nvim_feedkeys(pum_keys, 'nx', false)
end)

-- B12-8: the syntax state machine. `syn_current_attr` runs once per CELL of
-- every highlighted line, and no phase above reaches it at all: none of them
-- has a syntax file loaded, so `syntax_present` is false and drawline skips
-- the whole path. This one defines items by hand rather than sourcing a
-- runtime syntax file, so the phase measures this module and not whatever
-- `runtime/syntax/c.vim` happens to contain: a keyword table (the hash
-- lookup), a nested region with `contains=` (the containment test and the
-- state stack), a `matchgroup=` region (the extra pushed item), a `keepend`
-- pair (check_keepend on every push) and a plain match. `syntax sync
-- minlines=200` keeps every redraw re-syncing rather than loading a cached
-- state, so `syn_sync` is in the measurement too.
vim.o.completefunc = ''
vim.api.nvim_buf_set_lines(0, 0, -1, false, {})
local synlines = {}
for i = 1, 200 do
  synlines[i] = string.format(
    'KEYA line %03d BEG inner KEYB nested <M matched M> tail END trailing KEYC text %03d',
    i,
    i
  )
end
vim.api.nvim_buf_set_lines(0, 0, -1, false, synlines)
vim.cmd('syntax keyword sbKeyA KEYA')
vim.cmd('syntax keyword sbKeyB KEYB nextgroup=sbNext skipwhite')
vim.cmd('syntax keyword sbKeyC KEYC')
vim.cmd('syntax match sbNext /nested/ contained')
vim.cmd('syntax match sbTail /trailing/')
vim.cmd('syntax region sbReg start=/BEG/ end=/END/ keepend contains=sbInner,sbMatch')
vim.cmd('syntax match sbInner /inner/ contained')
vim.cmd('syntax region sbMatch matchgroup=sbDelim start=/<M/ end=/M>/ contained')
vim.cmd('hi link sbKeyA Statement | hi link sbKeyB Type | hi link sbKeyC Identifier')
vim.cmd('hi link sbNext Todo | hi link sbTail Special | hi link sbReg Comment')
vim.cmd('hi link sbInner Constant | hi link sbMatch String | hi link sbDelim Error')
vim.cmd('syntax sync minlines=200')
vim.o.wrap = true
vim.api.nvim_win_set_cursor(0, { 1, 0 })

phase('syntax', 120, function()
  vim.cmd('redraw!')
end)

-- B19-1: the statusline item loop. No phase above reaches `build_stl_str_hl`
-- at all -- scrbench sets 'laststatus' but leaves 'statusline' empty, so
-- `win_redr_status` draws the default line and never enters the item loop.
--
-- `:redrawstatus!` marks every window's status line and runs update_screen
-- with the buffer text still valid, so the only work is the four custom
-- formats: three 'statusline's, three 'winbar's and one ruler, plus the
-- 'tabline' that the following `:redrawtabline` forces (`redraw_tabline` is
-- only set when something changes, so without it the tabline is built once
-- for the whole phase). **Eight `build_stl_str_hl` calls per round**,
-- counted with a `%{}` that increments a Lua global.
--
-- The ruler needs the trick: `redraw_ruler` returns immediately when the
-- window it picks has a status line, and with 'laststatus' 2 every ordinary
-- window has one. It picks `curwin` when `curwin->w_status_height == 0`, so
-- parking the cursor in a FLOATING window (which has no status height, and
-- is not the global-statusline case either) is what lets 'rulerformat' reach
-- `win_redr_custom`. Drop the float and the ruler silently stops being
-- measured.
--
-- The formats exercise groups (`%(..%)`, `%8(..%)`), truncation (`%<`, and
-- the built line is wider than the window so it really cuts), min-width and
-- precision (`%-14.14{}`, `%20.20{}`, `%.30{}`), the re-evaluated
-- `%{%..%}` form, a `%@Func@..%X` click definition (so the click-def arena
-- is allocated every round) and a spread of plain item letters. The `%{}`
-- expressions are cheap Vimscript, not `v:lua`, so the phase weighs the item
-- loop rather than the Lua bridge.
--
-- Measured share: with all four formats replaced by the single character
-- 'x' -- same driver, same redraws, `build_stl_str_hl` still called eight
-- times -- the same 800 rounds run in 11-13 ms against 106-113, so **~89%
-- of the phase is the item loop**, the evaluator re-entry and drawing the
-- built line. Comparing against EMPTY formats measures nothing: the empty
-- case is not a no-op (the default status line and tabline are built by
-- hand, and cost more than this one), and it read *slower* than the heavy
-- one.
--
-- Self-A/B floor at 5e23ad6128, CGU-1, 8 rounds, sides swapped:
-- 98.3 / 99.1 and 98.4 / 98.5 ms, +0.8% and +0.1%.
local stl_fmt = table.concat({
  '%#StatusLine#',
  '%( %{mode()}%h%w%q%r%m %)',
  '%<',
  '%( %.30{repeat("dir/",8)."file.txt"} %)',
  '%( [%{&filetype}]%)',
  '%{%"%#Search#".(winnr()%2==0?"even":"odd")."%#StatusLine#"%}',
  '%( %-14.14{&fileencoding.",".&fileformat} %)',
  '%@ScrbenchClick@%( %{bufnr("%")}:%{line("$")} %)%X',
  '%=',
  '%( %20.20{repeat("pad",9)} %)',
  '%( %{winnr()}/%{winnr("$")} %)',
  '%=',
  '%(%8(%l,%c%V%) %P%)',
  '%( %{&shiftwidth}:%{&tabstop} %)',
}, '')
local ruf_fmt = '%50(%l,%c%V%( %{&fenc}%)%=%P %{winnr()}%)'
local wb_fmt = '%(%{expand("%:t")} %)%<%( %{&ft} %)%=%( %{line(".")} %)'
local tal_fmt =
  '%( %{tabpagenr()}/%{tabpagenr("$")} %)%<%( %{bufname()} %)%=%(%{winnr("$")} wins %)'

vim.cmd('syntax clear')
vim.o.syntax = ''
fill(wrapped)
vim.cmd('vsplit | split')
local stl_float_buf = vim.api.nvim_create_buf(false, true)
vim.api.nvim_buf_set_lines(stl_float_buf, 0, -1, false, { 'float' })
local stl_float_win = vim.api.nvim_open_win(stl_float_buf, true, {
  relative = 'editor',
  row = 2,
  col = 2,
  width = 30,
  height = 5,
})
vim.o.laststatus = 2
vim.o.showtabline = 2
vim.o.ruler = true
vim.o.statusline = stl_fmt
vim.o.rulerformat = ruf_fmt
vim.o.winbar = wb_fmt
vim.o.tabline = tal_fmt

phase('stl', 800, function()
  vim.cmd('redrawstatus!')
  vim.cmd('redrawtabline')
end)

-- Effect guard. A phase whose work is invisible in its own timing can stay a
-- no-op for years -- `scroll` above did, for six batches -- so a phase that
-- drives commands rather than redraws proves its effect ONCE, outside the
-- measurement, and refuses to report a number if the effect is not there.
local function effect(name, ok, detail)
  if not ok then
    error(('scrbench: %s is a no-op -- %s'):format(name, detail))
  end
end

-- Window churn: `win_split_ins`, `win_close` and, because 'equalalways' is
-- on, `win_equal_rec` on every one of them, plus the frame walk behind
-- `wincmd x`/`wincmd r` and `frame_setheight`/`frame_setwidth` behind
-- `:resize`. No phase above manipulates the layout at all: `winsplit` builds
-- six windows ONCE and then only redraws them, so every line of the split,
-- rotate, resize and close code was unmeasured.
--
-- Deliberately no `redraw` in the round: the draw stack already has eight
-- phases of its own above, and folding one in here would bury the layout
-- arithmetic under `update_screen`. Each round starts and ends at exactly one
-- window, so the layout is the same at every round boundary and the work per
-- round is fixed.
--
-- `:only` does NOT close a floating window, so `stl`'s float is closed by id.
vim.api.nvim_win_close(stl_float_win, true)
vim.o.statusline = ''
vim.o.rulerformat = ''
vim.o.winbar = ''
vim.o.tabline = ''
vim.o.ruler = false
vim.o.showtabline = 1
-- Guarded: `:only` on a single window PRINTS "Already only one window", and
-- an unprefixed line on stdout swallows the `SCRBENCH` prefix of the next
-- one, so the harness silently loses a phase.
if vim.fn.winnr('$') > 1 then
  vim.cmd('only')
end
fill(wrapped)
vim.o.equalalways = true
vim.o.eadirection = 'both'
vim.o.winminheight = 1
vim.o.winminwidth = 1
vim.o.winheight = 1
vim.o.winwidth = 10
vim.o.splitbelow = false
vim.o.splitright = false

local function winchurn_split()
  vim.cmd('split')
  vim.cmd('vsplit')
  vim.cmd('wincmd j')
  vim.cmd('vsplit')
end
-- `wincmd x` and `wincmd r` in the SAME two-window row cancel out -- a rotate
-- of two is an exchange -- and the first draft of this phase read
-- `winlayout()` unchanged across both. They are applied to different rows,
-- and `wincmd J` (which is `winframe_remove` + `win_split_ins`, not a
-- swap) moves a window between frames.
local function winchurn_move()
  vim.cmd('wincmd x')
  vim.cmd('wincmd k')
  vim.cmd('wincmd r')
  vim.cmd('wincmd J')
end
local function winchurn_size()
  vim.cmd('resize +4')
  vim.cmd('vertical resize -7')
end
local function winchurn_shut()
  vim.cmd('close')
  vim.cmd('close')
  vim.cmd('close')
end

-- The effect probe: the same round, stage by stage, with what each stage is
-- supposed to move read back between the stages.
do
  local n0 = vim.fn.winnr('$')
  local l0 = vim.inspect(vim.fn.winlayout())
  winchurn_split()
  local n1, l1, h1 = vim.fn.winnr('$'), vim.inspect(vim.fn.winlayout()), vim.fn.winheight(0)
  vim.cmd('wincmd x')
  local lx = vim.inspect(vim.fn.winlayout())
  vim.cmd('wincmd k')
  vim.cmd('wincmd r')
  local lr = vim.inspect(vim.fn.winlayout())
  vim.cmd('wincmd J')
  local l2 = vim.inspect(vim.fn.winlayout())
  effect('winchurn/exchange', lx ~= l1, 'wincmd x exchanged nothing')
  effect('winchurn/rotate', lr ~= lx, 'wincmd r rotated nothing')
  winchurn_size()
  local h2 = vim.fn.winheight(0)
  vim.cmd('wincmd =')
  local h3 = vim.fn.winheight(0)
  winchurn_shut()
  effect(
    'winchurn/split',
    n0 == 1 and n1 == 4 and l1 ~= l0,
    'the splits did not change winlayout()'
  )
  effect('winchurn/move', l2 ~= lr, 'wincmd J moved no window between frames')
  effect('winchurn/size', h2 ~= h1, ':resize +4 changed no height')
  effect('winchurn/equal', h3 ~= h2, 'wincmd = did not re-equalise')
  effect('winchurn/close', vim.fn.winnr('$') == 1, 'the round did not come back to one window')
end

phase('winchurn', 120, function()
  winchurn_split()
  winchurn_move()
  winchurn_size()
  vim.cmd('wincmd =')
  winchurn_shut()
end)

-- Buffer churn: `do_buffer_ext` on every `:bnext`/`:buffer`, the
-- `BufLeave`/`BufEnter` pair each of those fires, `open_buffer` for the
-- buffer the round creates and `close_buffer` for the one it wipes. Nothing
-- above this line touches the buffer list -- every phase so far reuses buffer
-- 1 and one unlisted float scratch buffer -- so `buffer.rs` had no timing
-- coverage anywhere.
--
-- The round walks the WHOLE listed cycle, so it ends where it started and the
-- buffer list is the same size at every round boundary. `BufEnter` writes a
-- Lua counter, which is both the "the autocmd really fires" evidence and a
-- little of the callback cost the real thing pays.
if vim.fn.winnr('$') > 1 then
  vim.cmd('only')
end
vim.o.swapfile = false
vim.o.more = false
vim.opt.shortmess:append('aFIsW')
vim.o.report = 9999

local bufchurn_seen = { enter = 0, leave = 0 }
vim.api.nvim_create_autocmd('BufEnter', {
  callback = function()
    bufchurn_seen.enter = bufchurn_seen.enter + 1
  end,
})
vim.api.nvim_create_autocmd('BufLeave', {
  callback = function()
    bufchurn_seen.leave = bufchurn_seen.leave + 1
  end,
})

local bufchurn_text = {}
for i = 1, 40 do
  bufchurn_text[i] = string.format('buffer line %03d %s', i, string.rep('payload ', 8))
end
for _ = 1, 40 do
  local b = vim.api.nvim_create_buf(true, false)
  vim.api.nvim_buf_set_lines(b, 0, -1, false, bufchurn_text)
end
local bufchurn_home = vim.api.nvim_get_current_buf()
local bufchurn_n = #vim.fn.getbufinfo({ buflisted = 1 })

local function bufchurn_round()
  for _ = 1, bufchurn_n do
    vim.cmd('bnext')
  end
  local b = vim.api.nvim_create_buf(true, false)
  vim.cmd('buffer ' .. b)
  vim.cmd('bwipeout! ' .. b)
  vim.cmd('buffer ' .. bufchurn_home)
end

do
  local e0, l0 = bufchurn_seen.enter, bufchurn_seen.leave
  vim.cmd('bnext')
  effect(
    'bufchurn/bnext',
    vim.api.nvim_get_current_buf() ~= bufchurn_home,
    ':bnext did not change the current buffer'
  )
  effect(
    'bufchurn/autocmd',
    bufchurn_seen.enter > e0 and bufchurn_seen.leave > l0,
    'no BufEnter/BufLeave fired'
  )
  vim.cmd('buffer ' .. bufchurn_home)
  for _ = 1, bufchurn_n do
    vim.cmd('bnext')
  end
  effect(
    'bufchurn/cycle',
    vim.api.nvim_get_current_buf() == bufchurn_home,
    (':bnext x %d did not close the cycle'):format(bufchurn_n)
  )
  local probe = vim.api.nvim_create_buf(true, false)
  vim.cmd('buffer ' .. probe)
  effect(
    'bufchurn/open',
    vim.api.nvim_get_current_buf() == probe,
    ':buffer did not enter the new buffer'
  )
  vim.cmd('bwipeout! ' .. probe)
  effect('bufchurn/wipe', not vim.api.nvim_buf_is_valid(probe), ':bwipeout! left the buffer alive')
  vim.cmd('buffer ' .. bufchurn_home)
end

phase('bufchurn', 40, bufchurn_round)

-- Terminal churn: `terminal_receive` -> `vterm_input_write` -> the vterm
-- parser, state and screen layers, the damage callbacks, `term_sb_push`,
-- `invalidate_terminal`, and then `refresh_terminal`'s mirror of all of it
-- into the buffer (`refresh_scrollback` + `refresh_screen`). Nothing in any
-- of the six benches fed a byte to a terminal before B21-2, though that is a
-- real hot path -- every line a `:terminal` child prints walks it.
--
-- WHY `nvim_open_term` AND NOT `:terminal`. A pty and a child process put
-- process scheduling, a read pump and wall-clock waiting inside the measured
-- round; a bench cannot have any of that. `nvim_open_term` builds the same
-- `Terminal` with the same vterm behind it and `nvim_chan_send` enters at
-- exactly the same `terminal_receive`, in-process and synchronously.
-- `force_crlf = false` is what a pty terminal gets, and it also skips the
-- whole-payload CRLF copy, so the bytes reach `vterm_input_write` unchanged.
--
-- WHY THE 'scrollback' POKE. The refresh half is deferred behind a 10 ms
-- timer (`REFRESH_DELAY`), and a bench must not wait on a clock. The one
-- synchronous door into `refresh_terminal` from script is
-- `did_set_scrollback`, which calls `on_scrollback_option_changed` only when
-- the new value is SMALLER than the old -- hence grow by one, shrink back.
-- Neither step re-sizes the ring (capacity is set once, at the first push),
-- so the pair is a pure "refresh now" lever: ~0.008 ms of option machinery
-- against ~0.1 ms of refresh.
--
-- The terminal is 12 rows in its own split, so each round scrolls ~24 rows
-- off the top: `term_sb_push` and the buffer-side scrollback mirror carry
-- real traffic, and the ring stays capped at 'scrollback' so the round's
-- cost is flat. The payload is built ONCE and sent byte-identically every
-- round -- no formatting, no counter, nothing wall-clock in the round.
local TERMCHURN_SB = 200
vim.cmd('new')
vim.cmd('resize 12')
local termchurn_buf = vim.api.nvim_get_current_buf()
vim.bo[termchurn_buf].scrollback = TERMCHURN_SB
local termchurn_chan = vim.api.nvim_open_term(termchurn_buf, { force_crlf = false })

-- A realistic mixed stream: a coloured build log that scrolls, a scroll
-- region rewritten in place, a cursor-addressed status area, a wide/
-- combining/tab line and a carriage-return progress bar.
local termchurn_payload
do
  local p = {}
  local function add(s)
    p[#p + 1] = s
  end
  add('\027]0;termchurn\007') -- OSC 0, lands in b:term_title
  for i = 1, 20 do
    add(('\027[3%d;1mbuild [%02d]\027[0m ok \027[4mstep\027[0m\r\n'):format(i % 8, i))
  end
  add('\027[5;10r\027[9;1H') -- DECSTBM: scroll inside rows 5-10 only
  for i = 1, 6 do
    add(('region %02d\r\n'):format(i))
  end
  add('\027[r')
  add('\027[H\027[Kstatus idle\027[12;1H\027[K')
  add('wide \228\184\173\230\150\135 comb e\204\129 tab\tend\r\n')
  add('working....\rprogress 100%\r\n')
  add('MARKER\r\n')
  termchurn_payload = table.concat(p)
end

local function termchurn_round()
  vim.api.nvim_chan_send(termchurn_chan, termchurn_payload)
  vim.bo[termchurn_buf].scrollback = TERMCHURN_SB + 1
  vim.bo[termchurn_buf].scrollback = TERMCHURN_SB
end

-- The effect probe. A terminal that parsed nothing, or a refresh that
-- mirrored nothing, would be the fastest possible implementation.
do
  vim.api.nvim_chan_send(termchurn_chan, termchurn_payload)
  local sent = vim.api.nvim_buf_line_count(termchurn_buf)
  vim.bo[termchurn_buf].scrollback = TERMCHURN_SB + 1
  local grown = vim.api.nvim_buf_line_count(termchurn_buf)
  vim.bo[termchurn_buf].scrollback = TERMCHURN_SB
  local shrunk = vim.api.nvim_buf_line_count(termchurn_buf)
  effect(
    'termchurn/deferred',
    sent == grown,
    'the refresh ran unasked -- the poke would be measuring nothing'
  )
  effect('termchurn/refresh', shrunk > grown, 'shrinking scrollback did not run refresh_terminal')
  for _ = 1, 12 do
    termchurn_round()
  end
  local lines = vim.api.nvim_buf_get_lines(termchurn_buf, 0, -1, false)
  local text = table.concat(lines, '\n')
  effect(
    'termchurn/scrollback',
    #lines == TERMCHURN_SB + 12,
    ('the scrollback did not fill: %d lines, expected %d'):format(#lines, TERMCHURN_SB + 12)
  )
  effect(
    'termchurn/title',
    vim.b[termchurn_buf].term_title == 'termchurn',
    'OSC 0 did not reach b:term_title'
  )
  effect(
    'termchurn/log',
    text:find('build [07] ok step', 1, true) ~= nil,
    'the build log is not in the scrollback'
  )
  effect(
    'termchurn/region',
    text:find('region 06', 1, true) ~= nil,
    'the scroll region wrote nothing'
  )
  effect(
    'termchurn/wide',
    text:find('wide \228\184\173\230\150\135 comb e\204\129 tab', 1, true) ~= nil,
    'wide/combining cells did not round-trip'
  )
  effect(
    'termchurn/overwrite',
    text:find('progress 100%', 1, true) ~= nil,
    'the carriage return did not overwrite in place'
  )
  effect(
    'termchurn/tail',
    lines[#lines - 1] == 'MARKER',
    ('the last output line is %q, not MARKER'):format(lines[#lines - 1] or '')
  )
end

phase('termchurn', 600, termchurn_round)

io.stdout:write(table.concat(out, '\n') .. '\n')
io.stdout:flush()
