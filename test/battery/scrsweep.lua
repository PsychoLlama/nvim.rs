-- scrsweep — screen-pipeline differential.
--
-- Drives a child nvim through `test/functional/ui/screen.lua` and dumps, per
-- scenario, the exact cell grid AND the definition of every highlight
-- attribute set that appears in it. The grid alone is NOT an oracle: cells
-- carry attribute *ids* assigned in discovery order, so two different colour
-- tables can render an identical-looking grid. `.attrs` is the artifact that
-- makes attribute changes visible; `.txt` is what a human reads.
--
-- Run through scrsweep.sh, which reproduces run-tests.sh's XDG/TMPDIR sandbox.
-- Args: <outdir> <label>.  NVIM_PRG selects the binary under test.

local root = assert(os.getenv('NVIMRS_ROOT'), 'NVIMRS_ROOT unset')
package.path = table.concat({
  root .. '/?.lua',
  root .. '/?/init.lua',
  root .. '/test/?.lua',
  root .. '/test/?/init.lua',
  root .. '/crates/nvim/src/?.lua',
  root .. '/runtime/lua/?.lua',
}, ';') .. ';' .. package.path

local outdir, label = _G.arg[1], _G.arg[2]
local fixdir = assert(os.getenv('SCRSWEEP_FIXTURES'), 'SCRSWEEP_FIXTURES unset')

local n = require('test.functional.testnvim')()
local Screen = require('test.functional.ui.screen')

local txt, attrs, vals = {}, {}, {}
local function say(t, ...)
  t[#t + 1] = table.concat({ ... }, '')
end

--- Canonical one-line rendering of one highlight-attribute set.
local function attr_repr(a)
  if a == nil then
    return 'nil'
  end
  local keys = {}
  for k in pairs(a) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    local v = a[k]
    if type(v) == 'table' then
      v = attr_repr(v)
    end
    parts[#parts + 1] = ('%s=%s'):format(k, tostring(v))
  end
  return '{' .. table.concat(parts, ' ') .. '}'
end

--- Snapshot the screen: grid rows into .txt, attr definitions into .attrs,
--- any ext_ UI state into .txt as well (it is small and readable).
local function snap(screen, name)
  -- Sample until two consecutive renders agree. A fixed sleep is NOT enough:
  -- `:syntax on` after `:edit` finishes highlighting some milliseconds after
  -- the redraw carrying the unhighlighted text, and a 10 ms snapshot caught
  -- the two states about half the time (the `syntax-runtime-lua` scenario
  -- differed between two runs of the *same* binary).
  --
  -- B12-9: "two consecutive renders agree" is not enough EITHER, because a
  -- render that has not been sent yet agrees with itself. `n.command` returns
  -- when the command is done, not when the redraw it caused has been flushed,
  -- so `syntax-runtime-vim` settled on the *unhighlighted* grid on a slow
  -- machine and on the highlighted one otherwise. Pumping the child's event
  -- loop first makes the first sample the final one.
  n.poke_eventloop()
  local kwargs, ext, prev
  for _ = 1, 40 do
    screen:sleep(15)
    kwargs, ext = screen:get_snapshot()
    if prev == kwargs.grid then
      break
    end
    prev = kwargs.grid
  end
  say(txt, '--- ', name)
  for _, line in ipairs(vim.split(kwargs.grid, '\n')) do
    say(txt, line)
  end
  local ids = kwargs.attr_ids or {}
  local keys = {}
  for k in pairs(ids) do
    keys[#keys + 1] = k
  end
  table.sort(keys, function(a, b)
    return tostring(a) < tostring(b)
  end)
  say(attrs, '--- ', name)
  for _, k in ipairs(keys) do
    say(attrs, ('  [%s] %s'):format(tostring(k), attr_repr(ids[k])))
  end
  -- `float_pos` only ever has content under ext_multigrid (the events that
  -- fill it are not sent otherwise), so adding it leaves every single-grid
  -- scenario byte-identical -- and without it a float that MOVES renders the
  -- same, because its cells live on a grid of their own.
  -- ALL of screen.lua's ext_keys, not the six the first draft dumped
  -- (B13-3).  The missing eight -- cmdline_block, wildmenu_items,
  -- wildmenu_pos, msg_history, showmode, showcmd, ruler, win_pos,
  -- win_viewport_margins -- are exactly the payload the `ext_cmdline` /
  -- `ext_messages` / `ext_wildmenu` scenarios produce, so without this
  -- turning those options on would have dropped most of what they say
  -- and every new scenario would have looked like a pass.  `tabline` is
  -- kept even though it is not in ext_keys: the harness carries it and
  -- one scenario reads it.  Empty tables are skipped, so every existing
  -- single-grid scenario stays byte-identical.
  for _, k in ipairs({
    'popupmenu',
    'cmdline',
    'cmdline_block',
    'wildmenu_items',
    'wildmenu_pos',
    'messages',
    'msg_history',
    'showmode',
    'showcmd',
    'ruler',
    'win_pos',
    'float_pos',
    'win_viewport',
    'win_viewport_margins',
    'tabline',
  }) do
    if ext[k] ~= nil and next(ext[k]) ~= nil then
      say(txt, ('  ext.%s = %s'):format(k, vim.inspect(ext[k], { newline = ' ', indent = '' })))
    end
  end
end

--- `n.feed` plus a synchronous round trip.
---
--- `snap`'s settle loop looks for two consecutive equal renders, and a long
--- typeahead run can pause long enough in the middle to look settled: the
--- 30-key `<C-n>` run in `pum-scroll` snapshotted one row short about one run
--- in four, of the SAME binary. `poke_eventloop` is the test suite's own
--- "process everything queued" round trip. Use this, not `n.feed`, whenever a
--- scenario feeds more than a couple of keys before a snapshot.
local function feed(keys)
  n.feed(keys)
  n.poke_eventloop()
end

--- Canonicalise an error string. A Lua-side error carries a `stack
--- traceback:` tail whose `[C]: at 0x...` frame is an ASLR address, so it
--- differs between two runs of the SAME binary; nothing downstream of the
--- message itself is an oracle.
local function errtext(e)
  local s = tostring(e):gsub('\n', ' ')
  s = s:gsub('%s*stack traceback:.*$', '')
  return (s:gsub('0x%x+', '0xADDR'))
end

--- Record a scalar answer (the value oracle — plines/synID/getmatches have no
--- visible grid of their own).
local function val(name, expr)
  local ok, got = pcall(function()
    return n.api.nvim_eval(expr)
  end)
  say(vals, ('%-22s %-58s %s'):format(name, expr, ok and vim.inspect(got, { newline = ' ', indent = '' }) or ('ERR ' .. errtext(got))))
end

--- Everything the syntax state machine can be asked about one position, in
--- one line: the item id, the id it resolves to through `:hi link`, the whole
--- containment stack, and what `'conceallevel'` would do with the cell.
---
--- `synID`/`synstack`/`synconcealed` answer out of `syn_current_attr`'s
--- current-state stack, not out of the grid, so a containment or transparency
--- regression that happens to leave the *colours* alone is visible here and
--- nowhere else.
local function syn_probe(name, lnum, cols)
  for _, c in ipairs(cols) do
    val(
      ('%s-%d-%d'):format(name, lnum, c),
      ('[synIDattr(synID(%d,%d,1),"name"),synIDattr(synIDtrans(synID(%d,%d,1)),"name"),'
        .. 'map(synstack(%d,%d),\'synIDattr(v:val,"name")\'),synconcealed(%d,%d)]'):format(
        lnum, c, lnum, c, lnum, c, lnum, c
      )
    )
  end
end

--- Record the output of an Ex command that reports (`:syntax list`,
--- `:syntime report`), one line per line, with `mask` applied.
---
--- `sortbody` sorts the run of lines between the header and the first blank
--- one. It exists for exactly one caller and it is a FLAKE FIX, not a
--- preference: `:syntime report` sorts its rows by MEASURED TIME, so when
--- two patterns time equal -- which two patterns over the same sixty lines
--- routinely do -- `syntime-report[2]` and `[3]` swap between two runs of
--- the same binary. It cost a false DIFFERS in a full battery at B19-3.
--- Masking cannot fix it (the digits are already `#`); only removing the
--- ordering can. What is left still pins the column layout, the row count
--- and the pattern text, which is all the report ever asserted.
local function report_val(name, cmd, mask, sortbody)
  local ok, out = pcall(function()
    return n.api.nvim_exec2(cmd, { output = true }).output
  end)
  if not ok then
    say(vals, ('%-22s %-58s %s'):format(name, cmd, 'ERR ' .. errtext(out)))
    return
  end
  local lines = vim.split(out, '\n')
  if mask then
    for i, line in ipairs(lines) do
      lines[i] = mask(line)
    end
  end
  if sortbody then
    local last = 1
    while lines[last + 1] and lines[last + 1]:match('%S') do
      last = last + 1
    end
    if last > 2 then
      local body = {}
      for i = 2, last do
        body[#body + 1] = lines[i]
      end
      table.sort(body)
      for i = 2, last do
        lines[i] = body[i - 1]
      end
    end
  end
  for i, line in ipairs(lines) do
    say(vals, ('%-22s %-58s %s'):format(('%s[%d]'):format(name, i), cmd, line))
  end
end

local function lua_val(name, chunk)
  local ok, got = pcall(function()
    return n.api.nvim_exec_lua('return ' .. chunk, {})
  end)
  say(vals, ('%-22s %-58s %s'):format(name, chunk, ok and vim.inspect(got, { newline = ' ', indent = '' }) or ('ERR ' .. errtext(got))))
end

--- Run an Ex command and record its error text (or `OK`). The error *text* is
--- the only observable half of a rejected `:sign` / `:match`: both commands
--- answer through `emsg` and change nothing, so a diagnosis that moves to a
--- different code path is invisible everywhere else.
local function cmd_val(name, cmd)
  local ok, err = pcall(n.command, cmd)
  say(vals, ('%-22s %-58s %s'):format(name, cmd, ok and 'OK' or errtext(err)))
end

--- Record the `win_extmark` events a `ui_watched` mark produced.
---
--- The grid cannot see these: a UIWatched decoration range reports a position
--- to the UI and draws nothing. It is also the ONLY consumer of the active
--- decoration list that compares a virt-text range against a non-virt-text
--- one, which is what makes it the oracle for the priority *packing* (see the
--- `decor-vtprio` scenario).
---
--- Events accumulate across redraws, so the tuples are deduplicated and
--- sorted: the column each mark reports is the discriminator, the number of
--- times it was reported is redraw-schedule noise.
local function extmark_val(screen, name)
  local seen, rows = {}, {}
  for grid, marks in pairs(screen._grid_win_extmarks or {}) do
    for _, m in ipairs(marks) do
      local key = ('grid=%s win=%s ns=%s id=%s row=%s col=%s'):format(
        tostring(grid), tostring(m[1]), tostring(m[2]), tostring(m[3]), tostring(m[4]),
        tostring(m[5]))
      if not seen[key] then
        seen[key] = true
        rows[#rows + 1] = key
      end
    end
  end
  table.sort(rows)
  say(vals, ('%-22s %-58s %s'):format(name, 'win_extmark events', table.concat(rows, ' | ')))
end

--- Every scenario starts from a fresh child nvim with a deterministic screen.
local function start(w, h, opts)
  n.clear()
  local screen = Screen.new(w or 50, h or 10, opts)
  n.command('set noruler noshowcmd laststatus=1 shortmess+=I')
  return screen
end

local S = {}
local function scenario(name, fn)
  S[#S + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------- grid.rs

scenario('grid-multibyte', function()
  local screen = start(30, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'ascii only',
    '日本語のテキスト',
    'e\u{0301}combining a\u{0300}\u{0316}x',
    'emoji \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466} zwj',
    'tab\tstop\tcols',
    'wide 漢 narrow',
  })
  snap(screen, 'grid-multibyte')
  n.command('set list listchars=tab:>-,eol:$')
  snap(screen, 'grid-multibyte-list')
end)

scenario('grid-arabic', function()
  local screen = start(30, 6)
  n.command('set arabicshape')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'العربية', 'mixed العربية text' })
  snap(screen, 'grid-arabic-on')
  n.command('set noarabicshape')
  snap(screen, 'grid-arabic-off')
end)

scenario('grid-scroll', function()
  local screen = start(24, 8)
  local lines = {}
  for i = 1, 40 do
    lines[i] = ('line %02d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.feed('3<C-e>')
  snap(screen, 'grid-scroll-down')
  n.feed('<C-y>')
  snap(screen, 'grid-scroll-up')
  n.feed('G')
  snap(screen, 'grid-scroll-bottom')
end)

scenario('grid-resize', function()
  local screen = start(40, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'resize me', 'second' })
  screen:try_resize(20, 5)
  snap(screen, 'grid-resize-small')
  screen:try_resize(60, 12)
  snap(screen, 'grid-resize-large')
end)

-- ------------------------------------------------------------ drawline.rs

scenario('draw-wrap', function()
  local screen = start(28, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    string.rep('abcdefghij', 6),
    'short',
    string.rep('x', 27),
    string.rep('y', 28),
    string.rep('z', 29),
  })
  snap(screen, 'draw-wrap-on')
  n.command('set nowrap sidescroll=1')
  n.feed('gg20|')
  snap(screen, 'draw-wrap-off-scrolled')
end)

scenario('draw-listchars', function()
  local screen = start(36, 9)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'a\tb  trail   ',
    '  lead and   multi   space',
    'non\u{00a0}breaking',
    string.rep('w', 50),
  })
  n.command(
    'set list listchars=eol:$,tab:>-,trail:~,extends:>,precedes:<,nbsp:%,lead:_,multispace:.:'
  )
  snap(screen, 'draw-listchars-wrap')
  n.command('set nowrap')
  n.feed('4G30|')
  snap(screen, 'draw-listchars-nowrap')
  -- 'list' WITHOUT a tab listchar. `init_charsize_arg` sets
  -- `use_tabstop = !wo_list || lcs.tab1`, so every combination where
  -- 'listchars' names a tab agrees with every combination where 'list' is
  -- off — an inverted `wo_list` test is invisible until this case exists.
  n.command('set wrap list listchars=eol:$')
  n.feed('gg')
  snap(screen, 'draw-listchars-notab')
  n.command('set listchars=tab:>-,eol:$ nolist')
  snap(screen, 'draw-listchars-nolist')
  -- 'linebreak' + 'list' spells a Tab out into its own buffer rather than
  -- repeating one character, and the buffer is sized from the widths of the
  -- three "tab" characters — in BYTES, though every one of them is a single
  -- cell. A "tab3" that is fewer bytes than "tab2" makes that arithmetic go
  -- negative on the way, and upstream lets `size_t` wrap through it. Caught
  -- by `test_listchars` while this sweep said IDENTICAL, so it is a fixture
  -- gap, not a suite one.
  n.command('set list linebreak listchars=tab:>\u{e9}.,eol:$')
  n.feed('gg')
  snap(screen, 'draw-listchars-lbr-tab3')
  n.command('set listchars=tab:>.\u{e9},eol:$')
  snap(screen, 'draw-listchars-lbr-tab3-wide')
  n.command('set nolinebreak listchars=tab:>-,eol:$ nolist')
end)

scenario('draw-breakindent', function()
  local screen = start(30, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    '    ' .. string.rep('indented text ', 6),
    '\t' .. string.rep('tabbed text ', 5),
  })
  n.command('set breakindent showbreak=++ linebreak')
  snap(screen, 'draw-breakindent')
  n.command('set breakindentopt=shift:4,min:10')
  snap(screen, 'draw-breakindent-shift')
  n.command('set breakindentopt=sbr showbreak=>>')
  snap(screen, 'draw-breakindent-sbr')
end)

scenario('draw-numbercol', function()
  local screen = start(34, 9)
  local lines = {}
  for i = 1, 12 do
    lines[i] = ('row %d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.feed('5G')
  n.command('set number cursorline')
  snap(screen, 'draw-number')
  n.command('set relativenumber numberwidth=6')
  snap(screen, 'draw-relativenumber')
  n.command('set cursorlineopt=number')
  snap(screen, 'draw-cursorline-number')
  n.command('set nonumber norelativenumber cursorlineopt=both cursorcolumn')
  snap(screen, 'draw-cursorcolumn')
end)

scenario('draw-colorcolumn', function()
  local screen = start(34, 7)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { string.rep('c', 30), 'short', '' })
  n.command('set colorcolumn=5,10,+2 textwidth=20')
  snap(screen, 'draw-colorcolumn')
end)

scenario('draw-folds', function()
  local screen = start(36, 12)
  local lines = {}
  for i = 1, 18 do
    lines[i] = ('fold body %02d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set foldmethod=manual foldcolumn=4')
  n.command('3,8fold')
  n.command('12,15fold')
  snap(screen, 'draw-folds-closed')
  n.feed('3Gzo')
  snap(screen, 'draw-folds-open')
  n.command('set fillchars=fold:-,foldopen:v,foldclose:>,foldsep:\\|')
  n.command('set foldtext=getline(v:foldstart)..\\ ##')
  snap(screen, 'draw-folds-fillchars')
end)

scenario('draw-conceal', function()
  local screen = start(34, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'keep <hide> keep',
    'keep <hide> keep',
    'plain line',
  })
  n.command('syntax match Hidden /<hide>/ conceal cchar=X')
  n.command('hi link Hidden Comment')
  for lvl = 0, 3 do
    n.command('set conceallevel=' .. lvl)
    snap(screen, 'draw-conceal-level' .. lvl)
  end
  n.command('set conceallevel=2 concealcursor=n')
  n.feed('gg')
  snap(screen, 'draw-conceal-cursor')
end)

scenario('draw-diff', function()
  local screen = start(46, 12)
  n.command('set diffopt=internal,filler,closeoff')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'same', 'left only', 'shared', 'changed LEFT' })
  n.command('diffthis')
  n.command('vnew')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'same', 'shared', 'changed RIGHT', 'right only' })
  n.command('diffthis')
  snap(screen, 'draw-diff')
end)

scenario('draw-statuscolumn', function()
  local screen = start(38, 9)
  local lines = {}
  for i = 1, 10 do
    lines[i] = ('sc %d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('sign define S1 text=>> texthl=Error')
  n.command('sign place 1 line=3 name=S1 buffer=1')
  n.command('set number relativenumber foldcolumn=2 signcolumn=yes')
  n.command([[set statuscolumn=%s%=%{v:relnum?v:relnum:v:lnum}%C\|]])
  n.feed('4G')
  snap(screen, 'draw-statuscolumn')
end)

scenario('draw-spell', function()
  local screen = start(40, 7)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'correct words here',
    'mispeled wrods heer',
    'lowercase start. lowercase again',
  })
  n.command('set spell spelllang=en_us')
  snap(screen, 'draw-spell')
end)

-- The six scenarios below were added at B12-10, before drawline.rs's column
-- and virtual-text halves were rewritten. Each closes a hole the existing
-- `draw-*` scenarios left; the notes say which arithmetic each one is the
-- only witness for.

