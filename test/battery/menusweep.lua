-- menusweep -- the seventeenth baselined differential.  Driven by
-- menusweep.sh, which builds the sandbox, pins $HOME/
-- $TMPDIR/$PATH and does the scrubs only the shell can see.  Read that
-- header first.
--
-- The subsystem is menu.rs (2,299 lines, 2,172 of them unchecked): the
-- `:menu` command family end to end.  Before this oracle the file had
-- NO differential at all -- B19's survey S3, hole 3, and the last
-- phase-16 file in that state after B18-5 closed `eval/fs.rs`.
--
-- THE DESIGN, in one paragraph.  Everything a menu does that can be
-- observed is TEXT: the `:menu` listing that `show_menus` prints, the
-- tree `menu_get()` builds, the dict `menu_info()` builds, the
-- completion candidates `get_menu_name`/`get_menu_names` yield, and the
-- error messages.  All of that runs in this `-l` process directly, one
-- `nvim_exec2(..., {output = true})` per command.  Two things cannot:
-- `:emenu` outside Normal mode (`execute_menu` reads `State` /
-- `restart_edit` / `VIsual_active`, and an `-l` script is never in
-- Insert or Visual) and `:popup` on a menu that exists (`show_popupmenu`
-- enters `pum_show_popupmenu`, whose `vgetc()` loop has no `K_EVENT`
-- arm and therefore never returns without a key).  Those two sections
-- drive an `--embed` child over `jobstart(rpc = true)` -- b19-3's
-- `Child`, verbatim: `nvim_input` is a FAST call and is dispatched even
-- from inside that frozen loop, which is what makes `<Esc>` an escape.
--
-- Sections:
--   s0  default     the runtime's own `PopUp` menu, recorded once
--   s1  cmds        every `:*menu` spelling -> `get_menu_cmd_modes`
--   s2  modes       one definition per mode x menu_info in every mode
--   s3  priority    `:menu 80.5`, `:1menu`, deep priority paths, order
--   s4  names       escapes, `<Tab>` accel, `&` mnemonic, separators,
--                   hidden `]`, the four special roots, unicode
--   s5  listing     `show_menus` output in every shape
--   s6  get         `menu_get()` / `menu_info()` over a fixed tree
--   s7  unmenu      removal, per-mode removal, the popup copies
--   s8  enable      `:menu enable` / `disable`, `*`, recursion
--   s9  complete    `set_context_in_menu_cmd` + the two name generators
--   s10 translate   `:menutranslate`, `menutrans_lookup`, en_name
--   s11 emenu       `:emenu` in every real mode, in an `--embed` child
--   s12 popup       `:popup` -- the error paths here, the frozen loop
--                   in a child that escapes it
--   s91 crashprobe  the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A sweep that goes
-- silently empty otherwise looks exactly like a healthy one.
--
-- DETERMINISM.  Menus are process-global and there is no `:menuclear`,
-- so every case starts from `wipe()`: `aunmenu *`, `aunmenu! *`,
-- `tlunmenu *`, `tunmenu *` and `menutranslate clear`, which between
-- them cover all eight mode indices plus the translation array.  s0 is
-- the ONE section that runs before the wipe.

