-- Driver for the search/navigation differential sweep; see
-- navsweep.sh.
--
-- Covers the six modules of batch B11:
--
--   fuzzy      fuzzy.rs   -- matchfuzzy()/matchfuzzypos() golden scores
--   search     search.rs  -- the *driver*: offsets, flags, searchpair,
--                            searchcount, 'ignorecase'/'smartcase'/
--                            'magic'/'wrapscan', %/[i/[d/:checkpath.
--                            The regexp engine itself was phase 14.
--   tag        tag.rs     -- a fixture tags corpus x 'tagcase'/
--                            'taglength'/'tagbsearch', the tag stack,
--                            Emacs-format tags and the rejection paths
--   path       path.rs +  -- expand()/glob()/globpath()/findfile()/
--              file_search.rs  finddir()/:find over the fixture tree
--   quickfix   quickfix.rs -- :vimgrep/:grep/:helpgrep/:make, the whole
--                            getqflist() `what` surface, the list stack,
--                            and the quickfix window
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart, so the report never carries a duration, a pid, a
-- wall-clock time or a path outside the work directory.  Two artifacts
-- come out: the readable report on stdout, and a canonical (sorted-key)
-- JSON dump on $NAV_STRUCT of every dict-shaped answer, because the
-- readable form elides fields -- a wrong `qfbufnr` or a dropped
-- `changedtick` is invisible in the report and loud in the struct.

local work = assert(os.getenv('NAV_WORK'), 'NAV_WORK unset')
local tree = work .. '/tree'
local structpath = assert(os.getenv('NAV_STRUCT'), 'NAV_STRUCT unset')
local structfd = assert(io.open(structpath, 'w'))

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''

--- Strip the bits of an answer that name where the run happened.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  -- Messages are truncated to the screen width, which lops the leading
  -- characters off a long path; the tail still has to be recognised.
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub(vim.pesc(work:gsub('/', '\\/')), '<WORK>')
  -- `:make` reports the shell command it ran, which names the per-process
  -- temporary directory nvim made for the redirection.  Both the
  -- directory component and the serial number inside it move every run.
  text = text:gsub('/tmp/nvim%.[^/%s]*/%w+/%d+', '<TMPFILE>')
  text = text:gsub('%s+\n', '\n'):gsub('%s+$', '')
  return text
end

--- Escape to one printable line, so a byte difference shows in the diff.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  vim.json.encode walks a Lua table in hash order,
-- which is stable within a build but is not something to bet a byte
-- oracle on; keys are sorted here instead.  Empty tables are tagged so
-- an empty list and an empty dict stay distinguishable (getqflist()
-- returns both, and they are different answers).
-- ---------------------------------------------------------------------
local function canon(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'number' then
    -- %.14g so a float that is really an integer does not print as
    -- 1.0 on one side and 1 on the other.
    return value == math.floor(value) and string.format('%d', value)
      or string.format('%.14g', value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    return '<' .. kind .. '>'
  end
  if next(value) == nil then
    return vim.tbl_isempty(value) and (vim.islist(value) and '[]' or '{}') or '{}'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for key in pairs(value) do
    keys[#keys + 1] = tostring(key)
  end
  table.sort(keys)
  local parts = {}
  for _, key in ipairs(keys) do
    parts[#parts + 1] = '"' .. esc(key) .. '":' .. canon(value[key])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, ' ', canon(value), '\n')
end

-- ---------------------------------------------------------------------
-- Running things.  An error is observable behaviour: a case that fails
-- has to fail the same way in both binaries, so it is reported, not
-- swallowed.
-- ---------------------------------------------------------------------
local function exec(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
  if ok then
    return scrub(res.output or '')
  end
  return 'ERROR ' .. scrub(res)
end

local function run(label, cmd)
  local out = exec(cmd)
  if out ~= '' then
    emit(label, '|', (out:gsub('\n', '\n' .. label .. ' | ')))
  else
    emit(label, '| (silent)')
  end
end

-- Every fixture file's basename, for the filter below.  Filled in once
-- the tree is known (see `fixture_names`).
local fixture_basenames = {}

--- `run`, minus the "which file am I searching" progress echo.
---
--- :vimgrep/:grep/:helpgrep display the name of the file they are on
--- `if (time(NULL) > seconds)`, with `seconds` starting at 0 -- so the
--- first file always prints and any file that happens to start in a new
--- wall-clock second prints too.  Which ones those are is a function of
--- how fast the machine ran, not of what the code did: an A/B of two
--- identical binaries differed by exactly one of these lines.  A line
--- that is nothing but the name of a fixture file is therefore dropped.
--- Nothing else the quickfix commands print has that shape.
local function runv(label, cmd)
  local out = exec(cmd)
  local kept = {}
  for line in (out .. '\n'):gmatch('([^\n]*)\n') do
    if not fixture_basenames[line] then
      kept[#kept + 1] = line
    end
  end
  while #kept > 0 and kept[#kept] == '' do
    kept[#kept] = nil
  end
  out = table.concat(kept, '\n')
  if out ~= '' then
    emit(label, '|', (out:gsub('\n', '\n' .. label .. ' | ')))
  else
    emit(label, '| (silent)')
  end
end

--- Evaluate a Vimscript expression and report both forms.
local function evalp(label, expr)
  local ok, res = pcall(vim.fn.eval, expr)
  if not ok then
    emit(label, '=', 'ERROR ' .. scrub(res))
    struct(label, 'ERROR ' .. scrub(tostring(res)))
    return nil
  end
  emit(label, '=', scrub(vim.inspect(res):gsub('%s+', ' ')))
  struct(label, res)
  return res
end

--- Call a Lua function, reporting the error rather than dying.
local function try(label, fn)
  local ok, res = pcall(fn)
  if not ok then
    emit(label, '!', scrub(res))
    return nil
  end
  return res
end

--- Call a Vimscript function, answering the error text rather than
--- dying: a rejected argument list is observable behaviour and has to
--- compare equal too.
local function vcall(name, ...)
  local ok, res = pcall(vim.fn[name], ...)
  if ok then
    return res
  end
  return 'ERROR ' .. scrub(tostring(res):gsub('^.*:%s*E', 'E'))
end

local function one(value)
  return (scrub(vim.inspect(value)):gsub('%s+', ' '))
end

local function cursor()
  local pos = vim.api.nvim_win_get_cursor(0)
  return ('(%d,%d)'):format(pos[1], pos[2] + 1)
end

--- Feed keys as if typed, then report where that left the cursor.
--- 'x' so the keys are consumed before this returns; 't' so mappings and
--- the command line behave as they would interactively.
local function feed(keys)
  pcall(vim.fn.feedkeys, vim.api.nvim_replace_termcodes(keys, true, true, true), 'xt')
end

--- Reset to a known editor state between sections.
local function reset()
  exec('silent! %bwipeout!')
  exec('silent! enew!')
  vim.o.ignorecase = false
  vim.o.smartcase = false
  vim.o.magic = true
  vim.o.wrapscan = true
  vim.o.hlsearch = false
  vim.o.incsearch = false
  vim.o.gdefault = false
  vim.o.shortmess = 'filnxtToOF'
  vim.o.report = 9999
  vim.o.more = false
  vim.o.swapfile = false
  vim.o.tagcase = 'followic'
  vim.o.taglength = 0
  vim.o.tagbsearch = true
  vim.o.tagrelative = true
  vim.o.tagstack = true
  vim.o.tagfunc = ''
  vim.o.wildignore = ''
  vim.o.wildignorecase = false
  vim.o.suffixes = '.bak,~,.o,.h,.info,.swp,.obj'
  vim.o.suffixesadd = ''
  vim.o.path = '.,,'
  vim.o.matchpairs = '(:),{:},[:]'
  vim.o.showmatch = false
  vim.o.errorformat = '%f:%l:%c: %t%*[^:]: %m,%f:%l: %m'
  vim.o.grepformat = '%f:%l:%c:%m,%f:%l:%m'
  vim.o.grepprg = 'internal'
  vim.o.makeprg = tree .. '/mk.sh'
  vim.o.quickfixtextfunc = ''
  vim.cmd('silent! cd ' .. vim.fn.fnameescape(tree))
end

local function heading(text)
  emit('')
  emit('===== ' .. text .. ' =====')
end

reset()

-- The fixture basenames, for `runv`'s progress-echo filter.  Collected
-- after the first `reset()` so the working directory is the tree.
for _, path in ipairs(vim.fn.glob(tree .. '/**', true, true)) do
  fixture_basenames[vim.fn.fnamemodify(path, ':t')] = true
end

-- =====================================================================
-- 1. fuzzy.rs -- matchfuzzy()/matchfuzzypos() golden score tables
--
-- Pure functions of their arguments, so this section is exact by
-- construction: any change to the bonus table, the gap penalties, the
-- recursion budget or the tie-break comparator moves a number here.
-- =====================================================================
heading('fuzzy')

local FUZZY_CORPUS = {
  'clay',
  'crayon',
  'declaration',
  'namespace',
  'CamelCaseWord',
  'camel_case_word',
  'a/b/c/clay.txt',
  'src/main.c',
  'src/util.h',
  'xyzzy',
  'ClAy',
  'c l a y',
  'cccclay',
  'yalc',
  '',
  'c',
  'clayclayclay',
  'the quick brown fox',
  'ÀÉÎÕÜ accented',
  'tab\tseparated',
}

local FUZZY_PATS = {
  'clay',
  'cl',
  'c',
  'ay',
  'CCW',
  'ccw',
  'sm',
  'src',
  'xyz',
  'zzz',
  '',
  'c/c',
  'ÀÎ',
  'the fox',
  'cccc',
}

for _, pat in ipairs(FUZZY_PATS) do
  local key = ('fuzzy pat=%q'):format(pat)
  local plain = try(key, function()
    return vim.fn.matchfuzzy(FUZZY_CORPUS, pat)
  end)
  local pos = try(key, function()
    return vim.fn.matchfuzzypos(FUZZY_CORPUS, pat)
  end)
  emit(key, 'matchfuzzy', vim.inspect(plain):gsub('%s+', ' '))
  emit(key, 'matchfuzzypos', vim.inspect(pos):gsub('%s+', ' '))
  struct(key .. ' plain', plain)
  struct(key .. ' pos', pos)
end

-- The option surface: matchseq changes the algorithm, camelcase changes
-- the bonus table, limit changes the sort-then-truncate order, and
-- key/text_cb change what is matched against.
local FUZZY_OPTS = {
  { desc = 'matchseq', opts = { matchseq = true } },
  { desc = 'nocamel', opts = { camelcase = false } },
  { desc = 'limit1', opts = { limit = 1 } },
  { desc = 'limit3', opts = { limit = 3 } },
  { desc = 'limit0', opts = { limit = 0 } },
}
for _, case in ipairs(FUZZY_OPTS) do
  for _, pat in ipairs({ 'clay', 'ccw', 'sm' }) do
    local key = ('fuzzy opt=%s pat=%q'):format(case.desc, pat)
    local got = try(key, function()
      return vim.fn.matchfuzzypos(FUZZY_CORPUS, pat, case.opts)
    end)
    emit(key, vim.inspect(got):gsub('%s+', ' '))
    struct(key, got)
  end
end

-- Dict entries, both `key` and `text_cb`, plus the "no such key" and
-- "wrong type" rejections.
local dicts = {
  { name = 'clay', extra = 1 },
  { name = 'crayon', extra = 2 },
  { name = 'declaration', extra = 3 },
}
struct('fuzzy dict key', try('fuzzy dict key', function()
  return vim.fn.matchfuzzypos(dicts, 'clay', { key = 'name' })
end))
vim.g.nav_textcb = function(item)
  return item.name:upper()
end
struct('fuzzy dict text_cb', try('fuzzy dict text_cb', function()
  return vim.fn.matchfuzzypos(dicts, 'CLAY', { text_cb = vim.g.nav_textcb })
end))
for _, bad in ipairs({
  'matchfuzzy(["a"], 1)',
  'matchfuzzy("notalist", "a")',
  'matchfuzzy([{"a": 1}], "a")',
  'matchfuzzy([{"a": 1}], "a", {"key": "b"})',
  'matchfuzzy(["a"], "a", {"limit": -1})',
  'matchfuzzy(["a"], "a", {"matchseq": "x"})',
  'matchfuzzypos(["a"], "a", {"key": 3})',
}) do
  evalp('fuzzy bad ' .. bad, bad)
end

-- fuzzy.rs is also reached from cmdline and insert-mode completion
-- ('wildoptions'/'completeopt' = fuzzy), which use fuzzy_match_str and
-- search_for_fuzzy_match rather than the matchfuzzy() entry points.
for _, wo in ipairs({ '', 'fuzzy' }) do
  vim.o.wildoptions = wo
  for _, spec in ipairs({
    { 'cn', 'command' },
    { 'sbf', 'command' },
    { 'iskw', 'option' },
    { 'nrfmt', 'option' },
    { 'zzzz', 'command' },
  }) do
    local key = ('fuzzy wildoptions=%s %s %s'):format(wo, spec[2], spec[1])
    struct(key, vcall('getcompletion', spec[1], spec[2]))
  end
end
vim.o.wildoptions = ''

-- =====================================================================
-- 2. search.rs -- the search driver
-- =====================================================================
heading('search')
reset()
exec('edit ' .. vim.fn.fnameescape(tree .. '/hay.txt'))

-- 2a. search() flag matrix.  Every flag the driver understands, against
-- a pattern that occurs before, at and after the cursor.
local SEARCH_FLAGS = {
  '',
  'b',
  'c',
  'e',
  'n',
  'p',
  's',
  'w',
  'W',
  'z',
  'bc',
  'be',
  'bn',
  'bW',
  'cn',
  'ce',
  'ne',
  'np',
  'sW',
  'ez',
  'bcnpsw',
}
for _, pat in ipairs({ 'alpha', 'Fox', 'fox', '^alpha', 'alpha$', 'nomatch' }) do
  for _, flags in ipairs(SEARCH_FLAGS) do
    vim.api.nvim_win_set_cursor(0, { 4, 5 })
    local got = vcall('search', pat, flags)
    emit(('search pat=%s flags=%q'):format(pat, flags), '->', tostring(got), 'at', cursor())
  end
end

-- searchpos() adds the sub-match column that search() drops, and takes a
-- stopline and a timeout; timeout is deliberately 0 (no limit) so the
-- answer is not a function of how fast the machine is.
for _, pat in ipairs({ 'alpha', 'a\\(l\\)pha', 'fox' }) do
  for _, flags in ipairs({ '', 'b', 'p', 'e', 'n' }) do
    vim.api.nvim_win_set_cursor(0, { 4, 5 })
    local got = vcall('searchpos', pat, flags, 0, 0)
    emit(('searchpos pat=%s flags=%q'):format(pat, flags), '->', one(got))
  end
end
for _, stop in ipairs({ 0, 1, 4, 5, 10 }) do
  vim.api.nvim_win_set_cursor(0, { 4, 5 })
  emit(
    'searchpos stopline=' .. stop,
    '->',
    one(vcall('searchpos', 'alpha', '', stop, 0))
  )
end

-- 2b. search offsets.  These go through do_search, not search(): the
-- offset is parsed off the command line and applied after the match.
local OFFSETS = {
  '/alpha/',
  -- A bare sign is its own arm of the offset parser (it means +/-1) and
  -- is not reachable through any of the numbered forms.
  '/alpha/+',
  '/alpha/-',
  '/alpha/e+',
  '/alpha/e-',
  '/alpha/s+',
  '/alpha/s-',
  '?alpha?+',
  '?alpha?-',
  '/alpha/e',
  '/alpha/e+1',
  '/alpha/e-2',
  '/alpha/s',
  '/alpha/s+3',
  '/alpha/s-1',
  '/alpha/b',
  '/alpha/b+2',
  '/alpha/+1',
  '/alpha/-1',
  '/alpha/2',
  '/alpha/;/beta/',
  '/alpha/;?fox?',
  '?alpha?',
  '?alpha?e',
  '?alpha?s-1',
  '?alpha?+2',
}
for _, spec in ipairs(OFFSETS) do
  vim.api.nvim_win_set_cursor(0, { 4, 0 })
  local key = 'offset ' .. spec
  local out = exec('silent! normal! ' .. spec .. '\r')
  emit(key, '->', cursor(), out ~= '' and out or '')
  -- `n` repeats with the same offset; that the offset survives the
  -- repeat is a property of the saved search pattern, not of the search.
  exec('silent! normal! n')
  emit(key .. ' then-n', '->', cursor())
  exec('silent! normal! N')
  emit(key .. ' then-N', '->', cursor())
end

-- 2c. option interactions.  'smartcase' only bites when 'ignorecase' is
-- on and only for a typed pattern, which is why the * command (which
-- builds its own pattern) is in the table.
for _, ic in ipairs({ false, true }) do
  for _, scs in ipairs({ false, true }) do
    for _, magic in ipairs({ false, true }) do
      vim.o.ignorecase, vim.o.smartcase, vim.o.magic = ic, scs, magic
      for _, pat in ipairs({ 'fox', 'Fox', 'FOX', 'f.x', 'f\\.x', 'a\\+', 'a+' }) do
        vim.api.nvim_win_set_cursor(0, { 1, 0 })
        emit(
          ('opts ic=%s scs=%s magic=%s pat=%s'):format(ic, scs, magic, pat),
          '->',
          tostring(vcall('search', pat, 'w')),
          cursor(),
          'again=' .. tostring(vcall('search', pat, 'n'))
        )
      end
    end
  end
end
reset()
exec('edit ' .. vim.fn.fnameescape(tree .. '/hay.txt'))

-- 'wrapscan' off is a different message and a different failure.
for _, ws in ipairs({ true, false }) do
  vim.o.wrapscan = ws
  vim.api.nvim_win_set_cursor(0, { 10, 0 })
  run('wrapscan=' .. tostring(ws) .. ' forward', 'normal! /alpha\r')
  emit('wrapscan=' .. tostring(ws) .. ' forward at', cursor())
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
  run('wrapscan=' .. tostring(ws) .. ' back', 'normal! ?alpha\r')
  emit('wrapscan=' .. tostring(ws) .. ' back at', cursor())
end
vim.o.wrapscan = true

-- 2d. the word-under-cursor commands, which build their own pattern.
for _, cmd in ipairs({ '*', '#', 'g*', 'g#' }) do
  for _, at in ipairs({ { 1, 4 }, { 3, 0 }, { 4, 0 } }) do
    vim.api.nvim_win_set_cursor(0, at)
    exec('silent! normal! ' .. cmd)
    emit(('word %s at (%d,%d)'):format(cmd, at[1], at[2] + 1), '->', cursor(), '@/=' .. vim.fn.getreg('/'))
  end
end

-- 2e. searchcount(): the whole option surface, with timeout 0 so the
-- `incomplete` field is a function of maxcount alone.  (The cmdline
-- search-stat message built on top of this is a known debug-build
-- timing flake -- Test_search_stat_option -- so the message is asked for
-- separately, below, with the same fixed limits.)
vim.fn.setreg('/', 'alpha')
for _, at in ipairs({ { 1, 0 }, { 4, 0 }, { 4, 17 }, { 10, 0 } }) do
  vim.api.nvim_win_set_cursor(0, at)
  for _, opts in ipairs({
    { timeout = 0 },
    { timeout = 0, maxcount = 2 },
    { timeout = 0, maxcount = 0 },
    { timeout = 0, recompute = false },
    { timeout = 0, pattern = 'fox' },
    { timeout = 0, pattern = 'alpha', pos = { 3, 1, 0 } },
  }) do
    local key = ('searchcount at=(%d,%d) %s'):format(at[1], at[2] + 1, canon(opts))
    local got = try(key, function()
      return vim.fn.searchcount(opts)
    end)
    emit(key, '->', vim.inspect(got):gsub('%s+', ' '))
    struct(key, got)
  end
end

-- 2f. searchpair()/searchpairpos(): flags, skip expressions, stopline.
reset()
exec('edit ' .. vim.fn.fnameescape(tree .. '/pairs.txt'))
for _, case in ipairs({
  { '(', '', ')', '' },
  { '(', '', ')', 'b' },
  { '(', '', ')', 'n' },
  { '(', '', ')', 'W' },
  { '(', '', ')', 'bW' },
  { '\\[', '', '\\]', '' },
  { '{', '', '}', '' },
  { '{', '', '}', 'b' },
  { '#if', '#else', '#endif', 'W' },
  { '<tag', '', '</tag', 'W' },
}) do
  for _, at in ipairs({ { 1, 6 }, { 2, 0 }, { 6, 0 }, { 9, 0 } }) do
    vim.api.nvim_win_set_cursor(0, at)
    local key = ('searchpair %s|%s|%s flags=%q at=(%d,%d)'):format(
      case[1],
      case[2],
      case[3],
      case[4],
      at[1],
      at[2] + 1
    )
    local got = try(key, function()
      return vim.fn.searchpair(case[1], case[2], case[3], case[4], '', 0, 0)
    end)
    emit(key, '->', tostring(got), 'at', cursor())
    vim.api.nvim_win_set_cursor(0, at)
    local pos = try(key, function()
      return vim.fn.searchpairpos(case[1], case[2], case[3], case[4], '', 0, 0)
    end)
    emit(key .. ' pos', '->', vim.inspect(pos):gsub('%s+', ' '))
  end
end
-- A skip expression that lies about every match, and one that is an
-- error: the second is the "skip evaluation failed" path.
vim.api.nvim_win_set_cursor(0, { 1, 6 })
evalp('searchpair skip-all', [[searchpair('(', '', ')', '', '1', 0, 0)]])
vim.api.nvim_win_set_cursor(0, { 1, 6 })
evalp('searchpair skip-err', [[searchpair('(', '', ')', '', 'nosuchfunc()', 0, 0)]])
vim.api.nvim_win_set_cursor(0, { 5, 0 })
evalp('searchpair skip-syn', [[searchpair('(', '', ')', '', 'getline(".") =~ "string"', 0, 0)]])
vim.api.nvim_win_set_cursor(0, { 1, 6 })
evalp('searchpair stopline', [[searchpair('(', '', ')', '', '', 1, 0)]])

-- 2g. '%' and the matchpairs family -- findmatchlimit, and the line
-- comment skip it consults.
for _, mps in ipairs({ '(:),{:},[:]', '(:),{:}', '<:>,(:)' }) do
  vim.o.matchpairs = mps
  for _, at in ipairs({ { 1, 6 }, { 1, 14 }, { 1, 25 }, { 2, 3 }, { 3, 3 }, { 4, 3 }, { 9, 0 } }) do
    vim.api.nvim_win_set_cursor(0, at)
    exec('silent! normal! %')
    emit(('percent mps=%s at=(%d,%d)'):format(mps, at[1], at[2] + 1), '->', cursor())
  end
end
vim.o.matchpairs = '(:),{:},[:]'
for _, cmd in ipairs({ '[(', '[{', '])', ']}', '[#', ']#' }) do
  for _, at in ipairs({ { 1, 20 }, { 2, 10 }, { 7, 0 } }) do
    vim.api.nvim_win_set_cursor(0, at)
    exec('silent! normal! ' .. cmd)
    emit(('bracket %s at=(%d,%d)'):format(cmd, at[1], at[2] + 1), '->', cursor())
  end
end

-- 2g2. The findmatchlimit modes pairs.txt does not reach: the C-comment
-- ends ([/ ]/ [* ]*), FM_BLOCKSTOP ([m ]m [[ ]]), the 'cpoptions' flags
-- '%' and 'M' that switch smart matching and backslash counting off,
-- 'rightleft' (which flips the direction), the whole Lisp branch --
-- comments, strings and #\( -- and raw strings, which are only reachable
-- through 'cindent'.
local mp = tree .. '/mp'

--- Put the cursor somewhere in a fixture and report where `keys` left it.
local function mpkeys(label, file, at, keys)
  exec('silent! edit! ' .. vim.fn.fnameescape(mp .. '/' .. file))
  vim.api.nvim_win_set_cursor(0, at)
  exec('silent! normal! ' .. keys)
  emit(('mp %s %s at=(%d,%d)'):format(label, keys, at[1], at[2] + 1), '->', cursor())
end

reset()
for _, keys in ipairs({ '[/', ']/', '[*', ']*', '%' }) do
  for _, at in ipairs({ { 1, 7 }, { 1, 21 }, { 2, 0 }, { 3, 18 }, { 5, 2 }, { 6, 20 }, { 7, 9 } }) do
    mpkeys('cmt', 'cmt.c', at, keys)
  end
end
for _, keys in ipairs({ '[m', ']m', '[[', ']]', '[{', ']}' }) do
  for _, at in ipairs({ { 4, 4 }, { 3, 8 }, { 6, 2 }, { 9, 0 } }) do
    mpkeys('blocks', 'blocks.c', at, keys)
  end
end

-- 'cpoptions' "%" turns smart matching off (braces in quotes and comments
-- then count); "M" stops backslashes before a brace from counting.
local escaped = { 'plain ( a ) end', 'esc \\( a \\) end', 'mix \\( a ) end', '"quoted ( a )" ( b )' }
for _, cpo in ipairs({ 'aABceFs', 'aABceFs%', 'aABceFsM', 'aABceFs%M' }) do
  vim.o.cpoptions = cpo
  exec('silent! enew!')
  vim.api.nvim_buf_set_lines(0, 0, -1, false, escaped)
  for lnum = 1, #escaped do
    for _, col in ipairs({ 6, 7, 10 }) do
      vim.api.nvim_win_set_cursor(0, { lnum, col })
      exec('silent! normal! %')
      emit(('mp cpo=%s at=(%d,%d)'):format(cpo, lnum, col + 1), '->', cursor())
    end
  end
end
-- Back to the default, underscore included.
vim.o.cpoptions = 'aABceFs_'

-- 'rightleft' is guessed at: the walk goes the other way.
exec('silent! edit! ' .. vim.fn.fnameescape(tree .. '/pairs.txt'))
for _, rl in ipairs({ true, false }) do
  vim.wo.rightleft = rl
  -- The braces and the angle brackets matter as much as the parens: the
  -- flip is keyed on a fixed set of eight characters.
  for _, at in ipairs({ { 1, 6 }, { 1, 48 }, { 2, 3 }, { 1, 23 }, { 1, 30 }, { 2, 14 }, { 2, 26 }, { 9, 0 } }) do
    vim.api.nvim_win_set_cursor(0, at)
    exec('silent! normal! %')
    emit(('mp rl=%s at=(%d,%d)'):format(tostring(rl), at[1], at[2] + 1), '->', cursor())
  end
end
vim.wo.rightleft = false

-- The Lisp branch: ';' is the comment character, but not inside a string
-- and not after "#\".
exec('silent! edit! ' .. vim.fn.fnameescape(mp .. '/code.lisp'))
for _, lisp in ipairs({ true, false }) do
  vim.bo.lisp = lisp
  -- The last three sit *inside* a Lisp comment, which confines the walk
  -- to it; the seventh is after a "#\\"" that must not open a string.
  for _, at in ipairs({
    { 1, 6 },
    { 2, 2 },
    { 2, 11 },
    { 3, 4 },
    { 3, 12 },
    { 3, 22 },
    { 7, 11 },
    { 7, 13 },
    { 8, 0 },
    { 8, 9 },
    { 2, 40 },
    { 5, 21 },
    { 5, 27 },
    { 5, 5 },
    { 6, 2 },
  }) do
    for _, keys in ipairs({ '%', '[(', '])' }) do
      vim.api.nvim_win_set_cursor(0, at)
      exec('silent! normal! ' .. keys)
      emit(
        ('mp lisp=%s %s at=(%d,%d)'):format(tostring(lisp), keys, at[1], at[2] + 1),
        '->',
        cursor()
      )
    end
  end
end
vim.bo.lisp = false

-- Raw strings: 'cindent' asks findmatchlimit for the start of one, so
-- re-indenting the file is the only way in.
exec('silent! edit! ' .. vim.fn.fnameescape(mp .. '/raw.cpp'))
vim.bo.cindent = true
exec('silent! normal! gg=G')
emit('mp cindent raw', '|', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), '\\n')))
vim.bo.cindent = false
exec('silent! edit!')
exec('silent! edit! ' .. vim.fn.fnameescape(mp .. '/cmt.c'))
vim.bo.cindent = true
exec('silent! normal! gg=G')
emit('mp cindent cmt', '|', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), '\\n')))
vim.bo.cindent = false
exec('silent! edit!')
reset()

-- 2h. searchc: f/t/F/T and their repeats, including 'cpoptions' ";".
reset()
exec('edit ' .. vim.fn.fnameescape(tree .. '/hay.txt'))
for _, keys in ipairs({ 'fo', '2fo', 'to', 'Fo', 'To', 'fo;', 'fo,', 'to;', 'to,', 'fz' }) do
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
  exec('silent! normal! ' .. keys)
  emit('charsearch ' .. keys, '->', cursor())
end

-- 2i. find_pattern_in_path: [i ]i [I ]I [d ]d [D ]D, :ilist, :dlist,
-- :isearch, :dsearch, :ijump, :djump and :checkpath.
reset()
vim.o.path = '.,' .. tree .. '/inc,' .. tree .. '/src'
vim.o.include = '^\\s*#\\s*include'
vim.o.define = '^\\s*#\\s*define'
exec('edit ' .. vim.fn.fnameescape(tree .. '/src/main.c'))
vim.api.nvim_win_set_cursor(0, { 3, 9 })
for _, cmd in ipairs({
  'ilist MAIN_LIMIT',
  'ilist! MAIN_LIMIT',
  'dlist MAIN_LIMIT',
  'dlist! MAIN_LIMIT',
  'isearch MAIN_LIMIT',
  'dsearch MAIN_LIMIT',
  'ilist util_run',
  'dlist UTIL_LIMIT',
  'ilist nosuchthing',
  'checkpath',
  'checkpath!',
  '2,3ilist MAIN_LIMIT',
  'ijump MAIN_LIMIT',
}) do
  run('path ' .. cmd, 'silent! ' .. cmd)
  emit('path ' .. cmd .. ' at', vim.fn.expand('%:t') .. ' ' .. cursor())
  exec('silent! edit ' .. vim.fn.fnameescape(tree .. '/src/main.c'))
  vim.api.nvim_win_set_cursor(0, { 3, 9 })
end
for _, cmd in ipairs({ '[i', ']i', '[I', ']I', '[d', ']d', '[D', ']D', '[ ', '] ' }) do
  exec('silent! edit ' .. vim.fn.fnameescape(tree .. '/src/main.c'))
  vim.api.nvim_win_set_cursor(0, { 5, 30 })
  run('bracket-path ' .. cmd, 'silent! normal! ' .. cmd)
end

-- 2i2. find_pattern_in_path in depth.  The whole suite reaches this
-- function through test_checkpath and test_find_complete only, so
-- everything its walk distinguishes is driven from here over the
-- fixture's own include tree ($tree/ipath):
--
--   the file stack   a two-deep chain, a diamond (the same header
--                    reached twice -- the "already searched" branch),
--                    headers that are not there in both the "" and the
--                    <> form, and a bare "#include"
--   'include'        plain, with \zs, buffer-local, empty, and
--                    'includeexpr' on top of it
--   'define'         plain, buffer-local, empty, and a backslash-
--                    continued definition (show_pat_in_path's own loop,
--                    once per line source: buffer and included file)
--   comments         :ilist vs :ilist! over a file of comment shapes --
--                    that bang is exactly the skip_comments flag
--   actions          SHOW, SHOW_ALL, GOTO, SPLIT, EXPAND and CHECK_PATH,
--                    with counts, ranges, the /regexp/ (whole == false)
--                    form and the three "not found" messages
local ip = tree .. '/ipath'

--- Run one find_pattern_in_path command from a known editor state and
--- report both what it printed and where it left the editor.  Windows
--- are reset too: :isplit, :dsplit and :psearch make them.
local function ipcmd(label, cmd, opts)
  opts = opts or {}
  exec('silent! only!')
  exec('silent! %bwipeout!')
  exec('silent! edit ' .. vim.fn.fnameescape(ip .. '/' .. (opts.file or 'top.c')))
  vim.api.nvim_win_set_cursor(0, opts.at or { 8, 20 })
  run('ip ' .. label, 'silent! ' .. cmd)
  emit(
    'ip ' .. label .. ' at',
    vim.fn.expand('%:t'),
    cursor(),
    'wins=' .. vim.fn.winnr('$')
  )
end

reset()
vim.o.path = '.,' .. ip
vim.o.include = '^\\s*#\\s*include'
vim.o.define = '^\\s*#\\s*define'

-- The include tree itself.  IP_LIMIT is defined in two different files
-- of one chain; ipleaf is declared in the chain's leaf and again in its
-- head, which is what makes the diamond visible.
for _, cmd in ipairs({
  'checkpath',
  'checkpath!',
  'ilist IP_LIMIT',
  'ilist! IP_LIMIT',
  'dlist IP_LIMIT',
  'dlist! IP_LIMIT',
  'ilist ipleaf',
  'ilist! ipleaf',
  'dlist IP_CONT',
  'dlist IP_LEAF',
  'ilist IP_ANGLE',
  'ilist nosuchword',
  'dlist nosuchword',
  'isearch ipleaf',
  -- A leading number is a *range* (which lines of the buffer the walk
  -- starts over); the count goes in front of the pattern.
  '2isearch ipleaf',
  '3isearch ipleaf',
  'isearch 2 ipleaf',
  'isearch 3 ipleaf',
  'isearch 9 ipleaf',
  'dsearch 2 IP_LIMIT',
  'dsearch IP_LEAF',
  'ijump ipleaf',
  '2ijump ipleaf',
  'ijump 2 ipleaf',
  'djump IP_LEAF',
  'djump 2 IP_LIMIT',
  'isplit ipleaf',
  'dsplit IP_LEAF',
  'psearch ipleaf',
  'isearch /ipl.*f/',
  'ilist /IP_[A-Z]*/',
  'ilist! /ip/',
  'isearch nosuchword',
  'dsearch nosuchword',
  'isearch ipmain',
  '1,2ilist IP_TOP',
  '7,8ilist IP_TOP',
  '1ilist IP_LIMIT',
}) do
  ipcmd(cmd, cmd)
end

-- The comment rules, at depth -1 (the current buffer).  The bang is the
-- skip_comments flag and nothing else.
for _, cmd in ipairs({
  'ilist cmtword',
  'ilist! cmtword',
  'dlist cmtword',
  'dlist! cmtword',
  'isearch cmtword',
  'ilist /cmtw/',
}) do
  ipcmd('cmt ' .. cmd, cmd, { file = 'cmt.c', at = { 1, 4 } })
end

-- The same continued #define, this time reached through an included
-- file rather than out of the buffer: show_pat_in_path reads its
-- continuation lines with vim_fgets there and with ml_get here.
for _, cmd in ipairs({ 'dlist IP_CONT', 'dsearch IP_CONT', 'ilist IP_CONT' }) do
  ipcmd('cont ' .. cmd, cmd, { file = 'cont.c', at = { 2, 22 } })
end

-- 'include' variants.  \zs changes which half of the line names the
-- file *and* how CHECK_PATH isolates the name when the file is missing;
-- an empty 'include' turns the walk off entirely.
for _, inc in ipairs({
  '',
  '^\\s*#\\s*include',
  '^\\s*#\\s*include\\s*\\zs["<][^">]*[">]',
  '^\\s*#\\s*include\\s*["<]\\zs[^">]*\\ze[">]',
  '#\\s*include',
}) do
  vim.o.include = inc
  for _, cmd in ipairs({ 'checkpath', 'checkpath!', 'ilist ipleaf', 'ilist IP_LIMIT' }) do
    ipcmd(("inc=%s %s"):format(esc(inc), cmd), cmd)
  end
end
vim.o.include = '^\\s*#\\s*include'

-- 'includeexpr' rewrites the name the walk resolves; the CHECK_PATH
-- display prefers the resolved name over the text on the line for
-- exactly that reason.
for _, iexpr in ipairs({
  'substitute(v:fname, "^chain", "chain", "")',
  'substitute(v:fname, "^ipgone.h$", "angle.h", "")',
  'v:fname . ".missing"',
}) do
  vim.o.includeexpr = iexpr
  for _, cmd in ipairs({ 'checkpath', 'checkpath!', 'ilist ipleaf' }) do
    ipcmd(('iexpr=%s %s'):format(esc(iexpr), cmd), cmd)
  end
end
vim.o.includeexpr = ''

-- Buffer-local 'include'/'define' win over the global ones; an empty
-- 'define' leaves FIND_DEFINE with no define pattern at all, which is
-- the "match anywhere on the line" fallback.
ipcmd('local-inc', 'setlocal include=^#include\\ \\" | ilist ipleaf')
ipcmd('local-def', 'setlocal define=^#define\\ IP_L | dlist IP_LIMIT')
ipcmd('local-def-leaf', 'setlocal define=^#define\\ IP_L | dlist IP_LEAF')
for _, def in ipairs({ '', '^\\s*#\\s*def', 'define' }) do
  vim.o.define = def
  for _, cmd in ipairs({ 'dlist IP_LIMIT', 'dsearch IP_LEAF', 'dlist /IP_/' }) do
    ipcmd(('def=%s %s'):format(esc(def), cmd), cmd)
  end
end
vim.o.define = '^\\s*#\\s*define'

-- 'path' variants: the entry list is what decides whether a header is
-- found at all, and CHECK_PATH reports the misses.
for _, path in ipairs({ '.', '', ',,', ip, '.,' .. ip .. ',' .. tree .. '/inc', '.,./nowhere' }) do
  vim.o.path = path
  for _, cmd in ipairs({ 'checkpath', 'checkpath!', 'ilist ipleaf' }) do
    ipcmd(('path=%s %s'):format(esc(path), cmd), cmd)
  end
end
vim.o.path = '.,' .. ip

-- The normal-mode half.  A count turns skip_comments off, and the case
-- of the character picks the action; the cursor sits on `ipleaf` in the
-- body of top.c.
for _, keys in ipairs({
  '[i',
  ']i',
  '[I',
  ']I',
  '[d',
  ']d',
  '[D',
  ']D',
  '[\t',
  ']\t',
  '[\4',
  ']\4',
  '2[i',
  '2[I',
  '2]i',
  '3[I',
}) do
  ipcmd('key ' .. esc(keys), 'normal! ' .. keys, { at = { 8, 30 } })
end
for _, keys in ipairs({ '[i', '[I', '[d', '[D' }) do
  ipcmd('cmt key ' .. esc(keys), 'normal! ' .. keys, { file = 'cmt.c', at = { 1, 4 } })
end

-- ACTION_EXPAND: insert-mode CTRL-X CTRL-I (identifiers in included
-- files) and CTRL-X CTRL-D (definitions).  Nothing else in the sweep
-- reaches that branch, and it is a quarter of the function -- the
-- "Scanning included file" message, the word-boundary walk, the
-- continue-onto-the-next-line case and ins_compl_add_infercase.
-- Cycling with CTRL-N k times reveals the k-th match in order, which is
-- the list the walk built.
-- The last case repeats CTRL-X CTRL-I on an already-completed word,
-- which is the "adding" branch: the walk then reads *past* the match to
-- pick up the word after it, following on to the next line when the
-- match ends one.
for _, case in ipairs({
  { 'ctrl-x-i ip', 'ip', '\24\9' },
  { 'ctrl-x-i IP', 'IP', '\24\9' },
  { 'ctrl-x-d IP', 'IP', '\24\4' },
  { 'ctrl-x-d ip', 'ip', '\24\4' },
  { 'ctrl-x-i cha', 'cha', '\24\9' },
  { 'ctrl-x-i again', 'ipl', '\24\9\24\9' },
  { 'ctrl-x-d again', 'IP_L', '\24\4\24\4' },
}) do
  local label, prefix, keys = case[1], case[2], case[3]
  for n = 0, 4 do
    exec('silent! only!')
    exec('silent! %bwipeout!')
    exec('silent! edit ' .. vim.fn.fnameescape(ip .. '/top.c'))
    vim.api.nvim_win_set_cursor(0, { 8, 0 })
    feed('o' .. prefix .. keys .. ('\14'):rep(n) .. '\27')
    emit(
      ('ip %s +%d'):format(label, n),
      '->',
      esc(vim.api.nvim_get_current_line()),
      'at',
      cursor()
    )
  end
end
exec('silent! only!')
exec('silent! %bwipeout!')

-- Leave the options as section 2i had them, so the sections after this
-- one see the state they were written against.
vim.o.path = '.,' .. tree .. '/inc,' .. tree .. '/src'
vim.o.include = '^\\s*#\\s*include'
vim.o.define = '^\\s*#\\s*define'

-- 2j. gd/gD, and 'gn'/'gN' (current_search).
reset()
exec('edit ' .. vim.fn.fnameescape(tree .. '/src/util.c'))
vim.api.nvim_win_set_cursor(0, { 2, 20 })
exec('silent! normal! gd')
emit('gd', '->', cursor())
vim.api.nvim_win_set_cursor(0, { 2, 20 })
exec('silent! normal! gD')
emit('gD', '->', cursor())
exec('edit ' .. vim.fn.fnameescape(tree .. '/hay.txt'))
vim.fn.setreg('/', 'alpha')
for _, keys in ipairs({ 'gn', 'gN', '2gn', 'cgnX' }) do
  exec('silent! edit! ' .. vim.fn.fnameescape(tree .. '/hay.txt'))
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
  vim.fn.setreg('/', 'alpha')
  exec('silent! normal! ' .. keys)
  emit('gn ' .. keys, '->', cursor(), 'line=' .. esc(vim.api.nvim_get_current_line()))
end
exec('silent! edit! ' .. vim.fn.fnameescape(tree .. '/hay.txt'))

-- 2k. the search-stat message.  Asked for with the same fixed limits as
-- searchcount() above; 'shortmess' without S is what turns it on.
vim.o.shortmess = 'filnxtToOF'
vim.opt.shortmess:remove('S')
for _, at in ipairs({ { 1, 0 }, { 4, 0 } }) do
  vim.api.nvim_win_set_cursor(0, unpack({ at }))
  run('searchstat', 'silent! normal! /alpha\r')
  emit('searchstat at', cursor())
end
vim.o.shortmess = 'filnxtToOF'

-- 2l. showmatch, and the search error messages.
vim.o.showmatch = true
vim.o.matchtime = 0
exec('silent! edit ' .. vim.fn.fnameescape(tree .. '/pairs.txt'))
exec('silent! normal! ggo(a[b]c)\27')
emit('showmatch line', esc(vim.api.nvim_get_current_line()))
vim.o.showmatch = false
exec('silent! undo')
for _, cmd in ipairs({
  'normal! /\\(\r',
  'normal! /nomatchatall\r',
  'normal! ?nomatchatall\r',
  '/alpha/nosuchoffset',
  'normal! /alpha/e+\r',
}) do
  run('search error ' .. cmd, 'silent! ' .. cmd)
end
evalp('search bad flag', [[search('a', 'q')]])
evalp('search bad timeout', [[search('a', '', 0, -1)]])
evalp('searchcount bad', [[searchcount({'maxcount': -1})]])
evalp('searchpair bad', [[searchpair('(', '', ')', 'q')]])

-- =====================================================================
-- 3. tag.rs
-- =====================================================================
heading('tags')
reset()
vim.o.tags = tree .. '/tags-sorted'

local function tagstate(label)
  emit(label, 'file=' .. vim.fn.expand('%:t'), 'at=' .. cursor())
  struct(label .. ' stack', vim.fn.gettagstack())
end

-- 3a. taglist() over the corpus, under every 'tagcase' and every
-- 'taglength'.  taglist() is the one entry point that reports the
-- parsed fields rather than jumping, so it is where a parser change
-- shows up in full.
for _, tagsopt in ipairs({
  tree .. '/tags-sorted',
  tree .. '/tags-unsorted',
  tree .. '/tags-static',
  tree .. '/tags-fields',
  tree .. '/TAGS',
  tree .. '/tags-sorted,' .. tree .. '/tags-unsorted',
  tree .. '/tags-nofields',
  tree .. '/tags-truncated',
  tree .. '/tags-sortedlie',
  tree .. '/tags-nul',
  tree .. '/tags-empty',
  tree .. '/tags-huge-line',
  tree .. '/tags-missing',
}) do
  vim.o.tags = tagsopt
  local short = tagsopt:gsub(vim.pesc(tree .. '/'), '')
  for _, pat in ipairs({ 'alpha', 'Alpha', 'ALPHA', 'beta', '^alpha$', 'al.*', 'zeta', 'emacstag', 'nope' }) do
    local key = ('taglist tags=%s pat=%s'):format(short, pat)
    local got = try(key, function()
      return vim.fn.taglist(pat)
    end)
    emit(key, '->', scrub(vim.inspect(got):gsub('%s+', ' ')))
    struct(key, got)
  end
  struct('tagfiles tags=' .. short, vim.fn.tagfiles())
end

vim.o.tags = tree .. '/tags-sorted'
for _, tc in ipairs({ 'followic', 'followscs', 'ignore', 'match', 'smart' }) do
  for _, ic in ipairs({ false, true }) do
    for _, scs in ipairs({ false, true }) do
      vim.o.tagcase, vim.o.ignorecase, vim.o.smartcase = tc, ic, scs
      for _, pat in ipairs({ 'alpha', 'Alpha', 'ALPHA' }) do
        local key = ('taglist tc=%s ic=%s scs=%s pat=%s'):format(tc, ic, scs, pat)
        local got = vim.fn.taglist(pat)
        emit(key, '->', #got, table.concat(vim.tbl_map(function(t)
          return t.name .. '@' .. vim.fn.fnamemodify(t.filename, ':t')
        end, got), ','))
      end
    end
  end
end
reset()
vim.o.tags = tree .. '/tags-sorted'
for _, tl in ipairs({ 0, 1, 3, 5 }) do
  for _, bs in ipairs({ true, false }) do
    vim.o.taglength, vim.o.tagbsearch = tl, bs
    for _, pat in ipairs({ 'alpha', 'alphabet', 'alp', 'a' }) do
      local got = vim.fn.taglist(pat)
      emit(('taglist tl=%d bsearch=%s pat=%s'):format(tl, bs, pat), '->', #got,
        table.concat(vim.tbl_map(function(t) return t.name end, got), ','))
    end
  end
end
reset()
vim.o.tags = tree .. '/tags-sorted'

-- 3b. the tag stack: :tag, :tnext, :tprev, :tfirst, :tlast, :pop,
-- :tags, and the two errors at the ends of the stack.
for _, cmd in ipairs({
  'tag alpha',
  'tnext',
  'tprevious',
  'tfirst',
  'tlast',
  'tnext',
  'pop',
  'tag',
  'tag beta',
  'tnext',
  'tnext',
  'pop',
  'pop',
  'pop',
  'tag nosuchtag',
  'tag /^al',
  'tag /nomatch',
  'tag! alpha',
}) do
  run('tagcmd ' .. cmd, 'silent! ' .. cmd)
  tagstate('tagcmd ' .. cmd)
end
run('tags listing', 'tags')

-- 3c. gettagstack()/settagstack() -- the whole dict, and every action.
struct('gettagstack initial', vim.fn.gettagstack())
for _, case in ipairs({
  { action = 'r', dict = { items = { { tagname = 'alpha', from = { 1, 1, 1, 0 } } } } },
  { action = 'a', dict = { items = { { tagname = 'beta', from = { 1, 2, 1, 0 } } } } },
  { action = 't', dict = { items = { { tagname = 'gamma', from = { 1, 3, 1, 0 } } } } },
  { action = 'r', dict = { curidx = 1 } },
  { action = 'r', dict = { length = 0 } },
  { action = 'x', dict = { items = {} } },
}) do
  local key = 'settagstack ' .. case.action .. ' ' .. canon(case.dict)
  local rc = try(key, function()
    return vim.fn.settagstack(vim.fn.win_getid(), case.dict, case.action)
  end)
  emit(key, '->', tostring(rc))
  struct(key, vim.fn.gettagstack())
end
evalp('settagstack badwin', [[settagstack(9999, {'items': []}, 'r')]])
evalp('gettagstack badwin', [[gettagstack(9999)]])

-- 3d. :ptag / :stag / :ltag and the preview and split behaviour.
reset()
vim.o.tags = tree .. '/tags-sorted'
for _, cmd in ipairs({ 'ptag alpha', 'pclose', 'stag beta', 'close', 'ltag alpha', 'lclose' }) do
  run('tagwin ' .. cmd, 'silent! ' .. cmd)
  emit('tagwin ' .. cmd .. ' wins', tostring(#vim.api.nvim_list_wins()))
end
struct('ltag loclist', vim.fn.getloclist(0))

-- 3e. :tselect / :tjump / g] / g] -- the listing print_tag_list produces.
-- The prompt is answered by feeding a choice, which is why this is fed
-- rather than executed.
reset()
vim.o.tags = tree .. '/tags-sorted'
for _, keys in ipairs({ ':tselect alpha\r1\r', ':tselect beta\r2\r', ':tjump alpha\r', ':tjump beta\r1\r' }) do
  exec('silent! %bwipeout!')
  local out = try('tselect ' .. keys, function()
    return vim.fn.execute('redir => g:nav_ts | silent! ' .. 'echo ""' .. ' | redir END')
  end)
  local _ = out
  feed(keys)
  emit('tselect ' .. esc(keys), '->', vim.fn.expand('%:t'), cursor())
end

-- 3f. 'tagfunc'.
reset()
vim.cmd([[
  function! NavTagFunc(pattern, flags, info) abort
    if a:pattern ==# 'bad'
      return 'notalist'
    elseif a:pattern ==# 'badentry'
      return [{'name': 'x'}]
    elseif a:pattern ==# 'none'
      return v:null
    elseif a:pattern ==# 'err'
      throw 'tagfunc blew up'
    endif
    return [{'name': a:pattern, 'filename': 'src/main.c', 'cmd': '1', 'kind': 'f'}]
  endfunction
]])
vim.o.tagfunc = 'NavTagFunc'
vim.o.tags = tree .. '/tags-sorted'
for _, pat in ipairs({ 'fromfunc', 'bad', 'badentry', 'none', 'err' }) do
  local key = 'tagfunc ' .. pat
  local got = try(key, function()
    return vim.fn.taglist(pat)
  end)
  emit(key, '->', scrub(vim.inspect(got):gsub('%s+', ' ')))
  struct(key, got)
  run(key .. ' :tag', 'silent! tag ' .. pat)
end
vim.o.tagfunc = ''

-- 3g. tag name completion (expand_tags) and 'tagrelative'.
reset()
vim.o.tags = tree .. '/tags-sorted'
struct('tag completion', vim.fn.getcompletion('a', 'tag'))
struct('tag completion listfiles', vim.fn.getcompletion('a', 'tag_listfiles'))
for _, rel in ipairs({ true, false }) do
  vim.o.tagrelative = rel
  local got = vim.fn.taglist('alpha')
  emit('tagrelative=' .. tostring(rel), '->', scrub(vim.inspect(got[1] and got[1].filename)))
end
vim.o.tagrelative = true
for _, ts in ipairs({ true, false }) do
  vim.o.tagstack = ts
  exec('silent! tag alpha')
  emit('tagstack=' .. tostring(ts), 'depth=' .. tostring(vim.fn.gettagstack().length))
  exec('silent! 999pop')
end

-- =====================================================================
-- 4. path.rs + file_search.rs
-- =====================================================================
heading('path')
reset()

local GLOBS = {
  '*',
  '*.txt',
  '*.[ch]',
  '?.txt',
  'a*',
  'sub/*',
  'sub/**',
  '**/a.txt',
  '**1/a.txt',
  '**5/a.txt',
  'chain/**/a.txt',
  'chain/**3/a.txt',
  'chain/**8/bottom.txt',
  'chain/**200/bottom.txt',
  'sub/**/*.txt',
  'nosuch*',
  'src/*.c',
  '.*',
  'sp*.txt',
  'u-*.txt',
  'dir[12]/*',
  '{a,b}.txt',
  'a.txt',
  '~',
  '$HOME',
  '**/*.h',
}
for _, pat in ipairs(GLOBS) do
  emit('glob ' .. pat, '->', scrub(esc(vim.fn.glob(pat))))
  struct('glob ' .. pat, vim.fn.glob(pat, false, true))
  struct('glob-nosuf ' .. pat, vim.fn.glob(pat, true, true))
  struct('glob-links ' .. pat, vim.fn.glob(pat, false, true, true))
  emit('expand ' .. pat, '->', scrub(esc(vim.fn.expand(pat))))
  struct('expand-list ' .. pat, vim.fn.expand(pat, false, true))
end
for _, wic in ipairs({ false, true }) do
  vim.o.wildignorecase = wic
  for _, pat in ipairs({ 'A*', 'SUB/*', 'a.TXT' }) do
    struct(('glob wildignorecase=%s %s'):format(wic, pat), vim.fn.glob(pat, false, true))
    struct(('expand wildignorecase=%s %s'):format(wic, pat), vim.fn.expand(pat, false, true))
  end
end
vim.o.wildignorecase = false
for _, wi in ipairs({ '', '*.o,*.log', '*.txt', 'sub/*' }) do
  vim.o.wildignore = wi
  struct('glob wildignore=' .. wi, vim.fn.glob('*', false, true))
  struct('glob wildignore=' .. wi .. ' nosuf', vim.fn.glob('*', true, true))
end
vim.o.wildignore = ''
for _, suf in ipairs({ '.bak,~,.o,.h', '', '.txt' }) do
  vim.o.suffixes = suf
  struct('glob suffixes=' .. suf, vim.fn.glob('a*', false, true))
end
vim.o.suffixes = '.bak,~,.o,.h,.info,.swp,.obj'

for _, args in ipairs({
  { tree, '*.txt' },
  { tree .. ',' .. tree .. '/sub', 'a.txt' },
  { tree .. '/**', 'a.txt' },
  { '.,sub,sub/deep', 'a.txt' },
  { tree, 'nosuch' },
}) do
  local key = ('globpath %s | %s'):format(scrub(args[1]), args[2])
  emit(key, '->', scrub(esc(vim.fn.globpath(args[1], args[2]))))
  struct(key, vim.fn.globpath(args[1], args[2], false, true))
end

-- findfile()/finddir(): the 'path' grammar (upward `;`, downward `**`,
-- stop directories) and 'suffixesadd'.
for _, pathopt in ipairs({
  '.,,',
  '.,sub,sub/deep',
  tree .. '/**',
  tree .. '/**2',
  'sub/deep;' .. tree,
  '.;',
  ',,',
  tree .. '/nosuch',
  -- Over the 12-deep chain: `**` defaults to 100 levels, `**3` stops
  -- short of the bottom, and `**200` is clamped by
  -- FF_MAX_STAR_STAR_EXPAND.  Without a tree this deep those limits are
  -- unobservable -- a mutation of the clamp went undetected until the
  -- chain was added.
  tree .. '/chain/**',
  tree .. '/chain/**3',
  tree .. '/chain/**8',
  tree .. '/chain/**200',
}) do
  for _, name in ipairs({ 'a.txt', 'a', 'c.txt', 'x', 'bottom.txt', 'nosuch' }) do
    for _, sa in ipairs({ '', '.txt,.h' }) do
      vim.o.path, vim.o.suffixesadd = pathopt, sa
      local key = ('findfile path=%s sa=%s name=%s'):format(scrub(pathopt), sa, name)
      emit(key, '->', scrub(vim.fn.findfile(name)))
      struct(key .. ' all', vim.fn.findfile(name, '', -1))
      for _, count in ipairs({ 1, 2, 3 }) do
        emit(key .. ' #' .. count, '->', scrub(vim.fn.findfile(name, '', count)))
      end
    end
  end
end
vim.o.path, vim.o.suffixesadd = '.,,', ''
for _, name in ipairs({ 'sub', 'deep', 'dir1', 'nosuch', '.' }) do
  for _, pathopt in ipairs({ '.,,', tree .. '/**', 'sub;' .. tree }) do
    vim.o.path = pathopt
    local key = ('finddir path=%s name=%s'):format(scrub(pathopt), name)
    emit(key, '->', scrub(vim.fn.finddir(name)))
    struct(key .. ' all', vim.fn.finddir(name, '', -1))
  end
end
vim.o.path = '.,,'
evalp('findfile explicit', ('findfile("a.txt", %q)'):format(tree .. '/sub'))
evalp('finddir explicit', ('finddir("deep", %q)'):format(tree .. '/sub'))
evalp('findfile star-star-limit', ('findfile("a.txt", %q)'):format(tree .. '/**31'))
evalp('findfile bad count', [[findfile("a.txt", "", "x")]])

-- :find and its completion (uniquefy_paths is what makes the candidate
-- list readable, and it only runs when several matches share a tail).
reset()
vim.o.path = '.,sub,sub/deep,sub/other,' .. tree .. '/**'
struct('find completion a', vim.fn.getcompletion('a', 'file_in_path'))
struct('find completion a.', vim.fn.getcompletion('a.', 'file_in_path'))
struct('find completion nosuch', vim.fn.getcompletion('nosuch', 'file_in_path'))
for _, cmd in ipairs({ 'find a.txt', '2find a.txt', '99find a.txt', 'find nosuch', 'sfind c.txt', 'close' }) do
  run('findcmd ' .. cmd, 'silent! ' .. cmd)
  emit('findcmd ' .. cmd .. ' at', scrub(vim.fn.expand('%')))
end

-- The pure name functions.
reset()
local NAMES = {
  '/a/b/c.txt',
  '/a/b/../c.txt',
  '/a/./b//c.txt',
  'a/b/c.txt',
  './a/b',
  '../a/b',
  '/',
  '//',
  '///a',
  '',
  'a',
  '/a/b/',
  'http://x/y',
  'ftp://x',
  'file:///a/b',
  'x://',
  '~/a',
  '$HOME/a',
  'a\\ b',
}
for _, name in ipairs(NAMES) do
  emit('simplify ' .. esc(name), '->', esc(vim.fn.simplify(name)))
  emit('pathshorten ' .. esc(name), '->', esc(vim.fn.pathshorten(name)))
  emit('pathshorten2 ' .. esc(name), '->', esc(vim.fn.pathshorten(name, 2)))
  emit('glob2regpat ' .. esc(name), '->', esc(vim.fn.glob2regpat(name)))
  emit('fnameescape ' .. esc(name), '->', esc(vim.fn.fnameescape(name)))
  for _, mods in ipairs({ ':p', ':h', ':t', ':r', ':e', ':p:h', ':~', ':.', ':s?a?A?', ':gs?a?A?' }) do
    local ok, got = pcall(vim.fn.fnamemodify, name, mods)
    emit(('fnamemodify %s %s'):format(esc(name), mods), '->', ok and esc(scrub(got)) or 'ERROR')
  end
end
for _, name in ipairs({ 'a.txt', 'a-link.txt', 'dangling', 'sub/alink', 'sub', 'nosuch' }) do
  emit('resolve ' .. name, '->', scrub(esc(vim.fn.resolve(name))))
  emit('fullpath ' .. name, '->', scrub(esc(vim.fn.fnamemodify(name, ':p'))))
end
for _, exe in ipairs({ 'sh', 'nosuchbinary', './mk.sh', 'mk.sh', '/bin/sh' }) do
  emit('exepath ' .. exe, '->', scrub(esc(vim.fn.exepath(exe))))
  emit('executable ' .. exe, '->', tostring(vim.fn.executable(exe)))
end
-- expand()'s special forms, plus the backtick escape (expand_backtick).
for _, spec in ipairs({
  '%',
  '%:p',
  '#',
  '<cword>',
  '<cfile>',
  '<sfile>',
  '<afile>',
  '$NOSUCHVAR',
  '$HOME',
  '`echo hi`',
  '`=1+1`',
  '`nosuchcommand-xyz`',
  '\\%',
}) do
  exec('silent! edit ' .. vim.fn.fnameescape(tree .. '/a.txt'))
  local ok, got = pcall(vim.fn.expand, spec)
  emit('expand-special ' .. esc(spec), '->', ok and scrub(esc(got)) or 'ERROR ' .. scrub(got))
end
-- has_special_wildchar: a backtick or brace in a name that is also
-- globbed sends expansion down a different road.
for _, pat in ipairs({ 'a{1}.txt', "a'b", 'a`b', '**/a.txt', '*/**/a.txt' }) do
  struct('special-wild ' .. pat, vim.fn.glob(pat, false, true))
end

-- =====================================================================
-- 5. quickfix.rs
-- =====================================================================
heading('quickfix')
reset()

-- Every `what` key the getter understands, asked for one at a time so a
-- key that starts answering differently is attributable.
local WHAT_KEYS = {
  'all',
  'changedtick',
  'context',
  'efm',
  'id',
  'idx',
  'items',
  'lines',
  'nr',
  'qfbufnr',
  'quickfixtextfunc',
  'size',
  'title',
  'winid',
}
local function dumpqf(label, loc)
  local function get(what)
    if loc then
      return what == nil and vcall('getloclist', 0) or vcall('getloclist', 0, what)
    end
    return what == nil and vcall('getqflist') or vcall('getqflist', what)
  end
  struct(label .. ' plain', get(nil))
  for _, key in ipairs(WHAT_KEYS) do
    local what = { [key] = key == 'nr' and 0 or (key == 'lines' and { 'x.c:1: y' } or 0) }
    struct(('%s what=%s'):format(label, key), get(what))
  end
  struct(label .. ' what=nr$', get({ nr = '$' }))
  struct(label .. ' what=all,nr$', get({ nr = '$', all = 0 }))
end

-- 5a. :vimgrep and friends.
for _, cmd in ipairs({
  'vimgrep /alpha/ ' .. tree .. '/*.txt',
  'vimgrep /alpha/g ' .. tree .. '/*.txt',
  'vimgrep /alpha/j ' .. tree .. '/*.txt',
  'vimgrep /alpha/gj ' .. tree .. '/*.txt',
  'vimgrep /alpha/f ' .. tree .. '/*.txt',
  'vimgrepadd /beta/j ' .. tree .. '/*.txt',
  'vimgrep /nomatchxyz/j ' .. tree .. '/*.txt',
  'vimgrep /alpha/j ' .. tree .. '/nosuch*',
  'vimgrep /alpha/j **/*.txt',
  '2vimgrep /alpha/j ' .. tree .. '/*.txt',
  'lvimgrep /alpha/j ' .. tree .. '/*.txt',
  'lvimgrepadd /beta/j ' .. tree .. '/*.txt',
  'vimgrep alpha ' .. tree .. '/hay.txt',
  'vimgrep /\\(/j ' .. tree .. '/hay.txt',
  'vimgrep',
}) do
  runv('vimgrep ' .. cmd, 'silent! ' .. cmd)
  emit('vimgrep ' .. cmd .. ' size', tostring(vim.fn.getqflist({ size = 0 }).size))
  struct('vimgrep ' .. cmd, vim.fn.getqflist({ all = 0 }))
  struct('vimgrep ' .. cmd .. ' loc', vim.fn.getloclist(0, { all = 0 }))
end
dumpqf('after-vimgrep', false)
dumpqf('after-lvimgrep', true)

-- 5b. :grep with the internal grep, and :make with a fixed program.
reset()
vim.o.grepprg = 'internal'
for _, cmd in ipairs({
  'grep /alpha/ ' .. tree .. '/*.txt',
  'grepadd /beta/ ' .. tree .. '/*.txt',
  'lgrep /alpha/ ' .. tree .. '/*.txt',
  'grep /nomatchxyz/ ' .. tree .. '/*.txt',
}) do
  runv('grep ' .. cmd, 'silent! ' .. cmd)
  struct('grep ' .. cmd, vim.fn.getqflist({ all = 0 }))
end
reset()
vim.o.makeprg = tree .. '/mk.sh'
vim.o.shellpipe = '>%s 2>&1'
for _, cmd in ipairs({ 'make', 'make!', 'lmake' }) do
  run('make ' .. cmd, 'silent! ' .. cmd)
  struct('make ' .. cmd, vim.fn.getqflist({ all = 0 }))
end

-- 5c. :helpgrep, pointed at the fixture doc/ tree.
reset()
-- The fixture doc/ tree *replaces* the real runtime here: :helpgrep
-- walks every 'runtimepath' doc directory, and pointing it at
-- runtime/doc would make this section move whenever the shipped help
-- does.  The code path is the same either way.
vim.o.runtimepath = tree .. '/rtp'
for _, cmd in ipairs({ 'helpgrep alpha', 'helpgrep nomatchxyz', 'lhelpgrep beta', 'helpgrep alpha@en' }) do
  runv('helpgrep ' .. cmd, 'silent! ' .. cmd)
  struct('helpgrep ' .. cmd, vim.fn.getqflist({ all = 0 }))
end
reset()

-- 5d. the errorformat surface: :cfile/:cgetfile/:caddfile/:cbuffer/
-- :cexpr, over the fixture error files and the multiline/dirstack
-- formats.
local EFMS = {
  { desc = 'basic', efm = '%f:%l:%c: %t%*[^:]: %m,%f:%l: %m', file = 'errors-basic.txt' },
  { desc = 'types', efm = '%E%f:%l: %m,%W%f:%l:%c: %m,%I%f:%l: note %m', file = 'errors-basic.txt' },
  {
    desc = 'multiline',
    efm = '%AError in %f line %l,%C  %m,%Zend of error',
    file = 'errors-multiline.txt',
  },
  { desc = 'dirstack', efm = '%DEntering dir `%f\',%XLeaving dir%.%#,%f:%l: %m', file = 'errors-dirstack.txt' },
  { desc = 'ignore', efm = '%-Gplain%.%#,%f:%l: %m', file = 'errors-none.txt' },
  { desc = 'general', efm = '%+Gplain%.%#,%f:%l: %m', file = 'errors-none.txt' },
  { desc = 'pushpop', efm = '%f:%l: %m,%O%m', file = 'errors-basic.txt' },
  { desc = 'nomatch', efm = '%f@@%l@@%m', file = 'errors-basic.txt' },
}
for _, case in ipairs(EFMS) do
  for _, cmd in ipairs({ 'cfile', 'cgetfile', 'caddfile', 'lfile', 'lgetfile' }) do
    reset()
    vim.o.errorformat = case.efm
    run(('efm %s %s'):format(case.desc, cmd), ('silent! %s %s/%s'):format(cmd, tree, case.file))
    struct(('efm %s %s'):format(case.desc, cmd), vim.fn.getqflist({ all = 0 }))
    struct(('efm %s %s loc'):format(case.desc, cmd), vim.fn.getloclist(0, { all = 0 }))
  end
end
reset()
for _, expr in ipairs({
  "cexpr ['src/main.c:1: from expr']",
  "caddexpr ['src/util.c:2: added']",
  "cgetexpr ['src/util.h:3: got']",
  'cexpr []',
  "cexpr 'src/main.c:9: string form'",
  'cexpr 42',
  "lexpr ['src/main.c:1: loc expr']",
}) do
  run('cexpr ' .. expr, 'silent! ' .. expr)
  struct('cexpr ' .. expr, vim.fn.getqflist({ all = 0 }))
end
reset()
exec('silent! enew!')
vim.api.nvim_buf_set_lines(0, 0, -1, false, {
  'src/main.c:5:1: from buffer one',
  'src/util.c:2:1: from buffer two',
})
for _, cmd in ipairs({ 'cbuffer', 'cgetbuffer', 'caddbuffer', '1cbuffer', '1,1cbuffer', 'lbuffer' }) do
  run('cbuffer ' .. cmd, 'silent! ' .. cmd)
  struct('cbuffer ' .. cmd, vim.fn.getqflist({ all = 0 }))
end

-- 5e. setqflist()/setloclist(): every action, the `what` form, and the
-- rejections.
reset()
local ENTRIES = {
  { filename = tree .. '/a.txt', lnum = 1, col = 2, text = 'entry one', type = 'E', nr = 11 },
  { filename = tree .. '/b.txt', lnum = 2, text = 'entry two', type = 'W', valid = 0 },
  { bufnr = vim.fn.bufnr('%'), lnum = 3, text = 'entry three', vcol = 1, pattern = 'alpha' },
  { module = 'mod', lnum = 4, text = 'entry four', end_lnum = 5, end_col = 6 },
  { text = 'entry five', user_data = { a = 1 } },
}
for _, action in ipairs({ ' ', 'a', 'r', 'f', 'u' }) do
  local key = 'setqflist action=' .. action
  local rc = try(key, function()
    return vim.fn.setqflist(ENTRIES, action)
  end)
  emit(key, '->', tostring(rc), 'size=' .. tostring(vim.fn.getqflist({ size = 0 }).size))
  struct(key, vim.fn.getqflist({ all = 0 }))
end
for _, what in ipairs({
  { title = 'a title' },
  { context = { any = 'thing' } },
  { items = ENTRIES },
  { lines = { 'src/main.c:1: from lines' }, efm = '%f:%l: %m' },
  { nr = '$', title = 'newest' },
  { idx = 2 },
  { quickfixtextfunc = 'NavQfTf' },
  { nr = 99, title = 'out of range' },
  { id = 99999, title = 'bad id' },
}) do
  local key = 'setqflist what=' .. canon(what)
  local rc = try(key, function()
    return vim.fn.setqflist({}, 'r', what)
  end)
  emit(key, '->', tostring(rc))
  struct(key, vim.fn.getqflist({ all = 0 }))
end
evalp('setqflist bad list', [[setqflist('notalist')]])
evalp('setqflist bad action', [[setqflist([], 'z')]])
evalp('setloclist bad win', [[setloclist(9999, [])]])
evalp('getloclist bad win', [[getloclist(9999)]])

-- 5f. the list stack: ten lists, then :colder/:cnewer to both ends.
reset()
for i = 1, 12 do
  exec(('silent! cexpr ["src/main.c:%d: stack entry %d"]'):format(i, i))
end
struct('stack after-12', vim.fn.getqflist({ nr = '$' }))
for _, cmd in ipairs({
  'colder',
  'colder 3',
  'colder 99',
  'cnewer',
  'cnewer 5',
  'cnewer 99',
  'chistory',
  '3chistory',
}) do
  run('stack ' .. cmd, 'silent! ' .. cmd)
  struct('stack ' .. cmd, vim.fn.getqflist({ nr = 0, title = 0, idx = 0, size = 0, id = 0 }))
end
for i = 0, 13 do
  struct('stack nr=' .. i, vim.fn.getqflist({ nr = i, all = 0 }))
end

-- 5g. navigation and the quickfix window.
reset()
exec('silent! cexpr ["' .. tree .. '/a.txt:1:1: one", "' .. tree .. '/a.txt:3:1: two", "'
  .. tree .. '/b.txt:2:1: three"]')
for _, cmd in ipairs({
  'cc',
  'cc 2',
  'cnext',
  'cnext',
  'cnext',
  'cprevious',
  'cfirst',
  'clast',
  'cnfile',
  'cpfile',
  'crewind',
  'cbottom',
  'cabove',
  'cbelow',
  'cafter',
  'cbefore',
  'cc 99',
}) do
  run('qfnav ' .. cmd, 'silent! ' .. cmd)
  emit('qfnav ' .. cmd .. ' at', scrub(vim.fn.expand('%:t')) .. ' ' .. cursor(),
    'idx=' .. tostring(vim.fn.getqflist({ idx = 0 }).idx))
end
-- 'switchbuf' picks between reusing a window, splitting, opening a tab
-- and staying put; it is the whole of qf_jump_to_usable_window's
-- decision table and nothing else in the sweep varies it.
for _, sb in ipairs({ '', 'useopen', 'usetab', 'split', 'vsplit', 'newtab', 'uselast', 'useopen,split' }) do
  exec('silent! only')
  exec('silent! %bwipeout!')
  exec('silent! cexpr ["' .. tree .. '/a.txt:1:1: sb one", "' .. tree .. '/b.txt:2:1: sb two"]')
  vim.o.switchbuf = sb
  exec('silent! copen')
  run('switchbuf=' .. sb, 'silent! cc 1')
  emit('switchbuf=' .. sb, 'wins=' .. #vim.api.nvim_list_wins(),
    'tabs=' .. #vim.api.nvim_list_tabpages(), 'buf=' .. vim.fn.expand('%:t'), 'at=' .. cursor())
  run('switchbuf=' .. sb .. ' next', 'silent! cnext')
  emit('switchbuf=' .. sb .. ' next', 'wins=' .. #vim.api.nvim_list_wins(),
    'tabs=' .. #vim.api.nvim_list_tabpages(), 'buf=' .. vim.fn.expand('%:t'), 'at=' .. cursor())
end
vim.o.switchbuf = ''
exec('silent! tabonly')
exec('silent! only')
reset()
exec('silent! cexpr ["' .. tree .. '/a.txt:1:1: one", "' .. tree .. '/a.txt:3:1: two", "'
  .. tree .. '/b.txt:2:1: three"]')
for _, cmd in ipairs({ 'copen', 'cclose', 'cwindow', 'copen 5', 'cclose', 'lopen', 'lclose', 'lwindow' }) do
  run('qfwin ' .. cmd, 'silent! ' .. cmd)
  emit('qfwin ' .. cmd .. ' wins', tostring(#vim.api.nvim_list_wins()))
end
exec('silent! copen')
emit('qfwin buffer', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), ' | ')))
struct('qfwin qfbufnr', vim.fn.getqflist({ qfbufnr = 0, winid = 0 }))
emit('qfwin bt', vim.bo.buftype .. ' ' .. vim.bo.filetype .. ' ' .. tostring(vim.bo.modifiable))
exec('silent! cclose')

-- 'quickfixtextfunc': the entry text becomes a function of the list.
vim.cmd([[
  function! NavQfTf(info) abort
    let l:items = getqflist({'id': a:info.id, 'items': 1}).items
    let l:out = []
    for l:i in range(a:info.start_idx - 1, a:info.end_idx - 1)
      call add(l:out, 'QFTF#' . (l:i + 1) . ':' . l:items[l:i].text)
    endfor
    return l:out
  endfunction
]])
vim.o.quickfixtextfunc = 'NavQfTf'
exec('silent! copen')
emit('qftf buffer', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), ' | ')))
exec('silent! cclose')
vim.o.quickfixtextfunc = ''

