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
  it('in an undo file line length', function()
    -- The header's `U` line length (byte 47, big-endian) is read straight
    -- after the hash, so a file that passes the hash check claims 2 GiB
    -- for it and holds nothing.
    local out = capped([[
      vim.fn.writefile({ 'alpha', 'beta' }, DIR .. '/text')
      vim.cmd.edit(DIR .. '/text')
      vim.cmd('normal! ggdd')
      vim.cmd.wundo(DIR .. '/good.un~')
      local f = assert(io.open(DIR .. '/good.un~', 'rb'))
      local raw = f:read('*a')
      f:close()
      f = assert(io.open(DIR .. '/bad.un~', 'wb'))
      f:write(raw:sub(1, 47) .. '\127\255\255\240')
      f:close()
      local ok, err = pcall(vim.cmd.rundo, DIR .. '/bad.un~')
      io.stdout:write(tostring(ok), ' ', tostring(err))
    ]])
    -- It ends where the line should be, which is no line, and the fields
    -- after it read as the end of the file.
    t.matches('^false Vim:E825: Corrupted undo file', out)
  end)

  it('in a ShaDa entry length', function()
    -- A register entry (type 5), timestamp 0, claiming 2 GiB of body.
    local out = capped([[
      local f = assert(io.open(DIR .. '/bad.shada', 'wb'))
      f:write('\005\000\206\127\255\255\240')
      f:close()
      local _, err = pcall(vim.cmd.rshada, { DIR .. '/bad.shada', bang = true })
      io.stdout:write(err)
    ]])
    t.matches('E576: .*occupies 2147483632 bytes, but file ended earlier', out)
  end)

  it('in a spell file tree length', function()
    -- No sections, then a case-folded tree claiming 2 GiB entries: two
    -- arrays of that many were allocated before a node was read.
    local out = capped([[
      vim.fn.mkdir(DIR .. '/spell', 'p')
      local f = assert(io.open(DIR .. '/spell/en.utf-8.spl', 'wb'))
      f:write('VIMspell\050\255\127\255\255\240\001\000')
      f:close()
      vim.o.runtimepath = DIR
      vim.o.spelllang = 'en'
      local _, err = pcall(vim.cmd, 'set spell')
      io.stdout:write(err)
    ]])
    t.matches('E758: Truncated spell file', out)
  end)

  it('in msgpack string, binary and extension headers', function()
    -- `vim.mpack.decode` and an `Unpacker` fed one piece: each header
    -- claims 4 GiB of payload and three bytes follow.
    local out = capped([[
      for _, head in ipairs({ '\219', '\198', '\201' }) do
        local claim = head .. '\255\255\255\240abc'
        io.stdout:write(select(2, pcall(vim.mpack.decode, claim)), '|')
        io.stdout:write(tostring(vim.mpack.Unpacker()(claim)), '|')
      end
    ]])
    eq(('incomplete msgpack string|nil|'):rep(3), out)
  end)

  it('in an RPC request', function()
    -- [0, 1, "nvim_eval", [<str 32 claiming 4 GiB>]], then three bytes of
    -- it, and the same with the claim as a map key: the server waits for
    -- the rest instead of reserving the claim.
    local alive = n.exec_lua(function(argv, env)
      local answers = {}
      for _, arg in ipairs({ '\219\255\255\255\240abc', '\129\219\255\255\255\240abc' }) do
        local job = vim.fn.jobstart(argv, { env = env })
        vim.fn.chansend(job, '\148\000\001\169nvim_eval\145' .. arg)
        vim.wait(500)
        answers[#answers + 1] = vim.fn.jobwait({ job }, 0)[1]
        vim.fn.jobstop(job)
      end
      return answers
    end, { 'prlimit', '--as=' .. CAP, n.nvim_prog, '--clean', '--embed', '--headless' }, {
      NVIM_LOG_FILE = dir .. '/nvim.log',
      XDG_STATE_HOME = dir .. '/state',
    })
    -- -1: still running.
    eq({ -1, -1 }, alive)
  end)
end)