local work = assert(os.getenv('MENU_WORK'), 'MENU_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local only = os.getenv('MENUSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('MENUSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
  end
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  text = text:gsub('nvim%.[%w_.-]+/[%w]+', 'nvim.<U>/<T>')
  return text
end

local function cap(text, limit)
  limit = limit or 600
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  Menu names carry tabs (the accel
--- separator), CTRL-A (the completion sentinel `get_menu_names` appends
--- to a submenu) and multibyte text.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

local SEEN = {}
local function label_once(label)
  if SEEN[label] then
    emit('!!', 'DUPLICATE', 'LABEL', label)
  end
  SEEN[label] = true
  return label
end

-- -------------------------------------------------------------- struct

local structfd =
  assert(io.open(assert(os.getenv('MENU_STRUCT'), 'MENU_STRUCT unset'), 'w'))

local function q(str)
  return '"'
    .. (str:gsub('[%c"\\\128-\255]', function(c)
      return string.format('\\x%02x', c:byte())
    end))
    .. '"'
end

local canon
function canon(value)
  local t = type(value)
  if t == 'string' then
    return q(scrub(value))
  elseif t == 'number' then
    if value == math.floor(value) and math.abs(value) < 2 ^ 53 then
      return string.format('%d', value)
    end
    return string.format('%.17g', value)
  elseif t ~= 'table' then
    return q(tostring(value))
  end
  local n = 0
  for _ in pairs(value) do
    n = n + 1
  end
  if n == #value then
    local parts = {}
    for i, item in ipairs(value) do
      parts[i] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for k in pairs(value) do
    keys[#keys + 1] = tostring(k)
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    parts[#parts + 1] = q(k)
      .. ':'
      .. canon(value[k] == nil and value[tonumber(k)] or value[k])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, '\t', canon(value), '\n')
end

--- Normalise an error to its message.  A pcall against an API function
--- prefixes the Lua source position, which is a line number in THIS
--- file and would re-baseline the whole artifact on any edit above.
local function errtext(res)
  local s = tostring(res)
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-menusweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('^nvim_exec2%(%), line %d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

local function ins(value)
  return (vim.inspect(value, { newline = ' ', indent = '' }))
end

--- Serialise a small table back into Lua source, so the parent can hand
--- a child's helper its arguments without a second roundtrip.
local function tolua(value)
  local t = type(value)
  if t == 'nil' then
    return 'nil'
  elseif t == 'boolean' or t == 'number' then
    return tostring(value)
  elseif t == 'string' then
    return string.format('%q', value)
  end
  local parts = {}
  if #value > 0 then
    for _, v in ipairs(value) do
      parts[#parts + 1] = tolua(v)
    end
  else
    local keys = {}
    for k in pairs(value) do
      keys[#keys + 1] = k
    end
    table.sort(keys)
    for _, k in ipairs(keys) do
      parts[#parts + 1] = string.format('[%q]=%s', k, tolua(value[k]))
    end
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

-- ------------------------------------------------------------ sections

local secrows = 0
local function section(name, fn)
  if only and not name:match(only) then
    return
  end
  if trace then
    io.stderr:write('== ' .. name .. '\n')
  end
  secrows = rows
  emit('##', name)
  local ok, err = pcall(fn)
  if not ok then
    emit('##', name, 'RAISED', esc(scrub(errtext(err))))
  end
  emit('##', name, string.format('rows=%d', rows - secrows - 1))
end

-- ============================================================ the world

--- Every menu, in every mode index, gone -- plus the translation array.
---
--- `aunmenu` covers `a` = n/v/s/o/i/c; the tip (`t`) and terminal (`tl`)
--- indices are NOT in it and each needs its own command, or a `tmenu`
--- from one case is still attached when the next case lists the tree.
--- `menutranslate clear` matters just as much: a stale translation
--- renames every menu DEFINED afterwards, which reads like a name-parse
--- regression rather than leaked state.
local function wipe()
  for _, c in ipairs({
    'silent! aunmenu *',
    'silent! aunmenu! *',
    'silent! tlunmenu *',
    'silent! tunmenu *',
    'silent! menutranslate clear',
  }) do
    pcall(vim.api.nvim_exec2, c, { output = false })
  end
end

local function options()
  pcall(
    vim.api.nvim_exec2,
    table.concat({
      'set noswapfile nobackup nowritebackup hidden',
      'set report=9999 shortmess=aoOtTIcCF nomore belloff=all',
      'set wildoptions= wildignorecase& ignorecase& wildmenu&',
      'set encoding=utf-8 langmenu= helplang= cpoptions=aABceFs_',
      'set selection=inclusive virtualedit= startofline',
      'set lines=24 columns=80 laststatus=2 showtabline=1 noruler noshowcmd',
      'set eventignore= verbose=1',
    }, ' | '),
    { output = false }
  )
end

--- The buffer every `:emenu` runs against.
local FIXTURE = {
  'alpha beta gamma',
  'one two three',
  'four five six',
  'seven eight nine',
  'ten eleven twelve',
}

local function refixture()
  local buf = vim.api.nvim_create_buf(true, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, FIXTURE)
  vim.api.nvim_win_set_buf(0, buf)
  vim.api.nvim_win_set_cursor(0, { 1, 0 })
end

--- Run one ex command line, capturing BOTH halves of the answer.
---
--- `:menu`'s listing goes through the message system, so `output = true`
--- is the only way to see it; an error comes back as a Lua error with
--- the `Vim(cmd):` prefix, which IS the information (it names the
--- command that raised).
local function ex(label, line, opts)
  opts = opts or {}
  label_once(label)
  local answer = { cmd = line }
  local ok, res = pcall(vim.api.nvim_exec2, line, { output = true })
  if ok then
    local out = res.output or ''
    answer.out = out
    -- `quiet` suppresses the REPORT row for a command that said nothing,
    -- never the struct row: a definition that starts printing is a
    -- regression, and the canonical JSON is where it shows.
    if not (opts.quiet and out == '') then
      emit(label, 'O', esc(scrub(out)))
    end
  else
    answer.err = errtext(res)
    emit(label, '!', esc(answer.err))
  end
  struct(label, answer)
  return answer
end

--- A structured answer: menu_get / menu_info / a completion list / the
--- editor state a menu left behind.
local function ask(label, value)
  label_once(label)
  emit(label, '=', esc(scrub(ins(value))))
  struct(label, value)
  return value
end

--- Call a Vim function, reducing an error to its message so a case that
--- raises is one row rather than a dead section.
local function fn(name, ...)
  local ok, res = pcall(vim.fn[name], ...)
  if ok then
    return res
  end
  return { ERR = errtext(res) }
end

--- Run a list of definition commands with no output rows.  The setup of
--- a tree is not the measurement; the tree is.
local function setup(lines)
  local bad = nil
  for _, line in ipairs(lines) do
    local ok, res = pcall(vim.api.nvim_exec2, line, { output = false })
    if not ok then
      bad = bad or {}
      bad[#bad + 1] = line .. ' -> ' .. errtext(res)
    end
  end
  return bad
end

--- The standing tree.  Deliberately mixed: two priority shapes, an
--- accelerator, a mnemonic, a separator, a `<silent>` and a `<script>`
--- item, a three-level path, a tip, the four special roots, and one
--- item defined only in Insert mode so that every per-mode question has
--- something to answer differently.
local TREE = {
  -- `<Tab>` is the accelerator separator; a literal `\t` in a menu name
  -- is `\` + `t`, i.e. an escaped ordinary letter, and the item is
  -- called `OpentCTRL-O`.  s4 keeps both spellings on purpose.
  'menu 10.10 &File.&Open<Tab>CTRL-O :echo "open"<CR>',
  'menu 10.20 File.&Save :echo "save"<CR>',
  'menu 10.30 File.-sep1- <Nop>',
  'menu 10.40 File.&Quit<Tab>ZZ :echo "quit"<CR>',
  'imenu 20.10 Edit.OnlyInsert insert-rhs',
  'vmenu 20.20 Edit.OnlyVisual visual-rhs',
  'nmenu <silent> 30.10 Tools.Quiet :echo "quiet"<CR>',
  'nmenu <script> 30.20 Tools.Scripted :echo "scripted"<CR>',
  'menu 40.10.10 Deep.Sub.Leaf :echo "leaf"<CR>',
  'menu ]Hidden.Item :echo "hidden"<CR>',
  'menu PopUp.Mine :echo "popup"<CR>',
  'menu ToolBar.Save :echo "toolbar"<CR>',
  'menu WinBar.Bar :echo "winbar"<CR>',
  'tmenu File.Open Open a file',
}

local function tree()
  wipe()
  return setup(TREE)
end

-- ================================================================ child
-- s11 and s12 only.  `execute_menu`'s mode index and
-- `pum_show_popupmenu`'s key loop both need a main input loop, and an
-- `-l` process has none.
-- ====================================================================

local Child = {}
Child.__index = Child

local function child_start(args)
  local argv = {
    work .. '/bin/nvim',
    '--headless',
    '--embed',
    '-u',
    'NONE',
    '-i',
    'NONE',
    '--cmd',
    'set noswapfile',
  }
  for _, a in ipairs(args or {}) do
    argv[#argv + 1] = a
  end
  local errlines = {}
  local chan = vim.fn.jobstart(argv, {
    rpc = true,
    cwd = work,
    clear_env = true,
    -- Without this a dying child's words are dropped on the floor:
    -- `jobstart` discards stderr it is not asked for, and s91's whole
    -- verdict is "did it die and what did it say".
    on_stderr = function(_, data)
      for _, line in ipairs(data or {}) do
        if line ~= '' then
          errlines[#errlines + 1] = line
        end
      end
    end,
    env = {
      HOME = work .. '/home',
      PATH = work .. '/bin',
      TMPDIR = work .. '/tmp',
      TERM = 'dumb',
      SHELL = '/bin/sh',
      LANG = 'C.UTF-8',
      VIMRUNTIME = runtime,
      NVIM_TEST = '1',
    },
  })
  assert(chan > 0, 'embed child failed to start')
  return setmetatable({ chan = chan, dead = false, err = errlines }, Child)
end

--- Run a chunk in the child.
---
--- A failed `rpcrequest` is TWO different events and conflating them
--- poisons a whole section (b19-3): a Lua error inside the chunk comes
--- back as an error RESPONSE and the child is fine, while a dead child
--- comes back as a closed channel.  Only the second means dead.
function Child:lua(code)
  if self.dead then
    return 'DEAD'
  end
  local ok, res = pcall(vim.rpcrequest, self.chan, 'nvim_exec_lua', code, {})
  if ok then
    return res
  end
  local text = errtext(res)
  if
    text:match('closed by the peer')
    or text:match('[Ii]nvalid channel')
    or text:match('channel closed')
  then
    self.dead = true
  end
  return 'RPCERR ' .. text
end

function Child:key(keys)
  if self.dead then
    return 'DEAD'
  end
  local ok, err = pcall(vim.rpcrequest, self.chan, 'nvim_input', keys)
  return ok and 'ok' or ('ERR ' .. errtext(err))
end

--- Bring the child back to a KNOWN state and prove that every key sent
--- so far has been consumed (b19-3, verbatim).
---
--- `nvim_input` is a FAST call that drops bytes straight into the input
--- buffer while `nvim_exec_lua` is DEFERRED to the main loop, so a
--- case's setup can run while the previous case's `<Esc>` is still in
--- the typeahead.  Polling `mode()` is not enough (a case that ended in
--- Normal mode leaves nothing to see), so the flush sends a NUMBERED
--- marker through the same FIFO the keys went through.  CTRL-C must NOT
--- be used in its place: it sets `got_int`, which interrupts the
--- `<Cmd>` and the marker never arrives.
function Child:flush()
  self.seq = (self.seq or 0) + 1
  for _ = 1, 200 do
    if self.dead then
      return false
    end
    -- One Esc is not always enough: a Visual selection started FROM
    -- INSERT answers Esc by going back to Insert.  Esc is also what
    -- breaks `pum_show_popupmenu`'s key loop, which answers no
    -- `nvim_exec_lua` at all while it is up.
    self:key('\27')
    self:key('<Cmd>let g:mnsync=' .. self.seq .. '<CR>')
    local v = self:lua('return {vim.g.mnsync, vim.fn.mode()}')
    if type(v) == 'table' and v[1] == self.seq and v[2] == 'n' then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- A KEYLESS barrier: one deferred request, answered.  The loop that
--- dispatches it takes the typeahead first, so an answer proves every
--- key sent before it has been executed.
function Child:barrier()
  return self:lua('return 1') == 1
end

--- One full turn of the child's state machine with no state change.
--- `startinsert` from a deferred event only sets `restart_edit`; Insert
--- begins when the loop next turns.
function Child:settle()
  self.tick = (self.tick or 0) + 1
  self:key('<Cmd>let g:mntick=' .. self.tick .. '<CR>')
  for _ = 1, 200 do
    if self.dead then
      return false
    end
    if self:lua('return vim.g.mntick') == self.tick then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- Wait for a deferred mode change.
function Child:want(mode)
  for _ = 1, 500 do
    if self.dead then
      return false
    end
    if self:lua('return vim.fn.mode()') == mode then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- `jobstop` alone is not enough: the child has to be reaped, or it
--- outlives this process holding $WORK as its cwd while the next run's
--- `rm -rf` races it (the cmdsweep s18 lesson).
function Child:stop()
  pcall(vim.fn.jobstop, self.chan)
  pcall(vim.fn.jobwait, { self.chan }, 10000)
end

--- A dying child's stderr, reduced to the part that is a FACT about the
--- editor rather than about this machine.  The source LINE goes too: it
--- is real information, but it moves whenever anything above it moves.
local function said(lines)
  local out = {}
  for _, line in ipairs(lines or {}) do
    if line:match('^stack backtrace:') or line:match('^%s*%d+:%s+0x') then
      break
    end
    if not line:match('^note: run with') and not line:match('^%s*$') then
      line = line:gsub('%((%d+)%)', '(<TID>)')
      line = line:gsub('0x%x+', '<ADDR>')
      line = line:gsub('(%.rs):%d+:%d+', '%1:<L>')
      line = line:gsub('/rustc/%w+/', '/rustc/<HASH>/')
      out[#out + 1] = scrub(line)
    end
  end
  return out
end

--- The child-side prelude: the same wipe/options/fixture the parent
--- uses, plus the one answer shape, so a case is one RPC roundtrip.
local CPRELUDE = [[
_G.MNFIX = ]] .. tolua(FIXTURE) .. [[

function _G.WIPE()
  for _, c in ipairs({'silent! aunmenu *', 'silent! aunmenu! *',
                      'silent! tlunmenu *', 'silent! tunmenu *',
                      'silent! menutranslate clear'}) do
    pcall(vim.cmd, c)
  end
end
function _G.RESET(cmds)
  pcall(vim.cmd, 'silent! stopinsert')
  if vim.fn.mode() ~= 'n' then pcall(vim.cmd, 'silent! normal! \27') end
  vim.o.lines = 24
  vim.o.columns = 80
  vim.o.report = 9999
  vim.o.more = false
  vim.o.shortmess = 'aoOtTIcCF'
  vim.o.selection = 'inclusive'
  vim.o.virtualedit = ''
  vim.o.mousemodel = 'popup_setpos'
  vim.o.swapfile = false
  _G.WIPE()
  local buf = vim.api.nvim_create_buf(true, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, _G.MNFIX)
  vim.api.nvim_win_set_buf(0, buf)
  vim.api.nvim_win_set_cursor(0, {1, 0})
  vim.g.mnout = ''
  vim.g.mnerr = ''
  vim.fn.setreg('"', '')
  for _, c in ipairs(cmds or {}) do
    local ok, err = pcall(vim.api.nvim_exec2, c, {output = false})
    if not ok then vim.g.mnerr = vim.g.mnerr .. '|' .. tostring(err) end
  end
  vim.cmd('redraw')
end

--- Run one command and keep BOTH halves for the parent to read later:
--- a case that ends inside a modal loop cannot answer an
--- `nvim_exec_lua` at all, so the answer has to be waiting in a
--- variable when the loop finally exits.
function _G.RUN(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, {output = true})
  if ok then vim.g.mnout = res.output or '' else vim.g.mnerr = tostring(res) end
  return ok
end

function _G.ANS(q)
  local r = {}
  local ok, m = pcall(vim.fn.mode, 1)
  r.md = ok and m or '?'
  r.cur = vim.api.nvim_win_get_cursor(0)
  local v = vim.fn.getpos('v')
  r.vs = {v[2], v[3]}
  r.buf = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  r.reg = vim.fn.getreg('"')
  r.g = {vim.g.mnhit, vim.g.mnout, vim.g.mnerr}
  if q then
    local f, e = load(q)
    if not f then r.q = 'LOADERR ' .. tostring(e) else
      local okq, res = pcall(f)
      if okq then r.q = res else r.q = 'ERR ' .. tostring(res) end
    end
  end
  return r
end
]]

local function child_new(args)
  local c = child_start(args)
  c:lua(CPRELUDE)
  return c
end

-- ==================================================================== s0
-- The runtime's own `PopUp` menu, recorded before anything wipes it.
--
-- `-u NONE` does not suppress it: `runtime/lua/vim/_defaults.lua`
-- defines fifteen entries and `ex_menu` copies each into five hidden
-- per-mode roots via `popup_mode_name`.  It is the only menu tree here
-- nobody wrote for a test, and the only exercise of the popup-copy path
-- against real input.  THIS SECTION IS COUPLED TO THE RUNTIME: an edit
-- to `_defaults.lua` re-baselines it and nothing else.
-- ====================================================================

section('s0-default', function()
  options()
  refixture()
  ex('s0/list/all', 'menu')
  ex('s0/list/popup', 'menu PopUp')
  ex('s0/list/nmode', 'nmenu')
  local names = {}
  for _, top in ipairs(fn('menu_get', '')) do
    local subs = {}
    for _, s in ipairs(top.submenus or {}) do
      local modes = {}
      for m in pairs(s.mappings or {}) do
        modes[#modes + 1] = m
      end
      table.sort(modes)
      subs[#subs + 1] = s.name .. '/' .. s.priority .. '/' .. table.concat(modes, '')
    end
    names[#names + 1] = { top.name, top.priority, top.hidden, #subs, subs }
  end
  ask('s0/tree/shape', names)
  ask('s0/info/inspect', fn('menu_info', 'PopUp.Inspect', 'n'))
  ask('s0/info/hiddenroot', fn('menu_info', 'PopUpn.Inspect', 'n'))
  ask('s0/complete/popup', fn('getcompletion', 'PopUp.', 'menu'))
end)

-- ==================================================================== s1
-- Every `:*menu` spelling.  `get_menu_cmd_modes` walks ONE character of
-- the command name and then decides `noremap` and `unmenu` from the
-- character it stopped on -- so `nnoremenu` and `nunmenu` differ from
-- `nmenu` only in what the *second* character is, and the `!` forms
-- change the default set to insert+cmdline.  The answer is the mode set
-- that came out, read back from `menu_info`.
-- ====================================================================

local ALLMODES = { 'n', 'v', 's', 'x', 'o', 'i', 'c', 't', 'tl', 'a', '' }

--- The mode set of `Probe.Item`, as a fixed-order string: one letter per
--- mode index that has an rhs, `-` where it does not.
local function modeset(path)
  local out = {}
  for _, m in ipairs({ 'n', 'v', 's', 'o', 'i', 'c', 't', 'tl' }) do
    local d = fn('menu_info', path, m)
    if type(d) == 'table' and d.rhs ~= nil then
      out[#out + 1] = m
    else
      out[#out + 1] = '-'
    end
  end
  return table.concat(out, ',')
end

section('s1-cmds', function()
  local CMDS = {
    'menu',
    'noremenu',
    'nmenu',
    'nnoremenu',
    'vmenu',
    'vnoremenu',
    'xmenu',
    'xnoremenu',
    'smenu',
    'snoremenu',
    'omenu',
    'onoremenu',
    'imenu',
    'inoremenu',
    'cmenu',
    'cnoremenu',
    'tlmenu',
    'tlnoremenu',
    'amenu',
    'anoremenu',
    'tmenu',
  }
  for _, cmd in ipairs(CMDS) do
    for _, bang in ipairs({ '', '!' }) do
      wipe()
      local label = 's1/def/' .. cmd .. (bang == '!' and '-bang' or '')
      local a =
        ex(label, cmd .. bang .. ' Probe.Item rhs-' .. cmd, { quiet = true })
      ask(label .. '/modes', {
        err = a.err,
        set = modeset('Probe.Item'),
        info = fn('menu_info', 'Probe.Item', 'a'),
      })
    end
  end

  -- The `un` and `nore` halves of the same walk: `unmenu` removes,
  -- `nunmenu` removes from Normal only, and `unmenu!` from insert +
  -- cmdline only.
  local UNS = {
    'unmenu',
    'unmenu!',
    'nunmenu',
    'vunmenu',
    'xunmenu',
    'sunmenu',
    'ounmenu',
    'iunmenu',
    'cunmenu',
    'tlunmenu',
    'tunmenu',
    'aunmenu',
    'aunmenu!',
  }
  for _, cmd in ipairs(UNS) do
    wipe()
    setup({
      'amenu Probe.Item rhs',
      'tlmenu Probe.Item tl-rhs',
      'tmenu Probe.Item tip',
    })
    local before = modeset('Probe.Item')
    local a = ex('s1/un/' .. cmd, cmd .. ' Probe.Item', { quiet = true })
    ask('s1/un/' .. cmd .. '/modes', {
      err = a.err,
      before = before,
      after = modeset('Probe.Item'),
    })
  end

  -- `noremap` is the character AFTER the mode letter, so `:nnoremenu`
  -- and `:nmenu` must differ in exactly one field of menu_info.
  wipe()
  setup({ 'nmenu Rm.Yes rhs', 'nnoremenu Rm.No rhs' })
  ask('s1/noremap', {
    yes = fn('menu_info', 'Rm.Yes', 'n'),
    no = fn('menu_info', 'Rm.No', 'n'),
  })

  -- `<script>`, `<silent>` and `<special>` in every order, plus the
  -- `<nop>` rhs, which `ex_menu` compares case-insensitively and turns
  -- into an EMPTY string rather than a mapping of five characters.
  wipe()
  local PRE = {
    { 'silent', '<silent>' },
    { 'script', '<script>' },
    { 'special', '<special>' },
    { 'silent-script', '<silent> <script>' },
    { 'script-silent', '<script> <silent>' },
    { 'all-three', '<script> <silent> <special>' },
    { 'nospace', '<silent><script>' },
  }
  for _, p in ipairs(PRE) do
    wipe()
    ex('s1/pre/' .. p[1], 'nmenu ' .. p[2] .. ' Pre.Item rhs', { quiet = true })
    ask('s1/pre/' .. p[1] .. '/info', fn('menu_info', 'Pre.Item', 'n'))
  end
  for _, nop in ipairs({ '<nop>', '<Nop>', '<NOP>', '<nop >' }) do
    wipe()
    ex('s1/nop/' .. nop, 'nmenu Nop.Item ' .. nop, { quiet = true })
    ask('s1/nop/' .. nop .. '/info', fn('menu_info', 'Nop.Item', 'n'))
  end

  -- `icon=` is parsed and thrown away on every UI nvim has; the escape
  -- handling inside it is still live code and it eats the argument.
  for _, icon in ipairs({
    'icon=foo.png Icon.Item rhs',
    'icon=with\\ space Icon.Item rhs',
    'icon=trailing Icon.Item rhs',
    'icon= Icon.Item rhs',
  }) do
    wipe()
    ex('s1/icon/' .. icon:sub(1, 14), 'nmenu ' .. icon, { quiet = true })
    ask('s1/icon/' .. icon:sub(1, 14) .. '/info', {
      item = fn('menu_info', 'Icon.Item', 'n'),
      tree = fn('menu_get', ''),
    })
  end
end)

-- ==================================================================== s2
-- Mode indices end to end: `MENU_INDEX_*`, `get_menu_mode_str` (the
-- letters the listing prints) and `menu_info`'s `modes` field, which is
-- a different alphabet again.
-- ====================================================================

section('s2-modes', function()
  local DEFS = {
    { 'normal', 'nmenu M.I n-rhs' },
    { 'visual', 'vmenu M.I v-rhs' },
    { 'xvisual', 'xmenu M.I x-rhs' },
    { 'select', 'smenu M.I s-rhs' },
    { 'oppend', 'omenu M.I o-rhs' },
    { 'insert', 'imenu M.I i-rhs' },
    { 'cmdline', 'cmenu M.I c-rhs' },
    { 'terminal', 'tlmenu M.I tl-rhs' },
    { 'all', 'amenu M.I a-rhs' },
    { 'default', 'menu M.I d-rhs' },
    { 'bang', 'menu! M.I b-rhs' },
  }
  for _, d in ipairs(DEFS) do
    wipe()
    setup({ d[2] })
    local per = {}
    for _, m in ipairs(ALLMODES) do
      per[m == '' and '<empty>' or m] = fn('menu_info', 'M.I', m)
    end
    ask('s2/def/' .. d[1], { set = modeset('M.I'), per = per })
    ex('s2/def/' .. d[1] .. '/list', 'menu M')
  end

  -- The same item defined in several modes at once: the listing groups
  -- by `get_menu_mode_str`, which has its own collapsing rules ("a" for
  -- all six, " " for the four non-insert/cmdline ones).
  local COMBOS = {
    { 'nvo', { 'nmenu C.I n', 'vmenu C.I v', 'omenu C.I o' } },
    { 'ic', { 'imenu C.I i', 'cmenu C.I c' } },
    { 'nvsoic', { 'amenu C.I a' } },
    { 'nvsoic-plus-tl', { 'amenu C.I a', 'tlmenu C.I tl' } },
    { 'sameall', { 'nmenu C.I x', 'vmenu C.I x', 'smenu C.I x', 'omenu C.I x' } },
    { 'differing', { 'nmenu C.I one', 'vmenu C.I two' } },
  }
  for _, c in ipairs(COMBOS) do
    wipe()
    setup(c[2])
    ex('s2/combo/' .. c[1], 'menu C')
    ask('s2/combo/' .. c[1] .. '/get', fn('menu_get', 'C'))
  end

  -- `menu_info` with a mode string nobody defined, an unknown letter,
  -- and the empty string (which means "the current mode").
  wipe()
  setup({ 'nmenu Q.I rhs' })
  for _, m in ipairs({ '', 'n', 'v', 'zz', '!', 'a', ' ', 'nv' }) do
    ask('s2/info/mode/' .. (m == '' and '<empty>' or esc(m)), fn('menu_info', 'Q.I', m))
  end
  ask('s2/info/noargs', fn('menu_info', 'Q.I'))
  ask('s2/info/missing', fn('menu_info', 'No.Such', 'n'))
  ask('s2/info/root', fn('menu_info', 'Q', 'n'))
end)

-- ==================================================================== s3
-- Priorities.  `ex_menu` parses a leading run of digits and dots as one
-- number per menu level, substitutes 500 for a zero, fills the rest of
-- MENUDEPTH with 500, and `add_menu_path` inserts by comparing them --
-- so the ORDER of the listing is the answer, not just the numbers.
-- ====================================================================

section('s3-priority', function()
  local PRIS = {
    { 'plain', 'menu P.A rhs' },
    { 'one', 'menu 10 P.B rhs' },
    { 'two', 'menu 80.5 P.C rhs' },
    { 'three', 'menu 10.20.30 P.D rhs' },
    { 'zero', 'menu 0 P.E rhs' },
    { 'zerodot', 'menu 0.0 P.F rhs' },
    { 'leadingdot', 'menu .5 P.G rhs' },
    { 'trailingdot', 'menu 5. P.H rhs' },
    { 'huge', 'menu 2147483647 P.I rhs' },
    { 'overflow', 'menu 99999999999999999999 P.J rhs' },
    { 'many', 'menu 1.2.3.4.5.6.7.8.9.10.11.12 P.K rhs' },
    { 'dotsonly', 'menu ... P.L rhs' },
  }
  for _, p in ipairs(PRIS) do
    wipe()
    local a = ex('s3/parse/' .. p[1], p[2], { quiet = true })
    ask('s3/parse/' .. p[1] .. '/tree', { err = a.err, tree = fn('menu_get', '') })
  end

  -- An address count instead of a priority: `:1menu` takes `line2` as
  -- the top-level priority, but only when no digits lead the argument.
  for _, a in ipairs({ '1menu X.A rhs', '99menu X.B rhs', '0menu X.C rhs', '1,5menu X.D rhs' }) do
    wipe()
    local r = ex('s3/addr/' .. a:match('^[%d,]*'), a, { quiet = true })
    ask('s3/addr/' .. a:match('^[%d,]*') .. '/tree', { err = r.err, tree = fn('menu_get', '') })
  end

  -- Ordering.  Six tops and six children defined out of order; the
  -- listing is the sorted answer, and equal priorities keep insertion
  -- order.
  wipe()
  setup({
    'menu 50 Ord.Z z',
    'menu 20 Ord.M m',
    'menu 20 Ord.N n',
    'menu 10 Ord.A a',
    'menu 900 Ord.Last l',
    'menu 30.10 Beta.One one',
    'menu 30.20 Beta.Two two',
    'menu 5 Alpha.Only only',
    'menu 30.5 Beta.Zero zero',
  })
  ex('s3/order/list', 'menu')
  ask('s3/order/get', fn('menu_get', ''))

  -- The same path re-defined at a different priority: the priority of an
  -- EXISTING node is not updated, which is a rule a rewrite loses for
  -- free.
  wipe()
  setup({ 'menu 10 Re.Item first' })
  ex('s3/repri/again', 'menu 900 Re.Item second', { quiet = true })
  ask('s3/repri/tree', fn('menu_get', ''))
end)

-- ==================================================================== s4
-- Names: `menu_name_skip` (the `\` and CTRL-V escapes and the `.`
-- split), `menu_translate_tab_and_shift`, `menu_text` (the `&` mnemonic
-- and the `<Tab>`/`^I` accelerator split), `menu_is_separator` /
-- `_hidden` / `_popup` / `_toolbar` / `_menubar` / `_winbar`.
-- ====================================================================

section('s4-names', function()
  local NAMES = {
    { 'plain', 'Foo.Bar' },
    { 'escdot', 'Foo\\.Bar' },
    { 'escspace', 'Foo.Bar\\ Baz' },
    { 'escbslash', 'Foo.Bar\\\\Baz' },
    { 'mnemonic', '&Foo.&Bar' },
    { 'ampamp', 'F&&oo.Bar' },
    { 'trailamp', 'Foo&.Bar' },
    { 'accel-tab', 'Foo.Bar<Tab>CTRL-B' },
    { 'accel-lit', 'Foo.Bar\\	CTRL-B' },
    { 'accel-both', 'Foo.&Bar<Tab>^X' },
    { 'accel-empty', 'Foo.Bar<Tab>' },
    { 'accel-twice', 'Foo.Bar<Tab>A<Tab>B' },
    { 'sep-dash', 'Foo.-sep-' },
    { 'sep-bare', 'Foo.-' },
    { 'sep-longer', 'Foo.-SEP1-' },
    { 'sep-notsep', 'Foo.-sep' },
    { 'hidden-root', ']Hidden.Item' },
    { 'hidden-child', 'Foo.]Child' },
    { 'popup-root', 'PopUp.Item' },
    { 'popup-sub', 'PopUp.Sub.Item' },
    { 'toolbar', 'ToolBar.Item' },
    { 'winbar', 'WinBar.Item' },
    { 'unicode', 'Ｆｉｌｅ.Ｏｐｅｎ' },
    { 'combining', 'Cafe\u{0301}.Item' },
    { 'deep', 'A.B.C.D.E.F' },
    { 'digits', '1234.5678' },
    { 'dotdot', 'Foo..Bar' },
    { 'ctrlv', 'Foo.Bar\u{0016}.Baz' },
    { 'space-in-path', 'Foo Bar.Baz' },
    { 'longname', string.rep('N', 200) .. '.Item' },
  }
  for _, n in ipairs(NAMES) do
    wipe()
    local a = ex('s4/def/' .. n[1], 'amenu ' .. n[2] .. ' the-rhs', { quiet = true })
    ask('s4/def/' .. n[1] .. '/tree', { err = a.err, tree = fn('menu_get', '') })
    ex('s4/def/' .. n[1] .. '/list', 'menu')
  end

  -- The same names through `menu_info`, whose `name` is the raw name and
  -- whose `display` is `menu_text`'s output -- the two differ exactly
  -- where the mnemonic and the accelerator are.
  wipe()
  setup({
    'amenu N.&Open\\tCTRL-O one',
    'amenu N.Pl&ain two',
    'amenu N.-sep- three',
    'amenu N.Tab<Tab>Acc four',
  })
  for _, p in ipairs({ 'N.Open', 'N.Plain', 'N.-sep-', 'N.Tab', 'N.&Open', 'N' }) do
    ask('s4/info/' .. p, fn('menu_info', p, 'a'))
  end
  ex('s4/info/list', 'menu N')
  ask('s4/info/get', fn('menu_get', 'N'))

  -- A separator carries no rhs and `menu_is_separator` is what stops it
  -- from being executed; defining one WITH an rhs is the interesting
  -- case (`add_menu_path` clears it).
  wipe()
  local a = ex('s4/sep/withrhs', 'amenu S.-sep- :echo "no"<CR>', { quiet = true })
  ask('s4/sep/withrhs/info', { err = a.err, info = fn('menu_info', 'S.-sep-', 'a') })
  ex('s4/sep/mid', 'amenu S.-sep-.Child rhs', { quiet = true })
  ask('s4/sep/mid/tree', fn('menu_get', ''))
end)

-- ==================================================================== s5
-- `show_menus` / `show_menus_recursive`: the listing text itself, which
-- is the only reader of `get_menu_mode_str` and the `*` / `&` / `-`
-- markers.
-- ====================================================================

section('s5-listing', function()
  tree()
  for _, c in ipairs({
    'menu',
    'menu!',
    'nmenu',
    'vmenu',
    'xmenu',
    'smenu',
    'omenu',
    'imenu',
    'cmenu',
    'tlmenu',
    'tmenu',
    'amenu',
    'noremenu',
    'anoremenu',
    'menu File',
    'menu File.Open',
    'menu Deep',
    'menu Deep.Sub',
    'menu Deep.Sub.Leaf',
    'menu ]Hidden',
    'menu PopUp',
    'menu ToolBar',
    'menu WinBar',
    'imenu Edit',
    'vmenu Edit',
    'nmenu Edit',
    'menu Tools',
    'menu 500',
    'menu enable',
    'menu disable',
  }) do
    ex('s5/list/' .. c:gsub('[ .!]', '-'), c)
  end

  -- Listing an empty tree and a missing path.
  wipe()
  ex('s5/empty/all', 'menu')
  ex('s5/empty/named', 'menu Nope')
  ex('s5/empty/nmode', 'nmenu')
  setup({ 'imenu Only.Ins rhs' })
  ex('s5/empty/wrongmode', 'nmenu Only')
  ex('s5/empty/rightmode', 'imenu Only')

  -- A rhs long enough to be truncated, one with control characters, and
  -- one with a multibyte body: the listing escapes them itself.
  wipe()
  setup({
    'nmenu L.Long :' .. string.rep('x', 300) .. '<CR>',
    'nmenu L.Ctrl <C-A><C-B><Esc>',
    'nmenu L.Multi :echo "\u{00e9}\u{4e2d}"<CR>',
    'nmenu <silent> L.Silent :echo 1<CR>',
    'nmenu <script> L.Script :echo 1<CR>',
    'nnoremenu L.Nore :echo 1<CR>',
  })
  ex('s5/rhs/list', 'menu L')
  ask('s5/rhs/get', fn('menu_get', 'L'))

  -- `:menu` while the listing is running is E1309; the guard is
  -- `menus_locked`, which only a `<Cmd>` from inside the pager can trip.
  -- The reachable half is the message it prints.
  ask('s5/locked/probe', fn('execute', 'silent! menu'))
end)

-- ==================================================================== s6
-- `menu_get` / `menuitem_getinfo`.  The tree walk is recursive and the
-- dict has ten keys; both are pure formatting of the same nodes the
-- listing prints, and they disagree in two places on purpose.
-- ====================================================================

section('s6-get', function()
  tree()
  for _, p in ipairs({ '', 'File', 'File.Open', 'Deep', 'Deep.Sub', 'Edit', 'PopUp', 'ToolBar', 'WinBar', ']Hidden', 'Nope', 'File.Nope' }) do
    for _, m in ipairs({ '', 'n', 'i', 'v', 'a' }) do
      ask(
        's6/get/' .. (p == '' and '<root>' or p) .. '/' .. (m == '' and '<empty>' or m),
        fn('menu_get', p, m)
      )
    end
  end
  for _, p in ipairs({
    'File.Open',
    'File.Save',
    'File.-sep1-',
    'File.Quit',
    'File',
    'Edit.OnlyInsert',
    'Edit.OnlyVisual',
    'Tools.Quiet',
    'Tools.Scripted',
    'Deep.Sub.Leaf',
    ']Hidden.Item',
    'PopUp.Mine',
    'ToolBar.Save',
    'WinBar.Bar',
  }) do
    for _, m in ipairs({ 'a', 'n', 'i', 't' }) do
      ask('s6/info/' .. p .. '/' .. m, fn('menu_info', p, m))
    end
  end
  -- Arguments menu_info rejects, and the ones it merely answers empty.
  for _, bad in ipairs({ '', '.', '..', 'File.', '.File', 'File..Open' }) do
    ask('s6/info/bad/' .. (bad == '' and '<empty>' or esc(bad)), fn('menu_info', bad, 'a'))
  end
end)

-- ==================================================================== s7
-- `remove_menu`: per-mode removal (a node survives while any mode is
-- left), the sub-menu rules (E330 / E336 / E333), and the `PopUp`
-- copies, which `ex_menu` removes through `popup_mode_name` in a second
-- pass nobody else runs.
-- ====================================================================

section('s7-unmenu', function()
  local CASES = {
    { 'leaf', { 'amenu U.Item rhs' }, 'aunmenu U.Item' },
    { 'onemode', { 'amenu U.Item rhs' }, 'nunmenu U.Item' },
    { 'lastmode', { 'nmenu U.Item rhs' }, 'nunmenu U.Item' },
    { 'wrongmode', { 'nmenu U.Item rhs' }, 'iunmenu U.Item' },
    { 'submenu', { 'amenu U.Sub.Item rhs' }, 'aunmenu U.Sub' },
    { 'root', { 'amenu U.Sub.Item rhs' }, 'aunmenu U' },
    { 'star', { 'amenu U.Item rhs', 'amenu V.Item rhs' }, 'aunmenu *' },
    { 'star-bang', { 'amenu U.Item rhs' }, 'unmenu! *' },
    { 'missing', { 'amenu U.Item rhs' }, 'aunmenu No.Such' },
    { 'trailing', { 'amenu U.Item rhs' }, 'aunmenu U.Item extra' },
    { 'deep-mid', { 'amenu U.A.B.C rhs' }, 'aunmenu U.A.B' },
    { 'deep-leaf', { 'amenu U.A.B.C rhs' }, 'aunmenu U.A.B.C' },
    { 'sep', { 'amenu U.-sep- x', 'amenu U.Item rhs' }, 'aunmenu U.-sep-' },
    { 'popup', { 'amenu PopUp.Item rhs' }, 'aunmenu PopUp.Item' },
    { 'popup-root', { 'amenu PopUp.Item rhs' }, 'aunmenu PopUp' },
    { 'popup-onemode', { 'amenu PopUp.Item rhs' }, 'nunmenu PopUp.Item' },
    { 'tip', { 'amenu U.Item rhs', 'tmenu U.Item a tip' }, 'aunmenu U.Item' },
    { 'tip-only', { 'amenu U.Item rhs', 'tmenu U.Item a tip' }, 'tunmenu U.Item' },
    { 'toolbar', { 'amenu ToolBar.Item rhs' }, 'aunmenu ToolBar.Item' },
    { 'winbar', { 'amenu WinBar.Item rhs' }, 'aunmenu WinBar.Item' },
    { 'hidden', { 'amenu ]Hid.Item rhs' }, 'aunmenu ]Hid.Item' },
    { 'empty-arg', { 'amenu U.Item rhs' }, 'aunmenu' },
  }
  for _, c in ipairs(CASES) do
    wipe()
    setup(c[2])
    local before = fn('menu_get', '')
    local a = ex('s7/' .. c[1], c[3], { quiet = true })
    ask('s7/' .. c[1] .. '/tree', {
      err = a.err,
      out = a.out ~= '' and a.out or nil,
      before_n = #before,
      after = fn('menu_get', ''),
    })
  end

  -- Removing one mode at a time from a six-mode item: the node must
  -- survive five removals and vanish on the sixth.
  wipe()
  setup({ 'amenu W.Item rhs' })
  for _, m in ipairs({ 'n', 'v', 's', 'o', 'i', 'c' }) do
    ex('s7/peel/' .. m, m .. 'unmenu W.Item', { quiet = true })
    ask('s7/peel/' .. m .. '/state', {
      set = modeset('W.Item'),
      tree = fn('menu_get', ''),
    })
  end
end)

-- ==================================================================== s8
-- `menu_enable_recurse`.  `enable`/`disable` walk the same path parser
-- as a definition, apply to a mode SET, recurse into children, and have
-- their own `*` spelling.
-- ====================================================================

section('s8-enable', function()
  local CASES = {
    { 'disable-leaf', 'menu disable E.Item' },
    { 'disable-sub', 'menu disable E.Sub' },
    { 'disable-root', 'menu disable E' },
    { 'disable-star', 'menu disable *' },
    { 'disable-nmode', 'nmenu disable E.Item' },
    { 'disable-imode', 'imenu disable E.Item' },
    { 'disable-bang', 'menu! disable E.Item' },
    { 'disable-popup', 'menu disable PopUp.Item' },
    { 'disable-missing', 'menu disable No.Such' },
    { 'disable-trailing', 'menu disable E.Item extra' },
    { 'enable-after', 'menu enable E.Item' },
    { 'enable-star', 'menu enable *' },
    { 'enable-sub', 'menu enable E.Sub' },
  }
  for _, c in ipairs(CASES) do
    wipe()
    setup({
      'amenu E.Item rhs',
      'amenu E.Sub.Leaf rhs',
      'amenu PopUp.Item rhs',
      'amenu Other.Item rhs',
    })
    if c[1]:match('^enable') then
      -- `menu disable *` is NOT a way to disable everything: the `*` path
      -- is the empty one, so `menu_enable_recurse` clears the flag on
      -- every TOP-LEVEL node and recurses into none of them. Setting up
      -- an enable case with it leaves every leaf enabled and the
      -- re-enable has nothing to restore -- a hole that let a
      -- `enabled |= modes` -> `|= 0` mutant through.
      setup({
        'menu disable E.Item',
        'menu disable E.Sub.Leaf',
        'menu disable PopUp.Item',
        'menu disable Other.Item',
        'imenu disable E.Item',
        'cmenu disable E.Item',
      })
    end
    local a = ex('s8/' .. c[1], c[2], { quiet = true })
    local state = {}
    -- THE INTERMEDIATE NODES ARE THE POINT.  `menu_enable_recurse` sets
    -- `enabled` on the node the path NAMES and does not touch its
    -- children, so `:menu disable E` and `:menu disable *` (which is the
    -- empty path, i.e. every top-level node) change nothing a leaf can
    -- show -- the first draft read only leaves and recorded four
    -- all-`true` rows that looked like the command had been ignored.
    for _, p in ipairs({
      'E',
      'E.Item',
      'E.Sub',
      'E.Sub.Leaf',
      'PopUp',
      'PopUp.Item',
      'Other',
      'Other.Item',
    }) do
      local per = {}
      for _, m in ipairs({ 'n', 'v', 'i', 'c' }) do
        local d = fn('menu_info', p, m)
        per[m] = type(d) == 'table' and d.enabled
      end
      state[p] = per
    end
    ask('s8/' .. c[1] .. '/state', { err = a.err, out = a.out ~= '' and a.out or nil, en = state })
    ex('s8/' .. c[1] .. '/list', 'menu')
  end

  -- A disabled item still lists, still completes and still executes
  -- through `:emenu` -- `enabled` is advisory outside a real UI, and a
  -- rewrite that starts enforcing it changes three sections at once.
  wipe()
  setup({ 'nmenu D.Item :let g:mnhit="ran"<CR>', 'menu disable D.Item' })
  ex('s8/disabled/list', 'menu D')
  ask('s8/disabled/complete', fn('getcompletion', 'D.', 'menu'))
  vim.g.mnhit = nil
  local a = ex('s8/disabled/emenu', 'emenu D.Item', { quiet = true })
  ask('s8/disabled/emenu/hit', { err = a.err, hit = vim.g.mnhit })
end)

-- ==================================================================== s9
-- Completion.  `set_context_in_menu_cmd` decides the context and the
-- mode set; `get_menu_name` (menu NAMES, submenus only) and
-- `get_menu_names` (names AND leaves, with a CTRL-A sentinel on a
-- submenu) generate the candidates.  Both keep static state across
-- calls -- `should_advance` alternates so that a translated menu offers
-- its English name second -- so the ORDER is part of the answer.
-- ====================================================================

section('s9-complete', function()
  tree()
  for _, p in ipairs({ '', 'F', 'File', 'File.', 'File.O', 'Deep.', 'Deep.Sub.', 'Edit.', 'PopUp.', ']', ']Hidden.', 'Nope.', 'Tools.', 'ToolBar.' }) do
    ask('s9/menu/' .. (p == '' and '<empty>' or p), fn('getcompletion', p, 'menu'))
  end
  local LINES = {
    'emenu ',
    'emenu F',
    'emenu File.',
    'emenu Deep.Sub.',
    'emenu! ',
    'popup ',
    'popup F',
    'popup File.',
    'menu ',
    'menu F',
    'menu File.',
    'menu enable ',
    'menu disable F',
    'menu 10.20 F',
    'unmenu ',
    'unmenu F',
    'unmenu File.',
    'aunmenu ',
    'nmenu ',
    'imenu ',
    'imenu E',
    'imenu Edit.',
    'vmenu Edit.',
    'tmenu F',
    'tlmenu ',
    'amenu ',
    'anoremenu F',
    'menutranslate ',
    'menutranslate F',
    'emenu File.Open ',
    'menu File.Open rhs',
  }
  for _, line in ipairs(LINES) do
    ask('s9/cmdline/' .. line:gsub('[ .!]', '-'), fn('getcompletion', line, 'cmdline'))
  end

  -- `expand_modes` is what makes a mode-specific completion skip the
  -- items that mode has no rhs for -- and `get_menu_name` answers an
  -- EMPTY STRING rather than skipping, which is visible as a blank
  -- candidate.
  wipe()
  setup({ 'imenu Only.Ins rhs', 'nmenu Both.N rhs', 'imenu Both.I rhs' })
  for _, line in ipairs({ 'imenu ', 'nmenu ', 'imenu Both.', 'nmenu Both.', 'emenu ', 'emenu Both.' }) do
    ask('s9/modes/' .. line:gsub('[ .]', '-'), fn('getcompletion', line, 'cmdline'))
  end

  -- A separator and a hidden root: `get_menu_name` skips both, and
  -- `get_menu_names` skips a separator only when the command is
  -- `:emenu`.
  wipe()
  setup({ 'amenu H.A rhs', 'amenu H.-sep- x', 'amenu ]Hid.B rhs', 'amenu H.Sub.C rhs' })
  for _, line in ipairs({ 'emenu H.', 'popup H.', 'menu H.', 'emenu ', 'popup ', 'menu ' }) do
    ask('s9/skip/' .. line:gsub('[ .]', '-'), fn('getcompletion', line, 'cmdline'))
  end
  ask('s9/skip/menu-fn', fn('getcompletion', 'H.', 'menu'))
end)

-- =================================================================== s10
-- `ex_menutranslate` / `menutrans_lookup` / `menu_skip_part`.  A
-- translation renames a menu AT DEFINITION TIME and keeps the English
-- name in `en_name`/`en_dname`, which is what makes completion offer
-- both spellings.
-- ====================================================================

section('s10-translate', function()
  local CASES = {
    { 'simple', { 'menutranslate File Fichier' }, 'menu File.Open :e<CR>' },
    { 'both-parts', { 'menutranslate File Fichier', 'menutranslate Open Ouvrir' }, 'menu File.Open :e<CR>' },
    { 'with-accel', { 'menutranslate &File &Fichier' }, 'menu &File.&Open :e<CR>' },
    { 'accel-tab', { 'menutranslate Open<Tab>CTRL-O Ouvrir<Tab>CTRL-O' }, 'menu File.Open<Tab>CTRL-O :e<CR>' },
    { 'escaped-space', { 'menutranslate Save\\ As Enregistrer\\ sous' }, 'menu File.Save\\ As :w<CR>' },
    { 'escaped-dot', { 'menutranslate A\\.B C\\.D' }, 'menu Top.A\\.B :e<CR>' },
    { 'unicode', { 'menutranslate File \u{30d5}\u{30a1}\u{30a4}\u{30eb}' }, 'menu File.Open :e<CR>' },
    { 'missing-to', { 'menutranslate File' }, 'menu File.Open :e<CR>' },
    { 'empty', { 'menutranslate' }, 'menu File.Open :e<CR>' },
    { 'clear-first', { 'menutranslate clear' }, 'menu File.Open :e<CR>' },
    { 'twice-same', { 'menutranslate File Un', 'menutranslate File Deux' }, 'menu File.Open :e<CR>' },
    { 'sep-translate', { 'menutranslate -sep- -tr-' }, 'menu File.-sep- x' },
  }
  for _, c in ipairs(CASES) do
    wipe()
    for _, t in ipairs(c[2]) do
      ex('s10/' .. c[1] .. '/tr', t, { quiet = true })
    end
    local a = ex('s10/' .. c[1] .. '/def', c[3], { quiet = true })
    ex('s10/' .. c[1] .. '/list', 'menu')
    ask('s10/' .. c[1] .. '/get', { err = a.err, tree = fn('menu_get', '') })
  end

  -- The English name survives: completion offers both, `menu_info`
  -- answers to the translated path, and `:aunmenu` needs the translated
  -- one.
  wipe()
  setup({ 'menutranslate File Fichier', 'menutranslate Open Ouvrir', 'amenu File.Open :e<CR>' })
  ask('s10/en/info-tr', fn('menu_info', 'Fichier.Ouvrir', 'a'))
  ask('s10/en/info-en', fn('menu_info', 'File.Open', 'a'))
  ask('s10/en/complete-root', fn('getcompletion', 'emenu ', 'cmdline'))
  ask('s10/en/complete-sub', fn('getcompletion', 'emenu Fichier.', 'cmdline'))
  ask('s10/en/complete-menu', fn('getcompletion', '', 'menu'))
  ex('s10/en/unmenu-en', 'aunmenu File.Open', { quiet = true })
  ask('s10/en/after-en', fn('menu_get', ''))
  ex('s10/en/unmenu-tr', 'aunmenu Fichier.Ouvrir', { quiet = true })
  ask('s10/en/after-tr', fn('menu_get', ''))

  -- `menutranslate clear` after the menus are defined: the names
  -- already applied do not change back.
  wipe()
  setup({ 'menutranslate File Fichier', 'amenu File.Open :e<CR>' })
  ex('s10/clear/before', 'menu')
  ex('s10/clear/do', 'menutranslate clear', { quiet = true })
  ex('s10/clear/after', 'menu')
  ex('s10/clear/redef', 'amenu File.Save :w<CR>', { quiet = true })
  ex('s10/clear/list', 'menu')
  ask('s10/clear/tree', fn('menu_get', ''))

  -- Argument shapes `ex_menutranslate` rejects.
  for _, bad in ipairs({ 'menutranslate a b c', 'menutranslate clear extra', 'menutranslate  ' }) do
    wipe()
    ex('s10/bad/' .. bad:gsub('[ ]', '-'), bad, { quiet = true })
  end
end)

-- =================================================================== s11
-- `:emenu` -- `ex_emenu` + `menu_getbyname` + `execute_menu`.
--
-- The mode index is chosen from `State`, `restart_edit`,
-- `VIsual_active` and the command's address count, so the mode has to
-- be REAL.  An `-l` script is never in Insert or Visual, which is why
-- this section is the first of two that need an `--embed` child.
-- ====================================================================

section('s11-emenu', function()
  -- The Normal-mode half runs here, where the answer is one roundtrip.
  wipe()
  refixture()
  setup({
    'nmenu X.Norm :let g:mnhit="n"<CR>',
    'imenu X.Ins <Esc>:let g:mnhit="i"<CR>',
    'vmenu X.Vis :<C-U>let g:mnhit="v"<CR>',
    'cmenu X.Cmd cmd-text',
    'omenu X.Op :let g:mnhit="o"<CR>',
    'nmenu X.Edit :s/alpha/ALPHA/<CR>',
    'nmenu <silent> X.Silent :let g:mnhit="silent"<CR>',
    'nmenu X.Move 3G',
    'amenu X.All :let g:mnhit="a"<CR>',
    'amenu X.Sub.Leaf :let g:mnhit="leaf"<CR>',
    'amenu X.-sep- x',
    'tmenu X.Norm a tip',
  })
  local BASIC = {
    { 'normal', 'emenu X.Norm' },
    { 'bang', 'emenu! X.Cmd' },
    { 'all', 'emenu X.All' },
    { 'leaf', 'emenu X.Sub.Leaf' },
    { 'silent', 'emenu X.Silent' },
    { 'edit', 'emenu X.Edit' },
    { 'move', 'emenu X.Move' },
    { 'submenu', 'emenu X.Sub' },
    { 'separator', 'emenu X.-sep-' },
    { 'missing', 'emenu No.Such' },
    { 'wrongmode', 'emenu X.Ins' },
    { 'root', 'emenu X' },
    { 'empty', 'emenu' },
    { 'trailing', 'emenu X.Norm extra' },
    { 'range', '1,3emenu X.Norm' },
    { 'range-vis', '2,4emenu X.Vis' },
  }
  for _, c in ipairs(BASIC) do
    vim.g.mnhit = nil
    refixture()
    local a = ex('s11/n/' .. c[1], c[2], { quiet = true })
    ask('s11/n/' .. c[1] .. '/state', {
      err = a.err,
      out = a.out ~= '' and a.out or nil,
      hit = vim.g.mnhit,
      cur = vim.api.nvim_win_get_cursor(0),
      buf = vim.api.nvim_buf_get_lines(0, 0, -1, false),
      mode = vim.fn.mode(1),
    })
  end

  -- THE MODE INDEX IS NOT REACHABLE FROM LUA AT ALL, and that costs a
  -- whole section if it is missed.  `execute_menu`'s Insert arm is
  -- `(State & MODE_INSERT || restart_edit) && current_sctx.sc_sid == 0`
  -- -- so an `:emenu` issued through `nvim_exec2` from a Lua callback,
  -- which is what a `v:lua` wrapper is, carries a non-zero script id and
  -- falls through to NORMAL even while the editor really is in Insert.
  -- The command therefore goes in as a TYPED `<Cmd>` (script id 0), and
  -- s11/sid records the difference rather than hiding it.
  --
  -- The mode itself is entered with `nvim_input`, because the case has
  -- to be IN the mode when the command runs.
  local DEFS = {
    'nmenu M.It :let g:mnhit="n"<CR>',
    'imenu M.It <C-R>=execute("let g:mnhit=\'i\'")<CR>',
    'vmenu M.It :<C-U>let g:mnhit="v"<CR>',
    'smenu M.It <C-O>:let g:mnhit="s"<CR>',
    'omenu M.It :<C-U>let g:mnhit="o"<CR>',
    'cmenu M.It cmdtext',
    'tlmenu M.It tltext',
    'nmenu M.Only :let g:mnhit="only-n"<CR>',
    'imenu M.Ionly ins-only',
  }
  local c = child_new()
  local MODES = {
    { 'normal', '', 'n' },
    { 'insert', 'i', 'i' },
    { 'visual', 'vjl', 'v' },
    { 'vline', 'Vj', 'V' },
    { 'vblock', '\22jl', '\22' },
    { 'select', 'vjl\15', 's' },
    { 'cmdline', ':', 'c' },
    { 'replace', 'R', 'R' },
  }
  for _, m in ipairs(MODES) do
    for _, path in ipairs({ 'M.It', 'M.Only', 'M.Ionly' }) do
      local label = 's11/mode/' .. m[1] .. '/' .. path:gsub('%.', '-')
      label_once(label)
      c:flush()
      c:lua('RESET(' .. tolua(DEFS) .. ') vim.g.mnhit = nil')
      c:settle()
      if m[2] ~= '' then
        c:key(m[2])
        if not c:want(m[3]) then
          c:key('\27')
          c:key(m[2])
          c:want(m[3])
        end
      end
      -- `<Cmd>` runs the command WITHOUT leaving the mode, which is the
      -- whole point: `:emenu` from a `<Cmd>` mapping sees Insert or
      -- Visual in `State`.  A cmdline-mode case cannot use `<Cmd>` (it
      -- is not recognised there), so it types the command instead.
      local body = 'try|emenu ' .. path .. '|catch|let g:mnerr=v:exception|endtry'
      if m[1] == 'cmdline' then
        c:key(body .. '\r')
      else
        c:key('<Cmd>' .. body .. '<CR>')
      end
      vim.wait(5)
      c:barrier()
      local ans = c:lua('return ANS()')
      emit(label, '=', esc(scrub(ins(ans))))
      struct(label, ans)
    end
  end

  -- The script-id rule itself, measured both ways in the same mode.
  -- `viaCmd` reaches the Insert arm; `viaLua` cannot, and answers the
  -- Normal rhs (or E335) instead.
  for _, how in ipairs({ 'viaCmd', 'viaLua' }) do
    local label = 's11/sid/' .. how
    label_once(label)
    c:flush()
    c:lua('RESET(' .. tolua(DEFS) .. ') vim.g.mnhit = nil')
    c:settle()
    c:key('i')
    c:want('i')
    if how == 'viaCmd' then
      c:key('<Cmd>try|emenu M.It|catch|let g:mnerr=v:exception|endtry<CR>')
    else
      c:key('<Cmd>call v:lua.RUN("emenu M.It")<CR>')
    end
    vim.wait(5)
    c:barrier()
    local ans = c:lua('return ANS()')
    emit(label, '=', esc(scrub(ins(ans))))
    struct(label, ans)
  end

  -- `:emenu` with a range while a Visual selection exists: the address
  -- count arm rebuilds the selection from `b_visual` when the lines
  -- match, and from `line1`/`line2` linewise when they do not.
  for _, r in ipairs({ '1,2', '2,4', '3,3', '1,5' }) do
    local label = 's11/range/' .. r:gsub(',', '-')
    label_once(label)
    c:flush()
    c:lua('RESET(' .. tolua(DEFS) .. ') vim.g.mnhit = nil')
    c:settle()
    c:key('2Gvjl\27')
    c:barrier()
    c:key('<Cmd>try|' .. r .. 'emenu M.It|catch|let g:mnerr=v:exception|endtry<CR>')
    vim.wait(5)
    c:barrier()
    local ans = c:lua('return ANS("return {vim.fn.getpos(\\"\'<\\"), vim.fn.getpos(\\"\'>\\")}")')
    emit(label, '=', esc(scrub(ins(ans))))
    struct(label, ans)
  end
  c:stop()
end)

-- =================================================================== s12
-- `:popup` -- `ex_emenu`'s sibling through `show_popupmenu`.
--
-- THE ERROR PATHS RETURN; THE SUCCESS PATH DOES NOT.  On a menu that
-- exists, `pum_show_popupmenu` runs a bare `vgetc()` loop with no
-- `K_EVENT` arm, so the whole event loop -- timers, jobs,
-- `vim.schedule`, RPC -- stops until a key arrives.  That is an upstream
-- bug (filed at B19-3); here it is a constraint: the error cases run in
-- this process, the success cases run in a child driven by `nvim_input`
-- (a FAST call, dispatched from inside the loop) and escaped with
-- `<Esc>`.
-- ====================================================================

section('s12-popup', function()
  wipe()
  vim.g.mnhit = nil
  setup({ 'nmenu P.Item :let g:mnhit="p"<CR>', 'imenu P.Ins x' })
  for _, c in ipairs({
    { 'missing', 'popup No.Such' },
    { 'leaf', 'popup P.Item' },
    { 'empty', 'popup' },
    { 'root-nomenu', 'popup Nope' },
    { 'bang-missing', 'popup! No.Such' },
  }) do
    ex('s12/err/' .. c[1], c[2], { quiet = true })
  end
  ask('s12/err/state', { hit = vim.g.mnhit })

  -- The success path, in a child.  Each case opens the menu, proves the
  -- loop is up (a KEYLESS barrier must TIME OUT), then escapes.
  local DEFS = {
    'nmenu Pop.One :let g:mnhit="one"<CR>',
    'nmenu Pop.Two :let g:mnhit="two"<CR>',
    'nmenu Pop.-sep- x',
    'nmenu Pop.Three :let g:mnhit="three"<CR>',
    'amenu Pop.All <Cmd>let g:mnhit="all"<CR>',
    'nmenu Empty.Sub.Deep :let g:mnhit="deep"<CR>',
  }
  local c = child_new()
  -- `show_popupmenu` picks the mode index with `get_menu_mode()`, which
  -- is a different function from the one `:emenu` uses, so the mode is
  -- part of the question here too.
  local CASES = {
    { 'escape', 'popup Pop', '\27' },
    { 'pick-first', 'popup Pop', '\r' },
    { 'pick-second', 'popup Pop', 'j\r' },
    { 'pick-past-sep', 'popup Pop', 'jjj\r' },
    { 'nested', 'popup Empty', '\27' },
    { 'bang', 'popup! Pop', '\27' },
    { 'insert', 'popup Pop', 'j\r', 'i', 'i' },
    { 'visual', 'popup Pop', 'j\r', 'vjl', 'v' },
  }
  for _, k in ipairs(CASES) do
    local label = 's12/pop/' .. k[1]
    label_once(label)
    c:flush()
    c:lua('RESET(' .. tolua(DEFS) .. ') vim.g.mnhit = nil')
    c:settle()
    if k[4] then
      c:key(k[4])
      c:want(k[5])
    end
    -- A ONE-SHOT timer that records `pumvisible()` when it fires, armed
    -- long enough after the command that the menu is certainly up.
    --
    -- It is the only way to ask "was the event loop running" without
    -- asking the child anything, and an `nvim_exec_lua` issued while the
    -- menu is up is not a failed call, it is a call that NEVER RETURNS
    -- -- the first draft of this section did exactly that and the sweep
    -- never finished.  A COUNTING timer does not work either: whatever
    -- it counted while frozen, it goes on counting the moment `<Esc>`
    -- frees the loop and before the parent can read it (measured: 1-2
    -- ticks, and the value moved run to run).  What the callback sees at
    -- its first fire does not move: 1 if the loop ran with the menu up,
    -- 0 if it only ran after the escape.
    c:lua(
      'vim.g.mnpum = -1 _G.MNT = vim.fn.timer_start(150, '
        .. 'function() vim.g.mnpum = vim.fn.pumvisible() end)'
    )
    -- The command must go in as a KEY, for the same reason.
    c:key('<Cmd>try|' .. k[2] .. '|catch|let g:mnerr=v:exception|endtry<CR>')
    vim.wait(400)
    -- Now escape, unconditionally and more than once: a `<CR>` case may
    -- have executed an item that left another mode behind, and every
    -- query after this point is an RPC that the child must be free to
    -- answer.
    c:key(k[3])
    c:key('\27')
    vim.wait(20)
    c:flush()
    local pum =
      c:lua('local n = vim.g.mnpum pcall(vim.fn.timer_stop, _G.MNT) return n')
    local ans = c:lua('return ANS("return vim.fn.pumvisible()")')
    local out = {
      -- 0 is the finding: the timer's first fire saw NO popup, i.e. it
      -- did not get to run until `<Esc>` had already closed one --
      -- the whole event loop stopped while the menu was up.  1 would
      -- mean the loop kept turning, -1 that the timer never fired at
      -- all.
      pum_at_first_timer = pum,
      ans = ans,
    }
    emit(label, '=', esc(scrub(ins(out))))
    struct(label, out)
  end
  c:stop()
end)

-- =================================================================== s91
-- The inputs that may kill the editor.  One child each, because the
-- verdict is "did it die", and a dead child takes the rest of a section
-- with it.
-- ====================================================================

section('s91-crashprobe', function()
  local PROBES = {
    { 'deep-path', { 'amenu ' .. string.rep('A.', 60) .. 'Leaf rhs' } },
    { 'deeper-path', { 'amenu ' .. string.rep('B.', 400) .. 'Leaf rhs' } },
    { 'long-name', { 'amenu Top.' .. string.rep('n', 5000) .. ' rhs' } },
    { 'long-rhs', { 'amenu Top.Item ' .. string.rep('x', 20000) } },
    { 'long-accel', { 'amenu Top.Item<Tab>' .. string.rep('a', 5000) .. ' rhs' } },
    { 'many-tops', (function()
      local t = {}
      for i = 1, 500 do
        t[#t + 1] = 'amenu T' .. i .. '.Item rhs'
      end
      return t
    end)() },
    { 'trailing-bslash', { 'amenu Top.Item\\ rhs' } },
    { 'only-dots', { 'amenu ...... rhs' } },
    { 'ctrlv-nul', { 'amenu Top.A\u{0016}\u{0001}B rhs' } },
    { 'bad-utf8', { 'amenu Top.\xff\xfe rhs' } },
    { 'huge-priority', { 'amenu ' .. string.rep('9', 400) .. ' Top.Item rhs' } },
    { 'priority-many-dots', { 'amenu ' .. string.rep('1.', 200) .. '1 Top.Item rhs' } },
    { 'translate-long', { 'menutranslate ' .. string.rep('a', 4000) .. ' ' .. string.rep('b', 4000), 'amenu ' .. string.rep('a', 4000) .. '.Item rhs' } },
    { 'translate-self', { 'menutranslate File File', 'amenu File.Open rhs' } },
    { 'popup-copies', { 'amenu PopUp.' .. string.rep('p', 2000) .. ' rhs' } },
    { 'recursive-tr', (function()
      local t = {}
      for i = 1, 300 do
        t[#t + 1] = 'menutranslate n' .. i .. ' n' .. (i + 1)
      end
      t[#t + 1] = 'amenu n1.Item rhs'
      return t
    end)() },
    { 'emenu-empty-name', { 'amenu Top.Item rhs', 'emenu ' } },
    { 'info-huge', { 'amenu Top.Item rhs', 'call menu_info(repeat("x", 100000), "a")' } },
    { 'complete-deep', { 'amenu ' .. string.rep('A.', 60) .. 'Leaf rhs', 'call getcompletion(repeat("A.", 60), "menu")' } },
    { 'unmenu-star-empty', { 'aunmenu *' } },
  }

  local aborted = 0
  for _, p in ipairs(PROBES) do
    local c = child_new()
    c:lua('RESET({})')
    local ran = c:lua(
      'local out = {} for _, cmd in ipairs('
        .. tolua(p[2])
        .. ') do local ok, e = pcall(vim.api.nvim_exec2, cmd, {output = false}) '
        .. 'out[#out+1] = ok and "ok" or tostring(e) end return out'
    )
    local alive = c:lua('return 1')
    local tree = c:lua('return #vim.fn.menu_get("")')
    local verdict
    if type(tree) == 'number' then
      verdict = { live = true, tops = tree, ran = ran }
    else
      aborted = aborted + 1
      -- `jobwait` first: the dying words arrive on stderr after the
      -- channel has already gone, so reading `c.err` before reaping is
      -- a race that answers an empty list about half the time.
      c:stop()
      verdict = {
        live = false,
        input_survived = (alive == 1),
        why = tostring(tree),
        said = said(c.err),
      }
    end
    label_once('s91/' .. p[1])
    emit('s91/' .. p[1], 'X', esc(scrub(ins(verdict))))
    struct('s91/' .. p[1], verdict)
    c:stop()
  end
  emit('s91', 'groups', string.format('cases=%d aborted=%d', #PROBES, aborted))
end)

structfd:close()
emit('##', 'TOTAL', string.format('rows=%d', rows))
