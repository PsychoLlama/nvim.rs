-- Timing canary for the msgpack-RPC transport, the channel reader and the
-- remote UI.
--
--   nvim --headless -c 'luafile rpcbench.lua' -c 'qa!'
--
-- Run it through `rpcbench.sh`, which does the interleaved A/B the drift
-- rule requires. Prints "RPCBENCH<TAB>phase<TAB>ms" lines so the shell can
-- pick them out of whatever else lands on stdout.
--
-- Why this exists: evalbench's `luapush luapop objconv objpop` time the
-- value converters, but nothing timed what carries those values between
-- processes -- the packer, the framer (`unpacker_advance`), `rpc_send_call`
-- and its reply path, the job channel's reader and the remote UI's packer.
-- Every phase talks to a child: an `--embed` nvim (the same binary, via
-- `v:progpath`) for the RPC phases, `cat` for the job phase. Under
-- `--cachegrind` only this process is counted (no `--trace-children`), so
-- the numbers are the parent's side of each exchange.
--
-- Phase map (which code each phase is the canary for):
--   ctl        nothing          -- noise floor: a Vimscript builtin call
--                                  that never reaches a channel
--   rpcreq     msgpack_rpc/     -- `rpcrequest()` round trips: the packer
--                                  for the request, `rpc_send_call`'s
--                                  wait, the framer and the reply's
--                                  unpack into a typval
--   rpcnotify  msgpack_rpc/     -- `rpcnotify()` flood, drained by one
--                                  request: the send path alone
--   chanjob    channel/         -- `jobstart(['cat'])` + `chansend()` and
--                                  `on_stdout` in streaming and buffered
--                                  mode: the reader, line splitting, the
--                                  callback delivery
--   uiattach   api/ui/, ui/     -- a remote UI (`ext_linegrid`) attached
--                                  over an rpc job channel while this
--                                  process redraws: `remote_ui_raw_line`,
--                                  the UI packer and its flush per redraw
--   uilua      api/ui/, lua/    -- a `vim.ui_attach` Lua sink receiving
--                                  `ext_messages` events: the Lua UI
--                                  fan-out and the event conversion
--   objarr     api/private/     -- a 100k-element array each way:
--                                  `object_to_vim`/`vim_to_object` and the
--                                  packer/unpacker over one big value (the
--                                  `ptr + len` Object question's phase)
--
-- Noise: see rpcbench.sh's header (the three-run cachegrind spread, 0.24 %). Every
-- phase waits on another process, so the number of event-loop turns it
-- takes varies with the child's scheduling; that is what this canary's
-- noise is made of, and why its floor is wider than mlbench's.

-- RPCBENCH_ONLY runs a comma-separated subset of the phases, for a
-- per-phase instruction count. The setup always runs.

local NROUND = tonumber(os.getenv('RPCBENCH_SCALE') or '1')
local ONLY = os.getenv('RPCBENCH_ONLY')

vim.cmd([==[
  silent! set nomore noruler noshowcmd noshowmode shortmess=filnxtToOFsW
  silent! set noswapfile nobackup nowritebackup noundofile undolevels=-1
  silent! set report=9999 cmdheight=1
]==])

-- ---------------------------------------------------------------- time
local function ms(f, rounds)
  local t0 = vim.uv.hrtime()
  for _ = 1, math.max(1, math.floor(rounds * NROUND)) do
    f()
  end
  return (vim.uv.hrtime() - t0) / 1e6
end

local function wanted(name)
  if not ONLY then
    return true
  end
  for p in ONLY:gmatch('[^,]+') do
    if p == name then
      return true
    end
  end
  return false
end

local out = {}
local function phase(name, rounds, f)
  if not wanted(name) then
    return
  end
  out[#out + 1] = ('RPCBENCH\t%s\t%.1f'):format(name, ms(f, rounds))
end

--- Raises unless `cond` holds: a phase that quietly does nothing is
--- invisible in a timing number, so each one checks its effect once first.
local function check(cond, what)
  if not cond then
    error('rpcbench: ' .. what, 0)
  end
end

-- --------------------------------------------------------------- setup
-- One embedded child for every RPC phase, started once.
local child = vim.fn.jobstart({
  vim.v.progpath,
  '--embed',
  '--headless',
  '--clean',
  '-n',
  '-i',
  'NONE',
}, { rpc = true })
check(child > 0, 'could not start the --embed child')
check(vim.rpcrequest(child, 'nvim_eval', '1 + 1') == 2, 'the child does not answer')

local SMALL = '[1, "two", 3.5, v:null, v:true, {"k": [1, 2, 3]}]'
local BIG = {}
for i = 1, 100000 do
  BIG[i] = i
end
check(vim.rpcrequest(child, 'nvim_eval', 'range(100000)')[100000] == 99999, 'objarr reply')

-- --------------------------------------------------------------- ctl
phase('ctl', 200000, function()
  local _ = vim.fn.strwidth('edit files/f01234.txt')
end)

-- -------------------------------------------------------------- rpcreq
check(vim.rpcrequest(child, 'nvim_eval', SMALL)[6].k[3] == 3, 'rpcreq reply')
phase('rpcreq', 12000, function()
  local _ = vim.rpcrequest(child, 'nvim_eval', SMALL)
end)

-- ----------------------------------------------------------- rpcnotify
-- 200 notifications that each set a counter, then one request that reads
-- it back: the request is answered after every notification is handled.
vim.rpcnotify(child, 'nvim_set_var', 'n', 0)
check(vim.rpcrequest(child, 'nvim_get_var', 'n') == 0, 'rpcnotify reset')
phase('rpcnotify', 400, function()
  for i = 1, 200 do
    vim.rpcnotify(child, 'nvim_set_var', 'n', i)
  end
  check(vim.rpcrequest(child, 'nvim_get_var', 'n') == 200, 'rpcnotify drain')
end)

-- ------------------------------------------------------------- chanjob
-- 2,000 lines through `cat`, read back once streaming and once buffered.
-- The trailing '' ends the last line, so in both modes every call's data
-- holds one more item than it completes lines: a streaming call's first item
-- continues the previous call's last, and the EOF call is {''}.
local LINES = {}
for i = 1, 2000 do
  LINES[i] = ('line %05d with some text after it'):format(i)
end
LINES[#LINES + 1] = ''
local function catjob(buffered)
  local got = 0
  local done = false
  local job = vim.fn.jobstart({ 'cat' }, {
    stdout_buffered = buffered,
    on_stdout = function(_, data)
      got = got + #data - 1
    end,
    on_exit = function()
      done = true
    end,
  })
  vim.fn.chansend(job, LINES)
  vim.fn.chanclose(job, 'stdin')
  vim.wait(10000, function()
    return done
  end, 1)
  return got
end
check(catjob(false) == 2000 and catjob(true) == 2000, 'chanjob line count')
phase('chanjob', 40, function()
  catjob(false)
  catjob(true)
end)

-- ------------------------------------------------------------ uiattach
-- A remote UI that is not an nvim: an `--embed` child cannot be the UI (an
-- nvim that is not a UI client closes the channel on its first `redraw`,
-- upstream too), so the UI is `sh` replaying one pre-encoded
-- `nvim_ui_attach` request on its stdout and then `cat`ting every event
-- this process sends it to /dev/null. Each round attaches it, rewrites and
-- fully redraws a 120x40 screen fifty times, and closes the channel, which
-- detaches it.
vim.o.columns = 120
vim.o.lines = 40
local SCREEN = {}
for i = 1, 80 do
  SCREEN[i] = ('%03d '):format(i) .. ('abcdefghij'):rep(11)
end
vim.api.nvim_buf_set_lines(0, 0, -1, false, SCREEN)
local ATTACH = vim.fn.tempname()
vim.fn.writefile(
  vim.fn.msgpackdump({ { 0, 1, 'nvim_ui_attach', { 120, 40, { ext_linegrid = true } } } }),
  ATTACH,
  'b'
)
local function nuis()
  return #vim.api.nvim_list_uis()
end
local function with_ui(f)
  local ui = vim.fn.jobstart(
    { 'sh', '-c', 'cat "$1"; exec cat >/dev/null', 'sh', ATTACH },
    { rpc = true }
  )
  vim.wait(10000, function()
    return nuis() == 1
  end, 1)
  local attached = nuis() == 1
  f()
  vim.fn.chanclose(ui)
  vim.wait(10000, function()
    return nuis() == 0
  end, 1)
  vim.fn.jobwait({ ui }, 10000)
  return attached
end
local flip = 0
local function storm()
  for _ = 1, 50 do
    flip = flip + 1
    -- Alternate the top line between two lengths and the view between two
    -- places, so every redraw rewrites the cells it sends.
    vim.cmd(flip % 2 == 0 and 'normal! gg' or 'normal! 30Gzt')
    vim.api.nvim_buf_set_lines(0, 0, 1, false, { ('%d'):format(flip):rep(30) })
    vim.cmd('redraw!')
  end
end
check(with_ui(function() end), 'uiattach: the UI did not attach')
check(nuis() == 0, 'uiattach: the UI did not detach')
phase('uiattach', 8, function()
  with_ui(storm)
end)

-- --------------------------------------------------------------- uilua
local ns = vim.api.nvim_create_namespace('rpcbench')
local seen = 0
local function sink(event)
  if event == 'msg_show' then
    seen = seen + 1
  end
end
vim.ui_attach(ns, { ext_messages = true }, sink)
vim.cmd('echomsg "probe" | redraw')
check(seen > 0, 'uilua: the Lua sink saw no msg_show')
vim.ui_detach(ns)
phase('uilua', 80, function()
  vim.ui_attach(ns, { ext_messages = true }, sink)
  for i = 1, 200 do
    vim.api.nvim_echo({ { 'message ', 'Normal' }, { tostring(i), 'ErrorMsg' } }, false, {})
    vim.cmd('redraw')
  end
  vim.ui_detach(ns)
end)

-- -------------------------------------------------------------- objarr
check(vim.rpcrequest(child, 'nvim_call_function', 'len', { BIG }) == 100000, 'objarr send')
phase('objarr', 6, function()
  local _ = vim.rpcrequest(child, 'nvim_eval', 'range(100000)')
  local _ = vim.rpcrequest(child, 'nvim_call_function', 'len', { BIG })
end)

vim.fn.jobstop(child)
io.write(table.concat(out, '\n'), '\n')
