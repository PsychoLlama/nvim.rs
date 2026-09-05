-- Driver for the multibyte/string differential sweep; see
-- utfsweep.sh.
--
-- Covers mbyte.rs (4,546 lines) and strings.rs (3,975), the two deepest
-- leaves in the tree (165 and 113 files fan in) and the two that no
-- existing differential reaches.  A regression in either surfaces as
-- *message text* in four other sweeps rather than as anything readable,
-- which is why this one asks them directly.
--
--   s01 the index family -- charidx / byteidx / byteidxcomp /
--       strgetchar / strcharpart / strpart / utf16idx, over every
--       corpus string at every index including the out-of-range ones
--   s02 the width family -- strchars / strcharlen / strdisplaywidth /
--       strwidth / strlen / strutf16len, under BOTH 'ambiwidth' values
--   s03 codepoint conversion -- nr2char / char2nr / list2str / str2list
--       / str2nr, the utf8 flag argument, and the edge codepoints
--   s04 setcellwidths() / getcellwidths(), and the widths they move
--   s05 THE TABLES -- charclass(), the cell width, the case folds and
--       printability, scanned over the codepoint space and run-length
--       encoded.  This is the only front door to utf_class_tab (399
--       lines) and to the utf_fold / utf_toupper / utf_tolower tables.
--   s06 the escape family -- keytrans / escape / shellescape /
--       fnameescape / tr / strtrans / trim / substitute / matchstr /
--       stridx / strridx / toupper / tolower / strrep
--   s07 printf()'s WHOLE format matrix.  vim_vsnprintf_typval is 1,297
--       lines and nothing in the tree tests it: every conversion, flag,
--       width, precision, `*` argument, `%<n>$` positional argument,
--       length modifier, float edge value, and multibyte string under a
--       width and a precision.
--   s08 iconv() and `:e ++enc=` / `:w ++enc=` round trips
--   s09 the Lua leaves -- vim.str_utfindex, vim.str_byteindex,
--       vim.str_utf_pos/start/end, vim.stricmp, vim.iconv -- which reach
--       the same mbyte entry points through a different caller, and
--       whose argument checking is its own code
--   s10 buffer-level multibyte -- charidx/byteidx against a real buffer
--       via the ++enc round trip, virtcol/col/charcol over multibyte
--   s90 the error arms, run UNCAPTURED, so the .stderr artifact carries
--       signal instead of being empty
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time, buffer handle or path outside the
-- work directory.  `%p` is the one conversion that can print an address
-- and it is only ever handed a Number.
--
-- UTFSWEEP_ONLY is a Lua pattern matched against each section name.
-- UTFSWEEP_TRACE=1 mirrors each section name to stderr.

local work = assert(os.getenv('UTF_WORK'), 'UTF_WORK unset')

