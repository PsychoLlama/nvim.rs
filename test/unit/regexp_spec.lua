-- Regexp engine coverage, driven straight through the C ABI
-- (`vim_regcomp`/`vim_regexec`) rather than through Vimscript, so a failure
-- points at the engine instead of at whatever called it.
--
-- Three overlapping nets, each catching a different kind of regression:
--
--   1. Golden expectations. Hand-written from the documented semantics
--      (`:help pattern`), not captured from a run, so they still mean
--      something when the implementation changes underneath them.
--   2. A differential oracle. The backtracking and NFA engines share almost
--      nothing but the entry point, so "both agree" is a cheap, self-
--      maintaining check over a corpus far larger than anyone would write
--      expectations for. Every golden case is also run on both engines.
--   3. Termination and liveness. Pathological patterns must finish and must
--      not take the process down. Each `itp` runs in a forked child, so a
--      crash fails one test; `deadline()` arms SIGALRM so a hang does too,
--      instead of wedging the suite.
--
-- Sibling spec: test/functional/editor/regexp_spec.lua covers what needs a
-- live buffer (multi-line matching, :substitute, search offsets).

local t = require('test.unit.testutil')
local itp = t.gen_itp(it)

local ffi = t.ffi
local eq = t.eq
local to_cstr = t.to_cstr

local lib = t.cimport('./src/nvim/regexp.h')

-- Not part of the crate's surface, so ffigen doesn't emit it. SIGALRM is the
-- only hang guard available inside the forked child: the parent blocks
-- reading the child's pipe, so nothing outside can time the child out.
ffi.cdef('unsigned int alarm(unsigned int seconds);')

--- Prefixes that pin the engine. `\%#=N` is stripped by `vim_regcomp` before
--- the pattern proper, so it composes with a following `\v`/`\M`/`\V`.
local ENGINES = { bt = '\\%#=1', nfa = '\\%#=2' }

--- Compile `pat` and match it against `line`, as a string that reads well in
--- a failure diff: `'compile-error'`, `'nomatch'`, or `'<start>-<end>'`
--- followed by `' <n>:<start>-<end>'` for each set submatch. Offsets are
--- byte offsets into `line`.
---
--- @param engine string one of the `ENGINES` prefixes, or `''` for the default
--- @param pat string
--- @param line string
--- @param ic boolean? ignore case (the `rm_ic` the caller would pass)
--- @return string
local function match(engine, pat, line, ic)
  local prog = lib.vim_regcomp(to_cstr(engine .. pat), lib.RE_MAGIC + lib.RE_STRING)
  if prog == nil then
    return 'compile-error'
  end
  local rm = ffi.new('regmatch_T')
  rm.regprog = prog
  rm.rm_ic = ic or false
  -- `to_cstr` result must outlive the match: startp/endp point into it.
  local cline = to_cstr(line)
  local hit = lib.vim_regexec(rm, cline, 0)
  local res --- @type string
  if not hit then
    res = 'nomatch'
  else
    res = ('%d-%d'):format(tonumber(rm.startp[0] - cline), tonumber(rm.endp[0] - cline))
    for i = 1, 9 do
      if rm.startp[i] ~= nil and rm.endp[i] ~= nil then
        res = res
          .. (' %d:%d-%d'):format(i, tonumber(rm.startp[i] - cline), tonumber(rm.endp[i] - cline))
      end
    end
  end
  lib.vim_regfree(rm.regprog)
  return res
end

--- Describes a case the way it should appear in a failure message: the
--- pattern and input as one would type them.
local function label(pat, line)
  return ('/%s/ on %q'):format(pat, line)
end

--- Assert a golden expectation, and assert the two engines agree on it.
--- Both engines run for every case: an expectation that only holds on one of
--- them is itself a finding.
---
--- @param exp string as returned by `match`
--- @param pat string
--- @param line string
--- @param ic boolean?
local function ok(exp, pat, line, ic)
  local ctx = label(pat, line)
  eq(exp, match(ENGINES.bt, pat, line, ic), 'bt ' .. ctx)
  eq(exp, match(ENGINES.nfa, pat, line, ic), 'nfa ' .. ctx)
end

--- Run a whole table of `{pattern, input, expected}` rows through `ok`.
--- A fourth element, when present, is the ignore-case flag.
local function golden(cases)
  for _, c in ipairs(cases) do
    ok(c[3], c[1], c[2], c[4])
  end
end

--- Arm SIGALRM for `seconds`, run `fn`, disarm. A pattern that fails to
--- terminate kills the forked child, which `itp_parent` reports as a failed
--- test rather than hanging the run.
local function deadline(seconds, fn)
  ffi.C.alarm(seconds)
  local ret = fn()
  ffi.C.alarm(0)
  return ret
end

