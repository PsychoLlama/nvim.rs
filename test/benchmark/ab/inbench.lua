-- Timing canary for the input and command-line layer (B13-3).
--
--   nvim --headless -c 'luafile inbench.lua' -c 'qa!'
--
-- Run it through `inbench.sh`, which does the interleaved A/B
-- the drift rule requires.  Prints "INBENCH<TAB>phase<TAB>ms" lines so
-- the shell can pick them out of whatever else lands on stdout.
--
-- Why `--headless -c` and not `--headless -l` (the same reason scrbench
-- has): in `-l` script mode `full_screen` is false, so every message
-- takes the no-screen path, `msg_puts_display` never scrolls a grid and
-- the `msgout` phase measures about a fifth of the work a real message
-- costs.  `-c` mode has `full_screen` set.  It is also the mode in which
-- `wait_return`/`do_more_prompt` stay out of the way: both guard on
-- `headless_mode && !ui_active()`, and no UI is attached here, so a
-- 10,000-message phase runs to completion without a prompt.
--
-- Phase map (which code each phase is the canary for):
--   ctl        nothing        -- noise floor; distrust any phase whose
--                                swing is not clearly larger than this
--   typeahead  getchar.rs     -- vgetorpeek/vgetc over a 100k-key run
--                                with no mapping to match
--   maps       mapping.rs     -- the same run against a 5,000-entry map
--                                table, i.e. maphash + handle_mapping
--   mapdict    mapping.rs     -- mapblock_fill_dict, 5,000 maps through
--                                maparg(); the listing/introspection half
--   termcodes  keycodes.rs    -- replace_termcodes + keytrans over every
--                                key-name family; B13-4 rewrites the
--                                tables this walks
--   cmdcompl   cmdexpand.rs   -- getcompletion() over a 20,000-file
--                                directory (ExpandFromContext + sort)
--   cmdctx     cmdexpand.rs   -- set_one_cmd_context over a corpus of
--                                partial command lines: the walker, not
--                                the expander
--   cmdline    ex_getln.rs    -- command_line_enter/_handle_key: 2,000
--                                command lines typed key by key
--   inscompl   insexpand.rs   -- <C-n> over a 10,000-word buffer
--   msgout     message.rs     -- 10,000 echomsg through msg_puts_display
--                                and the history
local work = os.getenv('INBENCH_WORK') or '/tmp/inbench'
vim.fn.mkdir(work, 'p')
vim.cmd('silent! cd ' .. vim.fn.fnameescape(work))

-- A quiet, fixed editor.  Every option the phases below can be slowed
-- down by is set explicitly: a canary whose baseline moves when a
-- default does is not a canary.
vim.cmd([[
  silent! set nomore noruler noshowcmd noshowmode shortmess=filnxtToOF
  silent! set noswapfile nobackup nowritebackup noundofile undolevels=-1
  silent! set report=9999 lazyredraw& timeout timeoutlen=1000 ttimeoutlen=50
  silent! set wildmenu wildmode=full wildoptions= wildignore= nowildignorecase
  silent! set completeopt=menu complete=. nolangremap langmap=
  silent! set messagesopt=hit-enter,history:500
]])

local NFILE = 20000 -- files for the completion phase
local NWORD = 10000 -- words for the insert-completion phase
local NMAP = 5000 -- entries in the map table

-- ------------------------------------------------------------ fixtures
--
-- Built once and reused, so the same bytes are measured on both sides of
-- an A/B and neither side pays for the mkdir.
local function build()
  local dir = work .. '/files'
  if vim.uv.fs_stat(dir .. '/f19999.txt') then
    return
  end
  vim.fn.mkdir(dir, 'p')
  for i = 0, NFILE - 1 do
    local fd = io.open(('%s/f%05d.txt'):format(dir, i), 'w')
    if fd then
      fd:write('x\n')
      fd:close()
    end
  end
end
build()

