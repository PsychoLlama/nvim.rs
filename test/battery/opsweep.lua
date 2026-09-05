-- Driver for the text-operator differential sweep; see
-- opsweep.sh.
--
-- Covers the B15 text-ops core -- ops.rs, register.rs, change.rs,
-- textobject.rs, edit.rs -- which no existing differential reaches at
-- all.  Every case is a *key sequence* run through nvim_feedkeys with
-- the 'x' flag against a fixed buffer, and the answer is the whole
-- observable state afterwards: the buffer lines, the cursor (with
-- curswant and coladd), the four operator/visual marks, how many
-- b:changedtick ticks the sequence spent, the mode it left behind, and
-- every register that is not empty.
--
--   s01 every operator x every motion, and the same for the operators
--       whose pending path differs (gq/gw/=/zf/g@).
--   s02 every operator x every text object, from two cursor positions.
--   s03 counts: before the operator, after it, and both (`3d2w`).
--   s04 forced motion (`dv`, `dV`, `d<C-v>`) and 'virtualedit'.
--   s05 blockwise: `$` (curswant MAXCOL), tabs, multibyte, virtualedit,
--       and the block-only commands I A c r x d y < > o O.
--   s06 Visual mode proper: v/V/C-v, gv, o/O, `'<`/`'>` afterwards,
--       'selection', and the operators applied to a selection.
--   s07 p P gp gP ]p [p zp zP ]P [P, with counts, from charwise,
--       linewise and blockwise registers, at line ends and on an empty
--       buffer -- do_put's whole surface.
--   s08 the named and numbered registers: a-z, A-Z append, the "0 yank
--       register, the "1-"9 shift, "- small delete, and what the
--       unnamed register points at.
--   s09 the special registers: "_ "= ". "% ": "/ "#, `"+`/`"*` behind a
--       FIXTURE clipboard provider (never the host clipboard), plus
--       setreg/getreg/getreginfo in every shape.
--   s10 `:registers` / `:display` text.
--   s11 :d :y :pu :pu! :normal :m :co with ranges and register args.
--   s12 `<` and `>` under 'shiftwidth'/'tabstop'/'softtabstop'/
--       'expandtab'/'shiftround', with counts and in Visual mode.
--   s13 gu gU g~ g? and `~` under 'tildeop', over multibyte.
--   s14 J and gJ: counts, 'joinspaces', 'formatoptions' j, comment
--       leaders, trailing whitespace, Visual join.
--   s15 CTRL-A / CTRL-X / g CTRL-A / g CTRL-X over every base and every
--       'nrformats' spelling, including Visual and blockwise.
--   s16 Insert-mode semantics: CTRL-O CTRL-R CTRL-V CTRL-W CTRL-U
--       CTRL-T CTRL-D CTRL-E CTRL-Y CTRL-A CTRL-@, digraphs, the
--       'backspace' matrix, 'revins', 'paste', and abbreviations.
--   s17 Replace and virtual-Replace mode, `r`/`gr`, and the one-key
--       change commands x X s S C D Y.
--   s18 what `'[ '] '< '>`, `` `. ``, `` `^ `` and b:changedtick answer
--       after each kind of change, and across undo/redo.
--   s19 TextYankPost's v:event, TextChanged, and 'clipboard=unnamed'.
--   s20 the same operators run *uncaptured*, so the .stderr artifact
--       carries the real message path -- "N fewer lines", "block of N
--       lines yanked", E353/E354/E749 -- which is the only view of
--       msg_* for this subsystem.
--   s21 a child `--embed` nvim for 'updatetime'/CursorHold/CursorHoldI:
--       feedkeys(..., 'x') runs the typeahead to completion and can
--       never let a timer expire, so the timeout arms are unreachable
--       from this process.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time or path outside the work directory.
--
-- b:changedtick is a monotonic counter, so the *absolute* value would
-- make every case a function of every case above it.  What is recorded
-- is the delta across the case, which is the actual question ("how many
-- changes did this sequence make").  Buffer/window handles are recorded
-- nowhere for the same reason.
--
-- Registers are global state.  Every case starts by *clearing* all 38
-- of them (setreg(r, {}) -- not by deleting anything, and not by
-- hoping), so "which registers are non-empty" is an answer about this
-- case alone.
--
-- OPSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.
-- OPSWEEP_TRACE=1 mirrors each section name to stderr, which is the
-- only way to see where a wedged run stopped.

local work = assert(os.getenv('OPS_WORK'), 'OPS_WORK unset')
local structpath = assert(os.getenv('OPS_STRUCT'), 'OPS_STRUCT unset')

local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('OPSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('OPSWEEP_TRACE') == '1'

-- Unbuffered: nvim's own messages go to stderr, but a Lua error would
-- otherwise lose the tail of the report.
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

--- Escape to one printable line, so a byte difference shows in the diff
--- and a report line stays a report line.  Multibyte fixtures make this
--- load-bearing: the whole point of the utf8 cases is which *bytes*
--- moved.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- A case label has to be *injective* over the key sequence it names:
--- two cases sharing a label read as one case in the diff and the
--- second one silently stops gating.  A blanket punctuation-to-`_`
--- mangle is not injective -- it collapsed `>>` and `<<` onto the same
--- label, and `g-` onto `g+`.  Every punctuation character gets its own
--- token here, and the duplicate check in shot() is the standing proof
--- that it worked.
local TAGMAP = {
  ['<'] = 'lt',
  ['>'] = 'gt',
  ['-'] = 'm',
  ['+'] = 'p',
  ['='] = 'eq',
  ['$'] = 'S',
  ['^'] = 'H',
  ['~'] = 'T',
  ['!'] = 'B',
  ['@'] = 'A',
  ['#'] = 'N',
  ['%'] = 'P',
  ['&'] = 'D',
  ['*'] = 'X',
  ['('] = 'ro',
  [')'] = 'rc',
  ['['] = 'so',
  [']'] = 'sc',
  ['{'] = 'co',
  ['}'] = 'cc',
  ['"'] = 'Q',
  ["'"] = 'q',
  ['`'] = 'K',
  ['/'] = 'sl',
  ['\\'] = 'bk',
  ['|'] = 'V',
  [';'] = 'sm',
  [','] = 'cm',
  ['.'] = 'dt',
  [':'] = 'cl',
  ['?'] = 'qm',
  [' '] = 'sp',
  ['_'] = 'u',
}

local function tag(keys)
  return (tostring(keys):gsub('%W', function(c)
    return TAGMAP[c] or string.format('x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  Verbatim from varsweep.lua / evalsweep.lua: the
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

-- Every register the sweep clears and scans.  `_` is excluded because
-- it is a black hole by definition and answers the same thing forever;
-- the special ones (. : % # / =) are asked by name in s09, where their
-- values are the question, rather than on every case, where they are
-- noise that only re-baselines the artifact.
local REGS = { '"', '-' }
for c = 0, 9 do
  REGS[#REGS + 1] = tostring(c)
end
for c = string.byte('a'), string.byte('z') do
  REGS[#REGS + 1] = string.char(c)
end

local SPECIAL_REGS = { '_', '=', '.', '%', '#', '/' }

local function clearregs()
  for _, r in ipairs(REGS) do
    pcall(vim.fn.setreg, r, {})
  end
  -- `/` and `=` are writable and would otherwise carry the last search
  -- pattern and the last expression across every case below the one
  -- that set them.
  pcall(vim.fn.setreg, '/', '')
  pcall(vim.fn.setreg, '=', '')
end

--- All the options any case touches, at their documented defaults, so
--- a case that forgets to restore one cannot poison its neighbours.
--- (`cpo` is spelled out because `cpo&vim` and `cpo&` differ and the
--- difference decides several put and join answers.)
local DEFAULTS = table.concat({
  'set shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab noshiftround',
  'set textwidth=0 virtualedit= selection=inclusive nostartofline',
  "set nrformats=bin,hex nojoinspaces backspace=indent,eol,start",
  'set norevins nopaste noautoindent nosmartindent nolisp nocindent',
  'set comments&vim commentstring&vim formatoptions=tcq formatexpr= formatprg=',
  'set indentexpr= equalprg= matchpairs&vim iskeyword&vim quoteescape&vim',
  'set clipboard= foldmethod=manual foldenable operatorfunc= tildeop&',
  'set whichwrap= wrapscan ignorecase& smartcase& magic& selectmode=',
  'set report=9999 shortmess=filnxtToOFS nomore noshowmode scrolloff=0 sidescrolloff=0',
  'set list& listchars&vim wrap& linebreak& conceallevel=0 undolevels=1000',
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

--- Back to Normal mode from wherever the previous case left off, with
--- any pending operator, Visual selection, Insert session or command
--- line abandoned.  Done at the *start* of a reset rather than the end
--- of a case, so the mode a case leaves behind is still reported
--- honestly.
local function normalise()
  pcall(vim.api.nvim_feedkeys, vim.api.nvim_replace_termcodes('<C-\\><C-N>', true, true, true), 'ntx', false)
end

local base_tick = 0

--- Put the world in a known state.  `lines` is the buffer, `pos` the
--- cursor as {row, col} (1-based row, 0-based byte column), `opts` a
--- `:set` argument string applied on top of DEFAULTS.
local function reset(lines, pos, opts)
  normalise()
  quiet(DEFAULTS)
  -- Folds are window state and outlive a buffer rewrite, so a `zf` case
  -- would otherwise decide what every case below it sees.  `normal!`
  -- swallows the rest of the line, so this cannot join the DEFAULTS bar
  -- chain.
  quiet('silent! normal! zE')
  vim.api.nvim_buf_set_lines(BUF, 0, -1, false, lines)
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  -- `.` is read-only, so setreg cannot clear it; it is *seeded*
  -- instead, with one empty insert, so that every case starts from the
  -- same value.  Without this a case added to s07 rewrites the answer
  -- of a case in s16, and an addition that was supposed to remove no
  -- baseline lines removes a few hundred.  Seeded under DEFAULTS,
  -- before `opts`: a 'formatoptions' or 'revins' the case asked for
  -- must not reach the seed.
  --
  -- `:` is not seedable the same way -- the only thing that sets it is
  -- a real command line, which echoes itself to stderr once per case --
  -- so it is left out of the scanned set entirely and asked for by name
  -- in s09 instead, where `":p` is the question rather than the noise.
  feed('i<Esc>')
  if opts and opts ~= '' then
    quiet('set ' .. opts)
  end
  for _, m in ipairs({ '<', '>', '[', ']' }) do
    pcall(vim.api.nvim_buf_del_mark, BUF, m)
  end
  clearregs()
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  base_tick = vim.api.nvim_buf_get_var(BUF, 'changedtick')
end

local function reginfo(name)
  local ok, info = pcall(vim.fn.getreginfo, name)
  if not ok then
    return { err = errtext(info) }
  end
  if type(info) ~= 'table' or next(info) == nil then
    return nil
  end
  return info
end

--- Render one register as `name=type:contents`, contents joined with a
--- literal newline that esc() then makes visible.  `points_to` is part
--- of the answer for `"`: it is the whole of what the unnamed register
--- *is*.
local function regstr(name, info)
  local parts = { name, '=', info.regtype or '?', ':' }
  parts[#parts + 1] = table.concat(info.regcontents or {}, '\n')
  if info.points_to then
    parts[#parts + 1] = '@' .. info.points_to
  end
  return esc(table.concat(parts))
end

--- Every non-empty register, in a fixed order.  `extra` adds the
--- special registers, which only s09/s10 want.
local function regsnap(extra)
  local names = REGS
  if extra then
    names = vim.list_extend(vim.list_extend({}, REGS), SPECIAL_REGS)
  end
  local shown, raw = {}, {}
  for _, name in ipairs(names) do
    local info = reginfo(name)
    if info then
      shown[#shown + 1] = regstr(name, info)
      raw[name] = { t = info.regtype, c = info.regcontents, p = info.points_to }
    end
  end
  return shown, raw
end

--- The whole observable state after a key sequence.  Three report lines
--- per case, always in this order, so the report reads top to bottom.
local SEEN = {}

local function shot(label, extra)
  -- A label collision is invisible in a 17,000-line report and turns
  -- two cases into one; say so in the artifact rather than in a comment.
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1
  local lines = vim.api.nvim_buf_get_lines(BUF, 0, -1, false)
  local view = vim.fn.winsaveview()
  local marks = {}
  local mparts = {}
  for _, m in ipairs({ '[', ']', '<', '>' }) do
    local ok, mk = pcall(vim.api.nvim_buf_get_mark, BUF, m)
    mk = ok and mk or { -1, -1 }
    -- An *unset* mark answers line 0 and whatever column it last held --
    -- nvim_buf_del_mark clears the line and leaves the column behind, so
    -- `'>` was still reporting MAXCOL from the last linewise Visual case
    -- several sections above.  A mark with no line has no column.
    if mk[1] == 0 then
      mk = { 0, 0 }
    end
    marks[m] = mk
    mparts[#mparts + 1] = string.format("'%s=%d,%d", m, mk[1], mk[2])
  end
  local tick = vim.api.nvim_buf_get_var(BUF, 'changedtick') - base_tick
  local mode = vim.api.nvim_get_mode()
  local shown, raw = regsnap(extra)
  emit(label, 'B', esc(table.concat(lines, '\n')))
  emit(
    label,
    'S',
    string.format(
      'c=%d,%d cw=%s ca=%d %s t=%d m=%s',
      view.lnum,
      view.col,
      view.curswant == 2147483647 and 'MAX' or tostring(view.curswant),
      view.coladd or 0,
      table.concat(mparts, ' '),
      tick,
      mode.mode
    )
  )
  emit(label, 'R', #shown == 0 and '-' or table.concat(shown, ' '))
  struct(label, {
    b = lines,
    c = { view.lnum, view.col, view.curswant, view.coladd or 0 },
    m = marks,
    t = tick,
    md = mode.mode,
    r = raw,
  })
end

--- One case: reset, feed, report.  `after` is a Vimscript expression
--- asked once the sequence has run, for the answers that are not buffer
--- state -- what an 'operatorfunc' saw, where a fold ended up, what
--- v:event carried.  It is rendered through Vimscript's own string()
--- rather than a Lua converter, so this sweep never asks evalsweep's
--- question by accident.
local function case(label, lines, pos, keys, opts, extra, after)
  reset(lines, pos, opts)
  local err = feed(keys)
  if err then
    emit(label, '!', esc(err))
  end
  shot(label, extra)
  if after then
    local ok, res = pcall(vim.fn.eval, 'string(' .. after .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    struct(label .. ' A', ok and scrub(res) or { err = errtext(res) })
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

-- Plain prose: word boundaries, punctuation, a blank line, paragraphs.
F.prose = {
  'alpha beta gamma',
  'delta.epsilon zeta',
  '',
  'eta theta  iota',
  'kappa-lambda mu',
}

-- Brackets, quotes and a tag, for the text objects.
F.nest = {
  'one (two [three {four} five] six) seven',
  'say "quoted text" and \'single\' and `back`',
  '<tag attr="v">inner <b>bold</b> text</tag>',
  'fn(a, b) { body; }',
}

-- Ragged right edge, for blockwise `$`, and a tab.
F.ragged = {
  'short',
  'a much longer line here',
  '',
  '\ttabbed line',
  'mid',
}

-- Multibyte: CJK wide, a combining mark, an emoji, latin-1 range.
F.utf = {
  'ab\xc3\xa9cd',
  '\xe4\xb8\xad\xe6\x96\x87 wide',
  'e\xcc\x81 combining',
  '\xf0\x9f\x98\x80 emoji tail',
}

-- Leading whitespace of several shapes, for < > and the indent ops.
F.indent = {
  '\tone tab',
  '        eight spaces',
  '    four spaces',
  'none',
  '\t    tab then spaces',
}

-- Numbers in every base the increment commands know.
F.nums = {
  'x 7 y -3 z',
  '0x1f 0X0A 0b101 0B11',
  '007 0779 -0x0',
  'a9z ver1.9.9 100%',
  '18446744073709551615 -9223372036854775808',
  -- The `-` here has a *non-blank* two characters back, which is the only
  -- shape 'nrformats' "blank" decides anything about; every minus above
  -- has a space there and the option is invisible to them.
  'ab-7 x=-5 12-34 -0x2f',
}

-- Comment leaders, for J and 'formatoptions' j.
F.comment = {
  '// first half',
  '// second half',
  ' * a block line',
  ' * another',
  'plain	trailing tab',
}

-- ---------------------------------------------------------------------
-- s01 -- every operator x every motion
-- ---------------------------------------------------------------------

-- {tag, keys, post, after}.  `post` finishes a sequence that would
-- otherwise leave Insert mode pending; `after` is the extra question an
-- operator answers somewhere other than the buffer.
local OPS = {
  { 'd', 'd' },
  -- The text typed after `c` is `zz` and not something readable on
  -- purpose: when the *motion* fails (`c%` on prose, `c;` with nothing
  -- to repeat) the operator is abandoned and whatever follows runs as
  -- Normal-mode commands.  `NEW<Esc>` did exactly that -- `N` searched
  -- backwards, which is E35 once the sweep started clearing `@/`, and
  -- `E`/`W` then moved the cursor, so a failed case reported a
  -- *different* failure than the one it was asking about.  `zz` is a
  -- scroll: it touches no line, no cursor column and no register.
  { 'c', 'c', 'zz<Esc>' },
  { 'y', 'y' },
  { 'lt', '<' },
  { 'gt', '>' },
  { 'gu', 'gu' },
  { 'gU', 'gU' },
  { 'gtilde', 'g~' },
  { 'grot13', 'g?' },
  { 'gq', 'gq' },
  { 'gw', 'gw' },
  { 'eq', '=' },
  { 'zf', 'zf', nil, "foldlevel(1).'/'.foldclosed(1).'/'.foldclosedend(1).'/'.foldlevel(4)" },
  { 'gat', 'g@', nil, 'get(g:, "opfunc", "UNSET")' },
}

-- Motions that need no prior state.  Every one of them is spelled in
-- full: a motion that depends on a previous f/t/search is a motion
-- whose answer is a function of the case above it.
local MOTIONS = {
  { 'h', 'h' },
  { 'l', 'l' },
  { 'j', 'j' },
  { 'k', 'k' },
  { 'w', 'w' },
  { 'W', 'W' },
  { 'e', 'e' },
  { 'E', 'E' },
  { 'b', 'b' },
  { 'B', 'B' },
  { 'ge', 'ge' },
  { 'gE', 'gE' },
  { 'zero', '0' },
  { 'caret', '^' },
  { 'dollar', '$' },
  { 'gunder', 'g_' },
  { 'gzero', 'g0' },
  { 'gdollar', 'g$' },
  { 'gm', 'gm' },
  { 'gM', 'gM' },
  { 'fa', 'fa' },
  { 'ta', 'ta' },
  { 'Fa', 'Fa' },
  { 'Ta', 'Ta' },
  { 'pct', '%' },
  { 'sentf', ')' },
  { 'sentb', '(' },
  { 'paraf', '}' },
  { 'parab', '{' },
  { 'sectf', ']]' },
  { 'sectb', '[[' },
  { 'sectfb', '][' },
  { 'sectbf', '[]' },
  { 'unopen', '[(' },
  { 'unbrace', '[{' },
  { 'unclose', '])' },
  { 'unbclose', ']}' },
  { 'G', 'G' },
  { 'gg', 'gg' },
  { 'H', 'H' },
  { 'M', 'M' },
  { 'L', 'L' },
  { 'plus', '+' },
  { 'minus', '-' },
  { 'under', '_' },
  { 'bar', '|' },
  { 'go', 'go' },
  { 'srchf', '/eta<CR>' },
  { 'srchb', '?alpha<CR>' },
  { 'srchoffe', '/eta/e<CR>' },
  { 'srchoffl', '/eta/+1<CR>' },
  { 'srchoffs', '/eta/s-2<CR>' },
}

-- Motions whose meaning is "again": the setup keys run *before* the
-- operator, in the same sequence, so nothing leaks in from the case
-- above.
local PRE_MOTIONS = {
  { 'semi', 'fa', ';' },
  { 'comma', '$Fa', ',' },
  { 'nsearch', '/eta<CR>gg', 'n' },
  { 'Nsearch', '/eta<CR>gg', 'N' },
  { 'star', '', '*' },
  { 'hash', '', '#' },
  { 'markbt', 'majj', '`a' },
  { 'markln', 'majj', "'a" },
  { 'ctxbt', 'Gk', '``' },
  { 'ctxln', 'Gk', "''" },
}

local function opfunc_setup()
  quiet([[
    function! OpFuncRec(type) abort
      let g:opfunc = a:type . " [" . line("'[") . "," . col("'[")
            \ . " ]" . line("']") . "," . col("']")
    endfunction
  ]])
end

section('s01-op-motion', function()
  for _, op in ipairs(OPS) do
    local opts = op[2] == 'g@' and 'operatorfunc=OpFuncRec' or nil
    for _, mo in ipairs(MOTIONS) do
      case(
        string.format('om %s %s', op[1], mo[1]),
        F.prose,
        { 2, 6 },
        op[2] .. mo[2] .. (op[3] or ''),
        opts,
        false,
        op[4]
      )
    end
    for _, mo in ipairs(PRE_MOTIONS) do
      case(
        string.format('op %s %s', op[1], mo[1]),
        F.prose,
        { 2, 6 },
        mo[2] .. op[2] .. mo[3] .. (op[3] or ''),
        opts,
        false,
        op[4]
      )
    end
  end
end)

section('s01-op-motion-nest', function()
  -- The bracket motions answer "nothing here" on prose, which is an
  -- answer but a weak one.  The same set on a fixture that actually
  -- nests is where %/[(/])/[{/]} have a decision to make.
  local BRACKET = {
    { 'pct', '%' },
    { 'unopen', '[(' },
    { 'unclose', '])' },
    { 'unbrace', '[{' },
    { 'unbclose', ']}' },
    { 'pct2', '2%' },
    { 'unopen2', '2[(' },
  }
  for _, op in ipairs({ { 'd', 'd' }, { 'y', 'y' }, { 'c', 'c', 'zz<Esc>' }, { 'gU', 'gU' } }) do
    for _, mo in ipairs(BRACKET) do
      for _, pos in ipairs({ { 1, 12 }, { 1, 20 }, { 4, 3 }, { 3, 18 } }) do
        case(
          string.format('ob %s %s %d.%d', op[1], mo[1], pos[1], pos[2]),
          F.nest,
          pos,
          op[2] .. mo[2] .. (op[3] or ''),
          nil
        )
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s02 -- every operator x every text object
-- ---------------------------------------------------------------------

local TEXTOBJS = {
  'iw',
  'aw',
  'iW',
  'aW',
  'is',
  'as',
  'ip',
  'ap',
  'i(',
  'a(',
  'ib',
  'ab',
  'i{',
  'a{',
  'iB',
  'aB',
  'i[',
  'a[',
  'i<',
  'a<',
  'i"',
  'a"',
  "i'",
  "a'",
  'i`',
  'a`',
  'it',
  'at',
}

section('s02-textobject', function()
  local POS = { { 1, 12 }, { 2, 10 }, { 3, 20 }, { 4, 4 } }
  for _, op in ipairs({ { 'd', 'd' }, { 'y', 'y' }, { 'c', 'c', 'zz<Esc>' } }) do
    for _, obj in ipairs(TEXTOBJS) do
      for _, pos in ipairs(POS) do
        local otag = tag(obj)
        case(
          string.format('to %s %s %d.%d', op[1], otag, pos[1], pos[2]),
          F.nest,
          pos,
          op[2] .. obj .. (op[3] or ''),
          nil
        )
      end
    end
  end
  -- The prose objects (sentence, paragraph, WORD) need prose: on
  -- F.nest, `ip` is the whole four-line block and every case answers
  -- the same thing, which reads as coverage and is not.
  for _, op in ipairs({ { 'd', 'd' }, { 'y', 'y' }, { 'gU', 'gU' }, { 'lt', '<' } }) do
    for _, obj in ipairs({ 'iw', 'aw', 'iW', 'aW', 'is', 'as', 'ip', 'ap' }) do
      for _, pos in ipairs({ { 1, 0 }, { 2, 6 }, { 3, 0 }, { 4, 10 }, { 5, 5 } }) do
        case(
          string.format('tp %s %s %d.%d', op[1], obj, pos[1], pos[2]),
          F.prose,
          pos,
          op[2] .. obj .. (op[3] or ''),
          nil
        )
      end
    end
  end
  -- Quote objects on a line where the cursor sits *outside* every pair,
  -- which is current_quote's other half, and with 'quoteescape'.
  for _, keys in ipairs({
    'di"',
    'da"',
    "di'",
    "da'",
    'di`',
    'da`',
    'yi"',
    '2di"',
    '2da"',
    'v2i"d',
  }) do
    for _, pos in ipairs({ { 2, 0 }, { 2, 3 }, { 2, 4 }, { 2, 17 }, { 2, 25 } }) do
      case(
        string.format('tq %s %d.%d', tag(keys), pos[1], pos[2]),
        F.nest,
        pos,
        keys
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s03 -- counts
-- ---------------------------------------------------------------------

section('s03-count', function()
  local COUNTED = {
    '2dw',
    'd2w',
    '2d2w',
    '3dw',
    'd3w',
    '2dd',
    'd2d',
    '3dj',
    'd3j',
    '2yy',
    'y2y',
    '2cw',
    'c2w',
    '2>>',
    '>2>',
    '3<<',
    '2guu',
    'gu2u',
    '2g~~',
    '2J',
    '3J',
    '2gJ',
    '0d5l',
    '5x',
    '5X',
    '3s',
    '2rX',
    '3~',
    'd0',
    'd^',
    '2d$',
    '10|',
    'd10|',
    '3G',
    'd3G',
    '2dip',
    'd2ip',
    '2daw',
    'd2aw',
    '2di(',
    '99dw',
    '0d99l',
  }
  for _, keys in ipairs(COUNTED) do
    local post = keys:find('c') and '<Esc>' or ''
    case('ct ' .. tag(keys), F.prose, { 2, 6 }, keys .. post)
    case('cn ' .. tag(keys), F.nest, { 1, 12 }, keys .. post)
  end
end)

-- ---------------------------------------------------------------------
-- s04 -- forced motion and 'virtualedit'
-- ---------------------------------------------------------------------

section('s04-forced', function()
  -- dv / dV / d<C-v> flip a motion's own inclusive/exclusive/linewise
  -- character, which is `do_pending_operator`'s first decision.
  for _, force in ipairs({ { 'plain', '' }, { 'v', 'v' }, { 'V', 'V' }, { 'blk', '<C-v>' } }) do
    for _, mo in ipairs({ 'j', 'w', 'e', '$', '}', 'G', 'gg', 'ip', 'aw', '/eta<CR>' }) do
      for _, op in ipairs({ { 'd', 'd' }, { 'y', 'y' }, { 'gU', 'gU' } }) do
        case(
          string.format('fm %s %s %s', op[1], force[1], tag(mo)),
          F.prose,
          { 2, 6 },
          op[2] .. force[2] .. mo
        )
      end
    end
  end
  for _, ve in ipairs({ '', 'all', 'block', 'insert', 'onemore', 'all,onemore' }) do
    for _, keys in ipairs({
      '$dl',
      '$x',
      '10|x',
      '30|D',
      '$a!<Esc>',
      '30|i!<Esc>',
      '$y$P',
      '<C-v>jj30|d',
      '<C-v>jj$d',
      '30|<C-v>jjI!<Esc>',
      '$<C-v>jjA!<Esc>',
      'j$hd0',
      '30|yl',
    }) do
      case(
        string.format('ve %s %s', ve == '' and 'none' or ve:gsub(',', '+'), tag(keys)),
        F.ragged,
        { 2, 4 },
        keys,
        'virtualedit=' .. ve
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s05 -- blockwise
-- ---------------------------------------------------------------------

section('s05-block', function()
  local BLOCK = {
    '<C-v>jjd',
    '<C-v>jj$d',
    '<C-v>jj$y',
    '<C-v>2j3ly',
    '<C-v>2j3ld',
    '<C-v>jjx',
    '<C-v>jjX',
    '<C-v>jjc==<Esc>',
    '<C-v>jj$c==<Esc>',
    '<C-v>jjI==<Esc>',
    '<C-v>jjA==<Esc>',
    '<C-v>jj$A==<Esc>',
    '<C-v>jjrZ',
    '<C-v>jj~',
    '<C-v>jjU',
    '<C-v>jju',
    '<C-v>jj<',
    '<C-v>jj>',
    '<C-v>jjJ',
    '<C-v>jjgJ',
    '<C-v>jjo3ld',
    '<C-v>jj3lO d',
    '<C-v>jjy0P',
    '<C-v>jjy$p',
    '<C-v>jj3lyGp',
    '<C-v>jjD',
    '<C-v>jjC==<Esc>',
    '<C-v>jjS==<Esc>',
    '<C-v>jjR==<Esc>',
    '<C-v>jj<C-a>',
    '<C-v>jjg<C-a>',
    '<C-v>jjp',
    '<C-v>jjP',
    '<C-v>jjs==<Esc>',
    '<C-v>jj$r-',
    '<C-v>G$d',
  }
  for _, fx in ipairs({ { 'ragged', F.ragged, { 1, 2 } }, { 'utf', F.utf, { 1, 2 } }, { 'indent', F.indent, { 1, 1 } } }) do
    for _, keys in ipairs(BLOCK) do
      case(
        string.format('bk %s %s', fx[1], tag(keys)),
        fx[2],
        fx[3],
        keys
      )
    end
  end
  -- The same block operations with the tab expanded differently: a
  -- block edge that lands inside a tab is `block_prep`'s hardest case,
  -- and 'tabstop' decides where the edge is.
  for _, opts in ipairs({ 'tabstop=4', 'tabstop=8', 'tabstop=2 expandtab', 'tabstop=8 list' }) do
    for _, keys in ipairs({ '<C-v>jj3ld', '<C-v>jj3ly', '<C-v>jj3lI#<Esc>', '<C-v>jj3lc#<Esc>', '<C-v>jj$d' }) do
      case(
        string.format('bt %s %s', tag(opts), tag(keys)),
        F.indent,
        { 1, 0 },
        keys,
        opts
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s06 -- Visual mode
-- ---------------------------------------------------------------------

section('s06-visual', function()
  local VIS = {
    'vjd',
    'vjy',
    'vjc==<Esc>',
    'Vjd',
    'Vjy',
    'Vjc==<Esc>',
    'v$y',
    'v$d',
    'vj$y',
    'vlolld',
    'vjOd',
    'v3ld',
    'vipd',
    'vapy',
    'vawd',
    'viwgU',
    'vjgu',
    'vjg?',
    'vj<',
    'vj>',
    'vjJ',
    'vjgJ',
    'vjr-',
    'vjs==<Esc>',
    'vjS==<Esc>',
    'vjx',
    'vjX',
    'vjD',
    'vjC==<Esc>',
    'vjY',
    'vj~',
    'vjU',
    'vju',
    'vj<C-a>',
    'vjg<C-a>',
    'vj=',
    'vjgq',
    'vjzf',
    'vjg@',
    'yiwvjp',
    'yiwVjp',
    'yyvjp',
    'yyVjp',
    'vjy`<D',
    'vjy<Esc>gvd',
    'vjy<Esc>gvo<Esc>gvd',
    'vj<Esc>gvgv d',
    'vjyjvjp',
    'Vy2jVp',
  }
  for _, sel in ipairs({ 'inclusive', 'exclusive', 'old' }) do
    for _, keys in ipairs(VIS) do
      case(
        string.format('vs %s %s', sel, tag(keys)),
        F.prose,
        { 1, 4 },
        keys,
        'selection=' .. sel .. ' operatorfunc=OpFuncRec',
        false,
        keys:find('g@') and 'get(g:, "opfunc", "UNSET")' or nil
      )
    end
  end
  -- Select mode: the same keys through 'selectmode', where a printable
  -- character replaces the selection instead of being a command.
  for _, keys in ipairs({ 'gh<Down>X', 'gHX', 'g<C-h>X', 'vj<C-g>X', 'Vj<C-g>X', 'gh<Down><C-o>d', 'ghjy' }) do
    case('sm ' .. tag(keys), F.prose, { 1, 4 }, keys, 'selectmode=cmd,key,mouse')
  end
end)

-- ---------------------------------------------------------------------
-- s07 -- put
-- ---------------------------------------------------------------------

--- A case whose register file is loaded by hand before the sequence
--- runs.  `setreg` does not touch the buffer, so the tick baseline
--- taken by reset() is still the right one.
local function loaded(label, lines, pos, load, keys, opts, extra, after)
  reset(lines, pos, opts)
  for _, cmd in ipairs(load) do
    quiet(cmd)
  end
  local err = feed(keys)
  if err then
    emit(label, '!', esc(err))
  end
  shot(label, extra)
  if after then
    local ok, res = pcall(vim.fn.eval, 'string(' .. after .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
  end
end

-- The five register shapes do_put has to tell apart, plus the two
-- degenerate ones that are the usual off-by-one.
local REGSHAPES = {
  { 'char1', { [[call setreg('a', 'XY', 'v')]] } },
  { 'char2', { [[call setreg('a', ['XY', 'ZW'], 'v')]] } },
  { 'charnl', { [[call setreg('a', "XY\n", 'v')]] } },
  { 'line1', { [[call setreg('a', ['L1'], 'V')]] } },
  { 'line2', { [[call setreg('a', ['L1', 'L2'], 'V')]] } },
  { 'blk', { [[call setreg('a', ['B1', 'B2'], "\<C-v>2")]] } },
  { 'blkwide', { [[call setreg('a', ['B1', 'BB22'], "\<C-v>6")]] } },
  { 'empty', { [[call setreg('a', '', 'v')]] } },
  { 'emptyline', { [[call setreg('a', [''], 'V')]] } },
  { 'utf', { [[call setreg('a', ["é中", 'x'], 'v')]] } },
}

local PUTS = {
  '"ap',
  '"aP',
  '"agp',
  '"agP',
  '"a]p',
  '"a[p',
  '"a]P',
  '"a[P',
  '"azp',
  '"azP',
  '"a2p',
  '"a2P',
  '"a3gp',
  '"a2]p',
  '2"ap',
  '"aP.',
  '"apu',
  '"ap<C-r>',
}

section('s07-put', function()
  for _, shape in ipairs(REGSHAPES) do
    for _, keys in ipairs(PUTS) do
      for _, pos in ipairs({ { 1, 0 }, { 2, 6 }, { 3, 0 }, { 5, 14 } }) do
        loaded(
          string.format('pt %s %s %d.%d', shape[1], tag(keys), pos[1], pos[2]),
          F.prose,
          pos,
          shape[2],
          keys
        )
      end
    end
  end
  -- Put onto an empty buffer, and onto a buffer of one empty line:
  -- do_put's "buffer has no text" arm, which every off-by-one on
  -- curbuf->b_ml.ml_line_count reaches.
  for _, shape in ipairs(REGSHAPES) do
    for _, keys in ipairs({ '"ap', '"aP', '"a]p', '"agp' }) do
      loaded('pe ' .. shape[1] .. ' ' .. tag(keys), { '' }, { 1, 0 }, shape[2], keys)
    end
  end
  -- ]p / [p are the indent-adjusting puts: what they adjust *to* is
  -- 'shiftwidth' and the surrounding indent, so ask them on the indent
  -- fixture under three 'shiftwidth'/'expandtab' spellings.
  for _, opts in ipairs({ 'shiftwidth=8 noexpandtab', 'shiftwidth=4 expandtab', 'shiftwidth=2 expandtab tabstop=4' }) do
    for _, keys in ipairs({ '"a]p', '"a[p', '"a]P', '"a[P', '"a2]p', '"azp', '"azP' }) do
      for _, pos in ipairs({ { 1, 0 }, { 2, 4 }, { 4, 0 }, { 5, 2 } }) do
        loaded(
          string.format('pi %s %s %d.%d', tag(opts), tag(keys), pos[1], pos[2]),
          F.indent,
          pos,
          { [[call setreg('a', ['  I1', '    I2'], 'V')]] },
          keys,
          opts
        )
      end
    end
  end
  -- Put over a Visual selection replaces it *and* rewrites the unnamed
  -- register with what was replaced -- the half of do_put nothing else
  -- reaches.
  for _, shape in ipairs(REGSHAPES) do
    for _, keys in ipairs({ 'vj"ap', 'Vj"ap', '<C-v>j"ap', 'viw"ap', 'vip"aP', 'v$"ap' }) do
      loaded('pv ' .. shape[1] .. ' ' .. tag(keys), F.prose, { 2, 4 }, shape[2], keys)
    end
  end
  -- The other registers put reads: the unnamed default, the black hole,
  -- the expression register and the clipboard fixture.
  for _, keys in ipairs({
    'yiwp',
    'yiwP',
    'yyp',
    'yyP',
    'ddp',
    'ddP',
    'dd2p',
    'x p',
    'yiw"0p',
    'dd"1p',
    'x"-p',
    '"_ddp',
    '"_yiwp',
    'yiw"_p',
    '"=1+1<CR>p',
    '"=[1,2]<CR>p',
    '"=nosuchfn()<CR>p',
    '"=<CR>p',
    'yiw"+p',
    '"+yiw"+p',
    '"*yiwjj"*p',
    'yiw"*p',
  }) do
    case('po ' .. tag(keys), F.prose, { 2, 6 }, keys, nil, true)
  end
end)

-- ---------------------------------------------------------------------
-- s08 -- the named and numbered registers
-- ---------------------------------------------------------------------

section('s08-registers', function()
  -- Naming a register on a yank or a delete, and the uppercase append
  -- form, whose charwise/linewise mixing rule is its own arm.
  for _, keys in ipairs({
    '"ayiw',
    '"ayy',
    '"Ayiw',
    '"ayiww"Ayiw',
    '"ayywj"Ayy',
    '"ayiwj"Ayy',
    '"ayyj"Ayiw',
    '"add',
    '"Add',
    '"addj"Add',
    '"adw',
    '"aD',
    '"ax',
    '"aciw==<Esc>',
    '"acc==<Esc>',
    '"aC==<Esc>',
    '"a<C-v>jjy',
    '"a<C-v>jjd',
    '"A<C-v>jjy',
    '"zyiw',
    '"Zyiw',
    '"1yiw',
    '"9yiw',
    '"0yiw',
    '"-yiw',
  }) do
    case('rn ' .. tag(keys), F.prose, { 2, 6 }, keys)
  end
  -- The numbered shift.  A multi-line delete pushes "1 down the rank;
  -- a small (within-line) delete goes to "- and does *not* shift, and
  -- the exceptions to that rule are the whole of the arm.
  for _, keys in ipairs({
    'dddddd',
    'ddddddddd',
    'dddddddddddd',
    'dd'.. 'dd' .. 'dd' .. 'dd' .. 'dd' .. 'dd' .. 'dd' .. 'dd' .. 'dd' .. 'dd',
    'dwdwdw',
    'xxx',
    'dwdd dw',
    'd}d}',
    'dGu dd',
    'yiwdd',
    'ddyiw',
    'ddyy',
    'x dd x',
    'cwA<Esc>cwB<Esc>',
    'S1<Esc>S2<Esc>',
    'd/eta<CR>d/eta<CR>',
    '/eta<CR>ggdn dd',
    '"add dd',
    '"_dd dd',
    'v$d v$d',
    'Vd Vd',
    '<C-v>jjd <C-v>jjd',
  }) do
    case('rs ' .. tag(keys), F.prose, { 1, 0 }, keys, nil, false, '@1 . "|" . @2 . "|" . @- . "|" . @0')
  end
  -- What the unnamed register *points at* is a pointer in register.rs,
  -- not a copy; a rewrite that copies instead is invisible to the
  -- contents and visible here.
  for _, keys in ipairs({ '"ayiw', 'yiw', 'dd', 'x', '"_dd', 'yy', '"aY', 'ciwX<Esc>' }) do
    loaded(
      'rp ' .. tag(keys),
      F.prose,
      { 2, 6 },
      { [[call setreg('a', 'SEED', 'v')]], [[call setreg('"', 'UNNAMED', 'v')]] },
      keys,
      nil,
      true
    )
  end
end)

-- ---------------------------------------------------------------------
-- s09 -- the special registers
-- ---------------------------------------------------------------------

section('s09-special', function()
  -- "_ is a black hole on both sides; "= is evaluated at the moment it
  -- is read; ". : % # / are read-only views of editor state.
  for _, keys in ipairs({
    '"_yiw',
    '"_dd',
    '"_x',
    '"_ciwX<Esc>',
    'iabc<Esc>".p',
    'ciwXY<Esc>".p',
    'oNEW<Esc>".P',
    ':set nowrap<CR>":p',
    ':nosuchcmd<CR>":p',
    '/eta<CR>"/p',
    ':let @/="zz"<CR>"/p',
    '"%p',
    '"#p',
    '"=v:count<CR>p',
    '2"=v:count<CR>p',
    '"=@a<CR>p',
    '"=<C-r>=1+1<CR><CR>p',
  }) do
    case('sp ' .. tag(keys), F.prose, { 2, 6 }, keys, nil, true)
  end
  -- The clipboard registers, behind the FIXTURE provider installed at
  -- startup.  g:cbstore is the whole of the "system" clipboard here, so
  -- this section can never read or write the host's.
  for _, keys in ipairs({
    '"+yiw',
    '"*yiw',
    '"+yy',
    '"*dd',
    '"+p',
    '"*p',
    '"+yiwjj"+p',
    '"*yy"*P',
    '"+dd"*p',
    '"+<C-v>jjy',
    '"+yiw"*p',
  }) do
    for _, cb in ipairs({ '', 'unnamed', 'unnamedplus', 'unnamed,unnamedplus' }) do
      loaded(
        string.format('cb %s %s', cb == '' and 'none' or cb:gsub(',', '+'), tag(keys)),
        F.prose,
        { 2, 6 },
        { [[let g:cbstore = {'+': [['SEEDPLUS'], 'v'], '*': [['SEEDSTAR'], 'v']}]] },
        keys,
        'clipboard=' .. cb,
        true,
        'g:cbstore'
      )
    end
  end
  -- setreg/getreg/getreginfo in every shape, which is register.rs's api
  -- surface rather than its key surface.
  for _, expr in ipairs({
    [[setreg('a', 'x')]],
    [[setreg('a', 'x', 'v')]],
    [[setreg('a', 'x', 'V')]],
    [[setreg('a', 'x', 'b')]],
    [[setreg('a', 'x', 'b12')]],
    [[setreg('a', ['x', 'y'])]],
    [[setreg('a', ['x', 'y'], 'b')]],
    [[setreg('a', "x\ny")]],
    [[setreg('a', 'x', 'a')]],
    [[setreg('A', 'y')]],
    [[setreg('a', {})]],
    [[setreg('a', {'regcontents': ['p','q'], 'regtype': 'V'})]],
    [[setreg('a', {'regcontents': ['p','q'], 'regtype': 'b3', 'points_to': 'b'})]],
    [[setreg('', 'unnamedwrite')]],
    [[setreg('_', 'blackholewrite')]],
    [[setreg('/', 'searchwrite')]],
    [[setreg('=', '1+1')]],
    [[setreg('nosuch', 'x')]],
  }) do
    local label = 'sr ' .. tag(expr)
    reset(F.prose, { 1, 0 })
    local ok, res = pcall(vim.fn.eval, 'string(' .. expr .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    shot(label, true)
  end
  for _, expr in ipairs({
    [[getreg('a')]],
    [[getreg('a', 1)]],
    [[getreg('a', 1, 1)]],
    [[getreg('=')]],
    [[getreg('=', 1)]],
    [[getregtype('a')]],
    [[getregtype('nosuch')]],
    [[getreginfo('a')]],
    [[getreginfo('"')]],
    [[getreginfo('=')]],
    [[getreginfo('_')]],
    [[getregion(getpos("'<"), getpos("'>"))]],
  }) do
    reset(F.prose, { 1, 0 })
    quiet([[call setreg('a', ['p', 'q'], 'b3')]])
    quiet([[call setreg('=', '1+1')]])
    feed('vjy')
    local label = 'gr ' .. tag(expr)
    local ok, res = pcall(vim.fn.eval, 'string(' .. expr .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
  end
end)

-- ---------------------------------------------------------------------
-- s10 -- :registers
-- ---------------------------------------------------------------------

section('s10-reglist', function()
  local function listing(label, load)
    reset(F.prose, { 1, 0 })
    for _, cmd in ipairs(load) do
      quiet(cmd)
    end
    for _, cmd in ipairs({ 'registers', 'display', 'registers ab', 'registers "0-', 'reg =', 'display a1' }) do
      local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
      emit(label .. ' ' .. tag(cmd), ok and esc(scrub(res.output or '')) or esc(errtext(res)))
    end
  end
  listing('rl empty', {})
  listing('rl one', { [[call setreg('a', 'short', 'v')]] })
  listing('rl types', {
    [[call setreg('a', 'charwise', 'v')]],
    [[call setreg('b', ['l1', 'l2'], 'V')]],
    [[call setreg('c', ['b1', 'b2'], 'b4')]],
    [[call setreg('0', 'yanked', 'v')]],
    [[call setreg('1', ['del'], 'V')]],
    [[call setreg('-', 'small', 'v')]],
    [[call setreg('=', '1+1')]],
  })
  listing('rl wide', {
    [[call setreg('a', repeat('x', 200), 'v')]],
    [[call setreg('b', "tab\there\nand\nmore", 'v')]],
    [[call setreg('c', "é中文 wide", 'v')]],
    [[call setreg('d', "ctrl\x01\x1b\x7f chars", 'v')]],
  })
end)

-- ---------------------------------------------------------------------
-- s11 -- the ex commands that go through the same paths
-- ---------------------------------------------------------------------

section('s11-excmd', function()
  for _, cmd in ipairs({
    'd',
    '1d',
    '1,2d',
    '%d',
    '.,+1d',
    '2,$d',
    'd a',
    '1,2d a',
    '1,2d A',
    '2d 3',
    'd 2',
    '1,2d a 3',
    'd _',
    'y',
    '1,2y',
    'y a',
    '1,2y A',
    '2y 3',
    'y 2',
    'pu',
    'pu!',
    '0pu',
    '$pu',
    'pu a',
    'pu ="X"',
    'pu _',
    '1,2m0',
    '1,2m$',
    '1m1',
    '1,2co$',
    '1t2',
    '1,2>',
    '1,2<',
    '1,2>>',
    '1,3normal! dw',
    "normal! yiwjP",
    'normal dw',
    "1,2normal! A!",
    'g/eta/d',
    'g/eta/normal! dw',
    'v/eta/d',
    '1,2j',
    '1,3j!',
    'j',
    '1,2left',
    '1,2right 20',
    '1,2center 20',
  }) do
    local load = { [[call setreg('a', ['REGA'], 'V')]] }
    loaded('ex ' .. tag(cmd), F.prose, { 2, 6 }, load, ':' .. cmd .. '<CR>', nil, true)
  end
end)

-- ---------------------------------------------------------------------
-- s12 -- shift
-- ---------------------------------------------------------------------

section('s12-shift', function()
  local SHIFTOPTS = {
    'shiftwidth=8 tabstop=8 noexpandtab noshiftround',
    'shiftwidth=4 tabstop=8 noexpandtab noshiftround',
    'shiftwidth=4 tabstop=8 expandtab noshiftround',
    'shiftwidth=3 tabstop=8 noexpandtab shiftround',
    'shiftwidth=3 tabstop=8 expandtab shiftround',
    'shiftwidth=0 tabstop=4 noexpandtab noshiftround',
    'shiftwidth=8 tabstop=4 softtabstop=2 expandtab',
    'shiftwidth=2 tabstop=8 noexpandtab shiftround',
  }
  local SHIFTKEYS = {
    '>>',
    '<<',
    '3>>',
    '3<<',
    '>2>',
    '>j',
    '<j',
    '>ip',
    '<ip',
    '>G',
    '<G',
    'Vj>',
    'Vj<',
    'Vj3>',
    'Vj3<',
    'vj>',
    '<C-v>jj>',
    '<C-v>jj<',
    ':1,3><CR>',
    ':1,3<<CR>',
    ':1,3>>><CR>',
    '>>.',
    '<<u',
  }
  for _, opts in ipairs(SHIFTOPTS) do
    for _, keys in ipairs(SHIFTKEYS) do
      case(
        string.format('sh %s %s', tag(opts), tag(keys)),
        F.indent,
        { 1, 0 },
        keys,
        opts
      )
    end
  end
  -- An empty line and a line of nothing but whitespace are the two
  -- shapes shift_line special-cases.
  for _, opts in ipairs({ 'shiftwidth=4 expandtab', 'shiftwidth=4 noexpandtab' }) do
    for _, keys in ipairs({ '>>', '<<', 'Vjjj>', 'Vjjj<' }) do
      case(
        'sw ' .. tag(opts) .. ' ' .. tag(keys),
        { '', '   ', '\t', 'text', '    ' },
        { 1, 0 },
        keys,
        opts
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s13 -- case operators
-- ---------------------------------------------------------------------

section('s13-case', function()
  local CASEKEYS = {
    'guu',
    'gUU',
    'g~~',
    'g??',
    'guw',
    'gUw',
    'g~w',
    'g?w',
    'guiw',
    'gUiw',
    'g~iw',
    'g?iw',
    'gu$',
    'gU$',
    'g~$',
    'g?$',
    '2guu',
    '2gUU',
    'gu2j',
    '~',
    '3~',
    '10~',
    'v$~',
    'vjU',
    'vju',
    'vj~',
    'vjg?',
    '<C-v>jj~',
    '<C-v>jjU',
    'g?g?',
    'g?ip',
    'guG',
    'gUG',
  }
  for _, fx in ipairs({ { 'prose', F.prose }, { 'utf', F.utf }, { 'nest', F.nest } }) do
    for _, keys in ipairs(CASEKEYS) do
      case('cs ' .. fx[1] .. ' ' .. tag(keys), fx[2], { 1, 1 }, keys)
    end
  end
  -- 'tildeop' turns `~` into an operator, which is a different function
  -- entirely.
  for _, keys in ipairs({ '~', '~w', '~iw', '~~', '2~w', '~$', '~j' }) do
    case('ti ' .. tag(keys), F.prose, { 1, 1 }, keys, 'tildeop')
  end
end)

-- ---------------------------------------------------------------------
-- s14 -- join
-- ---------------------------------------------------------------------

section('s14-join', function()
  local JOINKEYS = {
    'J',
    'gJ',
    '2J',
    '3J',
    '4J',
    '2gJ',
    '3gJ',
    'VjJ',
    'VjgJ',
    'VjjJ',
    'vjJ',
    '<C-v>jjJ',
    'JJ',
    'J.',
    ':1,3j<CR>',
    ':1,3j!<CR>',
    ':j<CR>',
    '100J',
  }
  local JOINFIX = {
    { 'prose', F.prose },
    { 'comment', F.comment },
    { 'space', { 'end.', '   next', 'a', '', 'b', ')close', 'tail\t', '  lead' } },
    { 'utf', F.utf },
    { 'empty', { '', '', 'x', '' } },
  }
  for _, opts in ipairs({ 'nojoinspaces formatoptions=tcq', 'joinspaces formatoptions=tcq', 'nojoinspaces formatoptions=tcqj', 'joinspaces formatoptions=j cpoptions=aABceFsq' }) do
    for _, fx in ipairs(JOINFIX) do
      for _, keys in ipairs(JOINKEYS) do
        case(
          string.format('jn %s %s %s', tag(opts), fx[1], tag(keys)),
          fx[2],
          { 1, 0 },
          keys,
          opts
        )
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s15 -- CTRL-A / CTRL-X
-- ---------------------------------------------------------------------

section('s15-addsub', function()
  local NRF = {
    '',
    'bin',
    'hex',
    'octal',
    'alpha',
    'unsigned',
    'blank',
    'bin,hex',
    'octal,hex',
    'bin,hex,octal',
    'alpha,bin,hex,octal',
    'alpha,bin,hex,octal,unsigned',
    'unsigned,blank,hex',
  }
  local ADDKEYS = {
    '<C-a>',
    '<C-x>',
    '5<C-a>',
    '5<C-x>',
    '100<C-a>',
    '100<C-x>',
    '<C-a><C-a>',
    '<C-a>.',
    '<C-a>u',
    'V<C-a>',
    'Vg<C-a>',
    'VjG<C-a>',
    'VGg<C-a>',
    'VGg<C-x>',
    'VG2g<C-a>',
    '<C-v>G<C-a>',
    '<C-v>Gg<C-a>',
    'v$<C-a>',
    'vjg<C-a>',
    '$<C-a>',
    '0<C-a>',
  }
  for _, nrf in ipairs(NRF) do
    for _, keys in ipairs(ADDKEYS) do
      for _, pos in ipairs({ { 1, 0 }, { 2, 0 }, { 3, 1 }, { 4, 0 }, { 5, 0 }, { 6, 1 } }) do
        case(
          string.format('as %s %s %d.%d', nrf == '' and 'none' or nrf:gsub(',', '+'), tag(keys), pos[1], pos[2]),
          F.nums,
          pos,
          keys,
          'nrformats=' .. nrf
        )
      end
    end
  end
  -- The overflow and sign edges, which are do_addsub's own arithmetic
  -- rather than its number scanner.
  for _, keys in ipairs({ '<C-a>', '<C-x>', '2<C-a>', '2<C-x>', '999999999999999999999<C-a>' }) do
    case(
      'ax ' .. tag(keys),
      {
        '9223372036854775807',
        '-9223372036854775808',
        '18446744073709551615',
        '0xffffffffffffffff',
        '-0',
        '007',
        '0b',
        '0x',
        '99',
        '-1',
      },
      { 1, 0 },
      'VG' .. keys,
      'nrformats=bin,hex,octal,unsigned'
    )
  end
end)

-- ---------------------------------------------------------------------
-- s16 -- Insert mode
-- ---------------------------------------------------------------------

section('s16-insert', function()
  local INS = {
    'iXY<Esc>',
    'aXY<Esc>',
    'IXY<Esc>',
    'AXY<Esc>',
    'oXY<Esc>',
    'OXY<Esc>',
    'i<C-o>dw<Esc>',
    'i<C-o>:set sw=1<CR>X<Esc>',
    'i<C-o>ggX<Esc>',
    'yiwi<C-r>0<Esc>',
    'yiwi<C-r><C-r>0<Esc>',
    'yiwi<C-r><C-o>0<Esc>',
    'yiwi<C-r><C-p>0<Esc>',
    'i<C-r>=1+1<CR><Esc>',
    'i<C-r>=nosuch()<CR><Esc>',
    'i<C-v>065<Esc>',
    'i<C-v>u00e9<Esc>',
    'i<C-v>x41<Esc>',
    'i<C-v>U0001f600<Esc>',
    'i<C-v><Tab><Esc>',
    'i<C-v><Esc>x<Esc>',
    'i<C-q>065<Esc>',
    'A<C-w><Esc>',
    'A<C-w><C-w><Esc>',
    'A<C-u><Esc>',
    'AXY<C-u><Esc>',
    'i<C-t><Esc>',
    'i<C-d><Esc>',
    'i0<C-d><Esc>',
    'i^<C-d><Esc>',
    'ji<C-e><Esc>',
    'ji<C-y><Esc>',
    'i<C-a><Esc>',
    'iXY<Esc>i<C-a><Esc>',
    'i<C-@><Esc>',
    'iXY<Esc>o<C-@><Esc>',
    'i<C-k>e:<Esc>',
    'i<C-k>Co<Esc>',
    'i<C-k>zz<Esc>',
    'i<CR><Esc>',
    'A<CR>x<Esc>',
    'i<C-j><Esc>',
    'i<Tab>x<Esc>',
    'i<BS><Esc>',
    'A<BS><Esc>',
    'A<C-h><Esc>',
    'i<Del><Esc>',
    'A<Del><Esc>',
    'oX<Esc>ji<BS><BS><Esc>',
    'A<C-o>D<Esc>',
    'i<Left><Right>X<Esc>',
    'i<C-Left>X<Esc>',
    'i<S-Left>X<Esc>',
    'i<Up>X<Esc>',
    'i<Down>X<Esc>',
    'i<C-g>u X<Esc>u',
    'i<C-g>jX<Esc>',
    'i<C-g>kX<Esc>',
    'gi X<Esc>',
    'iabc<Esc>gi!<Esc>',
    'i<C-\\><C-o>ddX<Esc>',
    'i<C-\\><C-n>x',
  }
  for _, keys in ipairs(INS) do
    case('in ' .. tag(keys), F.prose, { 2, 6 }, keys, nil, true)
  end
  -- The 'backspace' matrix -- five spellings, each deciding whether the
  -- three boundaries (autoindent, line start, insert start) can be
  -- crossed.
  for _, bs in ipairs({ '', 'indent', 'eol', 'start', 'indent,eol', 'indent,eol,start', 'indent,eol,nostop', 'indent,eol,start,nostop', '0', '1', '2', '3' }) do
    for _, keys in ipairs({
      'jjA<BS><BS><BS><Esc>',
      'jjI<BS><BS><Esc>',
      'jjoXY<BS><BS><BS><BS><Esc>',
      'jjA<C-w><C-w><C-w><Esc>',
      'jjA<C-u><C-u><Esc>',
      'jjAX<BS><BS><BS><BS><BS><Esc>',
    }) do
      case(
        string.format('bs %s %s', bs == '' and 'none' or bs:gsub(',', '+'), tag(keys)),
        F.indent,
        { 1, 0 },
        keys,
        'backspace=' .. bs .. ' autoindent'
      )
    end
  end
  -- 'revins', 'paste' and abbreviations: three whole-mode switches that
  -- rewrite what the same keys do.
  for _, keys in ipairs({ 'iabc<Esc>', 'iab<BS>c<Esc>', 'A xyz<Esc>', 'i<CR>x<Esc>' }) do
    case('ri ' .. tag(keys), F.prose, { 2, 6 }, keys, 'revins')
    case('pa ' .. tag(keys), F.prose, { 2, 6 }, keys, 'paste')
    case('pi ' .. tag(keys), F.prose, { 2, 6 }, keys, 'paste autoindent expandtab shiftwidth=4 textwidth=20')
  end
  for _, keys in ipairs({
    'ifoo <Esc>',
    'ifoo<Esc>',
    'ifoo<CR><Esc>',
    'ifoo<C-]><Esc>',
    'i#i1 <Esc>',
    'ifoo bar <Esc>',
    'ofoo <Esc>',
    'ifoo<C-v> <Esc>',
    ':normal ifoo <CR>',
  }) do
    loaded(
      'ab ' .. tag(keys),
      F.prose,
      { 2, 6 },
      { 'iabbrev foo FOOBAR', 'iabbrev #i1 HASH', 'abbrev bar BARBAR' },
      keys
    )
  end
  -- 'textwidth' auto-wrap and 'formatoptions' from Insert mode is
  -- textformat.rs reached through edit.rs; fmtsweep owns the gq half,
  -- this is the typing half.
  for _, fo in ipairs({ 'tcq', 'tcqa', 'tcqn', 'tcql', 'cq', 'tcqw', 'tcqr', 'tcqo' }) do
    for _, keys in ipairs({
      'A more words here to force a wrap<Esc>',
      'o- a list item that is long enough to wrap around<Esc>',
      'A<CR>continued<Esc>',
      'oplain<CR>more<Esc>',
    }) do
      case(
        string.format('tw %s %s', fo, tag(keys)),
        F.comment,
        { 1, 0 },
        keys,
        'textwidth=20 formatoptions=' .. fo
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s17 -- Replace mode and the one-key changes
-- ---------------------------------------------------------------------

section('s17-replace', function()
  local REP = {
    'rX',
    '3rX',
    'r<CR>',
    '3r<CR>',
    'r<Tab>',
    '$rX',
    '99rX',
    'grX',
    '3grX',
    'gr<CR>',
    -- Visual `r` is the only way into ops.rs's op_replace: Normal-mode
    -- `r` is normal.rs's own path, so without these the whole function
    -- is unreached.  On the utf8 fixture the cursor starts on the lead
    -- byte of a two-byte character, which is the arm that decides
    -- between replace_character() and a single-byte poke.
    'vlrX',
    'v$rX',
    'VrX',
    'VjrX',
    '<C-v>jlrX',
    'vlr<CR>',
    'RXY<Esc>',
    'RXY<BS><BS><Esc>',
    'RXY<BS><BS><BS><BS><Esc>',
    'R<CR>X<Esc>',
    'gRXY<Esc>',
    'gRXY<BS><BS><Esc>',
    'R<C-o>dw<Esc>',
    'R<C-v>065<Esc>',
    '2RX<Esc>',
    '3RXY<Esc>',
    'x',
    'X',
    '3x',
    '3X',
    '$x',
    's',
    'sXY<Esc>',
    '3sXY<Esc>',
    'S',
    'SXY<Esc>',
    '2SXY<Esc>',
    'C',
    'CXY<Esc>',
    '2CXY<Esc>',
    'D',
    '2D',
    'Y',
    '2Y',
    'yiwvjp',
    'ciwXY<Esc>.',
    'cwXY<Esc>w.',
    'xp',
    'ddp',
    'ddu',
    'ddU',
    'ddup',
    'x3.',
  }
  for _, fx in ipairs({ { 'prose', F.prose }, { 'utf', F.utf } }) do
    for _, keys in ipairs(REP) do
      case('rp ' .. fx[1] .. ' ' .. tag(keys), fx[2], { 1, 2 }, keys, nil, true)
    end
  end
end)

-- ---------------------------------------------------------------------
-- s18 -- marks and b:changedtick
-- ---------------------------------------------------------------------

section('s18-marks', function()
  local MARKKEYS = {
    'yiw',
    'yy',
    'dd',
    'dw',
    'x',
    'p',
    'yyp',
    'yyP',
    'ciwXY<Esc>',
    'oNEW<Esc>',
    'ONEW<Esc>',
    'A!<Esc>',
    'J',
    'gJ',
    '>>',
    '<<',
    'gUU',
    '<C-a>',
    'rX',
    'RXY<Esc>',
    'vjd',
    'Vjy',
    '<C-v>jjd',
    'ddu',
    'ddu<C-r>',
    'yiwu',
    ':1,2d<CR>',
    ':1,2y<CR>',
    ':pu<CR>',
    ':1,2m$<CR>',
    ':1,2co0<CR>',
    ':g/eta/d<CR>',
    ':1,2normal! dw<CR>',
    'ma`a',
    "majj'a",
    'iX<Esc>`.',
    'iX<Esc>gg`^',
    'yiwjjgv',
    'vjy<Esc>gv',
  }
  for _, keys in ipairs(MARKKEYS) do
    case(
      'mk ' .. tag(keys),
      F.prose,
      { 2, 6 },
      keys,
      nil,
      false,
      [[string(getpos("'[")) . string(getpos("']")) . string(getpos("'<")) . string(getpos("'>")) . string(getpos("'.")) . string(getpos("'^"))]]
    )
  end
  -- b:changedtick under undo/redo, and the edits that make no change at
  -- all -- the ones a rewrite most easily makes tick anyway.
  -- undotree().seq_cur is one counter for the buffer, so the absolute
  -- value would make each of these a function of every case above it.
  -- g:opsseq is taken after the reset and the answer is the delta,
  -- which is the actual question: where in its own undo history did
  -- this sequence leave the buffer, and how far did the history grow.
  for _, keys in ipairs({
    'u',
    '<C-r>',
    'ddu',
    'ddu<C-r>',
    'dduu',
    'dd<C-r>',
    'yiw',
    'i<Esc>',
    'a<Esc>',
    ':d<CR>u',
    'guu',
    'gUUgUU',
    '>><<',
    'x u <C-r>',
    'ddddu u <C-r><C-r>',
    'g-',
    'g+',
    'ddg-',
    'ddg-g+',
    ':undo 0<CR>',
    ':earlier 1<CR>',
    ':later 1<CR>',
  }) do
    loaded(
      'tk ' .. tag(keys),
      F.prose,
      { 2, 6 },
      { 'let g:opsseq = undotree().seq_cur' },
      keys,
      nil,
      false,
      -- Deltas only: len(undotree().entries) is a count over the whole
      -- run (and saturates at 'undolevels'), so it moved when a case was
      -- added to a section eight above this one.
      '(undotree().seq_cur - g:opsseq) . "/" . (undotree().seq_last - g:opsseq)'
    )
  end
end)

-- ---------------------------------------------------------------------
-- s19 -- what an operator tells the rest of the editor
-- ---------------------------------------------------------------------

section('s19-events', function()
  local WATCH = table.concat({
    'let g:ev = []',
    'autocmd TextYankPost * call add(g:ev, "yank " . string(v:event))',
    'autocmd TextChanged * call add(g:ev, "chg")',
    'autocmd TextChangedI * call add(g:ev, "chgi")',
    'autocmd TextChangedP * call add(g:ev, "chgp")',
  }, ' | ')
  for _, keys in ipairs({
    'yiw',
    'yy',
    '"ayy',
    '"Ayy',
    'dd',
    'dw',
    'x',
    'ciwX<Esc>',
    '"_yiw',
    '"_dd',
    'vjy',
    '<C-v>jjy',
    ':1,2y<CR>',
    ':1,2d<CR>',
    'yiwp',
    'iX<Esc>',
    'oX<Esc>',
    '<C-a>',
    'J',
    '>>',
    'yiw"+yiw',
  }) do
    reset(F.prose, { 2, 6 })
    quiet('augroup OPSEV | autocmd! | ' .. WATCH .. ' | augroup END')
    feed(keys)
    local label = 'ev ' .. tag(keys)
    shot(label)
    local ok, res = pcall(vim.fn.eval, 'string(g:ev)')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    quiet('augroup OPSEV | autocmd! | augroup END')
  end
  -- cursor_pos_info: `g CTRL-G` in every mode, which is 415 lines of
  -- ops.rs reachable from nothing else.
  for _, keys in ipairs({ 'g<C-g>', 'vjg<C-g>', 'Vjg<C-g>', '<C-v>jjg<C-g>', 'v$g<C-g>', 'GVg<C-g>' }) do
    for _, fx in ipairs({ { 'prose', F.prose }, { 'utf', F.utf }, { 'empty', { '' } } }) do
      reset(fx[2], { 1, 0 })
      feed(keys)
      local label = 'cp ' .. fx[1] .. ' ' .. tag(keys)
      local ok, res = pcall(vim.fn.eval, 'string(v:statusmsg)')
      emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    end
  end
end)

-- ---------------------------------------------------------------------
-- s20 -- the message path, uncaptured
-- ---------------------------------------------------------------------

section('s20-messages', function()
  -- Everything above runs with report=9999 so that "3 fewer lines" does
  -- not land in every case's stderr in an order nobody reads.  This
  -- section is the opposite: report=0 and nothing captured, so the
  -- third artifact carries the real msg_* path for this subsystem --
  -- which is where a rewrite's off-by-one on a line count shows up
  -- first.
  emit('msg see stderr')
  vim.api.nvim_command('echo "-- s20 begin"')
  for _, keys in ipairs({
    '3dd',
    '3yy',
    'dG',
    'yG',
    '<C-v>jjy',
    '<C-v>jjd',
    'Vjy',
    'Vjd',
    ':1,3d<CR>',
    ':1,3y<CR>',
    ':1,3>><CR>',
    ':1,3m$<CR>',
    ':1,3co0<CR>',
    'u',
    '<C-r>',
    'g<C-g>',
    '3J',
    'ggVGg?',
  }) do
    reset(F.prose, { 1, 0 }, 'report=0')
    vim.api.nvim_command('echo "> ' .. tag(keys) .. '"')
    feed(keys)
  end
  -- The error arms.  Each one is the only observable behaviour its
  -- branch has.
  for _, keys in ipairs({
    '"zp',
    '"zP',
    '"zgp',
    ':pu z<CR>',
    ':pu =nosuch()<CR>',
    '"=nosuchfn()<CR>p',
    'ggdk',
    'Gdj',
    'gg0dh',
    '$dl',
    'y<Esc>',
    'd<Esc>',
    ':1,2d qq<CR>',
    ':1,2y "<CR>',
    ':normal! <CR>',
    ':1,2j<CR>:1,2j<CR>',
    'G100J',
    ':d 0<CR>',
    ':1,2d 99999999999999999999<CR>',
    'ggVGzf zfj',
    '"=<C-r>=<CR><CR>p',
  }) do
    reset(F.prose, { 1, 0 }, 'report=0')
    vim.api.nvim_command('echo "! ' .. tag(keys) .. '"')
    feed(keys)
  end
  reset(F.prose, { 1, 0 })
  vim.api.nvim_command('echo "-- s20 registers"')
  vim.api.nvim_command([[call setreg('a', 'charwise', 'v')]])
  vim.api.nvim_command([[call setreg('b', ['l1', 'l2'], 'V')]])
  vim.api.nvim_command([[call setreg('c', ['b1', 'b2'], 'b4')]])
  vim.api.nvim_command('registers')
  vim.api.nvim_command('echo "-- s20 end"')
  quiet('set report=9999')
end)

-- ---------------------------------------------------------------------
-- s21 -- the timeout arms, in a child
-- ---------------------------------------------------------------------

section('s21-child-updatetime', function()
  -- nvim_feedkeys(..., 'x') runs the typeahead to completion and
  -- returns; the main loop never becomes idle inside it, so
  -- 'updatetime' can never expire and CursorHold/CursorHoldI are
  -- unreachable from this process however the keys are spelled.  The
  -- only way to ask is a second nvim that is genuinely idle, driven
  -- over RPC.
  --
  -- Nothing here records a duration: a wall-clock number is not
  -- reproducible.  What is recorded is *whether* each event fired, in
  -- what order, and what the editor looked like when it did.
  local ok, chan = pcall(vim.fn.jobstart, {
    vim.v.progpath,
    '--embed',
    '--headless',
    '-n',
    '-u',
    'NONE',
    '-i',
    'NONE',
  }, { rpc = true })
  if not ok or chan <= 0 then
    emit('ch spawn', '!', esc(tostring(chan)))
    return
  end

  local function rr(method, ...)
    local sok, res = pcall(vim.rpcrequest, chan, method, ...)
    if not sok then
      return nil, errtext(res)
    end
    return res, nil
  end

  local function ask(expr)
    local res, err = rr('nvim_eval', 'string(' .. expr .. ')')
    return err and ('!' .. err) or scrub(tostring(res))
  end

  --- Poll until `expr` is true or the deadline passes.  The deadline is
  --- an order of magnitude past 'updatetime' so that a slow machine
  --- answers the same as a fast one; the answer recorded is the
  --- predicate, never the elapsed time.
  local function await(expr)
    for _ = 1, 60 do
      local res = rr('nvim_eval', expr)
      if res == 1 or res == true then
        return true
      end
      vim.uv.sleep(50)
    end
    return false
  end

  rr(
    'nvim_exec2',
    table.concat({
      'set updatetime=50 noswapfile nomore shortmess=filnxtToOF',
      'let g:hold = 0 | let g:holdi = 0 | let g:trace = []',
      'autocmd CursorHold * let g:hold += 1 | call add(g:trace, "hold ".mode().":".line(".").":".col("."))',
      'autocmd CursorHoldI * let g:holdi += 1 | call add(g:trace, "holdi ".mode().":".line(".").":".col("."))',
      'autocmd CursorMoved * call add(g:trace, "moved")',
      'autocmd CursorMovedI * call add(g:trace, "movedi")',
      'call setline(1, ["alpha beta", "second line", "third"])',
    }, ' | '),
    vim.empty_dict()
  )

  local STEPS = {
    { 'normal-idle', 'jl', 'g:hold > 0' },
    { 'insert-idle', 'iX', 'g:holdi > 0' },
    { 'insert-again', 'Y', 'g:holdi > 1' },
    { 'back-to-normal', '<Esc>l', 'g:hold > 1' },
  }
  for _, step in ipairs(STEPS) do
    rr('nvim_input', step[2] == '<Esc>l' and '\27l' or step[2])
    local fired = await(step[3])
    emit('ch ' .. step[1], 'fired=' .. tostring(fired), 'hold=' .. ask('g:hold'), 'holdi=' .. ask('g:holdi'), 'mode=' .. ask('mode(1)'))
  end
  emit('ch trace', esc(ask('g:trace')))
  emit('ch buffer', esc(ask('getline(1, "$")')))
  emit('ch tick', esc(ask('b:changedtick - 2')))
  struct('ch trace', ask('g:trace'))

  -- 'updatetime' also drives the swap-file write and CursorHold's
  -- one-shot rule: it does not fire again until a key arrives.
  rr('nvim_exec2', 'let g:hold = 0 | let g:trace = []', vim.empty_dict())
  vim.uv.sleep(400)
  emit('ch quiescent', 'hold=' .. ask('g:hold'), 'trace=' .. esc(ask('g:trace')))

  pcall(vim.fn.jobstop, chan)
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- The FIXTURE clipboard provider.  `"+` and `"*` go through
-- provider#clipboard#Call, and the host's real clipboard is never
-- touched: g:cbstore is the whole of the "system" clipboard for this
-- run.  cache_enabled is 0 so that a paste always calls back in rather
-- than answering from nvim's own copy, which is the arm register.rs
-- actually has to get right.
quiet([[
  let g:cbstore = {'+': [[''], 'v'], '*': [[''], 'v']}
  function! OpsCbCopy(reg, lines, regtype) abort
    let g:cbstore[a:reg] = [a:lines, a:regtype]
  endfunction
  function! OpsCbPaste(reg) abort
    return get(g:cbstore, a:reg, [[''], 'v'])
  endfunction
  let g:clipboard = {
        \ 'name': 'opsweep-fixture',
        \ 'copy': {
        \   '+': {lines, regtype -> OpsCbCopy('+', lines, regtype)},
        \   '*': {lines, regtype -> OpsCbCopy('*', lines, regtype)},
        \ },
        \ 'paste': {
        \   '+': {-> OpsCbPaste('+')},
        \   '*': {-> OpsCbPaste('*')},
        \ },
        \ 'cache_enabled': 0,
        \ }
]])

opfunc_setup()

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 fileencoding= isprint=@,161-255 ambiwidth=single')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set nofoldenable foldmethod=manual undolevels=1000 undofile&')
quiet('language C')
quiet(DEFAULTS)

-- One scratch buffer for the whole run: a fresh buffer per case would
-- hand out thousands of monotonic handles and make the artifact a
-- function of where in the run a case sits.
quiet('enew!')
BUF = vim.api.nvim_get_current_buf()
quiet('setlocal buftype=nofile bufhidden=hide noswapfile')

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
      -- OPSWEEP_TRACE writes into the .stderr artifact and must be off
      -- for a baseline.
      io.stderr:write(string.format('   %s %.1fs\n', entry.name, (vim.uv.hrtime() - started) / 1e9))
    end
  end
end

emit('')
emit('== done ==')
if structfd then
  structfd:close()
end