describe('vim_regcomp()', function()
  itp('accepts well-formed patterns at every magic level', function()
    for _, pat in ipairs({
      'abc',
      'a.c',
      'a\\+',
      '\\(a\\)\\1',
      '\\%(a\\)\\+',
      '\\va+',
      '\\v(a|b)*',
      '\\Ma\\*',
      '\\Va.c',
      '\\v%(ab){2,3}',
      '[a-z]\\{2,}',
      '\\%d65',
      '\\%^abc\\%$',
      'a\\@<=b',
      'x\\zsy\\zez',
    }) do
      for name, engine in pairs(ENGINES) do
        local prog = lib.vim_regcomp(to_cstr(engine .. pat), lib.RE_MAGIC + lib.RE_STRING)
        t.ok(prog ~= nil, 'compiles', ('%s rejected /%s/'):format(name, pat))
        if prog ~= nil then
          lib.vim_regfree(prog)
        end
      end
    end
  end)

  itp('rejects malformed patterns on both engines', function()
    -- Both engines must agree that these are errors: a pattern one accepts
    -- and the other rejects is a divergence, and a rejection that turns into
    -- a silent mis-compile is how crashes get in.
    for _, pat in ipairs({
      '\\(', -- E54: unmatched \(
      '\\)', -- E55: unmatched \)
      '\\%(', -- unmatched \%(
      'a\\{', -- E554: unmatched \{
      '\\v(', -- unmatched ( at very magic
      '\\v)',
      '\\z(a\\)', -- \z( is only valid in syntax patterns
      '\\v(a){1,2}{3}', -- multi directly after multi
      'a**',
      '\\+', -- multi with nothing to repeat
      '\\v+',
      '\\%d', -- \%d wants a number
      '\\%[', -- unmatched \%[
      '\\@=', -- lookaround with nothing in front
    }) do
      for name, engine in pairs(ENGINES) do
        local prog = lib.vim_regcomp(to_cstr(engine .. pat), lib.RE_MAGIC + lib.RE_STRING)
        t.ok(prog == nil, 'rejects', ('%s accepted /%s/'):format(name, pat))
        if prog ~= nil then
          lib.vim_regfree(prog)
        end
      end
    end
  end)

  itp('frees a program without matching first', function()
    -- Exercises the compile/free path on its own: a leak or double free here
    -- is otherwise masked by a successful match.
    for _ = 1, 100 do
      local prog = lib.vim_regcomp(to_cstr('\\(a\\+\\)\\(b\\|c\\)\\{2,5}'), lib.RE_MAGIC)
      t.ok(prog ~= nil, 'compiles', 'compile failed')
      lib.vim_regfree(prog)
    end
  end)
end)

describe('magic levels', function()
  itp('magic (the default) makes . * [] special', function()
    golden({
      { 'a.c', 'abc', '0-3' },
      { 'a.c', 'a.c', '0-3' },
      { 'ab*c', 'ac', '0-2' },
      { 'ab*c', 'abbbc', '0-5' },
      { '[abc]\\+', 'xxabcxx', '2-5' },
      -- while + ? ( ) | { need escaping
      { 'a+', 'a+', '0-2' },
      { 'a\\+', 'aaa', '0-3' },
      { 'a?', 'a?', '0-2' },
      { 'a\\?', 'a', '0-1' },
      { '(a)', '(a)', '0-3' },
      { '\\(a\\)', 'a', '0-1 1:0-1' },
      { 'a|b', 'a|b', '0-3' },
      { 'a\\|b', 'b', '0-1' },
      { 'a{2}', 'a{2}', '0-4' },
      { 'a\\{2}', 'aa', '0-2' },
    })
  end)

  itp('\\v (very magic) makes everything but word chars special', function()
    golden({
      { '\\va+', 'aaa', '0-3' },
      { '\\v(a|b)+', 'abab', '0-4 1:3-4' },
      { '\\v%(ab)+', 'abab', '0-4' },
      { '\\va{2,3}', 'aaaa', '0-3' },
      { '\\va{-2,3}', 'aaaa', '0-2' },
      { '\\v<word>', 'a word here', '2-6' },
      { '\\v.{3}', 'abcd', '0-3' },
      { '\\v\\.', 'a.b', '1-2' },
      { '\\v(foo)@<=bar', 'foobar', '3-6 1:0-3' },
      { '\\vx=y', 'y', '0-1' },
      { '\\v[[:digit:]]+', 'ab12', '2-4' },
    })
  end)

  itp('\\M (nomagic) leaves only ^ $ special', function()
    golden({
      { '\\Ma.c', 'a.c', '0-3' },
      { '\\Ma.c', 'abc', 'nomatch' },
      { '\\Ma\\.c', 'abc', '0-3' },
      { '\\Mab*c', 'ab*c', '0-4' },
      { '\\Mab\\*c', 'abbc', '0-4' },
      { '\\M[abc]', '[abc]', '0-5' },
      { '\\M\\[abc]', 'b', '0-1' },
      { '\\M^abc$', 'abc', '0-3' },
      -- \+ \? \( \| \{ keep working: they are backslashed at every level
      { '\\Ma\\+', 'aaa', '0-3' },
      { '\\M\\(a\\)\\1', 'aa', '0-2 1:0-1' },
    })
  end)

  itp('\\V (very nomagic) leaves only the backslash special', function()
    golden({
      { '\\Va.c', 'a.c', '0-3' },
      { '\\Va*c', 'a*c', '0-3' },
      { '\\Va\\*', 'aaa', '0-3' },
      { '\\V[abc]', '[abc]', '0-5' },
      { '\\V^abc', '^abc', '0-4' },
      { '\\V\\^abc', 'abc', '0-3' },
      { '\\Vabc\\$', 'abc', '0-3' },
      { '\\V\\(a\\)\\1', 'aa', '0-2 1:0-1' },
    })
  end)

  itp('a magic level applies from where it appears, not globally', function()
    golden({
      -- \v..\V.. — the switch takes effect mid-pattern
      { 'a\\v(b|c)', 'ac', '0-2 1:1-2' },
      { '\\va\\M(b)', 'a(b)', '0-4' },
      { '\\Ma.\\vb+', 'a.bbb', '0-5' },
    })
  end)
end)

describe('anchors and boundaries', function()
  itp('^ and $ anchor to the ends of the string', function()
    golden({
      { '^abc', 'abcdef', '0-3' },
      { '^abc', 'xabcdef', 'nomatch' },
      { 'abc$', 'xyzabc', '3-6' },
      { 'abc$', 'abcx', 'nomatch' },
      { '^$', '', '0-0' },
      { '^$', 'x', 'nomatch' },
      { '^', 'abc', '0-0' },
      { '$', 'abc', '3-3' },
      { '^abc$', 'abc', '0-3' },
      -- ^ is only an anchor at the start of a branch; elsewhere it is literal
      { 'a^b', 'a^b', '0-3' },
      { 'a$b', 'a$b', '0-3' },
      { '\\(^a\\|^b\\)', 'b', '0-1 1:0-1' },
    })
  end)

  itp('\\%^ and \\%$ anchor to the ends of the text', function()
    golden({
      { '\\%^abc', 'abcdef', '0-3' },
      { '\\%^bc', 'abcdef', 'nomatch' },
      { 'def\\%$', 'abcdef', '3-6' },
      { 'de\\%$', 'abcdef', 'nomatch' },
    })
  end)

  itp('\\< and \\> anchor to word boundaries', function()
    golden({
      { '\\<mat\\>', 'on the mat', '7-10' },
      { '\\<mat\\>', 'on the matt', 'nomatch' },
      { '\\<the', 'blithe theory', '7-10' },
      { 'the\\>', 'blithe theory', '3-6' },
      { '\\<\\w\\+\\>', '  word  ', '2-6' },
      -- \zs after a boundary must not shift the boundary itself
      { '\\<\\zsword', 'a word', '2-6' },
    })
  end)

  itp('\\%V and cursor/line/column atoms compile in string mode', function()
    -- These need a buffer to mean anything; here we only assert they compile
    -- and terminate. Their semantics live in the functional spec.
    for _, pat in ipairs({
      '\\%V.',
      '\\%23l.',
      '\\%>3c.',
      '\\%<9c.',
      '\\%5v.',
      '\\%#.',
    }) do
      for _, engine in pairs(ENGINES) do
        t.ok(
          match(engine, pat, 'abcdef') ~= nil,
          'terminates',
          ('/%s/ did not terminate'):format(pat)
        )
      end
    end
  end)
end)

describe('quantifiers', function()
  itp('greedy quantifiers take as much as they can', function()
    golden({
      { 'a*', 'aaa', '0-3' },
      { 'a*', 'bbb', '0-0' }, -- matches empty at position 0
      { 'a\\+', 'aaa', '0-3' },
      { 'a\\+', 'bbb', 'nomatch' },
      { 'a\\?', 'aaa', '0-1' },
      { 'a\\?', 'bbb', '0-0' },
      { 'a\\=', 'aaa', '0-1' },
      { '.*', 'abc', '0-3' },
      { 'a.*b', 'axbyb', '0-5' },
      { 'ab*c*', 'abbb', '0-4' },
    })
  end)

  itp('non-greedy quantifiers take as little as they can', function()
    golden({
      { 'a\\{-}', 'aaa', '0-0' },
      { 'a\\{-}b', 'aaab', '0-4' },
      { 'a.\\{-}b', 'axbyb', '0-3' },
      { 'a\\{-1,}', 'aaa', '0-1' },
      { 'a\\{-1,3}', 'aaaa', '0-1' },
      { 'a\\{-2,3}', 'aaaa', '0-2' },
      { '\\v.{-}b', 'aaab', '0-4' },
    })
  end)

  itp('counted quantifiers respect their bounds', function()
    golden({
      { 'a\\{2}', 'aaaa', '0-2' },
      { 'a\\{3}', 'aaaa', '0-3' },
      { 'a\\{5}', 'aaaa', 'nomatch' },
      { 'a\\{2,3}', 'aaaa', '0-3' },
      { 'a\\{3,6}', 'aaaaaaaa', '0-6' },
      { 'a\\{,3}', 'aaaa', '0-3' },
      { 'a\\{0,}', 'aaaa', '0-4' },
      { 'a\\{}', 'aaaa', '0-4' },
      { 'a\\{2,}', 'aaaa', '0-4' },
      { 'a\\{0}', 'aaaa', '0-0' },
      { 'a\\{1}', 'aaaa', '0-1' },
      -- an upper bound below the lower bound is clamped up to the lower one
      -- rather than rejected; `:help /\{` leaves the case undefined, so this
      -- pins what both engines actually do.
      { 'a\\{3,2}', 'aaaa', '0-3' },
    })
  end)

  itp('quantifiers apply to the preceding atom, not the whole branch', function()
    golden({
      { 'ab\\+', 'abbb', '0-4' },
      { '\\(ab\\)\\+', 'abab', '0-4 1:2-4' },
      { '\\%(ab\\)\\{2}', 'ababab', '0-4' },
      { '[ab]\\{3}', 'abab', '0-3' },
      { '\\w\\{2,}', ' word ', '1-5' },
    })
  end)

  itp('a quantified group that can match empty still terminates', function()
    -- The classic way to hang a backtracker: an inner atom with a zero-width
    -- match under an outer `*`.
    deadline(10, function()
      golden({
        { '\\(a*\\)\\+b', 'aaab', '0-4 1:3-3' },
        { '\\v(a|)+', 'aa', '0-2 1:2-2' },
        { '\\v(){0,10}', 'abc', '0-0 1:0-0' },
        { '\\(\\)*x', 'x', '0-1 1:0-0' },
      })
      -- `\(a*\)*` belongs here too, but the engines disagree about what the
      -- group captured. See 'known engine divergences'.
    end)
  end)
end)

describe('groups, alternation and backreferences', function()
  itp('captures record the last iteration', function()
    golden({
      { '\\(a\\)', 'a', '0-1 1:0-1' },
      { '\\(a\\)\\(b\\)', 'ab', '0-2 1:0-1 2:1-2' },
      { '\\(a\\+\\)\\(b\\+\\)', 'aabbb', '0-5 1:0-2 2:2-5' },
      { '\\(\\(a\\)\\(b\\)\\)', 'ab', '0-2 1:0-2 2:0-1 3:1-2' },
      { '\\(a\\)\\+', 'aaa', '0-3 1:2-3' },
      -- nine groups is the documented maximum
      {
        '\\(a\\)\\(b\\)\\(c\\)\\(d\\)\\(e\\)\\(f\\)\\(g\\)\\(h\\)\\(i\\)',
        'abcdefghi',
        '0-9 1:0-1 2:1-2 3:2-3 4:3-4 5:4-5 6:5-6 7:6-7 8:7-8 9:8-9',
      },
    })
  end)

  itp('\\%( ... \\) groups without capturing', function()
    golden({
      { '\\%(ab\\)\\+', 'abab', '0-4' },
      { '\\%(a\\|b\\)\\(c\\)', 'bc', '0-2 1:1-2' },
      { '\\%(\\(a\\)\\)', 'a', '0-1 1:0-1' },
    })
  end)

  itp('alternation prefers the leftmost branch that matches', function()
    golden({
      { 'a\\|ab', 'ab', '0-1' },
      { 'ab\\|a', 'ab', '0-2' },
      { '\\(foo\\|foobar\\)', 'foobar', '0-3 1:0-3' },
      { 'x\\|y\\|z', 'zzz', '0-1' },
      { '\\v(a|b|c){3}', 'cba', '0-3 1:2-3' },
      -- an empty branch matches empty
      { '\\va|', 'b', '0-0' },
    })
  end)

  itp('\\1 - \\9 match what the group matched', function()
    golden({
      { '\\(a\\)\\1', 'aa', '0-2 1:0-1' },
      { '\\(a\\)\\1', 'ab', 'nomatch' },
      { '\\(ab\\)\\1', 'abab', '0-4 1:0-2' },
      { '\\(.\\)\\1', 'xaab', '1-3 1:1-2' },
      { '\\(\\w\\+\\) \\1', 'the the', '0-7 1:0-3' },
      { '\\(a\\)\\(b\\)\\2\\1', 'abba', '0-4 1:0-1 2:1-2' },
      -- a backref to a group that matched empty matches empty
      { '\\(x*\\)y\\1', 'y', '0-1 1:0-0' },
    })
  end)

  itp('\\%[ ... ] matches an optional trailing sequence', function()
    golden({
      { 'r\\%[ead]', 'r', '0-1' },
      { 'r\\%[ead]', 're', '0-2' },
      { 'r\\%[ead]', 'rea', '0-3' },
      { 'r\\%[ead]', 'read', '0-4' },
      { 'r\\%[ead]', 'reads', '0-4' },
      { 'f\\%[oo]x', 'fx', '0-2' },
      { 'f\\%[oo]x', 'foox', '0-4' },
      { 'f\\%[oo]x', 'fooox', 'nomatch' },
    })
  end)
end)

describe('character classes', function()
  itp('the named classes match their documented sets', function()
    golden({
      { '\\d\\+', 'ab123cd', '2-5' },
      { '\\D\\+', '12ab34', '2-4' },
      { '\\w\\+', ' _a1! ', '1-4' },
      { '\\W\\+', 'ab!?cd', '2-4' },
      { '\\s\\+', 'a \t b', '1-4' },
      { '\\S\\+', '  ab  ', '2-4' },
      { '\\a\\+', '12ab34', '2-4' },
      { '\\A\\+', 'ab12cd', '2-4' },
      { '\\l\\+', 'ABabAB', '2-4' },
      { '\\u\\+', 'abABab', '2-4' },
      { '\\x\\+', 'zzdeadbeefzz', '2-10' },
      { '\\X\\+', 'abzzab', '2-4' },
      { '\\o\\+', '89012789', '2-6' },
      { '\\O\\+', '01889012', '2-5' },
      { '\\h\\+', '12_ab12', '2-5' },
      { '\\H\\+', 'ab12ab', '2-4' },
      -- \i \I \k \K \f \F depend on 'isident'/'iskeyword'/'isfname'
      { '\\k\\+', ' word ', '1-5' },
      { '\\i\\+', ' word ', '1-5' },
      { '\\p\\+', 'ab', '0-2' },
    })
  end)

  itp('[] collections match, negate and range', function()
    golden({
      { '[abc]\\+', 'xxabcxx', '2-5' },
      { '[^abc]\\+', 'abcxyz', '3-6' },
      { '[a-c]\\+', 'xxabcxx', '2-5' },
      { '[a-cx-z]\\+', 'defxyzabc', '3-9' },
      { '[^a-c]\\+', 'abcdef', '3-6' },
      { '[0-9]\\{2}', 'ab12cd', '2-4' },
      -- a ] first in the collection is literal
      { '[]]', 'a]b', '1-2' },
      { '[^]]\\+', ']]ab]', '2-4' },
      -- a - first or last is literal
      { '[-a]\\+', 'x-ax', '1-3' },
      { '[a-]\\+', 'xa-x', '1-3' },
      -- a ^ that is not first is literal
      { '[a^]\\+', 'x^ax', '1-3' },
      -- backslash escapes inside a collection
      { '[\\]]', 'a]b', '1-2' },
      { '[\\\\]', 'a\\b', '1-2' },
      { '[\\t]', 'a\tb', '1-2' },
      { '[\\d65]', 'xAy', '1-2' },
      -- an empty-looking collection: [] is a literal [ followed by ]
      { '[]', '[]', '0-2' },
      -- an unterminated collection is not an error: the [ is literal
      { '[a-', 'x[a-y', '1-4' },
      { '[a', 'x[ay', '1-3' },
    })
  end)

  itp('[: :] POSIX classes work inside collections', function()
    golden({
      { '[[:digit:]]\\+', 'ab123', '2-5' },
      { '[[:alpha:]]\\+', '12abc34', '2-5' },
      { '[[:alnum:]]\\+', '!!ab12!!', '2-6' },
      { '[[:lower:]]\\+', 'ABabAB', '2-4' },
      { '[[:upper:]]\\+', 'abABab', '2-4' },
      { '[[:space:]]\\+', 'a \t b', '1-4' },
      { '[[:punct:]]\\+', 'ab!?cd', '2-4' },
      { '[[:xdigit:]]\\+', 'zzbeefzz', '2-6' },
      { '[[:blank:]]\\+', 'a \tb', '1-3' },
      { '[[:cntrl:]]\\+', 'a\1\2b', '1-3' },
      { '[[:print:]]\\+', 'ab', '0-2' },
      { '[[:graph:]]\\+', ' ab ', '1-3' },
      -- combined with ordinary members
      { '[[:digit:]x]\\+', 'ab1x2ab', '2-5' },
      { '[^[:digit:]]\\+', '12ab34', '2-4' },
    })
  end)

  itp('\\%d \\%x \\%o \\%u name a character by its code', function()
    golden({
      { '\\%d65', 'xAy', '1-2' },
      { '\\%x41', 'xAy', '1-2' },
      { '\\%o101', 'xAy', '1-2' },
      { '\\%u0041', 'xAy', '1-2' },
      { '\\%U00000041', 'xAy', '1-2' },
      { '\\%d233', 'xéy', '1-3' }, -- é, two bytes in utf-8
      { '\\%u00e9', 'xéy', '1-3' },
      { '\\%d65\\%d66', 'xABy', '1-3' },
      { '\\%d65\\+', 'xAAAy', '1-4' },
    })
  end)

  itp('\\_x adds the newline to a class', function()
    -- With `vim_regexec` (not `_nl`) the string has no newline to match, so
    -- these assert the class still matches everything it did before.
    golden({
      { '\\_s\\+', 'a  b', '1-3' },
      { '\\_d\\+', 'ab12', '2-4' },
      { '\\_a\\+', '12ab', '2-4' },
      { '\\_.\\+', 'abc', '0-3' },
      { '\\_[a-c]\\+', 'xxabc', '2-5' },
      { '\\_^abc', 'abc', '0-3' },
      { 'abc\\_$', 'abc', '0-3' },
    })
  end)

  itp('escape sequences name control characters', function()
    golden({
      { '\\t', 'a\tb', '1-2' },
      { '\\e', 'a\27b', '1-2' },
      { '\\r', 'a\rb', '1-2' },
      { '\\b', 'a\bb', '1-2' },
      { '\\\\', 'a\\b', '1-2' },
      { '\\/', 'a/b', '1-2' },
      { '\\.', 'a.b', '1-2' },
      { '\\*', 'a*b', '1-2' },
      { '\\[', 'a[b', '1-2' },
    })
  end)
end)

describe('case sensitivity', function()
  itp('\\c and \\C override the caller regardless of position', function()
    golden({
      { '\\cabc', 'ABC', '0-3' },
      { 'abc\\c', 'ABC', '0-3' },
      { 'a\\cbc', 'ABC', '0-3' },
      { '\\CABC', 'abc', 'nomatch' },
      { '\\Cabc', 'abc', '0-3' },
      -- \c wins over the rm_ic the caller passed, and \C over both
      { '\\cabc', 'ABC', '0-3', false },
      { '\\CABC', 'abc', 'nomatch', true },
    })
  end)

  itp('rm_ic folds case when no \\c or \\C is present', function()
    golden({
      { 'abc', 'ABC', '0-3', true },
      { 'abc', 'ABC', 'nomatch', false },
      { 'ABC', 'abc', '0-3', true },
      { '[a-c]\\+', 'ABC', '0-3', true },
      { '\\(a\\)\\1', 'aA', '0-2 1:0-1', true },
    })
  end)

  itp('case folding reaches non-ASCII characters', function()
    golden({
      { '\\cé', 'É', '0-2' },
      { '\\cÉ', 'é', '0-2' },
      { '\\cabcé', 'ABCÉ', '0-5' },
      { '\\Cé', 'É', 'nomatch' },
    })
  end)
end)

describe('multibyte input', function()
  itp('. and quantifiers count characters, offsets count bytes', function()
    golden({
      { '.', 'é', '0-2' },
      { '.\\{2}', 'éé', '0-4' },
      { '\\v.{3}', 'aéb', '0-4' },
      { '.\\+', 'héllo', '0-6' },
      { 'é\\+', 'xééy', '1-5' },
      { '\\v(é)+', 'éé', '0-4 1:2-4' },
    })
  end)

  itp('collections and classes handle multibyte members', function()
    golden({
      { '[é]', 'xéy', '1-3' },
      { '[éè]\\+', 'xéèy', '1-5' },
      { '[^é]\\+', 'ééab', '4-6' },
      { '[a-é]\\+', 'xyz', '0-3' },
      { '\\w\\+', 'aéb', '0-1' }, -- é is not a word character by default
    })
  end)

  itp('an incomplete or invalid byte sequence does not derail matching', function()
    -- Patterns and inputs are byte strings; the engine must cope with bytes
    -- that are not valid utf-8 rather than reading past them.
    deadline(10, function()
      for _, line in ipairs({ '\192', 'a\192b', '\255\255', 'a\224\160', '\237\160\128' }) do
        for _, pat in ipairs({ '.', '.\\+', '\\w\\+', '[a-z]\\+', '\\v.{1,3}', 'a\\|\\192' }) do
          eq(
            match(ENGINES.bt, pat, line),
            match(ENGINES.nfa, pat, line),
            'engines disagree on ' .. label(pat, line)
          )
        end
      end
    end)
  end)
end)

describe('\\zs and \\ze', function()
  itp('move the reported match without moving the match itself', function()
    golden({
      { 'foo\\zsbar', 'foobar', '3-6' },
      { 'foo\\zebar', 'foobar', '0-3' },
      { 'foo\\zsbar\\zebaz', 'foobarbaz', '3-6' },
      { '\\zsfoo', 'foo', '0-3' },
      { 'foo\\ze', 'foo', '0-3' },
      -- \zs inside a group still applies to the whole match
      { '\\(foo\\zs\\)bar', 'foobar', '3-6 1:0-3' },
      -- with a quantifier in front, \zs follows the last iteration
      { 'a\\+\\zsb', 'aaab', '3-4' },
      -- \ze before the start yields an empty match
      { 'foo\\zebar', 'xfoobar', '1-4' },
    })
  end)
end)

describe('lookaround', function()
  itp('\\@= and \\@! assert without consuming', function()
    golden({
      { 'foo\\(bar\\)\\@=', 'foobar', '0-3 1:3-6' },
      { 'foo\\(bar\\)\\@=', 'foobaz', 'nomatch' },
      { 'foo\\(bar\\)\\@!', 'foobaz', '0-3' },
      { 'foo\\(bar\\)\\@!', 'foobar', 'nomatch' },
      { '\\v(a)@=..', 'ab', '0-2 1:0-1' },
      { '\\v\\d+(px)@!', '12em', '0-2' },
    })
  end)

  itp('\\@<= and \\@<! look behind', function()
    golden({
      { '\\(foo\\)\\@<=bar', 'foobar', '3-6 1:0-3' },
      { '\\(foo\\)\\@<=bar', 'bazbar', 'nomatch' },
      { '\\(foo\\)\\@<!bar', 'bazbar', '3-6' },
      { '\\(foo\\)\\@<!bar', 'foobar', 'nomatch' },
      { '\\v(a)@<=b', 'ab', '1-2 1:0-1' },
      -- a bounded look-behind: \@123<= limits how far back to look
      { '\\(a\\)\\@1<=b', 'ab', '1-2 1:0-1' },
    })
  end)

  itp('\\@> matches atomically, without giving anything back', function()
    golden({
      { '\\(a*\\)\\@>a', 'aaa', 'nomatch' },
      { '\\(a*\\)\\@>b', 'aaab', '0-4 1:0-3' },
      { '\\v(a+)@>b', 'aab', '0-3 1:0-2' },
    })
  end)

  itp('nested lookaround terminates', function()
    deadline(10, function()
      golden({
        { '\\(a\\(b\\)\\@!\\)\\+', 'aac', '0-2 1:1-2' },
      })
      -- Nesting a capture inside a lookaround is a known divergence; see
      -- 'known engine divergences'.
    end)
  end)
end)

describe('known engine divergences', function()
  -- Cases where the backtracking and NFA engines disagree today. They are
  -- excluded from the differential corpus so the oracle stays meaningful,
  -- and pinned here instead: both sides are asserted, so a refactor that
  -- moves either engine still fails, and one that makes them agree fails
  -- loudly enough to come delete this block.
  --
  -- All of them concern which span a capture reports, never whether or where
  -- the overall match lands.

  --- @param pat string
  --- @param line string
  --- @param bt string expected on the backtracking engine
  --- @param nfa string expected on the NFA engine
  local function diverges(pat, line, bt, nfa)
    eq(bt, match(ENGINES.bt, pat, line), 'bt ' .. label(pat, line))
    eq(nfa, match(ENGINES.nfa, pat, line), 'nfa ' .. label(pat, line))
    t.neq(bt, nfa, 'divergence is gone; fold ' .. label(pat, line) .. ' back into the corpus')
  end

  itp('a star over a group that can match empty captures differently', function()
    -- The backtracking engine reports the span the group consumed on its
    -- last non-empty iteration; the NFA engine reports the trailing empty
    -- one. Both agree on the overall match.
    diverges('\\(a*\\)*', 'a', '0-1 1:0-1', '0-1 1:1-1')
    diverges('\\(a*\\)*', 'aaa', '0-3 1:0-3', '0-3 1:3-3')
  end)

  itp('a capture inside a lookaround is left unset by the NFA engine', function()
    diverges('\\(\\(a\\)\\@=a\\)\\@=a', 'aaa', '0-1 1:0-1 2:0-1', '0-1 2:0-1')
    diverges('\\v((a)@<=b)@<=c', 'abc', '2-3 1:1-2 2:0-1', '2-3 2:0-1')
  end)
end)

describe('the two engines agree', function()
  -- The differential corpus: a cross product of patterns and inputs, run on
  -- both engines. No expectations to maintain — any disagreement is a bug in
  -- one of them, and after a refactor it is almost always a fresh one.
  --
  -- Patterns that diverge today are deliberately absent; they live in 'known
  -- engine divergences' with both behaviours pinned. Adding one back here is
  -- how you find out it has been fixed.
  local PATTERNS = {
    -- literals and dots
    'abc',
    'a.c',
    '.',
    '.*',
    '.\\+',
    '.\\{2,4}',
    -- anchors
    '^abc',
    'abc$',
    '^.*$',
    '^\\(.\\)\\1',
    '\\<\\w\\+\\>',
    '\\<a',
    'a\\>',
    -- quantifiers
    'a*',
    'a\\+',
    'a\\?',
    'a\\{2,3}',
    'a\\{-}',
    'a\\{-1,}',
    'ab*c',
    '\\(ab\\)*',
    '\\(ab\\)\\{2,}',
    '\\(a*\\)\\+b',
    'a\\{-}b',
    'a.\\{-}b',
    -- alternation and grouping
    'a\\|b',
    'ab\\|ba',
    '\\(a\\|b\\)\\+',
    '\\%(a\\|b\\)\\{2}',
    '\\(a\\)\\(b\\)\\(c\\)',
    '\\(\\(a\\)b\\)c',
    'foo\\|foobar\\|f',
    -- backreferences
    '\\(a\\)\\1',
    '\\(.\\)\\1\\+',
    '\\(\\w\\)\\(\\w\\)\\2\\1',
    '\\(x*\\)y\\1',
    -- classes
    '\\d\\+',
    '\\w\\+',
    '\\s\\+',
    '\\a\\+',
    '\\u\\+',
    '\\x\\+',
    '\\h\\w*',
    '[abc]\\+',
    '[^abc]\\+',
    '[a-z0-9]\\+',
    '[[:alpha:]]\\+',
    '[[:digit:][:space:]]\\+',
    '[]a-]\\+',
    '\\_s\\+',
    '\\_.\\{2}',
    -- \zs \ze
    'a\\zsb',
    'a\\zeb',
    '\\w\\+\\zs\\d\\+',
    '\\zs.*\\ze',
    -- lookaround
    'a\\(b\\)\\@=',
    'a\\(b\\)\\@!',
    '\\(a\\)\\@<=b',
    '\\(a\\)\\@<!b',
    '\\(a*\\)\\@>b',
    '\\v(\\d+)@<=px',
    -- optional sequence
    'r\\%[ead]',
    'f\\%[oo]\\d',
    -- magic levels
    '\\v(a|b)+c?',
    '\\v\\d{2,}',
    '\\v(.)\\1',
    '\\Ma\\*b',
    '\\Va.c',
    '\\M\\(a\\)\\1',
    -- case
    '\\cabc',
    '\\CABC',
    '\\ca\\+',
    -- character codes
    '\\%d97\\+',
    '\\%x61\\%x62',
    '\\%u0061',
    -- text anchors
    '\\%^a',
    'c\\%$',
    -- empty and near-empty
    '',
    '\\(\\)',
    '\\(\\)*',
    'a\\{0}',
  }

  local INPUTS = {
    '',
    'a',
    'b',
    'ab',
    'abc',
    'aaa',
    'aaab',
    'abab',
    'abcabc',
    'ABC',
    'aAbBcC',
    '  ab  ',
    'a\tb',
    '123',
    'ab123cd',
    'x_y1',
    'read',
    'reads',
    'foo',
    'foobar',
    'the the',
    'aabbaa',
    '[a-]',
    'a]b',
    'héllo',
    'ééé',
    'a\192b',
    ('a'):rep(64),
    ('ab'):rep(32),
    'aaaaaaaaaaaaaaaaaaaaX',
  }

  -- Split across several `itp`s so a crash on one slice still leaves the
  -- others reporting, and so no single forked child runs unbounded.
  local SLICES = 4
  for slice = 1, SLICES do
    itp(('slice %d/%d of the differential corpus'):format(slice, SLICES), function()
      deadline(60, function()
        local checked = 0
        for i, pat in ipairs(PATTERNS) do
          if (i - 1) % SLICES == slice - 1 then
            for _, line in ipairs(INPUTS) do
              for _, ic in ipairs({ false, true }) do
                local bt = match(ENGINES.bt, pat, line, ic)
                local nfa = match(ENGINES.nfa, pat, line, ic)
                eq(bt, nfa, ('engines disagree (ic=%s) on %s'):format(ic, label(pat, line)))
                checked = checked + 1
              end
            end
          end
        end
        t.ok(checked > 0, 'nonempty slice', 'slice checked nothing')
      end)
    end)
  end

  itp('the automatic engine agrees with whichever it picks', function()
    -- `\%#=0` lets vim_regcomp choose, and fall back to the backtracking
    -- engine when the NFA one refuses the pattern. The result must still be
    -- one of the two, never a third answer.
    deadline(60, function()
      for _, pat in ipairs(PATTERNS) do
        for _, line in ipairs(INPUTS) do
          local auto = match('\\%#=0', pat, line)
          local bt = match(ENGINES.bt, pat, line)
          local nfa = match(ENGINES.nfa, pat, line)
          t.ok(
            auto == bt or auto == nfa,
            'auto matches an engine',
            ('auto=%s bt=%s nfa=%s for %s'):format(auto, bt, nfa, label(pat, line))
          )
        end
      end
    end)
  end)
end)

describe('pathological patterns', function()
  -- Everything here is about liveness, not results: the engine may match or
  -- fail, but it must return, and it must not take the process with it.
  -- SIGALRM turns a hang into a failed test.

  -- Shapes with an exponential number of ways to split the input: nested
  -- quantifiers over an atom that can match the same text more than one way.
  local BOMBS = {
    '\\(a\\+\\)\\+b',
    '\\(a*\\)*b',
    '\\(\\(a\\)\\+\\)\\+b',
    '\\([a-z]\\+\\)*x',
    '\\v(a|a)+b',
    '\\v(a|aa)+b',
    '\\v(.*){0,20}x',
    '.*.*.*.*.*x',
    '\\v(a{1,10}){1,10}b',
    '\\v(a+)+(b+)+c',
  }

  --- Inputs that force the full search: no `b`/`c`/`x` to match, so every
  --- split has to be tried before the engine can report failure.
  --- @param n integer
  local function bomb_inputs(n)
    return { ('a'):rep(n), ('a'):rep(n) .. 'b', ('a'):rep(n) .. 'c', ('ab'):rep(n / 2) }
  end

  itp('the NFA engine stays linear on backtracking bombs', function()
    -- This is the property that keeps the editor usable: the automatic
    -- engine tries the NFA one first, so anything it handles in stride never
    -- reaches the backtracker. An input long enough that an exponential
    -- search could not possibly finish inside the deadline.
    deadline(30, function()
      for _, pat in ipairs(BOMBS) do
        for _, line in ipairs(bomb_inputs(40)) do
          t.ok(
            match(ENGINES.nfa, pat, line) ~= nil,
            'terminates',
            ('no result for %s'):format(label(pat, line))
          )
        end
      end
    end)
  end)

  itp('the automatic engine stays linear on backtracking bombs', function()
    deadline(30, function()
      for _, pat in ipairs(BOMBS) do
        for _, line in ipairs(bomb_inputs(40)) do
          t.ok(
            match('\\%#=0', pat, line) ~= nil,
            'terminates',
            ('no result for %s'):format(label(pat, line))
          )
        end
      end
    end)
  end)

  itp('the backtracking engine terminates on bombs it can still afford', function()
    -- The backtracking engine really is exponential in the input length —
    -- that is what the NFA engine exists to avoid, not a defect to assert
    -- away. `\%#=1` with a longer input than this does not come back, and a
    -- user who types it gets what they asked for. Keep the input short so
    -- the code path is exercised without the test becoming the hang.
    deadline(30, function()
      for _, pat in ipairs(BOMBS) do
        for _, line in ipairs(bomb_inputs(10)) do
          t.ok(
            match(ENGINES.bt, pat, line) ~= nil,
            'terminates',
            ('no result for %s'):format(label(pat, line))
          )
        end
      end
    end)
  end)

  itp('deeply nested and deeply repeated patterns terminate', function()
    local DEEP = {
      ('\\%('):rep(40) .. 'a' .. ('\\)'):rep(40),
      '\\v' .. ('%('):rep(40) .. 'a' .. (')'):rep(40),
      ('\\('):rep(9) .. 'a' .. ('\\)'):rep(9), -- nine is the group limit
      '\\v' .. ('('):rep(9) .. 'a' .. (')'):rep(9),
      ('a\\?'):rep(40) .. ('a'):rep(40),
      '\\v' .. ('a?'):rep(40) .. ('a'):rep(40),
      '[a-z]\\{1,1000}',
      'a\\{1,1000}', -- see 'a large \{n,m} bound' below for why not larger
      ('\\|a'):rep(200):sub(3),
      ('.'):rep(200),
      '\\v' .. ('(a)'):rep(9) .. '\\1\\2\\3\\4\\5\\6\\7\\8\\9',
    }
    deadline(60, function()
      for _, pat in ipairs(DEEP) do
        for _, line in ipairs({ '', 'a', ('a'):rep(60), ('ab'):rep(30) }) do
          for _, engine in pairs(ENGINES) do
            t.ok(
              match(engine, pat, line) ~= nil,
              'terminates',
              ('no result for %s'):format(label(pat, line))
            )
          end
        end
      end
    end)
  end)

  itp('malformed patterns are rejected rather than mis-compiled', function()
    -- Truncated and unbalanced constructs: the engine may reject them, but
    -- it must not read past the end of the pattern to decide.
    local BROKEN = {
      '\\',
      '\\v',
      '\\M',
      '\\V',
      '\\%',
      '\\%#',
      '\\%#=',
      '\\%#=9',
      '\\%d',
      '\\%x',
      '\\%u',
      '\\%[',
      '\\%[abc',
      '\\(',
      '\\)',
      '\\%(',
      '\\z(',
      '\\z1',
      '[',
      '[a',
      '[a-',
      '[[:',
      '[[:foo:]]',
      '\\{',
      'a\\{',
      'a\\{1',
      'a\\{1,',
      '\\@',
      'a\\@',
      'a\\@<',
      '\\v(',
      '\\v)',
      '\\v[',
      '\\v{',
      '\\v%(',
      '\\v%[',
      ('\\('):rep(20),
      ('('):rep(100),
      ('['):rep(50),
      '\\v\\C[\\zs',
      '\\%#=1\\v(a+)+b\\',
    }
    deadline(60, function()
      for _, pat in ipairs(BROKEN) do
        for _, engine in pairs(ENGINES) do
          t.ok(
            match(engine, pat, 'aaabbb') ~= nil,
            'terminates',
            ('no result for /%s/'):format(pat)
          )
        end
      end
    end)
  end)

  itp('a large \\{n,m} bound is refused rather than expanded, up to a point', function()
    -- `nfa_regpiece` compiles `\{n,m}` by emitting the atom `m` times, and
    -- `post2nfa` then walks that postfix recursively — so the bound is a
    -- recursion depth. The guard against it (regexp.rs, `maxval > 500`) is
    -- conditional on RE_AUTO, which means it only fires for the automatic
    -- engine. Forcing `\%#=2` walks straight past it.
    --
    -- These are the bounds that are safe today. The unsafe ones are the
    -- pending test below; keep this one as the record of where the edge is.
    deadline(30, function()
      for _, n in ipairs({ 10, 100, 255, 256, 500, 1000 }) do
        local pat = ('a\\{1,%d}'):format(n)
        eq('0-1', match(ENGINES.nfa, pat, 'a'), label(pat, 'a'))
        eq('0-1', match(ENGINES.bt, pat, 'a'), label(pat, 'a'))
        eq('0-1', match('\\%#=0', pat, 'a'), label(pat, 'a'))
      end
      -- The automatic engine is safe at any bound: it declines the NFA
      -- compile and falls back to the backtracking engine, which expands
      -- `\{n,m}` iteratively.
      eq('0-1', match('\\%#=0', 'a\\{1,50000}', 'a'))
      eq('0-1', match(ENGINES.bt, 'a\\{1,50000}', 'a'))
    end)
  end)

  pending('a large \\{n,m} bound on a forced NFA engine overflows the stack', function()
    -- Aborts the process: `match('aaa', '\%#=2a\{1,5000}')` is enough to
    -- kill nvim outright, from a plain :echo. Unfixed; the RE_AUTO-only
    -- guard described above is the cause. Restore this to `itp` once the
    -- recursion in post2nfa is bounded independently of RE_AUTO.
    deadline(30, function()
      for _, n in ipairs({ 5000, 20000, 100000 }) do
        local pat = ('a\\{1,%d}'):format(n)
        t.ok(match(ENGINES.nfa, pat, 'a') ~= nil, 'terminates', label(pat, 'a'))
      end
    end)
  end)

  itp('long inputs and long patterns terminate', function()
    deadline(60, function()
      local long = ('abcdefghij'):rep(500)
      for _, pat in ipairs({
        '.*',
        '.\\+x',
        '\\w\\+',
        '\\(abc\\)\\+',
        '[a-j]\\{100,}',
        'j\\zsa',
        '\\(a\\)\\@<=b',
        ('abcdefghij'):rep(50),
      }) do
        for _, engine in pairs(ENGINES) do
          t.ok(
            match(engine, pat, long) ~= nil,
            'terminates',
            ('no result for /%s/ on a %d byte line'):format(pat, #long)
          )
        end
      end
    end)
  end)
end)

describe('fuzzing', function()
  -- Randomly assembled patterns, seeded so a failure reproduces. Most are
  -- syntactically invalid, which is the point: rejection paths get far less
  -- hand-written coverage than matching ones, and that is where the reads
  -- past the end of the pattern live.
  local ATOMS = {
    'a',
    'b',
    '1',
    '.',
    '*',
    '\\+',
    '\\?',
    '\\{',
    '\\{2,3}',
    '\\{-}',
    '[',
    ']',
    '[a-z]',
    '\\(',
    '\\)',
    '\\%(',
    '\\|',
    '^',
    '$',
    '\\<',
    '\\>',
    '\\zs',
    '\\ze',
    '\\_',
    '\\',
    '\\d',
    '\\w',
    '\\s',
    '\\1',
    '\\@=',
    '\\@!',
    '\\@<=',
    '\\@>',
    '\\%[',
    '\\%d',
    '\\%^',
    '\\%$',
    '\\c',
    '\\C',
    'é',
    '\192',
  }
  local PREFIXES = { '', '\\v', '\\m', '\\M', '\\V' }
  local INPUT_CHARS = 'abc123 \t.[](){}|*+?\\^$éA'

  --- @param rand fun(n: integer): integer
  local function random_pattern(rand)
    local pat = {}
    for _ = 1, rand(12) + 1 do
      pat[#pat + 1] = ATOMS[rand(#ATOMS)]
    end
    return PREFIXES[rand(#PREFIXES)] .. table.concat(pat)
  end

  --- @param rand fun(n: integer): integer
  local function random_input(rand)
    local s = {}
    for _ = 1, rand(20) do
      local i = rand(#INPUT_CHARS)
      s[#s + 1] = INPUT_CHARS:sub(i, i)
    end
    return table.concat(s)
  end

  -- Several seeds, each its own forked child: a crash names the seed that
  -- produced it, and the remaining seeds still run.
  for _, seed in ipairs({ 1, 42, 1337, 20260727 }) do
    itp(('seed %d survives 2000 random patterns'):format(seed), function()
      math.randomseed(seed)
      local rand = function(n)
        return math.random(n)
      end
      deadline(120, function()
        for i = 1, 2000 do
          local pat = random_pattern(rand)
          local line = random_input(rand)
          for _, engine in pairs(ENGINES) do
            t.ok(
              match(engine, pat, line) ~= nil,
              'terminates',
              ('seed %d iteration %d: no result for %s'):format(seed, i, label(pat, line))
            )
          end
        end
      end)
    end)
  end
end)
