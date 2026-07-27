-- Regexp behaviour that only exists once there is a buffer: `vim_regexec_multi`
-- and everything layered on it. The single-line engine is covered far more
-- densely in test/unit/regexp_spec.lua, which calls `vim_regcomp`/`vim_regexec`
-- directly; this spec deliberately does not repeat it. What lives here is the
-- part that spec structurally cannot reach:
--
--   * matching across line boundaries (`\n`, `\_.`, `\_s`)
--   * the buffer-position atoms (`\%23l`, `\%23c`, `\%23v`, `\%V`, `\%#`)
--   * :substitute, including backreferences, `\zs`, and `\=submatch()`
--   * search offsets and directions
--
-- As in the unit spec, every case runs on both the backtracking (`\%#=1`) and
-- NFA (`\%#=2`) engines, and the pathological patterns assert liveness rather
-- than results — an editor that survives them is the whole point.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear = n.clear
local command = n.command
local exec = n.exec
local fn = n.fn
local api = n.api
local eq = t.eq
local ok = t.ok
local pcall_err = t.pcall_err

--- Engine-selecting prefixes, as `\%#=N`.
local ENGINES = { bt = '\\%#=1', nfa = '\\%#=2' }

--- Run `fn(prefix)` once per engine, labelling any failure with the engine.
local function each_engine(f)
  for name, prefix in pairs(ENGINES) do
    local okay, err = pcall(f, prefix)
    if not okay then
      error(('[%s engine] %s'):format(name, tostring(err)), 0)
    end
  end
end

local function set_lines(lines)
  api.nvim_buf_set_lines(0, 0, -1, true, lines)
end

describe('regexp matching across lines', function()
  before_each(clear)

  it('matches \\n at the end of a line', function()
    set_lines({ 'foo', 'bar', 'baz' })
    each_engine(function(e)
      -- search() reports the line the match starts on
      eq(1, fn.search(e .. 'foo\\nbar', 'nw'))
      eq(2, fn.search(e .. 'bar\\nbaz', 'nw'))
      eq(0, fn.search(e .. 'foo\\nbaz', 'nw'))
    end)
  end)

  it('\\_. and \\_s cross the line boundary', function()
    set_lines({ 'abc', 'def' })
    each_engine(function(e)
      eq(1, fn.search(e .. 'c\\_.d', 'nw'))
      eq(1, fn.search(e .. 'c\\_sd', 'nw'))
      -- "abc\ndef": five characters between the a and the f, newline included
      eq(1, fn.search(e .. 'a\\_.\\{5}f', 'nw'))
      eq(0, fn.search(e .. 'a\\_.\\{4}f', 'nw'))
      -- without \_ the class stops at the end of the line
      eq(0, fn.search(e .. 'c.d', 'nw'))
    end)
  end)

  it('\\_[] and \\_x classes span lines', function()
    set_lines({ 'ab', 'cd' })
    each_engine(function(e)
      eq(1, fn.search(e .. 'b\\_[a-z]c', 'nw'))
      eq(1, fn.search(e .. '\\_a\\{4}', 'nw'))
    end)
  end)

  it('a quantifier can span many lines', function()
    set_lines({ 'a', 'a', 'a', 'a', 'b' })
    each_engine(function(e)
      eq(1, fn.search(e .. '\\(a\\n\\)\\{4}b', 'nw'))
      eq(0, fn.search(e .. '\\(a\\n\\)\\{5}b', 'nw'))
    end)
  end)

  it('^ and $ anchor per line, \\%^ and \\%$ to the buffer', function()
    set_lines({ 'foo', 'foo', 'foo' })
    each_engine(function(e)
      eq(1, fn.search(e .. '\\%^foo', 'nw'))
      eq(3, fn.search(e .. 'foo\\%$', 'nw'))
      -- ^ matches on every line, so a wrapping search from line 2 finds 3
      command('call cursor(2, 1)')
      eq(3, fn.search(e .. '^foo', 'nW'))
    end)
  end)

  it('reports the end position of a multi-line match', function()
    set_lines({ 'start', 'middle', 'end' })
    each_engine(function(e)
      -- searchpos with 'e' reports where the match ends
      eq({ 3, 3 }, fn.searchpos(e .. 'start\\_.*end', 'nwe'))
    end)
  end)
end)

