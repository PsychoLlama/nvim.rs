-- Driver for the fold-TREE differential sweep; see foldsweep.sh.
--
-- Covers crates/nvim/src/fold/ -- mod.rs, level.rs, open_close.rs,
-- adjust.rs, marker.rs, text.rs, builtins.rs, session.rs -- which is
-- batch B-fold.  Phase 20's survey (p20-1 §D.2 GAP 3) found that the
-- fold *display* is watched from six directions (scrsweep 77 refs,
-- diffsweep, utfsweep, mousesweep, stlsweep) and fold *persistence* by
-- sessgold's `put_view` block, while the fold TREE under edits was
-- watched by nothing: `tests/unit/fold.rs` reaches
-- `foldMarkAdjustRecurse` and stops there.  This is that oracle.
--
-- The question every case asks is "what does the fold tree look like",
-- and the answer is read back through the only four windows the editor
-- opens onto it:
--
--   foldlevel(l)        per line, for every line -- the nesting depth
--   foldclosed(l)       / foldclosedend(l) -- which ranges are closed
--   foldtextresult(l)   what a closed fold renders as
--   the cursor          after zj/zk/[z/]z, which is how the tree's
--                       boundaries are observable at all
--
-- ... plus, for the cases that ask for it, a LEVEL SCAN: 'foldlevel' is
-- walked 0..N and the closed ranges recorded at each step.  Two trees
-- that agree at one 'foldlevel' need not agree at the next, so the scan
-- is what turns "which lines are closed" into "what shape is the tree".
--
-- Sections:
--   s01 zf/zF creation over motions, counts, Visual, `:{range}fold`,
--       and the E350/E490 arms.
--   s02 nesting: outer-then-inner, inner-then-outer, and the
--       overlapping cases that SPLIT or MERGE an existing fold.
--   s03 deletion: zd zD zE, nested, on a closed fold, with counts.
--   s04 open/close: zo zO zc zC za zA zR zM zr zm zv zx zX zn zN zi,
--       with counts, from inside nested folds.
--   s05 motions: zj zk [z ]z, nested, and their no-move arms.
--   s06 foldmethod=indent, 'shiftwidth', blank lines, re-indent.
--   s07 foldmethod=marker: default and custom 'foldmarker', numbered
--       markers, unbalanced markers, markers written by zf and removed
--       by zd -- the one method where the TEXT is part of the answer.
--   s08 foldmethod=expr: every foldexpr verdict (0 1 2 a1 s1 <1 >1 = -1
--       and the two special ones), a Vimscript and a Lua expr.
--   s09 foldmethod=syntax with a real syntax region.
--   s10 foldmethod=diff across two windows, and foldUpdate after the
--       diff is re-computed.
--   s11 'foldminlines' 0/1/2/5.
--   s12 'foldnestmax' 1/2/3 for indent, expr and syntax.
--   s13 'foldignore' for indent.
--   s14 'foldenable' / 'foldlevel' / 'foldlevelstart' and what a fresh
--       window inherits.
--   s15 the ADJUST path: :move :d :put dd o :g//d :s//\r/ across and
--       inside fold boundaries, and undo of each.
--   s16 'foldtext', foldtextresult() and the default fold text.
--   s17 the eval builtins' error arms (line 0, negative, past the end).
--   s18 :foldopen :foldclose :foldd{o,oclosed} with ranges.
--   s19 the same commands run UNCAPTURED, so E350/E351/E490 and the
--       "N lines folded" messages reach the .stderr artifact.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart from two working directories, so the report carries no
-- address, pid, wall-clock time or path outside the work directory.
--
-- FOLDSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.
-- FOLDSWEEP_TRACE=1 mirrors each section name to stderr, which is the
-- only way to see where a wedged run stopped.  It must be OFF for a
-- baseline: the trace goes into the compared .stderr artifact.

local work = assert(os.getenv('FOLD_WORK'), 'FOLD_WORK unset')
local structpath = assert(os.getenv('FOLD_STRUCT'), 'FOLD_STRUCT unset')

