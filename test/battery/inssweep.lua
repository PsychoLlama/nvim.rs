-- Differential oracle for the insert-completion family:
-- crates/nvim/src/insexpand/{mod,session,getexp,matchlist,sources,mode,
-- keys,insert,pum,text,vimscript,callbacks}.rs -- `ins_complete` and its
-- whole key alphabet, `ins_compl_next`/`_prev` and the WRAPAROUND,
-- `compl_leader` editing while the menu is up, every `'complete'` source
-- letter, every `'completeopt'` flag, the three completion callbacks
-- (`'completefunc'`/`'omnifunc'`/`'thesaurusfunc'`), `complete()` /
-- `complete_add()` / `complete_check()` / `complete_info()` /
-- `nvim__complete_set`, `v:completed_item` and the
-- CompleteChanged/CompleteDonePre/CompleteDone ORDER.
--
--   inssweep.sh <nvim-binary> <vimruntime> <outdir> <label>
--
-- THE HOLE IT FILLS (p22-1 survey, section 3, gap 1).  insexpand is the
-- second-largest `cell_ptr` family in the tree -- 215 sites over 24
-- files, 20 interacting globals -- and it had NO differential at all.
-- `cxprobe` is *cmdline* expansion; `1785449630-insweep.sh` is the TUI
-- INPUT layer; `1786233895-editprobe.lua`'s `pum` phase is one line of
-- one probe.  Everything else that watches completion is a point
-- assertion (`test_ins_complete`, `editor/completion_spec`,
-- `ui/popupmenu_spec`) over a state machine, and point assertions cannot
-- gate a rewrite of the state itself.
--
-- WHY EVERY CASE RUNS IN AN `--embed` CHILD.  Insert-mode completion is
-- a state that exists only BETWEEN two keystrokes.  `feedkeys(..., 'x')`
-- runs the whole sequence to completion and hands back an editor that
-- has already left the mode, so the pum, `compl_leader`,
-- `compl_shown_match` and `complete_info()` are never observable at all
-- -- which is exactly why editprobe's `pum` phase could only print the
-- buffer afterwards.  So the sweep drives a child over
-- `jobstart(rpc=true)`: `nvim_input` is a FAST call that drops the key
-- into the input buffer, and `nvim_exec_lua` is DEFERRED to the main
-- loop, which consumes the typeahead first and then answers.  The
-- child's insert loop dispatches `K_EVENT` while the menu is up, so the
-- observation lands mid-completion.
--
-- A UI IS ATTACHED over a raw socket (`sockconnect(rpc=false)`, the
-- b21-4 termsweep trick, with an hand-built msgpack `nvim_ui_attach`).
-- Nothing reads it back; it is attached so the child has a real screen,
-- draws a real pum and answers `pum_getpos()` with real geometry.
-- Without it `ui_active()` is false and half of `pum.rs` never runs.
--
-- Sections:
--   i0  canary     the fixture, the option defaults, the counts
--   i1  sources    every `'complete'` letter, alone, x CTRL-N/CTRL-P
--   i2  cot        every `'completeopt'` flag and the pairs that fight
--   i3  ctrlx      CTRL-X CTRL-{N,P,L,F,K,T,I,D,V,O,U,S,],Z,E,Y} and
--                  the "not defined yet" state
--   i4  leader     typing and backspacing while the menu is up
--   i5  wrap       `ins_compl_next`'s wraparound, both directions
--   i6  observe    complete_info() key subsets, v:completed_item and
--                  the CompleteChanged/DonePre/Done ORDER
--   i7  funcs      completefunc/omnifunc/thesaurusfunc, complete(),
--                  complete_add(), complete_check(), nvim__complete_set
--   i8  end        CTRL-E / CTRL-Y / Esc / CR / CTRL-C / BS past start,
--                  the undo block and the `.` repeat
--   i91 abortprobe the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A section that
-- goes silently empty otherwise looks exactly like a healthy one.
--
-- DETERMINISM.  `'autocomplete'` and `'autocompletetimeout'` put a
-- WALL-CLOCK budget on match collection (`compl_time_slice_expired`), so
-- the harness pins `autocomplete` off and `autocompletetimeout` to 0 and
-- the sweep never measures a timed collection.  That is a real hole and
-- it is named in the slice file, not hidden here.