describe('buffer position atoms', function()
  before_each(clear)

  it('\\%23l restricts a match to a line', function()
    set_lines({ 'match', 'match', 'match' })
    each_engine(function(e)
      eq(2, fn.search(e .. '\\%2lmatch', 'nw'))
      eq(3, fn.search(e .. '\\%>2lmatch', 'nw'))
      eq(1, fn.search(e .. '\\%<2lmatch', 'nw'))
      eq(0, fn.search(e .. '\\%9lmatch', 'nw'))
    end)
  end)

  it('\\%23c and \\%23v restrict a match to a column', function()
    set_lines({ 'abcdef' })
    each_engine(function(e)
      -- 'c' so the search may match at the cursor: without it `\%<3c` would
      -- skip column 1 and report the only other column it allows.
      eq({ 1, 3 }, fn.searchpos(e .. '\\%3c.', 'ncw'))
      eq({ 1, 4 }, fn.searchpos(e .. '\\%>3c.', 'ncw'))
      eq({ 1, 1 }, fn.searchpos(e .. '\\%<3c.', 'ncw'))
      eq({ 1, 3 }, fn.searchpos(e .. '\\%3v.', 'ncw'))
    end)
  end)

  it('\\%23v counts screen columns, not bytes', function()
    -- A tab is one byte but eight screen columns, so \%c and \%v part ways.
    command('setlocal tabstop=8 noexpandtab')
    set_lines({ '\tX' })
    each_engine(function(e)
      eq({ 1, 2 }, fn.searchpos(e .. '\\%2cX', 'nw')) -- byte column 2
      eq({ 1, 2 }, fn.searchpos(e .. '\\%9vX', 'nw')) -- screen column 9
      eq({ 0, 0 }, fn.searchpos(e .. '\\%2vX', 'nw'))
    end)
  end)

  it('\\%# matches at the cursor', function()
    set_lines({ 'aaaa' })
    each_engine(function(e)
      fn.cursor(1, 3)
      eq({ 1, 3 }, fn.searchpos(e .. '\\%#a', 'ncw'))
    end)
  end)

  it('\\%V restricts a match to the visual selection', function()
    set_lines({ 'abcdefghij' })
    each_engine(function(e)
      -- Select columns 3-6 (cdef) and leave the selection behind in '< '>.
      fn.cursor(1, 3)
      command('normal! v3l\27')
      eq({ 0, 1, 3, 0 }, fn.getpos("'<"))
      eq({ 0, 1, 6, 0 }, fn.getpos("'>"))
      fn.cursor(1, 1)
      eq({ 1, 3 }, fn.searchpos(e .. '\\%Vc', 'ncw'))
      eq({ 1, 6 }, fn.searchpos(e .. '\\%Vf', 'ncw'))
      -- b and g sit just outside it
      eq({ 0, 0 }, fn.searchpos(e .. '\\%Vb', 'ncw'))
      eq({ 0, 0 }, fn.searchpos(e .. '\\%Vg', 'ncw'))
    end)
  end)

  it('rejects a \\% value that cannot fit', function()
    eq([[Vim:E951: \% value too large]], pcall_err(command, '/\\v%18446744071562067968c'))
    eq([[Vim:E951: \% value too large]], pcall_err(command, '/\\v%2147483648c'))
  end)
end)

