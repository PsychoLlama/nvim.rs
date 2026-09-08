-- Timing canary for the memline hot path (B10-4).
--
-- `ml_get_buf_impl` / `ml_find_line` run on every keystroke; `ml_append_int`
-- / `ml_delete_int` on every edit; `ml_find_line_or_offset` on every
-- line2byte/byte2line and on 'ruler' with 'go+=b'.  This drives all four
-- against a buffer big enough (default 120k lines) that the block tree is
-- three levels deep, so a walk is not free.
--
-- Deterministic: fixed corpus, fixed LCG seed, fixed operation order.
-- Run each binary several times and take the minimum; the machine is noisy.
--
--   nvim --clean --headless -l mlbench.lua
--
-- Env: MLBENCH_LINES (default 120000), MLBENCH_REPS (default 1).

local NLINES = tonumber(os.getenv('MLBENCH_LINES') or '120000')
local REPS = tonumber(os.getenv('MLBENCH_REPS') or '1')

-- Cheap reproducible PRNG; Lua's math.random is not stable across builds.
local seed = 12345
local function rnd(n)
  seed = (seed * 1103515245 + 12345) % 2147483648
  return seed % n + 1
end

local api = vim.api
local hrtime = vim.uv.hrtime

local results = {}
local function phase(name, fn)
  local t = hrtime()
  local out = fn()
  local ns = hrtime() - t
  results[#results + 1] = { name, ns, out }
end

-- Varying line lengths so data blocks hold varying line counts, and the
-- per-line index arithmetic is exercised at more than one width.
local corpus = {}
for i = 1, NLINES do
  local n = i % 7
  if n == 0 then
    corpus[i] = 'line ' .. i
  elseif n == 1 then
    corpus[i] = ''
  elseif n == 2 then
    corpus[i] = string.rep('x', 30) .. i
  elseif n == 3 then
    corpus[i] = string.rep('word ', 12) .. i
  elseif n == 4 then
    corpus[i] = 'tab\tand \194\171unicode\194\187 ' .. i
  elseif n == 5 then
    corpus[i] = string.rep('a', 200) .. i
  else
    corpus[i] = 'the quick brown fox jumps over the lazy dog ' .. i
  end
end

vim.o.swapfile = false
vim.o.undofile = false
vim.o.undolevels = -1
vim.o.shada = ''

for _ = 1, REPS do
  -- ml_append_int, bulk: builds the whole tree from one line.
  phase('build', function()
    api.nvim_buf_set_lines(0, 0, -1, true, corpus)
    return api.nvim_buf_line_count(0)
  end)

  -- ml_get_buf_impl sequential: the ml_locked / ml_line_lnum caches both hit.
  phase('get-seq', function()
    local total = 0
    for _ = 1, 3 do
      local lines = api.nvim_buf_get_lines(0, 0, -1, true)
      total = total + #lines[#lines]
    end
    return total
  end)

  -- Control: same shape as get-rand but always line 1, so the memline work
  -- is one cache hit.  Subtract it from get-rand to see the walk alone.
  phase('get-ctl', function()
    local total = 0
    for _ = 1, 250000 do
      total = total + #api.nvim_buf_get_lines(0, 0, 1, true)[1]
    end
    return total
  end)

  -- Pure sequential ml_get with almost no caller overhead: search() walks
  -- every line of the buffer looking for a pattern that is not there.
  phase('search', function()
    local hits = 0
    for _ = 1, 24 do
      hits = hits + vim.fn.search('zqxjvkw', 'w')
    end
    return hits
  end)

  -- ml_get_buf_impl random: every call misses both caches and walks the
  -- tree from the root (or from a stack frame that no longer covers lnum).
  phase('get-rand', function()
    local total = 0
    for _ = 1, 250000 do
      local i = rnd(NLINES)
      total = total + #api.nvim_buf_get_lines(0, i - 1, i, true)[1]
    end
    return total
  end)

  -- ml_replace + ml_flush_line: the in-place path when the new line fits.
  phase('replace', function()
    for _ = 1, 60000 do
      local i = rnd(NLINES)
      api.nvim_buf_set_lines(0, i - 1, i, true, { 'replaced ' .. i })
    end
    return api.nvim_buf_line_count(0)
  end)

  -- ml_find_line_or_offset both ways, plus the chunk accelerator.
  phase('offsets', function()
    local total = 0
    for _ = 1, 60000 do
      total = total + vim.fn.line2byte(rnd(NLINES))
    end
    local max = vim.fn.line2byte(NLINES)
    for _ = 1, 60000 do
      total = total + vim.fn.byte2line(rnd(max))
    end
    return total
  end)

  -- Whole-buffer offset walk (ml_updatechunk's accelerator is what keeps
  -- this from being quadratic).
  phase('wordcount', function()
    local wc = vim.fn.wordcount()
    return wc.bytes
  end)

  -- ml_append_int at scattered positions: block splits, and pointer block
  -- splits above them.
  phase('append', function()
    for _ = 1, 60000 do
      local i = rnd(NLINES)
      api.nvim_buf_set_lines(0, i, i, true, { 'inserted here ' .. i })
    end
    return api.nvim_buf_line_count(0)
  end)

  -- ml_delete_int at scattered positions: empties data blocks and prunes
  -- pointer blocks.
  phase('delete', function()
    local n = api.nvim_buf_line_count(0)
    for _ = 1, 60000 do
      local i = rnd(n - 1)
      api.nvim_buf_set_lines(0, i - 1, i, true, {})
      n = n - 1
    end
    return n
  end)

  api.nvim_buf_set_lines(0, 0, -1, true, {})
end

local total = 0
for _, r in ipairs(results) do
  total = total + r[2]
  io.stdout:write(string.format('%-10s %8.1f ms  (%s)\n', r[1], r[2] / 1e6, tostring(r[3])))
end
io.stdout:write(string.format('%-10s %8.1f ms\n', 'TOTAL', total / 1e6))
vim.cmd('qa!')
