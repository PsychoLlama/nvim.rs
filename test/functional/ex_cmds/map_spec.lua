local t = require('test.testutil')
local n = require('test.functional.testnvim')()
local Screen = require('test.functional.ui.screen')

local eq = t.eq
local exec = n.exec
local exec_capture = n.exec_capture
local feed = n.feed
local api = n.api
local clear = n.clear
local command = n.command
local expect = n.expect
local insert = n.insert
local eval = n.eval

describe(':*map', function()
  before_each(clear)

  it('are not affected by &isident', function()
    api.nvim_set_var('counter', 0)
    command('nnoremap <C-x> :let counter+=1<CR>')
    api.nvim_set_option_value('isident', ('%u'):format(('>'):byte()), {})
    command('nnoremap <C-y> :let counter+=1<CR>')
    -- &isident used to disable keycode parsing here as well
    feed('\24\25<C-x><C-y>')
    eq(4, api.nvim_get_var('counter'))
  end)

  it(':imap <M-">', function()
    command('imap <M-"> foo')
    feed('i-<M-">-')
    expect('-foo-')
  end)

  it('shows <Nop> as mapping rhs', function()
    command('nmap asdf <Nop>')
    eq(
      [[

n  asdf          <Nop>]],
      exec_capture('nmap asdf')
    )
  end)

  it('mappings with description can be filtered', function()
    api.nvim_set_keymap('n', 'asdf1', 'qwert', { desc = 'do the one thing' })
    api.nvim_set_keymap('n', 'asdf2', 'qwert', { desc = 'doesnot really do anything' })
    api.nvim_set_keymap('n', 'asdf3', 'qwert', { desc = 'do the other thing' })
    eq(
      [[

n  asdf3         qwert
                 do the other thing
n  asdf1         qwert
                 do the one thing]],
      exec_capture('filter the nmap')
    )
  end)

  it('<Plug> mappings ignore nore', function()
    command('let x = 0')
    eq(0, api.nvim_eval('x'))
    command [[
      nnoremap <Plug>(Increase_x) <cmd>let x+=1<cr>
      nmap increase_x_remap <Plug>(Increase_x)
      nnoremap increase_x_noremap <Plug>(Increase_x)
    ]]
    feed('increase_x_remap')
    eq(1, api.nvim_eval('x'))
    feed('increase_x_noremap')
    eq(2, api.nvim_eval('x'))
  end)

  it("Doesn't auto ignore nore for keys before or after <Plug> mapping", function()
    command('let x = 0')
    eq(0, api.nvim_eval('x'))
    command [[
      nnoremap x <nop>
      nnoremap <Plug>(Increase_x) <cmd>let x+=1<cr>
      nmap increase_x_remap x<Plug>(Increase_x)x
      nnoremap increase_x_noremap x<Plug>(Increase_x)x
    ]]
    insert('Some text')
    eq('Some text', eval("getline('.')"))

    feed('increase_x_remap')
    eq(1, api.nvim_eval('x'))
    eq('Some text', eval("getline('.')"))
    feed('increase_x_noremap')
    eq(2, api.nvim_eval('x'))
    eq('Some te', eval("getline('.')"))
  end)

  it(':unmap with rhs works when lhs is in another bucket #21530', function()
    command('map F <Plug>Foo')
    command('unmap <Plug>Foo')
    eq('\nNo mapping found', exec_capture('map F'))
  end)
end)

