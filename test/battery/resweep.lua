-- resweep -- the differential oracle for the regexp family.
--
-- Covers crates/nvim/src/nvim/regexp.rs and regexp/{api,bt,bt/*,nfa,
-- nfa/*,parse,chars,mbyte,equi_class,context,submatch,substitute}.rs:
-- `vim_regcomp` over every magic level, `vim_regexec_nl` /
-- `vim_regexec_multi` / `vim_regexec_prog`, the backtracking engine and
-- the NFA engine SIDE BY SIDE, the zero-width and position atoms, the
-- submatch bookkeeping, the `substitute()` replacement alphabet and the
-- error paths.
--
-- Driven by resweep.sh; see that file for the sandbox, the
-- artifacts and the scrub.
--
-- THE AXIS THAT MATTERS is `'regexpengine'`.  Both engines share `rex`
-- (the `RegExec` the whole family threads through itself), so a bug in
-- the shared state shows on ONE engine and not the other -- and nothing
-- else in this tree drives them side by side.  Every corpus row is run
-- under `re=0` (NFA, falling back to BT when compilation fails),
-- `re=1` (BT only) and `re=2` (NFA only), and `r0/enginedelta` counts
-- the rows on which 1 and 2 disagree.  That count is the sweep's own
-- assertion that the option is doing anything at all: if it collapses to
-- zero, one engine is not being reached and two thirds of the sweep is
-- measuring the other one twice.
--
-- DETERMINISM.  No wall clock, no `pairs()` over a hash in an answer, no
-- buffer or window handle in a row.  `matchlist()`/`matchbufline()`
-- answers are rendered through a FIXED field order.  Every caught error
-- is normalised to its `E<n>: <text>` tail, so the Lua traceback and the
-- runtime path that `pcall` prepends never reach an artifact.  Each
-- section restores `'regexpengine'`, `'ignorecase'`, `'smartcase'` and
-- `'magic'` before it returns.

local api = vim.api

local ONLY = os.getenv("RESWEEP_ONLY") or ""
local WORK = os.getenv("RE_WORK") or "/tmp"
local NVIM = os.getenv("RE_NVIM") or "nvim"
local TIMEOUT = os.getenv("RE_TIMEOUT") or "timeout"
local ERR_OUT = os.getenv("RE_ERR") or (WORK .. "/err")

local report = {}
local err_lines = {}
local section_rows = {}
local section_order = {}
local total_rows = 0

local function say(s)
  report[#report + 1] = s
end

local function digest(s)
  return vim.fn.sha256(s):sub(1, 12)
end

local cur_section = nil

local function section(name)
  cur_section = name
  section_order[#section_order + 1] = name
  section_rows[name] = 0
end

local function want(name)
  return ONLY == "" or name:match(ONLY) ~= nil
end

local function row(tag, extra)
  section_rows[cur_section] = section_rows[cur_section] + 1
  total_rows = total_rows + 1
  say(string.format("%-46s %s", tag, extra))
end

-- ------------------------------------------------------------ rendering
-- A pattern goes into a row VERBATIM; it is ASCII by construction except
-- where a case is deliberately multibyte, and those bytes are escaped.
local function esc(s)
  s = tostring(s)
  return (s:gsub("[^\32-\126]", function(c)
    return string.format("\\x%02x", string.byte(c))
  end))
end

local function ser(v)
  local t = type(v)
  if t == "table" then
    local parts = {}
    for i = 1, #v do
      parts[#parts + 1] = ser(v[i])
    end
    -- Trailing empties are noise: `matchlist` always answers ten slots.
    while #parts > 0 and parts[#parts] == "" do
      parts[#parts] = nil
    end
    return "{" .. table.concat(parts, "|") .. "}"
  elseif t == "string" then
    return esc(v)
  else
    return tostring(v)
  end
end

-- Normalise a caught error to the `E<n>: ...` tail.  `pcall` prepends a
-- Lua chunk name and a line number, and `nvim_exec2` prepends the whole
-- `vim/_core/editor` frame -- neither is a property of the regexp
-- engines and both move when the runtime moves.
local function ecode(msg)
  msg = tostring(msg):gsub("[\r\n]+", " ")
  local tail = msg:match("(E%d+:.*)$")
  if tail then
    return tail
  end
  tail = msg:match("Vim%b():%s*(.*)$") or msg:match("Vim:%s*(.*)$")
  return esc(tail or msg)
end

local errseen = 0

-- Call a driver; answer either its rendered result or `!<Ecode>`, and
-- record the full message in the `.err` layer either way.
local function try(tag, driver, fn, ...)
  local ok, res = pcall(fn, ...)
  if ok then
    return ser(res)
  end
  errseen = errseen + 1
  local code = ecode(res)
  err_lines[#err_lines + 1] = string.format("%-46s %-14s %s", tag, driver, code)
  return "!" .. (code:match("^(E%d+)") or "ERR")
end

-- ------------------------------------------------------------- drivers
-- The scalar drivers: everything that takes a String subject and reaches
-- `vim_regexec_nl` through `regexp/api.rs`.
local function strprobe(tag, pat, subj)
  local out = {}
  out[#out + 1] = "m=" .. try(tag, "match", vim.fn.match, subj, pat)
  out[#out + 1] = "e=" .. try(tag, "matchend", vim.fn.matchend, subj, pat)
  out[#out + 1] = "s=" .. try(tag, "matchstr", vim.fn.matchstr, subj, pat)
  out[#out + 1] = "p=" .. try(tag, "matchstrpos", vim.fn.matchstrpos, subj, pat)
  out[#out + 1] = "l=" .. try(tag, "matchlist", vim.fn.matchlist, subj, pat)
  out[#out + 1] = "u=" .. try(tag, "substitute", vim.fn.substitute, subj, pat, "<&>", "g")
  return table.concat(out, " ")
end

-- A Vimscript string literal.  Lua's `%q` is Lua syntax and only happens to
-- round-trip; `string()` is the real thing.
local function vstr(s)
  return vim.fn.string(s)
end

local ENGINES = { 0, 1, 2 }

local function set_engine(e)
  vim.o.regexpengine = e
end

-- Run `fn(engine)` under each of the three settings and answer the three
-- rendered results.  `re=1` and `re=2` are the two engines; `re=0` is
-- "NFA, then BT if it will not compile", which is what a user gets by
-- default and is the only setting that exercises the fallback.
local function per_engine(fn)
  local answers = {}
  for _, e in ipairs(ENGINES) do
    set_engine(e)
    answers[#answers + 1] = fn(e)
  end
  set_engine(0)
  return answers
end

local delta12 = 0
local delta_rows = {}

-- One corpus entry, three rows.  Records whether the two engines
-- disagreed -- that tally is `r0/enginedelta`.
local function corpus_case(prefix, name, pat, subj, probe)
  probe = probe or strprobe
  local answers = per_engine(function(e)
    local tag = string.format("%s/%s/e%d", prefix, name, e)
    if not want(tag) then
      return nil
    end
    return probe(tag, pat, subj)
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("%s/%s/e%d", prefix, name, e)
    if answers[i] ~= nil then
      row(tag, string.format("pat=%-28s subj=%-20s %s", esc(pat), esc(subj), answers[i]))
    end
  end
  if answers[2] ~= nil and answers[3] ~= nil and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = string.format("%s/%s", prefix, name)
  end
end

local function corpus(prefix, cases, probe)
  for _, c in ipairs(cases) do
    corpus_case(prefix, c[1], c[2], c[3], probe)
  end
end

-- --------------------------------------------------------- r1 magic ---
-- The magic level decides which of `. * [ ~ $ ^` and which of the
-- backslashed forms are special, and it is the first thing `vim_regcomp`
-- reads.  Every meaning below is written FOUR ways, once per level, plus
-- the `'magic'` option's own two settings.
local MAGIC = {
  -- one-or-more `a` then `b`
  { "plus/v", [==[\v a+b]==], "  aaab " },
  { "plus/m", [==[\m a\+b]==], "  aaab " },
  { "plus/M", [==[\M a\+b]==], "  aaab " },
  { "plus/V", [==[\V a\+b]==], "  aaab " },
  -- any character
  { "any/v", [==[\va.c]==], "a-c" },
  { "any/m", [==[\ma.c]==], "a-c" },
  { "any/M", [==[\Ma\.c]==], "a-c" },
  { "any/V", [==[\Va\.c]==], "a-c" },
  -- literal dot
  { "dot/v", [==[\va\.c]==], "a.c abc" },
  { "dot/m", [==[\ma\.c]==], "a.c abc" },
  { "dot/M", [==[\Ma.c]==], "a.c abc" },
  { "dot/V", [==[\Va.c]==], "a.c abc" },
  -- grouping and alternation
  { "alt/v", [==[\v(ab|cd)+]==], "xxabcdab" },
  { "alt/m", [==[\m\(ab\|cd\)\+]==], "xxabcdab" },
  { "alt/M", [==[\M\(ab\|cd\)\+]==], "xxabcdab" },
  { "alt/V", [==[\V\(ab\|cd\)\+]==], "xxabcdab" },
  -- star
  { "star/v", [==[\vab*c]==], "ac abbbc" },
  { "star/m", [==[\mab*c]==], "ac abbbc" },
  { "star/M", [==[\Mab\*c]==], "ac abbbc" },
  { "star/V", [==[\Vab\*c]==], "ac abbbc" },
  -- braces
  { "brace/v", [==[\va{2,3}]==], "a aa aaaa" },
  { "brace/m", [==[\ma\{2,3}]==], "a aa aaaa" },
  { "brace/M", [==[\Ma\{2,3}]==], "a aa aaaa" },
  { "brace/V", [==[\Va\{2,3}]==], "a aa aaaa" },
  -- word boundary
  { "word/v", [==[\v<ab>]==], "ab abc xab" },
  { "word/m", [==[\m\<ab\>]==], "ab abc xab" },
  { "word/M", [==[\M\<ab\>]==], "ab abc xab" },
  { "word/V", [==[\V\<ab\>]==], "ab abc xab" },
  -- optional / at-most-one
  { "opt/v", [==[\vab?c]==], "ac abc abbc" },
  { "opt/m", [==[\mab\=c]==], "ac abc abbc" },
  { "opt/M", [==[\Mab\=c]==], "ac abc abbc" },
  { "opt/V", [==[\Vab\=c]==], "ac abc abbc" },
  -- anchors
  { "anch/v", [==[\v^ab$]==], "ab" },
  { "anch/m", [==[\m^ab$]==], "ab" },
  { "anch/M", [==[\M^ab$]==], "ab" },
  { "anch/V", [==[\V^ab$]==], "ab" },
  -- a caret and a dollar in the MIDDLE are literal at every level
  { "midanch/v", [==[\va^b$c]==], "a^b$c" },
  { "midanch/m", [==[\ma^b$c]==], "a^b$c" },
  { "midanch/M", [==[\Ma^b$c]==], "a^b$c" },
  { "midanch/V", [==[\Va^b$c]==], "a^b$c" },
  -- collection
  { "coll/v", [==[\v[b-d]+]==], "axbcdy" },
  { "coll/m", [==[\m[b-d]\+]==], "axbcdy" },
  { "coll/M", [==[\M\[b-d]\+]==], "axbcdy" },
  { "coll/V", [==[\V\[b-d]\+]==], "axbcdy" },
  -- `~` is the previous substitute string; literal under \V
  { "tilde/M", [==[\Ma~b]==], "a~b" },
  { "tilde/V", [==[\Va~b]==], "a~b" },
  -- `&` -- the branch concat, magic-level sensitive
  { "branch/v", [==[\v(foo&.*bar)]==], "foobar" },
  { "branch/m", [==[\mfoo\&.*bar]==], "foobar" },
  { "branch/M", [==[\Mfoo\&.\*bar]==], "foobar" },
}

-- --------------------------------------------------------- r2 atoms ---
local ATOMS = {
  { "cls-d", [==[\d\+]==], "ab 1234 cd" },
  { "cls-D", [==[\D\+]==], "12ab34" },
  { "cls-w", [==[\w\+]==], "-- a_b9 --" },
  { "cls-W", [==[\W\+]==], "ab -- cd" },
  { "cls-s", [==[\s\+]==], "a \t b" },
  { "cls-S", [==[\S\+]==], "  abc  " },
  { "cls-a", [==[\a\+]==], "12abc34" },
  { "cls-A", [==[\A\+]==], "abc12def" },
  { "cls-l", [==[\l\+]==], "ABCdefGHI" },
  { "cls-u", [==[\u\+]==], "abcDEFghi" },
  { "cls-x", [==[\x\+]==], "zz1aFzz" },
  { "cls-X", [==[\X\+]==], "1aFzz1a" },
  { "cls-o", [==[\o\+]==], "89012789" },
  { "cls-O", [==[\O\+]==], "1289012" },
  { "cls-h", [==[\h\+]==], "12_abc12" },
  { "cls-H", [==[\H\+]==], "_ab12cd" },
  { "cls-i", [==[\i\+]==], " ab_1 " },
  { "cls-k", [==[\k\+]==], " ab_1 " },
  { "cls-f", [==[\f\+]==], " /tmp/x " },
  { "cls-p", [==[\p\+]==], "ab\tcd" },
  { "cls-n", [==[\n]==], "ab" },
  { "cls-e", [==[\e]==], "a\027b" },
  { "cls-t", [==[\t]==], "a\tb" },
  { "cls-r", [==[\r]==], "a\rb" },
  { "cls-b", [==[\b]==], "a\bb" },
  { "posix-alpha", [==[[[:alpha:]]\+]==], "12abcXY34" },
  { "posix-digit", [==[[[:digit:]]\+]==], "ab123cd" },
  { "posix-alnum", [==[[[:alnum:]]\+]==], "--ab12--" },
  { "posix-punct", [==[[[:punct:]]\+]==], "ab,.;cd" },
  { "posix-space", [==[[[:space:]]\+]==], "a  \tb" },
  { "posix-upper", [==[[[:upper:]]\+]==], "abCDef" },
  { "posix-lower", [==[[[:lower:]]\+]==], "ABcdEF" },
  { "posix-xdigit", [==[[[:xdigit:]]\+]==], "zz9afzz" },
  { "posix-cntrl", [==[[[:cntrl:]]\+]==], "a\001\002b" },
  { "posix-print", [==[[[:print:]]\+]==], "\001ab\002" },
  { "posix-graph", [==[[[:graph:]]\+]==], " ab " },
  { "posix-blank", [==[[[:blank:]]\+]==], "a \t b" },
  { "posix-ident", [==[[[:ident:]]\+]==], " a_1 " },
  { "posix-keyword", [==[[[:keyword:]]\+]==], " a_1 " },
  { "posix-fname", [==[[[:fname:]]\+]==], " /a/b " },
  { "coll-range", [==[[a-cx-z]\+]==], "qabcxyzq" },
  { "coll-neg", [==[[^a-c]\+]==], "abcXYZabc" },
  { "coll-esc", [==[[\]\-]\+]==], "a]-b" },
  { "coll-caret", [==[[a^]\+]==], "b^ab" },
  { "coll-backslash", [==[[\\]\+]==], [==[a\b]==] },
  { "coll-class-in", [==[[\d_]\+]==], "ab1_2cd" },
  { "coll-nl", [==[[^x]\+]==], "abxcd" },
  { "equi-e", [==[[[=e=]]\+]==], "a\195\169e\195\168b" },
  -- Upstream keeps the equivalence tables TWICE, once per engine, and
  -- the two copies drifted: U+0200 is in the NFA copy's `A` class and
  -- not the backtracking copy's.  This row is that drift.
  { "equi-A-u200", "[[=A=]]\\+", "x\200\128Ay" },
  { "equi-i-dup", "[[=i=]]\\+", "x\225\187\139iy" },
  { "equi-a", [==[[[=a=]]\+]==], "x\195\161a\195\160y" },
  { "coll-dot", [==[[[.a.]]\+]==], "xaay" },
  { "dec-A", [==[\%d65]==], "xAy" },
  { "hex-A", [==[\%x41]==], "xAy" },
  { "oct-A", [==[\%o101]==], "xAy" },
  { "uni-e9", [==[\%u00e9]==], "x\195\169y" },
  { "uni-U", [==[\%U0001F600]==], "x\240\159\152\128y" },
  { "any-nl", [==[\_.\+]==], "ab" },
  { "space-nl", [==[\_s\+]==], "a  b" },
  { "coll-nl-cls", [==[\_[abc]\+]==], "xabcx" },
  { "mb-utf8", [==[\%u00e9\+]==], "a\195\169\195\169b" },
  { "mb-class", [==[\a\+]==], "\195\169abc" },
  { "mb-dot", [==[a.c]==], "a\195\169c" },
  { "mb-coll", "[\195\169\195\168]\\+", "x\195\169\195\168y" },
  { "opt-seq", [==[\%[foo]]==], "foobar" },
  { "opt-partial", [==[f\%[oobar]]==], "food" },
  { "opt-empty", [==[a\%[]]==], "a" },
}

-- --------------------------------------------------------- r3 multi ---
local MULTI = {
  { "star", [==[a*]==], "baaac" },
  { "star-greedy", [==[<.*>]==], "<a><b>" },
  { "star-lazy", [==[<.\{-}>]==], "<a><b>" },
  { "plus", [==[a\+]==], "baaac" },
  { "plus-lazy", [==[a\{-1,}]==], "baaac" },
  { "quest", [==[ab\?c]==], "ac abc" },
  { "eq", [==[ab\=c]==], "ac abc" },
  { "brace-n", [==[a\{3}]==], "aaaaa" },
  { "brace-nm", [==[a\{2,4}]==], "aaaaa" },
  { "brace-n-", [==[a\{2,}]==], "aaaaa" },
  { "brace--m", [==[a\{,3}]==], "aaaaa" },
  { "brace-any", [==[a\{}]==], "aaaaa" },
  { "brace-lazy", [==[a\{-}]==], "aaaaa" },
  { "brace-lazy-nm", [==[a\{-2,4}]==], "aaaaa" },
  { "brace-lazy-n", [==[a\{-3}]==], "aaaaa" },
  { "brace-zero", [==[a\{0}b]==], "b ab" },
  { "brace-big", [==[a\{1,100}]==], "aaaaaaaa" },
  { "nested-star", [==[\(ab\)*]==], "ababx" },
  { "nested-plus", [==[\(ab\)\+c]==], "xababc" },
  { "nested-lazy", [==[\(ab\)\{-1,}c]==], "xababc" },
  { "alt-multi", [==[\(a\|b\)\{2,3}]==], "xabab" },
  { "multi-empty", [==[\(x*\)\+]==], "yyy" },
  { "atomic", [==[\(a*\)\@>b]==], "aaab" },
  { "atomic-fail", [==[\(a*\)\@>ab]==], "aaab" },
  { "star-anchored", [==[^a*$]==], "aaa" },
  { "dotstar-dollar", [==[.*$]==], "abc" },
  { "greedy-backref", [==[\(.*\)\1]==], "abcabc" },
}

-- ---------------------------------------------------------- r4 zero ---
local ZERO = {
  { "zs", [==[foo\zsbar]==], "foobar" },
  { "ze", [==[foo\zebar]==], "foobar" },
  { "zs-ze", [==[a\zsbc\zed]==], "abcd" },
  { "zs-alt", [==[\(foo\|f\)\zsoo]==], "fooo" },
  { "zs-twice", [==[a\zsb\zsc]==], "abc" },
  { "ze-twice", [==[a\zeb\zec]==], "abc" },
  { "look-ahead", [==[foo\(bar\)\@=]==], "foobar fooba" },
  { "look-ahead-neg", [==[foo\(bar\)\@!]==], "foobar foobaz" },
  { "look-behind", [==[\(foo\)\@<=bar]==], "foobar zzbar" },
  { "look-behind-neg", [==[\(foo\)\@<!bar]==], "foobar zzbar" },
  { "look-behind-lim", [==[\(a\)\@2<=b]==], "ab" },
  { "look-nested", [==[\(a\(b\)\@=\)\@=ab]==], "ab" },
  { "bof", [==[\%^ab]==], "ab" },
  { "eof", [==[ab\%$]==], "ab" },
  { "group-nc", [==[\%(ab\)\+c]==], "ababc" },
  { "backref-1", [==[\(a\+\)b\1]==], "aabaa" },
  { "backref-2", [==[\(a\)\(b\)\2\1]==], "abba" },
  { "backref-empty", [==[\(x*\)y\1]==], "y" },
  { "backref-9", [==[\(a\)\(b\)\(c\)\(d\)\(e\)\(f\)\(g\)\(h\)\(i\)\9]==], "abcdefghii" },
  { "concat-and", [==[.*foo\&.*bar]==], "xfooybarz" },
  { "concat-and-no", [==[.*foo\&.*bar]==], "xfooyz" },
  { "opt-seq-nest", [==[r\%[ead]]==], "read re r" },
  { "cursor", [==[\%#]==], "abc" },
  { "comp-char", [==[\%C]==], "abc" },
}

-- ---------------------------------------------------------- r5 case ---
local CASECORP = {
  { "plain", [==[ABC]==], "xabcx ABC" },
  { "esc-c", [==[\cABC]==], "xabcx" },
  { "esc-C", [==[\CABC]==], "xabcx ABC" },
  { "esc-c-mid", [==[A\cBC]==], "xabcx" },
  { "coll", [==[[abc]\+]==], "XYZABC" },
  { "coll-neg", [==[[^abc]\+]==], "abcABC" },
  { "class-l", [==[\l\+]==], "ABC" },
  { "class-u", [==[\u\+]==], "abc" },
  { "backref", [==[\(a\)\1]==], "aA" },
  { "brace", [==[A\{2}]==], "aa" },
  { "equi", [==[[[=e=]]\+]==], "\195\137E" },
  { "word", [==[\<ABC\>]==], "abc" },
}

-- --------------------------------------------------------- r9 errors ---
local ERRORS = {
  { "nested-star", [==[a**]==] },
  { "star-nothing", [==[\{1}]==] },
  { "plus-nothing", [==[\+]==] },
  { "lazy-nothing", [==[\{-}]==] },
  { "underscore", [==[\_]==] },
  { "brace-open", [==[a\{]==] },
  { "brace-bad", [==[a\{2,x}]==] },
  { "paren-open", [==[\(a]==] },
  { "paren-close", [==[a\)]==] },
  { "nc-paren-open", [==[\%(a]==] },
  { "backref-illegal", [==[\1]==] },
  { "zref-illegal", [==[\z1]==] },
  { "zopen-illegal", [==[\z(a\)]==] },
  { "rev-range", [==[[z-a]]==] },
  { "pct-bad", [==[\%d]==] },
  { "pct-x-bad", [==[\%xzz]==] },
  { "pct-eq-bad", [==[\%#=3a]==] },
  { "pct-unknown", [==[\%y]==] },
  { "opt-seq-nested", [==[\%[a\%[b]]]==] },
  { "opt-seq-open", [==[\%[abc]==] },
  { "posix-bad", [==[[[:nosuch:]]]==] },
  { "at-bad", [==[a\@z]==] },
  { "v-brace", [==[\v{1}]==] },
  { "v-paren", [==[\v(a]==] },
  { "v-bad-at", [==[\v(a)@zz]==] },
  { "V-nothing", [==[\V\+]==] },
  { "M-nothing", [==[\M\*]==] },
  { "look-behind-huge", [==[\(a\)\@123<=b]==] },
  { "brace-huge", [==[a\{1,1000000}]==] },
  { "empty", [==[]==] },
  { "lone-backslash", [==[a\]==] },
  { "tilde-nosub", [==[a~b]==] },
  { "equi-open", [==[[[=a]]==] },
  { "coll-open", [==[[abc]==] },
  { "class-open", [==[[[:alpha:]]==] },
  { "mark-atom-bad", [==[\%'z]==] },
  -- A multi applied to a ZERO-WIDTH atom.  `re_mult_next` is the only
  -- caller that cares which KIND of multi follows (`MULTI_MULT` refuses,
  -- `MULTI_ONE` is allowed through), so nothing else in the corpus can
  -- tell the two classes apart.
  { "repeat-zs", [==[a\zs*b]==] },
  { "repeat-zs-brace", [==[a\zs\{2}b]==] },
  { "repeat-zs-plus", [==[a\zs\+b]==] },
  { "repeat-zs-opt", [==[a\zs\=b]==] },
  { "repeat-ze", [==[a\ze*b]==] },
  { "repeat-bof", [==[\%^\+a]==] },
  { "repeat-visual", [==[\%V*a]==] },
  { "repeat-look", [==[\(a\)\@=*b]==] },
}

-- =====================================================================
--                                 r0
-- =====================================================================
section("r0-canary")

if want("r0/version") then
  row("r0/version", string.format("magic=%s ic=%s scs=%s re=%s", tostring(vim.o.magic),
    tostring(vim.o.ignorecase), tostring(vim.o.smartcase), tostring(vim.o.regexpengine)))
end

-- The engine override atom.  `\%#=N` beats `'regexpengine'`, and it is
-- the only way a pattern can name its own engine.
for _, n in ipairs({ 0, 1, 2 }) do
  local tag = "r0/override/" .. n
  if want(tag) then
    local pat = string.format([==[\%%#=%da**]==], n)
    set_engine(1)
    local a = try(tag, "match", vim.fn.match, "aaa", pat)
    set_engine(2)
    local b = try(tag, "match", vim.fn.match, "aaa", pat)
    set_engine(0)
    row(tag, string.format("pat=%-16s under_re1=%-58s under_re2=%s", esc(pat), a, b))
  end
end

if want("r0/badengine") then
  local ok, e = pcall(function()
    vim.o.regexpengine = 3
  end)
  set_engine(0)
  row("r0/badengine", string.format("ok=%s err=%s", tostring(ok), ok and "-" or ecode(e)))
end

if want("r0/stable") then
  -- The oracle's own determinism: the same probe twice, byte for byte.
  local a = strprobe("r0/stable", [==[\(a\+\)b]==], "xaaabx")
  local b = strprobe("r0/stable", [==[\(a\+\)b]==], "xaaabx")
  row("r0/stable", string.format("same=%s a=%s", tostring(a == b), a))
end

-- The two engines' own fingerprints.  `a**` is E61 in the backtracking
-- parser and E871 in the NFA one, `\{1}` is E64 vs E866, and nested
-- `\%[]` is E369 in BT and compiles in NFA.  If these three rows stop
-- disagreeing, `'regexpengine'` is not selecting anything and two thirds
-- of this sweep is measuring one engine twice.
for _, c in ipairs({
  { "nested-star", [==[a**]==] },
  { "brace-nothing", [==[\{1}]==] },
  { "optseq-nested", [==[\%[a\%[b]]]==] },
}) do
  local tag = "r0/enginemark-" .. c[1]
  if want(tag) then
    set_engine(1)
    local bt = try(tag, "match", vim.fn.match, "aab", c[2])
    set_engine(2)
    local nfa = try(tag, "match", vim.fn.match, "aab", c[2])
    set_engine(0)
    row(tag, string.format("pat=%-14s bt=%-8s nfa=%-8s differ=%s", esc(c[2]), bt, nfa, tostring(bt ~= nfa)))
  end
end

if want("r0/magicagree") then
  -- The four magic levels spelling ONE meaning must answer one thing.
  local agree, checked = 0, 0
  for _, quad in ipairs({
    { [==[\v(ab|cd)+]==], [==[\m\(ab\|cd\)\+]==], [==[\M\(ab\|cd\)\+]==], [==[\V\(ab\|cd\)\+]==] },
    { [==[\v a+b]==], [==[\m a\+b]==], [==[\M a\+b]==], [==[\V a\+b]==] },
    { [==[\v<ab>]==], [==[\m\<ab\>]==], [==[\M\<ab\>]==], [==[\V\<ab\>]==] },
  }) do
    local subj = "xx aaab abcdab ab "
    local first = vim.fn.matchstr(subj, quad[1])
    for i = 2, 4 do
      checked = checked + 1
      if vim.fn.matchstr(subj, quad[i]) == first then
        agree = agree + 1
      end
    end
  end
  row("r0/magicagree", string.format("agree=%d/%d", agree, checked))
end

-- =====================================================================
section("r1-magic")
corpus("r1", MAGIC)

-- The `'magic'` OPTION, not the in-pattern prefix.  With `nomagic` a
-- bare `.` and `*` are literal and `\.`/`\*` are the specials.
for _, m in ipairs({ true, false }) do
  for _, c in ipairs({
    { "opt-any", [==[a.c]==], "a.c abc" },
    { "opt-any-esc", [==[a\.c]==], "a.c abc" },
    { "opt-star", [==[ab*c]==], "ac abbc" },
    { "opt-star-esc", [==[ab\*c]==], "ac abbc" },
    { "opt-coll", [==[[abc]\+]==], "xxabcxx" },
    { "opt-tilde", [==[a~c]==], "a~c" },
  }) do
    local name = string.format("%s/magic%s", c[1], tostring(m))
    vim.o.magic = m
    corpus_case("r1", name, c[2], c[3])
    vim.o.magic = true
  end
end
vim.o.magic = true

-- =====================================================================
section("r2-atoms")
corpus("r2", ATOMS)

-- =====================================================================
section("r3-multi")
corpus("r3", MULTI)

-- =====================================================================
section("r4-zero")
corpus("r4", ZERO)

-- `\%23l` / `\%23c` / `\%23v` and `\%V` need a BUFFER and a WINDOW: they
-- are the only atoms that read the editor's position state, and they run
-- through `vim_regexec_multi`, not the string path.
local function bufbuild(lines)
  local buf = api.nvim_create_buf(false, true)
  api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  api.nvim_win_set_buf(0, buf)
  return buf
end

local POSFIX = {
  "aaa bbb ccc",
  "ddd eee fff",
  "ggg hhh iii",
  "jjj kkk lll",
  "mmm nnn ooo",
}

local POSPATS = {
  { "line-eq", [==[\%3lggg]==] },
  { "line-lt", [==[\%<3l\w\+]==] },
  { "line-gt", [==[\%>3l\w\+]==] },
  { "col-eq", [==[\%5c\w\+]==] },
  { "col-lt", [==[\%<5cb\w\+]==] },
  { "col-gt", [==[\%>5c\w\+]==] },
  { "vcol-eq", [==[\%5v\w\+]==] },
  { "vcol-lt", [==[\%<5v\w\+]==] },
  { "vcol-gt", [==[\%>9v\w\+]==] },
  { "line-col", [==[\%2l\%5c\w\+]==] },
  { "mark-lt", [==[\%<'a\w\+]==] },
  { "mark-gt", [==[\%>'a\w\+]==] },
  { "cursor-at", [==[\%#\w\+]==] },
  { "visual", [==[\%V\w\+]==] },
}

for _, c in ipairs(POSPATS) do
  local answers = per_engine(function(e)
    local tag = string.format("r4/pos-%s/e%d", c[1], e)
    if not want(tag) then
      return nil
    end
    bufbuild(POSFIX)
    api.nvim_win_set_cursor(0, { 2, 4 })
    vim.cmd("normal! ma")
    api.nvim_win_set_cursor(0, { 3, 4 })
    -- A real Visual area for `\%V`: lines 2-3, columns 5-7.
    vim.cmd("normal! 2G4lv1j2l\027")
    api.nvim_win_set_cursor(0, { 3, 4 })
    local hits = {}
    local ok, res = pcall(function()
      return vim.fn.matchbufline(api.nvim_get_current_buf(), c[2], 1, "$")
    end)
    if not ok then
      errseen = errseen + 1
      err_lines[#err_lines + 1] = string.format("%-46s %-14s %s", tag, "matchbufline", ecode(res))
      return "!" .. (ecode(res):match("^(E%d+)") or "ERR")
    end
    for _, h in ipairs(res) do
      hits[#hits + 1] = string.format("%d:%d:%s", h.lnum, h.byteidx, esc(h.text))
    end
    -- ... and the same pattern through `search()`, which is the other
    -- door into `vim_regexec_multi`.
    local sp = vim.fn.searchpos(c[2], "nw")
    return string.format("hits=%d [%s] search=%d,%d", #hits, table.concat(hits, " "), sp[1], sp[2])
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r4/pos-%s/e%d", c[1], e)
    if answers[i] ~= nil then
      row(tag, string.format("pat=%-24s %s", esc(c[2]), answers[i]))
    end
  end
  if answers[2] and answers[3] and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = "r4/pos-" .. c[1]
  end
end

-- =====================================================================
section("r5-case")
for _, ic in ipairs({ false, true }) do
  for _, scs in ipairs({ false, true }) do
    vim.o.ignorecase = ic
    vim.o.smartcase = scs
    for _, c in ipairs(CASECORP) do
      corpus_case("r5", string.format("%s/ic%d-scs%d", c[1], ic and 1 or 0, scs and 1 or 0), c[2], c[3])
    end
  end
end
vim.o.ignorecase = false
vim.o.smartcase = false

-- `'smartcase'` only applies to a SEARCH (`search()`, `/`, `:s`), never
-- to `match()` -- which is exactly the sort of thing a shared-state bug
-- breaks, so drive it through the search door too.
for _, ic in ipairs({ false, true }) do
  for _, scs in ipairs({ false, true }) do
    for _, pat in ipairs({ [==[abc]==], [==[Abc]==], [==[ABC]==], [==[\cabc]==], [==[\CAbc]==] }) do
      local name = string.format("scs-%s/ic%d-scs%d", esc(pat), ic and 1 or 0, scs and 1 or 0)
      local answers = per_engine(function(e)
        local tag = string.format("r5/%s/e%d", name, e)
        if not want(tag) then
          return nil
        end
        vim.o.ignorecase = ic
        vim.o.smartcase = scs
        bufbuild({ "zzz", "abc", "Abc", "ABC", "aBc" })
        api.nvim_win_set_cursor(0, { 1, 0 })
        local out = {}
        for _ = 1, 4 do
          local p = vim.fn.searchpos(pat, "W")
          out[#out + 1] = string.format("%d,%d", p[1], p[2])
          if p[1] == 0 then
            break
          end
        end
        return "search=" .. table.concat(out, ";")
      end)
      for i, e in ipairs(ENGINES) do
        local tag = string.format("r5/%s/e%d", name, e)
        if answers[i] ~= nil then
          row(tag, answers[i])
        end
      end
      if answers[2] and answers[3] and answers[2] ~= answers[3] then
        delta12 = delta12 + 1
        delta_rows[#delta_rows + 1] = "r5/" .. name
      end
    end
  end
end
vim.o.ignorecase = false
vim.o.smartcase = false

-- =====================================================================
section("r6-drivers")
-- Every remaining door into the engines that takes a String.
local DRIVEPAT = [==[\(\w\)\(\d\+\)]==]
local DRIVESUBJ = "a1 bb22 c333"

local DRIVERS = {
  { "match", function(p, s) return vim.fn.match(s, p) end },
  { "match-start", function(p, s) return vim.fn.match(s, p, 5) end },
  { "match-count", function(p, s) return vim.fn.match(s, p, 0, 2) end },
  { "matchend", function(p, s) return vim.fn.matchend(s, p) end },
  { "matchstr", function(p, s) return vim.fn.matchstr(s, p) end },
  { "matchstr-idx", function(p, s) return vim.fn.matchstr(s, p, 0, 3) end },
  { "matchstrpos", function(p, s) return vim.fn.matchstrpos(s, p) end },
  { "matchlist", function(p, s) return vim.fn.matchlist(s, p) end },
  { "substitute", function(p, s) return vim.fn.substitute(s, p, [==[<\1|\2>]==], "g") end },
  { "split", function(p, s) return vim.fn.split(s, p) end },
  { "split-keep", function(p, s) return vim.fn.split(s, p, 1) end },
  { "match-list", function(p, s) return vim.fn.match({ s, "zz", "d4" }, p) end },
  { "matchstr-list", function(p, s) return vim.fn.matchstr({ s, "zz", "d4" }, p) end },
  { "eq-tilde", function(p, s) return vim.fn.eval(vstr(s) .. " =~ " .. vstr(p)) end },
  { "eq-tilde-hash", function(p, s) return vim.fn.eval(vstr(s) .. " =~# " .. vstr(p)) end },
  { "eq-tilde-quest", function(p, s) return vim.fn.eval(vstr(s) .. " =~? " .. vstr(p)) end },
  { "not-tilde", function(p, s) return vim.fn.eval(vstr(s) .. " !~ " .. vstr(p)) end },
  { "filter", function(p, s) return vim.fn.filter({ s, "zz", "d4" }, "v:val =~ " .. vstr(p)) end },
  { "map", function(p, s) return vim.fn.map({ s, "zz" }, "substitute(v:val, " .. vstr(p) .. [[, "X", "g")]]) end },
  { "getcompletion", function(p, _) return vim.fn.getcompletion(p, "event") end },
}

for _, d in ipairs(DRIVERS) do
  local answers = per_engine(function(e)
    local tag = string.format("r6/%s/e%d", d[1], e)
    if not want(tag) then
      return nil
    end
    return try(tag, d[1], d[2], DRIVEPAT, DRIVESUBJ)
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r6/%s/e%d", d[1], e)
    if answers[i] ~= nil then
      row(tag, answers[i])
    end
  end
  if answers[2] and answers[3] and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = "r6/" .. d[1]
  end
end

-- `search()`'s flag alphabet, which is the position half of the engines.
local SEARCHFLAGS = { "", "b", "w", "W", "n", "e", "c", "s", "z", "bW", "ce", "nw" }
for _, f in ipairs(SEARCHFLAGS) do
  local answers = per_engine(function(e)
    local tag = string.format("r6/searchflag-%s/e%d", f == "" and "none" or f, e)
    if not want(tag) then
      return nil
    end
    bufbuild({ "one two", "three two", "two four", "five two" })
    api.nvim_win_set_cursor(0, { 3, 0 })
    local p = vim.fn.searchpos("two", f)
    local q = vim.fn.search("two", f)
    local c = api.nvim_win_get_cursor(0)
    return string.format("searchpos=%d,%d search=%d cursor=%d,%d", p[1], p[2], q, c[1], c[2])
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r6/searchflag-%s/e%d", f == "" and "none" or f, e)
    if answers[i] ~= nil then
      row(tag, answers[i])
    end
  end
end

-- `searchpair()` / `searchpairpos()` compile THREE patterns and walk
-- them together.
for _, c in ipairs({
  { "pair-plain", [==[(]==], "", [==[)]==], "" },
  { "pair-nested", [==[{]==], "", [==[}]==], "b" },
  { "pair-mid", [==[\<if\>]==], [==[\<else\>]==], [==[\<endif\>]==], "" },
}) do
  local answers = per_engine(function(e)
    local tag = string.format("r6/%s/e%d", c[1], e)
    if not want(tag) then
      return nil
    end
    bufbuild({ "if ( { x", "  else }", "endif )", "tail" })
    api.nvim_win_set_cursor(0, { 2, 2 })
    local p = try(tag, "searchpairpos", vim.fn.searchpairpos, c[2], c[3], c[4], c[5] .. "n")
    return "pairpos=" .. p
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r6/%s/e%d", c[1], e)
    if answers[i] ~= nil then
      row(tag, answers[i])
    end
  end
end

-- =====================================================================
section("r7-multiline")
-- `vim_regexec_multi`.  A pattern that spans a line break can only be
-- reached from a buffer, and `:g`, `:s` and `search()` are its three
-- doors.
local MLFIX = {
  "alpha one",
  "beta two",
  "gamma three",
  "alpha four",
  "beta five",
  "delta six",
  "alpha seven",
}

local MLPATS = {
  { "nl-literal", [==[one\nbeta]==] },
  { "nl-class", [==[one\_sbeta]==] },
  { "nl-any", [==[one\_.beta]==] },
  { "nl-star", [==[alpha\_.\{-}beta]==] },
  { "nl-greedy", [==[alpha\_.*beta]==] },
  { "nl-coll", [==[e\_[a-z ]\+a]==] },
  { "nl-anchor", [==[^beta\n]==] },
  { "nl-dollar", [==[two$\n^gamma]==] },
  { "nl-group", [==[\(alpha\|beta\)\n]==] },
  { "nl-backref", [==[\(\w\+\)\n.*\1]==] },
  { "nl-zs", [==[one\n\zsbeta]==] },
  { "nl-ze", [==[one\zen\nbeta]==] },
  { "nl-look", [==[beta\(\ntwo\)\@!]==] },
  { "nl-bof", [==[\%^alpha]==] },
  { "nl-eof", [==[six\%$]==] },
  { "single", [==[alpha]==] },
  { "single-anchor", [==[^alpha$]==] },
  { "empty-match", [==[x*]==] },
}

for _, c in ipairs(MLPATS) do
  local answers = per_engine(function(e)
    local tag = string.format("r7/%s/e%d", c[1], e)
    if not want(tag) then
      return nil
    end
    local out = {}
    -- (1) `:g` -- `global_exe` drives `vim_regexec_multi` line by line.
    bufbuild(MLFIX)
    local hit = {}
    _G.__re_g = function()
      hit[#hit + 1] = vim.fn.line(".")
      return 0
    end
    local okg, eg = pcall(vim.cmd, string.format([[silent g/%s/call luaeval('_G.__re_g()')]], c[2]:gsub("/", "\\/")))
    out[#out + 1] = "g=" .. (okg and ("{" .. table.concat(hit, "|") .. "}") or ("!" .. (ecode(eg):match("^(E%d+)") or "ERR")))
    if not okg then
      errseen = errseen + 1
      err_lines[#err_lines + 1] = string.format("%-46s %-14s %s", tag, "global", ecode(eg))
    end
    -- (2) `:v` -- the complement, same walk.
    bufbuild(MLFIX)
    local vhit = {}
    _G.__re_g = function()
      vhit[#vhit + 1] = vim.fn.line(".")
      return 0
    end
    local okv = pcall(vim.cmd, string.format([[silent v/%s/call luaeval('_G.__re_g()')]], c[2]:gsub("/", "\\/")))
    out[#out + 1] = "v=" .. (okv and ("{" .. table.concat(vhit, "|") .. "}") or "!ERR")
    -- (3) `:s` across lines -- `do_sub` with a multi-line match.
    bufbuild(MLFIX)
    local oks, es = pcall(vim.cmd, string.format([[%%s/%s/<&>/g]], c[2]:gsub("/", "\\/")))
    if oks then
      local lines = api.nvim_buf_get_lines(0, 0, -1, false)
      out[#out + 1] = string.format("s=%d:%s", #lines, digest(table.concat(lines, "\n")))
      out[#out + 1] = "sub=" .. esc(table.concat(lines, "\\n"):sub(1, 90))
    else
      out[#out + 1] = "s=!" .. (ecode(es):match("^(E%d+)") or "ERR")
      errseen = errseen + 1
      err_lines[#err_lines + 1] = string.format("%-46s %-14s %s", tag, "substitute", ecode(es))
    end
    -- (4) `search()` from the top, and `matchbufline`.
    bufbuild(MLFIX)
    api.nvim_win_set_cursor(0, { 1, 0 })
    local sp = vim.fn.searchpos(c[2], "nW")
    out[#out + 1] = string.format("search=%d,%d", sp[1], sp[2])
    local okm, mres = pcall(vim.fn.matchbufline, api.nvim_get_current_buf(), c[2], 1, "$")
    if okm then
      local hs = {}
      for _, h in ipairs(mres) do
        hs[#hs + 1] = string.format("%d:%d:%s", h.lnum, h.byteidx, esc(h.text))
      end
      out[#out + 1] = string.format("mbl=%d{%s}", #hs, table.concat(hs, "|"))
    else
      out[#out + 1] = "mbl=!" .. (ecode(mres):match("^(E%d+)") or "ERR")
    end
    return table.concat(out, " ")
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r7/%s/e%d", c[1], e)
    if answers[i] ~= nil then
      row(tag, string.format("pat=%-24s %s", esc(c[2]), answers[i]))
    end
  end
  if answers[2] and answers[3] and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = "r7/" .. c[1]
  end
end

-- =====================================================================
section("r8-subst")
-- The replacement alphabet -- `regexp/substitute.rs`'s
-- `vim_regsub`/`vim_regsub_multi` -- and `:s`'s own flags.
local SUBCORP = {
  { "amp", [==[\(a\+\)b]==], [==[<&>]==] },
  { "amp-esc", [==[\(a\+\)b]==], [==[<\&>]==] },
  { "sub-0", [==[\(a\+\)b]==], [==[<\0>]==] },
  { "sub-1", [==[\(a\+\)b]==], [==[<\1>]==] },
  { "sub-2", [==[\(a\)\(a*\)b]==], [==[<\2|\1>]==] },
  { "upper-one", [==[\(a\+\)]==], [==[\u\1]==] },
  { "upper-all", [==[\(a\+\)]==], [==[\U\1\E!]==] },
  { "lower-one", [==[\(A\+\)]==], [==[\l\1]==] },
  { "lower-all", [==[\(A\+\)]==], [==[\L\1\E!]==] },
  { "newline", [==[b]==], [==[x\ry]==] },
  { "nul", [==[b]==], [==[x\ny]==] },
  { "tab", [==[b]==], [==[x\ty]==] },
  { "backslash", [==[b]==], [==[x\\y]==] },
  { "tilde", [==[b]==], [==[x~y]==] },
  { "expr", [==[\(a\+\)]==], [==[\=strlen(submatch(1))]==] },
  { "expr-list", [==[\(a\+\)]==], [==[\=submatch(0)->toupper()]==] },
  { "expr-sm", [==[\(a\)\(a*\)]==], [==[\=submatch(2) . "-" . submatch(1)]==] },
  { "empty", [==[a*]==], [==[X]==] },
}

for _, c in ipairs(SUBCORP) do
  local answers = per_engine(function(e)
    local tag = string.format("r8/%s/e%d", c[1], e)
    if not want(tag) then
      return nil
    end
    local out = {}
    out[#out + 1] = "fn=" .. try(tag, "substitute", vim.fn.substitute, "xaaabx aab", c[2], c[3], "g")
    out[#out + 1] = "fn1=" .. try(tag, "substitute", vim.fn.substitute, "xaaabx aab", c[2], c[3], "")
    bufbuild({ "xaaabx aab", "AAB zz", "qqq" })
    local ok, err = pcall(vim.cmd, string.format([[%%s/%s/%s/g]], c[2]:gsub("/", "\\/"), c[3]:gsub("/", "\\/")))
    if ok then
      out[#out + 1] = "buf=" .. esc(table.concat(api.nvim_buf_get_lines(0, 0, -1, false), "\\n"))
    else
      out[#out + 1] = "buf=!" .. (ecode(err):match("^(E%d+)") or "ERR")
      errseen = errseen + 1
      err_lines[#err_lines + 1] = string.format("%-46s %-14s %s", tag, "ex-sub", ecode(err))
    end
    return table.concat(out, " ")
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r8/%s/e%d", c[1], e)
    if answers[i] ~= nil then
      row(tag, string.format("pat=%-18s rep=%-24s %s", esc(c[2]), esc(c[3]), answers[i]))
    end
  end
  if answers[2] and answers[3] and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = "r8/" .. c[1]
  end
end

-- `:s`'s flag alphabet.  `c` is excluded: it prompts, and a prompt in a
-- headless run reads from the (empty) stdin and takes the default.
for _, f in ipairs({ "", "g", "i", "I", "e", "n", "gn", "gi", "&", "g&", "p", "l", "#" }) do
  local answers = per_engine(function(e)
    local tag = string.format("r8/flag-%s/e%d", f == "" and "none" or f:gsub("[&#]", { ["&"] = "amp", ["#"] = "hash" }), e)
    if not want(tag) then
      return nil
    end
    bufbuild({ "aAa bBb", "AAA", "ccc", "aaa" })
    vim.v.errmsg = ""
    local out = vim.fn.execute(string.format([[silent! %%s/a\+/<&>/%s]], f), "silent")
    out = out:gsub("[\r\n]+", "|"):gsub("^|", "")
    return string.format("buf=%-42s msg=%s err=%s", esc(table.concat(api.nvim_buf_get_lines(0, 0, -1, false), "\\n")), esc(out:sub(1, 60)), esc(vim.v.errmsg))
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r8/flag-%s/e%d", f == "" and "none" or f:gsub("[&#]", { ["&"] = "amp", ["#"] = "hash" }), e)
    if answers[i] ~= nil then
      row(tag, string.format("flags=%-4s %s", f, answers[i]))
    end
  end
end
vim.v.errmsg = ""

-- =====================================================================
section("r9-errors")
-- The error paths are OBSERVABLE ROWS, not aborts.  A pattern that
-- refuses to compile is a first-class answer, and the two engines refuse
-- with DIFFERENT codes for the same input more often than they agree.
for _, c in ipairs(ERRORS) do
  local answers = per_engine(function(e)
    local tag = string.format("r9/%s/e%d", c[1], e)
    if not want(tag) then
      return nil
    end
    local out = {}
    out[#out + 1] = "m=" .. try(tag, "match", vim.fn.match, "abc", c[2])
    out[#out + 1] = "u=" .. try(tag, "substitute", vim.fn.substitute, "abc", c[2], "X", "g")
    bufbuild({ "abc", "def" })
    local ok, err = pcall(vim.cmd, string.format([[%%s/%s/X/g]], c[2]:gsub("/", "\\/")))
    out[#out + 1] = "ex=" .. (ok and esc(table.concat(api.nvim_buf_get_lines(0, 0, -1, false), "\\n")) or ("!" .. ecode(err)))
    local oks, sres = pcall(vim.fn.searchpos, c[2], "nw")
    out[#out + 1] = "search=" .. (oks and string.format("%d,%d", sres[1], sres[2]) or ("!" .. (ecode(sres):match("^(E%d+)") or "ERR")))
    return table.concat(out, " ")
  end)
  for i, e in ipairs(ENGINES) do
    local tag = string.format("r9/%s/e%d", c[1], e)
    if answers[i] ~= nil then
      row(tag, string.format("pat=%-22s %s", esc(c[2]), answers[i]))
    end
  end
  if answers[2] and answers[3] and answers[2] ~= answers[3] then
    delta12 = delta12 + 1
    delta_rows[#delta_rows + 1] = "r9/" .. c[1]
  end
end

-- =====================================================================
section("r10-levers")
if want("r10/enginedelta") then
  row("r10/enginedelta", string.format("rows=%d", delta12))
end
if want("r10/deltalist") then
  table.sort(delta_rows)
  row("r10/deltalist", table.concat(delta_rows, " "))
end
if want("r10/errseen") then
  row("r10/errseen", string.format("errors=%d", errseen))
end

-- =====================================================================
section("r91-abortprobe")
-- Pathological patterns in FRESH CHILDREN.  Catastrophic backtracking is
-- a real property of the BT engine, so every child is under a hard
-- timeout and the row records `rc`, not a wall time.
local PROBES = {
  {
    "catastrophic-bt",
    [[
      vim.o.regexpengine = 1
      io.stdout:write('r=', tostring(vim.fn.match('aaaaaaaaaaaaaaaaaaaaaaaaX', [==[\(a*\)*b]==])), '\n')
    ]],
  },
  {
    "catastrophic-nfa",
    [[
      vim.o.regexpengine = 2
      io.stdout:write('r=', tostring(vim.fn.match('aaaaaaaaaaaaaaaaaaaaaaaaX', [==[\(a*\)*b]==])), '\n')
    ]],
  },
  {
    "deep-nest",
    [[
      local p = string.rep([==[\(]==], 60) .. 'a' .. string.rep([==[\)]==], 60)
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, 'a', p)
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "long-pattern",
    [[
      local p = string.rep('a', 40000)
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, 'a', p)
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r):sub(1,60), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "huge-alt",
    [[
      local parts = {}
      for i = 1, 500 do parts[i] = 'w' .. i end
      local p = table.concat(parts, [==[\|]==])
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, 'zzw499zz', p)
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r):sub(1,40), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "brace-huge",
    [[
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, string.rep('a', 200), [==[a\{1,100000}]==])
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r):sub(1,40), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "optseq-deep",
    [[
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, 'ab', 'a\\%[bcdefghijklmnopqrstuvwxyz]')
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r):sub(1,40), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "invalid-utf8",
    [[
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        local ok, r = pcall(vim.fn.match, 'a\255\254b', 'a\255')
        io.stdout:write('e', e, '=', tostring(ok), '/', tostring(r):sub(1,40), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "sub-recursive-expr",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'aaa','bbb','ccc'})
      local ok, e = pcall(vim.cmd, [==[%s/a/\=substitute(submatch(0), 'a', 'z', 'g')/g]==])
      io.stdout:write('ok=', tostring(ok), ' buf=', table.concat(vim.api.nvim_buf_get_lines(0,0,-1,false), '|'), '\n')
    ]],
  },
  {
    "sub-expr-error",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'aaa','bbb'})
      local ok = pcall(vim.cmd, [==[%s/a/\=nosuchfunction()/g]==])
      io.stdout:write('ok=', tostring(ok), ' buf=', table.concat(vim.api.nvim_buf_get_lines(0,0,-1,false), '|'), '\n')
    ]],
  },
  {
    "sub-delete-lines",
    [[
      local l = {}
      for i = 1, 200 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      pcall(vim.cmd, [==[%s/\n//]==])
      io.stdout:write('n=', vim.api.nvim_buf_line_count(0), '\n')
    ]],
  },
  {
    "global-nested",
    [[
      local l = {}
      for i = 1, 60 do l[i] = 'a' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      pcall(vim.cmd, [==[g/a/normal! dd]==])
      io.stdout:write('n=', vim.api.nvim_buf_line_count(0), '\n')
    ]],
  },
  {
    "global-append",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'a','b','a','b'})
      pcall(vim.cmd, [==[g/a/put ='a']==])
      io.stdout:write('n=', vim.api.nvim_buf_line_count(0), '\n')
    ]],
  },
  {
    "smallwindow-zerowidth",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'abc','def'})
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        pcall(vim.cmd, [==[%s/\zs/X/g]==])
      end
      io.stdout:write('buf=', table.concat(vim.api.nvim_buf_get_lines(0,0,-1,false), '|'), '\n')
    ]],
  },
  {
    "search-empty-loop",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'abc','def'})
      for _, e in ipairs({1,2}) do
        vim.o.regexpengine = e
        for _ = 1, 50 do vim.fn.search('x*', 'W') end
      end
      io.stdout:write('ok\n')
    ]],
  },
  {
    "syntax-engine",
    [[
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'int x = 1;', 'char *s;'})
      vim.cmd('syntax on')
      vim.cmd([==[syntax match reFoo /\v(int|char)/]==])
      vim.cmd('redraw')
      io.stdout:write('ok\n')
    ]],
  },
}

local aborted = 0
for i, p in ipairs(PROBES) do
  local tag = "r91/" .. p[1]
  if want(tag) then
    local file = string.format("%s/reprobe-%02d.lua", WORK, i)
    local fh = io.open(file, "w")
    fh:write(p[2], "\n")
    fh:close()
    local cmd = string.format(
      "%s -k 2 30 %s --headless -u NONE -i NONE "
        .. "--cmd 'set noswapfile nomore report=9999 shortmess=aoOtTIcCF' "
        .. "-c 'luafile %s' -c 'qa!' 2>&1",
      TIMEOUT,
      NVIM,
      file
    )
    local out = vim.fn.system(cmd)
    local rc = vim.v.shell_error
    out = out
      :gsub("[\r\n]+", " ")
      :gsub("0x%x+", "<ADDR>")
      :gsub("%.rs:%d+:%d+", ".rs:<LINE>")
      :gsub("%.rs:%d+", ".rs:<LINE>")
      :gsub(vim.pesc(WORK), "<WORK>")
      :gsub("%s+$", "")
    if rc ~= 0 then
      aborted = aborted + 1
    end
    section_rows[cur_section] = section_rows[cur_section] + 1
    total_rows = total_rows + 1
    say(string.format("%-46s rc=%-4d alive=%-5s said=%s", tag, rc, tostring(rc == 0), esc(out:sub(1, 220))))
  end
end

if want("r91/groups") then
  section_rows[cur_section] = section_rows[cur_section] + 1
  total_rows = total_rows + 1
  say(string.format("%-46s cases=%d aborted=%d", "r91/groups", #PROBES, aborted))
end

-- ---------------------------------------------------------------- output
local tail = {}
for _, s in ipairs(section_order) do
  tail[#tail + 1] = string.format("## %s rows=%d", s, section_rows[s] or 0)
end
tail[#tail + 1] = string.format("## TOTAL rows=%d", total_rows)

io.stdout:write(table.concat(report, "\n"), "\n", table.concat(tail, "\n"), "\n")

local fh = io.open(ERR_OUT, "w")
fh:write(table.concat(err_lines, "\n"), "\n")
fh:close()