local words = {}
for i = 1, NWORD do
  words[i] = ('word%05d alt%05d'):format(i, i)
end

-- Every key-name family the tables carry, so `termcodes` walks the whole
-- structure rather than one bucket of it.
-- stylua: ignore
local KEYNAMES = {
  '<Nul>', '<BS>', '<Tab>', '<NL>', '<FF>', '<CR>', '<Return>', '<Enter>',
  '<Esc>', '<Space>', '<lt>', '<Bslash>', '<Bar>', '<Del>', '<CSI>', '<xCSI>',
  '<Up>', '<Down>', '<Left>', '<Right>', '<xUp>', '<xDown>', '<xLeft>', '<xRight>',
  '<S-Up>', '<S-Down>', '<S-Left>', '<S-Right>', '<C-Left>', '<C-Right>',
  '<F1>', '<F5>', '<F12>', '<S-F1>', '<xF1>', '<Help>', '<Undo>', '<Insert>',
  '<Home>', '<End>', '<PageUp>', '<PageDown>', '<kHome>', '<kEnd>', '<kPlus>',
  '<kMinus>', '<kMultiply>', '<kDivide>', '<kEnter>', '<kPoint>', '<k0>', '<k9>',
  '<LeftMouse>', '<RightMouse>', '<MiddleMouse>', '<LeftDrag>', '<LeftRelease>',
  '<ScrollWheelUp>', '<ScrollWheelDown>', '<MouseMove>',
  '<C-a>', '<C-z>', '<C-@>', '<C-^>', '<C-_>', '<C-?>', '<M-a>', '<A-b>',
  '<D-c>', '<T-d>', '<S-C-M-x>', '<Plug>Foo', '<SNR>1_Bar', '<Cmd>', '<ScriptCmd>',
  '<Ignore>', '<Nop>', '<Char-0x41>', '<Char-65>', '<C-Space>', '<S-Tab>',
}

-- Partial command lines chosen so the context walker takes a different
-- arm for each: this phase must not degenerate into one hot path.
-- stylua: ignore
local CTXLINES = {
  'se', 'set backspace=', 'setlocal ', 'edit files/f000', 'buffer ',
  'help getcmd', 'highlight Nor', 'nmap ,', 'augroup ', 'autocmd Buf',
  'let g:', 'echo g:', 'call getcm', 'syntax ', 'command Fo', 'sign ',
  'menu ', 'history ', '!l', 'echo $HO', 'lua vim.ap', 'messages ',
  's/a/b/', 'g/a/d', 'cd files/', 'runtime ', 'colorscheme ', 'packadd ',
}

-- ---------------------------------------------------------------- time
local function ms(f, rounds)
  local t0 = vim.uv.hrtime()
  for _ = 1, rounds do
    f()
  end
  return (vim.uv.hrtime() - t0) / 1e6
end