describe('Screen', function()
  local screen
  before_each(function()
    clear()
    screen = Screen.new(20, 5)
  end)

  it('cursor is restored after :map <expr> which calls input()', function()
    command('map <expr> x input("> ")')
    screen:expect([[
      ^                    |
      {1:~                   }|*3
                          |
    ]])
    feed('x')
    screen:expect([[
                          |
      {1:~                   }|*3
      > ^                  |
    ]])
    feed('\n')
    screen:expect([[
      ^                    |
      {1:~                   }|*3
      >                   |
    ]])
  end)

  it('cursor is restored after :imap <expr> which calls input()', function()
    command('imap <expr> x input("> ")')
    feed('i')
    screen:expect([[
      ^                    |
      {1:~                   }|*3
      {5:-- INSERT --}        |
    ]])
    feed('x')
    screen:expect([[
                          |
      {1:~                   }|*3
      > ^                  |
    ]])
    feed('\n')
    screen:expect([[
      ^                    |
      {1:~                   }|*3
      {5:-- INSERT --}        |
    ]])
  end)

  it('cursor position does not move after empty-string :cmap <expr> #19046', function()
    command([[cnoremap <expr> <F2> '']])
    feed(':<F2>')
    screen:expect([[
                          |
      {1:~                   }|*3
      :^                   |
    ]])
  end)

  -- oldtest: Test_expr_map_restore_cursor()
  it('cursor is restored after :map <expr> which redraws statusline vim-patch:8.1.2336', function()
    exec([[
      call setline(1, ['one', 'two', 'three'])
      2
      set ls=2
      hi! link StatusLine ErrorMsg
      noremap <expr> <C-B> Func()
      func Func()
          let g:on = !get(g:, 'on', 0)
          redraws
          return ''
      endfunc
      func Status()
          return get(g:, 'on', 0) ? '[on]' : ''
      endfunc
      set stl=%{Status()}
    ]])
    feed('<C-B>')
    screen:expect([[
      one                 |
      ^two                 |
      three               |
      {9:[on]                }|
                          |
    ]])
  end)

  it('error in :nmap <expr> does not mess up display vim-patch:4.2.4338', function()
    screen:try_resize(40, 5)
    command('nmap <expr> <F2> execute("throw 42")')
    feed('<F2>')
    screen:expect([[
                                              |
      {3:                                        }|
      {9:Error in :}                              |
      {9:E605: Exception not caught: 42}          |
      {6:Press ENTER or type command to continue}^ |
    ]])
    feed('<CR>')
    screen:expect([[
      ^                                        |
      {1:~                                       }|*3
                                              |
    ]])
  end)

  it('error in :cmap <expr> handled correctly vim-patch:4.2.4338', function()
    screen:try_resize(40, 5)
    command('cmap <expr> <F2> execute("throw 42")')
    feed(':echo "foo')
    screen:expect([[
                                              |
      {1:~                                       }|*3
      :echo "foo^                              |
    ]])
    feed('<F2>')
    screen:expect([[
      {3:                                        }|
      :echo "foo                              |
      {9:Error in :}                              |
      {9:E605: Exception not caught: 42}          |
      :echo "foo^                              |
    ]])
    feed('"')
    screen:expect([[
      {3:                                        }|
      :echo "foo                              |
      {9:Error in :}                              |
      {9:E605: Exception not caught: 42}          |
      :echo "foo"^                             |
    ]])
    feed('\n')
    screen:expect([[
      :echo "foo                              |
      {9:Error in :}                              |
      {9:E605: Exception not caught: 42}          |
      foo                                     |
      {6:Press ENTER or type command to continue}^ |
    ]])
  end)

  -- oldtest: Test_map_listing()
  it('listing mappings clears command line vim-patch:8.2.4401', function()
    screen:try_resize(40, 5)
    command('nmap a b')
    feed(':                      nmap a<CR>')
    screen:expect([[
      ^                                        |
      {1:~                                       }|*3
      n  a             b                      |
    ]])
  end)
end)

