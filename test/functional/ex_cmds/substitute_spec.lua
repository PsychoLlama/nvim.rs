-- The `:s` result-line oracle.
--
-- `:substitute` builds its result line in one growable buffer, copying the
-- untouched runs of the old line around each replacement, and `:s///c` drives
-- that buffer one answer at a time: `y` and `n` decide a match, `a` and `q`
-- and `l` end the loop, and CTRL-E/CTRL-Y scroll under the prompt without
-- answering it. What the screen shows between two answers is the only place
-- the half-built line is visible, so these assert the screen as well as the
-- buffer.
--
-- The Ex-mode marker `:s///c` prints instead of highlighting is a documented
-- divergence from Vim and is asserted here as it stands.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()
local Screen = require('test.functional.ui.screen')

local clear = n.clear
local command = n.command
local eq = t.eq
local feed = n.feed
local fn = n.fn
local api = n.api

--- The buffer, as a list of lines.
local function lines()
  return api.nvim_buf_get_lines(0, 0, -1, true)
end

--- Fill the buffer with `text`, one line per element.
local function fill(text)
  api.nvim_buf_set_lines(0, 0, -1, true, text)
  command('1')
end

describe(':substitute', function()
  before_each(clear)

  describe('with the c flag', function()
    local screen

    before_each(function()
      screen = Screen.new(75, 8)
      fill({ 'x x x', 'y', 'x x', 'z' })
    end)

    it('replaces the matches answered y and leaves the ones answered n', function()
      feed(':%s/x/Q/gc<CR>')
      -- The match under the prompt carries its own highlight; the ones still
      -- to come carry Search.
      screen:expect([[
        {2:x} {10:x} {10:x}                                                                      |
        y                                                                          |
        {10:x} {10:x}                                                                        |
        z                                                                          |
        {1:~                                                                          }|*3
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      feed('y')
      -- The result line is rebuilt from the start: the answered match is
      -- already replaced while the rest of the line is still the old text.
      screen:expect([[
        Q {2:x} {10:x}                                                                      |
        y                                                                          |
        {10:x} {10:x}                                                                        |
        z                                                                          |
        {1:~                                                                          }|*3
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      feed('n')
      -- A refused match leaves the old text where it stood, and the copy of
      -- the untouched run either side of it is what the next answer extends.
      screen:expect([[
        Q {10:x} {2:x}                                                                      |
        y                                                                          |
        {10:x} {10:x}                                                                        |
        z                                                                          |
        {1:~                                                                          }|*3
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      feed('y')
      eq({ 'Q x Q', 'y', 'x x', 'z' }, lines())
      -- ... and the following lines are still offered.
      feed('n')
      feed('y')
      eq({ 'Q x Q', 'y', 'x Q', 'z' }, lines())
      eq('', api.nvim_get_vvar('errmsg'))
    end)

    it('a replaces the rest without asking and q stops where it stands', function()
      fill({ 'x x x', 'x x x' })
      feed(':%s/x/Q/gc<CR>')
      feed('y')
      feed('a')
      eq({ 'Q Q Q', 'Q Q Q' }, lines())

      fill({ 'x x x', 'x x x' })
      feed(':%s/x/Q/gc<CR>')
      feed('y')
      feed('q')
      eq({ 'Q x x', 'x x x' }, lines())
    end)

    it('l replaces the match it is answering and then stops', function()
      fill({ 'x x x', 'x x x' })
      feed(':%s/x/Q/gc<CR>')
      feed('y')
      feed('l')
      eq({ 'Q Q x', 'x x x' }, lines())
    end)

    it('CTRL-E and CTRL-Y scroll under the prompt without answering', function()
      fill({ 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'x x' })
      command('10')
      feed(':.s/x/Q/gc<CR>')
      screen:expect([[
        d                                                                          |
        e                                                                          |
        f                                                                          |
        g                                                                          |
        h                                                                          |
        i                                                                          |
        {2:x} {10:x}                                                                        |
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      feed('<C-E>')
      screen:expect([[
        e                                                                          |
        f                                                                          |
        g                                                                          |
        h                                                                          |
        i                                                                          |
        {2:x} {10:x}                                                                        |
        {1:~                                                                          }|
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      feed('<C-Y>')
      screen:expect([[
        d                                                                          |
        e                                                                          |
        f                                                                          |
        g                                                                          |
        h                                                                          |
        i                                                                          |
        {2:x} {10:x}                                                                        |
        {6:replace with Q? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)}^      |
      ]])
      -- Neither answered anything: the match is still on offer.
      feed('y')
      eq('Q x', fn.getline(10))
    end)

    it('asks again across a multi-line, multi-match run', function()
      fill({ 'x y x', 'y x y', 'x x x' })
      feed(':%s/x/Q/gc<CR>')
      for _ = 1, 7 do
        feed('y')
      end
      eq({ 'Q y Q', 'y Q y', 'Q Q Q' }, lines())
    end)
  end)

  describe('replacement text', function()
    it('evaluates a \\= expression once per match', function()
      fill({ 'a a a' })
      command([[let g:n = 0]])
      command([[%s/a/\=execute('let g:n += 1')[1:] . g:n/g]])
      eq({ '1 2 3' }, lines())
      eq(3, api.nvim_get_var('n'))
    end)

    it('gives ~ the previous replacement and & the whole match', function()
      fill({ 'foo foo', 'foo' })
      command([[1s/foo/BAR/]])
      -- `~` stands for the previous replacement...
      command([[2s/foo/~/]])
      eq({ 'BAR foo', 'BAR' }, lines())
      -- ... and `&` inside a replacement is the matched text, with `\0`
      -- spelling the same thing.
      fill({ 'abc' })
      command([[s/b/[&]/]])
      eq({ 'a[b]c' }, lines())
      fill({ 'abc' })
      command([[s/b/[\0]/]])
      eq({ 'a[b]c' }, lines())
    end)

    it('splits the line at \\r and inserts a NUL at \\n', function()
      fill({ 'a-b' })
      command([[s/-/\r/]])
      eq({ 'a', 'b' }, lines())

      fill({ 'a-b' })
      command([[s/-/\n/]])
      eq(1, #lines())
      eq('a\0b', lines()[1])

      -- Two breaks in one replacement, and a break on the last line.
      fill({ 'a-b-c' })
      command([[s/-/\r/g]])
      eq({ 'a', 'b', 'c' }, lines())
    end)

    it("obeys 'gdefault', which inverts the meaning of the g flag", function()
      fill({ 'x x x' })
      command('set gdefault')
      command([[s/x/Q/]])
      eq({ 'Q Q Q' }, lines())
      fill({ 'x x x' })
      command([[s/x/Q/g]])
      eq({ 'Q x x' }, lines())
      command('set nogdefault')
      fill({ 'x x x' })
      command([[s/x/Q/]])
      eq({ 'Q x x' }, lines())
    end)

    it('advances past a zero-width match instead of looping on it', function()
      fill({ 'abc' })
      command([[s/x*/-/g]])
      eq({ '-a-b-c' }, lines())

      fill({ 'abc' })
      command([[s/\zs/./g]])
      eq({ '.a.b.c' }, lines())

      -- An anchored empty match hits once per line, not once per column.
      fill({ 'abc', 'de' })
      command([[%s/^/> /]])
      eq({ '> abc', '> de' }, lines())
    end)

    it('copies whole characters around a multibyte match', function()
      fill({ 'áé xx áé' })
      command([[s/xx/ÿ/]])
      eq({ 'áé ÿ áé' }, lines())

      -- The untouched runs on both sides of several matches keep their bytes.
      fill({ '日x本x語' })
      command([[s/x/・/g]])
      eq({ '日・本・語' }, lines())

      -- A multibyte pattern and a longer multibyte replacement, so that the
      -- result line has to grow while it is being built.
      fill({ 'ѫ ѫ ѫ ѫ' })
      command([[s/ѫ/ѫѫѫ/g]])
      eq({ 'ѫѫѫ ѫѫѫ ѫѫѫ ѫѫѫ' }, lines())
    end)

    it('reports the count it made', function()
      fill({ 'x x x', 'x' })
      command([[%s/x/Q/g]])
      eq({ 'Q Q Q', 'Q' }, lines())
      eq('4 substitutions on 2 lines', fn.trim(fn.execute('1messages')))
    end)
  end)
end)
