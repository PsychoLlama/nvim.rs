-- The Lua executor's edges, one case per path through `lua/executor/`: the
-- `:lua` family, `vim.schedule`, `print`, Lua user commands, `vim.fn` and
-- `vim.call` across every value kind, and the exact text of each error the
-- executor builds. `commands_spec`, `vim_spec` and `overrides_spec` cover the
-- features; these pin the messages and the argument tables, so a rewrite of
-- the executor's buffers and error paths is checked against the same bytes.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear = n.clear
local eq = t.eq
local matches = t.matches
local api = n.api
local command = n.command
local exec_lua = n.exec_lua
local exec_capture = n.exec_capture
local eval = n.eval
local fn = n.fn
local pcall_err = t.pcall_err
local remove_trace = t.remove_trace
local write_file = t.write_file
local assert_alive = n.assert_alive
local NIL = vim.NIL

before_each(clear)

describe(':lua', function()
  it('prints every argument of print(), nil and tables included', function()
    eq('1 nil a true', exec_capture('lua print(1, nil, "a", true)'))
    matches('^table: 0x%x+$', exec_capture('lua print({})'))
    eq('', exec_capture('lua print()'))
    eq('a\nb', exec_capture('lua print("a\\nb")'))
    -- A NUL reaches the message area and is shown as ^@.
    eq('x^@y', exec_capture('lua print("x\\0y")'))
    eq('3.5 -0', exec_capture('lua print(3.5, -0)'))
  end)

  it('prints a line longer than the old IOSIZE buffer whole', function()
    local long = ('x'):rep(3000)
    eq(long, exec_capture('lua print(("x"):rep(3000))'))
    eq(long .. ' ' .. long, exec_capture('lua print(("x"):rep(3000), ("x"):rep(3000))'))
  end)

  it('reports a tostring that does not answer a string', function()
    eq(
      'Vim(lua):E5108: Lua: E5114: Converting print argument #2: '
        .. '<Unknown error: lua_tolstring returned NULL for tostring result>',
      remove_trace(
        pcall_err(
          command,
          'lua print(1, setmetatable({}, {__tostring = function() return {} end}))'
        )
      )
    )
  end)

  it('evaluates =expr and prints each result', function()
    eq('1\nnil\n{\n  a = 1\n}', exec_capture('lua =1, nil, {a=1}'))
    eq('nil', exec_capture('lua =nil'))
    eq('"s"', exec_capture('lua ="s"'))
    eq(
      [[Vim(lua):E5107: Lua: [string ":lua"]:0: unexpected symbol near ')']],
      pcall_err(command, 'lua =')
    )
  end)

  it('runs a chunk longer than IOSIZE', function()
    eq('701', exec_capture('lua print(' .. ('1+'):rep(700) .. '1)'))
  end)

  it('builds each error message', function()
    eq('Vim(lua):E471: Argument required', pcall_err(command, 'lua'))
    eq(
      [[Vim(lua):E5107: Lua: [string ":lua"]:0: unexpected symbol near '<eof>']],
      pcall_err(command, 'lua ((')
    )
    eq(
      [[Vim(lua):E5108: Lua: [string ":lua"]:0: boom]],
      remove_trace(pcall_err(command, 'lua error("boom")'))
    )
    eq([=[Vim(lua):E5108: Lua: [NULL]]=], pcall_err(command, 'lua error(nil)'))
    eq([=[Vim(lua):E5108: Lua: [NULL]]=], pcall_err(command, 'lua error({})'))
    -- The message is cut at its NUL.
    eq(
      [[Vim(lua):E5108: Lua: [string ":lua"]:0: a]],
      remove_trace(pcall_err(command, 'lua error("a\\0b")'))
    )
    eq(
      'Vim(lua):E5108: Lua: [string ":lua"]:0: line one\nline two',
      remove_trace(pcall_err(command, 'lua error("line one\\nline two")'))
    )
  end)
end)

describe(':luado', function()
  before_each(function()
    api.nvim_buf_set_lines(0, 0, -1, false, { 'one', 'two', 'three' })
  end)

  it('replaces each line in the range with the returned string', function()
    command('2,3luado return linenr .. ":" .. line:upper()')
    eq({ 'one', '2:TWO', '3:THREE' }, api.nvim_buf_get_lines(0, 0, -1, false))
  end)

  it('keeps the line for a nil or non-string result', function()
    command('luado if linenr == 2 then return nil end return #line')
    eq({ '3', 'two', '5' }, api.nvim_buf_get_lines(0, 0, -1, false))
  end)

  it('builds its error messages', function()
    eq('Vim(luado):E471: Argument required: luado', pcall_err(command, 'luado'))
    eq(
      [[Vim(luado):E5109: Lua: [string ":luado"]:0: unexpected symbol near ')']],
      pcall_err(command, 'luado )')
    )
    eq(
      [[Vim(luado):E5111: Lua: [string ":luado"]:0: line 1]],
      remove_trace(pcall_err(command, 'luado error("line " .. linenr)'))
    )
    eq({ 'one', 'two', 'three' }, api.nvim_buf_get_lines(0, 0, -1, false))
  end)
end)