-- 5h. :cdo/:cfdo/:ldo/:lfdo.
reset()
exec('silent! cexpr ["' .. tree .. '/a.txt:1:1: one", "' .. tree .. '/a.txt:2:1: two", "'
  .. tree .. '/b.txt:1:1: three"]')
for _, cmd in ipairs({
  'cdo echo expand("%:t") . ":" . line(".")',
  'cfdo echo expand("%:t")',
  '2cdo echo "count"',
  'cdo nosuchcommand',
  'ldo echo "loc"',
}) do
  run('cdo ' .. cmd, 'silent! ' .. cmd)
end

-- 5i. qf_mark_adjust: entries follow the lines they point at.
reset()
exec('silent! edit! ' .. vim.fn.fnameescape(tree .. '/hay.txt'))
exec('silent! cexpr ["' .. tree .. '/hay.txt:4:1: marked"]')
struct('markadjust before', vim.fn.getqflist({ items = 0 }))
exec('silent! 1delete')
struct('markadjust after-delete', vim.fn.getqflist({ items = 0 }))
exec('silent! 1put =\'inserted\'')
struct('markadjust after-put', vim.fn.getqflist({ items = 0 }))
exec('silent! edit!')

-- 5j. the error paths.
reset()
for _, cmd in ipairs({
  'cc',
  'cnext',
  'cprevious',
  'colder',
  'cnewer',
  'll',
  'lnext',
  'lopen',
  'lwindow',
  'clist',
  'clist 1,2',
  'clist!',
  'cfile /nonexistent/file',
  'cbuffer 9999',
  'helpgrep',
  'vimgrep //j ' .. tree .. '/a.txt',
}) do
  run('qferr ' .. cmd, 'silent! ' .. cmd)
end
reset()
exec('silent! cexpr ["' .. tree .. '/a.txt:1:1: listed", "' .. tree .. '/b.txt:2:1: also"]')
for _, cmd in ipairs({ 'clist', 'clist!', 'clist 1', 'clist 1,2', 'clist -1', 'clist +1' }) do
  run('clist ' .. cmd, 'silent! ' .. cmd)
end

-- Location lists are per-window: a split shares, a new list does not.
reset()
exec('silent! lexpr ["' .. tree .. '/a.txt:1:1: loc one"]')
exec('silent! split')
struct('loclist after-split', vim.fn.getloclist(0, { all = 0 }))
exec('silent! lexpr ["' .. tree .. '/b.txt:1:1: loc two"]')
struct('loclist new-in-split', vim.fn.getloclist(0, { all = 0 }))
exec('silent! wincmd p')
struct('loclist other-window', vim.fn.getloclist(0, { all = 0 }))
exec('silent! only')
struct('loclist after-only', vim.fn.getloclist(0, { all = 0 }))
dumpqf('final-qf', false)
dumpqf('final-loc', true)

emit('')
emit('===== end =====')
structfd:close()
