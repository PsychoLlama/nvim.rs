-- Timing canary for `:mkspell` without a .sug file.
--
--   MKSPELLBENCH_WORK=dir nvim --clean --headless -l mkspellbench.lua
--
-- Run it through `mkspellbench.sh`, which does the interleaved A/B.
-- Prints "MKSPELLBENCH<TAB>phase<TAB>ms" lines.
--
-- The `.sug` pass used to be 97 % of a large `:mkspell` (a quadratic walk,
-- now linear) and hid everything else; without SAL/SOFO there is no `.sug`,
-- and what is left is the path this bench is for: reading the .aff/.dic,
-- building the word tree, compressing it and writing the .spl.
--
-- Phase map:
--   ctl     nothing            -- noise floor
--   nosug   spellfile/         -- :mkspell of 100,000 words, default
--                                 'mkspellmem' (a few compressions)
--   tight   spellfile/wordtree -- every fifth word of it under 'mkspellmem'
--                                 16,16,1: hundreds of compression runs
--
-- The input is generated here, deterministically (fixed LCG), into
-- MKSPELLBENCH_WORK: made-up words of two to five syllables, some
-- capitalised (so the keep-case tree grows too), with S/D suffix and G
-- prefix flags and REP items.
--
-- MKSPELLBENCH_ONLY runs a comma-separated subset of the phases after the
-- setup, for a per-phase instruction count.

local WORK = assert(os.getenv('MKSPELLBENCH_WORK'), 'MKSPELLBENCH_WORK is not set')
local NWORDS = tonumber(os.getenv('MKSPELLBENCH_WORDS') or '100000')
local ONLY = os.getenv('MKSPELLBENCH_ONLY')

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
  if not wanted(name) then
    return
  end
  local t = vim.uv.hrtime()
  f()
  io.stdout:write(('MKSPELLBENCH\t%s\t%.1f\n'):format(name, (vim.uv.hrtime() - t) / 1e6))
end

-- Park-Miller: the product stays below 2^53, so a Lua number holds it
-- exactly (glibc's LCG multiplier overflows a double's mantissa and the
-- sequence collapses into a short cycle).
local seed = 37
local function rnd(n)
  seed = (seed * 16807) % 2147483647
  return seed % n + 1
end

-- stylua: ignore
local SYL = {
  'ka', 'ta', 'ra', 'mi', 'no', 'su', 'be', 'lo', 've', 'qui', 'dra', 'sto',
  'pen', 'gor', 'lim', 'ash', 'eth', 'ung', 'or', 'el', 'an', 'ix', 'um',
  'ez', 'ou', 'ch', 'sh', 'th', 'ph', 'é', 'ö', 'ñ',
}
local FLAGS = { '', '', '', '/S', '/D', '/SD', '/G' }

vim.fn.mkdir(WORK, 'p')
local seen, words = {}, {}
while #words < NWORDS do
  local parts = {}
  for i = 1, rnd(4) + 1 do
    parts[i] = SYL[rnd(#SYL)]
  end
  local w = table.concat(parts)
  if rnd(20) == 1 then
    w = w:sub(1, 1):upper() .. w:sub(2)
  end
  if not seen[w] then
    seen[w] = true
    words[#words + 1] = w
  end
end
table.sort(words)
local dic = { tostring(#words) }
for i, w in ipairs(words) do
  dic[i + 1] = w .. FLAGS[rnd(#FLAGS)]
end
vim.fn.writefile(dic, WORK .. '/bench.dic')
-- Every fifth word for `tight`: a compression run walks the whole tree,
-- so their cost grows with the square of the input.
local small = { tostring(math.floor(#words / 5)) }
for i = 5, #words, 5 do
  small[#small + 1] = dic[i + 1]
end
vim.fn.writefile(small, WORK .. '/small.dic')
local aff = {
  'SET UTF-8',
  'TRY esianrtolcdugmphbyfvkwzéöñ',
  'SFX S Y 2',
  'SFX S 0 s [^s]',
  'SFX S 0 es s',
  'SFX D Y 2',
  'SFX D 0 ed [^e]',
  'SFX D 0 d e',
  'PFX G Y 1',
  'PFX G 0 re .',
  'REP 2',
  'REP ph f',
  'REP k qu',
}
vim.fn.writefile(aff, WORK .. '/bench.aff')
vim.fn.writefile(aff, WORK .. '/small.aff')

vim.fn.delete(WORK .. '/out.utf-8.spl')
local input = vim.fn.fnameescape(WORK .. '/bench')
local small_input = vim.fn.fnameescape(WORK .. '/small')
local out = vim.fn.fnameescape(WORK .. '/out.utf-8.spl')

phase('ctl', function()
  for _ = 1, 100000 do
    local _ = vim.fn.strwidth('quick brown fox')
  end
end)

phase('nosug', function()
  vim.cmd('silent mkspell! ' .. out .. ' ' .. input)
end)

vim.o.mkspellmem = '16,16,1'
phase('tight', function()
  vim.cmd('silent mkspell! ' .. out .. ' ' .. small_input)
end)

-- A round whose `:mkspell` failed would report a spuriously small time.
if wanted('nosug') or wanted('tight') then
  assert(vim.fn.filereadable(WORK .. '/out.utf-8.spl') == 1, 'mkspell wrote nothing')
  assert(vim.fn.filereadable(WORK .. '/out.utf-8.sug') == 0, 'a .sug was written')
end
