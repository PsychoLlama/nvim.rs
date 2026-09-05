-- Driver for the diff differential sweep; see diffsweep.sh.
--
-- Covers diff.rs (4,923 lines) and the six vendored xdiff/ files (3,992),
-- which no existing differential reaches.  `test_diffmode` is the only
-- thing that does, and 104 of its assertions are skipped screendumps.
--
-- Every case is a *fixture pair* (sometimes a triple) put into one
-- tabpage of diff'd windows under one 'diffopt' spelling, and the answer
-- is the whole observable diff state:
--
--   H  one letter per line from `diff_hlID(lnum, 1)` -- `.` none,
--      A DiffAdd, C DiffChange, T DiffText, D DiffDelete, X DiffTextAdd.
--      This is the hunk computation read back line by line.
--   F  `diff_filler(lnum)` per line: the filler the other window needs
--      to line up, i.e. the same hunks read from the other end.
--   O  the closed-fold ranges 'foldmethod=diff' produces (diff_infold),
--      plus the fold level of every line.
--   N  `]c` and `[c` traversal from both ends (diff_move_to).
--   I  for the `inline:` modes, `diff_hlID(lnum, col)` per *column* on
--      every changed line.  f_diff_hlID resolves the same change list
--      the drawer does, so this is the only way to see the inline diff
--      from a headless process -- the highlight itself only exists on a
--      screen, and this sweep deliberately has none.
--
--   s01 the eleven fixture pairs at the default 'diffopt'
--   s02 the flag half of the 'diffopt' matrix
--   s03 algorithm:{myers,minimal,patience,histogram} x indent-heuristic
--   s04 iwhite / iwhiteall / iwhiteeol / iblank / icase
--   s05 context:N -- and therefore the fold ranges
--   s06 linematch:N
--   s07 inline:{none,simple,char,word}, per column
--   s08 `]c` / `[c` with counts, from every line of a fixture
--   s09 :diffget / :diffput / do / dp, with ranges, two-way and three-way
--   s10 diffopt=external -- the shell-out path
--   s11 'diffexpr' -- what the callback is handed, and a hand diff
--   s12 :diffpatch and 'patchexpr'
--   s13 'diffanchors' (global and buffer-local) x diffopt+=anchor
--   s14 diff_set_topline: the topline/topfill a scroll-bound partner gets
--   s15 diff_mark_adjust: edits made *without* a :diffupdate
--   s16 :diffsplit / :diffthis / :diffoff / :diffoff!, and what the
--       window-local option save-and-restore does
--   s17 diff_filler() / diff_hlID() outside a diff, and at edge lnums
--   s18 `vim.diff()` -- lua/xdiff.rs, the other consumer of xdiff/, and
--       the only one that reaches xemit.rs's formatter or
--       XDF_IGNORE_CR_AT_EOL
--   s90 the error and message arms, run UNCAPTURED, so the .stderr
--       artifact carries signal instead of being empty
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time, buffer handle or path outside the
-- work directory.  Temp file names in particular are random by
-- construction (nvim makes "$TMPDIR/nvim.<user>/<6 random>/<counter>"),
-- so a callback records *whether* it was handed a name, never the name.
--
-- DIFFSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.
-- DIFFSWEEP_TRACE=1 mirrors each section name to stderr, which is the
-- only way to see where a wedged run stopped.

local work = assert(os.getenv('DIFF_WORK'), 'DIFF_WORK unset')
local structpath = assert(os.getenv('DIFF_STRUCT'), 'DIFF_STRUCT unset')

local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('DIFFSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('DIFFSWEEP_TRACE') == '1'

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
  -- The temp file family first: its prefix is inside <WORK>, so the
  -- <WORK> substitution would otherwise eat the part that identifies it.
  text = text:gsub(vim.pesc(work) .. '/tmp/nvim%.[^/%s]*/%w+/%d+', '<TMPFILE>')
  text = text:gsub(vim.pesc(work) .. '/tmp/nvim%.[^/%s]*/%w+', '<TMPDIR>')
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
--- and a report line stays a report line.  The multibyte and near-binary
--- fixtures make this load-bearing: the whole point of them is which
--- *bytes* the hunk computation paired up.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- A case label has to be *injective* over what it names: two cases
--- sharing a label read as one case in the diff and the second one
--- silently stops gating.  A blanket punctuation-to-`_` mangle is not
--- injective (it collapsed `>>` onto `<<` in opsweep), so every
--- punctuation character gets its own token and shot() checks.
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
  [','] = 'c',
  ['.'] = 'dt',
  [':'] = 'l',
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
-- Canonical dump.  Verbatim from opsweep.lua / varsweep.lua: the
-- artifacts are read side by side often enough that they must escape and
-- sort the same way.
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

--- Normalise an error to its message: a pcall against vim.fn or vim.api
--- prefixes the Lua source position, which is a line number in this file
--- and would re-baseline the artifact on any edit.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^nvim_exec2%(%), line %d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

local function quiet(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = false })
  if not ok then
    return errtext(res)
  end
  return nil
end

-- ---------------------------------------------------------------------
-- Fixtures.  Named, so a case says which shape it is asking about and
-- two sections asking the same question use the same bytes.
-- ---------------------------------------------------------------------

local BASE = {
  '#include <stdio.h>',
  '#include <stdlib.h>',
  '',
  '/* a small program used as the diff corpus */',
  '',
  'static int total;',
  'static const char *name = "corpus";',
  '',
  'int add(int a, int b)',
  '{',
  '  total += 1;',
  '  return a + b;',
  '}',
  '',
  'int sub(int a, int b)',
  '{',
  '  total += 1;',
  '  return a - b;',
  '}',
  '',
  'int mul(int a, int b)',
  '{',
  '  total += 1;',
  '  return a * b;',
  '}',
  '',
  'int main(int argc, char **argv)',
  '{',
  '  int x = add(1, 2);',
  '  int y = sub(5, 3);',
  '  int z = mul(x, y);',
  '  printf("%s %d %d %d", name, x, y, z);',
  '  if (argc > 1) {',
  '    printf("arg %s", argv[1]);',
  '  }',
  '  return 0;',
  '}',
  '',
  '/* end of corpus */',
}

local function copy(list)
  return vim.list_extend({}, list)
end