local work = assert(os.getenv('INS_WORK'), 'INS_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local nvim = assert(os.getenv('INS_NVIM'), 'INS_NVIM unset')
local script = debug.getinfo(1, 'S').source:sub(2)
local dumppath = assert(os.getenv('INS_DUMP'), 'INS_DUMP unset')

local only = os.getenv('INSSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('INSSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')
local dumpfh = assert(io.open(dumppath, 'w'))

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end
local function dump(...)
  dumpfh:write(table.concat({ ... }, ' '), '\n')
end

-- ---------------------------------------------------------------- text

local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
  end
  text = text:gsub('0x%x+', '<ADDR>')
  return text
end

--- One printable line.  Completion text carries tabs (the thesaurus
--- separator), control bytes (the keys themselves) and multibyte words.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\"]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

local function cap(text, limit)
  limit = limit or 160
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

local function d12(text)
  return vim.fn.sha256(tostring(text)):sub(1, 12)
end

local function errtext(err)
  if type(err) == 'table' then
    return table.concat(vim.tbl_map(tostring, err), ' ')
  end
  return tostring(err)
end

--- Deterministic Lua literal: arrays in order, maps by sorted key.
local function tolua(value)
  local t = type(value)
  if t == 'string' then
    return string.format('%q', value)
  elseif t == 'number' or t == 'boolean' then
    return tostring(value)
  elseif t == 'nil' then
    return 'nil'
  elseif t ~= 'table' then
    return string.format('%q', tostring(value))
  end
  local parts = {}
  if vim.islist(value) then
    for _, v in ipairs(value) do
      parts[#parts + 1] = tolua(v)
    end
  else
    local keys = {}
    for k in pairs(value) do
      keys[#keys + 1] = k
    end
    table.sort(keys, function(a, b)
      return tostring(a) < tostring(b)
    end)
    for _, k in ipairs(keys) do
      parts[#parts + 1] = string.format('[%q]=%s', tostring(k), tolua(value[k]))
    end
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

--- Flatten an observation to `k=v k=v ...`, keys sorted, values escaped.
local function flat(t)
  local keys = {}
  for k in pairs(t) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    local v = t[k]
    if type(v) == 'table' then
      v = tolua(v)
    end
    parts[#parts + 1] = string.format('%s=%s', k, esc(cap(tostring(v))))
  end
  return table.concat(parts, ' ')
end

-- ------------------------------------------------------------ sections

local secrows = 0
local function section(name, fn)
  if only then
    local hit = false
    for pat in only:gmatch('[^|]+') do
      if name:match(pat) then
        hit = true
        break
      end
    end
    if not hit then
      return
    end
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

-- =============================================================== child

local Child = {}
Child.__index = Child

--- The hand-built msgpack for
--- `nvim_ui_attach(80, 24, {rgb = true, ext_messages = true})`.
---
--- `ext_messages` IS NOT DECORATION.  Without it the first error a case
--- provokes (i7's `completefunc` that throws, i91's whole point) puts
--- the child on a HIT-ENTER PROMPT, which answers no deferred request
--- at all, and the sweep hangs forever with no output -- exactly what
--- the first draft did.  With it every message leaves as a `msg_show`
--- event and `need_wait_return` is never set.
local UI_ATTACH = string.char(0x94, 0x00, 0x01)
  .. string.char(0xa0 + #'nvim_ui_attach')
  .. 'nvim_ui_attach'
  .. string.char(0x93, 80, 24, 0x82)
  .. string.char(0xa0 + #'rgb')
  .. 'rgb'
  .. string.char(0xc3)
  .. string.char(0xa0 + #'ext_messages')
  .. 'ext_messages'
  .. string.char(0xc3)

local childseq = 0

--- The child-side prelude.  Everything a case needs is a function here,
--- so a case costs ONE roundtrip per observation rather than one per
--- question -- and so the questions are asked in the same order every
--- time, which a table walk in the parent would not guarantee.
local PRELUDE = [==[
_G.INS_W = ...
_G.AULOG = {}

function _G.OPTS()
  vim.o.lines = 24
  vim.o.columns = 80
  vim.o.report = 9999
  vim.o.more = false
  vim.o.shortmess = 'aoOtTIcCF'
  vim.o.swapfile = false
  vim.o.undolevels = 1000
  vim.o.backspace = 'indent,eol,start'
  vim.o.autocomplete = false
  vim.o.autocompletedelay = 0
  vim.o.autocompletetimeout = 0
  vim.o.pumheight = 0
  vim.o.pumwidth = 15
  vim.o.infercase = false
  vim.o.ignorecase = false
  vim.o.smartcase = false
  vim.o.spell = false
  vim.o.wrap = false
  vim.o.wrapscan = true
  vim.o.textwidth = 0
  vim.o.autoindent = false
  vim.o.expandtab = false
  vim.o.paste = false
  vim.o.langremap = false
  vim.o.timeoutlen = 1000
  vim.o.ttimeoutlen = 50
  vim.o.iskeyword = '@,48-57,_,192-255'
  vim.o.complete = '.,w,b,u,t'
  vim.o.completeopt = 'menu,popup'
  vim.o.completefunc = ''
  vim.o.omnifunc = ''
  vim.o.thesaurusfunc = ''
  vim.o.dictionary = ''
  vim.o.thesaurus = ''
  vim.o.tags = INS_W .. '/tags'
  vim.o.path = INS_W .. '/inc'
  vim.o.include = ''
  vim.o.define = ''
  vim.o.wildignore = ''
  vim.o.suffixes = ''
  vim.o.spelllang = 'ins'
  vim.o.spellfile = ''
end

--- The standing world, built once per child: the current (scratch)
--- buffer plus one buffer in a WINDOW (`'complete'` `w`), one loaded and
--- hidden (`b`), one listed-but-unloaded (`u`) and one unlisted (`U`).
--- Each carries words no other buffer has, so which source answered is
--- readable off the words alone.
function _G.SETUP()
  vim.opt.runtimepath:append(INS_W)
  _G.OPTS()
  vim.cmd('silent! only')
  local main = vim.api.nvim_create_buf(true, true)
  vim.api.nvim_buf_set_name(main, 'inswork')
  vim.api.nvim_win_set_buf(0, main)
  _G.MAIN = main
  -- `w`: visible in another window.
  vim.cmd('silent! belowright split ' .. INS_W .. '/win.txt')
  _G.WINBUF = vim.api.nvim_get_current_buf()
  vim.cmd('silent! wincmd k')
  -- `b`: loaded, hidden.
  vim.cmd('silent! badd ' .. INS_W .. '/hid.txt')
  _G.HIDBUF = vim.fn.bufnr(INS_W .. '/hid.txt')
  vim.fn.bufload(_G.HIDBUF)
  -- `u`: listed, never loaded.
  vim.cmd('silent! badd ' .. INS_W .. '/unl.txt')
  _G.UNLBUF = vim.fn.bufnr(INS_W .. '/unl.txt')
  -- `U`: unlisted.
  vim.cmd('silent! badd ' .. INS_W .. '/uls.txt')
  _G.ULSBUF = vim.fn.bufnr(INS_W .. '/uls.txt')
  vim.bo[_G.ULSBUF].buflisted = false
  vim.api.nvim_win_set_buf(0, main)

  vim.api.nvim_create_autocmd({'CompleteChanged','CompleteDonePre','CompleteDone'}, {
    callback = function(ev)
      local e = vim.v.event or {}
      local rec = { ev.event }
      if ev.event == 'CompleteChanged' then
        rec[2] = 'w=' .. tostring((e.completed_item or {}).word)
        rec[3] = 'size=' .. tostring(e.size)
      else
        rec[2] = 'w=' .. tostring((vim.v.completed_item or {}).word)
        rec[3] = 'reason=' .. tostring(e.reason)
      end
      AULOG[#AULOG + 1] = table.concat(rec, ',')
    end,
  })
  return { main = main, win = _G.WINBUF, hid = _G.HIDBUF, unl = _G.UNLBUF, uls = _G.ULSBUF }
end

--- Back to a known editor.  Every case starts here.
--- NORMALISE BEFORE HASHING.  `$WORK` is a fresh `mktemp -d` on every
--- run and CTRL-X CTRL-F completes real paths, so a digest taken over
--- the raw text moves every run while the sed-scrubbed artifact around
--- it stays put (the optsweep trap, hit verbatim on the first
--- determinism run).  Every `sha256` below goes through this.
function _G.NORM(s)
  s = tostring(s)
  s = s:gsub(vim.pesc(INS_W), '<WORK>')
  local rt = vim.env.VIMRUNTIME or ''
  if rt ~= '' then
    s = s:gsub(vim.pesc(rt), '<RT>')
  end
  return s
end

function _G.SORTED(t)
  local k = {}
  for key in pairs(t) do k[#k + 1] = key end
  table.sort(k)
  return k
end

function _G.RESET(o)
  pcall(vim.cmd, 'silent! stopinsert')
  if vim.fn.mode() ~= 'n' then
    pcall(vim.api.nvim_feedkeys, '\27', 'nx', false)
  end
  vim.api.nvim_win_set_buf(0, _G.MAIN)
  _G.OPTS()
  AULOG = {}
  vim.g.insmark = nil
  vim.api.nvim_buf_set_lines(_G.MAIN, 0, -1, false, o.lines)
  vim.api.nvim_win_set_cursor(0, o.cur)
  for _, k in ipairs(_G.SORTED(o.opts or {})) do
    local ok, err = pcall(vim.api.nvim_set_option_value, k, o.opts[k], {})
    if not ok then
      vim.g.insmark = 'OPT:' .. k .. ':' .. tostring(err)
    end
  end
  if o.pre then
    local ok, err = pcall(loadstring(o.pre))
    if not ok then vim.g.insmark = 'PRE:' .. tostring(err) end
  end
  vim.cmd('silent! messages clear')
  _G.UNDO0 = vim.fn.undotree().seq_cur
  return vim.fn.mode(1)
end

--- One observation.  ORDER MATTERS ONLY HERE: the parent formats what
--- this returns, so the artifact's field order is this table's.
function _G.OBS()
  local ci = vim.fn.complete_info()
  local words, full = {}, {}
  for i, it in ipairs(ci.items or {}) do
    words[i] = it.word
    full[i] = table.concat({
      it.word or '', it.abbr or '', it.kind or '', it.menu or '',
      type(it.user_data) == 'string' and it.user_data or vim.json.encode(it.user_data or ''),
      tostring(it.hl_group or ''),
    }, '|')
  end
  local pos = vim.fn.pum_getpos()
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  return {
    mode = vim.fn.mode(1),
    pumv = vim.fn.pumvisible(),
    cimode = ci.mode,
    sel = ci.selected,
    n = #words,
    pre = ci.preinserted_text or '',
    line = vim.api.nvim_get_current_line(),
    cur = vim.api.nvim_win_get_cursor(0),
    nl = #lines,
    bd = vim.fn.sha256(_G.NORM(table.concat(lines, '\n'))):sub(1, 12),
    wd = vim.fn.sha256(_G.NORM(table.concat(words, '\n'))):sub(1, 12),
    pos = { pos.height or -1, pos.width or -1, pos.size or -1, pos.scrollbar and 1 or 0 },
    done = (vim.v.completed_item or {}).word or '',
    words = words,
    full = full,
    buf = lines,
  }
end

--- End of a case: leave the mode, and hand back what only survives the
--- leaving -- the final buffer, `v:completed_item`, the autocmd ORDER
--- and any E-code the case provoked.
function _G.FINI()
  pcall(vim.cmd, 'silent! stopinsert')
  if vim.fn.mode() ~= 'n' then
    pcall(vim.api.nvim_feedkeys, '\27', 'nx', false)
  end
  local msg = vim.api.nvim_exec2('messages', { output = true }).output or ''
  return {
    mode = vim.fn.mode(1),
    line = vim.api.nvim_get_current_line(),
    buf = vim.api.nvim_buf_get_lines(0, 0, -1, false),
    done = (vim.v.completed_item or {}).word or '',
    donefull = vim.v.completed_item or {},
    au = table.concat(AULOG, ' '),
    msg = msg,
    mark = tostring(vim.g.insmark),
    undo = vim.fn.undotree().seq_cur - (_G.UNDO0 or 0),
  }
end
]==]

local function child_start(extra)
  childseq = childseq + 1
  local argv = {
    nvim,
    '--headless',
    '--embed',
    '-u',
    'NONE',
    '-i',
    'NONE',
    '--listen',
    string.format('%s/c%d.sock', work, childseq),
    '--cmd',
    'set noswapfile',
  }
  for _, a in ipairs(extra or {}) do
    argv[#argv + 1] = a
  end
  local errlines = {}
  local chan = vim.fn.jobstart(argv, {
    rpc = true,
    cwd = work,
    clear_env = true,
    -- Without this a dying child's words go on the floor: `jobstart`
    -- discards stderr it is not asked for, and i91's whole verdict is
    -- "did it die and what did it say".
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
  return setmetatable({
    chan = chan,
    dead = false,
    err = errlines,
    sock = string.format('%s/c%d.sock', work, childseq),
  }, Child)
end

--- A Lua error inside the chunk comes back as an error RESPONSE and the
--- child is fine; a dead child comes back as a closed channel.  Only the
--- second means dead (the b19-3 lesson, kept verbatim).
function Child:lua(code, args)
  if self.dead then
    return 'DEAD'
  end
  local ok, res = pcall(vim.rpcrequest, self.chan, 'nvim_exec_lua', code, args or {})
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

--- Attach a UI over a RAW socket so the child has a real screen (the
--- b21-4 termsweep trick).  Nothing decodes what comes back; the point
--- is that `ui_active()` is true, `pum.rs` draws for real and
--- `pum_getpos()` answers with geometry rather than an empty dict.
function Child:attach()
  for _ = 1, 400 do
    local ok, c = pcall(vim.fn.sockconnect, 'pipe', self.sock, {
      rpc = false,
      on_data = function() end,
    })
    if ok and c > 0 then
      self.ui = c
      vim.fn.chansend(c, UI_ATTACH)
      for _ = 1, 400 do
        if self:lua('return #vim.api.nvim_list_uis()') == 1 then
          return true
        end
        vim.wait(2)
      end
      return false
    end
    vim.wait(5)
  end
  return false
end

function Child:stop()
  if self.ui then
    pcall(vim.fn.chanclose, self.ui)
  end
  pcall(vim.fn.jobstop, self.chan)
  pcall(vim.fn.jobwait, { self.chan }, 10000)
end

--- The dying child's words, reduced to the part that is a fact about the
--- editor rather than about this machine.
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

-- ============================================================ fixtures

local FIX = {
  'alpha alphabet alpine album',
  'bravo bravado bracket',
  'charlie chart charm charter',
  'delta deluge delegate',
  'ALPHA Alpha AlPhA',
  'al',
}

--- The `i`/`d` sources walk `'path'` from the CURRENT buffer's
--- `#include` lines, so those two cases need a buffer that has some.
local FIXINC = {
  '#include "one.h"',
  '#include "two.h"',
  '#define LOCALMACRO 1',
  'al',
}

--- The fixture with a different LEADER on its last line.  Half the
--- `'complete'` sources answer with words that share no prefix with the
--- default `al` -- tags start `tag`, the include tree's macros start
--- `INC`, the buffer names start with the sandbox path -- and a source
--- that answers nothing looks exactly like a source that is broken.
local function lead(text, base)
  local l = vim.deepcopy(base or FIX)
  l[#l] = text
  return l
end

local function lines_for(kind)
  if kind == 'plain' then
    return vim.deepcopy(FIX)
  end
  return vim.deepcopy(FIX)
end

-- ================================================================ core

local C -- the standing child

local function row(name, keys, o)
  if type(o) ~= 'table' then
    emit(name, 'keys="' .. esc(keys) .. '"', 'BAD', esc(tostring(o)))
    return
  end
  local core = {
    mode = o.mode,
    pumv = o.pumv,
    cimode = o.cimode,
    sel = o.sel,
    n = o.n,
    pre = o.pre,
    line = o.line,
    cur = table.concat(o.cur, ','),
    nl = o.nl,
    bd = o.bd,
    wd = o.wd,
    pos = table.concat(o.pos, ','),
    done = o.done,
  }
  emit(name, 'keys="' .. esc(keys) .. '"', flat(core))
  dump('== ' .. name .. ' keys="' .. esc(keys) .. '"')
  for i, f in ipairs(o.full) do
    dump(string.format('   it%02d %s', i, esc(scrub(f))))
  end
  for i, l in ipairs(o.buf) do
    dump(string.format('   bl%02d %s', i, esc(scrub(cap(l, 200)))))
  end
end

--- One case: reset, then key/observe for every step, then finish.
---
--- `opts.steps` is a list of `{label, keys}`.  Every step gets its own
--- row, because the whole point of the oracle is the state BETWEEN
--- keystrokes -- a case that only reported its end would be editprobe.
local function case(name, opts)
  local o = {
    lines = opts.lines or lines_for('plain'),
    cur = opts.cur or { 6, 2 },
    opts = opts.opts,
    pre = opts.pre,
  }
  local m = C:lua('return _G.RESET(...)', { o })
  if m ~= 'n' then
    emit(name .. '/reset', 'RESETFAIL', esc(tostring(m)))
  end
  for _, st in ipairs(opts.steps) do
    C:key(st[2])
    row(name .. '/' .. st[1], st[2], C:lua('return _G.OBS()'))
  end
  local f = C:lua('return _G.FINI()')
  if type(f) ~= 'table' then
    emit(name .. '/fini', 'BAD', esc(tostring(f)))
    return
  end
  emit(
    name .. '/fini',
    flat({
      mode = f.mode,
      line = f.line,
      nl = #f.buf,
      bd = d12(scrub(table.concat(f.buf, '\n'))),
      done = f.done,
      undo = f.undo,
      mark = f.mark,
      au = f.au,
      msg = (f.msg:gsub('%s+', ' ')),
    })
  )
  dump('== ' .. name .. '/fini')
  dump('   done ' .. esc(scrub(tolua(f.donefull))))
  for i, l in ipairs(f.buf) do
    dump(string.format('   fl%02d %s', i, esc(scrub(cap(l, 200)))))
  end
  if f.msg ~= '' then
    dump('   msg ' .. esc(scrub(cap(f.msg, 600))))
  end
end

--- The shorthand every source/flag section uses: append at the end of
--- the fixture's last line (`al`) and press one completion key N times.
local function press(name, keys, n, opts)
  local steps = { { 'A', 'A' } }
  for i = 1, n do
    steps[#steps + 1] = { 'k' .. i, keys }
  end
  opts = opts or {}
  opts.steps = steps
  case(name, opts)
end

--- The `pre` snippets a case can name.  They run in the CHILD, inside
--- `RESET`, after the options are applied and before the first key.
PRE = {
  INSFUNC = [[_G.InsFunc = function(findstart, base)
      if findstart == 1 then return vim.fn.col('.') - 1 end
      return {'fnone','fntwo','fnthree'}
    end]],
  INSOMNI = [[_G.InsOmni = function(findstart, base)
      if findstart == 1 then return vim.fn.col('.') - 1 end
      return {'omone','omtwo','omthree'}
    end]],
  INSREG = [[vim.fn.setreg('a', 'regalpha') vim.fn.setreg('b', 'regbravo')]],
}

-- ============================================================== running

C = child_start()
do
  local pre = C:lua(PRELUDE, { work })
  if type(pre) == 'string' and pre:match('^RPCERR') then
    io.stderr:write('PRELUDE failed: ' .. pre .. '\n')
  end
  emit('i0/attach', 'ui=' .. tostring(C:attach()))
  WORLD = C:lua('return _G.SETUP()')
end

-- ================================================================= i0

section('i0-canary', function()
  emit('i0/world', flat(type(WORLD) == 'table' and WORLD or { bad = tostring(WORLD) }))
  emit(
    'i0/fixture',
    flat({ lines = #FIX, d = d12(table.concat(FIX, '\n')) })
  )
  local opts = C:lua([[
    local t = {}
    for _, o in ipairs({'complete','completeopt','completefunc','omnifunc',
                        'thesaurusfunc','dictionary','thesaurus','infercase',
                        'ignorecase','pumheight','pumwidth','autocomplete',
                        'autocompletedelay','autocompletetimeout','tags',
                        'path','include','define','iskeyword','backspace'}) do
      t[o] = tostring(vim.api.nvim_get_option_value(o, {}))
    end
    t.uis = #vim.api.nvim_list_uis()
    t.version = tostring(vim.version().api_level)
    return t
  ]])
  emit('i0/opts', flat(type(opts) == 'table' and opts or { bad = tostring(opts) }))
  local files = {}
  for _, f in ipairs({
    'win.txt', 'hid.txt', 'unl.txt', 'uls.txt', 'words.dict', 'thes.txt',
    'tags', 'inc/one.h', 'inc/two.h', 'src.c', 'files/aardvark.txt',
  }) do
    local fh = io.open(work .. '/' .. f)
    files[f] = fh and d12(fh:read('*a')) or 'MISSING'
    if fh then
      fh:close()
    end
  end
  emit('i0/files', flat(files))
  -- The five `'complete'` letters that need a WORLD rather than a
  -- fixture: if any of these is empty the whole of i1 is measuring
  -- nothing, and it looks exactly like a healthy section.
  local probe = C:lua([[
    return { win = #vim.api.nvim_buf_get_lines(_G.WINBUF, 0, -1, false),
             hid = #vim.api.nvim_buf_get_lines(_G.HIDBUF, 0, -1, false),
             unlloaded = vim.api.nvim_buf_is_loaded(_G.UNLBUF) and 1 or 0,
             ulslisted = vim.bo[_G.ULSBUF].buflisted and 1 or 0,
             tags = #vim.fn.taglist('.'),
             bufs = #vim.fn.getbufinfo() }
  ]])
  emit('i0/probe', flat(type(probe) == 'table' and probe or { bad = tostring(probe) }))
end)

-- ================================================================= i1

--- Every `'complete'` source letter, ALONE, so the words alone say which
--- source answered.  Both directions, because `ins_compl_next` and
--- `ins_compl_prev` reach a different arm of the same walk.
section('i1-sources', function()
  local SRC = {
    { 'dot', '.' },
    { 'win', 'w' },
    { 'buf', 'b' },
    { 'unloaded', 'u' },
    { 'unlisted', 'U' },
    { 'dict', 'k' .. work .. '/words.dict' },
    { 'kspell', 'kspell' },
    { 'thes', 's' .. work .. '/thes.txt' },
    { 'incl', 'i' },
    -- `d` answers NOTHING here and that is the baseline, not a broken
    -- fixture: `get_normal_compl_info` builds `compl_pattern` as the
    -- regexp `\<INC` unless `ctrl_x_mode_path_defines()`, and
    -- `'complete'` never sets that mode -- so FIND_DEFINE is handed a
    -- regexp it cannot match.  CTRL-X CTRL-D two sections down finds
    -- the same three macros.  Inherited from upstream verbatim; see
    -- ~/agents/context/1786212071-upstream-neovim-bugs.
    { 'defs', 'd' },
    { 'bufnames', 'f' },
    { 'tagsq', ']' },
    { 'tags', 't' },
    { 'func', 'F' },
    { 'omni', 'o' },
    { 'all', '.,w,b,u,t' },
    { 'empty', '' },
    { 'bogus', 'Q' },
  }
  -- Which leader each source needs to answer at all.  Default `al`.
  local LEAD = {
    incl = { 'INC', FIXINC },
    defs = { 'INC', FIXINC },
    bufnames = { 'in', nil },
    tagsq = { 'tag', nil },
    tags = { 'tag', nil },
  }
  for _, s in ipairs(SRC) do
    for _, dir in ipairs({ { 'n', '<C-n>' }, { 'p', '<C-p>' } }) do
      local ld = LEAD[s[1]]
      local lines = ld and lead(ld[1], ld[2]) or nil
      press('i1/' .. s[1] .. '/' .. dir[1], dir[2], 3, {
        opts = {
          complete = s[2],
          completefunc = s[1] == 'func' and 'v:lua.InsFunc' or '',
          omnifunc = s[1] == 'omni' and 'v:lua.InsOmni' or '',
          include = '^\\s*#\\s*include',
          define = '^\\s*#\\s*define',
          spell = s[1] == 'kspell',
          completeopt = 'menu,menuone',
        },
        pre = (s[1] == 'func' and PRE.INSFUNC) or (s[1] == 'omni' and PRE.INSOMNI) or nil,
        lines = lines,
        cur = lines and { #lines, #ld[1] } or nil,
      })
    end
  end
end)

-- ================================================================= i2

--- `'completeopt'`, one flag at a time and then the pairs that fight:
--- `longest` against `noinsert`, `noselect` against `preinsert`,
--- `fuzzy` against `nosort`, `nearest` against everything.
section('i2-cot', function()
  local FLAGS = {
    'menu', 'menuone', 'longest', 'preview', 'popup', 'noinsert',
    'noselect', 'fuzzy', 'nosort', 'preinsert', 'nearest',
    'menu,menuone', 'menu,longest', 'menu,noinsert', 'menu,noselect',
    'menu,longest,noinsert', 'menu,noinsert,noselect',
    'menu,fuzzy', 'menu,fuzzy,nosort', 'menu,fuzzy,noselect',
    -- `menu,fuzzy,noinsert` is the ONLY combination that reaches
    -- `ins_compl_fuzzy_sort`'s reselect arm (`nosort` off, `noinsert`
    -- set, `noselect` clear).  It was missing from the first draft and
    -- a mutant that swapped that arm's two branches went UNCAUGHT.
    'menu,fuzzy,noinsert', 'menuone,fuzzy,noinsert,noselect',
    'menu,nearest,noinsert', 'menu,fuzzy,nearest',
    'menuone,noselect,popup', 'menu,preinsert', 'menu,preinsert,noselect',
    'menu,nearest', 'menuone,nearest,noselect', 'menu,longest,preview',
    '',
  }
  for i, f in ipairs(FLAGS) do
    press(string.format('i2/f%02d', i), '<C-n>', 3, {
      opts = { complete = '.', completeopt = f },
    })
  end
  -- `longest` with ONE match and with NO match: the two arms
  -- `ins_compl_longest_match` gets wrong in different ways.
  press('i2/longest-one', '<C-n>', 2, {
    lines = { 'zebra', 'ze' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu,longest' },
  })
  press('i2/longest-none', '<C-n>', 2, {
    lines = { 'zebra', 'qq' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu,longest' },
  })
  -- `menuone` is the difference between a menu and no menu at all when
  -- exactly one match exists.
  press('i2/menuone-one', '<C-n>', 2, {
    lines = { 'zebra', 'ze' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu,menuone' },
  })
  -- `'pumheight'` clamps the menu but must NOT clamp the match list.
  press('i2/pumheight', '<C-n>', 4, {
    opts = { complete = '.', completeopt = 'menu', pumheight = 2 },
  })
  -- `'infercase'` rewrites the inserted word's case; `'ignorecase'` is
  -- what makes it reachable.
  for _, ic in ipairs({ { 'noinfer', false, false }, { 'infer', true, true }, { 'igncase', false, true } }) do
    press('i2/case-' .. ic[1], '<C-n>', 3, {
      lines = { 'alphaBET', 'ALPHAbet', 'alp' },
      cur = { 3, 2 },
      opts = { complete = '.', infercase = ic[2], ignorecase = ic[3] },
    })
  end
end)

-- ================================================================= i3

--- The CTRL-X family.  Each key needs its own world -- a path prefix for
--- CTRL-F, a command for CTRL-V, an `#include` for CTRL-D/CTRL-I -- so
--- this is a table of cases rather than a loop over keys.
section('i3-ctrlx', function()
  local CX = {
    { 'n', '<C-x><C-n>', {} },
    { 'p', '<C-x><C-p>', {} },
    { 'line', '<C-x><C-l>', {} },
    {
      'file',
      '<C-x><C-f>',
      { lines = { work .. '/files/a' }, cur = { 1, #work + 9 } },
    },
    { 'dict', '<C-x><C-k>', { opts = { dictionary = work .. '/words.dict' } } },
    { 'thes', '<C-x><C-t>', { opts = { thesaurus = work .. '/thes.txt' } } },
    {
      'incl',
      '<C-x><C-i>',
      {
        lines = lead('INC', FIXINC),
        cur = { #FIXINC, 3 },
        opts = { include = '^\\s*#\\s*include', path = work .. '/inc' },
      },
    },
    {
      'defs',
      '<C-x><C-d>',
      {
        lines = { '#include "one.h"', '#include "two.h"', 'INC' },
        cur = { 3, 3 },
        opts = {
          define = '^\\s*#\\s*define',
          include = '^\\s*#\\s*include',
          path = work .. '/inc',
        },
      },
    },
    { 'cmdline', '<C-x><C-v>', { lines = { 'sil' }, cur = { 1, 2 } } },
    { 'omni', '<C-x><C-o>', { opts = { omnifunc = 'v:lua.InsOmni' }, pre = 'INSOMNI' } },
    { 'user', '<C-x><C-u>', { opts = { completefunc = 'v:lua.InsFunc' }, pre = 'INSFUNC' } },
    { 'spell', '<C-x><C-s>', { lines = { 'alpah' }, cur = { 1, 4 }, opts = { spell = true } } },
    { 'tag', '<C-x><C-]>', { lines = nil, cur = nil, tagged = true } },
    { 'reg', '<C-x><C-r>', { pre = 'INSREG', regged = true } },
    { 'evalz', '<C-x><C-z>', {} },
    { 'scrolle', '<C-x><C-e>', {} },
    { 'scrolly', '<C-x><C-y>', {} },
    -- A bare CTRL-X puts `ctrl_x_mode` in NOT_DEFINED_YET; what the next
    -- key does with that is a whole arm of `ins_compl_prep`.
    { 'bare-then-char', '<C-x>z', {} },
    { 'bare-then-esc', '<C-x><Esc>', {} },
    { 'bare-then-n', '<C-x><C-n>', {} },
    { 'xx', '<C-x><C-x>', {} },
  }
  for _, c in ipairs(CX) do
    local o = vim.deepcopy(c[3])
    if o.tagged then
      o.lines, o.cur, o.tagged = lead('tag'), { #FIX, 3 }, nil
    end
    if o.regged then
      o.lines, o.cur, o.regged = lead('reg'), { #FIX, 3 }, nil
    end
    o.opts = o.opts or {}
    o.opts.complete = o.opts.complete or '.,w,b,u,t'
    o.opts.completeopt = o.opts.completeopt or 'menu,menuone'
    o.pre = PRE[o.pre or ''] or nil
    o.steps = {
      { 'A', 'A' },
      { 'k1', c[2] },
      { 'k2', '<C-n>' },
      { 'k3', '<C-p>' },
      { 'k4', c[2] },
    }
    case('i3/' .. c[1], o)
  end
end)

-- ================================================================= i4

--- `compl_leader`: what happens to the match list when the user keeps
--- TYPING while the menu is up.  This is the single most intricate part
--- of the family (`ins_compl_addleader` / `ins_compl_restart` /
--- `ins_compl_new_leader`) and nothing else in the tree watches it.
section('i4-leader', function()
  local COTS = {
    'menu', 'menu,noselect', 'menu,noinsert', 'menu,longest', 'menu,fuzzy',
    'menu,fuzzy,noinsert', 'menu,preinsert',
  }
  for i, cot in ipairs(COTS) do
    case(string.format('i4/type%d', i), {
      opts = { complete = '.', completeopt = cot },
      steps = {
        { 'A', 'A' },
        { 'n', '<C-n>' },
        { 'l', 'l' },
        { 'p', 'p' },
        { 'h', 'h' },
        { 'a', 'a' },
        { 'n2', '<C-n>' },
        { 'bs1', '<BS>' },
        { 'bs2', '<BS>' },
        { 'bs3', '<BS>' },
        { 'n3', '<C-n>' },
      },
    })
  end
  -- Backspacing all the way past the start of the completion is the
  -- documented way OUT of the menu, and it is a different exit from
  -- CTRL-E.
  case('i4/bs-past-start', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'bs1', '<BS>' },
      { 'bs2', '<BS>' },
      { 'bs3', '<BS>' },
      { 'bs4', '<BS>' },
      { 'bs5', '<BS>' },
      { 'n2', '<C-n>' },
    },
  })
  -- CTRL-W and CTRL-U delete a word / the line WHILE the menu is up.
  case('i4/ctrl-w', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'w', '<C-w>' }, { 'n2', '<C-n>' } },
  })
  case('i4/ctrl-u', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'u', '<C-u>' }, { 'n2', '<C-n>' } },
  })
  -- Typing a NON-keyword character ends the completion; typing one that
  -- matches nothing leaves an empty list but stays in the mode.
  case('i4/nonkeyword', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'dot', '.' }, { 'n2', '<C-n>' } },
  })
  case('i4/no-match-leader', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'q', 'q' }, { 'z', 'z' }, { 'bs', '<BS>' } },
  })
  -- Multibyte leader: `compl_leader` is a byte string and the backspace
  -- has to take a whole character off it.
  case('i4/multibyte', {
    lines = { 'naïve naïveté naïvely', 'na' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'i', 'ï' },
      { 'n2', '<C-n>' },
      { 'bs', '<BS>' },
      { 'n3', '<C-n>' },
    },
  })
end)

-- ================================================================= i5

--- `ins_compl_next`'s WRAPAROUND.  Press past the end of the list and it
--- goes back to the ORIGINAL TEXT, then round again -- and the original
--- is a match entry too (`compl_orig_text`), which is why the counts
--- here are n+1 and not n.
section('i5-wrap', function()
  for _, cot in ipairs({ 'menu', 'menu,noselect', 'menu,noinsert', 'menu,menuone' }) do
    press('i5/fwd-' .. cot:gsub(',', '-'), '<C-n>', 7, {
      opts = { complete = '.', completeopt = cot },
    })
    press('i5/back-' .. cot:gsub(',', '-'), '<C-p>', 7, {
      opts = { complete = '.', completeopt = cot },
    })
  end
  -- Alternating: the direction flag flips mid-walk.
  case('i5/alternate', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n1', '<C-n>' },
      { 'n2', '<C-n>' },
      { 'p1', '<C-p>' },
      { 'p2', '<C-p>' },
      { 'p3', '<C-p>' },
      { 'n3', '<C-n>' },
      { 'n4', '<C-n>' },
    },
  })
  -- The DOWN/UP arrows are the same walk through a different key.
  case('i5/arrows', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'down', '<Down>' },
      { 'down2', '<Down>' },
      { 'up', '<Up>' },
      { 'pgd', '<PageDown>' },
      { 'pgu', '<PageUp>' },
    },
  })
  -- One match only: the wrap has nowhere to go.
  press('i5/single', '<C-n>', 4, {
    lines = { 'zebra', 'ze' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu' },
  })
  -- No match at all: `ins_compl_next` must not move and must not crash.
  press('i5/none', '<C-n>', 4, {
    lines = { 'zebra', 'qq' },
    cur = { 2, 1 },
    opts = { complete = '.', completeopt = 'menu' },
  })
end)

-- ================================================================= i6

--- The OBSERVATION surface itself: `complete_info()`'s key subsets,
--- `v:completed_item` at each stage, and the ORDER of the three
--- completion autocommands.  A rewrite that keeps every word right and
--- fires CompleteDone before CompleteDonePre is still a regression.
section('i6-observe', function()
  local KEYSETS = {
    '{}',
    "{'mode'}",
    "{'pum_visible'}",
    "{'selected'}",
    "{'items'}",
    "{'preinserted_text'}",
    "{'matches'}",
    "{'selected','mode'}",
    "{'nosuchkey'}",
  }
  case('i6/ci-setup', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'n2', '<C-n>' } },
  })
  -- The menu is gone by now (the case finished), so re-open it and ask
  -- every key subset in ONE roundtrip while it is up.
  C:lua('return _G.RESET(...)', { { lines = lines_for('plain'), cur = { 6, 2 }, opts = { complete = '.', completeopt = 'menu' } } })
  C:key('A')
  C:key('<C-n>')
  for i, ks in ipairs(KEYSETS) do
    local r = C:lua('return vim.inspect(vim.fn.complete_info(' .. ks .. '))')
    emit(string.format('i6/keys%02d', i), 'set=' .. esc(ks), 'd=' .. d12(scrub(tostring(r))))
    dump(string.format('== i6/keys%02d %s', i, esc(ks)))
    dump('   ' .. esc(scrub(cap(tostring(r), 1200))))
  end
  C:key('<Esc>')
  C:lua('return _G.FINI()')

  -- `v:completed_item` at each of the four exits.
  for _, ex in ipairs({
    { 'accept', '<C-y>' },
    { 'cancel', '<C-e>' },
    { 'esc', '<Esc>' },
    { 'cr', '<CR>' },
    { 'ctrlc', '<C-c>' },
    { 'space', ' ' },
  }) do
    case('i6/exit-' .. ex[1], {
      opts = { complete = '.', completeopt = 'menu' },
      steps = {
        { 'A', 'A' },
        { 'n', '<C-n>' },
        { 'n2', '<C-n>' },
        { 'exit', ex[2] },
      },
    })
  end
  -- `enter_selects`: with `noselect` a CR does NOT accept, with
  -- `menuone,noinsert` it does.
  for _, cot in ipairs({ 'menu', 'menu,noselect', 'menuone,noinsert', 'menuone,noinsert,noselect' }) do
    case('i6/enter-' .. cot:gsub(',', '-'), {
      opts = { complete = '.', completeopt = cot },
      steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'cr', '<CR>' } },
    })
  end
end)

-- ================================================================= i7

--- The three callbacks and the four script-facing entry points.  The
--- `findstart` protocol is the interesting half: -1, -2 and -3 each mean
--- something different and only one of them leaves the mode.
section('i7-funcs', function()
  local FINDSTART = { '0', '2', '-1', '-2', '-3', '9999', '"bogus"' }
  for i, fs in ipairs(FINDSTART) do
    case(string.format('i7/findstart%d', i), {
      opts = { completefunc = 'v:lua.InsFunc', complete = '.' },
      pre = string.format(
        [[_G.InsFunc = function(findstart, base)
            if findstart == 1 then return %s end
            return {'fone','ftwo','fthree'}
          end]],
        fs
      ),
      steps = { { 'A', 'A' }, { 'u', '<C-x><C-u>' }, { 'n', '<C-n>' } },
    })
  end
  -- What the function may RETURN: a list of strings, a list of dicts, a
  -- dict with `words`, a dict with `refresh='always'`, an empty list,
  -- v:null, and a function that throws.
  local RETS = {
    { 'strings', "{'aone','atwo'}" },
    { 'dicts', "{{word='dw1',abbr='AB',menu='ME',kind='K',info='IN',user_data='UD'},{word='dw2'}}" },
    { 'dupes', "{'same','same','same'}" },
    { 'dupdict', "{{word='same',dup=1},{word='same',dup=1}}" },
    { 'empty', '{}' },
    { 'wordsdict', "{words={'wd1','wd2'}}" },
    { 'refresh', "{words={'rf1','rf2'},refresh='always'}" },
    { 'null', 'vim.NIL' },
    { 'icase', "{{word='ICASE',icase=1},{word='xyz'}}" },
    { 'equal', "{{word='eq1',equal=1},{word='eq2'}}" },
    { 'empties', "{'','ok',''}" },
    { 'hlgroup', "{{word='hl1',hl_group='ErrorMsg',abbr_hlgroup='Title'}}" },
  }
  for _, r in ipairs(RETS) do
    case('i7/ret-' .. r[1], {
      opts = { completefunc = 'v:lua.InsFunc', complete = '.' },
      pre = string.format(
        [[_G.InsFunc = function(findstart, base)
            if findstart == 1 then return vim.fn.col('.') - 1 end
            return %s
          end]],
        r[2]
      ),
      steps = { { 'A', 'A' }, { 'u', '<C-x><C-u>' }, { 'n', '<C-n>' }, { 'n2', '<C-n>' } },
    })
  end
  case('i7/raises', {
    opts = { completefunc = 'v:lua.InsFunc', complete = '.' },
    pre = [[_G.InsFunc = function(findstart, base)
             if findstart == 1 then return vim.fn.col('.') - 1 end
             error('deliberate')
           end]],
    steps = { { 'A', 'A' }, { 'u', '<C-x><C-u>' }, { 'n', '<C-n>' } },
  })
  -- `complete_add()` + `complete_check()`: the incremental protocol.
  case('i7/complete-add', {
    opts = { completefunc = 'v:lua.InsFunc', complete = '.' },
    pre = [[_G.InsFunc = function(findstart, base)
             if findstart == 1 then return vim.fn.col('.') - 1 end
             for i = 1, 5 do
               vim.fn.complete_add('add' .. i)
               if vim.fn.complete_check() ~= 0 then break end
             end
             return {}
           end]],
    steps = { { 'A', 'A' }, { 'u', '<C-x><C-u>' }, { 'n', '<C-n>' } },
  })
  -- `complete()` from an insert-mode expression: the OTHER way into the
  -- same state machine, and the one `test_ins_complete` leans on.
  for _, c in ipairs({
    { 'basic', "vim.fn.complete(vim.fn.col('.'), {'cone','ctwo','cthree'})" },
    { 'dicts', "vim.fn.complete(vim.fn.col('.'), {{word='cd1',menu='M'},{word='cd2'}})" },
    { 'badcol', "vim.fn.complete(0, {'x'})" },
    { 'empty', "vim.fn.complete(vim.fn.col('.'), {})" },
  }) do
    case('i7/complete-' .. c[1], {
      opts = { complete = '.', completeopt = 'menu' },
      pre = '_G.InsCall = function() ' .. c[2] .. ' end',
      steps = {
        { 'A', 'A' },
        { 'call', '<C-r>=luaeval("(function() local ok,e = pcall(_G.InsCall) return ok and \'\' or \'E\' end)()")<CR>' },
        { 'n', '<C-n>' },
        { 'n2', '<C-n>' },
      },
    })
  end
  -- `'thesaurusfunc'` -- the third callback, and the one with no
  -- coverage anywhere in `test/`.
  case('i7/thesaurusfunc', {
    opts = { thesaurusfunc = 'v:lua.InsThes', complete = '.' },
    pre = [[_G.InsThes = function(findstart, base)
             if findstart == 1 then return vim.fn.col('.') - 1 end
             return {'tone','ttwo'}
           end]],
    steps = { { 'A', 'A' }, { 't', '<C-x><C-t>' }, { 'n', '<C-n>' } },
  })
  -- `nvim__complete_set`: sets info on the SELECTED item mid-menu.
  case('i7/complete-set', {
    opts = { complete = '.', completeopt = 'menu,popup' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      {
        'set',
        '<C-r>=luaeval("(function() local ok,e = pcall(vim.api.nvim__complete_set, 0, {info=\'INFO\'}) return ok and \'\' or \'E\' end)()")<CR>',
      },
      { 'n2', '<C-n>' },
    },
  })
end)

-- ================================================================= i8

--- Leaving the completion, and what the buffer and the UNDO tree look
--- like afterwards.  `ins_compl_prep`'s teardown is where `compl_*` is
--- freed, so this is the section a leak or a double free shows up in.
section('i8-end', function()
  for _, e in ipairs({
    { 'ctrl-y', '<C-y>' },
    { 'ctrl-e', '<C-e>' },
    { 'esc', '<Esc>' },
    { 'ctrl-c', '<C-c>' },
    { 'cr', '<CR>' },
    { 'ctrl-o', '<C-o>x' },
    { 'ctrl-l', '<C-l>' },
    { 'left', '<Left>' },
    { 'tab', '<Tab>' },
  }) do
    case('i8/exit-' .. e[1], {
      opts = { complete = '.', completeopt = 'menu' },
      steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'n2', '<C-n>' }, { 'exit', e[1] and e[2] } },
    })
  end
  -- The undo block: a completion is ONE change, and `.` repeats it.
  case('i8/undo', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'esc', '<Esc>' },
      { 'undo', 'u' },
      { 'redo', '<C-r>' },
    },
  })
  case('i8/dot-repeat', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'esc', '<Esc>' },
      { 'o', 'o' },
      { 'esc2', '<Esc>' },
      { 'dot', '.' },
    },
  })
  -- CTRL-G u inside a completion splits the undo block.
  case('i8/ctrl-g-u', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n', '<C-n>' },
      { 'gu', '<C-g>u' },
      { 'x', 'x' },
      { 'esc', '<Esc>' },
      { 'undo', 'u' },
    },
  })
  -- Two completions in a row without leaving insert: the second must not
  -- see the first's leader.
  case('i8/twice', {
    opts = { complete = '.', completeopt = 'menu' },
    steps = {
      { 'A', 'A' },
      { 'n1', '<C-n>' },
      { 'y', '<C-y>' },
      { 'sp', ' ' },
      { 'b', 'b' },
      { 'n2', '<C-n>' },
      { 'y2', '<C-y>' },
    },
  })
  -- A completion at the very start of an empty line, and at end of file.
  case('i8/empty-line', {
    lines = { 'alpha', 'beta', '' },
    cur = { 3, 0 },
    opts = { complete = '.', completeopt = 'menu' },
    steps = { { 'A', 'A' }, { 'n', '<C-n>' }, { 'n2', '<C-n>' }, { 'esc', '<Esc>' } },
  })
