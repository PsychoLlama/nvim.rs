-- Timing canary for undo: recording, walking, saving and reading the tree.
--
--   nvim --clean --headless -l undobench.lua
--
-- Run it through undobench.sh, which does the interleaved A/B. Prints
-- "phase<TAB>ms" lines prefixed with UNDOBENCH.
--
-- Every other bench runs with 'undolevels' at -1, so none of them records an
-- undo step. Phases, in order, each on the tree the one before left:
--
--   record  u_savecommon / u_saveline -- N one-line changes, each its own
--                                        undo step, plus N/10 three-line
--                                        replacements and N/10 delete+append
--                                        pairs
--   undo    u_undoredo                -- `:undo 0` through every step, then
--                                        back to the last one
--   file    u_write_undo / u_read_undo -- `:wundo`, `:edit!`, `:rundo`, twice
--   tree    undotree() / :undolist    -- the tree rendered 20 times as a
--                                        dict and 5 times as a listing
--
-- Env: UNDOBENCH_N (default 20000); UNDOBENCH_ONLY runs a comma-separated
-- subset of the phases after `record`'s setup, which always runs (every
-- other phase needs its tree) but is reported only when asked for.

local N = tonumber(os.getenv('UNDOBENCH_N') or '20000')
local ONLY = os.getenv('UNDOBENCH_ONLY')

local function wanted(name)
  if not ONLY then
    return true
  end
  for p in ONLY:gmatch('[^,]+') do
    if p == name then
      return true
    end
  end
  return false
end

local function phase(name, f)
  local t = vim.uv.hrtime()
  f()
  local elapsed = (vim.uv.hrtime() - t) / 1e6
  if wanted(name) then
    io.stdout:write(('UNDOBENCH\t%s\t%.3f\n'):format(name, elapsed))
  end
end

-- nvim's own temporary directory, removed when it exits.
local dir = vim.fn.fnamemodify(vim.fn.tempname(), ':h')
vim.o.swapfile = false
vim.o.undofile = false
vim.o.shada = ''
vim.o.more = false
vim.o.undolevels = 1000000

local lines = {}
for i = 1, 2000 do
  lines[i] = ('line %05d %s h\u{e9}llo'):format(i, ('x'):rep(40))
end
vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
vim.cmd('silent write! ' .. vim.fn.fnameescape(dir .. '/undobench.txt'))
local function sync()
  vim.cmd('let &ul = &ul')
end
sync()

phase('record', function()
  local setline = vim.fn.setline
  for i = 1, N do
    setline((i * 7919) % 2000 + 1, 'changed ' .. i)
    sync()
    if i % 10 == 0 then
      local l = (i * 104729) % 1990 + 1
      vim.api.nvim_buf_set_lines(0, l, l + 3, false, { 'a' .. i, 'b' .. i, 'c' .. i })
      sync()
      vim.api.nvim_buf_set_lines(0, l, l + 1, false, {})
      vim.api.nvim_buf_set_lines(0, l, l, false, { 'ins' .. i })
      sync()
    end
  end
end)

local last = vim.fn.changenr()
-- A tree that did not record measured nothing: refuse to report on it.
if last < N then
  io.stderr:write(('undobench: %d undo steps recorded, wanted %d\n'):format(last, N))
  os.exit(1)
end

if wanted('undo') then
  phase('undo', function()
    vim.cmd('silent undo 0')
    vim.cmd('silent undo ' .. last)
  end)
end

if wanted('file') then
  local undofile = vim.fn.fnameescape(dir .. '/undobench.un')
  vim.cmd('silent write')
  phase('file', function()
    for _ = 1, 2 do
      vim.cmd('silent wundo! ' .. undofile)
      vim.cmd('silent edit!')
      vim.cmd('silent rundo ' .. undofile)
    end
  end)
  if vim.fn.changenr() ~= last then
    io.stderr:write('undobench: :rundo did not restore the tree\n')
    os.exit(1)
  end
end

if wanted('tree') then
  phase('tree', function()
    for _ = 1, 20 do
      vim.fn.undotree()
    end
    for _ = 1, 5 do
      vim.fn.execute('undolist')
    end
  end)
end