--- Build a variant of BASE.  `edits` is applied in order; each entry is
--- {at, remove, insert...} with `at` a 1-based line of the ORIGINAL, so
--- the edits must be listed bottom-up.
local function variant(edits)
  local out = copy(BASE)
  for _, e in ipairs(edits) do
    local at, remove = e[1], e[2]
    local ins = {}
    for i = 3, #e do
      ins[#ins + 1] = e[i]
    end
    local tailidx = at + remove
    local head = {}
    for i = 1, at - 1 do
      head[#head + 1] = out[i]
    end
    vim.list_extend(head, ins)
    for i = tailidx, #out do
      head[#head + 1] = out[i]
    end
    out = head
  end
  return out
end

local LONG = ('abcdefghij'):rep(200)

local PAIRS = {}
local function pair(name, a, b, note)
  PAIRS[#PAIRS + 1] = { name = name, a = a, b = b, note = note }
  PAIRS[name] = PAIRS[#PAIRS]
end

pair('same', BASE, copy(BASE), 'byte identical: every hunk routine must find nothing')

pair(
  'ins',
  BASE,
  variant({ { 24, 0, '  if (a == 0 || b == 0) {', '    return 0;', '  }' } }),
  'insert-only, three lines inside mul()'
)

pair('del', BASE, variant({ { 15, 6 } }), 'delete-only: the whole sub() function goes')

pair('move', BASE, variant({
  { 21, 6 },
  { 9, 0, 'int mul(int a, int b)', '{', '  total += 1;', '  return a * b;', '}', '' },
}), 'a five-line block moved from the middle to the top')

pair('white', BASE, (function()
  local out = copy(BASE)
  for i, line in ipairs(out) do
    if line:match('^  ') then
      out[i] = line:gsub('^  ', '\t'):gsub('%+%= 1', '+=  1')
    elseif i % 7 == 0 then
      out[i] = line .. '   '
    end
  end
  return out
end)(), 'whitespace only: leading tabs for spaces, doubled internal, trailing')

pair('blank', BASE, variant({
  { 26, 0, '', '' },
  { 14, 0, '' },
  { 3, 0, '', '' },
}), 'blank lines added and nothing else')

pair('case', BASE, (function()
  local out = copy(BASE)
  out[6] = 'static int TOTAL;'
  out[11] = '  TOTAL += 1;'
  out[21] = 'int MUL(int A, int B)'
  out[31] = '  int z = MUL(x, y);'
  return out
end)(), 'case-only differences on four lines')

pair('crlf', BASE, (function()
  local out = copy(BASE)
  for i, line in ipairs(out) do
    out[i] = line .. '\r'
  end
  return out
end)(), 'every line gains a CR: dos endings read as a text difference')

pair('mb', {
  'ascii head',
  'κόσμος alpha βήτα',
  '日本語のテキストです',
  'e\204\129 combining acute',
  'emoji \240\159\145\168\226\128\141\240\159\146\187 family',
  'tail ascii',
  'shared 1',
  'shared 2',
  'shared 3',
  'shared 4',
  'shared 5',
  'shared 6',
  'shared 7',
  'shared 8',
}, {
  'ascii head',
  'κόσμος ALPHA βήτα',
  '日本語のテキストだ',
  '\195\169 precomposed acute',
  'emoji \240\159\145\169\226\128\141\240\159\146\187 family',
  'tail ascii',
  'shared 1',
  'shared 2',
  'shared 3',
  'shared 4',
  'shared 5',
  'shared 6',
  'shared 7',
  'shared 8',
}, 'multibyte: CJK, combining vs precomposed, a ZWJ emoji sequence')

pair('long', {
  'head',
  LONG,
  'tail',
  'shared 1',
  'shared 2',
  'shared 3',
  'shared 4',
  'shared 5',
}, {
  'head',
  LONG:sub(1, 900) .. 'ZZZZ' .. LONG:sub(905),
  'tail',
  'shared 1',
  'shared 2',
  'shared 3',
  'shared 4',
  'shared 5',
}, 'one 2,000-byte line differing four bytes into the middle')

pair('empty', { '' }, copy(BASE), 'an empty buffer against a full one')

pair('bothempty', { '' }, { '' }, 'two empty buffers')

pair('bin', {
  'head',
  'ctrl \1\2\3\4\5\6\7\8 bytes',
  'del \127 and high \200\201\202',
  'lone lead \195 here',
  'tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, {
  'head',
  'ctrl \1\2\3\11\12\6\7\8 bytes',
  'del \127 and high \200\255\202',
  'lone lead \194 here',
  'tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, 'near-binary: control bytes, DEL, invalid UTF-8, a lone lead byte')

-- Repeated low-information lines: the classic case where myers, patience
-- and histogram place the hunk boundary in three different spots.
pair('algo', {
  'preamble',
  '{',
  '  alpha();',
  '}',
  '{',
  '  beta();',
  '}',
  '{',
  '  gamma();',
  '}',
  'postamble',
}, {
  'preamble',
  '{',
  '  alpha();',
  '}',
  '{',
  '  inserted();',
  '}',
  '{',
  '  beta();',
  '}',
  '{',
  '  gamma();',
  '}',
  'postamble',
}, 'repeated braces around a unique line: myers/patience/histogram differ')

-- The upstream indent-heuristic demonstration: a hunk that can slide,
-- and only the heuristic puts it at the shallower indent.
pair('indent', {
  'function f() {',
  '  if (a) {',
  '    x();',
  '  }',
  '}',
  '',
  'function g() {',
  '  y();',
  '}',
}, {
  'function f() {',
  '  if (a) {',
  '    x();',
  '  }',
  '}',
  '',
  'function newone() {',
  '  z();',
  '}',
  '',
  'function g() {',
  '  y();',
  '}',
}, 'a slidable hunk: indent-heuristic decides which brace pair moves')

-- Two changed blocks two lines apart: what linematch:N re-pairs.
pair('near', {
  'alpha one',
  'beta two',
  'gamma three',
  'delta four',
  'epsilon five',
  'zeta six',
  'eta seven',
  'theta eight',
}, {
  'alpha ONE',
  'beta two changed',
  'gamma three',
  'delta FOUR',
  'epsilon five!',
  'zeta six',
  'eta seven',
  'theta eight',
}, 'two changed blocks two lines apart, for linematch')

-- A single insertion whose hunk an anchor at line 6 visibly splits: with
-- `diffopt+=anchor` and 'diffanchors' at 6 the same pair produces two
-- hunks instead of one, which is the only shape in this corpus where the
-- anchor path changes an answer at all.
pair('anch', {
  'a',
  'b',
  'c',
  'd',
  'e',
  'f',
  'g',
  'h',
}, {
  'a',
  'b',
  'X',
  'c',
  'd',
  'e',
  'f',
  'g',
  'h',
}, 'one insertion; an anchor at line 6 splits its hunk in two')

-- Every pair below was added because a seeded mutation measured NOT
-- CAUGHT against the first corpus.  Each one names the arm it reaches.

-- diff_equal_char's *multibyte* icase arm.  `case` and `mb` between them
-- did not reach it: `case` differs only in ASCII, and `mb`'s differences
-- are not case differences, so the fold was only ever asked about
-- single-byte characters.
pair('mbcase', {
  'ΚΌΣΜΟΣ ΑΛΦΑ ΒΉΤΑ',
  'ЖУРНАЛ ПРИМЕР',
  'ÉCOLE NAÏVE FAÇADE',
  'STRASSE GROSS',
  -- The four above differ *only* in case, so under 'icase' they fold to
  -- the same bytes in diff_write_buffer and no block survives -- which
  -- means diff_equal_char is never asked about them.  These three differ
  -- in case AND in one word, so the block stays, and the inline scan in
  -- diff_find_change_simple walks a Greek/Cyrillic/accented pair through
  -- diff_equal_char's multibyte arm looking for where they diverge.
  'ΚΌΣΜΟΣ mixed ALPHA',
  'ЖУРНАЛ tail one',
  'ÉCOLE δέλτα Xray',
  'ascii tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, {
  'κόσμος αλφα βήτα',
  'журнал пример',
  'école naïve façade',
  'strasse gross',
  'κόσμος mixed BETA',
  'журнал tail two',
  'école δέλτα Yoke',
  'ascii tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, 'case-only differences in Greek, Cyrillic and accented Latin, plus three that also differ in content')

-- diff_cmp's 'iblank' arm, which calls two lines equal when *either* is
-- blank.  The `blank` pair only ever *adds* blank lines, so the two
-- sides had different counts and the comparison never ran.
pair('bvsx', {
  'alpha',
  'beta',
  '',
  '   ',
  'delta',
  'epsilon',
  'zeta',
  'eta',
}, {
  'alpha',
  'beta',
  'INSERTED',
  'ALSO HERE',
  'delta',
  'epsilon',
  'zeta',
  'eta',
}, 'a blank line against content at the same line number, for iblank')

-- A second shape on which myers/minimal and patience/histogram disagree
-- (two functions swapped), so the `algorithm:` table is gated by more
-- than the `move` pair alone.
pair('swap', {
  '#include <stdio.h>',
  '',
  'int f(void)',
  '{',
  '  return 1;',
  '}',
  '',
  'int g(void)',
  '{',
  '  return 2;',
  '}',
}, {
  '#include <stdio.h>',
  '',
  'int g(void)',
  '{',
  '  return 2;',
  '}',
  '',
  'int f(void)',
  '{',
  '  return 1;',
  '}',
}, 'two functions swapped: myers/minimal see changes, patience/histogram a move')

-- xdl_change_compact's indent-heuristic pass.  The first corpus had
-- 'indent-heuristic' on and off answering *identically everywhere* --
-- a whole option that read as covered and gated nothing.  This is the
-- shape where the slide is genuinely ambiguous and the heuristic moves
-- the hunk one line up.
pair('slide', {
  '\t{',
  '\t\ta();',
  '\t}',
  '\t{',
  '\t\tb();',
  '\t}',
}, {
  '\t{',
  '\t\ta();',
  '\t}',
  '\t{',
  '\t\tc();',
  '\t}',
  '\t{',
  '\t\tb();',
  '\t}',
}, 'a slidable hunk: with indent-heuristic it lands one line higher')

-- The same slide with the braces indented by *six spaces* and the bodies
-- by a tab.  xdiff hard-codes a tab at eight columns regardless of
-- 'tabstop', so which of the two candidate lines counts as deeper is a
-- function of that constant and of nothing else.
pair('slidemix', {
  '      {',
  '\ta();',
  '      }',
  '      {',
  '\tb();',
  '      }',
}, {
  '      {',
  '\ta();',
  '      }',
  '      {',
  '\tc();',
  '      }',
  '      {',
  '\tb();',
  '      }',
}, 'the same slide with tab bodies and six-space braces: xget_indent(tab)=8')

-- diff_refine_inline_word_highlight merges two inline changes separated
-- by no more than diff_word_gap (5) bytes.  Three lines bracket the
-- bound at 4, 5 and 6 so an off-by-one on it changes an answer.
pair('wordgap', {
  'head AAA.. BBB tail',
  'head AAA... BBB tail',
  'head AAA.... BBB tail',
  'head AAA..... BBB tail',
  'head AAA   BBB tail',
  'head AAA     BBB tail',
  'head AAA      BBB tail',
  'head AAA1234BBB tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, {
  'head XXX.. YYY tail',
  'head XXX... YYY tail',
  'head XXX.... YYY tail',
  'head XXX..... YYY tail',
  'head XXX   YYY tail',
  'head XXX     YYY tail',
  'head XXX      YYY tail',
  'head XXX1234YYY tail',
  'shared 1',
  'shared 2',
  'shared 3',
}, 'inline changes separated by 3..6 non-word bytes; diff_word_gap is 5, and the last line\'s gap is *word* characters, which merge whatever their length')

-- ---------------------------------------------------------------------
-- The world a case runs in.
-- ---------------------------------------------------------------------

--- Every option any section touches, at a fixed value, so a case that
--- forgets to restore one cannot poison its neighbours.  'diffopt' is
--- reset to its default and then overridden per case.
local DEFAULTS = table.concat({
  'set nomore noswapfile nobackup nowritebackup noundofile undolevels=1000',
  'set report=9999 belloff=all shortmess=filnxtToOFS noshowmode nowritebackup',
  'set diffopt& diffexpr= patchexpr= diffanchors=',
  'set fileformat=unix fileformats=unix encoding=utf-8 nobinary noendofline',
  'set scrolloff=0 sidescrolloff=0 lines=24 columns=80 cmdheight=1 laststatus=0',
  'set ignorecase& smartcase& hidden& autoread& equalalways&',
  'set foldopen&vim foldclose& foldminlines=1 foldnestmax=20 fillchars&vim',
}, ' | ')

local function tc(keys)
  return vim.api.nvim_replace_termcodes(keys, true, true, true)
end

--- Back to Normal mode, one window, one buffer.  Buffers are wiped
--- rather than left around: a case that reads a stale diff buffer from
--- three sections above is a case whose answer is a function of the run.
local function teardown()
  pcall(vim.api.nvim_feedkeys, tc('<C-\\><C-N>'), 'ntx', false)
  quiet('silent! diffoff!')
  quiet('silent! tabonly!')
  quiet('silent! only!')
  quiet('silent! enew!')
  local keep = vim.api.nvim_get_current_buf()
  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    if b ~= keep then
      pcall(vim.api.nvim_buf_delete, b, { force = true, unload = false })
    end
  end
  quiet(DEFAULTS)
  quiet('silent! setlocal buftype=nofile bufhidden=wipe noswapfile nomodified')
end

local function setbuf(lines)
  local ok = pcall(vim.api.nvim_buf_set_lines, 0, 0, -1, true, lines)
  if not ok then
    -- The near-binary fixture: fall back to setline, which does not
    -- validate. If that fails too the case says so rather than aborting.
    pcall(vim.fn.deletebufline, '%', 1, '$')
    pcall(vim.fn.setline, 1, lines)
  end
  quiet('silent! setlocal nomodified')
end

--- Build a tabpage of `#bodies` diff'd windows, left to right in the
--- order given, under the 'diffopt' spelling `opts`, and leave the
--- cursor in window 1 at line 1.
local function world(bodies, opts, extra)
  teardown()
  if opts and opts ~= '' then
    local err = quiet('set diffopt=' .. opts)
    if err then
      emit('  !diffopt', esc(opts), esc(err))
    end
  end
  if extra and extra ~= '' then
    quiet('set ' .. extra)
  end
  quiet('silent! setlocal buftype=nofile bufhidden=hide noswapfile')
  setbuf(bodies[1])
  for i = 2, #bodies do
    quiet('silent! botright vsplit')
    quiet('silent! enew!')
    quiet('silent! setlocal buftype=nofile bufhidden=hide noswapfile')
    setbuf(bodies[i])
  end
  quiet('silent! windo diffthis')
  quiet('silent! diffupdate')
  pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
  pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
end

-- ---------------------------------------------------------------------
-- Reporting.
-- ---------------------------------------------------------------------

-- diff_hlID answers a highlight id.  Which *number* that is depends on
-- the order the default groups were registered, so the report carries a
-- letter looked up through hlID() instead: the letter survives a
-- renumbering, and an unexpected id is spelled out rather than hidden.
local HLCODE = { [0] = '.' }
local function hlcodes()
  for name, code in pairs({
    DiffAdd = 'A',
    DiffChange = 'C',
    DiffText = 'T',
    DiffDelete = 'D',
    DiffTextAdd = 'X',
  }) do
    HLCODE[vim.fn.hlID(name)] = code
  end
end

local function hlcode(id)
  return HLCODE[id] or ('?' .. tostring(id))
end

--- The closed folds of the current window, as ranges.  'foldmethod' is
--- diff in a diff window, so this is diff_infold answering from the
--- other side of the same block list.
local function foldinfo(n)
  local ranges, levels = {}, {}
  local l = 1
  while l <= n do
    levels[#levels + 1] = tostring(vim.fn.foldlevel(l))
    local start = vim.fn.foldclosed(l)
    if start == l then
      local stop = vim.fn.foldclosedend(l)
      ranges[#ranges + 1] = start .. '-' .. stop
      for i = l + 1, stop do
        levels[#levels + 1] = tostring(vim.fn.foldlevel(i))
      end
      l = stop + 1
    else
      l = l + 1
    end
  end
  return ranges, levels
end

--- `]c` / `[c` from a fixed start, recorded as the sequence of lines the
--- cursor visits.  `normal!` swallows the rest of the line, so it cannot
--- share an exec with anything.
local function traverse(keys, from, steps)
  pcall(vim.api.nvim_win_set_cursor, 0, { from, 0 })
  local seen = {}
  for _ = 1, steps do
    quiet('silent! normal! ' .. keys)
    seen[#seen + 1] = vim.fn.line('.')
  end
  return seen
end

local function rle(list)
  local out, prev, run = {}, nil, 0
  for _, v in ipairs(list) do
    local s = tostring(v)
    if s == prev then
      run = run + 1
    else
      if prev then
        out[#out + 1] = run > 1 and (prev .. '*' .. run) or prev
      end
      prev, run = s, 1
    end
  end
  if prev then
    out[#out + 1] = run > 1 and (prev .. '*' .. run) or prev
  end
  return table.concat(out, ',')
end

local SEEN = {}

--- The whole observable diff state of the current tabpage.  `inline`
--- adds the per-column scan, which only the inline sections want: it is
--- O(columns) per changed line and says nothing extra under
--- `inline:none`.
local function shot(label, inline)
  -- A label collision is invisible in a 20,000-line report and turns two
  -- cases into one; say so in the artifact rather than in a comment.
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1

  local nwin = vim.fn.winnr('$')
  local all = {}
  for w = 1, nwin do
    pcall(vim.fn.win_gotoid, vim.fn.win_getid(w))
    local n = vim.api.nvim_buf_line_count(0)
    local hl, fill, cols = {}, {}, {}
    for l = 1, n do
      local id = vim.fn.diff_hlID(l, 1)
      hl[#hl + 1] = hlcode(id)
      fill[#fill + 1] = vim.fn.diff_filler(l)
      if inline and (id ~= 0 and hlcode(id) ~= 'A' and hlcode(id) ~= 'D') then
        -- Scan the columns of a *changed* line only: an added or deleted
        -- line answers the same id at every column by construction, and
        -- the H row already carries it.  Run-length encoded rather than
        -- truncated: the `long` fixture's change is 900 bytes in, so a
        -- cap would have hidden the only thing that fixture asks about,
        -- and a 2,000-column scan compresses to three tokens.
        local width = math.min(#vim.fn.getline(l) + 1, 2100)
        local s = {}
        for c = 1, width do
          s[#s + 1] = hlcode(vim.fn.diff_hlID(l, c))
        end
        cols[#cols + 1] = l .. '=' .. rle(s)
      end
    end
    local ranges, levels = foldinfo(n)
    local fwd = traverse(']c', 1, 6)
    local back = traverse('[c', n, 6)
    emit(label, 'w' .. w, 'H', table.concat(hl))
    emit(label, 'w' .. w, 'F', rle(fill))
    emit(label, 'w' .. w, 'O', (#ranges == 0 and '-' or table.concat(ranges, ',')) .. ' lv=' .. rle(levels))
    emit(label, 'w' .. w, 'N', ']c=' .. table.concat(fwd, ',') .. ' [c=' .. table.concat(back, ','))
    if inline then
      emit(label, 'w' .. w, 'I', #cols == 0 and '-' or table.concat(cols, ' '))
    end
    all['w' .. w] = { h = hl, f = fill, o = ranges, lv = levels, n = { fwd, back }, i = cols }
  end
  struct(label, all)
  pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
end

--- The buffer text of every window, for the sections that mutate.
local function bufs(label)
  local nwin = vim.fn.winnr('$')
  local all = {}
  for w = 1, nwin do
    local lines = vim.api.nvim_buf_get_lines(vim.fn.winbufnr(w), 0, -1, false)
    emit(label, 'w' .. w, 'B', esc(table.concat(lines, '\n')))
    all['w' .. w] = lines
  end
  struct(label .. ' B', all)
end

--- One diff case: build the world, report it.
local function case(label, spec, opts, inline, extra)
  world(spec, opts, extra)
  shot(label, inline)
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

local function pairbodies(name)
  local p = PAIRS[name]
  return { p.a, p.b }
end

local ALLPAIRS = {}
for _, p in ipairs(PAIRS) do
  ALLPAIRS[#ALLPAIRS + 1] = p.name
end

-- ---------------------------------------------------------------------
-- s01 -- every fixture pair at the default 'diffopt'
-- ---------------------------------------------------------------------

section('s01-pairs', function()
  for _, name in ipairs(ALLPAIRS) do
    emit('-- ' .. name .. ': ' .. PAIRS[name].note)
    case('p1 ' .. name, pairbodies(name), '', true)
    -- and the same pair the other way round: the hunk computation is not
    -- symmetric (insert vs delete take different arms of process_hunk).
    case('p1r ' .. name, { PAIRS[name].b, PAIRS[name].a }, '', true)
  end
end)

-- ---------------------------------------------------------------------
-- s02 -- the flag half of the 'diffopt' matrix
-- ---------------------------------------------------------------------

local FLAGSETS = {
  'internal,filler',
  'internal,filler,closeoff',
  'internal',
  'filler',
  'internal,filler,horizontal',
  'internal,filler,vertical',
  'internal,filler,foldcolumn:0',
  'internal,filler,foldcolumn:5',
  'internal,filler,hiddenoff',
  'internal,filler,followwrap',
  'internal,filler,indent-heuristic',
  'internal,filler,algorithm:histogram,indent-heuristic,linematch:40,inline:char',
}

section('s02-flags', function()
  for _, opts in ipairs(FLAGSETS) do
    for _, name in ipairs({ 'ins', 'del', 'move', 'white', 'algo', 'near' }) do
      case('f2 ' .. tag(opts) .. ' ' .. name, pairbodies(name), opts, false)
    end
  end
  -- What the option layer itself does with a bad spelling.  diffopt_changed
  -- rejects the whole string, so 'diffopt' keeps its old value -- which is
  -- the answer, not the error.
  for _, bad in ipairs({
    'nosuchflag',
    'algorithm:nosuch',
    'context:',
    'context:abc',
    'linematch:',
    'inline:nosuch',
    'foldcolumn:-1',
    'internal,,filler',
    'internal,filler,',
  }) do
    teardown()
    local err = quiet('set diffopt=' .. bad)
    emit('f2bad ' .. tag(bad), esc(err or '-'), 'now=' .. esc(vim.o.diffopt))
    struct('f2bad ' .. tag(bad), { err = err, opt = vim.o.diffopt })
  end
end)

-- ---------------------------------------------------------------------
-- s03 -- algorithm x indent-heuristic
-- ---------------------------------------------------------------------

section('s03-algorithm', function()
  for _, algo in ipairs({ 'myers', 'minimal', 'patience', 'histogram' }) do
    for _, ih in ipairs({ '', ',indent-heuristic' }) do
      local opts = 'internal,filler,algorithm:' .. algo .. ih .. ',inline:none,linematch:0'
      for _, name in ipairs({ 'algo', 'indent', 'slide', 'slidemix', 'swap', 'ins', 'del', 'move', 'near', 'mb' }) do
        case('a3 ' .. algo .. tag(ih) .. ' ' .. name, pairbodies(name), opts, false)
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s04 -- the whitespace and case flags
-- ---------------------------------------------------------------------

local WHITE = {
  '',
  ',iwhite',
  ',iwhiteall',
  ',iwhiteeol',
  ',iblank',
  ',icase',
  ',iwhite,iblank',
  ',iwhiteall,icase',
  ',iwhite,iwhiteall,iwhiteeol,iblank,icase',
}

section('s04-white', function()
  for _, w in ipairs(WHITE) do
    for _, name in ipairs({ 'white', 'blank', 'bvsx', 'case', 'mbcase', 'crlf', 'same', 'ins' }) do
      case(
        'w4 ' .. tag(w) .. ' ' .. name,
        pairbodies(name),
        'internal,filler,inline:none,linematch:0' .. w,
        false
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s05 -- context:N, and therefore the folds
-- ---------------------------------------------------------------------

section('s05-context', function()
  for _, n in ipairs({ 0, 1, 2, 3, 6, 8, 20, 999 }) do
    for _, name in ipairs({ 'ins', 'del', 'move', 'near', 'blank' }) do
      case(
        'c5 ' .. n .. ' ' .. name,
        pairbodies(name),
        'internal,filler,context:' .. n .. ',inline:none,linematch:0',
        false
      )
    end
  end
  -- The fold layer proper: 'foldenable' off, and a manual fold made
  -- before diffthis, both of which diff_win_options has an opinion about.
  for _, extra in ipairs({ 'nofoldenable', 'foldlevel=1', 'foldminlines=99', 'foldnestmax=0' }) do
    case('c5x ' .. tag(extra), pairbodies('ins'), 'internal,filler,context:1', false, extra)
  end
end)

-- ---------------------------------------------------------------------
-- s06 -- linematch:N
-- ---------------------------------------------------------------------

section('s06-linematch', function()
  for _, n in ipairs({ 0, 1, 2, 5, 10, 40, 200 }) do
    for _, name in ipairs({ 'near', 'algo', 'indent', 'ins', 'del', 'move', 'mb' }) do
      case(
        'l6 ' .. n .. ' ' .. name,
        pairbodies(name),
        'internal,filler,linematch:' .. n .. ',inline:none',
        false
      )
    end
  end
  -- linematch is defined over at most three buffers; the three-way arm is
  -- a different code path in run_linematch_algorithm.
  for _, n in ipairs({ 0, 40 }) do
    case(
      'l6t ' .. n,
      { PAIRS.near.a, PAIRS.near.b, PAIRS.algo.b },
      'internal,filler,linematch:' .. n .. ',inline:none',
      false
    )
  end
end)

-- ---------------------------------------------------------------------
-- s07 -- inline:{none,simple,char,word}, per column
-- ---------------------------------------------------------------------

section('s07-inline', function()
  for _, mode in ipairs({ 'none', 'simple', 'char', 'word' }) do
    for _, name in ipairs({ 'near', 'case', 'mbcase', 'wordgap', 'white', 'mb', 'bin', 'long', 'ins', 'algo' }) do
      case(
        'i7 ' .. mode .. ' ' .. name,
        pairbodies(name),
        'internal,filler,inline:' .. mode .. ',linematch:0',
        true
      )
    end
  end
  -- inline over three windows, and with the whitespace flags on: the
  -- refine pass reads them again, separately from the hunk computation.
  for _, mode in ipairs({ 'char', 'word' }) do
    case(
      'i7t ' .. mode,
      { PAIRS.near.a, PAIRS.near.b, PAIRS.case.b },
      'internal,filler,inline:' .. mode,
      true
    )
    case(
      'i7w ' .. mode,
      pairbodies('white'),
      'internal,filler,iwhiteall,icase,inline:' .. mode,
      true
    )
  end
  -- 'icase' with the inline scan on.  This is the only way to reach
  -- diff_equal_char's *multibyte* arm: diff_write_buffer folds the whole
  -- line before xdiff sees it, so a pair differing only in case leaves no
  -- block for anything to compare.  A mutation on that arm measured NOT
  -- CAUGHT until this pass and `mbcase`'s content-differing lines existed.
  for _, mode in ipairs({ 'simple', 'char', 'word' }) do
    for _, name in ipairs({ 'mbcase', 'case', 'mb' }) do
      case(
        'i7c ' .. mode .. ' ' .. name,
        pairbodies(name),
        'internal,filler,icase,inline:' .. mode,
        true
      )
    end
  end
end)

-- ---------------------------------------------------------------------
-- s08 -- ]c / [c
-- ---------------------------------------------------------------------

section('s08-nav', function()
  for _, name in ipairs({ 'ins', 'del', 'move', 'near', 'same', 'empty' }) do
    world(pairbodies(name), 'internal,filler,inline:none', nil)
    local n = vim.api.nvim_buf_line_count(0)
    local rows = {}
    for from = 1, n do
      local f = traverse(']c', from, 1)[1]
      local b = traverse('[c', from, 1)[1]
      rows[#rows + 1] = from .. ':' .. f .. '/' .. b
    end
    emit('n8 ' .. name, 'each', table.concat(rows, ' '))
    struct('n8 ' .. name .. ' each', rows)
    -- counted forms, and the wrap-around message arm
    local counted = {}
    for _, keys in ipairs({ ']c', '2]c', '5]c', '99]c', '[c', '2[c', '9[c' }) do
      pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
      quiet('silent! normal! ' .. keys)
      counted[#counted + 1] = tag(keys) .. '=' .. vim.fn.line('.')
      pcall(vim.api.nvim_win_set_cursor, 0, { n, 0 })
      quiet('silent! normal! ' .. keys)
      counted[#counted + 1] = tag(keys) .. 'E=' .. vim.fn.line('.')
    end
    emit('n8 ' .. name, 'count', table.concat(counted, ' '))
    struct('n8 ' .. name .. ' count', counted)
  end
end)

-- ---------------------------------------------------------------------
-- s09 -- :diffget / :diffput / do / dp
-- ---------------------------------------------------------------------

local GETPUT = {
  { 'do', 'normal', 'do' },
  { 'dp', 'normal', 'dp' },
  { '2do', 'normal', '2do' },
  { 'get', 'ex', 'diffget' },
  { 'put', 'ex', 'diffput' },
  { 'getr', 'ex', '1,20diffget' },
  { 'putr', 'ex', '1,20diffput' },
  -- Ranges that start *inside* a block.  This is the only way to reach
  -- diffgetput's start_skip/end_skip clamp at all: a range beginning at
  -- line 1 makes start_skip zero and the whole clamp is dead code.  A
  -- mutation on it measured NOT CAUGHT until these six arrived.
  { 'get2', 'ex', '2diffget' },
  { 'put2', 'ex', '2diffput' },
  { 'getmid', 'ex', '2,3diffget' },
  { 'putmid', 'ex', '2,3diffput' },
  { 'getin', 'ex', '5,6diffget' },
  { 'putin', 'ex', '5,6diffput' },
  { 'getall', 'ex', '%diffget' },
  { 'putall', 'ex', '%diffput' },
  { 'get0', 'ex', '0diffget' },
  { 'getlast', 'ex', '$diffget' },
}

section('s09-getput', function()
  for _, name in ipairs({ 'ins', 'del', 'move', 'near', 'white', 'mb' }) do
    for _, gp in ipairs(GETPUT) do
      for _, at in ipairs({ 1, 5, 15 }) do
        local label = 'g9 ' .. name .. ' ' .. gp[1] .. ' ' .. at
        world(pairbodies(name), 'internal,filler,inline:none', nil)
        pcall(vim.api.nvim_win_set_cursor, 0, { math.min(at, vim.api.nvim_buf_line_count(0)), 0 })
        local err
        if gp[2] == 'normal' then
          err = quiet('silent! normal! ' .. gp[3])
        else
          err = quiet('silent! ' .. gp[3])
        end
        emit(label, '!', esc(err or '-'))
        bufs(label)
        shot(label, false)
      end
    end
  end
  -- three-way, where :diffget needs a buffer argument and errors without
  -- one (E101), and the argument itself can miss (E102/E103).
  world({ PAIRS.near.a, PAIRS.near.b, PAIRS.algo.b }, 'internal,filler', nil)
  local names = {}
  for w = 1, 3 do
    names[w] = vim.fn.winbufnr(w)
  end
  for _, spelling in ipairs({ 'diffget', 'diffput', 'diffget 999999', 'diffget nosuchbuffer' }) do
    world({ PAIRS.near.a, PAIRS.near.b, PAIRS.algo.b }, 'internal,filler', nil)
    pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
    local err = quiet('silent! ' .. spelling)
    emit('g9t ' .. tag(spelling), '!', esc(err or '-'))
    bufs('g9t ' .. tag(spelling))
  end
  -- and with an explicit, valid buffer handle
  world({ PAIRS.near.a, PAIRS.near.b, PAIRS.algo.b }, 'internal,filler', nil)
  local target = vim.fn.winbufnr(3)
  pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
  local err = quiet('silent! diffget ' .. target)
  emit('g9t byhandle', '!', esc(err or '-'))
  bufs('g9t byhandle')
end)

-- ---------------------------------------------------------------------
-- s10 -- diffopt=external: the shell-out path
-- ---------------------------------------------------------------------

section('s10-external', function()
  local have = vim.fn.executable('diff')
  emit('x10 have-diff', tostring(have))
  for _, w in ipairs({ '', ',iwhite', ',iwhiteall', ',iwhiteeol', ',iblank', ',icase' }) do
    for _, name in ipairs({ 'ins', 'del', 'move', 'white', 'blank', 'case', 'same', 'mb' }) do
      case('x10 ' .. tag(w) .. ' ' .. name, pairbodies(name), 'filler' .. w, false)
    end
  end
  -- external with context and folds, which the *parser* has to recover
  -- from `diff`'s ed-style output rather than from a hunk callback.
  for _, n in ipairs({ 0, 3, 20 }) do
    case('x10c ' .. n, pairbodies('near'), 'filler,context:' .. n, false)
  end
  -- 'diffopt' naming both: `internal` wins unless the internal diff fails.
  case('x10 both', pairbodies('ins'), 'internal,filler', false)
end)

-- ---------------------------------------------------------------------
-- s11 -- 'diffexpr'
-- ---------------------------------------------------------------------

section('s11-diffexpr', function()
  -- 'diffexpr' is an *expression*, not a command: eval_diff hands it to
  -- eval1() and `call Fn()` there is E121 "Undefined variable: call".
  -- The whole first draft of this section measured nothing because of
  -- that, and it looked like a working sweep -- every case reported a
  -- clean diff, which is what an unset diffexpr also reports.
  --
  -- What the callback is *handed*: v:fname_in and friends name temp
  -- files whose basenames differ per run by construction, so the answer
  -- recorded is whether each was set and whether the named file exists,
  -- never the path.  The *number* of calls is part of the answer too --
  -- check_external_diff probes with a two-line fixture before the real
  -- diff runs.
  quiet([[
    let g:dxsaw = []
    function! DxProbe() abort
      call add(g:dxsaw, printf('in=%d/%d new=%d/%d out=%d',
            \ v:fname_in != '', filereadable(v:fname_in),
            \ v:fname_new != '', filereadable(v:fname_new),
            \ v:fname_out != ''))
      call writefile(['1c1'], v:fname_out)
      return 0
    endfunction
    function! DxReal() abort
      let l:a = readfile(v:fname_in)
      let l:b = readfile(v:fname_new)
      let l:out = []
      let l:n = max([len(l:a), len(l:b)])
      for l:i in range(l:n)
        let l:x = get(l:a, l:i, v:null)
        let l:y = get(l:b, l:i, v:null)
        if l:x is v:null
          call add(l:out, printf('%da%d', len(l:a), l:i + 1))
        elseif l:y is v:null
          call add(l:out, printf('%dd%d', l:i + 1, len(l:b)))
        elseif l:x !=# l:y
          call add(l:out, printf('%dc%d', l:i + 1, l:i + 1))
        endif
      endfor
      call writefile(l:out, v:fname_out)
      return 0
    endfunction
    function! DxUnified() abort
      call writefile(['@@ -1,2 +1,3 @@', ' alpha one', '-beta two',
            \ '+beta TWO', '+beta extra'], v:fname_out)
      return 0
    endfunction
    function! DxEmpty() abort
      call writefile([], v:fname_out)
      return 0
    endfunction
    function! DxGarbage() abort
      call writefile(['not a diff at all', '@@@@', '0a0', '99c99'], v:fname_out)
      return 0
    endfunction
    function! DxNothing() abort
      return 0
    endfunction
    function! DxThrow() abort
      throw 'dx-failed'
    endfunction
  ]])
  for _, fn in ipairs({
    'DxProbe()',
    'DxReal()',
    'DxUnified()',
    'DxEmpty()',
    'DxGarbage()',
    'DxNothing()',
    'DxThrow()',
  }) do
    quiet('let g:dxsaw = []')
    for _, name in ipairs({ 'near', 'ins' }) do
      quiet('let g:dxsaw = []')
      world(pairbodies(name), 'filler', 'diffexpr=' .. fn)
      local label = 'e11 ' .. tag(fn) .. ' ' .. name
      -- `diffupdate!` and not `diffupdate`: check_external_diff caches
      -- its verdict in diff_a_works and the plain form does not re-ask,
      -- so the E97 an unusable 'diffexpr' earns is only visible with the
      -- bang.  Four of the seven callbacks here fail that probe -- it
      -- feeds them a two-line fixture and demands literally `1c1` or
      -- `@@ -1 +1 @@` back -- and without this line the section recorded
      -- "no differences" for all four, which is what a *working* diff of
      -- two identical buffers also records.
      local uerr = quiet('diffupdate!')
      emit(label, '!', esc(uerr or '-'))
      emit(label, 'dex=' .. esc(vim.o.diffexpr), 'calls=' .. #(vim.g.dxsaw or {}))
      emit(label, 'saw=' .. esc(table.concat(vim.g.dxsaw or {}, ' ')))
      shot(label, false)
      struct(label .. ' saw', vim.g.dxsaw)
    end
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s12 -- :diffpatch and 'patchexpr'
-- ---------------------------------------------------------------------

section('s12-patch', function()
  emit('P12 have-patch', tostring(vim.fn.executable('patch')))
  local pfile = work .. '/files/one.patch'
  vim.fn.writefile({
    '--- a/one',
    '+++ b/one',
    '@@ -1,5 +1,6 @@',
    ' alpha one',
    '-beta two',
    '+beta TWO',
    '+beta extra',
    ' gamma three',
    ' delta four',
    ' epsilon five',
  }, pfile)
  local bad = work .. '/files/bad.patch'
  vim.fn.writefile({ 'this is not a patch' }, bad)

  for _, spec in ipairs({ { 'ok', pfile }, { 'bad', bad }, { 'missing', work .. '/files/nope.patch' } }) do
    teardown()
    quiet('silent! enew!')
    setbuf(PAIRS.near.a)
    local err = quiet('silent! diffpatch ' .. vim.fn.fnameescape(spec[2]))
    emit('P12 ' .. spec[1], '!', esc(err or '-'), 'wins=' .. vim.fn.winnr('$'))
    bufs('P12 ' .. spec[1])
    -- The patched buffer is named after a temp file, whose basename is a
    -- monotonic counter over the whole run.  scrub() folds the whole path
    -- to <TMPFILE>; taking `:t` first would have left the counter behind
    -- and made this line a function of every temp file above it.
    local names = {}
    for w = 1, vim.fn.winnr('$') do
      names[#names + 1] = scrub(vim.fn.bufname(vim.fn.winbufnr(w)))
    end
    emit('P12 ' .. spec[1], 'names', esc(table.concat(names, ',')))
  end

  -- 'patchexpr', which like 'diffexpr' is an *expression*.
  quiet([[
    function! PxCopy() abort
      call writefile(readfile(v:fname_in), v:fname_out)
      return 0
    endfunction
    function! PxSaw() abort
      let g:pxsaw = printf('in=%d diff=%d out=%d',
            \ filereadable(v:fname_in), filereadable(v:fname_diff), v:fname_out != '')
      call writefile(readfile(v:fname_in) + ['patched'], v:fname_out)
      return 0
    endfunction
    function! PxNothing() abort
      return 0
    endfunction
  ]])
  for _, fn in ipairs({ 'PxCopy()', 'PxSaw()', 'PxNothing()' }) do
    teardown()
    quiet('let g:pxsaw = "-"')
    quiet('silent! enew!')
    setbuf(PAIRS.near.a)
    quiet('set patchexpr=' .. fn)
    local err = quiet('silent! diffpatch ' .. vim.fn.fnameescape(pfile))
    emit(
      'P12x ' .. tag(fn),
      '!',
      esc(err or '-'),
      'wins=' .. vim.fn.winnr('$'),
      'saw=' .. esc(tostring(vim.g.pxsaw))
    )
    bufs('P12x ' .. tag(fn))
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s13 -- 'diffanchors'
-- ---------------------------------------------------------------------

-- 'diffanchors' is a comma-separated list of *addresses*, re-evaluated
-- in each diff buffer, so a pattern or a relative address resolves to a
-- different line per buffer -- which is the whole point of the feature
-- and the reason `$` and `/pat/` are in the list below.
local ANCHORS = {
  { 'none', '' },
  { 'one', '4' },
  { 'six', '6' },
  { 'two', '3,6' },
  { 'mark', "'a,'b" },
  { 'pat', '/f/' },
  { 'rel', '.+2' },
  { 'dollar', '$' },
  { 'zero', '0' },
  { 'past', '9999' },
  { 'empty-el', '3,,6' },
  { 'trailing', '3,' },
  { 'leading', ',3' },
  { 'garbage', 'nosuch' },
}

section('s13-anchor', function()
  for _, spec in ipairs(ANCHORS) do
    for _, flag in ipairs({ 'internal,filler', 'internal,filler,anchor' }) do
      for _, name in ipairs({ 'anch', 'near', 'move' }) do
        local label = 'A13 ' .. spec[1] .. ' ' .. tag(flag) .. ' ' .. name
        world(pairbodies(name), flag, nil)
        -- the marks the `'a,'b` spelling needs, in every diff buffer
        for w = 1, vim.fn.winnr('$') do
          pcall(vim.fn.win_gotoid, vim.fn.win_getid(w))
          pcall(vim.api.nvim_buf_set_mark, 0, 'a', 2, 0, {})
          pcall(vim.api.nvim_buf_set_mark, 0, 'b', 6, 0, {})
        end
        pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
        local err = quiet('set diffanchors=' .. vim.fn.escape(spec[2], ' \\'))
        local err2 = quiet('diffupdate')
        emit(label, '!', esc(err or '-'), esc(err2 or '-'), 'dia=' .. esc(vim.o.diffanchors))
        shot(label, false)
      end
    end
  end
  -- buffer-local 'diffanchors' overrides the global one, and only in the
  -- buffer that sets it: the two windows then split at different lines,
  -- which is the arm parse_diffanchors takes per buffer.
  for _, l2 in ipairs({ '5', '2', '' }) do
    world(pairbodies('anch'), 'internal,filler,anchor', nil)
    quiet('set diffanchors=6')
    pcall(vim.fn.win_gotoid, vim.fn.win_getid(2))
    quiet('setlocal diffanchors=' .. l2)
    pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
    quiet('diffupdate')
    emit('A13 local ' .. (l2 == '' and 'empty' or l2), 'g=' .. esc(vim.o.diffanchors), 'l=' .. esc(vim.bo.diffanchors))
    shot('A13 local ' .. (l2 == '' and 'empty' or l2), false)
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s14 -- diff_set_topline
-- ---------------------------------------------------------------------

section('s14-topline', function()
  for _, name in ipairs({ 'ins', 'del', 'move', 'near' }) do
    for _, opts in ipairs({ 'internal,filler', 'internal', 'internal,filler,context:0' }) do
      world(pairbodies(name), opts, nil)
      quiet('silent! windo setlocal scrollbind')
      pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
      local rows = {}
      for _, l in ipairs({ 1, 3, 6, 10, 14, 20, 30 }) do
        if l <= vim.api.nvim_buf_line_count(0) then
          pcall(vim.api.nvim_win_set_cursor, 0, { l, 0 })
          quiet('silent! normal! zt')
          quiet('silent! syncbind')
          local t = {}
          for w = 1, vim.fn.winnr('$') do
            pcall(vim.fn.win_gotoid, vim.fn.win_getid(w))
            local view = vim.fn.winsaveview()
            t[#t + 1] = ('w%d=%d/%d'):format(w, view.topline, view.topfill or 0)
          end
          pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
          rows[#rows + 1] = 'zt' .. l .. ':' .. table.concat(t, ',')
        end
      end
      emit('t14 ' .. name .. ' ' .. tag(opts), table.concat(rows, ' '))
      struct('t14 ' .. name .. ' ' .. tag(opts), rows)
    end
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s15 -- diff_mark_adjust: edits without a :diffupdate
-- ---------------------------------------------------------------------

local EDITS = {
  { 'ins-top', 'call append(0, ["NEW TOP"])' },
  { 'ins-mid', 'call append(5, ["NEW MID"])' },
  { 'del-top', '1delete _' },
  { 'del-mid', '5,7delete _' },
  { 'chg-mid', 'call setline(5, "CHANGED")' },
  { 'app-end', 'call append(line("$"), ["NEW END"])' },
  { 'wipe-all', '%delete _' },
  { 'move', '3move 8' },
  { 'undo-after', 'call append(3, ["X"]) | undo' },
  -- diff_mark_adjust_tp only calls diff_check_unchanged when an edit
  -- *grows or shrinks a block it is inside*, and the trim only does
  -- anything when the two sides then agree at an edge.  The four below
  -- insert exactly the partner's text at the top and bottom of `near`'s
  -- two changed blocks, which is the only shape in this sweep that
  -- reaches the forward and backward trims at all -- a mutation on the
  -- forward one measured NOT CAUGHT until they arrived.
  { 'eq-top', 'call append(0, ["alpha one"])' },
  { 'eq-top2', 'call append(0, ["alpha ONE"])' },
  { 'eq-bot', 'call append(2, ["beta two"])' },
  { 'eq-bot2', 'call append(2, ["beta two changed"])' },
  { 'eq-blk2', 'call append(3, ["delta four"])' },
  { 'eq-in', 'call append(1, ["beta two"])' },
}

section('s15-adjust', function()
  -- BOTH spellings of 'diffopt', and the *external* one is the load-bearing
  -- half.  diff_mark_adjust_tp opens with `if diff_internal() { tp_diff_invalid
  -- = true }`, so under the internal diff every block adjustment it then
  -- performs is thrown away by the full recompute that the next diff_hlID()
  -- triggers -- the section measured the recompute and not the adjustment,
  -- and a mutation on diff_check_unchanged measured NOT CAUGHT because of it.
  -- With `diffopt=filler` (external) the adjusted block list is what the
  -- reader sees.
  for _, name in ipairs({ 'ins', 'near' }) do
    for _, opts in ipairs({ 'internal,filler,inline:none', 'filler' }) do
    for _, e in ipairs(EDITS) do
      for _, which in ipairs({ 1, 2 }) do
        -- `e<n>` and not `w<n>`: the report's own rows are keyed w1/w2,
        -- and a label ending in `w1` produced `... w1 w1 H`, which reads
        -- as a malformed row rather than as "edited window 1".
        local label = 'd15 ' .. name .. ' ' .. tag(opts) .. ' ' .. tag(e[1]) .. ' e' .. which
        world(pairbodies(name), opts, nil)
        pcall(vim.fn.win_gotoid, vim.fn.win_getid(which))
        quiet('silent! setlocal modifiable')
        local err = quiet('silent! ' .. e[2])
        pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
        -- deliberately NO :diffupdate: this is diff_mark_adjust and
        -- diff_update_line answering on their own.
        emit(label, '!', esc(err or '-'))
        shot(label, false)
        -- and then the recompute, which must agree with a fresh world
        quiet('silent! diffupdate')
        shot(label .. ' upd', false)
      end
    end
    end
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s16 -- the lifecycle: diffthis / diffsplit / diffoff
-- ---------------------------------------------------------------------

local WINOPTS = { 'diff', 'wrap', 'foldmethod', 'foldcolumn', 'foldenable', 'foldlevel', 'scrollbind', 'cursorbind', 'foldopen' }

local function winopts(label)
  local rows = {}
  for w = 1, vim.fn.winnr('$') do
    pcall(vim.fn.win_gotoid, vim.fn.win_getid(w))
    local t = {}
    for _, o in ipairs(WINOPTS) do
      -- `ok and v or 'ERR'` is wrong here and read as coverage for a
      -- whole run: every *false* boolean reported ERR, so the row that
      -- proves diff_win_options turned 'wrap' off said the option could
      -- not be read.  Branch on `ok` explicitly.
      local ok, v = pcall(vim.api.nvim_get_option_value, o, { win = 0 })
      t[#t + 1] = o .. '=' .. (ok and tostring(v) or 'ERR')
    end
    rows[#rows + 1] = 'w' .. w .. ':' .. table.concat(t, ',')
  end
  pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
  emit(label, 'W', esc(table.concat(rows, ' ')))
  struct(label .. ' W', rows)
end

section('s16-lifecycle', function()
  -- the option save/restore round trip
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  quiet('silent! setlocal wrap foldmethod=marker foldcolumn=4 foldlevel=3 nofoldenable')
  winopts('L16 before')
  quiet('silent! botright vsplit')
  quiet('silent! enew!')
  setbuf(PAIRS.near.b)
  quiet('silent! windo diffthis')
  pcall(vim.fn.win_gotoid, vim.fn.win_getid(1))
  winopts('L16 on')
  quiet('silent! windo diffoff')
  winopts('L16 off')

  -- diffoff! from one window turns off the whole tab
  world(pairbodies('near'), 'internal,filler', nil)
  quiet('silent! diffoff!')
  winopts('L16 offbang')
  shot('L16 offbang', false)

  -- :diffsplit of a real file
  local f = work .. '/files/split.txt'
  vim.fn.writefile(PAIRS.near.b, f)
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  local err = quiet('silent! diffsplit ' .. vim.fn.fnameescape(f))
  emit('L16 split', '!', esc(err or '-'), 'wins=' .. vim.fn.winnr('$'))
  winopts('L16 split')
  shot('L16 split', false)
  quiet('silent! vertical diffsplit ' .. vim.fn.fnameescape(f))
  emit('L16 vsplit', 'wins=' .. vim.fn.winnr('$'))

  -- more diff buffers than DB_COUNT allows
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  local errs = {}
  for i = 2, 10 do
    quiet('silent! botright vsplit')
    quiet('silent! enew!')
    setbuf({ 'buffer ' .. i, 'shared', 'shared', 'shared' })
    -- No `silent!`: the ninth :diffthis is the E96 arm, and `silent!`
    -- suppresses the exception as well as the message, so the whole
    -- DB_COUNT limit read as "no error" for a run.
    errs[#errs + 1] = i .. ':' .. esc(quiet('diffthis') or '-')
  end
  emit('L16 dbcount', table.concat(errs, ' '), 'wins=' .. vim.fn.winnr('$'))
  struct('L16 dbcount', errs)

  -- the same buffer diff'd in two windows, and a diff window whose
  -- buffer is then hidden (diff_buf_delete / diff_buf_adjust).
  world(pairbodies('near'), 'internal,filler', nil)
  quiet('silent! wincmd w')
  quiet('silent! enew!')
  emit('L16 replaced', 'wins=' .. vim.fn.winnr('$'))
  shot('L16 replaced', false)
  teardown()
end)

-- ---------------------------------------------------------------------
-- s17 -- diff_filler() / diff_hlID() outside a diff, and at edges
-- ---------------------------------------------------------------------

section('s17-funcs', function()
  world(pairbodies('near'), 'internal,filler', nil)
  local rows = {}
  for _, l in ipairs({ -5, -1, 0, 1, 2, 8, 9, 99, 100000 }) do
    -- 7..9 is where line 1's change actually is ("alpha one" vs
    -- "alpha ONE"), so these are the columns that can answer T rather
    -- than C.  The first draft sampled -1/0/1/2/200 only and every row
    -- said C, which is a section answering one thing.
    for _, c in ipairs({ -1, 0, 1, 2, 6, 7, 8, 9, 10, 200 }) do
      local ok, v = pcall(vim.fn.diff_hlID, l, c)
      local ok2, v2 = pcall(vim.fn.diff_filler, l)
      rows[#rows + 1] = ('%d/%d=%s,%s'):format(l, c, ok and hlcode(v) or 'E', ok2 and tostring(v2) or 'E')
    end
  end
  emit('F17 edges', table.concat(rows, ' '))
  struct('F17 edges', rows)
  -- string and special arguments
  local weird = {}
  for _, a in ipairs({ '.', '$', "'a", 'nosuch', '1.9', '-0' }) do
    local ok, v = pcall(vim.fn.diff_hlID, a, 1)
    local ok2, v2 = pcall(vim.fn.diff_filler, a)
    weird[#weird + 1] = esc(a) .. '=' .. (ok and hlcode(v) or errtext(v)) .. ',' .. (ok2 and tostring(v2) or errtext(v2))
  end
  emit('F17 weird', esc(table.concat(weird, ' ')))
  -- outside diff mode entirely
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  local out = {}
  for _, l in ipairs({ 1, 2, 8 }) do
    out[#out + 1] = l .. '=' .. hlcode(vim.fn.diff_hlID(l, 1)) .. ',' .. vim.fn.diff_filler(l)
  end
  emit('F17 nodiff', table.concat(out, ' '), 'infold=' .. tostring(vim.fn.foldlevel(1)))
  -- the cache: diff_hlID keeps a static answer keyed on lnum/tick/flags,
  -- so asking the same line twice, then changing 'diffopt', then asking
  -- again is the whole of that invalidation rule.
  world(pairbodies('near'), 'internal,filler,inline:simple', nil)
  local seq = {}
  local function ask(l, c)
    seq[#seq + 1] = ('%d/%d=%s'):format(l, c, hlcode(vim.fn.diff_hlID(l, c)))
  end
  ask(1, 1)
  ask(1, 8)
  ask(2, 8)
  ask(1, 8)
  quiet('set diffopt=internal,filler,inline:char')
  ask(1, 8)
  quiet('call setline(1, "alpha one")')
  ask(1, 8)
  quiet('diffupdate')
  ask(1, 8)
  quiet('set diffopt=internal,filler,inline:none')
  ask(1, 8)
  emit('F17 cache', table.concat(seq, ','))
  struct('F17 cache', seq)
  teardown()
end)

-- ---------------------------------------------------------------------
-- s18 -- `vim.diff()`, the OTHER consumer of xdiff/
--
-- lua/xdiff.rs calls xdl_diff() directly, and it is the reason half of
-- xemit.rs and xutils.rs exists: the in-editor path sets `ctxlen = 0`
-- and a `hunk_func`, so xdl_emit_diff, xdl_emit_diffrec,
-- xdl_emit_hunk_hdr, xdl_format_hunk_hdr and xdl_num_out are *never*
-- reached from :diffupdate.  Neither is XDF_IGNORE_CR_AT_EOL, which
-- diff_file_internal does not set and 'diffopt' has no spelling for --
-- a mutation on ends_with_optional_cr measured NOT CAUGHT for exactly
-- that reason, and the tell was that the crlf/iwhiteeol cases already
-- answered "equal" through a different flag.
--
-- Everything here is a pure function of its two string arguments, so it
-- is the cheapest coverage in the sweep.
-- ---------------------------------------------------------------------

local DIFFOPTS = {
  { 'default', {} },
  { 'indices', { result_type = 'indices' } },
  { 'ctx0', { ctxlen = 0 } },
  { 'ctx1', { ctxlen = 1 } },
  { 'ctx5', { ctxlen = 5 } },
  { 'inter0', { ctxlen = 3, interhunkctxlen = 0 } },
  { 'inter5', { ctxlen = 3, interhunkctxlen = 5 } },
  { 'myers', { algorithm = 'myers' } },
  { 'minimal', { algorithm = 'minimal' } },
  { 'patience', { algorithm = 'patience' } },
  { 'histogram', { algorithm = 'histogram' } },
  { 'iw', { ignore_whitespace = true } },
  { 'iwc', { ignore_whitespace_change = true } },
  { 'iweol', { ignore_whitespace_change_at_eol = true } },
  { 'icr', { ignore_cr_at_eol = true } },
  { 'iblank', { ignore_blank_lines = true } },
  { 'indent', { indent_heuristic = true } },
  { 'lm-true', { result_type = 'indices', linematch = true } },
  { 'lm-10', { result_type = 'indices', linematch = 10 } },
  { 'lm-40', { result_type = 'indices', linematch = 40 } },
  { 'all', {
    result_type = 'indices',
    algorithm = 'histogram',
    ctxlen = 2,
    interhunkctxlen = 2,
    ignore_whitespace_change = true,
    ignore_cr_at_eol = true,
    ignore_blank_lines = true,
    indent_heuristic = true,
    linematch = 40,
  } },
}

section('s18-vimdiff', function()
  for _, name in ipairs(ALLPAIRS) do
    local a = table.concat(PAIRS[name].a, '\n') .. '\n'
    local b = table.concat(PAIRS[name].b, '\n') .. '\n'
    for _, spec in ipairs(DIFFOPTS) do
      local label = 'v18 ' .. spec[1] .. ' ' .. name
      local ok, res = pcall(vim.diff, a, b, spec[2])
      if not ok then
        emit(label, '!', esc(errtext(res)))
        struct(label, { err = errtext(res) })
      elseif type(res) == 'string' then
        emit(label, 'U', esc(res))
        struct(label, res)
      else
        emit(label, 'I', esc(vim.inspect(res):gsub('%s+', ' ')))
        struct(label, res)
      end
    end
  end
  -- the on_hunk callback, which is a third emit mode (kNluaXdiffModeOnHunkCB)
  for _, name in ipairs({ 'ins', 'del', 'near', 'crlf', 'blank', 'slide' }) do
    local a = table.concat(PAIRS[name].a, '\n') .. '\n'
    local b = table.concat(PAIRS[name].b, '\n') .. '\n'
    for _, opts in ipairs({
      { ctxlen = 3 },
      { ctxlen = 0, ignore_cr_at_eol = true },
      { ctxlen = 1, ignore_blank_lines = true, linematch = 40 },
    }) do
      local hunks = {}
      local o = vim.tbl_extend('force', opts, {
        on_hunk = function(sa, ca, sb, cb)
          hunks[#hunks + 1] = ('%d,%d/%d,%d'):format(sa, ca, sb, cb)
        end,
      })
      local ok, res = pcall(vim.diff, a, b, o)
      emit(
        'v18h ' .. name .. ' ' .. tag(vim.inspect(opts):gsub('%s+', '')),
        ok and table.concat(hunks, ' ') or esc(errtext(res)),
        'ret=' .. tostring(ok and (res == nil and 'nil' or type(res)) or 'err')
      )
      struct('v18h ' .. name .. ' ' .. tag(vim.inspect(opts):gsub('%s+', '')), hunks)
    end
  end
  -- the rejection arms of the option converter
  for _, spec in ipairs({
    { 'bad-algo', { algorithm = 'nosuch' } },
    { 'bad-result', { result_type = 'nosuch' } },
    { 'bad-linematch', { linematch = 'yes' } },
    { 'bad-onhunk', { on_hunk = 42 } },
    { 'bad-ctxlen', { ctxlen = -1 } },
    { 'lm-unified', { linematch = 10 } },
    { 'unknown-key', { nosuchkey = true } },
  }) do
    local ok, res = pcall(vim.diff, 'a\nb\n', 'a\nc\n', spec[2])
    emit('v18e ' .. spec[1], ok and ('ok ' .. esc(tostring(res))) or esc(errtext(res)))
  end
  -- and the argument arms
  for _, spec in ipairs({
    { 'empty-both', '', '' },
    { 'empty-a', '', 'x\n' },
    { 'empty-b', 'x\n', '' },
    { 'no-final-nl', 'a\nb', 'a\nc' },
    { 'nul', 'a\0b\n', 'a\0c\n' },
    { 'invalid-utf8', 'a\255b\n', 'a\254c\n' },
  }) do
    local ok, res = pcall(vim.diff, spec[2], spec[3], {})
    emit('v18a ' .. spec[1], ok and esc(tostring(res)) or esc(errtext(res)))
    struct('v18a ' .. spec[1], ok and res or { err = errtext(res) })
  end
  teardown()
end)

-- ---------------------------------------------------------------------
-- s90 -- the message and error arms, UNCAPTURED
--
-- Everything above runs through nvim_exec2 with output off, so the
-- .stderr artifact would be empty and an artifact that never carries
-- signal gates nothing.  This section deliberately lets nvim write.
-- ---------------------------------------------------------------------

--- Run one command so that whatever it says lands on the prompt.  A bare
--- `pcall(nvim_command, ...)` turns an emsg into a *Lua* error and the
--- message never reaches stderr at all -- the first draft of s90
--- produced two lines of shell noise and nothing else, which is an
--- artifact that gates nothing.  `silent!` is equally wrong: it
--- suppresses the message and the exception together.  try/echo prints
--- the exception text where the emsg would have gone.
local function loud(cmd)
  pcall(
    vim.api.nvim_exec2,
    table.concat({ 'try', cmd, 'catch', 'echomsg v:exception', 'endtry' }, '\n'),
    { output = false }
  )
end

section('s90-msg', function()
  emit('-- s90 writes to stderr, not here')
  teardown()
  loud('echo "-- s90 begin"')

  -- E99/E100: not in diff mode / no other buffer in diff mode
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  loud('diffget')
  loud('diffput')
  loud('diffthis')
  loud('diffget')
  loud('normal! do')
  loud('normal! ]c')

  -- E101: more than two buffers in diff mode
  world({ PAIRS.near.a, PAIRS.near.b, PAIRS.algo.b }, 'internal,filler', nil)
  loud('diffget')
  loud('diffput')
  -- E102 / E103: the named buffer
  loud('diffget nosuchbuffer')
  loud('diffput nosuchbuffer')
  loud('diffget 999999')

  -- ]c past the last change, [c before the first
  world(pairbodies('ins'), 'internal,filler', nil)
  pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
  loud('normal! 99]c')
  loud('normal! 99[c')
  loud('normal! ]c')
  loud('normal! [c')

  -- the E97 arm: a 'diffexpr' naming a function that does not exist
  world(pairbodies('near'), 'filler', 'diffexpr=DxNoSuchFunction()')
  loud('diffupdate')
  loud('set diffexpr=')

  -- :diffpatch failures
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  loud('diffpatch ' .. work .. '/files/nope.patch')
  loud('diffpatch')

  -- 'diffanchors' rejections, including the MAX_DIFF_ANCHORS overflow
  world(pairbodies('near'), 'internal,filler,anchor', nil)
  for _, bad in ipairs({
    ',5',
    '3,',
    '1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21',
    'nosuch',
    '9999',
  }) do
    loud('set diffanchors=' .. vim.fn.escape(bad, ' \\'))
    loud('diffupdate')
  end
  loud('set diffanchors=')

  -- a bad 'diffopt', which emsgs through the option layer
  loud('set diffopt=nosuchflag')
  loud('set diffopt=algorithm:nosuch')
  loud('set diffopt=context:abc')
  loud('set diffopt&')

  -- the report path: 'report' at 0 makes :diffget/:diffput say how many
  -- lines moved, which is the only view of msg_* for this subsystem.
  world(pairbodies('del'), 'internal,filler', nil)
  loud('set report=0')
  pcall(vim.api.nvim_win_set_cursor, 0, { 14, 0 })
  loud('diffget')
  loud('%diffput')
  loud('1,20diffget')
  loud('set report=9999')

  -- and the E96 arm, which needs nine diff buffers
  teardown()
  quiet('silent! enew!')
  setbuf(PAIRS.near.a)
  for i = 2, 10 do
    quiet('silent! botright vsplit')
    quiet('silent! enew!')
    setbuf({ 'buffer ' .. i, 'shared', 'shared' })
    loud('diffthis')
  end

  loud('echo "-- s90 end"')
  teardown()
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 fileencoding= fileencodings= isprint=@,161-255 ambiwidth=single')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd& winminwidth=1')
quiet('set undolevels=1000 undofile& hidden nowritebackup nobackup')
quiet('language C')
quiet('silent! cd ' .. vim.fn.fnameescape(work .. '/files'))
quiet(DEFAULTS)

hlcodes()
emit('hl legend', 'A=' .. vim.fn.hlID('DiffAdd'), 'C=' .. vim.fn.hlID('DiffChange'), 'T=' .. vim.fn.hlID('DiffText'), 'D=' .. vim.fn.hlID('DiffDelete'), 'X=' .. vim.fn.hlID('DiffTextAdd'))
emit('default diffopt', esc(vim.o.diffopt))
emit('tools', 'diff=' .. vim.fn.executable('diff'), 'patch=' .. vim.fn.executable('patch'), 'sh=' .. esc(vim.o.shell))

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
      -- DIFFSWEEP_TRACE writes into the .stderr artifact and must be off
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