end)

-- =============================================================== i91

--- One FRESH child per case.  A case here may kill the editor, and a
--- dead child takes the rest of a section with it.  `aborted=0` is the
--- baseline: there is no expected abort in this sweep, so ANY abort is a
--- regression.
section('i91-abortprobe', function()
  local CASES = {
    { 'recursive-complete', [[
        vim.o.completefunc = 'v:lua.F'
        _G.F = function(fs) if fs == 1 then return vim.fn.col('.') - 1 end
          pcall(vim.fn.complete, 1, {'x'}) return {'r1','r2'} end
      ]], { 'A', '<C-x><C-u>', '<C-n>', '<Esc>' } },
    { 'wipe-in-func', [[
        vim.o.completefunc = 'v:lua.F'
        _G.F = function(fs) if fs == 1 then return vim.fn.col('.') - 1 end
          pcall(vim.cmd, 'silent! new') pcall(vim.cmd, 'silent! bwipeout!')
          return {'w1','w2'} end
      ]], { 'A', '<C-x><C-u>', '<C-n>', '<Esc>' } },
    { 'setline-in-changed', [[
        vim.api.nvim_create_autocmd('CompleteChanged', {
          callback = function() pcall(vim.fn.setline, 1, 'clobbered') end })
      ]], { 'A', '<C-n>', '<C-n>', '<Esc>' } },
    { 'stopinsert-in-done', [[
        vim.api.nvim_create_autocmd('CompleteDone', {
          callback = function() pcall(vim.cmd, 'stopinsert') end })
      ]], { 'A', '<C-n>', '<C-y>', '<Esc>' } },
    { 'complete-in-donepre', [[
        vim.api.nvim_create_autocmd('CompleteDonePre', {
          callback = function() pcall(vim.fn.complete, 1, {'z'}) end })
      ]], { 'A', '<C-n>', '<C-y>', '<Esc>' } },
    { 'huge-leader', '', { 'A', '<C-n>', string.rep('a', 200), '<C-n>', '<Esc>' } },
    { 'nul-in-func', [[
        vim.o.completefunc = 'v:lua.F'
        _G.F = function(fs) if fs == 1 then return vim.fn.col('.') - 1 end
          return {'a\0b', 'c\nd', 'e\tf'} end
      ]], { 'A', '<C-x><C-u>', '<C-n>', '<Esc>' } },
    { 'negative-col', [[
        vim.o.completefunc = 'v:lua.F'
        _G.F = function(fs) if fs == 1 then return -99 end return {'n1'} end
      ]], { 'A', '<C-x><C-u>', '<C-n>', '<Esc>' } },
    { 'ctrlx-storm', '', { 'A', '<C-x><C-x><C-x><C-n><C-p><C-x><C-l><C-e>', '<Esc>' } },
    { 'bwipe-mid-menu', '', { 'A', '<C-n>', '<C-r>=luaeval("(pcall(vim.cmd, [[silent! bwipeout!]]) and \'\') or \'\'")<CR>', '<Esc>' } },
    { 'undo-mid-menu', '', { 'A', '<C-n>', '<C-o>u', '<C-n>', '<Esc>' } },
    { 'cot-change-mid-menu', '', { 'A', '<C-n>', '<C-r>=luaeval("(function() vim.o.completeopt=[[menuone,noselect]] return \'\' end)()")<CR>', '<C-n>', '<Esc>' } },
  }
  local aborted = 0
  for _, c in ipairs(CASES) do
    local ch = child_start()
    ch:lua(PRELUDE, { work })
    ch:attach()
    ch:lua('return _G.SETUP()')
    ch:lua('return _G.RESET(...)', { { lines = lines_for('plain'), cur = { 6, 2 }, opts = { complete = '.', completeopt = 'menu' } } })
    if c[2] ~= '' then
      ch:lua(c[2])
    end
    for _, k in ipairs(c[3]) do
      ch:key(k)
    end
    local alive = ch:lua('return {vim.fn.mode(1), vim.api.nvim_get_current_line()}')
    local dead = (ch.dead or alive == 'DEAD') and 1 or 0
    aborted = aborted + dead
    emit(
      'i91/' .. c[1],
      flat({ dead = dead, state = type(alive) == 'table' and table.concat(alive, '|') or tostring(alive) })
    )
    local words = said(ch.err)
    if #words > 0 then
      dump('== i91/' .. c[1] .. ' stderr')
      for _, l in ipairs(words) do
        dump('   ' .. esc(cap(l, 300)))
      end
    end
    ch:stop()
  end
  emit('i91/groups', string.format('cases=%d aborted=%d', #CASES, aborted))
end)

-- ================================================================ tail

emit('##', 'TOTAL', string.format('rows=%d', rows))
dumpfh:close()
if C then
  C:stop()
end