describe(':luafile', function()
  local fname = 'Xexecutor_seam.lua'
  after_each(function()
    os.remove(fname)
  end)

  it('runs a file and sees its arguments as ...', function()
    write_file(fname, 'vim.g.got = {...}\nprint("ran")\n')
    eq('ran', exec_capture('luafile ' .. fname))
    eq({}, api.nvim_get_var('got'))
  end)

  it('builds its error messages', function()
    eq(
      'Vim(luafile):E5112: Lua chunk: cannot open Xnonexistent.lua: No such file or directory',
      pcall_err(command, 'luafile Xnonexistent.lua')
    )
    write_file(fname, 'local x = (\n')
    eq(
      ("Vim(luafile):E5112: Lua chunk: %s:0: unexpected symbol near '<eof>'"):format(fname),
      pcall_err(command, 'luafile ' .. fname)
    )
    write_file(fname, 'error("in file")\n')
    eq(
      ('Vim(luafile):E5113: Lua chunk: %s:0: in file'):format(fname),
      remove_trace(pcall_err(command, 'luafile ' .. fname))
    )
  end)
end)

describe('vim.schedule', function()
  it('runs callbacks in order, before a zero-delay defer_fn', function()
    eq(
      { 'a', 'b', 'nested', 'deferred' },
      exec_lua(function()
        local order = {}
        vim.defer_fn(function()
          order[#order + 1] = 'deferred'
        end, 0)
        vim.schedule(function()
          order[#order + 1] = 'a'
          vim.schedule(function()
            order[#order + 1] = 'nested'
          end)
        end)
        vim.schedule(function()
          order[#order + 1] = 'b'
        end)
        vim.wait(1000, function()
          return #order == 4
        end)
        return order
      end)
    )
  end)

  it('survives a callback that drops the last reference to itself', function()
    eq(
      3,
      exec_lua(function()
        _G.count = 0
        _G.cb = function()
          _G.count = _G.count + 1
          _G.cb = nil
          collectgarbage()
          collectgarbage()
        end
        for _ = 1, 3 do
          vim.schedule(_G.cb or function()
            _G.count = _G.count + 1
          end)
        end
        vim.wait(1000, function()
          return _G.count == 3
        end)
        return _G.count
      end)
    )
    assert_alive()
  end)

  it('re-enters through vim.wait inside a callback', function()
    eq(
      { 'outer', 'inner', 'outer done' },
      exec_lua(function()
        local order = {}
        vim.schedule(function()
          order[#order + 1] = 'outer'
          vim.schedule(function()
            order[#order + 1] = 'inner'
          end)
          vim.wait(1000, function()
            return #order == 2
          end)
          order[#order + 1] = 'outer done'
        end)
        vim.wait(2000, function()
          return #order == 3
        end)
        return order
      end)
    )
  end)

  it('refuses a non-function', function()
    eq(
      'vim.schedule: expected function',
      exec_lua(function()
        local _, err = pcall(vim.schedule, 1)
        return err
      end)
    )
  end)

  it('reports a failing callback', function()
    exec_lua(function()
      vim.schedule(function()
        error('scheduled boom')
      end)
    end)
    matches(
      '^vim%.schedule callback: .-executor_seam_spec%.lua:%d+: scheduled boom',
      api.nvim_get_vvar('errmsg')
    )
  end)
end)

describe('Lua user commands', function()
  --- Defines `:Seam` with `opts` and a callback that stores the argument
  --- table, runs `cmdline`, and answers the stored table less `smods`.
  local function args_of(opts, cmdline)
    return exec_lua(function()
      vim.api.nvim_create_user_command('Seam', function(a)
        a.smods = nil
        _G.seam_args = a
      end, opts)
      vim.cmd(cmdline)
      return _G.seam_args
    end)
  end

  before_each(function()
    api.nvim_buf_set_lines(0, 0, -1, false, { 'a', 'b', 'c', 'd', 'e' })
  end)

  it('passes a range', function()
    eq({
      name = 'Seam',
      args = '',
      fargs = {},
      nargs = '0',
      bang = false,
      line1 = 2,
      line2 = 4,
      range = 2,
      count = 4,
      reg = '',
      mods = '',
    }, args_of({ range = true }, '2,4Seam'))
    eq(1, args_of({ range = true }, '3Seam').range)
    eq(
      { 0, 1, 1 },
      (function()
        local a = args_of({ range = true }, 'Seam')
        return { a.range, a.line1, a.line2 }
      end)()
    )
  end)

  it('passes a count', function()
    -- No count given: the default, and no range.
    local a = args_of({ count = 7 }, 'Seam')
    eq({ 7, 0 }, { a.count, a.range })
    a = args_of({ count = 7 }, 'Seam 3')
    eq({ 3, 1 }, { a.count, a.range })
    a = args_of({ count = 7 }, '2Seam')
    eq({ 2, 1 }, { a.count, a.range })
  end)

  it('passes the bang, the register and the modifiers', function()
    local a = args_of({ bang = true, register = true }, 'silent keepjumps Seam! x')
    -- `mods` is in the modifier table's order, not the typed one.
    eq({ true, 'x', 'keepjumps silent' }, { a.bang, a.reg, a.mods })
    eq(
      { silent = true, keepjumps = true, verbose = -1, split = '' },
      exec_lua(function()
        local smods = {}
        vim.api.nvim_create_user_command('Seam2', function(a)
          smods = a.smods
        end, {})
        vim.cmd('silent keepjumps Seam2')
        return {
          silent = smods.silent,
          keepjumps = smods.keepjumps,
          verbose = smods.verbose,
          split = smods.split,
        }
      end)
    )
  end)

  it('splits fargs by nargs', function()
    eq({ 'a', 'b c' }, args_of({ nargs = '*' }, [[Seam a b\ c]]).fargs)
    eq({ [[a b\ c]] }, args_of({ nargs = 1 }, [[Seam a b\ c]]).fargs)
    eq({}, args_of({ nargs = '?' }, 'Seam').fargs)
    eq({ 'x' }, args_of({ nargs = '+' }, 'Seam x').fargs)
    eq('a b\\ c', args_of({ nargs = '*' }, [[Seam a b\ c]]).args)
  end)

  it('reports a failing callback', function()
    exec_lua(function()
      vim.api.nvim_create_user_command('Fail', function()
        error('cmd boom')
      end, {})
    end)
    eq(
      [[Vim:Lua :command callback: .../executor_seam_spec.lua:0: cmd boom]],
      remove_trace(pcall_err(command, 'Fail'))
    )
  end)
end)

describe('vim.fn and vim.call', function()
  before_each(function()
    command('function! Id(x)\n return a:x\nendfunction')
  end)

  it('round-trips every value kind', function()
    eq(
      {
        number = { 'number', 42 },
        float = { 'number', 1.5 },
        string = { 'string', 'a\0b' },
        list = { 'table', 3 },
        dict = { 'table', 'v' },
        empty_dict = { 'table', true },
        ['true'] = { 'boolean', true },
        ['false'] = { 'boolean', false },
        null = { 'userdata', true },
        blob = { 'string', 'xy' },
      },
      exec_lua(function()
        local out = {}
        local function rt(name, v, probe)
          local got = vim.fn.Id(v)
          out[name] = { type(got), probe(got) }
        end
        local same = function(x)
          return x
        end
        rt('number', 42, same)
        rt('float', 1.5, same)
        rt('string', 'a\0b', same)
        rt('list', { 1, 2, 3 }, function(x)
          return #x
        end)
        rt('dict', { k = 'v' }, function(x)
          return x.k
        end)
        rt('empty_dict', vim.empty_dict(), function(x)
          return vim.tbl_isempty(x) and getmetatable(x) ~= nil
        end)
        rt('true', true, same)
        rt('false', false, same)
        rt('null', vim.NIL, function(x)
          return x == vim.NIL
        end)
        local blob = vim.fn.eval('0z7879')
        out.blob = { type(blob), blob }
        return out
      end)
    )
  end)

  it('round-trips a funcref and a lambda', function()
    eq(
      { 6, 'userdata', 'userdata' },
      exec_lua(function()
        local f = vim.fn.Id(function(x)
          return x * 2
        end)
        local ref = vim.fn.funcref('Id')
        local lambda = vim.fn.eval('{x -> x + 6}')
        return { f(3), type(ref), type(lambda) }
      end)
    )
  end)

  it('builds the call errors', function()
    eq(
      [[Vim(lua):E5108: Lua: Vim:E117: Unknown function: nosuchfunc]],
      remove_trace(pcall_err(command, 'lua vim.fn.nosuchfunc()'))
    )
    eq(
      [[Vim(lua):E5108: Lua: Vim:E117: Unknown function: nosuchfunc]],
      remove_trace(pcall_err(command, 'lua vim.call("nosuchfunc")'))
    )
    eq(
      [[Vim(lua):E5108: Lua: Vim:E119: Not enough arguments for function: add]],
      remove_trace(pcall_err(command, 'lua vim.fn.add()'))
    )
    eq(
      [[Vim(lua):E5108: Lua: Vim:E118: Too many arguments for function: sin]],
      remove_trace(pcall_err(command, 'lua vim.fn.call("sin", {1, 2})'))
    )
    eq(
      [[Vim(lua):E5108: Lua: [string ":lua"]:0: Function called with too many arguments]],
      remove_trace(pcall_err(command, 'lua vim.call("sin", ' .. ('1,'):rep(21) .. '1)'))
    )
    eq(
      -- The conversion reports itself (`emsg`) and the call answers nil.
      [[Vim(lua):E5100: Cannot convert given Lua table: table should contain either only integer keys or only string keys]],
      remove_trace(pcall_err(command, 'lua vim.fn.Id({1, a = 2})'))
    )
  end)

  it('refuses a non-fast function in a fast event', function()
    local name = ('x'):rep(150)
    eq(
      {
        'E5560: Vimscript function "sin" must not be called in a fast event context',
        -- The name is cut at 100 bytes plus one: the buffer's length leaves
        -- room for the closing quote, but the cut falls on it.
        ('E5560: Vimscript function "%s must not be called in a fast event context'):format(
          name:sub(1, 101)
        ),
        'E5560: Vimscript function "sin" must not be called in a fast event context',
        'E5560: rpcrequest must not be called in a fast event context',
      },
      exec_lua(function()
        local res
        local timer = vim.uv.new_timer()
        timer:start(0, 0, function()
          local out = {}
          for _, f in ipairs({
            function()
              return vim.call('sin', 1)
            end,
            function()
              return vim.call(name, 1)
            end,
            function()
              return vim.fn.sin(1)
            end,
            function()
              return vim.rpcrequest(1, 'nvim_eval', '1')
            end,
          }) do
            local _, err = pcall(f)
            out[#out + 1] = err:gsub('^.-:%d+: ', '')
          end
          res = out
          timer:close()
        end)
        vim.wait(2000, function()
          return res ~= nil
        end)
        return res
      end)
    )
  end)

  it('calls a fast function from a fast event', function()
    eq(
      'abc',
      exec_lua(function()
        local res
        local timer = vim.uv.new_timer()
        timer:start(0, 0, function()
          res = vim.fn.tolower('ABC')
          timer:close()
        end)
        vim.wait(2000, function()
          return res ~= nil
        end)
        return res
      end)
    )
  end)
end)

describe('require', function()
  it('wraps a missing module', function()
    matches(
      [[^Vim%(lua%):E5108: Lua: %[string ":lua"%]:0: module 'nosuchmod' not found:]],
      pcall_err(command, 'lua require("nosuchmod")')
    )
  end)

  it('wraps an error inside a module', function()
    exec_lua(function()
      package.preload['seam_broken'] = function()
        error('broken module')
      end
    end)
    eq(
      [[Vim(lua):E5108: Lua: .../executor_seam_spec.lua:0: broken module]],
      remove_trace(pcall_err(command, 'lua require("seam_broken")'))
    )
  end)
end)

describe('v:lua', function()
  it('calls a global and a module function', function()
    exec_lua(function()
      _G.seam_add = function(a, b)
        return a + b
      end
      package.loaded.seam_mod = {
        twice = function(x)
          return x * 2
        end,
      }
    end)
    eq(5, eval('v:lua.seam_add(2, 3)'))
    eq(8, eval([[v:lua.require'seam_mod'.twice(4)]]))
    eq(1, fn.exists('*v:lua.seam_add'))
    eq(0, fn.exists('*v:lua.seam_missing'))
  end)

  it('builds its errors', function()
    eq(
      [[Vim:E5108: Lua: [string "v:lua"]:0: attempt to call global 'seam_missing' (a nil value)]],
      remove_trace(pcall_err(eval, 'v:lua.seam_missing()'))
    )
    eq(NIL, exec_lua('return _G.seam_missing'))
  end)
end)