-- `fill_foldcolumn`'s depth arithmetic. `draw-folds` has two *sibling* manual
-- folds and a foldcolumn wide enough for them, so `level` never exceeds 1:
-- `first_level` is 1 for every column, `closedcol` equals `level`, and the
-- whole `foldcolumn_sep_char` ladder below its first arm is dead. Nesting
-- plus a narrow column is what makes the digits, the `>` overflow and the
-- `foldinner` fill observable, and `virt_lines` on a fold start is the only
-- way into the `is_virt` outer-level branch.
scenario('draw-foldcol-nested', function()
  local screen = start(40, 14)
  local lines = { 'top of file' }
  for depth = 1, 10 do
    lines[#lines + 1] = ('%sdepth %d'):format(('  '):rep(depth), depth)
  end
  -- A second line at the deepest level: the first one *opens* the fold, so
  -- every column of it takes the `foldopen` marker. Only a line that opens
  -- nothing reaches `foldcolumn_sep_char` for every column, which is where
  -- the `>` overflow past level nine lives.
  lines[#lines + 1] = ('  '):rep(10) .. 'depth 10 again'
  lines[#lines + 1] = '  back to one'
  lines[#lines + 1] = 'top again'
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set foldmethod=indent shiftwidth=2 foldlevel=99')
  -- fdc smaller than the nesting depth: `first_level` is above 1, so the
  -- separator column is a digit, and past nine it is the `>` overflow.
  n.command('set foldcolumn=2')
  snap(screen, 'draw-foldcol-nested-narrow')
  n.command('set foldcolumn=1')
  snap(screen, 'draw-foldcol-nested-one')
  n.command('set foldcolumn=9')
  snap(screen, 'draw-foldcol-nested-wide')
  -- 'foldinner' replaces the digits entirely.
  n.command('set fillchars=foldopen:v,foldclose:>,foldsep:\\|,foldinner:.')
  snap(screen, 'draw-foldcol-nested-inner')
  -- Closed nested folds: `closedcol` = MIN(fdc, level) picks the column the
  -- `foldclose` marker lands in, and `closed` shifts `first_level` by one.
  n.command('set fillchars& foldlevel=1 foldcolumn=3')
  snap(screen, 'draw-foldcol-nested-closed')
  n.command('set foldlevel=4')
  snap(screen, 'draw-foldcol-nested-partly')
  -- A virtual line above a fold start draws the foldcolumn of the line
  -- *above* the fold (the `is_virt` branch), which no other scenario reaches.
  local ns = n.api.nvim_create_namespace('foldvirt')
  n.api.nvim_buf_set_extmark(0, ns, 3, 0, {
    virt_lines = { { { 'virt over fold', 'DiffAdd' } } },
    virt_lines_above = true,
  })
  n.command('set foldlevel=99 foldcolumn=4')
  snap(screen, 'draw-foldcol-nested-virt')
  val('foldcol-level', 'foldlevel(5)')
end)

-- `get_line_number_attr` / `use_cursor_line_nr` / `get_line_number_str` /
-- `draw_lnum_col`. `draw-numbercol` sets 'relativenumber' but never gives
-- LineNrAbove and LineNrBelow colours of their own — they link to LineNr by
-- default, so swapping the two arms of `get_line_number_attr` renders
-- identically. It also never sets 'cpoptions' "n", 'rightleft' or
-- 'signcolumn'=number, each of which is a separate branch of `draw_lnum_col`.
scenario('draw-lnum-hl', function()
  local screen = start(44, 12)
  local lines = {}
  for i = 1, 14 do
    lines[i] = ('lnum body %02d'):format(i)
  end
  lines[3] = 'lnum body 03 ' .. ('wrapped '):rep(9)
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('hi LineNr guifg=#00ff00 ctermfg=10')
  n.command('hi LineNrAbove guifg=#ff0000 ctermfg=9')
  n.command('hi LineNrBelow guifg=#0000ff ctermfg=12')
  n.command('hi CursorLineNr guifg=#ffff00 ctermfg=11 gui=bold cterm=bold')
  n.command('set number relativenumber cursorline')
  n.feed('5G')
  snap(screen, 'draw-lnum-hl-rnu')
  -- 'number' + 'relativenumber' on the cursor line uses the left-aligned
  -- format; every other line uses the right-aligned one.
  n.command('set nornu')
  snap(screen, 'draw-lnum-hl-nu')
  -- A wrapped line with 'cpoptions' "n": no number column on the continuation
  -- rows at all, which is a different branch from "blank number column".
  n.command('set rnu cpoptions+=n')
  n.feed('3G')
  snap(screen, 'draw-lnum-hl-cpo-n')
  n.command('set cpoptions-=n')
  snap(screen, 'draw-lnum-hl-cpo-none')
  -- 'cursorlineopt' decides whether the continuation rows of the cursor line
  -- keep the CursorLineNr attribute.
  for _, opt in ipairs({ 'number', 'line', 'both', 'screenline' }) do
    n.command('set cursorlineopt=' .. opt)
    snap(screen, 'draw-lnum-hl-culopt-' .. opt)
  end
  n.command('set cursorlineopt=both')
  -- 'signcolumn'=number puts the sign *in* the number column.
  n.command('sign define LN text=@@ texthl=Error numhl=Search')
  n.command('sign place 7 line=6 name=LN buffer=1')
  n.command('set signcolumn=number')
  n.feed('6G')
  snap(screen, 'draw-lnum-hl-sclnum')
  n.command('set signcolumn=auto')
  snap(screen, 'draw-lnum-hl-numhl')
  -- Reversed line numbers.
  n.command('set rightleft numberwidth=6')
  snap(screen, 'draw-lnum-hl-rl')
  n.command('set norightleft numberwidth=4')
  -- A virtual line belonging to the line *above* takes that line's sign
  -- numhl, which is `get_line_number_attr`'s `prev_num_attr` branch.
  local ns = n.api.nvim_create_namespace('lnumvirt')
  n.api.nvim_buf_set_extmark(0, ns, 6, 0, {
    virt_lines = { { { 'below six', 'DiffAdd' } } },
  })
  n.api.nvim_buf_set_extmark(0, ns, 8, 0, {
    virt_lines = { { { 'above nine', 'DiffChange' } } },
    virt_lines_above = true,
  })
  snap(screen, 'draw-lnum-hl-virtnum')
  val('lnum-numwidth', 'strwidth(printf("%*d ", &numberwidth, 14))')
end)

-- 'smoothscroll' is the only way to get `w_skipcol > 0`, which four separate
-- places in the column code test: `draw_lnum_col` replaces the leading blanks
-- of the number with `-`, `handle_breakindent` and
-- `handle_showbreak_and_filler` each decide whether 'showbreak' still applies
-- to the first visible row, and 'breakindentopt' "sbr" flips which of the two
-- clears `need_showbreak`. Nothing in the corpus set it.
scenario('draw-smoothscroll', function()
  local screen = start(36, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    string.rep('long wrapped body ', 8),
    'after the long line',
    '    indented ' .. string.rep('tail ', 12),
    'last line',
  })
  n.command('set wrap smoothscroll number showbreak=++ list listchars=eol:$')
  n.feed('<C-e>')
  snap(screen, 'draw-smoothscroll-one')
  n.feed('<C-e>')
  snap(screen, 'draw-smoothscroll-two')
  n.command('set breakindent breakindentopt=sbr')
  snap(screen, 'draw-smoothscroll-bri-sbr')
  n.command('set breakindentopt=shift:2')
  snap(screen, 'draw-smoothscroll-bri-shift')
  n.command('set nonumber')
  snap(screen, 'draw-smoothscroll-nonu')
  -- 'number' + 'relativenumber' is the one combination whose number column is
  -- still drawn on a partially scrolled first row, and `draw_lnum_col` then
  -- replaces its leading blanks with `-`.
  n.command('set number relativenumber breakindentopt=')
  snap(screen, 'draw-smoothscroll-dashes')
  -- On the cursor line 'number'+'relativenumber' uses the left-aligned
  -- format, which has no leading blank to replace; the dashes only appear
  -- once the cursor is somewhere else.
  n.feed('3G')
  n.poke_eventloop()
  snap(screen, 'draw-smoothscroll-dashes-off')
  val('smoothscroll-skipcol', 'winsaveview().skipcol')
end)

-- `advance_color_col` is reached from three places, and `draw-colorcolumn`
-- only reaches the one in the character loop: the calls inside `draw_col_buf`
-- (which runs with `inc_vcol` only for 'showbreak') and inside
-- `handle_breakindent` need a colorcolumn that falls inside the 'showbreak' /
-- 'breakindent' padding of a wrapped line. `get_rightmost_vcol` combines the
-- colorcolumn list with 'cursorcolumn', so both are set here.
scenario('draw-colorcolumn-wrap', function()
  local screen = start(30, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    '        ' .. string.rep('wrapping colorcolumn body ', 3),
    'short',
    string.rep('x', 60),
  })
  n.command('set wrap number showbreak=>>>> breakindent')
  n.command('set colorcolumn=3,10,29,31,34,40 cursorcolumn textwidth=20')
  n.feed('gg')
  snap(screen, 'draw-colorcolumn-wrap')
  -- The cursor further right moves `w_virtcol`, so `get_rightmost_vcol`
  -- answers from 'cursorcolumn' rather than from the list.
  n.feed('3G$')
  snap(screen, 'draw-colorcolumn-wrap-far')
  n.command('set colorcolumn=+1,+3,-2')
  n.feed('gg')
  snap(screen, 'draw-colorcolumn-relative')
  n.command('set nocursorcolumn colorcolumn=1')
  snap(screen, 'draw-colorcolumn-first')
  val('colorcolumn-opt', '&colorcolumn')
end)

-- `draw_statuscol`'s error, growth and segment paths. `draw-statuscolumn`
-- draws one valid statuscolumn once; it never makes the expression fail (the
-- reset branch), never makes the built string outgrow `stcp->width` (the
-- `w_nrwidth` growth branch), never gives a segment a user highlight, and
-- never puts one on a filler or virtual line, where `v:virtnum` is non-zero
-- and the reported `v:lnum` belongs to the line above.
scenario('draw-statuscol-more', function()
  local screen = start(46, 12)
  local lines = {}
  for i = 1, 12 do
    lines[i] = ('stc %d'):format(i)
  end
  lines[4] = 'stc 4 ' .. string.rep('wrap ', 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set number relativenumber foldcolumn=3 signcolumn=yes wrap')
  n.command('sign define SC text=%% texthl=Error numhl=Todo')
  n.command('sign place 3 line=2 name=SC buffer=1')
  n.command('3,6fold')
  n.command('4,5fold')
  local ns = n.api.nvim_create_namespace('stc')
  n.api.nvim_buf_set_extmark(0, ns, 7, 0, {
    virt_lines = { { { 'stc virt below', 'DiffAdd' } } },
  })
  n.api.nvim_buf_set_extmark(0, ns, 9, 0, {
    virt_lines = { { { 'stc virt above', 'DiffChange' } } },
    virt_lines_above = true,
  })
  -- A user highlight on one segment, the sign segment and the fold segment.
  -- SignColumn and FoldColumn render identically in the default colorscheme,
  -- so which of the two `draw_statuscol` gives a segment is invisible until
  -- they are told apart here.
  n.command('hi User1 guifg=#ff00ff ctermfg=13')
  n.command('hi SignColumn guifg=#00ffff guibg=#003333 ctermfg=14 ctermbg=23')
  -- FoldColumn deliberately sets a foreground ONLY: `draw_statuscol` gives
  -- the `%C` segment no base attribute at all and lets the segment's own
  -- group supply everything, so a base attribute wrongly combined underneath
  -- is only visible where the group leaves something unset.
  n.command('hi clear FoldColumn')
  n.command('hi FoldColumn guifg=#ff8800 ctermfg=3')
  n.command([[set statuscolumn=%C%s%1*%l%*\|%{v:virtnum}]])
  n.feed('2G')
  snap(screen, 'draw-statuscol-segments')
  -- An expression that fails: 'statuscolumn' is reset and the number column
  -- goes back to `number_width`.
  n.command([[set statuscolumn=%{undefined_function()}]])
  snap(screen, 'draw-statuscol-error')
  val('statuscol-after-error', '&statuscolumn')
  -- A statuscolumn wider than the number column it was sized for: the
  -- `w_nrwidth` growth path, then the truncation guard at MAX_STCWIDTH.
  n.command([=[set statuscolumn=[%l\ %{repeat('=',v:lnum)}]]=])
  snap(screen, 'draw-statuscol-grow')
  val('statuscol-grow-textoff', 'getwininfo(win_getid())[0].textoff')
  n.command([[set statuscolumn=%{repeat('#',99)}%l]])
  snap(screen, 'draw-statuscol-max')
  n.command('set statuscolumn=')
  snap(screen, 'draw-statuscol-off')
end)

-- `draw_virt_text` places every alignment except "eol" and "inline" through a
-- branch of its own, and `draw_virt_text_item` has a whole skip-cells and
-- blend half. `decor-extmarks` uses only eol/inline/overlay at the default
-- hl_mode with the window wide enough for all of them, so the right-align
-- accumulator, the win_col clamp, the blend/combine modes, a Tab inside a
-- virtual text and the "text starts left of the window" skip are all
-- unwitnessed.
scenario('draw-virttext-align', function()
  local screen = start(40, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'align one',
    'align two',
    'align three',
    'align four',
    'align five',
    string.rep('scrolled body ', 6),
  })
  local ns = n.api.nvim_create_namespace('align')
  n.command('hi VtA guifg=#ff0000 guibg=#002200 ctermfg=9 ctermbg=22')
  n.command('hi VtB guifg=#00ff00 guibg=#220000 ctermfg=10 ctermbg=52')
  -- Two right-aligned-at-eol texts on one line: the only shape that runs the
  -- look-ahead loop that sums their widths (and the `- 1` for the last one).
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    virt_text = { { 'RA1', 'VtA' } },
    virt_text_pos = 'eol_right_align',
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    virt_text = { { 'RA2', 'VtB' } },
    virt_text_pos = 'eol_right_align',
  })
  -- Window-right-aligned, which walks `right_pos` leftwards per item.
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, {
    virt_text = { { 'right-one', 'VtA' } },
    virt_text_pos = 'right_align',
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, {
    virt_text = { { 'right-two', 'VtB' } },
    virt_text_pos = 'right_align',
  })
  -- 'win_col' positions: one inside the window, one past its right edge (the
  -- out-of-window clamp) and one that would land left of column zero.
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    virt_text = { { 'WC', 'VtA' } },
    virt_text_win_col = 20,
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    virt_text = { { 'OFF', 'VtB' } },
    virt_text_win_col = 90,
  })
  -- The three hl_modes over an existing highlight, plus a Tab and a
  -- double-width character inside the virtual text.
  n.api.nvim_buf_set_extmark(0, ns, 3, 0, { end_col = 10, hl_group = 'Search' })
  n.api.nvim_buf_set_extmark(0, ns, 3, 2, {
    virt_text = { { 'co\tmb', 'VtA' } },
    virt_text_pos = 'overlay',
    hl_mode = 'combine',
  })
  n.api.nvim_buf_set_extmark(0, ns, 4, 0, { end_col = 10, hl_group = 'Search' })
  n.api.nvim_buf_set_extmark(0, ns, 4, 2, {
    virt_text = { { ' bl 漢 ', 'VtB' } },
    virt_text_pos = 'overlay',
    hl_mode = 'blend',
  })
  snap(screen, 'draw-virttext-align')
  -- Inline virtual text that starts left of the first visible column: the
  -- `skip_cells` half of `handle_inline_virtual_text`, including a chunk
  -- dropped whole and one cut in the middle of a double-width character.
  n.api.nvim_buf_set_extmark(0, ns, 5, 4, {
    virt_text = { { 'INLINE-漢-TEXT', 'VtA' }, { '/second', 'VtB' } },
    virt_text_pos = 'inline',
  })
  -- 'colorcolumn' and 'cursorcolumn' are what make `wlv.vcol` observable
  -- after the virtual text: the cells the skip accounting mis-numbers are
  -- exactly the ones that decide where the column highlights land. Without
  -- them a one-cell error in `skipped_cells` renders identically.
  n.command('set nowrap sidescroll=1 cursorcolumn colorcolumn=20,30,40,50')
  n.feed('6G')
  for _, col in ipairs({ '1|', '26|', '44|', '48|', '52|', '56|', '60|', '84|' }) do
    n.feed(col)
    n.poke_eventloop()
    snap(screen, 'draw-virttext-inline-skip-' .. col:sub(1, -2))
    val('inline-skip-vcol-' .. col:sub(1, -2), '[virtcol("."), winsaveview().leftcol]')
    lua_val(
      'inline-skip-pos-' .. col:sub(1, -2),
      'vim.fn.screenpos(0, 6, vim.fn.col("."))'
    )
  end
  n.command('set wrap')
  n.feed('gg')
  snap(screen, 'draw-virttext-align-wrap')
  lua_val('virttext-marks', 'vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})')
end)

-- ---------------------------------------------------------- drawscreen.rs

-- The Visual and 'incsearch' inverted ranges (B12-11). `win_line`'s prologue
-- spends ~120 lines of upstream C working `fromcol`/`tocol` out and NOTHING in
-- the corpus entered Visual mode or reached `highlight_match` before this:
-- every mutation of that arithmetic was unkillable.
scenario('draw-visual', function()
  local screen = start(44, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'first line of text',
    '',
    'third line is a good deal longer than the rest of them',
    'fourth',
    'fifth line ends here',
  })
  n.command('set selection=inclusive')
  feed('gg0lllvjjll')
  snap(screen, 'draw-visual-charwise')
  -- The selection *starts* on an empty line: `gchar_pos(top) == NUL` is the
  -- only way to reach `tocol = fromcol + 1`. 'colorcolumn' is what makes the
  -- value observable: without something that wants drawing past the end of
  -- the line, the loop stops one cell after it and any `tocol` beyond that
  -- renders identically.
  n.command('set colorcolumn=10')
  feed('<Esc>2Gv2jll')
  snap(screen, 'draw-visual-emptystart')
  n.command('set colorcolumn=')
  feed('<Esc>ggVjj')
  snap(screen, 'draw-visual-linewise')
  -- Blockwise takes `w_old_cursor_{f,l}col` instead, and `$` puts MAXCOL in
  -- the second of them.
  feed('<Esc>gg0<C-v>3jlll')
  snap(screen, 'draw-visual-blockwise')
  feed('<Esc>gg0<C-v>3j$')
  snap(screen, 'draw-visual-block-dollar')
  -- 'selection' "exclusive" has two special cases of its own: the end of the
  -- selection sitting at column 0 of a line means none of that line is in it,
  -- and `getvvcol` is asked for the start rather than the end column.
  n.command('set selection=exclusive')
  feed('<Esc>ggvjjll')
  snap(screen, 'draw-visual-exclusive')
  feed('<Esc>ggv3j0')
  snap(screen, 'draw-visual-excl-col0')
  n.command('set selection=inclusive')
  -- Scrolled sideways: the inverted range starts left of the first drawn
  -- column, which is the `fromcol < vcol` adjustment in the skip loop.
  n.command('set nowrap')
  feed('<Esc>3Gv$')
  feed('20zl')
  snap(screen, 'draw-visual-leftcol')
  n.command('set wrap')
  feed('<Esc>')
  val('visual-getpos-start', "getpos(\"'<\")")
  val('visual-getpos-end', "getpos(\"'>\")")
end)

-- Virtual lines above the *top* line, partly scrolled into. `w_topfill` is
-- the only thing that clamps `n_virt_lines`, and nothing in the corpus
-- reached that branch with virtual lines before (B12-11).
scenario('draw-virtlines-top', function()
  local screen = start(36, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight',
  })
  local ns = n.api.nvim_create_namespace('vltop')
  n.api.nvim_buf_set_extmark(0, ns, 3, 0, {
    virt_lines = { { { 'VL-a', 'Search' } }, { { 'VL-b', 'Search' } }, { { 'VL-c', 'Search' } } },
    virt_lines_above = true,
  })
  snap(screen, 'draw-virtlines-top-all')
  for i = 1, 5 do
    feed('<C-e>')
    snap(screen, 'draw-virtlines-top-' .. i)
  end
  val('virtlines-top-w0', "line('w0')")
  val('virtlines-top-height', 'nvim_win_text_height(0, {})')
end)

scenario('draw-incsearch', function()
  local screen = start(44, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alpha bravo charlie',
    'delta echo foxtrot',
    'golf hotel india',
    'juliett kilo lima',
  })
  n.command('set incsearch hlsearch')
  -- Typing a search: `highlight_match` with the match on the cursor line.
  feed('gg0/hotel')
  snap(screen, 'draw-incsearch-typing')
  feed('<Esc>')
  -- A match spanning two lines: the second line takes `fromcol = 0` and the
  -- `lnum == cursor.lnum + search_match_lines` end column.
  feed('gg0/foxtrot\\ngolf')
  snap(screen, 'draw-incsearch-multiline')
  feed('<Esc>')
  -- `:s///c` sets the same state from the other direction and leaves the
  -- confirm prompt up.
  feed(':%s/o/O/gc<CR>')
  snap(screen, 'draw-incsearch-confirm')
  feed('q')
  snap(screen, 'draw-incsearch-after')
end)

-- The six scenarios below were added at B12-12, before win_line's character
-- loop was rewritten. Each closes a hole the B12-11 handoff named: the corpus
-- had no 'rightleft' beyond the number column, no 'conceallevel' crossed with
-- inline virtual text, no 'diffopt' linematch/inline (so the loop's
-- `change_index` never advanced past the first change), no 'virtualedit' (the
-- only thing that draws past the end of a line, and what makes the empty-line
-- `tocol` observable), no spell scene that reaches the loop's own
-- `spell_check` bookkeeping, and no full redraw at a scroll position where
-- 'topfill' is partway through a run of virtual lines.

-- The whole 'rightleft' half of the loop. `draw-lnum-hl` sets 'rightleft'
-- for the number column only; nothing reached `linebuf_mirror` with text in
-- the buffer, and nothing at all reached the two `rl_mirror_ascii` calls,
-- which need a non-printable character AND 'rightleft' (and, for the second,
-- 'display' "uhex").
scenario('draw-rightleft', function()
  local screen = start(36, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'plain ascii line',
    'wide ' .. '\u{6f22}\u{5b57}' .. ' mixed ascii',
    'tab\tafter\ttabs',
    'ctrl ' .. string.char(1) .. ' and ' .. string.char(255) .. ' illegal',
    string.rep('long wrapped rightleft text ', 3),
    '   lead and trail   ',
  })
  n.command('set rightleft')
  snap(screen, 'draw-rightleft-plain')
  n.command('set number list listchars=eol:$,tab:>-,trail:~,precedes:<,extends:>')
  snap(screen, 'draw-rightleft-list')
  -- 'display' "uhex" plus 'rightleft' is the only way into the
  -- `rl_mirror_ascii` beside `transchar_buf`; the illegal byte reaches the
  -- one beside `transchar_hex` either way.
  n.command('set display=uhex')
  snap(screen, 'draw-rightleft-uhex')
  -- Without 'wrap' the mirrored line is also horizontally scrolled, which is
  -- where "precedes"/"extends" swap sides.
  n.command('set display= nowrap sidescroll=1')
  feed('5G$')
  snap(screen, 'draw-rightleft-nowrap-end')
  feed('5G40|')
  snap(screen, 'draw-rightleft-nowrap-mid')
  n.command('set nolist')
  n.command('set wrap cursorline colorcolumn=8 cursorcolumn conceallevel=2')
  n.command('syntax match rlHide /ascii/ conceal cchar=#')
  n.command('hi link rlHide Comment')
  feed('gg')
  snap(screen, 'draw-rightleft-conceal')
  val('rightleft-virtcol', 'virtcol("$")')
  val('rightleft-screenpos', 'screenpos(0, 2, 8)')
  val('rightleft-screencol', 'screencol()')
end)