local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('FOLDSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('FOLDSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub('0x%x+', '<ADDR>')
  text = text:gsub('<lambda>%d+', '<lambda>')
  text = text:gsub('<SNR>%d+_', '<SNR>_')
  text = text:gsub('%S*/target/debug/nvim', '<NVIM>')
  text = text:gsub('%S*/nvim%-%x+', '<NVIM>')
  return text
end

--- Escape to one printable line.  The fold text carries 'fillchars'
--- multibyte glyphs and the marker fixtures carry braces, so which
--- BYTES came back is part of the answer.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  Verbatim from opsweep.lua / varsweep.lua: the
-- artifacts are read side by side often enough that they must escape
-- and sort the same way.
-- ---------------------------------------------------------------------
local canon

local function canon_number(value)
  if value ~= value then
    return '"nan"'
  elseif value == math.huge then
    return '"inf"'
  elseif value == -math.huge then
    return '"-inf"'
  elseif value == math.floor(value) and math.abs(value) < 2 ^ 53 then
    return string.format('%d', value)
  end
  return string.format('%.17g', value)
end

canon = function(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'nil' then
    return 'nil'
  elseif kind == 'number' then
    return canon_number(value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    return '"<' .. kind .. '>"'
  end
  if next(value) == nil then
    return getmetatable(value) and '{}' or '[]'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys, byname = {}, {}
  for key in pairs(value) do
    local name = type(key) == 'string' and esc(key)
      or ('<' .. (type(key) == 'table' and canon(key) or scrub(tostring(key))) .. '>')
    keys[#keys + 1] = name
    byname[name] = key
  end
  table.sort(keys)
  local parts = {}
  for _, name in ipairs(keys) do
    parts[#parts + 1] = '"' .. name .. '":' .. canon(value[byname[name]])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  if structfd then
    structfd:write(label, ' ', canon(value), '\n')
  end
end

--- Normalise an error to its message: a pcall against vim.fn or
--- vim.api prefixes the Lua source position, which is a line number in
--- this file and would re-baseline the artifact on any edit.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^nvim_exec2%(%), line %d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

-- ---------------------------------------------------------------------
-- The world a case runs in.
-- ---------------------------------------------------------------------

-- Every fold option, at a fixed value, so a case that forgets to
-- restore one cannot poison its neighbours.  'fillchars' is pinned
-- rather than left at its default because the DEFAULT is display
-- policy that scrsweep owns; what this sweep wants is a stable glyph
-- under foldtextresult().  'foldtext' likewise: nvim's default has
-- moved between releases and this oracle is not the place to notice.
local DEFAULTS = table.concat({
  'setlocal foldmethod=manual foldenable foldlevel=0 foldlevelstart=-1',
  'setlocal foldminlines=1 foldnestmax=20 foldignore=# foldcolumn=0',
  'setlocal foldmarker={{{,}}} foldexpr=0 foldtext=foldtext()',
  'setlocal shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab autoindent&',
  'setlocal diff& nomodified',
  'set fillchars=vert:|,fold:-,foldopen:-,foldsep:|,foldclose:+',
  'set foldopen=block,hor,mark,percent,quickfix,search,tag,undo foldclose=',
  'set report=9999 shortmess=filnxtToOFS nomore noshowmode',
  'set scrolloff=0 sidescrolloff=0 wrap& list& conceallevel=0',
}, ' | ')

local BUF

local function quiet(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = false })
  if not ok then
    return errtext(res)
  end
  return nil
end

local function feed(keys)
  local codes = vim.api.nvim_replace_termcodes(keys, true, true, true)
  local ok, err = pcall(vim.api.nvim_feedkeys, codes, 'ntx', false)
  if not ok then
    return errtext(err)
  end
  return nil
end

--- Back to Normal mode from wherever the previous case left off.  Done
--- at the START of a reset rather than the end of a case, so the mode
--- and the cursor a case leaves behind are still reported honestly.
local function normalise()
  pcall(
    vim.api.nvim_feedkeys,
    vim.api.nvim_replace_termcodes('<C-\\><C-N>', true, true, true),
    'ntx',
    false
  )
end

local CUR_LINES = {}

--- Put the world in a known state.  `lines` is the buffer, `pos` the
--- cursor as {row, col}, `opts` a `:setlocal` argument string applied
--- on top of DEFAULTS.
---
--- ORDER MATTERS.  'foldmethod' goes back to `manual` BEFORE zE runs:
--- under `marker` zE deletes every marker from the TEXT, and under any
--- computed method it raises E350 instead of clearing anything.  The
--- folds of the previous case would then survive into this one, which
--- is exactly the class of cross-talk that makes a fold artifact a
--- function of the case above it rather than of itself.
local function reset(lines, pos, opts)
  normalise()
  quiet('silent! setlocal foldmethod=manual foldenable')
  quiet('silent! normal! zE')
  quiet(DEFAULTS)
  quiet('silent! normal! zE')
  vim.api.nvim_buf_set_lines(BUF, 0, -1, false, lines)
  CUR_LINES = vim.deepcopy(lines)
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  if opts and opts ~= '' then
    quiet('setlocal ' .. opts)
  end
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
end

-- ---------------------------------------------------------------------
-- Reading the tree back.
-- ---------------------------------------------------------------------

--- foldlevel() for every line, as one digit per line.  A level above 9
--- (only 'foldnestmax' cases can reach it) is spelled `[N]` so the
--- string stays injective.
local function levels()
  local n = vim.api.nvim_buf_line_count(BUF)
  local parts, raw = {}, {}
  for l = 1, n do
    local ok, lv = pcall(vim.fn.foldlevel, l)
    lv = ok and lv or -1
    raw[#raw + 1] = lv
    parts[#parts + 1] = (lv >= 0 and lv <= 9) and tostring(lv) or ('[' .. tostring(lv) .. ']')
  end
  return table.concat(parts), raw
end

--- Every CLOSED range, walking top to bottom.  foldclosed() answers the
--- first line of the closed fold containing `l`, so walking from line 1
--- and jumping past foldclosedend() enumerates them without ever asking
--- about a line already covered.
local function closedranges()
  local n = vim.api.nvim_buf_line_count(BUF)
  local shown, raw = {}, {}
  local l = 1
  while l <= n do
    local ok, s = pcall(vim.fn.foldclosed, l)
    s = ok and s or -1
    if s >= 1 then
      local ok2, e = pcall(vim.fn.foldclosedend, l)
      e = (ok2 and e >= s) and e or s
      shown[#shown + 1] = string.format('%d-%d', s, e)
      raw[#raw + 1] = { s, e }
      l = e + 1
    else
      l = l + 1
    end
  end
  return shown, raw
end

--- What each closed fold renders as.  This is text.rs's `get_foldtext`
--- reached through the eval layer, which is the only way to see it
--- without a screen.
local function foldtexts(raw)
  local out = {}
  for _, range in ipairs(raw) do
    local ok, t = pcall(vim.fn.foldtextresult, range[1])
    out[#out + 1] = ok and esc(scrub(t)) or esc(errtext(t))
  end
  return out
end

-- {long name, the spelling the report uses}.  The short names are
-- vim's own, not a prefix mangle: `foldlevel`/`foldminlines` share a
-- four-letter prefix and a mangle would have collapsed them.
local WOPTS = {
  { 'foldmethod', 'fdm' },
  { 'foldlevel', 'fdl' },
  { 'foldenable', 'fen' },
  { 'foldminlines', 'fml' },
  { 'foldnestmax', 'fdn' },
  { 'foldignore', 'fdi' },
  { 'foldmarker', 'fmr' },
  { 'foldcolumn', 'fdc' },
  { 'foldexpr', 'fde' },
  { 'foldtext', 'fdt' },
  { 'diff', 'diff' },
}

local function optsnap()
  local shown, raw = {}, {}
  for _, entry in ipairs(WOPTS) do
    local name, short = entry[1], entry[2]
    -- `ok and v or '?'` is WRONG here and was: 'diff' and 'foldenable'
    -- are booleans, and a `false` value took the `or` arm, so every
    -- case in the report claimed `diff=?`.  Spell the branch out.
    local ok, v = pcall(function()
      return vim.wo[name]
    end)
    if not ok then
      v = '?'
    end
    raw[name] = v
    if type(v) == 'boolean' then
      v = v and 1 or 0
    end
    shown[#shown + 1] = string.format('%s=%s', short, esc(tostring(v)))
  end
  return table.concat(shown, ' '), raw
end

local SEEN = {}

--- The whole observable fold state.  A fixed number of lines per case,
--- always in this order, so the report reads top to bottom.
local function shot(label)
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1

  local lines = vim.api.nvim_buf_get_lines(BUF, 0, -1, false)
  local changed = #lines ~= #CUR_LINES
  if not changed then
    for i = 1, #lines do
      if lines[i] ~= CUR_LINES[i] then
        changed = true
        break
      end
    end
  end

  local lvstr, lvraw = levels()
  local shownr, rawr = closedranges()
  local texts = foldtexts(rawr)
  local optstr, optraw = optsnap()
  local pos = vim.api.nvim_win_get_cursor(0)

  -- `B =` rather than the text: only the marker method rewrites the
  -- buffer, and repeating an unchanged fixture on every one of ~300
  -- cases would bury the lines that ARE the answer.
  emit(label, 'B', changed and esc(table.concat(lines, '\n')) or '=')
  emit(label, 'L', lvstr == '' and '-' or lvstr)
  emit(label, 'C', #shownr == 0 and '-' or table.concat(shownr, ' '))
  if #texts > 0 then
    emit(label, 'T', table.concat(texts, ' | '))
  end
  emit(label, 'S', string.format('c=%d,%d %s', pos[1], pos[2], optstr))

  struct(label, {
    b = changed and lines or nil,
    l = lvraw,
    c = rawr,
    t = texts,
    p = pos,
    o = optraw,
  })
end

--- Walk 'foldlevel' 0..`upto` and record the closed ranges at each
--- step.  This is the TREE probe: two different nestings can present
--- the same closed set at one level and diverge at the next.
---
--- DESTRUCTIVE -- setting 'foldlevel' opens and closes folds for real
--- (`newFoldLevelWin`), so it runs after shot(), never before, and the
--- case ends here.
local function levelscan(label, upto)
  local rows, raw = {}, {}
  for lv = 0, upto do
    local err = quiet('silent! setlocal foldlevel=' .. lv)
    local shownr, rawr = closedranges()
    rows[#rows + 1] = string.format('%d:%s', lv, #shownr == 0 and '-' or table.concat(shownr, ','))
    raw[#raw + 1] = { lv = lv, c = rawr, e = err }
  end
  emit(label, 'Z', table.concat(rows, ' '))
  struct(label .. ' Z', raw)
end

--- One case: reset, feed, report.  `after` is a Vimscript expression
--- asked once the keys have run, rendered through Vimscript's own
--- string() so this sweep never asks evalsweep's question by accident.
local function case(label, lines, pos, keys, opts, opt2)
  opt2 = opt2 or {}
  reset(lines, pos, opts)
  if opt2.pre then
    local err = quiet(opt2.pre)
    if err then
      emit(label, '!pre', esc(err))
    end
  end
  if keys and keys ~= '' then
    local err = feed(keys)
    if err then
      emit(label, '!', esc(err))
    end
  end
  if opt2.ex then
    local err = quiet(opt2.ex)
    if err then
      emit(label, '!ex', esc(err))
    end
  end
  shot(label)
  if opt2.after then
    local ok, res = pcall(vim.fn.eval, 'string(' .. opt2.after .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    struct(label .. ' A', ok and scrub(res) or { err = errtext(res) })
  end
  if opt2.scan then
    levelscan(label, opt2.scan)
  end
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- Fixtures.  Named, so a case says which shape it is asking about and
-- two sections asking the same question use the same bytes.
-- ---------------------------------------------------------------------

local F = {}

-- Sixteen flat, distinguishable lines.  Every manual-fold case uses
-- this one, so a range in the report reads directly as line numbers.
F.flat = {}
for i = 1, 16 do
  F.flat[i] = string.format('l%02d', i)
end

-- Indentation: 0 1 2 2 1 0 ... with a blank line inside a block and a
-- 'foldignore' candidate (`#`) at the start of one.
F.indent = {
  'top-a',
  '\tone-a',
  '\t\ttwo-a',
  '\t\ttwo-b',
  '',
  '\t\ttwo-c',
  '\tone-b',
  'top-b',
  '#comment',
  '\tone-c',
  '\t\ttwo-d',
  '\t\t\tthree-a',
  '\t\t\t\tfour-a',
  '\tone-d',
  'top-c',
  '',
}

-- The marker method's fixture.  Balanced, nested, and with one stray
-- close marker that has no opener -- foldmethod=marker's handling of an
-- unmatched `}}}` is a real branch in marker.rs.
F.marker = {
  'head {{{',
  'body 1',
  'inner {{{',
  'body 2',
  'inner end }}}',
  'body 3',
  'head end }}}',
  'loose }}}',
  'plain',
  'numbered {{{2',
  'deep',
  'close2 }}}2',
  'tail',
}

-- Numbered markers only, which set an ABSOLUTE level rather than
-- nesting one deeper.
F.marknum = {
  'a',
  'b {{{1',
  'c',
  'd {{{3',
  'e',
  'f }}}3',
  'g',
  'h }}}1',
  'i',
}

-- A C-ish body for the syntax method and for `[z`/`]z` over braces.
F.code = {
  'int main(void) {',
  '  int i = 0;',
  '  if (i) {',
  '    i++;',
  '  }',
  '  return i;',
  '}',
  '',
  'void other(void) {',
  '  return;',
  '}',
}

-- Two nearly-equal texts for the diff method.
F.diffa = { 'same 1', 'same 2', 'same 3', 'AAA', 'same 4', 'same 5', 'same 6', 'same 7' }
F.diffb = { 'same 1', 'same 2', 'same 3', 'BBB', 'same 4', 'same 5', 'same 6', 'same 7' }

-- ---------------------------------------------------------------------
-- s01 -- creation
-- ---------------------------------------------------------------------
section('s01-create', function()
  local motions = {
    { 'zfj', 'zf-j' },
    { 'zfk', 'zf-k' },
    { 'zf3j', 'zf-3j' },
    { 'zfG', 'zf-G' },
    { 'zfgg', 'zf-gg' },
    { 'zf}', 'zf-brace' },
    { 'zfap', 'zf-ap' },
    { 'zfip', 'zf-ip' },
    { 'zf/l09<CR>', 'zf-search' },
    { "zf'a", 'zf-mark' },
    { '3zF', 'zF-3' },
    { 'zF', 'zF-1' },
    { '5zF', 'zF-5' },
  }
  for _, m in ipairs(motions) do
    case('s01-' .. m[2], F.flat, { 5, 0 }, m[1], '', { pre = "silent! normal! 8Gma5G" })
  end
  -- From the last line, from the first line, and past the end.
  case('s01-last-zfj', F.flat, { 16, 0 }, 'zfj')
  case('s01-first-zfk', F.flat, { 1, 0 }, 'zfk')
  case('s01-zfG-from-1', F.flat, { 1, 0 }, 'zfG')
  -- Visual.
  case('s01-vis-V', F.flat, { 3, 0 }, 'V4jzf')
  case('s01-vis-v', F.flat, { 3, 1 }, 'v4j2lzf')
  case('s01-vis-blk', F.flat, { 3, 1 }, '<C-v>4jlzf')
  case('s01-vis-gv', F.flat, { 3, 0 }, 'V4j<Esc>gvzf')
  -- Ex.
  case('s01-ex-range', F.flat, { 1, 0 }, '', '', { ex = '3,7fold' })
  case('s01-ex-percent', F.flat, { 1, 0 }, '', '', { ex = '%fold' })
  case('s01-ex-single', F.flat, { 1, 0 }, '', '', { ex = '5fold' })
  case('s01-ex-backwards', F.flat, { 1, 0 }, '', '', { ex = 'silent! 7,3fold' })
  case('s01-ex-fo-alias', F.flat, { 1, 0 }, '', '', { ex = '2,4fo' })
  -- One line, and the empty buffer.
  case('s01-one-line', F.flat, { 4, 0 }, 'zfl')
  case('s01-empty-buf', { '' }, { 1, 0 }, 'zfj')
  case('s01-single-line-buf', { 'only' }, { 1, 0 }, 'zfj')
  -- E350: zf under a computed method.
  for _, fdm in ipairs({ 'indent', 'expr', 'syntax', 'diff' }) do
    case('s01-E350-' .. fdm, F.indent, { 2, 0 }, 'zfj', 'foldmethod=' .. fdm)
  end
  -- zf under marker writes markers into the TEXT.
  case('s01-marker-zfj', F.flat, { 5, 0 }, 'zfj', 'foldmethod=marker')
  case('s01-marker-zf3j', F.flat, { 5, 0 }, 'zf3j', 'foldmethod=marker')
  case('s01-marker-custom', F.flat, { 5, 0 }, 'zfj', 'foldmethod=marker foldmarker=/*,*/')
  -- Creating a fold moves the cursor to its start and closes it.
  case('s01-cursor-after', F.flat, { 9, 2 }, 'zf3k')
end)

-- ---------------------------------------------------------------------
-- s02 -- nesting, splitting, merging
-- ---------------------------------------------------------------------
section('s02-nest', function()
  case('s02-outer-then-inner', F.flat, { 2, 0 }, 'zf8jzo5Gzfj', '', { scan = 3 })
  case('s02-inner-then-outer', F.flat, { 5, 0 }, 'zfj2Gzf8j', '', { scan = 3 })
  case('s02-three-deep', F.flat, { 2, 0 }, 'zf12jzo4Gzf8jzo6Gzf4j', '', { scan = 4 })
  case('s02-two-siblings', F.flat, { 2, 0 }, 'zf2jzo6Gzf2j', '', { scan = 2 })
  case('s02-adjacent', F.flat, { 2, 0 }, 'zf2jzo5Gzf2j', '', { scan = 2 })
  -- A new fold that starts inside an existing one and ends outside it.
  case('s02-overlap-tail', F.flat, { 3, 0 }, 'zf5jzo6Gzf5j', '', { scan = 3 })
  -- ... and one that starts outside and ends inside.
  case('s02-overlap-head', F.flat, { 6, 0 }, 'zf5jzo3Gzf5j', '', { scan = 3 })
  -- Identical range twice.
  case('s02-same-range-twice', F.flat, { 3, 0 }, 'zf5jzozf5j', '', { scan = 3 })
  -- A fold wholly containing two siblings.
  case('s02-wrap-two', F.flat, { 3, 0 }, 'zf2jzo8Gzf2jzo2Gzf10j', '', { scan = 3 })
  -- Nesting to the 'foldnestmax' bound under the manual method (which
  -- does NOT clamp -- that is the point of the case).
  case('s02-manual-past-nestmax', F.flat, { 1, 0 }, 'zf15jzo2Gzf12jzo3Gzf10jzo4Gzf8j', 'foldnestmax=2', { scan = 5 })
end)

-- ---------------------------------------------------------------------
-- s03 -- deletion
-- ---------------------------------------------------------------------
section('s03-delete', function()
  local build = 'zf12jzo4Gzf6jzo6Gzf2jzo'
  case('s03-zd-inner', F.flat, { 1, 0 }, build .. '7Gzd', '', { scan = 3 })
  case('s03-zd-middle', F.flat, { 1, 0 }, build .. '5Gzd', '', { scan = 3 })
  case('s03-zd-outer', F.flat, { 1, 0 }, build .. '2Gzd', '', { scan = 3 })
  case('s03-zd-count', F.flat, { 1, 0 }, build .. '7G2zd', '', { scan = 3 })
  case('s03-zD-inner', F.flat, { 1, 0 }, build .. '7GzD', '', { scan = 3 })
  case('s03-zD-outer', F.flat, { 1, 0 }, build .. '2GzD', '', { scan = 3 })
  case('s03-zE', F.flat, { 1, 0 }, build .. '7GzE', '', { scan = 3 })
  case('s03-zd-nofold', F.flat, { 1, 0 }, '15Gzd')
  case('s03-zD-nofold', F.flat, { 1, 0 }, '15GzD')
  case('s03-zd-on-closed', F.flat, { 1, 0 }, 'zf5j5Gzd', '', { scan = 2 })
  case('s03-zE-empty', F.flat, { 1, 0 }, 'zE')
  -- Under marker, zd/zE remove the markers from the TEXT.
  case('s03-marker-zd', F.marker, { 3, 0 }, 'zd', 'foldmethod=marker')
  case('s03-marker-zD', F.marker, { 1, 0 }, 'zD', 'foldmethod=marker')
  case('s03-marker-zE', F.marker, { 1, 0 }, 'zE', 'foldmethod=marker')
  case('s03-marker-zE-numbered', F.marknum, { 1, 0 }, 'zE', 'foldmethod=marker')
  -- E350 under a computed method.
  for _, fdm in ipairs({ 'indent', 'expr', 'syntax' }) do
    case('s03-E350-' .. fdm, F.indent, { 3, 0 }, 'zd', 'foldmethod=' .. fdm)
    case('s03-E352-' .. fdm, F.indent, { 3, 0 }, 'zE', 'foldmethod=' .. fdm)
  end
  -- :foldd... is not a delete; the Ex deleters are `zd`'s Visual form.
  case('s03-vis-zd', F.flat, { 1, 0 }, build .. '5GV3jzd', '', { scan = 3 })
  case('s03-vis-zD', F.flat, { 1, 0 }, build .. '5GV3jzD', '', { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s04 -- open and close
-- ---------------------------------------------------------------------
section('s04-openclose', function()
  -- Build three nested folds and leave them all closed.
  local build = 'zf12jzo4Gzf6jzo6Gzf2jzo1GzM'
  local keys = {
    'zo',
    'zO',
    'zc',
    'zC',
    'za',
    'zA',
    'zR',
    'zM',
    'zr',
    'zm',
    '2zr',
    '2zm',
    'zv',
    'zx',
    'zX',
    'zn',
    'zN',
    'zi',
    'zizi',
  }
  for _, k in ipairs(keys) do
    -- From the outermost line, and from a line inside the innermost.
    case('s04-out-' .. k, F.flat, { 1, 0 }, build .. '1G' .. k, '', { scan = 3 })
    case('s04-in-' .. k, F.flat, { 1, 0 }, build .. '7G' .. k, '', { scan = 3 })
  end
  -- Counted zo/zc/za walk the nesting.
  for _, k in ipairs({ '2zo', '3zo', '2zc', '2za', '2zA' }) do
    case('s04-cnt-' .. k, F.flat, { 1, 0 }, build .. '7G' .. k, '', { scan = 3 })
  end
  -- zo on an already-open fold, zc on an already-closed one, and both
  -- on a line with no fold at all (E490).
  case('s04-zo-open', F.flat, { 1, 0 }, 'zf5jzo5Gzo')
  case('s04-zc-closed', F.flat, { 1, 0 }, 'zf5j1Gzc')
  case('s04-zo-nofold', F.flat, { 15, 0 }, 'zo')
  case('s04-zc-nofold', F.flat, { 15, 0 }, 'zc')
  case('s04-za-nofold', F.flat, { 15, 0 }, 'za')
  -- 'foldopen' -- a motion into a closed fold opens it, or does not.
  case('s04-fdo-search', F.flat, { 1, 0 }, 'zf5jzo3Gzf2jzc1G/l04<CR>', '', { scan = 2 })
  case('s04-fdo-empty', F.flat, { 1, 0 }, 'zf5jzo3Gzf2jzc1G/l04<CR>', '', { pre = 'set foldopen=', scan = 2 })
  case('s04-fdo-all', F.flat, { 1, 0 }, 'zf5jzo3Gzf2jzc1G/l04<CR>', '', { pre = 'set foldopen=all', scan = 2 })
  -- 'foldclose'=all re-closes on leaving.
  case('s04-fdc-all', F.flat, { 1, 0 }, 'zf5jzo3Gzf2jzo1Gj', '', { pre = 'set foldclose=all', scan = 2 })
  -- zv from deep inside.
  case('s04-zv-deep', F.flat, { 1, 0 }, build .. 'zM7Gzv', '', { scan = 3 })
  -- 'foldenable' off hides every closed fold from foldclosed().
  case('s04-fen-off', F.flat, { 1, 0 }, 'zf5j', 'nofoldenable', { scan = 2 })
  -- zn/zN round trip preserves which folds were closed.
  case('s04-zn-zN', F.flat, { 1, 0 }, build .. 'znzN', '', { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s05 -- fold motions
-- ---------------------------------------------------------------------
section('s05-motion', function()
  local build = 'zf12jzo4Gzf6jzo6Gzf2jzo'
  for _, k in ipairs({ 'zj', 'zk', '[z', ']z', '2zj', '2zk', '2[z', '2]z' }) do
    for _, at in ipairs({ 1, 3, 5, 7, 9, 13, 16 }) do
      case(string.format('s05-%s-at%02d', k:gsub('%[', 'so'):gsub('%]', 'sc'), at), F.flat, { 1, 0 }, build .. at .. 'G' .. k)
    end
  end
  -- All folds closed: the motions still see the tree.
  case('s05-zj-closed', F.flat, { 1, 0 }, build .. 'zM1Gzj')
  case('s05-zk-closed', F.flat, { 1, 0 }, build .. 'zM16Gzk')
  -- No folds at all: every motion is a no-move.
  for _, k in ipairs({ 'zj', 'zk', '[z', ']z' }) do
    case('s05-nofold-' .. k:gsub('%[', 'so'):gsub('%]', 'sc'), F.flat, { 8, 0 }, k)
  end
  -- Under the marker method, over the nested fixture.
  for _, k in ipairs({ 'zj', 'zk', '[z', ']z' }) do
    case('s05-mk-' .. k:gsub('%[', 'so'):gsub('%]', 'sc'), F.marker, { 4, 0 }, k, 'foldmethod=marker')
  end
end)

-- ---------------------------------------------------------------------
-- s06 -- foldmethod=indent
-- ---------------------------------------------------------------------
section('s06-indent', function()
  case('s06-base', F.indent, { 1, 0 }, '', 'foldmethod=indent', { scan = 4 })
  for _, sw in ipairs({ 1, 2, 4, 8, 16 }) do
    case('s06-sw' .. sw, F.indent, { 1, 0 }, '', 'foldmethod=indent shiftwidth=' .. sw, { scan = 4 })
  end
  -- 'tabstop' decides what a tab is worth, and therefore the level.
  for _, ts in ipairs({ 2, 4, 8 }) do
    case('s06-ts' .. ts, F.indent, { 1, 0 }, '', 'foldmethod=indent shiftwidth=2 tabstop=' .. ts, { scan = 4 })
  end
  -- Spaces rather than tabs.
  local spaces = {}
  for i, l in ipairs(F.indent) do
    spaces[i] = (l:gsub('\t', '    '))
  end
  case('s06-spaces', spaces, { 1, 0 }, '', 'foldmethod=indent shiftwidth=4', { scan = 4 })
  -- A blank line takes the level of the line AFTER it, unless it is at
  -- the end.  Both arms are in the fixture; these ask them directly.
  case('s06-blank-mid', { 'a', '\tb', '', '\tc', 'd' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8', { scan = 2 })
  case('s06-blank-end', { 'a', '\tb', '' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8', { scan = 2 })
  case('s06-blank-start', { '', '\ta', 'b' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8', { scan = 2 })
  case('s06-all-blank', { '', '', '' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8', { scan = 2 })
  -- Re-indenting recomputes the tree (foldUpdate through the change
  -- path, not through a fresh :setlocal).
  case('s06-reindent', F.indent, { 1, 0 }, '3G>>', 'foldmethod=indent shiftwidth=8', { scan = 4 })
  case('s06-unindent', F.indent, { 1, 0 }, '3G<<', 'foldmethod=indent shiftwidth=8', { scan = 4 })
  case('s06-insert-deep', F.indent, { 1, 0 }, '4Go\t\t\t\tdeeper<Esc>', 'foldmethod=indent shiftwidth=8', { scan = 5 })
  case('s06-delete-head', F.indent, { 1, 0 }, '2Gdd', 'foldmethod=indent shiftwidth=8', { scan = 4 })
end)

-- ---------------------------------------------------------------------
-- s07 -- foldmethod=marker
-- ---------------------------------------------------------------------
section('s07-marker', function()
  case('s07-base', F.marker, { 1, 0 }, '', 'foldmethod=marker', { scan = 3 })
  case('s07-numbered', F.marknum, { 1, 0 }, '', 'foldmethod=marker', { scan = 4 })
  case('s07-custom-fmr', {
    'a /* ',
    'b',
    'c */',
    'd',
  }, { 1, 0 }, '', 'foldmethod=marker foldmarker=/*,*/', { scan = 2 })
  -- An unmatched opener runs to the end of the buffer; an unmatched
  -- closer ends whatever is open, or nothing.
  case('s07-open-unmatched', { 'a {{{', 'b', 'c' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 2 })
  case('s07-close-unmatched', { 'a', 'b }}}', 'c' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 2 })
  case('s07-close-first', { 'a }}}', 'b {{{', 'c' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 2 })
  -- Both markers on ONE line.
  case('s07-same-line', { 'a', 'b {{{ inner }}}', 'c' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 2 })
  -- Two openers on one line.
  case('s07-two-open', { 'a', 'b {{{ {{{', 'c', 'd' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 3 })
  -- A numbered close that is deeper than anything open.
  case('s07-close-too-deep', { 'a {{{1', 'b', 'c }}}5', 'd' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 3 })
  -- A numbered open that jumps two levels.
  case('s07-open-jump', { 'a {{{1', 'b {{{3', 'c', 'd }}}', 'e' }, { 1, 0 }, '', 'foldmethod=marker', { scan = 4 })
  -- Editing markers in place must recompute.
  case('s07-add-marker', F.marker, { 9, 0 }, 'A {{{<Esc>', 'foldmethod=marker', { scan = 3 })
  case('s07-remove-marker', F.marker, { 1, 0 }, '1G$xxxx', 'foldmethod=marker', { scan = 3 })
  case('s07-dd-opener', F.marker, { 1, 0 }, '1Gdd', 'foldmethod=marker', { scan = 3 })
  case('s07-dd-closer', F.marker, { 7, 0 }, 'dd', 'foldmethod=marker', { scan = 3 })
  -- 'foldnestmax' does NOT clamp the marker method (upstream behaviour
  -- differs from indent/syntax); the case is here to pin which.
  case('s07-nestmax1', F.marker, { 1, 0 }, '', 'foldmethod=marker foldnestmax=1', { scan = 3 })
  -- 'foldminlines' over markers.
  case('s07-minlines3', F.marker, { 1, 0 }, '', 'foldmethod=marker foldminlines=3', { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s08 -- foldmethod=expr
-- ---------------------------------------------------------------------
section('s08-expr', function()
  local exprs = {
    { '0', 'zero' },
    { '1', 'one' },
    { '2', 'two' },
    { "v:lnum", 'lnum' },
    { "v:lnum % 3", 'mod3' },
    { "v:lnum <= 4 ? 1 : 0", 'first4' },
    { "getline(v:lnum) =~ '^\\\\t' ? 1 : 0", 'tabbed' },
    { "'='", 'eq' },
    { "'-1'", 'undefined' },
    { "v:lnum == 3 ? '>1' : (v:lnum == 6 ? '<1' : '=')", 'gtlt' },
    { "v:lnum == 2 ? 'a1' : (v:lnum == 5 ? 's1' : '=')", 'as' },
    { "v:lnum == 1 ? '>1' : (v:lnum == 4 ? '>2' : '=')", 'nested-gt' },
    { "v:lnum == 1 ? 1 : (v:lnum == 5 ? 0 : 1)", 'split' },
  }
  for _, e in ipairs(exprs) do
    case('s08-' .. e[2], F.flat, { 1, 0 }, '', 'foldmethod=expr foldexpr=' .. e[1]:gsub(' ', '\\ '), { scan = 4 })
  end
  -- A Lua expr, and a Vimscript function -- two different entry points
  -- into the same `foldexpr` evaluation.
  case('s08-luaexpr', F.flat, { 1, 0 }, '', '', {
    pre = 'lua _G.FoldE = function() return vim.v.lnum <= 5 and 1 or 0 end',
    ex = "setlocal foldmethod=expr foldexpr=v:lua._G.FoldE()",
    scan = 3,
  })
  case('s08-funcexpr', F.flat, { 1, 0 }, '', '', {
    pre = 'function! FoldF()\nreturn v:lnum >= 8 ? 2 : 1\nendfunction',
    ex = 'setlocal foldmethod=expr foldexpr=FoldF()',
    scan = 4,
  })
  -- An expr that errors, and one that returns a garbage string.
  case('s08-err', F.flat, { 1, 0 }, '', '', {
    ex = 'silent! setlocal foldmethod=expr foldexpr=nosuchfunc()',
    scan = 2,
  })
  case('s08-garbage', F.flat, { 1, 0 }, '', "foldmethod=expr foldexpr='zzz'", { scan = 2 })
  -- 'foldnestmax' clamps the expr method.
  case('s08-nestmax2', F.flat, { 1, 0 }, '', 'foldmethod=expr foldexpr=v:lnum foldnestmax=2', { scan = 4 })
  -- An edit re-evaluates the expr for the changed window only.
  case('s08-edit-recompute', F.flat, { 1, 0 }, '5GO\tinserted<Esc>', "foldmethod=expr foldexpr=v:lnum<=6?1:0", { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s09 -- foldmethod=syntax
-- ---------------------------------------------------------------------
section('s09-syntax', function()
  local syn = table.concat({
    'syntax clear',
    'syntax region cBlock start="{" end="}" transparent fold',
  }, '\n')
  case('s09-base', F.code, { 1, 0 }, '', '', { pre = syn, ex = 'setlocal foldmethod=syntax', scan = 3 })
  case('s09-nestmax1', F.code, { 1, 0 }, '', '', {
    pre = syn,
    ex = 'setlocal foldmethod=syntax foldnestmax=1',
    scan = 3,
  })
  case('s09-minlines3', F.code, { 1, 0 }, '', '', {
    pre = syn,
    ex = 'setlocal foldmethod=syntax foldminlines=3',
    scan = 3,
  })
  -- `fold` on a MATCH rather than a region, and a region with
  -- `keepend`.
  case('s09-match', F.code, { 1, 0 }, '', '', {
    pre = 'syntax clear\nsyntax match cRet "return.*;" fold',
    ex = 'setlocal foldmethod=syntax',
    scan = 3,
  })
  -- An edit inside a region recomputes.
  case('s09-edit', F.code, { 1, 0 }, '4Goi--;<Esc>', '', {
    pre = syn,
    ex = 'setlocal foldmethod=syntax',
    scan = 3,
  })
  case('s09-delete-brace', F.code, { 5, 0 }, 'dd', '', {
    pre = syn,
    ex = 'setlocal foldmethod=syntax',
    scan = 3,
  })
end)

-- ---------------------------------------------------------------------
-- s10 -- foldmethod=diff
-- ---------------------------------------------------------------------
section('s10-diff', function()
  -- The diff method needs two windows over two buffers, so this section
  -- builds its own world and tears it down.  Anything it leaves behind
  -- ('diff' is window-local and sets six other options) would decide
  -- every case below it.
  local function difframe(label, opts, edit)
    normalise()
    quiet('silent! setlocal foldmethod=manual foldenable | silent! normal! zE')
    quiet('silent! only!')
    quiet(DEFAULTS)
    vim.api.nvim_buf_set_lines(BUF, 0, -1, false, F.diffa)
    CUR_LINES = vim.deepcopy(F.diffa)
    quiet('silent! set diffopt=' .. (opts or 'internal,filler,closeoff'))
    quiet('silent! diffthis')
    quiet('silent! vnew')
    local other = vim.api.nvim_get_current_buf()
    quiet('silent! setlocal buftype=nofile bufhidden=wipe noswapfile')
    vim.api.nvim_buf_set_lines(other, 0, -1, false, F.diffb)
    quiet('silent! diffthis')
    if edit then
      local err = feed(edit)
      if err then
        emit(label, '!', esc(err))
      end
    end
    quiet('silent! wincmd p')
    shot(label)
    levelscan(label, 2)
    quiet('silent! diffoff!')
    quiet('silent! only!')
    quiet('silent! setlocal diff& foldmethod=manual foldenable')
  end
  difframe('s10-base')
  difframe('s10-ctx0', 'internal,filler,context:0')
  difframe('s10-ctx3', 'internal,filler,context:3')
  difframe('s10-ctx99', 'internal,filler,context:99')
  difframe('s10-nofiller', 'internal,closeoff')
  -- An edit in the OTHER window re-diffs and must re-fold this one.
  difframe('s10-edit-other', 'internal,filler,context:0', '4GceCCC<Esc>')
  difframe('s10-add-line', 'internal,filler,context:0', 'GoZZZ<Esc>')
end)

-- ---------------------------------------------------------------------
-- s11 -- 'foldminlines'
-- ---------------------------------------------------------------------
section('s11-minlines', function()
  for _, n in ipairs({ 0, 1, 2, 3, 5, 20 }) do
    -- A two-line manual fold, a three-line one and a six-line one, so
    -- each 'foldminlines' value sits on a different side of each.
    case('s11-manual-' .. n, F.flat, { 1, 0 }, '2Gzfj5Gzf2j9Gzf5j', 'foldminlines=' .. n, { scan = 2 })
    case('s11-indent-' .. n, F.indent, { 1, 0 }, '', 'foldmethod=indent foldminlines=' .. n, { scan = 4 })
  end
  -- A one-line fold: below every 'foldminlines' but 0.
  case('s11-oneline-0', F.flat, { 4, 0 }, 'zfl', 'foldminlines=0', { scan = 2 })
  case('s11-oneline-1', F.flat, { 4, 0 }, 'zfl', 'foldminlines=1', { scan = 2 })
  -- 'foldminlines' is not a bar to zc: the fold exists, it just does
  -- not DISPLAY closed.  Both questions are asked here.
  case('s11-zc-under-min', F.flat, { 2, 0 }, 'zfjzo2Gzc', 'foldminlines=5', { after = "[foldlevel(2), foldclosed(2), foldclosedend(2)]", scan = 2 })
end)

-- ---------------------------------------------------------------------
-- s12 -- 'foldnestmax'
-- ---------------------------------------------------------------------
section('s12-nestmax', function()
  local deep = {}
  for i = 1, 10 do
    deep[i] = string.rep('\t', i - 1) .. 'd' .. i
  end
  for i = 11, 20 do
    deep[i] = string.rep('\t', 20 - i) .. 'u' .. i
  end
  for _, n in ipairs({ 0, 1, 2, 3, 5, 20 }) do
    case('s12-indent-' .. n, deep, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8 foldnestmax=' .. n, { scan = 6 })
    case('s12-expr-' .. n, F.flat, { 1, 0 }, '', 'foldmethod=expr foldexpr=v:lnum foldnestmax=' .. n, { scan = 6 })
  end
  case('s12-syntax-1', F.code, { 1, 0 }, '', '', {
    pre = 'syntax clear\nsyntax region cBlock start="{" end="}" transparent fold',
    ex = 'setlocal foldmethod=syntax foldnestmax=1',
    scan = 3,
  })
  case('s12-marker-1', F.marker, { 1, 0 }, '', 'foldmethod=marker foldnestmax=1', { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s13 -- 'foldignore'
-- ---------------------------------------------------------------------
section('s13-ignore', function()
  local body = {
    'top',
    '\ta',
    '#hash',
    '\tb',
    '//slash',
    '\tc',
    '\t#indented-hash',
    '\t\td',
    'bottom',
  }
  for _, fdi in ipairs({ '', '#', '#/', '/', 'abc' }) do
    case(
      's13-fdi-' .. (fdi == '' and 'empty' or fdi:gsub('/', 'sl')),
      body,
      { 1, 0 },
      '',
      'foldmethod=indent shiftwidth=8 foldignore=' .. (fdi == '' and '' or fdi),
      { scan = 3 }
    )
  end
  -- 'foldignore' applies to the INDENT method only; expr must ignore it.
  case('s13-expr-unaffected', body, { 1, 0 }, '', 'foldmethod=expr foldexpr=indent(v:lnum)/8 foldignore=#', { scan = 3 })
  -- An ignored line at the very start, and at the very end.
  case('s13-first', { '#x', '\ta', 'b' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8 foldignore=#', { scan = 2 })
  case('s13-last', { 'a', '\tb', '#x' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8 foldignore=#', { scan = 2 })
  case('s13-all-ignored', { '#a', '#b', '#c' }, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8 foldignore=#', { scan = 2 })
end)

-- ---------------------------------------------------------------------
-- s14 -- 'foldenable', 'foldlevel', 'foldlevelstart'
-- ---------------------------------------------------------------------
section('s14-levels', function()
  local build = 'zf12jzo4Gzf6jzo6Gzf2jzo'
  for _, fdl in ipairs({ 0, 1, 2, 3, 9 }) do
    case('s14-fdl' .. fdl, F.flat, { 1, 0 }, build, 'foldlevel=' .. fdl)
  end
  case('s14-fen-off', F.flat, { 1, 0 }, build, 'nofoldenable')
  case('s14-fen-toggle', F.flat, { 1, 0 }, build .. 'zMznzN')
  -- 'foldlevelstart' is applied when a window starts editing a buffer,
  -- so it needs a real :edit.  The scratch buffer is unnamed, so the
  -- round trip is :enew + :buffer back.
  for _, fls in ipairs({ -1, 0, 1, 99 }) do
    case('s14-fls' .. (fls < 0 and 'm1' or tostring(fls)), F.indent, { 1, 0 }, '', '', {
      pre = 'setlocal foldmethod=indent shiftwidth=8',
      ex = 'set foldlevelstart=' .. fls .. ' | let s:b = bufnr("%") | silent! enew | silent! exe "buffer" s:b | set foldlevelstart=-1',
      after = '[&l:foldlevel, &l:foldmethod]',
    })
  end
  -- What a SPLIT window inherits: fold state is window-local, and a
  -- split copies it.
  case('s14-split-inherit', F.flat, { 1, 0 }, build .. 'zM', '', {
    ex = 'silent! split | silent! wincmd p',
    after = '[winnr("$"), &l:foldlevel, foldclosed(1)]',
  })
  quiet('silent! only!')
end)

-- ---------------------------------------------------------------------
-- s15 -- the adjust path
-- ---------------------------------------------------------------------
section('s15-adjust', function()
  -- Three nested folds: 2-14 outer, 5-11 middle, 7-9 inner, all left
  -- open so the edits below act on the TREE and not on a closed range.
  local build = '2Gzf12jzo5Gzf6jzo7Gzf2jzo1G'
  local edits = {
    { 'dd-above', '1Gdd' },
    { 'dd-at-start', '2Gdd' },
    { 'dd-inside', '8Gdd' },
    { 'dd-at-end', '14Gdd' },
    { 'dd-below', '16Gdd' },
    { 'dd-3-across-start', '1G3dd' },
    { 'dd-3-across-end', '13G3dd' },
    { 'dd-whole-inner', '7G3dd' },
    { 'dd-whole-outer', '2G13dd' },
    { 'o-above', '1GoNEW<Esc>' },
    { 'o-inside', '8GoNEW<Esc>' },
    { 'O-at-start', '2GONEW<Esc>' },
    { 'o-at-end', '14GoNEW<Esc>' },
    { 'p-lines', '3Gyy8Gp' },
    { 'J-inside', '8GJ' },
    { 'J-across-end', '9G3J' },
    { 'sub-split', "8G:s/l08/x\\rY/<CR>" },
  }
  for _, e in ipairs(edits) do
    case('s15-' .. e[1], F.flat, { 1, 0 }, build .. e[2], '', { scan = 3 })
    case('s15-' .. e[1] .. '-undo', F.flat, { 1, 0 }, build .. e[2] .. 'u', '', { scan = 3 })
  end
  -- :move is the one edit that both deletes and inserts, and the only
  -- producer of a backwards adjustment.
  local moves = {
    { 'm-into', '1move 8' },
    { 'm-out', '8move 1' },
    { 'm-inner-out', '8move 15' },
    { 'm-range-in', '1,2move 9' },
    { 'm-range-out', '7,9move 0' },
    { 'm-within', '6move 10' },
    { 'm-to-end', '3move $' },
  }
  for _, m in ipairs(moves) do
    case('s15-' .. m[1], F.flat, { 1, 0 }, build, '', { ex = m[2], scan = 3 })
    case('s15-' .. m[1] .. '-undo', F.flat, { 1, 0 }, build, '', { ex = m[2] .. ' | undo', scan = 3 })
  end
  -- :copy, :delete and :put with ranges.
  case('s15-copy', F.flat, { 1, 0 }, build, '', { ex = '2,3copy 9', scan = 3 })
  case('s15-del-range', F.flat, { 1, 0 }, build, '', { ex = '6,10delete', scan = 3 })
  case('s15-put', F.flat, { 1, 0 }, build, '', { ex = '3yank | 8put', scan = 3 })
  case('s15-put-0', F.flat, { 1, 0 }, build, '', { ex = '3yank | 0put', scan = 3 })
  case('s15-global-del', F.flat, { 1, 0 }, build, '', { ex = 'silent! g/l0[369]/delete', scan = 3 })
  case('s15-global-move', F.flat, { 1, 0 }, build, '', { ex = 'silent! g/l1[012]/move 0', scan = 3 })
  -- The same edits under the computed methods, where the tree is
  -- rebuilt rather than shifted.
  case('s15-indent-dd', F.indent, { 1, 0 }, '3Gdd', 'foldmethod=indent shiftwidth=8', { scan = 4 })
  case('s15-indent-move', F.indent, { 1, 0 }, '', 'foldmethod=indent shiftwidth=8', { ex = '3move 12', scan = 4 })
  case('s15-marker-dd', F.marker, { 1, 0 }, '4Gdd', 'foldmethod=marker', { scan = 3 })
  case('s15-marker-move', F.marker, { 1, 0 }, '', 'foldmethod=marker', { ex = '3move 8', scan = 3 })
  case('s15-marker-yank-put', F.marker, { 1, 0 }, '', 'foldmethod=marker', { ex = '1,3yank | 10put', scan = 3 })
  -- An edit made while the fold is CLOSED.
  case('s15-closed-dd', F.flat, { 1, 0 }, '2Gzf12jzo5Gzf6j1G5Gdd', '', { scan = 3 })
  case('s15-closed-p', F.flat, { 1, 0 }, '2Gzf12jzo5Gzf6j1Gyy5Gp', '', { scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s16 -- fold text
-- ---------------------------------------------------------------------
section('s16-text', function()
  case('s16-default', F.flat, { 1, 0 }, '2Gzf5j', '', { after = 'foldtextresult(2)' })
  case('s16-nested', F.flat, { 1, 0 }, '2Gzf12jzo5Gzf6j1GzM', '', { after = '[foldtextresult(2), foldtextresult(5)]' })
  case('s16-one-line', F.flat, { 4, 0 }, 'zfl', 'foldminlines=0', { after = 'foldtextresult(4)' })
  case('s16-empty-lines', { '', '', '', '' }, { 1, 0 }, 'zf3j', '', { after = 'foldtextresult(1)' })
  case('s16-leading-ws', { '\t\tindented', '\tless', 'x' }, { 1, 0 }, 'zf2j', '', { after = 'foldtextresult(1)' })
  case('s16-marker-stripped', F.marker, { 1, 0 }, '', 'foldmethod=marker', { after = 'foldtextresult(1)' })
  case('s16-comment-stripped', {
    '/* head {{{ */',
    'body',
    '/* }}} */',
  }, { 1, 0 }, '', 'foldmethod=marker commentstring=/*\\ %s\\ */', { after = 'foldtextresult(1)' })
  case('s16-custom-fdt', F.flat, { 1, 0 }, '2Gzf5j', '', {
    pre = 'function! MyFT()\nreturn "FT " . v:foldstart . "-" . v:foldend . " lv" . v:foldlevel . " [" . v:folddashes . "]"\nendfunction',
    ex = 'setlocal foldtext=MyFT()',
    after = 'foldtextresult(2)',
  })
  case('s16-fdt-empty', F.flat, { 1, 0 }, '2Gzf5j', '', { ex = 'setlocal foldtext=', after = 'foldtextresult(2)' })
  case('s16-fdt-error', F.flat, { 1, 0 }, '2Gzf5j', '', {
    ex = 'silent! setlocal foldtext=nosuchfn()',
    after = 'foldtextresult(2)',
  })
  case('s16-fdt-list', F.flat, { 1, 0 }, '2Gzf5j', '', {
    pre = 'lua _G.FT2 = function() return { { "chunk", "Comment" } } end',
    ex = 'setlocal foldtext=v:lua._G.FT2()',
    after = 'foldtextresult(2)',
  })
  -- 'fillchars' fold: the pad glyph is part of the answer.  The label
  -- has to be injective over the VALUE -- stripping punctuation
  -- collapsed `fold:-` and `fold:.` onto one label and the second case
  -- silently stopped gating.
  for _, fc in ipairs({ { 'fold:-', 'dash' }, { 'fold:.', 'dot' }, { 'fold:\\u00b7', 'middot' } }) do
    case('s16-fillchar-' .. fc[2], F.flat, { 1, 0 }, '2Gzf5j', '', {
      ex = 'set fillchars=' .. fc[1],
      after = 'foldtextresult(2)',
    })
  end
  quiet('set fillchars=vert:|,fold:-,foldopen:-,foldsep:|,foldclose:+')
  -- foldtextresult() on a line that is not a fold start, and out of
  -- range.
  case('s16-notafold', F.flat, { 1, 0 }, '2Gzf5j', '', { after = '[foldtextresult(1), foldtextresult(9)]' })
end)

-- ---------------------------------------------------------------------
-- s17 -- the eval builtins' edges
-- ---------------------------------------------------------------------
section('s17-builtins', function()
  local probes = {
    { 'foldlevel(0)', 'fl0' },
    { 'foldlevel(1)', 'fl1' },
    { 'foldlevel(99)', 'fl99' },
    { 'foldlevel(-1)', 'flm1' },
    { 'foldclosed(0)', 'fc0' },
    { 'foldclosed(99)', 'fc99' },
    { 'foldclosed(-1)', 'fcm1' },
    { 'foldclosedend(0)', 'fce0' },
    { 'foldclosedend(99)', 'fce99' },
    { 'foldtextresult(0)', 'ftr0' },
    { 'foldtextresult(99)', 'ftr99' },
    { "foldlevel('.')", 'fldot' },
    { "foldlevel('$')", 'fldollar' },
    { "foldclosed('.')", 'fcdot' },
    { "foldtextresult('.')", 'ftrdot' },
    { "foldlevel('x')", 'flbad' },
    { "foldclosed([])", 'fclist' },
    { "foldlevel('.', 'extra')", 'flarity' },
  }
  for _, p in ipairs(probes) do
    case('s17-' .. p[2], F.flat, { 3, 0 }, '2Gzf5j3G', '', { after = p[1] })
  end
  -- foldtext() itself, called outside a fold context.
  case('s17-foldtext-bare', F.flat, { 1, 0 }, '', '', { after = 'foldtext()' })
  -- The whole per-line answer as one list, so the struct artifact
  -- carries the tree in one record.
  case('s17-sweep-list', F.flat, { 1, 0 }, '2Gzf12jzo5Gzf6j1GzM', '', {
    after = 'map(range(1, line("$")), \'[v:val, foldlevel(v:val), foldclosed(v:val), foldclosedend(v:val)]\')',
  })
end)

-- ---------------------------------------------------------------------
-- s18 -- the Ex fold commands
-- ---------------------------------------------------------------------
section('s18-ex', function()
  local build = '2Gzf12jzo5Gzf6jzo7Gzf2jzo1GzM'
  local cmds = {
    { 'foldopen', 'foldopen' },
    { '5foldopen', 'foldopen-5' },
    { 'foldopen!', 'foldopen-bang' },
    { '5foldopen!', 'foldopen-bang-5' },
    { 'foldclose', 'foldclose' },
    { 'foldclose!', 'foldclose-bang' },
    { '2,14foldopen', 'foldopen-range' },
    { '2,14foldopen!', 'foldopen-range-bang' },
    { '%foldopen!', 'foldopen-pct' },
    { '%foldclose!', 'foldclose-pct' },
    { 'silent! 15foldopen', 'foldopen-nofold' },
    { 'silent! foldclose', 'foldclose-line1' },
  }
  for _, c in ipairs(cmds) do
    case('s18-' .. c[2], F.flat, { 1, 0 }, build, '', { ex = c[1], scan = 3 })
  end
  -- :folddoopen / :folddoclosed run a command on lines NOT in a closed
  -- fold / in one.
  case('s18-folddoopen', F.flat, { 1, 0 }, build, '', { ex = 'silent! folddoopen s/^/O/', scan = 3 })
  case('s18-folddoclosed', F.flat, { 1, 0 }, build, '', { ex = 'silent! folddoclosed s/^/C/', scan = 3 })
  case('s18-folddoopen-range', F.flat, { 1, 0 }, build, '', { ex = 'silent! 1,8folddoopen s/^/O/', scan = 3 })
  case('s18-folddoclosed-del', F.flat, { 1, 0 }, build, '', { ex = 'silent! folddoclosed delete', scan = 3 })
end)

-- ---------------------------------------------------------------------
-- s19 -- uncaptured, for the .stderr artifact
-- ---------------------------------------------------------------------
section('s19-messages', function()
  -- `report` at 0 so "N lines folded" and friends are printed, and no
  -- `silent!`, so every E-code reaches the prompt -- which in a
  -- headless process is stderr, the third artifact.
  --
  -- The per-probe marker goes to STDERR, not stdout.  nvim's messages
  -- carry no trailing newline in a headless process, so without a
  -- separator the whole section arrives as one
  -- `E350...E350...E490...` line and a diff can only say "the blob
  -- moved".  With it, .stderr is one probe per paragraph and names the
  -- command whose message changed.
  local function mark(kind, what)
    io.stdout:flush()
    io.stderr:write('\n-- s19 ', kind, ' ', what, ': ')
    io.stderr:flush()
  end
  reset(F.flat, { 1, 0 }, '')
  quiet('set report=0')
  local seq = {
    'zfj',
    'zE',
    'zd',
    'zD',
    'zo',
    'zc',
    'za',
    'zj',
    'zk',
    '[z',
    ']z',
  }
  for _, k in ipairs(seq) do
    mark('keys', k)
    feed(k)
  end
  for _, fdm in ipairs({ 'indent', 'expr', 'syntax', 'diff', 'marker', 'manual' }) do
    mark('fdm', fdm)
    quiet('setlocal foldmethod=' .. fdm)
    feed('3Gzf5j')
    feed('zd')
    feed('zE')
  end
  local excmds = {
    '3,7fold',
    '7,3fold',
    'foldopen',
    'foldclose',
    '99foldopen',
    'folddoopen echo "x"',
    'folddoclosed echo "y"',
    'setlocal foldmethod=nosuch',
    'setlocal foldmarker=onlyone',
    'setlocal foldmarker=,',
    'setlocal foldnestmax=-1',
    'setlocal foldminlines=-1',
    'setlocal foldcolumn=13',
    'echo foldlevel(0)',
    'echo foldclosed(-4)',
    'echo foldtextresult(-1)',
  }
  for _, c in ipairs(excmds) do
    mark('ex', c)
    local ok, err = pcall(vim.api.nvim_exec2, c, { output = false })
    if not ok then
      io.stderr:write('EX-ERR ', errtext(err))
    end
  end
  io.stderr:write('\n')
  quiet('set report=9999')
  reset(F.flat, { 1, 0 }, '')
  emit('s19-done', 'S', 'ok')
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 fileencoding= isprint=@,161-255 ambiwidth=single')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set undolevels=1000 undofile& hidden')
quiet('language C')
quiet('syntax off')

-- One scratch buffer for the whole run: a fresh buffer per case would
-- hand out thousands of monotonic handles and make the artifact a
-- function of where in the run a case sits.
quiet('enew!')
BUF = vim.api.nvim_get_current_buf()
quiet('setlocal buftype=nofile bufhidden=hide noswapfile')
quiet(DEFAULTS)

-- Which 'foldtext' and 'fillchars' the build ships with is real signal
-- but it is NOT this oracle's question -- DEFAULTS pins both.  Record
-- the shipped values once, here, so a change in them is still visible
-- as one line rather than as three hundred.
emit('== defaults ==')
do
  quiet('silent! new')
  emit(
    'defaults',
    'D',
    string.format(
      'fdm=%s fdt=%s fdl=%d fen=%s fml=%d fdn=%d fdi=%s fmr=%s fdls=%d fdo=%s fdc=%s fcs=%s',
      vim.wo.foldmethod,
      esc(vim.wo.foldtext),
      vim.wo.foldlevel,
      tostring(vim.wo.foldenable),
      vim.wo.foldminlines,
      vim.wo.foldnestmax,
      esc(vim.wo.foldignore),
      esc(vim.wo.foldmarker),
      vim.o.foldlevelstart,
      esc(vim.o.foldopen),
      esc(vim.o.foldclose),
      esc(vim.o.fillchars)
    )
  )
  quiet('silent! close!')
end

for _, entry in ipairs(SECTIONS) do
  if not only or entry.name:match(only) then
    local started = trace and vim.uv.hrtime() or 0
    if trace then
      io.stderr:write('== ', entry.name, '\n')
    end
    emit('')
    emit('== ' .. entry.name .. ' ==')
    local ok, err = pcall(entry.fn)
    if not ok then
      emit('== ' .. entry.name .. ' ABORTED:', esc(errtext(err)))
    end
    if trace then
      -- A duration is never reproducible, so it goes to the trace only:
      -- FOLDSWEEP_TRACE writes into the .stderr artifact and must be
      -- off for a baseline.
      io.stderr:write(string.format('   %s %.1fs\n', entry.name, (vim.uv.hrtime() - started) / 1e9))
    end
  end
end

emit('')
emit('== done ==')
if structfd then
  structfd:close()
end
