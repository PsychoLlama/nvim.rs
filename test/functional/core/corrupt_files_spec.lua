-- Files read from disk are input: a length or count a corrupt one claims
-- must not size an allocation before the bytes are there to back it. Each
-- case writes a small file whose header claims gigabytes and reads it in a
-- child capped at a fraction of that address space; the child surviving
-- with the reader's own error is the assertion.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq

--- The child's address-space cap. A debug build starts in well under it;
--- a single claimed allocation of the sizes below cannot fit.
local CAP = 1024 * 1024 * 1024

describe('a corrupt file claiming a huge size', function()
  local dir

  before_each(function()
    -- Under ASan the child reserves terabytes of shadow up front, which the
    -- cap refuses before any reader runs.
    t.skip(t.is_asan(), 'the address-space cap does not fit ASan')
    n.clear()
    dir = t.tmpname(false)
    t.mkdir(dir)
  end)

  after_each(function()
    if dir then
      n.rmdir(dir)
    end
  end)

  --- Runs `code` (Lua, with `dir` as the global `DIR`) in a fresh capped
  --- child and answers what it printed to stdout, failing if it died.
  local function capped(code)
    local script = dir .. '/probe.lua'
    t.write_file(script, ('DIR = %q\n%s\nvim.cmd("qa!")\n'):format(dir, code), true)
    -- The runner has no `vim.system`; the session does.
    local done = n.exec_lua(
      function(argv, cwd, env)
        local r = vim.system(argv, { cwd = cwd, env = env, text = true }):wait(60000)
        return { code = r.code, signal = r.signal, stdout = r.stdout, stderr = r.stderr }
      end,
      { 'prlimit', '--as=' .. CAP, n.nvim_prog, '--clean', '--headless', '-l', script },
      dir,
      {
        NVIM_LOG_FILE = dir .. '/nvim.log',
        XDG_STATE_HOME = dir .. '/state',
        XDG_DATA_HOME = dir .. '/data',
        XDG_CONFIG_HOME = dir .. '/config',
        XDG_CACHE_HOME = dir .. '/cache',
        TMPDIR = dir,
      }
    )
    local said = ('the reader survives the claim\n%s%s'):format(done.stdout, done.stderr)
    eq({ 0, 0 }, { done.code, done.signal }, said)
    return done.stdout
  end

  --- Lua for the child: a preserved swap file of a two-line buffer, read
  --- into `raw`, then `patch(raw)` recovered from as `bad.swp`, with what
  --- recovery said printed.
  local function recover_patched(patch)
    return capped([[
      vim.fn.writefile({ 'alpha', 'beta' }, DIR .. '/text')
      vim.o.directory = DIR .. '//'
      vim.o.swapfile = true
      vim.o.updatecount = 200 -- `-l` turns swap files off with a zero
      vim.cmd.edit(DIR .. '/text')
      vim.cmd('normal! ggdd')
      vim.cmd.preserve()
      local f = assert(io.open(vim.fn.swapname('%'), 'rb'))
      local raw = f:read('*a')
      f:close()
      vim.cmd('bwipe!')
      local bad = DIR .. '/bad.swp'
      f = assert(io.open(bad, 'wb'))
      f:write(]] .. patch .. [[)
      f:close()
      local _, said = pcall(vim.fn.execute, 'recover! ' .. vim.fn.fnameescape(bad))
      io.stdout:write(said)
    ]])
  end

  it('in a swap file page size', function()
    -- Block zero's page size is little-endian at byte 12. Recovery used to
    -- reallocate block zero's page at whatever it said.
    local out = recover_patched([[raw:sub(1, 12) .. '\240\255\255\255' .. raw:sub(17)]])
    t.matches('has been damaged %(page size is larger than maximum value%)', out)
  end)
end)