-- 'conceallevel' crossed with inline virtual text. The two feed the same
-- `wlv.n_extra`/`wlv.skip_cells` pair from opposite ends and no scenario put
-- them on the same cells: `handle_inline_virtual_text` runs while
-- `skip_cells` is counting concealed columns, and the conceal branch's
-- `boguscols` bookkeeping has to survive it.
scenario('draw-conceal-virt', function()
  local screen = start(40, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'aaa <hide> bbb <hide> ccc tail',
    'wide ' .. '\u{6f22}\u{5b57}\u{6f22}' .. ' hidden here',
    'decor <gone> range here',
    'plain tail line',
  })
  local ns = n.api.nvim_create_namespace('cv')
  -- Inline texts just before, inside and just after the first concealed run.
  n.api.nvim_buf_set_extmark(0, ns, 0, 4, {
    virt_text = { { '[A]', 'Todo' } }, virt_text_pos = 'inline',
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 7, {
    virt_text = { { '[B]', 'Search' } }, virt_text_pos = 'inline',
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 10, {
    virt_text = { { '[C]', 'DiffAdd' } }, virt_text_pos = 'inline',
  })
  -- A decoration conceal with a replacement character and a highlight of its
  -- own: `decor_state.conceal_char` / `conceal_attr`, which is a different
  -- arm from the syntax `cchar`.
  n.api.nvim_buf_set_extmark(0, ns, 2, 6, {
    end_col = 12, conceal = '#', hl_group = 'DiffText',
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 8, {
    virt_text = { { '<i>', 'Error' } }, virt_text_pos = 'inline',
  })
  -- Concealing a double-width character: the loop's `schar_cells(mb_schar)
  -- > 1` arm adds a virtual column, and `concealed_wide` adds a bogus one.
  n.command('syntax match CvHide /<hide>/ conceal cchar=X')
  n.command('syntax match CvWide /\u{6f22}\u{5b57}/ conceal cchar=W')
  n.command('hi link CvHide Comment')
  n.command('hi link CvWide Statement')
  -- 'colorcolumn' and 'cursorcolumn' are what make a one-cell vcol error
  -- show: concealed cells have no rendering of their own.
  n.command('set colorcolumn=12,20 cursorcolumn')
  for lvl = 0, 3 do
    n.command('set conceallevel=' .. lvl)
    snap(screen, 'draw-conceal-virt-level' .. lvl)
  end
  n.command('set conceallevel=2 concealcursor=nv')
  feed('gg')
  snap(screen, 'draw-conceal-virt-cursor')
  -- Narrow enough that the concealed run wraps: the fake columns exist only
  -- so that a wrapped line takes the same screen space.
  screen:try_resize(20, 12)
  snap(screen, 'draw-conceal-virt-wrap')
  screen:try_resize(40, 12)
  val('conceal-virt-virtcol', 'virtcol("$")')
  val('conceal-virt-height', 'nvim_win_text_height(0, {})')
  val('conceal-virt-screenpos', 'screenpos(0, 1, 20)')
end)

-- 'diffopt' "linematch" and "inline:" are the only settings that give one
-- line more than one changed range, so `line_changes.num_changes > 1` and
-- the loop's `change_index += 1` step was dead. Inline virtual text on a
-- changed line is the other half: extra text takes the *line's* diff
-- highlight, never the changed-text one.
scenario('draw-diff-linematch', function()
  local screen = start(60, 14)
  n.command('set diffopt=internal,filler,closeoff,inline:word,linematch:60')
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'common head',
    'alpha bravo charlie delta echo',
    'one two three four five six',
    'removed left only',
    'common tail',
  })
  n.command('diffthis')
  n.command('vnew')
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'common head',
    'alpha BRAVO charlie DELTA echo',
    'one two THREE four FIVE six',
    'common tail',
  })
  n.command('diffthis')
  snap(screen, 'draw-diff-linematch-word')
  n.command('set diffopt-=inline:word diffopt+=inline:char')
  n.command('diffupdate')
  snap(screen, 'draw-diff-linematch-char')
  local ns = n.api.nvim_create_namespace('dl')
  n.api.nvim_buf_set_extmark(0, ns, 1, 8, {
    virt_text = { { '{vt}', 'Todo' } }, virt_text_pos = 'inline',
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    virt_text = { { '(eol)', 'Todo' } },
  })
  n.command('redraw!')
  snap(screen, 'draw-diff-linematch-virt')
  n.command('set cursorline number list listchars=eol:$,trail:~')
  n.command('redraw!')
  snap(screen, 'draw-diff-linematch-cul')
  n.command('set diffopt=internal,filler,closeoff')
  n.command('diffupdate')
  n.command('redraw!')
  snap(screen, 'draw-diff-linematch-off')
end)

-- 'virtualedit'. Nothing in the corpus set it, so the loop's only branch
-- that draws a cell *past* the end of a line — the blockwise/charwise
-- selection extending beyond it — was dead, and so was the `did_wcol`
-- "cursor beyond end of the line" correction.
scenario('draw-virtualedit', function()
  local screen = start(40, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'short',
    '',
    'a somewhat longer line here',
    'tab\there',
    'end',
  })
  n.command('set virtualedit=all')
  feed('gg0<C-v>3j12l')
  snap(screen, 'draw-virtualedit-block')
  feed('<Esc>2Gv2j10l')
  snap(screen, 'draw-virtualedit-charwise')
  n.command('set list listchars=eol:$,tab:>-')
  snap(screen, 'draw-virtualedit-list')
  -- 'listchars' "eol" set while 'list' is OFF: the eol cell is then drawn
  -- only because the selection runs past the end of the line, which is the
  -- one spelling where the `'list'` disjunct beside that test is false.
  n.command('set nolist listchars=eol:$,tab:>-')
  snap(screen, 'draw-virtualedit-nolist-eol')
  feed('<Esc>gg0<C-v>3j12l')
  snap(screen, 'draw-virtualedit-nolist-block')
  n.command('set listchars&')
  feed('<Esc>')
  n.command('set virtualedit=block')
  feed('gg0<C-v>4j20l')
  snap(screen, 'draw-virtualedit-blockonly')
  feed('<Esc>')
  -- Past the end of a line the cursor column has to be worked out at the
  -- end of the line rather than when the loop reaches it; concealment is
  -- what makes `w_wcol` observable.
  n.command('set virtualedit=all conceallevel=2 concealcursor=nv')
  n.command('syntax match VeHide /somewhat/ conceal cchar=~')
  n.command('hi link VeHide Comment')
  feed('3G40|')
  snap(screen, 'draw-virtualedit-conceal')
  val('ve-screencol', 'screencol()')
  val('ve-getcurpos', 'getcurpos()')
  val('ve-virtcol', 'virtcol(".")')
  feed('1G30|')
  val('ve-screencol-short', 'screencol()')
  val('ve-screenpos-short', 'screenpos(0, 1, 1)')
end)

-- The loop's own spell bookkeeping. `draw-spell` is a single snapshot of
-- three unwrapped lines: it never joins a word across the line break (the
-- `nextline` buffer and `spv_checked_lnum`/`spv_checked_col`), never sets
-- the capital column for the following line, never runs in Insert mode
-- beside the cursor, never turns `can_spell` off through 'spelloptions' or a
-- decoration, and never scrolls a line sideways, which is the only thing
-- that makes the setup half call `spell_move_to` — and with it the
-- `_on_spell_nav` providers.
scenario('draw-spell-more', function()
  local screen = start(40, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'a correct sentence goes here.',
    'lowercase after a full stop here',
    'this line ends with a split misspel',
    'led word and then more text after it.',
    'goed morgen und ALLCAPS wrods here',
    'plain trailing line with a wrods far off to the right of the window edge',
  })
  n.command('set spell spelllang=en_us')
  -- Lines 3/4 split "misspelled" across the line break: the setup half joins
  -- the head of line 4 onto line 3 in `nextline`, and the loop records that
  -- the word continues so line 4 does not re-check it. Without the join,
  -- "misspel" and "led" are both bad words.
  snap(screen, 'draw-spell-more-plain')
  -- In Insert mode a bad word the cursor is touching is left alone.
  feed('5G0ivery wrods')
  snap(screen, 'draw-spell-more-insert')
  feed('<Esc>')
  snap(screen, 'draw-spell-more-afterinsert')
  -- A decoration overrides `can_spell` either way (the `spell` tristate),
  -- and it has to be tested before any syntax item exists or the syntax
  -- half of the same decision hides it.
  local ns = n.api.nvim_create_namespace('sp')
  n.api.nvim_buf_set_extmark(0, ns, 4, 0, { end_col = 24, spell = false })
  n.command('redraw!')
  snap(screen, 'draw-spell-more-decor')
  n.api.nvim_buf_clear_namespace(0, ns, 0, -1)
  -- 'spelloptions' "noplainbuffer" turns it off unless a syntax item puts it
  -- back through @Spell.
  n.command('syntax match spOk /goed/ contains=@NoSpell')
  n.command('syntax match spBad /ALLCAPS/ contains=@Spell')
  n.command('set spelloptions=noplainbuffer')
  n.command('redraw!')
  snap(screen, 'draw-spell-more-noplain')
  n.command('set spelloptions= | syntax clear')
  n.command('redraw!')
  -- Scrolled sideways with 'nowrap': the setup half has to find whether the
  -- first drawn column is inside a badly spelled word, which is the only
  -- caller of `spell_move_to` in the drawing path.
  -- 29 `zl`s put the left edge inside "wrods" (columns 28..32), which is
  -- the case `spell_move_to` exists for: the first drawn column is in the
  -- middle of a badly spelled word the loop never saw start.
  n.command('set nowrap sidescroll=1')
  feed('6G0')
  feed('29zl')
  snap(screen, 'draw-spell-more-nowrap-mid')
  feed('6G$')
  snap(screen, 'draw-spell-more-nowrap-end')
  n.command('set wrap')
  val('spell-more-badword', 'spellbadword("wrods heer")')
  val('spell-more-suggest', 'spellsuggest("mispeled", 2)')
end)

-- `decor_providers_invoke_spell` — the `_on_spell_nav` callback, which only
-- the spell *navigation* commands reach. Nothing in the corpus registered
-- one, and nothing else in the tree calls it.
scenario('draw-spell-nav', function()
  local screen = start(40, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'first line is fine',
    'second has wrods in it',
    'third is fine as well',
    'fourth has anoter typo here',
    'fifth line is fine',
  })
  n.command('set spell spelllang=en_us')
  n.api.nvim_exec_lua(
    [[
      _G.spell_nav_calls = {}
      local ns = vim.api.nvim_create_namespace('spellnav')
      vim.api.nvim_set_decoration_provider(ns, {
        _on_spell_nav = function(_, _, _, srow, scol, erow, ecol)
          table.insert(_G.spell_nav_calls, ('%d:%d-%d:%d'):format(srow, scol, erow, ecol))
        end,
      })
    ]],
    {}
  )
  feed('gg]s')
  snap(screen, 'draw-spell-nav-first')
  feed(']s')
  snap(screen, 'draw-spell-nav-second')
  feed('[s')
  snap(screen, 'draw-spell-nav-back')
  lua_val('spell-nav-calls', 'table.concat(_G.spell_nav_calls, " | ")')
  val('spell-nav-cursor', 'getcurpos()')
end)

-- 'topfill' partway through a run of virtual lines above the top line.
-- `draw-virtlines-top` scrolls into exactly this state but never forces a
-- full redraw, so `win_update` scrolled the already-drawn rows and `win_line`
-- was never called for them: the top line's `n_virt_lines` clamp — and the
-- loop's `filler_todo - (filler_lines - n_virt_lines)` index that reads it —
-- had no witness at all. (B12-11 recorded the clamp as possibly dead code;
-- it is not, it was a corpus gap.)
scenario('draw-topfill', function()
  local screen = start(36, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight',
  })
  local ns = n.api.nvim_create_namespace('vltop2')
  n.api.nvim_buf_set_extmark(0, ns, 3, 0, {
    virt_lines = { { { 'VL-a', 'Search' } }, { { 'VL-b', 'Search' } }, { { 'VL-c', 'Search' } } },
    virt_lines_above = true,
  })
  for i = 1, 6 do
    feed('<C-e>')
    n.command('redraw!')
    snap(screen, 'draw-topfill-' .. i)
  end
  val('topfill-view', 'winsaveview()')
end)

scenario('screen-splits', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'buffer one' })
  n.command('vsplit')
  n.command('split')
  n.command('set fillchars=vert:!,horiz:=,horizup:^,horizdown:v,vertleft:<,vertright:>,verthoriz:+')
  snap(screen, 'screen-splits-fillchars')
  n.command('set laststatus=0')
  snap(screen, 'screen-splits-nostatus')
  n.command('set laststatus=3')
  snap(screen, 'screen-splits-global-status')
end)

scenario('screen-statusline', function()
  local screen = start(44, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'stl' })
  n.command([[set laststatus=2 statusline=%#Error#L%l%#Search#\ C%c%=%{'tail'}%<%f]])
  snap(screen, 'screen-statusline-groups')
  n.command([[set statusline=%-20.10(%f\ %m%)%=%P]])
  snap(screen, 'screen-statusline-trunc')
  n.command('set ruler rulerformat=%30(%=%l,%c%V\\ %P%)')
  n.command('set laststatus=0')
  snap(screen, 'screen-ruler')
end)

scenario('screen-tabline', function()
  local screen = start(44, 8)
  n.command('set showtabline=2')
  n.command('tabnew')
  n.command('tabnew')
  n.command('tabnext')
  snap(screen, 'screen-tabline-default')
  n.command([[set tabline=%#TabLine#A%#TabLineSel#B%=%#TabLineFill#C]])
  snap(screen, 'screen-tabline-custom')
end)

scenario('screen-winbar', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'winbar body', 'second' })
  n.command([[set winbar=%#Search#bar%=%#Error#right]])
  snap(screen, 'screen-winbar')
  n.command('split')
  snap(screen, 'screen-winbar-split')
end)

scenario('screen-msgarea', function()
  local screen = start(40, 8)
  n.command('echo "plain message"')
  snap(screen, 'screen-msg-plain')
  n.command('echohl ErrorMsg | echo "red message" | echohl NONE')
  snap(screen, 'screen-msg-hl')
  n.feed(':echoerr "boom"<CR>')
  snap(screen, 'screen-msg-hitenter')
  n.feed('<CR>')
  n.command('set cmdheight=3')
  n.command('echo "one\\ntwo"')
  snap(screen, 'screen-msg-cmdheight3')
  n.command('set cmdheight=0')
  snap(screen, 'screen-msg-cmdheight0')
end)

scenario('screen-showmode', function()
  local screen = start(38, 7)
  n.command('set showmode showcmd')
  n.feed('i')
  snap(screen, 'screen-mode-insert')
  n.feed('<Esc>R')
  snap(screen, 'screen-mode-replace')
  n.feed('<Esc>v')
  snap(screen, 'screen-mode-visual')
  n.feed('<Esc>qq')
  snap(screen, 'screen-mode-recording')
  n.feed('q')
end)

scenario('screen-lazyredraw', function()
  local screen = start(34, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'a', 'b', 'c' })
  n.command('set lazyredraw')
  n.command('call setline(1, ["X","Y","Z"])')
  snap(screen, 'screen-lazyredraw-pending')
  n.command('redraw')
  snap(screen, 'screen-lazyredraw-forced')
  n.command('set nolazyredraw')
  n.command('call setline(1, ["1","2","3"])')
  snap(screen, 'screen-lazyredraw-off')
end)

-- Window separator connectors. `draw_sep_connectors_win` returns immediately
-- unless the *global* statusline is on ('laststatus' 3), and no scenario ever
-- combined that with a window that has a separator -- so `hsep_connected`,
-- `vsep_connected` and `get_corner_sep_connector`, ~180 lines of frame-tree
-- walking, were never executed at all.
--
-- 'fillchars' gives each of the five connector kinds a distinct ASCII glyph,
-- so the grid says which arm of `get_corner_sep_connector` ran; the
-- box-drawing defaults are snapshotted too, because that is what a user sees.
scenario('screen-separators', function()
  local screen = start(41, 13)
  -- No '|' anywhere in the value: it ends the `:set` command.
  local marked =
    'vert:!,horiz:=,horizup:A,horizdown:B,vertleft:C,vertright:D,verthoriz:E,eob:~'
  n.command('set laststatus=3 fillchars=' .. marked)

  -- Four-way cross: BOTH columns split at the same row, so every inner corner
  -- is vsep- and hsep-connected -> verthoriz.
  n.command('vsplit')
  n.command('split')
  n.command('wincmd l')
  n.command('split')
  snap(screen, 'screen-sep-cross')
  val('sep-cross-layout', 'winlayout()')

  -- One tall column on the RIGHT beside a split one: vsep-connected, not
  -- hsep-connected, and the corner is on the right -> vertleft.
  n.command('only')
  n.command('vsplit')
  n.command('split')
  snap(screen, 'screen-sep-tee-left')
  val('sep-tee-left-layout', 'winlayout()')

  -- The mirror image: the split column is on the right -> vertright.
  n.command('only')
  n.command('vsplit')
  n.command('wincmd l')
  n.command('split')
  snap(screen, 'screen-sep-tee-right')
  val('sep-tee-right-layout', 'winlayout()')

  -- Full-width window above two columns: the corner below it is not
  -- vsep-connected and is a TOP corner -> horizdown.
  n.command('only')
  n.command('split')
  n.command('wincmd j')
  n.command('vsplit')
  snap(screen, 'screen-sep-horizdown')
  val('sep-horizdown-layout', 'winlayout()')

  -- Two columns above a full-width window: BOTTOM corner, not
  -- vsep-connected -> horizup.
  n.command('only')
  n.command('split')
  n.command('vsplit')
  snap(screen, 'screen-sep-horizup')
  val('sep-horizup-layout', 'winlayout()')

  -- A 3x2 grid so several connectors of different kinds appear in one frame,
  -- and the "walk to the neighbouring frame at this row/column" loops have
  -- more than one candidate to skip past.
  n.command('only')
  n.command('vsplit')
  n.command('vsplit')
  n.command('split')
  n.command('wincmd l')
  n.command('split')
  n.command('wincmd l')
  n.command('split')
  snap(screen, 'screen-sep-grid')
  val('sep-grid-layout', 'winlayout()')

  -- Unequal splits: the neighbouring column's separator is at a different
  -- row, so `hsep_connected`'s "walk down to the frame that covers sep_row"
  -- loop actually has to walk.
  n.command('wincmd k')
  n.command('resize 2')
  snap(screen, 'screen-sep-uneven')

  -- The defaults, which is what the connectors normally look like.
  n.command('set fillchars=')
  snap(screen, 'screen-sep-defaults')

  -- Without the global statusline every corner is a plain statusline row and
  -- nothing is drawn: the control that says the glyphs above come from
  -- `draw_sep_connectors_win` and not from the separators themselves.
  n.command('set laststatus=2 fillchars=' .. marked)
  snap(screen, 'screen-sep-no-global-stl')
end)

-- `ext_multigrid`: every other scenario runs the default single-grid UI, so
-- `win_grid_alloc`'s per-window grids, the `win_viewport` events and
-- `update_screen`'s `!ui_has(kUIMultigrid)` guard around the msg_scrolled
-- invalidation were all untested.
scenario('screen-multigrid', function()
  local screen = start(46, 12, { ext_multigrid = true })
  local lines = {}
  for i = 1, 40 do
    lines[i] = ('multigrid line %02d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set number')
  snap(screen, 'screen-mg-plain')

  n.command('split')
  snap(screen, 'screen-mg-split')
  n.command('vsplit')
  snap(screen, 'screen-mg-vsplit')

  -- Scrolling a per-window grid: the viewport event is the only report of
  -- where the window is, and win_update's scroll shortcut is what moves the
  -- rows inside it.
  feed('3<C-e>')
  snap(screen, 'screen-mg-scrolled')
  feed('20G')
  snap(screen, 'screen-mg-jumped')

  -- A float gets a grid of its own.
  local buf = n.api.nvim_create_buf(false, true)
  n.api.nvim_buf_set_lines(buf, 0, -1, true, { 'FLOAT one', 'FLOAT two' })
  local win = n.api.nvim_open_win(buf, false, {
    relative = 'editor',
    row = 2,
    col = 6,
    width = 12,
    height = 2,
    border = 'single',
  })
  snap(screen, 'screen-mg-float')
  n.api.nvim_win_set_config(win, { relative = 'editor', row = 5, col = 20 })
  snap(screen, 'screen-mg-float-moved')
  n.api.nvim_win_close(win, true)
  snap(screen, 'screen-mg-float-closed')

  -- Messages that scroll the screen: under multigrid `update_screen` must
  -- NOT clear rows of the default grid for the message area.
  -- Messages that scroll the screen. Both keystrokes have to go through
  -- `n.feed` and neither may be followed by a round trip: three lines into a
  -- one-line cmdline stops at a hit-enter prompt, which DEFERS `nvim_eval`,
  -- so `n.command` or the `feed` helper would wedge the run. The state worth
  -- snapshotting is the one after the prompt is answered anyway: `msg_scrolled`
  -- is only reset inside `update_screen`, so the next redraw is the one that
  -- runs the restore branch -- whose window-invalidation half is skipped
  -- entirely when the UI has multigrid.
  n.command('set cmdheight=1')
  n.feed(':echo "one\\ntwo\\nthree"<CR>')
  n.feed('<CR>')
  snap(screen, 'screen-mg-msg-restored')

  n.command('only')
  snap(screen, 'screen-mg-only')
  val('mg-viewport', 'nvim_win_get_position(0)')
end)

-- The single-grid half of `update_screen`'s msg_scrolled restore: with no
-- multigrid the function clears the rows the message area covered and marks
-- every window whose bottom reached into them for a full redraw, per window
-- and per status line. `screen-msgarea` never scrolled the message area and
-- never had more than one window, so that loop drew nothing.
--
-- The prompt itself cannot be snapshotted -- a hit-enter prompt defers
-- `nvim_eval`, which is what the sweep's settle loop needs -- so both keys are
-- fed without a round trip and the snapshot is of the restored screen.
scenario('screen-msgscroll', function()
  local screen = start(44, 14)
  local lines = {}
  for i = 1, 30 do
    lines[i] = ('msgscroll %02d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set number laststatus=2')
  n.command('split')
  n.command('wincmd j')
  n.command('vsplit')
  snap(screen, 'screen-msgscroll-anchor')

  -- Two lines into a one-line cmdline: only the bottom windows reach into the
  -- invalidated rows.
  n.command('set cmdheight=1')
  n.feed(':echo "alpha\\nbeta"<CR>')
  n.feed('<CR>')
  snap(screen, 'screen-msgscroll-two')

  -- Far enough to reach the top window and its status line as well.
  n.feed(':echo join(map(range(1,9),\'"msg ".v:val\'),"\\n")<CR>')
  n.feed('<CR>')
  snap(screen, 'screen-msgscroll-deep')

  -- The same with the global statusline, which takes the other arm of the
  -- per-window loop (`is_stl_global`).
  n.command('set laststatus=3')
  n.feed(':echo join(map(range(1,9),\'"msg ".v:val\'),"\\n")<CR>')
  n.feed('<CR>')
  snap(screen, 'screen-msgscroll-global-stl')
  val('msgscroll-view', 'winsaveview()')
end)

-- 'redrawdebug'. This one is deliberately not a *differential* scenario: its
-- flags change what the grid layer SENDS, not what it draws, so a correct
-- implementation renders identically with and without them. What it does buy
-- is the `invalid` flag, which poisons the line buffer before every batch so
-- that any batch depending on the previous contents trips an assertion, and
-- `nodelta`, which forces every cell through the copy path. Both are run over
-- a scene busy enough to reach the interesting code, and the assertion is the
-- oracle: if drawing ever depends on stale linebuf contents this scenario
-- aborts the child instead of quietly agreeing.
scenario('screen-redrawdebug', function()
  local screen = start(42, 12)
  n.command('set redrawdebug=invalid,nodelta,compositor')
  val('rdb-value', '&redrawdebug')
  local lines = {}
  for i = 1, 30 do
    lines[i] = ('rdb line %02d with 漢字 and\ttab'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set number list listchars=tab:>-,eol:$ cursorline colorcolumn=30')
  snap(screen, 'screen-rdb-plain')
  n.command('vsplit')
  n.command('split')
  snap(screen, 'screen-rdb-splits')
  feed('3<C-e>')
  snap(screen, 'screen-rdb-scrolled')
  local buf = n.api.nvim_create_buf(false, true)
  n.api.nvim_buf_set_lines(buf, 0, -1, true, { 'over one', 'over two' })
  n.api.nvim_open_win(buf, false, {
    relative = 'editor',
    row = 3,
    col = 8,
    width = 10,
    height = 2,
  })
  snap(screen, 'screen-rdb-float')
  n.command('echo "message under redrawdebug"')
  snap(screen, 'screen-rdb-msg')
  n.command('set redrawdebug=')
  n.command('redraw!')
  snap(screen, 'screen-rdb-off')
end)

-- The scroll-region optimisation inside `win_update`: the three cases its own
-- comment names (off the top -> scroll down; topline below the first
-- w_lines[] entry -> scroll up; topline unchanged -> find the first stale
-- entry) plus the insert/delete of screen rows for changed lines in the
-- middle of the loop.
--
-- Nothing in the corpus drove these deliberately. They matter because a bug
-- here leaves STALE CELLS on the grid -- the rows `win_update` moved instead
-- of redrawing -- which is exactly what a screen differential sees and what a
-- suite that says `redraw!` cannot. **No snapshot here forces a redraw**: the
-- whole point is what `win_update` decides to leave alone.
scenario('screen-scrollregion', function()
  local screen = start(40, 12)
  local lines = {}
  for i = 1, 60 do
    lines[i] = ('line %02d %s'):format(i, ('.'):rep(i % 7))
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set nowrap number')
  feed('30G')
  snap(screen, 'screen-scroll-anchor')

  -- Case 1: the new topline is a few lines ABOVE the old one -> scroll the
  -- window down, set top_end, move the w_lines[] entries down and invalidate
  -- the ones now at the top.
  feed('3<C-y>')
  snap(screen, 'screen-scroll-down-3')
  feed('<C-y>')
  snap(screen, 'screen-scroll-down-1')

  -- Case 2: the new topline is BELOW the first entry -> scroll up, set
  -- bot_start, copy the surviving entries upwards.
  feed('5<C-e>')
  snap(screen, 'screen-scroll-up-5')
  feed('<C-e>')
  snap(screen, 'screen-scroll-up-1')

  -- Case 3: "not too far off" fails -- the jump exceeds the window height, so
  -- the shortcut is abandoned and everything is redrawn.
  feed('55G')
  snap(screen, 'screen-scroll-faroff')
  feed('5G')
  snap(screen, 'screen-scroll-faroff-back')

  -- Changed lines in the middle with valid entries below: xtra_rows > 0
  -- inserts screen rows, xtra_rows < 0 deletes them, and w_lines[] is shifted
  -- to match. 'number' makes a missed shift loud.
  --
  -- The change has to leave SEVERAL undisturbed rows below it inside the
  -- window. A first draft changed a line near the bottom; `win_update` then
  -- finds no valid entry below the change, sets bot_start = 0 and redraws the
  -- rest, and every xtra_rows mutation was invisible. `zt` pins the topline so
  -- the change lands a third of the way down.
  feed('30G')
  feed('zt')
  feed('3j')
  snap(screen, 'screen-scroll-mod-anchor')
  n.command('call append(32, ["INSERTED A", "INSERTED B"])')
  snap(screen, 'screen-scroll-mod-insert')
  n.command('33,34d')
  snap(screen, 'screen-scroll-mod-delete')

  -- A line that changes HEIGHT moves xtra_rows by a fraction of a buffer
  -- line, which is the other half of the same arithmetic.
  n.command('set wrap')
  n.command('call setline(33, repeat("wrapped ", 8))')
  snap(screen, 'screen-scroll-mod-wrapgrow')
  n.command('call setline(33, "short again")')
  snap(screen, 'screen-scroll-mod-wrapshrink')
  n.command('set nowrap')

  -- A change that reaches the end of the window: no valid entry below it, so
  -- bot_start collapses to 0 and the rest is redrawn.
  feed('20G')
  snap(screen, 'screen-scroll-mod-tail-anchor')
  n.command('call setline(28, "TAIL CHANGED")')
  snap(screen, 'screen-scroll-mod-tail')

  -- w_lines[] reuse with no buffer change at all: moving the cursor with
  -- 'cursorline' redraws the two cursorline rows, and 'relativenumber'
  -- redraws the number column of every other row through the col_rows arm of
  -- `win_line` -- nothing else may move.
  n.command('set cursorline relativenumber')
  feed('30G')
  snap(screen, 'screen-scroll-rnu-anchor')
  feed('3j')
  snap(screen, 'screen-scroll-rnu-moved')
  feed('2k')
  snap(screen, 'screen-scroll-rnu-back')
  n.command('set nocursorline norelativenumber')

  -- A closed fold is one screen row for many buffer lines, so the walk is
  -- driven by wl_lastlnum rather than wl_lnum and every shift above has a
  -- fold arm of its own.
  n.command('set foldmethod=manual')
  feed('gg')
  n.command('20,26fold')
  feed('16G')
  snap(screen, 'screen-scroll-fold-anchor')
  feed('4<C-e>')
  snap(screen, 'screen-scroll-fold-up')
  feed('2<C-y>')
  snap(screen, 'screen-scroll-fold-down')
  n.command('call append(30, "AFTER FOLD")')
  snap(screen, 'screen-scroll-fold-mod')
  val('scroll-fold-view', 'winsaveview()')

  -- Two windows on the same buffer: the change is applied to both, but only
  -- one of them is `curwin`, which is the difference `dollar_vcol` and the
  -- w_botline recursion test look at.
  n.command('set nofoldenable')
  n.command('split')
  feed('10G')
  n.command('wincmd j')
  feed('40G')
  snap(screen, 'screen-scroll-two-anchor')
  n.command('call setline(12, "CHANGED IN BOTH")')
  n.command('call append(41, "APPENDED BELOW")')
  snap(screen, 'screen-scroll-two-changed')
  val('scroll-two-view', 'winsaveview()')
end)

-- The tail of `win_update`: what fills the rows below the last buffer line,
-- and what it does when the last line does not fit.
scenario('screen-winupdate-tail', function()
  local screen = start(36, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'one', 'two', 'three' })
  n.command('set fillchars=eob:%,lastline:@')
  snap(screen, 'screen-tail-eob')
  n.command('set fillchars=eob:\\ ,lastline:@')
  snap(screen, 'screen-tail-eob-blank')

  -- A line taller than the room left: 'display' chooses between a column of
  -- "@" (neither flag), "@@@" in the last row (truncate) and "@@@" at the end
  -- of it (lastline).
  --
  -- The long line must NOT be the topline: `win_update` has an arm above the
  -- 'display' ladder for "single line that does not fit", which draws no
  -- marker at all and made a first draft of these three snapshots identical.
  n.command('set fillchars=lastline:@ wrap display=')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'first', 'second', ('long word '):rep(30) })
  feed('gg')
  snap(screen, 'screen-tail-toolong-plain')
  n.command('set display=lastline')
  snap(screen, 'screen-tail-toolong-lastline')
  n.command('set display=truncate')
  snap(screen, 'screen-tail-toolong-truncate')

  -- The four-cell marker: 'lastline' widens "@@@" to "@@@@" when putting it
  -- at three would cut a double-width character in half.
  n.command('set display=lastline')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'first', ('日本語'):rep(60) })
  feed('gg')
  snap(screen, 'screen-tail-toolong-wide')

  -- Window ending in filler lines: `win_get_fill` covers the rest of the
  -- window, so w_filler_rows is set, w_botline stays on the line and no
  -- marker is drawn.
  n.command('set display= wrap')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'a', 'b', 'c', 'd', 'e', 'f' })
  local ns = n.api.nvim_create_namespace('tail')
  local vl = {}
  for i = 1, 12 do
    vl[i] = { { ('filler %02d'):format(i), 'DiffAdd' } }
  end
  n.api.nvim_buf_set_extmark(0, ns, 3, 0, { virt_lines = vl })
  feed('gg')
  snap(screen, 'screen-tail-filler')
  feed('3<C-e>')
  snap(screen, 'screen-tail-filler-scrolled')
  val('tail-filler-view', 'winsaveview()')
  val('tail-filler-botline', 'line("w$")')

  -- The eob area after a deletion that shortens the buffer: `lastline` is
  -- computed from bot_scroll_start / mid_start and a wrong answer leaves the
  -- deleted text on screen.
  for _, m in ipairs(n.api.nvim_buf_get_extmarks(0, ns, 0, -1, {})) do
    n.api.nvim_buf_del_extmark(0, ns, m[1])
  end
  local many = {}
  for i = 1, 20 do
    many[i] = ('tail %02d'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, many)
  n.command('set fillchars=eob:%')
  feed('gg')
  snap(screen, 'screen-tail-full')
  -- The buffer becomes shorter than the window, so the eob fill has to cover
  -- what the deleted lines left behind: `lastline` is computed from
  -- bot_scroll_start / mid_start and a wrong answer leaves stale text there.
  n.command('6,20d')
  snap(screen, 'screen-tail-shrunk')
  n.command('call append(5, ["back one", "back two"])')
  snap(screen, 'screen-tail-regrown')
  n.command('3,4d')
  snap(screen, 'screen-tail-shrunk-again')

  -- Zero-height and zero-width windows: `win_update` draws only the
  -- separator and returns before touching a single buffer line.
  -- `resize 0` does NOT get there: only `wincmd _` / `wincmd |` on the
  -- *other* window squeezes this one to nothing.
  n.command('set winminheight=0 winminwidth=0 laststatus=2')
  n.command('set fillchars=vert:!,horiz:=,eob:%')
  n.command('only')
  n.command('split')
  n.command('wincmd j')
  n.command('wincmd _')
  snap(screen, 'screen-tail-zeroheight')
  val('tail-zeroheight', 'map(range(1,winnr("$")), "winheight(v:val)")')
  n.command('only')
  n.command('vsplit')
  n.command('wincmd l')
  n.command('wincmd |')
  snap(screen, 'screen-tail-zerowidth')
  val('tail-zerowidth', 'map(range(1,winnr("$")), "winwidth(v:val)")')
  val('tail-zero-layout', 'winlayout()')
end)

-- ------------------------------------------- highlight.rs / highlight_group.rs

scenario('hl-groups', function()
  local screen = start(46, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'hl target line' })
  n.command('hi MyBold gui=bold cterm=bold guifg=#ff0000 ctermfg=Red')
  n.command('hi MyUnd gui=underline,italic guisp=#00ff00 cterm=underline')
  n.command('hi MyRev gui=reverse,standout guibg=Blue ctermbg=4 blend=30')
  n.command('hi link MyLink MyBold')
  n.command('hi default MyDef guifg=Yellow')
  n.command('syntax match MyBold /hl/ | syntax match MyUnd /target/ | syntax match MyRev /line/')
  snap(screen, 'hl-groups-render')
  for _, g in ipairs({ 'MyBold', 'MyUnd', 'MyRev', 'MyLink', 'MyDef', 'Normal', 'Comment' }) do
    lua_val('hl-' .. g, ("vim.api.nvim_get_hl(0, {name=%q})"):format(g))
    lua_val('hl-link-' .. g, ("vim.api.nvim_get_hl(0, {name=%q, link=true})"):format(g))
    val('synIDattr-' .. g, ('synIDattr(hlID("%s"), "fg")'):format(g))
  end
  say(txt, '--- hl-groups-list')
  local out = n.api.nvim_exec2('hi MyBold', { output = true }).output
  for _, line in ipairs(vim.split(out, '\n')) do
    say(txt, '  ' .. line)
  end
end)

scenario('hl-termgui', function()
  local screen = start(40, 7)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'colours' })
  n.command('hi T1 guifg=#123456 guibg=#abcdef ctermfg=33 ctermbg=200')
  n.command('syntax match T1 /colours/')
  n.command('set notermguicolors')
  snap(screen, 'hl-termgui-off')
  n.command('set termguicolors')
  snap(screen, 'hl-termgui-on')
  for _, c in ipairs({ 'Red', 'lightblue', 'DarkYellow', '#0a0b0c', 'NvimDarkGrey3', 'nosuchcolor' }) do
    val('color-' .. c, ('synIDattr(hlID("Normal"), "fg")'))
    lua_val('setfg-' .. c, ([[(function() local ok,e=pcall(vim.api.nvim_set_hl,0,'Probe',{fg=%q}) return {ok, ok and vim.api.nvim_get_hl(0,{name='Probe'}) or tostring(e)} end)()]]):format(c))
  end
end)

scenario('hl-colorscheme', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'scheme' })
  for _, bg in ipairs({ 'dark', 'light' }) do
    n.command('set background=' .. bg)
    snap(screen, 'hl-colorscheme-' .. bg)
    for _, g in ipairs({ 'Normal', 'Comment', 'Visual', 'Search', 'DiffAdd', 'Pmenu' }) do
      lua_val(('cs-%s-%s'):format(bg, g), ("vim.api.nvim_get_hl(0, {name=%q})"):format(g))
    end
  end
  n.command('colorscheme vim')
  snap(screen, 'hl-colorscheme-vim')
end)

scenario('hl-errors', function()
  start(40, 7)
  for _, cmd in ipairs({
    'hi Foo bad',
    'hi Foo gui',
    'hi Foo gui=',
    'hi Foo guifg=#zzz',
    'hi Foo ctermfg=999',
    'hi link',
    'hi link A',
    'hi clear NoSuchGroupAtAll',
    'hi NoSuchGroupAtAll2',
    'hi Foo blend=200',
    'hi Foo start=x stop=y',
  }) do
    local ok, err = pcall(n.command, cmd)
    say(vals, ('%-22s %-58s %s'):format('hlerr', cmd, ok and 'OK' or errtext(err)))
  end
end)

scenario('hl-blend', function()
  local screen = start(40, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'under text one',
    'under text two',
    'under text three',
  })
  local buf = n.api.nvim_create_buf(false, true)
  n.api.nvim_buf_set_lines(buf, 0, -1, true, { 'FLOAT', 'FLOAT' })
  local win = n.api.nvim_open_win(buf, false, {
    relative = 'editor',
    row = 1,
    col = 4,
    width = 8,
    height = 2,
  })
  snap(screen, 'hl-blend-none')
  n.api.nvim_set_option_value('winblend', 30, { win = win })
  snap(screen, 'hl-blend-30')
  n.api.nvim_set_option_value('winblend', 80, { win = win })
  snap(screen, 'hl-blend-80')
end)

scenario('hl-namespace', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'namespaced' })
  local ns = n.api.nvim_create_namespace('scrsweep')
  n.api.nvim_set_hl(ns, 'Normal', { fg = '#00ff00', bg = '#000080' })
  n.api.nvim_set_hl(ns, 'NonText', { fg = '#ff00ff' })
  snap(screen, 'hl-ns-before')
  n.api.nvim_win_set_hl_ns(0, ns)
  snap(screen, 'hl-ns-after')
  n.command('set winhighlight=Normal:Search,NonText:ErrorMsg')
  snap(screen, 'hl-winhighlight')
  lua_val('get_hl_ns', ('vim.api.nvim_get_hl(%d, {})'):format(ns))
end)

-- ------------------------------ decoration.rs / decoration_provider.rs

scenario('decor-extmarks', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'first line', 'second line', 'third line' })
  local ns = n.api.nvim_create_namespace('decor')
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    end_col = 5,
    hl_group = 'Search',
    priority = 100,
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 6, {
    virt_text = { { ' EOL', 'Error' } },
    virt_text_pos = 'eol',
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 3, {
    virt_text = { { '[IN]', 'Comment' } },
    virt_text_pos = 'inline',
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, {
    virt_text = { { 'OVR', 'Todo' } },
    virt_text_pos = 'overlay',
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, {
    virt_lines = { { { 'virt above', 'DiffAdd' } } },
    virt_lines_above = true,
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    virt_lines = { { { 'virt below', 'DiffDelete' } } },
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, { end_col = 5, conceal = 'Z' })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, { line_hl_group = 'CursorLine' })
  n.command('set conceallevel=2')
  snap(screen, 'decor-extmarks')
  lua_val('decor-get', 'vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})')
  lua_val('decor-height', 'vim.api.nvim_win_text_height(0, {})')
end)

scenario('decor-signs', function()
  local screen = start(42, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'sa', 'sb', 'sc', 'sd' })
  local ns = n.api.nvim_create_namespace('dsign')
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    sign_text = 'A>',
    sign_hl_group = 'Error',
    number_hl_group = 'Search',
    line_hl_group = 'DiffAdd',
    priority = 10,
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, { sign_text = 'B>', priority = 20 })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    sign_text = 'C>',
    cursorline_hl_group = 'ErrorMsg',
  })
  n.command('set number signcolumn=auto:3 cursorline')
  snap(screen, 'decor-signs')
  lua_val('decor-signcols', "vim.fn.getwininfo(vim.api.nvim_get_current_win())[1].textoff")
end)

-- Competing decorations on the SAME cells at different priorities. Added
-- because the `decor-vtprio` mutation (the virt-text priority shift in
-- `decor_range_add_virt`) went undetected: with one decoration per position
-- there is nothing to reorder, so no priority arithmetic is observable.
scenario('decor-priority', function()
  local screen = start(48, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'priority row one',
    'priority row two',
    'priority row three',
  })
  local ns = n.api.nvim_create_namespace('prio')
  -- Three overlapping ranges over the same cells, ascending priority.
  for i, spec in ipairs({
    { 'Search', 10 },
    { 'ErrorMsg', 200 },
    { 'DiffAdd', 100 },
  }) do
    n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
      end_col = 8 + i,
      hl_group = spec[1],
      priority = spec[2],
    })
  end
  -- Inline virt text competing at the same position; render order is the
  -- priority order, which is exactly what the shift arithmetic decides.
  for _, spec in ipairs({
    { '[lo]', 'Comment', 1 },
    { '[hi]', 'Todo', 65000 },
    { '[mid]', 'DiffChange', 4096 },
  }) do
    n.api.nvim_buf_set_extmark(0, ns, 1, 4, {
      virt_text = { { spec[1], spec[2] } },
      virt_text_pos = 'inline',
      priority = spec[3],
    })
  end
  -- Two virt_lines below the same row, and two signs on the same row.
  for _, spec in ipairs({ { 'vlA', 30 }, { 'vlB', 20 }, { 'vlC', 25 } }) do
    n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
      virt_lines = { { { spec[1], 'DiffText' } } },
      priority = spec[2],
    })
  end
  for _, spec in ipairs({ { 'p1', 5 }, { 'p2', 4096 }, { 'p3', 100 } }) do
    n.api.nvim_buf_set_extmark(0, ns, 0, 0, { sign_text = spec[1], priority = spec[2] })
  end
  n.command('set signcolumn=auto:3')
  snap(screen, 'decor-priority')
  lua_val('decor-prio-marks', 'vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})')

  -- A virt text and a range highlight competing for the SAME cells, with
  -- priorities that straddle. `decor_range_add_virt` packs virt-text
  -- priority as `p << 16` while `decor_range_add_sh` packs `p << 16 +
  -- subpriority`; a mutation to the virt-text shift alone is invisible when
  -- only virt texts compete (their order is monotonic either way) and only
  -- shows against a range.
  n.command('enew!')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'straddle cells here', 'straddle cells here' })
  local ns2 = n.api.nvim_create_namespace('prio2')
  for row, vtprio in ipairs({ 40, 300 }) do
    n.api.nvim_buf_set_extmark(0, ns2, row - 1, 0, {
      end_col = 19,
      hl_group = 'Search',
      priority = 100,
    })
    n.api.nvim_buf_set_extmark(0, ns2, row - 1, 9, {
      virt_text = { { 'VT', 'ErrorMsg' } },
      virt_text_pos = 'overlay',
      hl_mode = 'combine',
      priority = vtprio,
    })
  end
  snap(screen, 'decor-priority-straddle')
end)

-- The virt-text priority *packing*, pinned (B12-1's open corpus gap, closed
-- in B12-5). `decor_range_add_virt` stores `priority << 16` while
-- `decor_range_add_sh` stores `priority << 16 + subpriority`; the
-- `decor-vtprio` mutation (shifting the virt-text side to `<< 15`) was caught
-- by nothing, and `decor-priority-straddle` above does not catch it either.
--
-- The reason is structural: every reader of the sorted active list either
-- ignores virt-text ranges entirely (the attribute / conceal / spell / url
-- combination in `decor_redraw_col_impl`, which only looks at highlight
-- ranges) or compares virt texts with each other only, where any *monotonic*
-- packing gives the same order.
--
-- `draw_virt_text` is the one exception: it walks the same list, an
-- end-of-line virt text advances `state->eol_col` as it is drawn, and a
-- `ui_watched` range — which comes from the `decor_range_add_sh` side —
-- reports whatever `eol_col` holds when its turn comes. So a ui_watched mark
-- and an eol virt text whose priorities straddle the shift
-- (p_ui < p_vt < 2 * p_ui) report different columns under the two packings.
-- The reported column is a `win_extmark` event, invisible to the grid, which
-- is why `extmark_val` exists.
scenario('decor-vtprio', function()
  local screen = start(44, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'vtprio one', 'vtprio two', 'vtprio three' })
  local ns = n.api.nvim_create_namespace('vtprio')

  -- Row 0 — the crossing pair: 150 << 16 > 100 << 16, but 150 << 15 < 100 << 16.
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    id = 1,
    virt_text = { { '<<eol0>>', 'Error' } },
    virt_text_pos = 'eol',
    priority = 150,
  })
  n.api.nvim_buf_set_extmark(0, ns, 0, 0, { id = 2, ui_watched = true, priority = 100 })

  -- Row 1 — the control: 100 and 150 order the same way under both packings,
  -- so this row must report the same column either way.
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, {
    id = 3,
    virt_text = { { '<<eol1>>', 'Todo' } },
    virt_text_pos = 'eol',
    priority = 100,
  })
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, { id = 4, ui_watched = true, priority = 150 })

  -- Row 2 — a ui_watched mark sandwiched between two eol virt texts, so the
  -- packing decides how many of them have advanced `eol_col` by its turn.
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    id = 5,
    virt_text = { { '<lo>', 'DiffAdd' } },
    virt_text_pos = 'eol',
    priority = 150,
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, {
    id = 6,
    virt_text = { { '<hi>', 'DiffText' } },
    virt_text_pos = 'eol',
    priority = 300,
  })
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, { id = 7, ui_watched = true, priority = 200 })

  -- Overlay ui_watched marks compete with overlay virt texts the same way.
  n.api.nvim_buf_set_extmark(0, ns, 0, 4, {
    id = 8,
    ui_watched = true,
    virt_text_pos = 'overlay',
    priority = 100,
  })

  screen._grid_win_extmarks = {}
  n.command('redraw!')
  snap(screen, 'decor-vtprio')
  extmark_val(screen, 'decor-vtprio-marks')
  lua_val('decor-vtprio-get', 'vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})')
end)

scenario('decor-provider', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'prov one', 'prov two', 'prov three' })
  n.api.nvim_exec_lua(
    [[
    local ns = vim.api.nvim_create_namespace('prov')
    _G.counts = {win = 0, line = 0, buf = 0}
    vim.api.nvim_set_decoration_provider(ns, {
      on_buf = function() _G.counts.buf = _G.counts.buf + 1 end,
      on_win = function() _G.counts.win = _G.counts.win + 1 return true end,
      on_line = function(_, _, buf, row)
        _G.counts.line = _G.counts.line + 1
        vim.api.nvim_buf_set_extmark(buf, ns, row, 0, {
          end_col = 4, hl_group = row % 2 == 0 and 'Search' or 'Error', ephemeral = true,
        })
      end,
    })
  ]],
    {}
  )
  n.command('redraw!')
  snap(screen, 'decor-provider')
  lua_val('decor-provider-called', '(_G.counts.win > 0 and _G.counts.line > 0)')
end)

-- The ORDER the decoration-provider callbacks run in, and where in the walk
-- each one sits. Nothing recorded it before: `decor-provider` above only
-- asserts the counts are non-zero, and a screen dump shows the *result* of a
-- callback, not when it ran -- so a provider invoked at a different point of
-- the line walk is invisible whenever the ephemeral marks it places happen to
-- land the same way. p22-1 §3.3 asked for exactly this row before
-- `decor_state` was threaded down the draw pass (S11).
--
-- Three providers, so the per-provider fan-out is pinned as well as the
-- per-row one:
--   * `p1` stays active for the whole redraw, places an ephemeral highlight
--     from every `on_line`, and DELETES a real mark from one of them -- the
--     deferred-free path, which is the only thing that reads
--     `running_decor_provider`.
--   * `p2` declines the window from `on_win`, so its `on_line` must not be
--     reached at all while its `on_start`/`on_buf`/`on_end` still are.
--   * `p3` has an `on_range` instead of an `on_line`, and the buffer's last
--     line is long enough to need more than one 100-byte chunk, which is the
--     only way `invoke_range_next` is called twice for one row.
--
-- Only the FIRST redraw cycle is recorded (`_G.cycle`): the RPC pump can
-- redraw between two requests, and how many times it does is scheduling
-- noise, while the order inside one cycle is not.
scenario('decor-provider-order', function()
  local screen = start(40, 8)
  n.command('set nomore shortmess+=sI')
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'order one',
    'order two',
    'order three',
    ('long %s'):format(('abcdefghij'):rep(24)),
  })
  n.api.nvim_exec_lua(
    [[
    _G.log, _G.cycle = {}, 0
    local function rec(s)
      if _G.cycle == 1 then _G.log[#_G.log + 1] = s end
    end
    local ns1 = vim.api.nvim_create_namespace('order1')
    local ns2 = vim.api.nvim_create_namespace('order2')
    local ns3 = vim.api.nvim_create_namespace('order3')
    -- A real mark for `on_line` to delete in the middle of the redraw.
    _G.doomed = vim.api.nvim_buf_set_extmark(0, ns1, 2, 0, {
      end_col = 5, hl_group = 'Search',
    })
    vim.api.nvim_set_decoration_provider(ns1, {
      on_start = function()
        _G.cycle = _G.cycle + 1
        rec('p1.start')
        return true
      end,
      on_buf = function() rec('p1.buf') end,
      on_win = function(_, _, _, toprow, botrow)
        rec(('p1.win %d %d'):format(toprow, botrow))
        return true
      end,
      on_line = function(_, _, buf, row)
        rec(('p1.line %d'):format(row))
        vim.api.nvim_buf_set_extmark(buf, ns1, row, 0, {
          end_col = 3, hl_group = 'Error', ephemeral = true,
        })
        if row == 1 then
          vim.api.nvim_buf_del_extmark(buf, ns1, _G.doomed)
        end
      end,
      on_end = function() rec('p1.end') end,
    })
    vim.api.nvim_set_decoration_provider(ns2, {
      on_start = function() rec('p2.start') return true end,
      on_buf = function() rec('p2.buf') end,
      on_win = function() rec('p2.win') return false end,
      on_line = function(_, _, _, row) rec(('p2.line %d'):format(row)) end,
      on_end = function() rec('p2.end') end,
    })
    vim.api.nvim_set_decoration_provider(ns3, {
      on_start = function() rec('p3.start') return true end,
      on_win = function() rec('p3.win') return true end,
      on_range = function(_, _, buf, srow, scol, erow, ecol)
        rec(('p3.range %d %d %d %d'):format(srow, scol, erow, ecol))
        vim.api.nvim_buf_set_extmark(buf, ns3, srow, scol, {
          end_col = scol + 2, hl_group = 'Todo', ephemeral = true,
        })
      end,
      on_end = function() rec('p3.end') end,
    })
  ]],
    {}
  )
  n.command('redraw!')
  snap(screen, 'decor-provider-order')
  lua_val('decor-provider-order-log', 'table.concat(_G.log, " | ")')
  lua_val(
    'decor-provider-order-marks',
    'vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})'
  )

  -- A second cycle, armed and dirtied in one chunk so that whichever redraw
  -- runs next is the one recorded. `on_buf` only runs for a buffer with
  -- `b_mod_set`, so the first cycle above cannot reach it; and the split
  -- makes `on_win` run twice, which is what proves the per-window state is
  -- reset (p2 declined the window in the cycle before).
  n.command('vsplit')
  n.api.nvim_exec_lua(
    [[
    _G.log, _G.cycle = {}, 0
    vim.api.nvim_buf_set_lines(0, 1, 2, true, { 'order two edited' })
  ]],
    {}
  )
  n.command('redraw!')
  snap(screen, 'decor-provider-order-2')
  lua_val('decor-provider-order-log2', 'table.concat(_G.log, " | ")')
end)

-- ---------------------------------------------------------- popupmenu.rs

scenario('pum-complete', function()
  local screen = start(40, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alpha',
    'alphabet',
    'alphanumeric',
    'alpine',
    '',
  })
  n.command('set completeopt=menu,menuone,noinsert pumheight=3')
  n.feed('Gcc<C-x><C-n>')
  snap(screen, 'pum-complete-open')
  n.feed('<C-n>')
  snap(screen, 'pum-complete-next')
  n.feed('<Esc>')
end)

-- Drives the pum's `kind` and `menu` columns, which decide `pum_kind_width`
-- and `pum_extra_width`. An earlier version defined the completion function
-- with `\`-continued Vimscript through `n.command`, which does NOT survive
-- (`:func` bodies need real lines) — the whole scenario silently produced a
-- hit-enter prompt instead of a menu, and the pum-width mutation went
-- undetected because no menu with a kind column was ever drawn.
scenario('pum-userdefined', function()
  local screen = start(46, 12)
  n.api.nvim_exec2(
    table.concat({
      'func! Comp(findstart, base)',
      '  if a:findstart',
      '    return 0',
      '  endif',
      "  return [{'word': 'one', 'kind': 'func', 'menu': 'from m1', 'info': 'info one'},",
      "        \\ {'word': 'twotwo', 'kind': 'v', 'menu': 'm2', 'info': 'info two'},",
      "        \\ {'word': 'three', 'kind': 'typedef', 'menu': 'menu three'}]",
      'endfunc',
    }, '\n'),
    {}
  )
  n.command('set completeopt=menu,menuone,noselect completefunc=Comp')
  n.feed('i<C-x><C-u>')
  snap(screen, 'pum-user-open')
  n.feed('<C-n><C-n>')
  snap(screen, 'pum-user-selected')
  n.command('set completeopt=menu,menuone,noselect,preview')
  n.feed('<C-n>')
  snap(screen, 'pum-user-info')
  n.feed('<Esc>')
end)

scenario('pum-wildmenu', function()
  local screen = start(46, 10)
  n.command('set wildmenu wildmode=full')
  n.feed(':sign <Tab>')
  snap(screen, 'pum-wildmenu-flat')
  n.feed('<Esc>')
  n.command('set wildoptions=pum')
  n.feed(':sign <Tab>')
  snap(screen, 'pum-wildmenu-pum')
  n.feed('<Esc>')
end)

-- The columns that do not fit. `pum_redraw` measures each column against
-- `pum_width` and, when a column is cut, marks the far cell of the row with
-- 'fillchars' `trunc` (or `<`/`>`). Nothing in the corpus reached that path:
-- every earlier pum scenario is wider than its items. 'pummaxwidth' is the
-- cheapest way to force it, and the double-width entry is here because the
-- cut can land in the middle of a two-cell character, which is the branch
-- that backfills a space.
scenario('pum-truncate', function()
  local screen = start(46, 10)
  n.api.nvim_exec2(
    table.concat({
      'func! Wide(findstart, base)',
      '  if a:findstart',
      '    return 0',
      '  endif',
      "  return [{'word': 'averyveryverylongcompletionword', 'kind': 'function', 'menu': 'from a long source name'},",
      "        \\ {'word': 'short', 'kind': 'v', 'menu': 'm'},",
      -- Exactly 12 cells wide, which with 'pummaxwidth'=14 lands on the
      -- boundary of the `cells + pad` test that decides whether a column
      -- counts as cut. Any other width agrees whatever `pad` is.
      "        \\ {'word': 'twelvechars!', 'kind': 'k', 'menu': 'm'},",
      "        \\ {'word': '\u{5E83}\u{3044}\u{5358}\u{8A9E}\u{306E}\u{9023}\u{7D9A}\u{3067}\u{3059}\u{3088}', 'kind': 'w', 'menu': 'wide'}]",
      'endfunc',
    }, '\n'),
    {}
  )
  n.command('set completeopt=menu,menuone,noselect completefunc=Wide')
  n.command('set pummaxwidth=14')
  feed('i<C-x><C-u>')
  snap(screen, 'pum-truncate-max')
  feed('<C-n><C-n><C-n>')
  snap(screen, 'pum-truncate-wide-selected')
  feed('<C-e><Esc>')
  -- One cell narrower, so the 12-cell item is marked as cut while its text
  -- still stops short of the edge. That is the only way to see the pum's OWN
  -- truncation glyph: when the text reaches the last column `grid_line_puts`
  -- has already put a '>' there for its own reasons and the two agree.
  n.command('set pummaxwidth=13')
  feed('i<C-x><C-u>')
  snap(screen, 'pum-truncate-narrow')
  feed('<C-e><Esc>')
  n.command('set pummaxwidth=14 fillchars=trunc:\u{2026}')
  feed('i<C-x><C-u>')
  snap(screen, 'pum-truncate-fcs')
  feed('<C-e><Esc>')
  n.command('set fillchars= pummaxwidth=0 pumwidth=30')
  feed('i<C-x><C-u>')
  snap(screen, 'pum-truncate-pumwidth')
  feed('<C-e><Esc>')
end)

-- The typed leader inside a match. `pum_compute_text_attrs` is skipped
-- whole when PmenuMatch equals Pmenu, when there is no leader, or when the
-- item is not the `abbr` column -- and every earlier scenario hit one of
-- those, so the per-cell attribute path drew nothing at all. Both spellings
-- are here: the plain leading-run compare and the 'completeopt' `fuzzy`
-- matcher, which highlights scattered characters instead.
scenario('pum-leader', function()
  local screen = start(46, 10)
  n.command('hi PmenuMatch guifg=#00ff00 guibg=#111111')
  n.command('hi PmenuMatchSel guifg=#ff0000 guibg=#333333 gui=bold')
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alphabet',
    'alphanumeric',
    'ALPHASOUP',
    'malformed',
    '',
  })
  n.command('set completeopt=menu,menuone,noinsert')
  feed('Gccalp<C-x><C-n>')
  snap(screen, 'pum-leader-prefix')
  feed('<C-n>')
  snap(screen, 'pum-leader-next')
  feed('<C-e><Esc>')
  n.command('set completeopt=menu,menuone,noinsert,fuzzy')
  feed('Gccamc<C-x><C-n>')
  snap(screen, 'pum-leader-fuzzy')
  feed('<C-e><Esc>')
end)

-- 'completeitemalign' reorders the three columns, which moves every width
-- and separator in the row loop (`cia_flags` is read once per row). The
-- default order is the only one the corpus had.
scenario('pum-align', function()
  local screen = start(46, 10)
  n.api.nvim_exec2(
    table.concat({
      'func! Ali(findstart, base)',
      '  if a:findstart',
      '    return 0',
      '  endif',
      "  return [{'word': 'one', 'kind': 'func', 'menu': 'from m1'},",
      "        \\ {'word': 'twotwo', 'kind': 'v', 'menu': 'm2'},",
      "        \\ {'word': 'three', 'kind': 'typedef'}]",
      'endfunc',
    }, '\n'),
    {}
  )
  n.command('set completeopt=menu,menuone,noselect completefunc=Ali')
  for _, order in ipairs({ 'kind,abbr,menu', 'menu,kind,abbr', 'abbr,menu,kind' }) do
    n.command('set completeitemalign=' .. order)
    feed('i<C-x><C-u><C-n>')
    snap(screen, 'pum-align-' .. order:gsub(',', '_'))
    feed('<C-e><Esc>')
  end
end)

-- 'rightleft'. Every step of `pum_redraw` has a second spelling for it --
-- the column offset, the padding space, the per-cell attribute mirror, the
-- reversed text, the truncation marker and the scrollbar side -- and none
-- of it was reached by anything in the corpus.
scenario('pum-rightleft', function()
  local screen = start(46, 10)
  n.command('set rightleft')
  n.command('hi PmenuMatch guifg=#00ff00')
  n.command('hi PmenuMatchSel guifg=#ff0000')
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alphabet',
    'alphanumeric',
    'alpine',
    'alpaca',
    '',
  })
  n.command('set completeopt=menu,menuone,noinsert pumheight=3')
  feed('Gccalp<C-x><C-n>')
  snap(screen, 'pum-rightleft-open')
  feed('<C-n><C-n>')
  snap(screen, 'pum-rightleft-scrolled')
  feed('<C-e><Esc>')
  n.command('set pummaxwidth=6')
  feed('Gcca<C-x><C-n>')
  snap(screen, 'pum-rightleft-truncated')
  feed('<C-e><Esc>')
end)

-- 'pumborder'. The border is a second grid geometry: it costs one or two
-- cells on each side, moves the first drawn row, and -- for a box border --
-- lends the scrollbar trough its own glyph and highlight. The shadow style
-- takes a different path again (two dedicated highlight groups, blending
-- on, no row offset).
scenario('pum-border', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alphabet',
    'alphanumeric',
    'alpine',
    'alpaca',
    '',
  })
  n.command('set completeopt=menu,menuone,noinsert pumheight=3')
  for _, border in ipairs({ 'single', 'double', 'rounded', 'shadow', 'none' }) do
    n.command('set pumborder=' .. border)
    feed('Gcca<C-x><C-n>')
    snap(screen, 'pum-border-' .. border)
    feed('<C-e><Esc>')
  end
  n.command('set pumborder=')
  cmd_val('pum-border-bad', 'set pumborder=nosuchborder')
end)

-- The scrollbar. `pum_redraw` sizes the thumb as height^2/size and places
-- it from `pum_first`, and `pum_set_selected` is what moves `pum_first`;
-- neither was watched anywhere but incidentally in `pum-complete`, which
-- has four items and never scrolls past the second.
scenario('pum-scroll', function()
  local screen = start(30, 12)
  local lines = {}
  for i = 1, 30 do
    lines[i] = ('scroll%02d'):format(i)
  end
  lines[#lines + 1] = ''
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set completeopt=menu,menuone,noinsert pumheight=6')
  feed('Gccscroll<C-x><C-n>')
  snap(screen, 'pum-scroll-top')
  feed(('<C-n>'):rep(12))
  snap(screen, 'pum-scroll-middle')
  feed(('<C-n>'):rep(30))
  snap(screen, 'pum-scroll-bottom')
  feed('<C-p>')
  snap(screen, 'pum-scroll-wrapped')
  val('pum-scroll-getpos', 'pum_getpos()')
  val('pum-scroll-visible', 'pumvisible()')
  feed('<C-e><Esc>')
end)

-- A Tab inside an item is drawn as two spaces and restarts the run, which
-- is the one place `pum_redraw` puts text more than once per column. Also
-- covers a control character, which `transstr` renders as ^X.
scenario('pum-tabtext', function()
  local screen = start(46, 10)
  n.command('set completeopt=menu,menuone,noselect')
  n.api.nvim_exec_lua(
    [[
    _G.tabsrc = function(findstart)
      if findstart == 1 then return 0 end
      return {
        { word = 'a\tb\tc', kind = 'k\tk', menu = 'm\tm' },
        { word = 'ctrl\1char', kind = 'c' },
        { word = 'plain', kind = 'p' },
      }
    end
  ]],
    {}
  )
  n.command('set completefunc=v:lua.tabsrc')
  feed('i<C-x><C-u>')
  snap(screen, 'pum-tabtext-open')
  feed('<C-n>')
  snap(screen, 'pum-tabtext-selected')
  feed('<C-e><Esc>')
end)

-- ------------------------------------------------------------- plines.rs

scenario('plines-values', function()
  local screen = start(30, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'plain',
    '\tone tab',
    '\t\ttwo tabs',
    string.rep('w', 90),
    '  indented ' .. string.rep('q', 60),
    'wide 日本語 wide',
    'e\u{0301}composed',
  })
  local combos = {
    'set wrap tabstop=8 nolist nobreakindent showbreak=',
    'set wrap tabstop=4 list listchars=tab:>-,eol:$',
    'set wrap breakindent showbreak=++',
    'set wrap linebreak breakat=\\ ',
    'set nowrap',
    'set vartabstop=4,8,12',
  }
  for i, cmd in ipairs(combos) do
    n.command(cmd)
    say(vals, ('== plines combo %d: %s'):format(i, cmd))
    for lnum = 1, 7 do
      val(('plines-%d-%d'):format(i, lnum), ('[virtcol([%d, "$"]), strdisplaywidth(getline(%d)), foldtextresult(%d)]'):format(lnum, lnum, lnum))
      lua_val(
        ('height-%d-%d'):format(i, lnum),
        ('vim.api.nvim_win_text_height(0, {start_row=%d, end_row=%d})'):format(lnum - 1, lnum - 1)
      )
    end
    val(('screenpos-%d'):format(i), 'screenpos(0, 4, 40)')
    val(('vcol-%d'):format(i), 'virtcol([4, 40])')
  end
  n.command('set wrap tabstop=8 nolist')
  local ns = n.api.nvim_create_namespace('pl')
  n.api.nvim_buf_set_extmark(0, ns, 1, 0, { virt_lines = { { { 'vl1' } }, { { 'vl2' } } } })
  lua_val('plines-virtlines', 'vim.api.nvim_win_text_height(0, {})')
  snap(screen, 'plines-virtlines')
end)

-- `init_charsize_arg` computes `use_tabstop = !wo_list || lcs.tab1`, and its
-- callers are edit.rs / ops.rs / cursor.rs / register.rs / mouse.rs — NOT the
-- `virtcol()`/`strdisplaywidth()` path above, which builds its CharsizeArg
-- with `use_tabstop: false` outright. Blockwise operators and insert-mode
-- column arithmetic are how the corpus reaches it.
scenario('plines-charsize-callers', function()
  local screen = start(40, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    '\tone\ttab',
    'a\tb\tc',
    '\t\tdeep',
    'no tabs here',
  })
  for _, cmd in ipairs({
    'set nolist tabstop=8',
    'set list listchars=eol:$ tabstop=8',
    'set list listchars=tab:>-,eol:$ tabstop=8',
    'set list listchars=eol:$ tabstop=4',
    'set nolist vartabstop=3,5,7',
  }) do
    n.command(cmd)
    say(vals, ('== charsize callers: %s'):format(cmd))
    -- ops.rs: blockwise yank has to size every tab in the block.
    n.feed('gg0<C-v>3j$y')
    lua_val('blockreg', "vim.split(vim.fn.getreg('\"'), '\\n')")
    val('blockregtype', 'getregtype("\\"")')
    -- ops.rs / cursor.rs: '$'-anchored blockwise and cursor column queries.
    n.feed('gg')
    val('curswant', 'winsaveview().curswant')
    val('screenpos-2', 'screenpos(0, 2, 3)')
    val('virtcol-cursor', '[virtcol("."), virtcol("$")]')
    -- edit.rs: insert-mode tab/indent arithmetic.
    n.feed('2Gi<Tab>x<Esc>')
    val('after-insert', 'getline(2)')
    n.command('silent undo')
    -- register.rs: blockwise put re-sizes the tabs it lands on.
    n.feed('gg0<C-v>jly3Gp')
    val('after-put', 'getline(3)')
    n.command('silent undo')
    snap(screen, 'plines-charsize-' .. cmd:gsub('[^%w]', '-'))
  end
end)

-- -------------------------------------------------------------- match.rs

scenario('match-add', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'match target here',
    'another target line',
  })
  n.command('call matchadd("Search", "target", 10)')
  n.command('call matchadd("ErrorMsg", "tar", 20)')
  n.command('call matchaddpos("Todo", [[1, 1, 5], [2, 9]], 30)')
  snap(screen, 'match-add')
  val('getmatches', 'getmatches()')
  n.command('match DiffAdd /here/')
  n.command('2match DiffDelete /line/')
  n.command('3match DiffChange /another/')
  snap(screen, 'match-excmd')
  val('matcharg1', 'matcharg(1)')
  val('matcharg2', 'matcharg(2)')
  val('matcharg3', 'matcharg(3)')
  n.command('call clearmatches()')
  snap(screen, 'match-cleared')
  val('getmatches-after', 'getmatches()')
  -- Matches at EQUAL priority. `match_add` walks the list with
  -- `while (cur && prio >= cur->priority)`, so `>=` vs `>` only changes
  -- where a tie lands — with three distinct priorities above, nothing does.
  n.command('call matchadd("Search", "target", 15)')
  n.command('call matchadd("ErrorMsg", "target here", 15)')
  n.command('call matchadd("DiffAdd", "here", 15)')
  n.command('call matchaddpos("Todo", [[2, 1, 7]], 15)')
  snap(screen, 'match-equal-priority')
  val('getmatches-equal', 'getmatches()')
end)

scenario('match-hlsearch', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'find me and find me again',
    'nothing here',
    'find me once more',
  })
  n.command('set hlsearch incsearch')
  n.feed('/find<CR>')
  snap(screen, 'match-hlsearch')
  n.feed('n')
  snap(screen, 'match-cursearch')
  n.feed('/me')
  snap(screen, 'match-incsearch')
  n.feed('<Esc>')
  n.command('nohlsearch')
  snap(screen, 'match-nohlsearch')
end)

-- Matches whose priority straddles SEARCH_HL_PRIORITY (0). Every other
-- scenario uses a positive priority, so the three `cur->mit_priority >
-- SEARCH_HL_PRIORITY` tests — which decide whether 'hlsearch' is consulted
-- before or after the match list in update_search_hl / get_search_match_hl —
-- are always taken the same way. A match *below* the search highlight is the
-- only thing that takes the other branch.
scenario('match-searchprio', function()
  local screen = start(44, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'target one target',
    'plain line',
    'target two',
  })
  n.command('set hlsearch')
  n.command('hi CurSearch guibg=#00ff00 guifg=#000000')
  n.feed('/target<CR>')
  snap(screen, 'match-searchprio-none')
  n.command([[call matchadd('DiffAdd', 'target', -1)]])
  snap(screen, 'match-searchprio-below')
  n.command([[call matchadd('DiffChange', 'get one', 5)]])
  snap(screen, 'match-searchprio-above')
  val('getmatches-searchprio', 'getmatches()')
  -- The cursor sits in the first match, so HLF_LC (CurSearch) applies to it
  -- and not to the others: `check_cur_search_hl` is the only writer of
  -- `has_cursor` and nothing else in the corpus distinguishes the two groups.
  n.feed('n')
  snap(screen, 'match-searchprio-cursearch')
end)

-- A pattern containing `\n`. `prepare_search_hl` only does anything when
-- `re_multiline(regprog)` is true AND the window top is below the match
-- start, so the backward walk from `w_topline` (and its fold test) is
-- unreachable with a single-line pattern. The same match also gives
-- `endcol == MAXCOL` on its first line, which is what `get_prevcol_hl_flag`
-- and update_search_hl's `rm.endpos[0].lnum != 0` branch read.
scenario('match-multiline', function()
  local screen = start(40, 8)
  local lines = {}
  for i = 1, 24 do
    lines[i] = ('line %02d body'):format(i)
  end
  lines[10] = 'line 10 START'
  lines[11] = 'MIDDLE of it'
  lines[12] = 'END line 12'
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set list listchars=eol:$')
  n.command([[call matchadd('Search', 'START\nMIDDLE')]])
  n.command('10')
  n.feed('zt')
  snap(screen, 'match-multiline-top')
  -- Window top *inside* the match: prepare_search_hl walks back to find the
  -- first line the match could have started on.
  n.command('11')
  n.feed('zt')
  snap(screen, 'match-multiline-inside')
  -- A closed fold above the top line breaks that walk early.
  n.command('7,9fold')
  n.command('11')
  n.feed('zt')
  snap(screen, 'match-multiline-fold')
  val('getmatches-multiline', 'getmatches()')
end)

-- Zero-width matches. `prepare_search_hl_line` and `update_search_hl` both
-- carry a "highlight one character for an empty match" branch, reachable only
-- when startcol == endcol, and each has a sub-branch for a match sitting on
-- the NUL at end of line. `'list'` decides whether the char past the end is
-- highlighted at all (update_search_hl's last test).
scenario('match-empty', function()
  local screen = start(40, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'axbxc',
    '',
    'trailing eol',
  })
  n.command([[call matchadd('Search', 'x\zs')]])
  n.command([[call matchadd('ErrorMsg', '$', 20)]])
  snap(screen, 'match-empty-nolist')
  n.command('set list listchars=eol:$')
  snap(screen, 'match-empty-list')
  n.command('set nolist')
  n.command('call clearmatches()')
  -- An empty *pattern* match at the start of every line.
  n.command([[call matchadd('Todo', '^')]])
  snap(screen, 'match-empty-bol')
  val('getmatches-empty', 'getmatches()')
end)

-- `matchadd()`'s `{conceal:}` dict key is the only writer of
-- `mit_conceal_char`, and a match whose group is literally `Conceal` is the
-- only reader of update_search_hl's `has_match_conc`/`match_conc` pair. The
-- 2-vs-1 answer (`col == startcol ? 2 : 1`) is what makes the replacement
-- character appear once per match rather than once per cell.
scenario('match-conceal', function()
  local screen = start(46, 8)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'conceal this word here',
    'and conceal that word too',
    'third word line',
  })
  n.command('set conceallevel=2 concealcursor=nvic')
  n.command([[call matchadd('Conceal', 'word', 10, -1, {'conceal': 'X'})]])
  snap(screen, 'match-conceal-2')
  n.command('set conceallevel=1')
  snap(screen, 'match-conceal-1')
  n.command('set conceallevel=3')
  snap(screen, 'match-conceal-3')
  val('getmatches-conceal', 'getmatches()')
  n.command('call clearmatches()')
  n.command('set conceallevel=1')
  -- No `conceal` key: mit_conceal_char stays 0 and the default cchar applies.
  n.command([[call matchadd('Conceal', 'this\|that')]])
  snap(screen, 'match-conceal-nochar')
  val('getmatches-nochar', 'getmatches()')
  -- A conceal match added with matchaddpos() goes down the is_addpos path,
  -- which `get_prevcol_hl_flag` and `get_search_match_hl` both special-case.
  n.command('call clearmatches()')
  n.command('call matchaddpos("Conceal", [[3, 7, 4]], 10, -1, {"conceal": "#"})')
  snap(screen, 'match-conceal-addpos')
  val('getmatches-conceal-pos', 'getmatches()')
end)

-- getmatches()/setmatches() round trip. setmatches() is ~90 lines of
-- dict-shape code that nothing else in the corpus reaches: it rebuilds the
-- whole match list from the pos1..pos8 keys, the conceal key and the
-- group/pattern/priority/id quartet, and it is the only caller of match_add
-- with a `pos_list` that came back out of getmatches().
scenario('match-setmatches', function()
  local screen = start(46, 9)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'restore me now',
    'second restore line',
    'third line here',
  })
  n.command([[call matchadd('Search', 'restore', 11, 42)]])
  n.command([[call matchadd('Conceal', 'me', 12, 43, {'conceal': 'Q'})]])
  n.command('call matchaddpos("ErrorMsg", [[2, 1, 6], [3, 7], 1], 13, 44)')
  snap(screen, 'match-setmatches-before')
  val('getmatches-saved', 'getmatches()')
  n.command('let g:saved = getmatches()')
  n.command('call clearmatches()')
  snap(screen, 'match-setmatches-cleared')
  val('setmatches-rc', 'setmatches(g:saved)')
  snap(screen, 'match-setmatches-after')
  val('getmatches-restored', 'getmatches()')
  -- Every rejection path in setmatches(): a non-dict item, a dict missing a
  -- required key, and a pos key that is not a list.
  val('setmatches-notlist', 'setmatches("nope")')
  val('setmatches-nondict', 'setmatches([1])')
  val('setmatches-missing', "setmatches([{'group': 'Search'}])")
  val('setmatches-badpos', "setmatches([{'group':'Search','pos1':7,'priority':10,'id':9}])")
  val('getmatches-after-bad', 'getmatches()')
end)

-- The `window` dict key and the optional window argument. Four functions take
-- one (matchadd, matchaddpos, matchdelete, clearmatches/getmatches) and
-- `matchadd_dict_arg`'s E957 path is otherwise never taken.
scenario('match-window', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'shared alpha', 'shared beta' })
  n.command('split')
  local other = n.api.nvim_eval('win_getid(2)')
  n.command(('call matchadd("Search", "alpha", 10, -1, {"window": %d})'):format(other))
  n.command([[call matchadd('ErrorMsg', 'beta', 10)]])
  snap(screen, 'match-window-split')
  val('getmatches-cur', 'getmatches()')
  val('getmatches-other', ('getmatches(%d)'):format(other))
  val('matchadd-badwin', "matchadd('Search', 'x', 10, -1, {'window': 9999})")
  val('matchadd-notdict', "matchadd('Search', 'x', 10, -1, 'notadict')")
  val('matchdelete-other', ('matchdelete(%d, %d)'):format(
    n.api.nvim_eval(('getmatches(%d)[0].id'):format(other)), other))
  snap(screen, 'match-window-deleted')
  val('clearmatches-other', ('clearmatches(%d)'):format(other))
  val('getmatches-other-after', ('getmatches(%d)'):format(other))
  snap(screen, 'match-window-cleared')
end)

-- Every diagnosed failure in match.rs. E798/E799/E801/E802/E803 and the two
-- position-list errors (E5030/E5031) have no coverage at all, and `ex_match`
-- has none of any kind: `:4match`, a one-argument `:match`, an unterminated
-- pattern and a trailing argument are four distinct returns.
scenario('match-errors', function()
  start(46, 8)
  n.command([[call matchadd('Search', 'x', 10, 77)]])
  for _, expr in ipairs({
    "matchadd('Search', 'x', 10, 2)",
    "matchadd('Search', 'x', 10, 0)",
    "matchadd('Search', 'x', 10, -7)",
    "matchadd('Search', 'x', 10, 77)",
    "matchadd('', 'x')",
    "matchadd('Search', '')",
    "matchadd('Search', 'x\\(')",
    "matchadd('Search', 'x\\\\(')",
    "matchaddpos('Search', 'notalist')",
    "matchaddpos('Search', [])",
    "matchaddpos('Search', [[]])",
    "matchaddpos('Search', ['x'])",
    "matchaddpos('Search', [[1, 1, 1]], 10, 1)",
    "matchaddpos('Search', [[1, 1, 1]], 10, 3)",
    "matchdelete(0)",
    "matchdelete(999)",
    "matcharg(0)",
    "matcharg(4)",
    "matcharg(1)",
  }) do
    val('matcherr', expr)
  end
  for _, cmd in ipairs({
    '4match Search /x/',
    'match',
    'match Search',
    'match Search /x',
    'match Search /x/ tail',
    'match NoSuchGroupHere /x/',
    'match none',
    'match NONE',
    '2match Search /x/ | echo "chained"',
  }) do
    cmd_val('matchexcmd', cmd)
  end
  val('matcharg-after', 'matcharg(2)')
  val('getmatches-errors', 'getmatches()')
end)

-- --------------------------------------------------------------- sign.rs

scenario('sign-place', function()
  local screen = start(42, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'g1', 'g2', 'g3', 'g4', 'g5' })
  n.command('sign define Low text=aa texthl=Search linehl=DiffAdd numhl=Error')
  n.command('sign define High text=bb texthl=ErrorMsg culhl=Todo')
  n.command('sign place 1 line=1 name=Low priority=5 buffer=1')
  n.command('sign place 2 line=1 name=High priority=50 buffer=1')
  n.command('sign place 3 line=3 group=grp name=Low buffer=1')
  n.command('set number cursorline signcolumn=auto:4')
  snap(screen, 'sign-place')
  val('sign_getplaced', 'sign_getplaced(1, {"group": "*"})')
  val('sign_getdefined', 'sign_getdefined()')
  say(txt, '--- sign-list')
  for _, cmd in ipairs({ 'sign list', 'sign place group=*' }) do
    local out = n.api.nvim_exec2(cmd, { output = true }).output
    for _, line in ipairs(vim.split(out, '\n')) do
      say(txt, '  ' .. line)
    end
  end
  for _, sc in ipairs({ 'yes', 'no', 'auto', 'auto:1', 'auto:9', 'number', 'yes:2' }) do
    n.command('set signcolumn=' .. sc)
    snap(screen, 'sign-signcolumn-' .. sc)
  end
  n.command('sign unplace 2 buffer=1')
  snap(screen, 'sign-unplaced')
end)

-- The whole `sign_*()` Vimscript surface. `:sign` reaches sign_define_by_name
-- / sign_place / sign_unplace through a different argument parser and none of
-- the dict-shaped entry points (sign_define_from_dict, sign_place_from_dict,
-- sign_unplace_from_dict and the three list wrappers) is reachable from it.
scenario('sign-vimscript', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'v1', 'v2', 'v3', 'v4', 'v5', 'v6' })
  val('sign_define-1', "sign_define('VA', {'text': '>>', 'texthl': 'Search', 'linehl': 'DiffAdd', 'numhl': 'Todo', 'culhl': 'ErrorMsg', 'priority': 30})")
  val('sign_define-list', "sign_define([{'name': 'VB', 'text': 'B '}, {'name': 'VC', 'text': 'CC', 'icon': '/no/such/icon.xpm'}, {'name': ''}])")
  val('sign_define-baditem', "sign_define(['notadict'])")
  val('sign_getdefined-all', 'sign_getdefined()')
  val('sign_getdefined-one', "sign_getdefined('VB')")
  val('sign_getdefined-none', "sign_getdefined('NoSuchSign')")
  val('sign_place-1', "sign_place(11, '', 'VA', '%', {'lnum': 1, 'priority': 40})")
  val('sign_place-auto', "sign_place(0, 'grpA', 'VB', '%', {'lnum': 2})")
  val('sign_placelist', "sign_placelist([{'id': 13, 'name': 'VC', 'buffer': '%', 'lnum': 3}, {'name': 'VA', 'buffer': '%', 'lnum': 4, 'group': 'grpB', 'priority': 5}, {'name': 'VA'}])")
  val('sign_placelist-baditem', "sign_placelist(['notadict'])")
  n.command('set number cursorline signcolumn=auto:4')
  snap(screen, 'sign-vimscript-placed')
  val('sign_getplaced-all', "sign_getplaced('%', {'group': '*'})")
  val('sign_getplaced-global', "sign_getplaced('%')")
  val('sign_getplaced-group', "sign_getplaced('%', {'group': 'grpA'})")
  val('sign_getplaced-lnum', "sign_getplaced('%', {'group': '*', 'lnum': 3})")
  val('sign_getplaced-id', "sign_getplaced('%', {'group': '*', 'id': 11})")
  val('sign_getplaced-idlnum', "sign_getplaced('%', {'group': '*', 'id': 13, 'lnum': 3})")
  val('sign_getplaced-nogroup', "sign_getplaced('%', {'group': 'NoSuchGroup'})")
  val('sign_getplaced-nobuf', 'sign_getplaced()')
  val('sign_jump', "sign_jump(13, '', '%')")
  val('sign_jump-line', 'line(".")')
  val('sign_unplace-id', "sign_unplace('', {'id': 11})")
  val('sign_unplacelist', "sign_unplacelist([{'group': 'grpA'}, {'group': 'NoSuchGroup'}])")
  val('sign_unplacelist-baditem', "sign_unplacelist(['notadict'])")
  snap(screen, 'sign-vimscript-unplaced')
  val('sign_unplace-all', "sign_unplace('*')")
  val('sign_getplaced-final', "sign_getplaced('%', {'group': '*'})")
  snap(screen, 'sign-vimscript-empty')
  val('sign_undefine-one', "sign_undefine('VB')")
  val('sign_undefine-list', "sign_undefine(['VC'])")
  val('sign_undefine-badlist', "sign_undefine(['NoSuchSign'])")
  val('sign_getdefined-left', 'sign_getdefined()')
  val('sign_undefine-all', 'sign_undefine()')
  val('sign_getdefined-none2', 'sign_getdefined()')
end)

-- Every diagnosed failure in sign.rs, from both entry points. E155/E156/E157/
-- E159/E160/E239/E885 and the "Invalid buffer name" path have no coverage.
scenario('sign-errors', function()
  start(46, 8)
  n.command('sign define EOK text=ok')
  for _, cmd in ipairs({
    'sign nosuchsubcmd',
    'sign undefine',
    'sign list NoSuchSign',
    'sign undefine NoSuchSign',
    'sign define EBAD nosucharg=1',
    'sign define EBAD text=toolongtext',
    'sign define EBAD text=',
    'sign place 1 line=1 name=NoSuchSign buffer=1',
    'sign place 1 line=1 name=EOK buffer=999',
    'sign place 1 line=1 name=EOK file=/no/such/file/at/all',
    'sign place 1 name=EOK buffer=1',
    'sign place 1 buffer=1',
    'sign place line=1',
    'sign place group= file=x',
    'sign place 1 nosucharg=2 buffer=1',
    'sign unplace 1 line=2',
    'sign unplace * 1',
    'sign jump',
    'sign jump 1 buffer=1',
    'sign jump 1 line=1 buffer=1',
    'sign place 1 line=1 name=EOK buffer=1 trailing',
  }) do
    cmd_val('signerr', cmd)
  end
  for _, expr in ipairs({
    "sign_define('')",
    "sign_define('X', 'notadict')",
    "sign_define([{'name': 'Y'}], 'extra')",
    "sign_place(-1, '', 'EOK', '%', {'lnum': 1})",
    "sign_place(1, '', 'NoSuchSign', '%', {'lnum': 1})",
    "sign_place(1, '', 'EOK', '%', {'lnum': 0})",
    "sign_place(1, '', 'EOK', 'nosuchbuffer', {'lnum': 1})",
    "sign_place(1, '*', 'EOK', '%', {'lnum': 1})",
    "sign_placelist('notalist')",
    "sign_getplaced('nosuchbuffer')",
    "sign_getplaced('%', 'notadict')",
    "sign_jump(0, '', '%')",
    "sign_jump(1, '', '%')",
    "sign_unplace(0)",
    "sign_unplace('', {'id': 0})",
    "sign_unplace('', {'buffer': 'nosuchbuffer'})",
    "sign_unplacelist('notalist')",
    "sign_undefine(1.5)",
  }) do
    val('signfnerr', expr)
  end
end)

-- Sign *ordering*. `sign_row_cmp` sorts by row, then hands off to
-- `sign_item_cmp` (priority, then id, then recency), and its answer is what
-- decides which sign the signcolumn shows first, which one `:sign unplace`
-- at a line removes, and the order of `sign_getplaced`. Two signs of equal
-- priority on one line are the only thing that reaches the id tiebreak.
scenario('sign-order', function()
  local screen = start(46, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'o1', 'o2', 'o3' })
  n.command('sign define OA text=aa texthl=Search')
  n.command('sign define OB text=bb texthl=ErrorMsg')
  n.command('sign define OC text=cc texthl=DiffAdd')
  n.command('sign place 5 line=1 name=OA priority=10 buffer=1')
  n.command('sign place 6 line=1 name=OB priority=10 buffer=1')
  n.command('sign place 7 line=1 name=OC priority=20 buffer=1')
  n.command('set signcolumn=yes:3')
  snap(screen, 'sign-order-three')
  val('sign_getplaced-order', "sign_getplaced('%', {'group': '*'})")
  -- ":sign unplace {id}" with no id and a line removes the HIGHEST priority
  -- sign on that line, which is the only reader of the sorted array's [0].
  -- ":sign unplace" with no arguments at all is the ONLY way to reach
  -- buf_delete_signs with an `atlnum`: every other spelling is rejected by
  -- sign_unplace_cmd's `lnum >= 0` test.
  n.command('1')
  cmd_val('signunplace-cursor', 'sign unplace')
  snap(screen, 'sign-order-topgone')
  val('sign_getplaced-topgone', "sign_getplaced('%', {'group': '*'})")
  n.command('sign unplace')
  val('sign_getplaced-nextgone', "sign_getplaced('%', {'group': '*'})")
  -- Nothing left on the cursor line: buf_delete_signs answers FAIL and the
  -- caller turns that into E159.
  cmd_val('signunplace-none', 'sign unplace')
  -- Redefining a placed sign rewrites every DecorSignHighlight that names it
  -- and forces a redraw; nothing else in the corpus takes that loop.
  n.command('sign place 8 line=2 name=OA buffer=1')
  n.command('sign place 9 line=3 name=OA buffer=1')
  snap(screen, 'sign-order-redef-before')
  n.command('sign define OA text=zz texthl=DiffChange linehl=Todo numhl=ErrorMsg')
  n.command('set number')
  snap(screen, 'sign-order-redef-after')
  val('sign_getdefined-redef', "sign_getdefined('OA')")
  -- A sign whose definition is undefined while it is still placed reads back
  -- as "[Deleted]" through sign_get_name.
  n.command('sign undefine OA')
  snap(screen, 'sign-order-undefined')
  val('sign_getplaced-deleted', "sign_getplaced('%', {'group': '*'})")
  say(txt, '--- sign-order-listing')
  for _, cmd in ipairs({ 'sign list', 'sign place group=*', 'sign place' }) do
    local ok, out = pcall(function()
      return n.api.nvim_exec2(cmd, { output = true }).output
    end)
    for _, line in ipairs(vim.split(ok and out or ('ERR ' .. tostring(out)), '\n')) do
      say(txt, '  ' .. line)
    end
  end
end)

-- Sign *text* shapes. `init_sign_text` counts display cells, unescapes
-- backslashes and rejects anything non-printable or wider than SIGN_WIDTH;
-- `describe_sign_text` reads the result back. A one-cell sign is padded with
-- a space, a two-cell character fills both slots, and `:sign list` /
-- `sign_getdefined()` are the two readers of the padded form.
scenario('sign-text', function()
  local screen = start(46, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 't1', 't2', 't3', 't4', 't5' })
  n.command('sign define T1 text=x')
  n.command([[sign define T2 text=\ y]])
  n.command('sign define T3 text=ab')
  n.command('sign define T4 text=あ')
  n.command('sign define T5 text=é')
  for i = 1, 5 do
    n.command(('sign place %d line=%d name=T%d buffer=1'):format(i, i, i))
  end
  n.command('set signcolumn=yes')
  snap(screen, 'sign-text')
  val('sign_getdefined-text', 'sign_getdefined()')
  say(txt, '--- sign-text-listing')
  for _, line in ipairs(vim.split(n.api.nvim_exec2('sign list', { output = true }).output, '\n')) do
    say(txt, '  ' .. line)
  end
  -- Rejections: three cells, a control character, and a lone backslash.
  for _, cmd in ipairs({
    'sign define TBAD text=abc',
    'sign define TBAD text=あa',
  }) do
    cmd_val('signtext', cmd)
  end
  val('sign_define-badtext', "sign_define('TBAD3', {'text': 'abc'})")
  -- A non-printable character stops the cell walk before `endp`, which is a
  -- different rejection from "too wide" and cannot be typed on a `:sign` line.
  lua_val('sign_define-ctrl', "vim.fn.sign_define('TBAD4', {text = '\1x'})")
  -- A number-named sign: leading zeroes are stripped so "007" and "7" are the
  -- same sign, but a bare "0" is kept.
  n.command('sign define 007 text=7')
  n.command('sign define 0 text=0')
  val('sign_getdefined-numeric', 'sign_getdefined()')
  cmd_val('signnum', 'sign place 20 line=1 name=0007 buffer=1')
  val('sign_getplaced-numeric', "sign_getplaced('%')")
end)

-- `:sign` command-line completion. `set_context_in_sign_cmd` is 130 lines of
-- state machine over seven `expand_what` values and is reached by nothing
-- else in the corpus or in any spec.
scenario('sign-complete', function()
  start(46, 8)
  n.command('sign define CA text=aa')
  n.command('sign define CB text=bb')
  n.command('sign place 1 line=1 name=CA group=cgrp buffer=1')
  n.command('sign place 2 line=1 name=CB group=cgrp2 buffer=1')
  for _, pat in ipairs({
    'sign ',
    'sign u',
    'sign define ',
    'sign define X ',
    'sign define X t',
    'sign define X texthl=Diff',
    'sign define X numhl=Diff',
    'sign define X culhl=Diff',
    'sign define X linehl=Diff',
    'sign define X priority=',
    'sign list ',
    'sign undefine C',
    'sign place ',
    'sign place 1 ',
    'sign place 1 name=C',
    'sign place 1 group=',
    'sign place 1 nosuch=',
    'sign unplace ',
    'sign unplace 1 group=',
    'sign jump 1 group=',
    'sign jump 1 nosuch=',
  }) do
    val('signcomplete', ("getcompletion('%s', 'cmdline')"):format(pat))
  end
end)

-- ------------------------------------------------------------- syntax.rs

scenario('syntax-handwritten', function()
  local screen = start(46, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'KEY value BEGIN inside END trailer',
    'nested BEGIN a KEY b END done',
    'unterminated BEGIN forever',
    'next KEY line',
  })
  n.command('syntax keyword swKey KEY nextgroup=swNext skipwhite')
  n.command('syntax match swNext /value/ contained')
  n.command('syntax region swReg start=/BEGIN/ end=/END/ contains=swInner keepend')
  n.command('syntax match swInner /inside\\|a\\|b/ contained')
  n.command('hi link swKey Statement | hi link swNext Constant')
  n.command('hi link swReg Comment | hi link swInner Todo')
  snap(screen, 'syntax-handwritten')
  for l = 1, 4 do
    for c = 1, 20, 5 do
      val(('synID-%d-%d'):format(l, c), ('synIDattr(synID(%d, %d, 1), "name")'):format(l, c))
      val(('synstack-%d-%d'):format(l, c), ('map(synstack(%d, %d), \'synIDattr(v:val, "name")\')'):format(l, c))
    end
  end
  say(txt, '--- syntax-list')
  local out = n.api.nvim_exec2('syntax list', { output = true }).output
  for _, line in ipairs(vim.split(out, '\n')) do
    say(txt, '  ' .. line)
  end
end)

scenario('syntax-sync', function()
  local screen = start(40, 10)
  local lines = {}
  for i = 1, 60 do
    lines[i] = (i % 10 == 1) and 'BEGIN block ' .. i or ('body %d END'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('syntax region syReg start=/BEGIN/ end=/END/')
  n.command('hi link syReg String')
  for _, sync in ipairs({
    'syntax sync minlines=1',
    'syntax sync minlines=50 maxlines=60',
    'syntax sync fromstart',
    'syntax sync ccomment',
    'syntax sync linebreaks=2',
  }) do
    n.command(sync)
    n.feed('50G')
    snap(screen, 'syntax-' .. sync:gsub('[^%w]', '-'))
    val('sync-synID', 'synIDattr(synID(50, 3, 1), "name")')
  end
end)

scenario('syntax-runtime', function()
  local screen = start(50, 14)
  n.command('edit ' .. fixdir .. '/fixture.vim')
  n.command('syntax on')
  snap(screen, 'syntax-runtime-vim')
  n.command('edit ' .. fixdir .. '/fixture.c')
  snap(screen, 'syntax-runtime-c')
  n.command('edit ' .. fixdir .. '/fixture.lua')
  snap(screen, 'syntax-runtime-lua')
  n.command('edit ' .. fixdir .. '/fixture.sh')
  snap(screen, 'syntax-runtime-sh')
  n.command('edit ' .. fixdir .. '/fixture.diff')
  snap(screen, 'syntax-runtime-diff')
end)

scenario('syntax-errors', function()
  start(40, 7)
  for _, cmd in ipairs({
    'syntax keyword',
    'syntax match',
    'syntax match Foo',
    'syntax region Foo',
    'syntax region Foo start=/a/',
    'syntax cluster',
    'syntax include',
    'syntax sync bogus',
    'syntax match Foo /a/ badopt',
    'syntax clear NoSuchGroup',
    'syntax list NoSuchGroup',
    'syntax match Foo /unclosed',
  }) do
    local ok, err = pcall(n.command, cmd)
    say(vals, ('%-22s %-58s %s'):format('synerr', cmd, ok and 'OK' or errtext(err)))
  end
end)

-- The syntax state machine (B12-8). `ex_cmds/syntax_spec.lua` is one test of
-- twenty lines, there is no `ui/syntax_spec.lua`, and every screendump
-- oldtest is skipped in this port -- so what follows is the only per-cell
-- oracle the state machine has beyond `test_syntax.vim`.

scenario('syntax-region-keepend', function()
  local screen = start(56, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'plain [P body P] tail',
    'keep [K one <I two K] three I> four',
    'ext [E one <X two E] three X> four E] five',
    'nokeep [N one <I two N] three I> four',
    'multi [K one',
    'still inside K] after',
  })
  -- `keepend` truncates a contained region at the outer end match;
  -- `extend` on the contained item cancels that. The two `[K`/`[N` lines are
  -- the same scene with and without the flag, which is what makes the
  -- difference attributable.
  n.command('syntax region rP start=/\\[P/ end=/P\\]/')
  n.command('syntax region rK start=/\\[K/ end=/K\\]/ keepend contains=rI')
  n.command('syntax region rN start=/\\[N/ end=/N\\]/ contains=rI')
  n.command('syntax region rI start=/<I/ end=/I>/ contained')
  n.command('syntax region rE start=/\\[E/ end=/E\\]/ keepend contains=rX')
  n.command('syntax region rX start=/<X/ end=/X>/ contained extend')
  n.command('hi link rP Comment | hi link rK Directory | hi link rN Statement')
  n.command('hi link rI Todo | hi link rE Constant | hi link rX Identifier')
  snap(screen, 'syntax-region-keepend')
  for l = 1, 6 do
    syn_probe('keepend', l, { 1, 8, 14, 20, 26, 32 })
  end
end)

scenario('syntax-region-trans', function()
  local screen = start(56, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'outer [P x [T body T] y P] done',
    'ucont [P x [U body U] y P] done',
    'bare [T body T] outside',
    'mg [M body M] after',
    'one [O never closed here',
    'still [O closed O] now',
    'disp displayonly and more',
  })
  -- `transparent` with a `contains=` of its own is HL_TRANSP; without one it
  -- also takes HL_TRANS_CONT and inherits the *parent's* contains list, which
  -- is why `body` inside `[U` is claimed by rIn and `body` inside `[T` is not.
  n.command('syntax region rP start=/\\[P/ end=/P\\]/ contains=rT,rU,rIn')
  n.command('syntax region rT start=/\\[T/ end=/T\\]/ transparent contains=rNone')
  n.command('syntax region rU start=/\\[U/ end=/U\\]/ transparent')
  n.command('syntax match rNone /qqq/ contained')
  n.command('syntax match rIn /body/ contained')
  n.command('syntax region rM matchgroup=rMD start=/\\[M/ end=/M\\]/')
  n.command('syntax region rO start=/\\[O/ end=/O\\]/ oneline')
  n.command('syntax match rD /displayonly/ display')
  n.command('hi link rP Comment | hi link rT String | hi link rU Statement')
  n.command('hi link rIn Todo | hi link rM Constant | hi link rMD Error')
  n.command('hi link rO Identifier | hi link rD Special')
  snap(screen, 'syntax-region-trans')
  for l = 1, 7 do
    syn_probe('trans', l, { 1, 8, 13, 18, 24 })
  end
end)

scenario('syntax-offsets', function()
  local screen = start(56, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'a A_S body A_E z',
    'b B_S body B_E z',
    'c C_S body C_E z',
    'd D_M z',
    'e xxE_S body E_E z',
    'f F_S body F_E z',
  })
  -- The seven SPO_* offsets, one per line, each with its own delimiters so a
  -- shifted boundary cannot be confused with a neighbour's.
  n.command('syntax region oA start=/A_S/ms=s+1 end=/A_E/me=e-1')
  n.command('syntax region oB start=/B_S/hs=e end=/B_E/he=s+1')
  n.command('syntax region oC start=/C_S/rs=e end=/C_E/re=s')
  n.command('syntax match oD /D_M/ms=s+1,me=e-1')
  n.command('syntax region oE start=/xxE_S/lc=2 end=/E_E/')
  n.command('syntax region oF matchgroup=oFD start=/F_S/ end=/F_E/me=s-1')
  n.command('hi link oA Comment | hi link oB String | hi link oC Statement')
  n.command('hi link oD Todo | hi link oE Constant | hi link oF Identifier')
  n.command('hi link oFD Error')
  snap(screen, 'syntax-offsets')
  for l = 1, 6 do
    syn_probe('offset', l, { 1, 3, 5, 9, 12, 15 })
  end
end)

scenario('syntax-nextgroup', function()
  local screen = start(50, 16)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'KWPLAIN target one',
    'KWWHITE   target two',
    'KWNL',
    'target three',
    'KWEMPTY',
    '',
    '',
    'target four',
    'KWPLAIN2',
    'target five',
  })
  -- `nextgroup` alone only matches at the very next column; each skip flag
  -- widens that by one shape. The five keywords differ only in flags.
  n.command('syntax keyword ngP KWPLAIN nextgroup=ngT')
  n.command('syntax keyword ngW KWWHITE nextgroup=ngT skipwhite')
  n.command('syntax keyword ngN KWNL nextgroup=ngT skipnl')
  n.command('syntax keyword ngE KWEMPTY nextgroup=ngT skipempty')
  n.command('syntax keyword ngP2 KWPLAIN2 nextgroup=ngT')
  n.command('syntax match ngT /target/ contained')
  n.command('hi link ngP Statement | hi link ngW Statement')
  n.command('hi link ngN Statement | hi link ngE Statement')
  n.command('hi link ngP2 Statement | hi link ngT Todo')
  snap(screen, 'syntax-nextgroup')
  for l = 1, 10 do
    syn_probe('nextgroup', l, { 1, 9, 12 })
  end
end)

scenario('syntax-conceal', function()
  local screen = start(50, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'a <hide>text</hide> b',
    'c [conceal me] d',
    'e KEYWORD f',
    'g <hide></hide> h',
    'i <keep>text</keep> j',
  })
  n.command('syntax region cR matchgroup=cD start=/<hide>/ end=|</hide>| concealends')
  n.command('syntax match cM /\\[conceal me\\]/ conceal cchar=X')
  n.command('syntax keyword cK KEYWORD conceal')
  n.command('syntax region cR2 matchgroup=cD2 start=/<keep>/ end=|</keep>| conceal')
  n.command('hi link cR Comment | hi link cD Error | hi link cM String')
  n.command('hi link cK Statement | hi link cR2 Todo | hi link cD2 Constant')
  for _, lvl in ipairs({ 0, 1, 2, 3 }) do
    n.command('set conceallevel=' .. lvl)
    n.command('set concealcursor=nvic')
    snap(screen, 'syntax-conceal-' .. lvl)
    for l = 1, 5 do
      syn_probe('conceal' .. lvl, l, { 3, 9, 14 })
    end
  end
  -- With the cursor on the line and `concealcursor` empty the line is shown
  -- unconcealed, which is a different path through the drawing side.
  n.command('set conceallevel=2 concealcursor=')
  feed('gg')
  snap(screen, 'syntax-conceal-cursor')
end)

scenario('syntax-sync-deep', function()
  local screen = start(46, 12)
  local lines = {}
  for i = 1, 300 do
    lines[i] = ('body line %d'):format(i)
  end
  -- A region that opens at line 3 and closes at 280: whether line 200 is
  -- inside it is entirely a question of how far back `syn_sync` looked, so
  -- each sync setting below gives a different answer for the same scene.
  lines[3] = 'BEGIN region opens here'
  lines[280] = 'region closes here END'
  lines[100] = '/* c comment opens'
  lines[103] = 'and closes */'
  lines[150] = 'continued line ending with \\'
  lines[151] = 'the continuation'
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('syntax region syReg start=/BEGIN/ end=/END/')
  n.command('syntax region cComment start=+/\\*+ end=+\\*/+')
  n.command('syntax match syMark /region opens here/ contained containedin=syReg')
  n.command('hi link syReg String | hi link cComment Comment | hi link syMark Todo')
  for _, sync in ipairs({
    'syntax sync minlines=5',
    'syntax sync minlines=250',
    'syntax sync maxlines=10 minlines=250',
    'syntax sync fromstart',
    'syntax sync ccomment cComment minlines=250',
    'syntax sync linebreaks=2',
    'syntax sync lines=40',
    'syntax sync linecont /\\\\$/',
    'syntax sync match syGH grouphere syReg /BEGIN/',
    'syntax sync match syGT groupthere NONE /END/',
    'syntax sync region start=/BEGIN/ end=/END/',
    'syntax sync clear',
  }) do
    n.command('syntax sync clear')
    local ok, err = pcall(n.command, sync)
    say(vals, ('%-22s %-58s %s'):format('synccmd', sync, ok and 'OK' or errtext(err)))
    for _, at in ipairs({ 200, 105, 152 }) do
      feed(('%dG'):format(at))
      val(
        ('sync-%d-%s'):format(at, sync:gsub('[^%w]', '')),
        ('map(synstack(%d, 3), \'synIDattr(v:val, "name")\')'):format(at)
      )
    end
    feed('200G')
    snap(screen, 'syntax-' .. sync:gsub('[^%w]+', '-'))
  end
end)

scenario('syntax-sync-boundary', function()
  local screen = start(46, 10)
  local lines = {}
  for i = 1, 400 do
    lines[i] = ('body line %d'):format(i)
  end
  -- Two things the wider `syntax-sync-deep` scene cannot resolve, because in
  -- it every strategy that finds the region start at all agrees about the
  -- probe line. Two regions, two probes, one scene:
  --
  --  * how far `minlines` looks back. The backoff is `minlines * 3/2` (or
  --    `* 2` under ten), and with `sbReg` opening at 185 and never closing,
  --    line 200 is inside it under minlines=20 (back to 170) and outside it
  --    under minlines=5 (back to 190) -- so a wrong multiplier shows.
  --  * `grouphere` versus `groupthere`. `grouphere` resumes parsing AT the
  --    sync match and therefore sees `sbG`'s end at 50; `groupthere` resumes
  --    at the line being drawn with the item already pushed, and never does.
  --    Line 60 is the only probe that can tell them apart.
  lines[40] = 'GBEG opens at 40'
  lines[50] = 'closes GEND at 50'
  lines[185] = 'BEGIN opens at 185'
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('syntax region sbReg start=/BEGIN/ end=/ENDR/')
  n.command('syntax region sbG start=/GBEG/ end=/GEND/')
  n.command('hi link sbReg String | hi link sbG Todo')
  for _, sync in ipairs({
    'syntax sync minlines=5',
    'syntax sync minlines=20',
    'syntax sync minlines=100',
    'syntax sync match sbHere grouphere sbG /GBEG/',
    'syntax sync match sbThere groupthere sbG /GBEG/',
  }) do
    n.command('syntax sync clear')
    n.command(sync)
    local tag = sync:gsub('[^%w]', '')
    -- Probe order matters: visiting 45 first fills the state cache and line
    -- 60 then loads from it instead of running `syn_sync` at all, which is
    -- exactly the difference this scenario exists to see.
    for _, at in ipairs({ 60, 200, 45 }) do
      feed(('%dG'):format(at))
      val(
        ('bound-%d-%s'):format(at, tag),
        ('map(synstack(%d, 3), \'synIDattr(v:val, "name")\')'):format(at)
      )
    end
    feed('60G')
    snap(screen, 'syntax-bound-' .. tag)
  end
end)

scenario('syntax-state-cache', function()
  local screen = start(46, 10)
  local lines = {}
  for i = 1, 400 do
    lines[i] = ('line %03d filler'):format(i)
  end
  for i = 10, 390, 20 do
    lines[i] = ('BEGIN at %d'):format(i)
    lines[i + 8] = ('END at %d'):format(i + 8)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('syntax region scReg start=/BEGIN/ end=/END/')
  n.command('hi link scReg String')
  n.command('syntax sync minlines=3')
  -- The synstate_T cache is only allocated for a buffer with enough lines
  -- (SST_MIN_ENTRIES), and only exercised by revisiting lines out of order
  -- and by invalidating it with an edit in the middle.
  for _, at in ipairs({ 400, 1, 200, 350, 200 }) do
    feed(('%dG'):format(at))
    snap(screen, ('syntax-cache-at-%d'):format(at))
    val(('cache-%d'):format(at), ('synIDattr(synID(%d, 3, 1), "name")'):format(at))
  end
  val('cache-pre-edit', 'map(synstack(105, 3), \'synIDattr(v:val, "name")\')')
  n.api.nvim_buf_set_lines(0, 99, 100, true, { 'BEGIN inserted at 100' })
  feed('105G')
  snap(screen, 'syntax-cache-after-edit')
  val('cache-edit', 'map(synstack(105, 3), \'synIDattr(v:val, "name")\')')
  val('cache-edit-far', 'map(synstack(310, 3), \'synIDattr(v:val, "name")\')')
  -- `silent`: a plain `:undo` echoes "N seconds ago", and the seconds tick
  -- over between two runs of the SAME binary (caught at B12-10 by a
  -- three-run determinism check).
  n.command('silent undo')
  feed('105G')
  snap(screen, 'syntax-cache-after-undo')
  val('cache-undo', 'map(synstack(105, 3), \'synIDattr(v:val, "name")\')')
  n.api.nvim_buf_set_lines(0, 50, 150, true, {})
  feed('150G')
  snap(screen, 'syntax-cache-after-delete')
  val('cache-delete', 'map(synstack(150, 3), \'synIDattr(v:val, "name")\')')
end)

scenario('syntax-cluster', function()
  local screen = start(50, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    '[ alpha beta gamma ]',
    'alpha beta gamma outside',
    '( alpha beta gamma )',
    '{ alpha beta gamma }',
  })
  n.command('syntax match clA /alpha/ contained')
  n.command('syntax match clB /beta/ contained')
  n.command('syntax match clC /gamma/ contained')
  n.command('syntax cluster clAll contains=clA,clB')
  n.command('syntax cluster clAll add=clC')
  n.command('syntax cluster clAll remove=clB')
  n.command('syntax region clOut start=/\\[/ end=/\\]/ contains=@clAll')
  n.command('syntax region clAllBut start=/(/ end=/)/ contains=ALLBUT,clA')
  n.command('syntax region clTop start=/{/ end=/}/ contains=TOP')
  n.command('hi link clA Todo | hi link clB Constant | hi link clC Statement')
  n.command('hi link clOut Comment | hi link clAllBut Directory')
  n.command('hi link clTop Identifier')
  snap(screen, 'syntax-cluster')
  for l = 1, 4 do
    syn_probe('cluster', l, { 3, 9, 14 })
  end
  report_val('clusterlist', 'syntax list @clAll')
  for _, cmd in ipairs({
    'syntax cluster clAll contains=NoSuchGroup',
    'syntax cluster clAll',
    'syntax cluster @clAll',
    'syntax cluster clAll bogus=x',
    'syntax region clBad start=/x/ end=/y/ contains=@NoSuchCluster',
  }) do
    cmd_val('clustererr', cmd)
  end
end)

scenario('syntax-include', function()
  local screen = start(50, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'INCKEY incm outside',
    'HOST INCBEG deep INCEND host',
    'INCBEG deep INCEND at toplevel',
  })
  -- `:syntax include` pushes an inclusion tag, so the included file's
  -- toplevel items take HL_INCLUDED_TOPLEVEL and are only reachable through
  -- the cluster it was given -- or through `contains=TOP`.
  n.command('syntax include @Inc ' .. fixdir .. '/fixture.syn')
  n.command('syntax region hostReg start=/HOST/ end=/host/ contains=@Inc')
  n.command('hi link hostReg Search')
  snap(screen, 'syntax-include')
  for l = 1, 3 do
    syn_probe('include', l, { 1, 6, 12, 18 })
  end
  report_val('includelist', 'syntax list @Inc')
end)

scenario('syntax-spell', function()
  local screen = start(50, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'plain wrng words here',
    '[ wrng inside region ]',
    '( wrng inside nospell )',
    '{ wrng inside spell }',
  })
  n.command('set spell spelllang=en_us')
  n.command('syntax region spDef start=/\\[/ end=/\\]/')
  n.command('syntax region spNo start=/(/ end=/)/ contains=@NoSpell')
  n.command('syntax region spYes start=/{/ end=/}/ contains=@Spell')
  n.command('hi link spDef Comment | hi link spNo String | hi link spYes Statement')
  for _, mode in ipairs({ 'default', 'toplevel', 'notoplevel' }) do
    n.command('syntax spell ' .. mode)
    n.command('syntax sync fromstart')
    snap(screen, 'syntax-spell-' .. mode)
  end
  cmd_val('spellerr', 'syntax spell bogus')
end)

scenario('syntax-foldlevel', function()
  local screen = start(50, 14)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'top',
    'OPEN outer',
    'OPEN inner',
    'CLOSE inner OPEN again',
    'CLOSE again',
    'CLOSE outer',
    'bottom',
  })
  n.command('syntax region fReg start=/OPEN/ end=/CLOSE/ fold contains=fReg')
  n.command('hi link fReg Comment')
  n.command('set foldmethod=syntax foldcolumn=3 foldlevel=9')
  for _, mode in ipairs({ 'start', 'minimum' }) do
    n.command('syntax foldlevel ' .. mode)
    n.command('syntax sync fromstart')
    n.command('normal! zx')
    snap(screen, 'syntax-foldlevel-' .. mode)
    for l = 1, 7 do
      val(('foldlevel-%s-%d'):format(mode, l), ('foldlevel(%d)'):format(l))
    end
  end
  report_val('foldlevelq', 'syntax foldlevel')
  cmd_val('foldlevelerr', 'syntax foldlevel bogus')
end)

scenario('syntax-case-iskeyword', function()
  local screen = start(50, 10)
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'Alpha alpha ALPHA',
    'be-ta be ta',
  })
  n.command('syntax case ignore')
  n.command('syntax keyword ciA Alpha')
  n.command('syntax case match')
  n.command('syntax keyword ciB be-ta')
  n.command('syntax iskeyword @,48-57,_,192-255,-')
  n.command('syntax keyword ciC be-ta')
  n.command('hi link ciA Todo | hi link ciB Constant | hi link ciC Statement')
  snap(screen, 'syntax-case-iskeyword')
  for l = 1, 2 do
    syn_probe('caseik', l, { 1, 7, 13 })
  end
  report_val('iskeywordq', 'syntax iskeyword')
  report_val('caseq', 'syntax case')
  cmd_val('caseerr', 'syntax case bogus')
  cmd_val('ikerr', 'syntax iskeyword bogus!!')
end)

scenario('syntax-listing', function()
  local screen = start(60, 12)
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'KEY body END' })
  n.command('syntax keyword liKey KEY nextgroup=liNext skipwhite')
  n.command('syntax match liNext /body/ contained containedin=liReg')
  n.command('syntax region liReg matchgroup=liDelim start=/KEY/ end=/END/ keepend fold')
  n.command('syntax cluster liCl contains=liKey,liNext')
  n.command('syntax sync match liSync grouphere liReg /KEY/')
  n.command('syntax sync minlines=7 maxlines=9 linebreaks=1')
  n.command('hi link liKey Statement | hi link liNext Todo | hi link liReg Comment')
  snap(screen, 'syntax-listing')
  report_val('list-all', 'syntax')
  report_val('list-one', 'syntax list liReg')
  report_val('list-kw', 'syntax list liKey')
  report_val('list-cl', 'syntax list @liCl')
  report_val('list-sync', 'syntax sync')
  report_val('list-lsync', 'syntax list Xsync')
end)

scenario('syntax-syntime', function()
  local screen = start(50, 10)
  local lines = {}
  for i = 1, 60 do
    lines[i] = ('KEY %d body %d END'):format(i, i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('syntax keyword stKey KEY')
  n.command('syntax region stReg start=/body/ end=/END/')
  n.command('hi link stKey Statement | hi link stReg Comment')
  -- The report's numbers are wall-clock; only its shape is an oracle, so
  -- every digit is masked. What that still pins is the column layout, the
  -- ordering key, the row count and the pattern text.
  -- Every digit run collapses to one `#` and every whitespace run to one
  -- space: a count of 9 and a count of 10 are printed in the same fixed-width
  -- column, so masking the digits alone would leave the *padding* varying.
  local mask = function(s)
    return (s:gsub('%d+', '#'):gsub('%s+', ' '))
  end
  n.command('syntime on')
  feed('G')
  snap(screen, 'syntax-syntime')
  feed('gg')
  -- `sortbody`: the report is ordered by measured time and the two patterns
  -- tie, so rows [2] and [3] swapped between runs of the same binary.
  report_val('syntime-report', 'syntime report', mask, true)
  n.command('syntime clear')
  report_val('syntime-cleared', 'syntime report', mask, true)
  n.command('syntime off')
  for _, cmd in ipairs({ 'syntime bogus', 'syntime', 'syntime report extra' }) do
    cmd_val('syntimeerr', cmd)
  end
  for _, pat in ipairs({ 'syntime ', 'syntax sy', 'syntax list li', 'syntax cluster ' }) do
    val('syncomplete', ("getcompletion('%s', 'cmdline')"):format(pat))
  end
end)

scenario('syntax-onoff', function()
  local screen = start(50, 10)
  n.command('edit ' .. fixdir .. '/fixture.c')
  for _, cmd in ipairs({
    'syntax on',
    'syntax off',
    'syntax enable',
    'syntax manual',
    'syntax reset',
    'syntax clear',
  }) do
    cmd_val('onoff', cmd)
    snap(screen, 'syntax-onoff-' .. cmd:gsub('[^%w]+', '-'))
    val('onoff-' .. cmd:gsub('[^%w]+', '-'), 'synIDattr(synID(1, 2, 1), "name")')
  end
  n.command('syntax on')
  n.command('ownsyntax lua')
  snap(screen, 'syntax-ownsyntax')
  val('ownsyntax', 'synIDattr(synID(2, 2, 1), "name")')
  val('ownsyntax-w', 'w:current_syntax')
  cmd_val('ownsyntaxerr', 'ownsyntax')
end)

-- ------------------------------------------------------- everything at once

scenario('mixed', function()
  local screen = start(52, 14)
  local lines = {}
  for i = 1, 20 do
    lines[i] = ('mixed KEY line %02d with target text'):format(i)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set number relativenumber cursorline foldcolumn=2 signcolumn=yes list')
  n.command('set listchars=tab:>-,trail:~ conceallevel=2 colorcolumn=45 hlsearch')
  n.command('sign define MX text=!! texthl=Error')
  n.command('sign place 1 line=4 name=MX buffer=1')
  n.command('syntax keyword mxKey KEY')
  n.command('syntax match mxHide /with/ conceal cchar=~')
  n.command('hi link mxKey Statement')
  n.command('8,12fold')
  n.command('call matchadd("Search", "target")')
  local ns = n.api.nvim_create_namespace('mx')
  n.api.nvim_buf_set_extmark(0, ns, 2, 0, { virt_text = { { '<<v', 'Todo' } } })
  n.api.nvim_buf_set_extmark(0, ns, 5, 0, { virt_lines = { { { 'virt line', 'DiffAdd' } } } })
  n.feed('/line<CR>')
  n.feed('3G')
  snap(screen, 'mixed')
end)


-- ------------------------------------------------- ext_ UI (B13-3)
--
-- Twelve scenarios that turn the `ext_` UI options ON.  Everything else
-- in this file runs the default single-grid UI, which means the eight
-- `cmdline_*` and five `msg_*` `ui_call_*` emitters in ex_getln.rs and
-- message.rs were observed by nothing at all -- they are only reached
-- when a UI advertises the capability.  These are B13-5/6 and
-- B13-11/12's only per-cell oracle, because the screendump oldtests
-- that used to cover them are all skipped in this port.
--
-- Two rules learned the hard way and baked in here:
--
--  * `snap()` had to be widened to all fourteen ext_keys FIRST.  With
--    the six-key dump, `ext_cmdline` scenarios recorded `cmdline` and
--    silently dropped `cmdline_block`, and `ext_messages` dropped
--    `msg_history`/`showmode`/`showcmd`/`ruler` -- every scenario would
--    have looked like it passed while asserting a third of its payload.
--  * With `ext_messages` on, a hit-enter prompt does NOT block the
--    harness (the message goes to the UI as an event instead), which is
--    what makes `wait_return` reachable here and nowhere else in the
--    B13 corpus -- keysweep's pager section can only get at
--    `do_more_prompt`, whose headless guard exempts embedded mode while
--    `wait_return`'s does not.

scenario('ext-cmdline', function()
  local screen = start(50, 8, { ext_cmdline = true })
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'alpha beta', 'gamma delta' })
  feed(':')
  snap(screen, 'ext-cmdline-empty')
  feed('let g:x = 1')
  snap(screen, 'ext-cmdline-typed')
  feed('<Left><Left>')
  snap(screen, 'ext-cmdline-cursor')
  feed('<CR>')
  snap(screen, 'ext-cmdline-done')
  feed('/gam')
  snap(screen, 'ext-cmdline-search')
  feed('<Esc>')
  feed(':')
  feed('echo "α β"')
  snap(screen, 'ext-cmdline-utf8')
  feed('<Esc>')
end)

scenario('ext-cmdline-nested', function()
  local screen = start(50, 8, { ext_cmdline = true })
  -- `<C-r>=` raises `cmdlevel`, which is the field the protocol carries
  -- and the reason `cmdline` is an *array* of cmdlines.
  feed(':let g:x = ')
  snap(screen, 'ext-cmdline-nested-outer')
  feed('<C-r>=')
  snap(screen, 'ext-cmdline-nested-inner')
  feed('1+2')
  snap(screen, 'ext-cmdline-nested-typed')
  feed('<CR>')
  snap(screen, 'ext-cmdline-nested-popped')
  feed('<CR>')
  snap(screen, 'ext-cmdline-nested-done')
end)

scenario('ext-cmdline-block', function()
  local screen = start(50, 8, { ext_cmdline = true })
  -- A multi-line `:if`/`:endif` is the `cmdline_block_show`/`_append`/
  -- `_hide` triple, and `cmdline_block` is one of the eight keys the
  -- old six-key dump dropped.
  feed(':if 1<CR>')
  snap(screen, 'ext-cmdline-block-open')
  feed('echo "in"<CR>')
  snap(screen, 'ext-cmdline-block-append')
  feed('endif<CR>')
  snap(screen, 'ext-cmdline-block-closed')
  feed(':function! Foo()<CR>')
  snap(screen, 'ext-cmdline-block-func')
  feed('return 1<CR>endfunction<CR>')
  snap(screen, 'ext-cmdline-block-func-done')
end)

scenario('ext-cmdline-special', function()
  local screen = start(50, 8, { ext_cmdline = true })
  -- `cmdline_special_char` (the `<C-v>`/`<C-k>` pending state) and the
  -- `'*'` masking `inputsecret()` turns on.
  -- NO snapshot between `<C-v>` and its argument.  `get_literal()` runs
  -- a `vgetc` that does not service RPC, so `snap`'s `poke_eventloop()`
  -- never returns while the cmdline is holding a pending special char --
  -- measured: the harness timed out at 60 s with the child sitting there.
  -- Each sequence is therefore fed whole and snapshotted at rest, which
  -- costs the `cmdline_special_char` event itself and keeps its effect.
  feed(':let g:x = "')
  snap(screen, 'ext-cmdline-special-before')
  feed('<C-v>065')
  snap(screen, 'ext-cmdline-special-decimal')
  feed('<C-k>Co')
  snap(screen, 'ext-cmdline-special-digraph')
  feed('<C-v><Tab>')
  snap(screen, 'ext-cmdline-special-literal-tab')
  feed('<Esc>')
  -- `inputsecret()` is NOT here.  Its `'*'` masking is the thing worth
  -- pinning, but it holds its own key loop and pre-queueing the answer
  -- with `feedkeys(..., 'n')` does not release it from this harness --
  -- the `nvim_command` request simply never returns (measured, 90 s).
  -- Same class as the pager: it belongs to a probe that does not need
  -- the request to come back.  `cmdline-api`'s `inputsecret-typed` case
  -- in keysweep covers the value it returns.
end)

scenario('ext-cmdline-prompt', function()
  local screen = start(50, 8, { ext_cmdline = true })
  -- `input()` sets `prompt` and `indent`; `getcmdprompt` is the only
  -- other view of it and it is not a screen view.
  n.feed(':call input("Say: ", "pre")<CR>')
  n.poke_eventloop()
  snap(screen, 'ext-cmdline-prompt-shown')
  feed('X')
  snap(screen, 'ext-cmdline-prompt-typed')
  feed('<CR>')
  snap(screen, 'ext-cmdline-prompt-done')
  n.feed(':call confirm("Really?", "&yes\\n&no", 1)<CR>')
  n.poke_eventloop()
  snap(screen, 'ext-cmdline-confirm')
  feed('y')
  snap(screen, 'ext-cmdline-confirm-done')
end)

scenario('ext-cmdline-inccommand', function()
  local screen = start(50, 8, { ext_cmdline = true })
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alpha beta alpha', 'gamma alpha', 'delta',
  })
  n.command('set inccommand=nosplit')
  feed(':%s/alpha/OMEGA')
  snap(screen, 'ext-cmdline-inccommand-preview')
  n.command('set inccommand=split')
  feed('<Esc>')
  feed(':%s/alpha/OMEGA')
  snap(screen, 'ext-cmdline-inccommand-split')
  feed('<CR>')
  snap(screen, 'ext-cmdline-inccommand-applied')
end)

scenario('ext-cmdline-highlight', function()
  local screen = start(50, 8, { ext_cmdline = true })
  -- A `Cmdline_highlight`-shaped callback: the `cmdline_show`
  -- `content` array is [[attrs, text], ...], and a highlighter is the
  -- only thing that makes it longer than one chunk.
  n.exec_lua([[
    vim.api.nvim_create_autocmd('CmdlineChanged', {
      callback = function()
        vim.g.seen = (vim.g.seen or 0) + 1
      end,
    })
  ]])
  n.command('set incsearch hlsearch')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'one two three', 'two three four' })
  feed(':/two')
  snap(screen, 'ext-cmdline-hl-incsearch')
  feed('<Esc>')
  val('ext-cmdline-hl-seen', 'get(g:, "seen", -1)')
end)

