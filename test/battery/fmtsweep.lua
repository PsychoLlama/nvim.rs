-- Driver for the indent/format differential sweep; see
-- fmtsweep.sh.
--
-- Covers indent_c.rs (4,308 lines -- of which `get_c_indent` alone is
-- 1,866, the largest single function in the batch), textformat.rs
-- (1,153) and indent.rs (683).  `test_cindent` is 5,516 lines and has no
-- screendumps, so it is a real gate for the C indenter; nothing at all
-- is a fine-grained gate for `gq`, `'formatoptions'` or the
-- 'formatexpr'/'indentexpr' callbacks, and no differential reaches any
-- of the three files.
--
--   s01 indent() / cindent() / lispindent() per line over every fixture
--       at the default options -- the whole-file indentation profile
--   s02 the 'cinoptions' matrix: 31 letters x several values each,
--       against the fixtures that letter is about
--   s03 'cinwords' / 'cinscopedecls' / 'cinkeys' (in_cinkeys, which is
--       what decides whether a keystroke reindents at all)
--   s04 the `=` operator: `=G`, `==`, `=ap` under 'cindent', 'lisp',
--       'smartindent', 'autoindent' and 'indentexpr'
--   s05 gq / gw over the prose fixtures x 'textwidth' x 'formatoptions'
--   s06 'comments' -- the leader matrix, which is what a comment block
--       reflows against
--   s07 'formatlistpat' and fo+=n / fo+=2
--   s08 auto-wrap while typing: fo+=t / fo+=c / fo+=a / fo+=w, plus
--       'smartindent' and the trailing-blank rules -- internal_format
--       and auto_format, which only a keystroke reaches
--   s09 'formatexpr' and 'indentexpr' -- COUNTER-BUMPING callbacks, so
--       the artifact says whether the callback ran rather than whether
--       the command errored
--   s10 'shiftwidth' / 'tabstop' / 'softtabstop' / 'expandtab' /
--       'vartabstop' / 'varsofttabstop' against indent(), set_indent
--       (via `:left`, `:>`) and get_number_indent
--   s11 lispindent(): 'lisp', 'lispwords', 'lispoptions'
--   s90 the error arms, run UNCAPTURED, so the .stderr artifact carries
--       signal instead of being empty
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time, buffer handle or path outside the
-- work directory.
--
-- FMTSWEEP_ONLY is a Lua pattern matched against each section name.
-- FMTSWEEP_TRACE=1 mirrors each section name to stderr.

local work = assert(os.getenv('FMT_WORK'), 'FMT_WORK unset')