describe(':map <expr> that edits the mapping table while it runs', function()
  before_each(clear)

  -- In every mode, and for abbreviations, :lmap, Lua callbacks,
  -- buffer-local entries and a simplified <C-H> pair: the expression unmaps
  -- itself, clears its whole table, or redefines its own lhs, and the keys
  -- it answers still go where they should. Expected output is upstream
  -- 0.12.5's.
  it('unmaps, clears or redefines itself in every mode', function()
    exec([==[
let g:log = []
func! Act(cmd, ret)
  call add(g:log, strtrans(a:cmd))
  exe a:cmd
  return a:ret
endfunc
func! Run(name, setup, keys)
  silent! mapclear | silent! mapclear! | silent! abclear | silent! lmapclear | silent! tmapclear
  let g:log = []
  unlet! g:c
  %delete _
  call setline(1, ['abcdef', 'ghijkl'])
  call cursor(1, 1)
  for s in a:setup | exe s | endfor
  let err = ''
  try
    if a:name =~ '^lmap'
      call feedkeys(a:keys, 'xt')
    else
      exe "normal " . a:keys
    endif
  catch
    let err = v:exception
  endtry
  call add(g:out, a:name . ': ' . string(getline(1, '$')) . ' ' . string(g:log) . ' ' . err . ' maps=' . string(map(maplist(), 'v:val.mode . v:val.lhs . "=" . get(v:val, "rhs", "")')) . ' c=' . get(g:, 'c', '-') . ' ab=' . string(map(maplist(1), 'v:val.mode . v:val.lhs . "=" . get(v:val, "rhs", "")')))
endfunc
let g:out = []
for [mode, keys] in [['n', ',q'], ['x', "v,q"], ['o', "d,q"], ['i', "i,q\<Esc>"], ['c', ":let g:c='" . ',q' . "'\<CR>"]]
  call Run(mode . '-unmap', [mode . "noremap <expr> ,q Act('" . mode . "unmap ,q', 'x')"], keys)
  call Run(mode . '-clear', [mode . "noremap <expr> ,q Act('" . mode . "mapclear', 'x')", mode . "noremap ,z zz"], keys)
  call Run(mode . '-redef', [mode . "noremap <expr> ,q Act('" . mode . "noremap ,q y', 'x')"], keys . keys)
  call Run(mode . '-remap', [mode . "map <expr> ,q Act('" . mode . "unmap ,q', ',r')", mode . "noremap ,r X"], keys)
  call Run(mode . '-simp', [mode . "noremap <expr> <C-H> Act('" . mode . "unmap <C-H>', 'x')"], substitute(keys, ',q', "\<C-H>", ''))
  call Run(mode . '-simp2', [mode . "noremap <expr> <C-H> Act('" . mode . "unmap <lt>C-H>', 'x')"], substitute(keys, ',q', "\<C-H>", ''))
  call Run(mode . '-simp3', [mode . "noremap <expr> <C-H> Act('" . mode . "mapclear', 'x')"], substitute(keys, ',q', "\<C-H>", ''))
  call Run(mode . '-buf', [mode . "noremap <buffer> <expr> ,q Act('" . mode . "mapclear <buffer>', 'x')", mode . "noremap ,q G"], keys . keys)
endfor
call Run('s-unmap', ["snoremap <expr> ,q Act('sunmap ,q', 'x')"], "gh,q")
call Run('ab-unab', ["inoreabbrev <expr> qq Act('iunabbrev qq', 'ZZ')"], "Aqq \<Esc>")
call Run('ab-clear', ["inoreabbrev <expr> qq Act('abclear', 'ZZ')", "iabbrev ww WW"], "Aqq ww \<Esc>")
call Run('ab-redef', ["inoreabbrev <expr> qq Act('inoreabbrev qq YY', 'ZZ')"], "Aqq qq \<Esc>")
call Run('ab-cmd', ["cnoreabbrev <expr> qq Act('cunabbrev qq', 'ZZ')"], ":qq\<C-R>=''\<CR>\<Esc>")
call Run('ab-ctrlrsb', ["inoreabbrev <expr> qq Act('iunabbrev qq', 'ZZ')"], "Aqq\<C-]>\<Esc>")
call Run('lmap', ["set iminsert=1", "lnoremap <expr> q Act('lunmap q', 'Q')"], "Aqq\<Esc>")
call Run('lmap-clear', ["set iminsert=1", "lnoremap <expr> q Act('lmapclear', 'Q')"], "Aqq\<Esc>")
set iminsert=0
call Run('lua', ["lua vim.keymap.set('n', ',q', function() vim.keymap.del('n', ',q') return 'x' end, {expr=true})"], ',q')
call Run('lua-clear', ["lua vim.keymap.set('n', ',q', function() vim.cmd('mapclear') return 'x' end, {expr=true})"], ',q,q')
    ]==])
    eq({
      [=[n-unmap: ['bcdef', 'ghijkl'] ['nunmap ,q']  maps=[] c=- ab=[]]=],
      [=[n-clear: ['bcdef', 'ghijkl'] ['nmapclear']  maps=[] c=- ab=[]]=],
      [=[n-redef: ['bcdef', 'ghijkl'] ['nnoremap ,q y']  maps=['n,q=y'] c=- ab=[]]=],
      [=[n-remap: ['abcdef', 'ghijkl'] ['nunmap ,q']  maps=['n,r=X'] c=- ab=[]]=],
      [=[n-simp: ['bcdef', 'ghijkl'] ['nunmap ^H']  maps=['n<C-H>=Act(''nunmap <C-H>'', ''x'')'] c=- ab=[]]=],
      [=[n-simp2: ['bcdef', 'ghijkl'] ['nunmap <C-H>']  maps=[] c=- ab=[]]=],
      [=[n-simp3: ['bcdef', 'ghijkl'] ['nmapclear']  maps=[] c=- ab=[]]=],
      [=[n-buf: ['bcdef', 'ghijkl'] ['nmapclear <buffer>']  maps=['n,q=G'] c=- ab=[]]=],
      [=[x-unmap: ['bcdef', 'ghijkl'] ['xunmap ,q']  maps=[] c=- ab=[]]=],
      [=[x-clear: ['bcdef', 'ghijkl'] ['xmapclear']  maps=[] c=- ab=[]]=],
      [=[x-redef: ['bcdef', 'ghijkl'] ['xnoremap ,q y']  maps=['x,q=y'] c=- ab=[]]=],
      [=[x-remap: ['ghijkl'] ['xunmap ,q']  maps=['x,r=X'] c=- ab=[]]=],
      [=[x-simp: ['bcdef', 'ghijkl'] ['xunmap ^H']  maps=['x<C-H>=Act(''xunmap <C-H>'', ''x'')'] c=- ab=[]]=],
      [=[x-simp2: ['bcdef', 'ghijkl'] ['xunmap <C-H>']  maps=[] c=- ab=[]]=],
      [=[x-simp3: ['bcdef', 'ghijkl'] ['xmapclear']  maps=[] c=- ab=[]]=],
      [=[x-buf: ['bcdef', 'ghijkl'] ['xmapclear <buffer>']  maps=['x,q=G'] c=- ab=[]]=],
      [=[o-unmap: ['abcdef', 'ghijkl'] ['ounmap ,q']  maps=[] c=- ab=[]]=],
      [=[o-clear: ['abcdef', 'ghijkl'] ['omapclear']  maps=[] c=- ab=[]]=],
      [=[o-redef: ['abcdef', 'ghijkl'] ['onoremap ,q y']  maps=['o,q=y'] c=- ab=[]]=],
      [=[o-remap: ['abcdef', 'ghijkl'] ['ounmap ,q']  maps=['o,r=X'] c=- ab=[]]=],
      [=[o-simp: ['abcdef', 'ghijkl'] ['ounmap ^H']  maps=['o<C-H>=Act(''ounmap <C-H>'', ''x'')'] c=- ab=[]]=],
      [=[o-simp2: ['abcdef', 'ghijkl'] ['ounmap <C-H>']  maps=[] c=- ab=[]]=],
      [=[o-simp3: ['abcdef', 'ghijkl'] ['omapclear']  maps=[] c=- ab=[]]=],
      [=[o-buf: ['abcdef', 'ghijkl'] ['omapclear <buffer>']  maps=['o,q=G'] c=- ab=[]]=],
      [=[i-unmap: ['xabcdef', 'ghijkl'] ['iunmap ,q']  maps=[] c=- ab=[]]=],
      [=[i-clear: ['xabcdef', 'ghijkl'] ['imapclear']  maps=[] c=- ab=[]]=],
      [=[i-redef: ['yxabcdef', 'ghijkl'] ['inoremap ,q y']  maps=['i,q=y'] c=- ab=[]]=],
      [=[i-remap: ['Xabcdef', 'ghijkl'] ['iunmap ,q']  maps=['i,r=X'] c=- ab=[]]=],
      [=[i-simp: ['xabcdef', 'ghijkl'] ['iunmap ^H']  maps=['i<C-H>=Act(''iunmap <C-H>'', ''x'')'] c=- ab=[]]=],
      [=[i-simp2: ['xabcdef', 'ghijkl'] ['iunmap <C-H>']  maps=[] c=- ab=[]]=],
      [=[i-simp3: ['xabcdef', 'ghijkl'] ['imapclear']  maps=[] c=- ab=[]]=],
      [=[i-buf: ['Gxabcdef', 'ghijkl'] ['imapclear <buffer>']  maps=['i,q=G'] c=- ab=[]]=],
      [=[c-unmap: ['abcdef', 'ghijkl'] ['cunmap ,q']  maps=[] c=x ab=[]]=],
      [=[c-clear: ['abcdef', 'ghijkl'] ['cmapclear']  maps=[] c=x ab=[]]=],
      [=[c-redef: ['abcdef', 'ghijkl'] ['cnoremap ,q y']  maps=['c,q=y'] c=y ab=[]]=],
      [=[c-remap: ['abcdef', 'ghijkl'] ['cunmap ,q']  maps=['c,r=X'] c=X ab=[]]=],
      [=[c-simp: ['abcdef', 'ghijkl'] ['cunmap ^H']  maps=['c<C-H>=Act(''cunmap <C-H>'', ''x'')'] c=x ab=[]]=],
      [=[c-simp2: ['abcdef', 'ghijkl'] ['cunmap <C-H>']  maps=[] c=x ab=[]]=],
      [=[c-simp3: ['abcdef', 'ghijkl'] ['cmapclear']  maps=[] c=x ab=[]]=],
      [=[c-buf: ['abcdef', 'ghijkl'] ['cmapclear <buffer>']  maps=['c,q=G'] c=G ab=[]]=],
      [=[s-unmap: ['xbcdef', 'ghijkl'] ['sunmap ,q']  maps=[] c=- ab=[]]=],
      [=[ab-unab: ['abcdefZZ ', 'ghijkl'] ['iunabbrev qq']  maps=[] c=- ab=[]]=],
      [=[ab-clear: ['abcdefZZ ww ', 'ghijkl'] ['abclear']  maps=[] c=- ab=[]]=],
      [=[ab-redef: ['abcdefZZ YY ', 'ghijkl'] ['inoreabbrev qq YY']  maps=[] c=- ab=['iqq=YY']]=],
      [=[ab-cmd: ['abcdef', 'ghijkl'] ['cunabbrev qq'] Vim:E492: Not an editor command: ZZ maps=[] c=- ab=[]]=],
      [=[ab-ctrlrsb: ['abcdefZZ', 'ghijkl'] ['iunabbrev qq']  maps=[] c=- ab=[]]=],
      [=[lmap: ['abcdefQq', 'ghijkl'] ['lunmap q']  maps=[] c=- ab=[]]=],
      [=[lmap-clear: ['abcdefQq', 'ghijkl'] ['lmapclear']  maps=[] c=- ab=[]]=],
      [=[lua: ['bcdef', 'ghijkl'] []  maps=[] c=- ab=[]]=],
      [=[lua-clear: ['bcdef', 'ghijkl'] []  maps=[] c=- ab=[]]=],
    }, eval('g:out'))
  end)
end)