local out = {}
local function phase(name, rounds, f)
  out[#out + 1] = ('INBENCH\t%s\t%.1f'):format(name, ms(f, rounds))
end

local function tc(keys)
  return vim.api.nvim_replace_termcodes(keys, true, true, true)
end

-- Noise floor.  A Vimscript call of comparable per-iteration cost that
-- reaches none of the seven B13 modules.
--
-- B18-5: `strwidth` is eval.rs -> mbyte/cells.rs, NOT charset.rs (the
-- `char2cells` call sits on the overlong-sequence arm an ASCII argument
-- never takes).  See the longer note in evalbench.lua; the short
-- version is that this phase is a weaker floor than the manual claimed
-- for a batch that owns charset.rs, so read TOTAL beside it.
phase('ctl', 200000, function()
  local _ = vim.fn.strwidth('edit files/f01234.txt')
end)

-- ------------------------------------------------------------ typeahead
--
-- 100,000 keys through vgetorpeek with an empty map table: the pure
-- typeahead path.  `hl` rather than `jk` so the cursor stays on one line
-- and the redraw cost is constant.
vim.api.nvim_buf_set_lines(0, 0, -1, false, { ('abcdefghij'):rep(40) })
local RUN = ('hl'):rep(25000)
phase('typeahead', 2, function()
  vim.api.nvim_win_set_cursor(0, { 1, 100 })
  vim.fn.feedkeys(RUN, 'xt')
end)

-- ----------------------------------------------------------------- maps
--
-- The same run with 5,000 mappings defined.  Every key now goes through
-- the maphash bucket walk, and `,aNNNN` shares a first byte with all of
-- them, so the ambiguity path is exercised rather than short-circuited.
for i = 1, NMAP do
  vim.cmd(('silent! nnoremap ,a%04d <Nop>'):format(i))
end
phase('maps', 2, function()
  vim.api.nvim_win_set_cursor(0, { 1, 100 })
  vim.fn.feedkeys(RUN, 'xt')
end)

-- The same table, but typing keys that actually *hit* it.  `maps` above
-- feeds `h`/`l`, whose maphash buckets are empty, and it measured within
-- 0.4% of `typeahead` -- a phase that cannot tell a 5,000-entry table
-- from an empty one is not a canary for `handle_mapping`.  This one
-- walks the populated bucket and resolves 2,000 mappings.
--
-- The trailing `l` is load-bearing: `<Nop>` consumes no key, so
-- `mapdepth` is never reset and 1,000 consecutive empty expansions raise
-- E223 (measured -- the first draft died there and the harness reported
-- a division by zero, because the phase printed nothing at all).  A real
-- key between mappings resets the counter.
local MAPRUN = {}
for i = 1, 2000 do
  MAPRUN[i] = (',a%04dl'):format(i)
end
MAPRUN = table.concat(MAPRUN)
phase('mapresolve', 2, function()
  vim.api.nvim_win_set_cursor(0, { 1, 100 })
  vim.fn.feedkeys(MAPRUN, 'xt')
end)

-- The introspection half: mapblock_fill_dict once per entry.
phase('mapdict', 2, function()
  for i = 1, NMAP do
    local _ = vim.fn.maparg((',a%04d'):format(i), 'n', false, true)
  end
end)
vim.cmd('silent! nmapclear')

-- ------------------------------------------------------------ termcodes
--
-- Both directions over every key-name family, which is exactly what
-- B13-4 replaces with a sorted table and a binary search.
phase('termcodes', 1600, function()
  for _, name in ipairs(KEYNAMES) do
    local raw = vim.api.nvim_replace_termcodes(name, true, true, true)
    local _ = vim.fn.keytrans(raw)
  end
end)

-- ------------------------------------------------------------- cmdcompl
--
-- 20,000 candidates through ExpandFromContext, the sort, and the
-- filtering.  Two prefixes: one that matches everything and one that
-- matches a hundredth of it.
phase('cmdcompl', 6, function()
  local _ = vim.fn.getcompletion('files/f', 'file')
end)
phase('cmdcompl-narrow', 60, function()
  local _ = vim.fn.getcompletion('files/f001', 'file')
end)

-- The walker on its own: no expansion, just set_one_cmd_context deciding
-- which of the 45 contexts each line is in.
phase('cmdctx', 2000, function()
  for _, line in ipairs(CTXLINES) do
    local _ = vim.fn.getcompletiontype(line)
  end
end)

-- -------------------------------------------------------------- cmdline
--
-- 2,000 command lines typed one key at a time: command_line_enter plus
-- one command_line_handle_key per character, and one CmdlineChanged
-- decision per character.
phase('cmdline', 2, function()
  for i = 1, 1000 do
    vim.fn.feedkeys(tc((':let g:b%04d = %d<CR>'):format(i, i)), 'xt')
  end
end)

-- ------------------------------------------------------------- inscompl
--
-- <C-n> over a 10,000-word buffer: ins_compl_get_exp scanning the whole
-- buffer, ins_compl_add per candidate, and the match list built twice.
vim.api.nvim_buf_set_lines(0, 0, -1, false, words)
vim.api.nvim_buf_set_lines(0, -1, -1, false, { 'word0' })
phase('inscompl', 6, function()
  vim.api.nvim_win_set_cursor(0, { NWORD + 1, 0 })
  vim.fn.feedkeys(tc('A<C-n><C-e><Esc>'), 'xt')
end)

-- --------------------------------------------------------------- msgout
--
-- 10,000 messages through msg_puts_display and msg_hist_add.  The
-- history is capped at 500, so this also measures the eviction.
phase('msgout', 1, function()
  for i = 1, 10000 do
    vim.cmd(('echomsg "message %05d for the output volume phase"'):format(i))
  end
end)
vim.cmd('silent! messages clear')

-- The same volume with 'shortmess' suppressing nothing and the message
-- going only to the history, so the two halves can be told apart.
phase('msghist', 10, function()
  for i = 1, 5000 do
    vim.fn.execute(('echomsg "history %05d"'):format(i))
  end
end)
vim.cmd('silent! messages clear')

-- ------------------------------------------------- srcfile / rtpsearch
--
-- Added at B16-5.  Nothing in any of the four benches drove `:source` or
-- the runtime-path search, and between them those are the whole of
-- startup cost -- the one thing this family is on the hot path for.
-- Both fixtures are built once, outside the timing, and both phases
-- restore what they change.

local NRTP = 200 -- entries on the wide 'runtimepath'
local NSRC = 2000 -- lines in the sourced script

local function build_runtime()
  local dir = work .. '/rtp'
  if not vim.uv.fs_stat(('%s/d%03d/plugin/p.vim'):format(dir, NRTP - 1)) then
    for i = 0, NRTP - 1 do
      local d = ('%s/d%03d/plugin'):format(dir, i)
      vim.fn.mkdir(d, 'p')
      local fd = io.open(d .. '/p.vim', 'w')
      if fd then
        fd:write(('let g:rtp_hit = %d\n'):format(i))
        fd:close()
      end
    end
  end
  local src = work .. '/srcbench.vim'
  if not vim.uv.fs_stat(src) then
    local fd = assert(io.open(src, 'w'))
    for i = 1, NSRC do
      -- One line in ten is a continuation, so `concat_continued_line`
      -- and the growsize ladder in `get_one_sourceline` are on the path
      -- and not just the plain-line fast case.
      if i % 10 == 0 then
        fd:write(("let s:c%d = 'a'\n      \\ . 'b'\n"):format(i))
      else
        fd:write(('let s:v%d = %d + %d\n'):format(i, i, i))
      end
    end
    fd:close()
  end
end
build_runtime()

-- do_source_ext over a 2,000-line script, re-sourced.  The script item
-- is reused after the first round (find_script_by_name), so this is the
-- reader and the executor, not the registry.
phase('srcfile', 50, function()
  vim.cmd('source ' .. work .. '/srcbench.vim')
end)

-- do_in_cached_path over a 200-entry 'runtimepath' where NOTHING
-- matches, so the whole path is walked every time and no file is
-- sourced: the search, isolated from `srcfile`'s answer.
local saved_rtp = vim.o.runtimepath
local rtplist = {}
for i = 0, NRTP - 1 do
  rtplist[#rtplist + 1] = ('%s/rtp/d%03d'):format(work, i)
end
vim.o.runtimepath = table.concat(rtplist, ',')
-- Warm the cache: the first search after an option write pays for
-- `runtime_search_path_build`, which is a different question.
vim.cmd('runtime! nosuchdir/warm.vim')
phase('rtpsearch', 220, function()
  vim.cmd('runtime! nosuchdir/nosuchfile.vim')
end)
vim.o.runtimepath = saved_rtp

io.write(table.concat(out, '\n'), '\n')