scenario('ext-messages', function()
  local screen = start(50, 8, { ext_messages = true })
  -- Each `kind` is its own `msg_show` call site: "" for :echo,
  -- "echomsg", "echoerr", "emsg", "wmsg", "return_prompt".
  feed(':echo "plain"<CR>')
  snap(screen, 'ext-messages-echo')
  feed(':echomsg "kept"<CR>')
  snap(screen, 'ext-messages-echomsg')
  feed(':echoerr "bad"<CR>')
  snap(screen, 'ext-messages-echoerr')
  feed('<CR>')
  feed(':nosuchcommand<CR>')
  snap(screen, 'ext-messages-emsg')
  feed('<CR>')
  feed(':echohl WarningMsg | echomsg "warned" | echohl None<CR>')
  snap(screen, 'ext-messages-warning')
  feed(':echo "a\\nb\\nc"<CR>')
  snap(screen, 'ext-messages-multiline')
end)

scenario('ext-messages-modes', function()
  local screen = start(50, 8, { ext_messages = true })
  -- `msg_showmode`, `msg_showcmd` and `msg_ruler` are three separate
  -- emitters and three of the keys the old dump dropped.
  n.command('set showmode showcmd ruler')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'alpha beta gamma', 'second' })
  feed('i')
  snap(screen, 'ext-messages-showmode-insert')
  feed('<Esc>')
  feed('R')
  snap(screen, 'ext-messages-showmode-replace')
  feed('<Esc>')
  feed('V')
  snap(screen, 'ext-messages-showmode-visual')
  feed('<Esc>')
  feed('2d')
  snap(screen, 'ext-messages-showcmd-pending')
  feed('<Esc>')
  feed('G')
  snap(screen, 'ext-messages-ruler')
  n.command('set noshowmode noshowcmd noruler')
  feed('i')
  snap(screen, 'ext-messages-modes-off')
  feed('<Esc>')