describe(':substitute', function()
  before_each(clear)

  it('substitutes with backreferences', function()
    each_engine(function(e)
      set_lines({ 'foo bar', 'baz qux' })
      command(('%%s/%s\\(\\w\\+\\) \\(\\w\\+\\)/\\2 \\1/'):format(e))
      eq({ 'bar foo', 'qux baz' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)

  it('honours \\zs and \\ze when deciding what to replace', function()
    each_engine(function(e)
      set_lines({ 'prefix-value-suffix' })
      command(('%%s/%sprefix-\\zsvalue\\ze-suffix/NEW/'):format(e))
      eq({ 'prefix-NEW-suffix' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)

  it('substitutes across lines', function()
    each_engine(function(e)
      set_lines({ 'one', 'two', 'three' })
      command(('%%s/%sone\\ntwo/MERGED/'):format(e))
      eq({ 'MERGED', 'three' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)

  it('evaluates \\= expressions with submatch()', function()
    each_engine(function(e)
      set_lines({ 'a1 b2 c3' })
      command(('%%s/%s\\(\\a\\)\\(\\d\\)/\\=submatch(2) .. submatch(1)/g'):format(e))
      eq({ '1a 2b 3c' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)

  it('replaces every empty match with the g flag, except at end of line', function()
    each_engine(function(e)
      set_lines({ 'abc' })
      command(('%%s/%sx*/-/g'):format(e))
      -- No trailing '-': `do_sub` skips a repeated empty match once it is
      -- sitting on the NUL, so the one at end of line is dropped.
      eq({ '-a-b-c' }, api.nvim_buf_get_lines(0, 0, -1, true))
      -- but a pattern that only matches there is still substituted
      set_lines({ 'abc' })
      command(('%%s/%s$/X/'):format(e))
      eq({ 'abcX' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)

  it('reports how many substitutions it made and on how many lines', function()
    each_engine(function(e)
      set_lines({ 'aaa', 'aaa', 'bbb' })
      eq('6 substitutions on 2 lines', (n.exec_capture(('%%s/%sa/x/g'):format(e)):gsub('^%s+', '')))
    end)
  end)

  it('\\~ and ~ reuse the previous replacement', function()
    each_engine(function(e)
      set_lines({ 'aaa', 'aaa' })
      command(('1s/%sa/X/'):format(e))
      command(('2s/%sa/~/'):format(e))
      eq({ 'Xaa', 'Xaa' }, api.nvim_buf_get_lines(0, 0, -1, true))
    end)
  end)
end)

describe('search offsets and direction', function()
  before_each(clear)

  it('searches backwards', function()
    set_lines({ 'target', 'middle', 'target' })
    each_engine(function(e)
      fn.cursor(2, 1)
      eq(1, fn.search(e .. 'target', 'bnW'))
      eq(3, fn.search(e .. 'target', 'nW'))
    end)
  end)

  it('applies /e and /s offsets', function()
    set_lines({ 'abc def' })
    each_engine(function(e)
      eq({ 1, 3 }, fn.searchpos(e .. 'abc', 'nwe'))
      eq({ 1, 1 }, fn.searchpos(e .. 'abc', 'nw'))
    end)
  end)

  it('wraps and stops as the flags ask', function()
    set_lines({ 'x', 'y', 'x' })
    each_engine(function(e)
      fn.cursor(3, 1)
      eq(1, fn.search(e .. 'x', 'nw')) -- wraps
      eq(0, fn.search(e .. 'x', 'nW')) -- does not
    end)
  end)
end)

describe('match functions over a buffer', function()
  before_each(clear)

  it('matchbufline finds every match in a range', function()
    set_lines({ 'a1', 'b2', 'c3' })
    each_engine(function(e)
      eq({
        { byteidx = 1, lnum = 1, text = '1' },
        { byteidx = 1, lnum = 2, text = '2' },
        { byteidx = 1, lnum = 3, text = '3' },
      }, fn.matchbufline(api.nvim_get_current_buf(), e .. '\\d', 1, '$'))
      -- a range narrower than the buffer
      eq(
        { { byteidx = 1, lnum = 2, text = '2' } },
        fn.matchbufline(api.nvim_get_current_buf(), e .. '\\d', 2, 2)
      )
    end)
  end)

  it('matchbufline returns submatches when asked, padded to nine', function()
    set_lines({ 'key=value' })
    each_engine(function(e)
      eq(
        {
          {
            byteidx = 0,
            lnum = 1,
            submatches = { 'key', 'value', '', '', '', '', '', '', '' },
            text = 'key=value',
          },
        },
        fn.matchbufline(
          api.nvim_get_current_buf(),
          e .. '\\(\\w\\+\\)=\\(\\w\\+\\)',
          1,
          '$',
          { submatches = true }
        )
      )
    end)
  end)
end)

describe('the two engines agree over a buffer', function()
  before_each(clear)

  -- The differential oracle again, this time through `vim_regexec_multi`.
  -- Whatever one engine finds in the buffer, the other must find in the same
  -- place — no expectations to keep current.
  local PATTERNS = {
    'foo',
    '\\<\\w\\+\\>',
    '^\\s*\\w\\+',
    '\\d\\+$',
    'foo\\nbar',
    'foo\\_.\\{-}baz',
    '\\_s\\+',
    '\\(\\w\\+\\)\\n\\1',
    'a\\{2,}',
    '\\v(foo|bar)+',
    '\\v^(\\w+)\\s+\\1$',
    '[[:alpha:]]\\+\\d',
    '\\%2l\\w\\+',
    '\\%>1l\\%<4l\\w\\+',
    '\\%3c.',
    '\\zsfoo\\ze',
    '\\(foo\\)\\@<=bar',
    '\\(foo\\)\\@<!bar',
    'x\\{-}y',
    '\\v(.)\\1',
  }

  local LINES = {
    'foo bar',
    'foo',
    'bar',
    '  indented',
    'aaa',
    'foo bar baz',
    'the the',
    'x123',
    '',
    'aabb',
    'end',
  }

  it('for search()', function()
    set_lines(LINES)
    for _, pat in ipairs(PATTERNS) do
      fn.cursor(1, 1)
      local bt = fn.searchpos(ENGINES.bt .. pat, 'ncw')
      fn.cursor(1, 1)
      local nfa = fn.searchpos(ENGINES.nfa .. pat, 'ncw')
      eq(bt, nfa, ('engines disagree on search(/%s/)'):format(pat))
    end
  end)

  it('for matchbufline()', function()
    set_lines(LINES)
    local buf = api.nvim_get_current_buf()
    for _, pat in ipairs(PATTERNS) do
      local opts = { submatches = true }
      local bt = fn.matchbufline(buf, ENGINES.bt .. pat, 1, '$', opts)
      local nfa = fn.matchbufline(buf, ENGINES.nfa .. pat, 1, '$', opts)
      eq(bt, nfa, ('engines disagree on matchbufline(/%s/)'):format(pat))
    end
  end)

  it('for :substitute', function()
    for _, pat in ipairs(PATTERNS) do
      local function subst(prefix)
        set_lines(LINES)
        pcall(command, ('silent! %%s/%s%s/<&>/g'):format(prefix, pat))
        return api.nvim_buf_get_lines(0, 0, -1, true)
      end
      eq(subst(ENGINES.bt), subst(ENGINES.nfa), ('engines disagree on :s/%s/'):format(pat))
    end
  end)
end)

describe('the editor survives pathological patterns', function()
  before_each(clear)

  -- Liveness, at the level a user actually meets it. Each pattern runs in a
  -- child Nvim under `jobwait`, so a hang is a timeout and a crash is a
  -- signal — either way one failed assertion, not a wedged test run. The
  -- in-process suite cannot do this: a hang there hangs the harness too.

  --- @param script string Vimscript to run in a child Nvim
  --- @param ms integer how long to allow before calling it a hang
  --- @return integer exit status, or -1 on timeout
  local function run_child(script, ms)
    local job = fn.jobstart({ n.nvim_prog, '--clean', '--headless', '-c', script, '-c', 'qall!' })
    local status = fn.jobwait({ job }, ms)[1]
    if status == -1 then
      fn.jobstop(job)
    end
    return status
  end

  --- Assert a child Nvim runs `script` to completion: not a timeout (-1) and
  --- not a signal (anything above 128).
  local function survives(what, script, ms)
    local status = run_child(script, ms or 20000)
    local why = status == -1 and 'timed out (the pattern did not terminate)'
      or status > 128 and ('died on signal %d'):format(status - 128)
      or ('exited with %d'):format(status)
    ok(status == 0, 'a clean exit', ('%s: child %s'):format(what, why))
  end

  --- `pat` as a Vimscript single-quoted literal, so it can be interpolated
  --- into the child's `-c` argument without another round of backslash
  --- mangling. `vim.fn` is the child's, not ours — this side has no Nvim.
  local function vimstr(pat)
    return "'" .. pat:gsub("'", "''") .. "'"
  end

  it('survives backtracking bombs on the default engine', function()
    -- The default engine is what a user gets by typing a pattern, so this is
    -- the case that decides whether the editor stays usable.
    local bombs = {
      '\\(a\\+\\)\\+b',
      '\\(a*\\)*b',
      '\\([a-z]\\+\\)*x',
      '\\v(a|a)+b',
      '\\v(a+)+(b+)+c',
      '.*.*.*.*.*x',
      '\\v(.{0,50}){0,50}x',
    }
    for _, pat in ipairs(bombs) do
      survives(
        ('match() with /%s/'):format(pat),
        ('call match(repeat("a", 40), %s)'):format(vimstr(pat))
      )
    end
  end)

  it('survives backtracking bombs in a buffer search', function()
    survives(
      'buffer search',
      table.concat({
        'call setline(1, repeat(["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"], 20))',
        'call search("\\\\(a\\\\+\\\\)\\\\+b", "nw")',
        'silent! %s/\\v(a+)+b/X/g',
      }, ' | ')
    )
  end)

  it('survives malformed patterns', function()
    -- Truncated constructs, run through the whole pattern surface at once:
    -- match(), search() and :substitute reach different callers of the same
    -- compiler, and a read past the end of the pattern shows up in whichever
    -- one happens to have the shortest buffer.
    local broken = {
      '\\',
      '\\%',
      '\\%#=',
      '\\(',
      '\\)',
      '\\%(',
      '\\z(',
      '[',
      '[a-',
      '[[:',
      'a\\{',
      'a\\{1,',
      '\\@',
      '\\v(',
      '\\v%[',
      '\\%[',
      string.rep('\\(', 30),
      string.rep('(', 100),
    }
    for _, pat in ipairs(broken) do
      local q = vimstr(pat)
      survives(
        ('malformed /%s/'):format(pat),
        table.concat({
          ('silent! call match("aaabbb", %s)'):format(q),
          'call setline(1, ["aaabbb"])',
          ('silent! call search(%s, "nw")'):format(q),
          ('silent! execute "%%s/" .. %s .. "/X/g"'):format(q),
        }, ' | ')
      )
    end
  end)

  it('survives a large \\{n,m} bound on the default engine', function()
    -- `\%#=2a\{1,5000}` overflows the stack and aborts; the default engine
    -- declines that NFA compile and falls back, so it must stay standing.
    -- See test/unit/regexp_spec.lua, 'a large \\{n,m} bound'.
    survives('large bound', 'call match("aaa", "a\\\\{1,50000}")')
  end)

  it('survives syntax highlighting built from many patterns', function()
    -- A real syntax file compiles hundreds of patterns in one go and then
    -- runs them over every line: the densest regexp workload the editor has,
    -- and the one where a compile-side leak or a stale `regprog` pointer
    -- shows up first. `synID()` forces the highlighter to actually run them —
    -- headless has no screen to redraw.
    survives(
      'syntax highlighting',
      table.concat({
        'call setline(1, repeat(["function! s:F(a) abort", "  let x = {\'k\': [1, 2]}",'
          .. ' "  \\" a comment", "  return a:a =~# \'\\\\vpat+\'", "endfunction"], 40))',
        'setfiletype vim',
        'syntax on',
        'for l in range(1, line("$")) | for c in range(1, col([l, "$"]))'
          .. ' | call synID(l, c, 1) | endfor | endfor',
      }, ' | '),
      60000
    )
  end)
end)

describe('regexpengine option', function()
  before_each(clear)

  it('selects the engine when no \\%#= prefix is given', function()
    set_lines({ 'aaa' })
    for _, value in ipairs({ 0, 1, 2 }) do
      command('set regexpengine=' .. value)
      eq(0, fn.match('aaa', 'a\\+'), 're=' .. value)
      eq(1, fn.search('a\\+', 'nw'), 're=' .. value)
    end
  end)

  it('rejects a value outside 0-2', function()
    eq('Vim(set):E474: Invalid argument: regexpengine=3', pcall_err(command, 'set regexpengine=3'))
  end)

  it('\\%#= overrides the option', function()
    exec('set regexpengine=1')
    -- Nothing observable should change; the point is that both compile and
    -- run with the option pointing the other way.
    each_engine(function(e)
      eq(0, fn.match('aaa', e .. 'a\\+'))
    end)
  end)

  it('rejects a \\%#= that is not 0, 1 or 2', function()
    eq(
      'Vim:E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used ',
      pcall_err(fn.match, 'aaa', '\\%#=3a')
    )
  end)
end)