-- Child mode.  s91 runs its cases in a *child* nvim because one of them
-- terminates the process (see the section's own comment), and an input
-- that aborts one side would otherwise take the whole report with it.
-- The child re-runs this same file -- every case list here is built
-- deterministically from constants -- and prints the cases from index
-- `argv[2]` onward, one line each, unbuffered so that a preserve_exit
-- cannot swallow the last completed case.
local argv = _G.arg or {}
local child_from = (argv[1] == '--child') and tonumber(argv[2]) or nil

local structfd, binfd
if not child_from then
  structfd = assert(io.open(assert(os.getenv('UTF_STRUCT'), 'UTF_STRUCT unset'), 'w'))
  binfd = assert(io.open(assert(os.getenv('UTF_BIN'), 'UTF_BIN unset'), 'wb'))
end
local only = os.getenv('UTFSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('UTFSWEEP_TRACE') == '1'

-- Unbuffered in a child: the case that kills the process must still
-- have its predecessors on stdout when it does.
io.stdout:setvbuf(child_from and 'no' or 'line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before.
---
--- NOTE the deliberate absence of an `0x%x+ -> <ADDR>` rule: s07 prints
--- several hundred `%p` and `%#x` answers and they are the report.
--- `%p` is only ever handed a Number, so no address reaches here.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  text = text:gsub(vim.pesc(work) .. '/tmp/nvim%.[^/%s]*/%w+/%d+', '<TMPFILE>')
  text = text:gsub(vim.pesc(work) .. '/tmp/nvim%.[^/%s]*/%w+', '<TMPDIR>')
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub('<lambda>%d+', '<lambda>')
  text = text:gsub('<SNR>%d+_', '<SNR>_')
  text = text:gsub('%S*/target/debug/nvim', '<NVIM>')
  text = text:gsub('%S*/nvim%-%x+', '<NVIM>')
  return text
end

--- Escape to one printable line.  The whole point of this sweep is
--- *which bytes* came back, so nothing may be dropped: every byte
--- outside printable ASCII becomes \xNN, and a backslash is doubled so
--- the escaping is invertible.
local function esc(bytes)
  return (tostring(bytes):gsub('[\\\128-\255%c]', function(c)
    if c == '\\' then
      return '\\\\'
    end
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
-- Canonical dump.  Verbatim from diffsweep.lua / opsweep.lua /
-- varsweep.lua: the artifacts are read side by side often enough that
-- they must escape and sort the same way.
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

--- The fourth artifact: the raw bytes of a string answer, framed so the
--- record boundary cannot occur inside a payload (\x1e and \x1f are the
--- two bytes no corpus string contains and no answer can produce -- the
--- corpus is checked for them at startup).  The .txt escapes; this does
--- not, so an escaper bug cannot hide a byte difference here.
local function bin(label, s)
  if binfd then
    binfd:write('\30', label, '\31', tostring(s))
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
-- The report primitive.
-- ---------------------------------------------------------------------

local SEEN = {}

--- Report one answer.  `label` must be injective; a collision is
--- invisible in a 30,000-line report and turns two probes into one, so
--- say so in the artifact rather than in a comment.
local function ans(label, ok, value)
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1
  if not ok then
    local text = errtext(value)
    emit(label, 'E', esc(text))
    struct(label, { e = text })
    bin(label .. ' E', text)
    return
  end
  if type(value) == 'string' then
    emit(label, '=', esc(scrub(value)))
    bin(label, value)
  else
    emit(label, '=', (canon(value):gsub('\n', '\\n')))
  end
  struct(label, value)
end

--- Call a function and report what it answered.  Every probe in the
--- sweep goes through here, so a crash-shaped regression shows as a
--- missing tail rather than as a wrong answer.
local function P(label, fn, ...)
  local ok, res = pcall(fn, ...)
  ans(label, ok, res)
end

--- Ask Vimscript.  Some of what this sweep probes has no Lua spelling
--- (`:e ++enc=`), and some answers differ between `vim.fn.f()` and
--- `eval('f()')` because the Lua bridge converts.
local function E(label, expr)
  local ok, res = pcall(vim.api.nvim_eval, expr)
  ans(label, ok, res)
end

--- Run-length encode a list of small values.  The table scans in s05
--- produce 100k+ answers each and are only readable -- and only diffable
--- -- compressed.  A single changed range is one changed token.
local function rle(list)
  local out, i = {}, 1
  while i <= #list do
    local j = i
    while j < #list and list[j + 1] == list[i] do
      j = j + 1
    end
    if j > i then
      out[#out + 1] = tostring(list[i]) .. 'x' .. (j - i + 1)
    else
      out[#out + 1] = tostring(list[i])
    end
    i = j + 1
  end
  return table.concat(out, ',')
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- The corpus.  Named, so a probe says which shape it is asking about and
-- two sections asking the same question use the same bytes.  Written as
-- decimal escapes rather than literal UTF-8 so that the file itself
-- survives being read by anything, and so that the invalid entries are
-- unambiguously the bytes they claim to be.
-- ---------------------------------------------------------------------

local C = {}
local ORDER = {}
local function corpus(name, bytes, note)
  C[name] = bytes
  ORDER[#ORDER + 1] = name
  assert(not bytes:find('[\30\31]'), 'corpus ' .. name .. ' contains a .bin frame byte')
end

corpus('empty', '', 'the empty string: every length routine has an early out for it')
corpus('ascii', 'Hello, World!', 'plain ASCII')
corpus('spaces', '  a b\tc  ', 'leading/internal/trailing whitespace and a tab')
corpus('digits', '0123456789', '')
corpus('punct', '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~', 'every ASCII punctuation character')
corpus('ctrl', '\1\2\9\27\127', 'control characters: strtrans and utf_printable')
corpus('latin1raw', '\233\232\252', 'e-acute/e-grave/u-diaeresis as LATIN-1 -- invalid UTF-8')
corpus('latin1u', '\195\169\195\168\195\188', 'the same three characters as UTF-8')
corpus('cjk', '\230\151\165\230\156\172\232\170\158', 'CJK: three double-width characters')
corpus('cjkmix', 'a\227\129\130b\230\188\162c', 'ASCII and double-width interleaved')
corpus('halfkana', '\239\189\177\239\189\178\239\189\179', 'halfwidth katakana: single-width above U+FF00')
corpus('hangul', '\237\149\156\234\184\128', 'Hangul syllables')
corpus('comb', 'e\204\129a\204\128', 'combining acute and grave: two composed characters')
corpus('comb5', 'a\204\129\204\130\204\131\204\132\204\133', 'one base with five combining marks -- MAX_MCO')
corpus('emoji', '\240\159\152\128', 'a 4-byte emoji')
corpus('zwj', '\240\159\145\168\226\128\141\240\159\146\187', 'ZWJ sequence: man + ZWJ + laptop')
corpus('flag', '\240\159\135\175\240\159\135\181', 'a regional-indicator flag pair')
corpus('ambi', '\194\177\195\151\195\183\226\128\166', 'plus-minus/times/divide/ellipsis -- ambiguous width')
corpus('invalid', '\255\254', 'two bytes that can never start a UTF-8 sequence')
corpus('overlong2', '\192\175', "overlong two-byte encoding of '/'")
corpus('overlong3', '\224\128\175', "overlong three-byte encoding of '/'")
corpus('overlong4', '\240\128\128\175', "overlong four-byte encoding of '/'")
corpus('surrogate', '\237\160\128', 'U+D800 encoded as UTF-8 -- a lone surrogate')
corpus('surrlow', '\237\191\191', 'U+DFFF encoded as UTF-8')
corpus('trunc', '\230\151', 'the first two bytes of a three-byte sequence')
corpus('contonly', '\128\129\130', 'continuation bytes with no lead')
corpus('above', '\244\144\128\128', 'U+110000: past the last codepoint')
corpus('fivebyte', '\248\136\128\128\128', 'a five-byte sequence -- not UTF-8 at all')
corpus('tabs', 'a\tbb\tccc\t', 'tabs, for strdisplaywidth')
corpus('nbsp', '\194\160x\194\173y\226\128\139z', 'nbsp, soft hyphen, zero-width space')
corpus('bom', '\239\187\191abc', 'a UTF-8 BOM followed by ASCII')
corpus(
  'mixed',
  'A\230\151\165\240\159\152\128e\204\129\255z',
  'ASCII + CJK + emoji + combining + an invalid byte, in one string'
)
corpus('long', ('\230\151\165\230\156\172'):rep(40), '80 double-width characters')
corpus('longa', ('abcdefghij'):rep(12), '120 ASCII characters, for width and precision')

--- The subset the O(n^2) matrices use.  Chosen so that every shape the
--- full corpus distinguishes is still present: ASCII, wide, combining,
--- 4-byte, invalid, empty.
local CORE = {
  'empty',
  'ascii',
  'latin1raw',
  'cjk',
  'comb',
  'emoji',
  'zwj',
  'invalid',
  'surrogate',
  'mixed',
}

-- ---------------------------------------------------------------------
-- s01 -- the index family
-- ---------------------------------------------------------------------

--- The indices every index probe is asked at: before the start, every
--- position a short string has, and past the end.  charidx/byteidx have
--- separate early-outs for negative and for past-the-end and both are
--- reachable only from here.
local IDX = { -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 20, 100 }

section('s01-index', function()
  for _, name in ipairs(ORDER) do
    local s = C[name]
    emit('-- ' .. name .. ': ' .. #s .. ' bytes')
    for _, i in ipairs(IDX) do
      P('i01 byteidx ' .. name .. ' ' .. tag(i), vim.fn.byteidx, s, i)
      P('i01 byteidxcomp ' .. name .. ' ' .. tag(i), vim.fn.byteidxcomp, s, i)
      P('i01 charidx ' .. name .. ' ' .. tag(i), vim.fn.charidx, s, i)
      -- countcc: the third argument decides whether a composing
      -- character is its own index, which is the whole difference
      -- between charidx and byteidx/byteidxcomp.
      P('i01 charidxcc ' .. name .. ' ' .. tag(i), vim.fn.charidx, s, i, 1)
      P('i01 strgetchar ' .. name .. ' ' .. tag(i), vim.fn.strgetchar, s, i)
      P('i01 utf16idx ' .. name .. ' ' .. tag(i), vim.fn.utf16idx, s, i)
      P('i01 utf16idxcc ' .. name .. ' ' .. tag(i), vim.fn.utf16idx, s, i, 1)
      P('i01 utf16idxchar ' .. name .. ' ' .. tag(i), vim.fn.utf16idx, s, i, 0, 1)
      -- byteidx with a utf16 index is a third traversal of the same
      -- string, and mb_utf_index_to_bytes is where it lives.
      P('i01 byteidxu16 ' .. name .. ' ' .. tag(i), vim.fn.byteidx, s, i, 1)
    end
  end
end)

section('s01b-part', function()
  local SPANS = { -3, -1, 0, 1, 2, 3, 5, 9, 40 }
  for _, name in ipairs(ORDER) do
    local s = C[name]
    for _, start in ipairs(SPANS) do
      P('i02 strcharpart ' .. name .. ' ' .. tag(start), vim.fn.strcharpart, s, start)
      P('i02 strpart ' .. name .. ' ' .. tag(start), vim.fn.strpart, s, start)
      for _, len in ipairs({ 0, 1, 2, 4, 100 }) do
        P(
          'i02 strcharpartl ' .. name .. ' ' .. tag(start) .. ' ' .. tag(len),
          vim.fn.strcharpart,
          s,
          start,
          len
        )
        P(
          'i02 strpartl ' .. name .. ' ' .. tag(start) .. ' ' .. tag(len),
          vim.fn.strpart,
          s,
          start,
          len
        )
        -- strpart's fourth argument is the "chars" flag, which routes it
        -- through a different traversal entirely.
        P(
          'i02 strpartc ' .. name .. ' ' .. tag(start) .. ' ' .. tag(len),
          vim.fn.strpart,
          s,
          start,
          len,
          1
        )
        P(
          'i02 strcharpartsk ' .. name .. ' ' .. tag(start) .. ' ' .. tag(len),
          vim.fn.strcharpart,
          s,
          start,
          len,
          1
        )
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s02 -- the width family, under both 'ambiwidth' values
-- ---------------------------------------------------------------------

section('s02-width', function()
  for _, aw in ipairs({ 'single', 'double' }) do
    quiet('set ambiwidth=' .. aw)
    for _, name in ipairs(ORDER) do
      local s = C[name]
      local pre = 'w02 ' .. aw .. ' ' .. name
      P(pre .. ' strlen', vim.fn.strlen, s)
      P(pre .. ' strchars', vim.fn.strchars, s)
      P(pre .. ' strcharscc', vim.fn.strchars, s, 1)
      P(pre .. ' strcharlen', vim.fn.strcharlen, s)
      P(pre .. ' strwidth', vim.fn.strwidth, s)
      P(pre .. ' strdisplaywidth', vim.fn.strdisplaywidth, s)
      P(pre .. ' strutf16len', vim.fn.strutf16len, s)
      P(pre .. ' strutf16lencc', vim.fn.strutf16len, s, 1)
      -- strdisplaywidth's second argument is a starting virtual column,
      -- which is the only way to reach the tab-expansion arm.
      for _, col in ipairs({ 0, 1, 3, 7, 8 }) do
        P(pre .. ' sdwcol' .. col, vim.fn.strdisplaywidth, s, col)
      end
    end
    -- 'tabstop' feeds strdisplaywidth's tab arm; 'list'/'listchars' do
    -- not, and that asymmetry is worth having in the artifact.
    for _, ts in ipairs({ 1, 4, 8, 17 }) do
      quiet('set tabstop=' .. ts)
      P('w02 ' .. aw .. ' tabs ts' .. ts, vim.fn.strdisplaywidth, C.tabs)
      P('w02 ' .. aw .. ' tabs ts' .. ts .. ' c3', vim.fn.strdisplaywidth, C.tabs, 3)
    end
    quiet('set tabstop=8')
  end
  quiet('set ambiwidth=single')
end)

-- ---------------------------------------------------------------------
-- s03 -- codepoint conversion
-- ---------------------------------------------------------------------

--- Codepoints chosen for the arms rather than for coverage: every
--- UTF-8 length boundary, both surrogate ends, the last codepoint, the
--- first invalid one, an ambiguous-width one, a combining one, an
--- East-Asian wide one, and the control range.
local CPS = {
  0,
  1,
  9,
  10,
  13,
  27,
  31,
  32,
  65,
  97,
  126,
  127,
  128,
  159,
  160,
  161,
  169,
  173,
  177,
  191,
  192,
  255,
  256,
  0x2FF,
  0x300,
  0x36F,
  0x370,
  0x5D0,
  0x7FF,
  0x800,
  0x1100,
  0x115F,
  0x1160,
  0x200B,
  0x200D,
  0x2010,
  0x2026,
  0x2500,
  0x25A0,
  0x2E80,
  0x3000,
  0x3001,
  0x4E00,
  0x9FFF,
  0xAC00,
  0xD7A3,
  0xD7FF,
  0xD800,
  0xDBFF,
  0xDC00,
  0xDFFF,
  0xE000,
  0xF8FF,
  0xF900,
  0xFB00,
  0xFDD0,
  0xFE00,
  0xFEFF,
  0xFF01,
  0xFF60,
  0xFF61,
  0xFF9F,
  0xFFA0,
  0xFFFD,
  0xFFFE,
  0xFFFF,
  0x10000,
  0x1F300,
  0x1F468,
  0x1F600,
  0x20000,
  0x2FFFF,
  0xE0001,
  0xE0100,
  0x10FFFD,
  0x10FFFF,
  0x110000,
  0x1FFFFF,
  0x7FFFFFFF,
  -1,
  -128,
}

section('s03-codepoint', function()
  for _, c in ipairs(CPS) do
    local t = tag(c)
    P('c03 nr2char ' .. t, vim.fn.nr2char, c)
    P('c03 nr2char8 ' .. t, vim.fn.nr2char, c, 1)
    P('c03 nr2char0 ' .. t, vim.fn.nr2char, c, 0)
    P('c03 list2str ' .. t, vim.fn.list2str, { c })
    P('c03 list2str8 ' .. t, vim.fn.list2str, { c }, 1)
    -- The round trip is the assertion: utf_char2bytes then utf_ptr2char.
    P('c03 rt ' .. t, function()
      return vim.fn.char2nr(vim.fn.nr2char(c))
    end)
  end
  for _, name in ipairs(ORDER) do
    local s = C[name]
    P('c03 char2nr ' .. name, vim.fn.char2nr, s)
    P('c03 char2nr8 ' .. name, vim.fn.char2nr, s, 1)
    P('c03 str2list ' .. name, vim.fn.str2list, s)
    P('c03 str2list8 ' .. name, vim.fn.str2list, s, 1)
    -- list2str(str2list(x)) is the byte-level round trip, and the .bin
    -- artifact is where an invalid sequence's fate is actually visible.
    P('c03 rtlist ' .. name, function()
      return vim.fn.list2str(vim.fn.str2list(s))
    end)
    P('c03 rtlist8 ' .. name, function()
      return vim.fn.list2str(vim.fn.str2list(s, 1), 1)
    end)
  end
  -- str2nr lives in strings.rs and has its own base parser.
  local NUMS = {
    '',
    '0',
    '-0',
    '007',
    '0x1f',
    '0X1F',
    '0b101',
    '0B101',
    '0o17',
    '0O17',
    '017',
    '  12  ',
    '+12',
    '-12',
    '12abc',
    'abc',
    '9223372036854775807',
    '9223372036854775808',
    '-9223372036854775808',
    '0xffffffffffffffff',
    "1'000",
    '1_000',
  }
  for _, n in ipairs(NUMS) do
    for _, base in ipairs({ 0, 2, 8, 10, 16 }) do
      P('c03 str2nr ' .. tag(n) .. ' b' .. base, vim.fn.str2nr, n, base)
    end
    P('c03 str2nrq ' .. tag(n), vim.fn.str2nr, n, 10, 1)
  end
end)

-- ---------------------------------------------------------------------
-- s04 -- setcellwidths(), the one option-like input to the width tables
-- ---------------------------------------------------------------------

section('s04-cellwidths', function()
  local SETS = {
    { name = 'none', list = {} },
    { name = 'ambi2', list = { { 0x00A1, 0x00A1, 2 }, { 0x2026, 0x2026, 2 } } },
    { name = 'ascii', list = { { 0x41, 0x5A, 2 } } },
    { name = 'wide1', list = { { 0x4E00, 0x9FFF, 1 } } },
    { name = 'emoji1', list = { { 0x1F300, 0x1FAFF, 1 } } },
    { name = 'range', list = { { 0x2000, 0x2FFF, 2 }, { 0x3000, 0x30FF, 1 } } },
    { name = 'single', list = { { 0x00B1, 0x00B1, 1 } } },
    { name = 'edge', list = { { 0x10FFFF, 0x10FFFF, 2 } } },
  }
  for _, set in ipairs(SETS) do
    for _, aw in ipairs({ 'single', 'double' }) do
      quiet('set ambiwidth=' .. aw)
      P('x04 set ' .. set.name .. ' ' .. aw, vim.fn.setcellwidths, set.list)
      P('x04 get ' .. set.name .. ' ' .. aw, vim.fn.getcellwidths)
      for _, cname in ipairs(CORE) do
        P('x04 w ' .. set.name .. ' ' .. aw .. ' ' .. cname, vim.fn.strwidth, C[cname])
        P('x04 d ' .. set.name .. ' ' .. aw .. ' ' .. cname, vim.fn.strdisplaywidth, C[cname])
      end
      for _, c in ipairs({ 0x41, 0xA1, 0xB1, 0x2026, 0x3000, 0x4E00, 0x1F600, 0x10FFFF }) do
        P('x04 c ' .. set.name .. ' ' .. aw .. ' ' .. tag(c), function()
          return vim.fn.strwidth(vim.fn.nr2char(c))
        end)
      end
    end
  end
  quiet('set ambiwidth=single')
  P('x04 clear', vim.fn.setcellwidths, {})
  -- The rejections: f_setcellwidths validates shape, range, order and
  -- overlap, and each arm has its own message.
  local BAD = {
    { 'notlist', 'x' },
    { 'shortrow', { { 1, 2 } } },
    { 'longrow', { { 1, 2, 2, 3 } } },
    { 'width3', { { 0x100, 0x100, 3 } } },
    { 'width0', { { 0x100, 0x100, 0 } } },
    { 'reversed', { { 0x200, 0x100, 2 } } },
    { 'ascii', { { 0x20, 0x30, 2 } } },
    { 'toohigh', { { 0x110000, 0x110000, 2 } } },
    { 'negative', { { -1, 5, 2 } } },
    { 'overlap', { { 0x100, 0x200, 2 }, { 0x180, 0x280, 1 } } },
    { 'unsorted', { { 0x300, 0x400, 2 }, { 0x100, 0x200, 1 } } },
    { 'strrow', { 'ab' } },
    { 'strval', { { '1', '2', '2' } } },
  }
  for _, b in ipairs(BAD) do
    P('x04 bad ' .. b[1], vim.fn.setcellwidths, b[2])
    P('x04 badget ' .. b[1], vim.fn.getcellwidths)
  end
  P('x04 final', vim.fn.setcellwidths, {})
end)

-- ---------------------------------------------------------------------
-- s05 -- THE TABLES.  charclass / cell width / case fold / printability
-- scanned over the codepoint space and run-length encoded.
--
-- This is the only front door to `utf_class_tab` (399 lines of range
-- table) and to the fold/upper/lower tables, and it is the section a
-- table-folding rewrite of mbyte.rs will actually be gated by: a single
-- moved range boundary is one changed token in one line.
--
-- The scan is dense where Unicode is dense and strided above, because a
-- complete scan of 0..0x110000 x five probes is 5.5M calls.  UTFSCAN_*
-- are not environment-tunable on purpose: a baseline taken at a
-- different stride is a different artifact.
-- ---------------------------------------------------------------------

section('s05-tables', function()
  local ranges = {
    { 1, 0x3400, 1 }, -- everything below the CJK ideographs, complete
    { 0x3400, 0x10000, 3 },
    { 0x10000, 0x30000, 7 },
    { 0x30000, 0x110000, 61 },
  }
  local nr2char, char2nr = vim.fn.nr2char, vim.fn.char2nr
  local charclass, strwidth = vim.fn.charclass, vim.fn.strwidth
  local tolower, toupper, strtrans = vim.fn.tolower, vim.fn.toupper, vim.fn.strtrans
  for _, aw in ipairs({ 'single', 'double' }) do
    quiet('set ambiwidth=' .. aw)
    for _, r in ipairs(ranges) do
      local cls, wid, low, up, pr, len = {}, {}, {}, {}, {}, {}
      local n = 0
      for c = r[1], r[2] - 1, r[3] do
        n = n + 1
        local ch = nr2char(c)
        if aw == 'single' then
          -- The class/case/printability tables do not depend on
          -- 'ambiwidth'; only the width one does.  Scanning them twice
          -- would double the runtime for a guaranteed-identical answer.
          cls[n] = charclass(ch)
          -- The fold tables are recorded as a *delta*, which run-length
          -- encodes to almost nothing and still names the exact
          -- codepoint any moved entry belongs to.
          low[n] = char2nr(tolower(ch)) - c
          up[n] = char2nr(toupper(ch)) - c
          pr[n] = strtrans(ch) == ch and 1 or 0
          len[n] = #ch
        end
        wid[n] = strwidth(ch)
      end
      -- `= ` between label and value, exactly as ans() spells it, so
      -- that one parser reads the whole artifact.
      local function row(label, list)
        ans(label, true, rle(list))
      end
      local span = string.format('%06x-%06x/%d', r[1], r[2], r[3])
      row('t05 ' .. aw .. ' ' .. span .. ' width', wid)
      if aw == 'single' then
        row('t05 ' .. span .. ' class', cls)
        row('t05 ' .. span .. ' lower', low)
        row('t05 ' .. span .. ' upper', up)
        row('t05 ' .. span .. ' printable', pr)
        row('t05 ' .. span .. ' bytes', len)
      end
    end
  end
  quiet('set ambiwidth=single')
  -- charclass() named at the boundaries too, so a diff in the RLE row
  -- has a readable neighbour to be located against.
  for _, c in ipairs(CPS) do
    P('t05 charclass ' .. tag(c), function()
      return vim.fn.charclass(vim.fn.nr2char(c))
    end)
  end
  for _, name in ipairs(ORDER) do
    P('t05 charclasss ' .. name, vim.fn.charclass, C[name])
  end
end)

-- ---------------------------------------------------------------------
-- s05b -- str_foldcase, added at B18-5.
--
-- `str_foldcase` (charset/display.rs) is rewritten by B18-8 and NOTHING
-- in the tree measures it: it has no Vimscript front door of its own.
-- It is reached three ways, and this section drives all three, because
-- the two arms behave differently and only one of them is bounded:
--
--   * `syntax/keyword.rs` add_keyword folds a `:syntax keyword` under
--     `:syntax case ignore` INTO A FIXED BUFFER of MAXKEYWLEN+1 bytes;
--   * `syntax/endpos.rs` folds the buffer word the same way at lookup;
--   * `insexpand/session.rs` folds the completion pattern under
--     'ignorecase' with a NULL buffer, i.e. into a fresh allocation
--     that grows.
--
-- THE BYTE LENGTH OF A CHARACTER CAN CHANGE WHEN IT FOLDS, and that is
-- the whole difficulty of the function.  A scan of the entire codepoint
-- space against this build finds:
--
--   * exactly TWO codepoints whose lowercase form is LONGER -- U+023A
--     -> U+2C65 and U+023E -> U+2C66, both 2 bytes -> 3.  They are the
--     only inputs in Unicode that reach the `nlen > olen` growth arm,
--     the `ga_grow` on the allocating path, and the "left as it was"
--     give-up on the fixed-buffer path.  A corpus without them does not
--     test that arm at all.
--   * twenty-five whose lowercase form is SHORTER, led by U+0130 -> i
--     (2 -> 1) and U+212A -> k (3 -> 1).
--
-- Both sets are in the corpus below, and the growth one is also run at
-- lengths that straddle MAXKEYWLEN so that the fixed-buffer give-up is
-- exercised on both sides of its bound.
-- ---------------------------------------------------------------------

local FOLDW = {
  { 'ascii', 'STRASSE', 'the ASCII path: one byte in, one byte out' },
  { 'latin', '\195\156BER', 'U+00DC: 2 bytes -> 2' },
  { 'greek', '\206\163\206\159\206\166\206\152', 'Greek capitals: 2 -> 2' },
  { 'sigma', '\206\145\206\163', 'ends in capital sigma -- final-form question' },
  { 'cyril', '\208\150\208\163\208\154', 'Cyrillic: 2 -> 2' },
  { 'shrink1', '\196\176D', 'U+0130 -> U+0069: 2 bytes -> 1' },
  { 'shrink2', '\226\132\170ELVIN', 'U+212A KELVIN SIGN -> k: 3 bytes -> 1' },
  { 'shrink3', '\225\186\158', 'U+1E9E -> U+00DF: 3 bytes -> 2' },
  { 'shrink4', '\226\132\166\226\132\171', 'U+2126 and U+212B, both 3 -> 2' },
  { 'grow1', '\200\186', 'U+023A -> U+2C65: 2 bytes -> 3, one of only two' },
  { 'grow2', '\200\190', 'U+023E -> U+2C66: the other one' },
  { 'growmix', 'A\200\186B\200\190C', 'the growth pair between ASCII' },
  { 'deseret', '\240\144\144\128\240\144\144\129', 'non-BMP: 4 bytes -> 4' },
  { 'title', '\199\133', 'U+01C5, a TITLECASE letter' },
  { 'roman', '\226\133\160\226\133\161', 'U+2160/U+2161: 3 -> 3' },
  { 'nocase', '\230\151\165\230\156\172', 'CJK: no case at all, must pass through' },
  { 'latin1raw', '\233\232\252', 'raw Latin-1 -- invalid UTF-8, must NOT fold' },
  {
    'mixed',
    'A\208\150e\204\129\200\186\240\144\144\128',
    'ASCII + Cyrillic + combining + growth + non-BMP',
  },
}

section('s05b-foldcase', function()
  -- ---- A: the FIXED-BUFFER arm, through :syntax keyword ------------
  --
  -- With `:syntax case ignore` the keyword table is keyed on the folded
  -- form and the lookup folds too, so a fold that loses or mangles a
  -- byte shows up as a word that stops highlighting.  The answer is the
  -- syntax group at every BYTE column, which also says where a fold
  -- that changed a length put the boundary.
  local function synrow(label, keyword, text)
    quiet('syntax clear')
    quiet('syntax case ignore')
    local err = quiet('syntax keyword FoldK ' .. keyword)
    if err then
      ans(label, false, err)
      return
    end
    vim.api.nvim_buf_set_lines(0, 0, -1, false, { text })
    local ids = {}
    for c = 1, #text do
      ids[c] = vim.fn.synIDattr(vim.fn.synID(1, c, 1), 'name')
    end
    ans(label, true, table.concat(ids, ','))
  end

  quiet('enew!')
  quiet('set filetype=')
  for _, w in ipairs(FOLDW) do
    local name, bytes = w[1], w[2]
    local lower = vim.fn.tolower(bytes)
    local upper = vim.fn.toupper(bytes)
    -- keyword folded down, text as given; and the reverse, so that a
    -- fold applied on only one of the two sides is still caught.
    synrow('t05b syn/' .. name .. '/lowkw', lower, bytes)
    synrow('t05b syn/' .. name .. '/upkw', upper, bytes)
    synrow('t05b syn/' .. name .. '/lowtext', bytes, lower)
    synrow('t05b syn/' .. name .. '/uptext', bytes, upper)
    -- The control: `syntax case match` does not fold at all, so a row
    -- that reads the same under both is not measuring the fold.
    quiet('syntax clear')
    quiet('syntax case match')
    quiet('syntax keyword FoldK ' .. lower)
    vim.api.nvim_buf_set_lines(0, 0, -1, false, { bytes })
    local ids = {}
    for c = 1, #bytes do
      ids[c] = vim.fn.synIDattr(vim.fn.synID(1, c, 1), 'name')
    end
    ans('t05b synmatch/' .. name, true, table.concat(ids, ','))
  end

  -- ---- B: the fixed buffer's BOUND ---------------------------------
  --
  -- add_keyword folds into MAXKEYWLEN+1 = 81 bytes and the doc comment
  -- says a character whose lowercase form no longer fits is left as it
  -- was.  U+023A is the only kind of input that can make a fold GROW,
  -- so a run of it is the only way to reach that give-up; the lengths
  -- straddle both 80 input bytes and 81 output bytes.
  for _, n in ipairs({ 1, 20, 26, 27, 39, 40, 41, 45 }) do
    local run = ('\200\186'):rep(n)
    synrow(string.format('t05b bound/grow%02d', n), vim.fn.tolower(run), run)
    local arun = ('A'):rep(n * 2)
    synrow(string.format('t05b bound/ascii%02d', n), vim.fn.tolower(arun), arun)
  end

  -- ---- C: the ALLOCATING arm, through 'ignorecase' completion ------
  --
  -- get_wholeline_compl_info folds the typed prefix with a NULL buffer
  -- when 'ignorecase' is set -- the growing path, which has no bound
  -- and therefore no give-up.  `<C-x><C-l>` is its front door.  With
  -- 'noignorecase' the same code takes the cbuf_to_string branch, so
  -- the PAIR of rows is the measurement and neither row alone is.
  local function wholeline(label, source, typed)
    quiet('enew!')
    vim.api.nvim_buf_set_lines(0, 0, -1, false, { source .. ' TAIL', '' })
    vim.api.nvim_win_set_cursor(0, { 2, 0 })
    local keys =
      -- `<C-y>` ACCEPTS the match.  `<C-e>` looks like the right key and
      -- is not: it CANCELS and restores what was typed, which makes
      -- every 'ignorecase' row read exactly like its 'noignorecase'
      -- twin and the whole part measure nothing.
      vim.api.nvim_replace_termcodes('i' .. typed .. '<C-x><C-l><C-y><Esc>', true, true, true)
    local ok, err = pcall(vim.api.nvim_feedkeys, keys, 'x', false)
    if not ok then
      ans(label, false, err)
      return
    end
    ans(label, true, vim.fn.getline(2))
  end

  -- 'shortmess' +cC.  Without them ins-completion writes 667 lines into
  -- the .stderr artifact for these thirty-six cases, none of it about
  -- the fold and all of it glued together because the messages carry no
  -- newline.  BOTH flags are needed and they are different messages: `c`
  -- silences "Pattern not found" / "The only match", `C` silences
  -- "Scanning: ...", which is the 631 of the 667.
  local shm = vim.o.shortmess
  quiet('set shortmess+=cC')
  for _, ic in ipairs({ true, false }) do
    quiet('set ' .. (ic and 'ignorecase' or 'noignorecase'))
    local tag_ic = ic and 'ic' or 'noic'
    for _, w in ipairs(FOLDW) do
      local name, bytes = w[1], w[2]
      -- Type the LOWERCASE form against an UPPERCASE source line: under
      -- 'ignorecase' the fold is what makes the two meet.
      wholeline('t05b compl/' .. tag_ic .. '/' .. name, bytes, vim.fn.tolower(bytes))
    end
  end
  quiet('set noignorecase')
  vim.o.shortmess = shm

  -- ---- D: the neighbouring answer ----------------------------------
  --
  -- tolower()/toupper() reach mb_tolower through strlow_save, i.e. the
  -- same case table but none of str_foldcase's buffer arithmetic.  A
  -- row that moves HERE too is a table change; a row that moves only
  -- above is the arithmetic.
  for _, w in ipairs(FOLDW) do
    local name, bytes = w[1], w[2]
    P('t05b tolower/' .. name, vim.fn.tolower, bytes)
    P('t05b toupper/' .. name, vim.fn.toupper, bytes)
    ans(
      't05b len/' .. name,
      true,
      string.format('%d,%d,%d', #bytes, #vim.fn.tolower(bytes), #vim.fn.toupper(bytes))
    )
  end

  quiet('syntax clear')
  quiet('syntax case match')
  quiet('enew!')
end)

-- ---------------------------------------------------------------------
-- s06 -- the escape family
-- ---------------------------------------------------------------------

section('s06-escape', function()
  -- vim_strsave_escaped_ext copies a multibyte character whole and never
  -- compares its bytes against the escape set, so only a set that can
  -- match a *lead or continuation byte* distinguishes "walked by
  -- character" from "walked by byte".  And the set has to spell that
  -- byte as a CHARACTER: vim_strchr() encodes anything >= 0x80 to UTF-8
  -- and searches for the sequence, so a raw 0xC3 in the set is the
  -- two-byte U+00C3 and never matches the byte 0xC3.
  --
  -- The last four sets are therefore the *characters whose codepoints
  -- are the bytes of a corpus character*: U+00C3 U+00A9 are the bytes of
  -- 'é', U+00E6 U+0097 U+00A5 the bytes of '日'.  A mutation on that
  -- copy step measured NOT CAUGHT against the raw-byte spelling and is
  -- caught by these.
  local ESCSETS = {
    '',
    ' ',
    '\\',
    'aeiou',
    '%#',
    '\t\n',
    '\255',
    ' \t\\"',
    '\195\131\194\169', -- U+00C3 U+00A9: the two bytes of 'é'
    '\195\166\194\151\194\165', -- U+00E6 U+0097 U+00A5: the three bytes of '日'
    '\195\176\194\159', -- U+00F0 U+009F: the first two bytes of an emoji
    '\195\191\195\190', -- U+00FF U+00FE: the `invalid` corpus entry's bytes
  }
  for _, name in ipairs(ORDER) do
    local s = C[name]
    P('e06 strtrans ' .. name, vim.fn.strtrans, s)
    P('e06 keytrans ' .. name, vim.fn.keytrans, s)
    P('e06 fnameescape ' .. name, vim.fn.fnameescape, s)
    P('e06 shellescape ' .. name, vim.fn.shellescape, s)
    P('e06 shellescape1 ' .. name, vim.fn.shellescape, s, 1)
    P('e06 toupper ' .. name, vim.fn.toupper, s)
    P('e06 tolower ' .. name, vim.fn.tolower, s)
    P('e06 trim ' .. name, vim.fn.trim, s)
    P('e06 trimchars ' .. name, vim.fn.trim, s, ' \t\230')
    for _, d in ipairs({ 0, 1, 2 }) do
      P('e06 trimd ' .. name .. ' ' .. d, vim.fn.trim, s, ' \tz', d)
    end
    for _, set in ipairs(ESCSETS) do
      P('e06 escape ' .. name .. ' ' .. tag(set), vim.fn.escape, s, set)
    end
    P('e06 strrep0 ' .. name, vim.fn.repeat_ or vim.fn['repeat'], s, 0)
    P('e06 strrep3 ' .. name, vim.fn['repeat'], s, 3)
  end
  -- 'shell' decides shellescape's quoting entirely, and the csh arm is a
  -- different function body.
  -- csh_like_shell() and fish_like_shell() are the two shape tests
  -- vim_strsave_shellescape makes; 'shellxquote' is NOT read by it at
  -- all (checked -- five spellings answered identically), so the axis
  -- here is the shell *name*.
  for _, sh in ipairs({ '/bin/sh', '/bin/csh', '/bin/tcsh', '/bin/fish', 'cmd.exe', 'powershell' }) do
    quiet('set shell=' .. vim.fn.escape(sh, ' \\'))
    for _, name in ipairs(CORE) do
      P('e06 sh ' .. tag(sh) .. ' ' .. name, vim.fn.shellescape, C[name])
      P('e06 sh1 ' .. tag(sh) .. ' ' .. name, vim.fn.shellescape, C[name], 1)
    end
    P('e06 sh ' .. tag(sh) .. ' bang', vim.fn.shellescape, 'a!b', 1)
    P('e06 sh ' .. tag(sh) .. ' nl', vim.fn.shellescape, 'a\nb', 1)
    P('e06 sh ' .. tag(sh) .. ' cr', vim.fn.shellescape, 'a\rb', 1)
    P('e06 sh ' .. tag(sh) .. ' pct', vim.fn.shellescape, 'a%b#c', 1)
  end
  quiet('set shell=/bin/sh')
  -- The two arms that only `do_special` reaches: find_cmdline_var, and
  -- the newline flag.  Each one is a `continue` of its own.
  for _, s in ipairs({ '%', '#', '%%', '<cword>', '<afile>', 'a%b#c', 'a\nb', 'a!b', "a'b", 'a\\b' }) do
    P('e06 spec0 ' .. tag(s), vim.fn.shellescape, s)
    P('e06 spec1 ' .. tag(s), vim.fn.shellescape, s, 1)
  end

  -- tr() -- its own transliteration loop, with a multibyte arm.
  local TRS = {
    { 'abc', 'xyz' },
    { 'abc', 'x' },
    { 'a', 'xyz' },
    { '', '' },
    { '\230\151\165', 'X' },
    { 'X', '\230\151\165' },
    { '\230\151\165\230\156\172', '\240\159\152\128\240\159\152\128' },
    { '\255', 'Z' },
    { 'e\204\129', 'ab' },
  }
  for _, name in ipairs(ORDER) do
    for i, t in ipairs(TRS) do
      P('e06 tr ' .. name .. ' ' .. i, vim.fn.tr, C[name], t[1], t[2])
    end
  end

  -- substitute() and matchstr() reach regexp/ through mbyte's
  -- boundary helpers; the point here is the multibyte arm, not the
  -- regex engine (which keysweep and evalsweep already cover).
  local PATS = {
    { '.', 'X', '' },
    { '.', 'X', 'g' },
    { '\\%d1c', '<>', '' },
    { '[[:alpha:]]', '_', 'g' },
    { '\\v(.)(.)', '\\2\\1', 'g' },
    { '\\zs', '-', 'g' },
    { '$', '!', '' },
    { '^', '>', '' },
    { '\\_.', '#', 'g' },
  }
  for _, name in ipairs(ORDER) do
    for i, p in ipairs(PATS) do
      P('e06 sub ' .. name .. ' ' .. i, vim.fn.substitute, C[name], p[1], p[2], p[3])
      P('e06 matchstr ' .. name .. ' ' .. i, vim.fn.matchstr, C[name], p[1])
      P('e06 matchstrpos ' .. name .. ' ' .. i, vim.fn.matchstrpos, C[name], p[1])
      P('e06 match ' .. name .. ' ' .. i, vim.fn.match, C[name], p[1])
      P('e06 matchend ' .. name .. ' ' .. i, vim.fn.matchend, C[name], p[1])
    end
    for _, needle in ipairs({ '', 'a', '\230\151\165', '\255', 'e\204\129' }) do
      P('e06 stridx ' .. name .. ' ' .. tag(needle), vim.fn.stridx, C[name], needle)
      P('e06 strridx ' .. name .. ' ' .. tag(needle), vim.fn.strridx, C[name], needle)
    end
  end
  -- 'ignorecase'/'smartcase' change what matchstr answers; they do not
  -- change stridx, and that asymmetry belongs in the artifact.
  for _, ic in ipairs({ 0, 1 }) do
    quiet('set ' .. (ic == 1 and 'ignorecase' or 'noignorecase'))
    for _, name in ipairs(CORE) do
      P('e06 ic' .. ic .. ' ' .. name, vim.fn.matchstr, C[name], 'A')
      P('e06 ics' .. ic .. ' ' .. name, vim.fn.stridx, C[name], 'A')
    end
  end
  quiet('set noignorecase')
end)

-- ---------------------------------------------------------------------
-- s07 -- printf().  vim_vsnprintf_typval is 1,297 lines and nothing in
-- the tree tests it.
-- ---------------------------------------------------------------------

local INTS = { 0, 1, -1, 7, 42, 255, 4095, -4096, 2147483647, -2147483648 }
local BIGINTS = { 9223372036854775807, -9223372036854775807 - 1 }
local FLOATS = {
  '0.0',
  '-0.0',
  '1.0',
  '-1.0',
  '0.5',
  '1.5',
  '2.5',
  '3.14159265358979',
  '1.0e-5',
  '1.0e20',
  '1.0e308',
  '5.0e-324',
  '1.0/0.0',
  '-1.0/0.0',
  '0.0/0.0',
  '123456789.123456789',
  '0.1',
  '1.0e-310',
  -- %g switches between fixed and exponent notation at 0.001 and at
  -- 1.0e7, and the corpus above straddles neither boundary closely
  -- enough: 123456789 is already past it.  These five sit either side
  -- of the two thresholds, and a mutation on the literal 10000000.0
  -- measured NOT CAUGHT without them.
  '999999.5',
  '1234567.5',
  '9999999.0',
  '10000001.0',
  '0.0009999',
}

section('s07-printf-int', function()
  local FLAGS = { '', '-', '0', '+', ' ', '#', '+-', '0#', "'", '-0' }
  local WIDTHS = { '', '1', '8', '12' }
  local PRECS = { '', '.0', '.3', '.8' }
  for _, spec in ipairs({ 'd', 'u', 'b', 'B', 'o', 'x', 'X' }) do
    for _, f in ipairs(FLAGS) do
      for _, w in ipairs(WIDTHS) do
        for _, pr in ipairs(PRECS) do
          local fmt = '%' .. f .. w .. pr .. spec
          for _, v in ipairs(INTS) do
            P('f07 ' .. tag(fmt) .. ' ' .. tag(v), vim.fn.printf, fmt, v)
          end
        end
      end
    end
    -- the 64-bit edges, separately, so they are readable
    for _, v in ipairs(BIGINTS) do
      for _, f in ipairs({ '', '#', '0' }) do
        local fmt = '%' .. f .. '24' .. spec
        P('f07 big ' .. tag(fmt) .. ' ' .. tag(v), vim.fn.printf, fmt, v)
      end
    end
  end
  -- %p: only ever a Number.  On a String argument tv_ptr hands back
  -- vval.v_string, i.e. an allocation address, which differs per run by
  -- construction; s90 asks that one uncaptured instead.
  for _, v in ipairs({ 0, 1, 255, 4096, 2147483647 }) do
    for _, fmt in ipairs({ '%p', '%12p', '%-12p', '%012p' }) do
      P('f07 ptr ' .. tag(fmt) .. ' ' .. tag(v), vim.fn.printf, fmt, v)
    end
  end
  -- length modifiers.  With a typval argument they select which tv_*
  -- fetcher runs, so they are not decoration.
  for _, mod in ipairs({ 'h', 'hh', 'l', 'll', 'L', 'z', 'j', 't' }) do
    for _, spec in ipairs({ 'd', 'u', 'x', 'o' }) do
      for _, v in ipairs({ 0, -1, 255, 65535, 2147483647 }) do
        local fmt = '%' .. mod .. spec
        P('f07 mod ' .. tag(fmt) .. ' ' .. tag(v), vim.fn.printf, fmt, v)
      end
    end
  end
  -- the pre-spec conversion modifiers Vim inherited from the original
  -- snprintf: %D %U %O are long, %i is d.
  for _, spec in ipairs({ 'i', 'D', 'U', 'O' }) do
    for _, v in ipairs({ 0, -1, 255, 2147483647 }) do
      P('f07 alt ' .. spec .. ' ' .. tag(v), vim.fn.printf, '%' .. spec, v)
      P('f07 altw ' .. spec .. ' ' .. tag(v), vim.fn.printf, '%08' .. spec, v)
    end
  end
end)

section('s07b-printf-str', function()
  local FLAGS = { '', '-' }
  local WIDTHS = { '', '3', '10', '20' }
  local PRECS = { '', '.0', '.1', '.3', '.6', '.20' }
  -- %s and %S differ in exactly one thing -- %S counts *cells*, not
  -- bytes -- so the multibyte corpus under a width and a precision is
  -- the only place that difference is visible.
  --
  -- %S is restricted here to formats with NO minimum field width,
  -- because `%<n>S` over a string whose cell count exceeds its byte
  -- count terminates the process: the 'S' arm computes
  -- `min_field_width += str_arg_l - i` in size_t, and for e.g. three
  -- LATIN-1 bytes (3 bytes, 12 cells as <e9><e8><fc>) that underflows to
  -- ~2^64, which xmalloc answers with E41 and preserve_exit.  The whole
  -- `%S`-with-a-width matrix is in s91 instead, one child per group.
  for _, f in ipairs(FLAGS) do
    for _, w in ipairs(WIDTHS) do
      for _, pr in ipairs(PRECS) do
        for _, name in ipairs(ORDER) do
          P('f07s ' .. tag('%' .. f .. w .. pr .. 's') .. ' ' .. name, vim.fn.printf, '%' .. f .. w .. pr .. 's', C[name])
        end
      end
    end
    for _, pr in ipairs(PRECS) do
      for _, name in ipairs(ORDER) do
        P('f07s ' .. tag('%' .. f .. pr .. 'S') .. ' ' .. name, vim.fn.printf, '%' .. f .. pr .. 'S', C[name])
      end
    end
  end
  -- %c takes a Number and emits it as a character; the multibyte arm is
  -- utf_char2bytes.
  for _, c in ipairs(CPS) do
    for _, fmt in ipairs({ '%c', '%5c', '%-5c', '%.2c' }) do
      P('f07c ' .. tag(fmt) .. ' ' .. tag(c), vim.fn.printf, fmt, c)
    end
  end
  -- a non-String argument to %s is stringified by tv_str, which is a
  -- different path from a plain string.
  P('f07s tv list', vim.fn.printf, '%s', { 1, 2, 3 })
  P('f07s tv dict', vim.fn.printf, '%s', { a = 1 })
  P('f07s tv num', vim.fn.printf, '%s', 42)
  P('f07s tv float', vim.fn.printf, '%s', 1.5)
  P('f07s tv bool', vim.fn.printf, '%s', true)
  P('f07s tv nil', vim.fn.printf, '%s', vim.NIL)
  P('f07S tv list', vim.fn.printf, '%S', { 1, 2, 3 })
  P('f07S tv num', vim.fn.printf, '%S', 42)
end)

section('s07c-printf-float', function()
  local FLAGS = { '', '-', '0', '+', '#' }
  local WIDTHS = { '', '10', '20' }
  local PRECS = { '', '.0', '.1', '.6', '.17' }
  for _, spec in ipairs({ 'f', 'F', 'e', 'E', 'g', 'G' }) do
    for _, f in ipairs(FLAGS) do
      for _, w in ipairs(WIDTHS) do
        for _, pr in ipairs(PRECS) do
          local fmt = '%' .. f .. w .. pr .. spec
          for _, v in ipairs(FLOATS) do
            E(
              'f07f ' .. tag(fmt) .. ' ' .. tag(v),
              'printf(' .. vim.fn.string(fmt) .. ', ' .. v .. ')'
            )
          end
        end
      end
    end
  end
  -- a Number argument to a float conversion goes through tv_float's
  -- widening arm; anything else is E807.
  for _, spec in ipairs({ 'f', 'e', 'g' }) do
    for _, v in ipairs({ 0, 1, -1, 2147483647 }) do
      P('f07f num ' .. spec .. ' ' .. tag(v), vim.fn.printf, '%' .. spec, v)
    end
  end
  -- huge precisions: the 350-byte `tmp` buffer in vim_vsnprintf_typval
  -- is what stands between these and a stack overwrite.
  for _, pr in ipairs({ '.30', '.60', '.100', '.200', '.340', '.400', '.1000' }) do
    for _, spec in ipairs({ 'f', 'e', 'g' }) do
      E('f07f wide ' .. tag(pr) .. spec, 'printf("%' .. pr .. spec .. '", 1.0/3.0)')
      E('f07f wideb ' .. tag(pr) .. spec, 'printf("%' .. pr .. spec .. '", 1.0e308)')
    end
  end
  for _, w in ipairs({ '100', '200', '340', '400', '1000' }) do
    E('f07f wwide ' .. w, 'printf("%' .. w .. '.2f", 1.5)')
    E('f07f wwides ' .. w, 'printf("%' .. w .. 's", "x")')
  end
end)

section('s07d-printf-args', function()
  -- `*` takes the width or precision from an argument, including a
  -- negative one (which means left-justify).
  local STARS = {
    { '%*d', { 8, 42 } },
    { '%*d', { -8, 42 } },
    { '%*d', { 0, 42 } },
    { '%.*d', { 5, 42 } },
    { '%.*d', { -5, 42 } },
    { '%*.*d', { 12, 5, 42 } },
    { '%*.*d', { -12, -5, 42 } },
    { '%*s', { 10, 'ab' } },
    { '%*s', { -10, 'ab' } },
    { '%.*s', { 1, '\230\151\165\230\156\172' } },
    { '%.*S', { 1, '\230\151\165\230\156\172' } },
    { '%*.*s', { 9, 3, '\240\159\152\128abc' } },
    { '%*.*f', { 14, 3, 1.5 } },
    { '%*c', { 4, 65 } },
    { '%*x', { 8, 255 } },
    { '%-*x', { 8, 255 } },
    { '%0*x', { 8, 255 } },
  }
  for i, s in ipairs(STARS) do
    P('f07a star ' .. i .. ' ' .. tag(s[1]), function()
      return vim.fn.printf(s[1], unpack(s[2]))
    end)
  end
  -- `%<n>$` positional arguments, which parse_fmt_types resolves in a
  -- pre-pass of its own.
  local POS = {
    { '%1$d', { 7 } },
    { '%2$d %1$d', { 1, 2 } },
    { '%1$d %1$d %1$d', { 5 } },
    { '%3$s-%1$s-%2$s', { 'a', 'b', 'c' } },
    { '%2$s', { 'a', 'b' } },
    { '%1$s %2$d %3$f', { 'x', 3, 1.5 } },
    { '%1$*2$d', { 42, 8 } },
    { '%1$.*2$f', { 1.5, 3 } },
    { '%10$d', { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 } },
    { '%1$c%2$c', { 65, 0x65E5 } },
    { '%1$x %1$X %1$o', { 255 } },
    { '%1$s %1$S', { '\230\151\165' } },
  }
  for i, s in ipairs(POS) do
    P('f07a pos ' .. i .. ' ' .. tag(s[1]), function()
      return vim.fn.printf(s[1], unpack(s[2]))
    end)
  end
  -- The malformed half.  Every one of these takes an arm of
  -- parse_fmt_types or of the conversion switch that nothing else does.
  local BADFMT = {
    '%',
    '%%',
    '%%%',
    'a%%b',
    '%q',
    '%y',
    '%1',
    '%1$',
    '%0$d',
    '%-1$d',
    '%1$d %d',
    '%d %1$d',
    '%2$d',
    '%99$d',
    '%1$$d',
    '%.d',
    '%.-1d',
    '%--d',
    '%++d',
    '%  d',
    '%##d',
    '%00d',
    '%*',
    '%.*',
    '%*$d',
    '%hhhd',
    '%llld',
    '%lc',
    '%ls',
    '%zs',
    '%Lf',
    '%hf',
    '%lf',
    '%d%',
    '%\230\151\165',
    '%\255',
    '\230\151\165%d',
    '%s%s%s%s%s',
    '%2147483647d',
    '%.2147483647f',
    '%99999999999999999999d',
  }
  for _, fmt in ipairs(BADFMT) do
    P('f07a bad ' .. tag(fmt), vim.fn.printf, fmt)
    P('f07a bad1 ' .. tag(fmt), vim.fn.printf, fmt, 1)
    P('f07a bad2 ' .. tag(fmt), vim.fn.printf, fmt, 'x', 2)
  end
  -- argument-count mismatches: E766 and E767.
  P('f07a few', vim.fn.printf, '%d %d %d', 1)
  P('f07a many', vim.fn.printf, '%d', 1, 2, 3)
  P('f07a none', vim.fn.printf, 'plain', 1)
  P('f07a onlyfmt', vim.fn.printf, '%d')
  -- printf() with no conversions at all is the memmove fast path.
  for _, name in ipairs(ORDER) do
    P('f07a lit ' .. name, vim.fn.printf, C[name])
  end
  -- and a format that is itself a corpus string, which is how a
  -- multibyte format reaches the `%` scanner.
  for _, name in ipairs(ORDER) do
    P('f07a fmt ' .. name, vim.fn.printf, C[name] .. '[%s]%d', 'v', 9)
  end
end)

-- ---------------------------------------------------------------------
-- s08 -- iconv() and the ++enc round trips
-- ---------------------------------------------------------------------

local ENCS = {
  'utf-8',
  'utf8',
  'UTF-8',
  'latin1',
  'iso-8859-1',
  'iso-8859-15',
  -- the two spellings enc_canonize rewrites rather than looks up: an
  -- unhyphenated iso8859 gains its hyphen, and a `microsoft-cp` prefix
  -- loses ten bytes.  Nothing else in the sweep reaches either fold.
  'iso8859-1',
  'ISO_8859-15',
  'microsoft-cp1252',
  'cp1252',
  'cp932',
  'euc-jp',
  'ucs-2',
  'ucs-2le',
  'utf-16',
  'utf-16le',
  'utf-32',
  'koi8-r',
  'macroman',
  'default',
  '',
  'no-such-encoding',
  'utf-8//IGNORE',
  '8bit-latin1',
  '2byte-euc-jp',
  'ansi',
  'japan',
}

section('s08-iconv', function()
  emit('x08 have-iconv ' .. vim.fn.has('iconv'))
  for _, from in ipairs(ENCS) do
    for _, to in ipairs(ENCS) do
      for _, name in ipairs(CORE) do
        P(
          'v08 ' .. tag(from) .. ' ' .. tag(to) .. ' ' .. name,
          vim.fn.iconv,
          C[name],
          from,
          to
        )
      end
    end
  end
  -- enc_canonize is only observable through what iconv accepts, and
  -- through 'fileencoding' normalising a value it was set to.
  for _, e in ipairs(ENCS) do
    P('v08 canon ' .. tag(e), function()
      local err = quiet('setlocal fileencoding=' .. vim.fn.escape(e, ' \\|"'))
      return { err = err, fenc = vim.bo.fileencoding }
    end)
  end
  quiet('setlocal fileencoding=')
end)

section('s08b-plusenc', function()
  -- `:e ++enc=` and `:w ++enc=`: string_convert_ext through the file
  -- reader and writer rather than through f_iconv, which is a different
  -- caller with a different buffer strategy.
  local dir = work .. '/files'
  local SAMPLES = {
    { 'ascii', 'Hello, World!' },
    { 'latin1raw', '\233\232\252' },
    { 'latin1u', '\195\169\195\168\195\188' },
    { 'cjk', '\230\151\165\230\156\172\232\170\158' },
    { 'emoji', '\240\159\152\128' },
    { 'invalid', '\255\254' },
    { 'bom', '\239\187\191abc' },
    { 'mixed', 'A\230\151\165\240\159\152\128e\204\129\255z' },
  }
  local USE = { 'utf-8', 'latin1', 'cp1252', 'euc-jp', 'ucs-2le', 'utf-16le', 'no-such' }
  for _, s in ipairs(SAMPLES) do
    local path = dir .. '/' .. s[1] .. '.txt'
    local fd = assert(io.open(path, 'wb'))
    fd:write(s[2], '\n')
    fd:close()
    for _, enc in ipairs(USE) do
      local label = 'p08 ' .. s[1] .. ' ' .. tag(enc)
      P(label, function()
        local err = quiet('silent! edit! ++enc=' .. enc .. ' ' .. vim.fn.fnameescape(path))
        local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
        return {
          err = err,
          fenc = vim.bo.fileencoding,
          bomb = vim.bo.bomb and 1 or 0,
          lines = lines,
        }
      end)
      -- the write half: read back the bytes the writer produced.
      local outp = dir .. '/out.' .. s[1] .. '.' .. tag(enc)
      P(label .. ' w', function()
        local err = quiet('silent! write! ++enc=' .. enc .. ' ' .. vim.fn.fnameescape(outp))
        local rd = io.open(outp, 'rb')
        local bytes = rd and rd:read('*a') or nil
        if rd then
          rd:close()
        end
        return { err = err, bytes = bytes }
      end)
      local rd = io.open(outp, 'rb')
      if rd then
        bin(label .. ' wbytes', rd:read('*a'))
        rd:close()
      end
    end
  end
  quiet('silent! %bwipeout!')
  -- 'fileencodings' drives the guess loop, which is the other consumer
  -- of enc_canonize and of the BOM check.
  for _, fencs in ipairs({
    'ucs-bom',
    'ucs-bom,utf-8,latin1',
    'utf-8,latin1',
    'latin1',
    'utf-8',
    '',
  }) do
    quiet('let &fileencodings = ' .. vim.fn.string(fencs))
    for _, s in ipairs(SAMPLES) do
      local path = dir .. '/' .. s[1] .. '.txt'
      P('p08 fencs ' .. tag(fencs) .. ' ' .. s[1], function()
        -- Wipe first: 'fileencoding' is buffer-local and sticky, so
        -- re-editing a buffer an earlier `++enc=` already set keeps that
        -- encoding and the whole 'fileencodings' axis measures nothing.
        -- It answered identically for all six spellings until this line.
        quiet('silent! bwipeout!')
        quiet('silent! edit! ' .. vim.fn.fnameescape(path))
        return {
          fenc = vim.bo.fileencoding,
          bomb = vim.bo.bomb and 1 or 0,
          lines = vim.api.nvim_buf_get_lines(0, 0, -1, false),
        }
      end)
    end
  end
  quiet('set fileencodings=ucs-bom,utf-8,default,latin1')
  quiet('silent! %bwipeout!')
end)

-- ---------------------------------------------------------------------
-- s09 -- the Lua leaves.  Same C, a different caller, and their own
-- argument checking.
-- ---------------------------------------------------------------------

section('s09-lua', function()
  for _, name in ipairs(ORDER) do
    local s = C[name]
    P('l09 str_utfindex ' .. name, vim.str_utfindex, s)
    P('l09 stricmp-self ' .. name, vim.stricmp, s, s)
    P('l09 stricmp-up ' .. name, vim.stricmp, s, vim.fn.toupper(s))
    P('l09 stricmp-a ' .. name, vim.stricmp, s, 'a')
    P('l09 str_utf_pos ' .. name, vim.str_utf_pos, s)
    for i = 0, math.min(#s, 12) do
      P('l09 str_utfindex ' .. name .. ' ' .. i, vim.str_utfindex, s, i)
      P('l09 str_utfindex16 ' .. name .. ' ' .. i, vim.str_utfindex, s, 'utf-16', i)
      P('l09 str_utfindex32 ' .. name .. ' ' .. i, vim.str_utfindex, s, 'utf-32', i)
      P('l09 str_byteindex ' .. name .. ' ' .. i, vim.str_byteindex, s, i)
      P('l09 str_byteindex16 ' .. name .. ' ' .. i, vim.str_byteindex, s, 'utf-16', i)
      P('l09 str_byteindex32 ' .. name .. ' ' .. i, vim.str_byteindex, s, 'utf-32', i)
    end
    for i = 1, math.min(#s, 12) do
      P('l09 str_utf_start ' .. name .. ' ' .. i, vim.str_utf_start, s, i)
      P('l09 str_utf_end ' .. name .. ' ' .. i, vim.str_utf_end, s, i)
    end
    -- the underscore spellings are the C functions themselves; the
    -- public ones are Lua wrappers with their own validation, and the
    -- two answer differently for an out-of-range index.
    P('l09 _str_utfindex ' .. name, vim._str_utfindex, s)
    P('l09 _str_byteindex ' .. name, vim._str_byteindex, s, 1)
    P('l09 iconv-l1 ' .. name, vim.iconv, s, 'utf-8', 'latin1')
    P('l09 iconv-back ' .. name, vim.iconv, s, 'latin1', 'utf-8')
    P('l09 iconv-same ' .. name, vim.iconv, s, 'utf-8', 'utf-8')
    P('l09 iconv-bad ' .. name, vim.iconv, s, 'utf-8', 'no-such')
  end
  -- the argument-check arms, which are Lua-side and have no Vimscript
  -- equivalent at all.
  P('l09 bad utfindex-type', vim.str_utfindex, 42)
  P('l09 bad utfindex-enc', vim.str_utfindex, 'abc', 'utf-9', 1)
  P('l09 bad utfindex-neg', vim.str_utfindex, 'abc', -1)
  P('l09 bad utfindex-big', vim.str_utfindex, 'abc', 99)
  P('l09 bad byteindex-neg', vim.str_byteindex, 'abc', -1)
  P('l09 bad byteindex-big', vim.str_byteindex, 'abc', 99)
  P('l09 bad utf_pos-type', vim.str_utf_pos, 42)
  P('l09 bad utf_start-zero', vim.str_utf_start, 'abc', 0)
  P('l09 bad utf_start-big', vim.str_utf_start, 'abc', 99)
  P('l09 bad utf_end-zero', vim.str_utf_end, 'abc', 0)
  P('l09 bad stricmp-num', vim.stricmp, 1, 2)
  P('l09 bad iconv-args', vim.iconv, 'x')
  P('l09 bad iconv-type', vim.iconv, 1, 2, 3)
end)

-- ---------------------------------------------------------------------
-- s10 -- the same questions against a real buffer, where the column
-- functions read mbyte's boundary helpers rather than a Lua string.
-- ---------------------------------------------------------------------

section('s10-buffer', function()
  quiet('silent! %bwipeout!')
  quiet('enew!')
  local lines = {}
  for _, name in ipairs(ORDER) do
    -- A buffer line cannot hold a NL, and the corpus has none; a line of
    -- its own per corpus entry keeps the lnum readable.
    lines[#lines + 1] = C[name]
  end
  vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
  for lnum, name in ipairs(ORDER) do
    local s = C[name]
    vim.api.nvim_win_set_cursor(0, { lnum, 0 })
    P('b10 ' .. name .. ' getline', vim.fn.getline, lnum)
    P('b10 ' .. name .. ' col', function()
      quiet('normal! $')
      return { vim.fn.col('.'), vim.fn.virtcol('.'), vim.fn.charcol('.') }
    end)
    for _, off in ipairs({ 1, 2, 3, 5, 9 }) do
      if off <= #s + 1 then
        P('b10 ' .. name .. ' v' .. off, vim.fn.virtcol, { lnum, off })
        P('b10 ' .. name .. ' vl' .. off, vim.fn.virtcol, { lnum, off }, 1)
        P('b10 ' .. name .. ' c' .. off, vim.fn.charcol, { lnum, off })
        P('b10 ' .. name .. ' b' .. off, vim.fn.col, { lnum, off })
        P('b10 ' .. name .. ' vcx' .. off, vim.fn.virtcol2col, 0, lnum, off)
      end
    end
    P('b10 ' .. name .. ' idxof', function()
      return { vim.fn.byteidx(s, 1), vim.fn.charidx(s, 1), vim.fn.strchars(s) }
    end)
  end
  quiet('silent! %bwipeout!')
end)

-- ---------------------------------------------------------------------
-- s11 -- case-insensitive *comparison*, which is the only front door to
-- utf_strnicmp and therefore to utf_fold.
--
-- `vim.stricmp()` looks like the obvious caller and is not: nlua_stricmp
-- calls libc `strcasecmp`, so nothing in s09 reaches a single line of
-- mbyte's folding.  `==?` goes through mb_strcmp_ic -> mb_stricmp ->
-- mb_strnicmp -> utf_strnicmp -> utf_fold, and so does `sort(l, 'i')`.
-- Mutations on utf_fold's ASCII arm and on utf_strnicmp's byte-wise tail
-- both measured NOT CAUGHT until this section existed.
-- ---------------------------------------------------------------------

section('s11-compare', function()
  local OPS = { '==', '==#', '==?', '!=?', '<?', '>?', '<=?', '>=?', '=~?', '!~?' }
  for _, a in ipairs(ORDER) do
    vim.g.utfa = C[a]
    for _, b in ipairs(CORE) do
      vim.g.utfb = C[b]
      for _, op in ipairs(OPS) do
        E('m11 ' .. a .. ' ' .. b .. ' ' .. tag(op), 'g:utfa ' .. op .. ' g:utfb')
      end
      -- and against the uppercased self, where the fold is the whole
      -- answer rather than an incidental one.
      vim.g.utfb = vim.fn.toupper(C[a])
      E('m11 ' .. a .. ' upper ' .. b, 'g:utfa ==? g:utfb')
    end
    vim.g.utfb = vim.fn.toupper(C[a])
    E('m11 ' .. a .. ' selfupper', 'g:utfa ==? g:utfb')
    E('m11 ' .. a .. ' selfupperx', 'g:utfa ==# g:utfb')
    P('m11 ' .. a .. ' sortic', vim.fn.sort, { C[a], vim.fn.toupper(C[a]), 'a', 'A' }, 'i')
    P('m11 ' .. a .. ' sortdefault', vim.fn.sort, { C[a], vim.fn.toupper(C[a]), 'a', 'A' })
    P('m11 ' .. a .. ' index-ic', vim.fn.index, { 'x', C[a], 'y' }, vim.fn.toupper(C[a]), 0, 1)
    P('m11 ' .. a .. ' count-ic', vim.fn.count, { C[a], vim.fn.toupper(C[a]) }, C[a], 1)
  end
  -- 'ignorecase' also routes `==` itself through the folding comparator.
  for _, ic in ipairs({ 0, 1 }) do
    quiet('set ' .. (ic == 1 and 'ignorecase' or 'noignorecase'))
    for _, a in ipairs(ORDER) do
      vim.g.utfa = C[a]
      vim.g.utfb = vim.fn.toupper(C[a])
      E('m11 ic' .. ic .. ' ' .. a, 'g:utfa == g:utfb')
    end
  end
  quiet('set noignorecase')
  quiet('silent! unlet g:utfa')
  quiet('silent! unlet g:utfb')
end)

-- ---------------------------------------------------------------------
-- s12 -- the `:registers` listing over multibyte and invalid contents.
--
-- The one headless caller of `utf_ptr2cells_len`, mbyte's *length-bounded*
-- width routine: everything else that reaches it is the screen (grid.rs,
-- tui/paint.rs, vterm).  register.rs only calls it for a **blockwise**
-- register, so the shape below -- setreg(..., 'b') then `:registers` --
-- is the whole section's reason to exist.
-- ---------------------------------------------------------------------

section('s12-registers', function()
  for _, name in ipairs(ORDER) do
    local s = C[name]
    for _, kind in ipairs({ 'v', 'V', 'b' }) do
      P('r12 ' .. name .. ' ' .. kind, function()
        vim.fn.setreg('a', { s, 'ascii tail', s .. s }, kind)
        return {
          typ = vim.fn.getregtype('a'),
          list = vim.fn.execute('registers a'),
          get = vim.fn.getreg('a'),
        }
      end)
    end
    -- and a blockwise register whose declared width disagrees with its
    -- contents, which is where the length bound actually bites.
    P('r12 ' .. name .. ' b40', function()
      vim.fn.setreg('a', { s, s, 'x' }, 'b40')
      return { typ = vim.fn.getregtype('a'), list = vim.fn.execute('registers a') }
    end)
    -- The same content as ONE STRING with embedded newlines, which is a
    -- different path into str_to_reg and the only one that reaches
    -- utf_ptr2cells_len's truncated-sequence early out: the line scan
    -- stops at the '\n' while the *length* bound runs to the end of the
    -- whole string, so a sequence cut short by a newline is the only
    -- shape that is both incomplete and length-bounded.  A list argument
    -- does not do this, and the mutation on that arm measured NOT CAUGHT
    -- until these three cases existed.
    for _, tail in ipairs({ '\nxy', '\n' .. s, '\nz\nw' }) do
      P('r12 ' .. name .. ' joined ' .. tag(tail), function()
        vim.fn.setreg('a', s .. tail, 'b')
        return { typ = vim.fn.getregtype('a'), get = vim.fn.getreg('a') }
      end)
    end
  end
  quiet('silent! call setreg("a", "")')
end)

-- ---------------------------------------------------------------------
-- s90 -- the error arms, UNCAPTURED, so .stderr carries signal.
-- ---------------------------------------------------------------------

--- Run a command so that whatever it says reaches the *prompt*, i.e. the
--- .stderr artifact.  A raising command run through the API turns its
--- message into an API error, which pcall then swallows and nothing is
--- recorded at all; the try/catch/echomsg wrapper (diffsweep's) puts the
--- exception back on the prompt where it belongs.
local function loud(cmd)
  pcall(
    vim.api.nvim_exec2,
    table.concat({ 'try', cmd, 'catch', 'echomsg v:exception', 'endtry' }, '\n'),
    { output = false }
  )
end

section('s90-errors', function()
  emit('-- s90: the arms below write to the prompt, i.e. the .stderr artifact')
  loud('echo "-- s90 start"')
  loud('echo printf("%d")')
  loud('echo printf("%d", 1, 2)')
  loud('echo printf("%f", "x")')
  loud('echo printf("%1$d %d", 1)')
  loud('echo printf("%0$d", 1)')
  loud('echo printf("%d", [])')
  loud('echo printf("%d", {})')
  loud('echo printf("%s", function("tr"))')
  loud('echo nr2char(-1)')
  loud('echo list2str([-1])')
  loud('echo list2str(["a"])')
  loud('echo list2str("x")')
  loud('echo str2list(0)')
  loud('echo charclass([])')
  loud('echo setcellwidths(0)')
  loud('echo setcellwidths([[1,2]])')
  loud('echo setcellwidths([[0x20,0x30,2]])')
  loud('echo setcellwidths([[0x110000,0x110000,2]])')
  loud('echo setcellwidths([[0x300,0x400,2],[0x100,0x200,1]])')
  loud('echo iconv("x", "no-such", "utf-8")')
  loud('echo iconv(0, "utf-8", "utf-8")')
  loud('echo strcharpart("abc")')
  loud('echo byteidx()')
  loud('echo strwidth([])')
  loud('echo strdisplaywidth([])')
  loud('echo keytrans(0)')
  loud('echo trim("x", "y", 3)')
  loud('echo tr("abc", "ab", "x")')
  loud('echo strgetchar("abc", -1)')
  loud('edit! ++enc=no-such-encoding ' .. vim.fn.fnameescape(work .. '/files/ascii.txt'))
  loud('write! ++enc=no-such-encoding ' .. vim.fn.fnameescape(work .. '/files/x.out'))
  loud('edit! ++enc=euc-jp ' .. vim.fn.fnameescape(work .. '/files/invalid.txt'))
  loud('echo iconv("' .. C.invalid:gsub('.', function(c)
    return string.format('\\x%02x', c:byte())
  end) .. '", "utf-8", "ascii")')
  loud('echo "-- s90 mid"')
  -- The one probe whose *value* cannot be baselined: %p on a String is
  -- an allocation address.  Asking it here proves the arm runs without
  -- putting an address in an artifact -- the message is what is
  -- compared, and there is none, so only the exit status is.
  loud('call printf("%p", "an allocation")')
  loud('echo "-- s90 end"')
end)

-- ---------------------------------------------------------------------
-- s91 -- CRASHPROBE.  The `%S`-with-a-minimum-field-width matrix, run in
-- a child, because some of it terminates the process.
--
-- `printf('%3S', "\xe9\xe8\xfc")` prints `E41: Out of memory!` and
-- exits: the 'S' arm of vim_vsnprintf_typval finishes with
--
--     if (min_field_width != 0) { min_field_width += str_arg_l - i; }
--
-- where `str_arg_l` is a byte count and `i` a cell count.  Three
-- LATIN-1 bytes display as `<e9><e8><fc>`, i.e. 3 bytes and 12 cells, so
-- the size_t subtraction underflows and min_field_width becomes ~2^64;
-- xmalloc answers that with E41 and preserve_exit().  Upstream's C is
-- byte-identical (v0.12.4 strings.c:1801), so this is not a
-- transpilation artifact -- it is a Vimscript expression that kills the
-- editor, and it is on the docket.
--
-- Run in a child so the abort is one diffable line rather than a
-- truncated report, and *resumable*, so the cases after a fatal one are
-- still measured: the parent restarts the child at the next index.  A
-- killed child reports its signal rather than its code and its stderr
-- carries a pid, so only the status is recorded and only as ABORTED/OK.
-- ---------------------------------------------------------------------

local CRASHCORPUS = {
  'empty',
  'ascii',
  'ctrl',
  'latin1raw',
  'latin1u',
  'cjk',
  'halfkana',
  'comb',
  'emoji',
  'zwj',
  'ambi',
  'invalid',
  'overlong2',
  'surrogate',
  'trunc',
  'contonly',
  'above',
  'fivebyte',
  'tabs',
  'nbsp',
  'mixed',
  'longa',
}

local CRASH = {}
for _, f in ipairs({ '', '-', '0' }) do
  for _, w in ipairs({ '1', '3', '20' }) do
    for _, pr in ipairs({ '', '.3', '.20' }) do
      local fmt = '%' .. f .. w .. pr .. 'S'
      for _, name in ipairs(CRASHCORPUS) do
        CRASH[#CRASH + 1] = { 'k91 ' .. tag(fmt) .. ' ' .. name, fmt, C[name] }
      end
    end
  end
end
-- and the `*` spelling of the same width, which reaches it through a
-- different assignment to min_field_width.
for _, name in ipairs(CRASHCORPUS) do
  CRASH[#CRASH + 1] = { 'k91 star3 ' .. name, '%*S', C[name], 3 }
  CRASH[#CRASH + 1] = { 'k91 starm3 ' .. name, '%*S', C[name], -3 }
end

--- One crash case, printed as `<idx> <label> = <escaped>` with no
--- buffering.  Shared by parent and child so the two agree on the
--- spelling of an answer.
local function crashline(i)
  local c = CRASH[i]
  local ok, res
  if c[4] then
    ok, res = pcall(vim.fn.printf, c[2], c[4], c[3])
  else
    ok, res = pcall(vim.fn.printf, c[2], c[3])
  end
  io.write(i, ' ', c[1], ok and ' = ' or ' E ', esc(scrub(tostring(res))), '\n')
end

section('s91-crashprobe', function()
  local progpath = vim.v.progpath
  local i, guard = 1, 0
  while i <= #CRASH and guard < #CRASH + 40 do
    guard = guard + 1
    local res = vim
      .system({
        progpath,
        '--headless',
        '-u',
        'NONE',
        '-i',
        'NONE',
        '-l',
        script,
        '--child',
        tostring(i),
      }, { text = true })
      :wait()
    local last = i - 1
    for line in (res.stdout or ''):gmatch('[^\n]+') do
      local idx = tonumber(line:match('^(%d+) '))
      if idx then
        last = idx
        emit((line:gsub('^%d+ ', '')))
      end
    end
    if res.code == 0 and last >= #CRASH then
      break
    end
    -- The child stopped early.  Name the case it stopped *on* -- which
    -- is the one after the last it printed -- and resume past it.
    local dead = last + 1
    if dead <= #CRASH then
      emit(CRASH[dead][1], 'ABORTED')
      struct(CRASH[dead][1], { aborted = true })
    end
    i = dead + 1
  end
  emit('k91 groups', tostring(guard))
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 ambiwidth=single tabstop=8 shiftwidth=8 noexpandtab')
quiet('set fileencodings=ucs-bom,utf-8,default,latin1 nobomb')
quiet('set isprint=@,161-255 isfname=@,48-57,/,.,-,_,+,,,#,$,%,~,=')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set noignorecase nosmartcase magic')
quiet('set shell=/bin/sh shellxquote= noshellslash shellcmdflag=-c')
quiet('set nowritebackup nobackup hidden')
quiet('language C')
quiet('silent! cd ' .. vim.fn.fnameescape(work .. '/files'))

-- The child branch is here, *below* the option block, so that a child's
-- answers are computed under the same 'ambiwidth' and 'encoding' as the
-- parent's -- utf_ptr2cells reads both, and s91 is entirely about what
-- it returns.
if child_from then
  for i = child_from, #CRASH do
    crashline(i)
  end
  os.exit(0)
end

emit('encoding', vim.o.encoding)
emit('ambiwidth', vim.o.ambiwidth)
emit('shell', esc(vim.o.shell))
emit('has-iconv', vim.fn.has('iconv'))
emit('has-multi_byte', vim.fn.has('multi_byte'))
emit('corpus', #ORDER, 'entries')

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
      -- UTFSWEEP_TRACE writes into the .stderr artifact and must be off
      -- for a baseline.
      io.stderr:write(string.format('   %s %.1fs\n', entry.name, (vim.uv.hrtime() - started) / 1e9))
    end
  end
end

emit('')
emit('== done ==')
structfd:close()
binfd:close()