end)

scenario('ext-messages-history', function()
  local screen = start(50, 8, { ext_messages = true })
  -- `:messages` under ext_messages is `msg_history_show`, a single
  -- event carrying the whole history -- a completely different emitter
  -- from the one that printed each message.
  for i = 1, 5 do
    n.command(('echomsg "hist %d"'):format(i))
  end
  feed(':messages<CR>')
  snap(screen, 'ext-messages-history-shown')
  feed(':messages clear<CR>')
  feed(':messages<CR>')
  snap(screen, 'ext-messages-history-cleared')
  n.command('set messagesopt=hit-enter,history:2')
  for i = 1, 5 do
    n.command(('echomsg "cap %d"'):format(i))
  end
  feed(':messages<CR>')
  snap(screen, 'ext-messages-history-capped')
end)

-- `ext-messages-more` is DELETED, not disabled, and the reason is worth
-- keeping: with `ext_messages` on and a UI attached, a `'more'`-length
-- message wedges this harness even with the answer queued by
-- `feedkeys(..., 'n')` before the command (the popupprobe recipe, which
-- works for `:popup` and for `inputsecret()` here).  The `nvim_command`
-- request never returns.  So the pager stays with keysweep's
-- `messages-pager` section, which drives an `--embed` child and samples
-- it with `fast` requests only -- and `wait_return` under `ext_messages`
-- is a genuine B13 blind spot, recorded in b13.md rather than papered
-- over with a scenario that hangs.

