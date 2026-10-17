-- Timing canary for the regexp engines and the commands built on them.
--
--   nvim --clean --headless -l rebench.lua
--
-- Run it through rebench.sh, which does the interleaved A/B the drift rule
-- requires. Prints "phase<TAB>ms" lines prefixed with REBENCH.
--
-- The busted `bench_regexp_spec.lua` times one pathological search against a
-- dev build; this drives the engines over a realistic buffer so that a
-- release binary's instruction count says what a rewrite of the matcher
-- cost. Phases:
--
--   ctl       nothing              -- noise floor (strdisplaywidth)
--   bt        the backtracking     -- the pattern set below through
--             engine, 're=1'          `:%s/pat//gn` (vim_regexec_multi, one
--                                     call per line, every match counted),
--                                     plus 4,000 bytes of the `re.freeze`
--                                     HTML fixture
--   nfa       the NFA, 're=2'      -- the same, under the NFA
--   auto      're=0'               -- the same, letting the compiler choose
--                                     (NFA first, BT when the NFA declines)
--   search    do_search/searchit   -- `search()` stepping through every match
--                                     of five patterns with wrap-around, and
--                                     `searchcount()`
--   subst     vim_regsub_multi     -- `:%s` that writes: `&`, `\1`, `\u`, `~`
--                                     and a `\=` expression, each over a fresh
--                                     copy of the buffer
--   matchlist vim_regexec_nl       -- matchlist()/matchstrpos()/substitute()/
--                                     split() over strings, no buffer
--   mbyte     the per-character    -- patterns that walk UTF-8 one character
--             paths                   at a time over text with composing
--                                     characters, CJK and Latin-1 letters:
--                                     `.`, `\Z`, `[[=e=]]`, `\%u`, `\c` over
--                                     multibyte case pairs, `\%v` -- the
--                                     cluster_len/char_at walk both engines
--                                     share
--
-- Deterministic: fixed corpus, fixed LCG, fixed order. REBENCH_LINES sizes
-- the buffer (default 6000); REBENCH_ONLY runs a comma-separated subset of
-- the phases after the setup, for a per-phase instruction count.

local api, fn = vim.api, vim.fn

local NLINES = tonumber(os.getenv('REBENCH_LINES') or '6000')
local ONLY = os.getenv('REBENCH_ONLY')
local HERE = debug.getinfo(1, 'S').source:sub(2):match('(.*/)') or './'
-- The whole line runs into search()'s timeout under the NFA, which would
-- make the instruction count depend on the wall clock; 4,000 bytes of it
-- finish in ~50 ms there.
local FREEZE = fn.readfile(HERE .. '../../old/testdir/samples/re.freeze.txt')[5]:sub(1, 4000)

local seed = 4242
local function rnd(n)
  seed = (seed * 1103515245 + 12345) % 2147483648
  return seed % n + 1
end

-- stylua: ignore
local WORDS = {
  'the', 'quick', 'brown', 'fox', 'jumps', 'over', 'lazy', 'dog', 'running',
  'walking', 'foo', 'bar', 'baz', 'foobar', 'return', 'function', 'local',
  'end', 'if', 'then', 'else', 'while', 'value', 'index', 'count', 'string',
  'number', 'table', 'nil', 'true', 'false', 'self', 'error', 'print',
}
-- stylua: ignore
local MBYTE = {
  'café', 'naïve', 'über', 'Straße', 'ÉCOLE', 'école', 'garçon', 'Ærø',
  'e\204\129t\204\129e\204\129', -- e + U+0301, three times
  'a\204\138ngstro\204\136m',    -- a + U+030A, o + U+0308
  '中文字符', '日本語の文', '한국어', 'Ελληνικά', 'ελληνικά', 'кириллица',
  'КИРИЛЛИЦА', '😀smile', 'x\204\163\204\135y',
}

