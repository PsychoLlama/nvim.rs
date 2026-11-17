-- A number the user hands over — a count, a line or column, an option
-- value, a builtin's argument, a line of 'diffexpr' output — may sit at the
-- ends of `int`. Upstream's arithmetic on such a number is signed overflow
-- (undefined; it wraps in practice). Here it is clamped where the number
-- comes in, or measured wide, so a debug build does not trap and a release
-- build does not wrap into nonsense. Each case is its old reproducer; the
-- editor surviving with a sensible answer is the assertion.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq
local command, eval, fn, api = n.command, n.eval, n.fn, n.api
local exec, exec_lua = n.exec, n.exec_lua

local INT_MAX = 2147483647

describe('a number at the ends of int', function()
  before_each(n.clear)

  after_each(function()
    n.assert_alive()
  end)

  describe('as a builtin argument', function()
    it('positions the cursor', function()
      fn.setline(1, { 'abc', 'def' })
      eq(1, fn.append(INT_MAX, 'x'))
      eq(0, fn.cursor(1, INT_MAX, 1))
      eq({ 1, 2 }, api.nvim_win_get_cursor(0))
      eq(0, fn.cursor(1, 2, INT_MAX))
      eq(0, fn.setcursorcharpos(2, -INT_MAX - 1))
      eq({ 2, 0 }, api.nvim_win_get_cursor(0))
      eq({ col = 1, row = 1, endcol = 1, curscol = 1 }, fn.screenpos(1, 1, -INT_MAX - 1))
      eq(-1, fn.byte2line(-INT_MAX - 1))
      eq(0, fn.setcmdpos(-INT_MAX - 1))
    end)

    it('cuts a string', function()
      eq('bc', eval("strpart('abc', 1, v:numbermax)"))
      eq('abc', eval("strpart('abc', v:numbermin)"))
      eq('', eval("strpart('abc', v:numbermin, v:numbermin)"))
      eq('', eval("strpart('abc', -v:numbermax, v:numbermax)"))
      eq('abc', eval("strcharpart('abc', -2147483648)"))
      eq('', eval("strcharpart('abc', v:numbermax, v:numbermax)"))
    end)

    it('repeats a value', function()
      eq('', eval('repeat(0z01, v:numbermax)'))
      t.matches('E342: Out of memory!', t.pcall_err(eval, "repeat('ab', v:numbermax)"))
    end)

    it('repeats a List', function()
      -- Room for every item is asked for up front, so a count no memory
      -- holds is refused rather than grown into until the process dies.
      -- E342 is said once per process, hence a case of its own.
      t.matches('E342: Out of memory!', t.pcall_err(eval, 'repeat([1, 2], v:numbermax)'))
      eq({ 1, 2, 1, 2 }, eval('repeat([1, 2], 2)'))
      eq({}, eval('repeat([1, 2], v:numbermin)'))
    end)

    it('names a match', function()
      fn.setline(1, 'abc')
      eq(INT_MAX, fn.matchadd('Search', 'b', 10, INT_MAX))
      -- A hand-picked id that high leaves the automatic ones where they were.
      eq(1000, fn.matchadd('Search', 'c'))
      fn.matchaddpos('Search', { { 1, 2, INT_MAX } })
      command('redraw')
      eq(INT_MAX, fn.getmatches()[3].pos1[3])
    end)

    it('asks for suggestions', function()
      command('set spell spelllang=en')
      -- A word with hundreds of suggestions rather than thousands: the count
      -- is clamped where it comes in whatever the word, and ranking every
      -- suggestion for 'helo' took 5 s under ASan.
      t.ok(#fn.spellsuggest('zebrra', INT_MAX) > 0)
    end)
  end)

  describe('as a count', function()
    it('moves a window boundary', function()
      command('split | vsplit')
      eq(1, fn.win_move_statusline(1, -INT_MAX - 1))
      eq(1, fn.win_move_separator(1, -INT_MAX - 1))
      eq(1, fn.winheight(1))
      command('resize +2147483647')
      eq(api.nvim_get_option_value('lines', {}) - 4, fn.winheight(0))
      command('vertical resize +2147483647')
    end)

    it('scrolls pages', function()
      fn.setline(1, fn.range(1, 100))
      n.feed('999999999<C-f>')
      eq(100, fn.line('.'))
      n.feed('999999999<C-b>')
      eq(1, fn.line('w0'))
    end)

    it('adds with g CTRL-A', function()
      fn.setline(1, { '1', '1', '1' })
      n.feed('VG999999999g<C-a>')
      eq({ '1000000000', '1999999999', '2147483648' }, fn.getline(1, '$'))
    end)

    it('places a tab page', function()
      command('tabnew')
      local tab = api.nvim_open_tabpage(0, false, { after = INT_MAX })
      eq(3, api.nvim_tabpage_get_number(tab))
    end)
  end)

  describe('as an option value', function()
    it('is a textwidth', function()
      command('set textwidth=2147483647')
      fn.setline(1, 'aaa bbb ccc')
      command('normal! gqq')
      eq({ 'aaa bbb ccc' }, fn.getline(1, '$'))
      -- `:center` takes its width from 'textwidth', counted as 9999.
      fn.setline(1, 'x')
      command('center')
      eq(4999, fn.indent(1))
    end)

    it('is a colorcolumn past textwidth', function()
      command('set textwidth=10 colorcolumn=+2147483647')
      command('redraw')
      command('set colorcolumn=-2147483647')
      command('redraw')
    end)

    it('is a fold level', function()
      fn.setline(1, { 'a', 'b', 'c', 'd' })
      for _, fde in ipairs({
        "'a2147483647'",
        "'s-2147483648'",
        "'<-2147483648'",
        "v:lnum == 2 ? -2147483648 : 'a2147483647'",
        "v:lnum % 2 ? 'a-2147483648' : '<-2147483648'",
      }) do
        command('setlocal foldmethod=expr foldexpr=' .. fde:gsub(' ', '\\ '))
        command('redraw!')
        eq('number', type(fn.foldlevel(3)))
      end
      fn.setline(1, { 'a {{{2147483647 {{{', 'b' })
      command('setlocal foldmethod=marker')
      command('redraw!')
      -- The fold tree stops at its own depth.
      eq(20, fn.foldlevel(1))
    end)

    it('is a terminal colour', function()
      command('hi Normal ctermfg=2147483647 ctermbg=2147483647')
      command('hi Visual ctermfg=fg blend=2147483647')
      t.matches('ctermfg=2147483647 ctermbg=2147483647', n.exec_capture('hi Normal'))
      eq('2147483647', fn.synIDattr(fn.hlID('Normal'), 'fg', 'cterm'))
      api.nvim_set_hl(0, 'Normal', { ctermbg = INT_MAX })
      command('redraw')
    end)
  end)

  it('is an API id', function()
    -- Its low 32 bits are zero, which names no group.
    t.matches('E367: No such group', t.pcall_err(api.nvim_del_augroup_by_id, 2 ^ 32))
    exec_lua(function()
      vim.api.nvim_set_decoration_provider(0, {})
      vim.api.nvim_set_hl(-1, 'Foo', {})
    end)
  end)

  -- 'diffexpr' output is the user's text. The first call is the check run
  -- on "line1" against "line2"; every call after it answers `bad`.
  it('is a line in a diff hunk', function()
    for _, bad in ipairs({
      "['2147483647a2147483647']",
      "['1,2147483647c1,2147483647']",
      "['0,2147483647c0,2147483647']",
      "['2147483646,2147483647c1']",
      "['2147483647d1']",
      "['--- a', '+++ b', '@@ -2147483647,0 +2147483647,0 @@']",
      "['--- a', '+++ b', '@@ -2147483640,100 +1,1 @@']",
      "['--- a', '+++ b', '@@ -1,2147483647 +1,1 @@']",
    }) do
      n.clear()
      exec(([[
        func MyDiff()
          if readfile(v:fname_in) == ['line1']
            call writefile(['1c1', '< line1', '---', '> line2'], v:fname_out)
          else
            call writefile(%s, v:fname_out)
          endif
        endfunc
        set diffexpr=MyDiff()
        call setline(1, ['a', 'b'])
        diffthis
        vnew
        call setline(1, ['a', 'c'])
        diffthis
        redraw!
      ]]):format(bad))
      eq(0, fn.diff_filler(1))
    end
  end)

  it('is a character in a pattern', function()
    -- An over-long UTF-8 form decodes to 0x7fffffff, which is no magic
    -- character.
    eq(0, eval([["a" =~ "x\xfd\xbf\xbf\xbf\xbf\xbf*"]]))
    command('set regexpengine=1')
    eq(0, eval([["a" =~ "x\xfd\xbf\xbf\xbf\xbf\xbf*"]]))
  end)

  -- Before these were bounded, each built megabytes of white space per line
  -- (or a pattern program per repeat) until the editor hung or ran out of
  -- memory.
  describe('as a width that sizes white space', function()
    local function refused(errnr, set)
      t.matches(errnr .. ':', t.pcall_err(set))
    end

    it('is refused past 9999 as an option, from every entry point', function()
      for _, name in ipairs({ 'shiftwidth', 'softtabstop' }) do
        local before = api.nvim_get_option_value(name, {})
        for _, set in ipairs({
          function()
            command('set ' .. name .. '=10000')
          end,
          function()
            command('setlocal ' .. name .. '=' .. INT_MAX)
          end,
          function()
            api.nvim_set_option_value(name, 10000, {})
          end,
          function()
            exec_lua('vim.o[...] = 10000', name)
          end,
          function()
            exec_lua('vim.bo[...] = 10000', name)
          end,
          function()
            command('let &' .. name .. ' = 10000')
          end,
        }) do
          refused('E474', set)
          eq(before, api.nvim_get_option_value(name, {}))
        end
      end
      command('set shiftwidth=9999 softtabstop=9999 softtabstop=-1')
      eq(9999, api.nvim_get_option_value('shiftwidth', {}))

      refused('E474', function()
        command('set cinoptions=>10000')
      end)
      refused('E474', function()
        command('set cinoptions=>1.99999s')
      end)
      command('set cinoptions=>9999,n-9999,(2s,.5s')
      refused('E487', function()
        command('set wrapmargin=-1')
      end)
    end)

    it('is refused past 9999 in a modeline', function()
      local file = t.tmpname()
      t.write_file(file, 'x\n/* vim: set sw=10000 cino=>99999: */\n')
      finally(function()
        os.remove(file)
      end)
      command('set modeline')
      command('silent! edit ' .. file)
      eq(8, api.nvim_get_option_value('shiftwidth', { buf = 0 }))
      eq('', api.nvim_get_option_value('cinoptions', { buf = 0 }):match('99999') or '')
      t.matches('E474:', eval('v:errmsg'))
    end)

    it('is refused past 9999 by :left, :center and :right', function()
      fn.setline(1, 'x')
      for _, cmd in ipairs({ 'left 10000', 'center ' .. INT_MAX, 'right 4294967297' }) do
        refused('E475', function()
          command(cmd)
        end)
        eq({ 'x' }, fn.getline(1, '$'))
      end
      command('left 9999')
      eq(9999, fn.indent(1))
    end)
  end)
end)