scenario('ext-popupmenu', function()
  local screen = start(50, 10, { ext_popupmenu = true })
  -- With ext_popupmenu the pum is an event, not cells: `popupmenu_show`
  -- carries the item array and `popupmenu_select` the index, and the
  -- grid stays clean.  That is the half `pum-*` cannot see.
  n.api.nvim_buf_set_lines(0, 0, -1, true, {
    'alpha alphabet alphanumeric', 'beta betamax', 'gamma', '',
  })
  n.command('set completeopt=menu,menuone,noselect')
  n.api.nvim_win_set_cursor(0, { 4, 0 })
  feed('ialp<C-n>')
  snap(screen, 'ext-popupmenu-shown')
  feed('<C-n>')
  snap(screen, 'ext-popupmenu-selected')
  feed('<C-n>')
  snap(screen, 'ext-popupmenu-selected2')
  feed('<C-e>')
  snap(screen, 'ext-popupmenu-cancelled')
  feed('<Esc>')
end)

scenario('ext-wildmenu', function()
  local screen = start(50, 10, { ext_wildmenu = true })
  -- `wildmenu_show`/`_select`/`_hide` fill `wildmenu_items` and
  -- `wildmenu_pos`, both of which the old six-key dump discarded.
  n.command('set wildmenu wildmode=full')
  feed(':sil')
  snap(screen, 'ext-wildmenu-before')
  feed('<Tab>')
  snap(screen, 'ext-wildmenu-first')
  feed('<Tab>')
  snap(screen, 'ext-wildmenu-second')
  feed('<S-Tab>')
  snap(screen, 'ext-wildmenu-back')
  feed('<Esc>')
  snap(screen, 'ext-wildmenu-hidden')
end)

