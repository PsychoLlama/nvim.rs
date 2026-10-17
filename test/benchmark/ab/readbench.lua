-- Timing canary for reading a file into a buffer.
--
--   READBENCH_WORK=dir nvim --clean --headless -l readbench.lua
--
-- Run it through readbench.sh, which builds the two 100 MB inputs on its
-- untimed warm-up and does the interleaved A/B. Prints "phase<TAB>ms" lines
-- prefixed with READBENCH.
--
-- Neither mlbench (lines come from Lua) nor any other bench reads a file:
-- the plain read is `readfile` -> `ml_append_int` at the end of a growing
-- buffer, a path that went +19 % once without a bench noticing. Phases:
--
--   plain   readfile + ml_append  -- `:edit` of 100 MB of UTF-8, then three
--                                    `:edit!`, each freeing the buffer's
--                                    blocks and reading them back
--   latin1  + the conversion      -- the same with `++enc=latin1`: every
--                                    line goes through the iconv-free
--                                    Latin-1 to UTF-8 widening
--   fencs   'fileencodings'       -- one `:edit` of the Latin-1 file with
--                                    the default list, so the UTF-8 attempt
--                                    fails part way and the read restarts
--
-- READBENCH_ONLY runs a comma-separated subset, for a per-phase count.

local WORK = assert(os.getenv('READBENCH_WORK'), 'READBENCH_WORK is not set')
local ONLY = os.getenv('READBENCH_ONLY')

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

local function phase(name, cmds)
  if not wanted(name) then
    return
  end
  local t = vim.uv.hrtime()
  for _, c in ipairs(cmds) do
    vim.cmd(c)
  end
  local elapsed = (vim.uv.hrtime() - t) / 1e6
  -- A read that failed is fast; refuse to report it.
  if vim.api.nvim_buf_line_count(0) < 1000000 then
    io.stderr:write('readbench: ' .. name .. ' did not read the file\n')
    os.exit(1)
  end
  io.stdout:write(('READBENCH\t%s\t%.3f\n'):format(name, elapsed))
  vim.cmd('enew | silent bwipeout! #')
end

vim.o.swapfile = false
vim.o.undofile = false
vim.o.shada = ''
vim.o.more = false

local big = vim.fn.fnameescape(WORK .. '/big.txt')
local lat = vim.fn.fnameescape(WORK .. '/lat.txt')

phase('plain', {
  'silent edit ' .. big,
  'silent edit!',
  'silent edit!',
  'silent edit!',
})
phase('latin1', {
  'silent edit ++enc=latin1 ' .. lat,
  'silent edit! ++enc=latin1',
  'silent edit! ++enc=latin1',
  'silent edit! ++enc=latin1',
})
phase('fencs', { 'silent edit ' .. lat })