local structfd = assert(io.open(assert(os.getenv('FMT_STRUCT'), 'FMT_STRUCT unset'), 'w'))
local only = os.getenv('FMTSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('FMTSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

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
  structfd:write(label, ' ', canon(value), '\n')
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
    return
  end
  if type(value) == 'string' then
    emit(label, '=', esc(scrub(value)))
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
-- Fixtures.  Named, so a case says which shape it is asking about and
-- two sections asking the same question use the same bytes.  Each one
-- exists for a *particular* group of 'cinoptions' letters or
-- 'formatoptions' flags, and its note says which.
-- ---------------------------------------------------------------------

local FIX = {}
local FIXORDER = {}
local function fixture(name, lines, note)
  FIX[name] = { lines = lines, note = note }
  FIXORDER[#FIXORDER + 1] = name
end

fixture('cbase', {
  '#include <stdio.h>',
  '',
  'static int total;',
  '',
  'int add(int a, int b)',
  '{',
  'int c = a + b;',
  'if (c > 0) {',
  'total += c;',
  '} else if (c < 0) {',
  'total -= c;',
  '} else {',
  'total = 0;',
  '}',
  'while (c--) {',
  'total++;',
  '}',
  'do {',
  'total--;',
  '} while (total > 0);',
  'for (int i = 0; i < 10; i++)',
  'total += i;',
  'return c;',
  '}',
  '',
  'int main(void)',
  '{',
  'return add(1, 2);',
  '}',
}, 'the ordinary shapes: braces, if/else, loops, a one-line body (cino-e n f { } ^ +)')

fixture('cparen', {
  'void f(void)',
  '{',
  'int x = call(a,',
  'b,',
  'c);',
  'int y = (a +',
  'b) *',
  '(c -',
  'd);',
  'if (aaa &&',
  'bbb ||',
  'ccc) {',
  'g();',
  '}',
  'int z = call( a,',
  'b );',
  'char *s = "a (paren in a string",',
  '*t = "another";',
  '}',
}, 'unclosed parentheses at every depth (cino- ( u U w W k m M ))')

fixture('cswitch', {
  'void f(int n)',
  '{',
  'switch (n) {',
  'case 1:',
  'g();',
  'break;',
  'case 2: {',
  'h();',
  'break;',
  '}',
  'default:',
  'i();',
  'break;',
  '}',
  'switch (n) {',
  'case 1:',
  'case 2:',
  'j();',
  '}',
  '}',
}, 'switch/case/default/break, plus a braced case (cino- : = l b)')

fixture('clabel', {
  'int f(void)',
  '{',
  'int rc = 0;',
  'if (rc)',
  'goto done;',
  'rc = 1;',
  'done:',
  'return rc;',
  'nested:;',
  '{',
  'inner:',
  'return 2;',
  '}',
  '}',
}, 'jump labels, which cino-L places (cino-L, and cino-: must not claim them)')

fixture('ccomment', {
  'int f(void)',
  '{',
  '/* a block comment',
  'continued here',
  'and here',
  '*/',
  '/*',
  '* a starred block',
  '*/',
  '// a line comment',
  '// continued',
  'int x = 1; /* trailing */',
  'return x;',
  '}',
  '/* at the top level',
  'still going',
  '*/',
}, 'comment bodies (cino- c C / and find_start_comment)')

fixture('cpre', {
  '#define A 1',
  '#define B \\',
  '2',
  'int f(void)',
  '{',
  '#if A',
  'int x = 1;',
  '#else',
  'int x = 2;',
  '#endif',
  '# pragma once',
  '#pragma pack(1)',
  'return x;',
  '}',
}, 'preprocessor lines and a continuation (cino-# and cino-P)')

fixture('cpp', {
  'namespace outer {',
  'namespace inner {',
  'class Thing : public Base,',
  'private Other',
  '{',
  'public:',
  'Thing();',
  'int value() const;',
  'private:',
  'int v_;',
  'protected:',
  'void helper();',
  '};',
  'Thing::Thing()',
  ': Base(1),',
  'Other(2),',
  'v_(3)',
  '{',
  '}',
  '}',
  '}',
  'extern "C" {',
  'void c_fn(void);',
  '}',
  'template <typename T>',
  'T identity(T t)',
  '{',
  'return t;',
  '}',
}, 'C++ scope declarations, namespaces, linkage, base-class lists (cino- g h N E i j)')

fixture('java', {
  'class Outer {',
  'void run() {',
  'new Thread(new Runnable() {',
  'public void run() {',
  'work();',
  '}',
  '});',
  'list.forEach(item -> {',
  'use(item);',
  '});',
  '}',
  'static {',
  'init();',
  '}',
  '}',
}, 'Java anonymous classes and a static initialiser (cino-j)')

fixture('cs', {
  'class C',
  '{',
  '#region stuff',
  'public int P { get; set; }',
  'public void M()',
  '{',
  'if (P > 0)',
  '{',
  'Console.WriteLine(P);',
  '}',
  'using (var x = Open())',
  '{',
  'x.Go();',
  '}',
  '}',
  '#endregion',
  '}',
}, 'C# regions, property bodies and a using block (cino-# with a brace after)')

fixture('js', {
  'var o = {',
  'a: 1,',
  'b: function () {',
  'return 2;',
  '},',
  'c: [',
  '1,',
  '2,',
  '],',
  '};',
  'if (o.a) {',
  'call(o);',
  '}',
}, 'JavaScript object literals, which cino-J stops reading as blocks')

fixture('ckr', {
  'int f(a, b)',
  'int a;',
  'char *b;',
  '{',
  'return a;',
  '}',
  'static',
  'int',
  'g(void)',
  '{',
  'return 1;',
  '}',
}, 'K&R parameter declarations and a split return type (cino-p and cino-t)')

fixture('cnobrace', {
  'int f(void)',
  '{',
  'if (a)',
  'b();',
  'else',
  'c();',
  'if (a)',
  'if (b)',
  'c();',
  'while (a)',
  'b();',
  'for (;;)',
  'b();',
  'if (a)',
  '{',
  'b();',
  '}',
  '}',
}, 'unbraced bodies, one and two deep (cino-n and cino-{)')

-- The three fixtures below exist because eleven 'cinoptions' letters
-- answered IDENTICALLY for every value against the corpus above: C, M,
-- W, *, c, ), u, w and the `s`/fraction spellings.  An option that is
-- "exercised" everywhere and never changes an answer gates nothing, and
-- a sort -u over the artifact is the only way to find that out.

fixture('cdeep', {
  'void f(void)',
  '{',
  'g(a,',
  'h(b,',
  'i(c,',
  'j(d,',
  'e)',
  ')',
  ')',
  ');',
  'int x = (a +',
  '(b *',
  '(c -',
  'd)',
  ')',
  ');',
  'if (aa &&',
  '(bb ||',
  'cc)',
  ') {',
  'k();',
  '}',
  'call(',
  'arg1,',
  'arg2',
  ');',
  'call2( ',
  'arg',
  ');',
  'int y[] = {',
  '1,',
  '2,',
  '};',
  '}',
}, 'unclosed parens TWO levels deep, lines that START with ) or }, and an opener with nothing after it (cino- u U w W m M ( ))')

fixture('ccomment2', {
  'int f(void)',
  '{',
  '/*',
  'a body line with no text after the opener',
  'a second body line',
  '*/',
  '/* text on the opener line',
  'a body line under text',
  'a second body line',
  '*/',
  '/* one-liner */',
  '/*',
  '*/',
  '/*',
  '',
  'after a blank line',
  '*/',
  'return 0;',
  '}',
}, "a comment with NOTHING after /* and one WITH text, which is the whole difference between cino-c and cino-C")

fixture('clong', (function()
  local out = { 'int f(void)', '{', 'g(a,' }
  for i = 1, 90 do
    out[#out + 1] = 'arg' .. i .. ','
  end
  out[#out + 1] = 'last);'
  out[#out + 1] = '/*'
  for i = 1, 90 do
    out[#out + 1] = 'comment body ' .. i
  end
  out[#out + 1] = '*/'
  out[#out + 1] = 'return 0;'
  out[#out + 1] = '}'
  return out
end)(), 'a 90-line unclosed paren and a 90-line comment: cino-) and cino-* are SEARCH LIMITS and only a region longer than the limit can move them')

fixture('lisp', {
  '(define (fact n)',
  '(if (= n 0)',
  '1',
  '(* n (fact (- n 1)))))',
  '',
  '(let ((a 1)',
  '(b 2))',
  '(+ a b))',
  '',
  "(cond ((null? x) '())",
  '(else',
  '(cons 1 x)))',
  '',
  '(setq foo',
  'bar)',
  '',
  '; a comment',
  '(list "a string with ( paren"',
  '2)',
}, 'the lispindent corpus: define/let/cond, a quoted paren, a comment')

-- Prose, for the format half.

fixture('prose', {
  'The quick brown fox jumps over the lazy dog and keeps on jumping for a good long while after that.',
  'A second sentence that is also quite long and will need to be wrapped by the formatter somewhere.',
  '',
  'A short one.',
  'Another paragraph starts here and runs on and on and on without any particular punctuation to help.',
}, 'plain paragraphs for gq at several textwidths')

fixture('bullets', {
  '- a bulleted item that goes on for quite a long time and will certainly need to be wrapped somewhere',
  '- a second item',
  '1. a numbered item that is also long enough to need wrapping when the textwidth is small enough',
  '2. another',
  '  a) a lettered sub-item which is long enough to wrap as well when the width is small',
  '* a star item, long enough to wrap under any reasonable textwidth setting we might choose here',
  '10) a parenthesised number, long enough to wrap under the settings this sweep uses for the tests',
}, "'formatlistpat' and fo+=n: the leader has to be repeated on the wrapped line")

fixture('comment', {
  '// a line comment that is long enough that it will have to be wrapped somewhere by the formatter',
  '// a second line of the same comment',
  '/* a block comment that is also long enough to require wrapping when the textwidth is small */',
  '/* a starred block',
  ' * with a middle line that is long enough to be wrapped by gq under a small textwidth setting',
  ' */',
  '# a hash comment that is long enough to be wrapped when the textwidth is set to something small',
  '> a quoted line from an email, long enough that it will need to be wrapped by the formatter too',
  '>> a doubly quoted line, also long enough to need wrapping under the settings used in this sweep',
}, "'comments' leaders: three-piece, one-line, and the nested quote flag")

fixture('numcomment', {
  '// 1. a numbered item inside a line comment, long enough that it will need to be wrapped',
  '// 2. a second numbered item, also long enough to require wrapping at the widths used here',
  ' * 10) a numbered item inside a block comment body, long enough to wrap under these settings',
  '# a) a lettered item after a hash leader which is long enough to wrap when the width is small',
  '> 1. a numbered item inside a quote, long enough to be wrapped by the formatter at 25 or 40',
}, "a list leader BEHIND a comment leader: get_number_indent measures the number's indent past the comment, and only fo+=n with 'comments' reaches that")

fixture('mbprose', {
  '\230\151\165\230\156\172\232\170\158\227\129\174\227\131\134\227\130\173\227\130\185\227\131\136\227\129\140\227\129\130\227\130\138\227\129\190\227\129\153\227\128\130\227\129\147\227\130\140\227\129\175\230\138\152\227\130\138\232\191\148\227\129\151\227\129\174\227\131\134\227\130\185\227\131\136\227\129\167\227\129\153\227\128\130',
  'mixed ascii and \230\151\165\230\156\172\232\170\158 in one line which is long enough to need to wrap somewhere',
  'a\204\129 line with combining marks e\204\129 and an emoji \240\159\152\128 that is long enough to wrap here too',
}, 'multibyte prose: utf_allow_break_before/after decide where a wrap can land')

fixture('indented', {
  '    an indented paragraph that is long enough to be wrapped and whose indent must be preserved here',
  '    continuing on the second line with more text that also needs to be wrapped by the formatter',
  '',
  '\ta tab-indented paragraph, long enough to wrap, whose leading tab is what fo+=2 and autoindent read',
  '\tsecond line of the tab-indented paragraph, also long enough to require wrapping at these widths',
}, "leading whitespace, for 'autoindent', fo+=2 and the second-line indent rule")

fixture('trailing', {
  'a line with trailing spaces   ',
  'a line with a trailing tab\t',
  'a paragraph line that is long and ends in a space so that fo+=w marks it as continuing onward ',
  'a paragraph line that does not end in a space',
}, "fo+=w, which makes a trailing space mean 'this paragraph continues'")

-- ---------------------------------------------------------------------
-- The world.
-- ---------------------------------------------------------------------

--- Every option any case reads, put back to a known value.  A sweep that
--- lets one case's 'cinoptions' reach the next is a sweep whose baseline
--- moves when a case is inserted above.
local DEFAULTS = table.concat({
  'setlocal cinoptions= cinkeys=0{,0},0),0],:,0#,!^F,o,O,e cinwords=if,else,while,do,for,switch',
  'setlocal cinscopedecls=public,protected,private',
  'setlocal formatoptions=tcq textwidth=0 comments=s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-',
  'setlocal formatlistpat=^\\\\s*\\\\d\\\\+[\\\\]:.)}\\\\t\\ ]\\\\s*',
  'setlocal shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab noshiftround',
  'setlocal vartabstop= varsofttabstop=',
  'setlocal nocindent nosmartindent noautoindent nolisp lispwords& lispoptions&',
  'setlocal indentexpr= formatexpr= equalprg= formatprg= indentkeys&',
  'setlocal comments& define& include&',
}, '\n')

local function reset(lines)
  quiet('silent! %bwipeout!')
  quiet('enew!')
  quiet(DEFAULTS)
  quiet('setlocal noswapfile nomodeline buftype= filetype=')
  if lines then
    vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
  end
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
end

local function opts(list)
  for _, o in ipairs(list or {}) do
    quiet('setlocal ' .. o)
  end
end

--- The indentation profile of the whole buffer, three ways.  `indent()`
--- is what the lines *have*; `cindent()` and `lispindent()` are what the
--- two engines say they *should* have, computed without changing
--- anything -- which is why one case can ask all three.
local function profile(label, want)
  local n = vim.api.nvim_buf_line_count(0)
  local have, cind, lind = {}, {}, {}
  for l = 1, n do
    have[l] = vim.fn.indent(l)
    if want ~= 'lisp' then
      local ok, v = pcall(vim.fn.cindent, l)
      cind[l] = ok and v or 'E'
    end
    if want ~= 'c' then
      local ok, v = pcall(vim.fn.lispindent, l)
      lind[l] = ok and v or 'E'
    end
  end
  ans(label .. ' I', true, table.concat(have, ','))
  if want ~= 'lisp' then
    ans(label .. ' C', true, table.concat(cind, ','))
  end
  if want ~= 'c' then
    ans(label .. ' L', true, table.concat(lind, ','))
  end
end

--- The buffer itself, newline-escaped to one line, plus the cursor.
--- Used by every case that *changes* the buffer.
local function text(label)
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local c = vim.api.nvim_win_get_cursor(0)
  ans(label .. ' B', true, table.concat(lines, '\n'))
  ans(label .. ' S', true, string.format('c=%d,%d t=%d', c[1], c[2], vim.b.changedtick))
end

--- Run a key sequence against a fresh copy of a fixture.  `nvim_feedkeys`
--- with 'ntx' is opsweep's spelling and behaves the same here: the `t`
--- makes an undo block, the `x` runs it to completion before returning.
local function keys(seq)
  vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(seq, true, true, true), 'ntx', false)
end

-- ---------------------------------------------------------------------
-- s01 -- the indentation profile of every fixture at the defaults
-- ---------------------------------------------------------------------

section('s01-profile', function()
  for _, name in ipairs(FIXORDER) do
    emit('-- ' .. name .. ': ' .. FIX[name].note)
    reset(FIX[name].lines)
    profile('p01 ' .. name)
    -- and again with the fixture already indented by the engine, which
    -- is the shape every *second* call sees: get_c_indent reads the
    -- lines above the one it is asked about.
    reset(FIX[name].lines)
    opts({ 'cindent' })
    keys('gg=G')
    text('p01 ' .. name .. ' eq')
    profile('p01 ' .. name .. ' eq')
  end
end)

-- ---------------------------------------------------------------------
-- s02 -- the 'cinoptions' matrix.  31 letters, several values each.
--
-- `get_c_indent` is 1,866 lines and every one of these letters selects a
-- different path through it; this section is the only fine-grained gate
-- it has.  Each letter is run against the fixtures whose shapes it is
-- about, plus `cbase`, so that a letter with no effect on a shape is
-- visible as an identical row rather than as an absent one.
-- ---------------------------------------------------------------------

local CINO = {
  { '>', { '0', '2', '4', '8', '-2', '2s', '.5s' }, { 'cbase', 'cnobrace' } },
  { 'e', { '0', '2', '-2', '4' }, { 'cbase', 'cparen' } },
  { 'n', { '0', '2', '-2', '4' }, { 'cnobrace', 'cbase' } },
  { 'f', { '0', '2', '4', '-2', 's' }, { 'cbase', 'ckr' } },
  { '{', { '0', '2', '4', '-2' }, { 'cbase', 'cnobrace' } },
  { '}', { '0', '2', '-2', '4' }, { 'cbase', 'cswitch' } },
  { '^', { '0', '2', '-2' }, { 'cbase', 'cpp' } },
  { 'L', { '-1', '0', '1', '4', '-4' }, { 'clabel', 'cswitch' } },
  { ':', { '0', '2', '4', '8', '-2' }, { 'cswitch', 'clabel' } },
  { '=', { '0', '2', '4', '-2' }, { 'cswitch' } },
  { 'l', { '0', '1' }, { 'cswitch' } },
  { 'b', { '0', '1' }, { 'cswitch' } },
  { 'g', { '0', '2', '4', '-2' }, { 'cpp' } },
  { 'h', { '0', '2', '4', '-2' }, { 'cpp' } },
  { 'N', { '0', '2', '-2', '4' }, { 'cpp' } },
  { 'E', { '0', '2', '-2', '4' }, { 'cpp' } },
  { 'p', { '0', '2', '4', '-2' }, { 'ckr' } },
  { 't', { '0', '2', '4', '-2' }, { 'ckr' } },
  { 'i', { '0', '2', '4', '-2' }, { 'cpp' } },
  { '+', { '0', '2', '4', '-2' }, { 'cdeep', 'cparen', 'cpre' } },
  { 'c', { '0', '2', '3', '-2' }, { 'ccomment2', 'ccomment' } },
  -- cino-C only does anything when cino-c is in the same string AND
  -- 'comments' treats `/*` as a `s0:` three-piece opener; with the
  -- default `s1:/*` it answered identically for both values.  The doc's
  -- own example says as much and this is the only entry that needs it.
  { 'C', { '0', '1' }, { 'ccomment2', 'ccomment' }, 'c0,', 'comments-=s1:/*', 'comments^=s0:/*' },
  { '/', { '0', '2', '4' }, { 'ccomment2', 'ccomment' } },
  { '(', { '0', '2', '4', '-2', '2s' }, { 'cdeep', 'cparen' } },
  { 'u', { '0', '2', '4', '-2' }, { 'cdeep', 'cparen' } },
  { 'U', { '0', '1' }, { 'cdeep', 'cparen' } },
  -- cino-w and cino-W are conditional on `(0` or `u0` being in the SAME
  -- 'cinoptions' string -- "either using (0 or u0" -- so spelled alone
  -- they are dead letters, and they measured exactly that.
  { 'w', { '0', '1' }, { 'cdeep', 'cparen' }, '(0,' },
  { 'w', { '0', '1' }, { 'cdeep', 'cparen' }, 'u0,' },
  { 'W', { '0', '2', '4' }, { 'cdeep', 'cparen' }, '(0,' },
  { 'W', { '0', '2', '4' }, { 'cdeep', 'cparen' }, 'u0,' },
  { 'k', { '0', '2', '4' }, { 'cdeep', 'cparen' } },
  { 'm', { '0', '1' }, { 'cdeep', 'cparen' } },
  { 'M', { '0', '1' }, { 'cdeep', 'cparen' } },
  { 'j', { '0', '1' }, { 'java', 'cpp' } },
  { 'J', { '0', '1' }, { 'js', 'java' } },
  { ')', { '20', '0', '3', '200' }, { 'clong', 'cdeep' } },
  { '*', { '70', '0', '3', '200' }, { 'clong', 'ccomment2' } },
  { '#', { '0', '1', '2' }, { 'cpre', 'cs' } },
  { 'P', { '0', '1' }, { 'cpre' } },
}

section('s02-cinoptions', function()
  for _, entry in ipairs(CINO) do
    local letter, values, fixtures = entry[1], entry[2], entry[3]
    -- entry[4] is a 'cinoptions' PREFIX and entry[5..] are extra
    -- `setlocal` arguments: three letters do nothing unless another
    -- letter or another option is set with them, and a letter that
    -- cannot move the artifact is coverage that gates nothing.
    local prefix = entry[4] or ''
    local extra = {}
    for i = 5, #entry do
      extra[#extra + 1] = entry[i]
    end
    for _, v in ipairs(values) do
      local spelling = prefix .. letter .. v
      for _, fname in ipairs(fixtures) do
        reset(FIX[fname].lines)
        opts(extra)
        opts({ 'cindent', 'cinoptions=' .. vim.fn.escape(spelling, ' \\|"') })
        profile('o02 ' .. tag(spelling) .. ' ' .. fname, 'c')
        keys('gg=G')
        text('o02 ' .. tag(spelling) .. ' ' .. fname)
      end
    end
  end
  -- combinations, because parse_cino reads the whole string in one pass
  -- and a letter's parser can eat the next letter's digits.
  for _, combo in ipairs({
    '>4,e2,n-2,f0,{2,}2,^-2,:0,=2,l1,b1,g2,h2,p2,t2,i4,+4,c3,C1,/2,(0,u0,U1,w1,W4,k2,m1,M1,j1,J1,)30,*40,#1,P1',
    '>2s,e.5s,f2s,(2s,us',
    '>-4,e-4,n-4,f-4,{-4,}-4,^-4,:-4,=-4,g-4,h-4,p-4,t-4,i-4,+-4,c-4,/-4,(-4,u-4',
    'l1,b1,:0,=0,L0',
    'N-s,E-s,g0,h0',
    ')1,*1',
    '>1.5s,(1.25s',
  }) do
    for _, fname in ipairs({ 'cbase', 'cparen', 'cswitch', 'cpp', 'ccomment' }) do
      reset(FIX[fname].lines)
      opts({ 'cindent', 'cinoptions=' .. vim.fn.escape(combo, ' \\|"') })
      ans('o02 combo ' .. tag(combo) .. ' ' .. fname .. ' cino', true, vim.bo.cinoptions)
      profile('o02 combo ' .. tag(combo) .. ' ' .. fname, 'c')
    end
  end
  -- and the spellings parse_cino has to survive rather than obey.
  --
  -- NOTE the absence of anything outside int range: `cinoptions=>2147483648`
  -- ABORTS the process (getdigits_int's assert, charset.rs:586) and would
  -- take the rest of the report with it.  s91 runs those in children.
  for _, bad in ipairs({ '>', 'q4', '4', ',', '>,', '>4,,e2', '>2147483647', '>-2147483647', '>.', '>0.0s', '>s4' }) do
    reset(FIX.cbase.lines)
    local err = quiet('setlocal cindent cinoptions=' .. vim.fn.escape(bad, ' \\|"'))
    ans('o02 bad ' .. tag(bad), true, { err = err, cino = vim.bo.cinoptions })
    profile('o02 bad ' .. tag(bad), 'c')
  end
end)

-- ---------------------------------------------------------------------
-- s03 -- 'cinwords', 'cinscopedecls' and 'cinkeys'
-- ---------------------------------------------------------------------

section('s03-cinwords', function()
  for _, words in ipairs({
    'if,else,while,do,for,switch',
    '',
    'if',
    'unless,until',
    'if,else,while,do,for,switch,try,catch,finally',
    'IF,ELSE',
  }) do
    for _, fname in ipairs({ 'cnobrace', 'cbase', 'cswitch' }) do
      reset(FIX[fname].lines)
      opts({ 'cindent', 'cinoptions=n2', 'cinwords=' .. vim.fn.escape(words, ' \\|"') })
      profile('w03 ' .. tag(words) .. ' ' .. fname, 'c')
    end
  end
  for _, decls in ipairs({
    'public,protected,private',
    '',
    'public',
    'public,protected,private,signals,slots',
    'PUBLIC',
  }) do
    reset(FIX.cpp.lines)
    opts({ 'cindent', 'cinoptions=g2,h2', 'cinscopedecls=' .. vim.fn.escape(decls, ' \\|"') })
    profile('w03 decls ' .. tag(decls), 'c')
  end
end)

section('s03b-cinkeys', function()
  -- in_cinkeys() decides whether a typed character reindents the line at
  -- all, and the only way to ask it is to type.  Each case types the
  -- trigger and reports what the buffer looks like afterwards.
  local TRIGGERS = {
    { 'brace-open', 'o{\27' },
    { 'brace-close', 'o}\27' },
    { 'paren-close', 'o)\27' },
    { 'bracket-close', 'o]\27' },
    { 'colon', 'olabel:\27' },
    { 'hash', 'o#define X\27' },
    { 'ctrl-f', 'o    x\6\27' },
    { 'word-else', 'oelse\27' },
    { 'word-while', 'owhile\27' },
    { 'newline-o', 'oint x;\27' },
    { 'newline-O', 'Oint x;\27' },
    { 'star-slash', 'o*/\27' },
    { 'equals', 'ox = 1;\27' },
  }
  for _, ck in ipairs({
    '0{,0},0),0],:,0#,!^F,o,O,e',
    '',
    '0{,0}',
    ':',
    '0#',
    '!^F,o,O',
    'e',
    '*<Return>',
    '=else,=while,=end',
    '0=break,0=case',
    ';',
  }) do
    for _, t in ipairs(TRIGGERS) do
      reset(FIX.cbase.lines)
      opts({ 'cindent', 'cinkeys=' .. vim.fn.escape(ck, ' \\|"') })
      vim.api.nvim_win_set_cursor(0, { 10, 0 })
      keys(t[2])
      text('k03 ' .. tag(ck) .. ' ' .. t[1])
    end
  end
end)

-- ---------------------------------------------------------------------
-- s04 -- the `=` operator and the four indent sources
-- ---------------------------------------------------------------------

section('s04-equal', function()
  local MODES = {
    { 'plain', {} },
    { 'cindent', { 'cindent' } },
    { 'smartindent', { 'smartindent' } },
    { 'autoindent', { 'autoindent' } },
    { 'ai-si', { 'autoindent', 'smartindent' } },
    { 'lisp', { 'lisp' } },
    { 'indentexpr', { 'indentexpr=IndentProbe()' } },
    { 'cindent-ie', { 'cindent', 'indentexpr=IndentProbe()' } },
  }
  quiet(table.concat({
    'function! IndentProbe() abort',
    '  let g:iexcount = get(g:, "iexcount", 0) + 1',
    '  let g:iexlnum = v:lnum',
    '  return v:lnum % 3 * 2',
    'endfunction',
  }, '\n'))
  for _, m in ipairs(MODES) do
    for _, fname in ipairs(FIXORDER) do
      for _, seq in ipairs({ 'gg=G', 'gg==', 'gg=ap', 'Gk==', 'ggjj=j' }) do
        reset(FIX[fname].lines)
        quiet('let g:iexcount = 0')
        opts(m[2])
        keys(seq)
        text('e04 ' .. m[1] .. ' ' .. fname .. ' ' .. tag(seq))
        ans(
          'e04 ' .. m[1] .. ' ' .. fname .. ' ' .. tag(seq) .. ' cb',
          true,
          { iex = vim.g.iexcount, lnum = vim.g.iexlnum }
        )
      end
    end
  end
  quiet('delfunction! IndentProbe')
end)

-- ---------------------------------------------------------------------
-- s05 -- gq / gw over the prose fixtures
-- ---------------------------------------------------------------------

local FO = {
  'tcq',
  '',
  't',
  'c',
  'q',
  'tcqj',
  'tcqn',
  'tcq2',
  'tcqw',
  'tcqa',
  'tcqaw',
  'tcqr',
  'tcqo',
  'tcqm',
  'tcqM',
  'tcqB',
  'tcq1',
  'tcqb',
  'tcql',
  'tcqv',
  'tcqp',
  'croql',
  'jcroql',
}

section('s05-gq', function()
  for _, tw in ipairs({ 0, 1, 10, 25, 40, 79, 200 }) do
    for _, fo in ipairs(FO) do
      for _, fname in ipairs({ 'prose', 'bullets', 'comment', 'mbprose', 'indented', 'trailing' }) do
        for _, seq in ipairs({ 'ggVGgq', 'ggVGgw', 'gggqG', 'gqip', 'gwip' }) do
          reset(FIX[fname].lines)
          opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo })
          keys(seq)
          text('q05 tw' .. tw .. ' ' .. tag(fo) .. ' ' .. fname .. ' ' .. tag(seq))
        end
      end
    end
  end
  -- 'wrapmargin' is the other input to comp_textwidth, and only applies
  -- when 'textwidth' is zero.
  for _, wm in ipairs({ 0, 20, 60, 79 }) do
    for _, tw in ipairs({ 0, 40 }) do
      reset(FIX.prose.lines)
      opts({ 'textwidth=' .. tw, 'formatoptions=tcq' })
      quiet('setlocal wrapmargin=' .. wm)
      keys('ggVGgq')
      text('q05 wm' .. wm .. ' tw' .. tw)
    end
  end
  quiet('setlocal wrapmargin=0')
end)

-- ---------------------------------------------------------------------
-- s06 -- 'comments'
-- ---------------------------------------------------------------------

section('s06-comments', function()
  local COMMENTS = {
    's1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-',
    '',
    '://',
    's1:/*,mb:*,ex:*/',
    'n:>',
    'n:>,n:|',
    'b:#',
    ':#',
    'f:-',
    'fb:-',
    'sr:/*,mb:*,ex:*/',
    'sO:*\\ -,mO:*\\ \\ ,exO:*/',
    'l:*',
    'r:*',
    'O://',
    'nb:>',
    ':;',
    'b:*',
  }
  for _, c in ipairs(COMMENTS) do
    for _, tw in ipairs({ 0, 25, 40 }) do
      for _, fname in ipairs({ 'comment', 'bullets', 'prose' }) do
        reset(FIX[fname].lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=tcqr', 'comments=' .. vim.fn.escape(c, ' \\|"') })
        keys('ggVGgq')
        text('c06 ' .. tag(c) .. ' tw' .. tw .. ' ' .. fname)
        -- and the insert-mode half: `o` on a comment line repeats the
        -- leader, which is get_leader_len rather than the formatter.
        reset(FIX[fname].lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=tcqro', 'comments=' .. vim.fn.escape(c, ' \\|"') })
        keys('ggoNEW\27')
        text('c06 ' .. tag(c) .. ' tw' .. tw .. ' ' .. fname .. ' o')
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s07 -- 'formatlistpat' with fo+=n and fo+=2
-- ---------------------------------------------------------------------

section('s07-listpat', function()
  local PATS = {
    '^\\s*\\d\\+[\\]:.)}\\t ]\\s*',
    '^\\s*[-*+]\\s\\+',
    '^\\s*\\a[\\])]\\s*',
    '^\\s*\\d\\+\\.\\s*',
    '^',
    '^\\s*',
    'nomatch',
  }
  for _, p in ipairs(PATS) do
    for _, fo in ipairs({ 'tcqn', 'tcq2', 'tcqn2', 'tcq' }) do
      for _, tw in ipairs({ 25, 40 }) do
        reset(FIX.bullets.lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo })
        quiet('let &l:formatlistpat = ' .. vim.fn.string(p))
        keys('ggVGgq')
        text('l07 ' .. tag(p) .. ' ' .. tag(fo) .. ' tw' .. tw)
        -- the same, over a list whose leader sits BEHIND a comment
        -- leader: get_number_indent only skips the comment when it is
        -- inserting or when fo has `q`, and that test is invisible
        -- against a fixture with no comments in it.
        reset(FIX.numcomment.lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo })
        quiet('let &l:formatlistpat = ' .. vim.fn.string(p))
        keys('ggVGgq')
        text('l07 com ' .. tag(p) .. ' ' .. tag(fo) .. ' tw' .. tw)
        reset(FIX.numcomment.lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo .. 'ro' })
        quiet('let &l:formatlistpat = ' .. vim.fn.string(p))
        keys('ggA MORE WORDS HERE TO PUSH IT WELL OVER THE MARGIN\27')
        text('l07 ins ' .. tag(p) .. ' ' .. tag(fo) .. ' tw' .. tw)
        reset(FIX.indented.lines)
        opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo, 'autoindent' })
        quiet('let &l:formatlistpat = ' .. vim.fn.string(p))
        keys('ggVGgq')
        text('l07 ind ' .. tag(p) .. ' ' .. tag(fo) .. ' tw' .. tw)
      end
    end
  end
end)

-- ---------------------------------------------------------------------
-- s08 -- auto-wrap while typing.  internal_format and auto_format are
-- only reachable from a keystroke; `gq` runs format_lines instead.
-- ---------------------------------------------------------------------

section('s08-autowrap', function()
  local TYPED = {
    { 'plain', 'A and then some more words that will certainly cross the margin at these widths\27' },
    { 'newpara', 'Gothis is a brand new line typed from scratch and long enough to wrap somewhere\27' },
    { 'midline', 'ggwiINSERTED TEXT THAT IS LONG ENOUGH TO PUSH THE LINE OVER THE MARGIN \27' },
    { 'space', 'A word word word word word word word word word word word word word word \27' },
    { 'nospace', 'Axxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\27' },
    { 'comment', 'ggA more comment text here that is long enough to be wrapped by the formatter\27' },
    { 'backspace', 'A word word word word word word word word\8\8\8\8 more words here now\27' },
    { 'cr', 'Aone two three\13four five six seven eight nine ten eleven twelve thirteen\27' },
    { 'mb', 'A \230\151\165\230\156\172\232\170\158\227\129\174\227\131\134\227\130\173\227\130\185\227\131\136\227\129\140\227\129\130\227\130\138\227\129\190\227\129\153\27' },
  }
  for _, tw in ipairs({ 0, 20, 40 }) do
    for _, fo in ipairs({ 'tcq', '', 't', 'tcqa', 'tcqaw', 'tcqw', 'tcql', 'tcqm', 'tcqM', 'tcqv', 'tcqb', 'tcq1', 'tcqB', 'croql', 'tcqro' }) do
      for _, fname in ipairs({ 'prose', 'comment', 'bullets', 'mbprose' }) do
        for _, t in ipairs(TYPED) do
          reset(FIX[fname].lines)
          opts({ 'textwidth=' .. tw, 'formatoptions=' .. fo })
          keys(t[2])
          text('a08 tw' .. tw .. ' ' .. tag(fo) .. ' ' .. fname .. ' ' .. t[1])
        end
      end
    end
  end
  -- 'smartindent' and 'autoindent' change what the new line starts with,
  -- which is open_line's half of the same path.
  for _, m in ipairs({ 'noautoindent nosmartindent', 'autoindent', 'smartindent', 'autoindent smartindent' }) do
    for _, fname in ipairs({ 'cbase', 'prose', 'indented' }) do
      reset(FIX[fname].lines)
      opts({ 'textwidth=40', 'formatoptions=tcq' })
      quiet('setlocal ' .. m)
      keys('ggoNEW LINE\27')
      text('a08 ' .. tag(m) .. ' ' .. fname .. ' o')
      keys('GoTAIL\27')
      text('a08 ' .. tag(m) .. ' ' .. fname .. ' G')
    end
  end
  -- 'paste' turns EVERY 'formatoptions' flag off at once -- it is the
  -- first line of has_format_option and nothing else in the sweep
  -- reaches it.  `set paste` also clobbers 'textwidth' and friends
  -- globally, so it is last in the section and `nopaste` is restored
  -- immediately; reset() re-applies the local defaults per case anyway.
  for _, pst in ipairs({ 'nopaste', 'paste' }) do
    for _, fname in ipairs({ 'prose', 'comment', 'bullets' }) do
      reset(FIX[fname].lines)
      opts({ 'textwidth=30', 'formatoptions=tcqa' })
      quiet('set ' .. pst)
      ans('a08 ' .. pst .. ' ' .. fname .. ' opts', true, {
        tw = vim.bo.textwidth,
        fo = vim.bo.formatoptions,
        ai = vim.bo.autoindent and 1 or 0,
      })
      keys('A and more words to push it over the margin\27')
      text('a08 ' .. pst .. ' ' .. fname)
    end
  end
  quiet('set nopaste')
end)

-- ---------------------------------------------------------------------
-- s09 -- 'formatexpr' and 'indentexpr'.
--
-- Both options are *expressions*, not commands: they go through eval1(),
-- so `call Fn()` is E121 and a section that only checks "did the command
-- error" reports a clean run for an option that never fired.  Every
-- callback here bumps a counter and records the v: variables it was
-- handed, and the artifact carries the counter.
-- ---------------------------------------------------------------------

section('s09-callbacks', function()
  quiet(table.concat({
    'function! FexNoop() abort',
    '  let g:fex = get(g:, "fex", 0) + 1',
    '  let g:fexargs = [v:lnum, v:count, v:char]',
    '  return 1',
    'endfunction',
    'function! FexInternal() abort',
    '  let g:fex = get(g:, "fex", 0) + 1',
    '  let g:fexargs = [v:lnum, v:count, v:char]',
    '  return 0',
    'endfunction',
    'function! FexUpper() abort',
    '  let g:fex = get(g:, "fex", 0) + 1',
    '  let g:fexargs = [v:lnum, v:count, v:char]',
    '  for l in range(v:lnum, v:lnum + v:count - 1)',
    '    call setline(l, toupper(getline(l)))',
    '  endfor',
    '  return 0',
    'endfunction',
    'function! FexThrow() abort',
    '  let g:fex = get(g:, "fex", 0) + 1',
    '  throw "fex-failed"',
    'endfunction',
    'function! IexConst() abort',
    '  let g:iex = get(g:, "iex", 0) + 1',
    '  let g:iexargs = [v:lnum]',
    '  return 4',
    'endfunction',
    'function! IexMinusOne() abort',
    '  let g:iex = get(g:, "iex", 0) + 1',
    '  let g:iexargs = [v:lnum]',
    '  return -1',
    'endfunction',
    'function! IexCindent() abort',
    '  let g:iex = get(g:, "iex", 0) + 1',
    '  let g:iexargs = [v:lnum]',
    '  return cindent(v:lnum)',
    'endfunction',
    'function! IexThrow() abort',
    '  let g:iex = get(g:, "iex", 0) + 1',
    '  throw "iex-failed"',
    'endfunction',
    'function! IexEdit() abort',
    '  let g:iex = get(g:, "iex", 0) + 1',
    '  call append(v:lnum, "INSERTED BY INDENTEXPR")',
    '  return 2',
    'endfunction',
  }, '\n'))

  local FEX = {
    { 'unset', '' },
    { 'noop', 'FexNoop()' },
    { 'internal', 'FexInternal()' },
    { 'upper', 'FexUpper()' },
    { 'throw', 'FexThrow()' },
    -- the spelling that looks right and is not: 'formatexpr' is an
    -- expression, so a command here is E121 and the fallback runs.
    { 'command', 'call FexNoop()' },
    { 'undefined', 'NoSuchFunction()' },
    { 'literal', '1' },
    { 'literal0', '0' },
  }
  for _, f in ipairs(FEX) do
    for _, seq in ipairs({ 'ggVGgq', 'gqip', 'ggVGgw', 'gggqG' }) do
      for _, fname in ipairs({ 'prose', 'comment', 'bullets' }) do
        reset(FIX[fname].lines)
        quiet('let g:fex = 0 | let g:fexargs = []')
        opts({ 'textwidth=40', 'formatoptions=tcq' })
        quiet('let &l:formatexpr = ' .. vim.fn.string(f[2]))
        keys(seq)
        text('f09 ' .. f[1] .. ' ' .. fname .. ' ' .. tag(seq))
        ans(
          'f09 ' .. f[1] .. ' ' .. fname .. ' ' .. tag(seq) .. ' cb',
          true,
          { ran = vim.g.fex, args = vim.g.fexargs }
        )
      end
    end
    -- and the auto-wrap path, which calls the same option from
    -- internal_format rather than from op_format.
    reset(FIX.prose.lines)
    quiet('let g:fex = 0 | let g:fexargs = []')
    opts({ 'textwidth=30', 'formatoptions=tcqa' })
    quiet('let &l:formatexpr = ' .. vim.fn.string(f[2]))
    keys('A and more words to push it over the margin here\27')
    text('f09 ' .. f[1] .. ' auto')
    ans('f09 ' .. f[1] .. ' auto cb', true, { ran = vim.g.fex, args = vim.g.fexargs })
  end

  local IEX = {
    { 'unset', '' },
    { 'const', 'IexConst()' },
    { 'minusone', 'IexMinusOne()' },
    { 'cindent', 'IexCindent()' },
    { 'throw', 'IexThrow()' },
    { 'edit', 'IexEdit()' },
    { 'command', 'call IexConst()' },
    { 'undefined', 'NoSuchIndent()' },
    { 'literal', '6' },
    { 'lnum', 'v:lnum * 2' },
  }
  for _, i in ipairs(IEX) do
    for _, seq in ipairs({ 'gg=G', 'gg==', 'ggo x\27', 'ggO y\27' }) do
      for _, fname in ipairs({ 'cbase', 'prose' }) do
        reset(FIX[fname].lines)
        quiet('let g:iex = 0 | let g:iexargs = []')
        opts({ 'autoindent' })
        quiet('let &l:indentexpr = ' .. vim.fn.string(i[2]))
        quiet('setlocal indentkeys=0{,0},:,0#,!^F,o,O,e')
        keys(seq)
        text('i09 ' .. i[1] .. ' ' .. fname .. ' ' .. tag(seq))
        ans(
          'i09 ' .. i[1] .. ' ' .. fname .. ' ' .. tag(seq) .. ' cb',
          true,
          { ran = vim.g.iex, args = vim.g.iexargs }
        )
      end
    end
  end
  -- 'indentkeys' decides which typed characters call 'indentexpr' at
  -- all -- the same in_cinkeys() that 'cinkeys' feeds.
  for _, ik in ipairs({ '0{,0},:,0#,!^F,o,O,e', '', 'o', 'O', ':', '0#', '*<Return>', '=end' }) do
    reset(FIX.cbase.lines)
    quiet('let g:iex = 0')
    opts({ 'indentexpr=IexConst()' })
    quiet('setlocal indentkeys=' .. vim.fn.escape(ik, ' \\|"'))
    vim.api.nvim_win_set_cursor(0, { 8, 0 })
    keys('ox = 1;\27o}\27o#if 1\27olabel:\27')
    text('i09 keys ' .. tag(ik))
    ans('i09 keys ' .. tag(ik) .. ' cb', true, { ran = vim.g.iex })
  end
  for _, name in ipairs({
    'FexNoop',
    'FexInternal',
    'FexUpper',
    'FexThrow',
    'IexConst',
    'IexMinusOne',
    'IexCindent',
    'IexThrow',
    'IexEdit',
  }) do
    quiet('delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- s10 -- the tab-stop layer: indent.rs's own arithmetic
-- ---------------------------------------------------------------------

section('s10-tabstops', function()
  local WIDTHS = {
    'shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab',
    'shiftwidth=4 tabstop=8 softtabstop=0 noexpandtab',
    'shiftwidth=4 tabstop=4 softtabstop=4 expandtab',
    'shiftwidth=2 tabstop=8 softtabstop=2 noexpandtab',
    'shiftwidth=0 tabstop=4 softtabstop=-1 expandtab',
    'shiftwidth=3 tabstop=8 softtabstop=0 noexpandtab shiftround',
    'shiftwidth=8 tabstop=8 softtabstop=0 expandtab',
    'shiftwidth=16 tabstop=8 softtabstop=0 noexpandtab',
    -- 'preserveindent' selects a whole branch of set_indent -- the one
    -- that walks the existing whitespace tab by tab rather than
    -- rebuilding it -- and nothing else in the sweep sets it.  A
    -- mutation on that walk measured NOT CAUGHT without these two.
    'shiftwidth=4 tabstop=8 softtabstop=0 noexpandtab preserveindent',
    'shiftwidth=3 tabstop=4 softtabstop=0 noexpandtab preserveindent copyindent',
  }
  local VTS = { '', '4', '4,8', '4,8,12', '2,4,6,8,10', '4,4,4,20', '1' }
  for _, w in ipairs(WIDTHS) do
    for _, vts in ipairs(VTS) do
      for _, fname in ipairs({ 'cbase', 'indented', 'bullets' }) do
        reset(FIX[fname].lines)
        opts({ w })
        quiet('setlocal vartabstop=' .. vts)
        profile('t10 ' .. tag(w) .. ' vts' .. tag(vts) .. ' ' .. fname, 'c')
        for _, seq in ipairs({ 'ggVG>', 'ggVG>>>', 'ggVG<', 'gg>G', 'ggVG=' }) do
          reset(FIX[fname].lines)
          opts({ w })
          quiet('setlocal vartabstop=' .. vts)
          keys(seq)
          text('t10 ' .. tag(w) .. ' vts' .. tag(vts) .. ' ' .. fname .. ' ' .. tag(seq))
        end
      end
    end
  end
  -- Typed Tab and BS, which is the only path to `get_sts_value`:
  -- 'softtabstop' decides how far one keystroke moves, and `=`/`>`/`<`
  -- read 'shiftwidth' instead.  A mutation on get_sts_value measured NOT
  -- CAUGHT until this block existed.
  for _, w in ipairs(WIDTHS) do
    for _, seq in ipairs({
      'ggIa\9b\9\27',
      'ggI\9\9\9x\27',
      'ggI\9\9\8\8y\27',
      'GoZ\9\9\27',
      'ggI    \8z\27',
    }) do
      for _, fname in ipairs({ 'cbase', 'indented' }) do
        reset(FIX[fname].lines)
        opts({ w })
        keys(seq)
        text('t10 sts ' .. tag(w) .. ' ' .. tag(seq) .. ' ' .. fname)
      end
    end
  end

  -- get_number_indent, which is what fo+=n reads and `:left` writes.
  for _, vsts in ipairs({ '', '4', '2,4,8' }) do
    reset(FIX.bullets.lines)
    opts({ 'shiftwidth=4', 'tabstop=8', 'expandtab' })
    quiet('setlocal varsofttabstop=' .. vsts)
    keys('ggVG>')
    text('t10 vsts' .. tag(vsts))
    for _, cmd in ipairs({ 'left', 'left 4', 'right 30', 'center 30', '%left 2' }) do
      reset(FIX.bullets.lines)
      opts({ 'shiftwidth=4', 'tabstop=8', 'expandtab' })
      quiet('setlocal varsofttabstop=' .. vsts)
      quiet('silent! %' .. cmd)
      text('t10 vsts' .. tag(vsts) .. ' ' .. tag(cmd))
    end
  end
end)

-- ---------------------------------------------------------------------
-- s11 -- lispindent()
-- ---------------------------------------------------------------------

section('s11-lisp', function()
  for _, lw in ipairs({
    '',
    'defun,define,let,cond',
    'define',
    'if,when,unless',
  }) do
    for _, lo in ipairs({ '', 'expr:0', 'expr:1' }) do
      reset(FIX.lisp.lines)
      opts({ 'lisp' })
      quiet('setlocal lispwords=' .. vim.fn.escape(lw, ' \\|"'))
      quiet('setlocal lispoptions=' .. lo)
      profile('z11 ' .. tag(lw) .. ' ' .. tag(lo), 'lisp')
      keys('gg=G')
      text('z11 ' .. tag(lw) .. ' ' .. tag(lo))
      -- and the insert-mode half, where 'lisp' drives open_line.
      reset(FIX.lisp.lines)
      opts({ 'lisp', 'autoindent' })
      quiet('setlocal lispwords=' .. vim.fn.escape(lw, ' \\|"'))
      quiet('setlocal lispoptions=' .. lo)
      keys('ggjo(new-form 1)\27')
      text('z11 ' .. tag(lw) .. ' ' .. tag(lo) .. ' o')
    end
  end
  -- lispindent() over the C fixtures too: the two engines are asked the
  -- same questions in s01 and this is where their disagreement is the
  -- point rather than an accident.
  for _, fname in ipairs({ 'cbase', 'cparen', 'prose' }) do
    reset(FIX[fname].lines)
    opts({ 'lisp' })
    profile('z11 cross ' .. fname, 'lisp')
  end
  -- the out-of-range arms
  reset(FIX.lisp.lines)
  for _, l in ipairs({ -1, 0, 1, 999 }) do
    P('z11 lispindent ' .. tag(l), vim.fn.lispindent, l)
    P('z11 cindent ' .. tag(l), vim.fn.cindent, l)
    P('z11 indent ' .. tag(l), vim.fn.indent, l)
  end
  P('z11 lispindent dot', vim.fn.lispindent, '.')
  P('z11 cindent dot', vim.fn.cindent, '.')
  P('z11 indent dot', vim.fn.indent, '.')
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
  reset(FIX.cbase.lines)
  loud('echo "-- s90 start"')
  loud('echo cindent([])')
  loud('echo lispindent([])')
  loud('echo indent([])')
  loud('echo indent("nosuchmark")')
  loud('setlocal cinoptions=>')
  loud('setlocal cinoptions=q9')
  loud('setlocal cinkeys=nosuchkey')
  loud('setlocal comments=x')
  loud('setlocal comments=s1:/*,m')
  loud('let &l:formatlistpat = "\\\\("')
  loud('setlocal textwidth=-1')
  loud('setlocal wrapmargin=-1')
  loud('setlocal shiftwidth=-1')
  loud('setlocal tabstop=0')
  loud('setlocal tabstop=100000')
  loud('setlocal vartabstop=0')
  loud('setlocal vartabstop=4,x')
  loud('setlocal varsofttabstop=-1')
  loud('setlocal lispoptions=nosuch')
  loud('setlocal indentexpr=NoSuchIndentFn()')
  loud('normal! gg=G')
  loud('setlocal indentexpr= formatexpr=NoSuchFormatFn()')
  loud('setlocal textwidth=40')
  loud('normal! ggVGgq')
  loud('setlocal formatexpr= equalprg=/nonexistent/prog')
  loud('normal! gg=G')
  loud('setlocal equalprg= formatprg=/nonexistent/prog')
  loud('normal! ggVGgq')
  loud('setlocal formatprg=')
  loud('echo "-- s90 end"')
end)

-- ---------------------------------------------------------------------
-- s91 -- CRASHPROBE.  Option values that terminate the process, run in a
-- child so that the abort is one diffable line rather than a truncated
-- report.
--
-- `parse_cino` reads every numeric field with `getdigits_int(&p, true, 0)`,
-- and `getdigits_int` opens its strict arm with
--
--     assert!(number >= INT_MIN && number <= INT_MAX)
--
-- so `set cinoptions=>2147483648` SIGABRTs.  Upstream's C has the same
-- `assert()` (charset.c:1134) but it is compiled out under NDEBUG, where
-- the value is silently truncated instead; the transpiled form is a
-- plain `assert!`, which is *not* compiled out -- measured aborting in
-- the release build as well as the debug one.  It is a one-line denial
-- of service from a modeline.  Both facts are on the docket.
--
-- One child per case, and only the exit status is recorded: a killed
-- child reports its signal rather than its code and its stderr carries a
-- pid and a backtrace, so the report says OK or ABORTED and nothing
-- else.
-- ---------------------------------------------------------------------

section('s91-crashprobe', function()
  local CASES = {
    { 'cino-int-max', 'cinoptions=>2147483647' },
    { 'cino-over', 'cinoptions=>2147483648' },
    { 'cino-huge', 'cinoptions=>99999999999' },
    { 'cino-negover', 'cinoptions=>-2147483648' },
    { 'cino-neghuge', 'cinoptions=>-99999999999' },
    { 'cino-paren-huge', 'cinoptions=)99999999999' },
    { 'cino-star-huge', 'cinoptions=*99999999999' },
    { 'cino-comment-huge', 'cinoptions=c99999999999' },
    { 'cino-frac-huge', 'cinoptions=>1.99999999999s' },
    { 'cino-i64max', 'cinoptions=>9223372036854775807' },
    { 'cino-i64over', 'cinoptions=>9223372036854775808' },
    { 'ts-huge', 'tabstop=2147483647' },
    { 'sw-huge', 'shiftwidth=2147483647' },
    { 'tw-huge', 'textwidth=2147483647' },
    { 'vts-huge', 'vartabstop=99999999999' },
  }
  for _, c in ipairs(CASES) do
    local res = vim
      .system({
        vim.v.progpath,
        '--headless',
        '-u',
        'NONE',
        '-i',
        'NONE',
        '--cmd',
        'set cindent ' .. c[2],
        '-c',
        'call setline(1, ["int f(void)", "{", "if (a) {", "b();", "}", "}"])',
        '-c',
        'echo join(map(range(1, 6), "cindent(v:val)"), ",")',
        '-c',
        'qa!',
      }, { text = true })
      :wait()
    -- A `:echo` from a headless process goes to the PROMPT, i.e. the
    -- child's stderr, not its stdout -- stdout is empty for a surviving
    -- child too.  And an aborting child reports its **signal**, leaving
    -- `code` at 0, so `code == 0` alone calls a SIGABRT a success (it
    -- did, for every case, until this line).  A dying child's stderr
    -- carries a pid and a backtrace, so it is never recorded: the answer
    -- is the indent profile or the word ABORTED.
    local died = (res.signal or 0) ~= 0 or (res.code or 0) ~= 0
    ans('k91 ' .. c[1], true, {
      ok = not died,
      out = died and 'ABORTED' or (res.stderr or ''):gsub('%s+$', ''),
    })
  end
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 ambiwidth=single nomodeline nostartofline')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set undolevels=1000 undofile& hidden nowritebackup nobackup')
quiet('set backspace=indent,eol,start whichwrap= virtualedit= selection=inclusive')
quiet('set nowrap sidescroll=0 conceallevel=0')
quiet('language C')
quiet('filetype off')
quiet('syntax off')
quiet('silent! cd ' .. vim.fn.fnameescape(work .. '/files'))

emit('shiftwidth', vim.o.shiftwidth)
emit('tabstop', vim.o.tabstop)
emit('comments', esc(vim.o.comments))
emit('cinoptions', esc(vim.o.cinoptions))
emit('cinkeys', esc(vim.o.cinkeys))
emit('formatlistpat', esc(vim.o.formatlistpat))
emit('fixtures', #FIXORDER)

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
      -- FMTSWEEP_TRACE writes into the .stderr artifact and must be off
      -- for a baseline.
      io.stderr:write(string.format('   %s %.1fs\n', entry.name, (vim.uv.hrtime() - started) / 1e9))
    end
  end
end

emit('')
emit('== done ==')
structfd:close()