scenario('ext-cmdline-wildmenu-pum', function()
  -- ext_cmdline + ext_popupmenu together: the combination that makes
  -- `cmdline_compl_use_pum` answer differently, and the one real
  -- interop path between cmdexpand.rs and the pum emitters.
  local screen = start(50, 10, { ext_cmdline = true, ext_popupmenu = true })
  n.command('set wildmenu wildoptions=pum wildmode=full')
  feed(':sil')
  snap(screen, 'ext-cmdpum-before')
  feed('<Tab>')
  snap(screen, 'ext-cmdpum-shown')
  feed('<Tab>')
  snap(screen, 'ext-cmdpum-next')
  feed('<C-e>')
  snap(screen, 'ext-cmdpum-cancelled')
  feed('<Esc>')
end)

scenario('ext-messages-cmdline-combined', function()
  -- All three at once, which is what a real GUI attaches with, and the
  -- configuration in which `msg_showmode` is suppressed in favour of
  -- the cmdline events.
  local screen = start(50, 10, {
    ext_cmdline = true,
    ext_messages = true,
    ext_popupmenu = true,
  })
  n.command('set showmode showcmd ruler')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'alpha beta', 'gamma' })
  feed('i')
  snap(screen, 'ext-combined-insert')
  feed('<Esc>')
  feed(':echomsg "combined"<CR>')
  snap(screen, 'ext-combined-message')
  feed(':nosuchcommandhere<CR>')
  snap(screen, 'ext-combined-error')
  feed('<CR>')
  feed(':sil<Tab>')
  snap(screen, 'ext-combined-wild')
  feed('<Esc>')
end)