local function line(i)
  local parts = {}
  local n = rnd(12)
  for _ = 1, n do
    local r = rnd(10)
    if r <= 6 then
      parts[#parts + 1] = WORDS[rnd(#WORDS)]
    elseif r <= 8 then
      parts[#parts + 1] = MBYTE[rnd(#MBYTE)]
    elseif r == 9 then
      parts[#parts + 1] = tostring(rnd(100000))
    else
      parts[#parts + 1] = WORDS[rnd(#WORDS)] .. '(' .. WORDS[rnd(#WORDS)] .. ')'
    end
  end
  local indent = (i % 5 == 0) and '\t' or string.rep(' ', i % 4 * 2)
  return indent .. table.concat(parts, rnd(3) == 1 and ', ' or ' ')
end

local corpus = {}
for i = 1, NLINES do
  corpus[i] = line(i)
end

-- The engine set: one of each family the compilers distinguish, all
-- expressible by both engines so the three phases run the same work.
-- stylua: ignore
local PATTERNS = {
  [[\<\w\+ing\>]],                  -- word boundaries, a class, a literal tail
  [[\d\{3,}]],                      -- counted repeat
  [[\(foo\|bar\|baz\)\+]],          -- alternation under a repeat
  [[\v(\w+)\s+\1]],                 -- a backreference
  [[^\s*\(local\|function\)\>]],    -- anchored, leading white
  [[(\([^()]*\))]],                 -- a negated collection inside a group
  [[.*fox.*dog]],                   -- greedy dot-star twice
  [[\%>20c\<[a-z]\+\>]],            -- a column atom
  [[\(value\)\@<=s\?\s]],           -- look-behind
  [[\(return\)\@!\<\l\{6}\>]],      -- negative look-ahead
  [[\cRETURN\|\cFUNCTION]],         -- ignore-case
  [[\S\+,\s\S\+]],                  -- non-white runs
}

-- The per-character set: the walk over multibyte text.
-- stylua: ignore
local MB_PATTERNS = {
  [[.\{5}]],                        -- five characters, composing included
  [[e\Z]],                          -- ignore the combining marks
  [=[[[=e=]]\+]=],                  -- an equivalence class
  [[\%u00e9\|\%u00fc]],             -- code points
  [[\c\<école\>]],                  -- multibyte case folding
  [[\cкириллица]],
  [[[^\x00-\x7f]\+]],               -- runs of non-ASCII
  [[\%>8v\k\+]],                    -- virtual column over wide characters
  [[\<\k\{4,}\>]],                  -- keyword runs in every script
  [=[[[:upper:]][[:lower:]]\+]=],   -- character classes over multibyte
}

local function ms(f)
  local t = vim.uv.hrtime()
  f()
  return (vim.uv.hrtime() - t) / 1e6
end

local function say(name, value)
  io.stdout:write(('REBENCH\t%s\t%.3f\n'):format(name, value))
end

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

vim.o.swapfile = false
vim.o.undofile = false
vim.o.undolevels = -1
vim.o.shada = ''
vim.o.more = false
vim.o.maxmempattern = 2000000

local function load()
  api.nvim_buf_set_lines(0, 0, -1, true, corpus)
end

local function count_all(pats)
  local total = 0
  for _, pat in ipairs(pats) do
    -- `:s///gn` counts without writing; the count lands in v:statusmsg.
    vim.cmd('silent! %s/' .. pat .. '//gne')
    total = total + (tonumber(vim.v.statusmsg:match('^(%d+)')) or 0)
  end
  return total
end

-- A phase that matched nothing measured nothing: refuse to report it.
local function check(name, n)
  if n == 0 then
    io.stderr:write('rebench: ' .. name .. ' matched nothing\n')
    os.exit(1)
  end
end

local function engine(name, re)
  if not wanted(name) then
    return
  end
  load()
  vim.o.regexpengine = re
  local n
  say(
    name,
    ms(function()
      n = count_all(PATTERNS)
      -- The line bench_regexp times: HTML where a trailing-white search
      -- with a look-behind on the cursor is quadratic under the NFA.
      api.nvim_buf_set_lines(0, 0, -1, true, { '', FREEZE, '' })
      api.nvim_win_set_cursor(0, { 2, 0 })
      vim.cmd([[silent! call search('\s\+\%#\@<!$', '', 0, 0)]])
    end)
  )
  check(name, n)
  vim.o.regexpengine = 0
end

load()
if fn.search([[\<fox\>]]) == 0 then
  io.stderr:write('rebench: corpus did not load\n')
  os.exit(1)
end

if wanted('ctl') then
  say(
    'ctl',
    ms(function()
      local n = 0
      for _ = 1, 10 do
        for i = 1, #corpus do
          n = n + fn.strdisplaywidth(corpus[i])
        end
      end
    end)
  )
end

engine('bt', 1)
engine('nfa', 2)
engine('auto', 0)

if wanted('search') then
  load()
  local hits = 0
  say(
    'search',
    ms(function()
      for _, pat in ipairs({ [[\<fox\>]], [[\d\{4}]], [[café\|über]], [[\v(\w+)\(]], [[^\t]] }) do
        api.nvim_win_set_cursor(0, { 1, 0 })
        for _ = 1, 150 do
          if fn.search(pat, 'w') > 0 then
            hits = hits + 1
          end
        end
        hits = hits + fn.searchcount({ pattern = pat, maxcount = 0, timeout = 0 }).total
      end
    end)
  )
  check('search', hits)
end

if wanted('subst') then
  local subs = {
    [[s/\<\(\w\)\(\w*\)\>/\u\1\2/g]],
    [[s/\d\+/<&>/g]],
    [[s/\(foo\|bar\)\(baz\)\?/\2\1/g]],
    [[s/fox/~-~/g]],
    [[s/\<\l\{5}\>/\=toupper(submatch(0))/g]],
    [[s/café\|naïve/\=strchars(submatch(0))/g]],
  }
  local changed = 0
  say(
    'subst',
    ms(function()
      for _, s in ipairs(subs) do
        load()
        vim.cmd('silent! %' .. s .. 'e')
        changed = changed + (api.nvim_buf_get_lines(0, 0, 1, true)[1] ~= corpus[1] and 1 or 0)
      end
    end)
  )
  check('subst', changed)
end

if wanted('matchlist') then
  local n = 0
  say(
    'matchlist',
    ms(function()
      for i = 1, #corpus do
        local s = corpus[i]
        n = n + #fn.matchlist(s, [[\(\w\+\)\s\+\(\w\+\)]])
        n = n + fn.matchstrpos(s, [[\d\+]])[2]
        n = n + #fn.substitute(s, [[\s\+]], '_', 'g')
        n = n + #fn.split(s, [[,\s*]])
      end
    end)
  )
  check('matchlist', n)
end

if wanted('mbyte') then
  load()
  local n
  say(
    'mbyte',
    ms(function()
      vim.o.regexpengine = 1
      n = count_all(MB_PATTERNS)
      vim.o.regexpengine = 2
      n = n + count_all(MB_PATTERNS)
      vim.o.regexpengine = 0
    end)
  )
  check('mbyte', n)
end
