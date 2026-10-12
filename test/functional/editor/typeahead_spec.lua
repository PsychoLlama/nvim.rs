-- User code that reshapes the typeahead while a key is being resolved:
-- `<expr>` mappings that feedkeys(), inputsave()/inputrestore() and `<Cmd>`
-- `:normal`, `:normal` nested in `:normal` with keys left over, a timer that
-- feeds keys while a --More-- prompt waits, and a `-s` script that sources
-- one that feeds keys. Expectations are upstream 0.12.5's, which the
-- pointer-held typeahead this replaced also produced.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()
local Screen = require('test.functional.ui.screen')

local eq = t.eq
local eval = n.eval
local clear = n.clear
local command = n.command
local exec = n.exec
local feed = n.feed
local fn = n.fn

describe('typeahead reshaped by user code', function()
  before_each(clear)

  it('by an <expr> mapping that feeds keys around inputsave()', function()
    exec([[
      let g:log = []
      function! Fx() abort
        call add(g:log, 'fx')
        call feedkeys('Aq', 'n')
        call inputsave()
        call feedkeys('zz', 'n')
        call add(g:log, getchar(0))
        call inputrestore()
        return 'ib'
      endfunction
      nnoremap <expr> x Fx()
      nnoremap <expr> xy Fx()
      call setline(1, 'base')
      call feedkeys("x\<Esc>", 'xt')
    ]])
    eq({ 'fx', 122 }, eval('g:log'))
    eq({ 'bbaseq' }, fn.getline(1, '$'))
  end)

  it('by an <expr> mapping that inserts in front of the keys it returns', function()
    exec([[
      function! Fy() abort
        call feedkeys('w', 'ni')
        return 'I1'
      endfunction
      nnoremap <expr> W Fy()
      call setline(1, 'base')
      call feedkeys("W\<Esc>", 'xt')
    ]])
    eq({ '1wbase' }, fn.getline(1, '$'))
  end)

  it('by a <Cmd> mapping that runs :normal mid-resolution', function()
    exec([[
      call setline(1, 'base')
      nnoremap Q <Cmd>exe "normal! A(" . getline(1) . ")"<CR>
      call feedkeys("QQ", 'xt')
    ]])
    eq({ 'base(base)(base(base))' }, fn.getline(1, '$'))
  end)

  it('by :normal nested in :normal with typeahead left over', function()
    exec([[
      call setline(1, 'base')
      call feedkeys("Ax\<Esc>", 'n')
      exe "normal! :normal! Iy\<CR>Az\<Esc>"
    ]])
    -- The nested `:normal`s run on keys of their own; the fed `Ax<Esc>`
    -- waits for them and runs once the request returns to the main loop.
    eq({ 'ybasezx' }, fn.getline(1, '$'))
  end)

  it('by a timer that feeds keys while --More-- waits', function()
    local screen = Screen.new(40, 6)
    command('set more | let g:fed = 0')
    feed(
      ':call timer_start(100, {-> execute("let g:fed = 1") + feedkeys("ix\\<lt>Esc>", "n")})'
        .. ' | echo join(range(1, 20), "\\n")<CR>'
    )
    screen:expect({ any = '%-%- More %-%-' })
    -- Only fast events are answered at the prompt: the timer waits for it.
    vim.uv.sleep(300)
    screen:expect({ any = '%-%- More %-%-', unchanged = true })
    feed('q')
    -- Then it runs, and the keys it fed are read in Normal mode.
    t.retry(nil, 2000, function()
      eq({ 1, { 'x' } }, { eval('g:fed'), fn.getline(1, '$') })
    end)
    eq('n', fn.mode())
  end)
end)

describe('-s script', function()
  it('that :sources a script feeding keys', function()
    local dir = t.tmpname(false)
    t.mkdir(dir)
    local fed = dir .. '/fed.vim'
    t.write_file(fed, 'call feedkeys("Afed\\<Esc>", "n")\n')
    local inner = dir .. '/inner.keys'
    t.write_file(inner, 'Atwo\027:source ' .. fed .. '\r')
    local outer = dir .. '/outer.keys'
    t.write_file(outer, 'ihello\027:source! ' .. inner .. '\r:source ' .. fed .. '\r')
    clear({ args = { '-s', outer } })
    t.retry(nil, 5000, function()
      eq({ 'hellotwofedfed' }, fn.getline(1, '$'))
    end)
    n.rmdir(dir)
  end)
end)