-- --------------------------------------------------------- cursor_shape.rs
--
-- B19-4.  `'guicursor'` appeared in NO differential before this: the whole
-- of `parse_shape_opt` (233 lines) and `cursor_get_mode_idx` were behind
-- `ui/cursor_spec` (4 `it`s) and `ui/mode_spec`.
--
-- The lever is the `mode_info_set` UI event, which is `mode_style_array`'s
-- dump of the ENTIRE `shape_table` -- every one of the eighteen entries with
-- its shape, percentage, three blink timings and both highlight ids. The
-- Screen harness records it in `_mode_info`, so one option assignment answers
-- the whole table. `mode_change` carries the resolved INDEX, and the harness
-- asserts its name against that same table, so feeding a mode is a direct
-- read of `cursor_get_mode_idx`.

scenario('screen-guicursor', function()
  local screen = start(40, 8)

  --- The whole shape table, one line per entry, in table order.
  ---
  --- Not `vim.inspect(mode_info)` as one blob: eighteen entries on one line
  --- is a 2 kB row where a single changed field is unreadable in a diff, and
  --- every entry moves when one does.
  local function shapes(tag)
    -- The Screen only sees a UI event when the session is pumped, and
    -- `mode_info_set` arrives once per `:set guicursor`, not on a redraw --
    -- without this every row read `n=0` and looked like the option had no
    -- effect at all.
    n.poke_eventloop()
    screen:sleep(5)
    local info = screen._mode_info or {}
    say(vals, ('%-22s %-58s %s'):format(
      'gcur-' .. tag,
      'cursor_style_enabled',
      tostring(screen._cursor_style_enabled) .. ' n=' .. tostring(#info)
    ))
    for i, item in ipairs(info) do
      local keys = {}
      for k in pairs(item) do
        keys[#keys + 1] = k
      end
      table.sort(keys)
      local parts = {}
      for _, k in ipairs(keys) do
        -- `attr`/`attr_lm` are TABLES: screen.lua resolves the attribute id
        -- into the definition, and `tostring` on one is `table: 0x...`, an
        -- ASLR address that differs between two runs of the same binary.
        local v = item[k]
        parts[#parts + 1] =
          ('%s=%s'):format(k, type(v) == 'table' and attr_repr(v) or tostring(v))
      end
      say(vals, ('%-22s %-58s %s'):format(
        ('gcur-%s[%02d]'):format(tag, i),
        'mode_info',
        table.concat(parts, ' ')
      ))
    end
  end

  -- The default, before anything touches the option: this row is the only
  -- record anywhere of what nvim ships.
  shapes('default')

  local VALUES = {
    { 'all-block', 'a:block' },
    { 'all-hor', 'a:hor10' },
    { 'all-ver', 'a:ver35' },
    { 'pct-1', 'a:ver1' },
    { 'pct-100', 'a:hor100' },
    { 'blink', 'a:blinkwait700-blinkoff400-blinkon250' },
    { 'blinkon0', 'a:block-blinkon0' },
    { 'hl-one', 'n:block-Cursor' },
    { 'hl-two', 'n:block-Cursor/lCursor' },
    { 'per-mode', 'n-v-c:block,i-ci:ver25,r-cr:hor20,o:hor50' },
    { 'sm', 'sm:block-blinkwait175-blinkoff150-blinkon175' },
    { 'term', 't:ver25-TermCursor' },
    { 've', 've:ver35-Cursor' },
    { 'a-then-n', 'a:hor10,n:ver90' },
    { 'n-then-a', 'n:ver90,a:hor10' },
    { 'empty', '' },
    -- `parse_shape_opt` runs twice over the option: once to validate, once
    -- to apply. A value that is legal in the first pass and not the second
    -- would leave the table half-written, so a bad tail after a good head
    -- is its own case.
    { 'good-then-bad', 'n:block,i:nosuchshape' },
  }
  for _, v in ipairs(VALUES) do
    cmd_val('gcurset-' .. v[1], ('set guicursor=%s'):format(v[2]))
    shapes(v[1])
  end

  -- The four error texts, each with the option left at its previous value.
  n.command('set guicursor=a:block')
  for _, bad in ipairs({
    'a',
    'a:',
    ':block',
    'x:block',
    'a:nosuch',
    'a:ver',
    'a:ver0',
    'a:hor101',
    'a:ver999',
    'a:block-blink',
    'a:block-blinkonx',
    'a:block-blinkon',
    'n-:block',
    'n-x:block',
    'a:block,',
    ',a:block',
    'a:block-',
    'a:block-Cursor/',
    'a:block-/lCursor',
    'a:0',
  }) do
    cmd_val('gcurerr-' .. bad, ('set guicursor=%s'):format(bad))
  end
  val('gcur-after-errors', "&guicursor")
  shapes('after-errors')

  -- `cursor_get_mode_idx`, read through the `mode_change` event's index --
  -- the harness asserts the name it carries against `_mode_info`, so the
  -- name IS the index. Every arm of the function is here except the
  -- terminal one, which needs a job.
  n.command('set guicursor=a:block')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'alpha beta', 'gamma delta', 'epsilon' })
  local function modeidx(tag, keys)
    if keys ~= '' then
      feed(keys)
    end
    -- `mode_change` is a UI event like any other and the Screen only sees
    -- it when the session is pumped; `feed` alone left every row reading
    -- `normal`.
    n.poke_eventloop()
    screen:sleep(5)
    say(vals, ('%-22s %-58s %s'):format(
      'gcur-mode-' .. tag,
      'mode_change idx name',
      -- `mode()` answers a RAW CTRL-V for blockwise Visual, which would put
      -- a control byte in the artifact and make `diff` call it binary.
      tostring(screen.mode)
        .. ' / '
        .. (tostring(n.api.nvim_get_mode().mode):gsub('%c', function(ch)
          return ('<%02x>'):format(ch:byte())
        end))
    ))
  end
  modeidx('normal', '<Esc>gg0')
  modeidx('insert', 'i')
  modeidx('replace', '<Esc>R')
  modeidx('vreplace', '<Esc>gR')
  modeidx('visual', '<Esc>v')
  modeidx('vline', '<Esc>V')
  modeidx('vblock', '<Esc><C-v>')
  modeidx('select', '<Esc>gh')
  modeidx('operator', '<Esc>d')
  modeidx('cmdline', '<Esc>:')
  modeidx('cmdline-insert', '<Left>')
  modeidx('cmdline-replace', '<Insert>')
  modeidx('normal-again', '<Esc>')
  -- `'selection'` decides between the `v` and `ve` entries, and nothing else
  -- reads `SHAPE_IDX_VE`.
  n.command('set selection=exclusive')
  modeidx('visual-exclusive', 'v')
  n.command('set selection=inclusive')
  modeidx('visual-inclusive', '<Esc>v')
  feed('<Esc>')

  -- `cursor_is_block_during_visual` and `cursor_mode_uses_syn_id` are the
  -- two readers of the table that are not the UI event. The first decides
  -- whether the selection highlight covers the cursor cell; the second
  -- whether a `:hi Cursor` change forces a redraw.
  for _, gc in ipairs({ 'a:block', 'a:block-blinkon0', 'v:ver25', 've:hor20-Cursor' }) do
    n.command('set guicursor=' .. gc)
    val('gcur-block-' .. gc, '[&guicursor]')
    n.command('set selection=exclusive')
    feed('<Esc>vll')
    snap(screen, 'screen-guicursor-' .. gc:gsub('[^%w]', '-') .. '-excl')
    feed('<Esc>')
    n.command('set selection=inclusive')
  end
  n.command('set guicursor&')
  val('gcur-reset', '&guicursor')
  shapes('reset')
end)

-- ------------------------------------------------------------------ move.rs
--
-- B19-4.  Nothing NAMED `update_topline`, `scroll_cursor_bot`/`_halfway`/
-- `_top`, `curs_columns` or `textpos2screenpos` before this: move.rs was
-- covered only sideways, through whatever redraw a keysweep motion happened
-- to cause. These scenarios are TEXTUAL -- `winsaveview()` plus `line('w0')`
-- / `line('w$')` plus `screenpos()` after each motion -- because the viewport
-- arithmetic is what B19-9/10 rewrite, and a grid snapshot only shows it
-- when a cell actually changes.

--- One viewport reading. `winsaveview()` carries topline, leftcol, skipcol,
--- topfill, curswant and coladd; `w0`/`w$` are the arithmetic's own answer
--- for the first and last visible line; `screenpos()` is `textpos2screenpos`
--- -> `curs_columns` for the cursor itself.
local function view_val(name)
  val(
    name,
    '[winsaveview(),line("w0"),line("w$"),screenpos(0,line("."),col(".")),'
      .. 'line("."),col("."),virtcol("."),winline(),wincol()]'
  )
end

--- A 200-line buffer whose text says which line it is, wide enough to wrap
--- three times at 40 columns on every fifth line.
local function view_fixture()
  local lines = {}
  for i = 1, 200 do
    if i % 5 == 0 then
      lines[i] = ('L%03d '):format(i) .. string.rep('wide ', 24)
    else
      lines[i] = ('L%03d short'):format(i)
    end
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
end

--- The motion corpus: forty keys that between them reach every arm of
--- `update_topline` (cursor above/below, one line out, more than half a
--- screen out), both `scroll_cursor_top` and `scroll_cursor_bot`,
--- `scroll_cursor_halfway`, `pagescroll` and `curs_columns`.
local VIEW_MOTIONS = {
  { 'gg', 'gg' },
  { 'G', 'G' },
  { 'mid', '100G' },
  { 'j', 'j' },
  { 'k', 'k' },
  { 'ctrl-e', '<C-e>' },
  { 'ctrl-y', '<C-y>' },
  { 'ctrl-d', '<C-d>' },
  { 'ctrl-u', '<C-u>' },
  { 'ctrl-f', '<C-f>' },
  { 'ctrl-b', '<C-b>' },
  { 'zt', 'zt' },
  { 'zz', 'zz' },
  { 'zb', 'zb' },
  { 'z-enter', 'z<CR>' },
  { 'z-dot', 'z.' },
  { 'z-minus', 'z-' },
  { 'H', 'H' },
  { 'M', 'M' },
  { 'L', 'L' },
  { 'plus5j', '5j' },
  { 'minus5k', '5k' },
  { 'plus20j', '20j' },
  { 'minus20k', '20k' },
  { 'gj', 'gj' },
  { 'gk', 'gk' },
  { 'dollar', '$' },
  { 'zero', '0' },
  { 'caret', '^' },
  { 'g-dollar', 'g$' },
  { 'g-zero', 'g0' },
  { 'gm', 'gm' },
  { 'bar60', '60|' },
  { 'w10', '10w' },
  { 'b5', '5b' },
  { 'brace', '}' },
  { 'search', '/L150<CR>' },
  { 'nsearch', 'n' },
  { 'ctrl-o', '<C-o>' },
  { 'ctrl-i', '<C-i>' },
}

scenario('view-motions', function()
  local screen = start(40, 12)
  view_fixture()
  n.command('set scrolloff=0 sidescrolloff=0 scrolljump=1 wrap nosmoothscroll')
  -- `nomore` and `shortmess+=s` are not tidiness. A search that hits the
  -- end of the buffer, or a motion that scrolls the message area, raises a
  -- hit-enter prompt in the child -- and a hit-enter prompt blocks the main
  -- loop, so the next `nvim_eval` this scenario issues never returns and the
  -- whole sweep wedges until its timeout. The first draft of `view-motions`
  -- hung at exactly that.
  n.command('set nostartofline wrapscan nomore shortmess+=sI report=9999')
  feed('gg')
  view_val('view-start')
  for _, m in ipairs(VIEW_MOTIONS) do
    feed(m[2])
    view_val('view-m-' .. m[1])
  end
  snap(screen, 'view-motions-end')
  -- The two `w0`/`w$` extremes on their own: `update_topline`'s "cursor is
  -- exactly one line off screen" arm scrolls by one, and its "more than half
  -- a screen off" arm re-centres, and the difference between them is the
  -- only thing that distinguishes `scroll_cursor_bot` from
  -- `scroll_cursor_halfway`.
  for _, step in ipairs({ 1, 2, 6, 11, 12, 13, 24, 100 }) do
    feed('gg')
    feed(('%dj'):format(step))
    view_val(('view-step-down-%d'):format(step))
    feed('G')
    feed(('%dk'):format(step))
    view_val(('view-step-up-%d'):format(step))
  end
end)

scenario('view-opts', function()
  local screen = start(40, 12)
  view_fixture()
  n.command('set nostartofline wrapscan nomore shortmess+=sI report=9999')
  -- A subset of the corpus, run against every option spread. The full forty
  -- against six spreads would be 240 rows of mostly the same answer; these
  -- fourteen are the ones whose answer the option can change.
  local SUB = {
    'gg',
    'G',
    '100G',
    'j',
    'k',
    '<C-e>',
    '<C-y>',
    '<C-d>',
    '<C-u>',
    '<C-f>',
    '<C-b>',
    'zt',
    'zz',
    'zb',
    'H',
    'L',
    '20j',
    '20k',
  }
  local SPREADS = {
    { 'so0', 'set scrolloff=0 scrolljump=1 wrap' },
    { 'so1', 'set scrolloff=1 scrolljump=1 wrap' },
    { 'so3', 'set scrolloff=3 scrolljump=1 wrap' },
    { 'so999', 'set scrolloff=999 scrolljump=1 wrap' },
    { 'sj5', 'set scrolloff=0 scrolljump=5 wrap' },
    { 'sj-50', 'set scrolloff=0 scrolljump=-50 wrap' },
    { 'nowrap', 'set scrolloff=0 scrolljump=1 nowrap sidescroll=0 sidescrolloff=0' },
    { 'smooth', 'set scrolloff=0 scrolljump=1 wrap smoothscroll' },
  }
  for _, sp in ipairs(SPREADS) do
    n.command(sp[2])
    feed('gg0')
    for i, keys in ipairs(SUB) do
      feed(keys)
      view_val(('view-%s-%02d-%s'):format(sp[1], i, keys:gsub('[^%w]', '')))
    end
    -- THE ONE-LINE-PAST-THE-EDGE WALK, and it is what makes this
    -- scenario gate `'scrolljump'` at all.
    --
    -- `scrolljump_value` is read in exactly two places -- the
    -- `scroll_cursor_top` call for a cursor one line ABOVE the viewport
    -- and the `scroll_cursor_bot` call for one line BELOW -- and neither
    -- is reached by an explicit scroll (`<C-e>`, `zt`, `<C-d>`) or by a
    -- jump big enough to re-centre. The eighteen motions above never put
    -- the cursor exactly one line out, so a mutant that pinned
    -- `'scrolljump'` to 1 survived every one of them. `zt`, `L`, `j` is
    -- the shortest sequence that does not.
    feed('50Gzt')
    view_val(('view-%s-edge-00-zt'):format(sp[1]))
    feed('L')
    for i = 1, 3 do
      feed('j')
      view_val(('view-%s-edge-dn-%d'):format(sp[1], i))
    end
    feed('H')
    for i = 1, 3 do
      feed('k')
      view_val(('view-%s-edge-up-%d'):format(sp[1], i))
    end
    snap(screen, 'view-opts-' .. sp[1])
  end
  n.command('set scrolloff=0 scrolljump=1 wrap nosmoothscroll')
end)

scenario('view-horizontal', function()
  local screen = start(40, 8)
  -- `curs_columns` is the horizontal half and it has no `winsaveview()`
  -- field of its own except `leftcol`: what it decides is `wincol()`, the
  -- `w_wcol`/`w_virtcol` pair `screenpos()` reports, and whether `@@@` is
  -- drawn at the edge.
  local lines = {}
  for i = 1, 20 do
    lines[i] = ('H%02d '):format(i) .. string.rep('0123456789', 20)
  end
  lines[5] = 'H05 \u{65e5}\u{672c}\u{8a9e}' .. string.rep('\u{ff21}', 60)
  lines[7] = 'H07 \t\ttabbed\ttext\there'
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set nomore shortmess+=sI report=9999 nostartofline')
  local COLS = { '0', '$', '20|', '40|', '41|', '80|', '120|', '199|', '200|', 'g0', 'g$', 'gm' }
  local HSPREADS = {
    { 'nowrap-ss0', 'set nowrap sidescroll=0 sidescrolloff=0 nolist' },
    { 'nowrap-ss1', 'set nowrap sidescroll=1 sidescrolloff=0 nolist' },
    { 'nowrap-sso5', 'set nowrap sidescroll=1 sidescrolloff=5 nolist' },
    { 'nowrap-sso999', 'set nowrap sidescroll=0 sidescrolloff=999 nolist' },
    { 'nowrap-nu', 'set nowrap sidescroll=0 sidescrolloff=0 number numberwidth=6' },
    { 'wrap', 'set wrap sidescroll=0 sidescrolloff=0 nolist' },
    { 'wrap-nu-list', 'set wrap number numberwidth=6 list listchars=tab:>-,extends:>' },
  }
  for _, sp in ipairs(HSPREADS) do
    n.command(sp[2])
    for _, ln in ipairs({ 1, 5, 7 }) do
      for i, keys in ipairs(COLS) do
        feed(('%dG'):format(ln))
        feed(keys)
        view_val(('view-h-%s-L%d-%02d'):format(sp[1], ln, i))
      end
    end
    snap(screen, 'view-horizontal-' .. sp[1])
  end
  n.command('set wrap nolist nonumber sidescroll=0 sidescrolloff=0')
end)

scenario('view-smoothscroll', function()
  local screen = start(40, 10)
  -- `skipcol` only ever moves under `'smoothscroll'`, and it is the one
  -- `winsaveview()` field `update_topline` shares with `curs_columns`.
  local lines = {}
  for i = 1, 30 do
    lines[i] = ('S%02d '):format(i) .. string.rep('wrapping text ', 12)
  end
  n.api.nvim_buf_set_lines(0, 0, -1, true, lines)
  n.command('set nomore shortmess+=sI report=9999 nostartofline')
  for _, sp in ipairs({
    { 'off', 'set wrap nosmoothscroll scrolloff=0' },
    { 'on', 'set wrap smoothscroll scrolloff=0' },
    { 'on-so3', 'set wrap smoothscroll scrolloff=3' },
    { 'on-nu', 'set wrap smoothscroll scrolloff=0 number numberwidth=6' },
  }) do
    n.command(sp[2])
    feed('gg0')
    for i, keys in ipairs({
      '<C-e>',
      '<C-e>',
      '<C-e>',
      '<C-y>',
      'j',
      'gj',
      'gj',
      'G',
      'gg',
      '<C-d>',
      '<C-u>',
      'zt',
      'zz',
      'zb',
      '10G$',
      'H',
      'L',
    }) do
      feed(keys)
      view_val(('view-ss-%s-%02d-%s'):format(sp[1], i, keys:gsub('[^%w]', '')))
    end
    snap(screen, 'view-smoothscroll-' .. sp[1])
  end
  n.command('set nosmoothscroll nonumber')

  -- Diff mode is the only producer of `topfill`, and `update_topline` has an
  -- arm for it that nothing else reaches.
  n.command('set diffopt=internal,filler')
  n.command('vnew')
  n.api.nvim_buf_set_lines(0, 0, -1, true, { 'a', 'b', 'c', 'x', 'y', 'z' })
  n.command('diffthis | wincmd p | diffthis')
  feed('gg')
  view_val('view-topfill-gg')
  feed('G')
  view_val('view-topfill-G')
  feed('<C-y>')
  view_val('view-topfill-cy')
  snap(screen, 'view-topfill')
  n.command('diffoff! | only')

  -- THE TALL-TOP-LINE SPREADS. Everything above leaves two arms of the
  -- 'smoothscroll' arithmetic unreached: `scroll_cursor_bot`'s sms arm (the
  -- `top_skipped_plines` correction, which only matters when the top line
  -- occupies MORE screen lines than the whole window) and the tail of
  -- `adjust_skipcol` (`sms_cursor_row`, where a cursor row past the bottom
  -- raises `w_skipcol`). Returning an absurd value from either left all
  -- three artifacts IDENTICAL.
  --
  -- Both need the same thing and the 30-line fixture above cannot give it:
  -- a top line TALLER THAN THE WINDOW that is already partly scrolled
  -- (`w_skipcol > 0`), with the cursor on it. Its lines are five screen
  -- lines each, so `<C-e>` walks off the top line long before `top_plines`
  -- can exceed the window height, and `adjust_skipcol` returns at its
  -- `w_cline_height` arm every time. Line 2 here is ~26 screen lines at 40
  -- columns; `2G zt <C-e><C-e><C-e>` parks the cursor mid-buffer under a
  -- partly scrolled tall top line and the following `gj`/`j` move it down
  -- one screen line, which is the whole gesture.
  --
  -- These run AFTER the diff block, not as two more entries in the loop
  -- above, because they need their own buffer: replacing it inside the loop
  -- would change what `diffthis` compares and move every `view-topfill-*`
  -- row. Rows are `view-ss-<spread>-NN`, so nothing above renumbers.
  local tall = {}
  for i = 1, 30 do
    tall[i] = ('T%02d '):format(i) .. string.rep('wrapping text ', 12)
  end
  tall[2] = 'T02 ' .. string.rep('tall line filler ', 60)
  n.api.nvim_buf_set_lines(0, 0, -1, true, tall)
  -- Built rather than written out: each segment re-establishes the tall line
  -- as a partly scrolled top line (`2G zt` then N x CTRL-E) and then moves.
  -- `j` off the bottom of it is what reaches `scroll_cursor_bot`'s sms arm;
  -- `4gj`/`8gj`/`12gj` DOWN THE TALL LINE ITSELF is what reaches the tail of
  -- `adjust_skipcol`. One CTRL-E is not enough for either -- the corrections
  -- both need `w_skipcol` several screen lines in.
  local TALL_KEYS = {}
  local function seg(nce, ...)
    TALL_KEYS[#TALL_KEYS + 1] = '2G'
    TALL_KEYS[#TALL_KEYS + 1] = 'zt'
    for _ = 1, nce do
      TALL_KEYS[#TALL_KEYS + 1] = '<C-e>'
    end
    for _, k in ipairs({ ... }) do
      TALL_KEYS[#TALL_KEYS + 1] = k
    end
  end
  seg(3, 'gj', 'gj', '4gj', '8gj', '12gj')
  seg(5, 'j', 'k')
  seg(8, 'g$', 'j')
  seg(3, '<C-d>')
  seg(3, '<C-f>')
  seg(3, 'zb')
  seg(10, 'j', '12gj')
  seg(0, 'G', 'gg')
  for _, sp in ipairs({
    { 'tall', 'set wrap smoothscroll scrolloff=0 nonumber' },
    { 'tall-so3', 'set wrap smoothscroll scrolloff=3 nonumber' },
  }) do
    n.command(sp[2])
    feed('gg0')
    for i, keys in ipairs(TALL_KEYS) do
      feed(keys)
      view_val(('view-ss-%s-%02d-%s'):format(sp[1], i, keys:gsub('[^%w]', '')))
    end
    snap(screen, 'view-smoothscroll-' .. sp[1])
  end
  n.command('set nosmoothscroll scrolloff=0')
end)

scenario('view-window', function()
  local screen = start(46, 14)
  view_fixture()
  n.command('set nomore shortmess+=sI report=9999 nostartofline')
  -- The same arithmetic with a window that is not the whole screen: the
  -- height `update_topline` divides by is `w_height`, and a split, a
  -- statusline and `'winbar'` each take a row off it.
  for _, sp in ipairs({
    { 'full', 'only | set laststatus=0 nowinbar' },
    { 'status', 'only | set laststatus=2' },
    { 'split', 'only | split | set laststatus=2' },
    { 'winbar', 'only | set laststatus=2 winbar=BAR' },
    { 'tiny', 'only | set laststatus=2 | resize 3' },
  }) do
    pcall(n.command, sp[1] == 'winbar' and 'set winbar=BAR' or 'set winbar=')
    pcall(n.command, sp[2])
    feed('gg0')
    for i, keys in ipairs({ 'G', 'gg', '<C-d>', '<C-u>', '<C-f>', '<C-b>', 'zz', 'zt', 'zb', 'H', 'M', 'L', '50G' }) do
      feed(keys)
      view_val(('view-w-%s-%02d-%s'):format(sp[1], i, keys:gsub('[^%w]', '')))
    end
    snap(screen, 'view-window-' .. sp[1])
  end
  n.command('set winbar= laststatus=1')
  n.command('only')
end)

-- ------------------------------------------------------------------- run

local only = os.getenv('SCRSWEEP_ONLY')
for _, s in ipairs(S) do
  if not only or s.name:match(only) then
    if os.getenv('SCRSWEEP_TRACE') then
      io.stderr:write('scrsweep: ', s.name, '\n')
    end
    local ok, err = pcall(s.fn)
    if not ok then
      say(txt, '--- ', s.name, ' SCENARIO ERROR')
      say(txt, '  ', errtext(err))
      say(vals, ('%-22s SCENARIO ERROR %s'):format(s.name, errtext(err)))
    end
    pcall(n.check_close)
  end
end

local function dump(path, t)
  local f = assert(io.open(path, 'w'))
  f:write(table.concat(t, '\n'), '\n')
  f:close()
end
dump(('%s/%s.txt'):format(outdir, label), txt)
dump(('%s/%s.attrs'):format(outdir, label), attrs)
dump(('%s/%s.vals'):format(outdir, label), vals)
io.write(
  ('scrsweep %s: %d txt / %d attrs / %d vals lines\n'):format(label, #txt, #attrs, #vals)
)
