local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq
local clear = n.clear
local command = n.command
local eval = n.eval
local api = n.api
local exec = n.exec
local exec_capture = n.exec_capture
local expect_exit = n.expect_exit
local source = n.source
local testprg = n.testprg

before_each(clear)

describe(':let', function()
  it('correctly lists variables with curly-braces', function()
    api.nvim_set_var('v', { 0 })
    eq('v                     [0]', exec_capture('let {"v"}'))
  end)

  it('correctly lists variables with subscript', function()
    api.nvim_set_var('v', { 0 })
    eq('v[0]                  #0', exec_capture('let v[0]'))
    eq('g:["v"][0]            #0', exec_capture('let g:["v"][0]'))
    eq('{"g:"}["v"][0]        #0', exec_capture('let {"g:"}["v"][0]'))
  end)

  it(':unlet self-referencing node in a List graph #6070', function()
    -- :unlet-ing a self-referencing List must not allow GC on indirectly
    -- referenced in-scope Lists. Before #6070 this caused use-after-free.
    expect_exit(
      1000,
      source,
      [=[
      let [l1, l2] = [[], []]
      echo 'l1:' . id(l1)
      echo 'l2:' . id(l2)
      echo ''
      let [l3, l4] = [[], []]
      call add(l4, l4)
      call add(l4, l3)
      call add(l3, 1)
      call add(l2, l2)
      call add(l2, l1)
      call add(l1, 1)
      unlet l2
      unlet l4
      call garbagecollect(1)
      call feedkeys(":\e:echo l1 l3\n:echo 42\n:cq\n", "t")
    ]=]
    )
  end)

  it('multibyte env var #8398 #9267', function()
    command("let $NVIM_TEST_LET = 'AìaB'")
    eq('AìaB', eval('$NVIM_TEST_LET'))
    command("let $NVIM_TEST_LET = 'AaあB'")
    eq('AaあB', eval('$NVIM_TEST_LET'))
    local mbyte = [[\p* .ม .ม .ม .ม่ .ม่ .ม่ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹֻ ֹֻ ֹֻ
                    .ֹֻ .ֹֻ .ֹֻ ֹֻ ֹֻ ֹֻ .ֹֻ .ֹֻ .ֹֻ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹֻ ֹֻ
                    .ֹֻ .ֹֻ .ֹֻ a a a ca ca ca à à à]]
    command("let $NVIM_TEST_LET = '" .. mbyte .. "'")
    eq(mbyte, eval('$NVIM_TEST_LET'))
  end)

  it('multibyte env var to child process #8398 #9267', function()
    local cmd_get_child_env = ("let g:env_from_child = system(['%s', 'NVIM_TEST_LET'])"):format(
      testprg('printenv-test')
    )
    command("let $NVIM_TEST_LET = 'AìaB'")
    command(cmd_get_child_env)
    eq(eval('$NVIM_TEST_LET'), eval('g:env_from_child'))

    command("let $NVIM_TEST_LET = 'AaあB'")
    command(cmd_get_child_env)
    eq(eval('$NVIM_TEST_LET'), eval('g:env_from_child'))

    local mbyte = [[\p* .ม .ม .ม .ม่ .ม่ .ม่ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹֻ ֹֻ ֹֻ
                    .ֹֻ .ֹֻ .ֹֻ ֹֻ ֹֻ ֹֻ .ֹֻ .ֹֻ .ֹֻ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹ ֹ ֹ .ֹ .ֹ .ֹ ֹֻ ֹֻ
                    .ֹֻ .ֹֻ .ֹֻ a a a ca ca ca à à à]]
    command("let $NVIM_TEST_LET = '" .. mbyte .. "'")
    command(cmd_get_child_env)
    eq(eval('$NVIM_TEST_LET'), eval('g:env_from_child'))
  end)

  it('release of list assigned to l: variable does not trigger assertion #12387, #12430', function()
    source([[
      func! s:f()
        let l:x = [1]
        let g:x = l:
      endfunc
      for _ in range(2)
        call s:f()
      endfor
      call garbagecollect()
      call feedkeys('i', 't')
    ]])
    eq(1, eval('1'))
  end)

  it('can apply operator to boolean option', function()
    eq(true, api.nvim_get_option_value('equalalways', {}))
    command('let &equalalways -= 1')
    eq(false, api.nvim_get_option_value('equalalways', {}))
    command('let &equalalways += 1')
    eq(true, api.nvim_get_option_value('equalalways', {}))
    command('let &equalalways *= 1')
    eq(true, api.nvim_get_option_value('equalalways', {}))
    command('let &equalalways /= 1')
    eq(true, api.nvim_get_option_value('equalalways', {}))
    command('let &equalalways %= 1')
    eq(false, api.nvim_get_option_value('equalalways', {}))
  end)

  it('concatenates a range with itself', function()
    -- Each item is both operands; growing the left one in place must not
    -- read the right one from where it used to be.
    for _, len in ipairs({ 10, 100, 1000, 100000 }) do
      command(('let l = [repeat("a", %d), "b"] | let l[0:1] .= l'):format(len))
      eq({ 2 * len, 'bb' }, eval('[len(l[0]), l[1]]'))
    end
  end)

  describe('when an index expression changes what it indexes', function()
    -- The target is found again after the index runs, so these answer an
    -- error or write where the C would have read freed memory.
    it('grows the outer list past its capacity', function()
      command('let l = [[1], [2]] | let l[0][len(extend(l, range(100))) * 0] = 7')
      eq({ { 7 }, { 2 }, 0 }, eval('l[0:2]'))
    end)

    it('removes the dictionary item being indexed', function()
      command("let d = {'a': {'x': 1}}")
      eq(
        'Vim(let):E716: Key not present in Dictionary: "a"',
        t.pcall_err(command, "let d.a[remove(d, 'a') is 0 ? 'x' : 'x'] = 5")
      )
      eq({}, eval('d'))
    end)

    it('unlets the variable being indexed', function()
      command('let d = {}')
      eq(
        'Vim(let):E121: Undefined variable: d',
        t.pcall_err(command, "let d[execute('unlet d')] = 1")
      )
      eq(0, eval('exists("d")'))
      command("let e = {'a': {}}")
      eq(
        'Vim(let):E716: Key not present in Dictionary: "a"',
        t.pcall_err(command, "let e.a[execute('unlet e.a')] = 1")
      )
    end)

    it('assigns loop targets while the loop edits its list', function()
      command('let l = [0, 0, 0] | let d = {}')
      command('for d[len(d) + 0 * len(add(l, 9))] in l | if len(d) > 4 | break | endif | endfor')
      eq({ ['0'] = 0, ['1'] = 0, ['2'] = 0, ['3'] = 9, ['4'] = 9 }, eval('d'))
      command('let l = [[1, 2], [3, 4]] | let out = []')
      command('for [a, b] in l | call add(out, a . b) | call remove(l, -1) | endfor')
      eq({ '12' }, eval('out'))
    end)
  end)

  it('names a locked curly-brace target by its expansion', function()
    command("let x = [1] | lockvar x | let y = {'a': 1} | lockvar y")
    eq('Vim(unlet):E741: Value is locked: x', t.pcall_err(command, "unlet {'x'}[0]"))
    eq('Vim(unlet):E741: Value is locked: y', t.pcall_err(command, "unlet {'y'}.a"))
  end)
end)

describe(':let and :const', function()
  it('have the same output when called without arguments', function()
    eq(exec_capture('let'), exec_capture('const'))
  end)

  it('can be used in sandbox', function()
    exec([[
      func Func()
        let l:foo = 'foo'
        const l:bar = 'bar'
      endfunc
      sandbox call Func()
    ]])
  end)
end)
