local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear = n.clear
local eq = t.eq
local environ = n.fn.environ
local exists = n.fn.exists
local system = n.fn.system
local nvim_prog = n.nvim_prog
local command = n.command
local eval = n.eval
local setenv = n.fn.setenv

describe('environment variables', function()
  it('environ() handles empty env variable', function()
    clear({ env = { EMPTY_VAR = '' } })
    eq('', environ()['EMPTY_VAR'])
    eq(nil, environ()['DOES_NOT_EXIST'])
  end)

  it('exists() handles empty env variable', function()
    clear({ env = { EMPTY_VAR = '' } })
    eq(1, exists('$EMPTY_VAR'))
    eq(0, exists('$DOES_NOT_EXIST'))
  end)

  it('environ() leaves out an entry without "="', function()
    t.skip(t.is_os('win'), 'execve')
    clear()
    -- Only a raw execve() can hand a child such an entry; LuaJIT's FFI
    -- makes the call from an `nvim -l` script.
    local script = t.tmpname()
    t.write_file(
      script,
      [[
      local ffi = require('ffi')
      ffi.cdef('int execve(const char *, char *const[], char *const[]);')
      local function strv(t)
        local v = ffi.new('char *[?]', #t + 1)
        for i, s in ipairs(t) do
          v[i - 1] = ffi.cast('char *', s)
        end
        return v
      end
      -- The test's own environment rides along (its NVIM_LOG_FILE and
      -- XDG dirs keep the child off the user's).
      ffi.cdef('extern char **environ;')
      local env = { 'NOEQUALS', 'KEPT=1' }
      local i = 0
      while ffi.C.environ[i] ~= nil do
        env[#env + 1] = ffi.string(ffi.C.environ[i])
        i = i + 1
      end
      local prog = arg[1]
      -- The tables stay referenced: they hold the strings the arrays point at.
      local args = { prog, '--clean', '--headless', '-u', 'NONE', '-i', 'NONE',
        '+lua local e = vim.fn.environ() io.stdout:write(vim.inspect({ e.KEPT, e.NOEQUALS }))',
        '+qa!' }
      ffi.C.execve(prog, strv(args), strv(env))
      os.exit(2)
    ]]
    )
    finally(function()
      os.remove(script)
    end)
    eq('{ "1" }', system({ nvim_prog, '-l', script, nvim_prog }))
    eq(0, eval('v:shell_error'))
  end)
end)

describe('empty $HOME', function()
  local original_home = os.getenv('HOME')

  before_each(clear)

  -- recover $HOME after each test
  after_each(function()
    if original_home ~= nil then
      setenv('HOME', original_home)
    end
    os.remove('test_empty_home')
    os.remove('./~')
  end)

  local function tilde_in_cwd()
    -- get files in cwd
    command("let test_empty_home_cwd_files = split(globpath('.', '*'), '\n')")
    -- get the index of the file named '~'
    command('let test_empty_home_tilde_index = index(test_empty_home_cwd_files, "./~")')
    return eval('test_empty_home_tilde_index') ~= -1
  end

  local function write_and_test_tilde()
    system({
      nvim_prog,
      '-u',
      'NONE',
      '-i',
      'NONE',
      '--headless',
      '-c',
      'write test_empty_home',
      '+q',
    })
    eq(false, tilde_in_cwd())
  end

  it("'~' folder not created in cwd if $HOME and related env not defined", function()
    command('unlet $HOME')
    write_and_test_tilde()

    command("let $HOMEDRIVE='C:'")
    command("let $USERPROFILE='C:\\'")
    write_and_test_tilde()

    command('unlet $HOMEDRIVE')
    write_and_test_tilde()

    command('unlet $USERPROFILE')
    write_and_test_tilde()

    command("let $HOME='%USERPROFILE%'")
    command("let $USERPROFILE='C:\\'")
    write_and_test_tilde()
  end)

  it("'~' folder not created in cwd if writing a file with invalid $HOME", function()
    setenv('HOME', '/path/does/not/exist')
    write_and_test_tilde()
  end)

  it("'~' folder not created in cwd if writing a file with $HOME=''", function()
    command("let $HOME=''")
    write_and_test_tilde()
  end)
end)
